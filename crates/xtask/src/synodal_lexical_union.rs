//! Reproducible Synodal Russian Church Slavonic lexical source-union audit.
//!
//! The ledger records every claim in the curated seed registry, the complete
//! adjudication table, and the preserved cross-source proposal queue. It does
//! not silently turn a surface match into a lexical identity or paradigm.

use crate::report_io::read_tsv;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};
use synodal_church_slavonic_core::normalize_lookup_accentless;

const LEDGER_PATH: &str = "data/synodal/lexical_source_claims.tsv";
const DECISIONS_PATH: &str = "data/synodal/lexical_source_decisions.tsv";
const SEED_PATH: &str = "data/synodal/lexemes.tsv";
const REVIEWS_PATH: &str = "data/synodal/lexical_reviews.tsv";
const QUEUE_PATH: &str = "reports/synodal-lexical-review-queue.tsv";
const JSON_REPORT_PATH: &str = "reports/synodal-lexical-union.json";
const MARKDOWN_REPORT_PATH: &str = "reports/synodal-lexical-union.md";
const LEDGER_HEADER: &str = "claim_id\tsource_id\tsource_record\tlemma\tlookup_key\tpart_of_speech\tsource_class\tunion_identity\tclassification\tengine_route\tsupport_state\tevidence\tnotes";
const CLASSIFICATIONS: &[&str] = &[
    "productive",
    "closed-irregular",
    "defective",
    "indeclinable",
    "ambiguous",
    "disputed",
    "out-of-scope",
];
const SUPPORT_STATES: &[&str] = &[
    "implemented",
    "evidence-final",
    "implementation-missing",
    "metadata-incomplete",
    "source-ambiguous",
    "not-applicable",
];
const NON_FINAL_SUPPORT_STATES: &[&str] = &[
    "implementation-missing",
    "metadata-incomplete",
    "source-ambiguous",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Claim {
    claim_id: String,
    source_id: String,
    source_record: String,
    lemma: String,
    lookup_key: String,
    part_of_speech: String,
    source_class: String,
    union_identity: String,
    classification: String,
    engine_route: String,
    support_state: String,
    evidence: String,
    notes: String,
}

#[derive(Clone, Debug)]
struct Decision {
    classification: String,
    engine_route: String,
    support_state: String,
    evidence: String,
    notes: String,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: u8,
    source_union_policy: &'static str,
    input_artifacts: Vec<InputArtifact>,
    claims: usize,
    union_identities: usize,
    by_claim_set: BTreeMap<String, usize>,
    by_part_of_speech: BTreeMap<String, usize>,
    by_classification: BTreeMap<String, usize>,
    by_support_state: BTreeMap<String, usize>,
    implementation_gaps_by_route: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct InputArtifact {
    path: &'static str,
    sha256: String,
    claim_policy: &'static str,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            value => return Err(format!("unknown synodal-lexical-union argument {value:?}").into()),
        }
    }

    let claims = derive_claims(root)?;
    validate_claims(&claims, false)?;
    let ledger = render_ledger(&claims);
    let report = report(root, &claims)?;
    let json = serde_json::to_vec_pretty(&report)?;
    let markdown = render_markdown(&report);

    if check {
        require_bytes(root, LEDGER_PATH, ledger.as_bytes())?;
        require_bytes(root, JSON_REPORT_PATH, &json)?;
        require_bytes(root, MARKDOWN_REPORT_PATH, markdown.as_bytes())?;
        println!(
            "Synodal lexical source-union ledger: current ({} non-final claims)",
            non_final_count(&claims)
        );
    } else {
        fs::write(root.join(LEDGER_PATH), ledger)?;
        fs::write(root.join(JSON_REPORT_PATH), json)?;
        fs::write(root.join(MARKDOWN_REPORT_PATH), markdown)?;
        println!(
            "wrote {LEDGER_PATH} ({} claims; {} non-final)",
            claims.len(),
            non_final_count(&claims)
        );
    }
    Ok(())
}

