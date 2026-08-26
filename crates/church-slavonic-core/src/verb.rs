//! The merged present-system verb inflection kernel
//! (docs/UNIFIED_LANGUAGE_PROMPT.md, execution plan step 4, fifth POS
//! slice; the past tenses and the copula live in [`crate::verb_past`], the
//! participle stem formations in [`crate::verb_participle`]).
//!
//! One recension-conditioned ending table per shared conjugational series.
//! Every cell is written with both recensions side by side so that a
//! difference is always visibly one of:
//!
//! - **realization** — related by the declared projection rules of
//!   `church-slavonic-orthography::projection` (cited inline by rule id,
//!   e.g. `gen:big-yus`, `gen:iotated-e`, `fold:uk`) or by a named Synodal
//!   spelling norm outside that rule set (checked by the
//!   realization-coherence test in the orthography crate);
//! - **a named divergence** — cited inline by its id in
//!   [`crate::divergence::NAMED`];
//! - **a per-recension lexical fact** — which never reaches this module:
//!   the Synodal suppletive first-singular/third-plural principal parts,
//!   both families' stem selection (iotation, palatalization seams), the
//!   reviewed irregular identity inventories, and the reflexive/periphrase
//!   constructions stay in the family cores (see
//!   [`crate::divergence::UNMERGED`]).
//!
//! The family cores are adapters over these tables: they own their lexeme
//! interfaces, principal-part validation, error vocabularies, and trace
//! provenance, and they read their own recension's column through thin
//! shims. Recensions other than the two attested ones yield empty cells.
//!
//! Each recension's column stores that family's canonical kernel spelling
//! (OCS ѫ/ѭ/ѧ/ѥ/ꙑ/ѣ; Synodal е/и/ѧ/ꙋ/й), because the columns feed the
//! family engines directly. The OCS column lists Polivanova's productive
//! terminal per populated cell; the Synodal column lists the Alypy ending
//! (variant sets appear only where Alypy prints ordered variants).

use crate::grammar::{Gender, Number, Person};
use crate::recension::Recension;

const NO_TEXTS: &[&str] = &[];

/// The present-tense conjugational series shared by both recensions.
///
/// OCS `IA1`/`IA2` read the hard or iotated first-conjugation column
/// depending on their explicit formation; OCS `II1`–`II3` read the soft or
/// hard-i second-conjugation column. The Synodal family's two productive
/// conjugations read the same columns (its iotation lives in the supplied
/// stem, so its first-conjugation endings are the plain e-series).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PresentSeries {
    /// Polivanova's first conjugation with the hard thematic vowel
    /// (несеши) ↔ Alypy §80 first conjugation.
    FirstHard,
    /// The OCS iotated first-conjugation print (знаѥши); the Synodal
    /// column is the same e-series because iotation lives in the stem
    /// (realization `gen:iotated-e` on every shared cell).
    FirstIotated,
    /// The second conjugation with the iotated first singular ѭ
    /// (хвалиши) ↔ Alypy §80 second conjugation.
    SecondSoft,
    /// The OCS second-conjugation subtype with the unpalatalized first
    /// singular ѫ; identical to [`Self::SecondSoft`] outside that cell.
    SecondHardI,
}

impl PresentSeries {
    pub const ALL: [Self; 4] = [
        Self::FirstHard,
        Self::FirstIotated,
        Self::SecondSoft,
        Self::SecondHardI,
    ];
}

