use std::{error::Error, fs, path::Path};

use serde_json::Value;

const OUTPUT: &str = "reports/synodal-v06-baseline.json";
const LOCKED_TOKENS: u64 = 1_313_344;
const LOCKED_TOP_K: u64 = 853_770;

const BASELINE_JSON: &str = r#"{
  "schema_version": 1,
  "milestone": "synodal-v0.6",
  "target_recension": "synodal-russian",
  "generation_policy": "strict",
  "orthography_profile": "synodal-liturgical",
  "tokenizer_contract": "synodal-dictionary-tokenize-v1",
  "corpus": {
    "passages": 74130,
    "tokens": 1313344,
    "token_types": 57476,
    "source_ids": [
      "ponomar-elizabeth-bible-2026-08-09",
      "wikisource-church-slavonic-bible-2026-08-09"
    ],
    "partitions": [
      "evaluation",
      "source"
    ]
  },
  "registry": {
    "reviewed_lexemes": 655,
    "reviewed_senses": 655,
    "generated_exact_forms": 2450,
    "typed_abbreviations": 149
  },
  "coverage": {
    "top_1_analyzed": 569630,
    "top_k_analyzed": 853770,
    "ambiguous": 15394,
    "cyrillic_numerals": 197,
    "top_k_uncovered": 459377,
    "unresolved": 458189,
    "by_status": {
      "abbreviation-expansion": 45921,
      "ambiguous": 15394,
      "cyrillic-numeral": 197,
      "exact-synodal-attestation": 549627,
      "spelling-variant": 1188,
      "synodal-normative-table": 224669,
      "synodal-productive-rule": 18159,
      "unresolved": 458189
    },
    "by_gap": {
      "ambiguity-or-spelling-variant": 16582,
      "missing-accent-or-orthographic-metadata": 7055,
      "missing-declension-or-class": 129,
      "missing-verb-principal-part": 60,
      "unknown-lexeme": 450945
    },
    "estimated_recovery_by_route": {
      "abbreviation-registry": 13429,
      "exact-evidence": 690,
      "reviewed-class": 129,
      "reviewed-principal-part": 60,
      "spelling-variant": 8243,
      "ungrouped-unknown": 436826
    }
  },
  "evaluation": {
    "morphological_cells": 1187,
    "abbreviation_cells": 64,
    "expanded_top_1": 1163,
    "expanded_top_k": 1187,
    "printed_top_1": 1118,
    "printed_top_k": 1187,
    "strict_top_k": 1187,
    "evaluation_runtime_passage_overlap": 0
  },
  "reviews": {
    "exact_rows": 515,
    "exact_admitted": 499,
    "exact_rejected": 16,
    "exact_admitted_predicted_tokens": 40683,
    "exact_admitted_realized_tokens": 40683,
    "abbreviation_rows": 70,
    "spelling_rows": 18,
    "verification_rows": 30
  },
  "marginal": {
    "batches": 1743,
    "overlap_adjusted_tokens": 184296,
    "diagnostic_projected_top_k": 1038066,
    "tokens_needed_for_strictly_more_than_70_percent": 65571,
    "by_route": {
      "abbreviation-registry": 6417,
      "reviewed-class": 129,
      "reviewed-principal-part": 42,
      "spelling-variant": 3121,
      "ungrouped-unknown": 174587
    },
    "by_readiness_effort": {
      "ready/small": 1661,
      "ready/medium": 0,
      "partial/medium": 14307,
      "partial/large": 10102,
      "weak/large": 158226
    }
  },
  "artifact_sha256": {
    "crates/synodal-church-slavonic-dictionary/generated/registry.rs": "99de7f994b4e443ed7116f00e1f5122dd88491288ba7bec52f82a72a0230d33a",
    "crates/synodal-church-slavonic/generated/registry.rs": "39873dfb08c6b32e75220d4145c837083bb41db101dcfa8bc33875f0506cba10",
    "data/synodal/abbreviations.tsv": "60bd91a7428d593cdac4ece31ce3e4d5f9cb43617796d4ba77207d6ce002cef3",
    "data/synodal/exact_forms.tsv": "7d6d4dff84a4bafb35ea6745168224a07b5712677b0c7f7e379ff8f988b3cc97",
    "data/synodal/family_reviews.tsv": "672000d2300943360dd467d6f0ab7bb34e1227282a3fb4f418a15b45844df3ed",
    "data/synodal/lexical_reviews.tsv": "27fcd9fd35efed25bb4f8b6f8df39c763d906ef5129d85280b76147f5de86c5a",
    "data/synodal/v06_abbreviation_reviews.tsv": "e7a31bd3b6c30159b4b352dcc366493bcac6aa287cfcec983086d5fa312f1bce",
    "data/synodal/v06_exact_reviews.tsv": "a4d34e229bef0bbfaf39fc76cf74733bb547d4c6172e35a8722eeb5cedf566b2",
    "data/synodal/v06_spelling_reviews.tsv": "1fbd044392153a28ffbbfff977ca94a11356c7a04c4ec84f3606b8895df2fb9e",
    "data/synodal/v06_verification.tsv": "d9078ddd520744d121bcd34f625361a4769137bc8ba3eb8e61cbd2044c1a306e",
    "docs/SYNODAL_V06_65_PERCENT_TOP_K_COVERAGE_AUDIT.md": "d077c207ccc12a9a3217d624f3978fa2e8c3c08631505677352baa870abb9ccb",
    "reports/synodal-coverage.json": "20257bb9b6a7c7cb8a4dab1cb4a35b21b23def2ef11a8a672f03e320f64ad292",
    "reports/synodal-evaluation.json": "fc4ed256fda2cbcdef26f86e9ab57ca96477f4b001834f5b6fe88be6a861e00c",
    "reports/synodal-family-review-queue.json": "0d57ac3f520ecde921d84e8014e0d9ee5bf814564e949310c715204ce3d74e5c",
    "reports/synodal-marginal-recovery.json": "5b821b15e48ad199365699645cd477825a4b69cf1765c14764a54967d0f76353",
    "reports/synodal-v06-review-packets.json": "23df35155b6c1cfd6a1dfb045ac8812b3fc333151ada65292888c29739c5b665"
  },
  "sources": [
    {
      "path": "data/intermediate/synodal/ponomar-elizabeth-bible-2026-08-09.jsonl",
      "sha256": "ef0323df940c93c9b72a3cbb6f7adfb062ba38ffcdcf401eff5cf369c4869c26",
      "evaluation_passages": 7574,
      "source_passages": 29637
    },
    {
      "path": "data/intermediate/synodal/wikisource-church-slavonic-bible-2026-08-09.jsonl",
      "sha256": "913d9781ef511988d8bcc5d19b1b8c63c7582cd5e476f62469eff199e7c2c08f",
      "evaluation_passages": 7481,
      "source_passages": 29438
    }
  ]
}
"#;

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            value => {
                return Err(format!("unknown synodal-v06-baseline argument {value:?}").into());
            }
        }
    }

    validate(BASELINE_JSON)?;
    let output = root.join(OUTPUT);
    if check {
        if fs::read_to_string(&output).ok().as_deref() != Some(BASELINE_JSON) {
            return Err(format!("{} is stale", output.display()).into());
        }
    } else if fs::read_to_string(&output).ok().as_deref() != Some(BASELINE_JSON) {
        fs::write(&output, BASELINE_JSON)?;
    }

    println!("Synodal v0.6 baseline: locked and current");
    Ok(())
}

