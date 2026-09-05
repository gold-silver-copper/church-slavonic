//! `cargo xtask eval`: the three numbers, each of which can go down —
//! held-out recall (UD PROIEL dev+test, Syntacticus), Bible coverage
//! through the analyzer, and guesser accuracy (leave-one-out over the
//! lexicon). Part 1 fills the guesser number; Part 2 the other two.

use church_slavonic::cell::Pos;
use church_slavonic::lexicon::Lexicon;
use std::error::Error;

pub fn run(_args: Vec<String>) -> Result<(), Box<dyn Error>> {
    for corpus in held_out_corpora()? {
        let r = recall(Lexicon::ocs(), &corpus);
        println!("held-out recall, {} ({} tokens, {} slots; {} skipped by the loader):", corpus.label, corpus.tokens, corpus.slots.len(), corpus.skipped_total());
        for (pos, hit, total) in &r {
            println!("  {pos:<10} {:.2}% ({hit}/{total})", 100.0 * *hit as f64 / (*total).max(1) as f64);
        }
    }
    match bible_coverage()? {
        Some(c) => println!(
            "Bible coverage, Synodal nouns (analyzer over {} tokens): one reading {} ({:.2}%), several {} ({:.2}%), none {} ({:.2}%); index {} entries in {:.2?}",
            c.tokens,
            c.one,
            100.0 * c.one as f64 / c.tokens.max(1) as f64,
            c.many,
            100.0 * c.many as f64 / c.tokens.max(1) as f64,
            c.none,
            100.0 * c.none as f64 / c.tokens.max(1) as f64,
            c.index_entries,
            c.index_time
        ),
        None => println!("Bible coverage (analyzer):            pinned Bible absent (scripts/fetch-bible.sh)"),
    }
    let g = guesser(Lexicon::synodal(), Pos::Noun);
    println!(
        "guesser accuracy, Synodal nouns (leave-one-out over {} lexemes): class {:.2}%, cells {:.2}% ({}/{})",
        g.lexemes,
        100.0 * g.class_right as f64 / g.lexemes.max(1) as f64,
        100.0 * g.cells_right as f64 / g.cells.max(1) as f64,
        g.cells_right,
        g.cells
    );
    Ok(())
}

/// The held-out corpora: UD PROIEL dev+test and Syntacticus, unpacked
/// from `references/downloads` into `target/sources` on first use.
fn held_out_corpora() -> Result<Vec<crate::sources::ud::Corpus>, Box<dyn Error>> {
    let root = crate::workspace_root();
    let sources = root.join("references/downloads");
    let artifacts = root.join("target/sources");
    let mut out = Vec::new();
    match crate::sources::ud::load_ud_proiel_heldout(&sources, &artifacts)? {
        Some(c) => out.push(c),
        None => println!("held-out recall (UD PROIEL dev+test): source absent under references/downloads (scripts/fetch-sources.sh)"),
    }
    if let Some(c) = crate::sources::ud::load_syntacticus(&sources, &artifacts)? {
        out.push(c);
    }
    Ok(out)
}

/// The manuscript-lax spelling key of the 1.2 harness, layered on the
/// comparison key: the scribes interchange `шт` and `щ`, write `ѣ` for
/// `ꙗ` and for `е`, `ѧ` for `е`, drop or confuse the jers, contract the
/// double vowels of the imperfect and the long adjective (бѣаше ~ бѣше,
/// свѧтааго ~ свѧтаѥго); Syntacticus leaves the Glagolitic `ⱕ`
/// untransliterated. Both sides pass through it, so it merges a surface
/// only with its own cell's forms.
pub fn corpus_fold(word: &str) -> String {
    let folded: String = church_slavonic::orthography::comparison_key(word)
        .replace("шт", "щ")
        .replace("шч", "щ")
        .chars()
        .filter_map(|c| match c {
            'ъ' | 'ь' | '\'' | 'ʼ' | '-' | '\u{2e2f}' => None,
            'ѣ' | 'ⱕ' | 'ѧ' => Some('е'),
            'ю' => Some('у'),
            other => Some(other),
        })
        .collect();
    let mut out = String::new();
    for c in folded.chars() {
        let vowel = matches!(c, 'а' | 'е' | 'и' | 'о' | 'у' | 'ы');
        if vowel && out.ends_with(c) {
            continue;
        }
        out.push(c);
    }
    out.replace("еа", "е").replace("ие", "е").replace("ае", "а").replace("ое", "о").replace("уе", "у")
}

/// Remove every `е` whose both neighbours are consonants — the jer
/// position; an `е` at an edge or beside a vowel is a real vowel.
fn elide_jer_e(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let vowel = |c: char| matches!(c, 'а' | 'е' | 'и' | 'о' | 'у' | 'ы' | 'ѣ' | 'ю' | 'ѧ');
    chars.iter().enumerate().filter(|(i, c)| **c != 'е' || *i == 0 || *i + 1 == chars.len() || vowel(chars[i - 1]) || vowel(chars[i + 1])).map(|(_, c)| *c).collect()
}

