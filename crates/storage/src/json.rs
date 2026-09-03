use std::io;
use std::path::PathBuf;

use coer_model::{Memory, Storage, StorageError, VerseRef};

/// One JSON file per memory, in a directory.
pub struct JsonStorage {
    path: PathBuf,
}

impl JsonStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Path a memory is stored at: "Matthew 1:1-5" -> `<dir>/Matthew1_1-5.json`.
    /// Spaces are dropped and filename-illegal characters replaced with `_`.
    ///
    /// This mangling is lossy, so the filename is only ever a lookup key — the
    /// reference inside the file is the source of truth. See `list`.
    fn path_for(&self, reference: &VerseRef) -> PathBuf {
        let safe: String = reference
            .as_str()
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                other => other,
            })
            .collect();
        self.path.join(format!("{safe}.json"))
    }
}

/// Anything that is not "the file is missing" is a backend failure.
fn backend(e: impl std::error::Error + Send + Sync + 'static) -> StorageError {
    StorageError::Backend(Box::new(e))
}

impl Storage for JsonStorage {
    fn save(&self, memory: &Memory) -> Result<(), StorageError> {
        let json = serde_json::to_string_pretty(memory).map_err(backend)?;
        std::fs::create_dir_all(&self.path).map_err(backend)?;
        std::fs::write(self.path_for(memory.reference()), json).map_err(backend)
    }

    fn load(&self, reference: &VerseRef) -> Result<Memory, StorageError> {
        let json = match std::fs::read_to_string(self.path_for(reference)) {
            Ok(json) => json,
            // A missing file is an ordinary outcome the caller can act on,
            // not a backend fault — so it gets its own variant.
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound(reference.clone()));
            }
            Err(e) => return Err(backend(e)),
        };
        serde_json::from_str(&json).map_err(backend)
    }

    fn list(&self) -> Result<Vec<VerseRef>, StorageError> {
        let entries = match std::fs::read_dir(&self.path) {
            Ok(entries) => entries,
            // Nothing saved yet is an empty list, not an error.
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(backend(e)),
        };

        let mut refs = Vec::new();
        for entry in entries {
            let path = entry.map_err(backend)?.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                // Read the reference out of the file rather than un-mangling the
                // filename, which `path_for` made impossible.
                let json = std::fs::read_to_string(&path).map_err(backend)?;
                let memory: Memory = serde_json::from_str(&json).map_err(backend)?;
                refs.push(memory.passage.reference);
            }
        }
        refs.sort();
        Ok(refs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coer_model::Passage;
    use std::collections::BTreeMap;

    /// A scratch directory of our own, so tests never touch the real data dir.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("coer-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn memory(reference: &str) -> Memory {
        let mut verses = BTreeMap::new();
        verses.insert(1u8, "In the beginning was the Word".to_string());
        Memory::new(Passage {
            reference: VerseRef::new(reference),
            heading: "The Word Became Flesh".to_string(),
            verses,
        })
    }

    #[test]
    fn saves_and_loads_the_same_memory() {
        let store = JsonStorage::new(scratch("round-trip"));
        let original = memory("John 1:1");

        store.save(&original).expect("should save");
        let loaded = store.load(&VerseRef::new("John 1:1")).expect("should load");

        assert_eq!(loaded, original);
        let _ = std::fs::remove_dir_all(scratch("round-trip"));
    }

    #[test]
    fn a_missing_memory_is_not_found_rather_than_a_backend_error() {
        let store = JsonStorage::new(scratch("missing"));
        assert!(matches!(
            store.load(&VerseRef::new("Psalm 23:1")),
            Err(StorageError::NotFound(_))
        ));
    }

    /// Listing must survive `path_for` mangling the reference into a filename —
    /// "John 1:1" becomes "John1_1.json", which cannot be reversed.
    #[test]
    fn list_recovers_references_the_filename_cannot_encode() {
        let dir = scratch("list");
        let store = JsonStorage::new(&dir);

        store.save(&memory("John 1:1")).unwrap();
        store.save(&memory("Matthew 1:1-5")).unwrap();

        assert_eq!(
            store.list().unwrap(),
            vec![VerseRef::new("John 1:1"), VerseRef::new("Matthew 1:1-5")]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn listing_before_anything_is_saved_is_empty_not_an_error() {
        let store = JsonStorage::new(scratch("empty"));
        assert_eq!(store.list().unwrap(), Vec::new());
    }
}
