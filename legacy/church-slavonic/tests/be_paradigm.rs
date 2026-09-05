//! Exhaustive coverage of the suppletive copula `бꙑти`/`быти`.
//!
//! In Old Church Slavonic the Kaikki dump attests nothing the rules do not
//! predict, so every cell routes through `ChurchSlavonicCore::to_be`; in the
//! Synodal recension the rule's matrix carries the print's accents and the
//! §81 tables of the Alypy grammar and Polyakov's corpus paradigm add
//! their variants — both spell the copula, so the sort spreads them over
//! `бы́ти` and its `_n` keys. This pins every reachable finite cell in both recensions,
//! including the shared `бꙑхъ`/`быхъ` aorist series (the OCS `бѣ` series is
//! the imperfective aorist, filed under the imperfect by the treebanks).

use church_slavonic_legacy::*;

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::Synodal;
const PERSONS: [Person; 3] = [Person::First, Person::Second, Person::Third];
const NUMBERS: [Number; 3] = [Number::Singular, Number::Dual, Number::Plural];

fn be(word: &str, tense: Tense, r: Recension) -> Vec<String> {
    let mut out = Vec::new();
    for n in NUMBERS {
        for p in PERSONS {
            out.push(ChurchSlavonic::verb(
                word,
                &p,
                &n,
                &tense,
                &Form::Finite,
                &r,
            ));
        }
    }
    out
}

#[test]
fn old_church_slavonic_finite_cells() {
    assert_eq!(
        be("бꙑти", Tense::Present, OCS),
        [
            "ѥсмь", "ѥси", "ѥстъ", "ѥсвѣ", "ѥста", "ѥсте", "ѥсмъ", "ѥсте", "сѫтъ"
        ]
    );
    assert_eq!(be("бꙑти", Tense::Imperfect, OCS)[0], "бѣахъ");
    // The dump attests the imperfective-aorist `бѣхъ` as the first
    // singular's primary; the rule's `бꙑхъ` series holds the rest.
    assert_eq!(
        be("бꙑти", Tense::Aorist, OCS)[..3],
        ["бѣхъ", "бꙑстъ", "бꙑстъ"]
    );
}

/// The published keys of the Synodal copula: the bare lemma and every `_n`
/// key that resolves to a table row.
fn synodal_keys() -> Vec<String> {
    // An unpublished suffix reads the bare row, then the rule: probing a few
    // keys past the published ones costs nothing.
    std::iter::once("бы́ти".to_string())
        .chain((2..8).map(|n| format!("бы́ти_{n}")))
        .collect()
}

/// Every cell of `expected` is produced by some published key.
fn reachable(tense: Tense, expected: [&str; 9]) {
    let rows: Vec<Vec<String>> = synodal_keys().iter().map(|k| be(k, tense, SYN)).collect();
    for (i, cell) in expected.iter().enumerate() {
        assert!(
            rows.iter().any(|r| r[i] == *cell),
            "{cell} (cell {i}) not reachable through {:?}",
            rows.iter().map(|r| r[i].clone()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn synodal_finite_cells_come_from_the_grammar_with_their_accents() {
    reachable(
        Tense::Present,
        [
            "є҆́смь",
            "є҆сѝ",
            "є҆́сть",
            "є҆сва̀",
            "є҆ста̀",
            "є҆ста̀",
            "є҆смы̀",
            "є҆стѐ",
            "сꙋ́ть",
        ],
    );
    reachable(
        Tense::Imperfect,
        [
            "бѧ́хъ",
            "бѧ́ше",
            "бѧ́ше",
            "бѧ́хова",
            "бѧ́ста",
            "бѧ́ста",
            "бѧ́хомъ",
            "бѧ́сте",
            "бѧ́хꙋ",
        ],
    );
    reachable(
        Tense::Aorist,
        [
            "бы́хъ",
            "бы́сть",
            "бы́сть",
            "бы́хова",
            "бы́ста",
            "бы́ста",
            "бы́хомъ",
            "бы́сте",
            "бы́ша",
        ],
    );
    // The imperfective aorist series is a published variant too.
    reachable(
        Tense::Aorist,
        [
            "бѣ́хъ",
            "бѣ̀",
            "бѣ̀",
            "бѣ́ховѣ",
            "бѣ́ста",
            "бѣ́ста",
            "бѣ́хомъ",
            "бѣ́сте",
            "бѣ́ша",
        ],
    );
    // The corpus's other spellings are reachable alongside the grammar's.
    reachable(
        Tense::Present,
        [
            "є҆смь",
            "є҆си",
            "є҆сть",
            "є҆сма̀",
            "є҆стѣ̀",
            "є҆стѣ̀",
            "є҆́смы",
            "є҆стѐ",
            "сꙋть",
        ],
    );
}

#[test]
fn participles_and_infinitive_stay_with_the_rule() {
    let participle = |word: &str, tense: Tense, r: Recension| {
        ChurchSlavonic::verb(
            word,
            &Person::Third,
            &Number::Singular,
            &tense,
            &Form::Participle,
            &r,
        )
    };
    assert_eq!(participle("бꙑти", Tense::Present, OCS), "сꙑ");
    assert_eq!(participle("бꙑти", Tense::Aorist, OCS), "бꙑвъ");
    // Synodal: the corpus attests the participle citations, so some key
    // serves the accented primary; an unpublished suffix stays with the rule.
    let keys = synodal_keys();
    assert!(
        keys.iter()
            .any(|k| participle(k, Tense::Present, SYN) == "сꙋ́щь")
    );
    assert!(
        keys.iter()
            .any(|k| participle(k, Tense::Aorist, SYN) == "бы́въ")
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "бы́ти_2",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Infinitive,
            &SYN
        ),
        "бы́ти"
    );
}
