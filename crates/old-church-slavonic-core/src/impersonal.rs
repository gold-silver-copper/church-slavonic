//! Lexical valency and morphology of source-identified impersonal predicates.
//!
//! Impersonality is not itself a conjugational class. `достоꙗти` is lexically
//! impersonal, while `мьнѣти` has both ordinary personal senses and a distinct
//! reflexive impersonal construction. Both retain their regular morphological
//! profiles; the construction selects third-person singular rather than
//! deleting the other mechanically possible cells.

use crate::verb::VerbLexeme;
use crate::{
    AoristFormation, Case, FiniteTense, FiniteVerbCell, ImperativeFormation, ImperfectFormation,
    ImperfectVariantPolicy, Number, PastActiveParticipleFormation, PastPassiveParticipleFormation,
    Person, PresentActiveParticipleFormation, PresentPassiveParticipleFormation, VerbAspect,
    VerbClass, VerbDefectKind, VerbMorphologySystem,
};

/// Whether impersonality belongs to the lexeme or to one use of a personal verb.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpersonalVerbStatus {
    LexicallyImpersonal,
    ImpersonalSenseOfPersonalVerb,
}

/// Exhaustive impersonal verb identities in the pinned OCS dictionary senses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImpersonalVerbIdentity {
    /// `достоꙗти` ‘befit, be proper’, governing the experiencer in the dative.
    Dostojati,
    /// Reflexive `мьнѣти сѧ` ‘seem’, distinct from personal `мьнѣти` ‘think’.
    MnetiReflexive,
}

impl ImpersonalVerbIdentity {
    pub const ALL: [Self; 2] = [Self::Dostojati, Self::MnetiReflexive];

    pub const fn lemma(self) -> &'static str {
        match self {
            Self::Dostojati => "достоꙗти",
            Self::MnetiReflexive => "мьнѣти",
        }
    }

    pub const fn status(self) -> ImpersonalVerbStatus {
        match self {
            Self::Dostojati => ImpersonalVerbStatus::LexicallyImpersonal,
            Self::MnetiReflexive => ImpersonalVerbStatus::ImpersonalSenseOfPersonalVerb,
        }
    }

    pub const fn governed_case(self) -> Case {
        Case::Dative
    }

    pub const fn reflexive_particle(self) -> Option<&'static str> {
        match self {
            Self::Dostojati => None,
            Self::MnetiReflexive => Some("сѧ"),
        }
    }

    /// The finite predicate cell selected by the impersonal construction.
    pub const fn predicate_cell(self, tense: FiniteTense) -> FiniteVerbCell {
        FiniteVerbCell {
            tense,
            person: Person::Third,
            number: Number::Singular,
        }
    }

    /// Complete reconstructable word morphology, independent of impersonal
    /// syntactic selection.
    pub fn lexeme(self) -> VerbLexeme {
        match self {
            Self::Dostojati => dostojati_lexeme(),
            Self::MnetiReflexive => mneti_lexeme(),
        }
    }
}

fn dostojati_lexeme() -> VerbLexeme {
    let mut lexeme = VerbLexeme::new("достоꙗти", VerbClass::II3);
    lexeme.aspect = Some(VerbAspect::Imperfective);
    lexeme.stems.present = Some("досто".to_string());
    lexeme.stems.present_first_singular = Some("досто".to_string());
    lexeme.stems.imperfect = Some("достоꙗ".to_string());
    lexeme.formations.imperfect = Some(ImperfectFormation::A);
    lexeme.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
    lexeme.stems.aorist = Some("достоꙗ".to_string());
    lexeme.stems.aorist_second_third_singular = Some("достоꙗ".to_string());
    lexeme.formations.aorist = Some(AoristFormation::SigmaticVowel);
    lexeme.stems.imperative = Some("досто".to_string());
    lexeme.formations.imperative = Some(ImperativeFormation::ISeries);
    lexeme.stems.l_participle = Some("достоꙗ".to_string());
    lexeme.stems.present_active_participle = Some("досто".to_string());
    lexeme.formations.present_active_participle = Some(PresentActiveParticipleFormation::YeshtSoft);
    lexeme.stems.past_active_participle = Some("достоꙗ".to_string());
    lexeme.formations.past_active_participle = Some(PastActiveParticipleFormation::Vush);
    // An impersonal intransitive predicate has no passive voice. This is a
    // grammatical defect, not an inference from missing dictionary rows.
    lexeme.defective_systems.insert(
        VerbMorphologySystem::Participle(crate::ParticipleKind::PresentPassive),
        VerbDefectKind::HistoricallyInvalid,
    );
    lexeme.defective_systems.insert(
        VerbMorphologySystem::Participle(crate::ParticipleKind::PastPassive),
        VerbDefectKind::HistoricallyInvalid,
    );
    lexeme
}

