//! The personal pronoun: a closed matrix, looked up rather than derived.
//!
//! The recension conditions of the divergence registry are visible cell by
//! cell: the Synodal genitive-shaped accusative (`мене`, `тебе`, `єго`,
//! `ихъ`) against the OCS nominal one (`мѧ`, `тѧ`, `и`, `ѩ`) —
//! pron:genitive-accusative; the levelled Synodal dual nominative (`мы`,
//! `вы` for OCS `вѣ`, `ва`) — pron:dual-nominative-leveling; the `-мь`/`-мъ`
//! instrumental and locative — pron:instr-loc-sg-jer; the Synodal
//! post-prepositional `н-` locative (`немъ`) and the gender-levelled dual
//! accusative `ѧ` — pron:dual-accusative-gender-leveling. Third-person
//! nominatives are the `онъ` series in both recensions (OCS uses the
//! demonstrative there; pron:third-person-nominative-on). The vocative
//! answers with the nominative; the clitics (`ми`, `мѧ`, `ны`) are not
//! primaries here. The Synodal cells are spelled in the print's typography,
//! accents and breathings included (Alypy §47; the corpus's other
//! spellings — `менѐ` for the genitive, the enclitics, the `ѻ҆нѝ`-less
//! anaphoric `и҆̀` — are table variants).

use crate::ChurchSlavonicCore;
use crate::grammar::*;

impl ChurchSlavonicCore {
    pub fn pronoun(
        person: &Person,
        number: &Number,
        gender: &Gender,
        case: &Case,
        recension: &Recension,
    ) -> &'static str {
        let case = if *case == Case::Vocative {
            &Case::Nominative
        } else {
            case
        };
        let synodal = *recension == Recension::Synodal;
        let row: &[&str; 6] = match (person, number, gender, synodal) {
            (Person::First, Number::Singular, _, false) => {
                &["азъ", "мене", "мьнѣ", "мѧ", "мъноѭ", "мьнѣ"]
            }
            (Person::First, Number::Singular, _, true) => {
                &["а҆́зъ", "менє̀", "мнѣ̀", "менѐ", "мно́ю", "мнѣ̀"]
            }
            (Person::First, Number::Dual, _, false) => &["вѣ", "наю", "нама", "на", "нама", "наю"],
            (Person::First, Number::Dual, _, true) => &["мы̀", "на́ю", "на́ма", "ны̀", "на́ма", "на́ю"],
            (Person::First, Number::Plural, _, false) => {
                &["мꙑ", "насъ", "намъ", "нꙑ", "нами", "насъ"]
            }
            (Person::First, Number::Plural, _, true) => {
                &["мы̀", "на́съ", "на́мъ", "на́съ", "на́ми", "на́съ"]
            }
            (Person::Second, Number::Singular, _, false) => {
                &["тꙑ", "тебе", "тебѣ", "тѧ", "тобоѭ", "тебѣ"]
            }
            (Person::Second, Number::Singular, _, true) => {
                &["ты̀", "тебє̀", "тебѣ̀", "тебѐ", "тобо́ю", "тебѣ̀"]
            }
            (Person::Second, Number::Dual, _, false) => &["ва", "ваю", "вама", "ва", "вама", "ваю"],
            (Person::Second, Number::Dual, _, true) => &["вы̀", "ва́ю", "ва́ма", "вы̀", "ва́ма", "ва́ю"],
            (Person::Second, Number::Plural, _, false) => {
                &["вꙑ", "васъ", "вамъ", "вꙑ", "вами", "васъ"]
            }
            (Person::Second, Number::Plural, _, true) => {
                &["вы̀", "ва́съ", "ва́мъ", "ва́съ", "ва́ми", "ва́съ"]
            }
            (Person::Third, Number::Singular, Gender::Masculine, false) => {
                &["онъ", "ѥго", "ѥмоу", "и", "имь", "ѥмь"]
            }
            (Person::Third, Number::Singular, Gender::Masculine, true) => {
                &["ѻ҆́нъ", "є҆гѡ̀", "є҆мꙋ̀", "є҆го̀", "и҆́мъ", "не́мъ"]
            }
            (Person::Third, Number::Singular, Gender::Feminine, false) => {
                &["она", "ѥѩ", "ѥи", "ѭ", "ѥѭ", "ѥи"]
            }
            (Person::Third, Number::Singular, Gender::Feminine, true) => {
                &["ѻ҆на̀", "є҆ѧ̀", "є҆́й", "ю҆̀", "є҆́ю", "не́й"]
            }
            (Person::Third, Number::Singular, Gender::Neuter, false) => {
                &["оно", "ѥго", "ѥмоу", "ѥ", "имь", "ѥмь"]
            }
            (Person::Third, Number::Singular, Gender::Neuter, true) => {
                &["ѻ҆но̀", "є҆гѡ̀", "є҆мꙋ̀", "є҆̀", "и҆́мъ", "не́мъ"]
            }
            (Person::Third, Number::Dual, Gender::Masculine, false) => {
                &["она", "ѥю", "има", "ꙗ", "има", "ѥю"]
            }
            (Person::Third, Number::Dual, _, false) => &["онѣ", "ѥю", "има", "и", "има", "ѥю"],
            (Person::Third, Number::Dual, Gender::Masculine, true) => {
                &["ѻ҆́на", "є҆́ю", "и҆́ма", "ѧ҆̀", "и҆́ма", "не́ю"]
            }
            (Person::Third, Number::Dual, _, true) => &["ѻ҆́нѣ", "є҆́ю", "и҆́ма", "ѧ҆̀", "и҆́ма", "не́ю"],
            (Person::Third, Number::Plural, Gender::Masculine, false) => {
                &["они", "ихъ", "имъ", "ѩ", "ими", "ихъ"]
            }
            (Person::Third, Number::Plural, Gender::Feminine, false) => {
                &["онꙑ", "ихъ", "имъ", "ѩ", "ими", "ихъ"]
            }
            (Person::Third, Number::Plural, Gender::Neuter, false) => {
                &["она", "ихъ", "имъ", "ꙗ", "ими", "ихъ"]
            }
            (Person::Third, Number::Plural, Gender::Feminine, true) => {
                &["ѻ҆нѣ̀", "и҆́хъ", "и҆́мъ", "и҆́хъ", "и҆́ми", "ни́хъ"]
            }
            (Person::Third, Number::Plural, _, true) => {
                &["ѻ҆нѝ", "и҆́хъ", "и҆́мъ", "и҆́хъ", "и҆́ми", "ни́хъ"]
            }
        };
        row[*case as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::Synodal;

    fn p(person: Person, number: Number, gender: Gender, case: Case, r: Recension) -> &'static str {
        ChurchSlavonicCore::pronoun(&person, &number, &gender, &case, &r)
    }

