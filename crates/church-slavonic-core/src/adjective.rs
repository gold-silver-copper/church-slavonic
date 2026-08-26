//! The merged adjective inflection kernel (docs/UNIFIED_LANGUAGE_PROMPT.md,
//! execution plan step 4, third POS slice).
//!
//! One recension-conditioned ending table per shared agreement paradigm: the
//! hard and soft classes, each in the short (nominal) and long (compound)
//! declension. Every cell is written with both recensions side by side so
//! that a difference is always visibly one of:
//!
//! - **realization** — related by the declared projection rules of
//!   `church-slavonic-orthography::projection` (cited inline by rule id,
//!   e.g. `gen:yery`, `gen:big-yus`, `fold:ja`, `fold:uk`) or by a named
//!   Synodal spelling norm outside that rule set (checked by the
//!   realization-coherence test in the orthography crate);
//! - **a named divergence** — cited inline by its id in
//!   [`crate::divergence::NAMED`];
//! - **a per-recension lexical fact** — which never reaches this module:
//!   the Synodal-only velar/sibilant/possessive classes and comparison
//!   series, the OCS comparative principal-part machinery, and the
//!   suppletive кꙑи interrogative stay in the family cores (see
//!   [`crate::divergence::UNMERGED`]).
//!
//! The family cores are adapters over these tables: they own their stem
//! selection (palatalization seams, jer-j workstems, mobile vowels), their
//! interface types, validation, and error vocabularies, and they read their
//! own recension's column through thin shims. The adjective-backed closed
//! classes (the Synodal Full* pronoun classes, the -скїй and long hard
//! determiners, both families' ordinals, the OCS long-only identities and
//! collective/compound-ordinal stems) all reach these tables through those
//! same family shims, which is what retires their `unmerged:` couplings.
//! Recensions other than the two attested ones yield empty cells.
//!
//! Each recension's column stores that family's canonical kernel spelling
//! (OCS ꙑ/оу/ѥ/ꙗ/ѫ/ѭ; Synodal ы/ꙋ/е/ѧ/ю with positional й/ї/ѡ), because
//! runtime projection is deliberately ambiguous; the equivalence of the
//! realization pairs is checked by the orthography-crate coherence test.

use crate::{Animacy, Case, Gender, Number, Recension};

const NO_TEXTS: &[&str] = &[];

/// The two agreement classes shared by both recensions' adjective kernels.
///
/// The Synodal family's further classes (velar, sibilant, the possessive
/// suffixes, the `-їй` class) are derived family-side from these tables or
/// carried as Synodal-only paradigms; the OCS family derives its velar
/// palatalization seams from the hard class. Neither reaches this enum.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdjectiveClass {
    /// Polivanova's hard `2/a` subtype ↔ Alypy §53/§57 hard declension.
    Hard,
    /// Polivanova's soft `2/a` subtype ↔ Alypy §53/§57 soft declension.
    Soft,
}

impl AdjectiveClass {
    pub const ALL: [Self; 2] = [Self::Hard, Self::Soft];
}

