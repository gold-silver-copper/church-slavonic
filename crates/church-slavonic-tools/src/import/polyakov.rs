//! Polyakov's corpus dictionary as a lexicon source, for every part of
//! speech: each entry with a paradigm code becomes a line — class = the
//! code (identity, with its fleeting-vowel and velar twins tried), gender
//! and animacy from the tags, the forms fitted per cell with the corpus
//! count choosing the primary. Titlo spellings (`9^` cells) are skipped
//! here (titlo lemmas are Part 5), unaccented forms in this accented
//! source are transliteration noise and are counted, not stored.

use super::fit::{Attested, Bundled, fit};
use super::{Outcome, Quarantined};
use crate::sources::polyakov::{self, Entry, Features, TenseTag, features};
use church_slavonic::cell::{AdjCell, Cell, FiniteTense, NounCell, PartTense, Pos, PronCell, VerbCell};
use church_slavonic::form::Form;
use church_slavonic::grammar::{Case, Gender, Number, Person, Recension, Series, Voice};
use church_slavonic::orthography::{is_accented, realise};
use church_slavonic::paradigm::{Derivation, table};
use std::collections::{BTreeMap, HashMap};
use std::error::Error;

const SYN: Recension = Recension::Synodal;

/// The source's class code as the table names it.
fn class_codes(class: &str) -> Vec<String> {
    class
        .split('/')
        .map(|c| c.trim().replace('е', "e")) // the legend's Latin e, once typed in Cyrillic
        .filter(|c| !c.is_empty())
        .collect()
}

/// Which lexicon part of speech a Polyakov entry belongs to.
fn pos_of(entry: &Entry) -> Option<Pos> {
    match entry.tags.first().map(String::as_str)? {
        "S" => Some(Pos::Noun),
        "A" | "ANUM" => Some(Pos::Adjective),
        "V" => Some(Pos::Verb),
        // a pronominal adjective with an adjective code declines as one
        "APRO" if !entry.class.starts_with("PA") => Some(Pos::Adjective),
        "APRO" | "SPRO" => Some(Pos::Pronoun),
        "ADV" | "ADVPRO" | "CONJ" | "PR" | "PART" | "INTJ" | "PRED" => Some(Pos::Closed),
        _ => None,
    }
}

