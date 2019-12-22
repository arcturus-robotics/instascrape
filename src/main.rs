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
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    thread,
    time::Duration,
};

/// The scraper's configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The user to fetch data from.
    pub user: String,
    /// The update interval in seconds.
    pub interval: u64,
}

impl Config {
    pub fn new(user: &str, interval: u64) -> Self {
        Self {
            user: String::from(user),
            interval,
        }
    }

    /// Load the config.
    pub fn load() -> Result<Self, Box<dyn Error>> {
        info!("opening configuration file...");
        let mut file = File::open("./config.toml")?;
        info!("reading configuration file...");
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        info!("deserializing configuration...");
        let new = toml::de::from_str(&buf)?;

        Ok(new)
    }
}

/// An Instagram scraper.
#[derive(Debug, Clone)]
pub struct Scraper {
    pub client: Client,
    pub config: Config,
}

impl Scraper {
    pub fn url(&self) -> String {
        format!("https://www.instagram.com/{}", self.config.user)
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.config.interval)
    }

    /// Scrape the followers from the document.
    pub fn followers(&self) -> Option<u64> {
        let document = match self.document() {
            Ok(document) => document,
            Err(e) => {
                error!("failed to get document: {}", e);
                return None;
            }
        };

        let selector = match Selector::parse(r#"meta[property="og:description"]"#) {
            Ok(selector) => selector,
            Err(e) => {
                error!("failed to parse selector: {:?}", e);
                return None;
            }
        };
        let meta = match document.select(&selector).next() {
            Some(meta) => meta,
            None => {
                error!("failed to find the description `meta` tag.");
                return None;
            }
        };
        let content = match meta.value().attr("content") {
            Some(content) => content,
            None => {
                error!("failed to get the `content` attribute of the description `meta` tag.");
                return None;
            }
        }
        .trim();

        match content.find("Followers") {
            Some(index) => Some(match content[..index].trim().parse::<u64>() {
                Ok(followers) => followers,
                Err(_) => {
                    error!("failed to parse follows.");
                    return None;
                }
            }),
            None => {
                error!("failed to find follows.");
                None
            }
        }
    }

    /// Run the scraper and output CSV to a file at the specified path.
    pub fn run<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let duration = self.duration();

        let mut file = OpenOptions::new().append(true).open(path.as_ref())?;
        loop {
            let followers = match self.followers() {
                Some(followers) => followers,
                None => continue,
            };

            let _ = file.write(format!("{},{}\n", Utc::now(), followers).as_bytes())?;
            file.flush()?;

            thread::sleep(duration);
        }
    }

    fn document(&self) -> reqwest::Result<Html> {
        Ok(Html::parse_document(
            &self.client.get(self.url().as_str()).send()?.text()?,
        ))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    info!("initializing scraper...");
    let scraper = Scraper {
        client: Client::new(),
        config: Config::load()?,
    };

    info!("running scraper...");
    scraper.run("./followers.csv")?;

    Ok(())
}