/// One short (nominal) adjective ending. The OCS column is Polivanova's
/// twofold nominal declension (o/a-stem endings, gendered plural obliques);
/// the Synodal column is the Alypy §53 short table. Beyond letter-for-letter
/// realization the columns differ by:
///
/// - divergence `adj:short-oblique-pronominalization` (Synodal levels the
///   OCS nominal obliques — омь/емь, ома/ема/ама, genitive plural ъ/ь,
///   омъ/емъ/амъ, ꙑ/и/ами, ѣхъ/ихъ/ахъ — to the pronominal/long-shaped
///   ымъ/имъ, ыма/има, ыхъ/ихъ, ыми/ими series);
/// - divergence `adj:soft-short-palatal-vowel-series` (the Synodal soft
///   column generalizes ѧ/ю/и where OCS prints а/оу/ѧ after the soft stem);
/// - divergence `adj:short-vocative-leveling` (Synodal levels the feminine
///   and soft-masculine vocative to the nominative; the hard masculine
///   vocative -е is shared);
/// - divergence `pron:genitive-accusative` on the animate accusative arms
///   (both recensions mark them here, but with their own columns' shapes).
#[must_use]
pub fn short_ending(
    class: AdjectiveClass,
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
        // ---- hard short (Polivanova 2/a hard ↔ Alypy §53) ----
        (AdjectiveClass::Hard, Nominative, Singular, Masculine) => (&["ъ"], &["ъ"]),
        (AdjectiveClass::Hard, Nominative, Singular, Feminine) => (&["а"], &["а"]),
        (AdjectiveClass::Hard, Nominative | Accusative | Vocative, Singular, Neuter) => {
            (&["о"], &["о"])
        }
        // pron:genitive-accusative on the animate arms (shared shape here).
        (AdjectiveClass::Hard, Accusative, Singular, Masculine) => {
            if animate {
                (&["а"], &["а"])
            } else {
                (&["ъ"], &["ъ"])
            }
        }
        // realization: gen:big-yus (ѫ → у, typographic ꙋ).
        (AdjectiveClass::Hard, Accusative, Singular, Feminine) => (&["ѫ"], &["ꙋ"]),
        (AdjectiveClass::Hard, Genitive, Singular, Masculine | Neuter) => (&["а"], &["а"]),
        // realization: gen:yery (ꙑ ~ ы).
        (AdjectiveClass::Hard, Genitive, Singular, Feminine) => (&["ꙑ"], &["ы"]),
        // realization: fold:uk (оу ~ ꙋ).
        (AdjectiveClass::Hard, Dative, Singular, Masculine | Neuter) => (&["оу"], &["ꙋ"]),
        (AdjectiveClass::Hard, Dative | Locative, Singular, Feminine) => (&["ѣ"], &["ѣ"]),
        // adj:short-oblique-pronominalization.
        (AdjectiveClass::Hard, Instrumental, Singular, Masculine | Neuter) => (&["омь"], &["ымъ"]),
        // realization: gen:iotated-big-yus (оѭ ~ ою).
        (AdjectiveClass::Hard, Instrumental, Singular, Feminine) => (&["оѭ"], &["ою"]),
        (AdjectiveClass::Hard, Locative, Singular, Masculine | Neuter) => (&["ѣ"], &["ѣ"]),
        (AdjectiveClass::Hard, Vocative, Singular, Masculine) => (&["е"], &["е"]),
        // adj:short-vocative-leveling (OCS -о against nominative-shaped -а).
        (AdjectiveClass::Hard, Vocative, Singular, Feminine) => (&["о"], &["а"]),
        (AdjectiveClass::Hard, Nominative | Accusative | Vocative, Dual, Masculine) => {
            (&["а"], &["а"])
        }
        (AdjectiveClass::Hard, Nominative | Accusative | Vocative, Dual, Feminine | Neuter) => {
            (&["ѣ"], &["ѣ"])
        }
        // realization: fold:uk (оу ~ ꙋ).
        (AdjectiveClass::Hard, Genitive | Locative, Dual, _) => (&["оу"], &["ꙋ"]),
        // adj:short-oblique-pronominalization (ома/ама → ыма).
        (AdjectiveClass::Hard, Dative | Instrumental, Dual, Masculine | Neuter) => {
            (&["ома"], &["ыма"])
        }
        (AdjectiveClass::Hard, Dative | Instrumental, Dual, Feminine) => (&["ама"], &["ыма"]),
        (AdjectiveClass::Hard, Nominative | Vocative, Plural, Masculine) => (&["и"], &["и"]),
        // realization: gen:yery (ꙑ ~ ы).
        (AdjectiveClass::Hard, Nominative | Vocative, Plural, Feminine) => (&["ꙑ"], &["ы"]),
        (AdjectiveClass::Hard, Nominative | Vocative, Plural, Neuter) => (&["а"], &["а"]),
        // pron:genitive-accusative and adj:short-oblique-pronominalization
        // on the animate arm; realization gen:yery on the inanimate arm.
        (AdjectiveClass::Hard, Accusative, Plural, Masculine) => {
            if animate {
                (&["ъ"], &["ыхъ"])
            } else {
                (&["ꙑ"], &["ы"])
            }
        }
        (AdjectiveClass::Hard, Accusative, Plural, Feminine) => (&["ꙑ"], &["ы"]),
        (AdjectiveClass::Hard, Accusative, Plural, Neuter) => (&["а"], &["а"]),
        // adj:short-oblique-pronominalization (genitive plural ъ → ыхъ).
        (AdjectiveClass::Hard, Genitive, Plural, _) => (&["ъ"], &["ыхъ"]),
        // adj:short-oblique-pronominalization (омъ/амъ → ымъ).
        (AdjectiveClass::Hard, Dative, Plural, Masculine | Neuter) => (&["омъ"], &["ымъ"]),
        (AdjectiveClass::Hard, Dative, Plural, Feminine) => (&["амъ"], &["ымъ"]),
        // adj:short-oblique-pronominalization (ꙑ/ами → ыми).
        (AdjectiveClass::Hard, Instrumental, Plural, Masculine | Neuter) => (&["ꙑ"], &["ыми"]),
        (AdjectiveClass::Hard, Instrumental, Plural, Feminine) => (&["ами"], &["ыми"]),
        // adj:short-oblique-pronominalization (ѣхъ/ахъ → ыхъ).
        (AdjectiveClass::Hard, Locative, Plural, Masculine | Neuter) => (&["ѣхъ"], &["ыхъ"]),
        (AdjectiveClass::Hard, Locative, Plural, Feminine) => (&["ахъ"], &["ыхъ"]),

        // ---- soft short (Polivanova 2/a soft ↔ Alypy §53) ----
        (AdjectiveClass::Soft, Nominative, Singular, Masculine) => (&["ь"], &["ь"]),
        // adj:soft-short-palatal-vowel-series (а → ѧ).
        (AdjectiveClass::Soft, Nominative, Singular, Feminine) => (&["а"], &["ѧ"]),
        (AdjectiveClass::Soft, Nominative | Accusative | Vocative, Singular, Neuter) => {
            (&["е"], &["е"])
        }
        // pron:genitive-accusative with the columns' own genitive shapes.
        (AdjectiveClass::Soft, Accusative, Singular, Masculine) => {
            if animate {
                (&["а"], &["ѧ"])
            } else {
                (&["ь"], &["ь"])
            }
        }
        // realization: gen:big-yus (ѫ → ю after the soft stem, cf. вьсѫ ~ всю).
        (AdjectiveClass::Soft, Accusative, Singular, Feminine) => (&["ѫ"], &["ю"]),
        // adj:soft-short-palatal-vowel-series (а → ѧ).
        (AdjectiveClass::Soft, Genitive, Singular, Masculine | Neuter) => (&["а"], &["ѧ"]),
        // adj:soft-short-palatal-vowel-series (feminine genitive ѧ → и).
        (AdjectiveClass::Soft, Genitive, Singular, Feminine) => (&["ѧ"], &["и"]),
        // adj:soft-short-palatal-vowel-series (оу → ю).
        (AdjectiveClass::Soft, Dative, Singular, Masculine | Neuter) => (&["оу"], &["ю"]),
        (AdjectiveClass::Soft, Dative | Locative, Singular, Feminine) => (&["и"], &["и"]),
        // adj:short-oblique-pronominalization (емь → имъ).
        (AdjectiveClass::Soft, Instrumental, Singular, Masculine | Neuter) => (&["емь"], &["имъ"]),
        // realization: gen:iotated-big-yus (еѭ ~ ею).
        (AdjectiveClass::Soft, Instrumental, Singular, Feminine) => (&["еѭ"], &["ею"]),
        (AdjectiveClass::Soft, Locative, Singular, Masculine | Neuter) => (&["и"], &["и"]),
        // adj:short-vocative-leveling (OCS -е against nominative-shaped -ь).
        (AdjectiveClass::Soft, Vocative, Singular, Masculine) => (&["е"], &["ь"]),
        (AdjectiveClass::Soft, Vocative, Singular, Feminine) => (&["а"], &["ѧ"]),
        // adj:soft-short-palatal-vowel-series (а → ѧ).
        (AdjectiveClass::Soft, Nominative | Accusative | Vocative, Dual, Masculine) => {
            (&["а"], &["ѧ"])
        }
        (AdjectiveClass::Soft, Nominative | Accusative | Vocative, Dual, Feminine | Neuter) => {
            (&["и"], &["и"])
        }
        // adj:soft-short-palatal-vowel-series (оу → ю).
        (AdjectiveClass::Soft, Genitive | Locative, Dual, _) => (&["оу"], &["ю"]),
        // adj:short-oblique-pronominalization (ема/ама → има).
        (AdjectiveClass::Soft, Dative | Instrumental, Dual, Masculine | Neuter) => {
            (&["ема"], &["има"])
        }
        (AdjectiveClass::Soft, Dative | Instrumental, Dual, Feminine) => (&["ама"], &["има"]),
        (AdjectiveClass::Soft, Nominative | Vocative, Plural, Masculine) => (&["и"], &["и"]),
        // adj:soft-short-palatal-vowel-series (feminine plural ѧ → и).
        (AdjectiveClass::Soft, Nominative | Vocative, Plural, Feminine) => (&["ѧ"], &["и"]),
        // adj:soft-short-palatal-vowel-series (а → ѧ).
        (AdjectiveClass::Soft, Nominative | Vocative, Plural, Neuter) => (&["а"], &["ѧ"]),
        // pron:genitive-accusative + adj:short-oblique-pronominalization on
        // the Synodal animate arm (OCS keeps the nominal genitive plural ь).
        (AdjectiveClass::Soft, Accusative, Plural, Masculine) => {
            if animate {
                (&["ь"], &["ихъ"])
            } else {
                (&["ѧ"], &["и"])
            }
        }
        (AdjectiveClass::Soft, Accusative, Plural, Feminine) => {
            if animate {
                (&["ѧ"], &["ихъ"])
            } else {
                (&["ѧ"], &["и"])
            }
        }
        (AdjectiveClass::Soft, Accusative, Plural, Neuter) => (&["а"], &["ѧ"]),
        // adj:short-oblique-pronominalization (genitive plural ь → ихъ).
        (AdjectiveClass::Soft, Genitive, Plural, _) => (&["ь"], &["ихъ"]),
        // adj:short-oblique-pronominalization (емъ/амъ → имъ).
        (AdjectiveClass::Soft, Dative, Plural, Masculine | Neuter) => (&["емъ"], &["имъ"]),
        (AdjectiveClass::Soft, Dative, Plural, Feminine) => (&["амъ"], &["имъ"]),
        // adj:short-oblique-pronominalization (и/ами → ими).
        (AdjectiveClass::Soft, Instrumental, Plural, Masculine | Neuter) => (&["и"], &["ими"]),
        (AdjectiveClass::Soft, Instrumental, Plural, Feminine) => (&["ами"], &["ими"]),
        (AdjectiveClass::Soft, Locative, Plural, Masculine | Neuter) => (&["ихъ"], &["ихъ"]),
        // adj:short-oblique-pronominalization (ахъ → ихъ).
        (AdjectiveClass::Soft, Locative, Plural, Feminine) => (&["ахъ"], &["ихъ"]),
    };
    by_recension(recension, ocs, syn)
}