/// Checks deterministic artifacts while allowing explicitly visible work to
/// remain during the long-running completion goal.
pub(crate) fn check_progress(root: &Path) -> Result<(), Box<dyn Error>> {
    let claims = derive_claims(root)?;
    validate_claims(&claims, false)?;
    require_current(root, &claims)?;
    println!(
        "Synodal lexical source-union ledger: current ({} non-final claims)",
        non_final_count(&claims)
    );
    Ok(())
}

/// Completion-gate check: every source claim must have a final disposition.
pub(crate) fn check_complete(root: &Path) -> Result<(), Box<dyn Error>> {
    let claims = derive_claims(root)?;
    validate_claims(&claims, true)?;
    require_current(root, &claims)?;
    println!("Synodal lexical source-union ledger: complete");
    Ok(())
}

fn require_current(root: &Path, claims: &[Claim]) -> Result<(), Box<dyn Error>> {
    require_bytes(root, LEDGER_PATH, render_ledger(claims).as_bytes())?;
    let report = report(root, claims)?;
    require_bytes(root, JSON_REPORT_PATH, &serde_json::to_vec_pretty(&report)?)?;
    require_bytes(
        root,
        MARKDOWN_REPORT_PATH,
        render_markdown(&report).as_bytes(),
    )
}

fn derive_claims(root: &Path) -> Result<Vec<Claim>, Box<dyn Error>> {
    let decisions = read_decisions(&root.join(DECISIONS_PATH))?;
    let mut claims = read_seed_claims(&root.join(SEED_PATH), &decisions)?;
    claims.extend(read_review_claims(&root.join(REVIEWS_PATH))?);
    claims.extend(read_queue_claims(&root.join(QUEUE_PATH))?);
    claims.sort();
    Ok(claims)
}

fn read_decisions(path: &Path) -> Result<BTreeMap<String, Decision>, Box<dyn Error>> {
    let table = read_tsv(path)?;
    let lexeme_id = table.index("lexeme_id")?;
    let classification = table.index("classification")?;
    let engine_route = table.index("engine_route")?;
    let support_state = table.index("support_state")?;
    let evidence = table.index("evidence")?;
    let notes = table.index("notes")?;
    let mut decisions = BTreeMap::new();
    for row in table.rows {
        let id = row[lexeme_id].clone();
        let decision = Decision {
            classification: row[classification].clone(),
            engine_route: row[engine_route].clone(),
            support_state: row[support_state].clone(),
            evidence: row[evidence].clone(),
            notes: row[notes].clone(),
        };
        if decisions.insert(id.clone(), decision).is_some() {
            return Err(format!("{DECISIONS_PATH} repeats {id:?}").into());
        }
    }
    Ok(decisions)
}

