//! The Old Church Slavonic importers:
//!
//! - `cargo xtask import kaikki --pos <pos>`: Kaikki's paradigm tables,
//!   read cell by cell from `data/intermediate/kaikki-cells.jsonl` (written
//!   by `scripts/kaikki-to-classes.py`, which also seeds the OCS class
//!   tables). Each entry is fitted to its seeded class first and to every
//!   class of the inventory, and keeps the one reproducing the most cells
//!   (a tie is a note). An entry whose citation cell is not its lemma is
//!   the source's typo class: quarantined as `kaikki-nom-mismatch`.
//! - `cargo xtask import ud --pos <pos>`: the UD PROIEL train split's
//!   attestations (`data/intermediate/ud_proiel.jsonl`, written by `cargo
//!   xtask filter-ud`) — variants with `U:` provenance on the lexemes the
//!   Kaikki import holds, new lexemes fitted to the inventory for the rest.
//!
//! No stress: the OCS print carries none.

use super::fit::{Attested, Bundled, Fit, canonical_in, fit, translit_equal};
use super::{Outcome, Quarantined};
use church_slavonic::cell::Cell;
use church_slavonic::form::Form;
use church_slavonic::grammar::{Gender, Recension};
use church_slavonic::lexicon::Lexeme;
use church_slavonic::orthography::comparison_key;
use church_slavonic::paradigm::{Class, table_of};
use church_slavonic::{Lexicon, Pos};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;

const OCS: Recension = Recension::OldChurchSlavonic;

#[derive(Debug, Deserialize)]
struct KaikkiRecord {
    pos: String,
    lemma: String,
    letters: String,
    class: String,
    gender: String,
    /// Stems the class cannot derive (the present stem of a verb).
    #[serde(default)]
    stems: BTreeMap<String, String>,
    cells: BTreeMap<String, Vec<String>>,
}

fn pos_of(tag: &str) -> Option<Pos> {
    match tag {
        "n" => Some(Pos::Noun),
        "a" => Some(Pos::Adjective),
        "v" => Some(Pos::Verb),
        "pron" => Some(Pos::Pronoun),
        _ => None,
    }
}

/// The citation cell of a part of speech.
fn lemma_cell(pos: Pos, lemma: &str) -> Option<Cell> {
    let name = match pos {
        Pos::Noun => "nom.sg",
        Pos::Adjective => {
            if lemma.ends_with('и') { "long.pos.m.sg.nom" } else { "short.pos.m.sg.nom" }
        }
        Pos::Verb => "inf",
        Pos::Pronoun => "m.sg.nom",
        Pos::Closed => return Some(Cell::Word),
    };
    Cell::parse(pos, name)
}

