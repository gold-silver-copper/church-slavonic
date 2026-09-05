//! `cargo xtask import alypy|ruwiktionary|witnesses --pos <pos> [--write]`:
//! the cross-checking sources. Each attests cells of lexemes the Polyakov
//! import already holds; the importer counts what the lexicon reproduces
//! (the primary form), what it reaches (an alternative or variant), and
//! adds the rest as variants carrying the source's provenance token
//! (`A:p034` Alypy's page, `R:` ru.wiktionary, `W:` a witnessed line of
//! the print). A lemma the lexicon does not hold is quarantined with the
//! reason — this importer never invents lexemes.

use super::fit::{canonical, translit_equal};
use super::{Outcome, Quarantined};
use crate::sources::alypy::{self, FormWord, TenseWord};
use church_slavonic::cell::{AdjCell, Cell, FiniteTense, NounCell, PronCell, VerbCell};
use church_slavonic::grammar::{Case, Degree, Gender, Number, Person, Recension, Series};
use church_slavonic::lexicon::{self, Lexeme};
use church_slavonic::orthography::{comparison_key, realise};
use church_slavonic::{Lexicon, Pos};
use std::collections::BTreeMap;
use std::error::Error;

const SYN: Recension = Recension::Synodal;
const GENDERS: [Gender; 3] = [Gender::Masculine, Gender::Feminine, Gender::Neuter];

/// One attested cell: the lemma (or the lexeme id for a pronoun), the
/// cell, its forms (primary first) and the provenance token.
#[derive(Debug, Clone)]
pub struct Attestation {
    pub pos: Pos,
    /// A lexeme id (`азъ.pron`) or a lemma to look up by its letters.
    pub lemma: String,
    pub cell: Cell,
    pub forms: Vec<String>,
    pub src: String,
}

// ---------------------------------------------------------------------------
// Alypy
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Block {
    Declension,
    Comparative,
    Finite { future_as_present: bool },
    Imperative,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Noun,
    Adj,
    Verb,
    Personal,
    NPron,
}

struct Paradigm {
    artifact: &'static str,
    index: usize,
    kind: Kind,
    lemma: Option<&'static str>,
    column_lemmas: &'static [&'static str],
    columns: Option<(usize, usize)>,
    defaults: alypy::Defaults,
    block: Block,
}

const fn decl(artifact: &'static str, index: usize, kind: Kind) -> Paradigm {
    Paradigm { artifact, index, kind, lemma: None, column_lemmas: &[], columns: None, defaults: alypy::Defaults { number: None, tense: None }, block: Block::Declension }
}
const fn verb(artifact: &'static str, index: usize, tense: Option<TenseWord>, block: Block) -> Paradigm {
    Paradigm { artifact, index, kind: Kind::Verb, lemma: None, column_lemmas: &[], columns: None, defaults: alypy::Defaults { number: None, tense }, block }
}

