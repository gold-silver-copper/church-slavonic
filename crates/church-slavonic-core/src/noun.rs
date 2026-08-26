//! The merged vocalic-stem noun inflection kernel
//! (docs/UNIFIED_LANGUAGE_PROMPT.md, execution plan step 4, fourth POS
//! slice; the consonant stems live in [`crate::noun_consonant`]).
//!
//! One recension-conditioned ending table per shared vocalic declension
//! class. Every cell is written with both recensions side by side so that a
//! difference is always visibly one of:
//!
//! - **realization** — related by the declared projection rules of
//!   `church-slavonic-orthography::projection` (cited inline by rule id,
//!   e.g. `gen:yery`, `gen:big-yus`, `fold:ja`, `fold:uk`) or by a named
//!   Synodal spelling norm outside that rule set (checked by the
//!   realization-coherence test in the orthography crate);
//! - **a named divergence** — cited inline by its id in
//!   [`crate::divergence::NAMED`];
//! - **a per-recension lexical fact** — which never reaches this module:
//!   the Synodal velar/sibilant/mixed subclasses, the lexeme-specific
//!   profiles (господь, день, камень, ꙋдъ, дщерь, ѻко/ꙋхо), the OCS
//!   class-0 substantives, and both families' citation parsing stay in the
//!   family cores (see [`crate::divergence::UNMERGED`]).
//!
//! The family cores are adapters over these tables: they own their stem
//! selection (palatalization seams, iotation/glide respelling, mobile
//! vowels, wide-letter dual spellings), their interface types, validation,
//! variant provenance, and error vocabularies, and they read their own
//! recension's column through thin shims. Recensions other than the two
//! attested ones yield empty cells.
//!
//! Each recension's column stores that family's canonical kernel spelling
//! (OCS ꙑ/оу/ѥ/ꙗ/ѩ/ѫ/ѭ with the iotated soft series; Synodal ы/ꙋ/е/ѧ/ю
//! with positional ї/є/ѡ), because the columns feed the family engines
//! directly. The OCS column lists exactly one primary ending per populated
//! cell (Polivanova's tables are variant-free at this altitude); the
//! Synodal column lists the Alypy variant set in its reviewed order.

use crate::grammar::{Animacy, Case, Number};
use crate::recension::Recension;

const NO_TEXTS: &[&str] = &[];

/// The vocalic declension classes shared by both recensions' noun kernels.
///
/// The Synodal family's further subclasses (velar, sibilant, mixed-ц,
/// glide-й, -ей, -їе, -їа, postvocalic ancient plural) are derived
/// family-side from these columns or carried as Synodal-only paradigms; the
/// OCS family derives its glide/sibilant respelling from the canonical
/// iotated soft columns. Neither reaches this enum.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VocalicNounClass {
    /// Polivanova's hard twofold masculine (o-stem) ↔ Alypy §§33–37 first
    /// declension hard masculine.
    OHardMasculine,
    /// Polivanova's hard twofold neuter (o-stem) ↔ Alypy §34 first
    /// declension hard neuter.
    OHardNeuter,
    /// Polivanova's soft twofold masculine (jo-stem) ↔ Alypy §§34–37 first
    /// declension soft masculine.
    JoSoftMasculine,
    /// Polivanova's soft twofold neuter (jo-stem) ↔ Alypy §34 first
    /// declension soft neuter.
    JoSoftNeuter,
    /// Polivanova's hard twofold feminine (a-stem) ↔ Alypy §39 second
    /// declension hard.
    AHard,
    /// Polivanova's soft twofold feminine (ja-stem) ↔ Alypy §§39–40 second
    /// declension soft.
    JaSoft,
    /// Polivanova's simplex feminine (i-stem) ↔ Alypy §41 third declension
    /// feminine.
    IFeminine,
    /// Polivanova's simplex masculine (i-stem) ↔ Alypy §41 third declension
    /// masculine.
    IMasculine,
    /// The OCS u-stem masculine ↔ the Alypy §§37–38 first-declension
    /// profile with ordered u-stem variants (divergence
    /// `noun:u-stem-dissolution`).
    UStemMasculine,
}

