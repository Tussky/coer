//! Adapters that keep memories somewhere.
//!
//! Depends on `coer-model` to learn the `Storage` contract. That direction
//! looks backwards until you notice it is the point: the adapter depends on
//! the abstraction, and the abstraction belongs to the core.

pub mod paths;

#[cfg(feature = "json")]
pub mod json;

pub use paths::data_dir;

#[cfg(feature = "json")]
pub use json::JsonStorage;
