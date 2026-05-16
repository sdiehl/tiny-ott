use thiserror::Error;

use crate::lexer::LexicalError;

pub type Span = (usize, usize);

#[derive(Debug, Error)]
pub enum TinyOttError {
    #[error("lexical error: {0}")]
    Lexical(#[from] LexicalError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("type error: {0}")]
    Type(#[from] TypeError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error, Clone)]
pub enum ParseError {
    #[error("unexpected token '{token}', expected one of: {}", .expected.join(", "))]
    UnexpectedToken {
        token: String,
        expected: Vec<String>,
        span: Span,
    },
    #[error("unexpected end of input, expected one of: {}", .expected.join(", "))]
    UnexpectedEof {
        expected: Vec<String>,
        offset: usize,
    },
    #[error("invalid token at offset {offset}")]
    InvalidToken { offset: usize },
    #[error("extra token '{token}' after input")]
    ExtraToken { token: String, span: Span },
    #[error("lexical error: {0}")]
    Lexical(#[from] LexicalError),
}

#[derive(Debug, Error, Clone)]
#[error("{message}")]
pub struct TypeError {
    pub message: String,
}

impl TypeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub type TinyOttResult<T> = Result<T, TinyOttError>;
