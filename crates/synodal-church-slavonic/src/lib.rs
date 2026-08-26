#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod abbreviation;
mod handles;
mod inflector;
mod kernel;
pub mod numeral_phrases;
mod paradigm;
pub mod phrases;
mod provider;
mod registry;
mod resolver;
mod spec;

pub use abbreviation::Abbreviation;
pub use handles::{Adjective, Capabilities, Determiner, Noun, Numeral, Participle, Pronoun, Verb};
pub use inflector::{Inflector, InflectorBuilder};
pub use numeral_phrases::{
    CardinalPhraseAnalysis, CompoundNumeralCell, NumeralComposition, NumeralGovernment,
    NumeralNounPosition, OrdinalPhraseAnalysis, RealizedCardinal, RealizedOrdinal, fraction,
    fractional_cardinal_parts, fractional_half_tenth_parts, fractional_ordinal_parts,
    multiplicative_krat, repeated_distributive,
};
pub use paradigm::{Paradigm, ParadigmIdentity, ParadigmRow, ParadigmStatus};
pub use provider::{
    BatchLexeme, BatchRequest, BatchResult, BatchRow, InMemoryLexemeProvider, LexemeProvider,
    Lexicon, ProviderLexeme, StaticLexemeProvider,
};
pub use registry::{
    AccentParadigmSummary, AccentSummary, AlignmentSummary, ExactFormSummary,
    IrregularOverrideSummary, IrregularVerbInventorySummary, LexemeSummary, LexicalMetadataSummary,
    NounRestrictionSummary, PartOfSpeech, PositionalRuleSummary, PrincipalPartSummary,
    RecensionConflictSummary, TransformationRuleSummary,
};
pub use spec::{
    AdjectiveSpec, DefectKind, DefectiveCell, DeterminerSpec, LexemeSpec, NounSpec, NumeralSpec,
    PronounSpec, SpecificationSource, SpecifiedForm, VerbSpec, VerbSpecBuilder,
};
pub use synodal_church_slavonic_core as core;
pub use synodal_church_slavonic_core::{
    AccentEnclitic, AccentEnvironment, AccentMark, AccentParadigm, AccentPlacement, AccentRule,
    AccentScope, ActiveParticipleShortFormation, AdjectiveClass, AoristFormation, Aspect,
    AuthorityRole, BreathingMark, BreathingRule, ComparisonFormation, EncliticParticle,
    EpistemicRole, Evidence, EvidenceId, EvidenceKind, ImperativeFormation, ImperfectFormation,
    NounDeclension, NounNumberInventory, NumeralDeclension, NumeralLexeme, NumeralNumberInventory,
    ParticiplePrincipalPart, PresentPrincipalParts, RuleId, ShortMasculineStemFormation, SourceId,
    VerbConjugation,
};
pub use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, AdverbialParticipleFormation, AnalyticConstruction, Animacy,
    Case, CollationKey, CollationProfile, CollationStrength, Comparison, CompoundAuxiliaryOrder,
    CompoundFutureAuxiliary, ConditionalCopulaOrder, ConditionalFormation, Confidence,
    CopulaOmissionContext, CyrillicNumeral, DeterminerDeclension, DeterminerLexeme,
    DeterminerNumberInventory, Error, ErrorCode, FiniteTense, FiniteVerbCell, FormSet, FormSource,
    Gender, GenerationPolicy, GrammarCell, ImperativeCell, InitialPresentation, LParticipleCell,
    LetterOccurrence, LexemeId, Loss, MetadataField, ModalConditionalAuxiliary,
    NegativePronounBase, NounCell, NounLexeme, Number, NumeralCell, NumeralKind,
    OptativeFiniteSystem, OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice,
    PassiveAgentGovernment, PassiveFormation, PerfectFormation, PeriphrasticFormation,
    PeriphrasticSemiAuxiliary, PeriphrasticTenseFormation, Person, PhraseFormation, PhraseOrder,
    PhraseRole, PhraseToken, PluperfectFormation, PositionalOperation, PositionalParadigm,
    PositionalReplacement, PositionalRule, PronounCell, PronounCliticProsody, PronounDeclension,
    PronounEnvironment, PronounFormSelection, PronounNumberInventory, PronounPostpositive,
    PronounPrefix, RealizedPhrase, Recension, RenderedText, Result, SynodalWord,
    TransliterationScheme, VariantPolicy, VerbSystem, VerbalNounFormation, VerbalNounPrincipalPart,
    apply_initial_presentation, collation_key, compare_synodal, format_cyrillic_numeral,
    normalize_lookup, normalize_lookup_accentless, parse_cyrillic_numeral, transliterate,
};

/// Resolves a lemma while retaining its stable target identity.
pub fn lookup(lemma: &str) -> Result<LexemeSummary> {
    handles::resolve_summary(lemma)
}

/// Returns every curated target lexeme in deterministic ID order.
pub fn lexemes() -> Result<Vec<LexemeSummary>> {
    registry::all_lexemes()
}

/// Build-time fingerprint of `generated/registry.rs` (FNV-1a over the raw
/// bytes, plus the byte length). The xtask staleness tripwire compares this
/// against the on-disk file so a stale binary refuses to measure.
pub const REGISTRY_FINGERPRINT: &str = env!("SYNODAL_REGISTRY_FINGERPRINT");

/// Returns the complete reviewable metadata associated with one target lexeme.
pub fn lexical_metadata(id: &LexemeId) -> Result<LexicalMetadataSummary> {
    registry::lexical_metadata(id)
}

