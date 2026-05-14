use crate::syntax::{name, Decl, Name, Raw};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tok {
    LParen,
    RParen,
    Comma,
    Colon,
    ColonEq,
    Arrow,
    FatArrow,
    Star,
    Backslash,
    Ident(String),
    Num(u64),
}

struct Lexer<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src: src.as_bytes(),
            pos: 0,
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek_byte() {
            if c.is_ascii_whitespace() {
                self.pos += 1;
            } else if c == b'-' && self.src.get(self.pos + 1) == Some(&b'-') {
                while let Some(c) = self.peek_byte() {
                    self.pos += 1;
                    if c == b'\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn ident_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || c == b'_' || c == b'-' || c == b'\''
    }

    fn next_tok(&mut self) -> Option<Tok> {
        self.skip_ws();
        let c = self.peek_byte()?;
        if c.is_ascii_digit() {
            let start = self.pos;
            while let Some(c) = self.peek_byte() {
                if c.is_ascii_digit() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let s = std::str::from_utf8(&self.src[start..self.pos]).unwrap();
            return Some(Tok::Num(s.parse().unwrap()));
        }
        if c.is_ascii_alphabetic() || c == b'_' {
            let start = self.pos;
            while let Some(c) = self.peek_byte() {
                if Self::ident_char(c) {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let s = std::str::from_utf8(&self.src[start..self.pos]).unwrap();
            return Some(Tok::Ident(s.to_string()));
        }
        self.pos += 1;
        match c {
            b'(' => Some(Tok::LParen),
            b')' => Some(Tok::RParen),
            b',' => Some(Tok::Comma),
            b'*' => Some(Tok::Star),
            b'\\' => Some(Tok::Backslash),
            b':' => {
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                    Some(Tok::ColonEq)
                } else {
                    Some(Tok::Colon)
                }
            }
            b'-' => {
                if self.peek_byte() == Some(b'>') {
                    self.pos += 1;
                    Some(Tok::Arrow)
                } else {
                    panic!("unexpected '-' at pos {}", self.pos - 1)
                }
            }
            b'=' => {
                if self.peek_byte() == Some(b'>') {
                    self.pos += 1;
                    Some(Tok::FatArrow)
                } else {
                    panic!("unexpected '=' at pos {}", self.pos - 1)
                }
            }
            _ => panic!("unexpected char {:?} at pos {}", c as char, self.pos - 1),
        }
    }

    fn tokens(mut self) -> Vec<Tok> {
        let mut v = Vec::new();
        while let Some(t) = self.next_tok() {
            v.push(t);
        }
        v
    }
}

#[derive(Debug)]
pub struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    pub fn new(src: &str) -> Self {
        Self {
            toks: Lexer::new(src).tokens(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn advance(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, t: &Tok) {
        let got = self.advance();
        assert_eq!(got.as_ref(), Some(t), "expected {t:?}, got {got:?}");
    }

    fn eat_keyword(&mut self, kw: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Ident(s)) if s == kw) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect_ident(&mut self) -> Name {
        match self.advance() {
            Some(Tok::Ident(s)) => name(&s),
            other => panic!("expected ident, got {other:?}"),
        }
    }

    pub fn parse_file(&mut self) -> Vec<Decl> {
        let mut decls = Vec::new();
        while self.peek().is_some() {
            decls.push(self.parse_decl());
        }
        decls
    }

    fn parse_decl(&mut self) -> Decl {
        if self.eat_keyword("def") {
            let n = self.expect_ident();
            self.expect(&Tok::Colon);
            let ty = self.parse_tm();
            self.expect(&Tok::ColonEq);
            let body = self.parse_tm();
            Decl::Def(n, ty, body)
        } else if self.eat_keyword("eval") {
            Decl::Eval(self.parse_tm())
        } else if self.eat_keyword("check") {
            let tm = self.parse_tm();
            self.expect(&Tok::Colon);
            let ty = self.parse_tm();
            Decl::Check(tm, ty)
        } else {
            panic!("expected decl keyword, got {:?}", self.peek())
        }
    }

    fn parse_tm(&mut self) -> Raw {
        self.parse_arrow_or_lam()
    }

    fn parse_arrow_or_lam(&mut self) -> Raw {
        match self.peek() {
            Some(Tok::Backslash) => {
                self.advance();
                let binders = self.parse_lam_binders();
                self.expect(&Tok::FatArrow);
                let body = self.parse_tm();
                lam_many(binders, body)
            }
            Some(Tok::Ident(s)) if s == "fun" => {
                self.advance();
                let binders = self.parse_lam_binders();
                self.expect(&Tok::FatArrow);
                let body = self.parse_tm();
                lam_many(binders, body)
            }
            Some(Tok::Ident(s)) if s == "let" => {
                self.advance();
                let n = self.expect_ident();
                self.expect(&Tok::Colon);
                let ty = self.parse_tm();
                self.expect(&Tok::ColonEq);
                let val = self.parse_tm();
                assert!(self.eat_keyword("in"), "expected 'in' after let");
                let body = self.parse_tm();
                Raw::Let(n, Box::new(ty), Box::new(val), Box::new(body))
            }
            _ => self.parse_arrow(),
        }
    }

    fn parse_lam_binders(&mut self) -> Vec<Name> {
        let mut bs = Vec::new();
        while let Some(Tok::Ident(_)) = self.peek() {
            bs.push(self.expect_ident());
        }
        assert!(!bs.is_empty(), "lambda needs at least one binder");
        bs
    }

    fn parse_arrow(&mut self) -> Raw {
        // Either dependent ((x : A) -> B) or non-dep (A -> B), parsed via lookahead.
        // First parse an app/atom; then check for -> or *.
        // For dependent: we need to detect `( ident+ : tm )` followed by `->` or `*`.
        if self.is_telescope_start() {
            return self.parse_dependent();
        }
        let left = self.parse_prod();
        match self.peek() {
            Some(Tok::Arrow) => {
                self.advance();
                let right = self.parse_arrow_or_lam();
                Raw::Arrow(Box::new(left), Box::new(right))
            }
            _ => left,
        }
    }

    fn parse_prod(&mut self) -> Raw {
        let left = self.parse_app();
        match self.peek() {
            Some(Tok::Star) => {
                self.advance();
                let right = self.parse_prod();
                Raw::Prod(Box::new(left), Box::new(right))
            }
            _ => left,
        }
    }

    fn is_telescope_start(&self) -> bool {
        if self.peek() != Some(&Tok::LParen) {
            return false;
        }
        // scan: ( ident+ : ... ) followed by arrow or star
        let mut i = self.pos + 1;
        let mut saw_ident = false;
        while let Some(t) = self.toks.get(i) {
            match t {
                Tok::Ident(_) => {
                    saw_ident = true;
                    i += 1;
                }
                Tok::Colon if saw_ident => {
                    // find matching paren
                    let mut depth = 1;
                    let mut j = i + 1;
                    while let Some(t) = self.toks.get(j) {
                        match t {
                            Tok::LParen => depth += 1,
                            Tok::RParen => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        j += 1;
                    }
                    return matches!(self.toks.get(j + 1), Some(Tok::Arrow | Tok::Star));
                }
                _ => return false,
            }
        }
        false
    }

    fn parse_dependent(&mut self) -> Raw {
        // ( ident+ : tm ) -> rest  OR  ( ident+ : tm ) * rest
        self.expect(&Tok::LParen);
        let mut bs = Vec::new();
        while let Some(Tok::Ident(_)) = self.peek() {
            bs.push(self.expect_ident());
        }
        self.expect(&Tok::Colon);
        let dom = self.parse_tm();
        self.expect(&Tok::RParen);
        let is_pi = match self.peek() {
            Some(Tok::Arrow) => true,
            Some(Tok::Star) => false,
            other => panic!("expected -> or * after telescope, got {other:?}"),
        };
        self.advance();
        let body = self.parse_arrow_or_lam();
        if is_pi {
            Raw::Pi(bs, Box::new(dom), Box::new(body))
        } else {
            Raw::Sigma(bs, Box::new(dom), Box::new(body))
        }
    }

    fn parse_app(&mut self) -> Raw {
        let head = self.parse_head_or_prefix();
        let mut acc = head;
        loop {
            if self.is_atom_start() {
                let arg = self.parse_atom();
                acc = Raw::App(Box::new(acc), Box::new(arg));
            } else {
                break;
            }
        }
        acc
    }

    fn is_atom_start(&self) -> bool {
        match self.peek() {
            Some(Tok::LParen | Tok::Num(_)) => true,
            Some(Tok::Ident(s)) => !matches!(s.as_str(), "def" | "eval" | "check" | "in"),
            _ => false,
        }
    }

    fn parse_head_or_prefix(&mut self) -> Raw {
        match self.peek().cloned() {
            Some(Tok::Ident(s)) => match s.as_str() {
                "suc" => {
                    self.advance();
                    let arg = self.parse_atom();
                    Raw::Suc(Box::new(arg))
                }
                "fst" => {
                    self.advance();
                    let arg = self.parse_atom();
                    Raw::Fst(Box::new(arg))
                }
                "snd" => {
                    self.advance();
                    let arg = self.parse_atom();
                    Raw::Snd(Box::new(arg))
                }
                "Eq" => {
                    self.advance();
                    let a = self.parse_atom();
                    let x = self.parse_atom();
                    let y = self.parse_atom();
                    Raw::Eq(Box::new(a), Box::new(x), Box::new(y))
                }
                "coe" => {
                    self.advance();
                    let a = self.parse_atom();
                    let b = self.parse_atom();
                    let p = self.parse_atom();
                    let t = self.parse_atom();
                    Raw::Coe(Box::new(a), Box::new(b), Box::new(p), Box::new(t))
                }
                "natrec" => {
                    self.advance();
                    let p = self.parse_atom();
                    let z = self.parse_atom();
                    let s2 = self.parse_atom();
                    let n = self.parse_atom();
                    Raw::NatRec(Box::new(p), Box::new(z), Box::new(s2), Box::new(n))
                }
                "boolrec" => {
                    self.advance();
                    let p = self.parse_atom();
                    let t = self.parse_atom();
                    let f = self.parse_atom();
                    let b = self.parse_atom();
                    Raw::BoolRec(Box::new(p), Box::new(t), Box::new(f), Box::new(b))
                }
                "empty-rec" => {
                    self.advance();
                    let p = self.parse_atom();
                    let e = self.parse_atom();
                    Raw::EmptyRec(Box::new(p), Box::new(e))
                }
                _ => self.parse_atom(),
            },
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Raw {
        match self.advance() {
            Some(Tok::Ident(s)) => match s.as_str() {
                "Type" | "U" => Raw::U,
                "Nat" => Raw::Nat,
                "zero" => Raw::Zero,
                "Bool" => Raw::Bool,
                "true" => Raw::BTrue,
                "false" => Raw::BFalse,
                "Unit" => Raw::Unit,
                "tt" => Raw::TT,
                "Empty" => Raw::Empty,
                "refl" => Raw::Refl,
                _ => Raw::Var(name(&s)),
            },
            Some(Tok::Num(n)) => Raw::NumLit(n),
            Some(Tok::LParen) => {
                let t = self.parse_tm();
                match self.advance() {
                    Some(Tok::RParen) => t,
                    Some(Tok::Comma) => {
                        let u = self.parse_tm();
                        self.expect(&Tok::RParen);
                        Raw::Pair(Box::new(t), Box::new(u))
                    }
                    other => panic!("expected ) or , got {other:?}"),
                }
            }
            other => panic!("expected atom, got {other:?}"),
        }
    }
}

fn lam_many(binders: Vec<Name>, body: Raw) -> Raw {
    Raw::Lam(binders, Box::new(body))
}

pub fn parse_str(src: &str) -> Vec<Decl> {
    Parser::new(src).parse_file()
}
