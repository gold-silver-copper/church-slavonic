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
//! - verb (542): four 9-cell finite blocks `Present, Imperfect, Aorist,
//!   Imperative` indexed `number * 3 + person`, the present active
//!   participle citation (36) and the past active participle citation (37)
//!   — these 38 cells are FROZEN — then the declined participle block at
//!   `38 + (((series * 2 + tense) * 3 + gender) * 3 + number) * 7 + case`,
//!   series `Short-Active, Long-Active, Short-Passive, Long-Passive`,
//!   tenses `Present, Past`, genders/numbers/cases in the adjective order
//!   (504 cells), and four PARTICIPLE-STEM cells (542 present active, 543
//!   past active, 544 present passive, 545 past passive) — a stem the
//!   extractor derived from the attested declension, which the runtime
//!   expands through the same declension rule, so a verb with a regular
//!   declension of an irregular stem costs four cells, not five hundred;
//!   the infinitive is the lemma itself;
//! - pronoun (90): the closed personal matrix — first person `number * 6 +
//!   case` (0..18), second person likewise at 18.., third person
//!   `36 + (gender * 3 + number) * 6 + case` — over the six non-vocative cases
//!   (the vocative answers with the nominative). The pronoun map is keyed
//!   `<tag>:personal`: the facade's `pronoun` call takes no lemma.

use church_slavonic_core::ChurchSlavonicCore;
use church_slavonic_core::grammar::*;
use church_slavonic_core::orthography::{comparison_key, realise};
pub use church_slavonic_core::verb::Conj;

pub use church_slavonic_core::schema::{
    l_participle_cell,
    npron_cell,
    CASES, DEGREES, GENDERS, NUMBERS, PERSONS, PRESENT_STEM_CELL, VERB_BLOCKS, VERB_CLASS_CELL,
    adj_cell, noun_cell, participle_cell, participle_stem_cell, pronoun_cell, verb_cell,
};

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

/// The pronoun row's lemma-less key.
pub const PRONOUN_KEY: &str = "personal";

/// The verb row as the runtime would answer it under a class/present-stem
/// override (cells 546/547), realised like [`Pos::predict`]: every cell
/// through [`church_slavonic_core::resolution::verb_fact_fallback`], the
/// one copy of the fact-resolution order.
pub fn predict_verb_override(
    lemma: &str,
    class: Option<&str>,
    present: Option<&str>,
    recension: &Recension,
) -> Vec<String> {
    let realised = realise(lemma, recension);
    let fact = |i: usize| -> Option<String> {
        if i == VERB_CLASS_CELL {
            class.map(str::to_string)
        } else if i == PRESENT_STEM_CELL {
            present.map(str::to_string)
        } else {
            None
        }
    };
    (0..Pos::Verb.arity())
        .map(|cell| {
            realise(
                &church_slavonic_core::resolution::verb_fact_fallback(
                    &realised, recension, cell, &fact,
                ),
                recension,
            )
        })
        .collect()
}

/// The five parts of speech the tables cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pos {
    Noun,
    Adj,
    Verb,
    Pronoun,
    NPron,
}

impl Pos {
    pub const ALL: [Pos; 5] = [Pos::Noun, Pos::Adj, Pos::Verb, Pos::Pronoun, Pos::NPron];

    pub fn arity(self) -> usize {
        use church_slavonic_core::schema as sch;
        match self {
            Pos::Noun => sch::NOUN_ARITY,
            Pos::Adj => sch::ADJ_ARITY,
            Pos::Verb => sch::VERB_ARITY,
            Pos::Pronoun => sch::PRONOUN_ARITY,
            Pos::NPron => sch::NPRON_ARITY,
        }
    }

    pub fn file_name(self) -> &'static str {
        match self {
            Pos::Noun => "noun_phf.rs",
            Pos::Adj => "adj_phf.rs",
            Pos::Verb => "verb_phf.rs",
            Pos::Pronoun => "pronoun_phf.rs",
            Pos::NPron => "npron_phf.rs",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Pos::Noun => "noun",
            Pos::Adj => "adj",
            Pos::Verb => "verb",
            Pos::Pronoun => "pronoun",
            Pos::NPron => "npron",
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
                for voice in &[Voice::Active, Voice::Passive] {
                    for series in &[Series::Short, Series::Long] {
                        for tense in &[Tense::Present, Tense::Aorist] {
                            for gender in &GENDERS {
                                for number in &NUMBERS {
                                    for case in &CASES {
                                        out[participle_cell(
                                            voice, series, tense, gender, number, case,
                                        )] = ChurchSlavonicCore::participle(
                                            lemma, tense, voice, series, case, number, gender,
                                            recension,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Pos::NPron => {
                for gender in &GENDERS {
                    for number in &NUMBERS {
                        for case in &CASES[..6] {
                            out[npron_cell(gender, number, case)] =
                                ChurchSlavonicCore::npron(lemma, gender, number, case, recension);
                        }
                    }
                }
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
        seen[church_slavonic_core::schema::NOUN_ACCENT_CELL] = true;
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
        seen[church_slavonic_core::schema::ADJ_ACCENT_CELL] = true;
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
        for v in [Voice::Active, Voice::Passive] {
            for sr in [Series::Short, Series::Long] {
                for t in [Tense::Present, Tense::Aorist] {
                    for g in &GENDERS {
                        for n in &NUMBERS {
                            for c in &CASES {
                                let i = participle_cell(&v, &sr, &t, g, n, c);
                                assert!(!seen[i]);
                                seen[i] = true;
                            }
                        }
                    }
                }
            }
            for t in [Tense::Present, Tense::Aorist] {
                let i = participle_stem_cell(&v, &t);
                assert!(!seen[i]);
                seen[i] = true;
            }
        }
        for g in &GENDERS {
            for n in &NUMBERS {
                let i = l_participle_cell(g, n);
                assert!(!seen[i]);
                seen[i] = true;
            }
        }
        for i in [
            PRESENT_STEM_CELL,
            VERB_CLASS_CELL,
            church_slavonic_core::schema::VERB_ACCENT_CELL,
        ] {
            assert!(!seen[i]);
            seen[i] = true;
        }
        assert!(seen.iter().all(|s| *s));
        assert_eq!(
            participle_cell(
                &Voice::Active,
                &Series::Short,
                &Tense::Imperfect,
                &Gender::Masculine,
                &Number::Singular,
                &Case::Nominative
            ),
            participle_cell(
                &Voice::Active,
                &Series::Short,
                &Tense::Aorist,
                &Gender::Masculine,
                &Number::Singular,
                &Case::Nominative
            )
        );
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