/// The paradigm tables of the grammar (the 1.x selection, kept): every
/// other `Decline` table is deliberately not a source (ending schemata,
/// numerals, periphrastic tenses, participle formation).
const ALYPY_PARADIGMS: &[Paradigm] = &[
    decl("p034.htm", 0, Kind::Noun),
    decl("p034.htm", 1, Kind::Noun),
    decl("p039.htm", 0, Kind::Noun),
    decl("p041.htm", 0, Kind::Noun),
    decl("p043.htm", 0, Kind::Noun),
    decl("p043.htm", 1, Kind::Noun),
    decl("p043.htm", 2, Kind::Noun),
    Paradigm { defaults: alypy::Defaults { number: Some(Number::Dual), tense: None }, ..decl("p044.htm", 0, Kind::Noun) },
    decl("p047.htm", 0, Kind::Personal),
    Paradigm { columns: Some((0, 3)), ..decl("p047.htm", 1, Kind::Personal) },
    Paradigm { columns: Some((3, 6)), ..decl("p047.htm", 1, Kind::NPron) },
    decl("p047.htm", 2, Kind::NPron),
    Paradigm { column_lemmas: &["кто̀", "что̀", "что̀"], defaults: alypy::Defaults { number: Some(Number::Singular), tense: None }, ..decl("p048.htm", 0, Kind::NPron) },
    decl("p048.htm", 1, Kind::NPron),
    decl("p048.htm", 2, Kind::NPron),
    Paradigm { columns: Some((0, 3)), ..decl("p048.htm", 4, Kind::NPron) },
    Paradigm { columns: Some((3, 6)), ..decl("p048.htm", 4, Kind::Adj) },
    decl("p053.htm", 0, Kind::Adj),
    decl("p053.htm", 1, Kind::Adj),
    decl("p057.htm", 0, Kind::Adj),
    decl("p057.htm", 1, Kind::Adj),
    decl("p057.htm", 2, Kind::Adj),
    decl("p057.htm", 3, Kind::Adj),
    Paradigm { block: Block::Comparative, ..decl("p060.htm", 0, Kind::Adj) },
    Paradigm { lemma: Some("бы́ти"), ..verb("p081.htm", 0, None, Block::Finite { future_as_present: false }) },
    Paradigm { lemma: Some("бы́ти"), ..verb("p081.htm", 1, None, Block::Finite { future_as_present: false }) },
    verb("p082.htm", 0, Some(TenseWord::Present), Block::Finite { future_as_present: false }),
    verb("p086.htm", 1, Some(TenseWord::Aorist), Block::Finite { future_as_present: false }),
    verb("p087.htm", 1, Some(TenseWord::Imperfect), Block::Finite { future_as_present: false }),
    verb("p087.htm", 2, Some(TenseWord::Imperfect), Block::Finite { future_as_present: false }),
    verb("p093.htm", 1, None, Block::Imperative),
    verb("p093.htm", 2, None, Block::Imperative),
    Paradigm { column_lemmas: &["да́ти", "ꙗ҆́сти", "вѣ́дѣти", "и҆мѣ́ти"], ..verb("p103.htm", 0, None, Block::Finite { future_as_present: true }) },
    Paradigm { column_lemmas: &["да́ти", "ꙗ҆́сти", "вѣ́дѣти", "и҆мѣ́ти"], ..verb("p103.htm", 1, None, Block::Imperative) },
];

fn personal_id(person: Person, number: Number) -> &'static str {
    match (person, number) {
        (Person::First, Number::Singular) => "азъ.pron",
        (Person::First, _) => "мы.pron",
        (Person::Second, Number::Singular) => "ты.pron",
        (Person::Second, _) => "вы.pron",
        (Person::Third, _) => "онъ.pron",
    }
}

