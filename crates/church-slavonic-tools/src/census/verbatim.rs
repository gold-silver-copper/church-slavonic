//! `census verbatim` (V3.3 Part 0): every verbatim leaf of the treebank
//! (`(w …)` without `:amb`) grouped by why the lexicon does not print it —
//! (a) found by the analyzer's key but not exactly, bucketed by the
//! letters that differ; (b) a titlo token the titlo index has no row for;
//! (c) no reading at all, by shape. And the `:amb` tokens whose readings
//! hold exactly one finite-verb reading beside another part of speech,
//! by the other reading's cell (Part 3's material).

use crate::treebank::node::Node;
use church_slavonic::cell::{Cell, VerbCell};
use church_slavonic::orthography::strip_marks;
use church_slavonic::{Lexicon, Recension};
use std::collections::BTreeMap;
use std::error::Error;
use unicode_normalization::UnicodeNormalization;

fn walk(node: &Node, out: &mut Vec<(String, bool)>) {
    match node {
        Node::W { surface, notes } => out.push((surface.clone(), notes.iter().any(|(k, _)| k == "amb"))),
        Node::Cap(inner) | Node::Abbr { child: inner, .. } => walk(inner, out),
        Node::Pw { host, .. } => walk(host, out),
        Node::Group { children, .. } => children.iter().for_each(|c| walk(c, out)),
        _ => {}
    }
}

fn is_titlo(surface: &str) -> bool {
    surface.chars().any(|c| c == '\u{483}' || c == '\u{487}' || ('\u{2de0}'..='\u{2dff}').contains(&c))
}

/// The first letter pair that differs between two prints (marks
/// stripped), named as a bucket.
fn letter_bucket(surface: &str, print: &str) -> String {
    let a: Vec<char> = strip_marks(surface).nfc().collect();
    let b: Vec<char> = strip_marks(print).nfc().collect();
    if a == b {
        return "marks only (stress, kamora, varia, breathing)".to_string();
    }
    let mut diffs: Vec<(usize, char, char)> = Vec::new();
    let n = a.len().max(b.len());
    for i in 0..n {
        let (x, y) = (a.get(i).copied().unwrap_or('∅'), b.get(i).copied().unwrap_or('∅'));
        if x != y {
            diffs.push((i, x, y));
        }
    }
    if diffs.len() > 2 || a.len() != b.len() {
        let jers = diffs.iter().all(|(_, x, y)| matches!((x, y), ('ъ', '∅') | ('∅', 'ъ') | ('ь', '∅') | ('∅', 'ь') | ('ъ', 'ь') | ('ь', 'ъ')));
        return if jers { "a jer".to_string() } else { "several letters".to_string() };
    }
    let (i, x, y) = diffs[0];
    match (x, y) {
        ('ѧ', 'ꙗ') | ('ꙗ', 'ѧ') if i == 0 => "head ѧ against ꙗ".to_string(),
        ('ѧ', 'ꙗ') | ('ꙗ', 'ѧ') => "ѧ against ꙗ inside the word".to_string(),
        ('ѷ', 'ѵ') | ('ѵ', 'ѷ') => "the izhitsa's kendema".to_string(),
        ('ѡ', 'о') | ('о', 'ѡ') | ('ѻ', 'о') | ('о', 'ѻ') | ('ѡ', 'ѻ') | ('ѻ', 'ѡ') => "wide/narrow о".to_string(),
        ('є', 'е') | ('е', 'є') => "wide/narrow е".to_string(),
        ('ї', 'і') | ('і', 'ї') | ('и', 'і') | ('і', 'и') | ('ї', 'и') | ('и', 'ї') => "і/ї/и".to_string(),
        ('ъ', 'ь') | ('ь', 'ъ') => "a jer".to_string(),
        ('ѵ', 'и') | ('и', 'ѵ') | ('ѵ', 'і') | ('і', 'ѵ') => "izhitsa against и".to_string(),
        ('ѳ', 'ф') | ('ф', 'ѳ') => "ѳ against ф".to_string(),
        ('ѕ', 'з') | ('з', 'ѕ') => "ѕ against з".to_string(),
        _ => format!("one letter ({x} against {y})"),
    }
}

