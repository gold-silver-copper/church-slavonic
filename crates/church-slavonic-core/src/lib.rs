//! The dependency-free REGULAR fallback / prediction engine for Church
//! Slavonic inflection.
//!
//! `church-slavonic-core` knows nothing about the source dumps or lookup
//! tables: it encodes a compact approximation of productive Church Slavonic
//! morphology — ending tables per declension and conjugation class, selected
//! by inspecting the lemma's ending — in both recensions the crate family
//! serves (Old Church Slavonic and the Synodal print). It is intentionally
//! small and is not a guarantee of correct inflection for arbitrary
//! out-of-vocabulary words; the table-backed `church-slavonic` crate is the
//! correctness-oriented public API.
//!
//! # The contract with the `church-slavonic` crate (do not break casually)
//!
//! These rules serve two roles at once:
//! 1. the runtime FALLBACK for any word the generated tables don't list;
//! 2. the extractor's PREDICTION: a source attestation equal to the rule
//!    output is dropped at table-generation time (the rule will produce it),
//!    and its presence reserves the bare key for the rule engine.
//!
//! Consequently, changing any rule here changes what counts as "irregular" and
//! REQUIRES regenerating the tables (`cargo xtask refresh-data`). Two dump-free
//! tests partially guard the drift — the `church-slavonic` crate's
//! `rule_table_sync` test and this crate's `regular_rules_golden` test — but
//! `cargo xtask accuracy` (with the sources) is the authoritative check, and
//! also measures rule quality against every attested form.
//!
//! Inputs are lowercase lemmas in the requested recension's own canonical
//! spelling (OCS: `ꙑ`, `оу`, `ѫ`, `ѥ`, `ꙗ`, unaccented; Synodal: `ы`, `ꙋ`,
//! `ѧ`, `е`, ACCENTED — the citation form's stress is the input the accent
//! rule reads, see the `accent` module); every rule takes a `&Recension` and
//! answers in that recension's spelling. Case handling and the printed-form
//! realisation ([`orthography::realise`]) are the `church-slavonic` crate's
//! responsibility, applied on output.

mod accent;
mod adj;
pub mod grammar;
mod noun;
mod participle;
pub mod orthography;
mod pronoun;
pub mod sense_key;
mod utils;
pub mod verb;
pub use crate::grammar::*;

/// Namespace struct for the rule engine — all functionality is associated
/// functions (`ChurchSlavonicCore::noun`, `::verb`, `::adj`, `::pronoun`,
/// ...); there is no state to construct.
pub struct ChurchSlavonicCore {}
