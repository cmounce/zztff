use nom::error::{ErrorKind, ParseError as NomParseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid ZZT file magic")]
    InvalidMagic,

    #[error("invalid tile count: {0}")]
    InvalidTileCount(usize),

    #[error("negative stat count")]
    NegativeStatCount,

    #[error("nom error: {0}")]
    NomError(String),
}

impl<I> NomParseError<I> for DecodeError {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Self::NomError(kind.description().to_string())
    }

    fn append(_input: I, kind: ErrorKind, other: Self) -> Self {
        Self::NomError(format!("{}: {:?}", other, kind))
    }
}

impl From<nom::Err<DecodeError>> for DecodeError {
    fn from(value: nom::Err<DecodeError>) -> Self {
        match value {
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
            nom::Err::Incomplete(needed) => Self::NomError(format!("incomplete: {:?}", needed)),
        }
    }
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("invalid tile count: {0}")]
    InvalidTileCount(usize),

    #[error("string too long")]
    StringTooLong { max: u8 },

    #[error("cannot encode character: {0}")]
    EncodingError(char),

    #[error("board data too large")]
    BoardTooLarge,
}
