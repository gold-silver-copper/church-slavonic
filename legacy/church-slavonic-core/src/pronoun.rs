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
            // The print marks the plural dative and accusative monosyllables
            // with the varia against the genitive's oxia (Alypy §47; the
            // Bible's «и҆̀мъ» 1,711, «и҆̀хъ» 1,220) — pron:plural-varia; the
            // neuter accusative is the short «ѧ҆̀» (§47 prints no other).
            (Person::Third, Number::Plural, Gender::Feminine, true) => {
                &["ѻ҆нѣ̀", "и҆́хъ", "и҆̀мъ", "и҆̀хъ", "и҆́ми", "ни́хъ"]
            }
            (Person::Third, Number::Plural, Gender::Neuter, true) => {
                &["ѻ҆нѝ", "и҆́хъ", "и҆̀мъ", "ѧ҆̀", "и҆́ми", "ни́хъ"]
            }
            (Person::Third, Number::Plural, _, true) => {
                &["ѻ҆нѝ", "и҆́хъ", "и҆̀мъ", "и҆̀хъ", "и҆́ми", "ни́хъ"]
            }
        };
        row[*case as usize]
    }

    /// The reflexive pronoun (себѐ): no person, number or gender; the
    /// nominative (and the vocative) is blank. The Synodal genitive spells
    /// є like the second person's (себє̀; the Bible: 111 against the
    /// accusative's себѐ 237).
    pub fn reflexive(case: &Case, recension: &Recension) -> &'static str {
        let row: &[&str; 6] = match recension {
            Recension::OldChurchSlavonic => &["", "себе", "себѣ", "сѧ", "собоѭ", "себѣ"],
            Recension::Synodal => &["", "себє̀", "себѣ̀", "себѐ", "собо́ю", "себѣ̀"],
        };
        match case {
            Case::Vocative => row[0],
            other => row[*other as usize],
        }
    }

    /// The enclitic form of a personal pronoun cell, or `None` where the
    /// language has none (Alypy §47 prints them as the alternatives:
    /// «мнѣ̀, мѝ»; «є҆го̀, и҆̀»; «ѧ҆̀, и҆̀хъ»). The first and second persons
    /// have a dative and an accusative clitic in the singular and an
    /// accusative in the dual and plural; the third person an accusative
    /// only. In Old Church Slavonic the accusative clitic IS the primary
    /// accusative (мѧ, и).
    pub fn clitic(
        person: &Person,
        number: &Number,
        gender: &Gender,
        case: &Case,
        recension: &Recension,
    ) -> Option<&'static str> {
        use Case::*;
        use Gender::*;
        use Number::*;
        use Person::*;
        let synodal = *recension == Recension::Synodal;
        Some(match (person, number, gender, case, synodal) {
            (First, Singular, _, Dative, false) => "ми",
            (First, Singular, _, Dative, true) => "мѝ",
            (First, Singular, _, Accusative, false) => "мѧ",
            (First, Singular, _, Accusative, true) => "мѧ̀",
            (First, Dual, _, Accusative, false) => "на",
            (First, Dual | Plural, _, Accusative, true) => "ны̀",
            (First, Plural, _, Accusative, false) => "нꙑ",
            (Second, Singular, _, Dative, false) => "ти",
            (Second, Singular, _, Dative, true) => "тѝ",
            (Second, Singular, _, Accusative, false) => "тѧ",
            (Second, Singular, _, Accusative, true) => "тѧ̀",
            (Second, Dual, _, Accusative, false) => "ва",
            (Second, Dual | Plural, _, Accusative, true) => "вы̀",
            (Second, Plural, _, Accusative, false) => "вꙑ",
            (Third, Singular, Masculine, Accusative, false) => "и",
            (Third, Singular, Masculine, Accusative, true) => "и҆̀",
            (Third, Singular, Feminine, Accusative, false) => "ѭ",
            (Third, Singular, Feminine, Accusative, true) => "ю҆̀",
            (Third, Singular, Neuter, Accusative, false) => "ѥ",
            (Third, Singular, Neuter, Accusative, true) => "є҆̀",
            (Third, Dual, Masculine, Accusative, false) => "ꙗ",
            (Third, Dual, _, Accusative, false) => "и",
            (Third, Plural, Neuter, Accusative, false) => "ꙗ",
            (Third, Plural, _, Accusative, false) => "ѩ",
            (Third, Dual | Plural, _, Accusative, true) => "ѧ҆̀",
            _ => return None,
        })
    }

    /// The reflexive's clitic (сѝ, сѧ̀), or `None`.
    pub fn reflexive_clitic(case: &Case, recension: &Recension) -> Option<&'static str> {
        let synodal = *recension == Recension::Synodal;
        Some(match (case, synodal) {
            (Case::Dative, false) => "си",
            (Case::Dative, true) => "сѝ",
            (Case::Accusative, false) => "сѧ",
            (Case::Accusative, true) => "сѧ̀",
            _ => return None,
        })
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
        // pron:plural-varia
        assert_eq!(p(Third, Plural, Masculine, Dative, SYN), "и҆̀мъ");
        assert_eq!(p(Third, Plural, Masculine, Accusative, SYN), "и҆̀хъ");
        assert_eq!(p(Third, Plural, Masculine, Genitive, SYN), "и҆́хъ");
        assert_eq!(p(Third, Plural, Neuter, Accusative, SYN), "ѧ҆̀");
        // the vocative answers with the nominative
        assert_eq!(p(Second, Plural, Feminine, Vocative, SYN), "вы̀");
    }

    #[test]
    fn the_reflexive_and_the_clitics() {
        use Case::*;
        use Gender::*;
        use Number::*;
        use Person::*;
        assert_eq!(ChurchSlavonicCore::reflexive(&Genitive, &SYN), "себє̀");
        assert_eq!(ChurchSlavonicCore::reflexive(&Accusative, &SYN), "себѐ");
        assert_eq!(ChurchSlavonicCore::reflexive(&Nominative, &SYN), "");
        assert_eq!(ChurchSlavonicCore::reflexive(&Instrumental, &OCS), "собоѭ");
        assert_eq!(ChurchSlavonicCore::clitic(&First, &Singular, &Neuter, &Dative, &SYN), Some("мѝ"));
        assert_eq!(ChurchSlavonicCore::clitic(&Second, &Singular, &Neuter, &Accusative, &SYN), Some("тѧ̀"));
        assert_eq!(ChurchSlavonicCore::clitic(&First, &Plural, &Neuter, &Accusative, &SYN), Some("ны̀"));
        assert_eq!(ChurchSlavonicCore::clitic(&First, &Plural, &Neuter, &Dative, &SYN), None);
        assert_eq!(ChurchSlavonicCore::clitic(&Third, &Singular, &Masculine, &Accusative, &SYN), Some("и҆̀"));
        assert_eq!(ChurchSlavonicCore::clitic(&Third, &Plural, &Feminine, &Accusative, &SYN), Some("ѧ҆̀"));
        assert_eq!(ChurchSlavonicCore::clitic(&Third, &Plural, &Feminine, &Genitive, &SYN), None);
        assert_eq!(ChurchSlavonicCore::clitic(&First, &Dual, &Neuter, &Accusative, &OCS), Some("на"));
        assert_eq!(ChurchSlavonicCore::reflexive_clitic(&Accusative, &SYN), Some("сѧ̀"));
        assert_eq!(ChurchSlavonicCore::reflexive_clitic(&Genitive, &SYN), None);
        // canonical typography throughout
        for c in [Genitive, Dative, Accusative, Instrumental, Locative] {
            let f = ChurchSlavonicCore::reflexive(&c, &SYN);
            assert_eq!(crate::orthography::realise(f, &SYN), f);
        }
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
