//! Offline, deterministic source ingestion for Church Slavonic (merge phase 6,
//! `docs/UNIFIED_DATA.md`).
//!
//! One extractor crate, two recension pipelines as modules:
//!
//! - [`ocs`] — the Kaikki/Wiktextract Old Church Slavonic pipeline
//!   (`data/ocs/extracted`, `data/ocs/dictionary`, the facade's generated Rust);
//! - [`synodal`] — the reviews/evidence/registry pipeline over `data/synodal`
//!   (candidate ingestion from `references/`, evidence review, the Synodal
//!   generated registries).
//!
//! [`shared`] owns the plumbing both pipelines genuinely share: source
//! checksumming, atomic artifact installation, and TSV field hygiene. The
//! recension-specific stages (Kaikki table parsing vs Ponomar/Alypy evidence
//! review) stay separate by nature; see `docs/UNIFIED_DATA.md` for the survey.
//!
//! This crate is never a runtime dependency; the published crates consume only
//! its committed outputs.
#![forbid(unsafe_code)]

pub mod ocs;
pub mod shared;
pub mod synodal;
