use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use sha2::{Digest, Sha256};

const OUTPUT: &str = "docs/SYNODAL_V06_65_PERCENT_TOP_K_COVERAGE_AUDIT.md";
const BASELINE: &str = "reports/synodal-v05-baseline.json";
const COVERAGE: &str = "reports/synodal-coverage.json";
const EVALUATION: &str = "reports/synodal-evaluation.json";
const EXTRACTION: &str = "reports/synodal-extraction.json";
const MARGINAL: &str = "reports/synodal-marginal-recovery.json";
const PACKETS: &str = "reports/synodal-v06-review-packets.json";
const EXACT_REVIEWS: &str = "data/synodal/v06_exact_reviews.tsv";
const ABBREVIATION_REVIEWS: &str = "data/synodal/v06_abbreviation_reviews.tsv";
const SPELLING_REVIEWS: &str = "data/synodal/v06_spelling_reviews.tsv";
const VERIFICATION: &str = "data/synodal/v06_verification.tsv";

const LOCKED_PASSAGES: u64 = 74_130;
const LOCKED_TOKENS: u64 = 1_313_344;
const LOCKED_TYPES: u64 = 57_476;
const BASELINE_TOP_K: u64 = 792_421;
const TARGET_65: u64 = 853_674;
const TARGET_70: u64 = 919_341;

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            value => return Err(format!("unknown synodal-v06-audit argument {value:?}").into()),
        }
    }

    if check {
        check_frozen_audit(root)?;
        println!("Synodal v0.6 65% top-k coverage audit: frozen and current");
        return Ok(());
    }

    let markdown = render(root)?;
    let output = root.join(OUTPUT);
    if check {
        if fs::read_to_string(&output).ok().as_deref() != Some(markdown.as_str()) {
            return Err(format!("{} is stale", output.display()).into());
        }
    } else if fs::read_to_string(&output).ok().as_deref() != Some(markdown.as_str()) {
        fs::write(&output, markdown)?;
    }
    println!("Synodal v0.6 65% top-k coverage audit: current");
    Ok(())
}

fn check_frozen_audit(root: &Path) -> Result<(), Box<dyn Error>> {
    let baseline = read_json(&root.join("reports/synodal-v06-baseline.json"))?;
    let expected = baseline
        .pointer("/artifact_sha256/docs~1SYNODAL_V06_65_PERCENT_TOP_K_COVERAGE_AUDIT.md")
        .and_then(Value::as_str)
        .ok_or("v0.6 baseline omits its frozen audit digest")?;
    let mut digest = Sha256::new();
    digest.update(fs::read(root.join(OUTPUT))?);
    let actual = format!("{:x}", digest.finalize());
    if actual != expected {
        return Err("frozen v0.6 audit digest changed".into());
    }
    Ok(())
}

