use crate::{EvidenceId, Recension, RecensionMappingId, RuleId};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TraceStep {
    pub rule: RuleId,
    pub stage: String,
    pub input: String,
    pub output: String,
    pub source_recension: Option<Recension>,
    pub target_recension: Recension,
    pub mapping: Option<RecensionMappingId>,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RuleTrace {
    steps: Vec<TraceStep>,
}

impl RuleTrace {
    #[must_use]
    pub fn new(steps: Vec<TraceStep>) -> Self {
        Self { steps }
    }

    #[must_use]
    pub fn steps(&self) -> &[TraceStep] {
        &self.steps
    }

    pub fn push(&mut self, step: TraceStep) {
        self.steps.push(step);
    }
}
