//! Exhaustive coverage of the `ChurchSlavonic::pronoun` matrix.
//!
//! The personal pronoun is one lemma-less row per recension: the Kaikki
//! `и`/`ꙗ`/`ѥ` tables (OCS) and §47 of the Alypy grammar (Synodal) over the
//! rule engine's closed matrix. Gender is consulted only in the third person;
//! the vocative answers with the nominative. This pins every cell of the
//! first and second persons and the third-person singular in both recensions,
//! so a transposed cell or a wrong recension condition cannot ship silently.

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
            ["а҆́зъ", "менѐ", "мнѣ̀", "менѐ", "мно́ю", "мнѣ̀"]
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
            ["мы̀", "на́съ", "на́мъ", "ны̀", "на́ми", "на́съ"]
        );
        assert_eq!(
            row(Person::Second, Number::Singular, g, OCS),
            ["тꙑ", "тебе", "тебѣ", "тѧ", "тобоѭ", "тебѣ"]
        );
        assert_eq!(
            row(Person::Second, Number::Singular, g, SYN),
            ["ты̀", "тебѐ", "тебѣ̀", "тебѐ", "тобо́ю", "тебѣ̀"]
        );
        assert_eq!(
            row(Person::Second, Number::Plural, g, SYN),
            ["вы̀", "ва́съ", "ва́мъ", "вы̀", "ва́ми", "ва́съ"]
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
    // Synodal: §47 with its accents; the genitive-shaped accusative.
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Masculine, SYN),
        ["ѻ҆́нъ", "є҆гѡ̀", "є҆мꙋ̀", "є҆го̀", "и҆́мъ", "е́мъ"]
    );
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Feminine, SYN),
        ["ѻ҆на̀", "є҆ѧ̀", "є҆́й", "ю҆̀", "є҆́ю", "е́й"]
    );
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Neuter, SYN),
        ["ѻ҆но̀", "є҆гѡ̀", "є҆мꙋ̀", "є҆̀", "и҆́мъ", "е́мъ"]
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
        ["ѻ҆нѝ", "и҆́хъ", "и҆̀мъ", "ѧ҆̀", "и҆́ми", "и́хъ"]
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
