//! Pure Old Church Slavonic morphology rules and shared grammar types.
//!
//! This crate has no bundled lexicon and performs no I/O. Use the
//! `old-church-slavonic` facade for dictionary-backed inflection.

#![forbid(unsafe_code)]

pub mod adjective;
pub mod copula;
pub mod grammar;
pub mod noun;
pub mod orthography;
pub mod phrase;
pub mod pronoun;
pub mod result;
pub mod trace;
pub mod verb;

pub use copula::{CopulaSeries, CopulaVariant, CopulaVariantStatus};
pub use grammar::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, AoristFormation, Case, ClosedClassCell,
    ComparativeFormation, FiniteTense, FiniteVerbCell, Gender, GenderedCell, ImperativeCell,
    ImperativeFormation, ImperfectFormation, ImperfectVariantPolicy, LParticipleCell, NounCell,
    NounClass, Number, NumberRestriction, PartOfSpeech, ParticipleCell, ParticipleKind,
    PastActiveParticipleFormation, PastPassiveParticipleFormation, Person, PersonalPronounCell,
    PresentActiveParticipleFormation, PresentPassiveParticipleFormation, RequestedCell,
    UngenderedCell, VerbAspect, VerbClass,
};
pub use orthography::{Lemma, Script};
pub use phrase::{
    AnalyticConstruction, ConditionalAuxiliary, FutureInfinitiveAuxiliary, FutureReferenceTense,
    PassiveAuxiliary, PhraseOrder, PhraseRole, PhraseToken, PluperfectAuxiliary, RealizedPhrase,
};
pub use pronoun::{
    AnaphoricEnvironment, PersonalPronounIdentity, PronounFormSelection, PronounVariant,
    PronounVariantStatus,
};
pub use result::{
    FormAnalysis, FormSet, FormSource, FormVariant, InflectionError, InflectionWarning,
    LexemeSummary, MetadataEvidence, MetadataField, MetadataProvenance, VariantPolicy,
    VariantSelectionError,
};
pub use trace::{PredictedForm, RuleId, RuleStep};