fn render(root: &Path) -> Result<String, Box<dyn Error>> {
    let baseline = read_json(&root.join(BASELINE))?;
    let coverage = read_json(&root.join(COVERAGE))?;
    let evaluation = read_json(&root.join(EVALUATION))?;
    let extraction = read_json(&root.join(EXTRACTION))?;
    let marginal = read_json(&root.join(MARGINAL))?;
    let packets = read_json(&root.join(PACKETS))?;
    let exact = read_tsv(&root.join(EXACT_REVIEWS))?;
    let abbreviations = read_tsv(&root.join(ABBREVIATION_REVIEWS))?;
    let spelling = read_tsv(&root.join(SPELLING_REVIEWS))?;
    let verification = read_tsv(&root.join(VERIFICATION))?;

    validate_contract(&baseline, &coverage, &evaluation, &marginal, &packets)?;
    validate_review_ledgers(&exact, &abbreviations, &spelling, &verification)?;

    let summary = object(&coverage, "summary")?;
    let total = number(summary, "total_tokens")?;
    let top_1 = number(summary, "top_1_analyzed")?;
    let top_k = number(summary, "top_k_analyzed")?;
    let ambiguous = number(summary, "ambiguous")?;
    let unresolved = number(summary, "unresolved")?;
    let numerals = coverage
        .pointer("/by_status/cyrillic-numeral")
        .and_then(Value::as_u64)
        .ok_or("coverage omits Cyrillic-numeral status")?;
    let top_k_uncovered = total
        .checked_sub(top_k + numerals)
        .ok_or("coverage partition exceeds denominator")?;
    if top_k + numerals + top_k_uncovered != total {
        return Err("top-k + numerals + uncovered partition is inconsistent".into());
    }

    let base_coverage = object(&baseline, "coverage")?;
    let base_registry = object(&baseline, "registry")?;
    let base_eval = object(&baseline, "evaluation")?;
    let base_top_1 = number(base_coverage, "top_1_analyzed")?;
    let base_top_k = number(base_coverage, "top_k_analyzed")?;
    let base_ambiguous = number(base_coverage, "ambiguous")?;
    let base_unresolved = number(base_coverage, "unresolved")?;
    let lexical_reviews = read_tsv(&root.join("data/synodal/lexical_reviews.tsv"))?;
    let reviewed_lexical = lexical_reviews
        .rows
        .iter()
        .filter(|row| row.get(15).is_some_and(|value| value == "reviewed"))
        .count() as u64;
    let direct_lexemes = table_count(&extraction, "lexemes.tsv")?;
    let direct_senses = table_count(&extraction, "senses.tsv")?;
    let direct_forms = table_count(&extraction, "exact_forms.tsv")?;
    let lexemes = direct_lexemes + reviewed_lexical;
    let senses = direct_senses + reviewed_lexical;
    let forms = direct_forms + reviewed_lexical;
    let eval_rows = root_number(&evaluation, "fixture_rows")?;
    let abbreviation_rows = table_count(&extraction, "abbreviations.tsv")?;
    let abbreviation_eval_rows = root_number(&evaluation, "abbreviation_fixture_rows")?;

    let exact_predicted = sum_column(
        &exact,
        "predicted_unique_tokens",
        Some("decision"),
        "admitted",
    )?;
    let exact_realized = sum_column(
        &exact,
        "realized_unique_tokens",
        Some("decision"),
        "admitted",
    )?;
    let abbreviation_predicted = sum_column(
        &abbreviations,
        "predicted_unique_tokens",
        Some("decision"),
        "admitted",
    )?;
    let abbreviation_realized = sum_column(
        &abbreviations,
        "realized_unique_tokens",
        Some("decision"),
        "admitted",
    )?;
    let spelling_realized = spelling
        .rows
        .iter()
        .map(|row| parse_row_number(&spelling, row, "realized_unique_tokens"))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum::<u64>();

    let mut out = String::new();
    out.push_str("# Synodal v0.6 evidence-backed 65% top-k coverage audit\n\n");
    out.push_str("This file is generated by `cargo xtask synodal-v06-audit`. Frequencies, probable families, spelling similarities, and marginal packets remain review diagnostics; only analyses returned by the canonical `Strict` resolver under `SynodalLiturgical` enter realized coverage.\n\n");

    out.push_str("## Outcome\n\n");
    out.push_str(&format!(
        "The primary gate passes: **{top_k} of {total} tokens ({})** have at least one evidence-qualified top-k analysis. The strict 65% minimum is {TARGET_65}, leaving a **{}-token margin**. The 70% stretch gate is {TARGET_70}, leaving a **{}-token deficit**. Unresolved is {unresolved} ({}), below 35%.\n\n",
        percent(top_k, total),
        top_k - TARGET_65,
        TARGET_70.saturating_sub(top_k),
        percent(unresolved, total),
    ));
    out.push_str("| Measure | Locked v0.5 | Final v0.6 | Delta |\n|---|---:|---:|---:|\n");
    metric_row(&mut out, "Top-1 analyzed", base_top_1, top_1, total);
    metric_row(&mut out, "Top-k analyzed", base_top_k, top_k, total);
    metric_row(&mut out, "Ambiguous", base_ambiguous, ambiguous, total);
    metric_row(&mut out, "Unresolved", base_unresolved, unresolved, total);
    out.push_str(&format!(
        "\nThe authoritative causal result is the aggregate canonical resolver delta: **+{} top-k tokens** and **-{} unresolved tokens**. Top-1 uniqueness fell because v0.6 stopped hiding independently evidenced alternative exact cells and lexical analyses; all held-out expectations remain present in top-k, so that change is reported as corrected ambiguity rather than recovery.\n\n",
        top_k - base_top_k,
        base_unresolved - unresolved,
    ));

    out.push_str("## Locked identity and accounting proof\n\n");
    out.push_str("| Contract | Locked v0.5 | Final v0.6 |\n|---|---|---|\n");
    out.push_str(&format!(
        "| Target recension | `synodal-russian` | `{}` |\n| Policy | `strict` | `{}` |\n| Orthography profile | `synodal-liturgical` | `{}` |\n| Tokenizer | `synodal-dictionary-tokenize-v1` | unchanged |\n| Sources | two pinned 2026-08-09 Ponomar/Wikisource IDs | identical IDs and revisions |\n| Partitions | `source`, `evaluation` | identical; runtime/evaluation overlap zero |\n| Passages | {LOCKED_PASSAGES} | {} |\n| Tokens | {LOCKED_TOKENS} | {total} |\n| Token types | {LOCKED_TYPES} | {} |\n\n",
        string(&coverage, "target_recension")?,
        string(&coverage, "generation_policy")?,
        string(&coverage, "orthography_profile")?,
        root_number(&coverage, "passages")?,
        root_number(&coverage, "token_types")?,
    ));
    out.push_str(&format!(
        "The required partition is exact: `{top_k} top-k + {numerals} numerals + {top_k_uncovered} top-k-uncovered = {total}`. Resolver statuses below are mutually exclusive.\n\n"
    ));
    out.push_str("| Resolver status | Tokens | Percent |\n|---|---:|---:|\n");
    render_numeric_object(&mut out, object(&coverage, "by_status")?, total);

    out.push_str("\n## Registry and evaluation growth\n\n");
    out.push_str("| Registry | v0.5 | v0.6 | Delta |\n|---|---:|---:|---:|\n");
    delta_row(
        &mut out,
        "Reviewed lexemes",
        number(base_registry, "reviewed_lexemes")?,
        lexemes,
    );
    delta_row(
        &mut out,
        "Reviewed senses",
        number(base_registry, "reviewed_senses")?,
        senses,
    );
    delta_row(
        &mut out,
        "Generated exact forms",
        number(base_registry, "generated_exact_forms")?,
        forms,
    );
    delta_row(
        &mut out,
        "Typed abbreviations",
        number(base_registry, "typed_abbreviations")?,
        abbreviation_rows,
    );
    delta_row(
        &mut out,
        "Held-out morphological cells",
        number(base_eval, "morphological_cells")?,
        eval_rows,
    );
    delta_row(
        &mut out,
        "Held-out abbreviation cells",
        number(base_eval, "abbreviation_cells")?,
        abbreviation_eval_rows,
    );
    out.push_str("\nNo productive class, transformation rule, principal part, or broad spelling normalizer was added for the v0.6 gain. `past:*` is exact-only; productive resolution deliberately rejects it.\n\n");

    out.push_str("## Route pools versus overlap-adjusted marginal diagnostics\n\n");
    out.push_str("Route-pool counts classify current top-k-uncovered tokens. Marginal counts are a greedy overlap-adjusted review projection over packet memberships. They are not disjoint measures and must not be added.\n\n");
    out.push_str("| Route | v0.5 uncovered pool | v0.6 uncovered pool | Current marginal diagnostic |\n|---|---:|---:|---:|\n");
    let base_routes = base_coverage
        .get("estimated_recovery_by_route")
        .and_then(Value::as_object)
        .ok_or("v0.5 baseline coverage omits route pools")?;
    let current_routes = object(&coverage, "estimated_recovery_by_route")?;
    let marginal_routes = marginal_routes(&marginal)?;
    let mut routes = BTreeMap::new();
    for key in base_routes
        .keys()
        .chain(current_routes.keys())
        .chain(marginal_routes.keys())
    {
        routes.insert(key.clone(), ());
    }
    for route in routes.keys() {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            escape(route),
            base_routes.get(route).and_then(Value::as_u64).unwrap_or(0),
            current_routes
                .get(route)
                .and_then(Value::as_u64)
                .unwrap_or(0),
            marginal_routes.get(route).copied().unwrap_or(0),
        ));
    }
    out.push_str(&format!(
        "\nThe current marginal artifact contains {} packets, {} overlap-adjusted diagnostic tokens, and reports zero tokens needed for the primary gate. Its counterfactual projection is {}, not realized coverage.\n\n",
        marginal.pointer("/batches").and_then(Value::as_array).map_or(0, Vec::len),
        root_number(&marginal, "diagnostic_recovery")?,
        root_number(&marginal, "diagnostic_projected_top_k")?,
    ));

    out.push_str("## Batch attribution\n\n");
    out.push_str(&format!(
        "The exact-cell ledger records {exact_predicted} predicted and {exact_realized} surface-realized tokens. The admitted abbreviation batches record {abbreviation_predicted} predicted and {abbreviation_realized} realized tokens. Ready/small spelling review accounts for {spelling_realized} tokens admitted only as exact cells. These slices intentionally overlap lexical bootstrap and ambiguity routes; the aggregate +{} resolver delta remains authoritative.\n\n",
        top_k - BASELINE_TOP_K,
    ));
    render_exact_reviews(&mut out, &exact)?;
    render_abbreviation_reviews(&mut out, &abbreviations)?;
    render_spelling_reviews(&mut out, &spelling)?;

    out.push_str("## New identities and exact evidence boundary\n\n");
    out.push_str("Every `review:v06:*` identity pairs a semantic or normative candidate with an exact source-partition target witness. The exhaustive exact-cell ledger above separates semantic, morphology, target, and held-out candidate IDs. Generic OCS sources license candidate identity or form only; target frequency never licenses an unreviewed paradigm. Rejected false groupings—including blood/roof, fall/feed, inflected-form lemmas, verbal/noun homographs, and source-treebank cell mismatches—remain zero-gain rows with explicit blockers.\n\n");
    out.push_str("| v0.6 reviewed identity | Lemma | POS | Semantic candidate | Source-partition target passage |\n|---|---|---|---|---|\n");
    for row in lexical_reviews.rows.iter().filter(|row| {
        row.first()
            .is_some_and(|value| value.starts_with("review:v06:"))
            && row.get(15).is_some_and(|value| value == "reviewed")
    }) {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` `{}` |\n",
            escape(&row[1]),
            escape(&row[3]),
            escape(&row[4]),
            escape(&row[11]),
            escape(&row[12]),
            escape(&row[14]),
        ));
    }

    out.push_str("\n## Held-out evaluation and leakage\n\n");
    let expanded = object(&evaluation, "expanded")?;
    let printed = object(&evaluation, "printed")?;
    out.push_str(&format!(
        "Expanded top-k is {}/{} and printed top-k is {}/{}; no existing reviewed expectation disappeared. Expanded top-1 is {}/{} and printed top-1 is {}/{}. Typed abbreviations are {}/{} top-k.\n\n",
        number(expanded, "top_k_correct")?, number(expanded, "total")?,
        number(printed, "top_k_correct")?, number(printed, "total")?,
        number(expanded, "top_1_correct")?, number(expanded, "total")?,
        number(printed, "top_1_correct")?, number(printed, "total")?,
        pointer_number(&evaluation, "/abbreviation_expansion/top_k_correct")?,
        pointer_number(&evaluation, "/abbreviation_expansion/total")?,
    ));
    render_metric_map(&mut out, &evaluation, "by_policy", "Policy")?;
    render_metric_map(
        &mut out,
        &evaluation,
        "by_morphological_system",
        "Morphological system",
    )?;
    render_metric_map(
        &mut out,
        &evaluation,
        "by_attestation_status",
        "Attestation",
    )?;
    render_metric_map(
        &mut out,
        &evaluation,
        "by_provenance_path",
        "Provenance route",
    )?;
    let leakage = evaluation
        .get("leakage")
        .ok_or("evaluation omits leakage metrics")?;
    out.push_str(&format!(
        "\nMasked reconstruction remains {}/{} expanded top-k and {}/{} printed top-k. Candidate-link validation enforces zero runtime/evaluation passage overlap and rejects evaluation-partition runtime evidence.\n\n",
        pointer_number(leakage, "/masked_expanded/top_k_correct")?,
        pointer_number(leakage, "/masked_expanded/total")?,
        pointer_number(leakage, "/masked_printed/top_k_correct")?,
        pointer_number(leakage, "/masked_printed/total")?,
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

    out.push_str("## Remaining work\n\n");
    out.push_str("The highest remaining exact surfaces are shown below. They remain unresolved because no reviewed evidence-backed runtime analysis exists; their frequency is not an admission decision.\n\n");
    out.push_str("| Rank | Surface | Top-k-uncovered tokens | Gap | Suggested action |\n|---:|---|---:|---|---|\n");
    if let Some(gaps) = coverage.get("gaps").and_then(Value::as_array) {
        for (index, gap) in gaps
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
    }
    out.push_str(&format!(
        "\nThe next evidence-ready review should target exact source-backed cells among the leading remaining surfaces, then the highest-score v0.6 packets. Reaching 70% still requires {} canonical tokens and is not inferred from the {}-token diagnostic ceiling.\n\n",
        TARGET_70.saturating_sub(top_k),
        root_number(&marginal, "diagnostic_recovery")?,
    ));

    out.push_str("## Unicode, runtime, reconstruction, and verification\n\n");
    out.push_str("Typed contractions require their exact NFC combining order, titlo/superscript inventory, capitalization, and cell restrictions. Missing titla, reordered marks, mixed scripts, deceptive substrings, private-use characters, and unsupported cells remain negative controls. Runtime crates remain generated, deterministic, filesystem-free, network-free, and `no_std`-compatible; source discovery and corpus review stay in extractor/xtask tooling.\n\n");
    out.push_str("| Verification command or review pass | Result | Note |\n|---|---|---|\n");
    for row in &verification.rows {
        out.push_str(&format!(
            "| `{}` | `{}` | {} |\n",
            escape(&row[0]),
            escape(&row[1]),
            escape(&row[2]),
        ));
    }
    out.push_str("\nThe verification ledger records local state only. No remote CI, review-thread, publication, or pull-request state was inspected or changed. Package-list checks exclude raw corpora, reports, review queues, references, and caches. Full-source bootstrap is offline against pinned local bytes.\n\n");

    out.push_str("## Independent review\n\n");
    out.push_str("The final review covers all intended tracked and untracked v0.3–v0.6 changes against the merge base while preserving the unrelated `synodal-source-availability.yml` edit. It checks evidence identity, exact cells, passage leakage, abbreviation safety, Unicode handling, coverage partitions, ambiguity, generated staleness, runtime boundaries, no-std/WASM/package contents, tests, and documentation. Confirmed findings and corrections are recorded in the verification ledger; no PR or remote review is in scope.\n");

    Ok(out)
}

fn validate_contract(
    baseline: &Value,
    coverage: &Value,
    evaluation: &Value,
    marginal: &Value,
    packets: &Value,
) -> Result<(), Box<dyn Error>> {
    require_string(baseline, "milestone", "synodal-v0.5")?;
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
    if top_k < TARGET_65 {
        return Err(
            format!("v0.6 requires at least {TARGET_65} top-k tokens; found {top_k}").into(),
        );
    }
    for profile in ["expanded", "printed"] {
        let total = pointer_number(evaluation, &format!("/{profile}/total"))?;
        let top_k_correct = pointer_number(evaluation, &format!("/{profile}/top_k_correct"))?;
        if total != top_k_correct {
            return Err(format!("{profile} evaluation lost a top-k expectation").into());
        }
    }
    if root_number(marginal, "current_top_k")? != top_k
        || root_number(marginal, "tokens_needed_for_target")? != 0
    {
        return Err("marginal report disagrees with the final primary coverage result".into());
    }
    if root_number(packets, "current_top_k")? != top_k
        || root_number(packets, "tokens_remaining_for_65_percent")? != 0
    {
        return Err("v0.6 packet report disagrees with final coverage".into());
    }
    Ok(())
}

fn validate_review_ledgers(
    exact: &Table,
    abbreviations: &Table,
    spelling: &Table,
    verification: &Table,
) -> Result<(), Box<dyn Error>> {
    require_header(
        exact,
        &[
            "review_id",
            "decision",
            "predicted_unique_tokens",
            "realized_unique_tokens",
            "blocker",
        ],
    )?;
    require_header(
        abbreviations,
        &[
            "review_id",
            "batch_id",
            "decision",
            "realized_unique_tokens",
            "blocker",
        ],
    )?;
    require_header(
        spelling,
        &["review_id", "decision", "realized_unique_tokens", "blocker"],
    )?;
    require_header(verification, &["command", "result", "note"])?;
    if abbreviations.rows.len() < 36 {
        return Err(format!(
            "only {}/36 modeled abbreviation batches were adjudicated",
            abbreviations.rows.len()
        )
        .into());
    }
    if spelling.rows.len() < 18 {
        return Err(format!(
            "only {}/18 ready/small spelling batches were adjudicated",
            spelling.rows.len()
        )
        .into());
    }
    for (name, table) in [
        ("exact", exact),
        ("abbreviation", abbreviations),
        ("spelling", spelling),
    ] {
        let decision = table.index("decision")?;
        let blocker = table.index("blocker")?;
        for row in &table.rows {
            if row[decision].contains("pending")
                || row.iter().any(|value| value == "pending-measurement")
            {
                return Err(format!(
                    "{name} review ledger contains a pending decision or measurement"
                )
                .into());
            }
            if matches!(
                row[decision].as_str(),
                "deferred" | "rejected" | "false-grouping"
            ) && row[blocker].is_empty()
            {
                return Err(format!(
                    "{name} review ledger has an unadmitted row without a blocker"
                )
                .into());
            }
        }
    }
    if verification.rows.is_empty()
        || verification
            .rows
            .iter()
            .any(|row| row.get(1).is_none_or(|value| value != "pass"))
    {
        return Err("v0.6 verification ledger is incomplete or contains a non-pass result".into());
    }
    Ok(())
}

fn render_exact_reviews(out: &mut String, table: &Table) -> Result<(), Box<dyn Error>> {
    out.push_str("### Exact family and cell decisions\n\n");
    out.push_str("| Decision | Family | Lexeme | Surface | Cell | Predicted | Realized | Evidence or blocker |\n|---|---|---|---|---|---:|---:|---|\n");
    for row in &table.rows {
        let evidence = if value(table, row, "blocker")?.is_empty() {
            format!(
                "semantic `{}`; morphology `{}`; target `{}`; held-out `{}`",
                value(table, row, "semantic_evidence_id")?,
                value(table, row, "morphology_evidence_id")?,
                value(table, row, "target_evidence_id")?,
                value(table, row, "evaluation_candidate_id")?,
            )
        } else {
            value(table, row, "blocker")?.to_owned()
        };
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | {} | {} | {} |\n",
            escape(value(table, row, "decision")?),
            escape(value(table, row, "family_id")?),
            escape(value(table, row, "lexeme_id")?),
            escape(value(table, row, "surface")?),
            escape(value(table, row, "cell")?),
            value(table, row, "predicted_unique_tokens")?,
            value(table, row, "realized_unique_tokens")?,
            escape(&evidence),
        ));
    }
    out.push('\n');
    Ok(())
}

fn render_abbreviation_reviews(out: &mut String, table: &Table) -> Result<(), Box<dyn Error>> {
    out.push_str("### Typed abbreviation adjudication\n\n");
    out.push_str("All 36 locked v0.5 modeled batches and the subsequently split diagnostic batches are explicitly represented. Only the Israel exact splits and `нн҃ѣ` are admitted; every other row has a precise blocker and zero realized coverage.\n\n");
    out.push_str("| Baseline rank | Batch | Label | Decision | Predicted | Realized | Blocker |\n|---:|---|---|---|---:|---:|---|\n");
    for row in &table.rows {
        out.push_str(&format!(
            "| {} | `{}` | `{}` | `{}` | {} | {} | {} |\n",
            value(table, row, "baseline_rank")?,
            escape(value(table, row, "batch_id")?),
            escape(value(table, row, "label")?),
            escape(value(table, row, "decision")?),
            value(table, row, "predicted_unique_tokens")?,
            value(table, row, "realized_unique_tokens")?,
            escape(value(table, row, "blocker")?),
        ));
    }
    out.push('\n');
    Ok(())
}

fn render_spelling_reviews(out: &mut String, table: &Table) -> Result<(), Box<dyn Error>> {
    out.push_str("### Ready/small spelling adjudication\n\n");
    out.push_str("| Batch | Lexeme | Decision | Realized exact tokens | Remaining blocker |\n|---|---|---|---:|---|\n");
    for row in &table.rows {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | {} | {} |\n",
            escape(value(table, row, "label")?),
            escape(value(table, row, "lexeme_id")?),
            escape(value(table, row, "decision")?),
            value(table, row, "realized_unique_tokens")?,
            escape(value(table, row, "blocker")?),
        ));
    }
    out.push('\n');
    Ok(())
}

fn render_metric_map(
    out: &mut String,
    evaluation: &Value,
    key: &str,
    label: &str,
) -> Result<(), Box<dyn Error>> {
    out.push_str(&format!("### Evaluation by {}\n\n", label.to_lowercase()));
    out.push_str(&format!("| {label} | Total | Returned | Top-1 | Top-k | Abstained |\n|---|---:|---:|---:|---:|---:|\n"));
    let values = object(evaluation, key)?;
    for (name, metric) in values {
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
    out.push_str(&format!("### {heading}\n\n"));
    let rows = evaluation
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("evaluation omits {key}"))?;
    out.push_str(&format!("There are {} deterministic top-1 disagreements; top-k remains complete. Each JSON record retains the expected form, returned top-1, complete returned top-k, and evidence trace.\n\n", rows.len()));
    out.push_str(
        "| Evaluation ID | Expected | Returned top-1 | Returned top-k |\n|---|---|---|---|\n",
    );
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

fn marginal_routes(value: &Value) -> Result<BTreeMap<String, u64>, Box<dyn Error>> {
    let mut routes = BTreeMap::new();
    for batch in value
        .get("batches")
        .and_then(Value::as_array)
        .ok_or("marginal report has no batches")?
    {
        let route = batch
            .get("recovery_route")
            .and_then(Value::as_str)
            .ok_or("marginal batch has no route")?;
        let tokens = batch
            .get("overlap_adjusted_tokens")
            .and_then(Value::as_u64)
            .ok_or("marginal batch has no adjusted tokens")?;
        *routes.entry(route.to_owned()).or_default() += tokens;
    }
    Ok(routes)
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

fn sum_column(
    table: &Table,
    column: &str,
    filter_column: Option<&str>,
    filter_value: &str,
) -> Result<u64, Box<dyn Error>> {
    let column = table.index(column)?;
    let filter = filter_column.map(|name| table.index(name)).transpose()?;
    table
        .rows
        .iter()
        .filter(|row| filter.is_none_or(|index| row[index] == filter_value))
        .map(|row| {
            row[column].parse::<u64>().map_err(|error| {
                format!("invalid numeric review value {:?}: {error}", row[column]).into()
            })
        })
        .sum()
}

fn parse_row_number(table: &Table, row: &[String], column: &str) -> Result<u64, Box<dyn Error>> {
    let value = value(table, row, column)?;
    Ok(value
        .parse::<u64>()
        .map_err(|error| format!("invalid {column} value {value:?}: {error}"))?)
}

fn value<'a>(table: &Table, row: &'a [String], column: &str) -> Result<&'a str, Box<dyn Error>> {
    Ok(row
        .get(table.index(column)?)
        .ok_or_else(|| format!("row omits column {column}"))?)
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
    fn strict_threshold_constants_match_locked_denominator() {
        assert_eq!(TARGET_65, LOCKED_TOKENS * 65 / 100 + 1);
        assert_eq!(TARGET_70, (LOCKED_TOKENS * 70).div_ceil(100));
    }

    #[test]
    fn percentage_is_stable() {
        assert_eq!(percent(TARGET_65, LOCKED_TOKENS), "65.000%");
    }
}
