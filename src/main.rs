mod config;
use std::fmt;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use regex::Regex;
use std::collections::HashMap;

pub struct Memory {
    pub stats: Vec<String>,
    pub chapter_header: String,
    pub verses: HashMap<u8, String>,
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
        let verse_regex = Regex::new(r"\[(\d+)\] ([^\[]+) \[").expect("Failed to compile verse regex");
        let mut verses = HashMap::new();

        for cap in verse_regex.captures_iter(self.passages.first().expect("Failed to find first passage")) {
            verses.insert(cap[1].parse::<u8>().expect("Expected a verse number - could not parse"), cap[2].to_string());
        }
        verses
    }


    pub fn get_chapter_header(&self) -> String {
        let heading_regex = Regex::new(r"^[^\n]+\n\n([^\n]+)\n\n").expect("Failed to compile Regex");

        match heading_regex.captures(self.passages.first().expect("Failed to get first element in chapter_header capture")) {
            Some(caps) => {return caps[1].to_string()}
            None => {panic!("Capture cannot be empty for chapter_header");}
        }
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

    let wanted_verses: EsvResponse = connection.request_verse("Matthew 1:1-30").await;
    let header = wanted_verses.get_chapter_header();
    let verses = wanted_verses.get_verses();
    dbg!(verses);
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
