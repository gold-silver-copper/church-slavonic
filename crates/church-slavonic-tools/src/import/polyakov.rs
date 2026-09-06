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
use church_slavonic::orthography::{comparison_key, is_accented, realise};
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

/// The prepositions' case frames as the grammar states them (Alypy,
/// the chapter on the preposition; the standard Church Slavonic
/// inventory), by the preposition's bare letters. The treebank census
/// (`data/prep-frames.tsv`) orders the cases by attestation and adds, in
/// the note, a case the grammar does not know but the print attests.
const GRAMMAR_FRAMES: &[(&str, &[Case])] = &[
    ("безъ", &[Case::Genitive]),
    ("без", &[Case::Genitive]),
    ("близъ", &[Case::Genitive]),
    ("въ", &[Case::Accusative, Case::Locative]),
    ("во", &[Case::Accusative, Case::Locative]),
    ("возлѣ", &[Case::Genitive]),
    ("возъ", &[Case::Accusative]),
    ("вмѣстѡ", &[Case::Genitive]),
    ("вмѣсто", &[Case::Genitive]),
    ("длѧ", &[Case::Genitive]),
    ("до", &[Case::Genitive]),
    ("дѣлѧ", &[Case::Genitive]),
    ("за", &[Case::Accusative, Case::Instrumental]),
    ("изъ", &[Case::Genitive]),
    ("из", &[Case::Genitive]),
    ("кромѣ", &[Case::Genitive]),
    ("къ", &[Case::Dative]),
    ("ко", &[Case::Dative]),
    ("междꙋ", &[Case::Instrumental, Case::Genitive]),
    ("на", &[Case::Accusative, Case::Locative]),
    ("надъ", &[Case::Instrumental, Case::Accusative]),
    ("над", &[Case::Instrumental, Case::Accusative]),
    ("наподобіе", &[Case::Genitive]),
    ("ѡ", &[Case::Locative, Case::Accusative]),
    ("о", &[Case::Locative, Case::Accusative]),
    ("ѡбъ", &[Case::Accusative, Case::Locative]),
    ("объ", &[Case::Accusative, Case::Locative]),
    ("ѡколѡ", &[Case::Genitive]),
    ("около", &[Case::Genitive]),
    ("оу", &[Case::Genitive]),
    ("ꙋ", &[Case::Genitive]),
    ("ѿ", &[Case::Genitive]),
    ("от", &[Case::Genitive]),
    ("отъ", &[Case::Genitive]),
    ("по", &[Case::Dative, Case::Locative, Case::Accusative]),
    ("подъ", &[Case::Instrumental, Case::Accusative]),
    ("под", &[Case::Instrumental, Case::Accusative]),
    ("предъ", &[Case::Instrumental, Case::Accusative]),
    ("пред", &[Case::Instrumental, Case::Accusative]),
    ("при", &[Case::Locative]),
    ("ради", &[Case::Genitive]),
    ("сквозѣ", &[Case::Accusative]),
    ("съ", &[Case::Instrumental, Case::Genitive, Case::Accusative]),
    ("со", &[Case::Instrumental, Case::Genitive, Case::Accusative]),
    ("чрезъ", &[Case::Accusative]),
    ("чрез", &[Case::Accusative]),
];

/// The treebank's frames (`data/prep-frames.tsv`): bare letters →
/// (case, unambiguous count), commonest first.
fn treebank_frames() -> HashMap<String, Vec<(Case, u64)>> {
    let path = crate::census::closed::frames_path();
    let mut out = HashMap::new();
    let Ok(text) = std::fs::read_to_string(&path) else { return out };
    for line in text.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 3 {
            continue;
        }
        let cases: Vec<(Case, u64)> = cols[2]
            .split_whitespace()
            .filter_map(|item| {
                let (c, n) = item.split_once(':')?;
                Some((church_slavonic::cell::parse_case(c)?, n.parse().ok()?))
            })
            .collect();
        out.insert(cols[0].to_string(), cases);
    }
    out
}

/// A preposition's `gov=` value and the note for the cases the print
/// attests beyond the grammar: the grammar's cases ordered by the
/// treebank's counts, then any treebank case with at least a twentieth
/// of the unambiguous tokens as a variant frame in the note.
fn government(letters: &str, frames: &HashMap<String, Vec<(Case, u64)>>) -> (Vec<Case>, Option<String>) {
    let key = church_slavonic::orthography::comparison_key(letters);
    let grammar: Vec<Case> = GRAMMAR_FRAMES
        .iter()
        .filter(|(w, _)| church_slavonic::orthography::comparison_key(w) == key)
        .flat_map(|(_, cases)| cases.iter().copied())
        .collect();
    let attested = frames.get(&key).cloned().unwrap_or_default();
    let total: u64 = attested.iter().map(|(_, n)| *n).sum();
    let mut cases: Vec<Case> = attested.iter().map(|(c, _)| *c).filter(|c| grammar.contains(c)).collect();
    for c in &grammar {
        if !cases.contains(c) {
            cases.push(*c);
        }
    }
    let extra: Vec<String> = attested
        .iter()
        .filter(|(c, n)| !grammar.contains(c) && *n * 20 >= total && *n >= 5)
        .map(|(c, n)| format!("{}:{n}", church_slavonic::cell::case_name(*c)))
        .collect();
    let note = (!extra.is_empty()).then(|| format!("gov? {}", extra.join(" ")));
    (cases, note)
}