fn read_seed_claims(
    path: &Path,
    decisions: &BTreeMap<String, Decision>,
) -> Result<Vec<Claim>, Box<dyn Error>> {
    let table = read_tsv(path)?;
    let id = table.index("id")?;
    let lemma = table.index("lemma")?;
    let pos = table.index("part_of_speech")?;
    let class = table.index("class")?;
    let source_id = table.index("source_id")?;
    let mut consumed_decisions = BTreeSet::new();
    let mut claims = Vec::with_capacity(table.rows.len());
    for row in table.rows {
        let lexeme_id = &row[id];
        let source_class = &row[class];
        let exceptional = source_class.is_empty() || source_class == "exact";
        let decision = if exceptional {
            consumed_decisions.insert(lexeme_id.clone());
            decisions.get(lexeme_id).ok_or_else(|| {
                format!("{DECISIONS_PATH} omits exact/blank seed identity {lexeme_id:?}")
            })?
        } else {
            if decisions.contains_key(lexeme_id) {
                return Err(format!(
                    "{DECISIONS_PATH} overrides productive seed identity {lexeme_id:?}"
                )
                .into());
            }
            let (classification, route) = if source_class == "indeclinable" {
                ("indeclinable", "typed-indeclinable-specification")
            } else {
                ("productive", productive_route(&row[pos]))
            };
            claims.push(Claim {
                claim_id: format!("seed:{lexeme_id}"),
                source_id: row[source_id].clone(),
                source_record: lexeme_id.clone(),
                lemma: row[lemma].clone(),
                lookup_key: lookup_key(&row[lemma]),
                part_of_speech: row[pos].clone(),
                source_class: source_class.clone(),
                union_identity: lexeme_id.clone(),
                classification: classification.into(),
                engine_route: route.into(),
                support_state: "implemented".into(),
                evidence: row[source_id].clone(),
                notes: "Curated seed identity has a typed productive class or explicit indeclinable specification; exact evidence remains higher priority.".into(),
            });
            continue;
        };
        claims.push(Claim {
            claim_id: format!("seed:{lexeme_id}"),
            source_id: row[source_id].clone(),
            source_record: lexeme_id.clone(),
            lemma: row[lemma].clone(),
            lookup_key: lookup_key(&row[lemma]),
            part_of_speech: row[pos].clone(),
            source_class: if source_class.is_empty() {
                "-".into()
            } else {
                source_class.clone()
            },
            union_identity: lexeme_id.clone(),
            classification: decision.classification.clone(),
            engine_route: decision.engine_route.clone(),
            support_state: decision.support_state.clone(),
            evidence: decision.evidence.clone(),
            notes: decision.notes.clone(),
        });
    }
    let unused = decisions
        .keys()
        .filter(|id| !consumed_decisions.contains(*id))
        .collect::<Vec<_>>();
    if !unused.is_empty() {
        return Err(format!("{DECISIONS_PATH} has unused identities: {unused:?}").into());
    }
    Ok(claims)
}

fn read_review_claims(path: &Path) -> Result<Vec<Claim>, Box<dyn Error>> {
    let table = read_tsv(path)?;
    let review_id = table.index("review_id")?;
    let lexeme_id = table.index("lexeme_id")?;
    let lemma = table.index("lemma")?;
    let pos = table.index("part_of_speech")?;
    let cell = table.index("cell")?;
    let semantic_source = table.index("semantic_source_id")?;
    let semantic_candidate = table.index("semantic_candidate_id")?;
    let attestation_source = table.index("attestation_source_id")?;
    let attestation_candidate = table.index("attestation_candidate_id")?;
    let citation = table.index("citation")?;
    let decision = table.index("decision")?;
    let review_note = table.index("review_note")?;
    table
        .rows
        .into_iter()
        .map(|row| {
            let (classification, route, support, identity) = match row[decision].as_str() {
                "rejected" => (
                    "out-of-scope",
                    "reviewed-rejection",
                    "not-applicable",
                    format!("rejected:{}", row[review_id]),
                ),
                "reviewed" if row[cell] == "indeclinable" => (
                    "indeclinable",
                    "reviewed-exact-indeclinable",
                    "implemented",
                    row[lexeme_id].clone(),
                ),
                "reviewed" => (
                    "ambiguous",
                    "reviewed-exact-form-without-paradigm-metadata",
                    "evidence-final",
                    row[lexeme_id].clone(),
                ),
                other => return Err(format!("{REVIEWS_PATH} has decision {other:?}").into()),
            };
            Ok(Claim {
                claim_id: format!("review:{}", row[review_id]),
                source_id: format!("{}+{}", row[semantic_source], row[attestation_source]),
                source_record: row[review_id].clone(),
                lemma: row[lemma].clone(),
                lookup_key: lookup_key(&row[lemma]),
                part_of_speech: row[pos].clone(),
                source_class: row[cell].clone(),
                union_identity: identity,
                classification: classification.into(),
                engine_route: route.into(),
                support_state: support.into(),
                evidence: format!(
                    "{},{};{};{}",
                    row[semantic_candidate],
                    row[attestation_candidate],
                    row[citation],
                    row[review_id]
                ),
                notes: row[review_note].clone(),
            })
        })
        .collect()
}

