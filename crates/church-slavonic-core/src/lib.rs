//! Shared grammatical vocabulary for the Church Slavonic crate families.
//!
//! This is the first slice of the rewrite-plan kernel (docs/REWRITE_PLAN.md,
//! phase 2): the closed grammatical category enums that both the Old Church
//! Slavonic and Synodal families declare independently today. Each enum
//! carries two spelling registries — the long `code()` used by the Synodal
//! data pipeline ("nominative") and the short `abbrev()` used by the OCS
//! pipeline ("nom") — so either family can adopt the shared type without
//! churning its serialized artifacts.

pub mod grammar;
pub mod identity;
pub mod recension;

pub use identity::{IdentityEntry, IdentityRegistry};
pub use recension::Recension;

pub use grammar::{
    AdjectiveForm, Animacy, Case, Comparison, FiniteTense, Gender, Number, NumeralKind,
    ParticipleTense, ParticipleVoice, Person, Voice,
};
