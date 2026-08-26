//! The merged determiner inflection kernel (docs/UNIFIED_LANGUAGE_PROMPT.md,
//! execution plan step 4, second POS slice).
//!
//! The two recensions' determiner inventories overlap only where the actual
//! sources overlap: the hard short (pronominal `2/p`) agreement terminals and
//! the totalizing вьсь/весь paradigm. Every cell is written with both
//! recensions side by side so that a difference is always visibly one of:
//!
//! - **realization** — related by the declared projection rules of
//!   `church-slavonic-orthography::projection` (cited inline by rule id) or
//!   by a named Synodal spelling norm outside that rule set (checked by the
//!   realization-coherence test in the orthography crate);
//! - **a named divergence** — cited inline by its id in
//!   [`crate::divergence::NAMED`];
//! - **a per-recension lexical fact** — which never reaches this module: the
//!   velar universal вьсакъ↔всѧкъ, the adjective-backed classes (которꙑи,
//!   ѥтеръ, `-скїй`, the Synodal long forms), and the non-overlapping
//!   identity inventories stay in the family cores (see
//!   [`crate::divergence::UNMERGED`]).
//!
//! The family cores are adapters over these tables: they own their interface
//! types, stems, validation, and error vocabularies. Recensions other than
//! the two attested ones yield empty cells.

use crate::pronoun::AgreeingClass;
use crate::{Animacy, Case, Gender, Number, Recension};

const NO_TEXTS: &[&str] = &[];

/// The ending set of one hard short (pronominal `2/p`) determiner cell.
///
/// The OCS column IS the merged pronoun kernel's hard agreeing class
/// (Polivanova's `2/p`, shared with тъ/такъ); the Synodal column is the
/// Alypy §§45 and 48 самъ short table with its co-listed doublets. Beyond
/// the pronoun-class realization rules, the columns differ by:
///
/// - divergence `det:hard-oblique-jat-doublets` (Synodal ѣй beside ой, ѣмъ
///   beside омъ, and the огѡ/ого and ого/огѡ variant orders);
/// - divergence `det:hard-feminine-plural-nominative` (Synodal -и where the
///   OCS hard class has -ы, in the feminine nominative and the inanimate
///   accusative plural);
/// - divergence `pron:genitive-accusative` on the Synodal animate arms;
/// - divergence `pron:instr-loc-sg-jer` on the -мь/-мъ terminals;
/// - realization `fold:omega` on the dual genitive/locative ою ~ ѡю.
///
/// Vocatives are empty in both recensions (the Synodal family renders its
/// vocative as the nominative before querying the kernel).
#[must_use]
pub fn hard_short_ending(
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};

    match recension {
        Recension::OldChurchSlavonic => crate::pronoun::agreeing_ending(
            AgreeingClass::Hard,
            case,
            number,
            gender,
            animacy,
            recension,
        ),
        Recension::SynodalRussian => {
            let animate = animacy == Animacy::Animate;
            match (case, number, gender) {
                (Nominative, Singular, Masculine) => &["ъ"],
                (Nominative, Singular, Feminine) => &["а"],
                (Nominative, Singular, Neuter) => &["о"],
                // det:hard-oblique-jat-doublets (огѡ/ого order).
                (Genitive, Singular, Masculine | Neuter) => &["огѡ", "ого"],
                (Genitive, Singular, Feminine) => &["оѧ"],
                (Dative, Singular, Masculine | Neuter) => &["омꙋ"],
                // det:hard-oblique-jat-doublets (ѣй beside ой).
                (Dative | Locative, Singular, Feminine) => &["ой", "ѣй"],
                // pron:genitive-accusative on the animate arm.
                (Accusative, Singular, Masculine) => {
                    if animate {
                        &["ого", "огѡ"]
                    } else {
                        &["ъ"]
                    }
                }
                (Accusative, Singular, Feminine) => &["ꙋ"],
                (Accusative, Singular, Neuter) => &["о"],
                // pron:instr-loc-sg-jer.
                (Instrumental, Singular, Masculine | Neuter) => &["ѣмъ"],
                (Instrumental, Singular, Feminine) => &["ою"],
                // det:hard-oblique-jat-doublets (ѣмъ beside омъ).
                (Locative, Singular, Masculine | Neuter) => &["омъ", "ѣмъ"],
                (Nominative | Accusative, Dual, Masculine) => &["а"],
                (Nominative | Accusative, Dual, Feminine | Neuter) => &["ѣ"],
                // realization: fold:omega (ою ~ ѡю).
                (Genitive | Locative, Dual, _) => &["ѡю"],
                (Dative | Instrumental, Dual, _) => &["ѣма"],
                (Nominative, Plural, Masculine) => &["и"],
                // det:hard-feminine-plural-nominative (сами, not самы).
                (Nominative, Plural, Feminine) => &["и"],
                (Nominative, Plural, Neuter) => &["а"],
                // pron:genitive-accusative and
                // det:hard-feminine-plural-nominative on the plural arms.
                (Accusative, Plural, Masculine | Feminine) => {
                    if animate {
                        &["ѣхъ"]
                    } else {
                        &["и"]
                    }
                }
                (Accusative, Plural, Neuter) => &["а"],
                (Genitive | Locative, Plural, _) => &["ѣхъ"],
                (Dative, Plural, _) => &["ѣмъ"],
                (Instrumental, Plural, _) => &["ѣми"],
                (Vocative, _, _) => NO_TEXTS,
            }
        }
        _ => NO_TEXTS,
    }
}

