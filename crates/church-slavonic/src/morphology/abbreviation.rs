//! Explicit, profile-local spellings. These rules neither infer expansions by
//! subsequence nor claim that the registered expansions exhaust a witness.
//! Abbreviated outputs must fit one segment of the current document tokenizer.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbbreviationRule {
    pub id: RuleId,
    pub lexeme: LexemeId,
    #[serde(with = "super::cell_codec")]
    pub cell: Cell,
    /// Exact rendered intermediate form; not necessarily licensed for printing.
    pub expanded: String,
    pub abbreviated: String,
    /// Editorial policy for this lexeme/cell in the selected profile. False
    /// replaces the intermediate full spelling, not the grammatical cell.
    pub retain_expanded: bool,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbbreviationTrace {
    pub rule: RuleId,
    pub expanded: String,
    pub abbreviated: String,
    pub evidence: Vec<EvidenceId>,
}
impl AbbreviationTrace {
    pub(crate) fn retained_bytes(&self) -> usize {
        self.rule.0.len()
            + self.expanded.len()
            + self.abbreviated.len()
            + self.evidence.iter().map(|e| e.0.len()).sum::<usize>()
    }
}

impl Derivation {
    pub(super) fn retained_bytes(&self) -> usize {
        self.claim_sha256.len() + self.review.retained_bytes()
            + self.stem_change.as_ref().map_or(0, super::StemTrace::retained_bytes)
            + self.underlying.len()
            + self
                .source_annotations
                .iter()
                .map(|id| id.0.len())
                .sum::<usize>()
            + self.surface.len()
            + self.accent.accented.len()
            + self.grammar.0.len()
            + self.orthography.0.len()
            + self.paradigm.0.len()
            + self.rule.0.len()
            + self
                .spelling_steps
                .iter()
                .map(|(a, b)| a.len() + b.len())
                .sum::<usize>()
            + self.evidence.iter().map(|e| e.0.len()).sum::<usize>()
            + self
                .abbreviation
                .as_ref()
                .map_or(0, AbbreviationTrace::retained_bytes)
    }
}

impl Model {
    pub(super) fn validate_abbreviations(&self) -> Result<(), ModelError> {
        let mut checks = 0usize;
        for profile in self.orthographies.values() {
            if profile.abbreviations.len() > self.limits.max_spelling_steps {
                return Err(ModelError::ResourceLimit("abbreviation rules"));
            }
            let mut ids = BTreeSet::new();
            let mut modes = BTreeMap::new();
            for rule in &profile.abbreviations {
                if rule.id.0.trim().is_empty() || !ids.insert(&rule.id) {
                    return Err(ModelError::Duplicate(rule.id.0.clone()));
                }
                self.check_evidence(&rule.evidence, &rule.id.0)?;
                let entry = self
                    .lexemes
                    .get(&rule.lexeme)
                    .ok_or_else(|| ModelError::MissingReference(rule.lexeme.0.clone()))?;
                let paradigm = &self.paradigms[&entry.paradigm];
                if !profile.compatible_grammars.contains(&paradigm.grammar) {
                    return Err(ModelError::IncompatibleProfiles);
                }
                if rule.expanded.len() > self.limits.max_form_bytes
                    || rule.abbreviated.len() > self.limits.max_form_bytes
                {
                    return Err(ModelError::ResourceLimit("abbreviation form bytes"));
                }
                if rule.expanded.is_empty()
                    || !crate::document::is_text_segment(&rule.abbreviated)
                    || rule.expanded == rule.abbreviated
                    || rule.cell.pos() != paradigm.pos
                {
                    return Err(ModelError::InvalidRule(rule.id.0.clone()));
                }
                if profile.unicode == UnicodePolicy::Nfc
                    && !rule.abbreviated.nfc().eq(rule.abbreviated.chars())
                {
                    return Err(ModelError::InvalidRule(format!(
                        "{}: abbreviation is not NFC",
                        rule.id.0
                    )));
                }
                let key = (&rule.lexeme, rule.cell.name(), &rule.expanded);
                if modes
                    .insert(key, rule.retain_expanded)
                    .is_some_and(|old| old != rule.retain_expanded)
                {
                    return Err(ModelError::InvalidRule(format!(
                        "{}: conflicting print policy",
                        rule.id.0
                    )));
                }
                checks = paradigm
                    .rules
                    .len()
                    .checked_mul(1 + profile.rules.len())
                    .and_then(|n| checks.checked_add(n + 1))
                    .ok_or(ModelError::ResourceLimit("abbreviation validation work"))?;
                if checks > self.limits.max_rule_checks {
                    return Err(ModelError::ResourceLimit("abbreviation validation work"));
                }
                // Validate against uncontracted intermediates, without applying
                // another abbreviation or recursively expanding this rule.
                let availability = self.realize(entry, paradigm, profile, rule.cell)?;
                let forms = match availability {
                    Availability::Licensed(forms) | Availability::Partial { forms, .. } => forms,
                    _ => Vec::new(),
                };
                if !forms.iter().any(|f| f.surface == rule.expanded) {
                    return Err(ModelError::InvalidRule(format!(
                        "{}: expanded form not realized",
                        rule.id.0
                    )));
                }
            }
        }
        Ok(())
    }

