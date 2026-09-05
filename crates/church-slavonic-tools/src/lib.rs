//! Tooling for the church-slavonic lexicon: source parsers, the importer,
//! the evaluation harness and the Bible treebank (`cargo xtask`).

pub mod eval;
pub mod legacy;
pub mod sources;
pub mod treebank;

use std::path::PathBuf;

/// The workspace root (this crate lives at `crates/church-slavonic-tools`).
pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
