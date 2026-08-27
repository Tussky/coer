use crate::memory::Memory;

pub trait Storage {
    fn store(&self, to_save: &Memory) -> Result<(), std::io::Error>;
    fn read(&self, file_loc: String) -> Result<Memory, Box<dyn std::error::Error>>;
}

#[derive(serde::Serialize)]
pub struct JsonStorage {
    pub path: String,
}

#[derive(serde::Serialize)]
pub struct SQLStorage {
    pub path: String,
}

impl Storage for JsonStorage {
    fn store(&self, to_save: &Memory) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self).expect("Serialize failed");
        let path = format!(
            "{}/data/{}.json",
            env!("CARGO_MANIFEST_DIR"),
            to_save.verse_header
        );
        std::fs::write(path, json).expect("Write failed");
        Ok(())
    }

    fn read(&self, file_loc: String) -> Result<Memory, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(file_loc)?;
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
