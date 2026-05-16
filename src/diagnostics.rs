use std::ops::Range;

use ariadne::{Color, Label, Report, ReportKind, Source};

use crate::errors::{ParseError, TinyOttError, TypeError};
use crate::lexer::LexicalError;

pub fn render(err: &TinyOttError, source_name: &str, source: &str) -> String {
    let mut buf = Vec::new();
    match err {
        TinyOttError::Lexical(e) => write_lexical(&mut buf, e, source_name, source),
        TinyOttError::Parse(e) => write_parse(&mut buf, e, source_name, source),
        TinyOttError::Type(e) => write_type(&mut buf, e, source_name, source),
        TinyOttError::Io(e) => {
            buf.extend(format!("io error: {e}\n").into_bytes());
        }
    }
    String::from_utf8_lossy(&buf).into_owned()
}

fn write_lexical(out: &mut Vec<u8>, e: &LexicalError, name: &str, source: &str) {
    let offset = e.offset();
    let span = offset..offset + 1;
    build(name, span.clone())
        .with_message("lexical error")
        .with_label(label(name, span).with_message(e.to_string()))
        .finish()
        .write((name, Source::from(source)), out)
        .ok();
}

fn write_parse(out: &mut Vec<u8>, e: &ParseError, name: &str, source: &str) {
    let (span, msg) = parse_span(e, source);
    build(name, span.clone())
        .with_message("parse error")
        .with_label(label(name, span).with_message(msg))
        .finish()
        .write((name, Source::from(source)), out)
        .ok();
}

fn write_type(out: &mut Vec<u8>, e: &TypeError, name: &str, source: &str) {
    let span = 0..source.len().max(1);
    build(name, span.clone())
        .with_message("type error")
        .with_label(label(name, span).with_message(&e.message))
        .finish()
        .write((name, Source::from(source)), out)
        .ok();
}

fn parse_span(e: &ParseError, source: &str) -> (Range<usize>, String) {
    match e {
        ParseError::UnexpectedToken {
            token,
            expected,
            span,
        } => (
            span.0..span.1,
            format!("found '{token}', expected one of: {}", expected.join(", ")),
        ),
        ParseError::UnexpectedEof { expected, offset } => {
            let o = (*offset).min(source.len());
            (
                o..o + 1,
                format!("unexpected end of input, expected: {}", expected.join(", ")),
            )
        }
        ParseError::InvalidToken { offset } => (*offset..*offset + 1, "invalid token".to_string()),
        ParseError::ExtraToken { token, span } => {
            (span.0..span.1, format!("unexpected extra token '{token}'"))
        }
        ParseError::Lexical(le) => (le.offset()..le.offset() + 1, le.to_string()),
    }
}

fn build(name: &str, span: Range<usize>) -> ariadne::ReportBuilder<'_, (&str, Range<usize>)> {
    Report::build(ReportKind::Error, (name, span))
}

fn label(name: &str, span: Range<usize>) -> Label<(&str, Range<usize>)> {
    Label::new((name, span)).with_color(Color::Red)
}
