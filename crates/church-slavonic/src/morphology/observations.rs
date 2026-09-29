//! Source observations and asserted analyses are separate records. Raw source
//! bytes are supplied by the caller and verified, never authenticated by a
//! serialized review flag. This first extraction operation is an exact UTF-8
//! byte slice; markup decoding and normalized projections require other mappings.
use super::*;
use crate::witness::{SourceAddress, Witness};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WitnessId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ObservationId(pub String);
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AnnotationId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpec {
    pub id: SourceId,
    pub uri: String,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitnessSpec {
    pub id: WitnessId,
    pub source: SourceId,
    pub start: usize,
    pub end: usize,
    pub location: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationKind {
    RunningText,
    PedagogicalExample,
    Heading,
    Apparatus,
    Unclassified,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub id: ObservationId,
    pub witness: WitnessId,
    pub start: usize,
    pub end: usize,
    pub surface: String,
    pub kind: ObservationKind,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationTarget {
    pub lexeme: LexemeId,
    #[serde(with = "super::cell_codec")]
    pub cell: Cell,
    pub orthography: OrthographyId,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAnnotation {
    pub id: AnnotationId,
    pub observation: ObservationId,
    /// Original annotation labels, including distinctions not yet mapped.
    pub original: BTreeMap<String, String>,
    /// None preserves an unmapped annotation; it does not delete the token.
    pub target: Option<AnnotationTarget>,
    pub evidence: Vec<EvidenceId>,
}
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationInput {
    pub sources: Vec<SourceSpec>,
    pub witnesses: Vec<WitnessSpec>,
    pub observations: Vec<Observation>,
    pub annotations: Vec<SourceAnnotation>,
}
#[derive(Debug, Default)]
pub struct Archive {
    sources: BTreeMap<SourceId, SourceSpec>,
    witnesses: BTreeMap<WitnessId, (WitnessSpec, Witness)>,
    observations: BTreeMap<ObservationId, Observation>,
    annotations: BTreeMap<AnnotationId, SourceAnnotation>,
    support: BTreeMap<(LexemeId, String, OrthographyId, String), Vec<AnnotationId>>,
}
impl Archive {
    pub(super) fn build(
        input: ObservationInput,
        raw: Vec<Vec<u8>>,
        limits: &ModelLimits,
    ) -> Result<Self, ModelError> {
        let mut bytes = 0usize;
        let mut charge = |n: usize| -> Result<(), ModelError> {
            bytes = bytes
                .checked_add(n)
                .ok_or(ModelError::ResourceLimit("observation bytes"))?;
            if bytes > limits.max_observation_bytes {
                return Err(ModelError::ResourceLimit("observation bytes"));
            }
            Ok(())
        };
        let count = raw.len().saturating_add(
            input
                .sources
                .len()
                .saturating_add(input.witnesses.len())
                .saturating_add(input.observations.len())
                .saturating_add(input.annotations.len()),
        );
        if count > limits.max_observation_records {
            return Err(ModelError::ResourceLimit("observation records"));
        }
        let mut material = BTreeMap::new();
        for data in raw {
            charge(data.len().saturating_add(64))?;
            let hash = format!("{:x}", Sha256::digest(&data));
            material.entry(hash).or_insert(data);
        }
        let mut archive = Self::default();
        for source in input.sources {
            charge(source.id.0.len() * 2 + source.uri.len() + source.sha256.len())?;
            if source.id.0.trim().is_empty() || source.uri.trim().is_empty() {
                return Err(ModelError::InvalidObservation(
                    "empty source identity".into(),
                ));
            }
            if !material.contains_key(&source.sha256) {
                return Err(ModelError::MissingSource(source.sha256));
            }
            let id = source.id.clone();
            if archive.sources.insert(id.clone(), source).is_some() {
                return Err(ModelError::Duplicate(id.0));
            }
        }
        for spec in input.witnesses {
            let source = archive
                .sources
                .get(&spec.source)
                .ok_or_else(|| ModelError::MissingReference(spec.source.0.clone()))?;
            let text = material[&source.sha256]
                .get(spec.start..spec.end)
                .and_then(|s| std::str::from_utf8(s).ok())
                .ok_or_else(|| ModelError::InvalidObservation(spec.id.0.clone()))?;
            if spec.id.0.trim().is_empty() || spec.location.is_empty() {
                return Err(ModelError::InvalidObservation(spec.id.0));
            }
            charge(
                text.len()
                    + spec.id.0.len() * 3
                    + spec.source.0.len()
                    + spec.location.len() * 2
                    + source.uri.len(),
            )?;
            let witness = Witness::new(
                text,
                Some(SourceAddress {
                    witness: spec.id.0.clone(),
                    document: source.uri.clone(),
                    unit: spec.location.clone(),
                }),
            );
            let id = spec.id.clone();
            if archive
                .witnesses
                .insert(id.clone(), (spec, witness))
                .is_some()
            {
                return Err(ModelError::Duplicate(id.0));
            }
        }
        for observation in input.observations {
            let (_, witness) = archive
                .witnesses
                .get(&observation.witness)
                .ok_or_else(|| ModelError::MissingReference(observation.witness.0.clone()))?;
            if observation.id.0.trim().is_empty()
                || observation.surface.is_empty()
                || witness.reproduce().get(observation.start..observation.end)
                    != Some(observation.surface.as_str())
            {
                return Err(ModelError::InvalidObservation(observation.id.0));
            }
            charge(
                observation.id.0.len() * 2
                    + observation.witness.0.len()
                    + observation.surface.len(),
            )?;
            let id = observation.id.clone();
            if archive
                .observations
                .insert(id.clone(), observation)
                .is_some()
            {
                return Err(ModelError::Duplicate(id.0));
            }
        }
        for annotation in input.annotations {
            let observation = archive
                .observations
                .get(&annotation.observation)
                .ok_or_else(|| ModelError::MissingReference(annotation.observation.0.clone()))?;
            if annotation.id.0.trim().is_empty() || annotation.original.is_empty() {
                return Err(ModelError::InvalidObservation(annotation.id.0));
            }
            charge(
                annotation.id.0.len() * 2
                    + annotation.observation.0.len()
                    + annotation
                        .original
                        .iter()
                        .map(|(k, v)| k.len() + v.len())
                        .sum::<usize>()
                    + annotation.evidence.iter().map(|e| e.0.len()).sum::<usize>(),
            )?;
            if let Some(target) = &annotation.target {
                charge(
                    target.lexeme.0.len() * 2
                        + target.orthography.0.len() * 2
                        + target.cell.name().len()
                        + observation.surface.len()
                        + annotation.id.0.len(),
                )?;
                archive
                    .support
                    .entry((
                        target.lexeme.clone(),
                        target.cell.name(),
                        target.orthography.clone(),
                        observation.surface.clone(),
                    ))
                    .or_default()
                    .push(annotation.id.clone());
            }
            let id = annotation.id.clone();
            if archive.annotations.insert(id.clone(), annotation).is_some() {
                return Err(ModelError::Duplicate(id.0));
            }
        }
        for ids in archive.support.values_mut() {
            ids.sort();
        }
        Ok(archive)
    }
    pub fn source(&self, id: &SourceId) -> Option<&SourceSpec> {
        self.sources.get(id)
    }
    pub fn witness(&self, id: &WitnessId) -> Option<&Witness> {
        self.witnesses.get(id).map(|(_, w)| w)
    }
    pub fn witness_spec(&self, id: &WitnessId) -> Option<&WitnessSpec> {
        self.witnesses.get(id).map(|(s, _)| s)
    }
    pub fn observation(&self, id: &ObservationId) -> Option<&Observation> {
        self.observations.get(id)
    }
    pub fn annotation(&self, id: &AnnotationId) -> Option<&SourceAnnotation> {
        self.annotations.get(id)
    }
    pub fn annotations(&self) -> impl Iterator<Item = &SourceAnnotation> {
        self.annotations.values()
    }
    pub fn observations_for(&self, witness: &WitnessId) -> impl Iterator<Item = &Observation> {
        self.observations
            .values()
            .filter(move |o| &o.witness == witness)
    }
    pub fn annotations_for(&self, witness: &WitnessId) -> impl Iterator<Item = &SourceAnnotation> {
        self.annotations
            .values()
            .filter(move |a| &self.observations[&a.observation].witness == witness)
    }
    pub(super) fn support(
        &self,
        lexeme: &LexemeId,
        cell: Cell,
        profile: &OrthographyId,
        surface: &str,
    ) -> &[AnnotationId] {
        self.support
            .get(&(lexeme.clone(), cell.name(), profile.clone(), surface.into()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}
