//! The merged numeral inflection kernel (docs/UNIFIED_LANGUAGE_PROMPT.md,
//! execution plan step 4, second POS slice).
//!
//! The merge target is the closed paradigm tables that exist on both sides:
//! the simple cardinals one through four, the five–nine cell system, ten,
//! hundred, and the agreeing collective plural terminals. Every cell is
//! written with both recensions side by side so that a difference is always
//! visibly one of:
//!
//! - **realization** — related by the declared projection rules of
//!   `church-slavonic-orthography::projection` (cited inline by rule id) or
//!   by a named Synodal spelling norm outside that rule set (checked by the
//!   realization-coherence test in the orthography crate);
//! - **a named divergence** — cited inline by its id in
//!   [`crate::divergence::NAMED`];
//! - **a per-recension lexical fact** — which never reaches this module:
//!   the ordinal and higher-collective classes (adjective-coupled), the
//!   noun-backed magnitudes and fractionals, and the value-composition
//!   machinery stay in the family cores (see
//!   [`crate::divergence::UNMERGED`]).
//!
//! The family cores are adapters over these tables: they own their stems,
//! variant statuses, trace plumbing, provenance, and error vocabularies, and
//! they append their own productive supplements (e.g. the OCS i-stem
//! productive forms of ten) after the reviewed kernel rows. Recensions other
//! than the two attested ones yield empty cells.

use crate::pronoun::AgreeingClass;
use crate::{Animacy, Case, Gender, Number, Recension};

const NO_TEXTS: &[&str] = &[];

fn by_recension(
    recension: Recension,
    ocs: &'static [&'static str],
    syn: &'static [&'static str],
) -> &'static [&'static str] {
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// One cell of the cardinal one (OCS ѥдинъ, Synodal єдинъ).
///
/// The OCS column is the stem ѥдин- plus the merged pronoun kernel's hard
/// agreeing class in all three numbers (a kernel test pins the identity);
/// the Synodal column is the Alypy §62 closed table. The OCS side ignores
/// `animacy`. Beyond the pronoun-class realization rules (`gen:big-yus`,
/// `gen:iotated-small-yus`, `gen:iotated-big-yus`, the є/ꙋ typography), the
/// columns differ by:
///
/// - divergence `num:one-long-genitive-shapes` (Synodal єдинагѡ/аго, єдинꙋю,
///   єдиной against the OCS pronominal ого, ѫ, ои);
/// - divergence `num:one-number-inventory` (the Synodal paradigm is
///   singular-only; the OCS dual and plural cells have no Synodal column);
/// - divergences `pron:genitive-accusative` and `pron:instr-loc-sg-jer`.
#[must_use]
pub fn cardinal_one_cell(
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
        (Nominative, Singular, Masculine) => (&["ѥдинъ"], &["єдинъ"]),
        (Nominative, Singular, Feminine) => (&["ѥдина"], &["єдина"]),
        (Nominative, Singular, Neuter) => (&["ѥдино"], &["єдино"]),
        // num:one-long-genitive-shapes (агѡ/аго against pronominal ого).
        (Genitive, Singular, Masculine | Neuter) => (&["ѥдиного"], &["єдинагѡ", "єдинаго"]),
        (Genitive, Singular, Feminine) => (&["ѥдиноѩ"], &["єдиноѧ"]),
        (Dative, Singular, Masculine | Neuter) => (&["ѥдиному"], &["єдиномꙋ"]),
        // num:one-long-genitive-shapes (ой against pronominal ои is the
        // shared realization; the doublet inventory matches).
        (Dative | Locative, Singular, Feminine) => (&["ѥдинои"], &["єдиной"]),
        // pron:genitive-accusative + num:one-long-genitive-shapes.
        (Accusative, Singular, Masculine) => (
            &["ѥдинъ"],
            if animate {
                &["єдинаго", "єдинагѡ"]
            } else {
                &["єдинъ"]
            },
        ),
        // num:one-long-genitive-shapes (long єдинꙋю against pronominal ѫ).
        (Accusative, Singular, Feminine) => (&["ѥдинѫ"], &["єдинꙋю"]),
        (Accusative, Singular, Neuter) => (&["ѥдино"], &["єдино"]),
        // pron:instr-loc-sg-jer.
        (Instrumental, Singular, Masculine | Neuter) => (&["ѥдинѣмь"], &["єдинѣмъ"]),
        (Instrumental, Singular, Feminine) => (&["ѥдиноѭ"], &["єдиною"]),
        (Locative, Singular, Masculine | Neuter) => (&["ѥдиномь"], &["єдиномъ"]),
        // num:one-number-inventory: the OCS dual and plural have no Synodal
        // column.
        (Nominative | Accusative, Dual, Masculine) => (&["ѥдина"], NO_TEXTS),
        (Nominative | Accusative, Dual, Feminine | Neuter) => (&["ѥдинѣ"], NO_TEXTS),
        (Genitive | Locative, Dual, _) => (&["ѥдиною"], NO_TEXTS),
        (Dative | Instrumental, Dual, _) => (&["ѥдинѣма"], NO_TEXTS),
        (Nominative, Plural, Masculine) => (&["ѥдини"], NO_TEXTS),
        (Nominative | Accusative, Plural, Feminine) | (Accusative, Plural, Masculine) => {
            (&["ѥдины"], NO_TEXTS)
        }
        (Nominative | Accusative, Plural, Neuter) => (&["ѥдина"], NO_TEXTS),
        (Genitive | Locative, Plural, _) => (&["ѥдинѣхъ"], NO_TEXTS),
        (Dative, Plural, _) => (&["ѥдинѣмъ"], NO_TEXTS),
        (Instrumental, Plural, _) => (&["ѥдинѣми"], NO_TEXTS),
        (Vocative, _, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn)
}

/// The two dual-only paired cardinals.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PairedCardinal {
    Two,
    Both,
}

