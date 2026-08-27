//! The table schema — the one contract shared with the `church-slavonic`
//! runtime (which re-derives the same indices in its `lib.rs`).
//!
//! Every generated map is keyed `"<recension-tag>:<key>"` (`ocs:градъ`,
//! `syn:рабъ_2`) and holds one fixed-arity array of cell strings per row. An
//! empty string means "not attested — fall back to the rule". Cell order:
//!
//! - noun (21): `number * 7 + case`, numbers `Singular, Dual, Plural`, cases
//!   `Nominative, Genitive, Dative, Accusative, Instrumental, Locative, Vocative`;
//! - adjective (126): `((degree * 3 + gender) * 3 + number) * 7 + case`,
//!   degrees `Positive, Comparative` (the superlative is always the rule's
//!   `пре-` prefix), genders `Masculine, Feminine, Neuter`;
//! - verb (38): four 9-cell finite blocks `Present, Imperfect, Aorist,
//!   Imperative` indexed `number * 3 + person`, then the present active
//!   participle citation (36) and the past active participle citation (37);
//!   the infinitive is the lemma itself;
//! - pronoun (90): the closed personal matrix — first person `number * 6 +
//!   case` (0..18), second person likewise at 18.., third person
//!   `36 + (gender * 3 + number) * 6 + case` — over the six non-vocative cases
//!   (the vocative answers with the nominative). The pronoun map is keyed
//!   `<tag>:personal`: the facade's `pronoun` call takes no lemma.

use church_slavonic_core::ChurchSlavonicCore;
use church_slavonic_core::grammar::*;
use church_slavonic_core::orthography::{comparison_key, realise};

pub const CASES: [Case; 7] = [
    Case::Nominative,
    Case::Genitive,
    Case::Dative,
    Case::Accusative,
    Case::Instrumental,
    Case::Locative,
    Case::Vocative,
];
pub const NUMBERS: [Number; 3] = [Number::Singular, Number::Dual, Number::Plural];
pub const GENDERS: [Gender; 3] = [Gender::Masculine, Gender::Feminine, Gender::Neuter];
pub const PERSONS: [Person; 3] = [Person::First, Person::Second, Person::Third];
pub const DEGREES: [Degree; 2] = [Degree::Positive, Degree::Comparative];
/// The finite blocks of the verb row, in cell order.
pub const VERB_BLOCKS: [(Tense, Form); 4] = [
    (Tense::Present, Form::Finite),
    (Tense::Imperfect, Form::Finite),
    (Tense::Aorist, Form::Finite),
    (Tense::Present, Form::Imperative),
];
/// The pronoun row's lemma-less key.
pub const PRONOUN_KEY: &str = "personal";

/// The two recension tags that prefix every key.
pub fn tag(recension: &Recension) -> &'static str {
    match recension {
        Recension::OldChurchSlavonic => "ocs",
        Recension::Synodal => "syn",
    }
}

pub fn recension_of_tag(tag: &str) -> Option<Recension> {
    match tag {
        "ocs" => Some(Recension::OldChurchSlavonic),
        "syn" => Some(Recension::Synodal),
        _ => None,
    }
}

pub fn noun_cell(case: &Case, number: &Number) -> usize {
    *number as usize * 7 + *case as usize
}

pub fn adj_cell(case: &Case, number: &Number, gender: &Gender, degree: &Degree) -> Option<usize> {
    let degree = match degree {
        Degree::Positive => 0,
        Degree::Comparative => 1,
        Degree::Superlative => return None,
    };
    Some(((degree * 3 + *gender as usize) * 3 + *number as usize) * 7 + *case as usize)
}

pub fn verb_cell(person: &Person, number: &Number, tense: &Tense, form: &Form) -> Option<usize> {
    let block = match (tense, form) {
        (Tense::Present, Form::Finite) => 0,
        (Tense::Imperfect, Form::Finite) => 1,
        (Tense::Aorist, Form::Finite) => 2,
        (_, Form::Imperative) => 3,
        (Tense::Present, Form::Participle) => return Some(36),
        (_, Form::Participle) => return Some(37),
        (_, Form::Infinitive) => return None,
    };
    Some(block * 9 + *number as usize * 3 + *person as usize)
}

pub fn pronoun_cell(person: &Person, number: &Number, gender: &Gender, case: &Case) -> usize {
    let case = if *case == Case::Vocative {
        0
    } else {
        *case as usize
    };
    match person {
        Person::First => *number as usize * 6 + case,
        Person::Second => 18 + *number as usize * 6 + case,
        Person::Third => 36 + (*gender as usize * 3 + *number as usize) * 6 + case,
    }
}

/// The four parts of speech the tables cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pos {
    Noun,
    Adj,
    Verb,
    Pronoun,
}

impl Pos {
    pub const ALL: [Pos; 4] = [Pos::Noun, Pos::Adj, Pos::Verb, Pos::Pronoun];

    pub fn arity(self) -> usize {
        match self {
            Pos::Noun => 21,
            Pos::Adj => 126,
            Pos::Verb => 38,
            Pos::Pronoun => 90,
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Pos::Noun => "noun_phf.rs",
            Pos::Adj => "adj_phf.rs",
            Pos::Verb => "verb_phf.rs",
            Pos::Pronoun => "pronoun_phf.rs",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Pos::Noun => "noun",
            Pos::Adj => "adj",
            Pos::Verb => "verb",
            Pos::Pronoun => "pronoun",
        }
    }

    /// The rule engine's answer for every cell of `lemma`'s row — what the
    /// extractor subtracts from the attestations and what `check-registry`
    /// audits the committed rows against. Exactly the facade's rule path: the
    /// lemma is realised into the recension's spelling before the rule and
    /// the answer realised after it.
    pub fn predict(self, lemma: &str, recension: &Recension) -> Vec<String> {
        let mut out = self.predict_raw(&realise(lemma, recension), recension);
        for cell in &mut out {
            *cell = realise(cell, recension);
        }
        out
    }

