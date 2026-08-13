use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use sha2::{Digest, Sha256};

const OUTPUT: &str = "docs/SYNODAL_V07_70_PERCENT_TOP_K_COVERAGE_AUDIT.md";
const FROZEN_OUTPUT_SHA256: &str =
    "7cabe2bed7ae70089372446236ed48a8f0d5fc15cecd7805c1d8bb4ed056e938";
const BASELINE: &str = "reports/synodal-v06-baseline.json";
const COVERAGE: &str = "reports/synodal-coverage.json";
const EVALUATION: &str = "reports/synodal-evaluation.json";
const EXTRACTION: &str = "reports/synodal-extraction.json";
const MARGINAL: &str = "reports/synodal-marginal-recovery.json";
const PACKETS: &str = "reports/synodal-v07-review-packets.json";
const VARIANTS: &str = "data/synodal/v07_variant_reviews.tsv";
const ABBREVIATIONS: &str = "data/synodal/v07_abbreviation_reviews.tsv";
const IDENTITY_CORRECTIONS: &str = "data/synodal/v07_identity_corrections.tsv";
const EVIDENCE_CORRECTIONS: &str = "data/synodal/v07_evidence_corrections.tsv";
const VERIFICATION: &str = "data/synodal/v07_verification.tsv";

const LOCKED_PASSAGES: u64 = 74_130;
const LOCKED_TOKENS: u64 = 1_313_344;
const LOCKED_TYPES: u64 = 57_476;
const BASELINE_TOP_K: u64 = 853_770;
const TARGET_66: u64 = 866_808;
const TARGET_67: u64 = 879_941;
const TARGET_68: u64 = 893_074;
const TARGET_69: u64 = 906_208;
const TARGET_70: u64 = 919_341;
const SECONDARY_UNRESOLVED: u64 = 392_618;

const REQUIRED_VERIFICATION: &[&str] = &[
    "cargo fmt --all --check",
    "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    "cargo test --workspace --all-targets --all-features",
    "cargo test --workspace --doc",
    "cargo xtask synodal-fixture-bootstrap",
    "fixture reconstruction twice from separate empty temporary caches",
    "cargo xtask synodal-check",
    "cargo xtask synodal-coverage --fixture --offline --check",
    "cargo xtask synodal-coverage --offline --check",
    "cargo xtask synodal-lexical-review-queue --check",
    "cargo xtask synodal-evaluation-queue --check",
    "cargo xtask synodal-family-review-queue --check",
    "cargo xtask synodal-marginal-recovery --check",
    "cargo xtask synodal-v04-audit --check",
    "cargo xtask synodal-v05-baseline --check",
    "cargo xtask synodal-v05-audit --check",
    "cargo xtask synodal-v06-review-packets --check",
    "cargo xtask synodal-v06-audit --check",
    "cargo xtask synodal-v07-baseline --check",
    "cargo xtask synodal-v07-review-packets --check",
    "cargo xtask synodal-v07-apply --check",
    "cargo xtask synodal-v07-audit --check",
    "cargo xtask check-all",
    "cargo xtask guard-witnesses",
    "cargo xtask synodal-guard-witnesses",
    "cargo check -p synodal-church-slavonic-core --no-default-features",
    "cargo check -p synodal-church-slavonic --no-default-features",
    "cargo check -p synodal-church-slavonic-dictionary --no-default-features",
    "cargo check -p synodal-church-slavonic-core --target wasm32-unknown-unknown --no-default-features",
    "cargo check -p synodal-church-slavonic --target wasm32-unknown-unknown --no-default-features",
    "cargo check -p synodal-church-slavonic-dictionary --target wasm32-unknown-unknown --no-default-features",
    "cargo package --list --allow-dirty -p synodal-church-slavonic-core",
    "cargo package --list --allow-dirty -p synodal-church-slavonic",
    "cargo package --list --allow-dirty -p synodal-church-slavonic-dictionary",
    "cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic-core",
    "cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic",
    "cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic-dictionary",
    "cargo xtask synodal-bootstrap --offline --cache references/downloads",
    "git diff --check",
    "full intended diff review against merge base",
    "final independent full-diff review",
];

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            value => return Err(format!("unknown synodal-v07-audit argument {value:?}").into()),
        }
    }
    let output = root.join(OUTPUT);
    let bytes = fs::read(&output)?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != FROZEN_OUTPUT_SHA256 {
        return Err(format!(
            "{} no longer matches the frozen v0.7 checkpoint: expected {}, found {}",
            output.display(),
            FROZEN_OUTPUT_SHA256,
            actual
        )
        .into());
    }
    if !check {
        println!(
            "Synodal v0.7 audit is an immutable historical snapshot; no live v0.8 reports were rendered"
        );
    }
    println!(
        "Synodal v0.7 evidence-backed 70% top-k coverage audit: immutable historical checkpoint"
    );
    Ok(())
}