/// One (dual-only) cell of два/оба. The OCS column is the stems дъв-/об-
/// plus the hard agreeing dual (realization `gen:jer-medial` дъва → два,
/// `fold:omega` on ою); the Synodal column is the Alypy §62 table.
/// Divergence `num:two-genitive-u-doublet`: Synodal два adds the
/// genitive/locative doublet двꙋ beside двою, while оба keeps only обою.
#[must_use]
pub fn paired_cardinal_cell(
    paradigm: PairedCardinal,
    case: Case,
    gender: Gender,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};

    let (ocs, syn): (&[&str], &[&str]) = match (paradigm, case, gender) {
        (PairedCardinal::Two, Nominative | Accusative, Masculine) => (&["дъва"], &["два"]),
        (PairedCardinal::Two, Nominative | Accusative, Feminine | Neuter) => (&["дъвѣ"], &["двѣ"]),
        // num:two-genitive-u-doublet.
        (PairedCardinal::Two, Genitive | Locative, _) => (&["дъвою"], &["двою", "двꙋ"]),
        (PairedCardinal::Two, Dative | Instrumental, _) => (&["дъвѣма"], &["двѣма"]),
        (PairedCardinal::Both, Nominative | Accusative, Masculine) => (&["оба"], &["оба"]),
        (PairedCardinal::Both, Nominative | Accusative, Feminine | Neuter) => (&["обѣ"], &["обѣ"]),
        (PairedCardinal::Both, Genitive | Locative, _) => (&["обою"], &["обою"]),
        (PairedCardinal::Both, Dative | Instrumental, _) => (&["обѣма"], &["обѣма"]),
        (_, Vocative, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn)
}

