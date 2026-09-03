//! The point of the port/adapter split, demonstrated: these tests exercise the
//! real controller with no network, no filesystem and no API key.

use std::collections::BTreeMap;
use std::sync::Mutex;

use coer_controller::{Coer, CoerError};
use coer_model::{Memory, Passage, SourceError, Storage, StorageError, VerseRef, VerseSource};

fn passage(reference: &str) -> Passage {
    let mut verses = BTreeMap::new();
    verses.insert(1u8, "In the beginning was the Word".to_string());
    verses.insert(2u8, "He was in the beginning with God".to_string());
    Passage {
        reference: VerseRef::new(reference),
        heading: "The Word Became Flesh".to_string(),
        verses,
    }
}

#[derive(Default)]
struct FakeStorage {
    saved: Mutex<Vec<Memory>>,
}

impl Storage for FakeStorage {
    fn save(&self, memory: &Memory) -> Result<(), StorageError> {
        self.saved.lock().unwrap().push(memory.clone());
        Ok(())
    }

    fn load(&self, reference: &VerseRef) -> Result<Memory, StorageError> {
        self.saved
            .lock()
            .unwrap()
            .iter()
            .find(|m| m.reference() == reference)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(reference.clone()))
    }

    fn list(&self) -> Result<Vec<VerseRef>, StorageError> {
        Ok(self
            .saved
            .lock()
            .unwrap()
            .iter()
            .map(|m| m.reference().clone())
            .collect())
    }
}

struct FakeSource(Passage);

impl VerseSource for FakeSource {
    async fn fetch(&self, _query: &str) -> Result<Passage, SourceError> {
        Ok(self.0.clone())
    }
}

/// A source that is reachable but refuses us — the case that is near-impossible
/// to test when the controller calls reqwest directly.
struct Unauthorised;

impl VerseSource for Unauthorised {
    async fn fetch(&self, _query: &str) -> Result<Passage, SourceError> {
        Err(SourceError::Unauthorised)
    }
}

#[tokio::test]
async fn add_passage_stores_what_it_fetched() {
    let coer = Coer::new(FakeStorage::default(), FakeSource(passage("John 1:1-2")));

    let added = coer.add_passage("John 1:1-2").await.expect("should add");
    assert_eq!(added.reference(), &VerseRef::new("John 1:1-2"));
    assert_eq!(added.passage.verses.len(), 2);

    // It is not just returned — it went through the storage port.
    let recalled = coer
        .recall(&VerseRef::new("John 1:1-2"))
        .expect("should recall");
    assert_eq!(recalled, added);
}

#[tokio::test]
async fn a_new_memory_starts_with_no_progress() {
    let coer = Coer::new(FakeStorage::default(), FakeSource(passage("John 1:1-2")));
    let added = coer.add_passage("John 1:1-2").await.unwrap();
    assert!(added.stats.is_empty());
}

#[tokio::test]
async fn a_rejected_key_does_not_save_anything() {
    let coer = Coer::new(FakeStorage::default(), Unauthorised);

    let error = coer.add_passage("John 1:1-2").await.unwrap_err();
    assert!(matches!(
        error,
        CoerError::Source(SourceError::Unauthorised)
    ));
    assert!(coer.memories().unwrap().is_empty());
}

#[tokio::test]
async fn recalling_something_never_added_is_not_found() {
    let coer = Coer::new(FakeStorage::default(), FakeSource(passage("John 1:1-2")));

    let error = coer.recall(&VerseRef::new("Psalm 23:1")).unwrap_err();
    assert!(matches!(
        error,
        CoerError::Storage(StorageError::NotFound(_))
    ));
}
