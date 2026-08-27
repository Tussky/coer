mod config;
mod esv;
mod memory;
mod storage;
use secrecy::SecretString;

use crate::esv::{BibleAPIConnection, EsvResponse};
use crate::memory::Memory;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let storage_dir: String =
        std::env::var("COER_DATA_DIR").expect("Unable to find storage location");

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
