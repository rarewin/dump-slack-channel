use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::Deserialize;
use serde_with::formats::Flexible;
use serde_with::TimestampSeconds;

#[derive(Parser, Debug)]
#[clap(version)]
struct Arg {
    #[clap(short, long, value_parser)]
    token: String,
}

#[derive(Debug, Deserialize)]
struct Channel {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ChannelListResult {
    ok: bool,
    channels: Vec<Channel>,
}

#[derive(Debug, Deserialize)]
struct ChannelHistoryResult {
    ok: bool,
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

fn main() -> Result<()> {
    let args = Arg::parse();

    // let client = reqwest::blocking::Client::new();
    // let mut params = HashMap::new();
    // params.insert("token", args.token);
    // params.insert("types", "private_channel,public_channel".into());

    // let response = client
    //     .post("https://slack.com/api/conversations.list")
    //     .form(&params)
    //     .send()?
    //     .json::<ChannelListResult>();

    let mut next_cursor: Option<String> = None;

    loop {
        let client = reqwest::blocking::Client::new();
        let mut params = HashMap::new();
        params.insert("token", args.token.clone());
        params.insert("channel", "".into());

        if let Some(nc) = next_cursor {
            params.insert("cursor", nc.clone());
        }

        let response = client
            .post("https://slack.com/api/conversations.history")
            .form(&params)
            .send()?;

        let response_json = response.json::<ChannelHistoryResult>()?;

        println!("{:#?}", response_json);

        if !response_json.has_more {
            break;
        }

        match response_json.response_metadata {
            Some(data) => {
                next_cursor = Some(data.next_cursor);
            }
            None => {
                break;
            }
        }
    }

    Ok(())
}
