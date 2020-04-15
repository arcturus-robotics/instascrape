//! An asynchronous and easy-to-use Instagram-scraping library.

#![deny(
    rust_2018_idioms,
    clippy::all,
    missing_debug_implementations,
    missing_copy_implementations
)]

use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

mod error;

const SELECTOR: &str = r#"meta[property="og:description"]"#;
const USER_AGENT: &str = concat!(
    "Instascrape (",
    env!("CARGO_PKG_HOMEPAGE"),
    ", ",
    env!("CARGO_PKG_VERSION"),
    ")"
);

pub use self::error::{Error, Result};

/// Data scraped from a user's Instagram profile.
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
    /// The HTTP client the scraper will use to fetch the user's
    /// Instagram profile.
    client: Client,
    /// The user whose profile will be scraped.
    pub user: String,
}

impl Scraper {
    /// Create a new scraper.
    ///
    /// An error will be returned if the client fails to be created.
    pub fn new(user: &str) -> Result<Self> {
        Ok(Self {
            client: if let Ok(client) = Client::builder().user_agent(USER_AGENT).build() {
                client
            } else {
                return Err(Error::CreateClient);
            },
            user: String::from(user),
        })
    }

    /// Scrape data from the URL of the user's profile.
    pub async fn scrape(&self) -> Result<Data> {
        // Scrape the document.
        let document = self.document().await?;

        // Create a selector to find the description `meta` tag.
        let selector = match Selector::parse(SELECTOR) {
            Ok(selector) => selector,
            Err(_) => {
                return Err(Error::ParseSelector);
            }
        };

        // Find SEO tag with the selector.
        let seo = match document.select(&selector).next() {
            Some(seo) => seo,
            None => {
                return Err(Error::FindSeo);
            }
        };

        // Get the content of the SEO tag. It will look something like the following:
        // "100 Followers, 50 Following, 30 Posts - See Instagram photos and videos from Foo Bar (@foobar)"
        let content = match seo.value().attr("content") {
            Some(content) => content,
            None => {
                return Err(Error::GetSeoContent);
            }
        }
        .trim();

        // Strip off the end half of the content string and
        // parse the followers, following, and posts from the result.
        let source: Vec<u64> = match content.find('-') {
            // If the split is found, then strip, split, and parse.
            Some(index) => {
                let source: Result<Vec<u64>> = content[..index]
                    .split_terminator(',')
                    .map(|s| {
                        let parsed = {
                            let split = s.trim().split_terminator(' ').next();

                            if let Some(split) = split {
                                split.parse::<u64>()
                            } else {
                                return Err(Error::SplitSeoContent);
                            }
                        };

                        match parsed {
                            Ok(parsed) => Ok(parsed),
                            Err(_) => Err(Error::ParseSeoData),
                        }
                    })
                    .collect();
                source?
            }
            // Otherwise, error.
            None => {
                return Err(Error::InvalidSeoContent);
            }
        };

        // If the source appears to be invalid as its length is
        // too short, error.
        if source.len() < 3 {
            return Err(Error::NotEnoughSeoData);
        }

        // Collect the data.
        Ok(Data {
            followers: source[0],
            following: source[1],
            posts: source[2],
        })
    }

    /// Scrape and parse the document at the URL of the user's profile.
    async fn document(&self) -> Result<Html> {
        match self
            .client
            .get(&format!("https://www.instagram.com/{}/", self.user))
            .send()
            .await
        {
            Ok(request) => match request.text().await {
                Ok(text) => Ok(Html::parse_document(&text)),
                Err(_) => Err(Error::ParseProfile),
            },
            Err(_) => Err(Error::GetProfile),
        }
    }
}

pub async fn scrape(user: &str) -> Result<Data> {
    Scraper::new(user)?.scrape().await
}
