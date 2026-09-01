//! The v1.1 consumer-defect ledger (V1.1-PROMPT.md, Part 0).
//!
//! Every entry reproduces a form the `vertograd` consumer rejected during
//! its 2026-08-31 audits, with the DIAGNOSED class and the justification
//! for the expected value. Tests are born `#[ignore]`d and un-ignored by
//! the part that fixes them. Two of the consumer's rejections turned out
//! to be AUDIT defects — the crate's answers are attested table cells —
//! and are recorded here as guards instead (see `the_audit_was_wrong`).
//!
//! Diagnosis summary (verified against the committed tables 2026-08-31):
//! - Class A (lookup folding): `ѷ`-spelled input misses `ѵ`-spelled keys.
//! - Class M (fact mechanism): rows whose ATTESTED finite cells reveal the
//!   present stem still answer unattested cells by the blind rule
//!   (стрищѝ, дои́ти); the accusative-shape fact teaches upward only, so
//!   an attested nominative-shaped PLURAL accusative does not teach the
//!   singular (ѻ҆гꙋре́цъ).
//! - Class B (rule default): a wholly-unattested masculine answers the
//!   animate accusative (ко́локолъ, прꙋ́дъ, ѡ҆́блакъ) — Part 2 measures
//!   which default the held-out corpora prefer before flipping anything.
//! - Class C (data): «ꙗ҆́блонь» has no row and its -ь hides the gender —
//!   only a witness can settle it.

use church_slavonic::*;

const SYN: Recension = Recension::Synodal;
const SG: Number = Number::Singular;

fn acc(w: &str) -> String {
    ChurchSlavonic::noun(w, &Case::Accusative, &SG, &SYN)
}

// -------------------------------------------------------------------------
// Class A — lookup folding (Part 1)
// -------------------------------------------------------------------------

/// The table key is «кѵпарі́съ» (ѵ, U+0475) with the inanimate accusative
/// ATTESTED at cell 3. The same word spelled with ѷ (U+0477) must reach
/// the same row: `ѷ ~ ѵ` is a kendema-carrying spelling of one letter, and
/// `orthography` already folds it in `comparison_key` — the lookup fold
/// must agree with the comparison fold.
#[test]
fn izhitsa_spellings_reach_one_row() {
    assert_eq!(acc("кѵпарі́съ"), "кѵпарі\u{301}съ", "the ѵ spelling (guard)");
    assert_eq!(acc("кѷпарі́съ"), "кѵпарі\u{301}съ", "the ѷ spelling must fold");
}

// -------------------------------------------------------------------------
// Class M — the fact mechanism (Part 2)
// -------------------------------------------------------------------------

/// «стрищѝ» has a row with the present block ATTESTED («стриже́ши,
/// стриже́тъ, стригꙋ́тъ» — the velar stem is proved by the row itself),
/// yet the unattested l-participle falls to the blind rule («стри́клъ»).
/// With the stem read from the attested present, the l-participle is
/// «стри́глъ» (стриг- + -лъ, the велярный stem the row attests).
#[test]
#[ignore = "Part 2: derive the present stem from attested finite cells"]
fn strishchi_l_participle_uses_the_attested_stem() {
    assert_eq!(
        ChurchSlavonic::l_participle("стрищѝ", &Gender::Masculine, &SG, &SYN),
        "стри́глъ"
    );
}

/// «дои́ти» (milk) has its own row — «дои́ши, дои́тъ, доѧ́ше, дои́ша,
/// дои́ла» all attested — yet the unattested aorist 3sg answers «дои́де»
/// and the imperative «дои́ди»: the rule missegmented the lemma as
/// до+и҆тѝ. The row's attested present proves the i-verb stem «дои́-»;
/// the regular i-verb aorist 3sg is «доѝ» (as напои́ти : напоѝ, attested
/// crate behavior) and the imperative «до́й» (as напои́ти : напо́й).
#[test]
#[ignore = "Part 2: derive the present stem from attested finite cells"]
fn doiti_the_milk_verb_is_not_a_compound_of_iti() {
    assert_eq!(
        ChurchSlavonic::verb("дои́ти", &Person::Third, &SG, &Tense::Aorist, &Form::Finite, &SYN),
        "доѝ"
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "дои́ти",
            &Person::Second,
            &SG,
            &Tense::Present,
            &Form::Imperative,
            &SYN
        ),
        "до́й"
    );
}