fn validate(source: &str) -> Result<(), Box<dyn Error>> {
    let value: Value = serde_json::from_str(source)?;
    if value.get("milestone").and_then(Value::as_str) != Some("synodal-v0.6") {
        return Err("v0.6 baseline has the wrong milestone".into());
    }
    if value.pointer("/corpus/tokens").and_then(Value::as_u64) != Some(LOCKED_TOKENS) {
        return Err("v0.6 baseline has the wrong corpus denominator".into());
    }
    if value
        .pointer("/coverage/top_k_analyzed")
        .and_then(Value::as_u64)
        != Some(LOCKED_TOP_K)
    {
        return Err("v0.6 baseline has the wrong top-k result".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_baseline_is_valid_json_with_expected_identity() {
        validate(BASELINE_JSON).expect("locked v0.6 baseline must be valid");
    }

    #[test]
    fn coverage_partition_is_exact() {
        let value: Value =
            serde_json::from_str(BASELINE_JSON).expect("locked v0.6 baseline must parse");
        let analyzed = value
            .pointer("/coverage/top_k_analyzed")
            .and_then(Value::as_u64)
            .expect("baseline top-k count");
        let numerals = value
            .pointer("/coverage/cyrillic_numerals")
            .and_then(Value::as_u64)
            .expect("baseline numeral count");
        let uncovered = value
            .pointer("/coverage/top_k_uncovered")
            .and_then(Value::as_u64)
            .expect("baseline uncovered count");
        assert_eq!(analyzed + numerals + uncovered, LOCKED_TOKENS);
    }
}
