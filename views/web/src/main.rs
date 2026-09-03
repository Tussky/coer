//! Not a server yet — but it builds the *same* `Coer` the CLI does, from the
//! same controller, and that is the thing worth proving. When this grows an
//! axum server and a `SqliteStorage`, the only line that changes is the one
//! naming the backend; nothing in `crates/` has to notice.

use std::error::Error;

use coer_controller::Coer;
use coer_esv::EsvClient;
use coer_storage::{JsonStorage, data_dir};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    // Swap JsonStorage for SqliteStorage here and everything above keeps working.
    let coer = Coer::new(JsonStorage::new(data_dir()), EsvClient::from_env()?);

    match coer.memories()?.as_slice() {
        [] => println!("No memories stored yet."),
        refs => {
            println!("{} memories stored:", refs.len());
            for reference in refs {
                println!("  {reference}");
            }
        }
    }

    Ok(())
}