fn is_subsequence(short: &str, long: &str) -> bool {
    let mut rest = long;
    for c in short.chars() {
        match rest.find(c) {
            Some(at) => rest = &rest[at + c.len_utf8()..],
            None => return false,
        }
    }
    true
}

/// A treebank surface matches a produced form when their folds agree; a
/// third-person pronoun may carry the post-prepositional н- the schema
/// has no cell for; a jer written as е or dropped is elided on both
/// sides; an abbreviated surface (under a titlo) matches when its letters
/// are an ordered proper subsequence of the full form sharing the first
/// letter. The 1.2 harness's rules, kept so the numbers compare.
pub fn corpus_matches(surface: &str, produced: &str, pos: Pos) -> bool {
    let s = corpus_fold(surface);
    let p = corpus_fold(produced);
    if s == p {
        return true;
    }
    if matches!(pos, Pos::Pronoun | Pos::Verb) && !p.starts_with('н') && s.strip_prefix('н').is_some_and(|rest| rest == p) {
        return true;
    }
    // the third-person pronoun's aphaeresis after a vowel (го for ѥго, мъ
    // for имъ): the surface is the produced form without its initial vowel
    if pos == Pos::Pronoun && p.chars().count() > 2 && p.chars().next().is_some_and(|c| matches!(c, 'и' | 'е')) && p.strip_prefix(['и', 'е']) == Some(s.as_str()) {
        return true;
    }
    if elide_jer_e(&s) == elide_jer_e(&p) {
        return true;
    }
    crate::sources::ud::is_abbreviated(surface) && s.chars().count() < p.chars().count() && s.chars().next() == p.chars().next() && is_subsequence(&s, &p)
}

/// Held-out recall per part of speech: the share of annotated slots whose
/// surface the lexicon produces for the annotated lemma and cell — any
/// lexeme with the lemma's letters, any form of the cell (the primary, an
/// alternative or a variant), compared by the accent-blind key. The
/// personal pronoun (the treebank's lemma `personal`) is reported apart
/// from the other pronouns, as the 1.2 baselines were.
pub fn recall(lexicon: &Lexicon, corpus: &crate::sources::ud::Corpus) -> Vec<(&'static str, u64, u64)> {
    use church_slavonic::cell::Cell;
    use church_slavonic::orthography::comparison_key;
    use std::collections::HashMap;
    let mut by_key: HashMap<(Pos, String), Vec<&church_slavonic::Lexeme>> = HashMap::new();
    for l in lexicon.iter() {
        by_key.entry((l.pos, comparison_key(&l.lemma))).or_default().push(l);
    }
    let personal = |cell: &Cell| matches!(cell, Cell::Pron(p) if p.person.is_some() || (p.gender.is_none() && p.number.is_none()));
    let mut cache: HashMap<(String, Cell), Vec<String>> = HashMap::new();
    let mut counts: Vec<(&'static str, u64, u64)> = vec![("nouns", 0, 0), ("adjectives", 0, 0), ("verbs", 0, 0), ("pronouns", 0, 0), ("npron", 0, 0)];
    let mut sampled = [0usize; 5];
    let mut blocks: std::collections::BTreeMap<(&'static str, String, bool), u64> = std::collections::BTreeMap::new();
    for slot in &corpus.slots {
        let row = match slot.pos {
            Pos::Noun => 0,
            Pos::Adjective => 1,
            Pos::Verb => 2,
            Pos::Pronoun if personal(&slot.cell) => 3,
            Pos::Pronoun => 4,
            Pos::Closed => continue,
        };
        counts[row].2 += 1;
        // the payerok stands for a jer; the titlo and the other marks stay
        // for the abbreviation rule (the fold strips them)
        let surface = slot.surface.replace('ꙿ', "ъ").to_lowercase();
        let guessed;
        let candidates: Vec<&church_slavonic::Lexeme> = if row == 3 {
            // the personal pronoun: every personal lexeme
            lexicon.iter().filter(|l| l.pos == Pos::Pronoun && matches!(l.class.as_str(), "PPja" | "PPmy" | "PPty" | "PPvy" | "PPseb" | "PP3")).collect()
        } else {
            match by_key.get(&(slot.pos, comparison_key(&slot.lemma))) {
                Some(found) => found.clone(),
                None => {
                    // the lexicon lacks the lemma: the guesser's paradigm
                    guessed = lexicon.guess(&slot.lemma, slot.pos);
                    vec![&guessed]
                }
            }
        };
        // the cells a slot may be answered from: its own, a pronoun's
        // clitic twin (the treebank tags ми as the dative), and for бꙑти
        // the aorist for an imperfect-tagged form (бѣ, бѣшѧ: the treebanks
        // tag the imperfective aorist Tense=Past|Aspect=Imp)
        let mut cells = vec![slot.cell];
        if let Cell::Pron(p) = slot.cell
            && !p.clitic
        {
            cells.push(Cell::Pron(church_slavonic::cell::PronCell { clitic: true, ..p }));
        }
        if let Cell::Verb(church_slavonic::cell::VerbCell::Finite { tense: church_slavonic::cell::FiniteTense::Imperfect, person, number }) = slot.cell
            && comparison_key(&slot.lemma) == comparison_key("бꙑти")
        {
            cells.push(Cell::Verb(church_slavonic::cell::VerbCell::Finite { tense: church_slavonic::cell::FiniteTense::Aorist, person, number }));
        }
        let hit = candidates.iter().any(|l| {
            cells.iter().any(|cell| {
                let prints = cache.entry((l.id.clone(), *cell)).or_insert_with(|| l.forms(*cell).iter().map(|f| f.print(lexicon.recension)).collect());
                prints.iter().any(|p| corpus_matches(&surface, p, slot.pos))
            })
        });
        if hit {
            counts[row].1 += 1;
        } else if std::env::var_os("CS_RECALL_BLOCKS").is_some() {
            let block = match slot.cell {
                Cell::Verb(church_slavonic::cell::VerbCell::Participle { tense, voice, series, .. }) => format!("part.{tense:?}.{voice:?}.{series:?}"),
                Cell::Verb(church_slavonic::cell::VerbCell::Finite { tense, .. }) => format!("{tense:?}"),
                Cell::Verb(v) => v.name().split('.').next().unwrap_or("").to_string(),
                Cell::Adj(a) => format!("{:?}.{:?}", a.series, a.degree),
                other => other.name(),
            };
            let guessed = candidates.iter().all(|l| l.provenance == church_slavonic::lexicon::Provenance::Guessed);
            *blocks.entry((counts[row].0, block, guessed)).or_default() += 1;
        } else if let Some(n) = std::env::var("CS_RECALL_MISSES").ok().and_then(|v| v.parse::<usize>().ok())
            && sampled[row] < n
        {
            sampled[row] += 1;
            let have: Vec<String> = candidates.iter().map(|l| format!("{}[{}{}]: {}", l.id, l.class, if l.provenance == church_slavonic::lexicon::Provenance::Guessed { " guessed" } else { "" }, l.forms(slot.cell).iter().map(|f| f.print(lexicon.recension)).collect::<Vec<_>>().join("|"))).collect();
            println!("  miss {:<10} {} {} = {} ; lexicon: {}", counts[row].0, slot.lemma, slot.cell.name(), slot.surface, if have.is_empty() { "(no lexeme)".to_string() } else { have.join(" ; ") });
        }
    }
    if !blocks.is_empty() {
        let mut rows: Vec<_> = blocks.into_iter().collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.1));
        for ((pos, block, guessed), n) in rows.iter().take(40) {
            println!("  misses {pos:<10} {block:<28} {n:>6}{}", if *guessed { "  (lemma guessed)" } else { "" });
        }
    }
    counts
}

