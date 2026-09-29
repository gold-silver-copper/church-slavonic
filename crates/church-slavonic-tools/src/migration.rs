//! Import selected legacy noun claims as source observations, never as rules.
use crate::{
    import::publication::StagedReplacement,
    model_artifact::{self, Artifact},
};
use church_slavonic::{
    Cell, Pos,
    lexicon::COLUMNS,
    matching::MatchPolicy,
    morphology::{observations::*, *},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    io::Write,
    path::Path,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub source_sha256: String,
    pub orthography: OrthographyId,
    pub entries: Vec<Correspondence>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correspondence {
    pub legacy_id: String,
    pub targets: Vec<LexemeId>,
    pub rationale: String,
}
#[derive(Serialize)]
pub struct ClaimResult {
    pub legacy_id: String,
    pub witness: String,
    pub field: String,
    pub cell: Option<String>,
    pub surface: String,
    pub proposed_targets: Vec<LexemeId>,
    pub exact: Vec<LexemeId>,
    pub tolerant: Vec<LexemeId>,
}
#[derive(Serialize)]
pub struct Report {
    pub source_sha256: String,
    pub source_records: usize,
    pub selected_records: usize,
    pub unselected_records: usize,
    pub imported_forms: usize,
    pub cell_claims: usize,
    pub exact_compatible: usize,
    pub tolerant_compatible: usize,
    pub claims: Vec<ClaimResult>,
    pub interpretation: &'static str,
}
struct BoundedOutput(Vec<u8>);
impl Write for BoundedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self
            .0
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > model_artifact::MAX_ARTIFACT_BYTES)
        {
            return Err(std::io::Error::other("migrated artifact byte limit"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct RawClaim {
    start: usize,
    end: usize,
    field: &'static str,
    cell: Option<Cell>,
    weight: Option<String>,
}
fn claims(line: &str, cols: &[&str]) -> Result<Vec<RawClaim>, Box<dyn Error>> {
    let offsets: Vec<usize> = cols
        .iter()
        .scan(0, |n, s| {
            let old = *n;
            *n += s.len() + 1;
            Some(old)
        })
        .collect();
    let mut out = vec![RawClaim {
        start: offsets[1],
        end: offsets[1] + cols[1].len(),
        field: "lemma",
        cell: None,
        weight: None,
    }];
    for (column, field) in [(8, "override"), (9, "variant")] {
        if cols[column] == "-" {
            continue;
        }
        let mut at = offsets[column];
        for group in cols[column].split(';') {
            let (name, forms) = group
                .split_once('=')
                .ok_or("invalid legacy cell assignment")?;
            let cell = Cell::parse(Pos::Noun, name)?;
            let mut start = at + name.len() + 1;
            for printed in forms.split('|') {
                let (surface, weight) = match printed.split_once('×') {
                    Some((s, w)) if field == "variant" => {
                        let _: u32 = w.parse()?;
                        (s, Some(w.to_string()))
                    }
                    _ => (printed, None),
                };
                if surface.is_empty() {
                    return Err("empty legacy supplied form".into());
                }
                out.push(RawClaim {
                    start,
                    end: start + surface.len(),
                    field,
                    cell: Some(cell),
                    weight,
                });
                if out.len() > 4096 {
                    return Err("legacy record form limit".into());
                }
                start += printed.len() + 1;
            }
            at += group.len() + 1;
        }
    }
    if out
        .iter()
        .any(|c| line.get(c.start..c.end).is_none_or(str::is_empty))
    {
        return Err("invalid source span".into());
    }
    Ok(out)
}

pub fn migrate(
    source_path: &Path,
    model_path: &Path,
    mapping_path: &Path,
    output_path: &Path,
) -> Result<Report, Box<dyn Error>> {
    if let Ok(destination) = output_path.canonicalize() {
        for input in [source_path, model_path, mapping_path] {
            if input.canonicalize()? == destination {
                return Err("migration output cannot replace an input".into());
            }
        }
    }
    let raw = model_artifact::read_bounded(source_path)?;
    let text = std::str::from_utf8(&raw)?;
    let sha = format!("{:x}", Sha256::digest(&raw));
    let mapping: Mapping = serde_json::from_slice(&model_artifact::read_bounded(mapping_path)?)?;
    if mapping.source_sha256 != sha {
        return Err("mapping source snapshot mismatch".into());
    }
    if mapping.entries.is_empty() || mapping.entries.len() > 256 {
        return Err("mapping requires 1..256 selected records".into());
    }
    let mut artifact: Artifact =
        serde_json::from_slice(&model_artifact::read_bounded(model_path)?)?;
    if artifact.version != 4 {
        return Err("unsupported model input version".into());
    }
    // Existing observation archives need explicit source inputs. This operation
    // accepts an observation-free seed and imports exactly the selected source.
    if !artifact.input.observations.sources.is_empty() {
        return Err("migration seed must have no registered sources".into());
    }
    let seed = serde_json::to_vec(&artifact)?;
    let mut retained = seed.len();
    let baseline = model_artifact::from_bytes(&seed)?;
    if baseline.orthography(&mapping.orthography).is_none() {
        return Err("missing target orthography".into());
    }
    let mut selected = BTreeMap::new();
    for entry in &mapping.entries {
        if entry.rationale.trim().is_empty()
            || entry.rationale.len() > 4096
            || entry.legacy_id.len() > 4096
            || entry.targets.is_empty()
            || entry.targets.len() > 16
            || selected.insert(entry.legacy_id.as_str(), entry).is_some()
        {
            return Err("invalid/duplicate correspondence".into());
        }
        let mut unique = BTreeSet::new();
        for id in &entry.targets {
            if id.0.len() > 4096 || !unique.insert(id) || baseline.lexeme(id).is_none() {
                return Err("duplicate/missing target identity".into());
            }
        }
    }
    let evidence = EvidenceId(format!("legacy-nouns:{sha}"));
    let source_id = SourceId(format!("legacy-nouns:{sha}"));
    artifact.input.evidence.push(Evidence{id:evidence.clone(),source:source_path.display().to_string(),source_sha256:sha.clone(),method:"Exact legacy TSV extraction; fitted source lineage and lexical correspondences unreviewed".into(),location:"Selected rows/columns addressed by witness spans".into(),claim:"This legacy file supplies these labels and forms. No independent grammatical or identity validation implied.".into(),kind:EvidenceKind::Annotation,review:ReviewStatus::Unverified});
    artifact.input.observations.sources.push(SourceSpec {
        id: source_id.clone(),
        uri: source_path.display().to_string(),
        sha256: sha.clone(),
    });
    let mut report = Report {
        source_sha256: sha,
        source_records: 0,
        selected_records: 0,
        unselected_records: 0,
        imported_forms: 0,
        cell_claims: 0,
        exact_compatible: 0,
        tolerant_compatible: 0,
        claims: vec![],
        interpretation: "Selected legacy claims imported as unreviewed source observations and proposed identity correspondences. No legacy classes, generated forms or variant claims become grammatical rules. Compatibility is model-relative, not adjudicated identity or grammaticality. Unselected records remain outside this increment; no new validated lexemes counted.",
    };
    let mut seen = BTreeSet::new();
    let mut offset = 0;
    let mut header = false;
    for raw_line in text.split_inclusive('\n') {
        let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
        let line_start = offset;
        offset += raw_line.len();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.first() == Some(&"id") {
            if header || cols != COLUMNS {
                return Err("invalid/duplicate TSV header".into());
            }
            header = true;
            continue;
        }
        if !header
            || cols.len() != 12
            || cols[2] != "n"
            || cols[0].is_empty()
            || cols[1].is_empty()
            || !seen.insert(cols[0])
        {
            return Err("invalid/duplicate noun source record".into());
        }
        report.source_records += 1;
        let Some(correspondence) = selected.get(cols[0]) else {
            report.unselected_records += 1;
            continue;
        };
        report.selected_records += 1;
        for (ordinal, c) in claims(line, &cols)?.into_iter().enumerate() {
            if report.imported_forms >= 4096 {
                return Err("migration form limit".into());
            }
            // Bound repeated raw metadata and identity copies before cloning
            // them into annotation records or compatibility results.
            let targets = serde_json::to_string(&correspondence.targets)?;
            let copies = if c.cell.is_some() {
                correspondence.targets.len()
            } else {
                1
            };
            let estimate = line
                .len()
                .checked_add(correspondence.rationale.len())
                .and_then(|n| n.checked_add(targets.len()))
                .and_then(|n| n.checked_add(4096 + 4 * cols[0].len()))
                .and_then(|n| n.checked_mul(copies))
                .ok_or("migration metadata budget")?;
            retained = retained
                .checked_add(estimate)
                .filter(|n| *n <= model_artifact::MAX_ARTIFACT_BYTES)
                .ok_or("migration metadata budget")?;
            let surface = &line[c.start..c.end];
            let witness = WitnessId(format!("legacy:{}:{ordinal}", cols[0]));
            let observation = ObservationId(format!("legacy:{}:{ordinal}", cols[0]));
            artifact.input.observations.witnesses.push(WitnessSpec {
                id: witness.clone(),
                source: source_id.clone(),
                start: line_start + c.start,
                end: line_start + c.end,
                location: format!(
                    "legacy noun {}; {}; raw row byte {}",
                    cols[0], c.field, line_start
                ),
            });
            artifact.input.observations.observations.push(Observation {
                id: observation.clone(),
                witness: witness.clone(),
                start: 0,
                end: surface.len(),
                surface: surface.into(),
                kind: ObservationKind::Unclassified,
            });
            let mut original: BTreeMap<String, String> = COLUMNS
                .iter()
                .zip(&cols)
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            original.insert(
                "record_kind".into(),
                "legacy lexical claim, not running-text attestation".into(),
            );
            original.insert(
                "correspondence_status".into(),
                "proposed; not adjudicated".into(),
            );
            original.insert(
                "correspondence_rationale".into(),
                correspondence.rationale.clone(),
            );
            original.insert("target_candidates".into(), targets);
            original.insert("field".into(), c.field.into());
            if let Some(weight) = c.weight {
                original.insert("raw_variant_weight".into(), weight);
            }
            if let Some(cell) = c.cell {
                original.insert("source_cell".into(), cell.name());
                for (i, id) in correspondence.targets.iter().enumerate() {
                    artifact
                        .input
                        .observations
                        .annotations
                        .push(SourceAnnotation {
                            id: AnnotationId(format!("legacy:{}:{ordinal}:{i}", cols[0])),
                            observation: observation.clone(),
                            original: original.clone(),
                            target: Some(AnnotationTarget {
                                lexeme: id.clone(),
                                cell,
                                orthography: mapping.orthography.clone(),
                            }),
                            evidence: vec![evidence.clone()],
                        });
                }
            } else {
                artifact
                    .input
                    .observations
                    .annotations
                    .push(SourceAnnotation {
                        id: AnnotationId(format!("legacy:{}:{ordinal}:lemma", cols[0])),
                        observation,
                        original,
                        target: None,
                        evidence: vec![evidence.clone()],
                    });
            }
            report.claims.push(ClaimResult {
                legacy_id: cols[0].into(),
                witness: witness.0,
                field: c.field.into(),
                cell: c.cell.map(|v| v.name()),
                surface: surface.into(),
                proposed_targets: correspondence.targets.clone(),
                exact: vec![],
                tolerant: vec![],
            });
            report.imported_forms += 1;
        }
    }
    if !header || report.selected_records != selected.len() {
        return Err("missing header/selected record".into());
    }
    let mut encoded = BoundedOutput(Vec::new());
    serde_json::to_writer_pretty(&mut encoded, &artifact)?;
    let encoded = encoded.0;
    if encoded.len() > model_artifact::MAX_ARTIFACT_BYTES {
        return Err("migrated artifact exceeds byte limit".into());
    }
    let model = model_artifact::from_bytes_with_sources(&encoded, vec![raw.clone()])?;
    for row in &mut report.claims {
        let Some(cell) = &row.cell else {
            continue;
        };
        report.cell_claims += 1;
        for (policy, out) in [
            (MatchPolicy::Exact, &mut row.exact),
            (MatchPolicy::LegacyOrthographic, &mut row.tolerant),
        ] {
            for found in model
                .analyze(&row.surface, &mapping.orthography, policy)?
                .candidates
            {
                if found.cell.name() == *cell
                    && row.proposed_targets.contains(&found.lexeme.id)
                    && !out.contains(&found.lexeme.id)
                {
                    out.push(found.lexeme.id.clone());
                }
            }
        }
        report.exact_compatible += usize::from(!row.exact.is_empty());
        report.tolerant_compatible += usize::from(!row.tolerant.is_empty());
    }
    StagedReplacement::prepare(output_path, &encoded, |bytes| {
        model_artifact::from_bytes_with_sources(bytes, vec![raw.clone()])?;
        Ok(())
    })?
    .commit()?;
    Ok(report)
}
