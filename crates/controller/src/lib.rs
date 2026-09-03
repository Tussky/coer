//! Application logic, in terms of the ports and nothing else.
//!
//! Nothing in this crate knows that verses come over HTTP or that memories land
//! in JSON files. Swapping either is a change at the call site in a view, not
//! a change here.

use coer_model::{Memory, SourceError, Storage, StorageError, VerseRef, VerseSource};

#[derive(Debug, thiserror::Error)]
pub enum CoerError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Source(#[from] SourceError),
}

/// The application. Generic over its ports, so each view names the backends it
/// wants and the compiler builds a specialised copy: `Coer<JsonStorage, EsvClient>`
/// and `Coer<SqliteStorage, EsvClient>` are separate types with no dispatch cost.
pub struct Coer<S, V> {
    storage: S,
    source: V,
}

impl<S: Storage, V: VerseSource> Coer<S, V> {
    pub fn new(storage: S, source: V) -> Self {
        Self { storage, source }
    }

    /// Fetch a passage and file it away.
    pub async fn add_passage(&self, query: &str) -> Result<Memory, CoerError> {
        let passage = self.source.fetch(query).await?;
        let memory = Memory::new(passage);
        self.storage.save(&memory)?;
        Ok(memory)
    }

    /// Read back something already taken on.
    pub fn recall(&self, reference: &VerseRef) -> Result<Memory, CoerError> {
        Ok(self.storage.load(reference)?)
    }

    /// Everything stored, for a menu or an index page.
    pub fn memories(&self) -> Result<Vec<VerseRef>, CoerError> {
        Ok(self.storage.list()?)
    }
}
