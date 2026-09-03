use directories::ProjectDirs;
use std::path::PathBuf;

pub const DATA_DIR_ENV_VAR: &str = "COER_DATA_DIR";

/// Directory for user data we must not lose (memorisation progress, cached verses).
///
/// Resolution order:
/// 1. `COER_DATA_DIR` override, for tests and local development.
/// 2. The platform data dir, e.g. `~/.local/share/coer` on Linux (honours `XDG_DATA_HOME`).
///
/// Lives in the storage crate, not a shared `config` module: it is a fact about
/// the filesystem backend, and a database backend has no use for it. Shared
/// config modules are where layering quietly rots.
pub fn data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(DATA_DIR_ENV_VAR)
        && !dir.trim().is_empty()
    {
        return PathBuf::from(dir);
    }

    ProjectDirs::from("", "", "coer")
        .expect("Could not determine a home directory for storing coer data")
        .data_dir()
        .to_path_buf()
}
