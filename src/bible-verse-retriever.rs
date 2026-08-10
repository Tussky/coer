use secrecy::{SecretString};
use reqwest;
use tokio;


pub struct BibleAPIConnection {
    pub api_key: SecretString,
    pub url_url: string,
}

impl BibleAPIConnection {
    #[tokio::main]
    pub async fn request_verse(&self, verse_query: &str) {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.esv.org/v3/passage/text/")
            .header("Authorization", format!("Token {}", self.api_key.expose_secret()))
            .query(&[("q", verse_query)])
            .send()
            .await
            .expect("Failed to fetch");

        let body = response.text().await.expect("Failed to read body");
        println!("{}", body);
    }
}
