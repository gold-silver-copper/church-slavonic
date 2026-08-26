//! Pins the Synodal family noun tables that do NOT read the merged noun
//! kernel directly to the kernel's Synodal columns, so the two copies of
//! each closed table cannot drift (docs/UNIFIED_LANGUAGE_PROMPT.md, phase-4
//! noun slice): the velar subclasses are family reshapes of the shared hard
//! columns (`unmerged:noun:synodal-subclass-tables`) and must equal the
//! kernel column everywhere outside their declared velar-specific cells.

use super::noun::{NounDeclension, NounLexeme, noun_endings};
use crate::{Animacy, Case, Gender, Number, SynodalWord};
use church_slavonic_core::Recension;
use church_slavonic_core::noun::{VocalicNounClass, vocalic_ending};

const SYN: Recension = Recension::SynodalRussian;

fn lexeme(lemma: &str, stem: &str, gender: Gender, declension: NounDeclension) -> NounLexeme {
    NounLexeme::new(
        SynodalWord::parse(lemma).expect("valid lemma"),
        SynodalWord::parse(stem).expect("valid stem"),
        gender,
        declension,
    )
}

#[test]
fn velar_masculine_table_matches_the_kernel_hard_column_outside_its_seams() {
    let velar = lexeme(
        "богъ",
        "бог",
        Gender::Masculine,
        NounDeclension::FirstHardVelarMasculine,
    );
    for case in Case::ALL {
        for number in Number::ALL {
            for animacy in Animacy::ALL {
                // The velar-specific cells: the positional ы/и nominative-
                // vocative plural, the inanimate accusative plural -и, and
                // the -и primary of the instrumental plural.
                let velar_specific = number == Number::Plural
                    && (matches!(case, Case::Nominative | Case::Vocative | Case::Instrumental)
                        || (case == Case::Accusative && animacy == Animacy::Inanimate));
                if velar_specific {
                    continue;
                }
                let cell = crate::NounCell {
                    case,
                    number,
                    animacy,
                };
                let family = noun_endings(&velar, cell).expect("velar family cell");
                let kernel =
                    vocalic_ending(VocalicNounClass::OHardMasculine, case, number, animacy, SYN);
                assert_eq!(family, kernel, "{case:?} {number:?} {animacy:?}");
            }
        }
    }
}

#[test]
fn velar_second_declension_table_matches_the_kernel_a_column_outside_its_seams() {
    let velar = lexeme(
        "владыка",
        "владык",
        Gender::Masculine,
        NounDeclension::SecondHardVelar,
    );
    for case in Case::ALL {
        for number in Number::ALL {
            for animacy in Animacy::ALL {
                // The velar-specific cells: the -и genitive singular and the
                // -и nominative/vocative and inanimate accusative plural.
                let velar_specific = (number == Number::Singular && case == Case::Genitive)
                    || (number == Number::Plural
                        && (matches!(case, Case::Nominative | Case::Vocative)
                            || (case == Case::Accusative && animacy == Animacy::Inanimate)));
                if velar_specific {
                    continue;
                }
                let cell = crate::NounCell {
                    case,
                    number,
                    animacy,
                };
                let family = noun_endings(&velar, cell).expect("velar family cell");
                let kernel = vocalic_ending(VocalicNounClass::AHard, case, number, animacy, SYN);
                assert_eq!(family, kernel, "{case:?} {number:?} {animacy:?}");
            }
        }
    }
}