pub fn run(write: bool) -> Result<(), Box<dyn Error>> {
    let lexicon = Lexicon::synodal();
    let mut found: Vec<(String, bool)> = Vec::new();
    for (_, _, _, tree) in super::treebank_trees()? {
        walk(&tree, &mut found);
    }
    let verbatim = found.iter().filter(|(_, amb)| !amb).count();
    // (a) by key, not exactly
    let mut a: BTreeMap<String, BTreeMap<String, (usize, String)>> = BTreeMap::new();
    // (b) titlo tokens with no row, by prefix (the letters up to the titlo)
    let mut b: BTreeMap<String, (usize, bool)> = BTreeMap::new();
    let mut b_surfaces: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    // (c) no reading, by shape
    let mut c: BTreeMap<&'static str, BTreeMap<String, usize>> = BTreeMap::new();
    let (mut na, mut nb, mut nc) = (0usize, 0usize, 0usize);
    let mut cache: BTreeMap<String, Option<(String, String)>> = BTreeMap::new();
    for (surface, amb) in &found {
        if *amb {
            continue;
        }
        let looked_up = crate::treebank::lift::decapitalized(surface).unwrap_or_else(|| surface.clone());
        if is_titlo(&looked_up) {
            nb += 1;
            let prefix: String = looked_up.chars().take_while(|ch| *ch != '\u{483}' && !('\u{2de0}'..='\u{2dff}').contains(ch)).collect();
            let prefix: String = strip_marks(&prefix).chars().take(4).collect();
            let e = b.entry(prefix.clone()).or_default();
            e.0 += 1;
            *b_surfaces.entry(prefix).or_default().entry(looked_up.clone()).or_default() += 1;
            continue;
        }
        let reading = cache.entry(looked_up.clone()).or_insert_with(|| {
            let mut rs = lexicon.analyze(&looked_up);
            rs.sort_by_key(|r| (r.exact, r.alt));
            rs.first().map(|r| (r.lexeme.id.clone(), r.print.clone()))
        });
        match reading {
            Some((id, print)) => {
                na += 1;
                let bucket = letter_bucket(&looked_up, print);
                let e = a.entry(bucket).or_default().entry(looked_up.clone()).or_insert((0, id.clone()));
                e.0 += 1;
            }
            None => {
                nc += 1;
                let capitalised = crate::treebank::lift::decapitalized(surface).is_some();
                let shape: &'static str = if looked_up.ends_with("надесѧть") || looked_up.contains("десѧт") {
                    "a -десѧть compound"
                } else if capitalised {
                    "capitalised (a name?)"
                } else {
                    "the rest"
                };
                *c.entry(shape).or_default().entry(looked_up.clone()).or_default() += 1;
            }
        }
    }
    println!("census verbatim: {verbatim} verbatim leaves — (a) found by key, not exactly {na}; (b) a titlo token with no row {nb}; (c) no reading {nc}");
    println!("== (a) by the letters that differ");
    let mut buckets: Vec<_> = a.iter().collect();
    buckets.sort_by_key(|(_, m)| std::cmp::Reverse(m.values().map(|(n, _)| *n).sum::<usize>()));
    for (bucket, m) in buckets {
        let total: usize = m.values().map(|(n, _)| *n).sum();
        let mut top: Vec<_> = m.iter().collect();
        top.sort_by_key(|(_, (n, _))| std::cmp::Reverse(*n));
        println!("{total:>7}  {bucket} ({} surfaces): {}", m.len(), top.iter().take(12).map(|(s, (n, id))| format!("{s} {n} ({id})")).collect::<Vec<_>>().join(", "));
    }
    if write {
        // `data/loanword-iota.tsv`: the surfaces the print spells with a
        // non-positional ї (кївѡ́тъ) for a lexeme Polyakov spells with і —
        // the importer's evidence for the lexeme's letter (V3.3 Part 1)
        let path = crate::workspace_root().join("data/loanword-iota.tsv");
        let mut out = String::from("lemma_key\tsurface\tcount\n");
        if let Some(m) = a.get("і/ї/и") {
            let mut rows: Vec<_> = m.iter().collect();
            rows.sort();
            for (surface, (n, id)) in rows {
                let Some(l) = lexicon.get(id) else { continue };
                if !surface.contains('ї') {
                    continue;
                }
                out.push_str(&format!("{}\t{surface}\t{n}\n", church_slavonic::orthography::comparison_key(&l.lemma)));
            }
        }
        std::fs::write(&path, out)?;
        println!("wrote {}", path.display());
    }
    println!("== (b) titlo tokens with no row, by prefix");
    let mut rows: Vec<_> = b.iter().collect();
    rows.sort_by_key(|(_, (n, _))| std::cmp::Reverse(*n));
    println!("{}", rows.iter().take(40).map(|(p, (n, _))| format!("{p} {n}")).collect::<Vec<_>>().join(", "));
    for (p, (n, _)) in rows.iter().take(40) {
        let mut top: Vec<_> = b_surfaces.get(*p).map(|m| m.iter().collect()).unwrap_or_default();
        top.sort_by_key(|(_, k)| std::cmp::Reverse(**k));
        println!("  {p} {n}: {}", top.iter().take(10).map(|(s, k)| format!("{s} {k}")).collect::<Vec<_>>().join(", "));
    }
    println!("== (c) no reading, by shape");
    for (shape, m) in &c {
        let total: usize = m.values().sum();
        let mut top: Vec<_> = m.iter().collect();
        top.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
        println!("{total:>7}  {shape} ({} surfaces): {}", m.len(), top.iter().take(60).map(|(s, n)| format!("{s} {n}")).collect::<Vec<_>>().join(", "));
    }
    // Part 3's material: the :amb tokens with one finite-verb reading
    let mut verb_amb: BTreeMap<String, (usize, BTreeMap<String, usize>)> = BTreeMap::new();
    let mut verb_amb_tokens = 0usize;
    for (surface, amb) in &found {
        if !*amb {
            continue;
        }
        let looked_up = crate::treebank::lift::decapitalized(surface).unwrap_or_else(|| surface.clone());
        let readings: Vec<_> = lexicon.readings(&looked_up).into_iter().filter(|r| r.exact).collect();
        let finite: Vec<_> = readings.iter().filter(|r| r.cells.iter().all(|(c, _)| matches!(c, Cell::Verb(VerbCell::Finite { .. })))).collect();
        let others: Vec<_> = readings.iter().filter(|r| !r.cells.iter().all(|(c, _)| matches!(c, Cell::Verb(VerbCell::Finite { .. })))).collect();
        if finite.len() != 1 || others.is_empty() {
            continue;
        }
        verb_amb_tokens += 1;
        let other_cells: String = others.iter().map(|r| r.cell_set().map(|c| c.name()).unwrap_or_else(|| "word".to_string())).collect::<Vec<_>>().join(" | ");
        let e = verb_amb.entry(other_cells).or_default();
        e.0 += 1;
        *e.1.entry(looked_up).or_default() += 1;
    }
    println!("== :amb tokens with exactly one finite-verb reading beside another part of speech: {verb_amb_tokens}");
    let mut kinds: Vec<_> = verb_amb.iter().collect();
    kinds.sort_by_key(|(_, (n, _))| std::cmp::Reverse(*n));
    for (cells, (n, surfaces)) in kinds.iter().take(25) {
        let mut top: Vec<_> = surfaces.iter().collect();
        top.sort_by_key(|(_, k)| std::cmp::Reverse(**k));
        println!("{n:>7}  the other reading: {cells}  — {}", top.iter().take(6).map(|(s, k)| format!("{s} {k}")).collect::<Vec<_>>().join(", "));
    }
    let _ = Recension::Synodal;
    Ok(())
}
