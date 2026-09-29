//! Versioned export of source text, spans and complete morphological candidates.
//! Reload recomputes and compares against the supplied immutable lexicon. It
//! never turns serialized identifiers into unchecked live analysis references.

use church_slavonic::{
    Lexicon, Recension,
    document::{
        AnalysisDocument, Backend, Candidate as DocumentCandidate, SegmentKind, UncertaintyReason,
    },
    matching::{MatchPolicy, MatchStep, Transformation},
    morphology::observations::{Observation, SourceAnnotation, SourceSpec, WitnessId, WitnessSpec},
    morphology::{CategoryAssertion, Derivation, Evidence, GenderAssertion, Model, OrthographyId},
    witness::{SourceAddress, Witness},
};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const MAX_RECORD_BYTES: usize = 16 * 1024 * 1024;
const MAX_SOURCE_BYTES: usize = church_slavonic::document::MAX_SOURCE_BYTES;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub version: u32,
    pub registered_witness: Option<WitnessId>,
    pub witness_spec: Option<WitnessSpec>,
    pub observation_source: Option<SourceSpec>,
    pub witness_observations: Vec<Observation>,
    pub witness_annotations: Vec<SourceAnnotation>,
    pub profile: String,
    pub model_data_sha256: Option<String>,
    pub engine_sha256: Option<String>,
    pub generation_policy_sha256: Option<String>,
    pub lexical_policy_sha256: Option<String>,
    pub evidence: Vec<Evidence>,
    pub verified_evidence: Vec<String>,
    pub source_sha256: String,
    pub segmentation: String,
    pub policy: String,
    pub source: String,
    pub address: Option<Address>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Address {
    pub witness: String,
    pub document: String,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
    pub start: usize,
    pub end: usize,
    pub kind: String,
    pub candidates: Vec<Candidate>,
    pub uncertainties: Vec<Uncertainty>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub lexeme: String,
    pub categories: Option<Vec<CategoryAssertion>>,
    pub lexical_genders: Option<Vec<GenderAssertion>>,
    pub lexical_claims: Option<Vec<church_slavonic::morphology::ReviewedLexicalClaim>>,
    pub pos: String,
    pub cell: String,
    pub alternative: Option<usize>,
    pub derivation: Option<Derivation>,
    pub generated: String,
    pub exact_bytes: bool,
    pub query_steps: Vec<Step>,
    pub generated_steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Uncertainty {
    pub lexeme: String,
    pub pos: String,
    pub cell: String,
    pub kind: String,
    pub details: Vec<String>,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: String,
    pub before: String,
    pub after: String,
}

fn step(value: &MatchStep) -> Step {
    let operation = match value.transformation {
        Transformation::UnicodeNormalization => "unicode-nfc",
        Transformation::Lowercase => "lowercase-nfc",
        Transformation::RemoveAccent => "remove-acute-grave-kamora-circumflex",
        Transformation::LegacyPrintProjection => "legacy-print-projection",
    };
    Step {
        operation: operation.into(),
        before: value.before.clone(),
        after: value.after.clone(),
    }
}

pub fn policy_name(policy: MatchPolicy) -> &'static str {
    match policy {
        MatchPolicy::Exact => "exact",
        MatchPolicy::UnicodeEquivalent => "unicode-equivalent",
        MatchPolicy::CaseInsensitive => "case-insensitive",
        MatchPolicy::AccentInsensitive => "accent-insensitive",
        MatchPolicy::LegacyOrthographic => "legacy-orthographic",
    }
}

pub fn parse_policy(name: &str) -> Result<MatchPolicy, RecordError> {
    match name {
        "exact" => Ok(MatchPolicy::Exact),
        "unicode-equivalent" => Ok(MatchPolicy::UnicodeEquivalent),
        "case-insensitive" => Ok(MatchPolicy::CaseInsensitive),
        "accent-insensitive" => Ok(MatchPolicy::AccentInsensitive),
        "legacy-orthographic" => Ok(MatchPolicy::LegacyOrthographic),
        _ => Err(RecordError::UnsupportedPolicy(name.into())),
    }
}

fn profile_name(recension: Recension) -> &'static str {
    match recension {
        Recension::Synodal => "legacy-synodal",
        Recension::OldChurchSlavonic => "legacy-ocs",
    }
}

