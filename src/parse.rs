use std::fmt;

use lalrpop_util::ParseError as LalrParseError;

use crate::errors::{ParseError, TinyOttResult};
use crate::lexer::{Lexer, LexicalError, Token};
use crate::syntax::{Decl, Raw, ReplInput};

#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    dead_code,
    unreachable_pub
)]
mod parser_impl {
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub parser);
}
use parser_impl::parser;

pub struct Parser {
    module: parser::ModuleParser,
    term: parser::TermParser,
    repl: parser::ReplParser,
}

impl fmt::Debug for Parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Parser").finish()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self {
            module: parser::ModuleParser::new(),
            term: parser::TermParser::new(),
            repl: parser::ReplParser::new(),
        }
    }

    pub fn parse_module(&self, input: &str) -> TinyOttResult<Vec<Decl>> {
        let tokens = Lexer::new(input);
        match self.module.parse(tokens) {
            Ok(m) => Ok(m),
            Err(e) => Err(convert(e, input).into()),
        }
    }

    pub fn parse_term(&self, input: &str) -> TinyOttResult<Raw> {
        let tokens = Lexer::new(input);
        match self.term.parse(tokens) {
            Ok(t) => Ok(t),
            Err(e) => Err(convert(e, input).into()),
        }
    }

    pub fn parse_repl(&self, input: &str) -> TinyOttResult<ReplInput> {
        let tokens = Lexer::new(input);
        match self.repl.parse(tokens) {
            Ok(r) => Ok(r),
            Err(e) => Err(convert(e, input).into()),
        }
    }
}

fn convert(err: LalrParseError<usize, Token, LexicalError>, input: &str) -> ParseError {
    match err {
        LalrParseError::InvalidToken { location } => ParseError::InvalidToken { offset: location },
        LalrParseError::UnrecognizedEof { location, expected } => ParseError::UnexpectedEof {
            expected,
            offset: location.min(input.len()),
        },
        LalrParseError::UnrecognizedToken {
            token: (start, tok, end),
            expected,
        } => ParseError::UnexpectedToken {
            token: tok.to_string(),
            expected,
            span: (start, end),
        },
        LalrParseError::ExtraToken {
            token: (start, tok, end),
        } => ParseError::ExtraToken {
            token: tok.to_string(),
            span: (start, end),
        },
        LalrParseError::User { error } => ParseError::Lexical(error),
    }
}
