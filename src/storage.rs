use std::path::{Path, PathBuf};

use crate::memory::Memory;

pub trait Storage {
    fn store(&self, to_save: &Memory) -> Result<(), std::io::Error>;
    fn read(&self, file_loc: String) -> Result<Memory, Box<dyn std::error::Error>>;
}

pub struct JsonStorage {
    pub path: PathBuf,
}

pub struct SQLStorage {
    pub path: PathBuf,
}

impl JsonStorage {
    /// Path a memory is stored at: "Matthew 1:1-5" -> `<dir>/Matthew1_1-5.json`.
    /// Spaces are dropped and filename-illegal characters replaced with `_`.
    fn path_for(&self, key: &str) -> PathBuf {
        let safe: String = key
            .chars()
            .filter(|c| !c.is_whitespace())
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                other => other,
            })
            .collect();
        self.path.join(format!("{safe}.json"))
    }
}

impl Storage for JsonStorage {
    fn store(&self, to_save: &Memory) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(to_save).map_err(std::io::Error::other)?;
        std::fs::create_dir_all(&self.path)?;
        std::fs::write(self.path_for(&to_save.verse_header), json)
    }

    fn read(&self, file_loc: String) -> Result<Memory, Box<dyn std::error::Error>> {
        // Accept either a bare key ("Matthew 1:1-5") or a full path to a stored file.
        let path = if Path::new(&file_loc).is_file() {
            PathBuf::from(&file_loc)
        } else {
            self.path_for(&file_loc)
        };

        let json = std::fs::read_to_string(path)?;
        let memory = serde_json::from_str(&json)?;
        Ok(memory)
    }
}

// impl Storage for SQLStorage {
// fn store(&self, to_save: &Memory) -> Result<(), std::io::Error> {
//     todo!()
// }
// fn read(&self, file_loc: String) -> Result<Memory, Box<dyn std::error::Error>> {
//     todo!()
// }
// }
