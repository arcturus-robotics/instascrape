//! An Instagram scraper created to help keep track of our Instagram followers.

#![deny(
    rust_2018_idioms,
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]

use self::error::{Error, Result};
use chrono::Utc;
use instascrape::Scraper;
use reqwest::Client;
use std::{collections::HashMap, path::PathBuf, time::Duration};
use structopt::StructOpt;
use tokio::{fs::OpenOptions, prelude::*, time};

mod error;
#[cfg(test)]
mod tests;

const USER_AGENT: &str = concat!(
    "DiscordBot (",
    env!("CARGO_PKG_HOMEPAGE"),
    ", ",
    env!("CARGO_PKG_VERSION"),
    ")"
);

#[derive(Debug, StructOpt)]
struct Opt {
    /// The Instagram user to scrape data from.
    #[structopt(short = "u", long = "user")]
    user: String,

    /// The interval in seconds at which to scrape.
    #[structopt(short = "i", long = "interval", parse(try_from_str = parse_interval))]
    interval: Duration,

    /// The path of the file to output data to.
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,

    /// A Discord webhook to send messages to.
    #[structopt(
        short = "w",
        long = "webhook",
        env = "INSTASCRAPE_WEBHOOK",
        hide_env_values = true
    )]
    webhook: Option<String>,
}

/// Parse the interval duration.
fn parse_interval(src: &str) -> Result<Duration> {
    Ok(Duration::from_secs(if let Ok(secs) = src.parse::<u64>() {
        secs
    } else {
        return Err(Error::ParseInterval);
    }))
}

/// Send a message through a Discord webhook.
async fn send_message_through_webhook(client: &Client, url: &str, message: &str) -> Result<()> {
    let mut post = HashMap::new();
    post.insert("content", message);

    if client.post(url).json(&post).send().await.is_err() {
        Err(Error::SendMessageThroughWebhook)
    } else {
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the scraper.
    println!("initializing the scraper...");
    let opt = Opt::from_args();
    let scraper = Scraper::new(&opt.user)?;

    // Create the HTTP client that will be used for the webhook.
    // FIXME: don't create this if a webhook is not specified
    let client = if let Ok(client) = Client::builder().user_agent(USER_AGENT).build() {
        client
    } else {
        return Err(Error::CreateClient);
    };

    // Open the file in append mode. We don't want to overwrite the data that's already there!
    println!("opening output file...");
    let mut file = if let Ok(file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(opt.output)
        .await
    {
        file
    } else {
        return Err(Error::OpenOutput);
    };

    // Run the scraper.
    println!("running the scraper...");
    loop {
        // Scrape the data.
        match scraper.scrape().await {
            // If we're successful, write the data with a timestamp to the file.
            Ok(data) => {
                // Serialize the data to be written to the file and log it.
                let ser = format!("{},{}", Utc::now(), data.followers);
                println!("{}", ser);

                // If we have a Discord webhook URL, post the data to it.
                if let Some(webhook) = &opt.webhook {
                    send_message_through_webhook(&client, webhook, &format!("```rust\n{}```", ser))
                        .await?;
                }

                // Write to the file.
                if file.write(format!("{}\n", &ser).as_bytes()).await.is_err() {
                    return Err(Error::WriteOutput);
                }

                // Flush the file to make sure the written data
                // was saved.
                if file.flush().await.is_err() {
                    return Err(Error::FlushOutput);
                }
            }
            // If not, log the error, send it to the webhook if it exists, and don't do anything.
            Err(err) => {
                eprintln!("{}", err);
                if let Some(webhook) = &opt.webhook {
                    send_message_through_webhook(
                        &client,
                        webhook,
                        &format!("```error: {}```", err),
                    )
                    .await?;
                }
            }
        };

        // Wait for the specified interval duration to scrape again.
        time::delay_for(opt.interval).await;
    }
}