impl VocalicNounClass {
    pub const ALL: [Self; 9] = [
        Self::OHardMasculine,
        Self::OHardNeuter,
        Self::JoSoftMasculine,
        Self::JoSoftNeuter,
        Self::AHard,
        Self::JaSoft,
        Self::IFeminine,
        Self::IMasculine,
        Self::UStemMasculine,
    ];
}

/// One vocalic noun ending cell. The OCS column is Polivanova's table
/// terminal (§§326–351); the Synodal column is the Alypy §§33–41 ending set
/// in its reviewed variant order. Animacy selects the genitive-shaped
/// accusative arm where a recension marks it (divergence
/// `pron:genitive-accusative` names the shared mechanism; the noun-specific
/// coverage differences are named per class below).
#[must_use]
pub fn vocalic_ending(
    class: VocalicNounClass,
    case: Case,
    number: Number,
    animacy: Animacy,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Number::{Dual, Plural, Singular};
    use VocalicNounClass::{
        AHard, IFeminine, IMasculine, JaSoft, JoSoftMasculine, JoSoftNeuter, OHardMasculine,
        OHardNeuter, UStemMasculine,
    };
    let animate = animacy == Animacy::Animate;
    let (ocs, syn): (&[&str], &[&str]) = match (class, case, number) {
        // ---- hard o-stem masculine (Polivanova table 327 ↔ Alypy §§33–37) ----
        (OHardMasculine, Nominative, Singular) => (&["ъ"], &["ъ"]),
        (OHardMasculine, Genitive, Singular) => (&["а"], &["а"]),
        // realization: fold:uk on -оу ~ -ꙋ; noun:hard-declension-variant-imports
        // on the Synodal u-stem doublet -ови.
        (OHardMasculine, Dative, Singular) => (&["оу"], &["ꙋ", "ови"]),
        (OHardMasculine, Accusative, Singular) => {
            if animate {
                (&["а"], &["а"])
            } else {
                (&["ъ"], &["ъ"])
            }
        }
        (OHardMasculine, Instrumental, Singular) => (&["омъ"], &["омъ"]),
        (OHardMasculine, Locative, Singular) => (&["ѣ"], &["ѣ"]),
        (OHardMasculine, Vocative, Singular) => (&["е"], &["е"]),
        (OHardMasculine, Nominative | Accusative | Vocative, Dual) => (&["а"], &["а"]),
        // realization: fold:uk.
        (OHardMasculine, Genitive | Locative, Dual) => (&["оу"], &["ꙋ"]),
        (OHardMasculine, Dative | Instrumental, Dual) => (&["ома"], &["ома"]),
        (OHardMasculine, Nominative | Vocative, Plural) => (&["и"], &["и"]),
        // noun:hard-declension-variant-imports (u-stem -овъ becomes the
        // primary Synodal genitive plural, the inherited -ъ the doublet).
        (OHardMasculine, Genitive, Plural) => (&["ъ"], &["овъ", "ъ"]),
        (OHardMasculine, Dative, Plural) => (&["омъ"], &["омъ"]),
        // realization: gen:yery on the inanimate arm;
        // noun:hard-declension-variant-imports on the animate arm (-овъ).
        (OHardMasculine, Accusative, Plural) => {
            if animate {
                (&["ъ"], &["овъ"])
            } else {
                (&["ꙑ"], &["ы"])
            }
        }
        // realization: gen:yery; noun:hard-declension-variant-imports on the
        // Synodal i-stem/a-stem instrumental doublets.
        (OHardMasculine, Instrumental, Plural) => (&["ꙑ"], &["ы", "ми", "ами"]),
        // noun:hard-declension-variant-imports on the a-stem locative doublet.
        (OHardMasculine, Locative, Plural) => (&["ѣхъ"], &["ѣхъ", "ахъ"]),

        // ---- hard o-stem neuter (Polivanova table 339 ↔ Alypy §34) ----
        (OHardNeuter, Nominative | Accusative | Vocative, Singular) => (&["о"], &["о"]),
        (OHardNeuter, Genitive, Singular) => (&["а"], &["а"]),
        // realization: fold:uk.
        (OHardNeuter, Dative, Singular) => (&["оу"], &["ꙋ"]),
        (OHardNeuter, Instrumental, Singular) => (&["омъ"], &["омъ"]),
        (OHardNeuter, Locative, Singular) => (&["ѣ"], &["ѣ"]),
        // noun:dual-direct-reshape (OCS -ѣ against the Synodal masculine-
        // shaped -а).
        (OHardNeuter, Nominative | Accusative | Vocative, Dual) => (&["ѣ"], &["а"]),
        // realization: fold:uk.
        (OHardNeuter, Genitive | Locative, Dual) => (&["оу"], &["ꙋ"]),
        (OHardNeuter, Dative | Instrumental, Dual) => (&["ома"], &["ома"]),
        (OHardNeuter, Nominative | Accusative | Vocative, Plural) => (&["а"], &["а"]),
        (OHardNeuter, Genitive, Plural) => (&["ъ"], &["ъ"]),
        (OHardNeuter, Dative, Plural) => (&["омъ"], &["омъ"]),
        // realization: gen:yery; noun:hard-declension-variant-imports (-ами).
        (OHardNeuter, Instrumental, Plural) => (&["ꙑ"], &["ы", "ами"]),
        // noun:hard-declension-variant-imports (-ахъ).
        (OHardNeuter, Locative, Plural) => (&["ѣхъ"], &["ѣхъ", "ахъ"]),

        // ---- soft jo-stem masculine (Polivanova table 327 ↔ Alypy §§34–37).
        // The OCS column is the canonical iotated series; the family shim
        // de-iotates after sibilants and plain consonants at its documented
        // seams. ----
        (JoSoftMasculine, Nominative, Singular) => (&["ь"], &["ь"]),
        // realization: fold:ja (ꙗ ~ ѧ).
        (JoSoftMasculine, Genitive, Singular) => (&["ꙗ"], &["ѧ"]),
        // noun:hard-declension-variant-imports (Synodal soft dative doublet
        // -еви after the u-stem analogy).
        (JoSoftMasculine, Dative, Singular) => (&["ю"], &["ю", "еви"]),
        (JoSoftMasculine, Accusative, Singular) => {
            if animate {
                // realization: fold:ja.
                (&["ꙗ"], &["ѧ"])
            } else {
                (&["ь"], &["ь"])
            }
        }
        // noun:instrumental-singular-jer (-ѥмь against -емъ).
        (JoSoftMasculine, Instrumental, Singular) => (&["ѥмь"], &["емъ"]),
        // noun:hard-declension-variant-imports (Synodal hard locative
        // doublet -ѣ beside the inherited -и).
        (JoSoftMasculine, Locative, Singular) => (&["и"], &["и", "ѣ"]),
        (JoSoftMasculine, Vocative, Singular) => (&["ю"], &["ю"]),
        // realization: fold:ja.
        (JoSoftMasculine, Nominative | Accusative | Vocative, Dual) => (&["ꙗ"], &["ѧ"]),
        (JoSoftMasculine, Genitive | Locative, Dual) => (&["ю"], &["ю"]),
        // realization: gen:iotated-e (ѥ ~ е).
        (JoSoftMasculine, Dative | Instrumental, Dual) => (&["ѥма"], &["ема"]),
        // noun:hard-declension-variant-imports (-їе after the i-stem plural).
        (JoSoftMasculine, Nominative | Vocative, Plural) => (&["и"], &["и", "їе"]),
        // noun:soft-genitive-plural-reinventory (-ь against -ей).
        (JoSoftMasculine, Genitive, Plural) => (&["ь"], &["ей"]),
        // realization: gen:iotated-e + gen:jer-final.
        (JoSoftMasculine, Dative, Plural) => (&["ѥмъ"], &["емъ"]),
        (JoSoftMasculine, Accusative, Plural) => {
            if animate {
                // noun:soft-genitive-plural-reinventory on the animate arm.
                (&["ь"], &["ей"])
            } else {
                // noun:soft-direct-plural-leveling (-ѩ against -и).
                (&["ѩ"], &["и"])
            }
        }
        // noun:hard-declension-variant-imports (-ьми/-ами doublets).
        (JoSoftMasculine, Instrumental, Plural) => (&["и"], &["и", "ьми", "ами"]),
        // noun:locative-plural-reinventory (-ихъ against -ехъ/-ѧхъ).
        (JoSoftMasculine, Locative, Plural) => (&["ихъ"], &["ехъ", "ѧхъ"]),

        // ---- soft jo-stem neuter (Polivanova table 339 ↔ Alypy §34) ----
        // realization: gen:iotated-e.
        (JoSoftNeuter, Nominative | Accusative | Vocative, Singular) => (&["ѥ"], &["е"]),
        // realization: fold:ja.
        (JoSoftNeuter, Genitive, Singular) => (&["ꙗ"], &["ѧ"]),
        (JoSoftNeuter, Dative, Singular) => (&["ю"], &["ю"]),
        // noun:instrumental-singular-jer.
        (JoSoftNeuter, Instrumental, Singular) => (&["ѥмь"], &["емъ"]),
        (JoSoftNeuter, Locative, Singular) => (&["и"], &["и"]),
        (JoSoftNeuter, Nominative | Accusative | Vocative, Dual) => (&["и"], &["и"]),
        (JoSoftNeuter, Genitive | Locative, Dual) => (&["ю"], &["ю"]),
        // realization: gen:iotated-e.
        (JoSoftNeuter, Dative | Instrumental, Dual) => (&["ѥма"], &["ема"]),
        // realization: fold:ja.
        (JoSoftNeuter, Nominative | Accusative | Vocative, Plural) => (&["ꙗ"], &["ѧ"]),
        // noun:soft-genitive-plural-reinventory.
        (JoSoftNeuter, Genitive, Plural) => (&["ь"], &["ей"]),
        // realization: gen:iotated-e + gen:jer-final.
        (JoSoftNeuter, Dative, Plural) => (&["ѥмъ"], &["емъ"]),
        // noun:hard-declension-variant-imports (-ьми/-ами doublets).
        (JoSoftNeuter, Instrumental, Plural) => (&["и"], &["и", "ьми", "ами"]),
        // noun:locative-plural-reinventory (-ихъ against -ѧхъ).
        (JoSoftNeuter, Locative, Plural) => (&["ихъ"], &["ѧхъ"]),

        // ---- hard a-stem (Polivanova table 343 ↔ Alypy §39) ----
        (AHard, Nominative, Singular) => (&["а"], &["а"]),
        // realization: gen:yery.
        (AHard, Genitive, Singular) => (&["ꙑ"], &["ы"]),
        (AHard, Dative | Locative, Singular) => (&["ѣ"], &["ѣ"]),
        // realization: gen:big-yus (ѫ → у, typographic ꙋ).
        (AHard, Accusative, Singular) => (&["ѫ"], &["ꙋ"]),
        // realization: gen:iotated-big-yus (оѭ ~ ою).
        (AHard, Instrumental, Singular) => (&["оѭ"], &["ою"]),
        (AHard, Vocative, Singular) => (&["о"], &["о"]),
        (AHard, Nominative | Accusative | Vocative, Dual) => (&["ѣ"], &["ѣ"]),
        // realization: fold:uk.
        (AHard, Genitive | Locative, Dual) => (&["оу"], &["ꙋ"]),
        (AHard, Dative | Instrumental, Dual) => (&["ама"], &["ама"]),
        (AHard, Nominative | Vocative, Plural) => (&["ꙑ"], &["ы"]),
        (AHard, Accusative, Plural) => {
            if animate {
                // noun:animate-accusative-coverage (OCS a-stems keep the
                // nominative-shaped accusative; Synodal marks animacy).
                (&["ꙑ"], &["ъ"])
            } else {
                // realization: gen:yery.
                (&["ꙑ"], &["ы"])
            }
        }
        (AHard, Genitive, Plural) => (&["ъ"], &["ъ"]),
        (AHard, Dative, Plural) => (&["амъ"], &["амъ"]),
        (AHard, Instrumental, Plural) => (&["ами"], &["ами"]),
        (AHard, Locative, Plural) => (&["ахъ"], &["ахъ"]),

        // ---- soft ja-stem (Polivanova table 343 ↔ Alypy §§39–40).
        // The OCS column is the canonical iotated series; the family shim
        // de-iotates after sibilants at its documented seam. ----
        // realization: fold:ja.
        (JaSoft, Nominative, Singular) => (&["ꙗ"], &["ѧ"]),
        // noun:soft-feminine-genitive-leveling (-ѩ against -и).
        (JaSoft, Genitive, Singular) => (&["ѩ"], &["и"]),
        (JaSoft, Dative | Locative, Singular) => (&["и"], &["и"]),
        // realization: gen:big-yus (iotated grade; cf. вьсѫ ~ всю).
        (JaSoft, Accusative, Singular) => (&["ѭ"], &["ю"]),
        // realization: gen:iotated-big-yus (еѭ ~ ею).
        (JaSoft, Instrumental, Singular) => (&["еѭ"], &["ею"]),
        (JaSoft, Vocative, Singular) => (&["е"], &["е"]),
        (JaSoft, Nominative | Accusative | Vocative, Dual) => (&["и"], &["и"]),
        (JaSoft, Genitive | Locative, Dual) => (&["ю"], &["ю"]),
        // realization: fold:ja.
        (JaSoft, Dative | Instrumental, Dual) => (&["ꙗма"], &["ѧма"]),
        // noun:soft-direct-plural-leveling (-ѩ against -и).
        (JaSoft, Nominative | Vocative, Plural) => (&["ѩ"], &["и"]),
        (JaSoft, Accusative, Plural) => {
            if animate {
                // noun:animate-accusative-coverage (Synodal genitive-shaped
                // animate arm; OCS keeps the nominative-shaped -ѩ).
                (&["ѩ"], &["ь"])
            } else {
                // noun:soft-direct-plural-leveling.
                (&["ѩ"], &["и"])
            }
        }
        (JaSoft, Genitive, Plural) => (&["ь"], &["ь"]),
        // realization: fold:ja.
        (JaSoft, Dative, Plural) => (&["ꙗмъ"], &["ѧмъ"]),
        (JaSoft, Instrumental, Plural) => (&["ꙗми"], &["ѧми"]),
        (JaSoft, Locative, Plural) => (&["ꙗхъ"], &["ѧхъ"]),

        // ---- simplex i-stem feminine (Polivanova table 351 ↔ Alypy §41) ----
        (IFeminine, Nominative | Accusative, Singular) => (&["ь"], &["ь"]),
        (IFeminine, Genitive | Dative | Locative, Singular) => (&["и"], &["и"]),
        // noun:i-stem-instrumental-i-grade (-ьѭ against -їю).
        (IFeminine, Instrumental, Singular) => (&["ьѭ"], &["їю"]),
        // noun:i-stem-vocative-leveling (-и against -е).
        (IFeminine, Vocative, Singular) => (&["и"], &["е"]),
        (IFeminine, Nominative | Accusative | Vocative, Dual) => (&["и"], &["и"]),
        // noun:i-stem-instrumental-i-grade (-ию against -їю is realization
        // fold:i-variants; kept here for the paired feminine columns).
        (IFeminine, Genitive | Locative, Dual) => (&["ию"], &["їю"]),
        // realization: gen:jer-medial (-ьма ~ -ема), with the inherited
        // -ьма co-listed.
        (IFeminine, Dative | Instrumental, Dual) => (&["ьма"], &["ема", "ьма"]),
        (IFeminine, Nominative | Accusative | Vocative, Plural) => (&["и"], &["и"]),
        // noun:soft-genitive-plural-reinventory (-ии against -ей).
        (IFeminine, Genitive, Plural) => (&["ии"], &["ей"]),
        // realization: gen:jer-medial (-ьмъ ~ -емъ).
        (IFeminine, Dative, Plural) => (&["ьмъ"], &["емъ"]),
        (IFeminine, Instrumental, Plural) => (&["ьми"], &["ьми"]),
        // realization: gen:jer-medial (-ьхъ ~ -ехъ).
        (IFeminine, Locative, Plural) => (&["ьхъ"], &["ехъ"]),

        // ---- simplex i-stem masculine (Polivanova table 335 ↔ Alypy §41) ----
        (IMasculine, Nominative | Accusative, Singular) => (&["ь"], &["ь"]),
        (IMasculine, Genitive | Dative | Locative, Singular) => (&["и"], &["и"]),
        // noun:instrumental-singular-jer.
        (IMasculine, Instrumental, Singular) => (&["ьмь"], &["емъ"]),
        // noun:i-stem-vocative-leveling (-и against -ь/-ю).
        (IMasculine, Vocative, Singular) => (&["и"], &["ь", "ю"]),
        (IMasculine, Nominative | Accusative | Vocative, Dual) => (&["и"], &["и"]),
        // realization: fold:i-variants.
        (IMasculine, Genitive | Locative, Dual) => (&["ию"], &["їю"]),
        (IMasculine, Dative | Instrumental, Dual) => (&["ьма"], &["ьма"]),
        // realization: gen:iotated-e + fold:i-variants (-иѥ ~ -їе).
        (IMasculine, Nominative | Vocative, Plural) => (&["иѥ"], &["їе"]),
        // noun:soft-genitive-plural-reinventory (-ии against -ій/-ей).
        (IMasculine, Genitive, Plural) => (&["ии"], &["ій", "ей"]),
        // realization: gen:jer-medial (-ьмъ ~ -емъ; є is the family's
        // positional wide-е norm).
        (IMasculine, Dative, Plural) => (&["ьмъ"], &["ємъ"]),
        (IMasculine, Accusative, Plural) => {
            if animate {
                // noun:soft-genitive-plural-reinventory on the animate arm.
                (&["и"], &["ій"])
            } else {
                (&["и"], &["и"])
            }
        }
        (IMasculine, Instrumental, Plural) => (&["ьми"], &["ьми"]),
        // realization: gen:jer-medial (-ьхъ ~ -ехъ).
        (IMasculine, Locative, Plural) => (&["ьхъ"], &["ехъ"]),

        // ---- u-stem masculine (Polivanova §333 ↔ Alypy §§37–38).
        // Divergence noun:u-stem-dissolution throughout: the OCS u-stem
        // paradigm dissolves into the Synodal first declension carrying the
        // u-stem endings as ordered variants; the distinct dual, vocative,
        // and jer-grade obliques are not preserved. ----
        (UStemMasculine, Nominative, Singular) => (&["ъ"], &["ъ"]),
        (UStemMasculine, Genitive, Singular) => (&["оу"], &["а", "ꙋ"]),
        (UStemMasculine, Dative, Singular) => (&["ови"], &["ꙋ", "ови"]),
        (UStemMasculine, Accusative, Singular) => {
            if animate {
                (&["ъ"], &["а"])
            } else {
                (&["ъ"], &["ъ"])
            }
        }
        (UStemMasculine, Instrumental, Singular) => (&["ъмь"], &["омъ"]),
        (UStemMasculine, Locative, Singular) => (&["оу"], &["ѣ", "ꙋ"]),
        (UStemMasculine, Vocative, Singular) => (&["оу"], &["е"]),
        (UStemMasculine, Nominative | Accusative | Vocative, Dual) => (&["ꙑ"], &["а"]),
        (UStemMasculine, Genitive | Locative, Dual) => (&["овоу"], &["ꙋ"]),
        (UStemMasculine, Dative | Instrumental, Dual) => (&["ъма"], &["ома"]),
        (UStemMasculine, Nominative | Vocative, Plural) => (&["ове"], &["и", "ове"]),
        (UStemMasculine, Genitive, Plural) => (&["овъ"], &["овъ"]),
        (UStemMasculine, Dative, Plural) => (&["ъмъ"], &["омъ", "овомъ"]),
        (UStemMasculine, Accusative, Plural) => {
            if animate {
                (&["ꙑ"], &["овъ"])
            } else {
                (&["ꙑ"], &["ы"])
            }
        }
        (UStemMasculine, Instrumental, Plural) => (&["ъми"], &["ы", "ми"]),
        (UStemMasculine, Locative, Plural) => (&["ъхъ"], &["ѣхъ", "овѣхъ", "ахъ"]),
    };
    by_recension(recension, ocs, syn)
}