/// The best fit of `attested` over `classes` (the first named class
/// preferred on a tie), with a note naming the runners-up that tie. A
/// class that does not produce the lemma from the citation cell is tried
/// only when no class does.
#[allow(clippy::too_many_arguments)]
fn best_fit(id: &str, lemma: &str, pos: Pos, classes: &[&Class], seeded: Option<&str>, gender: Option<Gender>, stems: Vec<(String, String)>, attested: &Attested, src: Vec<String>, note: String) -> Option<Fit> {
    let letters = Form::from_print(lemma).letters;
    let citation = lemma_cell(pos, &letters);
    let produces_lemma = |class: &Class| -> bool {
        let Some(cell) = citation else { return true };
        let subject = church_slavonic::paradigm::Subject { lemma: &letters, animate: None, stems: &stems };
        class.letters(cell, &subject).iter().any(|l| comparison_key(&l.letters) == comparison_key(&letters))
    };
    let fitting: Vec<&Class> = classes.iter().copied().filter(|c| produces_lemma(c)).collect();
    let mut classes: Vec<&Class> = if fitting.is_empty() { classes.to_vec() } else { fitting };
    // a residue class (`V:res:`) exists for the lexemes that seeded it,
    // whose present stems sit on their own lines; a lexeme the seeding
    // did not place is fitted to the derived classes only
    if seeded.is_none() && classes.iter().any(|c| !c.name.starts_with("V:res:")) {
        classes.retain(|c| !c.name.starts_with("V:res:"));
    }
    // a tie goes to the class whose exemplar ends like the lemma (the
    // class names encode the ending); a class seeded from entries with
    // no present forms (`?` in its name) comes last
    let shared_suffix = |a: &str, b: &str| a.chars().rev().zip(b.chars().rev()).take_while(|(x, y)| x == y).count();
    classes.sort_by_key(|c| {
        let junk = c.name.contains('?') || c.name.starts_with("V:res:");
        let seeded = seeded == Some(c.name.as_str());
        (junk, !seeded, std::cmp::Reverse(shared_suffix(&c.exemplar, &letters)))
    });
    let mut best: Option<Fit> = None;
    let mut ties: Vec<String> = Vec::new();
    for class in classes {
        let f = fit(id, lemma, pos, OCS, class, gender, None, stems.clone(), attested, &Bundled::new(), src.clone(), note.clone());
        match &best {
            None => best = Some(f),
            Some(b) if f.reproduced > b.reproduced => {
                ties.clear();
                best = Some(f);
            }
            Some(b) if f.reproduced == b.reproduced && f.reproduced > 0 => ties.push(f.lexeme.class.clone()),
            _ => {}
        }
    }
    if let Some(b) = &mut best
        && !ties.is_empty()
    {
        let tie = format!("tie {}", ties.join(","));
        b.lexeme.note = if b.lexeme.note == "-" || b.lexeme.note.is_empty() { tie } else { format!("{}; {tie}", b.lexeme.note) };
    }
    best
}

/// The lexeme id of OCS letters: the letters and the tag, numbered for a
/// homograph.
fn id_for(ids: &mut HashMap<String, u32>, letters: &str, pos: Pos) -> String {
    let n = ids.entry(letters.to_string()).or_default();
    *n += 1;
    if *n == 1 { format!("{letters}.{}", pos.tag()) } else { format!("{letters}.{}.{n}", pos.tag()) }
}

fn record_fit(o: &mut Outcome, f: Fit) {
    *o.counts.entry("cells attested").or_default() += f.attested as u64;
    *o.counts.entry("cells reproduced").or_default() += f.reproduced as u64;
    *o.counts.entry("cells reachable (any alternative/variant)").or_default() += f.reachable as u64;
    *o.counts.entry("cells: true exceptions (no alternative fits)").or_default() += f.exceptions as u64;
    if f.exceptions > 0 {
        o.bump("lexemes with a true exception");
    }
    if !f.lexeme.overrides.is_empty() {
        o.bump("lexemes with overrides");
    }
    for cell in &f.letter_misses {
        *o.letter_misses.entry((f.lexeme.class.clone(), cell.name())).or_default() += 1;
    }
    for (cell, attested_form) in &f.lexeme.overrides {
        *o.override_cells.entry(cell.name()).or_default() += 1;
        let mut bare = f.lexeme.clone();
        bare.overrides.retain(|(c, _)| c != cell);
        let predicted = bare.inflect(*cell).map(|x| x.print(OCS)).unwrap_or_default();
        o.exception_samples.push((f.lexeme.lemma.clone(), f.lexeme.class.clone(), f.lexeme.stress.clone(), cell.name(), attested_form.clone(), predicted));
    }
    o.lexemes.push(f.lexeme);
}

