//! Talks to the ESV HTTP API and hands back a `Passage`.
//!
//! The JSON shape ESV happens to return today is `pub(crate)` in `wire`, and the
//! parsing of their plain-text layout lives in `parse`. Neither escapes this
//! crate — so when ESV changes their format, the blast radius is these files.

mod client;
mod parse;
mod wire;

pub use client::{AUTH_TOKEN_ENV_VAR, EsvClient, MissingApiKey};
