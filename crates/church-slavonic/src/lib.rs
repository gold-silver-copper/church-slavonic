//! Church Slavonic inflection backed by source-derived lookup tables with a
//! rule-engine fallback, in both recensions of the language.
//!
//! # Architecture
//!
//! Every query follows the same two-tier shape: consult the generated PHF
//! tables (`generated/*_phf.rs`, compiled in via `include!`) for an attested
//! cell first, and fall back to [`ChurchSlavonicCore`]'s regular rules
//! otherwise. The tables and the rules are NOT independent: the extractor
//! blanks any cell the rules already predict, so the tables hold exactly the
//! attested exceptions and nothing else. Changing a rule in
//! `church-slavonic-core` therefore requires regenerating the tables (`cargo
//! xtask refresh-data`). Two source-free tests partially guard this: the
//! `rule_table_sync` test below catches a rule change that makes an existing
//! table cell redundant, and church-slavonic-core's `regular_rules_golden` test
//! catches one that breaks the fallback for un-tabled regular words. Neither is
//! exhaustive — `cargo xtask accuracy` (with the sources) is the authoritative
//! check after a rule change.
//!
//! # Recensions and the table schema
//!
//! Every call takes a [`Recension`] by reference, like `&Number`. Each
//! recension has one source — the Kaikki/Wiktextract Old Church Slavonic dump
//! (`ocs`, unaccented) and the Alypy grammar's printed Synodal paradigms
//! (`syn`, accented) — and every table key carries the recension tag:
//! `"ocs:градъ"`, `"syn:рабъ_2"`. A row is a fixed-arity array of cells in a
//! documented order (nouns 21: `number * 7 + case`; adjectives 126:
//! `((degree * 3 + gender) * 3 + number) * 7 + case` over the positive and
//! comparative degrees; verbs 38: four 9-cell finite blocks present /
//! imperfect / aorist / imperative at `number * 3 + person`, then the present
//! and past active participle citations; the personal pronoun 90: first and
//! second person `number * 6 + case`, third `36 + (gender * 3 + number) * 6 +
//! case`, six cases). An empty cell means the rule serves it. Table cells are
//! returned as attested — Synodal cells keep their printed accents — while
//! rule output is passed through [`orthography::realise`] so it is spelled in
//! the requested recension's letters.
//!
//! # Sense-numbered keys
//!
//! Homograph senses and attested variants are published as `_<n>`-suffixed
//! keys (`сꙑнъ_2`). The underscore is unambiguous: the extractor never admits a
//! lemma containing `_`, and a Church Slavonic lemma never carries ASCII
//! digits, so a trailing `_<digits>` can only be a sense suffix.
//!
//! Key numbers are DETERMINISTIC but NOT immutable. They are assigned by a pure
//! sort of each lemma's emitted forms (see `extractor::assign`): the bare key
//! goes to the standard sense with the lexicographically-smallest signature (or
//! is reserved for the rule engine when a regular paradigm is attested), and
//! the rest number upward. Regenerating from newer sources can therefore
//! renumber a lemma's `_<n>` keys if its attested forms change — there is no
//! lockfile or frozen identity. What is stable is the *set* of forms a lemma
//! exposes and the rule/table layering below.
//!
//! # Lookup semantics (invariants shared by [`ChurchSlavonic::noun`]/[`ChurchSlavonic::adj`]/[`ChurchSlavonic::verb`])
//!
//! 1. Case-insensitive convenience: Title-case and ALL-CAPS input hit the
//!    all-lowercase tables with the casing restored on the value; regular-rule
//!    fallbacks get the identical treatment via the internal `rule_with_case`
//!    helper. Mixed case is never guessed at. Accented input (`ра́бъ`) is
//!    folded to its unaccented key the same way.
//! 2. Base-lemma agreement: when a `_<n>` suffix strips (the word or its base
//!    is a table key), EVERY code path treats the input as that base lemma —
//!    lookups fall back `get_*(word)` then `get_*(base)` so `сꙑнъ_2` inflects
//!    exactly like `сꙑнъ` where its own row is blank.
//! 3. Opaqueness: input that resolves to no key inflects by rule on the whole
//!    string, unchanged (the nominative of `градъ_9` is `градъ_9ъ`, not
//!    `градъ`).

