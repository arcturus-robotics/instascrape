use std::fmt::{Display, Formatter};

/// Any of the many errors that can occur while scraping data
/// from an Instagram profile.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Error {
    CreateClient,
    GetProfile,
    ParseProfile,
    ParseSelector,
    FindSeo,
    GetSeoContent,
    SplitSeoContent,
    ParseSeoData,
    InvalidSeoContent,
    NotEnoughSeoData,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::CreateClient => "failed to create HTTP client",
                Self::GetProfile => "failed to fetch user profile",
                Self::ParseProfile => "failed to parse user profile",
                Self::ParseSelector =>
                    "failed to parse selector pointing to SEO tag; this shouldn't happen",
                Self::FindSeo => "failed to find SEO tag using selector",
                Self::GetSeoContent => "failed to get the SEO tag's content attribute",
                Self::SplitSeoContent => "failed to split the SEO tag's content in two",
                Self::ParseSeoData => "failed to parse data in the SEO tag",
                Self::InvalidSeoContent => "invalid SEO content",
                Self::NotEnoughSeoData => "not enough SEO data",

                // TODO: replace this with a #[non_exhaustive] attribute on Error once it is stable
                #[allow(unreachable_patterns)]
                _ => "unknown error; this is a bug!",
            }
        )
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