impl Record {
    pub fn from_document(document: &AnalysisDocument<'_>) -> Self {
        let (profile, model_data_sha256, engine_sha256, evidence) = match document.backend() {
            Backend::Legacy(lexicon) => (
                profile_name(lexicon.recension()).into(),
                None,
                None,
                Vec::new(),
            ),
            Backend::Model { model, orthography } => (
                orthography.0.clone(),
                Some(model.data_sha256().into()),
                Some(model.engine_sha256().into()),
                model.evidence_records().cloned().collect(),
            ),
        };
        let (witness_spec, observation_source, witness_observations, witness_annotations) =
            match (document.backend(), document.registered_witness()) {
                (Backend::Model { model, .. }, Some(id)) => {
                    let archive = model.observations();
                    let spec = archive.witness_spec(id);
                    (
                        spec.cloned(),
                        spec.and_then(|s| archive.source(&s.source)).cloned(),
                        archive.observations_for(id).cloned().collect(),
                        archive.annotations_for(id).cloned().collect(),
                    )
                }
                _ => (None, None, Vec::new(), Vec::new()),
            };
        let mut record = Self {
            version: 9,
            lexical_policy_sha256: match document.backend() {
                Backend::Legacy(_) => None,
                Backend::Model { model, .. } => Some(model.lexical_policy_sha256().into()),
            },
            generation_policy_sha256: match document.backend() {
                Backend::Legacy(_) => None,
                Backend::Model { model, .. } => Some(model.generation_policy_sha256().into()),
            },
            registered_witness: document.registered_witness().cloned(),
            witness_spec,
            observation_source,
            witness_observations,
            witness_annotations,
            profile,
            model_data_sha256,
            engine_sha256,
            evidence,
            verified_evidence: match document.backend() {
                Backend::Legacy(_) => Vec::new(),
                Backend::Model { model, .. } => model
                    .verified_evidence_ids()
                    .map(|id| id.0.clone())
                    .collect(),
            },
            source_sha256: document.witness().sha256(),
            segmentation: AnalysisDocument::SEGMENTATION.into(),
            policy: policy_name(document.policy()).into(),
            source: document.witness().reproduce().into(),
            address: document.witness().address().map(|a| Address {
                witness: a.witness.clone(),
                document: a.document.clone(),
                unit: a.unit.clone(),
            }),
            segments: document
                .segments()
                .iter()
                .map(|s| Segment {
                    start: s.range().start,
                    end: s.range().end,
                    kind: match s.kind() {
                        SegmentKind::Whitespace => "whitespace",
                        SegmentKind::Punctuation => "punctuation",
                        SegmentKind::Text => "text",
                    }
                    .into(),
                    candidates: s
                        .candidates()
                        .iter()
                        .map(|c| Candidate {
                            lexeme: c.lexeme_id().into(),
                            categories: c.categories().map(<[_]>::to_vec),
                            lexical_genders: c.lexical_genders().map(<[_]>::to_vec),
                            lexical_claims: c.lexical_claims().map(<[_]>::to_vec),
                            pos: c.cell().pos().tag().into(),
                            cell: c.cell().name(),
                            alternative: match c {
                                DocumentCandidate::Legacy(a) => Some(a.analysis.alt),
                                DocumentCandidate::Modeled(_) => None,
                            },
                            derivation: match c {
                                DocumentCandidate::Legacy(_) => None,
                                DocumentCandidate::Modeled(a) => Some(a.derivation.clone()),
                            },
                            generated: c.generated().into(),
                            exact_bytes: &document.witness().reproduce()[s.range()]
                                == c.generated(),
                            query_steps: c.trace().query_steps.iter().map(step).collect(),
                            generated_steps: c.trace().generated_steps.iter().map(step).collect(),
                        })
                        .collect(),
                    uncertainties: s
                        .uncertainties()
                        .iter()
                        .map(|u| {
                            let (kind, details, evidence) = match &u.reason {
                                UncertaintyReason::MissingStems(stems) => {
                                    ("missing-stems", stems.clone(), Vec::new())
                                }
                                UncertaintyReason::UnverifiedRestriction(r) => (
                                    "unverified-restriction",
                                    vec![r.reason.clone()],
                                    r.evidence.iter().map(|e| e.0.clone()).collect(),
                                ),
                            };
                            Uncertainty {
                                lexeme: u.lexeme.clone(),
                                pos: u.cell.pos().tag().into(),
                                cell: u.cell.name(),
                                kind: kind.into(),
                                details,
                                evidence,
                            }
                        })
                        .collect(),
                })
                .collect(),
        };
        record.canonicalize();
        record
    }

    fn canonicalize(&mut self) {
        for segment in &mut self.segments {
            segment.candidates.sort();
            segment.uncertainties.sort();
        }
    }

    /// A record is evidence to validate, not authority to replace the lexicon.
    /// Changed, missing or additional candidates are rejected. Candidate order
    /// carries no selection semantics.
    fn inputs(&self, profile: &str) -> Result<(Witness, MatchPolicy), RecordError> {
        if self.version != 9 {
            return Err(RecordError::UnsupportedVersion(self.version));
        }
        if self.source.len() > MAX_SOURCE_BYTES {
            return Err(RecordError::SourceTooLarge);
        }
        if self.profile != profile {
            return Err(RecordError::ProfileMismatch);
        }
        if self.segmentation != AnalysisDocument::SEGMENTATION {
            return Err(RecordError::SegmentationMismatch);
        }
        let mut end = 0;
        for segment in &self.segments {
            if segment.start != end
                || segment.end <= segment.start
                || self.source.get(segment.start..segment.end).is_none()
            {
                return Err(RecordError::InvalidSpans);
            }
            end = segment.end;
        }
        if end != self.source.len() {
            return Err(RecordError::InvalidSpans);
        }
        let address = self.address.as_ref().map(|a| SourceAddress {
            witness: a.witness.clone(),
            document: a.document.clone(),
            unit: a.unit.clone(),
        });
        let witness = Witness::new(self.source.clone(), address);
        if self.source_sha256 != witness.sha256() {
            return Err(RecordError::SourceMismatch);
        }
        Ok((witness, parse_policy(&self.policy)?))
    }

