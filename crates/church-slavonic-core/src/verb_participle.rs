//! The merged participle stem-formation kernel
//! (docs/UNIFIED_LANGUAGE_PROMPT.md, execution plan step 4, fifth POS
//! slice; the present system lives in [`crate::verb`], the past tenses in
//! [`crate::verb_past`]).
//!
//! This module merges the participle STEM formations: the suffix that
//! builds the participial (adjectival) stem, and the masculine/neuter
//! nominative-singular citation edge that escapes the agreement paradigm.
//! The declined agreement endings already ride the merged adjective kernel
//! (slice 3); the family cores keep their stem supply, lexical нн
//! doubling, sibilant subclassing, and ов→оу transformations as lexical
//! facts (`unmerged:verb:participle-stem-supply`).
//!
//! Suffixes are written relative to the bare formation stem. The OCS
//! column is the Polivanova/UT reviewed suffix; the Synodal column lists
//! the Alypy edge variants in their reviewed order. The OCS -шт- against
//! the Synodal -щ- is realization by the ligature identity щ = шт — a
//! Synodal spelling norm outside the declared projection rule set, named
//! as such in the realization-coherence residue.

use crate::recension::Recension;

const NO_TEXTS: &[&str] = &[];

/// The present-active participle formations. A formation absent from a
/// recension yields empty columns.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PresentActiveFormation {
    /// The hard first-conjugation formation: OCS -ѫшт- with nominative
    /// -ꙑ against Synodal -ꙋщ- with the contracted -ый (and the retained
    /// -ꙋщь print) — divergence
    /// `verb:present-active-nominative-contraction`.
    HardUsht,
    /// The OCS soft variant of the hard formation (nominative -ѩ over the
    /// same -ѫшт- oblique); no distinct Synodal column.
    SoftUsht,
    /// The iotated formation: OCS -ѭшт-/-ѩ against Synodal -ющ-/-ѧ
    /// (with the retained -ющь print).
    IotatedUsht,
    /// The second-conjugation formation: OCS -ѧшт-/-ѧ against Synodal
    /// -ѧщ-/-ѧ (with the retained -ѧщь print).
    SoftAsht,
    /// The OCS mixed formation (-ѫшт- oblique, -ѧ nominative); no
    /// Synodal column.
    MixedUsht,
    /// The Synodal after-sibilant formation -ащ- with the -а/-ѧ edge
    /// doublet (Alypy §95); no OCS column.
    SibilantAsht,
}

impl PresentActiveFormation {
    pub const ALL: [Self; 6] = [
        Self::HardUsht,
        Self::SoftUsht,
        Self::IotatedUsht,
        Self::SoftAsht,
        Self::MixedUsht,
        Self::SibilantAsht,
    ];
}

