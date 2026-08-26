//! The merged pronoun inflection kernel (docs/UNIFIED_LANGUAGE_PROMPT.md,
//! execution plan step 4, first POS slice).
//!
//! One paradigm per closed pronominal system, recension-conditioned at named
//! points. Every cell is written with both recensions side by side so that a
//! difference is always visibly one of:
//!
//! - **realization** — the surfaces are related by the declared projection
//!   rules of `church-slavonic-orthography::projection` (cited inline by
//!   rule id, e.g. `gen:yery`, `gen:jer-medial`, `fold:ja`); the kernel
//!   stores each recension's canonical spelling because runtime projection
//!   is deliberately ambiguous (jer branching, zelo), but the equivalence is
//!   checked by the realization-coherence test in the orthography crate;
//! - **a named divergence** — cited inline by its id in
//!   [`crate::divergence::NAMED`];
//! - **a per-recension lexical fact** — which never reaches this module: it
//!   stays in the family cores (see [`crate::divergence::UNMERGED`]).
//!
//! The family cores (`old-church-slavonic-core`, `synodal-church-slavonic-
//! core`) are adapters over these tables: they own their interface types
//! (variant statuses, `FormSet`/trace plumbing, validation and error
//! vocabularies) and translate to and from the plain types here. Recensions
//! other than the two attested ones yield empty cells.

use crate::{Animacy, Case, Gender, Number, Recension};

/// Evidential/syntactic role of one reviewed surface within its cell, the
/// recension-neutral union of the two families' status vocabularies.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SurfaceRole {
    /// The source table's primary form.
    Primary,
    /// A co-listed variant after the source-ordered primary.
    Variant,
    /// A form the source explicitly marks as clitic/enclitic.
    Clitic,
    /// Listed as a clitic by UT while Polivanova finds no OCS attestation.
    DisputedClitic,
}

impl SurfaceRole {
    #[must_use]
    pub const fn is_clitic(self) -> bool {
        matches!(self, Self::Clitic | Self::DisputedClitic)
    }
}

/// One reviewed surface of a pronoun cell in its recension's canonical
/// kernel spelling (pre-display: the Synodal families apply their own
/// orthography profiles downstream).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PronounSurface {
    pub text: &'static str,
    pub role: SurfaceRole,
}

macro_rules! p {
    ($text:literal) => {
        PronounSurface {
            text: $text,
            role: SurfaceRole::Primary,
        }
    };
}

macro_rules! v {
    ($text:literal) => {
        PronounSurface {
            text: $text,
            role: SurfaceRole::Variant,
        }
    };
}

macro_rules! c {
    ($text:literal) => {
        PronounSurface {
            text: $text,
            role: SurfaceRole::Clitic,
        }
    };
}

macro_rules! d {
    ($text:literal) => {
        PronounSurface {
            text: $text,
            role: SurfaceRole::DisputedClitic,
        }
    };
}

const EMPTY: &[PronounSurface] = &[];
const NO_TEXTS: &[&str] = &[];

fn by_recension<T: ?Sized>(
    recension: Recension,
    ocs: &'static T,
    synodal: &'static T,
    empty: &'static T,
) -> &'static T {
    match recension {
        Recension::OldChurchSlavonic => ocs,
        Recension::SynodalRussian => synodal,
        _ => empty,
    }
}

/// The two person-bearing personal-pronoun paradigms.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PersonalParadigm {
    First,
    Second,
}