    fn check(&self, document: &AnalysisDocument<'_>) -> Result<(), RecordError> {
        let mut supplied = self.clone();
        supplied.canonicalize();
        if supplied != Self::from_document(document) {
            return Err(RecordError::AnalysisMismatch);
        }
        Ok(())
    }

    pub fn resolve<'a>(&self, lexicon: &'a Lexicon) -> Result<AnalysisDocument<'a>, RecordError> {
        if self.model_data_sha256.is_some() || self.engine_sha256.is_some() || self.generation_policy_sha256.is_some() || self.lexical_policy_sha256.is_some() {
            return Err(RecordError::SnapshotMismatch);
        }
        let (witness, policy) = self.inputs(profile_name(lexicon.recension()))?;
        let document = AnalysisDocument::analyze(lexicon, witness, policy);
        self.check(&document)?;
        Ok(document)
    }

    pub fn resolve_model<'a>(&self, model: &'a Model) -> Result<AnalysisDocument<'a>, RecordError> {
        if self.model_data_sha256.as_deref() != Some(model.data_sha256())
            || self.engine_sha256.as_deref() != Some(model.engine_sha256())
            || self.generation_policy_sha256.as_deref() != Some(model.generation_policy_sha256())
            || self.lexical_policy_sha256.as_deref() != Some(model.lexical_policy_sha256())
        {
            return Err(RecordError::SnapshotMismatch);
        }
        let (witness, policy) = self.inputs(&self.profile)?;
        let document = match &self.registered_witness {
            Some(id) => AnalysisDocument::analyze_registered_witness(
                model,
                id.clone(),
                OrthographyId(self.profile.clone()),
                policy,
            ),
            None => AnalysisDocument::analyze_model(
                model,
                witness,
                OrthographyId(self.profile.clone()),
                policy,
            ),
        }
        .map_err(|e| RecordError::Model(e.to_string()))?;
        self.check(&document)?;
        Ok(document)
    }
}

#[derive(Debug)]
pub enum RecordError {
    RecordTooLarge,
    SourceTooLarge,
    InvalidJson(serde_json::Error),
    UnsupportedVersion(u32),
    UnsupportedPolicy(String),
    ProfileMismatch,
    SegmentationMismatch,
    InvalidSpans,
    AnalysisMismatch,
    SnapshotMismatch,
    SourceMismatch,
    Model(String),
}
impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "analysis document: {self:?}")
    }
}
impl std::error::Error for RecordError {}

pub fn to_json(document: &AnalysisDocument<'_>) -> Result<String, RecordError> {
    if document.witness().reproduce().len() > MAX_SOURCE_BYTES {
        return Err(RecordError::SourceTooLarge);
    }
    struct Buffer {
        bytes: Vec<u8>,
        exceeded: bool,
    }
    impl std::io::Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self
                .bytes
                .len()
                .checked_add(bytes.len())
                .is_none_or(|n| n > MAX_RECORD_BYTES)
            {
                self.exceeded = true;
                return Err(std::io::Error::other("analysis record byte limit"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Buffer {
        bytes: Vec::new(),
        exceeded: false,
    };
    if let Err(error) = serde_json::to_writer_pretty(&mut buffer, &Record::from_document(document))
    {
        return if buffer.exceeded {
            Err(RecordError::RecordTooLarge)
        } else {
            Err(RecordError::InvalidJson(error))
        };
    }
    String::from_utf8(buffer.bytes)
        .map_err(|_| RecordError::Model("serializer emitted invalid UTF-8".into()))
}

pub fn from_json<'a>(
    bytes: &[u8],
    lexicon: &'a Lexicon,
) -> Result<AnalysisDocument<'a>, RecordError> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(RecordError::RecordTooLarge);
    }
    let record: Record = serde_json::from_slice(bytes).map_err(RecordError::InvalidJson)?;
    record.resolve(lexicon)
}

pub fn from_json_model<'a>(
    bytes: &[u8],
    model: &'a Model,
) -> Result<AnalysisDocument<'a>, RecordError> {
    if bytes.len() > MAX_RECORD_BYTES {
        return Err(RecordError::RecordTooLarge);
    }
    let record: Record = serde_json::from_slice(bytes).map_err(RecordError::InvalidJson)?;
    record.resolve_model(model)
}
