//! The statistical layer of homonymy (V2.2 Part 5): an averaged
//! perceptron that scores the readings the analyzer returns for a token
//! — (part of speech, cell) — from the token's folded surface, its
//! suffixes, the neighbouring surfaces and the previous choice. Trained
//! on the gold morphology of the Old Church Slavonic treebanks (UD
//! PROIEL train, Syntacticus) by `cargo xtask train-tagger`, applied to
//! the Synodal Bible through the manuscript fold that makes the two
//! spellings look alike. The tagger is asked only where the constraint
//! layer left several readings; its choice is written on the leaf as
//! `:by tagger :p 0.87` and never counted as analysed.
//!
//! The model is a committed binary, `data/models/tagger.bin`: a list of
//! (feature hash, weight) pairs, the feature strings hashed by FNV-1a to
//! 64 bits (no collision among the model's features is checked at
//! training). Surfaces enter the features through [`fold`], never a
//! lexeme id, so the model transfers between the recensions (their ids
//! differ).

use church_slavonic::cell::{Cell, Pos};
use std::collections::HashMap;

/// The manuscript-lax spelling key both recensions pass through: the
/// accent-blind comparison key, then the scribal interchanges (шт ~ щ,
/// ѣ/ѧ ~ е, ю ~ у, the jers dropped, doubled vowels contracted).
pub fn fold(word: &str) -> String {
    let folded: String = church_slavonic::orthography::comparison_key(word)
        .replace("шт", "щ")
        .replace("шч", "щ")
        .chars()
        .filter_map(|c| match c {
            'ъ' | 'ь' | '\'' | 'ʼ' | '-' | '\u{2e2f}' => None,
            'ѣ' | 'ⱕ' | 'ѧ' | 'ꙗ' => Some('е'),
            'ю' | 'ѫ' | 'ѭ' => Some('у'),
            'ѩ' => Some('е'),
            'ꙑ' => Some('ы'),
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

/// The whole-word key: [`fold`] with the jer vowels of the Synodal print
/// (е, о between consonants: де́нь ~ дьнь, сотворѝ ~ сътвори) removed.
/// An е or о at an edge or beside a vowel is a real vowel and stays.
pub fn fold_word(word: &str) -> String {
    let chars: Vec<char> = fold(word).chars().collect();
    let vowel = |c: char| matches!(c, 'а' | 'е' | 'и' | 'о' | 'у' | 'ы');
    chars
        .iter()
        .enumerate()
        .filter(|(i, c)| !matches!(**c, 'е' | 'о') || *i == 0 || *i + 1 == chars.len() || vowel(chars[i - 1]) || vowel(chars[i + 1]))
        .map(|(_, c)| *c)
        .collect()
}

/// One reading the tagger may choose: its part of speech and cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    pub pos: Pos,
    pub cell: Cell,
}

/// The context a token is scored in: its surface, the surfaces beside it
/// and the reading chosen for the previous token.
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub surface: String,
    pub prev: Option<String>,
    pub next: Option<String>,
    /// the neighbours' lemmas where they are known (an analyzed leaf, a
    /// closed-class word; the gold lemma in training): the abbreviations
    /// and the jer spellings differ between the recensions, the lemmas
    /// do not
    pub prev_lemma: Option<String>,
    pub next_lemma: Option<String>,
    pub prev_choice: Option<Candidate>,
}

fn suffix(word: &str, n: usize) -> String {
    let chars: Vec<char> = word.chars().collect();
    chars[chars.len().saturating_sub(n)..].iter().collect()
}

/// The feature strings of a candidate in a context.
pub fn features(ctx: &Context, c: &Candidate) -> Vec<String> {
    let cell = c.cell.name();
    let pos = c.pos.tag();
    let w = fold_word(&ctx.surface);
    let sw = fold(&ctx.surface);
    let case = c.cell.case().map(church_slavonic::cell::case_name).unwrap_or("-");
    let num = c.cell.number().map(church_slavonic::cell::number_name).unwrap_or("-");
    let prev = ctx.prev.as_deref().map(fold_word).unwrap_or_else(|| "<s>".to_string());
    let next = ctx.next.as_deref().map(fold_word).unwrap_or_else(|| "</s>".to_string());
    let prev_lemma = ctx.prev_lemma.as_deref().map(fold_word);
    let next_lemma = ctx.next_lemma.as_deref().map(fold_word);
    let (pc, pp) = match &ctx.prev_choice {
        Some(p) => (p.cell.name(), p.pos.tag().to_string()),
        None => ("<s>".to_string(), "<s>".to_string()),
    };
    let mut f = vec![
        format!("c={cell}"),
        format!("p={pos}"),
        format!("p={pos}|case={case}"),
        format!("p={pos}|num={num}"),
        format!("w={w}|c={cell}"),
        format!("w={w}|p={pos}"),
        format!("s1={}|c={cell}", suffix(&sw, 1)),
        format!("s2={}|c={cell}", suffix(&sw, 2)),
        format!("s3={}|c={cell}", suffix(&sw, 3)),
        format!("s2={}|p={pos}", suffix(&sw, 2)),
        format!("pw={prev}|c={cell}"),
        format!("pw={prev}|p={pos}"),
        format!("pw={prev}|case={case}"),
        format!("nw={next}|c={cell}"),
        format!("nw={next}|p={pos}"),
        format!("nw={next}|case={case}"),
        format!("pc={pc}|c={cell}"),
        format!("pp={pp}|p={pos}"),
        format!("pp={pp}|case={case}"),
    ];
    if let Some(l) = &prev_lemma {
        f.push(format!("pl={l}|c={cell}"));
        f.push(format!("pl={l}|p={pos}"));
        f.push(format!("pl={l}|case={case}"));
    }
    if let Some(l) = &next_lemma {
        f.push(format!("nl={l}|c={cell}"));
        f.push(format!("nl={l}|p={pos}"));
        f.push(format!("nl={l}|case={case}"));
    }
    if let Cell::Verb(church_slavonic::cell::VerbCell::Finite { person, .. }) = c.cell {
        f.push(format!("pw={prev}|person={}", church_slavonic::cell::person_name(person)));
        f.push(format!("nw={next}|person={}", church_slavonic::cell::person_name(person)));
    }
    f
}

/// The 64-bit FNV-1a hash a feature string is stored under.
pub fn feature_id(feature: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in feature.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The model: feature hash → weight.
#[derive(Debug, Default, Clone)]
pub struct Tagger {
    pub weights: HashMap<u64, f32>,
}

impl Tagger {
    /// The committed model, or an empty tagger when none is built yet
    /// (`cargo xtask train-tagger` writes `data/models/tagger.bin`).
    pub fn bundled() -> Tagger {
        static MODEL: &[u8] = include_bytes!("../../../data/models/tagger.bin");
        Tagger::from_bytes(MODEL).unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }

    /// Set a feature's weight by its string (tests and hand models).
    pub fn set(&mut self, feature: &str, weight: f32) {
        self.weights.insert(feature_id(feature), weight);
    }

    /// The score of a candidate in a context.
    pub fn score(&self, ctx: &Context, c: &Candidate) -> f32 {
        features(ctx, c).iter().map(|f| self.weights.get(&feature_id(f)).copied().unwrap_or(0.0)).sum()
    }

    /// The best candidate and its softmax probability among the
    /// candidates; `None` for an empty list or when the model has no
    /// preference (the best score shared by several candidates).
    pub fn choose(&self, ctx: &Context, candidates: &[Candidate]) -> Option<(usize, f32)> {
        if candidates.is_empty() {
            return None;
        }
        let scores: Vec<f32> = candidates.iter().map(|c| self.score(ctx, c)).collect();
        let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        if scores.iter().filter(|s| **s == max).count() > 1 {
            return None;
        }
        let best = scores.iter().position(|s| *s == max)?;
        let denom: f32 = scores.iter().map(|s| (s - max).exp()).sum();
        Some((best, 1.0 / denom))
    }

    /// The binary format: the magic `CST1`, a little-endian u32 count,
    /// then per feature its u64 hash and f32 weight, sorted by hash.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::from(*b"CST1");
        let mut items: Vec<(&u64, &f32)> = self.weights.iter().filter(|(_, w)| **w != 0.0).collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        out.extend_from_slice(&(items.len() as u32).to_le_bytes());
        for (k, w) in items {
            out.extend_from_slice(&k.to_le_bytes());
            out.extend_from_slice(&w.to_le_bytes());
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Tagger> {
        if bytes.len() < 8 || &bytes[0..4] != b"CST1" {
            return None;
        }
        let n = u32::from_le_bytes(bytes[4..8].try_into().ok()?) as usize;
        let mut at = 8;
        let mut weights = HashMap::with_capacity(n);
        for _ in 0..n {
            let k = u64::from_le_bytes(bytes.get(at..at + 8)?.try_into().ok()?);
            at += 8;
            let w = f32::from_le_bytes(bytes.get(at..at + 4)?.try_into().ok()?);
            at += 4;
            weights.insert(k, w);
        }
        Some(Tagger { weights })
    }
}

/// The averaged perceptron's training state.
#[derive(Debug, Default)]
pub struct Trainer {
    weights: HashMap<u64, f32>,
    totals: HashMap<u64, f32>,
    stamps: HashMap<u64, u64>,
    /// hash → the feature string first seen under it (a collision is a
    /// training error)
    names: HashMap<u64, String>,
    steps: u64,
}

impl Trainer {
    fn update(&mut self, feature: &str, delta: f32) {
        let id = feature_id(feature);
        let known = self.names.entry(id).or_insert_with(|| feature.to_string());
        assert!(known == feature, "feature hash collision: {known:?} and {feature:?}");
        let steps = self.steps;
        let w = self.weights.entry(id).or_default();
        let stamp = self.stamps.entry(id).or_default();
        let total = self.totals.entry(id).or_default();
        *total += (steps - *stamp) as f32 * *w;
        *stamp = steps;
        *w += delta;
    }

    /// One training example: the gold candidate index among the
    /// candidates in a context; a wrong prediction moves the weights.
    /// Returns whether the prediction was right.
    pub fn step(&mut self, ctx: &Context, candidates: &[Candidate], gold: usize) -> bool {
        self.steps += 1;
        let scores: Vec<f32> = candidates.iter().map(|c| features(ctx, c).iter().map(|f| self.weights.get(&feature_id(f)).copied().unwrap_or(0.0)).sum()).collect();
        let (pred, _) = scores.iter().enumerate().fold((0, f32::NEG_INFINITY), |acc, (i, s)| if *s > acc.1 { (i, *s) } else { acc });
        if pred == gold {
            return true;
        }
        for f in features(ctx, &candidates[gold]) {
            self.update(&f, 1.0);
        }
        for f in features(ctx, &candidates[pred]) {
            self.update(&f, -1.0);
        }
        false
    }

    /// The averaged weights.
    pub fn finish(mut self) -> Tagger {
        let steps = self.steps.max(1);
        let mut out = HashMap::new();
        let keys: Vec<u64> = self.weights.keys().copied().collect();
        for k in keys {
            let w = self.weights[&k];
            let stamp = self.stamps.get(&k).copied().unwrap_or(0);
            let total = self.totals.get(&k).copied().unwrap_or(0.0) + (steps - stamp) as f32 * w;
            let avg = total / steps as f32;
            if avg != 0.0 {
                out.insert(k, avg);
            }
        }
        self.weights.clear();
        Tagger { weights: out }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_bytes_round_trip() {
        let mut t = Tagger::default();
        t.set("c=nom.sg", 1.5);
        t.set("w=землѧ|c=nom.sg", -0.25);
        let back = Tagger::from_bytes(&t.to_bytes()).expect("the bytes round-trip");
        assert_eq!(back.weights, t.weights);
    }

    #[test]
    fn the_fold_makes_the_recensions_alike() {
        assert_eq!(fold("землѧ̀"), fold("землꙗ"));
        assert_eq!(fold("свѧта́агѡ"), fold("свѧтаго"));
        assert_eq!(fold_word("де́нь"), fold_word("дьнь"));
        assert_eq!(fold_word("сотворѝ"), fold_word("сътвори"));
        // lossy by design: the jer positions go on both sides alike
        assert_eq!(fold_word("сотворѝ"), "стври");
    }
}
