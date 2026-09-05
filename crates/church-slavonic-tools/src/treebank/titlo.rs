//! The titlo layer lives in the library (`church_slavonic::titlo`); the
//! treebank's `(abbr "гдⷭ҇" X)` wrapper renders the child in full and
//! abbreviates it under the matching row.

pub use church_slavonic::titlo::{Row, abbreviate, rows, skeleton};
