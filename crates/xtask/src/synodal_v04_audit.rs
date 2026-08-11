use std::{error::Error, fs, path::Path};

use serde_json::Value;
use sha2::{Digest, Sha256};

const OUTPUT: &str = "docs/SYNODAL_V04_MORPHOLOGICAL_FAMILY_AUDIT.md";
const BASELINE: &str = "reports/synodal-v04-baseline.json";
const LOCKED_AUDIT_SHA256: &str =
    "4d62941f0ed13e2285082a482d01988818fe6ee1cfbc4f5a8a02786b7fef71fa";
const LOCKED_TOKENS: u64 = 1_313_344;
const LOCKED_TOP_K: u64 = 569_418;

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    for argument in args {
        match argument.as_str() {
            "--check" => {}
            value => return Err(format!("unknown synodal-v04-audit argument {value:?}").into()),
        }
    }

    // v0.4 is an immutable comparison point. Reading the live registries here would
    // silently relabel later milestones as v0.4, so validate the locked baseline and
    // the byte identity of its already-generated audit instead.
    let baseline: Value = serde_json::from_str(&fs::read_to_string(root.join(BASELINE))?)?;
    require_string(&baseline, "milestone", "synodal-v0.4")?;
    require_string(&baseline, "target_recension", "synodal-russian")?;
    require_string(&baseline, "generation_policy", "strict")?;
    require_string(&baseline, "orthography_profile", "synodal-liturgical")?;
    require_string(
        &baseline,
        "tokenizer_contract",
        "synodal-dictionary-tokenize-v1",
    )?;
    require_number(&baseline, "/corpus/tokens", LOCKED_TOKENS)?;
    require_number(&baseline, "/coverage/top_k_analyzed", LOCKED_TOP_K)?;

    let output = root.join(OUTPUT);
    let bytes = fs::read(&output)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    if digest != LOCKED_AUDIT_SHA256 {
        return Err(format!(
            "{} differs from the locked v0.4 audit (expected {}, found {})",
            output.display(),
            LOCKED_AUDIT_SHA256,
            digest
        )
        .into());
    }

    println!("Synodal v0.4 morphological-family audit: locked and current");
    Ok(())
}

fn require_string(value: &Value, key: &str, expected: &str) -> Result<(), Box<dyn Error>> {
    let actual = value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("v0.4 baseline omits string field {key:?}"))?;
    if actual != expected {
        return Err(format!(
            "v0.4 baseline field {key:?} differs: expected {expected:?}, found {actual:?}"
        )
        .into());
    }
    Ok(())
}

fn require_number(value: &Value, pointer: &str, expected: u64) -> Result<(), Box<dyn Error>> {
    let actual = value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("v0.4 baseline omits numeric field {pointer:?}"))?;
    if actual != expected {
        return Err(format!(
            "v0.4 baseline field {pointer:?} differs: expected {expected}, found {actual}"
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_audit_digest_has_sha256_width() {
        assert_eq!(LOCKED_AUDIT_SHA256.len(), 64);
        assert!(
            LOCKED_AUDIT_SHA256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        );
    }
}
