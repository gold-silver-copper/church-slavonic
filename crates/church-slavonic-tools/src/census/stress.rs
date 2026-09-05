//! `census stress`: every Synodal stress column with an exception list
//! (`a{gen.pl=E;…}`), the lists normalised to their shape (base paradigm
//! plus the sorted cell → place pairs), the shapes ranked, and how many
//! lines the twelve commonest shapes would absorb as named paradigms.

use crate::import::fit::{StressSample, stress_sample};
use church_slavonic::cell::Cell;
use church_slavonic::form::Form;
use church_slavonic::paradigm::Subject;
use church_slavonic::stress::{Place, StressSpec, Vowels, resolve_in};
use church_slavonic::{Lexicon, Pos};
use std::collections::BTreeMap;
use std::error::Error;

/// The shape of a stress column: the base name and its exception pairs
/// sorted, so two lexemes with the same rule read the same.
pub fn shape(column: &str) -> Option<String> {
    let (base, rest) = column.split_once('{')?;
    let inner = rest.strip_suffix('}')?;
    let mut pairs: Vec<&str> = inner.split(';').map(str::trim).filter(|p| !p.is_empty()).collect();
    pairs.sort();
    pairs.dedup();
    Some(format!("{base}{{{}}}", pairs.join(";")))
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let lexicon = Lexicon::synodal();
    for pos in [Pos::Noun, Pos::Adjective, Pos::Verb, Pos::Pronoun] {
        let mut shapes: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut plain: BTreeMap<String, usize> = BTreeMap::new();
        let mut with_list = 0;
        // a column whose braces hold only `sg=`/`du=`/`pl=` is a paradigm
        // with one number moved, which the notation spells inline (3.0
        // Part 1 step 4): counted apart from the exception lists
        let mut number_moves = 0;
        let total = lexicon.iter().filter(|l| l.pos == pos).count();
        for l in lexicon.iter().filter(|l| l.pos == pos) {
            match shape(&l.stress) {
                Some(s) => {
                    let inner = s.split_once('{').map(|(_, r)| r.trim_end_matches('}')).unwrap_or("");
                    if inner.split(';').all(|e| e.starts_with("sg=") || e.starts_with("du=") || e.starts_with("pl=")) {
                        number_moves += 1;
                        *plain.entry(s.clone()).or_default() += 1;
                        continue;
                    }
                    with_list += 1;
                    shapes.entry(s).or_default().push(l.lemma.clone());
                }
                None => *plain.entry(l.stress.clone()).or_default() += 1,
            }
        }
        let mut ranked: Vec<_> = shapes.iter().collect();
        ranked.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
        let twelve: usize = ranked.iter().take(12).map(|(_, v)| v.len()).sum();
        println!("== {} stress: {total} lines; plain columns {}; one number moved {number_moves}; with an exception list {with_list} in {} shapes; the twelve commonest shapes absorb {twelve}", pos.tag(), plain.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", "), shapes.len());
        for (s, v) in ranked.iter().take(12) {
            println!("{:>6}  {s}  {}", v.len(), v.iter().take(4).cloned().collect::<Vec<_>>().join(", "));
        }
        places(lexicon, pos);
    }
    compounds_and_adverbs(lexicon);
    Ok(())
}

/// Which places name an attested stressed vowel, in the fitter's order.
fn places_naming(s: &StressSample, lemma_stress: Option<u8>) -> Vec<&'static str> {
    let k = Some(s.index);
    let mut out = Vec::new();
    for (place, name) in [(Place::Stem, "S"), (Place::End, "E"), (Place::StemLast, "L"), (Place::Final, "F"), (Place::Pre, "P")] {
        if resolve_in(place, lemma_stress, s.vowels) == k {
            out.push(name);
        }
    }
    out
}