fn mneti_lexeme() -> VerbLexeme {
    let mut lexeme = VerbLexeme::new("мьнѣти", VerbClass::II2);
    lexeme.aspect = Some(VerbAspect::Imperfective);
    lexeme.stems.present = Some("мьн".to_string());
    lexeme.stems.present_first_singular = Some("мьн".to_string());
    lexeme.stems.imperfect = Some("мьн".to_string());
    lexeme.formations.imperfect = Some(ImperfectFormation::YatA);
    lexeme.formations.imperfect_variant_policy = Some(ImperfectVariantPolicy::UncontractedOnly);
    lexeme.stems.aorist = Some("мьнѣ".to_string());
    lexeme.stems.aorist_second_third_singular = Some("мьнѣ".to_string());
    lexeme.formations.aorist = Some(AoristFormation::SigmaticVowel);
    lexeme.stems.imperative = Some("мьн".to_string());
    lexeme.formations.imperative = Some(ImperativeFormation::ISeries);
    lexeme.stems.l_participle = Some("мьнѣ".to_string());
    lexeme.stems.present_active_participle = Some("мьн".to_string());
    lexeme.formations.present_active_participle = Some(PresentActiveParticipleFormation::YeshtSoft);
    lexeme.stems.present_passive_participle = Some("мьн".to_string());
    lexeme.formations.present_passive_participle = Some(PresentPassiveParticipleFormation::Im);
    lexeme.stems.past_active_participle = Some("мьнѣ".to_string());
    lexeme.formations.past_active_participle = Some(PastActiveParticipleFormation::Vush);
    lexeme.stems.past_passive_participle = Some("мьнѣ".to_string());
    lexeme.formations.past_passive_participle = Some(PastPassiveParticipleFormation::N);
    lexeme
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verb::{finite, imperative, infinitive, l_participle, participle, supine};
    use crate::{
        AdjectiveCell, AdjectiveForm, Animacy, Gender, ImperativeCell, InflectionError,
        LParticipleCell, ParticipleCell, ParticipleKind,
    };

    fn short_nominative(kind: ParticipleKind) -> ParticipleCell {
        ParticipleCell {
            kind,
            adjective: AdjectiveCell {
                form: AdjectiveForm::Short,
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
            },
        }
    }

    #[test]
    fn impersonal_inventory_distinguishes_lexeme_from_construction() {
        assert_eq!(ImpersonalVerbIdentity::ALL.len(), 2);
        assert_eq!(
            ImpersonalVerbIdentity::Dostojati.status(),
            ImpersonalVerbStatus::LexicallyImpersonal
        );
        assert_eq!(
            ImpersonalVerbIdentity::MnetiReflexive.status(),
            ImpersonalVerbStatus::ImpersonalSenseOfPersonalVerb
        );
        for identity in ImpersonalVerbIdentity::ALL {
            assert_eq!(identity.governed_case(), Case::Dative);
            for tense in FiniteTense::ALL {
                let cell = identity.predicate_cell(tense);
                assert_eq!(cell.person, Person::Third);
                assert_eq!(cell.number, Number::Singular);
            }
        }
    }

    #[test]
    fn source_forms_and_reconstructable_aorists_are_explicit() {
        let present = FiniteTense::Present;
        let imperfect = FiniteTense::Imperfect;
        let aorist = FiniteTense::Aorist;
        for (identity, expected_present, expected_imperfect, expected_aorist) in [
            (
                ImpersonalVerbIdentity::Dostojati,
                "достоитъ",
                "достоꙗаше",
                "достоꙗ",
            ),
            (
                ImpersonalVerbIdentity::MnetiReflexive,
                "мьнитъ",
                "мьнѣаше",
                "мьнѣ",
            ),
        ] {
            let lexeme = identity.lexeme();
            assert_eq!(
                finite(&lexeme, identity.predicate_cell(present))
                    .expect("source present")
                    .text,
                expected_present
            );
            assert_eq!(
                finite(&lexeme, identity.predicate_cell(imperfect))
                    .expect("source imperfect")
                    .text,
                expected_imperfect
            );
            assert_eq!(
                finite(&lexeme, identity.predicate_cell(aorist))
                    .expect("reconstructable vowel aorist")
                    .text,
                expected_aorist
            );
        }
    }

    #[test]
    fn word_morphology_remains_complete_outside_syntactic_selection() {
        for identity in ImpersonalVerbIdentity::ALL {
            let lexeme = identity.lexeme();
            for cell in FiniteVerbCell::all() {
                finite(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{identity:?} {cell:?}: {error:?}"));
            }
            for cell in ImperativeCell::SUPPORTED {
                imperative(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{identity:?} {cell:?}: {error:?}"));
            }
            infinitive(&lexeme).expect("infinitive");
            supine(&lexeme).expect("supine");
            for cell in LParticipleCell::all() {
                l_participle(&lexeme, cell)
                    .unwrap_or_else(|error| panic!("{identity:?} {cell:?}: {error:?}"));
            }
            for kind in [ParticipleKind::PresentActive, ParticipleKind::PastActive] {
                for cell in ParticipleCell::for_kind(kind) {
                    participle(&lexeme, cell)
                        .unwrap_or_else(|error| panic!("{identity:?} {cell:?}: {error:?}"));
                }
            }
        }

        let dostojati = ImpersonalVerbIdentity::Dostojati.lexeme();
        for kind in [ParticipleKind::PresentPassive, ParticipleKind::PastPassive] {
            let cell = short_nominative(kind);
            assert!(matches!(
                participle(&dostojati, cell),
                Err(InflectionError::HistoricallyInvalidCell { .. })
            ));
        }

        let mneti = ImpersonalVerbIdentity::MnetiReflexive.lexeme();
        assert_eq!(
            participle(&mneti, short_nominative(ParticipleKind::PresentPassive))
                .expect("source present passive")
                .text,
            "мьнимъ"
        );
        assert_eq!(
            participle(&mneti, short_nominative(ParticipleKind::PastPassive))
                .expect("source past passive")
                .text,
            "мьнѣнъ"
        );
    }
}