/// «ѻ҆гꙋре́цъ» stores the nominative-shaped PLURAL accusative
/// («ѻ҆гꙋрцы̀», cell 17) — the row attests the word inanimate — but the
/// 0.9.0 shape fact reads lower cells only, so the singular accusative
/// still answers the animate shape. Any attested nominative-shaped
/// accusative teaches the others: acc sg = nom sg.
#[test]
#[ignore = "Part 2: the accusative-shape fact reads every attested accusative"]
fn ogurets_the_stored_plural_shape_teaches_the_singular() {
    assert_eq!(acc("ѻ҆гꙋре́цъ"), "ѻ\u{486}гꙋре\u{301}цъ");
}

// -------------------------------------------------------------------------
// Class B — the unattested-masculine default (Part 3; expectations
// CONDITIONAL on the held-out measurement — see V1.1-PROMPT Part 2)
// -------------------------------------------------------------------------

/// Wholly-unattested (or rule-reserved) masculines currently answer the
/// animate accusative. These three name things that are not persons or
/// animals; if Part 3's held-out measurement confirms the inanimate
/// default, they answer acc = nom — otherwise they are witnessed
/// individually (Part 4) and this test's justification moves to the
/// witness file.
#[test]
#[ignore = "Part 3: default (measured) or witness"]
fn unattested_inanimates_answer_the_nominative_shape() {
    assert_eq!(acc("ко́локолъ"), "ко́локолъ");
    assert_eq!(acc("прꙋ́дъ"), "прꙋ́дъ");
    assert_eq!(acc("ѡ҆́блакъ"), "ѡ\u{486}\u{301}блакъ");
}

// -------------------------------------------------------------------------
// Class C — data a witness must settle (Part 4)
// -------------------------------------------------------------------------

/// «ꙗ҆́блонь» has no row, and -ь hides the gender from the rule (masc jo
/// vs fem i) — it currently declines like «ко́нь» («ꙗ҆́блонѧ»). The word
/// is feminine i-stem in the print; only a witness row can teach that.
#[test]
#[ignore = "Part 4: witnessed as a feminine i-stem"]
fn jablon_is_a_feminine_i_stem() {
    assert_eq!(acc("ꙗ҆́блонь"), "ꙗ\u{486}\u{301}блонь");
}

// -------------------------------------------------------------------------
// Part 5 — the audit was wrong: these crate answers are ATTESTED cells
// and must never "regress" toward the consumer's mistaken expectations.
// -------------------------------------------------------------------------

#[test]
fn the_audit_was_wrong_these_are_attested() {
    // «вожжѝ» — imperative of «возжещѝ», cell 28 of its row (Polyakov);
    // the assimilated spelling is real print (variants «возжгѝ»,
    // «возжзѝ» live at the _2/_3 sense keys).
    assert_eq!(
        ChurchSlavonic::verb(
            "возжещѝ",
            &Person::Second,
            &SG,
            &Tense::Present,
            &Form::Imperative,
            &SYN
        ),
        "вожжѝ"
    );
    // «пожа́тъ» — aorist 3sg of «пожа́ти», cells 19/20 attested (the -ѧти
    // class takes -ѧ́тъ: «прїѧ́тъ»); the bare variant «пожа̀» is the _2
    // sense.
    assert_eq!(
        ChurchSlavonic::verb("пожа́ти", &Person::Third, &SG, &Tense::Aorist, &Form::Finite, &SYN),
        "пожа́тъ"
    );
    assert_eq!(
        ChurchSlavonic::verb("пожа́ти_2", &Person::Third, &SG, &Tense::Aorist, &Form::Finite, &SYN),
        "пожа̀"
    );
}