/// One present-tense ending cell. The OCS column covers all nine cells;
/// the Synodal column leaves the first singular and third plural empty
/// because the family treats them as suppletive principal parts
/// (`unmerged:verb:synodal-suppletive-present-edges`).
#[must_use]
pub fn present_ending(
    series: PresentSeries,
    person: Person,
    number: Number,
    recension: Recension,
) -> &'static [&'static str] {
    use Number::{Dual, Plural, Singular};
    use Person::{First, Second, Third};
    use PresentSeries::{FirstHard, FirstIotated, SecondHardI, SecondSoft};
    let (ocs, syn): (&[&str], &[&str]) = match (series, person, number) {
        // ---- first conjugation, hard thematic vowel ----
        // The Synodal first singular and third plural are suppletive
        // principal parts (see the module docs); realization gen:big-yus
        // would otherwise relate ѫ ~ ꙋ/ю.
        (FirstHard, First, Singular) => (&["ѫ"], NO_TEXTS),
        (FirstHard, Second, Singular) => (&["еши"], &["еши"]),
        (FirstHard, Third, Singular) => (&["етъ"], &["етъ"]),
        // verb:dual-first-person-va
        (FirstHard, First, Dual) => (&["евѣ"], &["ева"]),
        (FirstHard, Second, Dual) => (&["ета"], &["ета"]),
        // verb:dual-third-person-leveling
        (FirstHard, Third, Dual) => (&["ете"], &["ета"]),
        (FirstHard, First, Plural) => (&["емъ"], &["емъ"]),
        (FirstHard, Second, Plural) => (&["ете"], &["ете"]),
        (FirstHard, Third, Plural) => (&["ѫтъ"], NO_TEXTS),
        // ---- first conjugation, iotated print ----
        // Realization gen:iotated-e on every shared cell (ѥ ~ е); the
        // Synodal stem supplies the glide, so its column is the e-series.
        (FirstIotated, First, Singular) => (&["ѭ"], NO_TEXTS),
        (FirstIotated, Second, Singular) => (&["ѥши"], &["еши"]),
        (FirstIotated, Third, Singular) => (&["ѥтъ"], &["етъ"]),
        // verb:dual-first-person-va
        (FirstIotated, First, Dual) => (&["ѥвѣ"], &["ева"]),
        (FirstIotated, Second, Dual) => (&["ѥта"], &["ета"]),
        // verb:dual-third-person-leveling
        (FirstIotated, Third, Dual) => (&["ѥте"], &["ета"]),
        (FirstIotated, First, Plural) => (&["ѥмъ"], &["емъ"]),
        (FirstIotated, Second, Plural) => (&["ѥте"], &["ете"]),
        (FirstIotated, Third, Plural) => (&["ѭтъ"], NO_TEXTS),
        // ---- second conjugation ----
        (SecondSoft, First, Singular) => (&["ѭ"], NO_TEXTS),
        (SecondHardI, First, Singular) => (&["ѫ"], NO_TEXTS),
        (SecondSoft | SecondHardI, Second, Singular) => (&["иши"], &["иши"]),
        (SecondSoft | SecondHardI, Third, Singular) => (&["итъ"], &["итъ"]),
        // verb:dual-first-person-va
        (SecondSoft | SecondHardI, First, Dual) => (&["ивѣ"], &["ива"]),
        (SecondSoft | SecondHardI, Second, Dual) => (&["ита"], &["ита"]),
        // verb:dual-third-person-leveling
        (SecondSoft | SecondHardI, Third, Dual) => (&["ите"], &["ита"]),
        (SecondSoft | SecondHardI, First, Plural) => (&["имъ"], &["имъ"]),
        (SecondSoft | SecondHardI, Second, Plural) => (&["ите"], &["ите"]),
        (SecondSoft | SecondHardI, Third, Plural) => (&["ѧтъ"], NO_TEXTS),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// The imperative formation series shared by (or restricted to) one
/// recension. The second/third singular cell is shared -и across the whole
/// vowel-grade axis; the non-singular cells carry the divergences.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImperativeSeries {
    /// The i-grade series (both recensions' second conjugation and the
    /// OCS `ISeries` ↔ the Synodal `ISeries`).
    I,
    /// The OCS yat-grade first-conjugation series against the Synodal
    /// e/i-grade first-conjugation series
    /// (divergence `verb:imperative-vowel-grade`).
    EGrade,
    /// The Synodal contracted j-series on vowel-final stems (Alypy §93);
    /// OCS has no counterpart (divergence `verb:imperative-vowel-grade`
    /// names the axis, the J column is Synodal-only).
    J,
}

impl ImperativeSeries {
    pub const ALL: [Self; 3] = [Self::I, Self::EGrade, Self::J];
}

/// One imperative ending cell. The valid cell inventory (2/3 singular,
/// 1/2 dual, 1/2 plural) is shared by both recensions; other cells are
/// empty in both columns.
#[must_use]
pub fn imperative_ending(
    series: ImperativeSeries,
    person: Person,
    number: Number,
    recension: Recension,
) -> &'static [&'static str] {
    use ImperativeSeries::{EGrade, I, J};
    use Number::{Dual, Plural, Singular};
    use Person::{First, Second, Third};
    let (ocs, syn): (&[&str], &[&str]) = match (series, person, number) {
        (I | EGrade, Second | Third, Singular) => (&["и"], &["и"]),
        // verb:dual-first-person-va
        (I, First, Dual) => (&["ивѣ"], &["ива"]),
        (I, Second, Dual) => (&["ита"], &["ита"]),
        (I, First, Plural) => (&["имъ"], &["имъ"]),
        (I, Second, Plural) => (&["ите"], &["ите"]),
        // verb:imperative-vowel-grade (with verb:dual-first-person-va in
        // the first dual)
        (EGrade, First, Dual) => (&["ѣвѣ"], &["ева"]),
        (EGrade, Second, Dual) => (&["ѣта"], &["ита"]),
        (EGrade, First, Plural) => (&["ѣмъ"], &["емъ"]),
        (EGrade, Second, Plural) => (&["ѣте"], &["ите"]),
        // The Synodal-only contracted series (Alypy §93).
        (J, Second | Third, Singular) => (NO_TEXTS, &["й"]),
        (J, First, Dual) => (NO_TEXTS, &["йва"]),
        (J, Second, Dual) => (NO_TEXTS, &["йта"]),
        (J, First, Plural) => (NO_TEXTS, &["ймъ"]),
        (J, Second, Plural) => (NO_TEXTS, &["йте"]),
        _ => (NO_TEXTS, NO_TEXTS),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// One l-participle (resultative) agreement ending. OCS keeps the gendered
/// dual and plural cells; Synodal levels the feminine/neuter dual and the
/// whole plural to -ли (divergence `verb:l-participle-leveling`).
#[must_use]
pub fn l_participle_ending(
    gender: Gender,
    number: Number,
    recension: Recension,
) -> &'static [&'static str] {
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    let (ocs, syn): (&[&str], &[&str]) = match (gender, number) {
        (Masculine, Singular) => (&["лъ"], &["лъ"]),
        (Feminine, Singular) => (&["ла"], &["ла"]),
        (Neuter, Singular) => (&["ло"], &["ло"]),
        (Masculine, Dual) => (&["ла"], &["ла"]),
        // verb:l-participle-leveling
        (Feminine | Neuter, Dual) => (&["лѣ"], &["ли"]),
        (Masculine, Plural) => (&["ли"], &["ли"]),
        // verb:l-participle-leveling
        (Feminine, Plural) => (&["лꙑ"], &["ли"]),
        // verb:l-participle-leveling
        (Neuter, Plural) => (&["ла"], &["ли"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

#[cfg(test)]
mod tests {
    use super::{ImperativeSeries, PresentSeries, imperative_ending, present_ending};
    use crate::grammar::{Number, Person};
    use crate::recension::Recension;

    #[test]
    fn ocs_present_columns_are_total_and_synodal_edges_are_suppletive() {
        for series in PresentSeries::ALL {
            for person in Person::ALL {
                for number in Number::ALL {
                    let ocs = present_ending(series, person, number, Recension::OldChurchSlavonic);
                    assert!(!ocs.is_empty(), "{series:?} {person:?} {number:?}");
                    let syn = present_ending(series, person, number, Recension::SynodalRussian);
                    let suppletive_edge = (person, number) == (Person::First, Number::Singular)
                        || (person, number) == (Person::Third, Number::Plural);
                    assert_eq!(
                        syn.is_empty(),
                        suppletive_edge,
                        "{series:?} {person:?} {number:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn imperative_cell_inventory_is_shared_and_j_series_is_synodal_only() {
        for person in Person::ALL {
            for number in Number::ALL {
                let valid = matches!(
                    (person, number),
                    (Person::Second | Person::Third, Number::Singular)
                        | (
                            Person::First | Person::Second,
                            Number::Dual | Number::Plural
                        )
                );
                for series in ImperativeSeries::ALL {
                    let ocs =
                        imperative_ending(series, person, number, Recension::OldChurchSlavonic);
                    let syn = imperative_ending(series, person, number, Recension::SynodalRussian);
                    if series == ImperativeSeries::J {
                        assert!(ocs.is_empty());
                        assert_eq!(syn.is_empty(), !valid);
                    } else {
                        assert_eq!(ocs.is_empty(), !valid);
                        assert_eq!(syn.is_empty(), !valid);
                    }
                }
            }
        }
    }

    #[test]
    fn other_recensions_yield_empty_cells() {
        assert!(
            present_ending(
                PresentSeries::FirstHard,
                Person::Second,
                Number::Singular,
                Recension::OldRussian,
            )
            .is_empty()
        );
    }
}
