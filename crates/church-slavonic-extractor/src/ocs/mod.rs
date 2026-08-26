//! The Kaikki/Wiktextract Old Church Slavonic pipeline (recension: OCS).
//!
//! Reads a local Kaikki JSONL dump and owns `data/ocs/extracted/*.tsv`,
//! `data/ocs/dictionary/*.json`, the extraction/dictionary coverage reports,
//! and the facade's generated Rust. See `docs/DATA_PIPELINE.md`.

pub mod emit;
pub mod extract;
pub mod normalize;
pub mod report;
pub mod schema;
pub mod semantics;
pub mod validate;
pub mod verb_metadata;
