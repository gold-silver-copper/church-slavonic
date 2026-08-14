use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

use crate::report_io::read_json;
use serde_json::Value;
use sha2::{Digest, Sha256};
use synodal_church_slavonic_dictionary::{FamilyId, entries, show_family_by_id};

const OUTPUT: &str = "docs/SYNODAL_V05_TOP_K_COVERAGE_AUDIT.md";
const BASELINE: &str = "reports/synodal-v04-baseline.json";
const LOCKED_V05_BASELINE: &str = "reports/synodal-v05-baseline.json";
const LOCKED_AUDIT_SHA256: &str =
    "465c4fe9b48a45d42bb1c4f356c24de2e536155a5a32a0a70453b5151618a92e";
const COVERAGE: &str = "reports/synodal-coverage.json";
const EVALUATION: &str = "reports/synodal-evaluation.json";
const MARGINAL: &str = "reports/synodal-marginal-recovery.json";
const FAMILY_QUEUE: &str = "reports/synodal-family-review-queue.json";
const TARGET_TOP_K: u64 = 788_007;

const DIRECT_V05_LEXEMES: [(&str, &str, &str, &str); 8] = [
    (
        "synodal:pronoun:moi",
        "sense:possessive-first-singular",
        "§§45–47",
        "complete 57-cell closed table",
    ),
    (
        "synodal:pronoun:tvoi",
        "sense:possessive-second-singular",
        "§§45, 46, 48.1 with §47",
        "complete 57-cell closed table",
    ),
    (
        "synodal:pronoun:svoi",
        "sense:possessive-reflexive",
        "§§45, 46, 48.1 with §47",
        "complete 57-cell closed table",
    ),
    (
        "synodal:pronoun:nash",
        "sense:possessive-first-plural",
        "§§45, 46, 48.3",
        "complete 57-cell closed table",
    ),
    (
        "synodal:pronoun:vash",
        "sense:possessive-second-plural",
        "§§45, 46, 48.3",
        "complete 57-cell closed table",
    ),
    (
        "synodal:pronoun:sebe",
        "sense:reflexive-self",
        "§47.1 and note 4",
        "eight exact singular-only rows",
    ),
    (
        "synodal:adjective:gospoden",
        "sense:v05:adjective:gospoden",
        "§§6, 35, 116, 123, 153, 157 and exercises",
        "seven exact typed contractions",
    ),
    (
        "synodal:adjective:bozhii",
        "sense:v05:adjective:bozhii",
        "§§3 and 56",
        "seven table-backed contracted surfaces",
    ),
];

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    for argument in args {
        match argument.as_str() {
            "--check" => {}
            value => return Err(format!("unknown synodal-v05-audit argument {value:?}").into()),
        }
    }

    // v0.5 is an immutable comparison point for later milestones. Reading the
    // live v0.6 registries or marginal target here would silently relabel v0.6
    // as v0.5, so validate the locked machine-readable baseline and audit bytes.
    let baseline = read_json(&root.join(LOCKED_V05_BASELINE))?;
    if string(&baseline, "milestone")? != "synodal-v0.5"
        || string(&baseline, "target_recension")? != "synodal-russian"
        || string(&baseline, "generation_policy")? != "strict"
        || string(&baseline, "orthography_profile")? != "synodal-liturgical"
        || string(&baseline, "tokenizer_contract")? != "synodal-dictionary-tokenize-v1"
        || pointer_number(&baseline, "/corpus/tokens")? != 1_313_344
        || pointer_number(&baseline, "/coverage/top_k_analyzed")? != 792_421
    {
        return Err("locked v0.5 baseline identity drifted".into());
    }
    let output = root.join(OUTPUT);
    let digest = format!("{:x}", Sha256::digest(fs::read(&output)?));
    if digest != LOCKED_AUDIT_SHA256 {
        return Err(format!(
            "{} differs from the locked v0.5 audit (expected {}, found {})",
            output.display(),
            LOCKED_AUDIT_SHA256,
            digest
        )
        .into());
    }
    println!("Synodal v0.5 top-k coverage audit: locked and current");
    Ok(())
}

