//! The contracts between the core and the outside world.
//!
//! These traits live here, next to the model, rather than beside the code that
//! implements them. That is the whole inversion: `coer-storage` depends on this
//! crate to learn what it must provide, and `coer-controller` depends on this
//! crate to say what it needs — so neither ever names the other.

use std::future::Future;

use crate::memory::{Memory, Passage};
use crate::verse::VerseRef;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("nothing stored for {0}")]
    NotFound(VerseRef),
    #[error("the storage backend failed")]
    Backend(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("no passage found for {0:?}")]
    NotFound(String),
    #[error("could not reach the verse source")]
    Transport(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("could not make sense of the response: {0}")]
    Malformed(String),
    #[error("the verse source rejected our credentials")]
    Unauthorised,
}

/// Where memories are kept. Files for the CLI, a database for the web app.
///
/// Deliberately keyed by `VerseRef` rather than a path or a row id: a SQL
/// backend has no "file location", so the port is named in domain terms and
/// each adapter maps that onto whatever it actually uses.
pub trait Storage {
    fn save(&self, memory: &Memory) -> Result<(), StorageError>;
    fn load(&self, reference: &VerseRef) -> Result<Memory, StorageError>;
    fn list(&self) -> Result<Vec<VerseRef>, StorageError>;
}

/// Where scripture comes from. The ESV API in the app, a canned fixture in tests.
///
/// Async, because fetching genuinely is; `Storage` stays sync because writing a
/// small file is not. Shape each port after the work it does, not a blanket rule.
///
/// Written as `-> impl Future<..> + Send` rather than `async fn` so the returned
/// future is guaranteed `Send`. Without that guarantee an async web server will
/// refuse to spawn it, and the error surfaces far from here.
pub trait VerseSource {
    fn fetch(&self, query: &str) -> impl Future<Output = Result<Passage, SourceError>> + Send;
}