/// The cells a tag set names in `pos`; empty when it names none.
fn cells_of(pos: Pos, f: &Features, class: &str) -> Vec<Cell> {
    let genders = |f: &Features| -> Vec<Gender> {
        match f.gender {
            Some(g) => vec![g],
            None => vec![Gender::Masculine, Gender::Feminine, Gender::Neuter],
        }
    };
    let series = |f: &Features| -> Vec<Series> {
        match f.series {
            Some(polyakov::Series::Short) => vec![Series::Short],
            Some(polyakov::Series::Long) => vec![Series::Long],
            None => vec![Series::Short, Series::Long],
        }
    };
    let mut out = Vec::new();
    match pos {
        Pos::Noun => {
            let Some(number) = f.number else { return out };
            for case in &f.cases {
                out.push(Cell::Noun(NounCell::new(*case, number)));
            }
        }
        Pos::Adjective => {
            let Some(number) = f.number else { return out };
            let degree = if f.comparative { church_slavonic::grammar::Degree::Comparative } else { church_slavonic::grammar::Degree::Positive };
            for s in series(f) {
                for g in genders(f) {
                    for case in &f.cases {
                        out.push(Cell::Adj(AdjCell { series: Some(s), degree, gender: g, number, case: *case }));
                    }
                }
            }
        }
        Pos::Pronoun => {
            let personal = class.starts_with("PN") || class.starts_with("PP");
            let person = match class {
                "PNja" | "PPja" | "PNmy" | "PPmy" => Some(Person::First),
                "PNty" | "PPty" | "PNvy" | "PPvy" => Some(Person::Second),
                _ => None,
            };
            if personal {
                let reflexive = class.ends_with("seb");
                let gender = if class.ends_with("kto") || class.ends_with("cto") { Some(Gender::Masculine) } else { None };
                let number = if reflexive { None } else { f.number.or(Some(Number::Singular)) };
                for case in &f.cases {
                    out.push(Cell::Pron(PronCell { clitic: f.clitic, person, gender, number, case: *case }));
                }
            } else {
                let Some(number) = f.number else { return out };
                for g in genders(f) {
                    for case in &f.cases {
                        out.push(Cell::Pron(PronCell { clitic: false, person: None, gender: Some(g), number: Some(number), case: *case }));
                    }
                }
            }
        }
        Pos::Verb => {
            if f.infinitive {
                out.push(Cell::Verb(VerbCell::Infinitive));
                return out;
            }
            if f.participle {
                match f.tense {
                    Some(TenseTag::Perfect) => {
                        let Some(number) = f.number else { return out };
                        // a genderless plural tag is the masculine form
                        let gs = if f.gender.is_none() && number == Number::Plural { vec![Gender::Masculine] } else { genders(f) };
                        for g in gs {
                            out.push(Cell::Verb(VerbCell::LPart { gender: g, number }));
                        }
                    }
                    tense => {
                        let tense = match tense {
                            Some(TenseTag::Present) | Some(TenseTag::Future) => PartTense::Present,
                            Some(TenseTag::Past) | Some(TenseTag::Aorist) => PartTense::Past,
                            _ => return out,
                        };
                        let voice = match f.voice {
                            Some(polyakov::Voice::Passive) => Voice::Passive,
                            _ => Voice::Active,
                        };
                        let Some(number) = f.number else { return out };
                        let cases: Vec<Case> = if f.cases.is_empty() { vec![Case::Nominative] } else { f.cases.clone() };
                        for s in series(f) {
                            for g in genders(f) {
                                for case in &cases {
                                    out.push(Cell::Verb(VerbCell::Participle { tense, voice, series: s, gender: g, number, case: *case }));
                                }
                            }
                        }
                    }
                }
                return out;
            }
            let (Some(person), Some(number)) = (f.person, f.number) else { return out };
            match f.mood {
                Some(polyakov::Mood::Imperative) => out.push(Cell::Verb(VerbCell::Imperative { person, number })),
                _ => {
                    let tense = match f.tense {
                        Some(TenseTag::Present) | Some(TenseTag::Future) => FiniteTense::Present,
                        Some(TenseTag::Aorist) => FiniteTense::Aorist,
                        Some(TenseTag::Imperfect) => FiniteTense::Imperfect,
                        _ => return out,
                    };
                    // бы́ти alone has a synthetic future
                    let tense = if class == "Vbyt" && f.tense == Some(TenseTag::Future) { FiniteTense::Future } else { tense };
                    out.push(Cell::Verb(VerbCell::Finite { tense, person, number }));
                }
            }
        }
        Pos::Closed => out.push(Cell::Word),
    }
    out
}

