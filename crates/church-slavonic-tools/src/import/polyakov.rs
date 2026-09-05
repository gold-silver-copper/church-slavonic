//! Polyakov's corpus dictionary as a lexicon source: every S entry with a
//! paradigm code becomes a noun line — class = the code (identity), gender
//! and animacy from the tags, the forms fitted per cell with the corpus
//! count choosing the primary. Titlo spellings (`9^` cells) are skipped
//! here (titlo lemmas are Part 3/5), unaccented forms in this accented
//! source are transliteration noise and are counted, not stored.

use super::fit::{Attested, Bundled, fit};
use super::{Outcome, Quarantined};
use crate::sources::polyakov::{self, Entry, features};
use church_slavonic::cell::{Cell, NounCell, Pos};
use church_slavonic::form::Form;
use church_slavonic::grammar::{Gender, Recension};
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

/// The lexeme's attested noun cells: print forms per cell, primary first
/// by corpus count.
fn attested_cells(entry: &Entry, o: &mut Outcome) -> (Attested, Bundled) {
    // per cell: form -> (unbundled count, bundled count). A form whose tag
    // set bundles several cases (`pl,gen/acc`) attests each of them only
    // weakly: where a cell has a form tagged for it alone, the bundled
    // forms are variants, never the primary (Polyakov's counts are per
    // form, so at a bundled cell the genitive's frequency would wear the
    // accusative's tag — the 1.x «га́ды/гадѡ́въ» finding).
    let mut counts: HashMap<Cell, BTreeMap<String, (u64, u64)>> = HashMap::new();
    let accented_lemma = is_accented(&entry.lemma);
    for form in &entry.forms {
        if form.cells.is_empty() {
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
        for set in &form.cells {
            let f = features(set);
            let Some(number) = f.number else {
                o.bump("forms skipped: no number");
                continue;
            };
            if f.cases.is_empty() {
                o.bump("forms skipped: no case");
                continue;
            }
            // the analysis this set came from: bundled if its printed tags
            // name several cases for one number
            let bundled = form
                .tags
                .split('|')
                .any(|analysis| analysis.contains("nom/") || analysis.contains("gen/") || analysis.contains("dat/") || analysis.contains("acc/") || analysis.contains("ins/") || analysis.contains("loc/"))
                && f.cases.len() == 1
                && form.cells.iter().filter(|c| features(c).number == Some(number)).count() > 1;
            for case in &f.cases {
                let cell = Cell::Noun(NounCell::new(*case, number));
                let e = counts.entry(cell).or_default().entry(printed.clone()).or_default();
                if bundled { e.1 += form.count } else { e.0 += form.count }
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
            // unbundled first (by count), then bundled (by count), names tie-break
            v.sort_by(|a, b| {
                let ka = (if any_unbundled { a.1 == 0 } else { false }, std::cmp::Reverse(a.1 + a.2));
                let kb = (if any_unbundled { b.1 == 0 } else { false }, std::cmp::Reverse(b.1 + b.2));
                ka.cmp(&kb).then(a.0.cmp(&b.0))
            });
            (cell, v.into_iter().map(|(f, _, _)| f).collect())
        })
        .collect();
    (attested, bundled)
}

/// Numbered stems read off the attested forms: for each stem the class
/// uses, the letters of every attested primary whose class alternative
/// ends in a known ending, minus that ending; the commonest reading that
/// differs from the derived stem is a candidate (`stems=1=льв` for
/// ле́въ : льва̀). The caller keeps it only when the fit improves.
fn inferred_stems(class: &church_slavonic::paradigm::Class, subject: &church_slavonic::paradigm::Subject<'_>, attested: &Attested) -> Vec<(String, String)> {
    use church_slavonic::paradigm::Shape;
    let derived = class.stems_of(subject);
    let mut votes: HashMap<u8, BTreeMap<String, usize>> = HashMap::new();
    for (cell, forms) in attested {
        let Some(primary) = forms.first() else { continue };
        let letters = Form::from_print(primary).letters;
        let letters: String = letters.chars().map(|c| match c { 'ѡ' => 'о', 'є' => 'е', other => other }).collect();
        let Some(alts) = class.cells.get(cell) else { continue };
        for alt in alts {
            if let Shape::Ending { stem, ending, .. } = &alt.shape
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

/// The base with its wide letters narrowed (`артемѡн` -> `артемон`): the
/// Greek names in -ѡнъ keep the omega in the citation form only.
fn narrowed(base: &str) -> String {
    base.chars().map(|c| match c { 'ѡ' => 'о', 'є' => 'е', other => other }).collect()
}

/// How many stress marks (oxia, varia, kamora) a form carries.
fn stress_marks(form: &str) -> usize {
    use unicode_normalization::UnicodeNormalization;
    form.nfd().filter(|c| matches!(*c, '\u{300}' | '\u{301}' | '\u{302}' | '\u{311}')).count()
}

/// The classes an entry's code may stand for: the code itself, its
/// fleeting-vowel twin (`N1t` -> `N1t*`: Polyakov codes ѻ҆се́лъ N1t though
/// the legend's N1t* exemplar is осел-ъ) and its velar twin for a stem in
/// к/г/х (`N1t` -> `N1k`/`N1g`/`N1x`: во́лхвъ, волсвѝ). The fit keeps the
/// best.
fn candidate_classes(code: &str, lemma_letters: &str) -> Vec<String> {
    let mut out = vec![code.to_string()];
    if !code.ends_with('*') {
        out.push(format!("{code}*"));
    }
    let stem_end = lemma_letters.chars().rev().nth(1);
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
/// («нас̑», «а҆рхіепс̑кпа»), never a form.
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
/// genitive plural where the rule would spell it differently.
fn inserted_stem(class: &church_slavonic::paradigm::Class, lemma_letters: &str, attested: &Attested) -> Option<(String, String)> {
    if !class.stems.iter().any(|(_, d)| *d == Derivation::Insert) {
        return None;
    }
    let gen_pl = Cell::Noun(NounCell::new(church_slavonic::grammar::Case::Genitive, church_slavonic::grammar::Number::Plural));
    let printed = attested.get(&gen_pl)?.first()?;
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

/// Print one lexeme's fit in full (`--debug <lemma>`).
pub fn debug_noun(wanted: &str) -> Result<(), Box<dyn Error>> {
    let path = super::intermediate_dir().join("polyakov.jsonl");
    let entries = polyakov::read(&path)?;
    let mut o = Outcome::default();
    let classes = table(Pos::Noun);
    for entry in entries.iter().filter(|e| e.tags.first().map(String::as_str) == Some("S")) {
        let lemma = realise(&entry.lemma, &SYN);
        if Form::from_print(&lemma).key() != Form::from_print(wanted).key() {
            continue;
        }
        println!("== {} {:?} class {}", lemma, entry.tags, entry.class);
        let lemma_form = Form::from_print(&lemma);
        println!("lemma letters {:?} stress {:?}", lemma_form.letters, lemma_form.stress);
        let (attested, bundled) = attested_cells(entry, &mut o);
        println!("bundled-only cells: {:?}", bundled.iter().map(|c| c.name()).collect::<Vec<_>>());
        for code in class_codes(&entry.class) {
            let Some(class) = classes.get(&code) else { println!("class {code} unknown"); continue };
            let stems: Vec<(String, String)> = inserted_stem(class, &lemma_form.letters, &attested).into_iter().collect();
            let f = fit("x.n", &lemma, Pos::Noun, class, None, entry.tags.iter().find_map(|t| match t.as_str() { "anim" => Some(true), "inan" => Some(false), _ => None }), stems, &attested, &bundled, vec![], String::new());
            println!("class {code}: stress {} reproduced {}/{}", f.lexeme.stress, f.reproduced, f.attested);
            let subject = church_slavonic::paradigm::Subject { lemma: &lemma_form.letters, animate: f.lexeme.animate, stems: &f.lexeme.stems };
            for (cell, forms) in &attested {
                let ev = super::fit::evidence(class, &subject, lemma_form.stress, *cell, &forms[0]);
                let letters = class.letters(*cell, &subject);
                let predicted = f.lexeme.inflect(*cell).map(|x| x.print(SYN)).unwrap_or_default();
                println!("  {:12} attested {:?} predicted {} evidence {:?} class-letters {:?}", cell.name(), forms, predicted, ev, letters.iter().map(|l| format!("{}{} sv={}", l.letters, if l.mark {"^"} else {""}, l.stem_vowels)).collect::<Vec<_>>());
            }
        }
    }
    Ok(())
}

pub fn import_nouns() -> Result<Outcome, Box<dyn Error>> {
    let path = super::intermediate_dir().join("polyakov.jsonl");
    let entries = polyakov::read(&path)?;
    let mut o = Outcome::default();
    let classes = table(Pos::Noun);
    let mut ids: HashMap<String, u32> = HashMap::new();
    for entry in &entries {
        if entry.tags.first().map(String::as_str) != Some("S") {
            continue;
        }
        o.bump("S entries");
        let lemma = realise(&entry.lemma, &SYN);
        let lemma_form = Form::from_print(&lemma);
        let quarantine = |o: &mut Outcome, reason: &'static str, detail: String| {
            o.quarantine.push(Quarantined {
                recension: SYN,
                pos: Pos::Noun,
                lemma: lemma.clone(),
                source: format!("P:{}", entry.class),
                reason,
                detail,
            });
        };
        let codes = class_codes(&entry.class);
        if codes.is_empty() {
            quarantine(&mut o, "no class in the source", String::new());
            continue;
        }
        if codes.iter().any(|c| c.starts_with('A')) {
            quarantine(&mut o, "adjectival class on a noun entry (Part 3)", entry.class.clone());
            continue;
        }
        let mut candidates: Vec<String> = Vec::new();
        for code in &codes {
            for c in candidate_classes(code, &lemma_form.letters) {
                if !candidates.contains(&c) {
                    candidates.push(c);
                }
            }
        }
        let known: Vec<&church_slavonic::paradigm::Class> = candidates.iter().filter_map(|c| classes.get(c)).collect();
        // the coded class is known but the entry may still fit a twin better
        if !codes.iter().any(|c| classes.get(c).is_some()) && !known.is_empty() {
            o.bump("entries: only a twin of the coded class is known");
        }
        if known.is_empty() {
            quarantine(&mut o, "class not in the inventory", entry.class.clone());
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
                "persn" | "topn" | "famn" | "patrn" => notes.push(t.clone()),
                _ => {}
            }
        }
        let (attested, bundled) = attested_cells(entry, &mut o);
        if attested.is_empty() {
            quarantine(&mut o, "no analysed forms", String::new());
            continue;
        }
        // the nominative singular must be the lemma
        let nom = Cell::Noun(NounCell::new(church_slavonic::grammar::Case::Nominative, church_slavonic::grammar::Number::Singular));
        if let Some(first) = attested.get(&nom).and_then(|v| v.first())
            && Form::from_print(first).key() != lemma_form.key()
        {
            quarantine(&mut o, "attested nominative differs from the lemma", first.clone());
            continue;
        }
        // id: the bare lemma; homographs number in source order
        let bare = lemma_form.letters.clone();
        let n = ids.entry(bare.clone()).or_default();
        *n += 1;
        let id = if *n == 1 { format!("{bare}.n") } else { format!("{bare}.n.{n}") };
        let src = vec![format!("P:{}", entry.class)];
        // fit every candidate class; keep the best
        // a plurale tantum: tagged `pl`, or a lemma the class's nominative
        // does not produce while it ends like a plural (лахі́сы, є҆квата́ны)
        let plurale_tantum = entry.tags.iter().any(|t| t == "pl")
            || (matches!(lemma_form.letters.chars().last(), Some('ы' | 'и'))
                && known.iter().all(|c| {
                    let subject = church_slavonic::paradigm::Subject { lemma: &lemma_form.letters, animate, stems: &[] };
                    c.letters(nom, &subject).first().is_none_or(|l| Form::new(l.letters.clone(), None, false).key() != lemma_form.key())
                }));
        if plurale_tantum && !notes.iter().any(|n| n == "pl-tantum") {
            notes.push("pl-tantum".into());
        }
        let mut best: Option<super::fit::Fit> = None;
        for class in &known {
            let mut stems: Vec<(String, String)> = inserted_stem(class, &lemma_form.letters, &attested).into_iter().collect();
            if plurale_tantum {
                // the lemma is the nominative plural: its base is the lemma
                // minus the plural ending's one letter
                let n = lemma_form.letters.chars().count().saturating_sub(1);
                stems.push(("base".to_string(), lemma_form.letters.chars().take(n).collect()));
            }
            let f = fit(&id, &lemma, Pos::Noun, class, gender, animate, stems.clone(), &attested, &bundled, src.clone(), notes.join("; "));
            if best.as_ref().is_none_or(|b| f.reproduced > b.reproduced) {
                best = Some(f);
            }
            // stems read off the attested forms, kept when they fit better
            {
                let subject = church_slavonic::paradigm::Subject { lemma: &lemma_form.letters, animate, stems: &stems };
                let inferred = inferred_stems(class, &subject, &attested);
                if !inferred.is_empty() {
                    let mut stems3 = stems.clone();
                    stems3.extend(inferred);
                    let f3 = fit(&id, &lemma, Pos::Noun, class, gender, animate, stems3, &attested, &bundled, src.clone(), notes.join("; "));
                    if best.as_ref().is_none_or(|b| f3.reproduced > b.reproduced) {
                        best = Some(f3);
                    }
                }
            }
            // a lemma with a wide letter may keep it in the citation form
            // only: try the narrowed base too
            if lemma_form.letters.contains(['ѡ', 'є']) {
                let n = lemma_form.letters.chars().count().saturating_sub(class.strip);
                let base: String = lemma_form.letters.chars().take(n).collect();
                let narrow = narrowed(&base);
                if narrow != base {
                    let mut stems2 = stems.clone();
                    stems2.retain(|(k, _)| k != "base");
                    stems2.push(("base".to_string(), narrow));
                    let f2 = fit(&id, &lemma, Pos::Noun, class, gender, animate, stems2, &attested, &bundled, src.clone(), notes.join("; "));
                    if best.as_ref().is_none_or(|b| f2.reproduced > b.reproduced) {
                        best = Some(f2);
                    }
                }
            }
        }
        let Some(f) = best else { continue };
        // the lemma itself must come out of the class (or its override)
        match f.lexeme.inflect(nom) {
            None => {
                quarantine(&mut o, "class declares no nominative", f.lexeme.class.clone());
                continue;
            }
            Some(form) if !plurale_tantum && form.key() != lemma_form.key() => {
                quarantine(&mut o, "class does not produce the lemma", form.print(SYN));
                continue;
            }
            _ => {}
        }
        *o.counts.entry("cells attested").or_default() += f.attested as u64;
        *o.counts.entry("cells reproduced").or_default() += f.reproduced as u64;
        *o.counts.entry("cells reachable (any alternative/variant)").or_default() += f.reachable as u64;
        *o.counts.entry("cells: true exceptions (no alternative fits)").or_default() += f.exceptions as u64;
        if f.exceptions > 0 {
            o.bump("lexemes with a true exception");
        }
        for (cell, alt) in &f.alt_matches {
            if let Some((idx, _)) = alt
                && *idx > 0
                && f.lexeme.overrides.iter().any(|(c, _)| c == cell)
            {
                o.bump("cells: alternative preference (override names a non-primary alternative)");
            }
        }
        *o.counts.entry("cells: letter miss").or_default() += f.letter_misses.len() as u64;
        *o.counts.entry("cells: stress miss").or_default() += f.stress_misses.len() as u64;
        for (cell, alt) in &f.alt_matches {
            if let Some((alt, marked)) = alt {
                *o.alt_preference.entry((f.lexeme.class.clone(), cell.name())).or_default().entry(*alt).or_default() += 1;
                if *alt == 0 {
                    let e = o.mark_preference.entry((f.lexeme.class.clone(), cell.name())).or_default();
                    if *marked { e.0 += 1 } else { e.1 += 1 }
                }
            }
        }
        for cell in &f.letter_misses {
            *o.letter_misses.entry((f.lexeme.class.clone(), cell.name())).or_default() += 1;
        }
        for (cell, attested) in &f.lexeme.overrides {
            *o.override_cells.entry(cell.name()).or_default() += 1;
            let alt_fits = f.alt_matches.iter().any(|(c, a)| c == cell && a.is_some());
            if !alt_fits || !f.stress_misses.contains(cell) {
                let mut bare = f.lexeme.clone();
                bare.overrides.retain(|(c, _)| c != cell);
                let reachable = bare.forms(*cell).iter().any(|x| super::fit::translit_equal(&x.print(SYN), attested));
                if !reachable {
                    let predicted = bare.inflect(*cell).map(|x| x.print(SYN)).unwrap_or_default();
                    o.exception_samples.push((f.lexeme.lemma.clone(), f.lexeme.class.clone(), f.lexeme.stress.clone(), cell.name(), attested.clone(), predicted));
                }
            }
            if f.stress_misses.contains(cell) {
                // predicted without the override: a copy of the lexeme minus it
                let mut bare = f.lexeme.clone();
                bare.overrides.retain(|(c, _)| c != cell);
                let predicted = bare.inflect(*cell).map(|x| x.print(SYN)).unwrap_or_default();
                o.stress_miss_samples.push((f.lexeme.lemma.clone(), f.lexeme.stress.clone(), cell.name(), attested.clone(), predicted));
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
