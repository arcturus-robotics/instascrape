//! An Instagram scraping library.

#![deny(
    rust_2018_idioms,
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]

use anyhow::{anyhow, Result};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const SELECTOR: &str = r#"meta[property="og:description"]"#;

/// Data scraped from a user's Instagram page.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Data {
    /// The amount of followers a user has.
    pub followers: u64,
    /// The amount of users a user is following.
    pub following: u64,
    /// The amount of posts a user has created.
    pub posts: u64,
}

/// An Instagram scraper.
#[derive(Debug, Clone)]
pub struct Scraper {
    /// The HTTP client the scraper can reuse.
    client: Client,
    /// The user to scrape data from.
    pub user: String,
}

impl Scraper {
    pub fn new(user: &str) -> Self {
        Self {
            client: Client::builder()
                .user_agent("Instascrape (https://github.com/arcturus-robotics/instascrape, 0.2.0)")
                .build()
                .expect("failed to create HTTP client"),
            user: String::from(user),
        }
    }

    /// Scrape data from the URL.
    pub async fn scrape(&self) -> Result<Data> {
        // Scrape the document.
        let document = self.document().await?;

        // Create a selector to find the description `meta` tag.
        let selector = match Selector::parse(SELECTOR) {
            Ok(selector) => selector,
            Err(err) => {
                return Err(anyhow!(
                    "failed to parse selector `{}`: {:?}",
                    SELECTOR,
                    err
                ))
            }
        };

        // Find the description `meta` tag with the selector.
        let meta = match document.select(&selector).next() {
            Some(meta) => meta,
            None => {
                return Err(anyhow!("`meta` tag of selector `{}` not found", SELECTOR));
            }
        };

        // Get the content of the description `meta` tag. It will look something like
        // `100 Followers, 50 Following, 30 Posts - See Instagram photos and videos from Foo Bar (@foobar)`.
        let content = match meta.value().attr("content") {
            Some(content) => content,
            None => {
                return Err(anyhow!("`content` attribute not found in `meta` tag"));
            }
        }
        .trim();

        // Strip off the end half of the content string and
        // parse the followers, following, and posts from the result.
        let source: Vec<u64> = match content.find('-') {
            // If the split is found, then strip, split, and parse.
            Some(index) => {
                let src: Result<Vec<u64>> = content[..index]
                    .split_terminator(',')
                    .map(|s| {
                        let parsed = s
                            .trim()
                            .split_terminator(' ')
                            .next()
                            .unwrap()
                            .parse::<u64>();

                        match parsed {
                            Ok(parsed) => Ok(parsed),
                            Err(err) => Err(anyhow!(
                                "failed to parse data in `content` attribute: {}",
                                err
                            )),
                        }
                    })
                    .collect();
                src?
            }
            // Otherwise, error.
            None => {
                return Err(anyhow!("failed to find data in `content` attribute"));
            }
        };

        // Construct `Data` out of these numbers.
        Ok(Data {
            followers: source[0],
            following: source[1],
            posts: source[2],
        })
    }

    /// Scrape and parse the document at the URL.
    async fn document(&self) -> Result<Html> {
        match self
            .client
            .get(&format!("https://www.instagram.com/{}/", self.user))
            .send()
            .await
        {
            Ok(request) => match request.text().await {
                Ok(text) => Ok(Html::parse_document(&text)),
                Err(err) => Err(anyhow!(
                    "failed to parse user `{}`'s Instagram page: {}",
                    self.user,
                    err
                )),
            },
            Err(err) => Err(anyhow!(
                "failed to request user `{}`'s Instagram page: {}",
                self.user,
                err
            )),
        }
    }
}

pub async fn scrape(user: &str) -> Result<Data> {
    Scraper::new(user).scrape().await
}
