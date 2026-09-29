//! Given-lemma generation against unchanged mapped cells. This does not run
//! an analyzer, infer lexical identity, or reconstruct original source labels.
use crate::sources::ud::Corpus;
use church_slavonic::{Cell, Lexicon, Pos};
use serde::Serialize;
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Default, Serialize)]
pub struct Counts {
    pub total: usize,
    pub lemma_present: usize,
    pub cell_has_forms: usize,
    pub exact_surface: usize,
    pub unicode_equivalent_surface: usize,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub definition: &'static str,
    pub corpus: String,
    pub status: &'static str,
    pub lemma_policy: &'static str,
    pub limitations: &'static str,
    pub input_tokens: usize,
    pub unmapped_tokens: usize,
    pub tokens: Counts,
    pub slots: Counts,
    pub adapter_skips: std::collections::BTreeMap<String, u64>,
}

impl Counts {
    fn add(&mut self, flags: [bool; 4]) {
        self.total += 1;
        self.lemma_present += usize::from(flags[0]);
        self.cell_has_forms += usize::from(flags[1]);
        self.exact_surface += usize::from(flags[2]);
        self.unicode_equivalent_surface += usize::from(flags[3]);
    }
}

pub fn evaluate(lexicon: &Lexicon, corpus: &Corpus) -> Result<Report, String> {
    let input_tokens = corpus.sentences.iter().map(Vec::len).sum::<usize>();
    if input_tokens as u64 != corpus.tokens {
        return Err("declared token count differs from sequence population".into());
    }
    let mut used = vec![false; corpus.slots.len()];
    for token in corpus.sentences.iter().flatten() {
        for &index in &token.slots {
            let seen = used.get_mut(index).ok_or("invalid slot reference")?;
            if *seen {
                return Err("slot referenced more than once".into());
            }
            *seen = true;
            if corpus.slots[index].surface != token.surface {
                return Err("mapped slot surface differs from sequence surface".into());
            }
        }
    }
    if used.contains(&false) {
        return Err("unreferenced mapped slot".into());
    }
    // NFC is the only lemma equivalence here. In particular, neither arbitrary
    // personal pronouns nor guessed lemmas enter this candidate population.
    let mut by_lemma: HashMap<(Pos, String), Vec<usize>> = HashMap::new();
    let lexemes: Vec<_> = lexicon.iter().collect();
    for (index, lexeme) in lexemes.iter().enumerate() {
        by_lemma
            .entry((lexeme.pos, lexeme.lemma.nfc().collect()))
            .or_default()
            .push(index);
    }
    let mut cache: HashMap<(usize, Cell), Vec<String>> = HashMap::new();
    let mut report = Report {
        definition: "given-lemma-generation-mapped-cells-v1",
        corpus: corpus.label.into(),
        status: if corpus.slots.is_empty() {
            "unavailable"
        } else {
            "available"
        },
        lemma_policy: "same POS and NFC-equivalent displayed lemma; all matching lexemes; no guessing",
        limitations: "Regression diagnostic conditional on annotated mapped lemmas and cells, not analysis accuracy or adjudicated lexical identity. Adapter-cleaned surfaces and labels, not original witness bytes. No extra cells, spelling tolerances, or accent removal. Source/lexicon independence unestablished. Token hits accept any mapped alternative; slot counts report alternatives separately.",
        input_tokens,
        unmapped_tokens: 0,
        tokens: Counts::default(),
        slots: Counts::default(),
        adapter_skips: corpus
            .skipped
            .iter()
            .map(|(k, v)| ((*k).into(), *v))
            .collect(),
    };
    for token in corpus.sentences.iter().flatten() {
        if token.slots.is_empty() {
            report.unmapped_tokens += 1;
            continue;
        }
        let mut token_flags = [false; 4];
        for &slot_index in &token.slots {
            let slot = &corpus.slots[slot_index];
            let mut flags = [false; 4];
            if let Some(indices) = by_lemma.get(&(slot.pos, slot.lemma.nfc().collect())) {
                flags[0] = true;
                let nfc_surface: String = slot.surface.nfc().collect();
                for &index in indices {
                    let forms = cache.entry((index, slot.cell)).or_insert_with(|| {
                        lexemes[index]
                            .forms(slot.cell)
                            .iter()
                            .map(|f| f.print(lexicon.recension()))
                            .collect()
                    });
                    flags[1] |= !forms.is_empty();
                    flags[2] |= forms.iter().any(|f| f == &slot.surface);
                    flags[3] |= forms.iter().any(|f| f.nfc().eq(nfc_surface.chars()));
                }
            }
            report.slots.add(flags);
            for (total, flag) in token_flags.iter_mut().zip(flags) {
                *total |= flag;
            }
        }
        report.tokens.add(token_flags);
    }
    Ok(report)
}

pub fn run(kind: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root = crate::workspace_root();
    let sources = root.join("references/downloads");
    let artifacts = root.join("target/sources");
    let corpus = match kind {
        "ud-heldout" => crate::sources::ud::load_ud_proiel_heldout(&sources, &artifacts)?,
        "syntacticus" => crate::sources::ud::load_syntacticus(&sources, &artifacts)?,
        _ => return Err("eval-generation <ud-heldout|syntacticus>".into()),
    }
    .ok_or("requested corpus absent")?;
    let report = evaluate(Lexicon::ocs(), &corpus)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if report.status == "unavailable" {
        return Err("no mapped evaluation targets".into());
    }
    Ok(())
}