use church_slavonic_core::ChurchSlavonicCore;
pub use church_slavonic_core::grammar::*;
pub use church_slavonic_core::orthography;
use church_slavonic_core::orthography::{realise, strip_marks};
use unicode_normalization::UnicodeNormalization;

mod noun_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/noun_phf.rs"
    ));
}
use noun_phf::*;
mod adj_phf {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/adj_phf.rs"));
}
use adj_phf::*;
mod verb_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/verb_phf.rs"
    ));
}
use verb_phf::*;
mod pronoun_phf {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/pronoun_phf.rs"
    ));
}
use pronoun_phf::*;

/// The key prefix of a recension's rows.
fn tag(recension: &Recension) -> &'static str {
    match recension {
        Recension::OldChurchSlavonic => "ocs",
        Recension::Synodal => "syn",
    }
}

// The cell indices of the schema (see the crate docs and `extractor::cells`).
fn noun_cell(case: &Case, number: &Number) -> usize {
    *number as usize * 7 + *case as usize
}

fn adj_cell(case: &Case, number: &Number, gender: &Gender, degree: &Degree) -> Option<usize> {
    let degree = match degree {
        Degree::Positive => 0,
        Degree::Comparative => 1,
        Degree::Superlative => return None,
    };
    Some(((degree * 3 + *gender as usize) * 3 + *number as usize) * 7 + *case as usize)
}

fn verb_cell(person: &Person, number: &Number, tense: &Tense, form: &Form) -> Option<usize> {
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

fn pronoun_cell(person: &Person, number: &Number, gender: &Gender, case: &Case) -> usize {
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

/// A non-empty table cell.
fn cell(cells: &'static [&'static str], i: Option<usize>) -> Option<&'static str> {
    let c = *cells.get(i?)?;
    (!c.is_empty()).then_some(c)
}

/// The base of a canonical sense-suffixed key (`сꙑнъ_2` -> `сꙑнъ`), or `None`
/// when the word carries no suffix. Decoding goes through
/// [`church_slavonic_core::sense_key::split`] — the single owner of the key
/// format, shared with the extractor's generator.
fn canonical_sense_suffix_base(word: &str) -> Option<&str> {
    church_slavonic_core::sense_key::split(word).map(|(base, _)| base)
}

/// Resolve the base lemma for inflection. A `_<digits>` sense suffix is honored
/// **only when it resolves to a table key** — either `word` itself is a key, or
/// the base is one. Otherwise the input is opaque and returned unchanged.
fn base_lemma(word: &str, is_key: impl Fn(&str) -> bool) -> &str {
    match canonical_sense_suffix_base(word) {
        Some(base) if is_key(word) || is_key(base) => base,
        _ => word,
    }
}

/// How a capitalized input's casing is restored onto a lowercase table value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseStyle {
    AsIs,
    Title,
    Upper,
}

fn case_style(word: &str) -> CaseStyle {
    let Some(first) = word.chars().next() else {
        return CaseStyle::AsIs;
    };
    if !first.is_uppercase() {
        return CaseStyle::AsIs;
    }
    let letters = word.chars().filter(|c| c.is_alphabetic());
    if letters.clone().count() > 1 && letters.clone().all(|c| !c.is_lowercase()) {
        CaseStyle::Upper
    } else if word.chars().skip(1).all(|c| !c.is_uppercase()) {
        CaseStyle::Title
    } else {
        CaseStyle::AsIs
    }
}

fn restyle(s: String, style: CaseStyle) -> String {
    match style {
        CaseStyle::AsIs => s,
        CaseStyle::Upper => s.to_uppercase(),
        CaseStyle::Title => ChurchSlavonic::capitalize_first(&s),
    }
}

/// The table key spelling of an input: NFC, marks stripped, lowercased.
fn fold(word: &str) -> String {
    strip_marks(&word.nfc().collect::<String>()).to_lowercase()
}

