use directories::ProjectDirs;
use std::path::PathBuf;

pub const AUTH_TOKEN_ENV_VAR: &str = "ESV_API_KEY";
pub const DATA_DIR_ENV_VAR: &str = "COER_DATA_DIR";

/// Directory for user data we must not lose (memorisation progress, cached verses).
///
/// Resolution order:
/// 1. `COER_DATA_DIR` override, for tests and local development.
/// 2. The platform data dir, e.g. `~/.local/share/coer` on Linux (honours `XDG_DATA_HOME`).
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(DATA_DIR_ENV_VAR) {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }

    ProjectDirs::from("", "", "coer")
        .expect("Could not determine a home directory for storing coer data")
        .data_dir()
        .to_path_buf()
}
