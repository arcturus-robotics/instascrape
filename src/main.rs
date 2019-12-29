//! An Instagram scraper created to help keep track of our Instagram followers.

#![deny(
    rust_2018_idioms,
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]

#[macro_use]
extern crate log;

use chrono::Utc;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs::OpenOptions,
    io::{self, Write},
    num::ParseIntError,
    path::Path,
    thread,
    time::Duration,
};

pub mod config;

use self::config::{Config, ConfigError};

/// Scraped Instagram data.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Data {
    pub followers: u64,
    pub following: u64,
    pub posts: u64,
}

/// Any error that may occur while initializing or running the scraper.
#[derive(Debug)]
pub enum InstascrapeError {
    Io(io::Error),
    Reqwest(reqwest::Error),
    ParseInt(ParseIntError),
    Config(ConfigError),
}

impl From<io::Error> for InstascrapeError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<reqwest::Error> for InstascrapeError {
    fn from(error: reqwest::Error) -> Self {
        Self::Reqwest(error)
    }
}

impl From<ParseIntError> for InstascrapeError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseInt(error)
    }
}

impl From<ConfigError> for InstascrapeError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

impl Display for InstascrapeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source().unwrap())
    }
}

impl Error for InstascrapeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::Io(error) => error,
            Self::Reqwest(error) => error,
            Self::ParseInt(error) => error,
            Self::Config(error) => error,
        })
    }
}

pub type InstascrapeResult<T> = Result<T, InstascrapeError>;

/// An Instagram scraper.
#[derive(Debug, Clone)]
pub struct Instascrape {
    client: Client,

    url: String,
    interval: Duration,
}

impl Instascrape {
    /// Get the URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the loop interval.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Scrape data from the URL.
    pub fn scrape(&self) -> InstascrapeResult<Data> {
        // Scrape the document.
        let document = self.document()?;

        // Create a selector to find the description `meta` tag.
        let selector = Selector::parse(r#"meta[property="og:description"]"#).unwrap();

        // Find the description `meta` tag with the selector.
        let meta = match document.select(&selector).next() {
            Some(meta) => meta,
            None => {
                return Err(InstascrapeError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "failed to find the description `meta` tag",
                )));
            }
        };

        // Get the content of the description `meta` tag. It will look something like
        // `100 Followers, 50 Following, 30 Posts - See Instagram photos and videos from Foo Bar (@foobar)`.
        let content = match meta.value().attr("content") {
            Some(content) => content,
            None => {
                return Err(InstascrapeError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "failed to get the `content` attribute of the description `meta` tag",
                )));
            }
        }
        .trim();

        // Strip off the end half of the content string and
        // parse the followers, following, and posts from the result.
        let src: Vec<u64> = match content.find('-') {
            // If the split is found, then strip, split, and parse.
            Some(index) => {
                let src: Result<_, _> = content[..index]
                    .split_terminator(',')
                    .map(|s| {
                        s.trim()
                            .split_terminator(' ')
                            .next()
                            .unwrap()
                            .parse::<u64>()
                    })
                    .collect();
                src?
            }
            // Otherwise, error.
            None => {
                return Err(InstascrapeError::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "failed to find data source",
                )));
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
    pub fn run<P>(&self, path: P) -> InstascrapeResult<()>
    where
        P: AsRef<Path>,
    {
        // Open the file in append mode. We don't want to overwrite the data that's already there!
        let mut file = OpenOptions::new().append(true).open(path.as_ref())?;

        loop {
            // Scrape the data.
            match self.scrape() {
                // If we're successful, write the data with a timestamp to the file.
                Ok(data) => {
                    let _ =
                        file.write(format!("{},{}\n", Utc::now(), data.followers).as_bytes())?;
                    file.flush()?;
                }
                // If not, log the error and don't do anything.
                Err(err) => error!("{}", err),
            };

            self.sleep();
        }
    }

    /// Scrape and parse the document at the URL.
    fn document(&self) -> InstascrapeResult<Html> {
        Ok(Html::parse_document(
            &self.client.get(&self.url).send()?.text()?,
        ))
    }

    /// Sleep for the duration of the interval.
    fn sleep(&self) {
        thread::sleep(self.interval);
    }
}

/// Helper for creating a scraper.
#[derive(Debug, Clone, Default)]
pub struct InstascrapeBuilder {
    client: Option<Client>,
    config: Option<Config>,
}

impl InstascrapeBuilder {
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
    pub fn build(&self) -> Instascrape {
        let client = self.client.clone().unwrap();
        let config = self.config.clone().unwrap();

        Instascrape {
            client,
            url: format!("https://www.instagram.com/{}/", config.user),
            interval: Duration::from_secs(config.interval),
        }
    }
}

fn main() -> InstascrapeResult<()> {
    // Initialize the logger.
    env_logger::init();

    // Build the scraper.
    info!("initializing scraper...");
    let scraper = InstascrapeBuilder::new()
        .client(Client::new())
        .config(Config::load("./config.toml")?)
        .build();

    // Run the scraper.
    info!("running scraper...");
    scraper.run("./followers.csv")?;

    Ok(())
}