#[allow(dead_code)]
fn render(root: &Path) -> Result<String, Box<dyn Error>> {
    let baseline = read_json(&root.join(BASELINE))?;
    let coverage = read_json(&root.join(COVERAGE))?;
    let evaluation = read_json(&root.join(EVALUATION))?;
    let marginal = read_json(&root.join(MARGINAL))?;
    let family_queue: Value = read_json(&root.join(FAMILY_QUEUE))?;
    validate_contract(&baseline, &coverage, &marginal)?;

    let summary = object(&coverage, "summary")?;
    let total = number(summary, "total_tokens")?;
    let top_1 = number(summary, "top_1_analyzed")?;
    let top_k = number(summary, "top_k_analyzed")?;
    let ambiguous = number(summary, "ambiguous")?;
    let unresolved = number(summary, "unresolved")?;
    if top_k < TARGET_TOP_K {
        return Err(format!(
            "v0.5 audit requires more than 60% top-k ({TARGET_TOP_K}); found {top_k}"
        )
        .into());
    }

    let baseline_registry = object(&baseline, "registry")?;
    let baseline_coverage = object(&baseline, "coverage")?;
    let baseline_evaluation = object(&baseline, "evaluation")?;
    let baseline_lexemes = number(baseline_registry, "reviewed_lexemes")?;
    let baseline_senses = number(baseline_registry, "reviewed_senses")?;
    let baseline_forms = number(baseline_registry, "generated_exact_forms")?;
    let baseline_cells = number(baseline_evaluation, "morphological_cells")?;
    let baseline_top_1 = number(baseline_coverage, "top_1_analyzed")?;
    let baseline_top_k = number(baseline_coverage, "top_k_analyzed")?;
    let baseline_ambiguous = number(baseline_coverage, "ambiguous")?;
    let baseline_unresolved = number(baseline_coverage, "unresolved")?;

    let entries = entries()?;
    let lexeme_count = entries.len() as u64;
    let sense_count = entries
        .iter()
        .map(|entry| entry.senses.len() as u64)
        .sum::<u64>();
    let exact_form_count = entries
        .iter()
        .map(|entry| entry.metadata.exact_forms.len() as u64)
        .sum::<u64>();
    let mut exact_only = 0_u64;
    let mut fully_classed = 0_u64;
    let mut partial = 0_u64;
    for entry in &entries {
        let family = show_family_by_id(&FamilyId::for_lexeme(entry.lexeme.id()))?;
        if family.fully_classed {
            fully_classed += 1;
        } else if family.exact_only {
            exact_only += 1;
        } else {
            partial += 1;
        }
    }

    let lexical_reviews = read_tsv(&root.join("data/synodal/lexical_reviews.tsv"))?;
    let exact_forms = read_tsv(&root.join("data/synodal/exact_forms.tsv"))?;
    let abbreviations = read_tsv(&root.join("data/synodal/abbreviations.tsv"))?;
    let family_reviews = read_tsv(&root.join("data/synodal/family_reviews.tsv"))?;
    let principal_parts = read_tsv(&root.join("data/synodal/principal_parts.tsv"))?;
    let review_counts = decision_counts(&family_reviews)?;
    let marginal_routes = marginal_route_summaries(&marginal)?;

    let current_cells = number(&evaluation, "fixture_rows")?;
    let threshold_margin = top_k - TARGET_TOP_K;
    let top_200_decided = family_queue
        .as_array()
        .ok_or("family review queue is not an array")?
        .iter()
        .take(200)
        .filter(|row| {
            row.get("review_status").and_then(Value::as_str) != Some("candidate-unreviewed")
        })
        .count();
    if top_200_decided != 200 {
        return Err(format!("v0.5 audit found only {top_200_decided}/200 review decisions").into());
    }

    let mut out = String::new();
    out.push_str("# Synodal v0.5 evidence-backed top-k coverage audit\n\n");
    out.push_str("This file is generated by `cargo xtask synodal-v05-audit`. Proposals and frequency groupings remain diagnostic; only analyses returned by the canonical resolver under the locked policy enter the coverage result.\n\n");

    out.push_str("## Outcome and locked comparison\n\n");
    out.push_str(&format!(
        "The primary milestone passes: **{top_k} of {total} tokens ({})** receive at least one `Strict` analysis under `SynodalLiturgical`. The strictly-more-than-60% minimum is {TARGET_TOP_K}, so the final result is **{threshold_margin} tokens above the gate**. Unresolved coverage is {}.\n\n",
        percent(top_k, total),
        percent(unresolved, total),
    ));
    out.push_str("| Measure | Locked v0.4 | Final v0.5 | Delta |\n|---|---:|---:|---:|\n");
    coverage_row(&mut out, "Top-1 analyzed", baseline_top_1, top_1, total);
    coverage_row(&mut out, "Top-k analyzed", baseline_top_k, top_k, total);
    coverage_row(
        &mut out,
        "Ambiguous tokens",
        baseline_ambiguous,
        ambiguous,
        total,
    );
    coverage_row(
        &mut out,
        "Unresolved tokens",
        baseline_unresolved,
        unresolved,
        total,
    );
    out.push_str("\nThe realized aggregate gain is the resolver delta, not the marginal report's counterfactual potential: top-k gained ");
    out.push_str(&(top_k - baseline_top_k).to_string());
    out.push_str(" tokens and unresolved fell by ");
    out.push_str(&(baseline_unresolved - unresolved).to_string());
    out.push_str(" tokens.\n\n");

    out.push_str("## Corpus, tokenizer, policy, and profile identity\n\n");
    out.push_str("| Contract | Locked value | Final value |\n|---|---|---|\n");
    out.push_str(&format!(
        "| Target recension | `synodal-russian` | `{}` |\n| Policy | `strict` | `{}` |\n| Orthography profile | `synodal-liturgical` | `{}` |\n| Tokenizer | `synodal-dictionary-tokenize-v1` | unchanged (`synodal-dictionary-tokenize-v1`) |\n| Passages | {} | {} |\n| Tokens | {} | {} |\n| Token types | {} | {} |\n| Source IDs | `ponomar-elizabeth-bible-2026-08-09`, `wikisource-church-slavonic-bible-2026-08-09` | identical pinned revisions |\n| Partitions | `evaluation`, `source` | identical |\n\n",
        string(&coverage, "target_recension")?,
        string(&coverage, "generation_policy")?,
        string(&coverage, "orthography_profile")?,
        pointer_number(&baseline, "/corpus/passages")?,
        number(&coverage, "passages")?,
        pointer_number(&baseline, "/corpus/tokens")?,
        total,
        pointer_number(&baseline, "/corpus/token_types")?,
        number(&coverage, "token_types")?,
    ));

    out.push_str("## Registry before and after\n\n");
    out.push_str("| Measure | v0.4 | v0.5 | Delta |\n|---|---:|---:|---:|\n");
    row_delta(&mut out, "Reviewed lexemes", baseline_lexemes, lexeme_count);
    row_delta(&mut out, "Reviewed senses", baseline_senses, sense_count);
    row_delta(
        &mut out,
        "Generated exact forms",
        baseline_forms,
        exact_form_count,
    );
    row_delta(
        &mut out,
        "Passage-held-out morphological cells",
        baseline_cells,
        current_cells,
    );
    out.push_str(&format!(
        "\nThe final public family view contains {exact_only} exact-only lexemes, {fully_classed} fully classed lexemes, and {partial} represented-but-partial lexemes. The principal-part registry remains at {} rows, so v0.5 adds no principal part. No productive rule was added; the milestone uses closed exact tables, exact attested cells, spelling/positional variants, and typed contractions.\n\n",
        principal_parts.rows.len(),
    ));

    out.push_str("## Realized coverage by resolver status\n\n");
    out.push_str("| Resolver status | Tokens | Percent |\n|---|---:|---:|\n");
    if let Some(statuses) = coverage.get("by_status").and_then(Value::as_object) {
        let mut rows: Vec<_> = statuses.iter().collect();
        rows.sort_by_key(|(name, _)| *name);
        for (name, value) in rows {
            let tokens = value.as_u64().unwrap_or(0);
            out.push_str(&format!(
                "| `{}` | {tokens} | {} |\n",
                escape(name),
                percent(tokens, total),
            ));
        }
    }
    out.push_str("\nThese categories are mutually exclusive token outcomes. Abbreviation, spelling, exact-attestation, normative-table, productive-rule, ambiguity, numeral, and unresolved results are therefore not double-counted.\n\n");

    out.push_str("## Diagnostic routes and overlap-adjusted marginal report\n\n");
    out.push_str("| Route | v0.4 diagnostic pool | Current diagnostic remainder | Delta (current - v0.4) |\n|---|---:|---:|---:|\n");
    let baseline_routes = object(baseline_coverage, "estimated_recovery_by_route")?
        .as_object()
        .expect("object() verified the baseline route map");
    let current_routes = object(&coverage, "estimated_recovery_by_route")?
        .as_object()
        .expect("object() verified the current route map");
    let mut route_names = BTreeSet::new();
    route_names.extend(baseline_routes.keys().cloned());
    route_names.extend(current_routes.keys().cloned());
    for route in route_names {
        let before = baseline_routes
            .get(&route)
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let after = current_routes
            .get(&route)
            .and_then(Value::as_u64)
            .unwrap_or(0);
        out.push_str(&format!(
            "| `{}` | {before} | {after} | {:+} |\n",
            escape(&route),
            after as i64 - before as i64,
        ));
    }
    out.push_str(&format!(
        "\nThe current marginal artifact contains {} top-k-uncovered review batches. Greedy overlap adjustment leaves {} diagnostic tokens, producing a counterfactual projected top-k of {}; none are counted as analyzed, and zero additional tokens are required for the milestone. The route reductions above are useful diagnostics but are not claimed as independent causal deltas because family regrouping and resolver precedence changed across the milestone.\n\n",
        marginal
            .get("batches")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        number(&marginal, "diagnostic_recovery")?,
        number(&marginal, "diagnostic_projected_top_k")?,
    ));
    out.push_str("| Rank | Remaining batch | Route | Raw | Overlap-adjusted | Status | Missing evidence |\n|---:|---|---|---:|---:|---|---|\n");
    for batch in array(&marginal, "batches")?.iter().take(20) {
        out.push_str(&format!(
            "| {} | `{}` | `{}` | {} | {} | `{}` | {} |\n",
            batch.get("rank").and_then(Value::as_u64).unwrap_or(0),
            escape(batch.get("label").and_then(Value::as_str).unwrap_or("")),
            escape(
                batch
                    .get("recovery_route")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            batch
                .get("raw_token_frequency")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            batch
                .get("overlap_adjusted_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            escape(
                batch
                    .get("review_status")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(&json_string_list(batch.get("missing_evidence"))),
        ));
    }
    out.push_str(&format!(
        "\nPer-admission family slices below report current observed tokens and are deliberately not mislabeled as causal deltas. Exact causal attribution would require a retained v0.4 per-token resolver trace; the locked baseline retains aggregate and marginal-gap data but not that trace. The authoritative realized value is the aggregate +{} top-k delta above.\n\n",
        top_k.saturating_sub(baseline_top_k)
    ));

    out.push_str("## New lexical identities and senses\n\n");
    out.push_str("The 66 `review:v05:*` rows below each pair independent semantic or normative evidence with a passage-disjoint target witness. Eight additional closed-table identities are listed first; together they equal the 74-lexeme registry delta.\n\n");
    out.push_str("| Lexeme | Sense | POS/table | Normative or semantic citation | Independent target citation |\n|---|---|---|---|---|\n");
    for (lexeme, sense, citation, table) in DIRECT_V05_LEXEMES {
        out.push_str(&format!(
            "| `{lexeme}` | `{sense}` | {} | Alypy {citation} | family review and reviewed-evidence rows |\n",
            escape(table),
        ));
    }
    for row in lexical_reviews.rows.iter().filter(|row| {
        lexical_reviews
            .value(row, "review_id")
            .starts_with("review:v05:")
    }) {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` / candidate `{}` | `{}` `{}` / candidate `{}` |\n",
            escape(lexical_reviews.value(row, "lexeme_id")),
            escape(lexical_reviews.value(row, "sense_id")),
            escape(lexical_reviews.value(row, "part_of_speech")),
            escape(lexical_reviews.value(row, "semantic_source_id")),
            escape(lexical_reviews.value(row, "semantic_candidate_id")),
            escape(lexical_reviews.value(row, "attestation_source_id")),
            escape(lexical_reviews.value(row, "citation")),
            escape(lexical_reviews.value(row, "attestation_candidate_id")),
        ));
    }
    out.push('\n');

    out.push_str("## Exact cell tables and reviewed batches\n\n");
    out.push_str("The table aggregates every current exact row whose evidence is v0.5-specific or belongs to the v0.5 closed-table citations. It is a complete index into `data/synodal/exact_forms.tsv`; rows are never inferred from the aggregation.\n\n");
    out.push_str("| Lexeme | Exact rows | Provenance kinds | Evidence IDs |\n|---|---:|---|---|\n");
    for (lexeme, batch) in exact_batches(&exact_forms)? {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            escape(&lexeme),
            batch.count,
            escape(
                &batch
                    .source_kinds
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            escape(&batch.evidence.into_iter().collect::<Vec<_>>().join(", ")),
        ));
    }
    out.push_str("\nComplete tables are materialized cell by cell. Partial families remain exact-only, so no uncited paradigm cell is licensed.\n\n");

    out.push_str("## Typed abbreviations\n\n");
    out.push_str(&format!(
        "The typed abbreviation registry contains {} rows versus 8 at v0.4, a delta of {}. Every row preserves its exact marks and expansion, carries a typed cell, and is non-reversible unless explicitly allowed. The final precision review removed 15 unsupported `господень` homographic alternatives while preserving all covered surfaces.\n\n",
        abbreviations.rows.len(),
        abbreviations.rows.len().saturating_sub(8),
    ));
    out.push_str("| Lexeme | Typed rows | Rule IDs | Evidence IDs |\n|---|---:|---|---|\n");
    for (lexeme, group) in abbreviation_groups(&abbreviations)? {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            escape(&lexeme),
            group.count,
            escape(&group.rules.into_iter().collect::<Vec<_>>().join(", ")),
            escape(&group.evidence.into_iter().collect::<Vec<_>>().join(", ")),
        ));
    }
    let abbreviation_eval = object(&evaluation, "abbreviation_expansion")?;
    out.push_str(&format!(
        "\nHeld-out typed abbreviation evaluation is {}/{} top-k and {}/{} top-1. Missing titla, incorrect mark order, deceptive surface matches, and mixed-script inputs remain negative controls.\n\n",
        number(abbreviation_eval, "top_k_correct")?,
        number(abbreviation_eval, "total")?,
        number(abbreviation_eval, "top_1_correct")?,
        number(abbreviation_eval, "total")?,
    ));

    out.push_str("## Durable admissions and observed family slices\n\n");
    out.push_str("| Decision ID | Lexeme | Class | Citation | Target evidence | Current family tokens | Current top-k |\n|---|---|---|---|---|---:|---:|\n");
    for row in family_reviews
        .rows
        .iter()
        .filter(|row| family_reviews.value(row, "decision") == "admitted")
    {
        let lexeme = family_reviews.value(row, "linked_lexeme_id");
        let slice = coverage
            .get("by_family")
            .and_then(|families| families.get(format!("family:{lexeme}")));
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` | `{}` `{}` | `{}` | {} | {} |\n",
            escape(family_reviews.value(row, "candidate_id")),
            escape(lexeme),
            escape(family_reviews.value(row, "admitted_class")),
            escape(family_reviews.value(row, "normative_source")),
            escape(family_reviews.value(row, "normative_citation")),
            escape(family_reviews.value(row, "target_evidence")),
            slice
                .and_then(|value| value.get("total_tokens"))
                .and_then(Value::as_u64)
                .unwrap_or(0),
            slice
                .and_then(|value| value.get("top_k_analyzed"))
                .and_then(Value::as_u64)
                .unwrap_or(0),
        ));
    }
    out.push_str(&format!(
        "\nThe durable review ledger contains {} admissions, {} deferrals, and {} rejections. The current top-200 queue is {top_200_decided}/200 explicitly decided. In the top-k-uncovered marginal queue, the spelling route contains {}; the abbreviation route contains {}. These are batchable diagnostics rather than the complete heuristic route pools above. Reviewed deferrals preserve their exact blockers, candidate-unreviewed batches remain explicitly labeled, and neither population contributes realized coverage.\n\n",
        review_counts.get("admitted").copied().unwrap_or(0),
        review_counts.get("deferred").copied().unwrap_or(0),
        review_counts.get("rejected").copied().unwrap_or(0),
        describe_marginal_route(marginal_routes.get("spelling-variant")),
        describe_marginal_route(marginal_routes.get("abbreviation-registry")),
    ));

    out.push_str(
        "<details>\n<summary>Complete deferred and rejected decision history</summary>\n\n",
    );
    out.push_str("| Candidate ID | Decision | Confidence | Blocker or rejection reason |\n|---|---|---:|---|\n");
    for row in family_reviews.rows.iter().filter(|row| {
        matches!(
            family_reviews.value(row, "decision"),
            "deferred" | "rejected"
        )
    }) {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} |\n",
            escape(family_reviews.value(row, "candidate_id")),
            escape(family_reviews.value(row, "decision")),
            escape(family_reviews.value(row, "confidence_bp")),
            escape(family_reviews.value(row, "review_note")),
        ));
    }
    out.push_str("\n</details>\n\n");

    out.push_str("## Remaining high-frequency gaps\n\n");
    out.push_str("The v0.4 leaders were `ꙗко` (13,077), `сь` (7,567), `весь` (7,028), `сотвор-` (4,946), `иже` (4,875), the possessive families, `сынъ`, `ꙗкож-`, `гл҃-`, `день`, and `господень`. v0.5 recovered the closed possessives and material portions of the named exact and abbreviation families; the current surface leaders are:\n\n");
    out.push_str("| Rank | Surface key | Top-k-uncovered tokens |\n|---:|---|---:|\n");
    let mut surfaces: Vec<_> = object(&coverage, "top_k_uncovered_frequency_by_surface")?
        .as_object()
        .expect("object() verified the surface map")
        .iter()
        .filter_map(|(key, value)| value.as_u64().map(|frequency| (key, frequency)))
        .collect();
    surfaces.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
    for (index, (surface, frequency)) in surfaces.into_iter().take(20).enumerate() {
        out.push_str(&format!(
            "| {} | `{}` | {frequency} |\n",
            index + 1,
            escape(surface),
        ));
    }
    out.push_str("\n| Rank | Probable family | Top-k-uncovered tokens | Route | Assumption/blocker |\n|---:|---|---:|---|---|\n");
    let mut probable: Vec<_> = object(&coverage, "unresolved_by_probable_family")?
        .as_object()
        .expect("object() verified the probable-family map")
        .values()
        .filter(|family| {
            family
                .get("top_k_uncovered_token_frequency")
                .and_then(Value::as_u64)
                .unwrap_or(0)
                > 0
        })
        .collect();
    probable.sort_by(|left, right| {
        right
            .get("top_k_uncovered_token_frequency")
            .and_then(Value::as_u64)
            .cmp(
                &left
                    .get("top_k_uncovered_token_frequency")
                    .and_then(Value::as_u64),
            )
            .then_with(|| {
                left.get("probable_family_id")
                    .and_then(Value::as_str)
                    .cmp(&right.get("probable_family_id").and_then(Value::as_str))
            })
    });
    for (index, family) in probable.into_iter().take(20).enumerate() {
        out.push_str(&format!(
            "| {} | `{}` | {} | `{}` | {} |\n",
            index + 1,
            escape(
                family
                    .get("probable_family_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            family
                .get("top_k_uncovered_token_frequency")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            escape(
                family
                    .get("recovery_route")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
            escape(
                family
                    .get("assumption")
                    .and_then(Value::as_str)
                    .unwrap_or("")
            ),
        ));
    }
    out.push('\n');

    out.push_str("## Evaluation, ambiguity, top-1 changes, and leakage\n\n");
    let expanded = object(&evaluation, "expanded")?;
    let printed = object(&evaluation, "printed")?;
    out.push_str(&format!(
        "Expanded lookup is {}/{} top-k and {}/{} top-1; printed lookup is {}/{} top-k and {}/{} top-1. Genuine variants remain in top-k, while top-1 is deterministic and is not treated as the truthfulness gate.\n\n",
        number(expanded, "top_k_correct")?,
        number(expanded, "total")?,
        number(expanded, "top_1_correct")?,
        number(expanded, "total")?,
        number(printed, "top_k_correct")?,
        number(printed, "total")?,
        number(printed, "top_1_correct")?,
        number(printed, "total")?,
    ));
    metric_table(&mut out, "Policy", evaluation.get("by_policy"));
    metric_table(
        &mut out,
        "Morphological system",
        evaluation.get("by_morphological_system"),
    );
    metric_table(
        &mut out,
        "Attestation status",
        evaluation.get("by_attestation_status"),
    );
    metric_table(&mut out, "Regularity", evaluation.get("by_regularity"));
    metric_table(
        &mut out,
        "Provenance path",
        evaluation.get("by_provenance_path"),
    );
    disagreement_table(
        &mut out,
        "Expanded top-1 disagreements",
        evaluation.get("expanded_disagreements"),
    );
    disagreement_table(
        &mut out,
        "Printed top-1 disagreements",
        evaluation.get("printed_disagreements"),
    );
    let leakage = object(&evaluation, "leakage")?;
    let masked_expanded = object(leakage, "masked_expanded")?;
    let masked_printed = object(leakage, "masked_printed")?;
    out.push_str(&format!(
        "Leakage-masked evaluation is {}/{} expanded top-k and {}/{} printed top-k. Fixture abstentions are zero; abstention behavior is exercised by hostile and unsupported-input tests rather than by adding deliberately invalid gold rows. Those tests cover missing metadata, unsupported cells, malformed combining sequences, missing titla, private-use characters, mixed scripts, `со`/`соти`, `ли`/`лити`, and `юже`/`югъ`.\n\n",
        number(masked_expanded, "top_k_correct")?,
        number(masked_expanded, "total")?,
        number(masked_printed, "top_k_correct")?,
        number(masked_printed, "total")?,
    ));

    out.push_str("## Reproducibility, CI, and verification\n\n");
    out.push_str("The final implementation gate is:\n\n```text\n");
    for command in verification_commands() {
        out.push_str(command);
        out.push('\n');
    }
    out.push_str("```\n\nFor this v0.5 completion pass, every command listed above passed locally on 2026-08-11. The full offline bootstrap verified 321 locked artifacts and reproduced the reviewed registries and reports byte for byte. Package and publish dry-runs passed with only Cargo's expected dirty-worktree warnings. The fixture workflow runs bounded reconstruction and stale-output checks, including marginal recovery and this audit. The manual full-source workflow verifies the pinned source cache and regenerates the complete reports. Remote CI state is not asserted by this local audit.\n\n");

    out.push_str("## Remaining risks and next work\n\n");
    out.push_str(&format!(
        "The threshold margin is {} tokens, so any evidence correction must rerun full coverage. The dominant next evidence-ready work is not a guessed productive class: it is explicit identity and typed-cell review for `нн҃ѣ`, the Israel/Jerusalem contraction families, `заⷱ҇`, and high-frequency exact verb/noun cells. `ꙗкѡ` is top-k-covered but remains semantically ambiguous until both adverb and conjunction identities are independently licensed. Reviewed spelling and abbreviation deferrals retain explicit blockers; lower-ranked candidate-unreviewed batches remain zero-coverage diagnostics rather than implied analyses.\n",
        top_k - TARGET_TOP_K
    ));

    Ok(out)
}

fn validate_contract(
    baseline: &Value,
    coverage: &Value,
    marginal: &Value,
) -> Result<(), Box<dyn Error>> {
    require_string(baseline, "milestone", "synodal-v0.4")?;
    require_string(baseline, "target_recension", "synodal-russian")?;
    require_string(baseline, "generation_policy", "strict")?;
    require_string(baseline, "orthography_profile", "synodal-liturgical")?;
    require_string(
        baseline,
        "tokenizer_contract",
        "synodal-dictionary-tokenize-v1",
    )?;
    require_string(coverage, "target_recension", "synodal-russian")?;
    require_string(coverage, "generation_policy", "Strict")?;
    require_string(coverage, "orthography_profile", "SynodalLiturgical")?;
    if number(coverage, "schema_version")? != 4 {
        return Err("v0.5 audit requires coverage schema version 4".into());
    }
    require_string(marginal, "target_recension", "synodal-russian")?;
    require_string(marginal, "generation_policy", "Strict")?;
    require_string(marginal, "orthography_profile", "SynodalLiturgical")?;
    for (label, actual, expected) in [
        (
            "coverage tokens",
            pointer_number(coverage, "/summary/total_tokens")?,
            1_313_344,
        ),
        ("coverage passages", number(coverage, "passages")?, 74_130),
        (
            "coverage token types",
            number(coverage, "token_types")?,
            57_476,
        ),
        (
            "marginal tokens",
            number(marginal, "total_tokens")?,
            1_313_344,
        ),
        (
            "marginal target",
            number(marginal, "target_top_k")?,
            TARGET_TOP_K,
        ),
    ] {
        if actual != expected {
            return Err(format!("{label} drifted: expected {expected}, found {actual}").into());
        }
    }
    if number(marginal, "current_top_k")? != pointer_number(coverage, "/summary/top_k_analyzed")? {
        return Err("marginal and coverage top-k counts differ".into());
    }
    Ok(())
}

#[derive(Default)]
struct BatchSummary {
    count: usize,
    source_kinds: BTreeSet<String>,
    evidence: BTreeSet<String>,
}

#[derive(Default)]
struct MarginalRouteSummary {
    batches: usize,
    raw_tokens: u64,
    statuses: BTreeMap<String, usize>,
}

fn marginal_route_summaries(
    marginal: &Value,
) -> Result<BTreeMap<String, MarginalRouteSummary>, Box<dyn Error>> {
    let mut routes: BTreeMap<String, MarginalRouteSummary> = BTreeMap::new();
    for batch in array(marginal, "batches")? {
        let route = string(batch, "recovery_route")?;
        let status = string(batch, "review_status")?;
        let summary = routes.entry(route.to_owned()).or_default();
        summary.batches += 1;
        summary.raw_tokens += number(batch, "raw_token_frequency")?;
        *summary.statuses.entry(status.to_owned()).or_default() += 1;
    }
    Ok(routes)
}

fn describe_marginal_route(summary: Option<&MarginalRouteSummary>) -> String {
    let Some(summary) = summary else {
        return "0 batches / 0 raw tokens".to_owned();
    };
    let statuses = summary
        .statuses
        .iter()
        .map(|(status, count)| format!("{status}: {count}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "{} batches / {} raw tokens ({statuses})",
        summary.batches, summary.raw_tokens
    )
}

fn exact_batches(table: &TsvTable) -> Result<BTreeMap<String, BatchSummary>, Box<dyn Error>> {
    table.require(&["lexeme_id", "evidence_id", "source_kind"])?;
    let mut batches: BTreeMap<String, BatchSummary> = BTreeMap::new();
    for row in &table.rows {
        let evidence = table.value(row, "evidence_id");
        let v05 = evidence.contains("v05")
            || ["alypy-45", "alypy-46", "alypy-47", "alypy-48", "alypy-56"]
                .iter()
                .any(|prefix| evidence.contains(prefix));
        if !v05 {
            continue;
        }
        let batch = batches
            .entry(table.value(row, "lexeme_id").to_owned())
            .or_default();
        batch.count += 1;
        batch
            .source_kinds
            .insert(table.value(row, "source_kind").to_owned());
        batch.evidence.extend(
            evidence
                .split(',')
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
        );
    }
    Ok(batches)
}

#[derive(Default)]
struct AbbreviationSummary {
    count: usize,
    rules: BTreeSet<String>,
    evidence: BTreeSet<String>,
}

fn abbreviation_groups(
    table: &TsvTable,
) -> Result<BTreeMap<String, AbbreviationSummary>, Box<dyn Error>> {
    table.require(&["lexeme_id", "rule_id", "evidence_id"])?;
    let mut groups: BTreeMap<String, AbbreviationSummary> = BTreeMap::new();
    for row in &table.rows {
        let group = groups
            .entry(table.value(row, "lexeme_id").to_owned())
            .or_default();
        group.count += 1;
        group.rules.insert(table.value(row, "rule_id").to_owned());
        group.evidence.extend(
            table
                .value(row, "evidence_id")
                .split(',')
                .filter(|value| !value.is_empty())
                .map(str::to_owned),
        );
    }
    Ok(groups)
}

fn decision_counts(table: &TsvTable) -> Result<BTreeMap<String, usize>, Box<dyn Error>> {
    table.require(&["decision"])?;
    let mut counts = BTreeMap::new();
    for row in &table.rows {
        *counts
            .entry(table.value(row, "decision").to_owned())
            .or_default() += 1;
    }
    Ok(counts)
}

struct TsvTable {
    header: BTreeMap<String, usize>,
    rows: Vec<Vec<String>>,
}

impl TsvTable {
    fn value<'a>(&self, row: &'a [String], name: &str) -> &'a str {
        self.header
            .get(name)
            .and_then(|index| row.get(*index))
            .map_or("", String::as_str)
    }

    fn require(&self, names: &[&str]) -> Result<(), Box<dyn Error>> {
        for name in names {
            if !self.header.contains_key(*name) {
                return Err(format!("TSV omits column {name:?}").into());
            }
        }
        Ok(())
    }
}

fn read_tsv(path: &Path) -> Result<TsvTable, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let header: BTreeMap<_, _> = lines
        .next()
        .ok_or_else(|| format!("{} is empty", path.display()))?
        .split('\t')
        .enumerate()
        .map(|(index, name)| (name.to_owned(), index))
        .collect();
    let width = header.len();
    let mut rows = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let row: Vec<_> = line.split('\t').map(str::to_owned).collect();
        if row.len() != width {
            return Err(format!(
                "{}:{}: expected {width} fields, found {}",
                path.display(),
                offset + 2,
                row.len()
            )
            .into());
        }
        rows.push(row);
    }
    Ok(TsvTable { header, rows })
}

