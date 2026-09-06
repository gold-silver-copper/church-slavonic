//! Tooling for the church-slavonic lexicon: source parsers, the importer,
//! the evaluation harness and the Bible treebank (`cargo xtask`).

// a test asserts with unwrap; the workspace denies it in the code it ships
#![cfg_attr(test, allow(clippy::unwrap_used))]

pub mod census;
pub mod eval;
pub mod import;
pub mod sources;
pub mod tagger;
pub mod treebank;

use std::path::PathBuf;

/// The workspace root (this crate lives at `crates/church-slavonic-tools`).
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