/// The lexeme's attested cells: print forms per cell, primary first by
/// corpus count. A form whose tag set bundles several cases (`gen/acc`)
/// attests each of them only weakly: where a cell has a form tagged for it
/// alone, the bundled forms are variants, never the primary.
fn attested_cells(entry: &Entry, pos: Pos, class: &str, o: &mut Outcome) -> (Attested, Bundled) {
    let mut counts: HashMap<Cell, BTreeMap<String, (u64, u64)>> = HashMap::new();
    let accented_lemma = is_accented(&entry.lemma);
    for form in &entry.forms {
        if form.cells.is_empty() && pos != Pos::Closed {
            o.bump("forms skipped: unanalysed");
            continue;
        }
        let printed = realise(&form.form, &SYN);
        if form.form.contains('\u{483}') || form.cells.iter().any(|c| c.iter().any(|t| t.starts_with('9'))) {
            o.bump("forms skipped: titlo spelling");
            continue;
        }
        if accented_lemma && !is_accented(&printed) && church_slavonic::orthography::vowel_count(&printed) > 1 {
            o.bump("forms skipped: unaccented in an accented source");
            continue;
        }
        if has_consonant_mark(&form.form) {
            o.bump("forms skipped: erok/abbreviation mark on a consonant");
            continue;
        }
        if stress_marks(&form.form) > 1 {
            o.bump("forms skipped: two stress marks");
            continue;
        }
        if printed.contains(' ') {
            o.bump("forms skipped: more than one word");
            continue;
        }
        if pos == Pos::Closed {
            let e = counts.entry(Cell::Word).or_default().entry(printed.clone()).or_default();
            e.0 += form.count;
            continue;
        }
        // per printed analysis (`|`-separated): a case slash inside it
        // bundles the cells it names
        for analysis in form.tags.split('|') {
            let bundled = ["nom/", "gen/", "dat/", "acc/", "ins/", "loc/", "voc/"].iter().any(|c| analysis.contains(c));
            for set in polyakov::expand(analysis) {
                let f = features(&set);
                let cells = cells_of(pos, &f, class);
                if cells.is_empty() {
                    o.bump("forms skipped: no cell for the tags");
                    continue;
                }
                for cell in cells {
                    let e = counts.entry(cell).or_default().entry(printed.clone()).or_default();
                    if bundled { e.1 += form.count } else { e.0 += form.count }
                }
            }
        }
    }
    let bundled: Bundled = counts
        .iter()
        .filter(|(_, forms)| forms.values().all(|(u, _)| *u == 0))
        .map(|(cell, _)| *cell)
        .collect();
    let attested = counts
        .into_iter()
        .map(|(cell, forms)| {
            let any_unbundled = forms.values().any(|(u, _)| *u > 0);
            let mut v: Vec<(String, u64, u64)> = forms.into_iter().map(|(f, (u, b))| (f, u, b)).collect();
            v.sort_by(|a, b| {
                let ka = (any_unbundled && a.1 == 0, std::cmp::Reverse(a.1 + a.2));
                let kb = (any_unbundled && b.1 == 0, std::cmp::Reverse(b.1 + b.2));
                ka.cmp(&kb).then(a.0.cmp(&b.0))
            });
            (cell, v.into_iter().map(|(f, _, _)| f).collect())
        })
        .collect();
    (attested, bundled)
}

/// The base with its wide letters narrowed (`артемѡн` -> `артемон`).
fn narrowed(base: &str) -> String {
    base.chars().map(|c| match c { 'ѡ' => 'о', 'є' => 'е', other => other }).collect()
}

/// How many stress marks (oxia, varia, kamora) a form carries.
fn stress_marks(form: &str) -> usize {
    use unicode_normalization::UnicodeNormalization;
    form.nfd().filter(|c| matches!(*c, '\u{300}' | '\u{301}' | '\u{302}' | '\u{311}')).count()
}

/// The classes an entry's code may stand for: the code itself, its
/// fleeting-vowel twin (`N1t` -> `N1t*`) and its velar twin for a stem in
/// к/г/х. The fit keeps the best.
fn candidate_classes(code: &str, lemma_letters: &str, strip: usize) -> Vec<String> {
    let mut out = vec![code.to_string()];
    if !code.ends_with('*') {
        out.push(format!("{code}*"));
    }
    // the short-only pronominal codes decline as their PA1 twins
    if let Some(rest) = code.strip_prefix("PA2") {
        out.push(format!("PA1{rest}"));
    }
    let stem_end = lemma_letters.chars().rev().nth(strip);
    // the -ск- adjectives: their plural takes -стїи, not -цыи
    if code.starts_with("A1k") && lemma_letters.chars().rev().nth(strip + 1) == Some('с') && stem_end == Some('к') {
        out.insert(0, "A1sk".to_string());
    }
    if let Some(prefix) = code.strip_suffix('t') {
        match stem_end {
            Some('к') => out.push(format!("{prefix}k")),
            Some('г') => out.push(format!("{prefix}g")),
            Some('х') => out.push(format!("{prefix}x")),
            _ => {}
        }
    }
    out
}

/// A stress-like mark on a consonant: Polyakov's abbreviation notation
/// («нас̑»), never a form.
fn has_consonant_mark(form: &str) -> bool {
    use unicode_normalization::UnicodeNormalization;
    let mut last_vowel = true;
    for c in form.nfd() {
        if matches!(c, '\u{300}' | '\u{301}' | '\u{302}' | '\u{311}' | '\u{487}') {
            if !last_vowel {
                return true;
            }
        } else if !matches!(c as u32, 0x300..=0x36f | 0x483..=0x489) {
            last_vowel = church_slavonic::orthography::is_vowel_letter(c);
        }
    }
    false
}

