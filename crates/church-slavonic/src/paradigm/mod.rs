//! Letter classes: per class and cell, an ending and a stem selector.
//! The tables live in `lexicon/classes/*.toml`; this module reads them
//! and applies the stem alternations. Part 1 fills the noun tables,
//! Part 3 the rest.

pub mod noun;