/// The guesser measured against the lexicon: for every lexeme, guess a
/// lexeme from its lemma alone and compare the class and every primary
/// form of the lexicon's paradigm.
pub struct GuessReport {
    pub lexemes: usize,
    pub class_right: usize,
    pub cells: usize,
    pub cells_right: usize,
}

pub fn guesser(lexicon: &Lexicon, pos: Pos) -> GuessReport {
    let mut r = GuessReport { lexemes: 0, class_right: 0, cells: 0, cells_right: 0 };
    for lexeme in lexicon.iter().filter(|l| l.pos == pos) {
        if lexeme.note.contains("pl-tantum") {
            continue;
        }
        r.lexemes += 1;
        let guessed = lexicon.guess(&lexeme.lemma, pos);
        if guessed.class == lexeme.class {
            r.class_right += 1;
        }
        for (cell, form) in lexeme.paradigm() {
            r.cells += 1;
            if guessed.inflect(cell).map(|f| f.print(lexicon.recension)) == Some(form.print(lexicon.recension)) {
                r.cells_right += 1;
            }
        }
    }
    r
}

pub struct BibleCoverage {
    pub tokens: usize,
    pub one: usize,
    pub many: usize,
    pub none: usize,
    pub index_entries: usize,
    pub index_time: std::time::Duration,
}

/// Every word token of the pinned Bible (punctuation split off,
/// apparatus tokens skipped) through the Synodal analyzer, EXACT readings
/// only: one, several, none.
pub fn bible_coverage() -> Result<Option<BibleCoverage>, Box<dyn Error>> {
    let Some(bible) = crate::treebank::bible::load()? else {
        return Ok(None);
    };
    let lexicon = Lexicon::synodal();
    let started = std::time::Instant::now();
    let index_entries = lexicon.index().len();
    let index_time = started.elapsed();
    let mut c = BibleCoverage { tokens: 0, one: 0, many: 0, none: 0, index_entries, index_time };
    for book in &bible.books {
        for chapter in &book.chapters {
            for verse in &chapter.verses {
                for token in crate::treebank::node::tokenize(verse.print()) {
                    let Some(core) = crate::treebank::lift::token_core(token) else { continue };
                    c.tokens += 1;
                    let looked_up = crate::treebank::lift::decapitalized(core).unwrap_or_else(|| core.to_string());
                    let n = lexicon.analyze(&looked_up).into_iter().filter(|a| a.exact).count();
                    match n {
                        0 => c.none += 1,
                        1 => c.one += 1,
                        _ => c.many += 1,
                    }
                }
            }
        }
    }
    Ok(Some(c))
}

