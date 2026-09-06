//! Training and scoring of the statistical tagger (V2.2 Part 5):
//! `cargo xtask train-tagger` learns from the gold morphology of the Old
//! Church Slavonic treebanks — UD PROIEL **train** and Syntacticus with
//! the sentences UD holds out removed — and scores on UD dev+test over
//! the tokens the analyzer gives several readings. The Bible is never
//! training material. The model goes to `data/models/tagger.bin`, its
//! record to `data/models/tagger.md`.

use church_slavonic::cell::{Cell, FiniteTense, PronCell, VerbCell};
use church_slavonic::orthography::comparison_key;
use church_slavonic::{Lexicon, Pos};
use church_slavonic_tagger::{fold, Candidate, Context, Tagger, Trainer};
use std::collections::HashSet;
use std::error::Error;

/// One scoring or training example: the context, the candidates the
/// analyzer offers, the gold candidate's index.
pub struct Example {
    pub ctx: Context,
    pub candidates: Vec<Candidate>,
    pub gold: usize,
}

/// The (pos, cell) readings of a surface, in the analyzer's order, each
/// once.
pub fn candidates_of(lexicon: &Lexicon, surface: &str) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = Vec::new();
    for r in lexicon.readings(surface) {
        for (cell, _) in &r.cells {
            let c = Candidate { pos: r.lexeme.pos, cell: *cell };
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    out
}

/// The cells a gold slot may be answered from (the recall harness's
/// tolerance: a pronoun's clitic twin, бꙑти's aorist for an
/// imperfect-tagged form), and for a direct object the treebanks tag in
/// the genitive the accusative first: UD PROIEL writes the
/// genitive-accusative (сътворимъ чловѣка) as `Case=Gen`, the Synodal
/// overlay as the animate accusative — the tagger is scored on the
/// overlay's convention.
fn gold_cells(lemma: &str, cell: Cell, object: bool) -> Vec<Cell> {
    let mut cells = Vec::new();
    if object && cell.case() == Some(church_slavonic::grammar::Case::Genitive) {
        match cell {
            Cell::Noun(n) => cells.push(Cell::Noun(church_slavonic::cell::NounCell { case: church_slavonic::grammar::Case::Accusative, ..n })),
            Cell::Adj(a) => cells.push(Cell::Adj(church_slavonic::cell::AdjCell { case: church_slavonic::grammar::Case::Accusative, ..a })),
            Cell::Pron(p) => cells.push(Cell::Pron(PronCell { case: church_slavonic::grammar::Case::Accusative, ..p })),
            _ => {}
        }
    }
    cells.push(cell);
    if let Cell::Pron(p) = cell
        && !p.clitic
    {
        cells.push(Cell::Pron(PronCell { clitic: true, ..p }));
    }
    if let Cell::Verb(VerbCell::Finite { tense: FiniteTense::Imperfect, person, number }) = cell
        && comparison_key(lemma) == comparison_key("бꙑти")
    {
        cells.push(Cell::Verb(VerbCell::Finite { tense: FiniteTense::Aorist, person, number }));
    }
    cells
}

/// The examples of a corpus: every token with several readings among
/// which the gold reading stands; `all` counts the tokens with a gold
/// reading among the readings at all (the denominator of the report).
pub fn examples(lexicon: &Lexicon, corpus: &crate::sources::ud::Corpus) -> (Vec<Example>, usize, usize) {
    let mut out = Vec::new();
    let mut with_gold = 0;
    let mut tokens = 0;
    for sentence in &corpus.sentences {
        let surfaces: Vec<String> = sentence.iter().map(|t| t.surface.replace('ꙿ', "ъ")).collect();
        let mut prev_choice: Option<Candidate> = None;
        for (i, token) in sentence.iter().enumerate() {
            let cands = candidates_of(lexicon, &surfaces[i]);
            let mut choice: Option<Candidate> = None;
            if token.slots.is_empty() {
                if !cands.is_empty() && cands.iter().all(|c| c.pos == Pos::Closed) {
                    choice = Some(cands[0]);
                }
            } else {
                tokens += 1;
                let golds: Vec<(Pos, Cell)> = token.slots.iter().flat_map(|&s| {
                    let slot = &corpus.slots[s];
                    gold_cells(&slot.lemma, slot.cell, token.object).into_iter().map(move |c| (slot.pos, c))
                }).collect();
                // the first gold cell the readings offer, in the golds' order
                // of preference
                let gold = golds.iter().find_map(|(p, cell)| cands.iter().position(|c| c.pos == *p && c.cell == *cell));
                if let Some(g) = gold {
                    with_gold += 1;
                    choice = Some(cands[g]);
                    if cands.len() >= 2 {
                        out.push(Example {
                            ctx: Context {
                                surface: surfaces[i].clone(),
                                prev: i.checked_sub(1).map(|j| surfaces[j].clone()),
                                next: surfaces.get(i + 1).cloned(),
                                prev_lemma: i.checked_sub(1).map(|j| sentence[j].lemma.clone()),
                                next_lemma: sentence.get(i + 1).map(|t| t.lemma.clone()),
                                prev_choice,
                            },
                            candidates: cands.clone(),
                            gold: g,
                        });
                    }
                }
            }
            prev_choice = choice;
        }
    }
    (out, with_gold, tokens)
}

fn sentence_key(sentence: &[crate::sources::ud::SequenceToken]) -> String {
    sentence.iter().map(|t| fold(&t.surface)).collect::<Vec<_>>().join(" ")
}

/// Accuracy of a chooser over examples.
fn accuracy(examples: &[Example], choose: impl Fn(&Example) -> usize) -> (usize, usize) {
    let right = examples.iter().filter(|e| choose(e) == e.gold).count();
    (right, examples.len())
}

fn pct(a: usize, b: usize) -> f64 {
    100.0 * a as f64 / b.max(1) as f64
}

/// `cargo xtask train-tagger [--epochs n]`.
/// `cargo xtask tagger-curve`: the bundled tagger's calibration on UD
/// PROIEL dev+test (never the overlay, never the Bible) — for each tenth
/// of its softmax share, how many tokens it chose there and how many
/// right, cumulatively from the top. 3.2 Part 5: a threshold is applied
/// only if the overlay's precision above it is ≥ 90%; the curve says
/// whether the share means anything.
pub fn curve() -> Result<(), Box<dyn Error>> {
    let root = crate::workspace_root();
    let sources = root.join("references/downloads");
    let artifacts = root.join("target/sources");
    let lexicon = Lexicon::ocs();
    let Some(heldout) = crate::sources::ud::load_ud_proiel_heldout(&sources, &artifacts)? else {
        return Err("UD PROIEL absent under references/downloads".into());
    };
    let tagger = church_slavonic_tagger::Tagger::bundled();
    if tagger.is_empty() {
        return Err("no bundled tagger model".into());
    }
    let (dev_examples, _, _) = examples(lexicon, &heldout);
    let mut buckets: std::collections::BTreeMap<u8, (usize, usize)> = std::collections::BTreeMap::new();
    for e in &dev_examples {
        let Some((i, p)) = tagger.choose(&e.ctx, &e.candidates) else { continue };
        let b = buckets.entry(((p * 10.0).floor() as u8).min(9)).or_default();
        b.0 += 1;
        if i == e.gold {
            b.1 += 1;
        }
    }
    println!("tagger-curve: UD PROIEL dev+test, {} tokens with several readings", dev_examples.len());
    let (mut above_n, mut above_r) = (0, 0);
    for (bucket, (n, r)) in buckets.iter().rev() {
        above_n += n;
        above_r += r;
        println!("  p ≥ 0.{bucket}: chose {above_n}, right {above_r} ({:.2}%); this tenth {n} chosen, {r} right ({:.2}%)", pct(above_r, above_n), pct(*r, *n));
    }
    Ok(())
}

pub fn train(args: &[String]) -> Result<(), Box<dyn Error>> {
    let epochs: usize = args.iter().position(|a| a == "--epochs").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(8);
    let root = crate::workspace_root();
    let sources = root.join("references/downloads");
    let artifacts = root.join("target/sources");
    let started = std::time::Instant::now();
    let lexicon = Lexicon::ocs();
    let Some(train_ud) = crate::sources::ud::load_ud_proiel_train(&sources, &artifacts)? else {
        return Err("UD PROIEL absent under references/downloads (scripts/fetch-sources.sh)".into());
    };
    let Some(heldout) = crate::sources::ud::load_ud_proiel_heldout(&sources, &artifacts)? else {
        return Err("UD PROIEL absent".into());
    };
    let mut syntacticus = crate::sources::ud::load_syntacticus(&sources, &artifacts)?;
    // Syntacticus carries the same Codex Marianus UD splits: the held-out
    // sentences leave the training material
    let held: HashSet<String> = heldout.sentences.iter().map(|s| sentence_key(s)).collect();
    let mut dropped = 0;
    if let Some(s) = syntacticus.as_mut() {
        let before = s.sentences.len();
        s.sentences.retain(|sent| !held.contains(&sentence_key(sent)));
        dropped = before - s.sentences.len();
    }
    let (mut train_examples, train_gold, train_tokens) = examples(lexicon, &train_ud);
    let ud_examples = train_examples.len();
    let mut synt_line = String::from("Syntacticus: absent");
    if let Some(s) = &syntacticus {
        let (ex, gold, tokens) = examples(lexicon, s);
        synt_line = format!("Syntacticus: {} sentences ({dropped} held-out sentences removed), {tokens} annotated tokens, gold among the readings {gold}, examples with several readings {}", s.sentences.len(), ex.len());
        train_examples.extend(ex);
    }
    let (dev_examples, dev_gold, dev_tokens) = examples(lexicon, &heldout);
    println!("corpora loaded and analyzed in {:.1?}", started.elapsed());
    println!("UD PROIEL train: {} sentences, {train_tokens} annotated tokens, gold among the readings {train_gold}, examples with several readings {ud_examples}", train_ud.sentences.len());
    println!("{synt_line}");
    println!("UD PROIEL dev+test: {} sentences, {dev_tokens} annotated tokens, gold among the readings {dev_gold} ({:.2}%), examples with several readings {}", heldout.sentences.len(), pct(dev_gold, dev_tokens), dev_examples.len());
    let (b, n) = accuracy(&dev_examples, |_| 0);
    println!("baseline (the analyzer's first reading): {b}/{n} = {:.2}%", pct(b, n));

    let mut trainer = Trainer::default();
    let mut order: Vec<usize> = (0..train_examples.len()).collect();
    let mut seed: u64 = 0x2545F4914F6CDD1D;
    let mut report = Vec::new();
    for epoch in 1..=epochs {
        // Fisher–Yates with a fixed-seed xorshift: the model is reproducible
        for i in (1..order.len()).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            order.swap(i, (seed % (i as u64 + 1)) as usize);
        }
        let mut right = 0;
        for &i in &order {
            let e = &train_examples[i];
            if trainer.step(&e.ctx, &e.candidates, e.gold) {
                right += 1;
            }
        }
        println!("epoch {epoch}: training accuracy {right}/{} = {:.2}%", order.len(), pct(right, order.len()));
        report.push(format!("epoch {epoch}: training accuracy {:.2}%", pct(right, order.len())));
    }
    let mut tagger = trainer.finish();
    // weights too small to move a decision go, for the model's size
    let prune: f32 = args.iter().position(|a| a == "--prune").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(0.0);
    if prune > 0.0 {
        let before = tagger.weights.len();
        tagger.weights.retain(|_, w| w.abs() >= prune);
        println!("pruned |w| < {prune}: {before} → {} features", tagger.weights.len());
    }
    let (r, n) = accuracy(&dev_examples, |e| tagger.choose(&e.ctx, &e.candidates).map(|(i, _)| i).unwrap_or(0));
    println!("tagger on UD dev+test, tokens with several readings: {r}/{n} = {:.2}% (baseline {:.2}%)", pct(r, n), pct(b, n));
    // by number of candidates
    let mut by_size: std::collections::BTreeMap<usize, (usize, usize)> = std::collections::BTreeMap::new();
    for e in &dev_examples {
        let k = e.candidates.len().min(6);
        let entry = by_size.entry(k).or_default();
        entry.1 += 1;
        if tagger.choose(&e.ctx, &e.candidates).map(|(i, _)| i).unwrap_or(0) == e.gold {
            entry.0 += 1;
        }
    }
    for (k, (a, t)) in &by_size {
        println!("  {} readings: {a}/{t} = {:.2}%", if *k == 6 { "6+".to_string() } else { k.to_string() }, pct(*a, *t));
    }
    // by part of speech (the gold reading's)
    let mut by_pos: std::collections::BTreeMap<&'static str, (usize, usize, usize)> = std::collections::BTreeMap::new();
    for e in &dev_examples {
        let entry = by_pos.entry(e.candidates[e.gold].pos.tag()).or_default();
        entry.2 += 1;
        if tagger.choose(&e.ctx, &e.candidates).map(|(i, _)| i).unwrap_or(0) == e.gold {
            entry.0 += 1;
        }
        if e.gold == 0 {
            entry.1 += 1;
        }
    }
    let mut pos_lines = Vec::new();
    for (pos, (a, b, t)) in &by_pos {
        println!("  {pos:<6} {a}/{t} = {:.2}% (first reading {:.2}%)", pct(*a, *t), pct(*b, *t));
        pos_lines.push(format!("{pos} {a}/{t} = {:.2}% (first reading {:.2}%)", pct(*a, *t), pct(*b, *t)));
    }
    println!("trained and scored in {:.1?}", started.elapsed());
    let model_path = root.join("data/models/tagger.bin");
    let bytes = tagger.to_bytes();
    std::fs::write(&model_path, &bytes)?;
    println!("model: {} features, {} bytes → {}", tagger.weights.len(), bytes.len(), model_path.display());
    let record = format!(
        "# The tagger model\n\nTrained by `cargo xtask train-tagger --epochs {epochs}{}` on {} (never on the Bible).\n\n- UD PROIEL train: {} sentences, {train_tokens} annotated tokens\n- {synt_line}\n- examples (tokens with several readings, the gold among them): {}\n- {}\n- UD PROIEL dev+test, tokens with several readings: {r}/{n} = {:.2}% (the analyzer's first reading: {:.2}%)\n- by part of speech: {}\n- features {}, {} bytes\n\nHashes in `tagger.sha256` (the model and the corpora it was trained on).\n",
        if prune > 0.0 { format!(" --prune {prune}") } else { String::new() },
        chrono_date(),
        train_ud.sentences.len(),
        train_examples.len(),
        report.join("; "),
        pct(r, n),
        pct(b, n),
        pos_lines.join("; "),
        tagger.weights.len(),
        bytes.len()
    );
    std::fs::write(root.join("data/models/tagger.md"), record)?;
    let _ = Tagger::default();
    Ok(())
}

