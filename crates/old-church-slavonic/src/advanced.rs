//! Specialist cell, by-ID, metadata, explicit-rule, and raw-feature APIs.
//!
//! Ordinary callers should start with the direct functions and resolved lexical
//! handles at the crate root.

/// Typed cell structures used by generic tools and paradigms.
pub mod cells {
    pub use old_church_slavonic_core::{
        AdjectiveCell, AdjectiveForm, ClosedClassCell, FiniteVerbCell, ImperativeCell,
        LParticipleCell, NounCell, ParticipleCell, ParticipleKind,
    };
}

/// Stable dictionary-identity operations.
pub mod by_id {
    pub use crate::resolver::{
        adjective_by_id, adjective_comparatives_by_id, adjective_paradigm_by_id, finite_verb_by_id,
        finite_verb_paradigm_by_id, imperative_by_id, imperative_paradigm_by_id, infinitive_by_id,
        l_participle_by_id, l_participle_paradigm_by_id, noun_by_id, noun_paradigm_by_id,
        participle_by_id, participle_citation_by_id, participle_paradigm_by_id, supine_by_id,
        verb_paradigm_by_id, verbal_noun_by_id,
    };
}

/// Explicit caller-supplied lexical metadata and productive rules.
pub mod rules {
    pub use crate::resolver::{
        adjective_with, finite_verb_with, imperative_with, infinitive_with, l_participle_with,
        noun_with, participle_with, supine_with,
    };
    pub use old_church_slavonic_core::adjective::AdjectiveLexeme;
    pub use old_church_slavonic_core::noun::NounLexeme;
    pub use old_church_slavonic_core::verb::{VerbLexeme, VerbLexemeBuilder};
    pub use old_church_slavonic_core::{
        AdjectiveClass, AoristFormation, ImperativeFormation, ImperfectFormation,
        ImperfectVariantPolicy, NounClass, NumberRestriction, PastActiveParticipleFormation,
        PastPassiveParticipleFormation, PresentActiveParticipleFormation,
        PresentPassiveParticipleFormation, VerbAspect, VerbClass,
    };
}

/// Source-backed dictionary principal parts and evaluation entry points.
pub mod metadata {
    pub use crate::metadata::{
        DictionaryVerbMetadata, ImperfectMetadataAnalysis, NormalizedVerbMetadataField,
        PresentMetadataAnalysis, SourcedMetadata, VerbStemMetadata, VerbSystemMetadata,
        verb_metadata_by_id,
    };
    pub use crate::resolver::{
        finite_verb_from_dictionary_metadata, imperative_from_dictionary_metadata,
        l_participle_from_dictionary_metadata, participle_from_dictionary_metadata,
    };
}

/// Generic feature-key access for extraction, evaluation, and diagnostics.
pub mod raw_features {
    pub use crate::resolver::{
        closed_class, closed_class_by_id, dictionary_form_by_id, dictionary_paradigm_by_id,
        form_by_id,
    };
}

pub use crate::resolver::{
    adjective as adjective_form, finite_verb as finite_verb_form, imperative as imperative_form,
    l_participle as l_participle_form, noun as noun_form, participle as participle_form,
    participle_citation,
};