/// One suppletive personal-pronoun cell. Vocatives are empty in both
/// recensions.
#[must_use]
pub fn personal_cell(
    paradigm: PersonalParadigm,
    case: Case,
    number: Number,
    recension: Recension,
) -> &'static [PronounSurface] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Number::{Dual, Plural, Singular};

    let (ocs, syn): (&[PronounSurface], &[PronounSurface]) = match (paradigm, case, number) {
        (PersonalParadigm::First, Nominative, Singular) => (&[p!("азъ")], &[p!("азъ")]),
        (PersonalParadigm::First, Genitive, Singular) => (&[p!("мене")], &[p!("мене")]),
        // realization: gen:jer-medial (мьнѣ → мнѣ).
        (PersonalParadigm::First, Dative, Singular) => {
            (&[p!("мьнѣ"), c!("ми")], &[p!("мнѣ"), c!("ми")])
        }
        // divergences pron:genitive-accusative, pron:accusative-clitic-status.
        (PersonalParadigm::First, Accusative, Singular) => (&[p!("мѧ")], &[p!("мене"), c!("мѧ")]),
        // realization: gen:jer-medial + gen:iotated-big-yus (мъноѭ → мною).
        (PersonalParadigm::First, Instrumental, Singular) => (&[p!("мъноѭ")], &[p!("мною")]),
        (PersonalParadigm::First, Locative, Singular) => (&[p!("мьнѣ")], &[p!("мнѣ")]),
        // divergence pron:dual-nominative-leveling.
        (PersonalParadigm::First, Nominative, Dual) => (&[p!("вѣ")], &[p!("мы")]),
        // divergence pron:dual-clitic-inventory (realization gen:yery on нꙑ).
        (PersonalParadigm::First, Accusative, Dual) => (&[p!("на"), c!("нꙑ")], &[c!("ны")]),
        (PersonalParadigm::First, Genitive | Locative, Dual) => (&[p!("наю")], &[p!("наю")]),
        // divergence pron:dual-clitic-inventory (the disputed dual dative на).
        (PersonalParadigm::First, Dative, Dual) => (&[p!("нама"), d!("на")], &[p!("нама")]),
        (PersonalParadigm::First, Instrumental, Dual) => (&[p!("нама")], &[p!("нама")]),
        // realization: gen:yery (мꙑ → мы).
        (PersonalParadigm::First, Nominative, Plural) => (&[p!("мꙑ")], &[p!("мы")]),
        // divergences pron:genitive-accusative, pron:accusative-clitic-status.
        (PersonalParadigm::First, Accusative, Plural) => (&[p!("нꙑ")], &[c!("ны"), p!("насъ")]),
        (PersonalParadigm::First, Genitive | Locative, Plural) => (&[p!("насъ")], &[p!("насъ")]),
        // divergence pron:dual-clitic-inventory (plural dative clitic нꙑ).
        (PersonalParadigm::First, Dative, Plural) => (&[p!("намъ"), c!("нꙑ")], &[p!("намъ")]),
        (PersonalParadigm::First, Instrumental, Plural) => (&[p!("нами")], &[p!("нами")]),

        // realization: gen:yery (тꙑ → ты).
        (PersonalParadigm::Second, Nominative, Singular) => (&[p!("тꙑ")], &[p!("ты")]),
        (PersonalParadigm::Second, Genitive, Singular) => (&[p!("тебе")], &[p!("тебе")]),
        (PersonalParadigm::Second, Dative, Singular) => {
            (&[p!("тебѣ"), c!("ти")], &[p!("тебѣ"), c!("ти")])
        }
        // divergences pron:genitive-accusative, pron:accusative-clitic-status.
        (PersonalParadigm::Second, Accusative, Singular) => (&[p!("тѧ")], &[p!("тебе"), c!("тѧ")]),
        // realization: gen:iotated-big-yus (тобоѭ → тобою).
        (PersonalParadigm::Second, Instrumental, Singular) => (&[p!("тобоѭ")], &[p!("тобою")]),
        (PersonalParadigm::Second, Locative, Singular) => (&[p!("тебѣ")], &[p!("тебѣ")]),
        // divergences pron:dual-nominative-leveling, pron:dual-clitic-inventory.
        (PersonalParadigm::Second, Nominative, Dual) => (&[p!("ва"), c!("вꙑ")], &[p!("вы")]),
        (PersonalParadigm::Second, Accusative, Dual) => (&[p!("ва"), c!("вꙑ")], &[c!("вы")]),
        (PersonalParadigm::Second, Genitive | Locative, Dual) => (&[p!("ваю")], &[p!("ваю")]),
        // divergence pron:dual-clitic-inventory (dual dative clitic ва).
        (PersonalParadigm::Second, Dative, Dual) => (&[p!("вама"), c!("ва")], &[p!("вама")]),
        (PersonalParadigm::Second, Instrumental, Dual) => (&[p!("вама")], &[p!("вама")]),
        (PersonalParadigm::Second, Nominative, Plural) => (&[p!("вꙑ")], &[p!("вы")]),
        // divergences pron:genitive-accusative, pron:accusative-clitic-status.
        (PersonalParadigm::Second, Accusative, Plural) => (&[p!("вꙑ")], &[c!("вы"), p!("васъ")]),
        (PersonalParadigm::Second, Genitive | Locative, Plural) => (&[p!("васъ")], &[p!("васъ")]),
        // divergence pron:dual-clitic-inventory (plural dative clitic вꙑ).
        (PersonalParadigm::Second, Dative, Plural) => (&[p!("вамъ"), c!("вꙑ")], &[p!("вамъ")]),
        (PersonalParadigm::Second, Instrumental, Plural) => (&[p!("вами")], &[p!("вами")]),
        (_, Vocative, _) => (EMPTY, EMPTY),
    };
    by_recension(recension, ocs, syn, EMPTY)
}

/// The numberless reflexive-pronoun cell. Nominative and vocative are empty
/// in both recensions.
#[must_use]
pub fn reflexive_cell(case: Case, recension: Recension) -> &'static [PronounSurface] {
    let (ocs, syn): (&[PronounSurface], &[PronounSurface]) = match case {
        Case::Nominative | Case::Vocative => (EMPTY, EMPTY),
        // divergences pron:genitive-accusative, pron:accusative-clitic-status.
        Case::Accusative => (&[p!("сѧ")], &[p!("себе"), c!("сѧ")]),
        Case::Genitive => (&[p!("себе")], &[p!("себе")]),
        Case::Locative => (&[p!("себѣ")], &[p!("себѣ")]),
        Case::Dative => (&[p!("себѣ"), c!("си")], &[p!("себѣ"), c!("си")]),
        // realization: gen:iotated-big-yus (собоѭ → собою).
        Case::Instrumental => (&[p!("собоѭ")], &[p!("собою")]),
    };
    by_recension(recension, ocs, syn, EMPTY)
}