fn chrono_date() -> String {
    // the date without a dependency: days since the epoch → civil date
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// The overlay's examples (3.4 Part 4): every hand leaf with one cell
/// whose token the auto lift, after the constraint layer, leaves with
/// several candidates the hand cell is among — with the book and chapter
/// it comes from, for the folds. The context is built the way the
/// treebank's tagger builds it, the previous token's choice being the
/// hand's (as the gold is in training).
pub fn overlay_examples(lexicon: &Lexicon) -> Result<Vec<(String, Example)>, Box<dyn Error>> {
    use crate::treebank::node::Node;
    let Some(bible) = crate::treebank::bible::load()? else {
        return Err("pinned Bible absent".into());
    };
    let lifter = crate::treebank::lift::Lifter::new(lexicon);
    let mut out = Vec::new();
    for (bi, book) in bible.books.iter().enumerate() {
        let hand_path = crate::treebank::runner::book_file(&crate::treebank::runner::hand_dir(), bi);
        let Ok(text) = std::fs::read_to_string(&hand_path) else { continue };
        let entries = crate::treebank::sexpr::parse_many(&text).map_err(|e| format!("{}: {e}", hand_path.display()))?;
        for entry in &entries {
            let (ch, vs, hand) = crate::treebank::runner::read_entry(entry)?;
            let Some(print) = book.chapters.iter().find(|c| c.chapter == ch).and_then(|c| c.verses.iter().find(|v| v.verse == vs)).map(|v| v.print().to_string()) else { continue };
            let (mut auto, _) = lifter.lift_verse(&print);
            crate::treebank::disambiguate::disambiguate(&mut auto, lexicon);
            let Node::Group { children, .. } = &auto else { continue };
            let h = crate::treebank::runner::word_nodes(&hand);
            let a: Vec<&Node> = crate::treebank::runner::word_nodes(&auto);
            if h.len() != a.len() {
                continue;
            }
            let surfaces: Vec<Option<String>> = children.iter().map(|c| crate::treebank::tag::surface_of(c, lexicon)).collect();
            // the auto's word nodes in order of the verse's children: map
            // each word node to its child index by identity
            let child_of: Vec<usize> = a.iter().map(|w| children.iter().position(|c| std::ptr::eq(c, *w) || crate::treebank::runner::word_nodes(c).iter().any(|x| std::ptr::eq(*x, *w))).unwrap_or(usize::MAX)).collect();
            let mut prev_choice: Option<Candidate> = None;
            for (k, (hn, an)) in h.iter().zip(a.iter()).enumerate() {
                let i = child_of[k];
                let Some(Node::Lex { id: hid, cells: hc, .. }) = crate::treebank::disambiguate::leaf(hn) else {
                    prev_choice = crate::treebank::tag::choice_of(hn, lexicon);
                    continue;
                };
                let Some(hpos) = lexicon.get(hid).map(|l| l.pos) else { continue };
                let gold_candidate = Candidate { pos: hpos, cell: hc.first() };
                if i == usize::MAX || hc.len() != 1 {
                    prev_choice = Some(gold_candidate);
                    continue;
                }
                let candidates: Vec<Candidate> = match an {
                    Node::Lex { id, cells, .. } => match lexicon.get(id).map(|l| l.pos) {
                        Some(pos) => cells.iter().map(|cell| Candidate { pos, cell }).collect(),
                        None => Vec::new(),
                    },
                    _ => match crate::treebank::disambiguate::amb_surface(an) {
                        Some(surface) => {
                            let looked_up = crate::treebank::lift::decapitalized(surface).unwrap_or_else(|| surface.to_string());
                            candidates_of(lexicon, &looked_up)
                        }
                        None => Vec::new(),
                    },
                };
                if candidates.len() >= 2
                    && let Some(g) = candidates.iter().position(|c| *c == gold_candidate)
                {
                    let before = i.checked_sub(1).filter(|j| !crate::treebank::disambiguate::boundary(children, *j));
                    let after = (i + 1 < children.len() && !crate::treebank::disambiguate::boundary(children, i + 1)).then_some(i + 1);
                    let ctx = Context {
                        surface: surfaces[i].clone().unwrap_or_default(),
                        prev: before.and_then(|j| surfaces[j].clone()),
                        next: after.and_then(|j| surfaces[j].clone()),
                        prev_lemma: before.and_then(|j| crate::treebank::tag::lemma_of(&children[j], lexicon)),
                        next_lemma: after.and_then(|j| crate::treebank::tag::lemma_of(&children[j], lexicon)),
                        prev_choice: if before.is_none() { None } else { prev_choice },
                    };
                    out.push((format!("{} {ch}", book.name), Example { ctx, candidates, gold: g }));
                }
                prev_choice = Some(gold_candidate);
            }
        }
    }
    Ok(out)
}

/// `cargo xtask tagger-transfer` (3.4 Part 4): what Synodal gold would
/// buy, measured and not shipped. The overlay's chapters go into five
/// folds; for each fold a tagger is trained on the Old Church Slavonic
/// material (as `train-tagger`) plus the other four folds' overlay
/// examples and scored on the fold; the bundled OCS-only model is scored
/// on the same examples as the baseline. The shipped model stays the
/// OCS-only one.
pub fn transfer(args: &[String]) -> Result<(), Box<dyn Error>> {
    let epochs: usize = args.iter().position(|a| a == "--epochs").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(8);
    let root = crate::workspace_root();
    let sources = root.join("references/downloads");
    let artifacts = root.join("target/sources");
    let started = std::time::Instant::now();
    let ocs = Lexicon::ocs();
    let Some(train_ud) = crate::sources::ud::load_ud_proiel_train(&sources, &artifacts)? else {
        return Err("UD PROIEL absent under references/downloads (scripts/fetch-sources.sh)".into());
    };
    let Some(heldout) = crate::sources::ud::load_ud_proiel_heldout(&sources, &artifacts)? else {
        return Err("UD PROIEL absent".into());
    };
    let mut syntacticus = crate::sources::ud::load_syntacticus(&sources, &artifacts)?;
    let held: HashSet<String> = heldout.sentences.iter().map(|s| sentence_key(s)).collect();
    if let Some(s) = syntacticus.as_mut() {
        s.sentences.retain(|sent| !held.contains(&sentence_key(sent)));
    }
    let (mut ocs_examples, _, _) = examples(ocs, &train_ud);
    if let Some(s) = &syntacticus {
        ocs_examples.extend(examples(ocs, s).0);
    }
    let synodal = Lexicon::synodal();
    let overlay = overlay_examples(synodal)?;
    let mut chapters: Vec<String> = overlay.iter().map(|(c, _)| c.clone()).collect();
    chapters.sort();
    chapters.dedup();
    println!("OCS examples {}, overlay examples {} in {} chapters; loaded in {:.1?}", ocs_examples.len(), overlay.len(), chapters.len(), started.elapsed());
    let bundled = Tagger::bundled();
    let (b_right, b_n) = accuracy_pairs(&overlay, |e| bundled.choose(&e.ctx, &e.candidates).map(|(i, _)| i).unwrap_or(0));
    println!("the bundled OCS-only model on the overlay's examples: {b_right}/{b_n} = {:.2}%", pct(b_right, b_n));
    let folds = 5;
    let mut total = (0usize, 0usize);
    let mut total_ocs_only = (0usize, 0usize);
    for f in 0..folds {
        let test_chapters: Vec<&String> = chapters.iter().enumerate().filter(|(k, _)| k % folds == f).map(|(_, c)| c).collect();
        let test: Vec<&(String, Example)> = overlay.iter().filter(|(c, _)| test_chapters.contains(&c)).collect();
        let train_overlay: Vec<&Example> = overlay.iter().filter(|(c, _)| !test_chapters.contains(&c)).map(|(_, e)| e).collect();
        let mut with: Vec<&Example> = ocs_examples.iter().collect();
        with.extend(train_overlay.iter().copied());
        let tagger_with = train_on(&with, epochs);
        let ocs_only: Vec<&Example> = ocs_examples.iter().collect();
        let tagger_ocs = train_on(&ocs_only, epochs);
        let (r, n) = accuracy_pairs(&test.iter().map(|(c, e)| (c.clone(), Example { ctx: e.ctx.clone(), candidates: e.candidates.clone(), gold: e.gold })).collect::<Vec<_>>(), |e| tagger_with.choose(&e.ctx, &e.candidates).map(|(i, _)| i).unwrap_or(0));
        let (r0, _) = accuracy_pairs(&test.iter().map(|(c, e)| (c.clone(), Example { ctx: e.ctx.clone(), candidates: e.candidates.clone(), gold: e.gold })).collect::<Vec<_>>(), |e| tagger_ocs.choose(&e.ctx, &e.candidates).map(|(i, _)| i).unwrap_or(0));
        println!("fold {}: test {} ({} examples, {} overlay training examples): OCS + overlay {r}/{n} = {:.2}%; OCS only {r0}/{n} = {:.2}%", f + 1, test_chapters.iter().map(|c| c.as_str()).collect::<Vec<_>>().join(", "), n, train_overlay.len(), pct(r, n), pct(r0, n));
        total.0 += r;
        total.1 += n;
        total_ocs_only.0 += r0;
        total_ocs_only.1 += n;
    }
    println!("five-fold over the overlay: OCS + the other folds {}/{} = {:.2}%; OCS only, retrained the same way {}/{} = {:.2}%; the bundled model {:.2}% — measured, not shipped ({:.1?})", total.0, total.1, pct(total.0, total.1), total_ocs_only.0, total_ocs_only.1, pct(total_ocs_only.0, total_ocs_only.1), pct(b_right, b_n), started.elapsed());
    Ok(())
}

fn train_on(examples: &[&Example], epochs: usize) -> Tagger {
    let mut trainer = Trainer::default();
    let mut order: Vec<usize> = (0..examples.len()).collect();
    let mut seed: u64 = 0x2545F4914F6CDD1D;
    for _ in 0..epochs {
        for i in (1..order.len()).rev() {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            order.swap(i, (seed % (i as u64 + 1)) as usize);
        }
        for &i in &order {
            let e = examples[i];
            trainer.step(&e.ctx, &e.candidates, e.gold);
        }
    }
    trainer.finish()
}

fn accuracy_pairs(examples: &[(String, Example)], choose: impl Fn(&Example) -> usize) -> (usize, usize) {
    let right = examples.iter().filter(|(_, e)| choose(e) == e.gold).count();
    (right, examples.len())
}
