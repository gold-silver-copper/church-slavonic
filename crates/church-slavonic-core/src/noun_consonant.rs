//! The merged consonant-stem noun inflection kernel
//! (docs/UNIFIED_LANGUAGE_PROMPT.md, execution plan step 4, fourth POS
//! slice; the vocalic stems live in [`crate::noun`]).
//!
//! Endings are given AFTER the extended oblique stem (ен-, ѧт-/ат-, ер-,
//! ес-, ъв-/ов-): the OCS family attaches its class extension to the short
//! citation stem, while the Synodal family supplies the extended stem
//! independently, and this convention lets both columns share one cell
//! space. The classification discipline is the same as [`crate::noun`]:
//! realization is cited inline by projection-rule id, true morphology by
//! its id in [`crate::divergence::NAMED`].
//!
//! Cells whose surface is the (reshaped) citation form — the direct
//! singulars of the athematic classes, where OCS keeps камꙑ/имѧ/мати/свекрꙑ
//! and Synodal reshapes to камень/имѧ/мати/свекры (divergence
//! `noun:consonant-direct-reshape` covers the generated members) — return
//! an empty column and stay family-owned lexical citation facts.

use crate::grammar::{Animacy, Case, Number};
use crate::noun::by_recension;
use crate::recension::Recension;

/// The consonant-stem declension classes shared by both recensions.
///
/// The lexeme-specific Synodal contracts over these classes (день, камень,
/// дщерь, the syncopating -овь members, the paired-body ѻко/ꙋхо duals, the
/// alternating -ес- backgrounds) stay in the Synodal family core.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ConsonantNounClass {
    /// OCS n-stem masculine (камꙑ : камен-) ↔ Alypy §§42–44 fourth
    /// declension masculine in -ен-.
    NMasculine,
    /// OCS n-stem neuter (имѧ : имен-) ↔ Alypy §§42–43 fourth declension
    /// neuter in -ен-.
    NNeuter,
    /// OCS nt-stem neuter (отроча : отрочѧт-) ↔ Alypy §§42–43 fourth
    /// declension neuter in -ат-.
    NtNeuter,
    /// OCS r-stem feminine (мати : матер-) ↔ Alypy §§42–43 fourth
    /// declension feminine in -ер-.
    RFeminine,
    /// OCS s-stem neuter (слово : словес-) ↔ Alypy §§42–43 fourth
    /// declension neuter in -ес-.
    SNeuter,
    /// OCS v-stem feminine (свекрꙑ : свекръв-) ↔ Alypy §§42–44 fourth
    /// declension feminine in -ов- (the OCS jer grade ъв against the
    /// Synodal vocalized ов is stem realization, family-side).
    VFeminine,
}

impl ConsonantNounClass {
    pub const ALL: [Self; 6] = [
        Self::NMasculine,
        Self::NNeuter,
        Self::NtNeuter,
        Self::RFeminine,
        Self::SNeuter,
        Self::VFeminine,
    ];
}