fn read_queue_claims(path: &Path) -> Result<Vec<Claim>, Box<dyn Error>> {
    let table = read_tsv(path)?;
    let rank = table.index("rank")?;
    let lemma = table.index("lemma")?;
    let pos = table.index("part_of_speech")?;
    let cell = table.index("cell")?;
    let semantic_candidate = table.index("semantic_candidate_id")?;
    let attestation_candidate = table.index("attestation_candidate_id")?;
    let passage = table.index("passage")?;
    let decision = table.index("decision")?;
    let reason = table.index("review_reason")?;
    table
        .rows
        .into_iter()
        .map(|row| {
            let (classification, route) = match row[decision].as_str() {
                "blocked-ambiguous-homograph" => ("ambiguous", "cross-source-homograph-ambiguity"),
                "candidate-unreviewed" => ("disputed", "cross-recension-identity-unconfirmed"),
                other => return Err(format!("{QUEUE_PATH} has decision {other:?}").into()),
            };
            let stable = format!("{}:{}", row[semantic_candidate], row[pos]);
            Ok(Claim {
                claim_id: format!(
                    "queue:{}:{}",
                    row[semantic_candidate], row[attestation_candidate]
                ),
                source_id:
                    "english-wiktionary-ocs-kaikki-2026-08-07+ponomar-elizabeth-bible-2026-08-09"
                        .into(),
                source_record: format!("queue-rank-{}", row[rank]),
                lemma: row[lemma].clone(),
                lookup_key: lookup_key(&row[lemma]),
                part_of_speech: row[pos].clone(),
                source_class: row[cell].clone(),
                union_identity: format!("proposal:{stable}"),
                classification: classification.into(),
                engine_route: route.into(),
                support_state: "evidence-final".into(),
                evidence: format!(
                    "{},{};{}",
                    row[semantic_candidate], row[attestation_candidate], row[passage]
                ),
                notes: row[reason].clone(),
            })
        })
        .collect()
}

fn productive_route(pos: &str) -> &'static str {
    match pos {
        "noun" => "typed-noun-specification",
        "adjective" => "typed-adjective-specification",
        "pronoun" => "typed-pronoun-specification",
        "determiner" => "typed-determiner-specification",
        "numeral" => "typed-numeral-specification",
        "verb" => "typed-verb-specification",
        _ => "typed-lexical-specification",
    }
}

fn lookup_key(value: &str) -> String {
    normalize_lookup_accentless(value).to_lowercase()
}