/// The closed subcategories with their prosody: the enclitics lean on the
/// word before them, the prepositions and the negations on the word after.
fn prosody_of(letters: &str, subcategory: &str) -> Option<&'static str> {
    let key = church_slavonic::orthography::comparison_key(letters);
    if ["же", "бо", "ли", "ꙋбо", "оубо"].iter().any(|w| church_slavonic::orthography::comparison_key(w) == key) {
        return Some("encl");
    }
    if subcategory == "prep" || ["не", "ни"].iter().any(|w| church_slavonic::orthography::comparison_key(w) == key) {
        return Some("procl");
    }
    None
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
        // the numerals (3.3 Part 2): five upward and the -десѧть compounds
        // decline as nouns (N41, сто̀ N2t); the hundreds the source lists in
        // one form (NUM100: двѣ́сти, три́ста) are closed words; two, both,
        // three and four (NUM2, NUMoba, NUM3, NUM4) are pronoun-class lines
        // by hand (the pronouns are never imported through this source)
        "NUM" if entry.class.starts_with('N') && !entry.class.starts_with("NUM") => Some(Pos::Noun),
        "NUM" if entry.class == "NUM100" => Some(Pos::Closed),
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
        let printed = bible_spelling(realise(&form.form, &SYN), &comparison_key(&realise(&entry.lemma, &SYN)));
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
                let mut f = features(&set);
                // a numeral's tags name the case alone (пѧ́ть: nom/acc, пѧтѝ:
                // gen/dat/loc): the noun's singular; the plural-shaped
                // пѧти́хъ, пѧти́мъ the class writes for itself (3.3 Part 2)
                if pos == Pos::Noun && f.number.is_none() && entry.tags.first().map(String::as_str) == Some("NUM") {
                    f.number = Some(Number::Singular);
                }
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
    // where the source's forms disagree the print decides: the form the
    // pinned Bible prints most (the treebank's counts, `census forms
    // --write`) is the primary, Polyakov's count next
    let bible = bible_counts();
    let lemma_key = comparison_key(&realise(&entry.lemma, &SYN));
    // never the citation cell: the lemma and the id follow the headword
    let citation = table(pos).get(class).and_then(|c| lemma_cell(pos, c));
    let attested = counts
        .into_iter()
        .map(|(cell, forms)| {
            let any_unbundled = forms.values().any(|(u, _)| *u > 0);
            // the print arbitrates the STRESS of a form (ѻ҆́вцꙋ against
            // ѻ҆вцꙋ̀): only forms with the letters of the source's own first
            // choice take a Bible count; a letter variant keeps its place
            let source_first = forms
                .iter()
                .max_by_key(|(f, (u, b))| (!any_unbundled || *u > 0, u + b, std::cmp::Reverse((*f).clone())))
                .map(|(f, _)| comparison_key(f))
                .unwrap_or_default();
            // the Bible's one-cell count of each stress twin; the print
            // arbitrates only among forms it never prints inside a set
            // leaf — a form that is syncretic anywhere (жєны̀ beside жены̑,
            // дре́ва beside древа̀) has tokens no one-cell count sees
            let bible_of = |f: &str| -> (u64, u64) {
                if Some(cell) == citation || comparison_key(f) != source_first {
                    return (0, 0);
                }
                bible.get(&(lemma_key.clone(), pos, cell.name(), super::fit::canonical(f))).copied().unwrap_or((0, 0))
            };
            let arbitrated = forms.iter().all(|(f, _)| bible_of(f).1 == 0);
            let mut v: Vec<(String, u64, u64, u64)> = forms
                .into_iter()
                .map(|(f, (u, b))| {
                    let printed = if arbitrated { bible_of(&f).0 } else { 0 };
                    (f, u, b, printed)
                })
                .collect();
            v.sort_by(|a, b| {
                let ka = (any_unbundled && a.1 == 0, std::cmp::Reverse(a.3), std::cmp::Reverse(a.1 + a.2));
                let kb = (any_unbundled && b.1 == 0, std::cmp::Reverse(b.3), std::cmp::Reverse(b.1 + b.2));
                ka.cmp(&kb).then(a.0.cmp(&b.0))
            });
            (cell, v.into_iter().map(|(f, u, b, _)| (f, u + b)).collect())
        })
        .collect();
    (attested, bundled)
}

/// The Bible's counts per (lemma key, pos, cell name, print key), from
/// `data/treebank-forms.tsv` when it exists (`cargo xtask census forms
/// --write`); empty otherwise.
/// (lemma key, pos, cell name, print) → (one-cell count, set count).
type BibleCounts = HashMap<(String, Pos, String, String), (u64, u64)>;

