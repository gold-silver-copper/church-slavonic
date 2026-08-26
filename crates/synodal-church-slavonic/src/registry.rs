use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope,
    ActiveParticipleShortFormation, AdjectiveClass, AdjectiveForm, AdjectiveLexeme, Animacy,
    AoristFormation, Aspect, AuthorityRole, BreathingMark, BreathingRule, Case, Comparison,
    ComparisonFormation, Confidence, DeterminerDeclension, DeterminerLexeme, EpistemicRole, Error,
    Evidence, EvidenceId, EvidenceKind, FiniteTense, Gender, GenerationPolicy, GrammarCell,
    ImperativeFormation, ImperfectFormation, InitialPresentation, LetterOccurrence, LexemeId,
    NounAnimacyInventory, NounDeclension, NounLexeme, NounNumberInventory, Number,
    NumeralDeclension, NumeralLexeme, ParticiplePrincipalPart, ParticipleTense, ParticipleVoice,
    PositionalOperation, PositionalParadigm, PositionalReplacement, PositionalRule,
    PronounDeclension, PronounEnvironment, PronounFormSelection, PronounLexeme,
    PronounPostpositive, PronounPrefix, Recension, RecensionMappingId, Result,
    ShortMasculineStemFormation, SourceId, SynodalWord, VerbConjugation, VerbLexeme,
    VerbalNounPrincipalPart, normalize_lookup_accentless, validate_adjective_lexeme,
    validate_determiner_lexeme, validate_numeral_lexeme, validate_pronoun_lexeme,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawLexeme(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawNounRestriction(pub [&'static str; 5]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawPrincipalPart(pub [&'static str; 6]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawExactForm(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAlignment(pub [&'static str; 11]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAbbreviation(pub [&'static str; 13]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAbbreviationFamily(pub [&'static str; 12]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAccent(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAccentParadigm(pub [&'static str; 11]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawPositionalRule(pub [&'static str; 7]);
#[derive(Clone, Copy, Debug)]
// Compiled and unit-tested, but not yet consumed: see the note in
// `resolver::apply_generated_presentation` on the accent/positional ordering.
#[allow(dead_code)]
pub(crate) struct RawPositionalParadigm(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawTransformationRule(pub [&'static str; 6]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawConflict(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawDefectiveInventory(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawIrregularVerbInventory(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawReviewedEvidence(pub [&'static str; 6]);

include!("../generated/registry.rs");

mod lexemes;
mod lookup;
mod types;

pub use types::*;

pub(crate) use lexemes::*;
pub(crate) use lookup::*;