/// The inserted-vowel stem of a fleeting class, read from the attested
/// zero-ending cell where the rule would spell it differently.
fn inserted_stem(class: &church_slavonic::paradigm::Class, lemma_letters: &str, attested: &Attested) -> Option<(String, String)> {
    if !class.stems.iter().any(|(_, d)| matches!(d, Derivation::Insert(_))) {
        return None;
    }
    let printed = attested
        .iter()
        .find(|(cell, _)| matches!(cell.name().as_str(), "gen.pl" | "short.pos.m.sg.nom"))
        .and_then(|(_, v)| v.first())?;
    let letters = Form::from_print(printed).letters;
    let stem = letters.strip_suffix('ъ').or_else(|| letters.strip_suffix('ь'))?;
    let base: String = {
        let n = lemma_letters.chars().count().saturating_sub(class.strip);
        lemma_letters.chars().take(n).collect()
    };
    let rule = church_slavonic::paradigm::insert_fleeting(&base);
    if stem != rule && Form::new(stem, None, false).key() != Form::new(rule, None, false).key() {
        Some(("ins".to_string(), stem.to_string()))
    } else {
        None
    }
}

/// Numbered stems read off the attested forms: for each stem the class
/// uses, the letters of every attested primary whose class alternative
/// ends in a known ending, minus that ending; the commonest reading that
/// differs from the derived stem is a candidate. The caller keeps it only
/// when the fit improves.
fn inferred_stems(class: &church_slavonic::paradigm::Class, subject: &church_slavonic::paradigm::Subject<'_>, attested: &Attested, refl: Option<&str>) -> Vec<(String, String)> {
    use church_slavonic::paradigm::Shape;
    let derived = class.stems_of(subject);
    let mut votes: HashMap<u8, BTreeMap<String, usize>> = HashMap::new();
    for (cell, forms) in attested {
        let Some(primary) = forms.first() else { continue };
        let letters = Form::from_print(primary).letters;
        let letters: String = letters.chars().map(|c| match c { 'ѡ' => 'о', 'є' => 'е', other => other }).collect();
        let letters = match refl {
            Some(r) => match letters.strip_suffix(r) {
                // the jer the print dropped before the enclitic returns
                Some(core) if !core.ends_with(|c: char| church_slavonic::orthography::is_vowel_letter(c) || matches!(c, 'ъ' | 'ь' | 'й')) => format!("{core}ъ"),
                Some(core) => core.to_string(),
                None => letters,
            },
            None => letters,
        };
        let Some(alts) = class.cells.get(cell).or_else(|| cell.block().and_then(|b| class.blocks.get(&b))) else { continue };
        // an alternative of the class already produces the form: no vote
        let produced = alts.iter().any(|alt| match &alt.shape {
            Shape::Ending { stem, ending, .. } => derived.get(stem).is_some_and(|d| format!("{d}{ending}") == letters),
            _ => false,
        });
        if produced {
            continue;
        }
        for alt in alts {
            // never a whole form as a stem (a cell with an empty ending
            // would make the form its own stem: the census's `artefact`)
            if let Shape::Ending { stem, ending, .. } = &alt.shape
                && !ending.is_empty()
                && let Some(candidate) = letters.strip_suffix(ending.as_str())
                && !candidate.is_empty()
                && derived.get(stem).is_some_and(|d| d != candidate)
            {
                *votes.entry(*stem).or_default().entry(candidate.to_string()).or_default() += 1;
                break;
            }
        }
    }
    let mut out = Vec::new();
    for (stem, candidates) in votes {
        if let Some((candidate, n)) = candidates.into_iter().max_by_key(|(_, n)| *n)
            && n >= 2
        {
            out.push((stem.to_string(), candidate));
        }
    }
    out.sort();
    out
}