/// One consonant-stem ending cell, after the extended oblique stem. The OCS
/// column is Polivanova's athematic tables; the Synodal column is the Alypy
/// §§42–44 ending set in its reviewed variant order. An empty column marks a
/// family-owned citation cell (see the module doc).
#[must_use]
pub fn consonant_ending(
    class: ConsonantNounClass,
    case: Case,
    number: Number,
    animacy: Animacy,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use ConsonantNounClass::{NMasculine, NNeuter, NtNeuter, RFeminine, SNeuter, VFeminine};
    use Number::{Dual, Plural, Singular};
    let animate = animacy == Animacy::Animate;
    let (ocs, syn): (&[&str], &[&str]) = match (class, case, number) {
        // ---- direct singular citation cells (family-owned) ----
        (NMasculine | RFeminine | VFeminine, Nominative | Vocative, Singular)
        | (NNeuter | NtNeuter | SNeuter, Nominative | Accusative | Vocative, Singular) => {
            (&[], &[])
        }
        // noun:consonant-direct-reshape: the OCS masculine accusative keeps
        // the athematic citation shape (камꙑ, family-owned); Synodal
        // generates -ь / genitive-shaped -е from the extended stem.
        (NMasculine, Accusative, Singular) => {
            if animate {
                (&[], &["е"])
            } else {
                (&[], &["ь"])
            }
        }

        // ---- shared obliques ----
        (_, Genitive, Singular) => (&["е"], &["е"]),
        (_, Dative, Singular) => (&["и"], &["и"]),
        (RFeminine | VFeminine, Accusative, Singular) => (&["ь"], &["ь"]),
        // noun:instrumental-singular-jer.
        (NMasculine | NNeuter | NtNeuter | SNeuter, Instrumental, Singular) => (&["ьмь"], &["емъ"]),
        // noun:i-stem-instrumental-i-grade (-ьѭ against -їю).
        (RFeminine | VFeminine, Instrumental, Singular) => (&["ьѭ"], &["їю"]),
        // noun:consonant-locative-singular-i (the athematic locative -е is
        // levelled to the dative-shaped -и).
        (NMasculine | NNeuter | NtNeuter | SNeuter | VFeminine, Locative, Singular) => {
            (&["е"], &["и"])
        }
        (RFeminine, Locative, Singular) => (&["и"], &["и"]),

        // ---- dual ----
        (NMasculine | RFeminine | VFeminine, Nominative | Accusative | Vocative, Dual) => {
            (&["и"], &["и"])
        }
        // noun:dual-direct-reshape (OCS -ѣ against the Synodal -и).
        (NNeuter | NtNeuter | SNeuter, Nominative | Accusative | Vocative, Dual) => {
            (&["ѣ"], &["и"])
        }
        // realization: fold:uk.
        (NMasculine | NNeuter | NtNeuter | SNeuter, Genitive | Locative, Dual) => (&["оу"], &["ꙋ"]),
        // noun:dual-oblique-reinventory (-оу against -їю).
        (RFeminine | VFeminine, Genitive | Locative, Dual) => (&["оу"], &["їю"]),
        (NMasculine, Dative | Instrumental, Dual) => (&["ьма"], &["ьма"]),
        // realization: gen:jer-medial (-ьма ~ -ема); the Alypy -ама doublet
        // is noun:hard-declension-variant-imports.
        (NNeuter | NtNeuter, Dative | Instrumental, Dual) => (&["ьма"], &["ема", "ама"]),
        // realization: gen:jer-medial.
        (SNeuter | RFeminine, Dative | Instrumental, Dual) => (&["ьма"], &["ема"]),
        (VFeminine, Dative | Instrumental, Dual) => (&["ама"], &["ама"]),

        // ---- plural ----
        // noun:consonant-direct-reshape (OCS камене against Synodal камени).
        (NMasculine, Nominative | Vocative, Plural) => (&["е"], &["и"]),
        (NNeuter | NtNeuter | SNeuter, Nominative | Accusative | Vocative, Plural) => {
            (&["а"], &["а"])
        }
        (RFeminine | VFeminine, Nominative | Vocative, Plural) => (&["и"], &["и"]),
        // noun:soft-genitive-plural-reinventory (-ъ against -їй/-ей).
        (NMasculine, Genitive, Plural) => (&["ъ"], &["їй"]),
        (NNeuter | NtNeuter | SNeuter, Genitive, Plural) => (&["ъ"], &["ъ"]),
        (RFeminine, Genitive, Plural) => (&["ъ"], &["їй", "ей"]),
        (VFeminine, Genitive, Plural) => (&["ъ"], &["ей"]),
        // realization: gen:jer-medial (-ьмъ ~ -емъ; є is the family's
        // positional wide-е norm); the -ѡмъ doublet is
        // noun:hard-declension-variant-imports.
        (NMasculine, Dative, Plural) => (&["ьмъ"], &["ємъ"]),
        (NNeuter | NtNeuter, Dative, Plural) => (&["ьмъ"], &["ємъ", "ѡмъ"]),
        (SNeuter, Dative, Plural) => (&["ьмъ"], &["ємъ"]),
        (RFeminine, Dative, Plural) => (&["ьмъ"], &["емъ"]),
        (VFeminine, Dative, Plural) => (&["амъ"], &["амъ"]),
        (NMasculine, Accusative, Plural) => {
            if animate {
                // noun:soft-genitive-plural-reinventory on the animate arm.
                (&["и"], &["їй"])
            } else {
                (&["и"], &["и"])
            }
        }
        (RFeminine | VFeminine, Accusative, Plural) => {
            if animate {
                // noun:animate-accusative-coverage: the Synodal feminine
                // athematics co-list the genitive-shaped and nominative-
                // shaped animate accusatives.
                (&["и"], &["ей", "и"])
            } else {
                (&["и"], &["и"])
            }
        }
        (NMasculine | RFeminine, Instrumental, Plural) => (&["ьми"], &["ьми"]),
        // realization: gen:yery.
        (NNeuter | NtNeuter | SNeuter, Instrumental, Plural) => (&["ꙑ"], &["ы"]),
        (VFeminine, Instrumental, Plural) => (&["ами"], &["ами"]),
        // realization: gen:jer-medial (-ьхъ ~ -ехъ).
        (NMasculine | RFeminine, Locative, Plural) => (&["ьхъ"], &["ехъ"]),
        // noun:locative-plural-reinventory (-ьхъ against -ѣхъ).
        (NNeuter | NtNeuter | SNeuter, Locative, Plural) => (&["ьхъ"], &["ѣхъ"]),
        (VFeminine, Locative, Plural) => (&["ахъ"], &["ахъ"]),
    };
    by_recension(recension, ocs, syn)
}

