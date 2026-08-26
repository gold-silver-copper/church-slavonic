//! The merged past-tense verb kernel: imperfect, aorist, and the closed
//! copula tables (docs/UNIFIED_LANGUAGE_PROMPT.md, execution plan step 4,
//! fifth POS slice; the present system lives in [`crate::verb`], the
//! participle stem formations in [`crate::verb_participle`]).
//!
//! The imperfect splits into two orthogonal tables mirroring both family
//! engines: a tense-marker table (the OCS contraction axis lives in the
//! ordered OCS variant column: uncontracted first, contracted second) and a
//! personal-ending table. The aorist is one series-conditioned ending
//! table; series missing from a recension yield empty columns — the
//! asymmetry itself is the named divergence `verb:aorist-inventory`.
//!
//! The copula бꙑти ~ бы́ти is the one suppletive paradigm attested closed
//! on both sides (Polivanova §§538–549 / UT OCS Online §§24, 27 against
//! Alypy §81 backed by `data/synodal/exact_forms.tsv`), so its tables merge
//! here in the pronoun-slice closed-system style; the remaining reviewed
//! irregular inventories stay family-side as lexical facts
//! (`unmerged:verb:irregular-identity-inventories`).

use crate::grammar::{Number, Person};
use crate::recension::Recension;

const NO_TEXTS: &[&str] = &[];

/// The imperfect tense markers shared by (or restricted to) one recension.
/// The OCS column lists the ordered uncontracted/contracted variants; the
/// Synodal column is the single Alypy §87 contracted grade.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ImperfectMarker {
    /// OCS -а-/-∅- (plain and present-stem formations) ↔ the Synodal bare
    /// marker (Alypy's -хъ series); the shared surface is the contracted
    /// zero grade (realization).
    A,
    /// OCS -ѣа-/-ѣ- against Synodal -ѧ- (divergence
    /// `verb:imperfect-contraction`).
    YatA,
    /// OCS -аа-/-а- (after a palatalized velar) against Synodal -а-; the
    /// contracted grade is shared (realization).
    PalatalizedA,
    /// The OCS iotated print -ꙗ- against Synodal -ѧ- (realization
    /// `fold:ja`).
    IotatedA,
    /// The OCS iotated yat print -ѣꙗ- against Synodal -ѧ- (divergence
    /// `verb:imperfect-contraction`).
    IotatedYatA,
    /// The OCS iotated palatalized print -аꙗ- against Synodal -ѧ-
    /// (divergence `verb:imperfect-contraction`).
    IotatedPalatalizedA,
}

impl ImperfectMarker {
    pub const ALL: [Self; 6] = [
        Self::A,
        Self::YatA,
        Self::PalatalizedA,
        Self::IotatedA,
        Self::IotatedYatA,
        Self::IotatedPalatalizedA,
    ];
}

