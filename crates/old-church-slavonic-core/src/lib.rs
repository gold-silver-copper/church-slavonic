//! Pure Old Church Slavonic morphology rules and shared grammar types.
//!
//! This crate has no bundled lexicon and performs no I/O. Use the
//! `old-church-slavonic` facade for dictionary-backed inflection.

#![forbid(unsafe_code)]

pub mod adjective;
pub mod grammar;
pub mod noun;
pub mod orthography;
pub mod pronoun;
pub mod result;
pub mod trace;
pub mod verb;

pub use grammar::*;
pub use result::*;
pub use trace::*;
