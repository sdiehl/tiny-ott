use std::fmt;

use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
pub enum Token {
    #[token("def")]
    Def,
    #[token("eval")]
    Eval,
    #[token("check")]
    Check,
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("fun")]
    Fun,

    #[token("Type")]
    Type,
    #[token("Nat")]
    Nat,
    #[token("Bool")]
    Bool,
    #[token("Unit")]
    Unit,
    #[token("Empty")]
    Empty,
    #[token("zero")]
    Zero,
    #[token("suc")]
    Suc,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("tt")]
    TT,
    #[token("refl")]
    Refl,
    #[token("coe")]
    Coe,
    #[token("Eq")]
    Eq,
    #[token("natrec")]
    NatRec,
    #[token("boolrec")]
    BoolRec,
    #[token("empty-rec")]
    EmptyRec,
    #[token("fst")]
    Fst,
    #[token("snd")]
    Snd,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(":=")]
    ColonEq,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("*")]
    Star,
    #[token("\\")]
    Backslash,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_'-]*", |lex| lex.slice().to_owned(), priority = 1)]
    Ident(String),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<u64>().ok())]
    Number(u64),

    #[regex(r"[ \t\n\r\f]+", logos::skip)]
    #[regex(r"--[^\n\r]*", logos::skip, allow_greedy = true)]
    Whitespace,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Def => write!(f, "def"),
            Self::Eval => write!(f, "eval"),
            Self::Check => write!(f, "check"),
            Self::Let => write!(f, "let"),
            Self::In => write!(f, "in"),
            Self::Fun => write!(f, "fun"),
            Self::Type => write!(f, "Type"),
            Self::Nat => write!(f, "Nat"),
            Self::Bool => write!(f, "Bool"),
            Self::Unit => write!(f, "Unit"),
            Self::Empty => write!(f, "Empty"),
            Self::Zero => write!(f, "zero"),
            Self::Suc => write!(f, "suc"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::TT => write!(f, "tt"),
            Self::Refl => write!(f, "refl"),
            Self::Coe => write!(f, "coe"),
            Self::Eq => write!(f, "Eq"),
            Self::NatRec => write!(f, "natrec"),
            Self::BoolRec => write!(f, "boolrec"),
            Self::EmptyRec => write!(f, "empty-rec"),
            Self::Fst => write!(f, "fst"),
            Self::Snd => write!(f, "snd"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::ColonEq => write!(f, ":="),
            Self::Arrow => write!(f, "->"),
            Self::FatArrow => write!(f, "=>"),
            Self::Star => write!(f, "*"),
            Self::Backslash => write!(f, "\\"),
            Self::Ident(s) => write!(f, "{s}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Whitespace => write!(f, "<whitespace>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LexicalError {
    #[error("invalid token at byte offset {0}")]
    InvalidToken(usize),
}

impl LexicalError {
    pub fn offset(&self) -> usize {
        match self {
            Self::InvalidToken(o) => *o,
        }
    }
}

#[derive(Debug)]
pub struct Lexer<'input> {
    inner: logos::Lexer<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            inner: Token::lexer(input),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<(usize, Token, usize), LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let tok = self.inner.next()?;
            let span = self.inner.span();
            return Some(match tok {
                Ok(Token::Whitespace) => continue,
                Ok(t) => Ok((span.start, t, span.end)),
                Err(()) => Err(LexicalError::InvalidToken(span.start)),
            });
        }
    }
}
