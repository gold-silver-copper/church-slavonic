//! Pinned sources -> deterministic lookup tables, in four stages:
//!
//! 1. [`args`]/[`pipeline`] — CLI and orchestration (called via `cargo xtask
//!    refresh-data`): filter each source into `data/intermediate/<source>.jsonl`
//!    ([`kaikki`], [`alypy`]), extract, and emit the tables.
//! 2. [`extract`] — reads the intermediates into per-lemma attestations,
//!    subtracts the rule engine cell by cell, and turns the rest into
//!    [`assign::Candidate`]s; this module owns every editorial POLICY (which
//!    forms are admissible, which table is which paradigm, what reserves the
//!    bare key).
//! 3. [`assign`] — numbers each lemma's candidates into deterministic `_<n>`
//!    keys by a pure sort of their emitted forms. No lockfile, no identity, no
//!    carry-forward: the output is a function of the current sources alone.
//! 4. [`bootstrap`]/[`file_generation`] — the single candidate -> PHF emission
//!    path; [`checks`] is the source-driven accuracy harness (`cargo xtask
//!    accuracy`).
//!
//! The division of labor is deliberate: `extract` owns policy (what SHOULD be
//! published) and `assign` owns numbering (a reproducible sort). Neither keeps
//! state between runs — a refresh regenerates everything from the sources.
//!
//! # Sources and recensions
//!
//! Exactly two labelled full-form sources, each tagged with the recension it
//! attests: the Kaikki/Wiktextract Old Church Slavonic dump (`ocs`, unaccented
//! inflection tables) and the Alypy grammar's printed paradigms (`syn`,
//! accented Synodal print). A raw text corpus is not a source.
//!
//! # Table schema (the contract with the `church-slavonic` runtime)
//!
//! One `phf` map per part of speech in `crates/church-slavonic/generated/`
//! (`noun_phf.rs`, `adj_phf.rs`, `verb_phf.rs`, `pronoun_phf.rs`), keyed
//! `"<recension-tag>:<key>"` (`ocs:градъ`, `syn:рабъ_2`), each row a
//! fixed-arity array of cell strings in the order [`cells`] documents (nouns
//! 21, adjectives 126, verbs 38, the personal pronoun 90). An empty string
//! means "not attested — fall back to the rule". Only lemmas with at least
//! one cell the rules do not predict get a row.

use std::error::Error;

pub mod alypy;
pub mod args;
pub mod assign;
pub mod bootstrap;
pub mod cells;
pub mod checks;
pub mod extract;
pub mod file_generation;
pub mod kaikki;
pub mod pipeline;

pub use args::Config;

pub fn run_from_env() -> Result<(), Box<dyn Error>> {
    let config = args::parse_args()?;
    pipeline::run(&config)
}
