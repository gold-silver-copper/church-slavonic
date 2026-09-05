//! `census clitics`: every Bible token that ends in an enclitic written
//! solid and is not a lexeme with `encl=`; how many analyse whole, how
//! many analyse as host + enclitic once the host's final varia is read as
//! an oxia (Землѧ́же → землѧ̀ + же), how many neither.

use church_slavonic::Lexicon;
use std::collections::BTreeMap;
use std::error::Error;

pub const ENCLITICS: &[&str] = &["же", "бо", "ли", "ми", "ти", "сѧ", "мѧ", "тѧ", "ны", "вы", "си"];

pub use crate::treebank::lift::host_standalone;

pub fn run() -> Result<(), Box<dyn Error>> {
    let Some(bible) = crate::treebank::bible::load()? else {
        return Err("pinned Bible absent".into());
    };
    let lexicon = Lexicon::synodal();
    let mut whole = 0;
    let mut split = 0;
    let mut neither = 0;
    let mut by_enclitic: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let mut split_examples: BTreeMap<String, usize> = BTreeMap::new();
    let mut neither_examples: BTreeMap<String, usize> = BTreeMap::new();
    for book in &bible.books {
        for chapter in &book.chapters {
            for verse in &chapter.verses {
                for token in crate::treebank::node::tokenize(verse.print()) {
                    let Some(core) = crate::treebank::lift::token_core(token) else { continue };
                    // a titlo-written token is the titlo index's business
                    if core.chars().any(|c| matches!(c, '\u{483}' | '\u{487}' | '\u{2DE0}'..='\u{2DFF}')) {
                        continue;
                    }
                    let looked_up = crate::treebank::lift::decapitalized(core).unwrap_or_else(|| core.to_string());
                    let Some(enclitic) = ENCLITICS.iter().find(|e| looked_up.ends_with(*e) && looked_up.chars().count() > e.chars().count() + 1) else { continue };
                    let exact_whole = lexicon.readings(&looked_up).into_iter().filter(|r| r.exact).count();
                    if exact_whole > 0 {
                        whole += 1;
                        continue;
                    }
                    let host = &looked_up[..looked_up.len() - enclitic.len()];
                    // the print drops the host's jer before the enclitic
                    // (и҆́хже = ихъ + же): try it back
                    let candidates = [host_standalone(host), Some(host.to_string()), host_standalone(host).map(|h| format!("{h}ъ")), Some(format!("{host}ъ"))];
                    let ok = candidates.iter().flatten().any(|h| lexicon.readings(h).into_iter().any(|r| r.exact));
                    let e = by_enclitic.entry(enclitic).or_default();
                    if ok {
                        split += 1;
                        e.0 += 1;
                        *split_examples.entry(looked_up.clone()).or_default() += 1;
                    } else {
                        neither += 1;
                        e.1 += 1;
                        if matches!(*enclitic, "же" | "бо" | "ли") {
                            *neither_examples.entry(looked_up.clone()).or_default() += 1;
                        }
                    }
                }
            }
        }
    }
    println!("== solid enclitics: tokens analysing whole {whole}; as host + enclitic {split}; neither {neither}");
    for (e, (s, n)) in &by_enclitic {
        println!("{s:>6} split / {n:>6} neither  -{e}");
    }
    let top = |m: &BTreeMap<String, usize>| {
        let mut v: Vec<_> = m.iter().collect();
        v.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
        v.iter().take(15).map(|(s, n)| format!("{s} {n}")).collect::<Vec<_>>().join(", ")
    };
    println!("split, commonest: {}", top(&split_examples));
    println!("neither (же/бо/ли only; -ти/-сѧ/-ми/-вы/-ны coincide with endings), commonest: {}", top(&neither_examples));
    Ok(())
}
