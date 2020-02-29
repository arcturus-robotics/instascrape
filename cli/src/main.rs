//! An Instagram scraper created to help keep track of our Instagram followers.

#![deny(
    rust_2018_idioms,
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]

use anyhow::Result;
use chrono::Utc;
use instascrape::Scraper;
use log::{error, info};
use reqwest::Client;
use std::{collections::HashMap, path::PathBuf, time::Duration};
use structopt::StructOpt;
use tokio::{fs::OpenOptions, prelude::*, time};

#[derive(Debug, StructOpt)]
struct Opt {
    /// The Instagram user to scrape data from.
    #[structopt(short = "u", long = "user")]
    user: String,

    /// The interval at which to scrape in seconds.
    #[structopt(short = "i", long = "interval", parse(try_from_str = parse_duration))]
    interval: Duration,

    /// The path of the file to output data to.
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,

    /// The Discord webhook to send messages to.
    #[structopt(
        short = "w",
        long = "webhook",
        env = "INSTASCRAPE_WEBHOOK",
        hide_env_values = true
    )]
    webhook: Option<String>,
}

fn parse_duration(src: &str) -> Result<Duration> {
    Ok(Duration::from_secs(src.parse::<u64>()?))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger.
    env_logger::init();

    // Initialize the scraper.
    info!("initializing the scraper...");
    let opt = Opt::from_args();
    let scraper = Scraper::new(&opt.user);
    let client = Client::builder()
        .user_agent("DiscordBot (https://github.com/arcturus-robotics/instascrape, 0.1.1)")
        .build()?;

    // Open the file in append mode. We don't want to overwrite the data that's already there!
    info!("opening output file...");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(opt.output)
        .await?;

    // Run the scraper.
    info!("running the scraper...");
    loop {
        // Scrape the data.
        match scraper.scrape().await {
            // If we're successful, write the data with a timestamp to the file.
            Ok(data) => {
                // Serialize the data to be written to the file and log it.
                let ser = format!("{},{}", Utc::now(), data.followers);
                info!("{}", ser);

                // If we have a Discord webhook URL, post the data to it.
                if let Some(webhook) = &opt.webhook {
                    let mut post = HashMap::new();
                    post.insert("content", &ser);

                    client.post(webhook).json(&post).send().await?;
                }

                // Write to the file.
                let _ = file.write(format!("{}\n", &ser).as_bytes()).await?;
                file.flush().await?;
            }
            // If not, log the error and don't do anything.
            Err(err) => error!("{}", err),
        };

        time::delay_for(opt.interval).await;
    }
}