fn metric_table(out: &mut String, label: &str, value: Option<&Value>) {
    out.push_str(&format!(
        "| {label} | Returned | Top-1 | Top-k | Abstained | Total |\n|---|---:|---:|---:|---:|---:|\n"
    ));
    if let Some(rows) = value.and_then(Value::as_object) {
        let mut rows: Vec<_> = rows.iter().collect();
        rows.sort_by_key(|(name, _)| *name);
        for (name, metric) in rows {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} |\n",
                escape(name),
                metric.get("returned").and_then(Value::as_u64).unwrap_or(0),
                metric
                    .get("top_1_correct")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                metric
                    .get("top_k_correct")
                    .and_then(Value::as_u64)
                    .unwrap_or(0),
                metric.get("abstained").and_then(Value::as_u64).unwrap_or(0),
                metric.get("total").and_then(Value::as_u64).unwrap_or(0),
            ));
        }
    }
    out.push('\n');
}

fn disagreement_table(out: &mut String, title: &str, value: Option<&Value>) {
    out.push_str(&format!("### {title}\n\n"));
    out.push_str("| Evaluation ID | Cell | Expected | Deterministic top-1 |\n|---|---|---|---|\n");
    if let Some(rows) = value.and_then(Value::as_array) {
        for row in rows {
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` |\n",
                escape(row.get("id").and_then(Value::as_str).unwrap_or("")),
                escape(row.get("cell").and_then(Value::as_str).unwrap_or("")),
                escape(row.get("expected").and_then(Value::as_str).unwrap_or("")),
                escape(
                    row.get("returned_top_1")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                ),
            ));
        }
    }
    out.push('\n');
}

fn verification_commands() -> &'static [&'static str] {
    &[
        "cargo fmt --all --check",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "cargo test --workspace --all-targets --all-features",
        "cargo test --workspace --doc",
        "cargo xtask synodal-fixture-bootstrap",
        "cargo xtask synodal-check",
        "cargo xtask synodal-coverage --fixture --offline --check",
        "cargo xtask synodal-coverage --offline --check",
        "cargo xtask synodal-lexical-review-queue --check",
        "cargo xtask synodal-evaluation-queue --check",
        "cargo xtask synodal-family-review-queue --check",
        "cargo xtask synodal-marginal-recovery --check",
        "cargo xtask synodal-v04-audit --check",
        "cargo xtask synodal-v05-audit --check",
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
    ]
}

fn object<'a>(value: &'a Value, key: &str) -> Result<&'a Value, Box<dyn Error>> {
    let object = value
        .get(key)
        .ok_or_else(|| format!("report omits object field {key:?}"))?;
    if !object.is_object() {
        return Err(format!("report field {key:?} is not an object").into());
    }
    Ok(object)
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("report omits array field {key:?}").into())
}

fn number(value: &Value, key: &str) -> Result<u64, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("report omits numeric field {key:?}").into())
}

fn pointer_number(value: &Value, pointer: &str) -> Result<u64, Box<dyn Error>> {
    value
        .pointer(pointer)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("report omits numeric field {pointer:?}").into())
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("report omits string field {key:?}").into())
}

fn require_string(value: &Value, key: &str, expected: &str) -> Result<(), Box<dyn Error>> {
    let actual = string(value, key)?;
    if actual != expected {
        return Err(format!(
            "report field {key:?} differs: expected {expected:?}, found {actual:?}"
        )
        .into());
    }
    Ok(())
}

fn row_delta(out: &mut String, label: &str, before: u64, after: u64) {
    out.push_str(&format!(
        "| {label} | {before} | {after} | {:+} |\n",
        after as i64 - before as i64,
    ));
}

fn coverage_row(out: &mut String, label: &str, before: u64, after: u64, total: u64) {
    out.push_str(&format!(
        "| {label} | {before} ({}) | {after} ({}) | {:+} |\n",
        percent(before, total),
        percent(after, total),
        after as i64 - before as i64,
    ));
}

fn percent(part: u64, total: u64) -> String {
    format!("{:.3}%", (part as f64 * 100.0) / total as f64)
}

fn json_string_list(value: Option<&Value>) -> String {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

fn escape(value: &str) -> String {
    value.replace('|', "\\|").replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_is_strictly_more_than_sixty_percent() {
        assert_eq!((1_313_344_u64 * 60) / 100 + 1, TARGET_TOP_K);
    }

    #[test]
    fn percent_is_deterministic() {
        assert_eq!(percent(3, 4), "75.000%");
    }
}
