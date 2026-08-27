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
//! primaries here.

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
                &["азъ", "мене", "мнѣ", "мене", "мною", "мнѣ"]
            }
            (Person::First, Number::Dual, _, false) => &["вѣ", "наю", "нама", "на", "нама", "наю"],
            (Person::First, Number::Dual, _, true) => &["мы", "наю", "нама", "ны", "нама", "наю"],
            (Person::First, Number::Plural, _, false) => {
                &["мꙑ", "насъ", "намъ", "нꙑ", "нами", "насъ"]
            }
            (Person::First, Number::Plural, _, true) => {
                &["мы", "насъ", "намъ", "насъ", "нами", "насъ"]
            }
            (Person::Second, Number::Singular, _, false) => {
                &["тꙑ", "тебе", "тебѣ", "тѧ", "тобоѭ", "тебѣ"]
            }
            (Person::Second, Number::Singular, _, true) => {
                &["ты", "тебе", "тебѣ", "тебе", "тобою", "тебѣ"]
            }
            (Person::Second, Number::Dual, _, false) => &["ва", "ваю", "вама", "ва", "вама", "ваю"],
            (Person::Second, Number::Dual, _, true) => &["вы", "ваю", "вама", "вы", "вама", "ваю"],
            (Person::Second, Number::Plural, _, false) => {
                &["вꙑ", "васъ", "вамъ", "вꙑ", "вами", "васъ"]
            }
            (Person::Second, Number::Plural, _, true) => {
                &["вы", "васъ", "вамъ", "васъ", "вами", "васъ"]
            }
            (Person::Third, Number::Singular, Gender::Masculine, false) => {
                &["онъ", "ѥго", "ѥмоу", "и", "имь", "ѥмь"]
            }
            (Person::Third, Number::Singular, Gender::Masculine, true) => {
                &["онъ", "єгѡ", "ємꙋ", "єго", "имъ", "немъ"]
            }
            (Person::Third, Number::Singular, Gender::Feminine, false) => {
                &["она", "ѥѩ", "ѥи", "ѭ", "ѥѭ", "ѥи"]
            }
            (Person::Third, Number::Singular, Gender::Feminine, true) => {
                &["она", "єѧ", "єй", "ю", "єю", "ней"]
            }
            (Person::Third, Number::Singular, Gender::Neuter, false) => {
                &["оно", "ѥго", "ѥмоу", "ѥ", "имь", "ѥмь"]
            }
            (Person::Third, Number::Singular, Gender::Neuter, true) => {
                &["оно", "єгѡ", "ємꙋ", "є", "имъ", "немъ"]
            }
            (Person::Third, Number::Dual, Gender::Masculine, false) => {
                &["она", "ѥю", "има", "ꙗ", "има", "ѥю"]
            }
            (Person::Third, Number::Dual, _, false) => &["онѣ", "ѥю", "има", "и", "има", "ѥю"],
            (Person::Third, Number::Dual, Gender::Masculine, true) => {
                &["она", "єю", "има", "ѧ", "има", "нею"]
            }
            (Person::Third, Number::Dual, _, true) => &["онѣ", "єю", "има", "ѧ", "има", "нею"],
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
                &["онѣ", "ихъ", "имъ", "ихъ", "ими", "нихъ"]
            }
            (Person::Third, Number::Plural, _, true) => {
                &["они", "ихъ", "имъ", "ихъ", "ими", "нихъ"]
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
        assert_eq!(p(First, Singular, Masculine, Accusative, SYN), "мене");
        assert_eq!(p(Third, Singular, Masculine, Accusative, OCS), "и");
        assert_eq!(p(Third, Singular, Masculine, Accusative, SYN), "єго");
        // pron:dual-nominative-leveling
        assert_eq!(p(First, Dual, Masculine, Nominative, OCS), "вѣ");
        assert_eq!(p(First, Dual, Masculine, Nominative, SYN), "мы");
        // pron:instr-loc-sg-jer and the post-prepositional locative
        assert_eq!(p(Third, Singular, Neuter, Instrumental, OCS), "имь");
        assert_eq!(p(Third, Singular, Neuter, Instrumental, SYN), "имъ");
        assert_eq!(p(Third, Singular, Feminine, Locative, SYN), "ней");
        // pron:dual-accusative-gender-leveling
        assert_eq!(p(Third, Dual, Feminine, Accusative, OCS), "и");
        assert_eq!(p(Third, Dual, Feminine, Accusative, SYN), "ѧ");
        // the vocative answers with the nominative
        assert_eq!(p(Second, Plural, Feminine, Vocative, SYN), "вы");
    }
}
