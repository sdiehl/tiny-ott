//! Tiny observational type theory checker.

#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::match_same_arms,
    clippy::len_without_is_empty,
    clippy::use_self,
    clippy::doc_markdown
)]

pub mod diagnostics;
pub mod driver;
pub mod elab;
pub mod errors;
pub mod eval;
pub mod lexer;
pub mod parse;
pub mod pretty;
pub mod syntax;
pub mod value;

pub use driver::check_str;
pub use errors::{TinyOttError, TinyOttResult};