/// Run a regular-rule fallback with the same case handling table hits get:
/// Title/ALL-CAPS input is lowercased for the rule and the casing restored on
/// the output; the rule's answer is realised in the recension's spelling.
fn rule_with_case(word: &str, recension: &Recension, rule: impl Fn(&str) -> String) -> String {
    let style = case_style(word);
    let answer = realise(&rule(&realise(word, recension)), recension);
    restyle(answer, style)
}

/// Table lookup: the exact key first; then the folded key (the tables are
/// unaccented lowercase), remembering how to restore the casing onto the value.
fn ci_lookup<T>(word: &str, get: impl Fn(&str) -> Option<T>) -> Option<(T, CaseStyle)> {
    if let Some(v) = get(word) {
        return Some((v, CaseStyle::AsIs));
    }
    let folded = fold(word);
    if folded != word
        && let Some(v) = get(&folded)
    {
        return Some((v, case_style(word)));
    }
    None
}

/// `ci_lookup` on the word, then on its base lemma — skipping the second,
/// byte-identical probe when no sense suffix stripped.
fn ci_lookup_with_base<T>(
    word: &str,
    base: &str,
    get: impl Fn(&str) -> Option<T>,
) -> Option<(T, CaseStyle)> {
    ci_lookup(word, &get).or_else(|| {
        if base != word {
            ci_lookup(base, &get)
        } else {
            None
        }
    })
}

/// Entry point for Church Slavonic inflection.
///
/// `ChurchSlavonic` is the low-level `&str` API for nouns, verbs, adjectives
/// and the personal pronoun in either recension. It delegates attested forms to
/// lookup tables and falls back on [`ChurchSlavonicCore`] for regular
/// inflection rules.
pub struct ChurchSlavonic;
impl ChurchSlavonic {
    /// Declines a noun.
    ///
    /// The lemma is the nominative singular in the recension's spelling. Table
    /// rows serve the attested exceptions; everything else is the rule.
    /// Sense-numbered keys resolve to their homograph or variant; a `_<n>`
    /// suffix that resolves to no table key stays opaque.
    ///
    /// # Examples
    /// ```rust
    /// use church_slavonic::{Case, ChurchSlavonic, Number, Recension};
    ///
    /// assert_eq!(
    ///     ChurchSlavonic::noun("градъ", &Case::Genitive, &Number::Singular, &Recension::OldChurchSlavonic),
    ///     "града"
    /// );
    /// assert_eq!(
    ///     ChurchSlavonic::noun("рабъ", &Case::Dative, &Number::Singular, &Recension::Synodal),
    ///     "рабꙋ̀"
    /// );
    /// ```
    pub fn noun(word: &str, case: &Case, number: &Number, recension: &Recension) -> String {
        let get = |w: &str| get_noun(&format!("{}:{w}", tag(recension)));
        let base = base_lemma(word, |w| ci_lookup(w, get).is_some());
        if let Some((row, style)) = ci_lookup_with_base(word, base, get)
            && let Some(c) = cell(row, Some(noun_cell(case, number)))
        {
            return restyle(c.to_string(), style);
        }
        rule_with_case(base, recension, |w| {
            ChurchSlavonicCore::noun(w, case, number, recension)
        })
    }

    /// Declines (and grades) an adjective. The lemma is the masculine
    /// nominative singular — the short (`новъ`) and long (`новꙑи`) paradigms
    /// are two lemmas, as in the sources.
    ///
    /// # Examples
    /// ```rust
    /// use church_slavonic::{Case, ChurchSlavonic, Degree, Gender, Number, Recension};
    ///
    /// assert_eq!(
    ///     ChurchSlavonic::adj(
    ///         "новъ", &Case::Nominative, &Number::Singular, &Gender::Feminine,
    ///         &Degree::Positive, &Recension::OldChurchSlavonic
    ///     ),
    ///     "нова"
    /// );
    /// ```
    pub fn adj(
        word: &str,
        case: &Case,
        number: &Number,
        gender: &Gender,
        degree: &Degree,
        recension: &Recension,
    ) -> String {
        let get = |w: &str| get_adj(&format!("{}:{w}", tag(recension)));
        let base = base_lemma(word, |w| ci_lookup(w, get).is_some());
        if let Some((row, style)) = ci_lookup_with_base(word, base, get)
            && let Some(c) = cell(row, adj_cell(case, number, gender, degree))
        {
            return restyle(c.to_string(), style);
        }
        rule_with_case(base, recension, |w| {
            ChurchSlavonicCore::adj(w, case, number, gender, degree, recension)
        })
    }