/// The oblique (adjectival) stem suffix of a present-active formation.
#[must_use]
pub fn present_active_oblique_suffix(
    formation: PresentActiveFormation,
    recension: Recension,
) -> &'static [&'static str] {
    use PresentActiveFormation::{
        HardUsht, IotatedUsht, MixedUsht, SibilantAsht, SoftAsht, SoftUsht,
    };
    let (ocs, syn): (&[&str], &[&str]) = match formation {
        // realization: щ = шт ligature norm + gen:big-yus
        HardUsht => (&["ѫшт"], &["ꙋщ"]),
        SoftUsht => (&["ѫшт"], NO_TEXTS),
        // realization: щ = шт ligature norm + gen:iotated-big-yus
        IotatedUsht => (&["ѭшт"], &["ющ"]),
        // realization: щ = шт ligature norm
        SoftAsht => (&["ѧшт"], &["ѧщ"]),
        MixedUsht => (&["ѫшт"], NO_TEXTS),
        SibilantAsht => (NO_TEXTS, &["ащ"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// The masculine/neuter nominative-singular citation edge of a
/// present-active formation, as a suffix on the bare formation stem.
#[must_use]
pub fn present_active_nominative_edge(
    formation: PresentActiveFormation,
    recension: Recension,
) -> &'static [&'static str] {
    use PresentActiveFormation::{
        HardUsht, IotatedUsht, MixedUsht, SibilantAsht, SoftAsht, SoftUsht,
    };
    let (ocs, syn): (&[&str], &[&str]) = match formation {
        // verb:present-active-nominative-contraction
        HardUsht => (&["ꙑ"], &["ый", "ꙋщь"]),
        SoftUsht => (&["ѩ"], NO_TEXTS),
        // verb:present-active-nominative-contraction (the primary -ѧ is
        // realization gen:iotated-small-yus of -ѩ; the retained -ющь
        // print is the divergent member)
        IotatedUsht => (&["ѩ"], &["ѧ", "ющь"]),
        // verb:present-active-nominative-contraction (shared -ѧ primary;
        // the retained -ѧщь print is the divergent member)
        SoftAsht => (&["ѧ"], &["ѧ", "ѧщь"]),
        MixedUsht => (&["ѧ"], NO_TEXTS),
        SibilantAsht => (NO_TEXTS, &["а", "ѧ", "ащь"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// The past-active participle formations.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PastActiveFormation {
    /// Consonant stems: OCS -ъш-/-ъ against Synodal -ш-/-ъ (with the
    /// retained -шъ print); the medial jer loss is realization
    /// `gen:jer-medial`.
    ConsonantHard,
    /// The OCS fronted i-stem formation -ьш-/-ь; no Synodal column (the
    /// Synodal family levels these stems into the consonant/vowel
    /// classes).
    SoftI,
    /// The OCS glide formation -ишь-/-и; the Synodal iotated class drops
    /// the citation edge entirely (bare stem citation), so the columns do
    /// not align and each stays one-recension.
    GlideI,
    /// Vowel stems: OCS -въш-/-въ against Synodal -вш-/-въ (with the
    /// retained -вшъ print); realization `gen:jer-medial`.
    Vowel,
    /// The Synodal iotated past formation: -ш- oblique with a bare-stem
    /// citation (Alypy §96); no OCS column.
    SynodalIotated,
}

impl PastActiveFormation {
    pub const ALL: [Self; 5] = [
        Self::ConsonantHard,
        Self::SoftI,
        Self::GlideI,
        Self::Vowel,
        Self::SynodalIotated,
    ];
}

/// The oblique (adjectival) stem suffix of a past-active formation.
#[must_use]
pub fn past_active_oblique_suffix(
    formation: PastActiveFormation,
    recension: Recension,
) -> &'static [&'static str] {
    use PastActiveFormation::{ConsonantHard, GlideI, SoftI, SynodalIotated, Vowel};
    let (ocs, syn): (&[&str], &[&str]) = match formation {
        // realization: gen:jer-medial on ъш ~ ш
        ConsonantHard => (&["ъш"], &["ш"]),
        SoftI => (&["ьш"], NO_TEXTS),
        GlideI => (&["ишь"], NO_TEXTS),
        // realization: gen:jer-medial on въш ~ вш
        Vowel => (&["въш"], &["вш"]),
        SynodalIotated => (NO_TEXTS, &["ш"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// The masculine/neuter nominative-singular citation edge of a
/// past-active formation, as a suffix on the bare formation stem.
#[must_use]
pub fn past_active_nominative_edge(
    formation: PastActiveFormation,
    recension: Recension,
) -> &'static [&'static str] {
    use PastActiveFormation::{ConsonantHard, GlideI, SoftI, SynodalIotated, Vowel};
    let (ocs, syn): (&[&str], &[&str]) = match formation {
        // verb:present-active-nominative-contraction names the retained
        // -щь/-шъ print axis; the shared primary -ъ is identity.
        ConsonantHard => (&["ъ"], &["ъ", "шъ"]),
        SoftI => (&["ь"], NO_TEXTS),
        GlideI => (&["и"], NO_TEXTS),
        Vowel => (&["въ"], &["въ", "вшъ"]),
        // The Synodal iotated citation is the bare stem.
        SynodalIotated => (NO_TEXTS, &[""]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// The present-passive participle suffixes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PresentPassiveFormation {
    Im,
    Em,
    /// The OCS iotated print -ѥм- against the plain Synodal -ем-
    /// (realization `gen:iotated-e`).
    IotatedEm,
    Om,
}

impl PresentPassiveFormation {
    pub const ALL: [Self; 4] = [Self::Im, Self::Em, Self::IotatedEm, Self::Om];
}

/// The present-passive adjectival stem suffix.
#[must_use]
pub fn present_passive_suffix(
    formation: PresentPassiveFormation,
    recension: Recension,
) -> &'static [&'static str] {
    use PresentPassiveFormation::{Em, Im, IotatedEm, Om};
    let (ocs, syn): (&[&str], &[&str]) = match formation {
        Im => (&["им"], &["им"]),
        Em => (&["ем"], &["ем"]),
        // realization: gen:iotated-e
        IotatedEm => (&["ѥм"], &["ем"]),
        Om => (&["ом"], &["ом"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

/// The past-passive participle suffixes (shared inventory; the Synodal
/// long-form нн doubling is a family lexical fact).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PastPassiveFormation {
    T,
    N,
    En,
}

impl PastPassiveFormation {
    pub const ALL: [Self; 3] = [Self::T, Self::N, Self::En];
}

/// The past-passive adjectival stem suffix.
#[must_use]
pub fn past_passive_suffix(
    formation: PastPassiveFormation,
    recension: Recension,
) -> &'static [&'static str] {
    use PastPassiveFormation::{En, N, T};
    let (ocs, syn): (&[&str], &[&str]) = match formation {
        T => (&["т"], &["т"]),
        N => (&["н"], &["н"]),
        En => (&["ен"], &["ен"]),
    };
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => syn,
        _ => NO_TEXTS,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PastActiveFormation, PastPassiveFormation, PresentActiveFormation, PresentPassiveFormation,
        past_active_nominative_edge, past_active_oblique_suffix, past_passive_suffix,
        present_active_nominative_edge, present_active_oblique_suffix, present_passive_suffix,
    };
    use crate::recension::Recension;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::SynodalRussian;

    #[test]
    fn oblique_and_nominative_columns_populate_together() {
        for formation in PresentActiveFormation::ALL {
            for recension in [OCS, SYN] {
                assert_eq!(
                    present_active_oblique_suffix(formation, recension).is_empty(),
                    present_active_nominative_edge(formation, recension).is_empty(),
                    "{formation:?} {recension:?}"
                );
            }
        }
        for formation in PastActiveFormation::ALL {
            for recension in [OCS, SYN] {
                assert_eq!(
                    past_active_oblique_suffix(formation, recension).is_empty(),
                    past_active_nominative_edge(formation, recension).is_empty(),
                    "{formation:?} {recension:?}"
                );
            }
        }
    }

    #[test]
    fn passive_suffix_inventories_are_shared() {
        for formation in PresentPassiveFormation::ALL {
            assert!(!present_passive_suffix(formation, OCS).is_empty());
            assert!(!present_passive_suffix(formation, SYN).is_empty());
        }
        for formation in PastPassiveFormation::ALL {
            assert_eq!(
                past_passive_suffix(formation, OCS),
                past_passive_suffix(formation, SYN)
            );
        }
    }
}
