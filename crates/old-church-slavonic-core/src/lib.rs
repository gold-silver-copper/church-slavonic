//! Pure Old Church Slavonic morphology rules and shared grammar types.
//!
//! This crate has no bundled lexicon and performs no I/O. Use the
//! `old-church-slavonic` facade for dictionary-backed inflection.

#![forbid(unsafe_code)]

pub mod accent;
pub mod adjective;
pub mod copula;
pub mod determiner;
pub mod grammar;
pub mod impersonal;
pub mod irregular_verb;
pub mod noun;
pub mod numeral;
pub mod orthography;
pub mod phrase;
pub mod pronoun;
pub mod result;
pub mod trace;
pub mod twofold_noun;
pub mod unique_noun;
pub mod unique_verb;
pub mod verb;

pub use accent::{
    AccentEvidence, AccentParadigm, AccentPlacement, AccentReconstructionStatus, AccentRule,
    AccentScope, ReconstructedAccent,
};
pub use adjective::LongOnlyAdjectiveIdentity;
pub use copula::{CopulaSeries, CopulaVariant, CopulaVariantStatus};
pub use determiner::{DeterminerDeclension, DeterminerIdentity, DeterminerLexeme};
pub use grammar::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, AoristFormation, Case, ClosedClassCell,
    CollectiveNumeralCell, ComparativeFormation, CompoundCardinalCell, DeterminerCell,
    DistributiveCardinalCell, FiniteTense, FiniteVerbCell, Gender, GenderedCell, ImperativeCell,
    ImperativeFormation, ImperfectFormation, ImperfectVariantPolicy, LParticipleCell, NounCell,
    NounClass, Number, NumberRestriction, NumeralCell, PartOfSpeech, ParticipleCell,
    ParticipleKind, PastActiveParticipleFormation, PastPassiveParticipleFormation, Person,
    PersonalPronounCell, PresentActiveParticipleFormation, PresentPassiveParticipleFormation,
    RequestedCell, UngenderedCell, VerbAspect, VerbClass, VerbDefectKind, VerbMorphologyCell,
    VerbMorphologySystem,
};
pub use impersonal::{ImpersonalVerbIdentity, ImpersonalVerbStatus};
pub use irregular_verb::{IrregularVerbAnalysis, IrregularVerbFamilyMember, IrregularVerbGroup};
pub use numeral::{
    CardinalCompositionOptions, CardinalMagnitudeIdentity, CardinalNumeralIdentity,
    CardinalPhraseAnalysis, CollectiveNumeralDeclension, CollectiveNumeralIdentity,
    DistributiveCardinalAnalysis, FractionalNumeralDeclension, FractionalNumeralIdentity,
    IndefiniteNumeralIdentity, MAX_COMPOUND_ORDINAL_VALUE, MIN_COMPOUND_ORDINAL_VALUE,
    NumeralGovernment, NumeralVariant, NumeralVariantStatus, OrdinalComposition,
    OrdinalNumeralIdentity, OrdinalPhraseAnalysis, RealizedCardinal, RealizedDistributiveCardinal,
    RealizedOrdinal,
};
pub use orthography::{
    GlagoliticProfile, Lemma, Script, TransliteratedForm, TransliterationDirection,
    TransliterationFidelity, TransliterationLoss, TransliterationLossKind,
    TransliterationLossPolicy, realize_glagolitic, transliterate_glagolitic_to_cyrillic,
};
pub use phrase::{
    AnalyticConstruction, ConditionalAuxiliary, FutureInfinitiveAuxiliary, FutureReferenceTense,
    PassiveAuxiliary, PhraseOrder, PhraseRole, PhraseToken, PluperfectAuxiliary, RealizedPhrase,
};
pub use pronoun::{
    AnaphoricEnvironment, DirectToTreatment, InterrogativePronounIdentity,
    IrregularAgreeingIdentity, PersonalPronounIdentity, PronominalDeclension, PronominalFamilySpec,
    PronominalLexeme, PronominalPostpositive, PronominalPrefix, PronounFormSelection,
    PronounVariant, PronounVariantStatus, StandardPronominalIdentity,
};
pub use result::{
    FormAnalysis, FormSet, FormSource, FormVariant, InflectionError, InflectionWarning,
    LexemeSummary, MetadataEvidence, MetadataField, MetadataProvenance, VariantPolicy,
    VariantSelectionError,
};
pub use trace::{PredictedForm, RuleId, RuleStep};
pub use twofold_noun::TwofoldNounFamilyMember;
pub use unique_noun::{
    UniqueNounFamilyMember, UniqueNounProfile, UniqueNounVariant, UniqueNounVariantStatus,
};
pub use unique_verb::{UniqueVerbFamilyMember, UniqueVerbIdentity, UniqueVerbProfileKind};
