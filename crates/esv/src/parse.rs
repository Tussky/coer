use std::collections::BTreeMap;
use std::sync::LazyLock;

use coer_model::{Passage, SourceError, VerseRef};
use regex::Regex;

use crate::wire::EsvResponse;

/// Verse markers in ESV plain text: `[1] In the beginning...`.
///
/// Compiled once. The pattern is a fact about the program, so a failure to
/// compile it is a bug in this file, not a runtime condition to handle.
static VERSE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[(\d+)\]\s*([^\[]+)").expect("verse regex is malformed"));

/// ESV returns one blob of text laid out as
///
/// ```text
/// Matthew 1:1–5            <- the reference, repeated
///
/// The Genealogy of Jesus   <- section heading, absent for some passages
///
///   [1] The book ... [2] ...
///
/// Footnotes                <- absent when the passage has none
/// ...
///  (ESV)
/// ```
///
/// Every one of those parts is optional in practice, so this walks the sections
/// and classifies them instead of indexing by position.
pub(crate) fn passage_from(response: EsvResponse) -> Result<Passage, SourceError> {
    let body = response
        .passages
        .first()
        .ok_or_else(|| SourceError::NotFound(response.canonical.clone()))?;

    let mut heading = String::new();
    let mut verse_text = String::new();

    for section in body.split("\n\n").map(str::trim).filter(|s| !s.is_empty()) {
        if section == "Footnotes" {
            break;
        }
        if section == response.canonical || section == "(ESV)" {
            continue;
        }
        if VERSE.is_match(section) {
            verse_text.push_str(section);
            verse_text.push(' ');
        } else if heading.is_empty() {
            heading = section.to_string();
        }
    }

    let mut verses = BTreeMap::new();
    for capture in VERSE.captures_iter(&verse_text) {
        let number: u8 = capture[1]
            .parse()
            .map_err(|_| SourceError::Malformed(format!("verse number {:?}", &capture[1])))?;
        verses.insert(number, capture[2].trim().to_string());
    }

    if verses.is_empty() {
        return Err(SourceError::Malformed(format!(
            "no verses found in the response for {:?}",
            response.canonical
        )));
    }

    Ok(Passage {
        reference: VerseRef::new(response.canonical),
        heading,
        verses,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(canonical: &str, body: &str) -> EsvResponse {
        EsvResponse {
            canonical: canonical.to_string(),
            passages: vec![body.to_string()],
        }
    }

    #[test]
    fn parses_reference_heading_and_verses() {
        let passage = passage_from(response(
            "John 1:1-2",
            "John 1:1-2\n\nThe Word Became Flesh\n\n  \
             [1] In the beginning was the Word. [2] He was with God.\n\n\
             Footnotes\n\n[1] 1:1 Or something\n\n (ESV)",
        ))
        .expect("should parse");

        assert_eq!(passage.reference, VerseRef::new("John 1:1-2"));
        assert_eq!(passage.heading, "The Word Became Flesh");
        assert_eq!(passage.verses.len(), 2);
        assert_eq!(passage.verses[&1], "In the beginning was the Word.");
        assert_eq!(passage.verses[&2], "He was with God.");
    }

    /// The old positional parser looped until it found the literal "Footnotes"
    /// and indexed past the end when there was none. ESV omits that section
    /// whenever a passage has no footnotes.
    #[test]
    fn survives_a_passage_with_no_footnotes() {
        let passage = passage_from(response(
            "Psalm 117:1-2",
            "Psalm 117:1-2\n\nThe LORD's Faithfulness\n\n  \
             [1] Praise the LORD. [2] For great is his love.\n\n (ESV)",
        ))
        .expect("should parse");

        assert_eq!(passage.verses.len(), 2);
        assert_eq!(passage.heading, "The LORD's Faithfulness");
    }

    #[test]
    fn survives_a_passage_with_no_heading() {
        let passage = passage_from(response(
            "John 11:35",
            "John 11:35\n\n  [35] Jesus wept.\n\n (ESV)",
        ))
        .expect("should parse");

        assert!(passage.heading.is_empty());
        assert_eq!(passage.verses[&35], "Jesus wept.");
    }

    /// The last verse used to need a trailing space to match.
    #[test]
    fn matches_the_final_verse_without_trailing_whitespace() {
        let passage = passage_from(response("John 11:35", "John 11:35\n\n  [35] Jesus wept."))
            .expect("should parse");
        assert_eq!(passage.verses[&35], "Jesus wept.");
    }

    #[test]
    fn verses_come_back_in_order() {
        let passage = passage_from(response(
            "Psalm 1:1-3",
            "Psalm 1:1-3\n\n  [1] one [2] two [3] three",
        ))
        .expect("should parse");

        let numbers: Vec<u8> = passage.verses.keys().copied().collect();
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn an_empty_response_is_an_error_not_a_panic() {
        let empty = EsvResponse {
            canonical: "Nowhere 1:1".to_string(),
            passages: vec![],
        };
        assert!(matches!(passage_from(empty), Err(SourceError::NotFound(_))));
    }
}
