use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[clap(version)]
struct Arg {
    #[clap(short, long, value_parser)]
    token: String,
}

fn main() -> Result<()> {
    let args = Arg::parse();

    let client = reqwest::blocking::Client::new();
    let mut params = HashMap::new();
    params.insert("token", args.token);

    let response = client
        .post("https://slack.com/api/conversations.list")
        .form(&params)
        .send()?;

    println!("{}", response.text()?);

    Ok(())
}
