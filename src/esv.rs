use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::fmt;

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
    pub fn split_response(&self) -> Vec<String> {
        let sections: Vec<String> = self
            .passages
            .first()
            .expect("Could not find first element in ESVResponse")
            .split("\n\n")
            .map(|s| s.to_string())
            .collect();

        let mut verses = String::new();
        let mut i = 2;
        while sections[i] != "Footnotes" {
            verses.push_str(&sections[i]);
            i += 1;
        }
        vec![sections[0].clone(), sections[1].clone(), verses]
    }
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
            .header(
                "Authorization",
                format!("Token {}", self.api_key.expose_secret()),
            )
            .query(&[("q", verse_query)])
            .send()
            .await
            .expect("Failed to fetch");

        let body: EsvResponse = response.json().await.expect("Failed to read body");
        return body;
    }
}