/// `census stress` continued: every exception entry of every list read
/// against the places the format has and the candidate place `P` (3.0:
/// the last vowel of the stem before the class's extension). An
/// exception cell is a cell the base paradigm (the column without its
/// braces) places elsewhere than the lexeme prints it; the report counts
/// the cells by the places that name them, the lists `P` would empty or
/// shorten, and the shapes that survive once every cell only `P` names is
/// written `P` — compressed to a block (`part.pres.pass=P`) where the
/// whole block agrees.
pub fn places(lexicon: &Lexicon, pos: Pos) {
    let mut by_places: BTreeMap<String, usize> = BTreeMap::new();
    let mut lists = 0usize;
    let mut emptied = 0usize;
    let mut shortened = 0usize;
    let mut unreadable = 0usize;
    let mut shapes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut only_p_examples: Vec<String> = Vec::new();
    for l in lexicon.iter().filter(|l| l.pos == pos && l.stress.contains('{')) {
        lists += 1;
        let Some(class) = l.class() else { continue };
        let base_column = l.stress.split_once('{').map(|(b, _)| b).unwrap_or("").to_string();
        let base_column = if base_column.is_empty() { "a".to_string() } else { base_column };
        let Ok(Some(base)) = StressSpec::parse(&base_column, pos) else { continue };
        let lemma = l.lemma_form();
        let subject = Subject { lemma: &lemma.letters, animate: l.animate, stems: &l.stems };
        // the exception cells with the places that name what they print
        let mut cells: Vec<(Cell, Vec<&'static str>)> = Vec::new();
        for cell in l.cells() {
            if l.overrides.iter().any(|(c, _)| *c == cell) {
                continue;
            }
            let Some(form) = l.inflect(cell) else { continue };
            let Some(sample) = stress_sample(class, &subject, cell, &form.print(l.recension)) else { continue };
            let expected = resolve_in(base.place(cell), lemma.stress, sample.vowels);
            if expected == Some(sample.index) {
                continue;
            }
            cells.push((cell, places_naming(&sample, lemma.stress)));
        }
        if cells.is_empty() {
            unreadable += 1;
            continue;
        }
        // a stuck cell is one no place of the format names; P decides a
        // list when it names every stuck cell
        let mut p_only = 0usize;
        let mut stuck = 0usize;
        for (_, names) in &cells {
            let key = if names.is_empty() { "index only".to_string() } else { names.join("") };
            *by_places.entry(key).or_default() += 1;
            if names.contains(&"P") && names.len() == 1 {
                p_only += 1;
                stuck += 1;
            } else if names.is_empty() {
                stuck += 1;
            }
        }
        let p_any = p_only;
        if p_only > 0 && only_p_examples.len() < 8 {
            let (cell, _) = cells.iter().find(|(_, n)| n == &["P"]).expect("one");
            only_p_examples.push(format!("{} {} {}", l.lemma, cell.name(), l.inflect(*cell).map(|f| f.print(l.recension)).unwrap_or_default()));
        }
        if p_any > 0 && p_any == stuck {
            emptied += 1;
        } else if p_any > 0 {
            shortened += 1;
        }
        // the shape with `P` written where only `P` names the cell, the
        // other cells under their first place, compressed by block
        let place_of = |names: &Vec<&'static str>| -> String {
            match names.first() {
                Some(n) => n.to_string(),
                None => "n".to_string(),
            }
        };
        let mut by_block: BTreeMap<String, Vec<(Cell, String)>> = BTreeMap::new();
        let mut loose: Vec<String> = Vec::new();
        for (cell, names) in &cells {
            match cell.block() {
                Some(b) => by_block.entry(b).or_default().push((*cell, place_of(names))),
                None => loose.push(format!("{}={}", cell.name(), place_of(names))),
            }
        }
        let mut items: Vec<String> = loose;
        for (block, members) in by_block {
            let declared = l.cells().into_iter().filter(|c| c.block().as_deref() == Some(block.as_str()) && !l.overrides.iter().any(|(o, _)| o == c)).count();
            let first = members[0].1.clone();
            if members.len() == declared && members.iter().all(|(_, p)| *p == first) {
                items.push(format!("{block}={first}"));
            } else {
                items.extend(members.iter().map(|(c, p)| format!("{}={p}", c.name())));
            }
        }
        items.sort();
        shapes.entry(format!("{base_column}{{{}}}", items.join(";"))).or_default().push(l.lemma.clone());
    }
    let mut ranked: Vec<_> = shapes.iter().collect();
    ranked.sort_by_key(|(_, v)| std::cmp::Reverse(v.len()));
    println!("-- {} places: {lists} lists ({unreadable} with no readable exception cell); exception cells by the places that name them: {}", pos.tag(), by_places.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(", "));
    println!("   P names every stuck cell (one no other place names) of {emptied} lists and some of {shortened}; cells only P names, e.g. {}", only_p_examples.join(", "));
    println!("   shapes with P written (block-compressed), the commonest:");
    for (s, v) in ranked.iter().take(24) {
        println!("{:>6}  {s}  {}", v.len(), v.iter().take(3).cloned().collect::<Vec<_>>().join(", "));
    }
}

