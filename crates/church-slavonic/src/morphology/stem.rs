//! Explicit rule-local stem operations, not automatic historical sound laws.
use super::{EvidenceId, ModelError, ModelLimits};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StemReplacement {
    /// Required nonempty ending of the supplied stem; mismatch is invalid input.
    pub from: String,
    pub to: String,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StemTrace {
    pub before: String,
    pub after: String,
}
impl StemTrace {
    pub(super) fn retained_bytes(&self) -> usize {
        self.before.len() + self.after.len()
    }
}

pub(super) fn apply<'a>(
    supplied: &'a str,
    change: Option<&StemReplacement>,
    limits: &ModelLimits,
) -> Result<Cow<'a, str>, ModelError> {
    if supplied.len() > limits.max_form_bytes {
        return Err(ModelError::ResourceLimit("supplied stem bytes"));
    }
    let Some(change) = change else {
        return Ok(Cow::Borrowed(supplied));
    };
    let prefix = supplied.strip_suffix(&change.from).ok_or_else(|| {
        ModelError::InvalidRule("stem does not satisfy explicit final replacement".into())
    })?;
    let size = prefix
        .len()
        .checked_add(change.to.len())
        .filter(|n| *n <= limits.max_form_bytes)
        .ok_or(ModelError::ResourceLimit("transformed stem bytes"))?;
    if supplied
        .len()
        .checked_add(size)
        .is_none_or(|n| n > limits.max_result_bytes)
    {
        return Err(ModelError::ResourceLimit("stem trace bytes"));
    }
    let mut output = String::with_capacity(size);
    output.push_str(prefix);
    output.push_str(&change.to);
    Ok(Cow::Owned(output))
}
