//! An Instagram scraper created to help keep track of our Instagram followers.

#![deny(
    rust_2018_idioms,
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]

use self::error::{DocumentError, OutputError, ParseError};
use chrono::Utc;
use log::{error, info};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{path::Path, time::Duration};
use tokio::{fs::OpenOptions, prelude::*, time};

pub mod config;
pub mod error;

pub use self::{
    config::Config,
    error::{Error, Result},
};

/// Scraped Instagram data.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Data {
    pub followers: u64,
    pub following: u64,
    pub posts: u64,
}

/// An Instagram scraper.
#[derive(Debug, Clone)]
pub struct Scraper {
    client: Client,

    url: String,
    interval: Duration,
}

impl Scraper {
    /// Get the URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the loop interval.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Scrape data from the URL.
    pub async fn scrape(&self) -> Result<Data> {
        // Scrape the document.
        let document = self.document().await?;

        // Create a selector to find the description `meta` tag.
        let selector = Selector::parse(r#"meta[property="og:description"]"#)
            .map_err(|_| Error::Parse(ParseError::SelectorParsingFailed))?;

        // Find the description `meta` tag with the selector.
        let meta = match document.select(&selector).next() {
            Some(meta) => meta,
            None => {
                return Err(Error::Parse(ParseError::DescriptionMetaTagNotFound));
            }
        };

        // Get the content of the description `meta` tag. It will look something like
        // `100 Followers, 50 Following, 30 Posts - See Instagram photos and videos from Foo Bar (@foobar)`.
        let content = match meta.value().attr("content") {
            Some(content) => content,
            None => {
                return Err(Error::Parse(ParseError::ContentAttributeNotFound));
            }
        }
        .trim();

        // Strip off the end half of the content string and
        // parse the followers, following, and posts from the result.
        let src: Vec<u64> = match content.find('-') {
            // If the split is found, then strip, split, and parse.
            Some(index) => {
                let src: Result<_> = content[..index]
                    .split_terminator(',')
                    .map(|s| {
                        s.trim()
                            .split_terminator(' ')
                            .next()
                            .unwrap()
                            .parse::<u64>()
                            .map_err(|_| Error::Parse(ParseError::DataParsingFailed))
                    })
                    .collect();
                src?
            }
            // Otherwise, error.
            None => {
                return Err(Error::Parse(ParseError::DataSourceNotFound));
            }
        };

        // Construct `Data` out of these numbers.
        Ok(Data {
            followers: src[0],
            following: src[1],
            posts: src[2],
        })
    }

    /// Run the scraper and output CSV to a file at the specified path.
    pub async fn run<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        // Open the file in append mode. We don't want to overwrite the data that's already there!
        let mut file = OpenOptions::new()
            .append(true)
            .open(path.as_ref())
            .await
            .map_err(|_| Error::Output(OutputError::OpeningFailed))?;

        loop {
            // Scrape the data.
            match self.scrape().await {
                // If we're successful, write the data with a timestamp to the file.
                Ok(data) => {
                    // Serialize the data to be written to the file and log it.
                    let ser = format!("{},{}", Utc::now(), data.followers);
                    info!("{}", ser);

                    // Write to the file.
                    let _ = file
                        .write(format!("{}\n", ser).as_bytes())
                        .await
                        .map_err(|_| Error::Output(OutputError::WritingFailed))?;
                    file.flush()
                        .await
                        .map_err(|_| Error::Output(OutputError::FlushingFailed))?;
                }
                // If not, log the error and don't do anything.
                Err(err) => error!("{}", err),
            };

            self.sleep().await;
        }
    }

    /// Scrape and parse the document at the URL.
    async fn document(&self) -> Result<Html> {
        Ok(Html::parse_document(
            &self
                .client
                .get(&self.url)
                .send()
                .await
                .map_err(|_| Error::Document(DocumentError::RequestingFailed))?
                .text()
                .await
                .map_err(|_| Error::Document(DocumentError::ParsingFailed))?,
        ))
    }

    /// Sleep for the duration of the interval.
    async fn sleep(&self) {
        time::delay_for(self.interval).await;
    }
}

/// Helper for creating an Instagram scraper.
#[derive(Debug, Clone, Default)]
pub struct ScraperBuilder {
    client: Option<Client>,
    config: Option<Config>,
}

impl ScraperBuilder {
    /// Initialize a new builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a client to the scraper.
    pub fn client(&mut self, client: Client) -> &mut Self {
        self.client = Some(client);
        self
    }

    /// Add configuration to the scraper.
    pub fn config(&mut self, config: Config) -> &mut Self {
        self.config = Some(config);
        self
    }

    /// Build the scraper.
    pub fn build(&self) -> Scraper {
        let client = self.client.clone().unwrap();
        let config = self.config.clone().unwrap();

        Scraper {
            client,
            url: format!("https://www.instagram.com/{}/", config.user),
            interval: Duration::from_secs(config.interval),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger.
    env_logger::init();

    // Build the scraper.
    info!("initializing scraper...");
    let scraper = ScraperBuilder::new()
        .client(Client::new())
        .config(Config::load("./config.toml").await?)
        .build();

    // Run the scraper.
    info!("running scraper...");
    scraper.run("./followers.csv").await?;

    Ok(())
}
