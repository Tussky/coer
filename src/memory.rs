use serde::{Serialize, Deserialize};
use regex::Regex;
use std::collections::HashMap;

use crate::esv::EsvResponse;

#[derive(Serialize, Deserialize, Debug)]
pub struct Memory {
    pub stats: HashMap<String, String>,
    pub chapter_header: String,
    pub verse_header: String,
    pub verses: HashMap<u8, String>,
}

impl Memory {
    pub fn write_to_json(&self) {
        let json = serde_json::to_string_pretty(self).expect("Serialize failed");
        let path = format!("{}/data/{}.json", env!("CARGO_MANIFEST_DIR"), self.verse_header);
        std::fs::write(path, json).expect("Write failed");
    }
}

impl From<EsvResponse> for Memory {
    fn from(response: EsvResponse) -> Self {
        let sections = response.split_response();
        let verse_regex = Regex::new(r"\[(\d+)\] ([^\[]+) ").expect("Failed to compile verse regex");
        let mut verses = HashMap::new();

        for cap in verse_regex.captures_iter(&sections[2]) {
            verses.insert(cap[1].parse::<u8>().expect("Expected a verse number - could not parse"), cap[2].to_string());
        }

        Memory {
            stats: HashMap::new(),
            chapter_header: sections[1].clone(),
            verse_header: sections[0].clone(),
            verses: verses,
        }
    }
}