    /// Conjugates a verb. The lemma is the infinitive; [`Form::Infinitive`]
    /// returns it unchanged, [`Form::Participle`] the active participle's
    /// masculine nominative-singular citation for the present or a past tense.
    ///
    /// # Examples
    /// ```rust
    /// use church_slavonic::{ChurchSlavonic, Form, Number, Person, Recension, Tense};
    ///
    /// assert_eq!(
    ///     ChurchSlavonic::verb(
    ///         "нести", &Person::Second, &Number::Singular, &Tense::Present, &Form::Finite,
    ///         &Recension::OldChurchSlavonic
    ///     ),
    ///     "несеши"
    /// );
    /// assert_eq!(
    ///     ChurchSlavonic::verb(
    ///         "бꙑти", &Person::First, &Number::Singular, &Tense::Present, &Form::Finite,
    ///         &Recension::OldChurchSlavonic
    ///     ),
    ///     "ѥсмь"
    /// );
    /// ```
    pub fn verb(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
    ) -> String {
        let get = |w: &str| get_verb(&format!("{}:{w}", tag(recension)));
        let base = base_lemma(word, |w| ci_lookup(w, get).is_some());
        if *form == Form::Infinitive {
            return base.to_string();
        }
        if let Some((row, style)) = ci_lookup_with_base(word, base, get)
            && let Some(c) = cell(row, verb_cell(person, number, tense, form))
        {
            return restyle(c.to_string(), style);
        }
        rule_with_case(base, recension, |w| {
            ChurchSlavonicCore::verb(w, person, number, tense, form, recension)
        })
    }

    /// Returns the personal pronoun for the given grammatical features. Gender
    /// is consulted only in the third person; the vocative answers with the
    /// nominative.
    ///
    /// # Examples
    /// ```rust
    /// use church_slavonic::{Case, ChurchSlavonic, Gender, Number, Person, Recension};
    ///
    /// assert_eq!(
    ///     ChurchSlavonic::pronoun(&Person::First, &Number::Singular, &Gender::Neuter, &Case::Nominative, &Recension::OldChurchSlavonic),
    ///     "азъ"
    /// );
    /// ```
    pub fn pronoun(
        person: &Person,
        number: &Number,
        gender: &Gender,
        case: &Case,
        recension: &Recension,
    ) -> &'static str {
        get_pronoun(&format!("{}:personal", tag(recension)))
            .and_then(|row| cell(row, Some(pronoun_cell(person, number, gender, case))))
            .unwrap_or_else(|| ChurchSlavonicCore::pronoun(person, number, gender, case, recension))
    }

    /// Capitalizes the first letter of a string.
    ///
    /// # Examples
    /// ```rust
    /// use church_slavonic::ChurchSlavonic;
    ///
    /// assert_eq!(ChurchSlavonic::capitalize_first(""), "");
    /// assert_eq!(ChurchSlavonic::capitalize_first("градъ"), "Градъ");
    /// ```
    pub fn capitalize_first(s: &str) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}

#[cfg(test)]
mod rule_table_sync_tests {
    //! Guard on the layering contract between `church-slavonic-core`'s regular
    //! rules and the generated tables: a table cell must hold something the
    //! rules cannot predict — a cell equal to the rule's output is dead weight,
    //! and its appearance means a core rule changed without regenerating.

    use super::*;
    use church_slavonic_core::orthography::comparison_key;

    fn same(recension: &Recension, attested: &str, predicted: &str) -> bool {
        match recension {
            Recension::OldChurchSlavonic => comparison_key(attested) == comparison_key(predicted),
            Recension::Synodal => attested == predicted,
        }
    }

    /// The facade's rule path: realise the lemma in, realise the answer out.
    fn rule(lemma: &str, r: &Recension, f: impl Fn(&str) -> String) -> String {
        realise(&f(&realise(lemma, r)), r)
    }

