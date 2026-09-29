//! Lazy immutable retrieval indexes. Construction and query budgets are separate.
//! Keys are retrieval equivalences, never lexical identities. All derivations
//! and model-wide uncertainty survive lookup; no target-grammar assertion here.
mod persistence;
use super::*;
use crate::matching::{MatchPolicy, compare, key};
pub use persistence::MAX_COMPILED_INDEX_BYTES;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct AnalysisIndexStats {
    pub indexes: usize,
    pub restored_indexes: usize,
    pub forms: usize,
    /// Logical retained records and string bytes, not allocator/RSS measurement.
    pub retained_bytes: usize,
    pub construction_checks: usize,
}
#[derive(Default)]
pub(super) struct Cache {
    entries: BTreeMap<(OrthographyId, usize), Prepared>,
    stats: AnalysisIndexStats,
}
#[derive(Default)]
struct Prepared {
    forms: BTreeMap<String, Vec<IndexedForm>>,
    restrictions: Vec<(LexemeId, Restriction)>,
    missing: Vec<(LexemeId, Cell, Vec<String>)>,
    stats: AnalysisIndexStats,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexedForm {
    lexeme: LexemeId,
    #[serde(with = "super::cell_codec")]
    cell: Cell,
    derivation: Derivation,
}
fn charge(
    used: &mut usize,
    amount: usize,
    limit: usize,
    label: &'static str,
) -> Result<(), ModelError> {
    *used = used
        .checked_add(amount)
        .filter(|n| *n <= limit)
        .ok_or(ModelError::ResourceLimit(label))?;
    Ok(())
}
fn poisoned() -> ModelError {
    ModelError::InvalidRule("analysis index lock poisoned; no complete result".into())
}
impl Model {
    /// Current successful cache construction only. A failed build publishes no
    /// index. This reports logical accounting, not a process-memory guarantee.
    pub fn analysis_index_stats(&self) -> Result<AnalysisIndexStats, ModelError> {
        Ok(self
            .analysis_cache
            .read()
            .map_err(|_| poisoned())?
            .stats
            .clone())
    }
}
impl Prepared {
    fn build(
        model: &Model,
        orthography: &OrthographyId,
        policy: MatchPolicy,
        existing: &AnalysisIndexStats,
    ) -> Result<Self, ModelError> {
        let spelling = model
            .orthography(orthography)
            .ok_or_else(|| ModelError::MissingReference(orthography.0.clone()))?;
        let mut out = Self::default();
        let bytes_limit = model
            .limits
            .max_index_bytes
            .saturating_sub(existing.retained_bytes);
        let forms_limit = model.limits.max_index_forms.saturating_sub(existing.forms);
        let work_limit = model
            .limits
            .max_index_checks
            .saturating_sub(existing.construction_checks);
        charge(
            &mut out.stats.retained_bytes,
            std::mem::size_of::<Self>() + orthography.0.len(),
            bytes_limit,
            "analysis index bytes",
        )?;
        for lexeme in model.lexemes.values() {
            charge(
                &mut out.stats.construction_checks,
                1,
                work_limit,
                "analysis index checks",
            )?;
            let paradigm = &model.paradigms[&lexeme.paradigm];
            if !spelling.compatible_grammars.contains(&paradigm.grammar) {
                continue;
            }
            for cell in model.inventory(&lexeme.id)? {
                let work = paradigm
                    .rules
                    .len()
                    .checked_mul(1 + spelling.rules.len() + spelling.abbreviations.len())
                    .and_then(|n| n.checked_add(1))
                    .ok_or(ModelError::ResourceLimit("analysis index checks"))?;
                charge(
                    &mut out.stats.construction_checks,
                    work,
                    work_limit,
                    "analysis index checks",
                )?;
                let mut availability = model.generate(&lexeme.id, cell, orthography)?;
                let forms = loop {
                    match availability {
                        Availability::UnresolvedRestriction {
                            restriction,
                            otherwise,
                        } => {
                            let bytes = std::mem::size_of::<(LexemeId, Restriction)>()
                                + lexeme.id.0.len()
                                + restriction.reason.len()
                                + restriction
                                    .evidence
                                    .iter()
                                    .map(|e| std::mem::size_of::<EvidenceId>() + e.0.len())
                                    .sum::<usize>();
                            charge(
                                &mut out.stats.retained_bytes,
                                bytes,
                                bytes_limit,
                                "analysis index bytes",
                            )?;
                            out.restrictions.push((lexeme.id.clone(), restriction));
                            availability = *otherwise;
                        }
                        Availability::Licensed(forms) => break forms,
                        Availability::Partial {
                            forms,
                            missing_stems,
                        } => {
                            out.add_missing(lexeme, cell, missing_stems, bytes_limit)?;
                            break forms;
                        }
                        Availability::MissingStems(missing) => {
                            out.add_missing(lexeme, cell, missing, bytes_limit)?;
                            break Vec::new();
                        }
                        Availability::Unavailable(_) | Availability::UnsupportedCell => {
                            break Vec::new();
                        }
                    }
                };
                for derivation in forms {
                    charge(
                        &mut out.stats.construction_checks,
                        derivation.source_annotations.len(),
                        work_limit,
                        "analysis index checks",
                    )?;
                    charge(&mut out.stats.forms, 1, forms_limit, "analysis index forms")?;
                    let retrieval = key(&derivation.surface, policy);
                    let bytes = std::mem::size_of::<IndexedForm>()
                        + lexeme.id.0.len()
                        + derivation.retained_bytes()
                        + std::mem::size_of::<(String, Vec<IndexedForm>)>()
                        + retrieval.len();
                    charge(
                        &mut out.stats.retained_bytes,
                        bytes,
                        bytes_limit,
                        "analysis index bytes",
                    )?;
                    out.forms.entry(retrieval).or_default().push(IndexedForm {
                        lexeme: lexeme.id.clone(),
                        cell,
                        derivation,
                    });
                }
            }
        }
        out.stats.indexes = 1;
        Ok(out)
    }
    fn add_missing(
        &mut self,
        lexeme: &LexicalEntry,
        cell: Cell,
        missing: Vec<String>,
        limit: usize,
    ) -> Result<(), ModelError> {
        let bytes = std::mem::size_of::<(LexemeId, Cell, Vec<String>)>()
            + lexeme.id.0.len()
            + missing
                .iter()
                .map(|s| std::mem::size_of::<String>() + s.len())
                .sum::<usize>();
        charge(
            &mut self.stats.retained_bytes,
            bytes,
            limit,
            "analysis index bytes",
        )?;
        self.missing.push((lexeme.id.clone(), cell, missing));
        Ok(())
    }
    fn query<'a>(
        &self,
        model: &'a Model,
        surface: &str,
        policy: MatchPolicy,
        checks: &mut usize,
    ) -> Result<ModelAnalysisSet<'a>, ModelError> {
        charge(checks, 1, model.limits.max_rule_checks, "rule checks")?;
        let mut result = ModelAnalysisSet {
            candidates: Vec::new(),
            unresolved_restrictions: Vec::new(),
            unresolved_cells: Vec::new(),
        };
        let mut bytes = 0;
        for (id, restriction) in &self.restrictions {
            charge(checks, 1, model.limits.max_rule_checks, "rule checks")?;
            charge(
                &mut bytes,
                restriction.reason.len()
                    + restriction
                        .evidence
                        .iter()
                        .map(|e| e.0.len())
                        .sum::<usize>(),
                model.limits.max_result_bytes,
                "result bytes",
            )?;
            result
                .unresolved_restrictions
                .push((&model.lexemes[id], restriction.clone()));
        }
        for (id, cell, missing) in &self.missing {
            charge(checks, 1, model.limits.max_rule_checks, "rule checks")?;
            charge(
                &mut bytes,
                missing.iter().map(String::len).sum::<usize>(),
                model.limits.max_result_bytes,
                "result bytes",
            )?;
            result
                .unresolved_cells
                .push((&model.lexemes[id], *cell, missing.clone()));
        }
        if let Some(forms) = self.forms.get(&key(surface, policy)) {
            for form in forms {
                let lexeme = &model.lexemes[&form.lexeme];
                charge(
                    checks,
                    1 + form.derivation.source_annotations.len(),
                    model.limits.max_rule_checks,
                    "rule checks",
                )?;
                let Some(trace) = compare(surface, &form.derivation.surface, policy) else {
                    continue;
                };
                let claims = lexeme
                    .categories
                    .len()
                    .saturating_add(lexeme.genders.len())
                    .saturating_add(lexeme.relations.len());
                charge(
                    checks,
                    claims,
                    model.limits.max_rule_checks,
                    "lexical claim checks",
                )?;
                let size = form.derivation.retained_bytes()
                    + trace
                        .query_steps
                        .iter()
                        .chain(&trace.generated_steps)
                        .map(|s| s.before.len() + s.after.len())
                        .sum::<usize>();
                charge(
                    &mut bytes,
                    size,
                    model.limits.max_result_bytes,
                    "analysis results",
                )?;
                if result.candidates.len() >= model.limits.max_candidates {
                    return Err(ModelError::ResourceLimit("analysis results"));
                }
                let lexical_claims = model
                    .lexical_claims_bounded(&lexeme.id, model.limits.max_result_bytes - bytes)?;
                charge(
                    &mut bytes,
                    lexical_claims
                        .iter()
                        .map(ReviewedLexicalClaim::retained_bytes)
                        .sum::<usize>(),
                    model.limits.max_result_bytes,
                    "analysis results",
                )?;
                result.candidates.push(ModelAnalysis {
                    lexeme,
                    cell: form.cell,
                    derivation: form.derivation.clone(),
                    lexical_claims,
                    trace,
                });
            }
        }
        Ok(result)
    }
}

