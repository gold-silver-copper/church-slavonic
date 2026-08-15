//! Specialist cell, by-ID, metadata, explicit-rule, and raw-feature APIs.
//!
//! Ordinary callers should start with the direct functions and resolved lexical
//! handles at the crate root.

/// Typed cell structures used by generic tools and paradigms.
pub mod cells {
    pub use old_church_slavonic_core::{
        AdjectiveCell, AdjectiveForm, ClosedClassCell, CollectiveNumeralCell, CompoundCardinalCell,
        DeterminerCell, FiniteVerbCell, GenderedCell, ImperativeCell, LParticipleCell, NounCell,
        NumeralCell, ParticipleCell, ParticipleKind, PersonalPronounCell, UngenderedCell,
    };
}

/// Stable dictionary- or reviewed-profile identity operations.
pub mod by_id {
    pub use crate::resolver::{
        adjective_by_id, adjective_paradigm_by_id, comparative_citation_by_id, determiner_by_id,
        determiner_paradigm_by_id, finite_by_id, finite_paradigm_by_id, gendered_numeral_by_id,
        gendered_numeral_paradigm_by_id, gendered_pronoun_by_id, gendered_pronoun_paradigm_by_id,
        imperative_by_id, imperative_paradigm_by_id, infinitive_by_id, l_participle_by_id,
        l_participle_paradigm_by_id, noun_by_id, noun_paradigm_by_id, numeral_by_id,
        numeral_paradigm_by_id, participle_by_id, participle_citation_by_id,
        participle_paradigm_by_id, personal_pronoun_by_id, personal_pronoun_paradigm_by_id,
        present_paradigm_by_id, pronoun_by_id, pronoun_paradigm_by_id, supine_by_id,
        verbal_noun_by_id, verbal_noun_form_by_id, verbal_noun_paradigm_by_id,
    };
}

/// Explicit caller-supplied lexical metadata and productive rules.
pub mod rules {
    pub use crate::phrases::pronominal_family_with;
    pub use crate::resolver::{
        adjective_with, comparative_paradigm_with, comparative_with, determiner_with,
        finite_verb_with, imperative_with, infinitive_with, l_participle_with, long_only_adjective,
        noun_with, participle_with, pre_superlative_with, pronominal_with, supine_with,
        verbal_noun_with,
    };
    pub use old_church_slavonic_core::adjective::{
        AdjectiveLexeme, ComparativeLexeme, LongOnlyAdjectiveIdentity, productive_new_comparative,
    };
    pub use old_church_slavonic_core::determiner::{DeterminerDeclension, DeterminerLexeme};
    pub use old_church_slavonic_core::noun::NounLexeme;
    pub use old_church_slavonic_core::pronoun::{
        DirectToTreatment, PronominalDeclension, PronominalFamilySpec, PronominalLexeme,
        PronominalPostpositive, PronominalPrefix,
    };
    pub use old_church_slavonic_core::verb::{VerbLexeme, VerbLexemeBuilder};
    pub use old_church_slavonic_core::{
        AdjectiveClass, AoristFormation, ComparativeFormation, ImperativeFormation,
        ImperfectFormation, ImperfectVariantPolicy, NounClass, NumberRestriction,
        PastActiveParticipleFormation, PastPassiveParticipleFormation, PolivanovaRegularNounClass,
        PolivanovaRegularNounSpecification, PolivanovaRegularVerbClass,
        PolivanovaRegularVerbSpecification, PresentActiveParticipleFormation, PresentFormation,
        PresentPassiveParticipleFormation, RegularNounFamily, RegularNounSourceMember,
        RegularVerbFamily, RegularVerbSourceMember, VerbAspect, VerbClass,
    };
}

/// Source-backed dictionary principal parts and evaluation entry points.
pub mod metadata {
    pub use crate::metadata::{
        AoristMetadataAnalysis, DictionaryVerbMetadata, ImperfectMetadataAnalysis,
        NormalizedVerbMetadataField, PresentMetadataAnalysis, SourcedMetadata, VerbStemMetadata,
        VerbSystemMetadata, verb_metadata_by_id,
    };
    pub use crate::resolver::{
        finite_verb_from_dictionary_metadata, imperative_from_dictionary_metadata,
        l_participle_from_dictionary_metadata, participle_from_dictionary_metadata,
        verbal_noun_from_dictionary_metadata,
    };
}

/// Generic feature-key access for extraction, evaluation, and diagnostics.
pub mod raw_features {
    pub use crate::resolver::{
        dictionary_form_by_id, dictionary_paradigm_by_id, form_by_id,
        raw_closed_class as closed_class, raw_closed_class_by_id as closed_class_by_id,
    };
}

/// Explicit source-bounded orthographic reconstruction.
pub mod orthography {
    pub use crate::{
        realize_glagolitic, realize_glagolitic_variants, reconstruct_accent,
        transliterate_glagolitic_to_cyrillic,
    };
    pub use old_church_slavonic_core::{
        AccentEvidence, AccentParadigm, AccentPlacement, AccentReconstructionStatus, AccentRule,
        AccentScope, GlagoliticProfile, ReconstructedAccent, TransliteratedForm,
        TransliterationDirection, TransliterationFidelity, TransliterationLoss,
        TransliterationLossKind, TransliterationLossPolicy,
    };
}

pub use crate::resolver::{
    adjective as adjective_form, finite_verb as finite_verb_form, imperative as imperative_form,
    l_participle as l_participle_form, noun as noun_form, participle as participle_form,
    participle_citation,
};