    #[test]
    fn the_registry_conditions_are_visible_cell_by_cell() {
        use Case::*;
        use Gender::*;
        use Number::*;
        use Person::*;
        // pron:genitive-accusative
        assert_eq!(p(First, Singular, Masculine, Accusative, OCS), "мѧ");
        assert_eq!(p(First, Singular, Masculine, Accusative, SYN), "менѐ");
        assert_eq!(p(Third, Singular, Masculine, Accusative, OCS), "и");
        assert_eq!(p(Third, Singular, Masculine, Accusative, SYN), "є҆го̀");
        // pron:dual-nominative-leveling
        assert_eq!(p(First, Dual, Masculine, Nominative, OCS), "вѣ");
        assert_eq!(p(First, Dual, Masculine, Nominative, SYN), "мы̀");
        // pron:instr-loc-sg-jer and the post-prepositional locative
        assert_eq!(p(Third, Singular, Neuter, Instrumental, OCS), "имь");
        assert_eq!(p(Third, Singular, Neuter, Instrumental, SYN), "и҆́мъ");
        assert_eq!(p(Third, Singular, Feminine, Locative, SYN), "не́й");
        // pron:dual-accusative-gender-leveling
        assert_eq!(p(Third, Dual, Feminine, Accusative, OCS), "и");
        assert_eq!(p(Third, Dual, Feminine, Accusative, SYN), "ѧ҆̀");
        // the vocative answers with the nominative
        assert_eq!(p(Second, Plural, Feminine, Vocative, SYN), "вы̀");
    }

    #[test]
    fn the_synodal_matrix_is_spelled_in_the_canonical_typography() {
        use crate::orthography::realise;
        for person in [Person::First, Person::Second, Person::Third] {
            for number in [Number::Singular, Number::Dual, Number::Plural] {
                for gender in [Gender::Masculine, Gender::Feminine, Gender::Neuter] {
                    for case in [
                        Case::Nominative,
                        Case::Genitive,
                        Case::Dative,
                        Case::Accusative,
                        Case::Instrumental,
                        Case::Locative,
                    ] {
                        let cell = p(person, number, gender, case, SYN);
                        assert_eq!(
                            realise(cell, &SYN),
                            cell,
                            "{person:?} {number:?} {gender:?} {case:?}"
                        );
                    }
                }
            }
        }
    }
}