/// The cell whose form is the lemma, per part of speech and class.
fn lemma_cell(pos: Pos, class: &church_slavonic::paradigm::Class) -> Option<Cell> {
    match pos {
        Pos::Noun => Cell::parse(pos, "nom.sg"),
        Pos::Adjective => Cell::parse(pos, if class.strip >= 2 { "long.pos.m.sg.nom" } else { "short.pos.m.sg.nom" }),
        Pos::Verb => Some(Cell::Verb(VerbCell::Infinitive)),
        Pos::Pronoun if class.name.starts_with("PA") => Cell::parse(pos, "m.sg.nom"),
        _ => None,
    }
}

/// A verb's reflexive suffix (`-сѧ`), written solid after every ending.
fn reflexive_suffix(pos: Pos, lemma_letters: &str) -> Option<String> {
    match pos {
        Pos::Verb => (lemma_letters.ends_with("сѧ") && lemma_letters.chars().count() > 4).then(|| "сѧ".to_string()),
        // the compound adjectives: an enclitic after the long ending
        // (пе́рвыйнадесѧть, кото́рыйждо, каковы́йлибо)
        Pos::Adjective | Pos::Pronoun => ["надесѧть", "либо", "жде", "ждо", "же", "то"]
            .into_iter()
            .find(|e| lemma_letters.strip_suffix(e).is_some_and(|core| core.ends_with('й') && core.chars().count() > 2))
            .map(str::to_string),
        _ => None,
    }
}

/// Print one lexeme's fit in full (`--debug <lemma>`).
pub fn debug(pos: Pos, wanted: &str) -> Result<(), Box<dyn Error>> {
    let path = super::intermediate_dir().join("polyakov.jsonl");
    let entries = polyakov::read(&path)?;
    let mut o = Outcome::default();
    let classes = table(pos);
    for entry in entries.iter().filter(|e| pos_of(e) == Some(pos)) {
        let lemma = realise(&entry.lemma, &SYN);
        if Form::from_print(&lemma).key() != Form::from_print(wanted).key() {
            continue;
        }
        println!("== {} {:?} class {}", lemma, entry.tags, entry.class);
        let lemma_form = Form::from_print(&lemma);
        println!("lemma letters {:?} stress {:?}", lemma_form.letters, lemma_form.stress);
        let (attested, bundled) = attested_cells(entry, pos, &entry.class, &mut o);
        println!("bundled-only cells: {:?}", bundled.iter().map(|c| c.name()).collect::<Vec<_>>());
        for code in class_codes(&entry.class) {
            let Some(class) = classes.get(&code) else { println!("class {code} unknown"); continue };
            let animate = entry.tags.iter().find_map(|t| match t.as_str() { "anim" => Some(true), "inan" => Some(false), _ => None });
            let refl = reflexive_suffix(pos, &lemma_form.letters);
            let mut stems: Vec<(String, String)> = inserted_stem(class, &lemma_form.letters, &attested).into_iter().collect();
            if let Some(r) = &refl {
                stems.push(("encl".to_string(), r.clone()));
            }
            let f = fit("x", &lemma, pos, SYN, class, None, animate, stems, &attested, &bundled, vec![], String::new());
            println!("class {code}: stress {} reproduced {}/{}", f.lexeme.stress, f.reproduced, f.attested);
            for (cell, forms) in &attested {
                let predicted = f.lexeme.inflect(*cell).map(|x| x.print(SYN)).unwrap_or_default();
                let ev = f.evidence.get(cell);
                println!("  {:32} attested {:?} predicted {} evidence {:?}", cell.name(), forms, predicted, ev);
            }
        }
    }
    Ok(())
}

