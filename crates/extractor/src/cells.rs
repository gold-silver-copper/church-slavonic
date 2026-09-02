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
//! - pronoun (119): the closed personal matrix — first person `number * 6 +
//!   case` (0..18), second person likewise at 18.., third person
//!   `36 + (gender * 3 + number) * 6 + case` — over the six non-vocative cases
//!   (the vocative answers with the nominative), then the reflexive
//!   (90..96) and the clitic cells (96..119; `schema::clitic_cell`). The
//!   pronoun map is keyed `<tag>:personal`: the facade's `pronoun` call
//!   takes no lemma.

use church_slavonic_core::ChurchSlavonicCore;
use church_slavonic_core::grammar::*;
use church_slavonic_core::orthography::{comparison_key, realise};
pub use church_slavonic_core::verb::Conj;

pub use church_slavonic_core::schema::{
    CASES, DEGREES, GENDERS, NUMBERS, PERSONS, PRESENT_STEM_CELL, VERB_BLOCKS, VERB_CLASS_CELL,
    adj_cell, clitic_cell, l_participle_cell, noun_cell, npron_cell, participle_cell,
    participle_stem_cell, pronoun_cell, reflexive_cell, reflexive_clitic_cell, verb_cell,
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
                                if let Some(i) = clitic_cell(person, number, gender, case) {
                                    out[i] =
                                        ChurchSlavonicCore::clitic(person, number, gender, case, recension)
                                            .unwrap_or_default()
                                            .to_string();
                                }
                            }
                        }
                    }
                }
                for case in &CASES[..6] {
                    out[reflexive_cell(case)] =
                        ChurchSlavonicCore::reflexive(case, recension).to_string();
                    if let Some(i) = reflexive_clitic_cell(case) {
                        out[i] = ChurchSlavonicCore::reflexive_clitic(case, recension)
                            .unwrap_or_default()
                            .to_string();
                    }
                }
            }
        }
        out
    }
}

