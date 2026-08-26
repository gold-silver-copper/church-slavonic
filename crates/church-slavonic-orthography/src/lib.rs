//! Recension-aware Church Slavonic orthography (docs/REWRITE_PLAN.md, target
//! layout). The recension is a parameter of this crate's module tree, not a
//! crate family: [`glagolitic`] carries the Old Church Slavonic
//! Cyrillic/Glagolitic transliteration engine, [`synodal`] carries the Synodal
//! liturgical lookup normalization and positional presentation rules, and
//! [`text`] carries the recension-agnostic primitives both share.
//!
//! This crate depends only on `church-slavonic-core` and
//! `unicode-normalization`; it never depends on either family core. The family
//! cores re-export from here so their public APIs are unchanged.

pub mod glagolitic;
pub mod synodal;
pub mod text;