    fn predict_raw(self, lemma: &str, recension: &Recension) -> Vec<String> {
        let mut out = vec![String::new(); self.arity()];
        match self {
            Pos::Noun => {
                for number in &NUMBERS {
                    for case in &CASES {
                        out[noun_cell(case, number)] =
                            ChurchSlavonicCore::noun(lemma, case, number, recension);
                    }
                }
            }
            Pos::Adj => {
                for degree in &DEGREES {
                    for gender in &GENDERS {
                        for number in &NUMBERS {
                            for case in &CASES {
                                if let Some(i) = adj_cell(case, number, gender, degree) {
                                    out[i] = ChurchSlavonicCore::adj(
                                        lemma, case, number, gender, degree, recension,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Pos::Verb => {
                for (tense, form) in &VERB_BLOCKS {
                    for number in &NUMBERS {
                        for person in &PERSONS {
                            if let Some(i) = verb_cell(person, number, tense, form) {
                                out[i] = ChurchSlavonicCore::verb(
                                    lemma, person, number, tense, form, recension,
                                );
                            }
                        }
                    }
                }
                let participle = |tense: &Tense| {
                    ChurchSlavonicCore::verb(
                        lemma,
                        &Person::Third,
                        &Number::Singular,
                        tense,
                        &Form::Participle,
                        recension,
                    )
                };
                out[36] = participle(&Tense::Present);
                out[37] = participle(&Tense::Aorist);
            }
            Pos::Pronoun => {
                for person in &PERSONS {
                    for gender in &GENDERS {
                        for number in &NUMBERS {
                            for case in &CASES[..6] {
                                out[pronoun_cell(person, number, gender, case)] =
                                    ChurchSlavonicCore::pronoun(
                                        person, number, gender, case, recension,
                                    )
                                    .to_string();
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

/// Does an attested surface count as "what the rule already says"? The
/// comparison follows the recension's sources' accent policy: the Kaikki dump
/// (OCS) is unaccented, so two spellings are one form when their
/// [`comparison_key`]s agree; the Alypy grammar and Polyakov's dictionary
/// (Synodal) print accents (and the grammar breathings and titla), and the
/// surface is compared exactly — the rules never produce accents, so an
/// accented Synodal cell is served by the table.
pub fn rule_matches(recension: &Recension, attested: &str, predicted: &str) -> bool {
    match recension {
        Recension::OldChurchSlavonic => comparison_key(attested) == comparison_key(predicted),
        Recension::Synodal => attested == predicted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_indices_tile_each_row_exactly_once() {
        let mut seen = vec![false; Pos::Noun.arity()];
        for n in &NUMBERS {
            for c in &CASES {
                let i = noun_cell(c, n);
                assert!(!seen[i]);
                seen[i] = true;
            }
        }
        assert!(seen.iter().all(|s| *s));

        let mut seen = vec![false; Pos::Adj.arity()];
        for d in &DEGREES {
            for g in &GENDERS {
                for n in &NUMBERS {
                    for c in &CASES {
                        let i = adj_cell(c, n, g, d).expect("indexed");
                        assert!(!seen[i]);
                        seen[i] = true;
                    }
                }
            }
        }
        assert!(seen.iter().all(|s| *s));
        assert_eq!(
            adj_cell(
                &Case::Nominative,
                &Number::Singular,
                &Gender::Masculine,
                &Degree::Superlative
            ),
            None
        );

        let mut seen = vec![false; Pos::Verb.arity()];
        for (t, f) in &VERB_BLOCKS {
            for n in &NUMBERS {
                for p in &PERSONS {
                    let i = verb_cell(p, n, t, f).expect("indexed");
                    assert!(!seen[i]);
                    seen[i] = true;
                }
            }
        }
        for t in [Tense::Present, Tense::Aorist] {
            let i = verb_cell(&Person::First, &Number::Plural, &t, &Form::Participle).expect("p");
            assert!(!seen[i]);
            seen[i] = true;
        }
        assert!(seen.iter().all(|s| *s));
        assert_eq!(
            verb_cell(
                &Person::First,
                &Number::Singular,
                &Tense::Present,
                &Form::Infinitive
            ),
            None
        );

        let mut seen = vec![false; Pos::Pronoun.arity()];
        for p in &PERSONS {
            for g in &GENDERS {
                for n in &NUMBERS {
                    for c in &CASES[..6] {
                        let i = pronoun_cell(p, n, g, c);
                        if *p != Person::Third && *g != Gender::Masculine {
                            assert_eq!(i, pronoun_cell(p, n, &Gender::Masculine, c));
                            continue;
                        }
                        assert!(!seen[i]);
                        seen[i] = true;
                    }
                }
            }
        }
        assert!(seen.iter().all(|s| *s));
        assert_eq!(
            pronoun_cell(
                &Person::First,
                &Number::Singular,
                &Gender::Neuter,
                &Case::Vocative
            ),
            0
        );
    }

    #[test]
    fn rule_matches_follows_the_source_accent_policy() {
        let ocs = Recension::OldChurchSlavonic;
        let syn = Recension::Synodal;
        assert!(rule_matches(&ocs, "рабоу", "рабоу"));
        assert!(rule_matches(&ocs, "ꙁима", "зима"));
        assert!(!rule_matches(&ocs, "рабови", "рабоу"));
        assert!(rule_matches(&syn, "рабꙋ", "рабꙋ"));
        assert!(!rule_matches(&syn, "рабꙋ̀", "рабꙋ"));
    }
}