pub fn import_kaikki(pos: Pos) -> Result<Outcome, Box<dyn Error>> {
    let path = super::intermediate_dir().join("kaikki-cells.jsonl");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e} (run scripts/kaikki-to-classes.py)", path.display()))?;
    let classes = table_of(pos, OCS);
    let all: Vec<&Class> = classes.iter().collect();
    let mut o = Outcome::default();
    let mut ids: HashMap<String, u32> = HashMap::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let r: KaikkiRecord = serde_json::from_str(line)?;
        if pos_of(&r.pos) != Some(pos) {
            continue;
        }
        o.bump("entries");
        let lemma = Form::unaccented(r.letters.clone()).print(OCS);
        let gender = match r.gender.as_str() {
            "m" => Some(Gender::Masculine),
            "f" => Some(Gender::Feminine),
            "n" => Some(Gender::Neuter),
            _ => None,
        };
        let mut attested: Attested = BTreeMap::new();
        for (name, forms) in &r.cells {
            let Some(cell) = Cell::parse(pos, name) else {
                o.bump("forms skipped: no cell for the tags");
                continue;
            };
            for f in forms {
                let print = Form::unaccented(f.clone()).print(OCS);
                let entry = attested.entry(cell).or_default();
                if !entry.contains(&print) {
                    entry.push(print);
                }
            }
        }
        if attested.is_empty() {
            o.quarantine.push(Quarantined { recension: OCS, pos, lemma: r.lemma.clone(), source: "K:".into(), reason: "no analysed forms", detail: String::new() });
            continue;
        }
        // a plurale tantum (кънигꙑ): no singular cell attested
        let plurale_tantum = pos == Pos::Noun && !attested.keys().any(|c| matches!(c, Cell::Noun(n) if n.number == church_slavonic::grammar::Number::Singular));
        let note = if plurale_tantum { "pl-tantum".to_string() } else { String::new() };
        // Kaikki's typo class: the citation cell does not print the lemma
        if let Some(cell) = lemma_cell(pos, &r.letters)
            && let Some(first) = attested.get(&cell).and_then(|v| v.first())
            && comparison_key(first) != comparison_key(&lemma)
        {
            o.quarantine.push(Quarantined { recension: OCS, pos, lemma: r.lemma.clone(), source: "K:".into(), reason: "kaikki-nom-mismatch", detail: first.clone() });
            continue;
        }
        let mut stems: Vec<(String, String)> = r.stems.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        if pos == Pos::Pronoun {
            // the third person is the seeded PP3 line (the treebank's lemma и)
            if r.letters == "и" {
                o.bump("entries: the third person (seeded by hand)");
                continue;
            }
            pronoun_stems(&r.letters, &mut stems);
        }
        let mut ordered: Vec<&Class> = classes.get(&r.class).into_iter().collect();
        ordered.extend(all.iter().copied().filter(|c| c.name != r.class));
        let src = vec![format!("K:{}", if r.class == "-" { "-".to_string() } else { r.class.clone() })];
        let Some(f) = best_fit(&id_for(&mut ids, &r.letters, pos), &lemma, pos, &ordered, Some(&r.class), gender, stems, &attested, src, note) else { continue };
        record_fit(&mut o, f);
    }
    o.lexemes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(o)
}

/// A verb's present stem read off its attested present, imperative and
/// present-participle forms (their longest common prefix), when it is
/// not the infinitive's stem.
fn present_stem(letters: &str, attested: &Attested) -> Option<String> {
    use church_slavonic::cell::VerbCell;
    let present: Vec<&str> = attested
        .iter()
        .filter(|(c, _)| matches!(c, Cell::Verb(VerbCell::Finite { tense: church_slavonic::cell::FiniteTense::Present, .. } | VerbCell::Imperative { .. } | VerbCell::Participle { tense: church_slavonic::cell::PartTense::Present, .. })))
        .flat_map(|(_, forms)| forms.iter().map(String::as_str))
        .collect();
    if present.is_empty() {
        return None;
    }
    let mut prefix: Vec<char> = present[0].chars().collect();
    for f in &present[1..] {
        let n = prefix.iter().zip(f.chars()).take_while(|(a, b)| *a == b).count();
        prefix.truncate(n);
    }
    let stem2: String = prefix.into_iter().collect();
    let stem1 = letters.strip_suffix("ти").or_else(|| letters.strip_suffix("щи")).unwrap_or(letters);
    // never a whole form (the census's `artefact`): the stem must be
    // shorter than every present form it was read from
    let n = stem2.chars().count();
    (n >= 2 && stem2 != stem1 && present.iter().all(|f| f.chars().count() > n)).then_some(stem2)
}

/// Is the cell one the present stem builds: the present, the imperative,
/// the present participles?
fn is_present_cell(cell: &Cell) -> bool {
    use church_slavonic::cell::VerbCell;
    matches!(cell, Cell::Verb(VerbCell::Finite { tense: church_slavonic::cell::FiniteTense::Present, .. } | VerbCell::Imperative { .. } | VerbCell::Participle { tense: church_slavonic::cell::PartTense::Present, .. }))
}

