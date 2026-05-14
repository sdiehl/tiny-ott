//! Tiny observational type theory checker.

/// Greet by name.
#[must_use]
pub fn greet(name: &str) -> String {
    format!("hello, {name}")
}