/// One cell of the totalizing determiner (OCS вьсь, Synodal весь).
///
/// Both recensions lack the dual and the vocative — the paradigm's number
/// inventory is a shared fact, not a divergence. The OCS side ignores
/// `animacy`; the Synodal animate accusatives take the genitive form
/// (divergence `pron:genitive-accusative`). Realization throughout:
/// `gen:jer-medial` (вьс- → вс-), `pron:instr-loc-sg-jer` on -мь/-мъ, and
/// the fold/typography letters (ѩ ~ ѧ, ѭ ~ ю). Beyond those the columns
/// differ by:
///
/// - divergence `det:ves-direct-reshape` (OCS вьса/вьсѣ and neuter plural
///   вьса/вьсѣ against Synodal soft-levelled всѧ; the accusative вьсѫ ~
///   всю pair is realization, `gen:big-yus` + `gen:jer-medial`);
/// - divergence `det:ves-plural-jat-leveling` (OCS вьсѣхъ against Synodal
///   всехъ in the genitive/locative and animate accusative plural, while
///   the dative and instrumental keep ѣ: всѣмъ, всѣми).
#[must_use]
pub fn total_ves_cell(
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};

    let animate = animacy == Animacy::Animate;
    let (ocs, syn): (&[&str], &[&str]) = match (case, number, gender) {
        (Nominative, Singular, Masculine) => (&["вьсь"], &["весь"]),
        // det:ves-direct-reshape (вьса/вьсѣ → всѧ).
        (Nominative, Singular, Feminine) => (&["вьса", "вьсѣ"], &["всѧ"]),
        (Nominative | Accusative, Singular, Neuter) => (&["вьсе"], &["все"]),
        // pron:genitive-accusative on the Synodal animate arm.
        (Accusative, Singular, Masculine) => (
            &["вьсь"],
            if animate {
                &["всего", "всегѡ"]
            } else {
                &["весь"]
            },
        ),
        // realization: gen:big-yus (ѫ → у/ю) + gen:jer-medial (вьсѫ ~ всю).
        (Accusative, Singular, Feminine) => (&["вьсѫ"], &["всю"]),
        (Genitive, Singular, Masculine | Neuter) => (&["вьсего"], &["всего", "всегѡ"]),
        (Genitive, Singular, Feminine) => (&["вьсеѩ"], &["всеѧ"]),
        (Dative, Singular, Masculine | Neuter) => (&["вьсему"], &["всемꙋ"]),
        (Dative | Locative, Singular, Feminine) => (&["вьсеи"], &["всей"]),
        // pron:instr-loc-sg-jer.
        (Instrumental, Singular, Masculine | Neuter) => (&["вьсѣмь"], &["всѣмъ"]),
        (Instrumental, Singular, Feminine) => (&["вьсеѭ"], &["всею"]),
        (Locative, Singular, Masculine | Neuter) => (&["вьсемь"], &["всемъ"]),
        (Nominative, Plural, Masculine) => (&["вьси"], &["вси"]),
        // det:ves-direct-reshape on the neuter plural (вьса/вьсѣ → всѧ).
        (Nominative | Accusative, Plural, Neuter) => (&["вьса", "вьсѣ"], &["всѧ"]),
        (Nominative, Plural, Feminine) => (&["вьсѧ"], &["всѧ"]),
        // pron:genitive-accusative + det:ves-plural-jat-leveling on the
        // Synodal animate arm.
        (Accusative, Plural, Masculine | Feminine) => (
            &["вьсѧ"],
            if animate {
                &["всехъ"]
            } else {
                &["всѧ"]
            },
        ),
        // det:ves-plural-jat-leveling (вьсѣхъ → всехъ).
        (Genitive | Locative, Plural, _) => (&["вьсѣхъ"], &["всехъ"]),
        (Dative, Plural, _) => (&["вьсѣмъ"], &["всѣмъ"]),
        (Instrumental, Plural, _) => (&["вьсѣми"], &["всѣми"]),
        (Vocative, _, _) | (_, Dual, _) => (NO_TEXTS, NO_TEXTS),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// One ending of a velar universal-determiner cell, with the stem treatment
/// it selects.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VelarEnding {
    pub text: &'static str,
    /// Whether the ending attaches to the second-palatalized stem
    /// (вьсац-/всѧц-) rather than the plain velar stem (вьсак-/всѧк-); see
    /// [`crate::pronoun::palatalize_final_velar`].
    pub palatalized: bool,
}