    pub(super) fn realize_with_abbreviations(
        &self,
        entry: &LexicalEntry,
        paradigm: &Paradigm,
        profile: &OrthographyProfile,
        cell: Cell,
    ) -> Result<Availability, ModelError> {
        let mut availability = self.realize(entry, paradigm, profile, cell)?;
        let forms = match &mut availability {
            Availability::Licensed(forms) | Availability::Partial { forms, .. } => forms,
            _ => return Ok(availability),
        };
        if profile.abbreviations.is_empty() {
            return Ok(availability);
        }
        let mut output = Vec::new();
        let mut bytes = 0usize;
        for form in forms.drain(..) {
            let matching: Vec<_> = profile
                .abbreviations
                .iter()
                .filter(|r| r.lexeme == entry.id && r.cell == cell && r.expanded == form.surface)
                .collect();
            let keep = matching.first().is_none_or(|r| r.retain_expanded);
            for rule in matching {
                // Compute the retained payload before cloning the derivation.
                let trace_bytes = rule.id.0.len()
                    + rule.expanded.len()
                    + rule.abbreviated.len()
                    + rule.evidence.iter().map(|e| e.0.len()).sum::<usize>();
                let size = form.retained_bytes() - form.surface.len()
                    + rule.abbreviated.len()
                    + trace_bytes
                    + rule.evidence.iter().map(|e| e.0.len()).sum::<usize>();
                bytes = bytes
                    .checked_add(size)
                    .ok_or(ModelError::ResourceLimit("abbreviation output"))?;
                if bytes > self.limits.max_result_bytes
                    || output.len() >= self.limits.max_candidates
                {
                    return Err(ModelError::ResourceLimit("abbreviation output"));
                }
                let mut rendered = form.clone();
                rendered.surface = rule.abbreviated.clone();
                rendered.abbreviation = Some(AbbreviationTrace {
                    rule: rule.id.clone(),
                    expanded: rule.expanded.clone(),
                    abbreviated: rule.abbreviated.clone(),
                    evidence: rule.evidence.clone(),
                });
                rendered.evidence.extend(rule.evidence.clone());
                rendered.evidence.sort();
                rendered.evidence.dedup();
                output.push(rendered);
            }
            if keep {
                bytes = bytes
                    .checked_add(form.retained_bytes())
                    .ok_or(ModelError::ResourceLimit("abbreviation output"))?;
                if bytes > self.limits.max_result_bytes
                    || output.len() >= self.limits.max_candidates
                {
                    return Err(ModelError::ResourceLimit("abbreviation output"));
                }
                output.push(form);
            }
        }
        *forms = output;
        Ok(availability)
    }
}
