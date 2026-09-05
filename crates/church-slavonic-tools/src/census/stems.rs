//! `cargo xtask census stems --pos <pos> [--ocs]`: the stored numbered
//! stems of a lexicon classified by their relation to the lemma's stem —
//! what a class-level derivation could replace (V2.1 Part 0). Relations:
//! `theme` (the infinitive stem minus its final vowel), `iot`/`pal1`/`pal2`
//! (that stem iotated or palatalised), `ov` (-ова-/-ева- → -ꙋ), `base`
//! (the stem unchanged), `artefact` (a whole present form taken as the
//! stem by the seeding's longest common prefix), `suppletive` (none of
//! the above).

use church_slavonic::cell::{Cell, FiniteTense, VerbCell};
use church_slavonic::form::Form;
use church_slavonic::grammar::Recension;
use church_slavonic::paradigm::{iotate, palatalise_in};
use church_slavonic::{Lexicon, Pos};
use std::collections::BTreeMap;
use std::error::Error;

fn infinitive_stem(letters: &str) -> &str {
    letters.strip_suffix("ти").or_else(|| letters.strip_suffix("щи")).unwrap_or(letters)
}

fn theme_dropped(stem: &str) -> Option<&str> {
    let last = stem.chars().last()?;
    church_slavonic::orthography::is_vowel_letter(last).then(|| &stem[..stem.len() - last.len_utf8()])
}

/// The relation of a stored stem to the lemma's infinitive stem.
pub fn relation(lexeme: &church_slavonic::Lexeme, stored: &str, recension: Recension) -> &'static str {
    let letters = Form::from_print(&lexeme.lemma).letters;
    let inf = infinitive_stem(&letters);
    if stored == inf {
        return "base";
    }
    if let Some(t) = theme_dropped(inf) {
        if stored == t {
            return "theme";
        }
        if stored == iotate(t) {
            return "iot";
        }
        if stored == palatalise_in(t, true, recension) {
            return "pal1";
        }
        if stored == palatalise_in(t, false, recension) {
            return "pal2";
        }
    }
    if stored == iotate(inf) {
        return "iot";
    }
    if stored == palatalise_in(inf, true, recension) {
        return "pal1";
    }
    if let Some(s) = inf.strip_suffix("ова").or_else(|| inf.strip_suffix("ева"))
        && stored == format!("{s}ꙋ")
    {
        return "ov";
    }
    // a whole attested present form (the seeding read one form's prefix)
    let present: Vec<String> = lexeme
        .cells()
        .into_iter()
        .filter(|c| matches!(c, Cell::Verb(VerbCell::Finite { tense: FiniteTense::Present, .. } | VerbCell::Imperative { .. })))
        .filter_map(|c| lexeme.inflect(c))
        .map(|f| f.letters)
        .collect();
    if present.iter().any(|f| stored == f || stored.len() >= f.len()) || stored.ends_with(['ѫ', 'ѭ', 'ъ']) || stored.ends_with("ши") || stored.ends_with("те") {
        return "artefact";
    }
    "suppletive"
}

pub fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let ocs = args.iter().any(|a| a == "--ocs");
    let pos = match args.iter().position(|a| a == "--pos").and_then(|i| args.get(i + 1)).map(String::as_str) {
        Some("noun") => Pos::Noun,
        Some("adj") => Pos::Adjective,
        Some("verb") | None => Pos::Verb,
        Some("pron") => Pos::Pronoun,
        Some(other) => return Err(format!("--pos {other}").into()),
    };
    let lexicon = if ocs { Lexicon::ocs() } else { Lexicon::synodal() };
    let mut counts: BTreeMap<(String, &'static str), Vec<String>> = BTreeMap::new();
    let mut lines = 0;
    for l in lexicon.iter().filter(|l| l.pos == pos) {
        let numbered: Vec<&(String, String)> = l.stems.iter().filter(|(k, _)| k.chars().all(|c| c.is_ascii_digit())).collect();
        if numbered.is_empty() {
            continue;
        }
        lines += 1;
        for (k, v) in numbered {
            let r = relation(l, v, lexicon.recension);
            counts.entry((k.clone(), r)).or_default().push(format!("{} {}={}", l.lemma, k, v));
        }
    }
    println!("== stored numbered stems: {lines} {} lines of {} ({})", pos.tag(), lexicon.iter().filter(|l| l.pos == pos).count(), if ocs { "ocs" } else { "syn" });
    let mut rows: Vec<_> = counts.into_iter().collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.1.len()));
    for ((stem, rel), examples) in rows {
        println!("{:>6}  stem {stem} {rel:<10} {}", examples.len(), examples.iter().take(10).cloned().collect::<Vec<_>>().join("; "));
    }
    Ok(())
}