/// `census stress` continued: the seven -надесѧть numerals (a stressed
/// second element the enclitic rule cannot place) and the `adv-of=`
/// closed lines — each adverb's stressed vowel against the places of the
/// adjective's `adv` cell.
pub fn compounds_and_adverbs(lexicon: &Lexicon) {
    let numerals: Vec<_> = lexicon.iter().filter(|l| l.stems.iter().any(|(k, v)| k == "encl" && v == "надесѧть")).collect();
    let overrides: usize = numerals.iter().map(|l| l.overrides.len()).sum();
    let variants: usize = numerals.iter().map(|l| l.variants.iter().map(|(_, v)| v.len()).sum::<usize>()).sum();
    let off_tail: Vec<String> = numerals
        .iter()
        .flat_map(|l| l.overrides.iter().map(|(c, f)| (l.lemma.clone(), c.name(), f.clone())))
        .filter(|(_, _, f)| {
            let form = Form::from_print(f);
            let total = form.letters.chars().filter(|c| church_slavonic::orthography::is_vowel_letter(*c)).count();
            // на́десѧть: three vowels, the first stressed
            form.stress.is_none_or(|k| usize::from(k) + 3 != total)
        })
        .map(|(l, c, f)| format!("{l} {c} {f}"))
        .collect();
    println!("-- -надесѧть numerals: {} lines with encl=надесѧть, {overrides} overrides, {variants} variants; overrides not stressed on на́десѧть's first vowel: {} {}", numerals.len(), off_tail.len(), off_tail.join(", "));
    let mut kinds: BTreeMap<&'static str, Vec<String>> = BTreeMap::new();
    for l in lexicon.iter().filter(|l| l.pos == Pos::Closed) {
        let Some((_, adj_id)) = l.stems.iter().find(|(k, _)| k == "adv-of") else { continue };
        let Some(adj) = lexicon.get(adj_id) else {
            kinds.entry("adjective missing").or_default().push(l.lemma.clone());
            continue;
        };
        let adverb = Form::from_print(&l.lemma);
        let Some(class) = adj.class() else { continue };
        let adj_lemma = adj.lemma_form();
        let subject = Subject { lemma: &adj_lemma.letters, animate: adj.animate, stems: &adj.stems };
        let adv_cell = Cell::parse(Pos::Adjective, "adv").expect("adv cell");
        let letters = class.letters(adv_cell, &subject);
        let Some(matched) = letters.iter().find(|x| Form::new(x.letters.clone(), None, false).key() == adverb.key()) else {
            kinds.entry("letters differ from the adjective's adv cell").or_default().push(format!("{} ({})", l.lemma, adj.lemma));
            continue;
        };
        let Some(k) = adverb.stress else {
            kinds.entry("adverb unaccented").or_default().push(l.lemma.clone());
            continue;
        };
        let total = matched.letters.chars().filter(|c| church_slavonic::orthography::is_vowel_letter(*c)).count();
        let sample = StressSample { index: k, vowels: Vowels { base: matched.base_vowels, pre: matched.pre_vowels, stem: matched.stem_vowels, total } };
        let names = places_naming(&sample, adj_lemma.stress);
        let spec = adj.stress_spec();
        let adjective_says = spec.as_ref().and_then(|s| resolve_in(s.place(adv_cell), adj_lemma.stress, sample.vowels));
        let kind: &'static str = if adjective_says == Some(k) {
            "the adjective's own adv cell already prints it"
        } else if names.contains(&"S") {
            "the adjective's S"
        } else if names.contains(&"E") {
            "the adjective's E"
        } else if names.contains(&"L") {
            "the adjective's L"
        } else {
            "none of the places"
        };
        kinds.entry(kind).or_default().push(format!("{} ({} {})", l.lemma, adj.lemma, adj.stress));
    }
    println!("-- adv-of adverbs: {}", kinds.iter().map(|(k, v)| format!("{k} {}", v.len())).collect::<Vec<_>>().join("; "));
    for (k, v) in &kinds {
        println!("   {k}: {}", v.iter().take(8).cloned().collect::<Vec<_>>().join(", "));
    }
}
