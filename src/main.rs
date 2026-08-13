mod config;
use std::fmt;
use secrecy::{ExposeSecret, SecretString};
use serde::{Serialize, Deserialize};
use regex::Regex;
use std::collections::HashMap;

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

#[derive(Deserialize, Debug)]
pub struct EsvResponse {
    pub canonical: String,
    pub passages: Vec<String>,
    // Note: we're ignoring PassageMeta, which has info such as chapter start/end, prev + next verse
}

impl fmt::Display for EsvResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.canonical)?;
        write!(f, "{}", self.passages.join("\n"))
    }
}

impl EsvResponse {
    pub fn get_verses(&self) -> HashMap<u8, String> {
        let verse_regex = Regex::new(r"\[(\d+)\] ([^\[]+) ").expect("Failed to compile verse regex");
        let mut verses = HashMap::new();

        for cap in verse_regex.captures_iter(self.passages.first().expect("Failed to find first passage")) {
            verses.insert(cap[1].parse::<u8>().expect("Expected a verse number - could not parse"), cap[2].to_string());
        }
        verses
    }

    pub fn split_response(&self) -> Vec<String> {
        let sections: Vec<String>  = self.passages.first().expect("Could not find first element in ESVResponse").split("\n\n").map(|s| s.to_string()).collect();

        let mut verses = String::new();
        let mut i = 2;
        while sections[i] != "Footnotes" {
            verses.push_str(&sections[i]);
            i += 1;
        }
        vec![sections[0].clone(), sections[1].clone(), verses]
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let api_key = SecretString::from(
        std::env::var(config::AUTH_TOKEN_ENV_VAR)
            .unwrap_or_else(|_| panic!("{} must be set", config::AUTH_TOKEN_ENV_VAR)),
    );

    let connection = BibleAPIConnection {
        api_key,
        url: "https://api.esv.org/v3/passage/text/".to_string(),
    };

    let query: EsvResponse = connection.request_verse("Matthew 1:1-5").await;
    let test_memory: Memory = query.into();
    test_memory.write_to_json();
}

pub struct BibleAPIConnection {
    pub api_key: SecretString,
    pub url: String,
}

impl BibleAPIConnection {
    pub async fn request_verse(&self, verse_query: &str) -> EsvResponse {
        let client = reqwest::Client::new();
        let response = client
            .get(&self.url)
            .header("Authorization", format!("Token {}", self.api_key.expose_secret()))
            .query(&[("q", verse_query)])
            .send()
            .await
            .expect("Failed to fetch");

        let body: EsvResponse = response.json().await.expect("Failed to read body");
        return body;
    }
}