/// One long (compound) adjective ending. The OCS column is Polivanova's
/// uncontracted compound declension (ending + enclitic *jь in every cell);
/// the Synodal column is the Alypy §57 contracted table. Beyond
/// letter-for-letter realization the columns differ by:
///
/// - divergence `adj:long-contraction` (the projection study's predicted
///   major family: Synodal contracts the OCS vowel + ѥ/и sequences —
///   аѥго → агѡ, оуѥмоу → омꙋ, ꙑимь → ымъ, ѣѥмь → ѣмъ, ѫѭ → ою,
///   ꙑима → ыма, ꙑихъ/ꙑимъ/ꙑими → ыхъ/ымъ/ыми, and the soft-column
///   counterparts — with `pron:instr-loc-sg-jer` on the -мь/-мъ terminals);
/// - divergence `adj:soft-long-vowel-grade` (the Synodal soft column levels
///   stem-vowel grades beyond contraction: аꙗ → ѧѧ, ѧѩ → їѧ, the feminine
///   dative/locative ии → ей, оую → юю);
/// - divergence `pron:genitive-accusative` on the animate accusative arms.
#[must_use]
pub fn long_ending(
    class: AdjectiveClass,
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
        // ---- hard long (Polivanova compound hard ↔ Alypy §57) ----
        // realization: gen:yery + the Synodal positional й (ꙑи ~ ый).
        (AdjectiveClass::Hard, Nominative | Vocative, Singular, Masculine) => (&["ꙑи"], &["ый"]),
        // realization: gen:iotated-e (оѥ ~ ое).
        (AdjectiveClass::Hard, Nominative | Accusative | Vocative, Singular, Neuter) => {
            (&["оѥ"], &["ое"])
        }
        // realization: fold:ja (аꙗ ~ аѧ).
        (AdjectiveClass::Hard, Nominative | Vocative, Singular, Feminine) => (&["аꙗ"], &["аѧ"]),
        // adj:long-contraction (аѥго → агѡ, with fold:omega typography).
        (AdjectiveClass::Hard, Genitive, Singular, Masculine | Neuter) => (&["аѥго"], &["агѡ"]),
        // realization: gen:yery + gen:iotated-small-yus (ꙑѩ ~ ыѧ).
        (AdjectiveClass::Hard, Genitive, Singular, Feminine) => (&["ꙑѩ"], &["ыѧ"]),
        // adj:long-contraction (оуѥмоу → омꙋ).
        (AdjectiveClass::Hard, Dative, Singular, Masculine | Neuter) => (&["оуѥмоу"], &["омꙋ"]),
        // realization: the Synodal positional й (ѣи ~ ѣй).
        (AdjectiveClass::Hard, Dative | Locative, Singular, Feminine) => (&["ѣи"], &["ѣй"]),
        // pron:genitive-accusative + adj:long-contraction on the animate arm.
        (AdjectiveClass::Hard, Accusative, Singular, Masculine) => {
            if animate {
                (&["аѥго"], &["аго"])
            } else {
                (&["ꙑи"], &["ый"])
            }
        }
        // realization: gen:big-yus + gen:iotated-big-yus (ѫѭ ~ ꙋю).
        (AdjectiveClass::Hard, Accusative, Singular, Feminine) => (&["ѫѭ"], &["ꙋю"]),
        // adj:long-contraction (ꙑимь → ымъ, with pron:instr-loc-sg-jer).
        (AdjectiveClass::Hard, Instrumental, Singular, Masculine | Neuter) => (&["ꙑимь"], &["ымъ"]),
        // adj:long-contraction (ѫѭ → ою).
        (AdjectiveClass::Hard, Instrumental, Singular, Feminine) => (&["ѫѭ"], &["ою"]),
        // adj:long-contraction (ѣѥмь → ѣмъ, with pron:instr-loc-sg-jer).
        (AdjectiveClass::Hard, Locative, Singular, Masculine | Neuter) => (&["ѣѥмь"], &["ѣмъ"]),
        // realization: fold:ja (аꙗ ~ аѧ).
        (AdjectiveClass::Hard, Nominative | Accusative | Vocative, Dual, Masculine) => {
            (&["аꙗ"], &["аѧ"])
        }
        (AdjectiveClass::Hard, Nominative | Accusative | Vocative, Dual, Feminine | Neuter) => {
            (&["ѣи"], &["ѣи"])
        }
        // realization: fold:uk (оую ~ ꙋю).
        (AdjectiveClass::Hard, Genitive | Locative, Dual, _) => (&["оую"], &["ꙋю"]),
        // adj:long-contraction (ꙑима → ыма).
        (AdjectiveClass::Hard, Dative | Instrumental, Dual, _) => (&["ꙑима"], &["ыма"]),
        // realization: the Synodal positional ї (ии ~ їи).
        (AdjectiveClass::Hard, Nominative | Vocative, Plural, Masculine) => (&["ии"], &["їи"]),
        (AdjectiveClass::Hard, Nominative | Vocative, Plural, Feminine) => (&["ꙑѩ"], &["ыѧ"]),
        (AdjectiveClass::Hard, Nominative | Accusative | Vocative, Plural, Neuter) => {
            (&["аꙗ"], &["аѧ"])
        }
        // pron:genitive-accusative + adj:long-contraction on the animate
        // arms (the OCS column keeps the nominal feminine accusative ꙑѩ).
        (AdjectiveClass::Hard, Accusative, Plural, Masculine) => {
            if animate {
                (&["ꙑихъ"], &["ыхъ"])
            } else {
                (&["ꙑѩ"], &["ыѧ"])
            }
        }
        (AdjectiveClass::Hard, Accusative, Plural, Feminine) => {
            (&["ꙑѩ"], if animate { &["ыхъ"] } else { &["ыѧ"] })
        }
        // adj:long-contraction (ꙑихъ/ꙑимъ/ꙑими → ыхъ/ымъ/ыми).
        (AdjectiveClass::Hard, Genitive | Locative, Plural, _) => (&["ꙑихъ"], &["ыхъ"]),
        (AdjectiveClass::Hard, Dative, Plural, _) => (&["ꙑимъ"], &["ымъ"]),
        (AdjectiveClass::Hard, Instrumental, Plural, _) => (&["ꙑими"], &["ыми"]),

        // ---- soft long (Polivanova compound soft ↔ Alypy §57) ----
        // realization: the Synodal positional ї/й (ии ~ їй).
        (AdjectiveClass::Soft, Nominative | Vocative, Singular, Masculine) => (&["ии"], &["їй"]),
        // realization: gen:iotated-e (еѥ ~ ее).
        (AdjectiveClass::Soft, Nominative | Accusative | Vocative, Singular, Neuter) => {
            (&["еѥ"], &["ее"])
        }
        // adj:soft-long-vowel-grade (аꙗ → ѧѧ).
        (AdjectiveClass::Soft, Nominative | Vocative, Singular, Feminine) => (&["аꙗ"], &["ѧѧ"]),
        // adj:long-contraction + adj:soft-long-vowel-grade (аѥго → ѧгѡ).
        (AdjectiveClass::Soft, Genitive, Singular, Masculine | Neuter) => (&["аѥго"], &["ѧгѡ"]),
        // adj:soft-long-vowel-grade (ѧѩ → їѧ).
        (AdjectiveClass::Soft, Genitive, Singular, Feminine) => (&["ѧѩ"], &["їѧ"]),
        // adj:long-contraction (оуѥмоу → емꙋ in the soft grade).
        (AdjectiveClass::Soft, Dative, Singular, Masculine | Neuter) => (&["оуѥмоу"], &["емꙋ"]),
        // adj:soft-long-vowel-grade (feminine dative/locative ии → ей).
        (AdjectiveClass::Soft, Dative | Locative, Singular, Feminine) => (&["ии"], &["ей"]),
        // pron:genitive-accusative + adj:long-contraction on the animate arm.
        (AdjectiveClass::Soft, Accusative, Singular, Masculine) => {
            if animate {
                (&["аѥго"], &["ѧго"])
            } else {
                (&["ии"], &["їй"])
            }
        }
        // realization: gen:big-yus + gen:iotated-big-yus (ѫѭ ~ юю).
        (AdjectiveClass::Soft, Accusative, Singular, Feminine) => (&["ѫѭ"], &["юю"]),
        // adj:long-contraction (иимь → имъ, with pron:instr-loc-sg-jer).
        (AdjectiveClass::Soft, Instrumental, Singular, Masculine | Neuter) => (&["иимь"], &["имъ"]),
        // realization: gen:iotated-big-yus (еѭ ~ ею).
        (AdjectiveClass::Soft, Instrumental, Singular, Feminine) => (&["еѭ"], &["ею"]),
        // adj:long-contraction (иѥмь → емъ, with pron:instr-loc-sg-jer).
        (AdjectiveClass::Soft, Locative, Singular, Masculine | Neuter) => (&["иѥмь"], &["емъ"]),
        // adj:soft-long-vowel-grade (аꙗ → ѧѧ).
        (AdjectiveClass::Soft, Nominative | Accusative | Vocative, Dual, Masculine) => {
            (&["аꙗ"], &["ѧѧ"])
        }
        (AdjectiveClass::Soft, Nominative | Accusative | Vocative, Dual, Feminine | Neuter) => {
            (&["ии"], &["їи"])
        }
        // adj:soft-long-vowel-grade (оую → юю).
        (AdjectiveClass::Soft, Genitive | Locative, Dual, _) => (&["оую"], &["юю"]),
        // adj:long-contraction (иима → има).
        (AdjectiveClass::Soft, Dative | Instrumental, Dual, _) => (&["иима"], &["има"]),
        (AdjectiveClass::Soft, Nominative | Vocative, Plural, Masculine) => (&["ии"], &["їи"]),
        // adj:soft-long-vowel-grade (ѧѩ → їѧ).
        (AdjectiveClass::Soft, Nominative | Vocative, Plural, Feminine) => (&["ѧѩ"], &["їѧ"]),
        // adj:soft-long-vowel-grade (аꙗ → ѧѧ).
        (AdjectiveClass::Soft, Nominative | Accusative | Vocative, Plural, Neuter) => {
            (&["аꙗ"], &["ѧѧ"])
        }
        // pron:genitive-accusative + adj:long-contraction on the animate
        // arms (the OCS column keeps the nominal feminine accusative ѧѩ).
        (AdjectiveClass::Soft, Accusative, Plural, Masculine) => {
            if animate {
                (&["иихъ"], &["ихъ"])
            } else {
                (&["ѧѩ"], &["їѧ"])
            }
        }
        (AdjectiveClass::Soft, Accusative, Plural, Feminine) => {
            (&["ѧѩ"], if animate { &["ихъ"] } else { &["їѧ"] })
        }
        // adj:long-contraction (иихъ/иимъ/иими → ихъ/имъ/ими).
        (AdjectiveClass::Soft, Genitive | Locative, Plural, _) => (&["иихъ"], &["ихъ"]),
        (AdjectiveClass::Soft, Dative, Plural, _) => (&["иимъ"], &["имъ"]),
        (AdjectiveClass::Soft, Instrumental, Plural, _) => (&["иими"], &["ими"]),
    };
    by_recension(recension, ocs, syn)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::SynodalRussian;

    #[test]
    fn unsupported_recensions_yield_empty_cells() {
        for recension in [Recension::OldRussian, Recension::Mixed, Recension::Unknown] {
            for class in AdjectiveClass::ALL {
                assert!(
                    short_ending(
                        class,
                        Case::Genitive,
                        Number::Singular,
                        Gender::Masculine,
                        Animacy::Inanimate,
                        recension
                    )
                    .is_empty()
                );
                assert!(
                    long_ending(
                        class,
                        Case::Genitive,
                        Number::Singular,
                        Gender::Masculine,
                        Animacy::Inanimate,
                        recension
                    )
                    .is_empty()
                );
            }
        }
    }

    #[test]
    fn attested_recension_tables_are_total() {
        for recension in [OCS, SYN] {
            for class in AdjectiveClass::ALL {
                for case in Case::ALL {
                    for number in Number::ALL {
                        for gender in Gender::ALL {
                            for animacy in Animacy::ALL {
                                for cell in [
                                    short_ending(class, case, number, gender, animacy, recension),
                                    long_ending(class, case, number, gender, animacy, recension),
                                ] {
                                    assert_eq!(
                                        cell.len(),
                                        1,
                                        "{recension:?} {class:?} {case:?} {number:?} {gender:?}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn short_oblique_pronominalization_holds() {
        // adj:short-oblique-pronominalization.
        assert_eq!(
            short_ending(
                AdjectiveClass::Hard,
                Case::Instrumental,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            ),
            ["омь"]
        );
        assert_eq!(
            short_ending(
                AdjectiveClass::Hard,
                Case::Instrumental,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["ымъ"]
        );
        assert_eq!(
            short_ending(
                AdjectiveClass::Hard,
                Case::Genitive,
                Number::Plural,
                Gender::Feminine,
                Animacy::Inanimate,
                OCS
            ),
            ["ъ"]
        );
        assert_eq!(
            short_ending(
                AdjectiveClass::Hard,
                Case::Genitive,
                Number::Plural,
                Gender::Feminine,
                Animacy::Inanimate,
                SYN
            ),
            ["ыхъ"]
        );
    }

    #[test]
    fn long_contraction_holds() {
        // adj:long-contraction (благꙑимъ-type against the contracted column).
        assert_eq!(
            long_ending(
                AdjectiveClass::Hard,
                Case::Instrumental,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            ),
            ["ꙑимь"]
        );
        assert_eq!(
            long_ending(
                AdjectiveClass::Hard,
                Case::Instrumental,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["ымъ"]
        );
        assert_eq!(
            long_ending(
                AdjectiveClass::Hard,
                Case::Genitive,
                Number::Singular,
                Gender::Neuter,
                Animacy::Inanimate,
                OCS
            ),
            ["аѥго"]
        );
        assert_eq!(
            long_ending(
                AdjectiveClass::Hard,
                Case::Genitive,
                Number::Singular,
                Gender::Neuter,
                Animacy::Inanimate,
                SYN
            ),
            ["агѡ"]
        );
    }

    #[test]
    fn soft_columns_diverge_in_the_declared_grades() {
        // adj:soft-short-palatal-vowel-series.
        assert_eq!(
            short_ending(
                AdjectiveClass::Soft,
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            ),
            ["а"]
        );
        assert_eq!(
            short_ending(
                AdjectiveClass::Soft,
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["ѧ"]
        );
        // adj:soft-long-vowel-grade.
        assert_eq!(
            long_ending(
                AdjectiveClass::Soft,
                Case::Nominative,
                Number::Singular,
                Gender::Feminine,
                Animacy::Inanimate,
                OCS
            ),
            ["аꙗ"]
        );
        assert_eq!(
            long_ending(
                AdjectiveClass::Soft,
                Case::Nominative,
                Number::Singular,
                Gender::Feminine,
                Animacy::Inanimate,
                SYN
            ),
            ["ѧѧ"]
        );
        // adj:short-vocative-leveling.
        assert_eq!(
            short_ending(
                AdjectiveClass::Soft,
                Case::Vocative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                OCS
            ),
            ["е"]
        );
        assert_eq!(
            short_ending(
                AdjectiveClass::Soft,
                Case::Vocative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                SYN
            ),
            ["ь"]
        );
    }
}
