//! Source-addressed morphological candidates, independent of contextual edits.
//!
//! Modeled candidates include registered abbreviation spellings, without
//! claiming exhaustive expansion. This layer does not infer clitic boundaries or
//! syntactic attachments. Its candidate sets are relative to the
//! supplied lexicon, matching policy and declared segmentation.

use crate::{
    Cell, Lexicon,
    analyze::MatchedAnalysis,
    matching::MatchPolicy,
    morphology::{Model, ModelAnalysis, ModelError, OrthographyId, Restriction},
    witness::{SourceSpan, Witness},
};
use std::ops::Range;
pub const MAX_SOURCE_BYTES: usize = 128 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    Whitespace,
    Punctuation,
    Text,
}

/// A simple surface segmentation, not a claim about syntactic words. Hyphens,
/// combining marks and unknown characters remain within text. Brackets are
/// retained as punctuation; no corpus-specific apparatus filtering is applied.
fn kind(ch: char) -> SegmentKind {
    if ch.is_whitespace() {
        SegmentKind::Whitespace
    } else if matches!(
        ch,
        ',' | '.' | ';' | ':' | '!' | '?' | '«' | '»' | '(' | ')' | '[' | ']' | '"'
    ) {
        SegmentKind::Punctuation
    } else {
        SegmentKind::Text
    }
}