/// The attested present cells a fit does not reproduce.
fn present_misses(f: &Fit, attested: &Attested) -> usize {
    attested
        .iter()
        .filter(|(c, forms)| is_present_cell(c) && !forms.is_empty())
        .filter(|(c, forms)| {
            let want = canonical_in(&forms[0], OCS);
            !f.lexeme.forms(**c).iter().any(|x| translit_equal(&x.print(OCS), &want))
        })
        .count()
}

/// A pronoun's line-level stems: the enclitic of a compound (иже,
/// къжьдо, никътоже) and the stem before -то of къто/чьто.
fn pronoun_stems(letters: &str, stems: &mut Vec<(String, String)>) {
    let mut core = letters;
    if let Some(e) = ["жьдо", "ждо", "жде", "же"].into_iter().find(|e| letters.strip_suffix(e).is_some_and(|c| !c.is_empty())) {
        stems.push(("encl".to_string(), e.to_string()));
        core = letters.strip_suffix(e).unwrap_or(letters);
    }
    if let Some(stem) = core.strip_suffix("то") {
        stems.push(("1".to_string(), stem.to_string()));
    }
}

/// The UD train records grouped by (pos, lemma): cells → forms by count.
type Groups = BTreeMap<String, BTreeMap<Cell, Vec<(String, u64)>>>;

fn ud_groups(pos: Pos) -> Result<Groups, Box<dyn Error>> {
    let path = super::intermediate_dir().join("ud_proiel.jsonl");
    let records = crate::sources::ud::read_train(&path).map_err(|e| format!("{}: {e} (run cargo xtask filter-ud)", path.display()))?;
    let mut groups: Groups = BTreeMap::new();
    for r in records {
        if pos_of(&r.pos) != Some(pos) {
            continue;
        }
        let Some(cell) = Cell::parse(pos, &r.cell) else { continue };
        groups.entry(r.lemma.clone()).or_default().entry(cell).or_default().push((r.form.clone(), r.count));
    }
    for cells in groups.values_mut() {
        for forms in cells.values_mut() {
            forms.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        }
    }
    Ok(groups)
}

/// The personal pronoun's lexeme for a UD cell (the treebank's lemma is
/// the constant `personal`).
fn personal_id(cell: &Cell) -> Option<&'static str> {
    use church_slavonic::grammar::{Number, Person};
    let Cell::Pron(p) = cell else { return None };
    Some(match (p.person, p.number) {
        (None, None) => "себе.pron",
        (Some(Person::First), Some(Number::Singular)) => "азъ.pron",
        (Some(Person::First), _) => "мы.pron",
        (Some(Person::Second), Some(Number::Singular)) => "ты.pron",
        (Some(Person::Second), _) => "вы.pron",
        (Some(Person::Third), _) => "и.pron",
        _ => return None,
    })
}

