use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Ser(toml::ser::Error),
    De(toml::de::Error),
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Ser(error)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        Self::De(error)
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.source().unwrap())
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(match self {
            Self::Io(error) => error,
            Self::Ser(error) => error,
            Self::De(error) => error,
        })
    }
}

pub type ConfigResult<T> = Result<T, ConfigError>;

/// The scraper's configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The user to fetch data from.
    pub user: String,
    /// The update interval in seconds.
    pub interval: u64,
}

impl Config {
    /// Load the config.
    pub fn load<P>(path: P) -> ConfigResult<Self>
    where
        P: AsRef<Path>,
    {
        info!("loading configuration...");

        debug!("opening configuration file...");
        let mut file = File::open(path.as_ref())?;

        debug!("reading configuration file...");
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        debug!("deserializing configuration...");
        let de = toml::from_str(&buf)?;

        Ok(de)
    }

    pub fn save<P>(&self, path: P) -> ConfigResult<()>
    where
        P: AsRef<Path>,
    {
        info!("saving configuration...");

        debug!("serializing configuration...");
        let ser = toml::to_string_pretty(self)?;

        debug!("creating configuration file...");
        let mut file = File::create(path.as_ref())?;

        debug!("writing configuration file...");
        file.write_all(ser.as_bytes())?;

        Ok(())
    }
}
