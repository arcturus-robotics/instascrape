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
use std::{path::PathBuf, time::Duration};
use structopt::StructOpt;
use tokio::{fs::OpenOptions, prelude::*, time};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short = "u", long = "user")]
    user: String,

    #[structopt(short = "i", long = "interval", parse(try_from_str = parse_duration))]
    interval: Duration,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    path: PathBuf,
}

fn parse_duration(src: &str) -> Result<Duration> {
    Ok(Duration::from_secs(src.parse::<u64>()?))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger.
    env_logger::init();

    // Get options.
    info!("initializing the scraper...");
    let opt = Opt::from_args();

    // Run the scraper.
    info!("running the scraper...");

    let scraper = Scraper::new(&opt.user);

    // Open the file in append mode. We don't want to overwrite the data that's already there!
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(opt.path)
        .await?;

    loop {
        // Scrape the data.
        match scraper.scrape().await {
            // If we're successful, write the data with a timestamp to the file.
            Ok(data) => {
                // Serialize the data to be written to the file and log it.
                let ser = format!("{},{}", Utc::now(), data.followers);
                info!("{}", ser);

                // Write to the file.
                let _ = file.write(format!("{}\n", ser).as_bytes()).await?;
                file.flush().await?;
            }
            // If not, log the error and don't do anything.
            Err(err) => error!("{}", err),
        };

        time::delay_for(opt.interval).await;
    }
}
