use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    result::Result as StdResult,
};

#[derive(Debug, Copy, Clone)]
pub enum ConfigError {
    OpeningFailed,
    CreationFailed,
    ReadingFailed,
    WritingFailed,
    DeserializationFailed,
    SerializationFailed,

    __Nonexhaustive,
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::OpeningFailed => "failed to open configuration file",
                Self::CreationFailed => "failed to create configuration file",
                Self::ReadingFailed => "failed to read configuration file",
                Self::WritingFailed => "failed to write configuration file",
                Self::DeserializationFailed => "failed to deserialize configuration",
                Self::SerializationFailed => "failed to serialize configuration",
                _ => "oops! this shouldn't ever happen!",
            }
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ParseError {
    SelectorParsingFailed,
    DescriptionMetaTagNotFound,
    ContentAttributeNotFound,
    DataSourceNotFound,
    DataParsingFailed,

    __Nonexhaustive,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::SelectorParsingFailed => "failed to parse selector",
                Self::DescriptionMetaTagNotFound => "failed to find description meta tag",
                Self::ContentAttributeNotFound => "failed to find content attribute",
                Self::DataSourceNotFound => "failed to find data source",
                Self::DataParsingFailed => "failed to parse data",
                _ => "oops! this shouldn't ever happen!",
            }
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OutputError {
    OpeningFailed,
    WritingFailed,
    FlushingFailed,

    __Nonexhaustive,
}

impl Display for OutputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::OpeningFailed => "failed to open output file",
                Self::WritingFailed => "failed to write output file",
                Self::FlushingFailed => "failed to flush output file",
                _ => "oops! this shouldn't ever happen!",
            }
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DocumentError {
    RequestingFailed,
    ParsingFailed,

    __Nonexhaustive,
}

impl Display for DocumentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::RequestingFailed => "failed to request document",
                Self::ParsingFailed => "failed to parse document",
                _ => "oops! this shouldn't ever happen!",
            }
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    Config(ConfigError),
    Parse(ParseError),
    Output(OutputError),
    Document(DocumentError),

    __Nonexhaustive,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Config(error) => format!("{}", error),
                Self::Parse(error) => format!("{}", error),
                Self::Output(error) => format!("{}", error),
                Self::Document(error) => format!("{}", error),
                _ => "oops! this shouldn't ever happen!".to_owned(),
            }
        )
    }
}

impl StdError for Error {}

pub type Result<T> = StdResult<T, Error>;