/// Decode a cell written in `witnesses.tsv`: a schema index, or a symbolic
/// name — nouns `<num>.<case>` (`pl.acc`), non-personal pronouns
/// `<g>.<num>.<case>` (`m.sg.gen`), the personal pronoun `<1|2>.<num>.<case>`
/// or `3.<g>.<num>.<case>` (`3.f.sg.dat`); numbers `sg|du|pl`, genders
/// `m|f|n`, cases `nom|gen|dat|acc|ins|loc|voc`. Adjectives and verbs take
/// the index only.
pub fn parse_cell(pos: Pos, text: &str) -> Option<usize> {
    if let Ok(i) = text.parse::<usize>() {
        return (i < pos.arity()).then_some(i);
    }
    let parts: Vec<&str> = text.split('.').collect();
    let number = |s: &str| match s {
        "sg" => Some(Number::Singular),
        "du" => Some(Number::Dual),
        "pl" => Some(Number::Plural),
        _ => None,
    };
    let gender = |s: &str| match s {
        "m" => Some(Gender::Masculine),
        "f" => Some(Gender::Feminine),
        "n" => Some(Gender::Neuter),
        _ => None,
    };
    let case = |s: &str| match s {
        "nom" => Some(Case::Nominative),
        "gen" => Some(Case::Genitive),
        "dat" => Some(Case::Dative),
        "acc" => Some(Case::Accusative),
        "ins" => Some(Case::Instrumental),
        "loc" => Some(Case::Locative),
        "voc" => Some(Case::Vocative),
        _ => None,
    };
    match (pos, parts.as_slice()) {
        (Pos::Noun, [n, c]) => Some(noun_cell(&case(c)?, &number(n)?)),
        (Pos::NPron, [g, n, c]) => {
            let (g, n, c) = (gender(g)?, number(n)?, case(c)?);
            (c != Case::Vocative).then(|| npron_cell(&g, &n, &c))
        }
        (Pos::Pronoun, [p @ ("1" | "2"), n, c]) => {
            let person = if *p == "1" { Person::First } else { Person::Second };
            let (n, c) = (number(n)?, case(c)?);
            (c != Case::Vocative).then(|| pronoun_cell(&person, &n, &Gender::Masculine, &c))
        }
        (Pos::Pronoun, ["3", g, n, c]) => {
            let (g, n, c) = (gender(g)?, number(n)?, case(c)?);
            (c != Case::Vocative).then(|| pronoun_cell(&Person::Third, &n, &g, &c))
        }
        (Pos::Pronoun, ["refl", c]) => {
            let c = case(c)?;
            (!matches!(c, Case::Vocative | Case::Nominative)).then(|| reflexive_cell(&c))
        }
        (Pos::Pronoun, ["refl", "clit", c]) => reflexive_clitic_cell(&case(c)?),
        (Pos::Pronoun, ["clit", p @ ("1" | "2"), n, c]) => {
            let person = if *p == "1" { Person::First } else { Person::Second };
            clitic_cell(&person, &number(n)?, &Gender::Masculine, &case(c)?)
        }
        (Pos::Pronoun, ["clit", "3", g, n, c]) => {
            clitic_cell(&Person::Third, &number(n)?, &gender(g)?, &case(c)?)
        }
        _ => None,
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
    #[test]
    fn symbolic_cells_decode_to_the_schema() {
        use super::*;
        assert_eq!(parse_cell(Pos::Noun, "pl.acc"), Some(noun_cell(&Case::Accusative, &Number::Plural)));
        assert_eq!(parse_cell(Pos::Noun, "3"), Some(3));
        assert_eq!(parse_cell(Pos::Noun, "99"), None);
        assert_eq!(
            parse_cell(Pos::Pronoun, "3.f.sg.dat"),
            Some(pronoun_cell(&Person::Third, &Number::Singular, &Gender::Feminine, &Case::Dative))
        );
        assert_eq!(
            parse_cell(Pos::Pronoun, "1.pl.nom"),
            Some(pronoun_cell(&Person::First, &Number::Plural, &Gender::Neuter, &Case::Nominative))
        );
        assert_eq!(parse_cell(Pos::Pronoun, "3.sg.dat"), None);
        assert_eq!(parse_cell(Pos::Pronoun, "refl.dat"), Some(reflexive_cell(&Case::Dative)));
        assert_eq!(parse_cell(Pos::Pronoun, "refl.nom"), None);
        assert_eq!(
            parse_cell(Pos::Pronoun, "clit.1.sg.dat"),
            clitic_cell(&Person::First, &Number::Singular, &Gender::Masculine, &Case::Dative)
        );
        // the dual and plural datives have a cell (blank by rule)
        assert_eq!(parse_cell(Pos::Pronoun, "clit.1.pl.dat"), Some(100));
        assert_eq!(
            parse_cell(Pos::Pronoun, "clit.3.f.pl.acc"),
            clitic_cell(&Person::Third, &Number::Plural, &Gender::Feminine, &Case::Accusative)
        );
        assert_eq!(parse_cell(Pos::Pronoun, "refl.clit.acc"), reflexive_clitic_cell(&Case::Accusative));
        assert_eq!(parse_cell(Pos::Pronoun, "1.pl.voc"), None);
        assert_eq!(
            parse_cell(Pos::NPron, "m.sg.gen"),
            Some(npron_cell(&Gender::Masculine, &Number::Singular, &Case::Genitive))
        );
    }

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
        for c in &CASES[..6] {
            let i = reflexive_cell(c);
            assert!(!seen[i]);
            seen[i] = true;
            if let Some(i) = reflexive_clitic_cell(c) {
                assert!(!seen[i]);
                seen[i] = true;
            }
        }
        for p in &PERSONS {
            for g in &GENDERS {
                for n in &NUMBERS {
                    for c in &CASES[..6] {
                        let Some(i) = clitic_cell(p, n, g, c) else { continue };
                        if *p != Person::Third && *g != Gender::Masculine {
                            assert_eq!(Some(i), clitic_cell(p, n, &Gender::Masculine, c));
                            continue;
                        }
                        assert!(!seen[i], "{i}");
                        seen[i] = true;
                    }
                }
            }
        }
        assert!(seen.iter().all(|s| *s));
        // the decoder is the inverse
        use church_slavonic_core::schema::{PronounCell, pronoun_features};
        assert_eq!(
            pronoun_features(reflexive_cell(&Case::Dative)),
            PronounCell::Reflexive { case: Case::Dative }
        );
        assert_eq!(
            pronoun_features(clitic_cell(&Person::Third, &Number::Plural, &Gender::Feminine, &Case::Accusative).unwrap()),
            PronounCell::Clitic { person: Person::Third, number: Number::Plural, gender: Gender::Feminine, case: Case::Accusative }
        );
        assert_eq!(
            pronoun_features(reflexive_clitic_cell(&Case::Accusative).unwrap()),
            PronounCell::ReflexiveClitic { case: Case::Accusative }
        );
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
