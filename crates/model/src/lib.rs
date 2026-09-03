//! The core of coer: what a memory *is*, and the contracts the rest of the
//! app must meet to store and fetch one.
//!
//! This crate names no adapter and performs no I/O. Every arrow in the
//! workspace points here; this one points nowhere.

pub mod memory;
pub mod ports;
pub mod verse;

pub use memory::{Memory, Passage};
pub use ports::{SourceError, Storage, StorageError, VerseSource};
pub use verse::VerseRef;