/// One third-person anaphoric cell, conditioned on the post-prepositional
/// `н-` environment. The OCS side ignores `animacy` (divergence
/// `pron:genitive-accusative`); its nominative is empty (divergence
/// `pron:third-person-nominative-on`), while its free locative is attested
/// (divergence `pron:third-person-locative-postprepositional` — the Synodal
/// free locative is empty and the Synodal family rejects the cell).
/// Realization throughout: `gen:iotated-e` (ѥ → е), the Synodal є/ѡ/ꙋ
/// letters are that family's positional typography, and the OCS palatal
/// mark `н҄` is written plain `н` in Synodal.
#[must_use]
pub fn anaphoric_cell(
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
    after_preposition: bool,
    recension: Recension,
) -> &'static [&'static str] {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative, Vocative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};

    let (ocs, syn): (&[&str], &[&str]) = match (case, number, gender, after_preposition) {
        (Genitive, Singular, Masculine | Neuter, false) => (&["ѥго"], &["єгѡ"]),
        (Genitive, Singular, Masculine | Neuter, true) => (&["н҄ѥго"], &["негѡ"]),
        (Genitive, Singular, Feminine, false) => (&["ѥѩ"], &["єѧ"]),
        (Genitive, Singular, Feminine, true) => (&["н҄ѥѩ"], &["неѧ"]),
        (Dative, Singular, Masculine | Neuter, false) => (&["ѥму"], &["ємꙋ"]),
        (Dative, Singular, Masculine | Neuter, true) => (&["н҄ѥму"], &["немꙋ"]),
        (Dative, Singular, Feminine, false) => (&["ѥи"], &["єй"]),
        (Dative, Singular, Feminine, true) => (&["н҄ѥи"], &["ней"]),
        // divergence pron:genitive-accusative on the Synodal animate arm.
        (Accusative, Singular, Masculine, false) => (
            &["и"],
            if animacy == Animacy::Animate {
                &["єго"]
            } else {
                &["и"]
            },
        ),
        (Accusative, Singular, Masculine, true) => (
            &["н҄ь"],
            if animacy == Animacy::Animate {
                &["него"]
            } else {
                &["нь"]
            },
        ),
        (Accusative, Singular, Neuter, false) => (&["ѥ"], &["є"]),
        (Accusative, Singular, Neuter, true) => (&["н҄ѥ"], &["не"]),
        // realization: gen:iotated-big-yus (ѭ → ю).
        (Accusative, Singular, Feminine, false) => (&["ѭ"], &["ю"]),
        (Accusative, Singular, Feminine, true) => (&["н҄ѭ"], &["ню"]),
        // divergence pron:instr-loc-sg-jer (имь → имъ).
        (Instrumental, Singular, Masculine | Neuter, false) => (&["имь"], &["имъ"]),
        (Instrumental, Singular, Masculine | Neuter, true) => (&["н҄имь"], &["нимъ"]),
        (Instrumental, Singular, Feminine, false) => (&["ѥѭ"], &["єю"]),
        (Instrumental, Singular, Feminine, true) => (&["н҄ѥѭ"], &["нею"]),
        // divergences pron:instr-loc-sg-jer and
        // pron:third-person-locative-postprepositional (free Synodal empty).
        (Locative, Singular, Masculine | Neuter, false) => (&["ѥмь"], NO_TEXTS),
        (Locative, Singular, Masculine | Neuter, true) => (&["н҄ѥмь"], &["немъ"]),
        (Locative, Singular, Feminine, false) => (&["ѥи"], NO_TEXTS),
        (Locative, Singular, Feminine, true) => (&["н҄ѥи"], &["ней"]),
        // divergence pron:third-person-nominative-on (OCS nominative empty).
        (Nominative, Singular, Masculine, false) => (NO_TEXTS, &["онъ"]),
        (Nominative, Singular, Feminine, false) => (NO_TEXTS, &["она"]),
        (Nominative, Singular, Neuter, false) => (NO_TEXTS, &["оно"]),
        (Nominative, Dual, Masculine, false) => (NO_TEXTS, &["она"]),
        (Nominative, Dual, Feminine, false) => (NO_TEXTS, &["онѣ"]),
        (Nominative, Dual, Neuter, false) => (NO_TEXTS, &["онѣ", "она"]),
        (Nominative, Plural, Masculine | Neuter, false) => (NO_TEXTS, &["они"]),
        (Nominative, Plural, Feminine, false) => (NO_TEXTS, &["онѣ"]),
        (Nominative, _, _, true) => (NO_TEXTS, NO_TEXTS),

        // divergence pron:dual-accusative-gender-leveling (ꙗ/и → ѧ).
        (Accusative, Dual, Masculine, false) => (&["ꙗ"], &["ѧ"]),
        (Accusative, Dual, Feminine | Neuter, false) => (&["и"], &["ѧ"]),
        (Accusative, Dual, Masculine, true) => (&["н҄ꙗ"], &["нѧ"]),
        (Accusative, Dual, Feminine | Neuter, true) => (&["н҄и"], &["нѧ"]),
        (Genitive | Locative, Dual, _, false) => (&["ѥю"], &["єю"]),
        (Genitive | Locative, Dual, _, true) => (&["н҄ѥю"], &["нею"]),
        (Dative | Instrumental, Dual, _, false) => (&["има"], &["има"]),
        (Dative | Instrumental, Dual, _, true) => (&["н҄има"], &["нима"]),

        // divergence pron:genitive-accusative on the Synodal animate arms;
        // realization gen:iotated-small-yus (ѩ → ѧ) and fold:ja on neuter.
        (Accusative, Plural, Masculine | Feminine, false) => (
            &["ѩ"],
            if animacy == Animacy::Animate {
                &["ихъ"]
            } else {
                &["ѧ"]
            },
        ),
        (Accusative, Plural, Masculine | Feminine, true) => (
            &["н҄ѩ"],
            if animacy == Animacy::Animate {
                &["нихъ"]
            } else {
                &["нѧ"]
            },
        ),
        (Accusative, Plural, Neuter, false) => (&["ꙗ"], &["ѧ"]),
        (Accusative, Plural, Neuter, true) => (&["н҄ꙗ"], &["нѧ"]),
        (Genitive | Locative, Plural, _, false) => (&["ихъ"], &["ихъ"]),
        (Genitive | Locative, Plural, _, true) => (&["н҄ихъ"], &["нихъ"]),
        (Dative, Plural, _, false) => (&["имъ"], &["имъ"]),
        (Dative, Plural, _, true) => (&["н҄имъ"], &["нимъ"]),
        (Instrumental, Plural, _, false) => (&["ими"], &["ими"]),
        (Instrumental, Plural, _, true) => (&["н҄ими"], &["ними"]),
        (Vocative, _, _, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn, NO_TEXTS)
}