pub fn import_ud(pos: Pos) -> Result<Outcome, Box<dyn Error>> {
    let groups = ud_groups(pos)?;
    let lexicon = Lexicon::ocs();
    let classes = table_of(pos, OCS);
    let all: Vec<&Class> = classes.iter().collect();
    let mut o = Outcome::default();
    // every existing lexeme of the part of speech is carried, variants added
    let mut carried: BTreeMap<String, Lexeme> = lexicon.iter().filter(|l| l.pos == pos).map(|l| (l.id.clone(), l.clone())).collect();
    let mut ids: HashMap<String, u32> = HashMap::new();
    for l in carried.values() {
        let stem = l.id.rsplit_once('.').map(|(s, _)| s).unwrap_or(&l.id);
        let stem = stem.rsplit_once('.').filter(|(_, n)| n.chars().all(|c| c.is_ascii_digit())).map(|(s, _)| s).unwrap_or(stem);
        let n = ids.entry(stem.to_string()).or_default();
        *n = (*n).max(1);
    }
    let mut by_key: HashMap<String, Vec<String>> = HashMap::new();
    for l in carried.values() {
        by_key.entry(comparison_key(&l.lemma)).or_default().push(l.id.clone());
    }
    for (lemma, cells) in &groups {
        o.bump("entries");
        let personal = lemma == "personal";
        // the lexemes this lemma names: by id for the personal pronoun's
        // cells, by the lemma's letters otherwise
        let targets: Vec<String> = if personal {
            cells.keys().filter_map(personal_id).map(str::to_string).collect::<std::collections::BTreeSet<_>>().into_iter().collect()
        } else {
            by_key.get(&comparison_key(lemma)).cloned().unwrap_or_default()
        };
        if targets.is_empty() {
            // a new lexeme: fitted to the inventory from its attested cells
            let attested: Attested = cells.iter().map(|(c, forms)| (*c, forms.iter().map(|(f, _)| f.clone()).collect())).collect();
            let letters = Form::from_print(lemma).letters;
            let print = Form::unaccented(letters.clone()).print(OCS);
            let mut stems: Vec<(String, String)> = Vec::new();
            if pos == Pos::Pronoun {
                pronoun_stems(&letters, &mut stems);
            }
            let id = id_for(&mut ids, &letters, pos);
            let Some(mut f) = best_fit(&id, &print, pos, &all, None, None, stems.clone(), &attested, vec!["U:".to_string()], String::new()) else { continue };
            // a verb's present stem is the class's derivation; the stem
            // read off the attested present is the fallback, kept only
            // where no class derivation reproduces the attested present
            if pos == Pos::Verb
                && present_misses(&f, &attested) > 0
                && let Some(stem2) = present_stem(&letters, &attested)
            {
                let mut with = stems.clone();
                with.push(("2".to_string(), stem2));
                if let Some(g) = best_fit(&id, &print, pos, &all, None, None, with, &attested, vec!["U:".to_string()], String::new())
                    && present_misses(&g, &attested) < present_misses(&f, &attested)
                {
                    o.bump("verbs: present stem stored (no derivation fits)");
                    f = g;
                }
            }
            // the citation cell must be the lemma, else the class is wrong
            if let Some(cell) = lemma_cell(pos, &letters)
                && f.lexeme.inflect(cell).is_some_and(|x| comparison_key(&x.print(OCS)) != comparison_key(&print))
                && !f.lexeme.overrides.iter().any(|(c, _)| *c == cell)
            {
                f.lexeme.overrides.push((cell, print.clone()));
                o.bump("lexemes: the citation cell overridden to the lemma");
            }
            o.bump("lexemes created");
            record_fit(&mut o, f);
            continue;
        }
        for id in &targets {
            let Some(lexeme) = carried.get_mut(id) else { continue };
            for (cell, forms) in cells {
                if personal && personal_id(cell) != Some(id.as_str()) {
                    continue;
                }
                let produced: Vec<String> = lexeme.forms(*cell).iter().map(|f| f.print(OCS)).collect();
                for (k, (form, _)) in forms.iter().enumerate() {
                    let want = canonical_in(form, OCS);
                    o.bump("cells attested");
                    if produced.first().is_some_and(|p| translit_equal(p, &want)) {
                        o.bump(if k == 0 { "cells reproduced" } else { "cells reachable (any alternative/variant)" });
                    } else if produced.iter().any(|p| translit_equal(p, &want)) {
                        o.bump("cells reachable (any alternative/variant)");
                    } else {
                        match lexeme.variants.iter_mut().find(|(c, _)| c == cell) {
                            Some((_, v)) => {
                                if !v.contains(&want) {
                                    v.push(want.clone());
                                }
                            }
                            None => lexeme.variants.push((*cell, vec![want.clone()])),
                        }
                        if !lexeme.src.iter().any(|s| s == "U:") {
                            lexeme.src.push("U:".to_string());
                        }
                        o.bump("cells added as variants");
                    }
                }
            }
        }
    }
    let created: Vec<Lexeme> = std::mem::take(&mut o.lexemes);
    o.lexemes = carried.into_values().chain(created).collect();
    o.lexemes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(o)
}