/// The -инъ singulative plural, after the syncopated stem (the singular and
/// dual decline as [`crate::noun::VocalicNounClass::OHardMasculine`] on the
/// expanded -ин- stem in both recensions). Polivanova's 2/m** class ↔ the
/// Alypy §37 ethnonym profile.
#[must_use]
pub fn in_singulative_plural_ending(
    case: Case,
    animacy: Animacy,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    let animate = animacy == Animacy::Animate;
    let (ocs, syn): (&[&str], &[&str]) = match case {
        Nominative | Vocative => (&["е"], &["е"]),
        Genitive => (&["ъ"], &["ъ"]),
        Dative => (&["омъ"], &["омъ"]),
        Accusative => {
            if animate {
                (&["ъ"], &["ъ"])
            } else {
                // noun:in-singulative-inanimate-accusative (OCS -ꙑ against
                // the Synodal nominative-shaped -е).
                (&["ꙑ"], &["е"])
            }
        }
        // realization: gen:yery.
        Instrumental => (&["ꙑ"], &["ы"]),
        Locative => (&["ѣхъ"], &["ѣхъ"]),
    };
    by_recension(recension, ocs, syn)
}

/// The agent -тель direct plural overrides (Polivanova's 2/m* class ↔ the
/// Alypy §37 agent profile); every other cell declines as
/// [`crate::noun::VocalicNounClass::JoSoftMasculine`]. The animate
/// accusative is not overridden in either recension and returns empty.
#[must_use]
pub fn agent_direct_plural_ending(
    case: Case,
    animacy: Animacy,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Nominative, Vocative};
    let (ocs, syn): (&[&str], &[&str]) = match case {
        // noun:agent-plural-reinventory (OCS -ѥ against the Synodal ordered
        // -и/-е/-їе variants).
        Nominative | Vocative => (&["ѥ"], &["и", "е", "їе"]),
        // noun:soft-direct-plural-leveling (-ѩ against -и).
        Accusative if animacy == Animacy::Inanimate => (&["ѩ"], &["и"]),
        _ => (&[], &[]),
    };
    by_recension(recension, ocs, syn)
}

#[cfg(test)]
mod tests {
    use super::{ConsonantNounClass, consonant_ending, in_singulative_plural_ending};
    use crate::grammar::{Animacy, Case, Number};
    use crate::recension::Recension;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::SynodalRussian;

    #[test]
    fn unsupported_recensions_yield_empty_cells() {
        for recension in Recension::ALL {
            if matches!(
                recension,
                Recension::OldChurchSlavonic | Recension::SynodalRussian
            ) {
                continue;
            }
            for class in ConsonantNounClass::ALL {
                for case in Case::ALL {
                    for number in Number::ALL {
                        for animacy in Animacy::ALL {
                            assert!(
                                consonant_ending(class, case, number, animacy, recension)
                                    .is_empty()
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn only_the_declared_citation_cells_are_empty() {
        for class in ConsonantNounClass::ALL {
            for case in Case::ALL {
                for number in Number::ALL {
                    for animacy in Animacy::ALL {
                        let ocs = consonant_ending(class, case, number, animacy, OCS);
                        let syn = consonant_ending(class, case, number, animacy, SYN);
                        let masculine_accusative_citation = class == ConsonantNounClass::NMasculine
                            && case == Case::Accusative
                            && number == Number::Singular;
                        let citation = number == Number::Singular
                            && match class {
                                ConsonantNounClass::NMasculine
                                | ConsonantNounClass::RFeminine
                                | ConsonantNounClass::VFeminine => {
                                    matches!(case, Case::Nominative | Case::Vocative)
                                }
                                _ => matches!(
                                    case,
                                    Case::Nominative | Case::Accusative | Case::Vocative
                                ),
                            };
                        assert_eq!(ocs.is_empty(), citation || masculine_accusative_citation);
                        assert_eq!(syn.is_empty(), citation);
                    }
                }
            }
        }
    }

    #[test]
    fn consonant_direct_reshape_holds() {
        // noun:consonant-direct-reshape.
        assert_eq!(
            consonant_ending(
                ConsonantNounClass::NMasculine,
                Case::Nominative,
                Number::Plural,
                Animacy::Inanimate,
                OCS,
            ),
            ["е"]
        );
        assert_eq!(
            consonant_ending(
                ConsonantNounClass::NMasculine,
                Case::Nominative,
                Number::Plural,
                Animacy::Inanimate,
                SYN,
            ),
            ["и"]
        );
        assert_eq!(
            consonant_ending(
                ConsonantNounClass::NMasculine,
                Case::Accusative,
                Number::Singular,
                Animacy::Inanimate,
                SYN,
            ),
            ["ь"]
        );
    }

    #[test]
    fn singulative_plural_keeps_the_shared_o_stem_shape() {
        for case in [
            Case::Nominative,
            Case::Genitive,
            Case::Dative,
            Case::Locative,
        ] {
            assert_eq!(
                in_singulative_plural_ending(case, Animacy::Inanimate, OCS),
                in_singulative_plural_ending(case, Animacy::Inanimate, SYN),
            );
        }
        // noun:in-singulative-inanimate-accusative.
        assert_eq!(
            in_singulative_plural_ending(Case::Accusative, Animacy::Inanimate, OCS),
            ["ꙑ"]
        );
        assert_eq!(
            in_singulative_plural_ending(Case::Accusative, Animacy::Inanimate, SYN),
            ["е"]
        );
    }
}
