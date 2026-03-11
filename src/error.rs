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

    #[error("unexpected end of file")]
    UnexpectedEof,

    #[error("unknown decode error")]
    Unknown,
}

impl<I> NomParseError<I> for DecodeError {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        match kind {
            ErrorKind::Eof => Self::UnexpectedEof,
            _ => Self::Unknown,
        }
    }

    fn append(_input: I, _kind: ErrorKind, other: Self) -> Self {
        other
    }
}

impl From<nom::Err<DecodeError>> for DecodeError {
    fn from(value: nom::Err<DecodeError>) -> Self {
        match value {
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
            nom::Err::Incomplete(_) => Self::UnexpectedEof,
        }
    }
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("string too long")]
    StringTooLong { max: u8 },

    #[error("cannot encode character: {0}")]
    EncodingError(char),

    #[error("board data too large")]
    BoardTooLarge,
}
