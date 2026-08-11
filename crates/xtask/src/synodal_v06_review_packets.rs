use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};
use synodal_church_slavonic::{GenerationPolicy, LexemeId, OrthographyProfile};
use synodal_church_slavonic_dictionary::{
    Analysis, Entry, coverage::Analyzer, coverage::CoverageReport,
    coverage::MarginalRecoveryReport, lookup_by_id, morphology::Inflector,
};

const CURRENT_JSON: &str = "reports/synodal-v06-review-packets.json";
const CURRENT_TSV: &str = "reports/synodal-v06-review-packets.tsv";
const CURRENT_MARKDOWN: &str = "reports/synodal-v06-review-packets.md";

#[derive(Clone, Debug, Deserialize)]
struct FamilySurfaceInput {
    original: String,
    normalized: String,
    frequency: usize,
    document_frequency: usize,
    possible_cells: Vec<String>,
    corpus: String,
    source_id: String,
    edition: String,
    passage: String,
    partition: String,
}

#[derive(Clone, Debug, Deserialize)]
struct FamilyProposalInput {
    candidate_id: String,
    proposed_lemma: String,
    part_of_speech: String,
    surfaces: Vec<FamilySurfaceInput>,
    token_frequency: usize,
    document_frequency: usize,
    documents: Vec<String>,
    contexts: Vec<synodal_church_slavonic_dictionary::coverage::GapContext>,
    corpora: Vec<String>,
    source_ids: Vec<String>,
    editions: Vec<String>,
    passages: Vec<String>,
    partitions: Vec<String>,
    possible_cells: Vec<String>,
    diagnostic_features: Vec<String>,
    compatible_existing_lexemes: Vec<String>,
    dictionary_candidate_ids: Vec<String>,
    supporting_evidence: Vec<String>,
    contradicting_evidence: Vec<String>,
    missing_metadata: Vec<String>,
    confidence_basis_points: u16,
    assumptions: Vec<String>,
    review_status: String,
    review_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SurfaceEvidence {
    original: String,
    normalized: String,
    candidate_frequency: usize,
    document_frequency: usize,
    corpus: String,
    source_id: String,
    edition: String,
    passage: String,
    partition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct PacketSurface {
    member_candidate_ids: Vec<String>,
    original: String,
    normalized: String,
    top_k_uncovered_frequency: usize,
    document_frequency: usize,
    proposed_cells: Vec<String>,
    source_records: Vec<SurfaceEvidence>,
    current_analyses: Vec<Analysis>,
    current_resolver_gap_details: Vec<String>,
    current_resolver_traces: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
struct EvaluationExclusion {
    evaluation_id: String,
    lexeme_id: String,
    cell: String,
    source_id: String,
    passage: String,
    artifact: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReviewPacket {
    rank: usize,
    packet_id: String,
    family_id: String,
    member_candidate_ids: Vec<String>,
    proposed_lemmas: Vec<String>,
    parts_of_speech: Vec<String>,
    surfaces: Vec<PacketSurface>,
    raw_token_gain: usize,
    unique_gap_tokens: usize,
    overlap_adjusted_token_gain: usize,
    cumulative_overlap_adjusted_tokens: usize,
    document_frequency: usize,
    member_reported_token_frequency: usize,
    member_reported_document_frequency: usize,
    representative_target_passages: Vec<String>,
    corpora: Vec<String>,
    source_ids: Vec<String>,
    editions: Vec<String>,
    passages: Vec<String>,
    partitions: Vec<String>,
    diagnostic_features: Vec<String>,
    candidate_lexemes_and_senses: Vec<Entry>,
    candidate_dictionary_ids: Vec<String>,
    proposed_typed_cells: Vec<String>,
    existing_reviewed_runtime_lexemes: Vec<String>,
    evidence_by_role: BTreeMap<String, Vec<String>>,
    missing_evidence: Vec<String>,
    contradictions: Vec<String>,
    false_grouping_risks: Vec<String>,
    assumptions: Vec<String>,
    predicted_top_1_gain: usize,
    predicted_top_k_gain: usize,
    predicted_ambiguity_gain: usize,
    predicted_abstention_reduction: usize,
    evaluation_passages_excluded_from_runtime_evidence: Vec<EvaluationExclusion>,
    evidence_readiness: String,
    review_effort: String,
    confidence_basis_points: u16,
    decision: String,
    reviewer_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReviewPacketReport {
    schema_version: u8,
    target_recension: String,
    generation_policy: GenerationPolicy,
    orthography_profile: OrthographyProfile,
    corpus_tokens: usize,
    current_top_k: usize,
    strictly_more_than_65_percent: usize,
    tokens_remaining_for_65_percent: usize,
    strictly_more_than_70_percent: usize,
    tokens_remaining_for_70_percent: usize,
    packets: Vec<ReviewPacket>,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            value => {
                return Err(
                    format!("unknown synodal-v06-review-packets argument {value:?}").into(),
                );
            }
        }
    }

    let coverage: CoverageReport = read_json(&root.join("reports/synodal-coverage.json"))?;
    let marginal: MarginalRecoveryReport =
        read_json(&root.join("reports/synodal-marginal-recovery.json"))?;
    let proposals: Vec<FamilyProposalInput> =
        read_json(&root.join("reports/synodal-family-review-queue.json"))?;
    if marginal.current_top_k != coverage.summary.top_k_analyzed
        || marginal.total_tokens != coverage.summary.total_tokens
        || marginal.generation_policy != GenerationPolicy::Strict
        || marginal.orthography_profile != OrthographyProfile::SynodalLiturgical
    {
        return Err(
            "coverage and marginal inputs do not describe the same canonical Strict run".into(),
        );
    }

    let analyzer = Analyzer::new(
        Inflector::builder()
            .generation_policy(GenerationPolicy::Strict)
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build(),
    )?;
    let exclusions = load_evaluation_exclusions(root)?;
    let proposal_index: BTreeMap<_, _> = proposals
        .into_iter()
        .map(|proposal| (proposal.candidate_id.clone(), proposal))
        .collect();
    let mut gaps_by_surface = BTreeMap::<String, Vec<_>>::new();
    for gap in &coverage.gaps {
        gaps_by_surface
            .entry(gap.normalized.clone())
            .or_default()
            .push(gap);
    }

    let mut packets = Vec::with_capacity(marginal.batches.len());
    for batch in &marginal.batches {
        let members: Vec<_> = batch
            .member_candidate_ids
            .iter()
            .filter_map(|id| proposal_index.get(id))
            .collect();
        if members.len() != batch.member_candidate_ids.len() {
            return Err(format!("packet {} refers to a missing family member", batch.id).into());
        }
        packets.push(build_packet(
            batch,
            &members,
            &analyzer,
            &gaps_by_surface,
            &exclusions,
        )?);
    }
    validate_packet_order(&packets)?;

    let target_65 = strict_threshold(coverage.summary.total_tokens, 6_500);
    let target_70 = strict_threshold(coverage.summary.total_tokens, 7_000);
    let report = ReviewPacketReport {
        schema_version: 1,
        target_recension: coverage.target_recension,
        generation_policy: coverage.generation_policy,
        orthography_profile: coverage.orthography_profile,
        corpus_tokens: coverage.summary.total_tokens,
        current_top_k: coverage.summary.top_k_analyzed,
        strictly_more_than_65_percent: target_65,
        tokens_remaining_for_65_percent: target_65.saturating_sub(coverage.summary.top_k_analyzed),
        strictly_more_than_70_percent: target_70,
        tokens_remaining_for_70_percent: target_70.saturating_sub(coverage.summary.top_k_analyzed),
        packets,
    };
    let outputs = [
        (
            root.join(CURRENT_JSON),
            format!("{}\n", serde_json::to_string_pretty(&report)?),
        ),
        (root.join(CURRENT_TSV), render_tsv(&report)),
        (root.join(CURRENT_MARKDOWN), render_markdown(&report)),
    ];
    for (path, contents) in outputs {
        if check {
            check_contents(&path, &contents)?;
        } else {
            write_if_changed(&path, &contents)?;
        }
    }
    println!(
        "Synodal v0.6 review packets: {} packets; {} tokens remain for >65%, {} for >70%",
        report.packets.len(),
        report.tokens_remaining_for_65_percent,
        report.tokens_remaining_for_70_percent,
    );
    Ok(())
}

fn build_packet(
    batch: &synodal_church_slavonic_dictionary::coverage::MarginalRecoveryBatch,
    members: &[&FamilyProposalInput],
    analyzer: &Analyzer,
    gaps_by_surface: &BTreeMap<
        String,
        Vec<&synodal_church_slavonic_dictionary::coverage::GapRecord>,
    >,
    exclusions: &[EvaluationExclusion],
) -> Result<ReviewPacket, Box<dyn Error>> {
    let member_ids = batch.member_candidate_ids.clone();
    let lexeme_ids = union(
        members
            .iter()
            .flat_map(|member| member.compatible_existing_lexemes.iter())
            .chain(batch.compatible_lexeme_ids.iter()),
    );
    let candidate_lexemes_and_senses = lexeme_ids
        .iter()
        .filter_map(|id| lookup_by_id(&LexemeId::from(id.as_str())).ok())
        .collect();

    let mut surface_members = BTreeMap::<String, BTreeSet<String>>::new();
    let mut surface_metadata = BTreeMap::<String, Vec<&FamilySurfaceInput>>::new();
    for member in members {
        for surface in &member.surfaces {
            surface_members
                .entry(surface.normalized.clone())
                .or_default()
                .insert(member.candidate_id.clone());
            surface_metadata
                .entry(surface.normalized.clone())
                .or_default()
                .push(surface);
        }
    }
    let mut surfaces = Vec::with_capacity(batch.surfaces.len());
    for surface in &batch.surfaces {
        let metadata = surface_metadata
            .get(&surface.key)
            .cloned()
            .unwrap_or_default();
        let document_frequency = metadata
            .iter()
            .map(|item| item.document_frequency)
            .max()
            .unwrap_or_default();
        let proposed_cells = union(metadata.iter().flat_map(|item| item.possible_cells.iter()));
        let source_records = metadata
            .iter()
            .map(|item| SurfaceEvidence {
                original: item.original.clone(),
                normalized: item.normalized.clone(),
                candidate_frequency: item.frequency,
                document_frequency: item.document_frequency,
                corpus: item.corpus.clone(),
                source_id: item.source_id.clone(),
                edition: item.edition.clone(),
                passage: item.passage.clone(),
                partition: item.partition.clone(),
            })
            .collect();
        let current_analyses = analyzer
            .analyze_profile(&surface.sample, OrthographyProfile::SynodalLiturgical)
            .unwrap_or_default();
        let gaps = gaps_by_surface
            .get(&surface.key)
            .cloned()
            .unwrap_or_default();
        let current_resolver_gap_details = union(gaps.iter().map(|gap| &gap.detail));
        let current_resolver_traces = gaps
            .iter()
            .filter_map(|gap| serde_json::to_value(&gap.resolver_trace).ok())
            .filter_map(|value| serde_json::to_string(&value).ok().map(|key| (key, value)))
            .collect::<BTreeMap<_, _>>()
            .into_values()
            .collect();
        surfaces.push(PacketSurface {
            member_candidate_ids: surface_members
                .get(&surface.key)
                .map_or_else(Vec::new, |ids| ids.iter().cloned().collect()),
            original: surface.sample.clone(),
            normalized: surface.key.clone(),
            top_k_uncovered_frequency: surface.frequency,
            document_frequency,
            proposed_cells,
            source_records,
            current_analyses,
            current_resolver_gap_details,
            current_resolver_traces,
        });
    }
    surfaces.sort_by(|left, right| {
        right
            .top_k_uncovered_frequency
            .cmp(&left.top_k_uncovered_frequency)
            .then_with(|| left.normalized.cmp(&right.normalized))
    });

    let mut evidence = BTreeMap::<String, Vec<String>>::new();
    for item in members
        .iter()
        .flat_map(|member| member.supporting_evidence.iter())
        .chain(batch.evidence_available.iter())
    {
        evidence
            .entry(evidence_role(item).to_owned())
            .or_default()
            .push(item.clone());
    }
    for values in evidence.values_mut() {
        values.sort();
        values.dedup();
    }
    let false_grouping_risks = false_grouping_risks(members, &lexeme_ids);
    let mut decision = consolidated_decision(members);
    if decision == "admitted"
        && surfaces
            .iter()
            .any(|surface| surface.current_analyses.is_empty())
    {
        decision = "partial-admission".into();
    }
    let reviewer_reason = union(members.iter().map(|member| &member.review_reason)).join(" | ");
    let representative_target_passages = union(
        members
            .iter()
            .flat_map(|member| member.contexts.iter().map(|context| &context.document))
            .chain(members.iter().flat_map(|member| member.documents.iter())),
    )
    .into_iter()
    .take(12)
    .collect();
    let evaluation_passages_excluded_from_runtime_evidence = exclusions
        .iter()
        .filter(|row| lexeme_ids.contains(&row.lexeme_id))
        .cloned()
        .collect();
    let family_id = if lexeme_ids.len() == 1 {
        format!("family:{}", lexeme_ids[0])
    } else {
        format!("candidate-family:{}", batch.id)
    };

    Ok(ReviewPacket {
        rank: batch.rank,
        packet_id: batch.id.clone(),
        family_id,
        member_candidate_ids: member_ids,
        proposed_lemmas: union(members.iter().map(|member| &member.proposed_lemma)),
        parts_of_speech: union(members.iter().map(|member| &member.part_of_speech)),
        surfaces,
        raw_token_gain: batch.raw_token_frequency,
        unique_gap_tokens: batch.unique_gap_tokens,
        overlap_adjusted_token_gain: batch.overlap_adjusted_tokens,
        cumulative_overlap_adjusted_tokens: batch.cumulative_overlap_adjusted_tokens,
        document_frequency: batch.document_frequency,
        member_reported_token_frequency: members.iter().map(|member| member.token_frequency).sum(),
        member_reported_document_frequency: members
            .iter()
            .map(|member| member.document_frequency)
            .sum(),
        representative_target_passages,
        corpora: union(members.iter().flat_map(|member| member.corpora.iter())),
        source_ids: union(members.iter().flat_map(|member| member.source_ids.iter())),
        editions: union(members.iter().flat_map(|member| member.editions.iter())),
        passages: union(members.iter().flat_map(|member| member.passages.iter())),
        partitions: union(members.iter().flat_map(|member| member.partitions.iter())),
        diagnostic_features: union(
            members
                .iter()
                .flat_map(|member| member.diagnostic_features.iter()),
        ),
        candidate_lexemes_and_senses,
        candidate_dictionary_ids: union(
            members
                .iter()
                .flat_map(|member| member.dictionary_candidate_ids.iter()),
        ),
        proposed_typed_cells: union(
            members
                .iter()
                .flat_map(|member| member.possible_cells.iter())
                .chain(batch.proposed_cells.iter()),
        ),
        existing_reviewed_runtime_lexemes: lexeme_ids,
        evidence_by_role: evidence,
        missing_evidence: union(
            members
                .iter()
                .flat_map(|member| member.missing_metadata.iter())
                .chain(batch.missing_evidence.iter()),
        ),
        contradictions: union(
            members
                .iter()
                .flat_map(|member| member.contradicting_evidence.iter())
                .chain(batch.contradictions.iter()),
        ),
        false_grouping_risks,
        assumptions: union(
            members
                .iter()
                .flat_map(|member| member.assumptions.iter())
                .chain(batch.assumptions.iter()),
        ),
        predicted_top_1_gain: batch.expected_top_1_gain,
        predicted_top_k_gain: batch.expected_top_k_gain,
        predicted_ambiguity_gain: batch.expected_ambiguity_gain,
        predicted_abstention_reduction: batch.expected_abstention_reduction,
        evaluation_passages_excluded_from_runtime_evidence,
        evidence_readiness: format!("{:?}", batch.evidence_readiness).to_lowercase(),
        review_effort: format!("{:?}", batch.review_effort).to_lowercase(),
        confidence_basis_points: members
            .iter()
            .map(|member| member.confidence_basis_points)
            .min()
            .unwrap_or(batch.confidence_basis_points),
        decision,
        reviewer_reason,
    })
}

fn evidence_role(item: &str) -> &'static str {
    if item.contains("target-recension")
        || item.contains("ponomar-")
        || item.contains("wikisource-")
    {
        "target-surface"
    } else if item.contains("ud-ocs") || item.contains("grammatical cell") {
        "typed-cell-candidate"
    } else if item.contains("wiktionary") || item.contains("dictionary") {
        "lexical-semantic-candidate"
    } else if item.contains("alypy") || item.contains("normative") {
        "normative"
    } else {
        "other-reviewed-or-candidate"
    }
}

fn false_grouping_risks(members: &[&FamilyProposalInput], lexeme_ids: &[String]) -> Vec<String> {
    let mut risks = BTreeSet::new();
    if members.len() > 1 {
        risks.insert(
            "consolidated packet still requires identity agreement for every member".to_owned(),
        );
    }
    if lexeme_ids.len() > 1 {
        risks.insert(
            "multiple compatible runtime lexemes must not be forced into one family".to_owned(),
        );
    }
    if members
        .iter()
        .map(|member| &member.part_of_speech)
        .collect::<BTreeSet<_>>()
        .len()
        > 1
    {
        risks.insert("candidate members cross parts of speech".to_owned());
    }
    if members
        .iter()
        .any(|member| member.part_of_speech == "ambiguous")
    {
        risks.insert("candidate evidence preserves lexical or cell ambiguity".to_owned());
    }
    risks.into_iter().collect()
}

fn consolidated_decision(members: &[&FamilyProposalInput]) -> String {
    let decisions: BTreeSet<_> = members
        .iter()
        .map(|member| member.review_status.as_str())
        .collect();
    if decisions.len() == 1 {
        decisions
            .into_iter()
            .next()
            .unwrap_or("candidate-unreviewed")
            .to_owned()
    } else {
        format!(
            "split:{}",
            decisions.into_iter().collect::<Vec<_>>().join("+")
        )
    }
}

fn load_evaluation_exclusions(root: &Path) -> Result<Vec<EvaluationExclusion>, Box<dyn Error>> {
    let mut rows = Vec::new();
    for (artifact, path, source_column, passage_column) in [
        (
            "evaluation.tsv",
            "data/synodal/evaluation.tsv",
            6_usize,
            7_usize,
        ),
        (
            "abbreviation_evaluation.tsv",
            "data/synodal/abbreviation_evaluation.tsv",
            6_usize,
            7_usize,
        ),
    ] {
        for (line_number, line) in fs::read_to_string(root.join(path))?.lines().enumerate() {
            if line_number == 0 || line.is_empty() {
                continue;
            }
            let fields: Vec<_> = line.split('\t').collect();
            if fields.len() <= passage_column {
                return Err(format!("malformed {path} row {}", line_number + 1).into());
            }
            rows.push(EvaluationExclusion {
                evaluation_id: fields[0].to_owned(),
                lexeme_id: fields[1].to_owned(),
                cell: if artifact == "evaluation.tsv" {
                    fields[2].to_owned()
                } else {
                    fields[3].to_owned()
                },
                source_id: fields[source_column].to_owned(),
                passage: fields[passage_column].to_owned(),
                artifact: artifact.to_owned(),
            });
        }
    }
    rows.sort();
    rows.dedup();
    Ok(rows)
}

fn validate_packet_order(packets: &[ReviewPacket]) -> Result<(), Box<dyn Error>> {
    for (index, packet) in packets.iter().enumerate() {
        if packet.rank != index + 1 {
            return Err(format!("packet ranks are unstable at {}", packet.packet_id).into());
        }
        if packet.overlap_adjusted_token_gain == 0 && packet.predicted_top_k_gain != 0 {
            return Err(
                format!("zero-marginal packet {} predicts a gain", packet.packet_id).into(),
            );
        }
    }
    Ok(())
}

fn render_tsv(report: &ReviewPacketReport) -> String {
    let mut out = String::from(
        "rank\tpacket_id\tfamily_id\tmember_candidate_ids\tlemmas\tparts_of_speech\tsurfaces\traw_tokens\tunique_tokens\toverlap_adjusted_tokens\tdocument_frequency\tlexeme_ids\tproposed_cells\tevidence_roles\tmissing_evidence\tcontradictions\tfalse_grouping_risks\tpredicted_top_1\tpredicted_top_k\tpredicted_ambiguity\tpredicted_abstention_reduction\tevaluation_exclusions\treadiness\teffort\tdecision\treviewer_reason\n",
    );
    for packet in &report.packets {
        let surfaces = packet
            .surfaces
            .iter()
            .map(|surface| {
                format!(
                    "{}:{}",
                    surface.normalized, surface.top_k_uncovered_frequency
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let exclusions = packet
            .evaluation_passages_excluded_from_runtime_evidence
            .iter()
            .map(|row| format!("{}:{}", row.source_id, row.passage))
            .collect::<Vec<_>>()
            .join(",");
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            packet.rank,
            tsv(&packet.packet_id),
            tsv(&packet.family_id),
            tsv(&packet.member_candidate_ids.join(",")),
            tsv(&packet.proposed_lemmas.join(",")),
            tsv(&packet.parts_of_speech.join(",")),
            tsv(&surfaces),
            packet.raw_token_gain,
            packet.unique_gap_tokens,
            packet.overlap_adjusted_token_gain,
            packet.document_frequency,
            tsv(&packet.existing_reviewed_runtime_lexemes.join(",")),
            tsv(&packet.proposed_typed_cells.join(",")),
            tsv(&packet.evidence_by_role.keys().cloned().collect::<Vec<_>>().join(",")),
            tsv(&packet.missing_evidence.join(",")),
            tsv(&packet.contradictions.join(" | ")),
            tsv(&packet.false_grouping_risks.join(" | ")),
            packet.predicted_top_1_gain,
            packet.predicted_top_k_gain,
            packet.predicted_ambiguity_gain,
            packet.predicted_abstention_reduction,
            tsv(&exclusions),
            packet.evidence_readiness,
            packet.review_effort,
            tsv(&packet.decision),
            tsv(&packet.reviewer_reason),
        ));
    }
    out
}

fn render_markdown(report: &ReviewPacketReport) -> String {
    let mut out = format!(
        "# Synodal v0.6 family review packets\n\nThese packets are diagnostic review aids. Their predicted gains do not count as coverage until the canonical `Strict` resolver changes.\n\n- Corpus tokens: {}\n- Current top-k: {}\n- Strictly-more-than-65% threshold: {}\n- Remaining for >65%: {}\n- Strictly-more-than-70% threshold: {}\n- Remaining for >70%: {}\n- Packets: {}\n\n| Rank | Packet | Family | Surfaces | Raw | Marginal | Readiness | Effort | Decision |\n|---:|---|---|---|---:|---:|---|---|---|\n",
        report.corpus_tokens,
        report.current_top_k,
        report.strictly_more_than_65_percent,
        report.tokens_remaining_for_65_percent,
        report.strictly_more_than_70_percent,
        report.tokens_remaining_for_70_percent,
        report.packets.len(),
    );
    for packet in &report.packets {
        let surfaces = packet
            .surfaces
            .iter()
            .take(4)
            .map(|surface| {
                format!(
                    "`{}` ({})",
                    escape_markdown(&surface.original),
                    surface.top_k_uncovered_frequency
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!(
            "| {} | `{}` | `{}` | {} | {} | {} | `{}` | `{}` | `{}` |\n",
            packet.rank,
            escape_markdown(&packet.packet_id),
            escape_markdown(&packet.family_id),
            surfaces,
            packet.raw_token_gain,
            packet.overlap_adjusted_token_gain,
            packet.evidence_readiness,
            packet.review_effort,
            escape_markdown(&packet.decision),
        ));
    }
    out
}

fn strict_threshold(total_tokens: usize, basis_points: usize) -> usize {
    total_tokens
        .saturating_mul(basis_points)
        .checked_div(10_000)
        .unwrap_or(total_tokens)
        .saturating_add(1)
}

fn union<'a>(items: impl Iterator<Item = &'a String>) -> Vec<String> {
    items
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Box<dyn Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn check_contents(path: &Path, expected: &str) -> Result<(), Box<dyn Error>> {
    if fs::read_to_string(path).ok().as_deref() == Some(expected) {
        Ok(())
    } else {
        Err(format!("stale {}; rerun synodal-v06-review-packets", path.display()).into())
    }
}

fn write_if_changed(path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(());
    }
    let temporary = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("new")
    ));
    fs::write(&temporary, contents)?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn escape_markdown(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "\\`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_targets_are_strictly_above_the_percentage() {
        assert_eq!(strict_threshold(1_313_344, 6_500), 853_674);
        assert_eq!(strict_threshold(1_313_344, 7_000), 919_341);
    }

    #[test]
    fn evidence_roles_do_not_treat_dictionary_candidates_as_target_evidence() {
        assert_eq!(
            evidence_role("english-wiktionary candidate"),
            "lexical-semantic-candidate"
        );
        assert_eq!(
            evidence_role("ud-ocs grammatical cell"),
            "typed-cell-candidate"
        );
        assert_eq!(
            evidence_role("ponomar target-recension witness"),
            "target-surface"
        );
    }
}
