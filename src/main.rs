use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::Deserialize;
use serde_with::formats::Flexible;
use serde_with::TimestampSeconds;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("API failed: `{0}'")]
    ApiFailed(String),
    #[error(transparent)]
    ReqwestFailed(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeJsonFailed(#[from] serde_json::Error),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Parser, Debug)]
#[clap(version)]
struct Arg {
    /// Secret Token
    #[clap(short, long, value_parser)]
    token: String,

    /// Output Directory
    #[clap(short, long, value_parser)]
    output: PathBuf,
}

#[derive(Debug, Deserialize)]
struct Channel {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ChannelListResult {
    ok: bool,
    error: Option<String>,
    channels: Vec<Channel>,
}

#[derive(Debug, Deserialize)]
struct ChannelHistoryResult {
    ok: bool,
    error: Option<String>,
    messages: Vec<Message>,
    has_more: bool,
    response_metadata: Option<ResponseMetadata>,
}

#[serde_with::serde_as]
#[derive(Debug, Deserialize)]
struct Message {
    client_msg_id: Option<String>,
    text: String,
    #[serde_as(as = "TimestampSeconds<String, Flexible>")]
    ts: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct ResponseMetadata {
    next_cursor: String,
}

fn get_channels(args: &Arg) -> Result<Vec<Channel>, Error> {
    let client = reqwest::blocking::Client::new();
    let mut params = HashMap::new();
    params.insert("token", args.token.clone());
    params.insert("types", "private_channel,public_channel".into());

    let response = client
        .post("https://slack.com/api/conversations.list")
        .form(&params)
        .send()?
        .json::<ChannelListResult>()?;

    if response.ok {
        Ok(response.channels)
    } else {
        Err(Error::ApiFailed(
            response.error.unwrap_or_else(|| "no reason".into()),
        ))
    }
}

fn get_channel_history_as_text_per_page(
    args: &Arg,
    channel: &Channel,
    cursor: Option<&String>,
) -> Result<String, Error> {
    let client = reqwest::blocking::Client::new();
    let mut params = HashMap::new();
    params.insert("token", &args.token);
    params.insert("channel", &channel.id);

    if let Some(cursor) = cursor {
        params.insert("cursor", cursor);
    }

    let response = client
        .post("https://slack.com/api/conversations.history")
        .form(&params)
        .send()?
        .text_with_charset("utf-8")?;

    Ok(response)
}

fn get_channel_history(args: &Arg, channel: &Channel) -> Result<Vec<ChannelHistoryResult>, Error> {
    let mut channel_histories = Vec::new();

    let mut file_path = PathBuf::from(&args.output);

    file_path.push(&channel.name);
    fs::create_dir_all(&file_path);

    file_path.push("history.txt");

    let mut output_file = File::create(&file_path)?;

    let mut cursor = None;

    loop {
        let json_text = get_channel_history_as_text_per_page(args, channel, cursor.as_ref())?;

        output_file.write_all(&json_text.as_bytes())?;

        let channel_history: ChannelHistoryResult = serde_json::from_str(&json_text)?;

        if let Some(response_metadata) = &channel_history.response_metadata {
            cursor = Some(response_metadata.next_cursor.clone());
            println!("next cursor: {:?}", cursor);
        }

        let has_more = channel_history.has_more;

        channel_histories.push(channel_history);

        if !has_more {
            break;
        }
    }

    Ok(channel_histories)
}

fn main() -> Result<()> {
    let args = Arg::parse();

    let channels = get_channels(&args)?;

    // println!("{:#?}", channels);

    for ch in &channels {
        println!("extracting {}", ch.name);
        get_channel_history(&args, ch)?;
    }

    // let mut next_cursor: Option<String> = None;

    // loop {

    //     if let Some(nc) = next_cursor {
    //         params.insert("cursor", nc.clone());
    //     }

    //     let response_json = response.json::<ChannelHistoryResult>()?;

    //     println!("{:#?}", response_json);

    //     if !response_json.has_more {
    //         break;
    //     }

    //     match response_json.response_metadata {
    //         Some(data) => {
    //             next_cursor = Some(data.next_cursor);
    //         }
    //         None => {
    //             break;
    //         }
    //     }
    // }

    Ok(())
}