#[allow(dead_code)]
fn render(root: &Path) -> Result<String, Box<dyn Error>> {
    let baseline = read_json(&root.join(BASELINE))?;
    let coverage = read_json(&root.join(COVERAGE))?;
    let evaluation = read_json(&root.join(EVALUATION))?;
    let extraction = read_json(&root.join(EXTRACTION))?;
    let marginal = read_json(&root.join(MARGINAL))?;
    let packets = read_json(&root.join(PACKETS))?;
    let variants = read_tsv(&root.join(VARIANTS))?;
    let abbreviations = read_tsv(&root.join(ABBREVIATIONS))?;
    let identity_corrections = read_tsv(&root.join(IDENTITY_CORRECTIONS))?;
    let evidence_corrections = read_tsv(&root.join(EVIDENCE_CORRECTIONS))?;
    let verification = read_tsv(&root.join(VERIFICATION))?;
    let lexical = read_tsv(&root.join("data/synodal/lexical_reviews.tsv"))?;
    let exact_forms = read_tsv(&root.join("data/synodal/exact_forms.tsv"))?;
    let evidence = read_tsv(&root.join("data/synodal/reviewed_evidence.tsv"))?;
    let evaluation_table = read_tsv(&root.join("data/synodal/evaluation.tsv"))?;
    let abbreviation_evaluation = read_tsv(&root.join("data/synodal/abbreviation_evaluation.tsv"))?;
    let exact_reviews = load_exact_reviews(root)?;

    validate_contract(&baseline, &coverage, &evaluation, &marginal, &packets)?;
    validate_ledgers(
        &packets,
        &exact_reviews,
        &variants,
        &abbreviations,
        &identity_corrections,
        &evidence_corrections,
        &verification,
    )?;
    validate_evidence_boundary(
        &lexical,
        &exact_forms,
        &evidence,
        &evaluation_table,
        &abbreviation_evaluation,
    )?;

    let summary = object(&coverage, "summary")?;
    let total = number(summary, "total_tokens")?;
    let top_1 = number(summary, "top_1_analyzed")?;
    let top_k = number(summary, "top_k_analyzed")?;
    let ambiguous = number(summary, "ambiguous")?;
    let unresolved = number(summary, "unresolved")?;
    let numerals = pointer_number(&coverage, "/by_status/cyrillic-numeral")?;
    let top_k_uncovered = total
        .checked_sub(top_k + numerals)
        .ok_or("coverage partition exceeds denominator")?;
    let base_coverage = object(&baseline, "coverage")?;
    let base_registry = object(&baseline, "registry")?;
    let base_evaluation = object(&baseline, "evaluation")?;
    let base_top_1 = number(base_coverage, "top_1_analyzed")?;
    let base_top_k = number(base_coverage, "top_k_analyzed")?;
    let base_ambiguous = number(base_coverage, "ambiguous")?;
    let base_unresolved = number(base_coverage, "unresolved")?;
    let reviewed_lexemes = lexical
        .rows
        .iter()
        .filter(|row| row.get(15).is_some_and(|value| value == "reviewed"))
        .count() as u64
        + table_count(&extraction, "lexemes.tsv")?;
    let reviewed_senses = lexical
        .rows
        .iter()
        .filter(|row| row.get(15).is_some_and(|value| value == "reviewed"))
        .count() as u64
        + table_count(&extraction, "senses.tsv")?;
    let forms = table_count(&extraction, "exact_forms.tsv")?;
    let typed_abbreviations = table_count(&extraction, "abbreviations.tsv")?;
    let eval_rows = root_number(&evaluation, "fixture_rows")?;
    let abbreviation_eval_rows = root_number(&evaluation, "abbreviation_fixture_rows")?;
    let variant_predicted = sum_column(&variants, "predicted_unique_tokens")?;
    let abbreviation_predicted = sum_column(&abbreviations, "predicted_unique_tokens")?;

    let latest_admitted = exact_reviews
        .values()
        .filter(|row| row.fields.get(1).is_some_and(|value| value == "admitted"))
        .count();
    let latest_deferred = exact_reviews
        .values()
        .filter(|row| row.fields.get(1).is_some_and(|value| value == "deferred"))
        .count();
    let latest_rejected = exact_reviews
        .values()
        .filter(|row| row.fields.get(1).is_some_and(|value| value == "rejected"))
        .count();

    let mut out = String::new();
    out.push_str("# Synodal v0.7 evidence-backed 70% top-k coverage audit\n\n");
    out.push_str("This file is generated by `cargo xtask synodal-v07-audit`. It freezes the evidence-backed result of the canonical `Strict` resolver under `SynodalLiturgical`; frequencies, family similarities, queue scores, and marginal projections remain diagnostics and never license runtime data.\n\n");

    out.push_str("## Outcome\n\n");
    out.push_str(&format!(
        "The primary gate passes: **{top_k} of {total} tokens ({})** have at least one evidence-qualified top-k analysis. The strict threshold is {TARGET_70}, leaving a **{}-token margin**. Canonical gain over v0.6 is **+{} tokens**. Unresolved is **{unresolved} ({})**, {} tokens below the secondary ceiling of {SECONDARY_UNRESOLVED}.\n\n",
        percent(top_k, total),
        top_k - TARGET_70,
        top_k - base_top_k,
        percent(unresolved, total),
        SECONDARY_UNRESOLVED - unresolved,
    ));
    out.push_str("| Measure | Locked v0.6 | Final v0.7 | Delta |\n|---|---:|---:|---:|\n");
    metric_row(&mut out, "Top-1 analyzed", base_top_1, top_1, total);
    metric_row(&mut out, "Top-k analyzed", base_top_k, top_k, total);
    metric_row(&mut out, "Ambiguous", base_ambiguous, ambiguous, total);
    metric_row(&mut out, "Unresolved", base_unresolved, unresolved, total);
    out.push_str("\n| Milestone | Minimum | Final margin |\n|---|---:|---:|\n");
    for (name, threshold) in [
        ("66%", TARGET_66),
        ("67%", TARGET_67),
        ("68%", TARGET_68),
        ("69%", TARGET_69),
        ("70%", TARGET_70),
    ] {
        out.push_str(&format!(
            "| {name} | {threshold} | +{} |\n",
            top_k - threshold
        ));
    }

    out.push_str("\n## Locked corpus, sources, partitions, and accounting\n\n");
    out.push_str(&format!(
        "The v0.6 baseline and v0.7 result use the same target recension (`synodal-russian`), policy (`Strict`), profile (`SynodalLiturgical`), tokenizer (`synodal-dictionary-tokenize-v1`), candidate cutoff and resolver-status meanings. Both retain {LOCKED_PASSAGES} passages, {LOCKED_TOKENS} tokens, {LOCKED_TYPES} token types, and the pinned Ponomar/Wikisource 2026-08-09 source IDs and revisions. Source/evaluation partitions are unchanged. The exact partition is `{top_k} top-k + {numerals} numerals + {top_k_uncovered} top-k-uncovered = {total}`.\n\n"
    ));
    out.push_str("| Pinned artifact | SHA-256 | Source passages | Evaluation passages |\n|---|---|---:|---:|\n");
    for source in baseline
        .get("sources")
        .and_then(Value::as_array)
        .ok_or("baseline omits source identities")?
    {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} |\n",
            escape(source.get("path").and_then(Value::as_str).unwrap_or("")),
            source.get("sha256").and_then(Value::as_str).unwrap_or(""),
            source
                .get("source_passages")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            source
                .get("evaluation_passages")
                .and_then(Value::as_u64)
                .unwrap_or(0),
        ));
    }
    out.push_str("\n| Resolver status | Tokens | Percent |\n|---|---:|---:|\n");
    render_numeric_object(&mut out, object(&coverage, "by_status")?, total);

    out.push_str("\n## Registry and evaluation growth\n\n");
    out.push_str("| Registry | v0.6 | v0.7 | Delta |\n|---|---:|---:|---:|\n");
    delta_row(
        &mut out,
        "Reviewed lexemes",
        number(base_registry, "reviewed_lexemes")?,
        reviewed_lexemes,
    );
    delta_row(
        &mut out,
        "Reviewed senses",
        number(base_registry, "reviewed_senses")?,
        reviewed_senses,
    );
    delta_row(
        &mut out,
        "Exact forms",
        number(base_registry, "generated_exact_forms")?,
        forms,
    );
    delta_row(
        &mut out,
        "Typed abbreviations",
        number(base_registry, "typed_abbreviations")?,
        typed_abbreviations,
    );
    delta_row(
        &mut out,
        "Held-out exact cells",
        number(base_evaluation, "morphological_cells")?,
        eval_rows,
    );
    delta_row(
        &mut out,
        "Held-out abbreviation cells",
        number(base_evaluation, "abbreviation_cells")?,
        abbreviation_eval_rows,
    );
    out.push_str(&format!(
        "\nNo new productive class, principal part, transformation, broad spelling normalizer, or weakened resolver policy was used. v0.7 admissions are bounded exact cells, explicit accent/case variants, and typed abbreviation rows. Independent consistency review merged {} duplicate identities and replaced {} source witnesses that overlapped held-out passages.\n\n",
        identity_corrections.rows.len(),
        evidence_corrections.rows.len(),
    ));

    out.push_str("## Coverage pools, gaps, and marginal recomputation\n\n");
    out.push_str("Route pools classify unresolved tokens and overlap by design. Gap counts classify resolver failure modes and may also overlap. Raw family candidates contain repeated surface memberships. Greedy marginal batches remove overlap in ordering. None of these diagnostic views is added to realized coverage.\n\n");
    out.push_str("| Route pool | v0.6 | Final v0.7 |\n|---|---:|---:|\n");
    render_before_after_objects(
        &mut out,
        map_object(base_coverage, "estimated_recovery_by_route")?,
        object(&coverage, "estimated_recovery_by_route")?,
    );
    out.push_str("\n| Gap class | v0.6 | Final v0.7 |\n|---|---:|---:|\n");
    render_before_after_objects(
        &mut out,
        map_object(base_coverage, "by_gap")?,
        object(&coverage, "by_gap")?,
    );
    out.push_str(&format!(
        "\nInitial v0.6 marginal ordering contained {} batches and {} overlap-adjusted diagnostic tokens, projecting {} top-k if every diagnostic batch were valid. Final recomputation contains {} batches and {} diagnostic tokens, projects {}, and reports zero tokens needed for 70%. The realized resolver gain is {}, not either projection.\n\n",
        pointer_number(&baseline, "/marginal/batches")?,
        pointer_number(&baseline, "/marginal/overlap_adjusted_tokens")?,
        pointer_number(&baseline, "/marginal/diagnostic_projected_top_k")?,
        marginal
            .get("batches")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        root_number(&marginal, "diagnostic_recovery")?,
        root_number(&marginal, "diagnostic_projected_top_k")?,
        top_k - BASELINE_TOP_K,
    ));
    render_marginal_routes(&mut out, &baseline, &marginal)?;

    out.push_str("## Admission and correction ledgers\n\n");
    out.push_str(&format!(
        "Across the complete wave history, the latest decision for each stable packet ID records {latest_admitted} admitted, {latest_deferred} deferred, and {latest_rejected} rejected packets. Packet-local `realized_unique_tokens` remains zero by design: overlapping exact analyses cannot be given additive credit. The authoritative canonical result is the aggregate +{} resolver delta. Variant predictions total {variant_predicted}; typed-abbreviation predictions total {abbreviation_predicted}; these projections also overlap and are not summed into realized coverage.\n\n",
        top_k - BASELINE_TOP_K,
    ));
    out.push_str("### Admitted exact packets\n\n| Packet | Decision ledger | Predicted | Canonical attribution |\n|---|---|---:|---|\n");
    for (packet_id, row) in exact_reviews
        .iter()
        .filter(|(_, row)| row.fields.get(1).is_some_and(|value| value == "admitted"))
    {
        out.push_str(&format!(
            "| `{}` | `{}` | packet-time diagnostic | aggregate-only; non-additive |\n",
            escape(packet_id),
            escape(&row.path),
        ));
    }
    render_table(
        &mut out,
        "Explicit accent/case variants",
        &variants,
        &[
            "review_id",
            "lexeme_id",
            "cell",
            "printed",
            "base_printed",
            "predicted_unique_tokens",
            "source_passage",
            "evaluation_passage",
            "decision",
            "review_note",
        ],
    )?;
    render_table(
        &mut out,
        "Typed abbreviation adjudication",
        &abbreviations,
        &[
            "review_id",
            "lexeme_id",
            "sense_id",
            "cell",
            "printed",
            "normative_evidence_id",
            "predicted_unique_tokens",
            "source_passage",
            "evaluation_passage",
            "decision",
            "review_note",
        ],
    )?;
    render_table(
        &mut out,
        "Identity merges",
        &identity_corrections,
        &[
            "correction_id",
            "obsolete_lexeme_id",
            "canonical_lexeme_id",
            "semantic_candidate_id",
            "decision",
            "review_note",
        ],
    )?;
    render_table(
        &mut out,
        "Passage-overlap evidence replacements",
        &evidence_corrections,
        &[
            "correction_id",
            "obsolete_evidence_id",
            "replacement_evidence_id",
            "source_id",
            "source_passage",
            "decision",
            "review_note",
        ],
    )?;

    out.push_str("## New reviewed identities and exact runtime facts\n\n");
    out.push_str("Each row below is source data, not an inferred family. Semantic identity, exact target spelling, source morphology, and passage-held-out evaluation remain separate evidence roles. Entries removed by the correction ledger are absent.\n\n");
    out.push_str("### v0.7 lexical and semantic identities\n\n| Review | Lexeme | Sense | Lemma | POS | Printed | Gloss | Semantic evidence | Target citation |\n|---|---|---|---|---|---|---|---|---|\n");
    for row in lexical.rows.iter().filter(|row| {
        row.first()
            .is_some_and(|value| value.starts_with("review:v07:"))
            && row.get(15).is_some_and(|value| value == "reviewed")
    }) {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | {} | `{}` | `{}` `{}` |\n",
            escape(&row[0]),
            escape(&row[1]),
            escape(&row[2]),
            escape(&row[3]),
            escape(&row[4]),
            escape(&row[7]),
            escape(&row[8]),
            escape(&row[11]),
            escape(&row[12]),
            escape(&row[14]),
        ));
    }
    out.push_str("\n### v0.7-backed exact forms\n\n| Lexeme | Cell | Expanded | Printed | Evidence | Source kind |\n|---|---|---|---|---|---|\n");
    for row in exact_forms.rows.iter().filter(|row| {
        row.get(4).is_some_and(|ids| {
            ids.split(',')
                .any(|id| id.starts_with("v07-") || id.starts_with("review:v07:"))
        })
    }) {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |\n",
            escape(&row[0]),
            escape(&row[1]),
            escape(&row[2]),
            escape(&row[3]),
            escape(&row[4]),
            escape(&row[5]),
        ));
    }

    out.push_str("\n## Evaluation, ambiguity, and leakage\n\n");
    let expanded = object(&evaluation, "expanded")?;
    let printed = object(&evaluation, "printed")?;
    out.push_str(&format!(
        "Expanded evaluation is {}/{} top-1 and {}/{} top-k; printed evaluation is {}/{} top-1 and {}/{} top-k. Typed abbreviations are {}/{} top-k. All top-k expectations and exact-registry round trips pass. Genuine alternatives remain visible in top-k, accounting for the higher ambiguity total.\n\n",
        number(expanded, "top_1_correct")?,
        number(expanded, "total")?,
        number(expanded, "top_k_correct")?,
        number(expanded, "total")?,
        number(printed, "top_1_correct")?,
        number(printed, "total")?,
        number(printed, "top_k_correct")?,
        number(printed, "total")?,
        pointer_number(&evaluation, "/abbreviation_expansion/top_k_correct")?,
        pointer_number(&evaluation, "/abbreviation_expansion/total")?,
    ));
    for (key, label) in [
        ("by_policy", "Policy"),
        ("by_morphological_system", "Morphological system"),
        ("by_attestation_status", "Attestation"),
        ("by_regularity", "Regularity"),
        ("by_provenance_path", "Provenance route"),
    ] {
        render_metric_map(&mut out, &evaluation, key, label)?;
    }
    out.push_str(&format!(
        "Masked reconstruction is {}/{} expanded top-k and {}/{} printed top-k. The v0.7 audit independently finds zero source/evaluation passage overlap among v0.7 reviewed evidence. `synodal-check` also enforces target recension, NFC text, whole-token candidates, evidence state, and generated/runtime consistency.\n\n",
        pointer_number(&evaluation, "/leakage/masked_expanded/top_k_correct")?,
        pointer_number(&evaluation, "/leakage/masked_expanded/total")?,
        pointer_number(&evaluation, "/leakage/masked_printed/top_k_correct")?,
        pointer_number(&evaluation, "/leakage/masked_printed/total")?,
    ));
    render_disagreements(
        &mut out,
        &evaluation,
        "expanded_disagreements",
        "Expanded top-1 disagreements",
    )?;
    render_disagreements(
        &mut out,
        &evaluation,
        "printed_disagreements",
        "Printed top-1 disagreements",
    )?;

    out.push_str("## Leading uncovered work before and after\n\n");
    out.push_str("The initial v0.6 horizon is represented by its preserved packet queue. The final list is recomputed from the canonical post-v0.7 gaps. Neither list licenses admission.\n\n### Initial v0.6 packet horizon\n\n| Rank | Surface | Predicted tokens | Route | Decision |\n|---:|---|---:|---|---|\n");
    let v06_packets = read_json(&root.join("reports/synodal-v06-review-packets.json"))?;
    for packet in v06_packets
        .get("packets")
        .and_then(Value::as_array)
        .ok_or("v0.6 packet report omits packets")?
        .iter()
        .take(30)
    {
        out.push_str(&format!(
            "| {} | `{}` | {} | `{}` | `{}` |\n",
            packet.get("rank").and_then(Value::as_u64).unwrap_or(0),
            escape(packet.get("surface").and_then(Value::as_str).unwrap_or("")),
            packet
                .get("predicted_unique_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            escape(
                packet
                    .get("evidence_lane")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(packet.get("decision").and_then(Value::as_str).unwrap_or("")),
        ));
    }
    out.push_str("\n### Final canonical gaps\n\n| Rank | Surface | Top-k-uncovered | Gap | Suggested route |\n|---:|---|---:|---|---|\n");
    for (index, gap) in coverage
        .get("gaps")
        .and_then(Value::as_array)
        .ok_or("coverage omits gaps")?
        .iter()
        .filter(|gap| {
            gap.get("top_k_uncovered_frequency")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                > 0
        })
        .take(30)
        .enumerate()
    {
        out.push_str(&format!(
            "| {} | `{}` | {} | `{}` | `{}` |\n",
            index + 1,
            escape(gap.get("original").and_then(Value::as_str).unwrap_or("")),
            gap.get("top_k_uncovered_frequency")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            escape(
                gap.get("kind")
                    .or_else(|| gap.get("gap"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(
                gap.get("suggested_action")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
        ));
    }
    out.push_str("\n### Next evidence queue after 70%\n\n| Rank | Surface | Candidate tokens | Identity | Blocker/risk |\n|---:|---|---:|---|---|\n");
    for packet in packets
        .get("packets")
        .and_then(Value::as_array)
        .ok_or("v0.7 packet report omits packets")?
        .iter()
        .take(30)
    {
        let risks = packet
            .get("risk_flags")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        out.push_str(&format!(
            "| {} | `{}` | {} | `{}` | {} |\n",
            packet.get("rank").and_then(Value::as_u64).unwrap_or(0),
            escape(packet.get("surface").and_then(Value::as_str).unwrap_or("")),
            packet
                .get("predicted_unique_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            escape(
                packet
                    .get("identity_status")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(&risks),
        ));
    }

    let identity_conflicts = packets
        .get("identity_conflicts")
        .and_then(Value::as_array)
        .ok_or("v0.7 packet report omits identity conflicts")?;
    out.push_str(&format!(
        "\n### Blocked identity conflicts\n\nThe recomputed queue retains {} source-morphology conflicts. They are review diagnostics with explicit blockers, not runtime identities or coverage.\n\n| Surface | Tokens | Proposed lemma | POS | Blocker |\n|---|---:|---|---|---|\n",
        identity_conflicts.len()
    ));
    for conflict in identity_conflicts.iter().take(30) {
        out.push_str(&format!(
            "| `{}` | {} | `{}` | `{}` | `{}` |\n",
            escape(
                conflict
                    .get("surface")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            conflict
                .get("top_k_uncovered_frequency")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            escape(
                conflict
                    .get("syntacticus_lemma")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(
                conflict
                    .get("syntacticus_part_of_speech")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(
                conflict
                    .get("blocker")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
        ));
    }

    out.push_str("\n## Unicode, runtime, reconstruction, and verification\n\n");
    out.push_str("Exact NFC combining order, positional letters, capitalization, titla, superscripts, context restrictions, and ambiguity are preserved. Missing or reordered marks, unsupported cells, mixed scripts, private-use characters, deceptive substrings, and unreviewed contractions remain abstentions or negative controls. Runtime crates stay generated, deterministic, filesystem-free, network-free, and `no_std`/WASM-compatible; acquisition remains in extractor/xtask tooling. Package inspection excludes corpora, reports, queues, references, caches, and source artifacts.\n\n| Verification command or review | Result | Note |\n|---|---|---|\n");
    for row in &verification.rows {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            escape(&row[0]),
            escape(&row[1]),
            escape(&row[2]),
        ));
    }

    out.push_str("\n## Independent review, corrections, and remaining risk\n\n");
    out.push_str("The separate full-diff pass covers every intended tracked and untracked v0.3-v0.7 file against the available merge-base state, including runtime code, data and evidence ledgers, generated registries, evaluation, corpus accounting, queues, reports, docs, tests, workflows, no-std/WASM builds, package lists, and publish dry-runs. Confirmed findings fixed during v0.7 were: one duplicate honour identity, six additional duplicate runtime identities, three v0.7 runtime witnesses overlapping held-out passages, stale queue decisions after recomputation, and an exact-abbreviation family validator that incorrectly required exactly one member after a reviewed capitalization variant was added. The final review additionally corrected the live marginal JSON's stale 65% primary target, added explicit 66–70 milestone accounting, separated the packet report's locked baseline from current top-k, retained blocked acquisition conflicts instead of requiring their disappearance, sealed the v0.4 marginal baseline against accidental refresh, and corrected stale current-version documentation and tests.\n\nPotential duplicate candidate IDs shared by governed grammar-section or passage records were rejected as findings when the source candidate legitimately witnesses several separately reviewed listed forms or tokens; those IDs are not lexical-entry identities. Candidate IDs attached only to earlier rejected lexical rows were also rejected as findings because rejected rows do not establish runtime identities. Decreased top-1 precision was rejected as a defect because every held-out expectation remains in top-k and the change reflects genuine independently evidenced alternatives. No P0/P1 finding remains. Remote CI, PR review threads, publication, and pull-request state were not inspected or changed. Remaining risk is evidence acquisition for the ranked post-70% queue; those candidates remain explicitly deferred and cannot affect runtime output.\n");

    Ok(out)
}

fn validate_contract(
    baseline: &Value,
    coverage: &Value,
    evaluation: &Value,
    marginal: &Value,
    packets: &Value,
) -> Result<(), Box<dyn Error>> {
    require_string(baseline, "milestone", "synodal-v0.6")?;
    require_string(baseline, "target_recension", "synodal-russian")?;
    require_string(baseline, "generation_policy", "strict")?;
    require_string(baseline, "orthography_profile", "synodal-liturgical")?;
    require_string(
        baseline,
        "tokenizer_contract",
        "synodal-dictionary-tokenize-v1",
    )?;
    require_number(baseline, "/corpus/passages", LOCKED_PASSAGES)?;
    require_number(baseline, "/corpus/tokens", LOCKED_TOKENS)?;
    require_number(baseline, "/corpus/token_types", LOCKED_TYPES)?;
    require_number(baseline, "/coverage/top_k_analyzed", BASELINE_TOP_K)?;
    require_string(coverage, "target_recension", "synodal-russian")?;
    require_string(coverage, "generation_policy", "Strict")?;
    require_string(coverage, "orthography_profile", "SynodalLiturgical")?;
    require_number(coverage, "/passages", LOCKED_PASSAGES)?;
    require_number(coverage, "/summary/total_tokens", LOCKED_TOKENS)?;
    require_number(coverage, "/token_types", LOCKED_TYPES)?;
    let top_k = pointer_number(coverage, "/summary/top_k_analyzed")?;
    let unresolved = pointer_number(coverage, "/summary/unresolved")?;
    if top_k < TARGET_70 {
        return Err(
            format!("v0.7 requires at least {TARGET_70} top-k tokens; found {top_k}").into(),
        );
    }
    if unresolved > SECONDARY_UNRESOLVED {
        return Err(format!(
            "v0.7 secondary unresolved ceiling is {SECONDARY_UNRESOLVED}; found {unresolved}"
        )
        .into());
    }
    for profile in ["expanded", "printed"] {
        let total = pointer_number(evaluation, &format!("/{profile}/total"))?;
        let top_k_correct = pointer_number(evaluation, &format!("/{profile}/top_k_correct"))?;
        if total != top_k_correct {
            return Err(format!("{profile} evaluation lost a top-k expectation").into());
        }
    }
    if pointer_number(evaluation, "/exact_registry_expanded_round_trip/total")?
        != pointer_number(
            evaluation,
            "/exact_registry_expanded_round_trip/top_k_correct",
        )?
        || pointer_number(evaluation, "/exact_registry_printed_round_trip/total")?
            != pointer_number(
                evaluation,
                "/exact_registry_printed_round_trip/top_k_correct",
            )?
    {
        return Err("exact registry round-trip is incomplete".into());
    }
    if root_number(marginal, "current_top_k")? != top_k
        || root_number(marginal, "target_top_k")? != TARGET_70
        || root_number(marginal, "tokens_needed_for_target")? != 0
    {
        return Err("marginal report disagrees with canonical coverage".into());
    }
    let milestones = marginal
        .get("milestones")
        .and_then(Value::as_array)
        .ok_or("marginal report omits v0.7 milestones")?;
    for (percent, target) in [
        (66_u64, TARGET_66),
        (67, TARGET_67),
        (68, TARGET_68),
        (69, TARGET_69),
        (70, TARGET_70),
    ] {
        let milestone = milestones
            .iter()
            .find(|row| row.get("percent").and_then(Value::as_u64) == Some(percent))
            .ok_or_else(|| format!("marginal report omits {percent}% milestone"))?;
        if root_number(milestone, "target_top_k")? != target
            || root_number(milestone, "basis_points")? != percent * 100
            || root_number(milestone, "tokens_needed")? != 0
            || root_number(milestone, "margin")? != top_k - target
        {
            return Err(format!("marginal {percent}% milestone disagrees with coverage").into());
        }
    }
    if root_number(packets, "current_top_k")? != top_k
        || root_number(packets, "tokens_needed_for_70_percent")? != 0
    {
        return Err("v0.7 packet report disagrees with final coverage".into());
    }
    let conflicts = packets
        .get("identity_conflicts")
        .and_then(Value::as_array)
        .ok_or("v0.7 packet report omits identity conflicts")?;
    if conflicts.iter().any(|conflict| {
        conflict
            .get("blocker")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
    }) {
        return Err("v0.7 packet identity conflict omits its blocker".into());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_ledgers(
    packets: &Value,
    exact_reviews: &BTreeMap<String, ExactReview>,
    variants: &Table,
    abbreviations: &Table,
    identity_corrections: &Table,
    evidence_corrections: &Table,
    verification: &Table,
) -> Result<(), Box<dyn Error>> {
    let packet_ids = packets
        .get("packets")
        .and_then(Value::as_array)
        .ok_or("packet report omits packets")?
        .iter()
        .map(|packet| {
            packet
                .get("packet_id")
                .and_then(Value::as_str)
                .ok_or("packet omits packet_id")
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    let current_reviews = exact_reviews
        .values()
        .filter(|review| review.path.ends_with("v07_exact_reviews_wave9.tsv"))
        .map(|review| review.fields[0].as_str())
        .collect::<BTreeSet<_>>();
    if packet_ids != current_reviews {
        return Err("latest exact-review ledger does not cover the current packet report".into());
    }
    for review in exact_reviews.values() {
        let decision = review.fields.get(1).map(String::as_str).unwrap_or("");
        let blocker = review.fields.get(3).map(String::as_str).unwrap_or("");
        let note = review.fields.get(4).map(String::as_str).unwrap_or("");
        if !matches!(decision, "admitted" | "deferred" | "rejected")
            || note.trim().is_empty()
            || (decision != "admitted" && blocker.trim().is_empty())
        {
            return Err(format!("incomplete exact review in {}", review.path).into());
        }
    }
    for (name, table) in [("variant", variants), ("abbreviation", abbreviations)] {
        let decision = table.index("decision")?;
        let note = table.index("review_note")?;
        if table.rows.is_empty()
            || table
                .rows
                .iter()
                .any(|row| row[decision] != "admitted" || row[note].trim().is_empty())
        {
            return Err(format!("v0.7 {name} ledger is incomplete").into());
        }
    }
    for (name, table, expected) in [
        ("identity correction", identity_corrections, "merged"),
        ("evidence correction", evidence_corrections, "replaced"),
    ] {
        let decision = table.index("decision")?;
        let note = table.index("review_note")?;
        if table.rows.is_empty()
            || table
                .rows
                .iter()
                .any(|row| row[decision] != expected || row[note].trim().is_empty())
        {
            return Err(format!("v0.7 {name} ledger is incomplete").into());
        }
    }
    require_header(verification, &["command", "result", "note"])?;
    let command = verification.index("command")?;
    let result = verification.index("result")?;
    let note = verification.index("note")?;
    let mut seen = BTreeSet::new();
    for row in &verification.rows {
        if !seen.insert(row[command].as_str()) {
            return Err(format!("duplicate verification entry {:?}", row[command]).into());
        }
        if row[result] != "pass" || row[note].trim().is_empty() {
            return Err(format!("verification did not pass: {}", row[command]).into());
        }
    }
    for required in REQUIRED_VERIFICATION {
        if !seen.contains(required) {
            return Err(format!("verification ledger omits required command {required:?}").into());
        }
    }
    Ok(())
}

fn validate_evidence_boundary(
    lexical: &Table,
    exact: &Table,
    evidence: &Table,
    evaluation: &Table,
    abbreviation_evaluation: &Table,
) -> Result<(), Box<dyn Error>> {
    let mut semantic_owners = BTreeMap::new();
    for row in lexical.rows.iter().filter(|row| {
        row.get(15).is_some_and(|value| value == "reviewed")
            && row
                .get(10)
                .is_some_and(|value| value == "english-wiktionary-ocs-kaikki-2026-08-07")
    }) {
        if let Some(previous) = semantic_owners.insert(row[11].as_str(), row[1].as_str())
            && previous != row[1]
        {
            return Err(format!(
                "reviewed semantic candidate {} maps to multiple runtime lexemes",
                row[11]
            )
            .into());
        }
    }
    let lexical_ids = lexical
        .rows
        .iter()
        .map(|row| row[0].as_str())
        .collect::<BTreeSet<_>>();
    let evidence_ids = evidence
        .rows
        .iter()
        .map(|row| row[0].as_str())
        .collect::<BTreeSet<_>>();
    for row in exact.rows.iter().filter(|row| {
        row.get(4).is_some_and(|ids| {
            ids.split(',')
                .any(|id| id.starts_with("v07-") || id.starts_with("review:v07:"))
        })
    }) {
        for id in row[4].split(',') {
            if !lexical_ids.contains(id) && !evidence_ids.contains(id) {
                return Err(format!("v0.7-backed exact row cites unknown evidence {id}").into());
            }
        }
    }
    let held_out = evaluation
        .rows
        .iter()
        .map(|row| (row[6].as_str(), row[7].as_str()))
        .chain(
            abbreviation_evaluation
                .rows
                .iter()
                .map(|row| (row[6].as_str(), row[7].as_str())),
        )
        .collect::<BTreeSet<_>>();
    for row in evidence.rows.iter().filter(|row| {
        row.first().is_some_and(|id| id.starts_with("v07-"))
            && row.get(4).is_some_and(|decision| decision == "reviewed")
    }) {
        if held_out.contains(&(row[2].as_str(), row[3].as_str())) {
            return Err(format!(
                "v0.7 runtime evidence {} overlaps held-out passage {}:{}",
                row[0], row[2], row[3]
            )
            .into());
        }
    }
    Ok(())
}

struct ExactReview {
    path: String,
    fields: Vec<String>,
}

fn load_exact_reviews(root: &Path) -> Result<BTreeMap<String, ExactReview>, Box<dyn Error>> {
    let mut paths = fs::read_dir(root.join("data/synodal"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name == "v07_exact_reviews.tsv"
                        || (name.starts_with("v07_exact_reviews_wave") && name.ends_with(".tsv"))
                })
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|path| exact_wave(path));
    let mut reviews = BTreeMap::new();
    for path in paths {
        let table = read_tsv(&path)?;
        require_header(
            &table,
            &[
                "packet_id",
                "decision",
                "realized_unique_tokens",
                "blocker",
                "review_note",
            ],
        )?;
        for fields in table.rows {
            reviews.insert(
                fields[0].clone(),
                ExactReview {
                    path: path
                        .strip_prefix(root)
                        .unwrap_or(&path)
                        .display()
                        .to_string(),
                    fields,
                },
            );
        }
    }
    Ok(reviews)
}

fn exact_wave(path: &Path) -> usize {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if name == "v07_exact_reviews.tsv" {
        1
    } else {
        name.strip_prefix("v07_exact_reviews_wave")
            .and_then(|value| value.strip_suffix(".tsv"))
            .and_then(|value| value.parse().ok())
            .unwrap_or(0)
    }
}

fn render_marginal_routes(
    out: &mut String,
    baseline: &Value,
    marginal: &Value,
) -> Result<(), Box<dyn Error>> {
    let initial = baseline
        .pointer("/marginal/by_route")
        .and_then(Value::as_object)
        .ok_or("baseline omits marginal routes")?;
    let mut current = BTreeMap::<String, u64>::new();
    for batch in marginal
        .get("batches")
        .and_then(Value::as_array)
        .ok_or("marginal report omits batches")?
    {
        let route = batch
            .get("recovery_route")
            .and_then(Value::as_str)
            .ok_or("marginal batch omits route")?;
        let tokens = batch
            .get("overlap_adjusted_tokens")
            .and_then(Value::as_u64)
            .ok_or("marginal batch omits adjusted tokens")?;
        *current.entry(route.into()).or_default() += tokens;
    }
    out.push_str("| Marginal route | Initial v0.6 | Final v0.7 |\n|---|---:|---:|\n");
    let keys = initial
        .keys()
        .chain(current.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in keys {
        out.push_str(&format!(
            "| `{}` | {} | {} |\n",
            escape(&key),
            initial.get(&key).and_then(Value::as_u64).unwrap_or(0),
            current.get(&key).copied().unwrap_or(0),
        ));
    }
    out.push('\n');
    Ok(())
}

fn render_before_after_objects(
    out: &mut String,
    before: &serde_json::Map<String, Value>,
    after: &serde_json::Map<String, Value>,
) {
    let keys = before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in keys {
        out.push_str(&format!(
            "| `{}` | {} | {} |\n",
            escape(&key),
            before.get(&key).and_then(Value::as_u64).unwrap_or(0),
            after.get(&key).and_then(Value::as_u64).unwrap_or(0),
        ));
    }
}

fn render_table(
    out: &mut String,
    heading: &str,
    table: &Table,
    columns: &[&str],
) -> Result<(), Box<dyn Error>> {
    out.push_str(&format!("\n### {heading}\n\n"));
    out.push('|');
    for column in columns {
        out.push_str(&format!(" {} |", column.replace('_', " ")));
    }
    out.push_str("\n|");
    for _ in columns {
        out.push_str("---|");
    }
    out.push('\n');
    let indexes = columns
        .iter()
        .map(|column| table.index(column))
        .collect::<Result<Vec<_>, _>>()?;
    for row in &table.rows {
        out.push('|');
        for index in &indexes {
            out.push_str(&format!(" `{}` |", escape(&row[*index])));
        }
        out.push('\n');
    }
    Ok(())
}

fn render_metric_map(
    out: &mut String,
    evaluation: &Value,
    key: &str,
    label: &str,
) -> Result<(), Box<dyn Error>> {
    out.push_str(&format!("### Evaluation by {}\n\n", label.to_lowercase()));
    out.push_str(&format!(
        "| {label} | Total | Returned | Top-1 | Top-k | Abstained |\n|---|---:|---:|---:|---:|---:|\n"
    ));
    for (name, metric) in object(evaluation, key)? {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            escape(name),
            pointer_number(metric, "/total")?,
            pointer_number(metric, "/returned")?,
            pointer_number(metric, "/top_1_correct")?,
            pointer_number(metric, "/top_k_correct")?,
            pointer_number(metric, "/abstained")?,
        ));
    }
    out.push('\n');
    Ok(())
}

fn render_disagreements(
    out: &mut String,
    evaluation: &Value,
    key: &str,
    heading: &str,
) -> Result<(), Box<dyn Error>> {
    let rows = evaluation
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("evaluation omits {key}"))?;
    out.push_str(&format!(
        "### {heading}\n\nThere are {} deterministic top-1 disagreements; all expected forms remain in top-k.\n\n| Evaluation | Expected | Returned top-1 | Returned top-k |\n|---|---|---|---|\n",
        rows.len()
    ));
    for row in rows {
        let top_k = row
            .get("returned_top_k")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` |\n",
            escape(
                row.get("evaluation_id")
                    .or_else(|| row.get("id"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(
                row.get("expected")
                    .or_else(|| row.get("expected_form"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(
                row.get("returned_top_1")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(&top_k),
        ));
    }
    out.push('\n');
    Ok(())
}

fn render_numeric_object(out: &mut String, values: &serde_json::Map<String, Value>, total: u64) {
    for (name, value) in values {
        let tokens = value.as_u64().unwrap_or(0);
        out.push_str(&format!(
            "| `{}` | {tokens} | {} |\n",
            escape(name),
            percent(tokens, total)
        ));
    }
}

fn metric_row(out: &mut String, label: &str, before: u64, after: u64, total: u64) {
    out.push_str(&format!(
        "| {label} | {before} ({}) | {after} ({}) | {:+} |\n",
        percent(before, total),
        percent(after, total),
        after as i128 - before as i128
    ));
}

fn delta_row(out: &mut String, label: &str, before: u64, after: u64) {
    out.push_str(&format!(
        "| {label} | {before} | {after} | {:+} |\n",
        after as i128 - before as i128
    ));
}

fn percent(value: u64, total: u64) -> String {
    format!("{:.3}%", value as f64 * 100.0 / total as f64)
}

fn table_count(value: &Value, name: &str) -> Result<u64, Box<dyn Error>> {
    value
        .pointer(&format!("/normalized_tables/{name}"))
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("extraction report omits {name}").into())
}

fn sum_column(table: &Table, column: &str) -> Result<u64, Box<dyn Error>> {
    let index = table.index(column)?;
    table
        .rows
        .iter()
        .map(|row| {
            row[index]
                .parse::<u64>()
                .map_err(|error| format!("invalid numeric value {:?}: {error}", row[index]).into())
        })
        .sum()
}

fn require_header(table: &Table, required: &[&str]) -> Result<(), Box<dyn Error>> {
    for column in required {
        table.index(column)?;
    }
    Ok(())
}

struct Table {
    path: PathBuf,
    header: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn index(&self, name: &str) -> Result<usize, Box<dyn Error>> {
        self.header
            .iter()
            .position(|column| column == name)
            .ok_or_else(|| format!("{} omits column {name:?}", self.path.display()).into())
    }
}

fn read_tsv(path: &Path) -> Result<Table, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let header = lines
        .next()
        .ok_or_else(|| format!("{} is empty", path.display()))?
        .split('\t')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let rows = lines
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(offset, line)| {
            let row = line.split('\t').map(str::to_owned).collect::<Vec<_>>();
            if row.len() != header.len() {
                Err(format!(
                    "{}:{} has {} fields; expected {}",
                    path.display(),
                    offset + 2,
                    row.len(),
                    header.len()
                ))
            } else {
                Ok(row)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Table {
        path: path.to_owned(),
        header,
        rows,
    })
}

fn read_json(path: &Path) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn object<'a>(
    value: &'a Value,
    key: &str,
) -> Result<&'a serde_json::Map<String, Value>, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("JSON omits object {key:?}").into())
}

fn map_object<'a>(
    value: &'a serde_json::Map<String, Value>,
    key: &str,
) -> Result<&'a serde_json::Map<String, Value>, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("JSON omits object {key:?}").into())
}

fn number(value: &serde_json::Map<String, Value>, key: &str) -> Result<u64, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("JSON omits number {key:?}").into())
}

fn root_number(value: &Value, key: &str) -> Result<u64, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("JSON omits number {key:?}").into())
}

fn pointer_number(value: &Value, pointer: &str) -> Result<u64, Box<dyn Error>> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("JSON omits number {pointer:?}").into())
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("JSON omits string {key:?}").into())
}

fn require_string(value: &Value, key: &str, expected: &str) -> Result<(), Box<dyn Error>> {
    let actual = string(value, key)?;
    if actual != expected {
        return Err(format!("field {key:?}: expected {expected:?}, found {actual:?}").into());
    }
    Ok(())
}

fn require_number(value: &Value, pointer: &str, expected: u64) -> Result<(), Box<dyn Error>> {
    let actual = pointer_number(value, pointer)?;
    if actual != expected {
        return Err(format!("field {pointer:?}: expected {expected}, found {actual}").into());
    }
    Ok(())
}

fn escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_threshold_constants_match_the_locked_denominator() {
        assert_eq!(LOCKED_TOKENS * 70 / 100 + 1, TARGET_70);
        assert_eq!(BASELINE_TOP_K + 65_571, TARGET_70);
    }

    #[test]
    fn percentage_is_stable() {
        assert_eq!(percent(919_752, LOCKED_TOKENS), "70.031%");
    }
}