fn validate_claims(claims: &[Claim], require_complete: bool) -> Result<(), Box<dyn Error>> {
    if claims.is_empty() {
        return Err("Synodal lexical source-union ledger is empty".into());
    }
    let mut ids = BTreeSet::new();
    for claim in claims {
        let fields = [
            claim.claim_id.as_str(),
            claim.source_id.as_str(),
            claim.source_record.as_str(),
            claim.lemma.as_str(),
            claim.lookup_key.as_str(),
            claim.part_of_speech.as_str(),
            claim.source_class.as_str(),
            claim.union_identity.as_str(),
            claim.classification.as_str(),
            claim.engine_route.as_str(),
            claim.support_state.as_str(),
            claim.evidence.as_str(),
            claim.notes.as_str(),
        ];
        if fields
            .iter()
            .any(|field| field.is_empty() || field.contains(['\t', '\n', '\r']))
        {
            return Err(format!("claim {:?} has an empty or unsafe field", claim.claim_id).into());
        }
        if !ids.insert(claim.claim_id.as_str()) {
            return Err(format!("duplicate source claim {:?}", claim.claim_id).into());
        }
        if !CLASSIFICATIONS.contains(&claim.classification.as_str()) {
            return Err(format!("claim {:?} has invalid classification", claim.claim_id).into());
        }
        if !SUPPORT_STATES.contains(&claim.support_state.as_str()) {
            return Err(format!("claim {:?} has invalid support state", claim.claim_id).into());
        }
        if require_complete && NON_FINAL_SUPPORT_STATES.contains(&claim.support_state.as_str()) {
            return Err(format!(
                "claim {:?} remains in non-final support state {:?}",
                claim.claim_id, claim.support_state
            )
            .into());
        }
        match claim.support_state.as_str() {
            "evidence-final"
                if !matches!(claim.classification.as_str(), "ambiguous" | "disputed") =>
            {
                return Err(format!(
                    "claim {:?} uses evidence-final without ambiguity or dispute",
                    claim.claim_id
                )
                .into());
            }
            "not-applicable" if claim.classification != "out-of-scope" => {
                return Err(format!(
                    "claim {:?} uses not-applicable without exclusion",
                    claim.claim_id
                )
                .into());
            }
            "implemented"
                if matches!(
                    claim.classification.as_str(),
                    "ambiguous" | "disputed" | "out-of-scope"
                ) =>
            {
                return Err(format!(
                    "claim {:?} marks a non-implementation classification implemented",
                    claim.claim_id
                )
                .into());
            }
            _ => {}
        }
    }
    Ok(())
}

fn render_ledger(claims: &[Claim]) -> String {
    let mut output = String::from(LEDGER_HEADER);
    output.push('\n');
    for claim in claims {
        output.push_str(
            &[
                claim.claim_id.as_str(),
                claim.source_id.as_str(),
                claim.source_record.as_str(),
                claim.lemma.as_str(),
                claim.lookup_key.as_str(),
                claim.part_of_speech.as_str(),
                claim.source_class.as_str(),
                claim.union_identity.as_str(),
                claim.classification.as_str(),
                claim.engine_route.as_str(),
                claim.support_state.as_str(),
                claim.evidence.as_str(),
                claim.notes.as_str(),
            ]
            .join("\t"),
        );
        output.push('\n');
    }
    output
}

fn report(root: &Path, claims: &[Claim]) -> Result<Report, Box<dyn Error>> {
    Ok(Report {
        schema_version: 1,
        source_union_policy: "Every row of the curated Synodal seed registry, every admitted or rejected cross-source lexical review, and every preserved open proposal is a claim in the locked union. Curated identities merge by their stable lexeme ID. Exact-form reviews remain evidence-final when their sources do not determine a productive class. Open OCS/Synodal matches remain ambiguous or disputed rather than being guessed. Raw corpus token types are evaluation witnesses, not lexical identities, and therefore are not silently promoted into the lexical denominator.",
        input_artifacts: vec![
            input_artifact(root, SEED_PATH, "every curated seed row")?,
            input_artifact(root, REVIEWS_PATH, "every reviewed and rejected row")?,
            input_artifact(root, QUEUE_PATH, "every preserved proposal row")?,
            input_artifact(
                root,
                DECISIONS_PATH,
                "every exceptional exact or classless seed row",
            )?,
        ],
        claims: claims.len(),
        union_identities: claims
            .iter()
            .map(|claim| claim.union_identity.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        by_claim_set: count_by(claims.iter().map(|claim| {
            if claim.claim_id.starts_with("seed:") {
                "curated-seed"
            } else if claim.claim_id.starts_with("review:") {
                "adjudicated-review"
            } else {
                "preserved-proposal"
            }
        })),
        by_part_of_speech: count_by(claims.iter().map(|claim| claim.part_of_speech.as_str())),
        by_classification: count_by(claims.iter().map(|claim| claim.classification.as_str())),
        by_support_state: count_by(claims.iter().map(|claim| claim.support_state.as_str())),
        implementation_gaps_by_route: count_by(
            claims
                .iter()
                .filter(|claim| NON_FINAL_SUPPORT_STATES.contains(&claim.support_state.as_str()))
                .map(|claim| claim.engine_route.as_str()),
        ),
    })
}

fn input_artifact(
    root: &Path,
    path: &'static str,
    claim_policy: &'static str,
) -> Result<InputArtifact, Box<dyn Error>> {
    Ok(InputArtifact {
        path,
        sha256: format!("{:x}", Sha256::digest(fs::read(root.join(path))?)),
        claim_policy,
    })
}

fn count_by<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_owned()).or_default() += 1;
    }
    counts
}

