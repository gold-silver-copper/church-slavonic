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
pub mod orthography;
pub mod phrase;
pub mod policy;
pub mod pronoun;
pub mod recension;
pub mod result;
pub mod trace;
pub mod transliteration;

pub use accent::{
    AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope, BreathingMark,
    BreathingRule,
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
    ComparisonFormation, ImperativeFormation, ImperfectFormation, NounDeclension, NounLexeme,
    NounNumberInventory, ParticiplePrincipalPart, PresentPrincipalParts, VerbConjugation,
    VerbLexeme, aorist, decline_adjective, decline_noun, decline_participle, imperative, imperfect,
    infinitive, l_participle, present, validate_noun_lexeme,
};
pub use numeral::{CyrillicNumeral, format_cyrillic_numeral, parse_cyrillic_numeral};
pub use orthography::{
    InitialPresentation, Loss, NormalizationReport, OrthographyProfile, RenderedText, SynodalWord,
    apply_initial_presentation, normalize_lookup, normalize_lookup_accentless,
};
pub use phrase::{
    AnalyticConstruction, NegativePronounBase, PhraseRole, PhraseToken, PronounCliticProsody,
    RealizedPhrase,
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