fn bible_counts() -> &'static BibleCounts {
    static COUNTS: std::sync::OnceLock<BibleCounts> = std::sync::OnceLock::new();
    COUNTS.get_or_init(|| {
        let mut out: BibleCounts = HashMap::new();
        let path = crate::workspace_root().join("data/treebank-forms.tsv");
        let Ok(text) = std::fs::read_to_string(path) else { return out };
        for line in text.lines().skip(1) {
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 6 {
                continue;
            }
            let Some(pos) = Pos::parse(cols[1]) else { continue };
            let (Ok(n), Ok(sets)) = (cols[4].parse::<u64>(), cols[5].parse::<u64>()) else { continue };
            let e = out.entry((comparison_key(cols[0]), pos, cols[2].to_string(), super::fit::canonical(cols[3]))).or_default();
            e.0 += n;
            e.1 += sets;
        }
        out
    })
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
        .and_then(|(_, v)| v.first())
        .map(|(f, _)| f)?;
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
        let Some((primary, _)) = forms.first() else { continue };
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
        Pos::Noun => Cell::parse(pos, "nom.sg").ok(),
        Pos::Adjective => Cell::parse(pos, if class.strip >= 2 { "long.pos.m.sg.nom" } else { "short.pos.m.sg.nom" }).ok(),
        Pos::Verb => Some(Cell::Verb(VerbCell::Infinitive)),
        Pos::Pronoun if class.name.starts_with("PA") => Cell::parse(pos, "m.sg.nom").ok(),
        _ => None,
    }
}

/// A verb's reflexive suffix (`-сѧ`), written solid after every ending;
/// a compound's enclitic (кото́рыйждо, каковы́йлибо) or stressed tail
/// (первыйна́десѧть: the stems entry is `tail=на́десѧть`, the tail's own
/// stress with it). Returns the stems key and the letters after the
/// first element.
fn reflexive_suffix(pos: Pos, lemma_letters: &str) -> Option<(String, String)> {
    match pos {
        Pos::Verb => (lemma_letters.ends_with("сѧ") && lemma_letters.chars().count() > 4).then(|| ("encl".to_string(), "сѧ".to_string())),
        Pos::Adjective | Pos::Pronoun => ["надесѧть", "либо", "жде", "ждо", "же", "то"]
            .into_iter()
            .find(|e| lemma_letters.strip_suffix(e).is_some_and(|core| core.ends_with('й') && core.chars().count() > 2))
            .map(|e| if e == "надесѧть" { ("tail".to_string(), "на́десѧть".to_string()) } else { ("encl".to_string(), e.to_string()) }),
        _ => None,
    }
}

/// The letters a suffix adds to the first element (the tail's, without
/// its stress mark).
fn suffix_letters(suffix: &(String, String)) -> String {
    Form::from_print(&suffix.1).letters
}