/// The nominative base of the relative pronoun (the `-же` compound's host:
/// иже, ѥже/єже, ꙗже, …). Oblique relative cells are the anaphoric cells
/// plus `-же`, composed by the families.
#[must_use]
pub fn relative_nominative_base(
    number: Number,
    gender: Gender,
    recension: Recension,
) -> &'static [&'static str] {
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    let (ocs, syn): (&[&str], &[&str]) = match (number, gender) {
        (Singular, Masculine) => (&["и"], &["и"]),
        // realization: gen:iotated-e (ѥ → е, Synodal typographic є).
        (Singular, Neuter) => (&["ѥ"], &["є"]),
        (Singular, Feminine) => (&["ꙗ"], &["ꙗ"]),
        // realization: fold:ja (ꙗ ~ ѧ) on the masculine dual.
        (Dual, Masculine) => (&["ꙗ"], &["ѧ"]),
        (Dual, Feminine | Neuter) => (&["и"], &["и"]),
        (Plural, Masculine) => (&["и"], &["и"]),
        (Plural, Neuter) => (&["ꙗ"], &["ꙗ"]),
        // realization: gen:iotated-small-yus and fold:ja meet in the shared
        // comparison key ѧ (ѩже ~ ꙗже are fold-equivalent).
        (Plural, Feminine) => (&["ѩ"], &["ꙗ"]),
    };
    by_recension(recension, ocs, syn, NO_TEXTS)
}

/// The two numberless, genderless interrogative pronouns.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum InterrogativeParadigm {
    Kto,
    Chto,
}

/// One interrogative cell. Realization: `gen:jer-medial` on the nominatives
/// (къто → кто, чьто → что) and `pron:instr-loc-sg-jer` on -мь/-мъ.
#[must_use]
pub fn interrogative_cell(
    paradigm: InterrogativeParadigm,
    case: Case,
    recension: Recension,
) -> &'static [PronounSurface] {
    let (ocs, syn): (&[PronounSurface], &[PronounSurface]) = match (paradigm, case) {
        (InterrogativeParadigm::Kto, Case::Nominative) => (&[p!("къто")], &[p!("кто")]),
        // divergence pron:genitive-accusative (Synodal accusative кого).
        (InterrogativeParadigm::Kto, Case::Accusative) => (&[p!("къто")], &[p!("кого")]),
        (InterrogativeParadigm::Kto, Case::Genitive) => (&[p!("кого")], &[p!("когѡ")]),
        (InterrogativeParadigm::Kto, Case::Dative) => (&[p!("кому")], &[p!("комꙋ")]),
        // divergence pron:kto-instrumental-stem (цѣмь vs кимъ).
        (InterrogativeParadigm::Kto, Case::Instrumental) => (&[p!("цѣмь")], &[p!("кимъ")]),
        (InterrogativeParadigm::Kto, Case::Locative) => (&[p!("комь")], &[p!("комъ")]),
        (InterrogativeParadigm::Chto, Case::Nominative) => (&[p!("чьто")], &[p!("что")]),
        // divergence pron:chto-oblique-inventory (accusative adds чесо).
        (InterrogativeParadigm::Chto, Case::Accusative) => {
            (&[p!("чьто")], &[p!("что"), v!("чесо")])
        }
        // divergence pron:chto-oblique-inventory.
        (InterrogativeParadigm::Chto, Case::Genitive) => (
            &[p!("чесо"), v!("чьсо"), v!("чесого")],
            &[p!("чегѡ"), v!("чесѡ"), v!("чесогѡ")],
        ),
        (InterrogativeParadigm::Chto, Case::Dative) => (
            &[p!("чему"), v!("чесому"), v!("чьсому")],
            &[p!("чемꙋ"), v!("чесомꙋ")],
        ),
        (InterrogativeParadigm::Chto, Case::Instrumental) => (&[p!("чимь")], &[p!("чимъ")]),
        (InterrogativeParadigm::Chto, Case::Locative) => {
            (&[p!("чемь"), v!("чесомь")], &[p!("чемъ"), v!("чесомъ")])
        }
        (_, Case::Vocative) => (EMPTY, EMPTY),
    };
    by_recension(recension, ocs, syn, EMPTY)
}

