//! Exhaustive coverage of the `ChurchSlavonic::pronoun` matrix.
//!
//! The personal pronoun is a lemma-less matrix per recension, keyed
//! `personal` with its attested variants at `personal_<n>`
//! (`ChurchSlavonic::pronoun_sense`): the Kaikki `и`/`ꙗ`/`ѥ` tables (OCS)
//! and, for Synodal, Polyakov's corpus primaries (frequency-ordered, full
//! forms before enclitics) merged with §47 of the Alypy grammar — the
//! print outranking the transliteration where they differ only in ꙗ/ѧ or
//! the monosyllable's varia, the witnessed Bible lines outranking both
//! (v1.2 part 1) — over the rule's matrix in the print's typography. Gender is consulted only in the
//! third person; the vocative answers with the nominative. This pins every
//! cell of the first and second persons and the third-person singular in
//! both recensions, so a transposed cell or a wrong recension condition
//! cannot ship silently.

use church_slavonic::*;

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::Synodal;
const GENDERS: [Gender; 3] = [Gender::Masculine, Gender::Feminine, Gender::Neuter];
const CASES: [Case; 6] = [
    Case::Nominative,
    Case::Genitive,
    Case::Dative,
    Case::Accusative,
    Case::Instrumental,
    Case::Locative,
];

fn row(person: Person, number: Number, gender: Gender, r: Recension) -> Vec<&'static str> {
    CASES
        .iter()
        .map(|c| ChurchSlavonic::pronoun(&person, &number, &gender, c, &r))
        .collect()
}

#[test]
fn first_and_second_person_ignore_gender() {
    for g in GENDERS {
        assert_eq!(
            row(Person::First, Number::Singular, g, OCS),
            ["азъ", "мене", "мьнѣ", "мѧ", "мъноѭ", "мьнѣ"]
        );
        assert_eq!(
            row(Person::First, Number::Singular, g, SYN),
            ["а҆́зъ", "менє̀", "мнѣ̀", "менѐ", "мно́ю", "мнѣ̀"]
        );
        assert_eq!(
            row(Person::First, Number::Dual, g, OCS),
            ["вѣ", "наю", "нама", "на", "нама", "наю"]
        );
        assert_eq!(
            row(Person::First, Number::Dual, g, SYN),
            ["мы̀", "на́ю", "на́ма", "ны̀", "на́ма", "на́ю"]
        );
        assert_eq!(
            row(Person::First, Number::Plural, g, OCS),
            ["мꙑ", "насъ", "намъ", "нꙑ", "нами", "насъ"]
        );
        assert_eq!(
            row(Person::First, Number::Plural, g, SYN),
            ["мы̀", "на́съ", "на́мъ", "на́съ", "на́ми", "на́съ"]
        );
        assert_eq!(
            row(Person::Second, Number::Singular, g, OCS),
            ["тꙑ", "тебе", "тебѣ", "тѧ", "тобоѭ", "тебѣ"]
        );
        assert_eq!(
            row(Person::Second, Number::Singular, g, SYN),
            ["ты̀", "тебє̀", "тебѣ̀", "тебѐ", "тобо́ю", "тебѣ̀"]
        );
        assert_eq!(
            row(Person::Second, Number::Plural, g, SYN),
            ["вы̀", "ва́съ", "ва́мъ", "ва́съ", "ва́ми", "ва́съ"]
        );
    }
}

#[test]
fn third_person_singular_varies_by_gender() {
    // OCS: the anaphoric nominatives are the Wiktionary `и`/`ꙗ`/`ѥ` cells;
    // the oblique cells equal the rule (Kaikki spells `его` for `ѥго`).
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Masculine, OCS),
        ["и", "ѥго", "ѥмоу", "и", "имь", "ѥмь"]
    );
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Feminine, OCS),
        ["ꙗ", "ѥѩ", "ѥи", "ѭ", "ѥѭ", "ѥи"]
    );
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Neuter, OCS),
        ["ѥ", "ѥго", "ѥмоу", "ѥ", "имь", "ѥмь"]
    );
    // Synodal: the nominatives are the ѻ҆́нъ series (Alypy §47; the
    // anaphor «и҆̀» has no nominative in the language, the dictionary's
    // `nom/acc` bundles notwithstanding); the prepositional `не́мъ`/`не́й`
    // hold the locative, the witnessed `є҆́й`/`є҆́ю` the dative and
    // instrumental (Gen 16:8, 9:1), in the print's typography.
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Masculine, SYN),
        ["ѻ҆́нъ", "є҆гѡ̀", "є҆мꙋ̀", "є҆го̀", "и҆́мъ", "не́мъ"]
    );
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Feminine, SYN),
        ["ѻ҆на̀", "є҆ѧ̀", "є҆́й", "ю҆̀", "є҆́ю", "не́й"]
    );
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Neuter, SYN),
        ["ѻ҆но̀", "є҆гѡ̀", "є҆мꙋ̀", "є҆̀", "и҆́мъ", "не́мъ"]
    );
    // The anaphor's short accusative «и҆̀» is the clitic cell's (v1.2).
    assert_eq!(
        ChurchSlavonic::clitic(&Person::Third, &Number::Singular, &Gender::Masculine, &Case::Accusative, &SYN),
        Some("и҆̀")
    );
}

#[test]
fn third_person_plural_and_dual() {
    assert_eq!(
        row(Person::Third, Number::Plural, Gender::Masculine, OCS)[0],
        "и"
    );
    assert_eq!(
        row(Person::Third, Number::Plural, Gender::Feminine, OCS)[0],
        "ѩ"
    );
    assert_eq!(
        row(Person::Third, Number::Plural, Gender::Neuter, OCS)[0],
        "ꙗ"
    );
    assert_eq!(
        row(Person::Third, Number::Dual, Gender::Masculine, OCS)[0],
        "ꙗ"
    );
    assert_eq!(
        row(Person::Third, Number::Plural, Gender::Masculine, SYN),
        // the print's varia on the dative and accusative monosyllables
        // (Alypy §47; Gen 9:1 «речѐ и҆̀мъ», Gen 6:13 «погꙋблю̀ и҆̀хъ»), the
        // prepositional locative (Gen 24:3 «въ ни́хъ»)
        ["ѻ҆нѝ", "и҆́хъ", "и҆̀мъ", "и҆̀хъ", "ни́ми", "ни́хъ"]
    );
    assert_eq!(
        row(Person::Third, Number::Dual, Gender::Feminine, SYN)[0],
        "ѻ҆́нѣ"
    );
}

#[test]
fn the_vocative_answers_with_the_nominative() {
    for r in [OCS, SYN] {
        for p in [Person::First, Person::Second, Person::Third] {
            for n in [Number::Singular, Number::Dual, Number::Plural] {
                for g in GENDERS {
                    assert_eq!(
                        ChurchSlavonic::pronoun(&p, &n, &g, &Case::Vocative, &r),
                        ChurchSlavonic::pronoun(&p, &n, &g, &Case::Nominative, &r)
                    );
                }
            }
        }
    }
}
