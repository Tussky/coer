mod config;
use std::fmt;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use regex::Regex;


pub struct Memory {
    pub verse: String,
    pub stats: Vec<String>,
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
    pub fn format_into_string(&self) -> String {
        for passage in &self.passages {
            println!("{:?}", passage);
        }
        "hi".to_string()
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

    let wanted_verses = connection.request_verse("Genesis 1:1-10").await;
    println!("{:#?}", wanted_verses);
    EsvResponse::format_into_string();
}

pub struct BibleAPIConnection {
    pub api_key: SecretString,
    pub url: String,
}

impl BibleAPIConnection {
    pub async fn request_verse(&self, verse_query: &str) -> Vec<String> {
        let client = reqwest::Client::new();
        let response = client
            .get(&self.url)
            .header("Authorization", format!("Token {}", self.api_key.expose_secret()))
            .query(&[("q", verse_query)])
            .send()
            .await
            .expect("Failed to fetch");

        let body: EsvResponse = response.json().await.expect("Failed to read body");
        return body.passages
    }

}