/// One (plural-only) cell of the cardinal three. The OCS side ignores
/// `animacy`. Divergences:
///
/// - `num:three-oblique-reinventory`: the OCS genitive трии is distinct from
///   the locative трьхъ; Synodal syncretizes genitive and locative in -хъ
///   and co-lists the masculine трїе- doublet series through the obliques;
/// - `pron:genitive-accusative` on the Synodal masculine animate arm.
///
/// Realization: `gen:jer-medial` vocalizes the trь- ~ тре- pair.
#[must_use]
pub fn cardinal_three_cell(
    case: Case,
    gender: Gender,
    animacy: Animacy,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};

    let animate = animacy == Animacy::Animate;
    let (ocs, syn): (&[&str], &[&str]) = match (case, gender) {
        (Nominative, Masculine) => (&["триѥ"], &["трїе", "три"]),
        (Nominative, Feminine | Neuter) => (&["три"], &["три"]),
        // pron:genitive-accusative on the Synodal masculine animate arm.
        (Accusative, Masculine) => (
            &["три"],
            if animate {
                &["трїехъ", "трехъ", "три"]
            } else {
                &["три"]
            },
        ),
        (Accusative, Feminine | Neuter) => (&["три"], &["три"]),
        // num:three-oblique-reinventory throughout the obliques.
        (Genitive, Masculine) => (&["трии"], &["трїехъ", "трехъ"]),
        (Genitive, Feminine | Neuter) => (&["трии"], &["трехъ"]),
        (Locative, Masculine) => (&["трьхъ"], &["трїехъ", "трехъ"]),
        (Locative, Feminine | Neuter) => (&["трьхъ"], &["трехъ"]),
        (Dative, Masculine) => (&["трьмъ"], &["трїемъ", "тремъ"]),
        (Dative, Feminine | Neuter) => (&["трьмъ"], &["тремъ"]),
        (Instrumental, Masculine) => (&["трьми"], &["трїеми", "треми"]),
        (Instrumental, Feminine | Neuter) => (&["трьми"], &["треми"]),
        (Vocative, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn)
}

/// One (plural-only) cell of the cardinal four. Divergence
/// `num:four-oblique-reinventory`: the OCS genitive четыръ against the
/// Synodal genitive-locative syncretism четырехъ, and the Synodal direct
/// cells co-list четыре/четыри doublets where OCS keeps one gendered form.
#[must_use]
pub fn cardinal_four_cell(
    case: Case,
    gender: Gender,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};

    let (ocs, syn): (&[&str], &[&str]) = match (case, gender) {
        // num:four-oblique-reinventory (Synodal direct doublets).
        (Nominative, Masculine) => (&["четыре"], &["четыре", "четыри"]),
        (Nominative, Feminine | Neuter) => (&["четыри"], &["четыри", "четыре"]),
        (Accusative, _) => (&["четыри"], &["четыри", "четыре"]),
        // num:four-oblique-reinventory (четыръ vs четырехъ).
        (Genitive, _) => (&["четыръ"], &["четырехъ"]),
        (Locative, _) => (&["четырехъ"], &["четырехъ"]),
        (Dative, _) => (&["четыремъ"], &["четыремъ"]),
        (Instrumental, _) => (&["четырьми"], &["четырьми"]),
        (Vocative, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn)
}

/// The ending of one plural oblique cell of the i-stem cardinals five
/// through nine. Divergence `num:five-nine-plural-obliques`: the OCS
/// paradigm is a singular-only i-stem noun (the OCS column is empty and the
/// OCS family serves only singular noun cells); Synodal adds the adjectival
/// plural obliques -ихъ (genitive/locative) and -имъ (dative).
#[must_use]
pub fn i_stem_cardinal_plural_oblique_ending(
    case: Case,
    recension: Recension,
) -> &'static [&'static str] {
    let syn: &[&str] = match case {
        Case::Genitive | Case::Locative => &["ихъ"],
        Case::Dative => &["имъ"],
        _ => NO_TEXTS,
    };
    by_recension(recension, NO_TEXTS, syn)
}

