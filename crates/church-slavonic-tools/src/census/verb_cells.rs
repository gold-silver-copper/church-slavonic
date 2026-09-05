//! `census verb-cells --ocs`: for every OCS verb class, the aorist,
//! imperfect and l-participle cells the table holds against what the
//! class's Leskien type predicts (the two-stem system: the sigmatic
//! aorist on a vowel stem, the -ох- aorist on a consonant stem with the
//! first palatalisation before е, class II keeping -нѫ-; the imperfect on
//! the stem the type names; the l-participle on the infinitive stem), and
//! the UD variants (`U:`) on verb lines in those blocks — what declaring
//! the blocks by type stands to reproduce.

use church_slavonic::cell::{Cell, FiniteTense, PartTense, VerbCell};
use church_slavonic::grammar::Recension;
use church_slavonic::paradigm::{Shape, table_of};
use church_slavonic::{Lexicon, Pos};
use std::collections::BTreeMap;
use std::error::Error;

/// The type's prediction for a cell: `<stem>-<ending>`; `None` where the
/// type makes no statement (the residue classes).
pub fn predicted(class: &str, cell: &Cell) -> Option<String> {
    let t: Vec<&str> = class.split(':').collect();
    let (kind, sub) = (t.get(1).copied().unwrap_or(""), t.get(2).copied().unwrap_or(""));
    if kind == "res" {
        return None;
    }
    let velar = matches!(sub, "к" | "г");
    let dental = matches!(sub, "т" | "д" | "з");
    let nasal = matches!(sub, "ьн" | "ьм");
    let vowel_stem = kind == "IV" || kind == "III" || sub == "a" && kind == "I";
    // the theme vowel the class strips off its base (strip 3 types) and
    // the sigmatic aorist and l-participle put back: пꙋсти-хъ, кыпѣ-хъ,
    // лежа-хъ, ора-хъ, таꙗ-хъ, кова-хъ
    let theme = match (kind, sub) {
        ("IV", "i") => "и",
        ("IV", "ě") => "ѣ",
        ("IV", "a") | ("III", "j") | ("I", "a") => "а",
        ("III", "ja") => "ꙗ",
        _ => "",
    };
    let person_ending = |person: u8, number: &str, sigmatic: bool| -> String {
        // the aorist endings after the theme: sigmatic (дѣла-хъ) or -ох-
        let base = match (person, number) {
            (1, "sg") => "хъ",
            (2, "sg") | (3, "sg") => "",
            (1, "du") => "ховѣ",
            (2, "du") => "ста",
            (3, "du") => "сте",
            (1, "pl") => "хомъ",
            (2, "pl") => "сте",
            (3, "pl") => "шѧ",
            _ => "",
        };
        if sigmatic {
            base.to_string()
        } else if base.is_empty() {
            "е".to_string()
        } else {
            format!("о{base}")
        }
    };
    Some(match cell {
        Cell::Verb(VerbCell::Finite { tense: FiniteTense::Aorist, person, number }) => {
            let p = *person as u8 + 1;
            let n = church_slavonic::cell::number_name(*number);
            if kind == "II" {
                format!("1-нѫ{}", person_ending(p, n, true))
            } else if nasal || vowel_stem {
                format!("1-{theme}{}", person_ending(p, n, true))
            } else if velar {
                // рекохъ, рече: the -ох- aorist, the palatalised stem before е
                if p == 2 || p == 3 { "3-е".to_string() } else { format!("2-{}", person_ending(p, n, false)) }
            } else if dental {
                format!("2-{}", person_ending(p, n, false))
            } else {
                format!("1-{}", person_ending(p, n, false))
            }
        }
        Cell::Verb(VerbCell::Finite { tense: FiniteTense::Imperfect, person, number }) => {
            let p = *person as u8 + 1;
            let n = church_slavonic::cell::number_name(*number);
            let ending = match (p, n) {
                (1, "sg") => "хъ",
                (2, "sg") | (3, "sg") => "ше",
                (1, "du") => "ховѣ",
                (2, "du") => "шета",
                (3, "du") => "шете",
                (1, "pl") => "хомъ",
                (2, "pl") => "шете",
                (3, "pl") => "хѫ",
                _ => "",
            };
            // the imperfect: -ѣа- after a consonant stem (несѣахъ, кльнѣахъ,
            // грѧдѣахъ; -аа- after the palatalised velar, речаахъ), -аа-
            // after the theme of the a-types (лежаахъ, писаахъ, коваахъ),
            // -ꙗа- on the iotated stem of class IV -ити and the jer type
            // (хождаахъ, пьꙗахъ), -ѣа- on the -ѣти type (кыпѣахъ), -а- after
            // a vowel stem (дѣлаахъ, таꙗахъ, вѣроваахъ)
            let (stem, theme) = match (kind, sub) {
                ("IV", "a") => ("1", "аа"),
                ("IV", "ě") => ("1", "ѣа"),
                ("IV", _) => ("2", "ꙗа"),
                ("III", "j") => ("1", "аа"),
                ("III", "jer") => ("2", "ꙗа"),
                ("III", "ja") => ("1", "ꙗа"),
                ("III", _) => ("1", "а"),
                ("II", _) => ("1", "нѣа"),
                ("I", "a") => ("1", "аа"),
                ("I", _) if velar => ("3", "аа"),
                ("I", _) if dental || nasal => ("2", "ѣа"),
                _ => ("1", "ѣа"),
            };
            format!("{stem}-{theme}{ending}")
        }
        Cell::Verb(VerbCell::LPart { gender, number }) => {
            let ending = match (gender, number) {
                (church_slavonic::Gender::Masculine, church_slavonic::Number::Singular) => "лъ",
                (church_slavonic::Gender::Feminine, church_slavonic::Number::Singular) => "ла",
                (church_slavonic::Gender::Neuter, church_slavonic::Number::Singular) => "ло",
                (church_slavonic::Gender::Masculine, church_slavonic::Number::Dual) => "ла",
                (church_slavonic::Gender::Feminine, church_slavonic::Number::Dual) | (church_slavonic::Gender::Neuter, church_slavonic::Number::Dual) => "лѣ",
                (church_slavonic::Gender::Masculine, church_slavonic::Number::Plural) => "ли",
                (church_slavonic::Gender::Feminine, church_slavonic::Number::Plural) => "лы",
                (church_slavonic::Gender::Neuter, church_slavonic::Number::Plural) => "ла",
            };
            if kind == "II" { format!("1-нѫ{ending}") } else if velar { format!("2-{ending}") } else { format!("1-{theme}{ending}") }
        }
        _ => return None,
    })
}

