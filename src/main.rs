mod config;
mod esv;
mod memory;
mod storage;
use secrecy::SecretString;

use crate::esv::{BibleAPIConnection, EsvResponse};
use crate::memory::Memory;
use crate::storage::{JsonStorage, Storage};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let storage = JsonStorage {
        path: config::data_dir(),
    };

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
    storage.store(&test_memory).expect("Failed to store memory");

    let loaded = storage
        .read(test_memory.verse_header.clone())
        .expect("Failed to read memory back");
    dbg!(loaded);
}