const NO_VELAR: &[VelarEnding] = &[];

macro_rules! ve {
    ($text:literal) => {
        VelarEnding {
            text: $text,
            palatalized: false,
        }
    };
}

macro_rules! vp {
    ($text:literal) => {
        VelarEnding {
            text: $text,
            palatalized: true,
        }
    };
}

/// One short cell of the velar universal determiner (OCS вьсакъ/вьсѣкъ,
/// Synodal всѧкъ).
///
/// The OCS column IS the merged pronoun kernel's hard agreeing class with
/// the positional second palatalization before и/ѣ-initial endings (checked
/// against [`crate::pronoun::agreeing_ending`] by a kernel test); OCS
/// ignores `animacy` and has no vocative. The Synodal column is the Alypy
/// §§45, 48, and 57 всѧкъ mixed short table; its family renders the
/// vocative as the nominative before querying the kernel, so vocatives are
/// empty here. Beyond the hard-class realization rules the columns differ
/// by divergence `det:velar-universal-reshape`, plus
/// `pron:genitive-accusative` on the Synodal animate arms and
/// `pron:instr-loc-sg-jer` on the -мь/-мъ terminals; the stem-grade pair
/// вьсак- ~ всѧк- is part of the same reshape. The Synodal paradigm drops
/// the dual (Alypy §48 excludes it explicitly).
#[must_use]
pub fn velar_universal_short_ending(
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
    recension: Recension,
) -> &'static [VelarEnding] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    let animate = animacy == Animacy::Animate;
    let (ocs, syn): (&[VelarEnding], &[VelarEnding]) = match (case, number, gender) {
        (Nominative, Singular, Masculine) => (&[ve!("ъ")], &[ve!("ъ")]),
        (Nominative, Singular, Feminine) => (&[ve!("а")], &[ve!("а")]),
        (Nominative, Singular, Neuter) => (&[ve!("о")], &[ve!("о")]),
        // det:velar-universal-reshape (long-adjective а-grade genitives with
        // the агѡ/аго and аго/агѡ variant orders against pronominal ого).
        (Genitive, Singular, Masculine | Neuter) => (&[ve!("ого")], &[ve!("агѡ"), ve!("аго")]),
        (Genitive, Singular, Feminine) => (&[ve!("оѩ")], &[ve!("оѧ")]),
        (Dative, Singular, Masculine | Neuter) => (&[ve!("ому")], &[ve!("омꙋ")]),
        // det:velar-universal-reshape (palatalized ѣй beside ой).
        (Dative | Locative, Singular, Feminine) => (&[ve!("ои")], &[vp!("ѣй"), ve!("ой")]),
        // pron:genitive-accusative + det:velar-universal-reshape.
        (Accusative, Singular, Masculine) => (
            &[ve!("ъ")],
            if animate {
                &[ve!("аго"), ve!("агѡ")]
            } else {
                &[ve!("ъ")]
            },
        ),
        // realization: gen:big-yus (ѫ → у, typographic ꙋ).
        (Accusative, Singular, Feminine) => (&[ve!("ѫ")], &[ve!("ꙋ")]),
        (Accusative, Singular, Neuter) => (&[ve!("о")], &[ve!("о")]),
        // pron:instr-loc-sg-jer (both columns palatalize before ѣ).
        (Instrumental, Singular, Masculine | Neuter) => (&[vp!("ѣмь")], &[vp!("ѣмъ")]),
        // realization: gen:iotated-big-yus (оѭ ~ ою).
        (Instrumental, Singular, Feminine) => (&[ve!("оѭ")], &[ve!("ою")]),
        // pron:instr-loc-sg-jer + det:velar-universal-reshape (омъ beside
        // the palatalized ѣмъ).
        (Locative, Singular, Masculine | Neuter) => (&[ve!("омь")], &[ve!("омъ"), vp!("ѣмъ")]),
        // det:velar-universal-reshape (the Synodal paradigm drops the dual).
        (Nominative | Accusative, Dual, Masculine) => (&[ve!("а")], NO_VELAR),
        (Nominative | Accusative, Dual, Feminine | Neuter) => (&[vp!("ѣ")], NO_VELAR),
        (Genitive | Locative, Dual, _) => (&[ve!("ою")], NO_VELAR),
        (Dative | Instrumental, Dual, _) => (&[vp!("ѣма")], NO_VELAR),
        (Nominative, Plural, Masculine) => (&[vp!("и")], &[vp!("ы")]),
        // det:velar-universal-reshape (feminine plural -и against -ы).
        (Nominative, Plural, Feminine) => (&[ve!("ы")], &[ve!("и")]),
        (Nominative, Plural, Neuter) => (&[ve!("а")], &[ve!("а")]),
        // pron:genitive-accusative + det:velar-universal-reshape on the
        // Synodal arms.
        (Accusative, Plural, Masculine | Feminine) => (
            &[ve!("ы")],
            if animate {
                &[vp!("ѣхъ")]
            } else {
                &[ve!("и")]
            },
        ),
        (Accusative, Plural, Neuter) => (&[ve!("а")], &[ve!("а")]),
        (Genitive | Locative, Plural, _) => (&[vp!("ѣхъ")], &[vp!("ѣхъ")]),
        (Dative, Plural, _) => (&[vp!("ѣмъ")], &[vp!("ѣмъ")]),
        (Instrumental, Plural, _) => (&[vp!("ѣми")], &[vp!("ѣми")]),
        (Vocative, _, _) => (NO_VELAR, NO_VELAR),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_VELAR,
    }
}

