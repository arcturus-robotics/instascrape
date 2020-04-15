use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    Scrape(instascrape::Error),
    ParseInterval,
    CreateClient,
    OpenOutput,
    SendMessageThroughWebhook,
    WriteOutput,
    FlushOutput,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Scrape(error) => write!(f, "{}", error),
            Self::ParseInterval => write!(f, "failed to parse interval"),
            Self::CreateClient => write!(
                f,
                "failed to create HTTP client for Discord webhook messages"
            ),
            Self::OpenOutput => write!(f, "failed to open output file"),
            Self::SendMessageThroughWebhook => {
                write!(f, "failed to send message through Discord webhook")
            }
            Self::WriteOutput => write!(f, "failed to write to output file"),
            Self::FlushOutput => write!(f, "failed to flush output file"),
        }
    }
}

impl std::error::Error for Error {}

impl From<instascrape::Error> for Error {
    fn from(error: instascrape::Error) -> Self {
        Self::Scrape(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
