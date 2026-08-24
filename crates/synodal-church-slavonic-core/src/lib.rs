#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod accent;
pub mod collation;
pub mod determiner;
pub mod error;
pub mod evidence;
pub mod grammar;
pub mod ids;
pub mod mapping;
pub mod morphology;
pub mod numeral;
pub mod numeral_morphology;
pub mod orthography;
pub mod phrase;
pub mod policy;
pub mod pronoun;
pub mod recension;
pub mod result;
pub mod trace;
pub mod transliteration;

pub use accent::{
    AccentEnclitic, AccentEnvironment, AccentMark, AccentParadigm, AccentPlacement, AccentRule,
    AccentScope, BreathingMark, BreathingRule, EncliticParticle,
};
pub use collation::{
    CollationKey, CollationProfile, CollationStrength, collation_key, compare_synodal,
};
pub use determiner::{
    DeterminerDeclension, DeterminerLexeme, DeterminerNumberInventory, decline_determiner,
    validate_determiner_lexeme,
};
pub use error::{Error, ErrorCode, MetadataField, Result};
pub use evidence::{
    Assumption, AuthorityRole, Confidence, Contradiction, EpistemicRole, Evidence, EvidenceKind,
    FormSource,
};
pub use grammar::*;
pub use ids::{EvidenceId, LexemeId, ModelId, RecensionMappingId, RuleId, SourceId};
pub use mapping::{
    LexemeRelation, MappingStatus, MorphologyAlignment, RecensionMapping, SemanticAlignment,
    Transformation,
};
pub use morphology::{
    ActiveParticipleShortFormation, AdjectiveClass, AdjectiveLexeme, AoristFormation, Aspect,
    ComparisonFormation, ImperativeFormation, ImperfectFormation, NounAnimacyInventory,
    NounDeclension, NounLexeme, NounNumberInventory, ParticiplePrincipalPart,
    PresentPrincipalParts, REFLEXIVE_RULE_ID, ShortMasculineStemFormation, VerbConjugation,
    VerbLexeme, VerbalNounFormation, VerbalNounPrincipalPart, aorist, decline_adjective,
    decline_noun, decline_participle, decline_verbal_noun, future, imperative, imperfect,
    infinitive, l_participle, present, reflexive_base_candidates, reflexive_surface,
    validate_adjective_lexeme, validate_noun_lexeme,
};
pub use numeral::{CyrillicNumeral, format_cyrillic_numeral, parse_cyrillic_numeral};
pub use numeral_morphology::{
    NumeralDeclension, NumeralLexeme, NumeralNumberInventory, decline_numeral,
    validate_numeral_lexeme,
};
pub use orthography::{
    InitialPresentation, LetterOccurrence, Loss, NormalizationReport, OrthographyProfile,
    PositionalOperation, PositionalParadigm, PositionalReplacement, PositionalRule, RenderedText,
    SynodalWord, apply_initial_presentation, normalize_lookup, normalize_lookup_accentless,
    present_initial_uk_digraph,
};
pub use phrase::{
    AdverbialParticipleFormation, AnalyticConstruction, CompoundAuxiliaryOrder,
    CompoundFutureAuxiliary, ConditionalCopulaOrder, ConditionalFormation, CopulaOmissionContext,
    ModalConditionalAuxiliary, NegativePronounBase, OptativeFiniteSystem, PassiveAgentGovernment,
    PassiveFormation, PerfectFormation, PeriphrasticFormation, PeriphrasticSemiAuxiliary,
    PeriphrasticTenseFormation, PhraseFormation, PhraseOrder, PhraseRole, PhraseToken,
    PluperfectFormation, PronounCliticProsody, RealizedPhrase,
};
pub use policy::{GenerationPolicy, VariantPolicy};
pub use pronoun::{
    PronounDeclension, PronounEnvironment, PronounFormSelection, PronounLexeme,
    PronounNumberInventory, PronounPostpositive, PronounPrefix, decline_pronoun,
    validate_pronoun_lexeme,
};
pub use recension::Recension;
pub use result::{FormSet, FormVariant, Romanization, VariantSelection};
pub use trace::{RuleTrace, TraceStep};
pub use transliteration::{TransliterationScheme, transliterate};