/// Reports the systems currently available for a stable target lexeme.
pub fn capabilities_by_id(id: &LexemeId, inflector: Inflector) -> Result<Capabilities> {
    handles::capabilities_by_id(id, inflector)
}

/// Lists metadata that prevents otherwise represented systems from running.
pub fn missing_metadata_by_id(id: &LexemeId) -> Result<Vec<MetadataField>> {
    handles::missing_metadata_by_id(id)
}

/// Returns the stable review/evaluation key for a typed grammar cell.
#[must_use]
pub fn grammar_cell_key(cell: GrammarCell) -> String {
    cell.key()
}

/// Returns canonical and compatible wildcard registry keys in lookup order.
///
/// Dictionaries and other reverse-analysis layers should use this function
/// instead of reconstructing the facade's exact-form compatibility rules.
#[must_use]
pub fn grammar_cell_registry_keys(cell: GrammarCell) -> Vec<String> {
    resolver::exact_lookup_keys(cell)
}

/// Returns the reviewed OCS-to-Synodal alignment gold registry, including
/// rejected negative rows.
pub fn recension_alignments() -> Result<Vec<AlignmentSummary>> {
    registry::alignments()
}

/// Returns the explicit, reviewed OCS-to-Synodal transformation-rule registry.
#[must_use]
pub fn recension_transformations() -> Vec<TransformationRuleSummary> {
    registry::transformation_rules()
}

/// Returns preserved conflicts and rejected alignment controls.
#[must_use]
pub fn recension_conflicts() -> Vec<RecensionConflictSummary> {
    registry::conflicts()
}

/// Returns the reviewable positional-letter rules and their exceptions.
#[must_use]
pub fn positional_rules() -> Vec<PositionalRuleSummary> {
    registry::positional_rules()
}

/// Returns the systems whose exact tables override productive formation.
#[must_use]
pub fn irregular_overrides() -> Vec<IrregularOverrideSummary> {
    registry::irregular_overrides()
}

/// Returns all 98 verb entries in Alypy §104's source order.
pub fn irregular_verb_inventory() -> Result<Vec<IrregularVerbInventorySummary>> {
    registry::irregular_verb_inventory()
}

pub fn noun(lemma: &str, case: Case, number: Number, animacy: Animacy) -> Result<FormSet> {
    Noun::resolve(lemma)?.form(case, number, animacy)
}

pub fn short_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> Result<FormSet> {
    Adjective::resolve(lemma)?.form(AdjectiveCell {
        case,
        number,
        gender,
        animacy,
        form: AdjectiveForm::Short,
        comparison: Comparison::Positive,
    })
}

pub fn long_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> Result<FormSet> {
    Adjective::resolve(lemma)?.form(AdjectiveCell {
        case,
        number,
        gender,
        animacy,
        form: AdjectiveForm::Long,
        comparison: Comparison::Positive,
    })
}

/// Inflects an adjective in a fully specified grammatical cell.
///
/// Use [`short_adjective`] or [`long_adjective`] when only a positive-form
/// convenience call is needed.
pub fn adjective(lemma: &str, cell: AdjectiveCell) -> Result<FormSet> {
    Adjective::resolve(lemma)?.form(cell)
}

pub fn present(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.present(person, number)
}

pub fn future(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.future(person, number)
}

pub fn imperfect(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.imperfect(person, number)
}

pub fn aorist(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.aorist(person, number)
}

pub fn imperative(lemma: &str, person: Person, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.imperative(person, number)
}

pub fn infinitive(lemma: &str) -> Result<FormSet> {
    Verb::resolve(lemma)?.infinitive()
}

pub fn l_participle(lemma: &str, gender: Gender, number: Number) -> Result<FormSet> {
    Verb::resolve(lemma)?.l_participle(gender, number)
}

pub fn pronoun(lemma: &str, cell: PronounCell) -> Result<FormSet> {
    Pronoun::resolve(lemma)?.form(cell)
}

pub fn numeral(lemma: &str, cell: NumeralCell) -> Result<FormSet> {
    Numeral::resolve(lemma)?.form(cell)
}

pub fn determiner(lemma: &str, cell: AdjectiveCell) -> Result<FormSet> {
    Determiner::resolve(lemma)?.form(cell)
}

pub fn participle(lemma: &str, cell: ParticipleCell) -> Result<FormSet> {
    Participle::resolve(lemma)?.form(cell)
}

pub fn supine(lemma: &str) -> Result<FormSet> {
    let verb = Verb::resolve(lemma)?;
    Inflector::default().form_by_id(verb.id(), GrammarCell::Supine)
}

pub fn verbal_noun(lemma: &str, cell: NounCell) -> Result<FormSet> {
    let verb = Verb::resolve(lemma)?;
    Inflector::default().form_by_id(verb.id(), GrammarCell::VerbalNoun(cell))
}

/// Specialist stable-ID operations. These delegate to the same canonical cell
/// resolver as direct calls and resolved handles.
pub mod advanced {
    use super::*;

    pub fn form_by_id(id: &LexemeId, cell: GrammarCell) -> Result<FormSet> {
        Inflector::default().form_by_id(id, cell)
    }

    pub fn lookup_by_id(id: &LexemeId) -> Result<LexemeSummary> {
        Inflector::default().from_id(id)
    }
}
#[cfg(test)]
mod tests;
