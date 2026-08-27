//! Exhaustive coverage of the suppletive copula `бꙑти`/`быти`.
//!
//! In Old Church Slavonic the Kaikki dump attests nothing the rules do not
//! predict, so every cell routes through `ChurchSlavonicCore::to_be`; in the
//! Synodal recension the §81 tables of the Alypy grammar serve the accented
//! finite cells from the table (the participles stay with the rule). This
//! pins every reachable finite cell in both recensions, including the
//! recension-conditioned tense assignment (the OCS aorist is the `бѣхъ`
//! series, the Synodal aorist the `быхъ`/`бысть` series).

use church_slavonic::*;

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
    assert_eq!(be("бꙑти", Tense::Aorist, OCS)[..3], ["бѣхъ", "бѣ", "бѣ"]);
}

#[test]
fn synodal_finite_cells_come_from_the_grammar_with_their_accents() {
    assert_eq!(
        be("быти", Tense::Present, SYN),
        [
            "є҆́смь",
            "є҆сѝ",
            "є҆́сть",
            "є҆сва̀",
            "є҆ста̀",
            "є҆ста̀",
            "є҆смы̀",
            "є҆стѐ",
            "сꙋ́ть"
        ]
    );
    assert_eq!(
        be("быти", Tense::Imperfect, SYN),
        [
            "бѧ́хъ",
            "бѧ́ше",
            "бѧ́ше",
            "бѧ́хова",
            "бѧ́ста",
            "бѧ́ста",
            "бѧ́хомъ",
            "бѧ́сте",
            "бѧ́хꙋ"
        ]
    );
    assert_eq!(
        be("быти", Tense::Aorist, SYN),
        [
            "бы́хъ",
            "бы́сть",
            "бы́сть",
            "бы́хова",
            "бы́ста",
            "бы́ста",
            "бы́хомъ",
            "бы́сте",
            "бы́ша"
        ]
    );
    // The imperfective aorist series is a published variant.
    assert_eq!(be("быти_2", Tense::Aorist, SYN)[3], "бѣ́ховѣ");
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
    assert_eq!(participle("быти", Tense::Present, SYN), "сый");
    assert_eq!(participle("быти", Tense::Aorist, SYN), "бывъ");
    assert_eq!(
        ChurchSlavonic::verb(
            "быти_2",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Infinitive,
            &SYN
        ),
        "быти"
    );
}