/// Print one lexeme's fit in full (`--debug <lemma>`).
pub fn debug(pos: Pos, wanted: &str) -> Result<(), Box<dyn Error>> {
    let path = super::intermediate_dir().join("polyakov.jsonl");
    let entries = polyakov::read(&path)?;
    let mut o = Outcome::default();
    let classes = table(pos);
    for entry in entries.iter().filter(|e| pos_of(e) == Some(pos)) {
        let lemma = bible_spelling(realise(&entry.lemma, &SYN), &comparison_key(&realise(&entry.lemma, &SYN)));
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
                stems.push(r.clone());
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
    let frames = treebank_frames();
    // the adverbs an adjective of the lexicon produces (its `adv` cell):
    // print → adjective id. A closed ADV entry printed so is not a lexeme
    // of its own (2.2 Part 2); an adjective takes the entry's count as
    // the provenance of its adverb.
    let lexicon = church_slavonic::Lexicon::synodal();
    let mut adverb_of: HashMap<String, String> = HashMap::new();
    // by the accent-blind key too: an adverb an adjective produces with
    // another accent or letter keeps its line and names the adjective
    let mut adverb_key_of: HashMap<String, String> = HashMap::new();
    if pos == Pos::Closed || pos == Pos::Adjective {
        for adj in lexicon.iter().filter(|l| l.pos == Pos::Adjective) {
            if let Ok(cell) = Cell::parse(Pos::Adjective, "adv") {
                for form in adj.forms(cell) {
                    let print = form.print(SYN);
                    adverb_key_of.entry(church_slavonic::orthography::comparison_key(&print)).or_insert_with(|| adj.id.clone());
                    adverb_of.entry(print).or_insert_with(|| adj.id.clone());
                }
            }
        }
    }
    let adverb_entries: HashMap<String, u64> = if pos == Pos::Adjective {
        entries
            .iter()
            .filter(|e| e.tags.first().map(String::as_str) == Some("ADV"))
            .map(|e| (Form::from_print(&bible_spelling(realise(&e.lemma, &SYN), &comparison_key(&realise(&e.lemma, &SYN)))).print(SYN), e.count))
            .collect()
    } else {
        HashMap::new()
    };
    // the same by the accent-blind key: Polyakov's adverb enters the
    // adjective's `adv` cell as an attested form, so the fitter reads
    // its accent (бла́гѡ beside благі́й: `b.adv`) and a true exception
    // becomes the cell's override — the adverb is the adjective's cell,
    // never a line beside it (3.0 Part 1 step 3)
    let mut adverb_by_key: HashMap<String, Vec<(String, u64)>> = HashMap::new();
    for (print, count) in &adverb_entries {
        adverb_by_key.entry(church_slavonic::orthography::comparison_key(print)).or_default().push((print.clone(), *count));
    }
    // 3.1 Part 3: what each fitted lexeme attested, for the twins' merge
    let mut twin_evidence: HashMap<String, (usize, Attested, Bundled)> = HashMap::new();
    for (entry_index, entry) in entries.iter().enumerate() {
        if pos_of(entry) != Some(pos) {
            continue;
        }
        o.bump("entries");
        let mut lemma = bible_spelling(realise(&entry.lemma, &SYN), &comparison_key(&realise(&entry.lemma, &SYN)));
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
        let gender = entry
            .tags
            .iter()
            .find_map(|t| match t.as_str() {
                "m" => Some(Gender::Masculine),
                "f" => Some(Gender::Feminine),
                "n" => Some(Gender::Neuter),
                _ => None,
            })
            // a numeral carries no gender tag: пѧ́ть … де́сѧть are feminine
            // i-stems, сто̀ a neuter o-stem
            .or_else(|| match (entry.tags.first().map(String::as_str), entry.class.as_str()) {
                (Some("NUM"), "N41") => Some(Gender::Feminine),
                (Some("NUM"), "N2t") => Some(Gender::Neuter),
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
        // a provisional id: `restore_ids` after the loop gives every fitted
        // lexeme the id the lexicon already holds for its lemma
        let mut id_for = |lemma_form: &Form| {
            let bare = lexeme_stem(lemma_form);
            let n = ids.entry(bare.clone()).or_default();
            *n += 1;
            if *n == 1 { format!("{bare}.{}", pos.tag()) } else { format!("{bare}.{}.{n}", pos.tag()) }
        };
        // a closed entry is numbered once, in its own block below
        let mut id = if pos == Pos::Closed { String::new() } else { id_for(&lemma_form) };
        let src = vec![format!("P:{}", if entry.class.is_empty() { "-" } else { &entry.class })];
        // the closed classes: one form, the rest variants
        if pos == Pos::Closed {
            let (attested, _) = attested_cells(entry, pos, "", &mut o);
            let forms: Vec<String> = attested.get(&Cell::Word).cloned().unwrap_or_default().into_iter().map(|(f, _)| f).collect();
            let primary = forms.first().cloned().unwrap_or_else(|| lemma.clone());
            let lemma_print = Form::from_print(&primary).print(SYN);
            let variants: Vec<String> = forms.iter().skip(1).map(|f| Form::from_print(f).print(SYN)).filter(|f| *f != lemma_print).collect();
            // the subcategory is the class; an adverb an adjective produces
            // is that adjective's cell, not a line
            let subcategory = match entry.tags.first().map(String::as_str) {
                Some("ADV") => "adv",
                Some("ADVPRO") => "advpro",
                Some("CONJ") => "conj",
                Some("PR") => "prep",
                Some("PART") => "part",
                Some("INTJ") => "intj",
                Some("PRED") => "pred",
                Some("NUM") => "num",
                _ => "adv",
            };
            if subcategory == "adv"
                && let Some(adj) = adverb_of.get(&lemma_print)
            {
                o.bump("adverbs an adjective produces (no line; the adjective's provenance)");
                let _ = adj;
                continue;
            }
            let mut stems: Vec<(String, String)> = Vec::new();
            let mut extra_notes: Vec<String> = Vec::new();
            if subcategory == "adv"
                && let Some(adj) = adverb_key_of.get(&church_slavonic::orthography::comparison_key(&lemma_print))
            {
                stems.push(("adv-of".to_string(), adj.clone()));
                extra_notes.push("the adjective's adverb printed with another accent or letter".to_string());
                o.bump("adverbs of an adjective printed differently (line kept, adv-of=)");
            }
            if subcategory == "prep" {
                let (cases, note) = government(&lemma_form.letters, &frames);
                if !cases.is_empty() {
                    stems.push(("gov".to_string(), cases.iter().map(|c| church_slavonic::cell::case_name(*c)).collect::<Vec<_>>().join("|")));
                    o.bump("prepositions with a case frame");
                } else {
                    o.bump("prepositions without a case frame");
                }
                if let Some(n) = note {
                    extra_notes.push(n);
                }
            }
            if let Some(p) = prosody_of(&lemma_form.letters, subcategory) {
                stems.push(("pros".to_string(), p.to_string()));
            }
            // the id follows the lemma as written (the primary form may
            // spell the headword differently: безѻпа́снѡ)
            let id = {
                let letters = Form::from_print(&lemma_print).letters;
                let bare = church_slavonic::orthography::id_stem(&letters);
                let n = ids.entry(bare.clone()).or_default();
                *n += 1;
                if *n == 1 { format!("{bare}.{}", pos.tag()) } else { format!("{bare}.{}.{n}", pos.tag()) }
            };
            twin_evidence.insert(id.clone(), (entry_index, Attested::new(), Bundled::new()));
            o.lexemes.push(church_slavonic::Lexeme {
                id,
                lemma: lemma_print,
                pos,
                gender: None,
                animate: None,
                class: subcategory.to_string(),
                stress: if lemma_form.stress.is_some() { "a".to_string() } else { String::new() },
                stems,
                overrides: Vec::new(),
                variants: if variants.is_empty() { Vec::new() } else { vec![(Cell::Word, variants)] },
                src,
                note: notes.iter().chain(extra_notes.iter()).cloned().collect::<Vec<_>>().join("; "),
                variant_weights: Vec::new(),
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
        let (mut attested, bundled) = attested_cells(entry, pos, &codes[0], &mut o);
        if attested.is_empty() {
            quarantine(&mut o, "no analysed forms", String::new());
            continue;
        }
        if pos == Pos::Adjective
            && let Ok(adv_cell) = Cell::parse(Pos::Adjective, "adv")
            && !attested.contains_key(&adv_cell)
        {
            let subject = church_slavonic::paradigm::Subject { lemma: &lemma_form.letters, animate: None, stems: &[] };
            let mut found: Vec<(String, u64)> = Vec::new();
            for class in &known {
                for letters in class.letters(adv_cell, &subject) {
                    let key = church_slavonic::orthography::comparison_key(&letters.letters);
                    for (print, count) in adverb_by_key.get(&key).map(Vec::as_slice).unwrap_or(&[]) {
                        if !found.iter().any(|(p, _)| p == print) {
                            found.push((print.clone(), *count));
                        }
                    }
                }
            }
            if !found.is_empty() {
                found.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
                attested.insert(adv_cell, found);
                o.bump("adjectives whose adv cell Polyakov's adverb entry attests");
            }
        }
        let refl = reflexive_suffix(pos, &lemma_form.letters);
        // the cell that is the lemma names it: where the source's headword
        // spells the citation form otherwise (тьма̀ against the attested
        // тма̀), the attested print is the lemma and the headword a note
        let lemma_cell_of = |class: &church_slavonic::paradigm::Class| lemma_cell(pos, class);
        if let Some(cell) = lemma_cell_of(known[0])
            && let Some((first, _)) = attested.get(&cell).and_then(|v| v.first())
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
        if pos == Pos::Adjective
            && let Ok(cell) = Cell::parse(Pos::Adjective, "adv")
            && let Some(existing) = lexicon.get(&id)
        {
            // Polyakov's ADV entry printed as this adjective's adverb: its
            // count is the cell's provenance (the closed line is gone)
            for form in existing.forms(cell) {
                if let Some(n) = adverb_entries.get(&form.print(SYN)) {
                    notes.push(format!("adv P:{n}"));
                    o.bump("adjectives whose adverb Polyakov attests");
                    break;
                }
            }
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
                stems.push(r.clone());
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
                let inferred = inferred_stems(class, &subject, &attested, refl.as_ref().map(suffix_letters).as_deref());
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
                Err(_) => {
                    quarantine(&mut o, "class declares no citation cell", f.lexeme.class.clone());
                    continue;
                }
                Ok(form) if !plurale_tantum && form.key() != lemma_form.key() => {
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
        twin_evidence.insert(f.lexeme.id.clone(), (entry_index, attested.clone(), bundled.clone()));
        o.lexemes.push(f.lexeme);
    }
    restore_ids(&mut o, lexicon, pos, &mut twin_evidence);
    if pos != Pos::Closed {
        merge_twins(&mut o, pos, &entries, &twin_evidence);
    }
    o.lexemes.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(o)
}

/// 3.1 Part 3 — the rule of identity. Polyakov splits one word into
/// entries by sense (ꙗзыкъ tongue / nation, зрѣти twice); the lexicon
/// holds one lexeme per line. Two fitted lexemes are one lexeme when they
/// share the lemma under the accent-blind key, the part of speech, the
/// gender and the animacy, and every cell both attest prints the same
/// primary (a proper subset of attestations is the same lexeme with fewer
/// forms; disjoint attestations say nothing against it). The survivor is
/// the lowest id (the one a consumer may hold); it is refitted once from
/// the union of both entries' forms (the arbiter sees the union), its
/// provenance and notes joined, and the absorbed id recorded in the note
/// (`twin: x.n.2`). A pair whose shared cells disagree (во́лна / волна̀,
/// ꙗзыкъ anim / inan by animacy) stays two lines. Ids never move: the
/// numbering was assigned before the merge.
/// The identity key of a lexeme for the twins' merge: the accent-blind
/// lemma, the part of speech, the gender and the animacy.
type TwinKey = (String, Pos, Option<Gender>, Option<bool>);

fn merge_twins(o: &mut Outcome, pos: Pos, entries: &[Entry], evidence: &HashMap<String, (usize, Attested, Bundled)>) {
    let classes = table(pos);
    let mut groups: BTreeMap<TwinKey, Vec<usize>> = BTreeMap::new();
    for (i, l) in o.lexemes.iter().enumerate() {
        groups.entry((comparison_key(&l.lemma), l.pos, l.gender, l.animate)).or_default().push(i);
    }
    let mut absorbed: Vec<usize> = Vec::new();
    let mut absorbed_by: HashMap<usize, Vec<usize>> = HashMap::new();
    let mut replacements: Vec<(usize, church_slavonic::Lexeme)> = Vec::new();
    for members in groups.values_mut() {
        if members.len() < 2 {
            continue;
        }
        members.sort_by(|a, b| o.lexemes[*a].id.cmp(&o.lexemes[*b].id));
        let mut survivors: Vec<(usize, Entry)> = Vec::new();
        'member: for &i in members.iter() {
            let Some((entry_index, attested, _)) = evidence.get(&o.lexemes[i].id) else { continue };
            for (si, merged_entry) in survivors.iter_mut() {
                let Some((_, sattested, _)) = evidence.get(&o.lexemes[*si].id) else { continue };
                let shared: Vec<&Cell> = attested.keys().filter(|c| sattested.contains_key(*c)).collect();
                let agree = shared.iter().all(|c| match (attested[*c].first(), sattested[*c].first()) {
                    (Some((a, _)), Some((b, _))) => super::fit::translit_equal(a, b),
                    _ => true,
                });
                if !agree {
                    o.bump("twins kept apart: a shared cell prints differently");
                    continue;
                }
                let kind = if shared.len() == attested.len() && shared.len() == sattested.len() {
                    "twins merged: identical attestations"
                } else if shared.len() == attested.len() || shared.len() == sattested.len() {
                    "twins merged: one side's attestations inside the other's"
                } else if shared.is_empty() {
                    "twins merged: disjoint attestations"
                } else {
                    "twins merged: overlapping attestations that agree"
                };
                o.bump(kind);
                merged_entry.forms.extend(entries[*entry_index].forms.iter().cloned());
                merged_entry.count += entries[*entry_index].count;
                for code in class_codes(&entries[*entry_index].class) {
                    if !class_codes(&merged_entry.class).contains(&code) {
                        merged_entry.class = if merged_entry.class.is_empty() { code } else { format!("{}/{code}", merged_entry.class) };
                    }
                }
                absorbed.push(i);
                absorbed_by.entry(*si).or_default().push(i);
                println!("TWIN {} → {}", o.lexemes[i].id, o.lexemes[*si].id);
                continue 'member;
            }
            survivors.push((i, entries[*entry_index].clone()));
        }
        // refit each survivor that absorbed anything from the union
        for (si, merged_entry) in survivors {
            let Some(taken) = absorbed_by.get(&si).cloned() else { continue };
            let twins: Vec<String> = taken.iter().map(|a| o.lexemes[*a].id.clone()).collect();
            let survivor = &o.lexemes[si];
            let Some(class) = classes.get(&survivor.class) else { continue };
            let mut scratch = Outcome::default();
            let (attested, bundled) = attested_cells(&merged_entry, pos, &survivor.class, &mut scratch);
            let mut src = survivor.src.clone();
            let mut note: Vec<String> = survivor.note.split("; ").filter(|n| !n.is_empty()).map(str::to_string).collect();
            for a in &taken {
                for t in &o.lexemes[*a].src {
                    if !src.contains(t) {
                        src.push(t.clone());
                    }
                }
                for n in o.lexemes[*a].note.split("; ").filter(|n| !n.is_empty()) {
                    if !note.iter().any(|x| x == n) {
                        note.push(n.to_string());
                    }
                }
            }
            for t in &twins {
                note.push(format!("twin: {t}"));
            }
            let f = fit(&survivor.id, &survivor.lemma, pos, SYN, class, survivor.gender, survivor.animate, survivor.stems.clone(), &attested, &bundled, src, note.join("; "));
            replacements.push((si, f.lexeme));
        }
    }
    for (si, lexeme) in replacements {
        o.lexemes[si] = lexeme;
    }
    let mut drop: Vec<usize> = absorbed;
    drop.sort_unstable();
    drop.dedup();
    for i in drop.into_iter().rev() {
        o.lexemes.remove(i);
    }
}

/// Polyakov transcribes the print's ligature ѿ (the prefix от-) as «ѡ҆т»,
/// and writes the same letters for о-т- (ѡ҆трѐ, ѡ҆то́къ); the pinned Bible
/// tells them apart — a lexeme whose Bible prints begin «ѡ҆т» keeps the
/// letters, every other «ѡ҆т-» lemma is the ligature (3.1). The lexeme's
/// letters carry the fact; the typography stage never guesses it.
fn ligature(print: String, lemma_key: &str) -> String {
    static KEEP: std::sync::OnceLock<std::collections::HashSet<String>> = std::sync::OnceLock::new();
    let keep = KEEP.get_or_init(|| {
        bible_counts()
            .keys()
            .filter(|(_, _, _, form)| form.starts_with("ѡ\u{486}т"))
            .map(|(key, _, _, _)| key.clone())
            .collect()
    });
    if keep.contains(lemma_key) {
        return print;
    }
    match print.strip_prefix("ѡ\u{486}т").or_else(|| print.strip_prefix("ѡт")) {
        Some(rest) => format!("ѿ{rest}"),
        None => print,
    }
}

/// Ids never move (3.1). After the loop every fitted lexeme takes the id
/// the lexicon already holds for its lemma: the lexemes of one lookup key
/// (the letters with the initial uk as «оу» and the initial ѿ as «ѡт»),
/// in the order of their source entries, take that key's existing ids in
/// numeric order — a quarantined entry never reaches the list, so it
/// consumes no number, whatever it did when the file was first made; a
/// lexeme beyond the existing ids gets the next free number on the key's
/// stem. The twins' merge then runs on the restored ids.
fn restore_ids(o: &mut Outcome, lexicon: &church_slavonic::Lexicon, pos: Pos, evidence: &mut HashMap<String, (usize, Attested, Bundled)>) {
    let suffix = |id: &str| -> u32 {
        match id.rsplit_once('.') {
            Some((head, n)) if head.matches('.').count() >= 1 && n.parse::<u32>().is_ok() => n.parse().unwrap_or(1),
            _ => 1,
        }
    };
    let mut existing: HashMap<String, Vec<String>> = HashMap::new();
    for l in lexicon.iter().filter(|l| l.pos == pos) {
        existing.entry(id_lookup_key(&Form::from_print(&l.lemma).letters)).or_default().push(l.id.clone());
    }
    // an id the twins' merge absorbed (`data/twins.tsv`) keeps its place in
    // the numbering, so the entries after it do not slide; the merge takes
    // it away again
    if let Ok(text) = std::fs::read_to_string(crate::workspace_root().join("data/twins.tsv")) {
        for line in text.lines().skip(1) {
            let Some((absorbed, _)) = line.split_once('\t') else { continue };
            let stem = id_stem_of(absorbed, pos);
            if !absorbed.ends_with(&format!(".{}", pos.tag())) && !absorbed.contains(&format!(".{}.", pos.tag())) {
                continue;
            }
            let list = existing.entry(id_lookup_key(&stem)).or_default();
            if !list.contains(&absorbed.to_string()) {
                list.push(absorbed.to_string());
            }
        }
    }
    for list in existing.values_mut() {
        list.sort_by_key(|id| (suffix(id), id.clone()));
    }
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, l) in o.lexemes.iter().enumerate() {
        groups.entry(id_lookup_key(&Form::from_print(&l.lemma).letters)).or_default().push(i);
    }
    let mut renamed: Vec<(String, String)> = Vec::new();
    for (key, mut members) in groups {
        members.sort_by_key(|i| evidence.get(&o.lexemes[*i].id).map(|e| e.0).unwrap_or(usize::MAX));
        let held = existing.get(&key).cloned().unwrap_or_default();
        let stem = held.first().map(|id| id_stem_of(id, pos)).unwrap_or_else(|| church_slavonic::orthography::id_stem(&Form::from_print(&o.lexemes[members[0]].lemma).letters));
        let mut next = held.iter().map(|id| suffix(id)).max().unwrap_or(0) + 1;
        for (k, i) in members.iter().enumerate() {
            let id = match held.get(k) {
                Some(id) => id.clone(),
                None => {
                    let id = if next == 1 { format!("{stem}.{}", pos.tag()) } else { format!("{stem}.{}.{next}", pos.tag()) };
                    next += 1;
                    id
                }
            };
            if id != o.lexemes[*i].id {
                renamed.push((o.lexemes[*i].id.clone(), id.clone()));
                o.lexemes[*i].id = id;
            }
        }
    }
    let moved: Vec<(String, (usize, Attested, Bundled))> = renamed.iter().filter_map(|(old, new)| evidence.remove(old).map(|e| (new.clone(), e))).collect();
    for (new, e) in moved {
        evidence.insert(new, e);
    }
    if !o.lexemes.is_empty() {
        o.bump("lexemes whose id the lexicon already held");
    }
}

/// The stem of an id (`зрѣти.v.3` → `зрѣти`).
fn id_stem_of(id: &str, pos: Pos) -> String {
    let tag = format!(".{}", pos.tag());
    match id.rsplit_once('.') {
        Some((head, n)) if n.parse::<u32>().is_ok() && head.ends_with(&tag) => head[..head.len() - tag.len()].to_string(),
        _ => id.strip_suffix(&tag).unwrap_or(id).to_string(),
    }
}

/// The print writes ї in a loanword before a consonant too (кївѡ́тъ,
/// вїно̀, пїла́тъ, галїле́а), where the ї rule of the typography writes і;
/// Polyakov writes і everywhere. The letter is the lexeme's, and the
/// pinned Bible decides it (3.3): the positions where the Bible's
/// commonest print of the lemma has a non-positional ї, applied to every
/// print of the entry that has і there.
fn loanword_iota(print: String, lemma_key: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    static POSITIONS: std::sync::OnceLock<HashMap<String, Vec<usize>>> = std::sync::OnceLock::new();
    let positions = POSITIONS.get_or_init(|| {
        // the commonest verbatim print with a ї per lemma key
        // (`data/loanword-iota.tsv`, `census verbatim --write`)
        let mut best: HashMap<String, (u64, String)> = HashMap::new();
        let path = crate::workspace_root().join("data/loanword-iota.tsv");
        for line in std::fs::read_to_string(path).unwrap_or_default().lines().skip(1) {
            let cols: Vec<&str> = line.split('\t').collect();
            let [key, form, n] = cols[..] else { continue };
            let count: u64 = n.parse().unwrap_or(0);
            let e = best.entry(key.to_string()).or_insert((0, String::new()));
            if count > e.0 {
                *e = (count, form.to_string());
            }
        }
        // the lifted prints of the same lemma (`treebank-forms.tsv`) are
        // the other side: the print decides by count — a lexeme the Bible
        // mostly prints with і (сі́мѡнъ, lifted 130 times) keeps і
        let mut lifted: HashMap<String, Vec<(String, u64)>> = HashMap::new();
        for ((key, _, _, form), (n, _)) in bible_counts() {
            lifted.entry(key.clone()).or_default().push((form.clone(), *n));
        }
        let mut out: HashMap<String, Vec<usize>> = HashMap::new();
        for (key, (count, form)) in best {
            let bases: Vec<char> = form.nfc().filter(|c| !unicode_normalization::char::is_combining_mark(*c)).collect();
            let mut at = Vec::new();
            for (i, c) in bases.iter().enumerate() {
                if *c == 'ї' {
                    let next = bases.get(i + 1).copied();
                    let positional = next.is_some_and(|n| church_slavonic::orthography::is_vowel_letter(n) || n == 'й');
                    if !positional {
                        at.push(i);
                    }
                }
            }
            if at.is_empty() {
                continue;
            }
            // the lifted prints that spell і at one of those positions
            // (an abbreviation hides the letter and does not vote)
            let against: u64 = lifted
                .get(&key)
                .map(|prints| {
                    prints
                        .iter()
                        .filter(|(p, _)| {
                            let b: Vec<char> = p.nfc().filter(|c| !unicode_normalization::char::is_combining_mark(*c)).collect();
                            at.iter().any(|i| b.get(*i) == Some(&'і'))
                        })
                        .map(|(_, n)| *n)
                        .sum()
                })
                .unwrap_or(0);
            if against >= count {
                continue;
            }
            out.insert(key, at);
        }
        out
    });
    let Some(at) = positions.get(lemma_key) else { return print };
    let mut out = String::with_capacity(print.len());
    let mut index = 0usize;
    for c in print.nfc() {
        if unicode_normalization::char::is_combining_mark(c) {
            out.push(c);
            continue;
        }
        if c == 'і' && at.contains(&index) {
            out.push('ї');
        } else {
            out.push(c);
        }
        index += 1;
    }
    out
}

/// The Bible's spelling of a Polyakov print: the ligature and the
/// loanword's ї.
fn bible_spelling(print: String, lemma_key: &str) -> String {
    loanword_iota(ligature(print, lemma_key), lemma_key)
}

/// The key an existing id is looked up by: the letters with the initial
/// uk as «оу» and the initial ѿ as «ѡт» — the two spellings the print
/// and Polyakov differ in, and nothing else (мі́ръ and ми́ръ stay apart).
fn id_lookup_key(letters: &str) -> String {
    let stem = church_slavonic::orthography::id_stem(letters);
    let stem = match stem.strip_prefix('ѿ') {
        Some(rest) if !rest.is_empty() => format!("ѡт{rest}"),
        _ => stem,
    };
    // the loanword's ї and the kendema are the print's letters (3.3), the
    // ids keep Polyakov's і and ѵ
    stem.replace('ї', "і").replace('ѷ', "ѵ")
}

/// The id's stem: the lemma's letters with marks stripped.
fn lexeme_stem(lemma: &Form) -> String {
    church_slavonic::orthography::id_stem(&lemma.letters)
}

/// The noun import (the Part 1 entry point, kept for the floor test).
pub fn import_nouns() -> Result<Outcome, Box<dyn Error>> {
    import(Pos::Noun)
}