/// Import every Polyakov entry of `pos`.
pub fn import(pos: Pos) -> Result<Outcome, Box<dyn Error>> {
    let path = super::intermediate_dir().join("polyakov.jsonl");
    let entries = polyakov::read(&path)?;
    let mut o = Outcome::default();
    let classes = table(pos);
    let mut ids: HashMap<String, u32> = HashMap::new();
    for entry in &entries {
        if pos_of(entry) != Some(pos) {
            continue;
        }
        o.bump("entries");
        let mut lemma = realise(&entry.lemma, &SYN);
        let mut lemma_form = Form::from_print(&lemma);
        let headword = lemma.clone();
        let quarantine = |o: &mut Outcome, reason: &'static str, detail: String| {
            o.quarantine.push(Quarantined {
                recension: SYN,
                pos,
                lemma: headword.clone(),
                source: format!("P:{}", entry.class),
                reason,
                detail,
            });
        };
        if lemma.contains(' ') {
            quarantine(&mut o, "lemma is more than one word", String::new());
            continue;
        }
        let gender = entry.tags.iter().find_map(|t| match t.as_str() {
            "m" => Some(Gender::Masculine),
            "f" => Some(Gender::Feminine),
            "n" => Some(Gender::Neuter),
            _ => None,
        });
        let animate = entry.tags.iter().find_map(|t| match t.as_str() {
            "anim" => Some(true),
            "inan" => Some(false),
            _ => None,
        });
        let mut notes: Vec<String> = Vec::new();
        for t in &entry.tags {
            match t.as_str() {
                "pl" => notes.push("pl-tantum".into()),
                "m/f" => notes.push("gender m/f".into()),
                "anim/inan" => notes.push("anim/inan".into()),
                "persn" | "topn" | "famn" | "patrn" | "poss" | "pf" | "ipf" | "tran" | "intr" | "med" | "comp" => notes.push(t.clone()),
                other if pos == Pos::Closed && other.chars().all(|c| c.is_ascii_uppercase()) => notes.push(other.to_lowercase()),
                _ => {}
            }
        }
        let mut id_for = |lemma_form: &Form| {
            let bare = lexeme_stem(lemma_form);
            let n = ids.entry(bare.clone()).or_default();
            *n += 1;
            if *n == 1 { format!("{bare}.{}", pos.tag()) } else { format!("{bare}.{}.{n}", pos.tag()) }
        };
        let mut id = id_for(&lemma_form);
        let src = vec![format!("P:{}", if entry.class.is_empty() { "-" } else { &entry.class })];
        // the closed classes: one form, the rest variants
        if pos == Pos::Closed {
            let (attested, _) = attested_cells(entry, pos, "", &mut o);
            let forms = attested.get(&Cell::Word).cloned().unwrap_or_default();
            let primary = forms.first().cloned().unwrap_or_else(|| lemma.clone());
            let lemma_print = Form::from_print(&primary).print(SYN);
            let variants: Vec<String> = forms.iter().skip(1).map(|f| Form::from_print(f).print(SYN)).filter(|f| *f != lemma_print).collect();
            // the id follows the lemma as written (the primary form may
            // spell the headword differently: безѻпа́снѡ)
            let id = {
                let bare = lexeme_stem(&Form::from_print(&lemma_print));
                let n = ids.entry(bare.clone()).or_default();
                *n += 1;
                if *n == 1 { format!("{bare}.{}", pos.tag()) } else { format!("{bare}.{}.{n}", pos.tag()) }
            };
            o.lexemes.push(church_slavonic::Lexeme {
                id,
                lemma: lemma_print,
                pos,
                gender: None,
                animate: None,
                class: "0".to_string(),
                stress: if lemma_form.stress.is_some() { "a".to_string() } else { String::new() },
                stems: Vec::new(),
                overrides: Vec::new(),
                variants: if variants.is_empty() { Vec::new() } else { vec![(Cell::Word, variants)] },
                src,
                note: notes.join("; "),
                provenance: church_slavonic::Provenance::Attested,
                recension: SYN,
            });
            continue;
        }
        let codes = class_codes(&entry.class);
        if codes.is_empty() {
            quarantine(&mut o, "no class in the source", String::new());
            continue;
        }
        let expected_prefix = match pos {
            Pos::Noun => "N",
            Pos::Adjective => "A",
            Pos::Verb => "V",
            Pos::Pronoun => "P",
            Pos::Closed => "",
        };
        if !codes.iter().any(|c| c.starts_with(expected_prefix) || c == "0") {
            quarantine(&mut o, "class of another part of speech", entry.class.clone());
            continue;
        }
        let strip_of = |code: &str| classes.get(code).map(|c| c.strip).unwrap_or(1);
        let mut candidates: Vec<String> = Vec::new();
        for code in &codes {
            for c in candidate_classes(code, &lemma_form.letters, strip_of(code)) {
                if !candidates.contains(&c) {
                    candidates.push(c);
                }
            }
        }
        let known: Vec<&church_slavonic::paradigm::Class> = candidates.iter().filter_map(|c| classes.get(c)).collect();
        if known.is_empty() {
            quarantine(&mut o, "class not in the inventory", entry.class.clone());
            continue;
        }
        let (attested, bundled) = attested_cells(entry, pos, &codes[0], &mut o);
        if attested.is_empty() {
            quarantine(&mut o, "no analysed forms", String::new());
            continue;
        }
        let refl = reflexive_suffix(pos, &lemma_form.letters);
        // the cell that is the lemma names it: where the source's headword
        // spells the citation form otherwise (тьма̀ against the attested
        // тма̀), the attested print is the lemma and the headword a note
        let lemma_cell_of = |class: &church_slavonic::paradigm::Class| lemma_cell(pos, class);
        if let Some(cell) = lemma_cell_of(known[0])
            && let Some(first) = attested.get(&cell).and_then(|v| v.first())
            && Form::from_print(first).key() != lemma_form.key()
        {
            let refl_kept = reflexive_suffix(pos, &Form::from_print(first).letters).is_some() == reflexive_suffix(pos, &lemma_form.letters).is_some();
            if !refl_kept || first.contains(' ') {
                quarantine(&mut o, "attested citation form differs from the lemma", first.clone());
                continue;
            }
            lemma = super::fit::canonical(first);
            lemma_form = Form::from_print(&lemma);
            id = id_for(&lemma_form);
            notes.push(format!("headword {headword}"));
            o.bump("lexemes: the attested citation form replaces the headword");
        }
        let plurale_tantum = pos == Pos::Noun
            && (entry.tags.iter().any(|t| t == "pl")
                || (matches!(lemma_form.letters.chars().last(), Some('ы' | 'и'))
                    && known.iter().all(|c| {
                        let subject = church_slavonic::paradigm::Subject { lemma: &lemma_form.letters, animate, stems: &[] };
                        lemma_cell_of(c).and_then(|cell| c.letters(cell, &subject).into_iter().next()).is_none_or(|l| Form::new(l.letters.clone(), None, false).key() != lemma_form.key())
                    })));
        if plurale_tantum && !notes.iter().any(|n| n == "pl-tantum") {
            notes.push("pl-tantum".into());
        }
        let mut best: Option<super::fit::Fit> = None;
        let consider = |f: super::fit::Fit, best: &mut Option<super::fit::Fit>| {
            if best.as_ref().is_none_or(|b| f.reproduced > b.reproduced) {
                *best = Some(f);
            }
        };
        for class in &known {
            let mut stems: Vec<(String, String)> = inserted_stem(class, &lemma_form.letters, &attested).into_iter().collect();
            if let Some(r) = &refl {
                stems.push(("encl".to_string(), r.clone()));
            }
            if plurale_tantum {
                let n = lemma_form.letters.chars().count().saturating_sub(1);
                stems.push(("base".to_string(), lemma_form.letters.chars().take(n).collect()));
            }
            let f = fit(&id, &lemma, pos, SYN, class, gender, animate, stems.clone(), &attested, &bundled, src.clone(), notes.join("; "));
            consider(f, &mut best);
            // stems read off the attested forms, kept when they fit better
            {
                let subject = church_slavonic::paradigm::Subject { lemma: &lemma_form.letters, animate, stems: &stems };
                let inferred = inferred_stems(class, &subject, &attested, refl.as_deref());
                if !inferred.is_empty() {
                    let mut stems3 = stems.clone();
                    stems3.extend(inferred);
                    let f3 = fit(&id, &lemma, pos, SYN, class, gender, animate, stems3, &attested, &bundled, src.clone(), notes.join("; "));
                    consider(f3, &mut best);
                }
            }
            // a lemma with a wide letter may keep it in the citation form only
            if lemma_form.letters.contains(['ѡ', 'є']) {
                let n = lemma_form.letters.chars().count().saturating_sub(class.strip);
                let base: String = lemma_form.letters.chars().take(n).collect();
                let narrow = narrowed(&base);
                if narrow != base {
                    let mut stems2 = stems.clone();
                    stems2.retain(|(k, _)| k != "base");
                    stems2.push(("base".to_string(), narrow));
                    let f2 = fit(&id, &lemma, pos, SYN, class, gender, animate, stems2, &attested, &bundled, src.clone(), notes.join("; "));
                    consider(f2, &mut best);
                }
            }
        }
        let Some(f) = best else { continue };
        if let Some(cell) = lemma_cell_of(f.lexeme.class().unwrap_or(known[0])) {
            match f.lexeme.inflect(cell) {
                None => {
                    quarantine(&mut o, "class declares no citation cell", f.lexeme.class.clone());
                    continue;
                }
                Some(form) if !plurale_tantum && form.key() != lemma_form.key() => {
                    quarantine(&mut o, "class does not produce the lemma", form.print(SYN));
                    continue;
                }
                _ => {}
            }
        }
        *o.counts.entry("cells attested").or_default() += f.attested as u64;
        *o.counts.entry("cells reproduced").or_default() += f.reproduced as u64;
        *o.counts.entry("cells reachable (any alternative/variant)").or_default() += f.reachable as u64;
        *o.counts.entry("cells: true exceptions (no alternative fits)").or_default() += f.exceptions as u64;
        if f.exceptions > 0 {
            o.bump("lexemes with a true exception");
        }
        *o.counts.entry("cells: letter miss").or_default() += f.letter_misses.len() as u64;
        *o.counts.entry("cells: stress miss").or_default() += f.stress_misses.len() as u64;
        for (cell, alt) in &f.alt_matches {
            if let Some((idx, marked)) = alt {
                *o.alt_preference.entry((f.lexeme.class.clone(), cell.name())).or_default().entry(*idx).or_default() += 1;
                if *idx == 0 {
                    let e = o.mark_preference.entry((f.lexeme.class.clone(), cell.name())).or_default();
                    if *marked { e.0 += 1 } else { e.1 += 1 }
                }
                if *idx > 0 && f.lexeme.overrides.iter().any(|(c, _)| c == cell) {
                    o.bump("cells: alternative preference (override names a non-primary alternative)");
                }
            }
        }
        for cell in &f.letter_misses {
            *o.letter_misses.entry((f.lexeme.class.clone(), cell.name())).or_default() += 1;
        }
        for (cell, attested_form) in &f.lexeme.overrides {
            *o.override_cells.entry(cell.name()).or_default() += 1;
            let alt_fits = f.alt_matches.iter().any(|(c, a)| c == cell && a.is_some());
            let mut bare = f.lexeme.clone();
            bare.overrides.retain(|(c, _)| c != cell);
            let reachable = bare.forms(*cell).iter().any(|x| super::fit::translit_equal(&x.print(SYN), attested_form));
            if !reachable {
                let predicted = bare.inflect(*cell).map(|x| x.print(SYN)).unwrap_or_default();
                o.exception_samples.push((f.lexeme.lemma.clone(), f.lexeme.class.clone(), f.lexeme.stress.clone(), cell.name(), attested_form.clone(), predicted));
            } else if alt_fits && f.stress_misses.contains(cell) {
                let predicted = bare.inflect(*cell).map(|x| x.print(SYN)).unwrap_or_default();
                o.stress_miss_samples.push((f.lexeme.lemma.clone(), f.lexeme.stress.clone(), cell.name(), attested_form.clone(), predicted));
            }
        }
        *o.stress_specs.entry(f.lexeme.stress.clone()).or_default() += 1;
        let base = if f.lexeme.stress.starts_with('b') { "b" } else { "a" };
        for (cell, e) in &f.evidence {
            let entry = o.stress_cells.entry((base.to_string(), cell.name())).or_default();
            match e {
                super::fit::Evidence::Stem => entry.0 += 1,
                super::fit::Evidence::End => entry.1 += 1,
                _ => {}
            }
        }
        o.lexemes.push(f.lexeme);
    }
    o.lexemes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(o)
}

/// The id's stem: the lemma's letters with marks stripped.
fn lexeme_stem(lemma: &Form) -> String {
    lemma.letters.clone()
}

/// The noun import (the Part 1 entry point, kept for the floor test).
pub fn import_nouns() -> Result<Outcome, Box<dyn Error>> {
    import(Pos::Noun)
}