    fn split(key: &str) -> (Recension, &str) {
        let (tag, rest) = key.split_once(':').expect("prefixed key");
        let recension = match tag {
            "ocs" => Recension::OldChurchSlavonic,
            _ => Recension::Synodal,
        };
        (recension, canonical_sense_suffix_base(rest).unwrap_or(rest))
    }

    #[test]
    fn table_cells_hold_only_what_the_rules_cannot_predict() {
        let mut redundant = Vec::new();
        let cases = [
            Case::Nominative,
            Case::Genitive,
            Case::Dative,
            Case::Accusative,
            Case::Instrumental,
            Case::Locative,
            Case::Vocative,
        ];
        let numbers = [Number::Singular, Number::Dual, Number::Plural];
        let genders = [Gender::Masculine, Gender::Feminine, Gender::Neuter];
        let persons = [Person::First, Person::Second, Person::Third];

        for (key, row) in NOUN_MAP.entries() {
            let (r, lemma) = split(key);
            for n in &numbers {
                for c in &cases {
                    let attested = row[noun_cell(c, n)];
                    if !attested.is_empty()
                        && same(
                            &r,
                            attested,
                            &rule(lemma, &r, |l| ChurchSlavonicCore::noun(l, c, n, &r)),
                        )
                    {
                        redundant.push(format!("noun {key} {c:?} {n:?} -> {attested}"));
                    }
                }
            }
        }
        for (key, row) in ADJ_MAP.entries() {
            let (r, lemma) = split(key);
            for d in [Degree::Positive, Degree::Comparative] {
                for g in &genders {
                    for n in &numbers {
                        for c in &cases {
                            let i = adj_cell(c, n, g, &d).expect("indexed");
                            let attested = row[i];
                            if !attested.is_empty()
                                && same(
                                    &r,
                                    attested,
                                    &rule(lemma, &r, |l| {
                                        ChurchSlavonicCore::adj(l, c, n, g, &d, &r)
                                    }),
                                )
                            {
                                redundant.push(format!("adj {key} cell {i} -> {attested}"));
                            }
                        }
                    }
                }
            }
        }
        for (key, row) in VERB_MAP.entries() {
            let (r, lemma) = split(key);
            let blocks = [
                (Tense::Present, Form::Finite),
                (Tense::Imperfect, Form::Finite),
                (Tense::Aorist, Form::Finite),
                (Tense::Present, Form::Imperative),
            ];
            for (t, f) in &blocks {
                for n in &numbers {
                    for p in &persons {
                        let i = verb_cell(p, n, t, f).expect("indexed");
                        let attested = row[i];
                        if !attested.is_empty()
                            && same(
                                &r,
                                attested,
                                &rule(lemma, &r, |l| ChurchSlavonicCore::verb(l, p, n, t, f, &r)),
                            )
                        {
                            redundant.push(format!("verb {key} cell {i} -> {attested}"));
                        }
                    }
                }
            }
            for (i, t) in [(36, Tense::Present), (37, Tense::Aorist)] {
                let attested = row[i];
                let predicted = rule(lemma, &r, |l| {
                    ChurchSlavonicCore::verb(
                        l,
                        &Person::Third,
                        &Number::Singular,
                        &t,
                        &Form::Participle,
                        &r,
                    )
                });
                if !attested.is_empty() && same(&r, attested, &predicted) {
                    redundant.push(format!("verb {key} cell {i} -> {attested}"));
                }
            }
        }
        for (key, row) in PRONOUN_MAP.entries() {
            let (r, _) = split(key);
            for p in &persons {
                for g in &genders {
                    for n in &numbers {
                        for c in &cases[..6] {
                            let attested = row[pronoun_cell(p, n, g, c)];
                            if !attested.is_empty()
                                && same(&r, attested, ChurchSlavonicCore::pronoun(p, n, g, c, &r))
                            {
                                redundant.push(format!(
                                    "pronoun {key} {p:?} {n:?} {g:?} {c:?} -> {attested}"
                                ));
                            }
                        }
                    }
                }
            }
        }

        assert!(
            redundant.is_empty(),
            "{} table cell(s) are redundant with the regular rules — a core rule changed \
             without regenerating: run `cargo xtask refresh-data`:\n  {}",
            redundant.len(),
            redundant.join("\n  ")
        );
    }
}