pub(crate) fn by_recension(
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

#[cfg(test)]
mod tests {
    use super::{VocalicNounClass, vocalic_ending};
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
            for class in VocalicNounClass::ALL {
                for case in Case::ALL {
                    for number in Number::ALL {
                        for animacy in Animacy::ALL {
                            assert!(
                                vocalic_ending(class, case, number, animacy, recension).is_empty()
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn attested_recension_tables_are_total() {
        for class in VocalicNounClass::ALL {
            for case in Case::ALL {
                for number in Number::ALL {
                    for animacy in Animacy::ALL {
                        for recension in [OCS, SYN] {
                            let endings = vocalic_ending(class, case, number, animacy, recension);
                            assert!(
                                !endings.is_empty(),
                                "{class:?} {case:?} {number:?} {animacy:?} {recension:?}"
                            );
                            assert!(endings.iter().all(|ending| !ending.is_empty()));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn ocs_columns_are_variant_free() {
        for class in VocalicNounClass::ALL {
            for case in Case::ALL {
                for number in Number::ALL {
                    for animacy in Animacy::ALL {
                        assert_eq!(vocalic_ending(class, case, number, animacy, OCS).len(), 1);
                    }
                }
            }
        }
    }

    #[test]
    fn instrumental_singular_jer_divergence_holds() {
        // noun:instrumental-singular-jer: every soft/jer-grade instrumental
        // singular pairs an OCS soft-jer ending with a Synodal -емъ/-омъ.
        for (class, ocs, syn) in [
            (VocalicNounClass::JoSoftMasculine, "ѥмь", "емъ"),
            (VocalicNounClass::JoSoftNeuter, "ѥмь", "емъ"),
            (VocalicNounClass::IMasculine, "ьмь", "емъ"),
            (VocalicNounClass::UStemMasculine, "ъмь", "омъ"),
        ] {
            assert_eq!(
                vocalic_ending(
                    class,
                    Case::Instrumental,
                    Number::Singular,
                    Animacy::Inanimate,
                    OCS,
                ),
                [ocs]
            );
            assert_eq!(
                vocalic_ending(
                    class,
                    Case::Instrumental,
                    Number::Singular,
                    Animacy::Inanimate,
                    SYN,
                )[0],
                syn
            );
        }
    }

    #[test]
    fn soft_genitive_plural_reinventory_holds() {
        // noun:soft-genitive-plural-reinventory.
        for (class, ocs, syn) in [
            (VocalicNounClass::JoSoftMasculine, "ь", "ей"),
            (VocalicNounClass::JoSoftNeuter, "ь", "ей"),
            (VocalicNounClass::IFeminine, "ии", "ей"),
            (VocalicNounClass::IMasculine, "ии", "ій"),
        ] {
            assert_eq!(
                vocalic_ending(
                    class,
                    Case::Genitive,
                    Number::Plural,
                    Animacy::Inanimate,
                    OCS
                ),
                [ocs]
            );
            assert_eq!(
                vocalic_ending(
                    class,
                    Case::Genitive,
                    Number::Plural,
                    Animacy::Inanimate,
                    SYN
                )[0],
                syn
            );
        }
    }

    #[test]
    fn u_stem_dissolution_keeps_the_synodal_variant_orders() {
        // noun:u-stem-dissolution: the Alypy §§37–38 ordered variant sets.
        assert_eq!(
            vocalic_ending(
                VocalicNounClass::UStemMasculine,
                Case::Genitive,
                Number::Singular,
                Animacy::Inanimate,
                SYN,
            ),
            ["а", "ꙋ"]
        );
        assert_eq!(
            vocalic_ending(
                VocalicNounClass::UStemMasculine,
                Case::Locative,
                Number::Plural,
                Animacy::Inanimate,
                SYN,
            ),
            ["ѣхъ", "овѣхъ", "ахъ"]
        );
        assert_eq!(
            vocalic_ending(
                VocalicNounClass::UStemMasculine,
                Case::Nominative,
                Number::Dual,
                Animacy::Inanimate,
                OCS,
            ),
            ["ꙑ"]
        );
    }
}
