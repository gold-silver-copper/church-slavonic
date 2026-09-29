//! Surface-only sequential tagger inference and a separate scoring pass.
//! This measures raw lexical lookup plus the feature tagger, not the legacy
//! syntactic-constraint composition. No gold field enters `predict`.
use crate::sources::ud::Corpus;
use church_slavonic::{
    Lexicon,
    matching::{self, MatchPolicy},
};
use church_slavonic_tagger::{Candidate, Context, Tagger};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalReading {
    pub lexeme: String,
    pub lemma: String,
    pub feature: Candidate,
}
#[derive(Debug, Clone)]
pub struct Prediction {
    context: Context,
    candidates: Vec<Candidate>,
    lexical_readings: Vec<LexicalReading>,
    choice: Option<usize>,
    /// A share among the supplied candidates, not a calibrated probability.
    softmax_share: Option<f32>,
}
impl Prediction {
    pub fn context(&self) -> &Context {
        &self.context
    }
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }
    pub fn lexical_readings(&self) -> &[LexicalReading] {
        &self.lexical_readings
    }
    pub fn selected_feature(&self) -> Option<Candidate> {
        self.choice.map(|i| self.candidates[i])
    }
    pub fn softmax_share(&self) -> Option<f32> {
        self.softmax_share
    }
}
fn unique_lemma(readings: &[LexicalReading], choice: Option<Candidate>) -> Option<String> {
    let mut lemmas = readings
        .iter()
        .filter(|r| choice.is_none_or(|c| c == r.feature))
        .map(|r| &r.lemma);
    let first = lemmas.next()?;
    lemmas.all(|l| l == first).then(|| first.clone())
}
/// Only surface strings, an explicit lexicon/matching policy, and the model
/// are available here. The returned candidates retain lexical ambiguity even
/// though the legacy scorer itself chooses only (POS, cell).
pub fn predict(
    lexicon: &Lexicon,
    tagger: &Tagger,
    surfaces: &[String],
    policy: MatchPolicy,
) -> Vec<Prediction> {
    let lookup: Vec<Vec<LexicalReading>> = surfaces
        .iter()
        .map(|surface| {
            let mut readings: Vec<_> = lexicon
                .analyze_with(surface, policy)
                .into_iter()
                .map(|a| LexicalReading {
                    lexeme: a.analysis.lexeme.id.clone(),
                    lemma: a.analysis.lexeme.lemma.clone(),
                    feature: Candidate {
                        pos: a.analysis.lexeme.pos,
                        cell: a.analysis.cell,
                    },
                })
                .collect();
            readings
                .sort_by_key(|r| (r.lexeme.clone(), r.feature.pos.tag(), r.feature.cell.name()));
            readings.dedup();
            readings
        })
        .collect();
    let mut predictions: Vec<Prediction> = Vec::new();
    for (i, surface) in surfaces.iter().enumerate() {
        let mut candidates: Vec<_> = lookup[i].iter().map(|r| r.feature).collect();
        candidates.sort_by_key(|c| (c.pos.tag(), c.cell.name()));
        candidates.dedup();
        let previous = predictions.last();
        let previous_choice = previous.and_then(|p| p.choice.map(|c| p.candidates[c]));
        let context = Context {
            surface: surface.clone(),
            prev: i.checked_sub(1).map(|j| surfaces[j].clone()),
            next: surfaces.get(i + 1).cloned(),
            prev_lemma: i
                .checked_sub(1)
                .and_then(|j| unique_lemma(&lookup[j], previous_choice)),
            next_lemma: lookup.get(i + 1).and_then(|r| unique_lemma(r, None)),
            prev_choice: previous_choice,
        };
        let selection = tagger
            .choose(&context, &candidates)
            .filter(|(_, p)| p.is_finite());
        predictions.push(Prediction {
            context,
            candidates,
            lexical_readings: lookup[i].clone(),
            choice: selection.map(|(i, _)| i),
            softmax_share: selection.map(|(_, p)| p),
        });
    }
    predictions
}