pub(crate) fn is_text_segment(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|ch| kind(ch) == SegmentKind::Text)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Candidate<'a> {
    Legacy(MatchedAnalysis<'a>),
    Modeled(Box<ModelAnalysis<'a>>),
}
impl Candidate<'_> {
    /// Legacy candidates have no independently represented lexical category.
    pub fn categories(&self) -> Option<&[crate::morphology::CategoryAssertion]> {
        match self {
            Self::Legacy(_) => None,
            Self::Modeled(a) => Some(&a.lexeme.categories),
        }
    }
    /// Modeled lexical assertions, not a resolved agreement gender. Legacy
    /// entries do not expose evidence-backed gender assertions through this API.
    pub fn lexical_genders(&self) -> Option<&[crate::morphology::GenderAssertion]> {
        match self {
            Self::Legacy(_) => None,
            Self::Modeled(a) => Some(&a.lexeme.genders),
        }
    }
    pub fn lexical_claims(&self) -> Option<&[crate::morphology::ReviewedLexicalClaim]> {
        match self {
            Self::Legacy(_) => None,
            Self::Modeled(a) => Some(&a.lexical_claims),
        }
    }
    pub fn lexeme_id(&self) -> &str {
        match self {
            Self::Legacy(a) => &a.analysis.lexeme.id,
            Self::Modeled(a) => &a.lexeme.id.0,
        }
    }
    pub fn cell(&self) -> Cell {
        match self {
            Self::Legacy(a) => a.analysis.cell,
            Self::Modeled(a) => a.cell,
        }
    }
    pub fn generated(&self) -> &str {
        match self {
            Self::Legacy(a) => &a.analysis.print,
            Self::Modeled(a) => &a.derivation.surface,
        }
    }
    pub fn trace(&self) -> &crate::matching::MatchTrace {
        match self {
            Self::Legacy(a) => &a.trace,
            Self::Modeled(a) => &a.trace,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UncertaintyReason {
    MissingStems(Vec<String>),
    UnverifiedRestriction(Restriction),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uncertainty {
    pub lexeme: String,
    pub cell: Cell,
    pub reason: UncertaintyReason,
}

#[derive(Clone)]
pub enum Backend<'a> {
    Legacy(&'a Lexicon),
    Model {
        model: &'a Model,
        orthography: OrthographyId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment<'a> {
    range: Range<usize>,
    kind: SegmentKind,
    candidates: Vec<Candidate<'a>>,
    uncertainties: Vec<Uncertainty>,
}

impl<'a> Segment<'a> {
    pub fn uncertainties(&self) -> &[Uncertainty] {
        &self.uncertainties
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn kind(&self) -> SegmentKind {
        self.kind
    }
    /// Exact feature combinations are stored separately, including closed
    /// lexical alternatives and distinct alternatives within the same cell.
    pub fn candidates(&self) -> &[Candidate<'a>] {
        &self.candidates
    }
}

#[derive(Clone)]
pub struct AnalysisDocument<'a> {
    registered_witness: Option<crate::morphology::observations::WitnessId>,
    backend: Backend<'a>,
    witness: Witness,
    policy: MatchPolicy,
    segments: Vec<Segment<'a>>,
}

impl<'a> AnalysisDocument<'a> {
    pub const SEGMENTATION: &'static str = "whitespace-punctuation-v1";

    fn partition(witness: &Witness) -> Vec<Segment<'a>> {
        let mut segments: Vec<Segment<'a>> = Vec::new();
        for (offset, ch) in witness.reproduce().char_indices() {
            let current = kind(ch);
            if let Some(last) = segments.last_mut()
                && last.kind == current
            {
                last.range.end = offset + ch.len_utf8();
            } else {
                segments.push(Segment {
                    range: offset..offset + ch.len_utf8(),
                    kind: current,
                    candidates: Vec::new(),
                    uncertainties: Vec::new(),
                });
            }
        }
        segments
    }

    pub fn analyze(lexicon: &'a Lexicon, witness: Witness, policy: MatchPolicy) -> Self {
        let mut segments = Self::partition(&witness);
        for segment in &mut segments {
            if segment.kind == SegmentKind::Text {
                segment.candidates = lexicon
                    .analyze_with(&witness.reproduce()[segment.range.clone()], policy)
                    .into_iter()
                    .map(Candidate::Legacy)
                    .collect();
            }
        }
        Self {
            backend: Backend::Legacy(lexicon),
            registered_witness: None,
            witness,
            policy,
            segments,
        }
    }

    pub fn analyze_model(
        model: &'a Model,
        witness: Witness,
        orthography: OrthographyId,
        policy: MatchPolicy,
    ) -> Result<Self, ModelError> {
        if witness.reproduce().len() > MAX_SOURCE_BYTES {
            return Err(ModelError::ResourceLimit("document source bytes"));
        }
        if model.orthography(&orthography).is_none() {
            return Err(ModelError::MissingReference(orthography.0));
        }
        let mut segments = Self::partition(&witness);
        let mut checks = 0;
        let mut total_bytes = 0usize;
        let mut total_records = 0usize;
        for segment in &mut segments {
            if segment.kind != SegmentKind::Text {
                continue;
            }
            let result = model.analyze_budgeted(
                &witness.reproduce()[segment.range.clone()],
                &orthography,
                policy,
                &mut checks,
            )?;
            let bytes = result
                .candidates
                .iter()
                .map(|a| {
                    a.lexical_claims.iter().map(crate::morphology::ReviewedLexicalClaim::retained_bytes).sum::<usize>()
                        + a.lexeme.id.0.len()
                        + a.derivation
                            .source_annotations
                            .iter()
                            .map(|id| id.0.len())
                            .sum::<usize>()
                        + a.derivation
                            .abbreviation
                            .as_ref()
                            .map_or(0, |a| a.retained_bytes())
                        + a.lexeme
                            .categories
                            .iter()
                            .map(|c| 16 + c.evidence.iter().map(|e| e.0.len()).sum::<usize>())
                            .sum::<usize>()
                        + a.derivation.claim_sha256.len() + a.derivation.review.retained_bytes()
                        + a.derivation.stem_change.as_ref().map_or(0, |t| t.before.len() + t.after.len())
                        + a.derivation.underlying.len()
                        + a.derivation.surface.len()
                        + a.derivation.grammar.0.len()
                        + a.derivation.orthography.0.len()
                        + a.derivation.paradigm.0.len()
                        + a.derivation.rule.0.len()
                        + a.derivation.accent.accented.len()
                        + a.derivation
                            .evidence
                            .iter()
                            .map(|e| e.0.len())
                            .sum::<usize>()
                        + a.derivation
                            .spelling_steps
                            .iter()
                            .map(|(a, b)| a.len() + b.len())
                            .sum::<usize>()
                        + a.trace
                            .query_steps
                            .iter()
                            .chain(&a.trace.generated_steps)
                            .map(|s| s.before.len() + s.after.len())
                            .sum::<usize>()
                })
                .sum::<usize>()
                + result
                    .unresolved_cells
                    .iter()
                    .map(|(l, _, missing)| {
                        l.id.0.len() + missing.iter().map(String::len).sum::<usize>()
                    })
                    .sum::<usize>()
                + result
                    .unresolved_restrictions
                    .iter()
                    .map(|(l, r)| {
                        l.id.0.len()
                            + r.reason.len()
                            + r.evidence.iter().map(|e| e.0.len()).sum::<usize>()
                    })
                    .sum::<usize>();
            total_bytes = total_bytes
                .checked_add(bytes)
                .ok_or(ModelError::ResourceLimit("document result bytes"))?;
            total_records = total_records
                .checked_add(
                    result.candidates.len()
                        + result
                            .candidates
                            .iter()
                            .map(|a| a.derivation.source_annotations.len())
                            .sum::<usize>()
                        + result.unresolved_cells.len()
                        + result.unresolved_restrictions.len(),
                )
                .ok_or(ModelError::ResourceLimit("document records"))?;
            if total_bytes > model.limits().max_result_bytes
                || total_records > model.limits().max_candidates
            {
                return Err(ModelError::ResourceLimit("document results"));
            }
            segment.candidates = result
                .candidates
                .into_iter()
                .map(|candidate| Candidate::Modeled(Box::new(candidate)))
                .collect();
            segment
                .uncertainties
                .extend(
                    result
                        .unresolved_cells
                        .into_iter()
                        .map(|(lexeme, cell, stems)| Uncertainty {
                            lexeme: lexeme.id.0.clone(),
                            cell,
                            reason: UncertaintyReason::MissingStems(stems),
                        }),
                );
            segment
                .uncertainties
                .extend(
                    result
                        .unresolved_restrictions
                        .into_iter()
                        .map(|(lexeme, restriction)| Uncertainty {
                            lexeme: lexeme.id.0.clone(),
                            cell: restriction.cell,
                            reason: UncertaintyReason::UnverifiedRestriction(restriction),
                        }),
                );
        }
        Ok(Self {
            registered_witness: None,
            backend: Backend::Model { model, orthography },
            witness,
            policy,
            segments,
        })
    }

    pub fn backend(&self) -> &Backend<'a> {
        &self.backend
    }

    pub fn analyze_registered_witness(
        model: &'a Model,
        id: crate::morphology::observations::WitnessId,
        orthography: OrthographyId,
        policy: MatchPolicy,
    ) -> Result<Self, ModelError> {
        let witness = model
            .observations()
            .witness(&id)
            .ok_or_else(|| ModelError::MissingReference(id.0.clone()))?
            .clone();
        let mut document = Self::analyze_model(model, witness, orthography, policy)?;
        document.registered_witness = Some(id);
        Ok(document)
    }

    pub fn registered_witness(&self) -> Option<&crate::morphology::observations::WitnessId> {
        self.registered_witness.as_ref()
    }
    pub fn witness(&self) -> &Witness {
        &self.witness
    }
    pub fn policy(&self) -> MatchPolicy {
        self.policy
    }
    pub fn segments(&self) -> &[Segment<'a>] {
        &self.segments
    }
    /// Resolve only this document's segment indices, never a caller-supplied
    /// segment whose byte range could belong to a different witness.
    pub fn span(&self, index: usize) -> Option<SourceSpan<'_>> {
        self.witness.span(self.segments.get(index)?.range.clone())
    }
}