fn non_final_count(claims: &[Claim]) -> usize {
    claims
        .iter()
        .filter(|claim| NON_FINAL_SUPPORT_STATES.contains(&claim.support_state.as_str()))
        .count()
}

fn render_markdown(report: &Report) -> String {
    let mut output = String::from("# Synodal Lexical Source Union\n\n");
    output.push_str("This report is generated by `cargo xtask synodal-lexical-union`. It records source claims, including rejected and ambiguous rows, rather than silently merging spellings.\n\n");
    output.push_str(&format!("- Source claims: {}\n", report.claims));
    output.push_str(&format!(
        "- Stable union identities: {}\n\n",
        report.union_identities
    ));
    output.push_str("## Claim sets\n\n| Set | Claims |\n|---|---:|\n");
    append_counts(&mut output, &report.by_claim_set);
    output.push_str("\n## Support states\n\n| State | Claims |\n|---|---:|\n");
    append_counts(&mut output, &report.by_support_state);
    output.push_str("\n## Classifications\n\n| Classification | Claims |\n|---|---:|\n");
    append_counts(&mut output, &report.by_classification);
    output.push_str("\n## Confirmed implementation gaps\n\n| Route | Claims |\n|---|---:|\n");
    append_counts(&mut output, &report.implementation_gaps_by_route);
    output.push_str("\nThe Ponomar and Wikisource corpora are target-form witnesses and evaluation inputs. Their unannotated token types are not stable lemma/POS identities, so treating every token spelling as a lexeme would fabricate the very classifications this ledger is designed to prevent.\n");
    output
}

fn append_counts(output: &mut String, counts: &BTreeMap<String, usize>) {
    if counts.is_empty() {
        output.push_str("| _none_ | 0 |\n");
    } else {
        for (key, count) in counts {
            output.push_str(&format!("| `{key}` | {count} |\n"));
        }
    }
}

fn require_bytes(root: &Path, path: &str, expected: &[u8]) -> Result<(), Box<dyn Error>> {
    if fs::read(root.join(path)).ok().as_deref() == Some(expected) {
        Ok(())
    } else {
        Err(format!("stale {path}; rerun cargo xtask synodal-lexical-union").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_support_states_match_final_classifications() {
        let claim = |classification: &str, support_state: &str| Claim {
            claim_id: format!("test:{classification}:{support_state}"),
            source_id: "source".into(),
            source_record: "record".into(),
            lemma: "слово".into(),
            lookup_key: "слово".into(),
            part_of_speech: "noun".into(),
            source_class: "class".into(),
            union_identity: "identity".into(),
            classification: classification.into(),
            engine_route: "route".into(),
            support_state: support_state.into(),
            evidence: "evidence".into(),
            notes: "notes".into(),
        };
        assert!(validate_claims(&[claim("ambiguous", "evidence-final")], true).is_ok());
        assert!(validate_claims(&[claim("out-of-scope", "not-applicable")], true).is_ok());
        assert!(validate_claims(&[claim("productive", "implemented")], true).is_ok());
        assert!(validate_claims(&[claim("productive", "implementation-missing")], true).is_err());
        assert!(validate_claims(&[claim("productive", "evidence-final")], true).is_err());
    }
}