/// The tense-marker segment between the imperfect stem and the personal
/// ending.
#[must_use]
pub fn imperfect_marker(marker: ImperfectMarker, recension: Recension) -> &'static [&'static str] {
    use ImperfectMarker::{A, IotatedA, IotatedPalatalizedA, IotatedYatA, PalatalizedA, YatA};
    let (ocs, syn): (&[&str], &[&str]) = match marker {
        A => (&["а", ""], &[""]),
        // verb:imperfect-contraction
        YatA => (&["ѣа", "ѣ"], &["ѧ"]),
        PalatalizedA => (&["аа", "а"], &["а"]),
        // realization: fold:ja on ꙗ ~ ѧ
        IotatedA => (&["ꙗ"], &["ѧ"]),
        // verb:imperfect-contraction
        IotatedYatA => (&["ѣꙗ"], &["ѧ"]),
        // verb:imperfect-contraction
        IotatedPalatalizedA => (&["аꙗ"], &["ѧ"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// One imperfect personal ending, attached after the tense marker.
#[must_use]
pub fn imperfect_personal_ending(
    person: Person,
    number: Number,
    recension: Recension,
) -> &'static [&'static str] {
    use Number::{Dual, Plural, Singular};
    use Person::{First, Second, Third};
    let (ocs, syn): (&[&str], &[&str]) = match (person, number) {
        (First, Singular) => (&["хъ"], &["хъ"]),
        (Second | Third, Singular) => (&["ше"], &["ше"]),
        // verb:dual-first-person-va
        (First, Dual) => (&["ховѣ"], &["хова"]),
        // verb:imperfect-hardening
        (Second, Dual) => (&["шета"], &["ста"]),
        // verb:imperfect-hardening + verb:dual-third-person-leveling
        (Third, Dual) => (&["шете"], &["ста"]),
        (First, Plural) => (&["хомъ"], &["хомъ"]),
        // verb:imperfect-hardening
        (Second, Plural) => (&["шете"], &["сте"]),
        // realization: gen:big-yus + fold:uk on хѫ ~ хꙋ
        (Third, Plural) => (&["хѫ"], &["хꙋ"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// The aorist formation series. A series absent from a recension yields
/// empty columns everywhere (divergence `verb:aorist-inventory`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AoristSeries {
    /// The OCS root (asigmatic) aorist; no Synodal counterpart.
    Asigmatic,
    /// The productive ох-aorist (OCS "new" ↔ Alypy §86 consonant stems).
    New,
    /// The OCS first sigmatic aorist in -с-; no Synodal counterpart.
    SigmaticPrimary,
    /// The OCS second sigmatic aorist on consonant stems (its ending
    /// columns are shared with [`Self::SigmaticVowel`]; Synodal admits the
    /// х-series only on vowel stems — a family availability constraint).
    SigmaticSecondary,
    /// The sigmatic х-aorist on vowel stems (OCS sigmatic-vowel ↔ Alypy
    /// §86 vowel stems).
    SigmaticVowel,
    /// The Alypy §86 closed list (ꙗти, начати, вити, пити, клѧти and
    /// compounds) with the ordered -тъ / bare-stem second/third singular;
    /// no OCS counterpart (divergence `verb:aorist-inventory`).
    VowelStemWithT,
}

impl AoristSeries {
    pub const ALL: [Self; 6] = [
        Self::Asigmatic,
        Self::New,
        Self::SigmaticPrimary,
        Self::SigmaticSecondary,
        Self::SigmaticVowel,
        Self::VowelStemWithT,
    ];
}

/// One aorist ending cell. Stem-side transformations (the OCS second/third
/// singular palatalization, the supplied syncretic sigmatic principal
/// part) stay family-side; the empty string is the genuine zero ending of
/// the sigmatic second/third singular.
#[must_use]
pub fn aorist_ending(
    series: AoristSeries,
    person: Person,
    number: Number,
    recension: Recension,
) -> &'static [&'static str] {
    use AoristSeries::{
        Asigmatic, New, SigmaticPrimary, SigmaticSecondary, SigmaticVowel, VowelStemWithT,
    };
    use Number::{Dual, Plural, Singular};
    use Person::{First, Second, Third};
    let (ocs, syn): (&[&str], &[&str]) = match (series, person, number) {
        // ---- the OCS-only root aorist (verb:aorist-inventory) ----
        (Asigmatic, First, Singular) => (&["ъ"], NO_TEXTS),
        (Asigmatic, Second | Third, Singular) => (&["е"], NO_TEXTS),
        (Asigmatic, First, Dual) => (&["овѣ"], NO_TEXTS),
        (Asigmatic, Second, Dual) => (&["ета"], NO_TEXTS),
        (Asigmatic, Third, Dual) => (&["ете"], NO_TEXTS),
        (Asigmatic, First, Plural) => (&["омъ"], NO_TEXTS),
        (Asigmatic, Second, Plural) => (&["ете"], NO_TEXTS),
        (Asigmatic, Third, Plural) => (&["ѫ"], NO_TEXTS),
        // ---- the shared ох-aorist ----
        (New, First, Singular) => (&["охъ"], &["охъ"]),
        (New, Second | Third, Singular) => (&["е"], &["е"]),
        // verb:dual-first-person-va
        (New, First, Dual) => (&["оховѣ"], &["охова"]),
        (New, Second, Dual) => (&["оста"], &["оста"]),
        // verb:dual-third-person-leveling
        (New, Third, Dual) => (&["осте"], &["оста"]),
        (New, First, Plural) => (&["охомъ"], &["охомъ"]),
        (New, Second, Plural) => (&["осте"], &["осте"]),
        // verb:aorist-third-plural-a-grade
        (New, Third, Plural) => (&["ошѧ"], &["оша"]),
        // ---- the OCS-only first sigmatic aorist (verb:aorist-inventory) ----
        (SigmaticPrimary, First, Singular) => (&["съ"], NO_TEXTS),
        (SigmaticPrimary, Second | Third, Singular) => (&[""], NO_TEXTS),
        (SigmaticPrimary, First, Dual) => (&["совѣ"], NO_TEXTS),
        (SigmaticPrimary, Second, Dual) => (&["ста"], NO_TEXTS),
        (SigmaticPrimary, Third, Dual) => (&["сте"], NO_TEXTS),
        (SigmaticPrimary, First, Plural) => (&["сомъ"], NO_TEXTS),
        (SigmaticPrimary, Second, Plural) => (&["сте"], NO_TEXTS),
        (SigmaticPrimary, Third, Plural) => (&["сѧ"], NO_TEXTS),
        // ---- the sigmatic х-aorist ----
        (SigmaticSecondary | SigmaticVowel, First, Singular) => (&["хъ"], &["хъ"]),
        (SigmaticSecondary | SigmaticVowel, Second | Third, Singular) => (&[""], &[""]),
        // verb:dual-first-person-va
        (SigmaticSecondary | SigmaticVowel, First, Dual) => (&["ховѣ"], &["хова"]),
        (SigmaticSecondary | SigmaticVowel, Second, Dual) => (&["ста"], &["ста"]),
        // verb:dual-third-person-leveling
        (SigmaticSecondary | SigmaticVowel, Third, Dual) => (&["сте"], &["ста"]),
        (SigmaticSecondary | SigmaticVowel, First, Plural) => (&["хомъ"], &["хомъ"]),
        (SigmaticSecondary | SigmaticVowel, Second, Plural) => (&["сте"], &["сте"]),
        // verb:aorist-third-plural-a-grade
        (SigmaticSecondary | SigmaticVowel, Third, Plural) => (&["шѧ"], &["ша"]),
        // ---- the Synodal-only §86 -тъ list (verb:aorist-inventory) ----
        (VowelStemWithT, First, Singular) => (NO_TEXTS, &["хъ"]),
        (VowelStemWithT, Second | Third, Singular) => (NO_TEXTS, &["тъ", ""]),
        (VowelStemWithT, First, Dual) => (NO_TEXTS, &["хова"]),
        (VowelStemWithT, Second | Third, Dual) => (NO_TEXTS, &["ста"]),
        (VowelStemWithT, First, Plural) => (NO_TEXTS, &["хомъ"]),
        (VowelStemWithT, Second, Plural) => (NO_TEXTS, &["сте"]),
        (VowelStemWithT, Third, Plural) => (NO_TEXTS, &["ша"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// The closed copula series shared by both recensions, keyed by the form
/// series (stem shape), not the tense label — the recensions assign the
/// бѣ- and бꙑ-series to different tense systems (divergence
/// `verb:copula-tense-reassignment`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CopulaSeries {
    /// The present ѥс- ~ єс- series.
    PresentEs,
    /// The future бѫд- ~ бꙋд- series.
    FutureBud,
    /// The OCS uncontracted imperfect бѣа- against the Synodal imperfect
    /// бѧ- (divergence `verb:copula-imperfect-restemming`).
    ImperfectBea,
    /// The бѣ- series: the OCS aorist, re-labelled the Synodal
    /// imperfect-be table (verb:copula-tense-reassignment).
    AoristBe,
    /// The бꙑ- ~ бы- series: the OCS conditional-aorist, the Synodal
    /// plain aorist (verb:copula-tense-reassignment); the Synodal
    /// second/third singular adds бысть (verb:copula-aorist-sti).
    AoristBy,
}

impl CopulaSeries {
    pub const ALL: [Self; 5] = [
        Self::PresentEs,
        Self::FutureBud,
        Self::ImperfectBea,
        Self::AoristBe,
        Self::AoristBy,
    ];
}

/// One full copula surface cell (the series are suppletive, so the tables
/// store whole forms, not endings). The OCS column is the reviewed
/// Polivanova/UT table; the Synodal column is the Alypy §81 ordered
/// normative-table / normative-variant set of
/// `data/synodal/exact_forms.tsv` (accent marks stripped: the kernel is
/// accent-blind, the family data carries the accents).
#[must_use]
pub fn copula_form(
    series: CopulaSeries,
    person: Person,
    number: Number,
    recension: Recension,
) -> &'static [&'static str] {
    use CopulaSeries::{AoristBe, AoristBy, FutureBud, ImperfectBea, PresentEs};
    use Number::{Dual, Plural, Singular};
    use Person::{First, Second, Third};
    let (ocs, syn): (&[&str], &[&str]) = match (series, person, number) {
        // ---- present ----
        (PresentEs, First, Singular) => (&["ѥсмь"], &["єсмь"]),
        (PresentEs, Second, Singular) => (&["ѥси"], &["єси"]),
        // verb:copula-third-person-soft-t
        (PresentEs, Third, Singular) => (&["ѥстъ"], &["єсть"]),
        // verb:dual-first-person-va (the -вѣ archaism survives as the
        // ordered Synodal variant)
        (PresentEs, First, Dual) => (&["ѥсвѣ"], &["єсва", "єсвѣ"]),
        (PresentEs, Second, Dual) => (&["ѥста"], &["єста", "єстѣ"]),
        // verb:dual-third-person-leveling
        (PresentEs, Third, Dual) => (&["ѥсте"], &["єста", "єстѣ"]),
        // verb:copula-first-plural-my
        (PresentEs, First, Plural) => (&["ѥсмъ"], &["єсмы"]),
        (PresentEs, Second, Plural) => (&["ѥсте"], &["єсте"]),
        // verb:copula-third-person-soft-t
        (PresentEs, Third, Plural) => (&["сѫтъ"], &["сꙋть"]),
        // ---- future ----
        (FutureBud, First, Singular) => (&["бѫдѫ"], &["бꙋдꙋ"]),
        (FutureBud, Second, Singular) => (&["бѫдеши"], &["бꙋдеши"]),
        (FutureBud, Third, Singular) => (&["бѫдетъ"], &["бꙋдетъ"]),
        // verb:dual-first-person-va
        (FutureBud, First, Dual) => (&["бѫдевѣ"], &["бꙋдева", "бꙋдевѣ"]),
        (FutureBud, Second, Dual) => (&["бѫдета"], &["бꙋдета", "бꙋдетѣ"]),
        // verb:dual-third-person-leveling
        (FutureBud, Third, Dual) => (&["бѫдете"], &["бꙋдета", "бꙋдетѣ"]),
        (FutureBud, First, Plural) => (&["бѫдемъ"], &["бꙋдемъ"]),
        (FutureBud, Second, Plural) => (&["бѫдете"], &["бꙋдете"]),
        (FutureBud, Third, Plural) => (&["бѫдѫтъ"], &["бꙋдꙋтъ"]),
        // ---- imperfect (verb:copula-imperfect-restemming throughout) ----
        (ImperfectBea, First, Singular) => (&["бѣахъ"], &["бѧхъ"]),
        (ImperfectBea, Second | Third, Singular) => (&["бѣаше"], &["бѧше"]),
        (ImperfectBea, First, Dual) => (&["бѣаховѣ"], &["бѧхова", "бѧховѣ"]),
        (ImperfectBea, Second, Dual) => (&["бѣашета"], &["бѧста", "бѧстѣ"]),
        (ImperfectBea, Third, Dual) => (&["бѣашете"], &["бѧста", "бѧстѣ"]),
        (ImperfectBea, First, Plural) => (&["бѣахомъ"], &["бѧхомъ"]),
        (ImperfectBea, Second, Plural) => (&["бѣашете"], &["бѧсте"]),
        (ImperfectBea, Third, Plural) => (&["бѣахѫ"], &["бѧхꙋ"]),
        // ---- the бѣ- series ----
        (AoristBe, First, Singular) => (&["бѣхъ"], &["бѣхъ"]),
        (AoristBe, Second | Third, Singular) => (&["бѣ"], &["бѣ"]),
        // verb:dual-first-person-va
        (AoristBe, First, Dual) => (&["бѣховѣ"], &["бѣхова", "бѣховѣ"]),
        (AoristBe, Second, Dual) => (&["бѣста"], &["бѣста", "бѣстѣ"]),
        // verb:dual-third-person-leveling
        (AoristBe, Third, Dual) => (&["бѣсте"], &["бѣста", "бѣстѣ"]),
        (AoristBe, First, Plural) => (&["бѣхомъ"], &["бѣхомъ"]),
        (AoristBe, Second, Plural) => (&["бѣсте"], &["бѣсте"]),
        // verb:aorist-third-plural-a-grade
        (AoristBe, Third, Plural) => (&["бѣшѧ"], &["бѣша"]),
        // ---- the бꙑ- series ----
        (AoristBy, First, Singular) => (&["бꙑхъ"], &["быхъ"]),
        // verb:copula-aorist-sti
        (AoristBy, Second | Third, Singular) => (&["бꙑ"], &["бысть", "бы"]),
        // verb:dual-first-person-va
        (AoristBy, First, Dual) => (&["бꙑховѣ"], &["быхова", "быховѣ"]),
        (AoristBy, Second, Dual) => (&["бꙑста"], &["быста", "быстѣ"]),
        // verb:dual-third-person-leveling
        (AoristBy, Third, Dual) => (&["бꙑсте"], &["быста", "быстѣ"]),
        (AoristBy, First, Plural) => (&["бꙑхомъ"], &["быхомъ"]),
        (AoristBy, Second, Plural) => (&["бꙑсте"], &["бысте"]),
        // verb:aorist-third-plural-a-grade
        (AoristBy, Third, Plural) => (&["бꙑшѧ"], &["быша"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

#[cfg(test)]
mod tests {
    use super::{AoristSeries, CopulaSeries, aorist_ending, copula_form};
    use crate::grammar::{Number, Person};
    use crate::recension::Recension;

    #[test]
    fn copula_series_are_total_in_both_recensions() {
        for series in CopulaSeries::ALL {
            for person in Person::ALL {
                for number in Number::ALL {
                    for recension in [Recension::OldChurchSlavonic, Recension::SynodalRussian] {
                        let forms = copula_form(series, person, number, recension);
                        assert!(
                            !forms.is_empty(),
                            "{series:?} {person:?} {number:?} {recension:?}"
                        );
                        assert!(forms.iter().all(|form| !form.is_empty()));
                    }
                }
            }
        }
    }

    #[test]
    fn aorist_series_availability_is_the_named_asymmetry() {
        for series in AoristSeries::ALL {
            for person in Person::ALL {
                for number in Number::ALL {
                    let ocs = aorist_ending(series, person, number, Recension::OldChurchSlavonic);
                    let syn = aorist_ending(series, person, number, Recension::SynodalRussian);
                    let ocs_expected = series != AoristSeries::VowelStemWithT;
                    let syn_expected = matches!(
                        series,
                        AoristSeries::New
                            | AoristSeries::SigmaticSecondary
                            | AoristSeries::SigmaticVowel
                            | AoristSeries::VowelStemWithT
                    );
                    assert_eq!(!ocs.is_empty(), ocs_expected, "{series:?}");
                    assert_eq!(
                        !syn.is_empty(),
                        syn_expected,
                        "{series:?} {person:?} {number:?}"
                    );
                }
            }
        }
    }
}
