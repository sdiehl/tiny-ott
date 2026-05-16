use crate::errors::{ParseError, TinyOttResult};
use crate::lexer::{Lexer, LexicalError, Token};
use crate::syntax::{Decl, Raw};

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
    module_parser: parser::ModuleParser,
    term_parser: parser::TermParser,
}

impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            module_parser: parser::ModuleParser::new(),
            term_parser: parser::TermParser::new(),
        }
    }

    pub fn parse_module(&self, input: &str) -> TinyOttResult<Vec<Decl>> {
        let tokens = Lexer::new(input);
        match self.module_parser.parse(tokens) {
            Ok(m) => Ok(m),
            Err(e) => Err(convert(e, input).into()),
        }
    }

    pub fn parse_term(&self, input: &str) -> TinyOttResult<Raw> {
        let tokens = Lexer::new(input);
        match self.term_parser.parse(tokens) {
            Ok(t) => Ok(t),
            Err(e) => Err(convert(e, input).into()),
        }
    }
}

fn convert(err: lalrpop_util::ParseError<usize, Token, LexicalError>, input: &str) -> ParseError {
    use lalrpop_util::ParseError as P;
    match err {
        P::InvalidToken { location } => ParseError::InvalidToken { offset: location },
        P::UnrecognizedEof { location, expected } => ParseError::UnexpectedEof {
            expected,
            offset: location.min(input.len()),
        },
        P::UnrecognizedToken {
            token: (start, tok, end),
            expected,
        } => ParseError::UnexpectedToken {
            token: tok.to_string(),
            expected,
            span: (start, end),
        },
        P::ExtraToken {
            token: (start, tok, end),
        } => ParseError::ExtraToken {
            token: tok.to_string(),
            span: (start, end),
        },
        P::User { error } => ParseError::Lexical(error),
    }
}
