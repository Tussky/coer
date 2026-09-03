use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::verse::VerseRef;

/// Scripture as it came back from a source, already parsed.
///
/// Note there is no `EsvResponse` in sight. A `Passage` is the shape *coer*
/// wants; meeting it is each source's job. That is what lets a second source
/// (an offline Bible, a different translation) exist without touching this file.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Passage {
    pub reference: VerseRef,
    /// The section heading a translation prints above the text,
    /// e.g. "The Genealogy of Jesus Christ". Empty when there is none.
    pub heading: String,
    /// Verse number -> text. A `BTreeMap`, not a `HashMap`, so iteration is in
    /// verse order — a view rendering these needs 1, 2, 3, not whatever order
    /// hashing happens to produce.
    pub verses: BTreeMap<u8, String>,
}

impl Passage {
    /// The verses joined back into one block of prose, in order.
    pub fn text(&self) -> String {
        self.verses
            .values()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A passage plus how the memorisation of it is going.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Memory {
    pub passage: Passage,
    pub stats: HashMap<String, String>,
}

impl Memory {
    /// A passage newly taken on, with no progress recorded yet.
    pub fn new(passage: Passage) -> Self {
        Self {
            passage,
            stats: HashMap::new(),
        }
    }

    pub fn reference(&self) -> &VerseRef {
        &self.passage.reference
    }
}

impl From<Passage> for Memory {
    fn from(passage: Passage) -> Self {
        Self::new(passage)
    }
}
