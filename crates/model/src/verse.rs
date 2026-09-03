use serde::{Deserialize, Serialize};
use std::fmt;

/// How a passage is named: "Matthew 1:1-5".
///
/// A newtype rather than a bare `String` so a reference cannot be passed
/// where some other string was meant. It stays opaque on purpose — nothing
/// yet needs to know the book or chapter separately.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VerseRef(String);

impl VerseRef {
    pub fn new(reference: impl Into<String>) -> Self {
        Self(reference.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for VerseRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for VerseRef {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for VerseRef {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
