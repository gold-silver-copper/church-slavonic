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

pub use grammar::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, AoristFormation, Case, ClosedClassCell,
    FiniteTense, FiniteVerbCell, Gender, ImperativeCell, ImperativeFormation, ImperfectFormation,
    ImperfectVariantPolicy, LParticipleCell, NounCell, NounClass, Number, NumberRestriction,
    PartOfSpeech, ParticipleCell, ParticipleKind, PastActiveParticipleFormation,
    PastPassiveParticipleFormation, Person, PresentActiveParticipleFormation,
    PresentPassiveParticipleFormation, VerbAspect, VerbClass,
};
pub use result::{
    FormAnalysis, FormSet, FormSource, FormVariant, InflectionError, InflectionWarning,
    LexemeSummary, MetadataEvidence, MetadataField, MetadataProvenance,
};
pub use trace::{PredictedForm, RuleId, RuleStep};