/// The reviewed rows of one cell of the cardinal ten, ordered
/// source-primary-first. The OCS family appends its productive i-stem
/// supplement after these rows; empty OCS cells (singular genitive/dative,
/// dual genitive/locative) are served productively by the family alone.
/// Realization: `gen:iotated-big-yus` and the ї typography on десѧтиѭ ~
/// десѧтїю, `fold:uk` on десѧту ~ десѧтꙋ. Divergence
/// `num:ten-oblique-reinventory`: the plural instrumental десѧты against
/// десѧтьми, the Synodal -ихъ/-имъ/-ихъ adjectival doublets and accusative
/// десѧте, and the reviewed Synodal singular obliques where OCS has only
/// productive forms.
#[must_use]
pub fn cardinal_ten_cell(
    case: Case,
    number: Number,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Number::{Dual, Plural, Singular};

    let (ocs, syn): (&[&str], &[&str]) = match (case, number) {
        (Nominative, Singular) => (&["десѧть"], &["десѧть"]),
        // num:ten-oblique-reinventory (Synodal reviewed accusative doublet).
        (Accusative, Singular) => (&["десѧть"], &["десѧть", "десѧте"]),
        (Genitive | Dative, Singular) => (NO_TEXTS, &["десѧти"]),
        (Locative, Singular) => (&["десѧти", "десѧте"], &["десѧти"]),
        // realization: gen:iotated-big-yus + ї typography.
        (Instrumental, Singular) => (&["десѧтиѭ"], &["десѧтїю"]),
        (Nominative | Accusative, Dual) => (&["десѧти", "десѧтѣ"], &["десѧти", "десѧтѣ"]),
        // realization: fold:uk (the OCS reviewed dual genitive lives in the
        // counted-ten construction десѧту, family-side).
        (Genitive | Locative, Dual) => (NO_TEXTS, &["десѧтꙋ"]),
        (Dative | Instrumental, Dual) => (&["десѧтьма"], &["десѧтьма"]),
        // num:ten-oblique-reinventory throughout the plural.
        (Nominative, Plural) => (&["десѧте"], &["десѧти", "десѧте"]),
        (Accusative, Plural) => (&["десѧти"], &["десѧти", "десѧте"]),
        (Genitive, Plural) => (&["десѧтъ"], &["десѧтъ", "десѧтихъ"]),
        (Dative, Plural) => (&["десѧтемъ", "десѧтьмъ"], &["десѧтемъ", "десѧтимъ"]),
        (Instrumental, Plural) => (&["десѧты"], &["десѧтьми"]),
        (Locative, Plural) => (&["десѧтехъ"], &["десѧтехъ", "десѧтихъ"]),
        (Vocative, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn)
}

/// One cell of the cardinal hundred (OCS съто, Synodal сто). Both columns
/// are the inherited neuter o-stem; the OCS column equals the family noun
/// kernel's o-neuter output (pinned family-side). Realization:
/// `gen:jer-medial` (сът- → ст-), `fold:uk` (сътоу ~ стꙋ), `gen:yery`
/// (сътꙑ ~ сты), and `gen:jer-medial`'s strong-jer vocalization on the
/// plural genitive (сътъ ~ сотъ). Vocatives are construction-invalid on the Synodal side
/// and family-served on the OCS side; the kernel leaves them empty.
#[must_use]
pub fn cardinal_hundred_cell(
    case: Case,
    number: Number,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Number::{Dual, Plural, Singular};

    let (ocs, syn): (&[&str], &[&str]) = match (case, number) {
        (Nominative | Accusative, Singular) => (&["съто"], &["сто"]),
        (Genitive, Singular) => (&["съта"], &["ста"]),
        (Dative, Singular) => (&["сътоу"], &["стꙋ"]),
        (Instrumental, Singular) => (&["сътомъ"], &["стомъ"]),
        (Locative, Singular) => (&["сътѣ"], &["стѣ"]),
        (Nominative | Accusative, Dual) => (&["сътѣ"], &["стѣ"]),
        (Genitive | Locative, Dual) => (&["сътоу"], &["стꙋ"]),
        (Dative | Instrumental, Dual) => (&["сътома"], &["стома"]),
        (Nominative | Accusative, Plural) => (&["съта"], &["ста"]),
        // realization: gen:jer-medial (сътъ ~ сотъ).
        (Genitive, Plural) => (&["сътъ"], &["сотъ"]),
        (Dative, Plural) => (&["сътомъ"], &["стомъ"]),
        (Instrumental, Plural) => (&["сътꙑ"], &["сты"]),
        (Locative, Plural) => (&["сътѣхъ"], &["стѣхъ"]),
        (Vocative, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn)
}

/// The ending of one agreeing-collective plural cell (OCS дъвои/обои/трои,
/// Synodal двои). The OCS column is the merged pronoun kernel's soft-J
/// plural (the OCS collectives decline through the full `2/p` J class; only
/// the plural overlaps the Synodal plural-only paradigm — divergence
/// `num:collective-agreeing-reshape` covers the number inventory, the
/// feminine nominative -и against -ѩ, the inanimate accusative -и against
/// -ѩ/-ꙗ, and the Synodal licensed vocative, which the OCS class lacks).
#[must_use]
pub fn collective_agreeing_plural_ending(
    case: Case,
    gender: Gender,
    animacy: Animacy,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};

    match recension {
        Recension::OldChurchSlavonic => crate::pronoun::agreeing_ending(
            AgreeingClass::SoftJ,
            case,
            Number::Plural,
            gender,
            animacy,
            recension,
        ),
        Recension::SynodalRussian => {
            let animate = animacy == Animacy::Animate;
            match (case, gender) {
                // num:collective-agreeing-reshape (feminine -и, vocative).
                (Nominative | Vocative, Masculine | Feminine) => &["и"],
                (Nominative | Vocative, Neuter) => &["ѧ"],
                (Genitive | Locative, _) => &["ихъ"],
                (Dative, _) => &["имъ"],
                // pron:genitive-accusative + num:collective-agreeing-reshape.
                (Accusative, Masculine | Feminine) => {
                    if animate {
                        &["ихъ"]
                    } else {
                        &["и"]
                    }
                }
                (Accusative, Neuter) => &["ѧ"],
                (Instrumental, _) => &["ими"],
            }
        }
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
                cardinal_one_cell(
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    recension
                )
                .is_empty()
            );
            assert!(cardinal_ten_cell(Case::Nominative, Number::Singular, recension).is_empty());
            assert!(
                collective_agreeing_plural_ending(
                    Case::Nominative,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    recension
                )
                .is_empty()
            );
        }
    }

    #[test]
    fn one_ocs_column_is_the_stem_plus_the_hard_pronoun_class() {
        for case in Case::ALL {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    let expected: Vec<String> = crate::pronoun::agreeing_ending(
                        AgreeingClass::Hard,
                        case,
                        number,
                        gender,
                        Animacy::Inanimate,
                        OCS,
                    )
                    .iter()
                    .map(|ending| format!("ѥдин{ending}"))
                    .collect();
                    let actual: Vec<String> =
                        cardinal_one_cell(case, number, gender, Animacy::Inanimate, OCS)
                            .iter()
                            .map(|text| (*text).to_owned())
                            .collect();
                    assert_eq!(actual, expected, "{case:?} {number:?} {gender:?}");
                }
            }
        }
    }

    #[test]
    fn one_number_inventory_divergence_holds() {
        // num:one-number-inventory.
        assert!(
            !cardinal_one_cell(
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            )
            .is_empty()
        );
        assert!(
            cardinal_one_cell(
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            )
            .is_empty()
        );
        // num:one-long-genitive-shapes.
        assert_eq!(
            cardinal_one_cell(
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["єдинагѡ", "єдинаго"]
        );
    }

    #[test]
    fn paired_cardinal_u_doublet_is_two_only() {
        // num:two-genitive-u-doublet.
        assert_eq!(
            paired_cardinal_cell(PairedCardinal::Two, Case::Genitive, Gender::Masculine, SYN),
            ["двою", "двꙋ"]
        );
        assert_eq!(
            paired_cardinal_cell(PairedCardinal::Both, Case::Genitive, Gender::Masculine, SYN),
            ["обою"]
        );
        assert_eq!(
            paired_cardinal_cell(PairedCardinal::Two, Case::Genitive, Gender::Masculine, OCS),
            ["дъвою"]
        );
    }

    #[test]
    fn three_and_four_oblique_reinventories_hold() {
        // num:three-oblique-reinventory.
        assert_eq!(
            cardinal_three_cell(Case::Genitive, Gender::Feminine, Animacy::Inanimate, OCS),
            ["трии"]
        );
        assert_eq!(
            cardinal_three_cell(Case::Genitive, Gender::Feminine, Animacy::Inanimate, SYN),
            ["трехъ"]
        );
        assert_eq!(
            cardinal_three_cell(Case::Accusative, Gender::Masculine, Animacy::Animate, SYN),
            ["трїехъ", "трехъ", "три"]
        );
        // num:four-oblique-reinventory.
        assert_eq!(
            cardinal_four_cell(Case::Genitive, Gender::Masculine, OCS),
            ["четыръ"]
        );
        assert_eq!(
            cardinal_four_cell(Case::Genitive, Gender::Masculine, SYN),
            ["четырехъ"]
        );
    }

    #[test]
    fn five_nine_plural_obliques_are_synodal_only() {
        // num:five-nine-plural-obliques.
        for case in Case::ALL {
            assert!(i_stem_cardinal_plural_oblique_ending(case, OCS).is_empty());
        }
        assert_eq!(
            i_stem_cardinal_plural_oblique_ending(Case::Genitive, SYN),
            ["ихъ"]
        );
        assert_eq!(
            i_stem_cardinal_plural_oblique_ending(Case::Dative, SYN),
            ["имъ"]
        );
        assert!(i_stem_cardinal_plural_oblique_ending(Case::Accusative, SYN).is_empty());
    }

    #[test]
    fn ten_plural_instrumental_diverges() {
        // num:ten-oblique-reinventory.
        assert_eq!(
            cardinal_ten_cell(Case::Instrumental, Number::Plural, OCS),
            ["десѧты"]
        );
        assert_eq!(
            cardinal_ten_cell(Case::Instrumental, Number::Plural, SYN),
            ["десѧтьми"]
        );
        assert_eq!(
            cardinal_ten_cell(Case::Genitive, Number::Plural, SYN),
            ["десѧтъ", "десѧтихъ"]
        );
    }

    #[test]
    fn hundred_columns_are_the_shared_o_stem() {
        assert_eq!(
            cardinal_hundred_cell(Case::Genitive, Number::Plural, OCS),
            ["сътъ"]
        );
        assert_eq!(
            cardinal_hundred_cell(Case::Genitive, Number::Plural, SYN),
            ["сотъ"]
        );
        assert_eq!(
            cardinal_hundred_cell(Case::Dative, Number::Singular, SYN),
            ["стꙋ"]
        );
    }

    #[test]
    fn collective_agreeing_reshape_holds() {
        // num:collective-agreeing-reshape.
        assert_eq!(
            collective_agreeing_plural_ending(
                Case::Nominative,
                Gender::Feminine,
                Animacy::Inanimate,
                OCS
            ),
            ["ѩ"]
        );
        assert_eq!(
            collective_agreeing_plural_ending(
                Case::Nominative,
                Gender::Feminine,
                Animacy::Inanimate,
                SYN
            ),
            ["и"]
        );
        assert!(
            collective_agreeing_plural_ending(
                Case::Vocative,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            )
            .is_empty()
        );
        assert_eq!(
            collective_agreeing_plural_ending(
                Case::Vocative,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["и"]
        );
    }
}
