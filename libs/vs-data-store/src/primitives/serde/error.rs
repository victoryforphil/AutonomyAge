use std;
use std::fmt::{self, Display};

use serde::{de, ser};

pub type PrimitiveResult<T> = std::result::Result<T, PrimitiveError>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Debug)]
pub enum PrimitiveError {
    Message(String),

    NotSupported,
    Eof,
}

impl ser::Error for PrimitiveError {
    fn custom<T: Display>(msg: T) -> Self {
        PrimitiveError::Message(msg.to_string())
    }
}

impl de::Error for PrimitiveError {
    fn custom<T: Display>(msg: T) -> Self {
        PrimitiveError::Message(msg.to_string())
    }
}

impl Display for PrimitiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrimitiveError::Message(msg) => formatter.write_str(msg),
            PrimitiveError::Eof => formatter.write_str("unexpected end of input"),
            PrimitiveError::NotSupported => formatter.write_str("operation not supported"),
            /* and so forth */
        }
    }
}

impl std::error::Error for PrimitiveError {}
