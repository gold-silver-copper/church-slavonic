//! Exhaustive coverage of the `ChurchSlavonic::pronoun` matrix.
//!
//! The personal pronoun is one lemma-less row per recension: the Kaikki
//! `и`/`ꙗ`/`ѥ` tables (OCS) and, for Synodal, Polyakov's corpus primaries
//! (frequency-ordered, full forms before enclitics) with §47 of the Alypy
//! grammar filling the cells the corpus lacks (the duals) — the grammar's
//! other spellings are the row's variant gap, since the lemma-less call
//! reaches one row. Gender is consulted only in the third person;
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
            ["а́зъ", "менє́", "мнѣ́", "мене́", "мно́ю", "мнѣ́"]
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
            ["ны́", "на́съ", "на́мъ", "на́съ", "на́ми", "на́съ"]
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
            ["вы́", "ва́съ", "ва́мъ", "ва́съ", "ва́ми", "ва́съ"]
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
    // Synodal: the corpus's anaphoric `и́`/`я́`/`є́` and its most frequent
    // oblique spellings (the prepositional `не́мъ`/`не́й` outnumber `є́мъ`).
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Masculine, SYN),
        ["и́", "єгѡ́", "єму́", "єго́", "и́мъ", "не́мъ"]
    );
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Feminine, SYN),
        ["я́", "єя́", "не́й", "ю́", "не́ю", "не́й"]
    );
    assert_eq!(
        row(Person::Third, Number::Singular, Gender::Neuter, SYN),
        ["є́", "єгѡ́", "єму́", "є́", "и́мъ", "не́мъ"]
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
        ["ѻ҆нѝ", "и́хъ", "и́мъ", "и́хъ", "ни́ми", "и́хъ"]
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
