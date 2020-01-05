use crate::error::{ConfigError, Error, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// The scraper's configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The user to scrape data off of.
    pub user: String,
    /// The update interval in seconds.
    pub interval: u64,
}

impl Config {
    /// Load configuration from a file.
    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        info!("loading configuration...");

        debug!("opening configuration file...");
        let mut file =
            File::open(path.as_ref()).map_err(|_| Error::Config(ConfigError::OpeningFailed))?;

        debug!("reading configuration file...");
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|_| Error::Config(ConfigError::ReadingFailed))?;

        debug!("deserializing configuration...");
        let de =
            toml::from_str(&buf).map_err(|_| Error::Config(ConfigError::DeserializationFailed))?;

        Ok(de)
    }

    /// Save configuration to a file.
    pub fn save<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        info!("saving configuration...");

        debug!("serializing configuration...");
        let ser = toml::to_string_pretty(self)
            .map_err(|_| Error::Config(ConfigError::SerializationFailed))?;

        debug!("creating configuration file...");
        let mut file =
            File::create(path.as_ref()).map_err(|_| Error::Config(ConfigError::CreationFailed))?;

        debug!("writing configuration file...");
        file.write_all(ser.as_bytes())
            .map_err(|_| Error::Config(ConfigError::WritingFailed))?;

        Ok(())
    }
}
