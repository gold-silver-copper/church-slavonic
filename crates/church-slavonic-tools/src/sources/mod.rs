//! The pinned sources' parsers: each turns a downloaded artifact into a
//! filtered intermediate (`data/intermediate/<source>.jsonl`) and reads
//! that back into typed records. Parsers only — what becomes a lexicon
//! line is the importer's decision (`crate::import`).

pub mod alypy;
pub mod kaikki;
pub mod polyakov;
pub mod ruwiktionary;
pub mod ud;
