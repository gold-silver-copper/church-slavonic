//! The titlo verification table: for every committed row, how many of the
//! family's distinct print tokens do the generated cells reproduce?
use church_slavonic_legacy::Recension;
use church_slavonic_syntax_legacy::{bible, lift, titlo};
use std::collections::{HashMap, HashSet};

fn main() {
    let bible = bible::load().expect("parse").expect("present");
    let index = lift::Index::build(&Recension::Synodal);
    let is_abbrev = |w: &str| {
        w.chars().any(|c| c == '\u{0483}' || c == '\u{0487}' || ('\u{2DE0}'..='\u{2DFF}').contains(&c))
    };
    let strip: &[char] = &['.', ',', ':', ';', '!', '?', '(', ')', '«', '»', '꙾', '[', ']'];
    let mut tokens: HashMap<String, u32> = HashMap::new();
    for b in &bible.books {
        for ch in &b.chapters {
            for v in &ch.verses {
                for t in v.print().split_whitespace() {
                    let w: String = t.chars().filter(|c| !strip.contains(c)).collect();
                    if is_abbrev(&w) {
                        *tokens.entry(w).or_default() += 1;
                    }
                }
            }
        }
    }
    let mut families: Vec<(&str, String)> = Vec::new();
    for row in titlo::rows() {
        let key = (row.abbr, titlo::skeleton(row.abbr));
        if !families.iter().any(|(a, _)| *a == key.0) {
            families.push((key.0, key.1));
        }
    }
    println!("| Prefix | Distinct print tokens | Token mass | Reproduced (distinct) | Mass covered |");
    println!("|---|---|---|---|---|");
    let (mut tot_mass, mut tot_cov) = (0u32, 0u32);
    for (abbr, skel) in &families {
        let fam: Vec<(&String, &u32)> = tokens
            .iter()
            .filter(|(w, _)| titlo::skeleton(w).starts_with(skel.as_str()))
            .collect();
        let distinct = fam.len();
        let mass: u32 = fam.iter().map(|(_, c)| **c).sum();
        let mut hit_d = 0usize;
        let mut hit_m = 0u32;
        let mut lowered = HashSet::new();
        for (w, c) in &fam {
            // match with the lift's decapitalization convention
            let low: String = {
                let mut it = w.chars();
                match it.next() {
                    Some(f) => f.to_lowercase().chain(it).collect(),
                    None => String::new(),
                }
            };
            if !index.analyses(&low).is_empty() {
                hit_d += 1;
                hit_m += **c;
                lowered.insert(low);
            }
        }
        tot_mass += mass;
        tot_cov += hit_m;
        println!("| {abbr} | {distinct} | {mass} | {hit_d} | {hit_m} |");
    }
    println!("| **admitted families** | | **{tot_mass}** | | **{tot_cov} ({:.1}%)** |", 100.0 * tot_cov as f64 / tot_mass as f64);
}