#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub definition: &'static str,
    pub annotation_basis: &'static str,
    pub lemma_match_policy: &'static str,
    pub matching_policy: String,
    pub corpus_label: String,
    pub declared_tokens: u64,
    pub input_tokens: usize,
    pub tokens_without_mapped_gold: usize,
    pub mapped_gold_tokens: usize,
    pub feature_gold_retained: usize,
    /// Candidate retention under Unicode-equivalent displayed-lemma matching.
    /// This is not source-to-opaque-lexeme identity adjudication.
    pub lemma_cell_retained: usize,
    pub no_candidates: usize,
    pub selected_features: usize,
    pub correct_selected_features: usize,
    pub abstained: usize,
    pub adapter_skips: std::collections::BTreeMap<String, u64>,
    pub confidence_bins: Vec<ConfidenceBin>,
    /// Same tokens, candidates and exact feature targets; only context is
    /// supplied from annotations. Never used to populate `predictions`.
    pub oracle_context_diagnostic: OracleDiagnostic,
}
#[derive(Debug, Default, Serialize)]
pub struct OracleDiagnostic {
    pub selected_features: usize,
    pub correct_selected_features: usize,
    pub abstained: usize,
}
#[derive(Debug, Default, Serialize)]
pub struct ConfidenceBin {
    pub lower_tenth: usize,
    pub selected_ambiguous_tokens: usize,
    pub correct_features: usize,
}
pub struct Evaluation {
    pub report: Report,
    pub predictions: Vec<Vec<Prediction>>,
}