fn block(cell: &Cell) -> Option<&'static str> {
    match cell {
        Cell::Verb(VerbCell::Finite { tense: FiniteTense::Aorist, .. }) => Some("aor"),
        Cell::Verb(VerbCell::Finite { tense: FiniteTense::Imperfect, .. }) => Some("impf"),
        Cell::Verb(VerbCell::LPart { .. }) => Some("lpart"),
        Cell::Verb(VerbCell::Participle { tense: PartTense::Past, .. }) => Some("part.past"),
        _ => None,
    }
}

pub fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    if !args.iter().any(|a| a == "--ocs") {
        return Err("census verb-cells --ocs (the Synodal classes are Polyakov's legend, not a seeding)".into());
    }
    let table = table_of(Pos::Verb, Recension::OldChurchSlavonic);
    let mut totals: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
    println!("== OCS verb classes: aorist / imperfect / l-participle cells against the type (agree / disagree / absent)");
    for class in table.iter() {
        let mut counts: BTreeMap<&str, (usize, usize, usize)> = BTreeMap::new();
        let mut disagreements = Vec::new();
        for cell in &class.order {
            let Some(b) = block(cell) else { continue };
            if b == "part.past" {
                continue;
            }
            let Some(want) = predicted(&class.name, cell) else { continue };
            let have = class.cells.get(cell).and_then(|alts| alts.first()).and_then(|a| match &a.shape {
                Shape::Ending { stem, ending, .. } => Some(format!("{stem}-{ending}")),
                _ => None,
            });
            let e = counts.entry(b).or_default();
            match have {
                None => e.2 += 1,
                Some(h) if h == want => e.0 += 1,
                Some(h) => {
                    e.1 += 1;
                    if disagreements.len() < 6 {
                        disagreements.push(format!("{} {h} (type {want})", cell.name()));
                    }
                }
            }
        }
        let line: Vec<String> = counts.iter().map(|(b, (a, d, n))| format!("{b} {a}/{d}/{n}")).collect();
        println!("{:<10} {:<10} {}  {}", class.name, class.exemplar, line.join("  "), disagreements.join("; "));
        for (b, (a, d, n)) in counts {
            let t = totals.entry(b).or_default();
            t.0 += a;
            t.1 += d;
            t.2 += n;
        }
    }
    for (b, (a, d, n)) in &totals {
        println!("total {b}: agree {a}, disagree {d}, absent {n}");
    }
    // the UD variants in those blocks
    let lexicon = Lexicon::ocs();
    let mut by_block: BTreeMap<&str, usize> = BTreeMap::new();
    let mut lines = 0;
    for l in lexicon.iter().filter(|l| l.pos == Pos::Verb && l.src.iter().any(|s| s == "U:")) {
        let mut any = false;
        for (cell, forms) in &l.variants {
            if let Some(b) = block(cell) {
                *by_block.entry(b).or_default() += forms.len();
                any = true;
            }
        }
        if any {
            lines += 1;
        }
    }
    println!("== UD variants on OCS verb lines in these blocks: {lines} lines");
    for (b, n) in by_block {
        println!("{n:>6}  {b}");
    }
    Ok(())
}
