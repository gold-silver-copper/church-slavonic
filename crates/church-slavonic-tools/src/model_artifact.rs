//! Runtime loading of the evidence-bearing model; no embedded lexicon fallback.
use church_slavonic::{
    Cell, Pos,
    matching::MatchPolicy,
    morphology::{Availability, LexemeId, Model, ModelError, ModelInput, OrthographyId},
};
use serde::{Deserialize, Serialize};
use std::{fmt, io::Read, path::Path};

pub const MAX_ARTIFACT_BYTES: usize = 16 * 1024 * 1024;
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub version: u32,
    pub input: ModelInput,
}

#[derive(Debug)]
pub enum ArtifactError {
    TooLarge,
    Version(u32),
    Json(serde_json::Error),
    Model(ModelError),
    Io(std::io::Error),
    InvalidFixture(String),
}
impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "model artifact: {self:?}")
    }
}
impl std::error::Error for ArtifactError {}

pub fn from_bytes(bytes: &[u8]) -> Result<Model, ArtifactError> {
    from_bytes_with_sources(bytes, Vec::new())
}

pub fn from_bytes_with_sources(
    bytes: &[u8],
    sources: Vec<Vec<u8>>,
) -> Result<Model, ArtifactError> {
    if bytes.len() > MAX_ARTIFACT_BYTES {
        return Err(ArtifactError::TooLarge);
    }
    let artifact: Artifact = serde_json::from_slice(bytes).map_err(ArtifactError::Json)?;
    if artifact.version != 4 {
        return Err(ArtifactError::Version(artifact.version));
    }
    Model::build_with_sources(artifact.input, Default::default(), sources)
        .map_err(ArtifactError::Model)
}

pub fn read_bounded(path: &Path) -> Result<Vec<u8>, ArtifactError> {
    let file = std::fs::File::open(path).map_err(ArtifactError::Io)?;
    let mut bytes = Vec::new();
    file.take(MAX_ARTIFACT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(ArtifactError::Io)?;
    if bytes.len() > MAX_ARTIFACT_BYTES {
        return Err(ArtifactError::TooLarge);
    }
    Ok(bytes)
}

pub fn load(path: &Path) -> Result<Model, ArtifactError> {
    from_bytes(&read_bounded(path)?)
}

pub fn load_with_sources(
    path: &Path,
    sources: &[std::path::PathBuf],
) -> Result<Model, ArtifactError> {
    let bytes = read_bounded(path)?;
    let mut raw = Vec::new();
    let mut size = 0usize;
    for path in sources {
        let data = read_bounded(path)?;
        size = size
            .checked_add(data.len())
            .ok_or(ArtifactError::TooLarge)?;
        if size > church_slavonic::morphology::ModelLimits::default().max_observation_bytes {
            return Err(ArtifactError::TooLarge);
        }
        raw.push(data);
    }
    from_bytes_with_sources(&bytes, raw)
}

#[derive(Debug, Deserialize)]
pub struct Fixture {
    pub id: String,
    pub forms: Vec<ExpectedForm>,
}
#[derive(Debug, Deserialize)]
pub struct ExpectedForm {
    pub pos: String,
    pub cell: String,
    pub surface: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct InventoryIssue {
    pub lexeme: String,
    pub pos: String,
    pub cell: String,
    pub kind: &'static str,
    pub details: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct FixtureReport {
    pub fixture: String,
    pub expected_forms: usize,
    pub exact_generated: usize,
    pub joint_analysis_retained: usize,
    /// Coverage warnings, deduplicated across queries. These do not assert
    /// that a particular expected form failed or that search was complete.
    pub inventory_issues: Vec<InventoryIssue>,
    pub failures: Vec<String>,
}

fn contains_surface(availability: &Availability, surface: &str) -> bool {
    match availability {
        Availability::Licensed(forms) | Availability::Partial { forms, .. } => {
            forms.iter().any(|form| form.surface == surface)
        }
        Availability::UnresolvedRestriction { otherwise, .. } => {
            contains_surface(otherwise, surface)
        }
        _ => false,
    }
}

/// This is generation with a supplied lexeme and joint-analysis retention,
/// not unknown-word accuracy or contextual selection accuracy.
pub fn check_fixture(
    model: &Model,
    fixture: Fixture,
    lexeme: &LexemeId,
    orthography: &OrthographyId,
) -> Result<FixtureReport, ArtifactError> {
    if fixture.forms.is_empty() {
        return Err(ArtifactError::InvalidFixture("empty fixture".into()));
    }
    let mut report = FixtureReport {
        fixture: fixture.id,
        expected_forms: fixture.forms.len(),
        exact_generated: 0,
        joint_analysis_retained: 0,
        inventory_issues: Vec::new(),
        failures: Vec::new(),
    };
    let mut inventory_issues = std::collections::BTreeSet::new();
    for expected in fixture.forms {
        let pos = Pos::parse(&expected.pos).ok_or_else(|| {
            ArtifactError::InvalidFixture(format!("unknown POS {:?}", expected.pos))
        })?;
        let cell = Cell::parse(pos, &expected.cell)
            .map_err(|e| ArtifactError::InvalidFixture(e.to_string()))?;
        let generated = model
            .generate(lexeme, cell, orthography)
            .map_err(ArtifactError::Model)?;
        if contains_surface(&generated, &expected.surface) {
            report.exact_generated += 1;
        } else {
            report.failures.push(format!(
                "{}: expected {:?}, generated {generated:?}",
                cell.name(),
                expected.surface
            ));
        }
        let analyses = model
            .analyze(&expected.surface, orthography, MatchPolicy::Exact)
            .map_err(ArtifactError::Model)?;
        if analyses
            .candidates
            .iter()
            .any(|a| a.lexeme.id == *lexeme && a.cell == cell)
        {
            report.joint_analysis_retained += 1;
        } else {
            report.failures.push(format!(
                "{}: correct lexical analysis missing for {:?}",
                cell.name(),
                expected.surface
            ));
        }
        for (entry, unresolved, stems) in analyses.unresolved_cells {
            inventory_issues.insert(InventoryIssue {
                lexeme: entry.id.0.clone(),
                pos: unresolved.pos().tag().into(),
                cell: unresolved.name(),
                kind: "missing_stems",
                details: stems,
            });
        }
        for (entry, restriction) in analyses.unresolved_restrictions {
            inventory_issues.insert(InventoryIssue {
                lexeme: entry.id.0.clone(),
                pos: restriction.cell.pos().tag().into(),
                cell: restriction.cell.name(),
                kind: "unverified_restriction",
                details: vec![restriction.reason],
            });
        }
    }
    report.inventory_issues = inventory_issues.into_iter().collect();
    Ok(report)
}