/// Predict all tokens before consulting annotations. Missing correct candidates
/// stay in the mapped-gold denominator; unsupported/unmapped tokens have their
/// own visible count. Source adapters may already have lost distinctions, which
/// are not reconstructed or silently folded again here.
pub fn evaluate(
    lexicon: &Lexicon,
    tagger: &Tagger,
    corpus: &Corpus,
    policy: MatchPolicy,
) -> Result<Evaluation, String> {
    // Every mapped annotation must belong to exactly one input token. Surface
    // equality alone cannot detect reused slots on repeated words or orphans.
    let mut used_slots = vec![false; corpus.slots.len()];
    for token in corpus.sentences.iter().flatten() {
        for &slot in &token.slots {
            let used = used_slots
                .get_mut(slot)
                .ok_or_else(|| format!("invalid gold slot {slot}"))?;
            if *used {
                return Err(format!("gold slot {slot} referenced more than once"));
            }
            *used = true;
        }
    }
    if used_slots.iter().any(|used| !used) {
        return Err("unreferenced gold slot in corpus".into());
    }
    let predictions: Vec<_> = corpus
        .sentences
        .iter()
        .map(|sentence| {
            let surfaces: Vec<_> = sentence.iter().map(|t| t.surface.clone()).collect();
            predict(lexicon, tagger, &surfaces, policy)
        })
        .collect();
    let mut report = Report {
        definition: "raw-lookup-sequential-feature-tagger-v1",
        annotation_basis: "existing CorpusSlot mappings; no additional gold-cell tolerances; not independently validated gold",
        lemma_match_policy: "Unicode-equivalent displayed lemma plus exact POS/cell; not opaque lexeme identification",
        matching_policy: crate::analysis_document::policy_name(policy).into(),
        corpus_label: corpus.label.into(),
        declared_tokens: corpus.tokens,
        adapter_skips: corpus
            .skipped
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        confidence_bins: (0..10)
            .map(|lower_tenth| ConfidenceBin {
                lower_tenth,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    for (sentence, predicted) in corpus.sentences.iter().zip(&predictions) {
        for (position, (token, prediction)) in sentence.iter().zip(predicted).enumerate() {
            report.input_tokens += 1;
            if prediction.candidates.is_empty() {
                report.no_candidates += 1;
            }
            if token.slots.is_empty() {
                report.tokens_without_mapped_gold += 1;
                continue;
            }
            report.mapped_gold_tokens += 1;
            let gold = token
                .slots
                .iter()
                .map(|&i| {
                    corpus
                        .slots
                        .get(i)
                        .ok_or_else(|| format!("invalid gold slot {i}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if gold.iter().any(|g| g.surface != token.surface) {
                return Err("gold/token surface alignment mismatch".into());
            }
            let correct = |c: &Candidate| gold.iter().any(|g| g.pos == c.pos && g.cell == c.cell);
            let annotated_lemma = |i: usize| {
                sentence
                    .get(i)
                    .and_then(|t| (!t.lemma.is_empty() && t.lemma != "_").then(|| t.lemma.clone()))
            };
            let previous_gold = position.checked_sub(1).and_then(|i| {
                let mut choices = sentence[i]
                    .slots
                    .iter()
                    .filter_map(|&s| corpus.slots.get(s))
                    .map(|s| Candidate {
                        pos: s.pos,
                        cell: s.cell,
                    });
                let first = choices.next()?;
                choices.all(|c| c == first).then_some(first)
            });
            let oracle_context = Context {
                prev_lemma: position.checked_sub(1).and_then(annotated_lemma),
                next_lemma: annotated_lemma(position + 1),
                prev_choice: previous_gold,
                ..prediction.context.clone()
            };
            if let Some((choice, _)) = tagger
                .choose(&oracle_context, &prediction.candidates)
                .filter(|(_, p)| p.is_finite())
            {
                report.oracle_context_diagnostic.selected_features += 1;
                report.oracle_context_diagnostic.correct_selected_features +=
                    usize::from(correct(&prediction.candidates[choice]));
            } else {
                report.oracle_context_diagnostic.abstained += 1;
            }
            if prediction.candidates.iter().any(correct) {
                report.feature_gold_retained += 1;
            }
            if prediction.lexical_readings.iter().any(|r| {
                gold.iter().any(|g| {
                    g.pos == r.feature.pos
                        && g.cell == r.feature.cell
                        && matching::compare(&g.lemma, &r.lemma, MatchPolicy::UnicodeEquivalent)
                            .is_some()
                })
            }) {
                report.lemma_cell_retained += 1;
            }
            if let Some(choice) = prediction.choice {
                report.selected_features += 1;
                let right = correct(&prediction.candidates[choice]);
                report.correct_selected_features += usize::from(right);
                if prediction.candidates.len() > 1 {
                    let tenth = (prediction.softmax_share.unwrap_or(0.0) * 10.0).floor() as usize;
                    let bin = &mut report.confidence_bins[tenth.min(9)];
                    bin.selected_ambiguous_tokens += 1;
                    bin.correct_features += usize::from(right);
                }
            } else {
                report.abstained += 1;
            }
        }
    }
    if report.declared_tokens != report.input_tokens as u64 {
        return Err("corpus token accounting mismatch".into());
    }
    if report.input_tokens == 0 || report.mapped_gold_tokens == 0 {
        return Err("evaluation unavailable: no tokens with mapped source labels".into());
    }
    Ok(Evaluation {
        report,
        predictions,
    })
}

pub fn prediction_json(p: &Prediction) -> serde_json::Value {
    let feature = |c: Candidate| serde_json::json!({"pos":c.pos.tag(),"cell":c.cell.name()});
    serde_json::json!({
        "context":{"surface":p.context.surface,"prev":p.context.prev,"next":p.context.next,"prev_lemma":p.context.prev_lemma,"next_lemma":p.context.next_lemma,"prev_choice":p.context.prev_choice.map(feature)},
        "candidates":p.candidates.iter().copied().map(feature).collect::<Vec<_>>(),
        "lexical_readings":p.lexical_readings.iter().map(|r|serde_json::json!({"lexeme":r.lexeme,"lemma":r.lemma,"feature":feature(r.feature)})).collect::<Vec<_>>(),
        "selected_feature":p.choice.map(|i|feature(p.candidates[i])),"softmax_share":p.softmax_share
    })
}
