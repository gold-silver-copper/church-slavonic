//! Review of lexical assertions without resolving or deleting alternatives.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LexicalClaim {
    Category(CategoryAssertion),
    Gender(GenderAssertion),
    Relation(LexicalRelation),
}
impl LexicalClaim {
    fn retained_bytes(&self) -> usize {
        match self {
            Self::Category(c) => ClaimRef::Category(c),
            Self::Gender(g) => ClaimRef::Gender(g),
            Self::Relation(r) => ClaimRef::Relation(r),
        }
        .retained_bytes()
    }
}

// Same serialized variants as LexicalClaim, but no assertion is cloned before
// its retained output and caller-review strings have passed the byte budget.
#[derive(Serialize)]
enum ClaimRef<'a> {
    Category(&'a CategoryAssertion),
    Gender(&'a GenderAssertion),
    Relation(&'a LexicalRelation),
}
impl ClaimRef<'_> {
    fn retained_bytes(&self) -> usize {
        let (evidence, target) = match self {
            Self::Category(c) => (&c.evidence, 0),
            Self::Gender(g) => (&g.evidence, 0),
            Self::Relation(r) => (&r.evidence, r.target.0.len()),
        };
        16 + target + evidence.iter().map(|id| id.0.len()).sum::<usize>()
    }
    fn into_owned(self) -> LexicalClaim {
        match self {
            Self::Category(c) => LexicalClaim::Category(c.clone()),
            Self::Gender(g) => LexicalClaim::Gender(g.clone()),
            Self::Relation(r) => LexicalClaim::Relation(r.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewedLexicalClaim {
    pub claim: LexicalClaim,
    pub claim_sha256: String,
    pub review: ClaimReview,
}
impl ReviewedLexicalClaim {
    pub(crate) fn retained_bytes(&self) -> usize {
        self.claim.retained_bytes() + self.claim_sha256.len() + self.review.retained_bytes()
    }
}

/// Caller input only; never a field of the untrusted model artifact.
#[derive(Debug, Clone, Serialize)]
pub struct LexicalAcceptance {
    pub lexeme: LexemeId,
    pub claim_sha256: String,
    pub decision_id: String,
    pub method: String,
}

impl Model {
    /// Category, lexical gender and directed derivation claims are independently
    /// reviewable. Raw LexicalEntry fields remain source assertions, not reviews.
    pub fn lexical_claims(&self, id: &LexemeId) -> Result<Vec<ReviewedLexicalClaim>, ModelError> {
        self.lexical_claims_bounded(id, self.limits.max_result_bytes)
    }

    pub(super) fn lexical_claims_bounded(
        &self,
        id: &LexemeId,
        remaining: usize,
    ) -> Result<Vec<ReviewedLexicalClaim>, ModelError> {
        let entry = self
            .lexeme(id)
            .ok_or_else(|| ModelError::MissingReference(id.0.clone()))?;
        let count = entry
            .categories
            .len()
            .saturating_add(entry.genders.len())
            .saturating_add(entry.relations.len());
        if count > self.limits.max_candidates || count > self.limits.max_rule_checks {
            return Err(ModelError::ResourceLimit("lexical claim count"));
        }
        let claims = entry
            .categories
            .iter()
            .map(ClaimRef::Category)
            .chain(entry.genders.iter().map(ClaimRef::Gender))
            .chain(entry.relations.iter().map(ClaimRef::Relation));
        let mut result = Vec::new();
        let mut bytes = 0usize;
        for claim in claims {
            let key = acceptance::digest(&(
                "lexical-claim-v1",
                self.data_sha256(),
                self.engine_sha256(),
                id,
                &claim,
            ))?;
            let decision = self.accepted_lexical.get(&key);
            bytes = bytes
                .checked_add(
                    claim.retained_bytes()
                        + key.len()
                        + decision.map_or(0, |d| d.method.len() + d.decision_id.len()),
                )
                .ok_or(ModelError::ResourceLimit("lexical claim bytes"))?;
            if bytes > remaining.min(self.limits.max_result_bytes) {
                return Err(ModelError::ResourceLimit("lexical claim bytes"));
            }
            result.push(ReviewedLexicalClaim {
                claim: claim.into_owned(),
                claim_sha256: key,
                review: match decision {
                    Some(d) => ClaimReview::CallerAccepted {
                        decision_id: d.decision_id.clone(),
                        method: d.method.clone(),
                    },
                    None => ClaimReview::Unreviewed,
                },
            });
        }
        Ok(result)
    }

    pub fn lexical_policy_sha256(&self) -> &str {
        &self.lexical_policy_sha256
    }

    /// Consume the unshared model and replace only its lexical caller policy.
    /// Accepting a relation does not accept its target's category or generation.
    pub fn with_lexical_acceptance(
        mut self,
        decisions: Vec<LexicalAcceptance>,
    ) -> Result<Self, ModelError> {
        if decisions.len() > self.limits.max_candidates.min(1024) {
            return Err(ModelError::ResourceLimit("lexical acceptance count"));
        }
        self.accepted_lexical.clear();
        let mut accepted = BTreeMap::new();
        let mut bytes = 0usize;
        let mut checks = 0usize;
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
                    "invalid lexical acceptance metadata".into(),
                ));
            }
            bytes = bytes
                .checked_add(
                    decision.decision_id.len()
                        + decision.method.len()
                        + decision.lexeme.0.len()
                        + 2 * decision.claim_sha256.len(),
                )
                .ok_or(ModelError::ResourceLimit("lexical acceptance bytes"))?;
            if bytes > self.limits.max_result_bytes {
                return Err(ModelError::ResourceLimit("lexical acceptance bytes"));
            }
            let entry = self
                .lexeme(&decision.lexeme)
                .ok_or_else(|| ModelError::MissingReference(decision.lexeme.0.clone()))?;
            checks = checks
                .checked_add(entry.categories.len())
                .and_then(|n| n.checked_add(entry.genders.len()))
                .and_then(|n| n.checked_add(entry.relations.len()))
                .ok_or(ModelError::ResourceLimit("lexical acceptance checks"))?;
            if checks > self.limits.max_rule_checks {
                return Err(ModelError::ResourceLimit("lexical acceptance checks"));
            }
            if !self
                .lexical_claims(&decision.lexeme)?
                .iter()
                .any(|c| c.claim_sha256 == decision.claim_sha256)
            {
                return Err(ModelError::InvalidRule(
                    "lexical acceptance does not match exact claim".into(),
                ));
            }
            if accepted
                .insert(decision.claim_sha256.clone(), decision)
                .is_some()
            {
                return Err(ModelError::Duplicate("lexical acceptance".into()));
            }
        }
        self.accepted_lexical = accepted;
        self.lexical_policy_sha256 = acceptance::digest(&self.accepted_lexical)?;
        Ok(self)
    }
}
