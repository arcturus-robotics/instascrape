#![deny(
    rust_2018_idioms,
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]

use chrono::Utc;
use reqwest::{Client, Result as ReqwestResult};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs::File,
    io::{Read, Result as IoResult, Write},
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
        let mut file = File::open("./config.toml")?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        Ok(toml::de::from_str(&buf)?)
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
            Err(_) => return None,
        };

        let selector = Selector::parse(r#"meta[property="og:description"]"#).unwrap();
        let meta = match document.select(&selector).next() {
            Some(meta) => meta,
            None => return None,
        };
        let content = meta.value().attr("content").unwrap().trim();

        match content.find("Followers") {
            Some(index) => Some(match content[..index].trim().parse::<u64>() {
                Ok(followers) => followers,
                Err(_) => return None,
            }),
            None => None,
        }
    }

    /// Run the scraper and output CSV to a file at the specified path.
    pub fn run<P>(&self, path: P) -> IoResult<()>
    where
        P: AsRef<Path>,
    {
        let duration = self.duration();

        let mut file = File::create(path.as_ref())?;
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

    fn document(&self) -> ReqwestResult<Html> {
        Ok(Html::parse_document(
            &self.client.get(self.url().as_str()).send()?.text()?,
        ))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let scraper = Scraper {
        client: Client::new(),
        config: Config::load()?,
    };

    scraper.run("./followers.csv")?;

    Ok(())
}
