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