/// One cell of the proximal demonstrative (OCS сь, Synodal сей). The OCS
/// side ignores `animacy`. Direct-cell reshapes are divergence
/// `pron:proximal-nominative-reshape`; animate accusatives are
/// `pron:genitive-accusative`; -мь/-мъ is `pron:instr-loc-sg-jer`; the
/// Synodal і/ї letters are positional typography over shared и.
#[must_use]
pub fn proximal_cell(
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
        (Nominative, Singular, Masculine) => (&["сь"], &["сей", "сій"]),
        (Accusative, Singular, Masculine) => (
            &["сь"],
            if animate {
                &["сего"]
            } else {
                &["сей", "сій"]
            },
        ),
        (Nominative, Singular, Feminine) => (&["си"], &["сїѧ"]),
        (Nominative | Accusative, Singular, Neuter) => (&["се"], &["сїе"]),
        // realization: gen:iotated-big-yus (сиѭ → сию, typographic сїю).
        (Accusative, Singular, Feminine) => (&["сиѭ"], &["сїю"]),
        (Genitive, Singular, Masculine | Neuter) => (&["сего"], &["сегѡ"]),
        (Genitive, Singular, Feminine) => (&["сеѩ"], &["сеѧ"]),
        (Dative, Singular, Masculine | Neuter) => (&["сему"], &["семꙋ"]),
        (Dative | Locative, Singular, Feminine) => (&["сеи"], &["сей"]),
        (Instrumental, Singular, Masculine | Neuter) => (&["симь"], &["симъ"]),
        (Instrumental, Singular, Feminine) => (&["сеѭ"], &["сею"]),
        (Locative, Singular, Masculine | Neuter) => (&["семь"], &["семъ"]),
        // divergence pron:proximal-nominative-reshape on the direct duals
        // (fold:ja relates сиꙗ ~ сїѧ; the feminine/neuter takes the full
        // shape сіи).
        (Nominative | Accusative, Dual, Masculine) => (&["сиꙗ"], &["сїѧ"]),
        (Nominative | Accusative, Dual, Feminine | Neuter) => (&["си"], &["сіи"]),
        (Genitive | Locative, Dual, _) => (&["сею"], &["сею"]),
        (Dative | Instrumental, Dual, _) => (&["сима"], &["сима"]),
        (Nominative, Plural, Masculine) => (&["сии"], &["сіи"]),
        // divergence pron:proximal-nominative-reshape (short си → full сїѧ).
        (Nominative, Plural, Neuter) => (&["си"], &["сїѧ"]),
        (Nominative, Plural, Feminine) => (&["сиѩ"], &["сїѧ"]),
        (Accusative, Plural, Masculine | Feminine) => {
            (&["сиѩ"], if animate { &["сихъ"] } else { &["сїѧ"] })
        }
        (Accusative, Plural, Neuter) => (&["си"], &["сїѧ"]),
        (Genitive | Locative, Plural, _) => (&["сихъ"], &["сихъ"]),
        (Dative, Plural, _) => (&["симъ"], &["симъ"]),
        (Instrumental, Plural, _) => (&["сими"], &["сими"]),
        (Vocative, _, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn, NO_TEXTS)
}

/// The regular agreeing pronominal declensions shared by the recensions.
/// The class inventories map: OCS hard 2/p ↔ Synodal short-hard; OCS soft
/// (вашь, нашь) ↔ Synodal mixed-possessive (нашъ); OCS *j* (мои) ↔ Synodal
/// soft (мой).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AgreeingClass {
    Hard,
    Soft,
    SoftJ,
}