/// One long cell of the velar universal determiner (Synodal всѧкїй, Alypy
/// §§48 and 57). The OCS вьсакъ is short-only, so the OCS column is empty
/// throughout — the long paradigm is a Synodal lexical fact carried here
/// because the family reads both forms of one lexeme from the kernel.
/// Vocatives and duals follow the short table's conventions.
#[must_use]
pub fn velar_universal_long_ending(
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
    recension: Recension,
) -> &'static [VelarEnding] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    if recension != Recension::SynodalRussian {
        return NO_VELAR;
    }
    let animate = animacy == Animacy::Animate;
    match (case, number, gender) {
        (Nominative, Singular, Masculine) => &[ve!("їй")],
        (Nominative, Singular, Feminine) => &[ve!("аѧ")],
        (Nominative | Accusative, Singular, Neuter) => &[ve!("ое")],
        (Genitive, Singular, Masculine | Neuter) => &[ve!("агѡ")],
        (Genitive, Singular, Feminine) => &[ve!("їѧ")],
        (Dative, Singular, Masculine | Neuter) => &[ve!("омꙋ")],
        (Dative | Locative, Singular, Feminine) => &[vp!("ѣй"), ve!("ой")],
        (Accusative, Singular, Masculine) => {
            if animate {
                &[ve!("аго")]
            } else {
                &[ve!("їй")]
            }
        }
        (Accusative, Singular, Feminine) => &[ve!("ꙋю")],
        (Instrumental, Singular, Masculine | Neuter) => &[ve!("имъ")],
        (Instrumental, Singular, Feminine) => &[ve!("ою")],
        (Locative, Singular, Masculine | Neuter) => &[vp!("ѣмъ"), ve!("омъ")],
        (Nominative, Plural, Masculine) => &[vp!("ыи")],
        (Nominative, Plural, Feminine) => &[ve!("їѧ")],
        (Nominative, Plural, Neuter) => &[ve!("аѧ")],
        (Accusative, Plural, Masculine | Feminine) => {
            if animate {
                &[ve!("ихъ")]
            } else {
                &[ve!("їѧ")]
            }
        }
        (Accusative, Plural, Neuter) => &[ve!("аѧ")],
        (Genitive | Locative, Plural, _) => &[ve!("ихъ")],
        (Dative, Plural, _) => &[ve!("имъ")],
        (Instrumental, Plural, _) => &[ve!("ими")],
        (Vocative, _, _) | (_, Dual, _) => NO_VELAR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::SynodalRussian;

    #[test]
    fn unsupported_recensions_yield_empty_cells() {
        for recension in [Recension::OldRussian, Recension::Mixed, Recension::Unknown] {
            assert!(
                hard_short_ending(
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    recension
                )
                .is_empty()
            );
            assert!(
                total_ves_cell(
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    recension
                )
                .is_empty()
            );
        }
    }

    #[test]
    fn velar_universal_ocs_column_is_the_palatalizing_hard_class() {
        // The OCS velar universal declines through the pronoun kernel's hard
        // class, second-palatalizing before и/ѣ-initial endings; the two
        // copies of that closed table must not drift.
        for case in Case::ALL {
            if case == Case::Vocative {
                continue;
            }
            for number in Number::ALL {
                for gender in Gender::ALL {
                    let hard = crate::pronoun::agreeing_ending(
                        AgreeingClass::Hard,
                        case,
                        number,
                        gender,
                        Animacy::Inanimate,
                        OCS,
                    );
                    let velar: Vec<(&str, bool)> =
                        velar_universal_short_ending(case, number, gender, Animacy::Inanimate, OCS)
                            .iter()
                            .map(|ending| (ending.text, ending.palatalized))
                            .collect();
                    let expected: Vec<(&str, bool)> = hard
                        .iter()
                        .map(|ending| (*ending, ending.starts_with(['и', 'ѣ'])))
                        .collect();
                    assert_eq!(velar, expected, "{case:?} {number:?} {gender:?}");
                }
            }
        }
    }

    #[test]
    fn velar_universal_long_paradigm_is_synodal_only() {
        for case in Case::ALL {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    assert!(
                        velar_universal_long_ending(case, number, gender, Animacy::Inanimate, OCS)
                            .is_empty()
                    );
                }
            }
        }
        assert_eq!(
            velar_universal_long_ending(
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            [VelarEnding {
                text: "агѡ",
                palatalized: false
            }]
        );
    }

    #[test]
    fn hard_ocs_column_is_the_pronoun_hard_class() {
        for case in Case::ALL {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    for animacy in Animacy::ALL {
                        assert_eq!(
                            hard_short_ending(case, number, gender, animacy, OCS),
                            crate::pronoun::agreeing_ending(
                                AgreeingClass::Hard,
                                case,
                                number,
                                gender,
                                animacy,
                                OCS
                            ),
                            "{case:?} {number:?} {gender:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn hard_divergences_hold() {
        // det:hard-oblique-jat-doublets.
        assert_eq!(
            hard_short_ending(
                Case::Dative,
                Number::Singular,
                Gender::Feminine,
                Animacy::Inanimate,
                SYN
            ),
            ["ой", "ѣй"]
        );
        assert_eq!(
            hard_short_ending(
                Case::Locative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["омъ", "ѣмъ"]
        );
        // det:hard-feminine-plural-nominative.
        assert_eq!(
            hard_short_ending(
                Case::Nominative,
                Number::Plural,
                Gender::Feminine,
                Animacy::Inanimate,
                OCS
            ),
            ["ы"]
        );
        assert_eq!(
            hard_short_ending(
                Case::Nominative,
                Number::Plural,
                Gender::Feminine,
                Animacy::Inanimate,
                SYN
            ),
            ["и"]
        );
        // pron:genitive-accusative.
        assert_eq!(
            hard_short_ending(
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate,
                SYN
            ),
            ["ого", "огѡ"]
        );
    }

    #[test]
    fn ves_shares_its_dual_gap_and_diverges_in_the_declared_cells() {
        for recension in [OCS, SYN] {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    assert!(
                        total_ves_cell(case, Number::Dual, gender, Animacy::Inanimate, recension)
                            .is_empty(),
                        "вьсь/весь has no dual in either recension"
                    );
                }
            }
        }
        // det:ves-direct-reshape.
        assert_eq!(
            total_ves_cell(
                Case::Nominative,
                Number::Singular,
                Gender::Feminine,
                Animacy::Inanimate,
                OCS
            ),
            ["вьса", "вьсѣ"]
        );
        assert_eq!(
            total_ves_cell(
                Case::Nominative,
                Number::Singular,
                Gender::Feminine,
                Animacy::Inanimate,
                SYN
            ),
            ["всѧ"]
        );
        // det:ves-plural-jat-leveling.
        assert_eq!(
            total_ves_cell(
                Case::Genitive,
                Number::Plural,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            ),
            ["вьсѣхъ"]
        );
        assert_eq!(
            total_ves_cell(
                Case::Genitive,
                Number::Plural,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["всехъ"]
        );
        assert_eq!(
            total_ves_cell(
                Case::Dative,
                Number::Plural,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["всѣмъ"]
        );
    }
}