pub(super) fn analyze<'a>(
    model: &'a Model,
    surface: &str,
    orthography: &OrthographyId,
    policy: MatchPolicy,
    checks: &mut usize,
) -> Result<ModelAnalysisSet<'a>, ModelError> {
    if surface.len() > model.limits.max_form_bytes {
        return Err(ModelError::ResourceLimit("query bytes"));
    }
    if model.orthography(orthography).is_none() {
        return Err(ModelError::MissingReference(orthography.0.clone()));
    }
    let cache_key = (orthography.clone(), policy.slot());
    let cache = model.analysis_cache.read().map_err(|_| poisoned())?;
    if let Some(index) = cache.entries.get(&cache_key) {
        return index.query(model, surface, policy, checks);
    }
    drop(cache);
    // Publish only a complete index. Serialize cold builds to bound temporary
    // construction storage; warm readers can run concurrently under read locks.
    let mut cache = model.analysis_cache.write().map_err(|_| poisoned())?;
    if !cache.entries.contains_key(&cache_key) {
        let built = Prepared::build(model, orthography, policy, &cache.stats)?;
        cache.stats.indexes += 1;
        cache.stats.forms += built.stats.forms;
        cache.stats.retained_bytes += built.stats.retained_bytes;
        cache.stats.construction_checks += built.stats.construction_checks;
        cache.entries.insert(cache_key.clone(), built);
    }
    cache.entries[&cache_key].query(model, surface, policy, checks)
}