/// The ending set of one regular agreeing cell (empty = invalid cell, e.g.
/// any vocative). The OCS side ignores `animacy` (divergence
/// `pron:genitive-accusative`); Synodal animate accusatives take the
/// genitive ending. Systematic realization: gen:big-yus (ѫ → у/ꙋ),
/// gen:iotated-big-yus (оѭ/еѭ/ѭ → ою/ею/ю), gen:iotated-e (ѥ- → е-),
/// gen:iotated-small-yus and fold:ja on plural direct endings, gen:yery,
/// and the Synodal positional letters й/ѡ/ꙋ/є. The masculine/neuter
/// instrumental and locative singular carry divergence
/// `pron:instr-loc-sg-jer`.
#[must_use]
pub fn agreeing_ending(
    class: AgreeingClass,
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
    let (ocs, syn): (&[&str], &[&str]) = match (class, case, number, gender) {
        // ---- hard (OCS тъ, такъ … ↔ Synodal он-, ов- short-hard) ----
        (AgreeingClass::Hard, Nominative, Singular, Masculine) => (&["ъ"], &["ъ"]),
        (AgreeingClass::Hard, Nominative, Singular, Feminine) => (&["а"], &["а"]),
        (AgreeingClass::Hard, Nominative, Singular, Neuter) => (&["о"], &["о"]),
        // divergence pron:genitive-accusative on the Synodal animate arm.
        (AgreeingClass::Hard, Accusative, Singular, Masculine) => {
            (&["ъ"], if animate { &["ого"] } else { &["ъ"] })
        }
        // realization: gen:big-yus (ѫ → у, typographic ꙋ).
        (AgreeingClass::Hard, Accusative, Singular, Feminine) => (&["ѫ"], &["ꙋ"]),
        (AgreeingClass::Hard, Accusative, Singular, Neuter) => (&["о"], &["о"]),
        (AgreeingClass::Hard, Genitive, Singular, Masculine | Neuter) => (&["ого"], &["огѡ"]),
        (AgreeingClass::Hard, Genitive, Singular, Feminine) => (&["оѩ"], &["оѧ"]),
        (AgreeingClass::Hard, Dative, Singular, Masculine | Neuter) => (&["ому"], &["омꙋ"]),
        (AgreeingClass::Hard, Dative | Locative, Singular, Feminine) => (&["ои"], &["ой"]),
        // divergence pron:instr-loc-sg-jer.
        (AgreeingClass::Hard, Instrumental, Singular, Masculine | Neuter) => (&["ѣмь"], &["ѣмъ"]),
        (AgreeingClass::Hard, Instrumental, Singular, Feminine) => (&["оѭ"], &["ою"]),
        (AgreeingClass::Hard, Locative, Singular, Masculine | Neuter) => (&["омь"], &["омъ"]),
        (AgreeingClass::Hard, Nominative | Accusative, Dual, Masculine) => (&["а"], &["а"]),
        (AgreeingClass::Hard, Nominative | Accusative, Dual, Feminine | Neuter) => (&["ѣ"], &["ѣ"]),
        // realization: fold:omega (ою ~ ѡю, Synodal dual disambiguation).
        (AgreeingClass::Hard, Genitive | Locative, Dual, _) => (&["ою"], &["ѡю"]),
        (AgreeingClass::Hard, Dative | Instrumental, Dual, _) => (&["ѣма"], &["ѣма"]),
        (AgreeingClass::Hard, Nominative, Plural, Masculine) => (&["и"], &["и"]),
        (AgreeingClass::Hard, Nominative, Plural, Feminine) => (&["ы"], &["ы"]),
        (AgreeingClass::Hard, Nominative, Plural, Neuter) => (&["а"], &["а"]),
        // divergence pron:genitive-accusative on the Synodal animate arm.
        (AgreeingClass::Hard, Accusative, Plural, Masculine | Feminine) => {
            (&["ы"], if animate { &["ѣхъ"] } else { &["ы"] })
        }
        (AgreeingClass::Hard, Accusative, Plural, Neuter) => (&["а"], &["а"]),
        (AgreeingClass::Hard, Genitive | Locative, Plural, _) => (&["ѣхъ"], &["ѣхъ"]),
        (AgreeingClass::Hard, Dative, Plural, _) => (&["ѣмъ"], &["ѣмъ"]),
        (AgreeingClass::Hard, Instrumental, Plural, _) => (&["ѣми"], &["ѣми"]),

        // ---- soft (OCS нашь ↔ Synodal нашъ mixed-possessive) ----
        // realization: post-husher jer hardening (gen:jer-final ь ~ ъ).
        (AgreeingClass::Soft, Nominative, Singular, Masculine) => (&["ь"], &["ъ"]),
        (AgreeingClass::Soft, Nominative, Singular, Feminine) => (&["а"], &["а"]),
        (AgreeingClass::Soft, Nominative, Singular, Neuter) => (&["е"], &["е"]),
        // divergence pron:genitive-accusative on the Synodal animate arm.
        (AgreeingClass::Soft, Accusative, Singular, Masculine) => {
            (&["ь"], if animate { &["его"] } else { &["ъ"] })
        }
        // realization: gen:big-yus (ѫ → у, typographic ꙋ).
        (AgreeingClass::Soft, Accusative, Singular, Feminine) => (&["ѫ"], &["ꙋ"]),
        (AgreeingClass::Soft, Accusative, Singular, Neuter) => (&["е"], &["е"]),
        (AgreeingClass::Soft, Genitive, Singular, Masculine | Neuter) => (&["его"], &["егѡ"]),
        (AgreeingClass::Soft, Genitive, Singular, Feminine) => (&["еѩ"], &["еѧ"]),
        (AgreeingClass::Soft, Dative, Singular, Masculine | Neuter) => (&["ему"], &["емꙋ"]),
        (AgreeingClass::Soft, Dative | Locative, Singular, Feminine) => (&["еи"], &["ей"]),
        // divergence pron:instr-loc-sg-jer.
        (AgreeingClass::Soft, Instrumental, Singular, Masculine | Neuter) => (&["имь"], &["имъ"]),
        (AgreeingClass::Soft, Instrumental, Singular, Feminine) => (&["еѭ"], &["ею"]),
        (AgreeingClass::Soft, Locative, Singular, Masculine | Neuter) => (&["емь"], &["емъ"]),
        (AgreeingClass::Soft, Nominative | Accusative, Dual, Masculine) => (&["а"], &["а"]),
        (AgreeingClass::Soft, Nominative | Accusative, Dual, Feminine | Neuter) => (&["и"], &["и"]),
        (AgreeingClass::Soft, Genitive | Locative, Dual, _) => (&["ею"], &["ею"]),
        (AgreeingClass::Soft, Dative | Instrumental, Dual, _) => (&["има"], &["има"]),
        (AgreeingClass::Soft, Nominative, Plural, Masculine) => (&["и"], &["и"]),
        (AgreeingClass::Soft, Nominative, Plural, Feminine) => (&["ѧ"], &["ѧ"]),
        (AgreeingClass::Soft, Nominative, Plural, Neuter) => (&["а"], &["а"]),
        // divergence pron:genitive-accusative on the Synodal animate arm.
        (AgreeingClass::Soft, Accusative, Plural, Masculine | Feminine) => {
            (&["ѧ"], if animate { &["ихъ"] } else { &["ѧ"] })
        }
        (AgreeingClass::Soft, Accusative, Plural, Neuter) => (&["а"], &["а"]),
        (AgreeingClass::Soft, Genitive | Locative, Plural, _) => (&["ихъ"], &["ихъ"]),
        // realization: и → ы after the husher stem (Synodal positional norm).
        (AgreeingClass::Soft, Dative, Plural, _) => (&["имъ"], &["ымъ"]),
        (AgreeingClass::Soft, Instrumental, Plural, _) => (&["ими"], &["ими"]),

        // ---- j-stem (OCS мои ↔ Synodal мой soft) ----
        // realization: fold:i-variants (и ~ й).
        (AgreeingClass::SoftJ, Nominative, Singular, Masculine) => (&["и"], &["й"]),
        // realization: fold:ja (ꙗ ~ ѧ).
        (AgreeingClass::SoftJ, Nominative, Singular, Feminine) => (&["ꙗ"], &["ѧ"]),
        // realization: gen:iotated-e (ѥ → е).
        (AgreeingClass::SoftJ, Nominative, Singular, Neuter) => (&["ѥ"], &["е"]),
        // divergence pron:genitive-accusative on the Synodal animate arm.
        (AgreeingClass::SoftJ, Accusative, Singular, Masculine) => {
            (&["и"], if animate { &["его"] } else { &["й"] })
        }
        // realization: gen:iotated-big-yus (ѭ → ю).
        (AgreeingClass::SoftJ, Accusative, Singular, Feminine) => (&["ѭ"], &["ю"]),
        (AgreeingClass::SoftJ, Accusative, Singular, Neuter) => (&["ѥ"], &["е"]),
        (AgreeingClass::SoftJ, Genitive, Singular, Masculine | Neuter) => (&["ѥго"], &["егѡ"]),
        (AgreeingClass::SoftJ, Genitive, Singular, Feminine) => (&["ѥѩ"], &["еѧ"]),
        (AgreeingClass::SoftJ, Dative, Singular, Masculine | Neuter) => (&["ѥму"], &["емꙋ"]),
        (AgreeingClass::SoftJ, Dative | Locative, Singular, Feminine) => (&["ѥи"], &["ей"]),
        // divergence pron:instr-loc-sg-jer.
        (AgreeingClass::SoftJ, Instrumental, Singular, Masculine | Neuter) => (&["имь"], &["имъ"]),
        (AgreeingClass::SoftJ, Instrumental, Singular, Feminine) => (&["ѥѭ"], &["ею"]),
        (AgreeingClass::SoftJ, Locative, Singular, Masculine | Neuter) => (&["ѥмь"], &["емъ"]),
        (AgreeingClass::SoftJ, Nominative | Accusative, Dual, Masculine) => (&["ꙗ"], &["ѧ"]),
        (AgreeingClass::SoftJ, Nominative | Accusative, Dual, Feminine | Neuter) => {
            (&["и"], &["и"])
        }
        // realization: gen:iotated-e (Synodal typographic єю).
        (AgreeingClass::SoftJ, Genitive | Locative, Dual, _) => (&["ѥю"], &["єю"]),
        (AgreeingClass::SoftJ, Dative | Instrumental, Dual, _) => (&["има"], &["има"]),
        (AgreeingClass::SoftJ, Nominative, Plural, Masculine) => (&["и"], &["и"]),
        // realization: gen:iotated-small-yus (ѩ → ѧ) / fold:ja (ꙗ ~ ѧ).
        (AgreeingClass::SoftJ, Nominative, Plural, Feminine | Neuter) => (
            if gender == Feminine {
                &["ѩ"]
            } else {
                &["ꙗ"]
            },
            &["ѧ"],
        ),
        // divergence pron:genitive-accusative on the Synodal animate arm.
        (AgreeingClass::SoftJ, Accusative, Plural, Masculine | Feminine) => {
            (&["ѩ"], if animate { &["ихъ"] } else { &["ѧ"] })
        }
        (AgreeingClass::SoftJ, Accusative, Plural, Neuter) => (&["ꙗ"], &["ѧ"]),
        (AgreeingClass::SoftJ, Genitive | Locative, Plural, _) => (&["ихъ"], &["ихъ"]),
        (AgreeingClass::SoftJ, Dative, Plural, _) => (&["имъ"], &["имъ"]),
        (AgreeingClass::SoftJ, Instrumental, Plural, _) => (&["ими"], &["ими"]),

        (_, Vocative, _, _) => (NO_TEXTS, NO_TEXTS),
    };
    by_recension(recension, ocs, syn, NO_TEXTS)
}