pub fn alypy_attestations() -> Result<Vec<Attestation>, Box<dyn Error>> {
    let path = super::intermediate_dir().join("alypy.jsonl");
    let mut out = Vec::new();
    for table in alypy::read(&path)? {
        let Some(paradigm) = ALYPY_PARADIGMS.iter().find(|p| p.artifact == table.artifact && p.index == table.index) else { continue };
        let src = format!("A:{}", table.artifact.trim_end_matches(".htm"));
        let all_rows = alypy::rows(&table, paradigm.defaults)?;
        let mut columns: Vec<usize> = all_rows.iter().map(|r| r.column).collect();
        columns.sort_unstable();
        columns.dedup();
        let rank = |column: usize| columns.iter().position(|c| *c == column).unwrap_or(0);
        let rows: Vec<&alypy::Row> = all_rows.iter().filter(|r| paradigm.columns.is_none_or(|(s, e)| (s..e).contains(&rank(r.column)))).collect();
        let masculine_lemma = rows
            .iter()
            .find(|r| r.cases.contains(&Case::Nominative) && r.number == Some(Number::Singular) && r.genders.contains(&Gender::Masculine))
            .and_then(|r| alypy::alternatives(&r.surface).into_iter().next())
            .and_then(|s| alypy::lemma_key(&s));
        for row in rows {
            let forms: Vec<String> = alypy::alternatives(&row.surface).into_iter().filter(|f| !f.contains(' ')).map(|f| realise(&f, &SYN)).collect();
            if forms.is_empty() {
                continue;
            }
            let lemma: String = match paradigm.kind {
                Kind::NPron | Kind::Adj if paradigm.column_lemmas.is_empty() => match (paradigm.block, &masculine_lemma) {
                    (Block::Comparative, Some(l)) => l.clone(),
                    (_, Some(l)) => l.clone(),
                    (_, None) => continue,
                },
                Kind::Personal => String::new(),
                _ => {
                    if let Some(l) = paradigm.lemma {
                        l.to_string()
                    } else if !paradigm.column_lemmas.is_empty() {
                        match paradigm.column_lemmas.get(rank(row.column)) {
                            Some(l) => l.to_string(),
                            None => continue,
                        }
                    } else {
                        match alypy::lemma_key(&row.headword) {
                            Some(l) => l,
                            None => continue,
                        }
                    }
                }
            };
            let genders = if row.genders.is_empty() { GENDERS.to_vec() } else { row.genders.clone() };
            let mut push = |pos: Pos, lemma: String, cell: Cell| {
                out.push(Attestation { pos, lemma, cell, forms: forms.clone(), src: src.clone() });
            };
            match (paradigm.kind, paradigm.block) {
                (Kind::Noun, _) => {
                    if let Some(number) = row.number {
                        for case in &row.cases {
                            push(Pos::Noun, lemma.clone(), Cell::Noun(NounCell::new(*case, number)));
                        }
                    }
                }
                (Kind::Adj, block) => {
                    let degree = if matches!(block, Block::Comparative) { Degree::Comparative } else { Degree::Positive };
                    let series = match row.form {
                        Some(FormWord::Short) => Some(Series::Short),
                        Some(FormWord::Long) => Some(Series::Long),
                        None => None,
                    };
                    if let Some(number) = row.number {
                        for case in &row.cases {
                            for gender in &genders {
                                push(Pos::Adjective, lemma.clone(), Cell::Adj(AdjCell { series, degree, gender: *gender, number, case: *case }));
                            }
                        }
                    }
                }
                (Kind::NPron, _) => {
                    if let Some(number) = row.number {
                        for case in &row.cases {
                            if *case == Case::Vocative {
                                continue;
                            }
                            for gender in &genders {
                                push(Pos::Pronoun, lemma.clone(), Cell::Pron(PronCell { clitic: false, person: None, gender: Some(*gender), number: Some(number), case: *case }));
                            }
                        }
                    }
                }
                (Kind::Verb, Block::Finite { future_as_present }) => {
                    let tense = match row.tense {
                        Some(TenseWord::Present) => FiniteTense::Present,
                        Some(TenseWord::Imperfect) => FiniteTense::Imperfect,
                        Some(TenseWord::Aorist) => FiniteTense::Aorist,
                        Some(TenseWord::Future) if future_as_present => FiniteTense::Present,
                        Some(TenseWord::Future) if paradigm.lemma == Some("бы́ти") => FiniteTense::Future,
                        _ => continue,
                    };
                    if let Some(number) = row.number {
                        for person in &row.persons {
                            push(Pos::Verb, lemma.clone(), Cell::Verb(VerbCell::Finite { tense, person: *person, number }));
                        }
                    }
                }
                (Kind::Verb, _) => {
                    if let Some(number) = row.number {
                        for person in &row.persons {
                            push(Pos::Verb, lemma.clone(), Cell::Verb(VerbCell::Imperative { person: *person, number }));
                        }
                    }
                }
                (Kind::Personal, _) => {
                    let reflexive = forms.iter().any(|f| matches!(church_slavonic::orthography::strip_marks(f).as_str(), "себе" | "себє" | "себѣ" | "собою"));
                    if reflexive {
                        for case in &row.cases {
                            if matches!(*case, Case::Vocative | Case::Nominative) {
                                continue;
                            }
                            push(Pos::Pronoun, "себе.pron".into(), Cell::Pron(PronCell { clitic: false, person: None, gender: None, number: None, case: *case }));
                        }
                        continue;
                    }
                    let person = if paradigm.index == 1 {
                        Person::Third
                    } else {
                        match alypy::lemma_key(&row.headword).map(|l| church_slavonic::orthography::strip_marks(&l)).as_deref() {
                            Some("азъ") => Person::First,
                            Some("ты") => Person::Second,
                            _ => continue,
                        }
                    };
                    if let Some(number) = row.number {
                        for case in &row.cases {
                            if *case == Case::Vocative {
                                continue;
                            }
                            let gender_set: Vec<Option<Gender>> = if person == Person::Third { genders.iter().map(|g| Some(*g)).collect() } else { vec![None] };
                            for gender in gender_set {
                                push(Pos::Pronoun, personal_id(person, number).into(), Cell::Pron(PronCell { clitic: false, person: Some(person), gender, number: Some(number), case: *case }));
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// ru.wiktionary
// ---------------------------------------------------------------------------

pub fn ruwiktionary_attestations() -> Result<Vec<Attestation>, Box<dyn Error>> {
    let path = super::intermediate_dir().join("ruwiktionary.jsonl");
    let mut out = Vec::new();
    for entry in crate::sources::ruwiktionary::read(&path)? {
        let pos = match entry.pos.as_str() {
            "noun" => Pos::Noun,
            "verb" => Pos::Verb,
            "adj" => Pos::Adjective,
            _ => continue,
        };
        let lemma = realise(&entry.word, &SYN);
        for form in &entry.forms {
            let forms: Vec<String> = crate::sources::ruwiktionary::alternatives(form).into_iter().map(|f| realise(&f, &SYN)).collect();
            if forms.is_empty() {
                continue;
            }
            let tag = |t: &str| form.tags.iter().any(|x| x == t);
            let number = if crate::sources::ruwiktionary::is_dual_note(form) || tag("dual") {
                Number::Dual
            } else if tag("plural") {
                Number::Plural
            } else if tag("singular") {
                Number::Singular
            } else {
                continue;
            };
            let case = [("nominative", Case::Nominative), ("genitive", Case::Genitive), ("dative", Case::Dative), ("accusative", Case::Accusative), ("instrumental", Case::Instrumental), ("locative", Case::Locative), ("prepositional", Case::Locative), ("vocative", Case::Vocative)]
                .into_iter()
                .find(|(t, _)| tag(t))
                .map(|(_, c)| c);
            let person = [("first-person", Person::First), ("second-person", Person::Second), ("third-person", Person::Third)].into_iter().find(|(t, _)| tag(t)).map(|(_, p)| p);
            let cell = match pos {
                Pos::Noun => match case {
                    Some(case) => Cell::Noun(NounCell::new(case, number)),
                    None => continue,
                },
                Pos::Verb => match person {
                    Some(person) if tag("imperative") => Cell::Verb(VerbCell::Imperative { person, number }),
                    Some(person) if tag("present") || tag("future") => Cell::Verb(VerbCell::Finite { tense: FiniteTense::Present, person, number }),
                    Some(person) if tag("imperfect") => Cell::Verb(VerbCell::Finite { tense: FiniteTense::Imperfect, person, number }),
                    Some(person) if tag("aorist") => Cell::Verb(VerbCell::Finite { tense: FiniteTense::Aorist, person, number }),
                    _ => continue,
                },
                _ => continue,
            };
            out.push(Attestation { pos, lemma: lemma.clone(), cell, forms, src: "R:".into() });
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Witnesses
// ---------------------------------------------------------------------------

/// `data/witnesses.tsv`: recension, pos, lemma, cell, form, file, quote.
/// A noun cell is the 1.x index (number × 7 + case); a pronoun cell a
/// name (`3.f.sg.dat`, `clit.1.pl.acc`, `refl.clit.acc`); an npron cell
/// `g.n.case`.
pub fn witness_attestations() -> Result<Vec<Attestation>, Box<dyn Error>> {
    let path = crate::workspace_root().join("data/witnesses.tsv");
    let text = std::fs::read_to_string(&path)?;
    let cases = [Case::Nominative, Case::Genitive, Case::Dative, Case::Accusative, Case::Instrumental, Case::Locative, Case::Vocative];
    let numbers = [Number::Singular, Number::Dual, Number::Plural];
    let mut out = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        let [rec, pos, lemma, cell, form, file, ..] = f.as_slice() else { continue };
        if *rec != "syn" {
            continue;
        }
        let src = format!("W:{}", file.rsplit('/').next().unwrap_or(file).trim_end_matches(".txt"));
        let (pos, lemma, cell) = match *pos {
            "noun" => {
                let i: usize = cell.parse()?;
                (Pos::Noun, lemma.to_string(), Cell::Noun(NounCell::new(cases[i % 7], numbers[i / 7])))
            }
            "pronoun" => {
                let parts: Vec<&str> = cell.split('.').collect();
                let clitic = parts.contains(&"clit");
                let refl = parts.first() == Some(&"refl");
                let case = church_slavonic::cell::parse_case(parts.last().copied().unwrap_or("")).ok_or_else(|| format!("witness cell {cell}"))?;
                if refl {
                    (Pos::Pronoun, "себе.pron".to_string(), Cell::Pron(PronCell { clitic, person: None, gender: None, number: None, case }))
                } else {
                    let rest: Vec<&str> = parts.iter().copied().filter(|p| *p != "clit").collect();
                    let person = church_slavonic::cell::parse_person(rest[0]).ok_or_else(|| format!("witness cell {cell}"))?;
                    let (gender, number) = if rest.len() == 4 {
                        (church_slavonic::cell::parse_gender(rest[1]), church_slavonic::cell::parse_number(rest[2]))
                    } else {
                        (None, church_slavonic::cell::parse_number(rest[1]))
                    };
                    let number = number.ok_or_else(|| format!("witness cell {cell}"))?;
                    (Pos::Pronoun, personal_id(person, number).to_string(), Cell::Pron(PronCell { clitic, person: Some(person), gender, number: Some(number), case }))
                }
            }
            "npron" => {
                let c = PronCell::parse(cell).ok_or_else(|| format!("witness cell {cell}"))?;
                (Pos::Pronoun, lemma.to_string(), Cell::Pron(c))
            }
            other => {
                let p = Pos::parse(other).ok_or_else(|| format!("witness pos {other}"))?;
                let c = Cell::parse(p, cell).ok_or_else(|| format!("witness cell {cell}"))?;
                (p, lemma.to_string(), c)
            }
        };
        out.push(Attestation { pos, lemma, cell, forms: vec![realise(form, &SYN)], src });
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// The check and the merge
// ---------------------------------------------------------------------------

/// The lexemes a lemma slot names: an id exactly, else every lexeme of the
/// part of speech whose lemma has the same letters; for an adjective also
/// the lexeme whose short masculine nominative the source used as its
/// headword (Alypy's мꙋ́дръ is the lexicon's мꙋ́дрый.a).
fn candidates<'a>(lexicon: &'a Lexicon, pos: Pos, lemma: &str, short_index: &BTreeMap<String, Vec<String>>) -> Vec<&'a Lexeme> {
    if crate::treebank::node::is_lexeme_id(lemma) {
        return lexicon.get(lemma).into_iter().collect();
    }
    let key = comparison_key(lemma);
    let mut out: Vec<&Lexeme> = lexicon.iter().filter(|l| l.pos == pos && comparison_key(&l.lemma) == key).collect();
    if out.is_empty() && pos == Pos::Adjective {
        out = short_index.get(&key).into_iter().flatten().filter_map(|id| lexicon.get(id)).collect();
    }
    out
}

/// Short masculine nominative → adjective ids.
fn short_nominatives(lexicon: &Lexicon) -> BTreeMap<String, Vec<String>> {
    let cell = Cell::Adj(AdjCell { series: Some(Series::Short), degree: Degree::Positive, gender: Gender::Masculine, number: Number::Singular, case: Case::Nominative });
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for l in lexicon.iter().filter(|l| l.pos == Pos::Adjective) {
        for f in l.forms(cell) {
            out.entry(comparison_key(&f.print(SYN))).or_default().push(l.id.clone());
        }
    }
    out
}

/// A form the source prints without its accent where the lexicon's forms
/// carry one says nothing about the letters' stress: skipped, counted.
fn unaccented(form: &str) -> bool {
    church_slavonic::form::Form::from_print(form).stress.is_none() && form.chars().filter(|c| church_slavonic::orthography::is_vowel_letter(*c)).count() > 1
}

/// The cells a source cell may stand for in the lexicon: an adjective
/// cell without a series is either series; a plain pronoun cell also
/// covers its clitic twin (the grammar prints the clitic as an alternative).
fn twins(cell: Cell) -> Vec<Cell> {
    match cell {
        Cell::Adj(a) if a.series.is_none() => vec![Cell::Adj(AdjCell { series: Some(Series::Long), ..a }), Cell::Adj(AdjCell { series: Some(Series::Short), ..a })],
        Cell::Pron(p) if !p.clitic => vec![cell, Cell::Pron(PronCell { clitic: true, ..p })],
        other => vec![other],
    }
}

pub fn import(source: &str, pos: Pos) -> Result<Outcome, Box<dyn Error>> {
    let attestations = match source {
        "alypy" => alypy_attestations()?,
        "ruwiktionary" => ruwiktionary_attestations()?,
        "witnesses" => witness_attestations()?,
        other => return Err(format!("import {other}: unknown source").into()),
    };
    let lexicon = Lexicon::synodal();
    let short_index = if pos == Pos::Adjective { short_nominatives(lexicon) } else { BTreeMap::new() };
    let mut o = Outcome::default();
    let mut modified: BTreeMap<String, Lexeme> = BTreeMap::new();
    for a in attestations.iter().filter(|a| a.pos == pos) {
        o.bump("cells attested");
        let found = candidates(lexicon, pos, &a.lemma, &short_index);
        if found.is_empty() {
            o.quarantine.push(Quarantined { recension: SYN, pos, lemma: a.lemma.clone(), source: a.src.clone(), reason: "lemma not in the lexicon", detail: format!("{}={}", a.cell.name(), a.forms.join("|")) });
            o.bump("cells: lemma not in the lexicon");
            continue;
        }
        for (k, form) in a.forms.iter().enumerate() {
            if unaccented(form) {
                o.bump("forms skipped: unaccented in an accented source");
                continue;
            }
            let want = canonical(form);
            let mut status = None;
            for lexeme in &found {
                for cell in twins(a.cell) {
                    let forms = lexeme.forms(cell);
                    if forms.first().is_some_and(|f| translit_equal(&f.print(SYN), &want)) {
                        status = Some(if k == 0 { "cells reproduced" } else { "cells reachable (any alternative/variant)" });
                        break;
                    }
                    if forms.iter().any(|f| translit_equal(&f.print(SYN), &want)) {
                        status = status.or(Some("cells reachable (any alternative/variant)"));
                    }
                }
            }
            match status {
                Some(s) => o.bump(s),
                None => {
                    // the first lexeme that declares the cell takes the variant
                    let Some(lexeme) = found.iter().find(|l| twins(a.cell).iter().any(|c| l.class().is_some_and(|k| k.has(*c)) || l.overrides.iter().any(|(oc, _)| oc == c))).or(found.first()) else { continue };
                    let entry = modified.entry(lexeme.id.clone()).or_insert_with(|| (*lexeme).clone());
                    let cell = twins(a.cell)[0];
                    let at = match entry.variants.iter().position(|(c, _)| *c == cell) {
                        Some(i) => i,
                        None => {
                            entry.variants.push((cell, Vec::new()));
                            entry.variants.len() - 1
                        }
                    };
                    if !entry.variants[at].1.contains(&want) {
                        entry.variants[at].1.push(want.clone());
                    }
                    if !entry.src.contains(&a.src) {
                        entry.src.push(a.src.clone());
                    }
                    o.bump("cells added as variants");
                    o.exception_samples.push((entry.lemma.clone(), entry.class.clone(), entry.stress.clone(), cell.name(), want.clone(), lexeme.inflect(cell).map(|f| f.print(SYN)).unwrap_or_default()));
                }
            }
        }
    }
    *o.counts.entry("lexemes modified").or_default() += modified.len() as u64;
    o.lexemes = modified.into_values().collect();
    Ok(o)
}

/// Merge the modified lexemes into the lexicon file (by id).
pub fn write(o: &Outcome, pos: Pos) -> Result<(), Box<dyn Error>> {
    let path = super::lexicon_dir().join("syn").join(super::lexicon_file(pos));
    let mut lexemes = lexicon::parse(&std::fs::read_to_string(&path)?, pos)?;
    let mut n = 0;
    for m in &o.lexemes {
        if let Some(slot) = lexemes.iter_mut().find(|l| l.id == m.id) {
            *slot = m.clone();
            n += 1;
        }
    }
    std::fs::write(&path, lexicon::format(&lexemes))?;
    println!("updated {n} lexemes in {}", path.display());
    Ok(())
}
