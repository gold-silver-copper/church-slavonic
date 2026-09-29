//! Caller decisions about exact generated claims, separate from source labels.
use super::*;
use std::io::Write;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ClaimReview {
    #[default]
    Unreviewed,
    /// A caller decision, not authentication of a reviewer or historical truth.
    CallerAccepted { decision_id: String, method: String },
}
impl ClaimReview {
    pub(crate) fn retained_bytes(&self) -> usize {
        match self {
            Self::Unreviewed => 0,
            Self::CallerAccepted {
                decision_id,
                method,
            } => decision_id.len() + method.len(),
        }
    }
}

/// Review name retained for the generation-specific API.
pub type GenerationReview = ClaimReview;

/// Supply through the caller API after reviewing a generated claim. This type
/// deliberately is not a field in untrusted ModelInput and has no Deserialize.
#[derive(Debug, Clone, Serialize)]
pub struct GenerationAcceptance {
    pub lexeme: LexemeId,
    #[serde(with = "super::cell_codec")]
    pub cell: Cell,
    pub orthography: OrthographyId,
    pub claim_sha256: String,
    pub decision_id: String,
    pub method: String,
}

struct DigestWriter(Sha256);
impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
pub(super) fn digest(value: &impl Serialize) -> Result<String, ModelError> {
    let mut writer = DigestWriter(Sha256::new());
    serde_json::to_writer(&mut writer, value).map_err(|e| ModelError::Encoding(e.to_string()))?;
    Ok(format!("{:x}", writer.0.finalize()))
}

impl Model {
    /// Replace the caller policy before sharing this immutable model. Changing
    /// model input, source claims, lexical attachment, cell, rendering, or the
    /// recorded engine identity invalidates the old claim key. Source review
    /// labels never activate this policy on their own.
    pub fn with_generation_acceptance(
        mut self,
        decisions: Vec<GenerationAcceptance>,
    ) -> Result<Self, ModelError> {
        if decisions.len() > self.limits.max_candidates.min(1024) {
            return Err(ModelError::ResourceLimit("generation acceptance count"));
        }
        self.analysis_cache = std::sync::RwLock::new(index::Cache::default());
        self.accepted_generation.clear();
        let mut accepted = BTreeMap::new();
        let mut checks = 0usize;
        let mut bytes = 0usize;
        for decision in decisions {
            if decision.decision_id.trim().is_empty()
                || decision.method.trim().is_empty()
                || decision.decision_id.len() > 4096
                || decision.method.len() > 4096
                || decision.claim_sha256.len() != 64
                || !decision
                    .claim_sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            {
                return Err(ModelError::InvalidRule(
                    "invalid generation review method/identity".into(),
                ));
            }
            bytes = bytes
                .checked_add(
                    decision.decision_id.len()
                        + decision.method.len()
                        + decision.lexeme.0.len()
                        + decision.orthography.0.len()
                        + 2 * decision.claim_sha256.len(),
                )
                .ok_or(ModelError::ResourceLimit("generation acceptance bytes"))?;
            if bytes > self.limits.max_result_bytes {
                return Err(ModelError::ResourceLimit("generation acceptance bytes"));
            }
            let entry = self
                .lexeme(&decision.lexeme)
                .ok_or_else(|| ModelError::MissingReference(decision.lexeme.0.clone()))?;
            let spelling = self
                .orthography(&decision.orthography)
                .ok_or_else(|| ModelError::MissingReference(decision.orthography.0.clone()))?;
            checks = checks
                .checked_add(self.paradigms[&entry.paradigm].rules.len())
                .and_then(|n| n.checked_add(spelling.abbreviations.len()))
                .ok_or(ModelError::ResourceLimit("generation acceptance checks"))?;
            if checks > self.limits.max_rule_checks {
                return Err(ModelError::ResourceLimit("generation acceptance checks"));
            }
            let mut generated =
                self.generate(&decision.lexeme, decision.cell, &decision.orthography)?;
            // A pending restriction does not silently become an accepted positive claim.
            let forms = match &mut generated {
                Availability::Licensed(forms) | Availability::Partial { forms, .. } => forms,
                _ => {
                    return Err(ModelError::InvalidRule(
                        "no unrestricted generated claim to accept".into(),
                    ));
                }
            };
            if !forms
                .iter()
                .any(|f| f.claim_sha256 == decision.claim_sha256)
            {
                return Err(ModelError::InvalidRule(
                    "generation acceptance does not match exact claim".into(),
                ));
            }
            if accepted
                .insert(decision.claim_sha256.clone(), decision)
                .is_some()
            {
                return Err(ModelError::Duplicate("generation acceptance".into()));
            }
        }
        self.accepted_generation = accepted;
        self.generation_policy_sha256 = digest(&self.accepted_generation)?;
        Ok(self)
    }

    /// Caller policy identity is separate from input-data identity.
    pub fn generation_policy_sha256(&self) -> &str {
        &self.generation_policy_sha256
    }

    pub(super) fn qualify_generation(
        &self,
        lexeme: &LexemeId,
        cell: Cell,
        form: &mut Derivation,
        remaining: usize,
    ) -> Result<usize, ModelError> {
        // Called exactly once on a freshly realized form, after source links.
        // No accepted label or claim key is fed into its own identity.
        let key = digest(&(
            "generated-claim-v1",
            self.data_sha256(),
            self.engine_sha256(),
            lexeme,
            cell.name(),
            &*form,
        ))?;
        let accepted = self.accepted_generation.get(&key);
        let extra = key.len() + accepted.map_or(0, |d| d.decision_id.len() + d.method.len());
        if extra > remaining {
            return Err(ModelError::ResourceLimit("generation review bytes"));
        }
        form.claim_sha256 = key;
        form.review = match accepted {
            Some(d) => GenerationReview::CallerAccepted {
                decision_id: d.decision_id.clone(),
                method: d.method.clone(),
            },
            None => GenerationReview::Unreviewed,
        };
        Ok(extra)
    }

    pub(super) fn empty_generation_policy_sha256() -> String {
        // JSON encoding of the empty ordered map, also used by the policy setter.
        format!("{:x}", Sha256::digest(b"{}"))
    }
}