/// Second palatalization of a stem-final velar before a front-vowel ending.
/// к → ц and х → с in both recensions; г → ѕ in OCS against з in Synodal
/// (realization: projection rule gen:zelo, ѕ ~ з). Returns `None` when the
/// stem does not end in a velar or the recension is unsupported.
#[must_use]
pub fn palatalize_final_velar(stem: &str, recension: Recension) -> Option<String> {
    let g_reflex = match recension {
        Recension::OldChurchSlavonic => "ѕ",
        Recension::SynodalRussian => "з",
        _ => return None,
    };
    let (base, replacement) = if let Some(base) = stem.strip_suffix('к') {
        (base, "ц")
    } else if let Some(base) = stem.strip_suffix('г') {
        (base, g_reflex)
    } else {
        (stem.strip_suffix('х')?, "с")
    };
    let mut result = String::with_capacity(stem.len());
    result.push_str(base);
    result.push_str(replacement);
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::SynodalRussian;

    fn texts(surfaces: &[PronounSurface]) -> Vec<&'static str> {
        surfaces.iter().map(|surface| surface.text).collect()
    }

    #[test]
    fn unsupported_recensions_yield_empty_cells() {
        for recension in [Recension::OldRussian, Recension::Mixed, Recension::Unknown] {
            assert!(
                personal_cell(
                    PersonalParadigm::First,
                    Case::Nominative,
                    Number::Singular,
                    recension
                )
                .is_empty()
            );
            assert!(
                agreeing_ending(
                    AgreeingClass::Hard,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    recension
                )
                .is_empty()
            );
            assert!(palatalize_final_velar("толик", recension).is_none());
        }
    }

    #[test]
    fn genitive_accusative_divergence_marks_synodal_animate_cells() {
        // pron:genitive-accusative / pron:accusative-clitic-status.
        assert_eq!(
            texts(personal_cell(
                PersonalParadigm::First,
                Case::Accusative,
                Number::Singular,
                OCS
            )),
            ["мѧ"]
        );
        assert_eq!(
            texts(personal_cell(
                PersonalParadigm::First,
                Case::Accusative,
                Number::Singular,
                SYN
            )),
            ["мене", "мѧ"]
        );
        assert_eq!(
            anaphoric_cell(
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate,
                false,
                SYN
            ),
            ["єго"]
        );
        assert_eq!(
            anaphoric_cell(
                Case::Accusative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate,
                false,
                OCS
            ),
            ["и"]
        );
        assert_eq!(
            agreeing_ending(
                AgreeingClass::Hard,
                Case::Accusative,
                Number::Plural,
                Gender::Masculine,
                Animacy::Animate,
                SYN
            ),
            ["ѣхъ"]
        );
        assert_eq!(
            interrogative_cell(InterrogativeParadigm::Kto, Case::Accusative, OCS)[0].text,
            "къто"
        );
        assert_eq!(
            interrogative_cell(InterrogativeParadigm::Kto, Case::Accusative, SYN)[0].text,
            "кого"
        );
    }

    #[test]
    fn dual_divergences_level_nominatives_and_accusatives() {
        // pron:dual-nominative-leveling.
        assert_eq!(
            texts(personal_cell(
                PersonalParadigm::First,
                Case::Nominative,
                Number::Dual,
                OCS
            )),
            ["вѣ"]
        );
        assert_eq!(
            texts(personal_cell(
                PersonalParadigm::First,
                Case::Nominative,
                Number::Dual,
                SYN
            )),
            ["мы"]
        );
        // pron:dual-accusative-gender-leveling.
        assert_eq!(
            anaphoric_cell(
                Case::Accusative,
                Number::Dual,
                Gender::Feminine,
                Animacy::Inanimate,
                false,
                OCS
            ),
            ["и"]
        );
        assert_eq!(
            anaphoric_cell(
                Case::Accusative,
                Number::Dual,
                Gender::Feminine,
                Animacy::Inanimate,
                false,
                SYN
            ),
            ["ѧ"]
        );
    }

    #[test]
    fn third_person_nominative_and_locative_divergences_hold() {
        // pron:third-person-nominative-on.
        assert!(
            anaphoric_cell(
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                false,
                OCS
            )
            .is_empty()
        );
        assert_eq!(
            anaphoric_cell(
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                false,
                SYN
            ),
            ["онъ"]
        );
        // pron:third-person-locative-postprepositional.
        assert_eq!(
            anaphoric_cell(
                Case::Locative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                false,
                OCS
            ),
            ["ѥмь"]
        );
        assert!(
            anaphoric_cell(
                Case::Locative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                false,
                SYN
            )
            .is_empty()
        );
    }

    #[test]
    fn instrumental_and_interrogative_stem_divergences_hold() {
        // pron:instr-loc-sg-jer.
        assert_eq!(
            agreeing_ending(
                AgreeingClass::Hard,
                Case::Instrumental,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            ),
            ["ѣмь"]
        );
        assert_eq!(
            agreeing_ending(
                AgreeingClass::Hard,
                Case::Instrumental,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["ѣмъ"]
        );
        // pron:kto-instrumental-stem.
        assert_eq!(
            interrogative_cell(InterrogativeParadigm::Kto, Case::Instrumental, OCS)[0].text,
            "цѣмь"
        );
        assert_eq!(
            interrogative_cell(InterrogativeParadigm::Kto, Case::Instrumental, SYN)[0].text,
            "кимъ"
        );
        // pron:chto-oblique-inventory.
        assert_eq!(
            texts(interrogative_cell(
                InterrogativeParadigm::Chto,
                Case::Genitive,
                OCS
            )),
            ["чесо", "чьсо", "чесого"]
        );
        assert_eq!(
            texts(interrogative_cell(
                InterrogativeParadigm::Chto,
                Case::Genitive,
                SYN
            )),
            ["чегѡ", "чесѡ", "чесогѡ"]
        );
    }

    #[test]
    fn relative_and_proximal_reshapes_hold() {
        // realization: ѩже ~ ꙗже are fold-equivalent (both keys ѧ).
        assert_eq!(
            relative_nominative_base(Number::Plural, Gender::Feminine, OCS),
            ["ѩ"]
        );
        assert_eq!(
            relative_nominative_base(Number::Plural, Gender::Feminine, SYN),
            ["ꙗ"]
        );
        // pron:proximal-nominative-reshape.
        assert_eq!(
            proximal_cell(
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            ),
            ["сь"]
        );
        assert_eq!(
            proximal_cell(
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["сей", "сій"]
        );
    }

    #[test]
    fn velar_palatalization_differs_only_in_the_zelo_reflex() {
        assert_eq!(palatalize_final_velar("так", OCS).as_deref(), Some("тац"));
        assert_eq!(palatalize_final_velar("так", SYN).as_deref(), Some("тац"));
        assert_eq!(palatalize_final_velar("наг", OCS).as_deref(), Some("наѕ"));
        assert_eq!(palatalize_final_velar("наг", SYN).as_deref(), Some("наз"));
        assert_eq!(palatalize_final_velar("тих", SYN).as_deref(), Some("тис"));
        assert_eq!(palatalize_final_velar("наш", OCS), None);
    }

    #[test]
    fn every_nonvocative_agreeing_cell_is_populated_in_both_recensions() {
        for class in [
            AgreeingClass::Hard,
            AgreeingClass::Soft,
            AgreeingClass::SoftJ,
        ] {
            for case in Case::ALL {
                for number in Number::ALL {
                    for gender in Gender::ALL {
                        for animacy in Animacy::ALL {
                            for recension in [OCS, SYN] {
                                let endings = agreeing_ending(
                                    class, case, number, gender, animacy, recension,
                                );
                                assert_eq!(
                                    endings.is_empty(),
                                    case == Case::Vocative,
                                    "{class:?} {case:?} {number:?} {gender:?} {recension:?}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
