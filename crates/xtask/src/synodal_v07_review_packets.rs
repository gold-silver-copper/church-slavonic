use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use synodal_church_slavonic_core::normalize_lookup_accentless;
use synodal_church_slavonic_dictionary::coverage::{CoverageReport, tokenize};
use unicode_normalization::UnicodeNormalization;

const COVERAGE: &str = "reports/synodal-coverage.json";
const SYNTACTICUS: &str = "data/intermediate/synodal/syntacticus-20230428.jsonl";
const KAIKKI: &str = "data/intermediate/synodal/english-wiktionary-ocs-kaikki-2026-08-07.jsonl";
const DYACHENKO: &str = "data/intermediate/synodal/dyachenko-1900-scan.jsonl";
const SEMANTIC_REVIEWS: &str = "data/synodal/v07_semantic_reviews.tsv";
const IDENTITY_REVIEWS: &str = "data/synodal/v07_identity_reviews.tsv";
const TARGET_SOURCES: [&str; 2] = [
    "ponomar-elizabeth-bible-2026-08-09",
    "wikisource-church-slavonic-bible-2026-08-09",
];
const OUTPUT_JSON: &str = "reports/synodal-v07-review-packets.json";
const OUTPUT_TSV: &str = "reports/synodal-v07-review-packets.tsv";
const OUTPUT_MARKDOWN: &str = "reports/synodal-v07-review-packets.md";
const BASELINE_TOP_K: usize = 853_770;

#[derive(Clone, Debug, Deserialize)]
struct SourceCandidate {
    candidate_id: String,
    source_id: String,
    passage: String,
    raw_spelling: String,
    normalized_spelling: String,
    part_of_speech: String,
    grammatical_cell: String,
    partition: String,
    target_recension: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct WiktionaryEntry {
    word: String,
    pos: String,
    #[serde(default)]
    forms: Vec<WiktionaryForm>,
    #[serde(default)]
    senses: Vec<WiktionarySense>,
    #[serde(default)]
    head_templates: Vec<WiktionaryTemplate>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct WiktionaryForm {
    form: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct WiktionarySense {
    #[serde(default)]
    glosses: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct WiktionaryTemplate {
    #[serde(default)]
    args: BTreeMap<String, String>,
    #[serde(default)]
    expansion: String,
}

#[derive(Clone, Debug)]
struct SemanticIdentity {
    source_id: String,
    candidate_id: String,
    passage: String,
    lemma: String,
    part_of_speech: String,
    gloss: String,
    aspectual_tense: Option<String>,
}

#[derive(Clone, Debug)]
struct DictionaryFormCandidate {
    source_id: String,
    candidate_id: String,
    passage: String,
    form: String,
    tags: BTreeSet<String>,
    identity: SemanticIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
struct TargetWitness {
    source_id: String,
    candidate_id: String,
    passage: String,
    partition: String,
    printed: String,
}

#[derive(Clone, Debug)]
struct TargetPair {
    source: TargetWitness,
    evaluation: TargetWitness,
}

#[derive(Clone, Debug, Default)]
struct RuntimeMetadata {
    identities: BTreeMap<(String, String), BTreeSet<String>>,
    lemma_by_lexeme: BTreeMap<String, String>,
    part_by_lexeme: BTreeMap<String, String>,
    animacy_by_lexeme: BTreeMap<String, BTreeSet<String>>,
    aspectual_tense_by_lexeme: BTreeMap<String, BTreeSet<String>>,
    numeral_kind_by_lexeme: BTreeMap<String, BTreeSet<String>>,
    semantic_candidate_by_lexeme: BTreeMap<String, String>,
    exact_keys: BTreeSet<(String, String, String)>,
}

#[derive(Clone, Debug)]
enum IdentityPreference {
    ExistingLexeme(String),
    SemanticCandidate(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct EvidenceRole {
    role: String,
    source_id: String,
    candidate_id: String,
    passage: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReviewPacket {
    rank: usize,
    packet_id: String,
    surface: String,
    normalized_surface: String,
    top_k_uncovered_frequency: usize,
    document_frequency: usize,
    lexeme_id: String,
    lemma: String,
    part_of_speech: String,
    cell: String,
    evidence_lane: String,
    identity_status: String,
    semantic_gloss: String,
    source_morphology: String,
    evidence_roles: Vec<EvidenceRole>,
    source_passage: String,
    evaluation_passage: String,
    contexts: Vec<String>,
    risk_flags: Vec<String>,
    prior_decisions: Vec<String>,
    predicted_unique_tokens: usize,
    decision: String,
    reviewer_note: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ReviewPacketReport {
    schema_version: u8,
    milestone: String,
    target_recension: String,
    generation_policy: String,
    orthography_profile: String,
    corpus_tokens: usize,
    baseline_top_k: usize,
    current_top_k: usize,
    strictly_more_than_70_percent: usize,
    tokens_needed_for_70_percent: usize,
    packet_rows: usize,
    unique_candidate_surfaces: usize,
    overlap_adjusted_candidate_tokens: usize,
    identity_conflicts: Vec<IdentityConflict>,
    packets: Vec<ReviewPacket>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct IdentityConflict {
    surface: String,
    top_k_uncovered_frequency: usize,
    syntacticus_lemma: String,
    syntacticus_part_of_speech: String,
    source_candidate_ids: Vec<String>,
    source_morphology_cells: Vec<String>,
    runtime_lexeme_ids: Vec<String>,
    semantic_candidate_ids: Vec<String>,
    blocker: String,
}

#[derive(Default)]
struct IdentityConflictAccumulator {
    surface: String,
    top_k_uncovered_frequency: usize,
    syntacticus_lemma: String,
    syntacticus_part_of_speech: String,
    source_candidate_ids: BTreeSet<String>,
    source_morphology_cells: BTreeSet<String>,
    runtime_lexeme_ids: BTreeSet<String>,
    semantic_candidate_ids: BTreeSet<String>,
    blocker: String,
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
                    format!("unknown synodal-v07-review-packets argument {value:?}").into(),
                );
            }
        }
    }

    let report = build_report(root)?;
    let outputs = [
        (
            root.join(OUTPUT_JSON),
            format!("{}\n", serde_json::to_string_pretty(&report)?),
        ),
        (root.join(OUTPUT_TSV), render_tsv(&report)),
        (root.join(OUTPUT_MARKDOWN), render_markdown(&report)),
    ];
    for (path, contents) in outputs {
        if check {
            check_contents(&path, &contents)?;
        } else {
            write_if_changed(&path, &contents)?;
        }
    }
    println!(
        "Synodal v0.7 review packets: {} exact-cell rows, {} unique surfaces, {} overlap-adjusted candidate tokens",
        report.packet_rows,
        report.unique_candidate_surfaces,
        report.overlap_adjusted_candidate_tokens,
    );
    Ok(())
}

fn build_report(root: &Path) -> Result<ReviewPacketReport, Box<dyn Error>> {
    let coverage: CoverageReport = read_json(&root.join(COVERAGE))?;
    if coverage.target_recension != "synodal-russian"
        || format!("{:?}", coverage.generation_policy) != "Strict"
        || format!("{:?}", coverage.orthography_profile) != "SynodalLiturgical"
    {
        return Err("v0.7 packets require the canonical Strict SynodalLiturgical run".into());
    }
    let gaps: BTreeMap<_, _> = coverage
        .gaps
        .iter()
        .filter(|gap| gap.top_k_uncovered_frequency > 0)
        .map(|gap| (surface_key(&gap.normalized), gap))
        .collect();
    let runtime = load_runtime_metadata(root)?;
    let dictionary = load_dictionary(root)?;
    let identity_preferences = load_identity_preferences(root, &runtime, &dictionary)?;
    let target_pairs = load_target_pairs(root, gaps.keys().cloned().collect())?;
    let syntacticus = read_json_lines::<SourceCandidate>(&root.join(SYNTACTICUS))?;

    let mut packets_by_key = BTreeMap::<(String, String, String), ReviewPacket>::new();
    let mut identity_conflicts =
        BTreeMap::<(String, String, String), IdentityConflictAccumulator>::new();
    for source in syntacticus.iter().filter(|candidate| {
        candidate.source_id == "syntacticus-20230428"
            && candidate.partition == "source"
            && candidate.grammatical_cell != "_"
            && candidate.grammatical_cell != "untyped"
    }) {
        let target_key = surface_key(&source.raw_spelling);
        if is_abbreviation_surface(&source.raw_spelling) {
            continue;
        }
        let (Some(gap), Some(target)) = (gaps.get(&target_key), target_pairs.get(&target_key))
        else {
            continue;
        };
        let allowed_parts = runtime_parts_for_syntacticus(&source.part_of_speech);
        if allowed_parts.is_empty() {
            continue;
        }
        let lemma_keys = lemma_keys(&source.normalized_spelling);
        let preference = identity_preferences.get(&(
            source.normalized_spelling.clone(),
            source.part_of_speech.clone(),
        ));
        let mut existing = matching_runtime_identities(&runtime, &lemma_keys, &allowed_parts);
        let mut semantics = matching_semantics(&dictionary, &lemma_keys, &allowed_parts);
        if let Some(preference) = preference {
            existing.clear();
            semantics.clear();
            match preference {
                IdentityPreference::ExistingLexeme(lexeme_id) => {
                    existing.insert((lexeme_id.clone(), runtime.part_by_lexeme[lexeme_id].clone()));
                }
                IdentityPreference::SemanticCandidate(candidate_id) => {
                    let semantic = dictionary.by_id[candidate_id].clone();
                    semantics.insert(candidate_id.clone(), semantic);
                }
            }
        }
        let (lexeme_id, lemma, part_of_speech, gloss, identity_status, semantic_candidate) =
            if existing.len() == 1 {
                let (lexeme_id, part_of_speech) = existing.iter().next().expect("one identity");
                let semantic_candidate =
                    runtime.semantic_candidate_by_lexeme.get(lexeme_id).cloned();
                (
                    lexeme_id.clone(),
                    source.normalized_spelling.clone(),
                    part_of_speech.clone(),
                    String::new(),
                    "existing-reviewed-identity".to_owned(),
                    semantic_candidate,
                )
            } else if existing.is_empty() && semantics.len() == 1 {
                let semantic = semantics.values().next().expect("one semantic identity");
                (
                    stable_new_lexeme_id(semantic),
                    semantic.lemma.clone(),
                    semantic.part_of_speech.clone(),
                    semantic.gloss.clone(),
                    "new-source-semantic-identity".to_owned(),
                    Some(semantic.candidate_id.clone()),
                )
            } else {
                let key = (
                    target_key.clone(),
                    source.normalized_spelling.clone(),
                    source.part_of_speech.clone(),
                );
                let conflict = identity_conflicts.entry(key).or_default();
                conflict.surface = target.source.printed.clone();
                conflict.top_k_uncovered_frequency = gap.top_k_uncovered_frequency;
                conflict.syntacticus_lemma = source.normalized_spelling.clone();
                conflict.syntacticus_part_of_speech = source.part_of_speech.clone();
                conflict
                    .source_candidate_ids
                    .insert(source.candidate_id.clone());
                conflict
                    .source_morphology_cells
                    .insert(source.grammatical_cell.clone());
                conflict
                    .runtime_lexeme_ids
                    .extend(existing.iter().map(|(id, _)| id.clone()));
                conflict
                    .semantic_candidate_ids
                    .extend(semantics.keys().cloned());
                conflict.blocker = if existing.len() > 1 {
                    "multiple-runtime-identities-require-reviewed-preference"
                } else if semantics.len() > 1 {
                    "multiple-semantic-identities-require-reviewed-preference"
                } else {
                    "semantic-identity-evidence-required"
                }
                .into();
                continue;
            };
        let semantic = semantic_candidate
            .as_ref()
            .and_then(|candidate_id| dictionary.by_id.get(candidate_id));
        let Some(cell) = map_syntacticus_cell(
            &part_of_speech,
            &source.grammatical_cell,
            &lexeme_id,
            &runtime,
            semantic,
        ) else {
            continue;
        };
        let exact_key = (lexeme_id.clone(), cell.clone(), target_key.clone());
        if runtime.exact_keys.contains(&exact_key) {
            continue;
        }

        let packet_id = stable_packet_id(
            &source.candidate_id,
            &lexeme_id,
            &cell,
            &target.source.printed,
        );
        let mut risk_flags = Vec::new();
        if strict_surface_key(&source.raw_spelling) != strict_surface_key(&target.source.printed) {
            risk_flags.push("reviewed-historical-orthography-equivalence-required".into());
        }
        if lemma_requires_weak_yer_correspondence(&source.normalized_spelling) {
            risk_flags.push("reviewed-weak-yer-identity-correspondence-required".into());
        }
        if lemma_requires_historical_letter_correspondence(&source.normalized_spelling) {
            risk_flags.push("reviewed-historical-lemma-letter-correspondence-required".into());
        }
        if target.evaluation.partition != "evaluation" {
            risk_flags.push("new-held-out-passage-reservation-required".into());
        }
        if identity_status == "new-source-semantic-identity"
            && part_of_speech == "noun"
            && cell.ends_with(":inanimate")
            && semantic.is_some_and(|identity| gloss_is_animate(&identity.gloss))
        {
            return Err(
                format!("animate semantic identity received inanimate cell: {packet_id}").into(),
            );
        }
        if gap.candidate_lexeme_ids.len() > 1 {
            risk_flags.push("current-runtime-identity-ambiguity".into());
        }
        let semantic_id = semantic_candidate.unwrap_or_default();
        let mut evidence_roles = vec![
            EvidenceRole {
                role: "source-typed-morphology".into(),
                source_id: source.source_id.clone(),
                candidate_id: source.candidate_id.clone(),
                passage: source.passage.clone(),
            },
            EvidenceRole {
                role: "target-source-orthography".into(),
                source_id: target.source.source_id.clone(),
                candidate_id: target.source.candidate_id.clone(),
                passage: target.source.passage.clone(),
            },
            EvidenceRole {
                role: "held-out-target-evaluation".into(),
                source_id: target.evaluation.source_id.clone(),
                candidate_id: target.evaluation.candidate_id.clone(),
                passage: target.evaluation.passage.clone(),
            },
        ];
        if identity_status == "new-source-semantic-identity" {
            let semantic = semantic.ok_or_else(|| {
                format!("semantic candidate {semantic_id} is absent from the governed index")
            })?;
            evidence_roles.push(EvidenceRole {
                role: "semantic-identity".into(),
                source_id: semantic.source_id.clone(),
                candidate_id: semantic_id,
                passage: semantic.passage.clone(),
            });
        } else {
            evidence_roles.push(EvidenceRole {
                role: "existing-reviewed-lexical-and-semantic-identity".into(),
                source_id: "synodal-v06-reviewed-registry".into(),
                candidate_id: lexeme_id.clone(),
                passage: lemma.clone(),
            });
        }
        evidence_roles.sort_by(|left, right| left.role.cmp(&right.role));
        let contexts = gap
            .contexts
            .iter()
            .take(3)
            .map(|context| sanitize(&context.excerpt))
            .collect();
        let packet = ReviewPacket {
            rank: 0,
            packet_id,
            surface: target.source.printed.clone(),
            normalized_surface: gap.normalized.clone(),
            top_k_uncovered_frequency: gap.top_k_uncovered_frequency,
            document_frequency: gap.top_k_uncovered_documents.len(),
            lexeme_id,
            lemma,
            part_of_speech,
            cell,
            evidence_lane: "syntacticus-source-typed-exact".into(),
            identity_status,
            semantic_gloss: gloss,
            source_morphology: source.grammatical_cell.clone(),
            evidence_roles,
            source_passage: format!(
                "{}:{}",
                target.source.source_id, target.source.passage
            ),
            evaluation_passage: format!(
                "{}:{}",
                target.evaluation.source_id, target.evaluation.passage
            ),
            contexts,
            risk_flags,
            prior_decisions: Vec::new(),
            predicted_unique_tokens: 0,
            decision: "candidate-unreviewed".into(),
            reviewer_note: "Requires explicit review of lexical identity, source cell, historical spelling correspondence, and target context before admission.".into(),
        };
        let key = (target_key, packet.lexeme_id.clone(), packet.cell.clone());
        packets_by_key.entry(key).or_insert(packet);
    }

    for (target_key, candidates) in &dictionary.by_form_surface {
        let (Some(gap), Some(target)) = (gaps.get(target_key), target_pairs.get(target_key)) else {
            continue;
        };
        for candidate in candidates {
            if is_abbreviation_surface(&candidate.form) {
                continue;
            }
            let lexical_form_only = candidate.tags.contains("lexical-form");
            let part_of_speech = candidate.identity.part_of_speech.clone();
            let existing = matching_runtime_identities(
                &runtime,
                &lemma_keys(&candidate.identity.lemma),
                &[part_of_speech.as_str()],
            );
            let (lexeme_id, identity_status) = if existing.len() == 1 {
                (
                    existing.iter().next().expect("one identity").0.clone(),
                    "existing-reviewed-identity".to_owned(),
                )
            } else if existing.is_empty() {
                (
                    stable_new_lexeme_id(&candidate.identity),
                    "new-source-semantic-identity".to_owned(),
                )
            } else {
                continue;
            };
            let Some(cell) = map_dictionary_form_cell(candidate, &lexeme_id, &runtime) else {
                continue;
            };
            let exact_key = (lexeme_id.clone(), cell.clone(), target_key.clone());
            if runtime.exact_keys.contains(&exact_key) {
                continue;
            }
            let packet_id = stable_packet_id(
                &candidate.candidate_id,
                &lexeme_id,
                &cell,
                &target.source.printed,
            );
            let mut risk_flags = if lexical_form_only {
                vec!["exact-lexical-identity-without-inflectional-cell".into()]
            } else {
                vec!["single-ocs-dictionary-record-fills-semantic-and-morphology-roles".into()]
            };
            if strict_surface_key(&candidate.form) != strict_surface_key(&target.source.printed) {
                risk_flags.push("reviewed-historical-orthography-equivalence-required".into());
            }
            if lemma_requires_weak_yer_correspondence(&candidate.identity.lemma) {
                risk_flags.push("reviewed-weak-yer-identity-correspondence-required".into());
            }
            if lemma_requires_historical_letter_correspondence(&candidate.identity.lemma) {
                risk_flags.push("reviewed-historical-lemma-letter-correspondence-required".into());
            }
            if target.evaluation.partition != "evaluation" {
                risk_flags.push("new-held-out-passage-reservation-required".into());
            }
            if gap.candidate_lexeme_ids.len() > 1 {
                risk_flags.push("current-runtime-identity-ambiguity".into());
            }
            let mut evidence_roles = vec![
                EvidenceRole {
                    role: "semantic-identity".into(),
                    source_id: candidate.source_id.clone(),
                    candidate_id: candidate.candidate_id.clone(),
                    passage: candidate.passage.clone(),
                },
                EvidenceRole {
                    role: if lexical_form_only {
                        "source-lexical-identity"
                    } else {
                        "source-typed-morphology"
                    }
                    .into(),
                    source_id: candidate.source_id.clone(),
                    candidate_id: candidate.candidate_id.clone(),
                    passage: candidate.passage.clone(),
                },
                EvidenceRole {
                    role: "target-source-orthography".into(),
                    source_id: target.source.source_id.clone(),
                    candidate_id: target.source.candidate_id.clone(),
                    passage: target.source.passage.clone(),
                },
                EvidenceRole {
                    role: "held-out-target-evaluation".into(),
                    source_id: target.evaluation.source_id.clone(),
                    candidate_id: target.evaluation.candidate_id.clone(),
                    passage: target.evaluation.passage.clone(),
                },
            ];
            evidence_roles.sort_by(|left, right| left.role.cmp(&right.role));
            let contexts = gap
                .contexts
                .iter()
                .take(3)
                .map(|context| sanitize(&context.excerpt))
                .collect();
            let packet = ReviewPacket {
                rank: 0,
                packet_id,
                surface: target.source.printed.clone(),
                normalized_surface: gap.normalized.clone(),
                top_k_uncovered_frequency: gap.top_k_uncovered_frequency,
                document_frequency: gap.top_k_uncovered_documents.len(),
                lexeme_id,
                lemma: candidate.identity.lemma.clone(),
                part_of_speech,
                cell,
                evidence_lane: if lexical_form_only {
                    "kaikki-source-lexical-form"
                } else {
                    "kaikki-source-exact-form"
                }
                .into(),
                identity_status,
                semantic_gloss: candidate.identity.gloss.clone(),
                source_morphology: candidate.tags.iter().cloned().collect::<Vec<_>>().join("|"),
                evidence_roles,
                source_passage: format!(
                    "{}:{}",
                    target.source.source_id, target.source.passage
                ),
                evaluation_passage: format!(
                    "{}:{}",
                    target.evaluation.source_id, target.evaluation.passage
                ),
                contexts,
                risk_flags,
                prior_decisions: Vec::new(),
                predicted_unique_tokens: 0,
                decision: "candidate-unreviewed".into(),
                reviewer_note: "Requires explicit review that the source dictionary record supports both the exact cell and semantic identity, plus target-recension spelling continuity; no paradigm is inferred.".into(),
            };
            let key = (
                target_key.clone(),
                packet.lexeme_id.clone(),
                packet.cell.clone(),
            );
            packets_by_key.entry(key).or_insert(packet);
        }
    }

    let mut packets: Vec<_> = packets_by_key.into_values().collect();
    packets.sort_by(|left, right| {
        Reverse(left.top_k_uncovered_frequency)
            .cmp(&Reverse(right.top_k_uncovered_frequency))
            .then_with(|| left.normalized_surface.cmp(&right.normalized_surface))
            .then_with(|| left.lexeme_id.cmp(&right.lexeme_id))
            .then_with(|| left.cell.cmp(&right.cell))
    });
    let mut credited_surfaces = BTreeSet::new();
    for (index, packet) in packets.iter_mut().enumerate() {
        packet.rank = index + 1;
        let key = surface_key(&packet.normalized_surface);
        if credited_surfaces.insert(key) {
            packet.predicted_unique_tokens = packet.top_k_uncovered_frequency;
        }
    }
    let overlap_adjusted_candidate_tokens = packets
        .iter()
        .map(|packet| packet.predicted_unique_tokens)
        .sum();
    let mut identity_conflicts: Vec<_> = identity_conflicts
        .into_values()
        .map(|conflict| IdentityConflict {
            surface: conflict.surface,
            top_k_uncovered_frequency: conflict.top_k_uncovered_frequency,
            syntacticus_lemma: conflict.syntacticus_lemma,
            syntacticus_part_of_speech: conflict.syntacticus_part_of_speech,
            source_candidate_ids: conflict.source_candidate_ids.into_iter().collect(),
            source_morphology_cells: conflict.source_morphology_cells.into_iter().collect(),
            runtime_lexeme_ids: conflict.runtime_lexeme_ids.into_iter().collect(),
            semantic_candidate_ids: conflict.semantic_candidate_ids.into_iter().collect(),
            blocker: conflict.blocker,
        })
        .collect();
    identity_conflicts.sort_by_key(|conflict| {
        (
            Reverse(conflict.top_k_uncovered_frequency),
            conflict.surface.clone(),
            conflict.syntacticus_lemma.clone(),
        )
    });
    let target = strict_threshold(coverage.summary.total_tokens, 7_000);
    Ok(ReviewPacketReport {
        schema_version: 1,
        milestone: "synodal-v0.7".into(),
        target_recension: coverage.target_recension,
        generation_policy: format!("{:?}", coverage.generation_policy),
        orthography_profile: format!("{:?}", coverage.orthography_profile),
        corpus_tokens: coverage.summary.total_tokens,
        baseline_top_k: BASELINE_TOP_K,
        current_top_k: coverage.summary.top_k_analyzed,
        strictly_more_than_70_percent: target,
        tokens_needed_for_70_percent: target.saturating_sub(coverage.summary.top_k_analyzed),
        packet_rows: packets.len(),
        unique_candidate_surfaces: credited_surfaces.len(),
        overlap_adjusted_candidate_tokens,
        identity_conflicts,
        packets,
    })
}

fn load_runtime_metadata(root: &Path) -> Result<RuntimeMetadata, Box<dyn Error>> {
    let mut metadata = RuntimeMetadata::default();
    for row in read_tsv(&root.join("data/synodal/lexemes.tsv"))? {
        let id = field(&row, 0, "lexeme id")?;
        let lemma = field(&row, 1, "lemma")?;
        let part = field(&row, 2, "part of speech")?;
        metadata.lemma_by_lexeme.insert(id.into(), lemma.into());
        metadata.part_by_lexeme.insert(id.into(), part.into());
        for key in lemma_keys(lemma) {
            metadata
                .identities
                .entry((key, part.into()))
                .or_default()
                .insert(id.into());
        }
        match row.get(6).map(String::as_str) {
            Some("perfective") => {
                metadata
                    .aspectual_tense_by_lexeme
                    .entry(id.into())
                    .or_default()
                    .insert("future".into());
            }
            Some("imperfective") => {
                metadata
                    .aspectual_tense_by_lexeme
                    .entry(id.into())
                    .or_default()
                    .insert("present".into());
            }
            _ => {}
        }
    }
    for row in read_tsv(&root.join("data/synodal/lexical_reviews.tsv"))? {
        if row.get(15).map(String::as_str) != Some("reviewed") {
            continue;
        }
        let id = field(&row, 1, "lexeme id")?;
        let lemma = field(&row, 3, "lemma")?;
        let part = field(&row, 4, "part of speech")?;
        metadata
            .lemma_by_lexeme
            .entry(id.into())
            .or_insert_with(|| lemma.into());
        metadata
            .part_by_lexeme
            .entry(id.into())
            .or_insert_with(|| part.into());
        for key in lemma_keys(lemma) {
            metadata
                .identities
                .entry((key, part.into()))
                .or_default()
                .insert(id.into());
        }
        metadata
            .semantic_candidate_by_lexeme
            .insert(id.into(), field(&row, 11, "semantic candidate")?.into());
    }
    for row in read_tsv(&root.join("data/synodal/exact_forms.tsv"))? {
        let id = field(&row, 0, "exact lexeme id")?;
        let cell = field(&row, 1, "exact cell")?;
        let expanded = field(&row, 2, "exact expanded form")?;
        let key = (id.into(), cell.into(), surface_key(expanded));
        metadata.exact_keys.insert(key);
        if let Some(animacy) = cell
            .strip_prefix("noun:")
            .and_then(|value| value.rsplit(':').next())
        {
            if matches!(animacy, "animate" | "inanimate") {
                metadata
                    .animacy_by_lexeme
                    .entry(id.into())
                    .or_default()
                    .insert(animacy.into());
            }
        }
        if cell.starts_with("present:") {
            metadata
                .aspectual_tense_by_lexeme
                .entry(id.into())
                .or_default()
                .insert("present".into());
        } else if cell.starts_with("future:") {
            metadata
                .aspectual_tense_by_lexeme
                .entry(id.into())
                .or_default()
                .insert("future".into());
        }
        if let Some(kind) = cell
            .strip_prefix("numeral:")
            .and_then(|value| value.split(':').next())
            && matches!(kind, "cardinal" | "ordinal" | "collective")
        {
            metadata
                .numeral_kind_by_lexeme
                .entry(id.into())
                .or_default()
                .insert(kind.into());
        }
    }
    Ok(metadata)
}

const IDENTITY_REVIEW_HEADER: &str = "review_id\tsyntacticus_lemma\tsyntacticus_part_of_speech\tselected_lexeme_id\tselected_semantic_candidate_id\tdecision\treviewer\treviewed_at\treview_note";

fn load_identity_preferences(
    root: &Path,
    runtime: &RuntimeMetadata,
    dictionary: &DictionaryIndex,
) -> Result<BTreeMap<(String, String), IdentityPreference>, Box<dyn Error>> {
    let text = fs::read_to_string(root.join(IDENTITY_REVIEWS))?;
    let mut lines = text.lines();
    if lines.next() != Some(IDENTITY_REVIEW_HEADER) {
        return Err("invalid v0.7 identity-review header".into());
    }
    let mut review_ids = BTreeSet::new();
    let mut preferences = BTreeMap::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 9 {
            return Err(format!("invalid v0.7 identity-review row {}", offset + 2).into());
        }
        if !review_ids.insert(fields[0]) {
            return Err(format!("duplicate v0.7 identity review {:?}", fields[0]).into());
        }
        if fields[5] != "reviewed"
            || fields[6].is_empty()
            || fields[7].is_empty()
            || fields[8].is_empty()
        {
            return Err(format!("incomplete identity decision at row {}", offset + 2).into());
        }
        let preference = match (fields[3].is_empty(), fields[4].is_empty()) {
            (false, true) => {
                let part = runtime.part_by_lexeme.get(fields[3]).ok_or_else(|| {
                    format!(
                        "unknown selected runtime lexeme {:?} at row {}",
                        fields[3],
                        offset + 2
                    )
                })?;
                if !runtime_parts_for_syntacticus(fields[2]).contains(&part.as_str()) {
                    return Err(format!(
                        "selected runtime part-of-speech mismatch at row {}",
                        offset + 2
                    )
                    .into());
                }
                IdentityPreference::ExistingLexeme(fields[3].into())
            }
            (true, false) => {
                let identity = dictionary.by_id.get(fields[4]).ok_or_else(|| {
                    format!(
                        "unknown selected semantic candidate {:?} at row {}",
                        fields[4],
                        offset + 2
                    )
                })?;
                if !runtime_parts_for_syntacticus(fields[2])
                    .contains(&identity.part_of_speech.as_str())
                {
                    return Err(format!(
                        "selected semantic part-of-speech mismatch at row {}",
                        offset + 2
                    )
                    .into());
                }
                IdentityPreference::SemanticCandidate(fields[4].into())
            }
            _ => {
                return Err(format!(
                    "identity row {} must select exactly one identity",
                    offset + 2
                )
                .into());
            }
        };
        let key = (fields[1].to_owned(), fields[2].to_owned());
        if preferences.insert(key.clone(), preference).is_some() {
            return Err(format!("duplicate v0.7 identity preference {key:?}").into());
        }
    }
    Ok(preferences)
}

#[derive(Default)]
struct DictionaryIndex {
    by_lemma: BTreeMap<(String, String), BTreeMap<String, SemanticIdentity>>,
    by_id: BTreeMap<String, SemanticIdentity>,
    by_form_surface: BTreeMap<String, Vec<DictionaryFormCandidate>>,
}

fn load_dictionary(root: &Path) -> Result<DictionaryIndex, Box<dyn Error>> {
    let mut index = DictionaryIndex::default();
    for candidate in read_json_lines::<SourceCandidate>(&root.join(KAIKKI))? {
        if candidate.partition != "source" {
            continue;
        }
        let entry: WiktionaryEntry = match serde_json::from_str(&candidate.raw_spelling) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let Some(part_of_speech) = map_dictionary_pos(&entry.pos) else {
            continue;
        };
        let glosses: BTreeSet<_> = entry
            .senses
            .iter()
            .flat_map(|sense| &sense.glosses)
            .map(|gloss| sanitize(gloss))
            .filter(|gloss| !gloss.is_empty() && !unsafe_gloss(gloss))
            .collect();
        if glosses.is_empty() {
            continue;
        }
        let aspectual_tense = dictionary_aspectual_tense(&entry);
        let identity = SemanticIdentity {
            source_id: candidate.source_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            passage: candidate.passage.clone(),
            lemma: entry.word.clone(),
            part_of_speech: part_of_speech.into(),
            gloss: glosses.into_iter().collect::<Vec<_>>().join("; "),
            aspectual_tense,
        };
        for form in std::iter::once(entry.word.as_str()).chain(
            entry
                .forms
                .iter()
                .filter(|form| {
                    form.tags
                        .iter()
                        .any(|tag| matches!(tag.as_str(), "alternative" | "canonical" | "lemma"))
                })
                .map(|form| form.form.as_str()),
        ) {
            if form.is_empty() {
                continue;
            }
            index
                .by_form_surface
                .entry(surface_key(form))
                .or_default()
                .push(DictionaryFormCandidate {
                    source_id: candidate.source_id.clone(),
                    candidate_id: candidate.candidate_id.clone(),
                    passage: candidate.passage.clone(),
                    form: form.to_owned(),
                    tags: BTreeSet::from(["lexical-form".into()]),
                    identity: identity.clone(),
                });
        }
        for form in &entry.forms {
            let tags: BTreeSet<_> = form.tags.iter().cloned().collect();
            if tags.iter().any(|tag| {
                matches!(
                    tag.as_str(),
                    "romanization" | "table-tags" | "class" | "alternative" | "canonical" | "lemma"
                )
            }) || form.form.is_empty()
            {
                continue;
            }
            index
                .by_form_surface
                .entry(surface_key(&form.form))
                .or_default()
                .push(DictionaryFormCandidate {
                    source_id: candidate.source_id.clone(),
                    candidate_id: candidate.candidate_id.clone(),
                    passage: candidate.passage.clone(),
                    form: form.form.clone(),
                    tags,
                    identity: identity.clone(),
                });
        }
        let mut forms = vec![entry.word];
        forms.extend(
            entry
                .forms
                .into_iter()
                .filter(|form| {
                    form.tags
                        .iter()
                        .any(|tag| matches!(tag.as_str(), "alternative" | "lemma" | "canonical"))
                })
                .map(|form| form.form),
        );
        for form in forms {
            for key in lemma_keys(&form) {
                index
                    .by_lemma
                    .entry((key, identity.part_of_speech.clone()))
                    .or_default()
                    .insert(identity.candidate_id.clone(), identity.clone());
            }
        }
        index.by_id.insert(identity.candidate_id.clone(), identity);
    }
    load_reviewed_semantics(root, &mut index)?;
    Ok(index)
}

const SEMANTIC_REVIEW_HEADER: &str = "review_id\tsyntacticus_lemma\tpart_of_speech\tsemantic_source_id\tsemantic_candidate_id\tsource_passage\treviewed_headword\tgloss\tdecision\treviewer\treviewed_at\treview_note";

fn load_reviewed_semantics(root: &Path, index: &mut DictionaryIndex) -> Result<(), Box<dyn Error>> {
    let source: BTreeMap<_, _> = read_json_lines::<SourceCandidate>(&root.join(DYACHENKO))?
        .into_iter()
        .map(|candidate| (candidate.candidate_id.clone(), candidate))
        .collect();
    let text = fs::read_to_string(root.join(SEMANTIC_REVIEWS))?;
    let mut lines = text.lines();
    if lines.next() != Some(SEMANTIC_REVIEW_HEADER) {
        return Err("invalid v0.7 semantic-review header".into());
    }
    let mut review_ids = BTreeSet::new();
    let mut reviewed_candidates = BTreeSet::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 12 {
            return Err(format!("invalid v0.7 semantic-review row {}", offset + 2).into());
        }
        if !review_ids.insert(fields[0]) {
            return Err(format!("duplicate v0.7 semantic review {:?}", fields[0]).into());
        }
        if fields[8] != "reviewed" {
            return Err(format!("non-reviewed semantic decision at row {}", offset + 2).into());
        }
        if fields[9].is_empty() || fields[10].is_empty() || fields[11].is_empty() {
            return Err(format!("incomplete semantic rationale at row {}", offset + 2).into());
        }
        let candidate = source.get(fields[4]).ok_or_else(|| {
            format!(
                "unknown D'yachenko candidate {:?} at row {}",
                fields[4],
                offset + 2
            )
        })?;
        if candidate.source_id != fields[3]
            || candidate.passage != fields[5]
            || candidate.partition != "source"
        {
            return Err(format!("semantic provenance mismatch at row {}", offset + 2).into());
        }
        if !reviewed_candidates.insert((fields[1], fields[2], fields[4])) {
            return Err(
                format!("duplicate reviewed semantic mapping at row {}", offset + 2).into(),
            );
        }
        let identity = SemanticIdentity {
            source_id: candidate.source_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            passage: candidate.passage.clone(),
            lemma: fields[1].to_owned(),
            part_of_speech: fields[2].to_owned(),
            gloss: fields[7].to_owned(),
            aspectual_tense: None,
        };
        for key in lemma_keys(&identity.lemma) {
            index
                .by_lemma
                .entry((key, identity.part_of_speech.clone()))
                .or_default()
                .insert(identity.candidate_id.clone(), identity.clone());
        }
        if index
            .by_id
            .insert(identity.candidate_id.clone(), identity)
            .is_some()
        {
            return Err(format!("duplicate governed semantic candidate {:?}", fields[4]).into());
        }
    }
    Ok(())
}

fn load_target_pairs(
    root: &Path,
    wanted: BTreeSet<String>,
) -> Result<BTreeMap<String, TargetPair>, Box<dyn Error>> {
    let mut occurrences = BTreeMap::<
        (String, String),
        (
            BTreeSet<TargetWitness>,
            BTreeSet<TargetWitness>,
            BTreeSet<TargetWitness>,
        ),
    >::new();
    for source_id in TARGET_SOURCES {
        let path = root.join(format!("data/intermediate/synodal/{source_id}.jsonl"));
        for candidate in read_json_lines::<SourceCandidate>(&path)? {
            if candidate.target_recension.as_deref() != Some("synodal-russian") {
                continue;
            }
            for token in tokenize(&candidate.normalized_spelling) {
                let key = surface_key(&token.original);
                if !wanted.contains(&key) {
                    continue;
                }
                let printed: String = token.original.nfc().collect();
                let witness = TargetWitness {
                    source_id: candidate.source_id.clone(),
                    candidate_id: candidate.candidate_id.clone(),
                    passage: candidate.passage.clone(),
                    partition: candidate.partition.clone(),
                    printed: printed.clone(),
                };
                let pair = occurrences.entry((key, printed)).or_default();
                pair.2.insert(witness.clone());
                match candidate.partition.as_str() {
                    "source" => {
                        pair.0.insert(witness);
                    }
                    "evaluation" => {
                        pair.1.insert(witness);
                    }
                    _ => {}
                }
            }
        }
    }
    let mut pairs = BTreeMap::new();
    for ((key, _printed), (source, explicit_evaluation, all)) in occurrences {
        let Some(source) = source.first() else {
            continue;
        };
        let evaluation = explicit_evaluation
            .iter()
            .chain(all.iter())
            .find(|candidate| candidate.passage != source.passage);
        let Some(evaluation) = evaluation else {
            continue;
        };
        pairs.entry(key).or_insert_with(|| TargetPair {
            source: source.clone(),
            evaluation: evaluation.clone(),
        });
    }
    Ok(pairs)
}

fn matching_runtime_identities(
    runtime: &RuntimeMetadata,
    lemma_keys: &BTreeSet<String>,
    allowed_parts: &[&str],
) -> BTreeSet<(String, String)> {
    let mut matches = BTreeSet::new();
    for key in lemma_keys {
        for part in allowed_parts {
            if let Some(ids) = runtime.identities.get(&(key.clone(), (*part).into())) {
                matches.extend(ids.iter().map(|id| (id.clone(), (*part).into())));
            }
        }
    }
    matches
}

fn matching_semantics(
    dictionary: &DictionaryIndex,
    lemma_keys: &BTreeSet<String>,
    allowed_parts: &[&str],
) -> BTreeMap<String, SemanticIdentity> {
    let mut matches = BTreeMap::new();
    for key in lemma_keys {
        for part in allowed_parts {
            if let Some(identities) = dictionary.by_lemma.get(&(key.clone(), (*part).into())) {
                matches.extend(identities.clone());
            }
        }
    }
    matches
}

fn map_syntacticus_cell(
    part_of_speech: &str,
    source: &str,
    lexeme_id: &str,
    runtime: &RuntimeMetadata,
    semantic: Option<&SemanticIdentity>,
) -> Option<String> {
    let fields = source_features(source);
    if matches!(
        part_of_speech,
        "adverb" | "preposition" | "conjunction" | "particle" | "interjection"
    ) {
        return Some("indeclinable".into());
    }
    if matches!(part_of_speech, "noun" | "proper-noun") {
        let case = map_case(fields.get("CASE")?)?;
        let number = map_number(fields.get("NUMB")?)?;
        let animacy = if part_of_speech == "proper-noun" {
            "animate"
        } else if let Some(values) = runtime.animacy_by_lexeme.get(lexeme_id) {
            if values.len() != 1 {
                return None;
            }
            values.first()?.as_str()
        } else if semantic.is_some_and(|identity| gloss_is_animate(&identity.gloss)) {
            "animate"
        } else {
            "inanimate"
        };
        return Some(format!("noun:{case}:{number}:{animacy}"));
    }
    if matches!(part_of_speech, "adjective" | "determiner") {
        let case = map_case(fields.get("CASE")?)?;
        let number = map_number(fields.get("NUMB")?)?;
        let gender = map_gender(fields.get("GEND")?)?;
        let form = match fields.get("STRE").map(String::as_str) {
            Some("s") => "short",
            Some("w" | "t") => "long",
            _ => return None,
        };
        let comparison = match fields.get("DEGR").map(String::as_str) {
            Some("p") => "positive",
            Some("c") => "comparative",
            Some("s") => "superlative",
            _ => return None,
        };
        return Some(format!(
            "{part_of_speech}:{case}:{number}:{gender}:any:{form}:{comparison}"
        ));
    }
    if part_of_speech == "pronoun" {
        let case = map_case(fields.get("CASE")?)?;
        let number = map_number(fields.get("NUMB")?)?;
        let gender = map_optional_gender(fields.get("GEND")?)?;
        let person = fields
            .get("PERS")
            .and_then(|value| map_person(value))
            .unwrap_or("none");
        return Some(format!("pronoun:{case}:{number}:{gender}:{person}:any"));
    }
    if part_of_speech == "numeral" {
        let case = map_case(fields.get("CASE")?)?;
        let number = map_number(fields.get("NUMB")?)?;
        let gender = map_optional_gender(fields.get("GEND")?)?;
        let kind = if let Some(values) = runtime.numeral_kind_by_lexeme.get(lexeme_id) {
            if values.len() != 1 {
                return None;
            }
            values.first()?.as_str()
        } else {
            infer_numeral_kind(semantic?)?
        };
        return Some(format!("numeral:{kind}:{case}:{number}:{gender}:any"));
    }
    if part_of_speech != "verb" {
        return None;
    }
    let number = map_number(fields.get("NUMB")?)?;
    match fields.get("MOOD").map(String::as_str) {
        Some("n") => return Some("infinitive".into()),
        Some("u") => return Some("supine".into()),
        Some("m") => {
            let person = map_person(fields.get("PERS")?)?;
            return Some(format!("imperative:{person}:{number}"));
        }
        Some("i") => {
            let person = map_person(fields.get("PERS")?)?;
            let tense = match fields.get("TENS").map(String::as_str) {
                Some("a") => "aorist".to_owned(),
                Some("i") => "imperfect".to_owned(),
                Some("f") => "future".to_owned(),
                Some("p") => {
                    let mut values = runtime
                        .aspectual_tense_by_lexeme
                        .get(lexeme_id)
                        .cloned()
                        .unwrap_or_default();
                    if semantic.is_some_and(|identity| surface_key(&identity.lemma) == "ити") {
                        values.clear();
                        values.insert("present".into());
                    } else if let Some(value) =
                        semantic.and_then(|identity| identity.aspectual_tense.clone())
                    {
                        values.insert(value);
                    }
                    if values.len() != 1 {
                        return None;
                    }
                    values.into_iter().next()?
                }
                _ => return None,
            };
            return Some(format!("{tense}:{person}:{number}"));
        }
        Some("p") => {}
        _ => return None,
    }
    let gender = map_gender(fields.get("GEND")?)?;
    if fields.get("TENS").map(String::as_str) == Some("s") {
        return Some(format!("l-participle:{gender}:{number}"));
    }
    let voice = match fields.get("VOIC").map(String::as_str) {
        Some("a") => "active",
        Some("p") => "passive",
        _ => return None,
    };
    let tense = match fields.get("TENS").map(String::as_str) {
        Some("p") => "present",
        Some("u") => "past",
        _ => return None,
    };
    let form = match fields.get("STRE").map(String::as_str) {
        Some("s") => "short",
        Some("w" | "t") => "long",
        _ => return None,
    };
    let case = map_case(fields.get("CASE")?)?;
    Some(format!(
        "participle:{tense}:{voice}:{case}:{number}:{gender}:any:{form}:positive"
    ))
}

fn map_dictionary_form_cell(
    candidate: &DictionaryFormCandidate,
    lexeme_id: &str,
    runtime: &RuntimeMetadata,
) -> Option<String> {
    let tags = &candidate.tags;
    let part_of_speech = candidate.identity.part_of_speech.as_str();
    if tags.contains("lexical-form") {
        return Some(
            if matches!(
                part_of_speech,
                "adverb" | "preposition" | "conjunction" | "particle" | "interjection"
            ) {
                "indeclinable"
            } else {
                "lexical-form"
            }
            .into(),
        );
    }
    if matches!(
        part_of_speech,
        "adverb" | "preposition" | "conjunction" | "particle" | "interjection"
    ) {
        return Some("indeclinable".into());
    }
    if matches!(part_of_speech, "noun" | "proper-noun") {
        let case = unique_tag(
            tags,
            &[
                "nominative",
                "genitive",
                "dative",
                "accusative",
                "instrumental",
                "locative",
                "vocative",
            ],
        )?;
        let number = unique_tag(tags, &["singular", "dual", "plural"])?;
        let animacy = if part_of_speech == "proper-noun" {
            "animate"
        } else if let Some(values) = runtime.animacy_by_lexeme.get(lexeme_id) {
            (values.len() == 1)
                .then(|| values.first())
                .flatten()?
                .as_str()
        } else if gloss_is_animate(&candidate.identity.gloss) {
            "animate"
        } else {
            "inanimate"
        };
        return Some(format!("noun:{case}:{number}:{animacy}"));
    }
    if part_of_speech == "pronoun" {
        let case = unique_tag(
            tags,
            &[
                "nominative",
                "genitive",
                "dative",
                "accusative",
                "instrumental",
                "locative",
                "vocative",
            ],
        )?;
        let number = unique_tag(tags, &["singular", "dual", "plural"])?;
        let gender = optional_unique_tag(tags, &["masculine", "feminine", "neuter"])?;
        let person = optional_unique_tag(tags, &["first-person", "second-person", "third-person"])?;
        let person = match person {
            "first-person" => "first",
            "second-person" => "second",
            "third-person" => "third",
            "any" => "none",
            _ => return None,
        };
        return Some(format!("pronoun:{case}:{number}:{gender}:{person}:any"));
    }
    if part_of_speech == "numeral" {
        let case = unique_tag(
            tags,
            &[
                "nominative",
                "genitive",
                "dative",
                "accusative",
                "instrumental",
                "locative",
                "vocative",
            ],
        )?;
        let number = unique_tag(tags, &["singular", "dual", "plural"])?;
        let gender = optional_unique_tag(tags, &["masculine", "feminine", "neuter"])?;
        let kind = if let Some(values) = runtime.numeral_kind_by_lexeme.get(lexeme_id) {
            (values.len() == 1)
                .then(|| values.first())
                .flatten()?
                .as_str()
        } else {
            infer_numeral_kind(&candidate.identity)?
        };
        return Some(format!("numeral:{kind}:{case}:{number}:{gender}:any"));
    }
    if part_of_speech != "verb" {
        return None;
    }
    if tags.contains("infinitive") {
        return Some("infinitive".into());
    }
    let mood = unique_tag(tags, &["indicative", "imperative"])?;
    let number = unique_tag(tags, &["singular", "dual", "plural"])?;
    let person = unique_tag(tags, &["first-person", "second-person", "third-person"])?;
    let person = match person {
        "first-person" => "first",
        "second-person" => "second",
        "third-person" => "third",
        _ => return None,
    };
    if mood == "imperative" {
        return Some(format!("imperative:{person}:{number}"));
    }
    let tense = unique_tag(tags, &["present", "future", "aorist", "imperfect"])?;
    Some(format!("{tense}:{person}:{number}"))
}

fn unique_tag<'a>(tags: &BTreeSet<String>, choices: &[&'a str]) -> Option<&'a str> {
    let mut matches = choices
        .iter()
        .copied()
        .filter(|choice| tags.contains(*choice));
    let value = matches.next()?;
    matches.next().is_none().then_some(value)
}

fn optional_unique_tag<'a>(tags: &BTreeSet<String>, choices: &[&'a str]) -> Option<&'a str> {
    let mut matches = choices
        .iter()
        .copied()
        .filter(|choice| tags.contains(*choice));
    let value = matches.next().unwrap_or("any");
    matches.next().is_none().then_some(value)
}

fn source_features(source: &str) -> BTreeMap<String, String> {
    source
        .split('|')
        .filter(|field| field.len() >= 5)
        .map(|field| (field[..4].into(), field[4..].into()))
        .collect()
}

fn runtime_parts_for_syntacticus(value: &str) -> Vec<&'static str> {
    match value {
        "N" => vec!["noun", "proper-noun"],
        "V" => vec!["verb"],
        // Explicit identity reviews may resolve the source's broad adjective
        // and pronoun tags to the public determiner identities. Without such a
        // review, ambiguous matches still abstain below.
        "A" => vec!["adjective", "determiner"],
        "P" => vec!["pronoun", "determiner"],
        "M" => vec!["numeral"],
        "D" => vec!["adverb"],
        "R" => vec!["preposition"],
        "C" => vec!["conjunction"],
        "G" => vec!["particle"],
        "I" => vec!["interjection"],
        _ => Vec::new(),
    }
}

fn map_dictionary_pos(value: &str) -> Option<&'static str> {
    Some(match value {
        "noun" => "noun",
        "name" => "proper-noun",
        "verb" => "verb",
        "adj" => "adjective",
        "pron" => "pronoun",
        "num" => "numeral",
        "adv" => "adverb",
        "prep" => "preposition",
        "conj" => "conjunction",
        "particle" => "particle",
        "intj" => "interjection",
        _ => return None,
    })
}

fn dictionary_aspectual_tense(entry: &WiktionaryEntry) -> Option<String> {
    let mut values = BTreeSet::new();
    for template in &entry.head_templates {
        for value in template
            .args
            .values()
            .map(String::as_str)
            .chain(std::iter::once(template.expansion.as_str()))
        {
            let tokens: BTreeSet<_> = value
                .split(|character: char| !character.is_ascii_alphabetic())
                .filter(|token| !token.is_empty())
                .collect();
            if tokens.contains("pf") {
                values.insert("future".to_owned());
            }
            if tokens.contains("impf") {
                values.insert("present".to_owned());
            }
        }
    }
    (values.len() == 1)
        .then(|| values.into_iter().next())
        .flatten()
}

fn strict_surface_key(value: &str) -> String {
    normalize_lookup_accentless(value)
        .to_lowercase()
        .replace(['і', 'ї'], "и")
        .replace(['ѡ', 'ѻ'], "о")
        .replace('ѿ', "от")
        .replace("ᲂу", "у")
        .replace('ꙋ', "у")
}

fn surface_key(value: &str) -> String {
    strict_surface_key(value)
        .replace('ѫ', "у")
        .replace('ꙑ', "ы")
}

fn is_abbreviation_surface(value: &str) -> bool {
    value
        .chars()
        .any(|character| character == '\u{0483}' || matches!(character, '\u{2de0}'..='\u{2dff}'))
}

fn lemma_keys(value: &str) -> BTreeSet<String> {
    let base = surface_key(strip_homograph_suffix(value));
    let mut keys = BTreeSet::from([base.clone(), base.replace("оу", "у")]);
    for key in keys.clone() {
        if let Some(stripped) = key.strip_suffix(['ъ', 'ь']) {
            keys.insert(stripped.into());
        }
    }
    for key in keys.clone() {
        keys.insert(key.replace(['ъ', 'ь'], ""));
    }
    for key in keys.clone() {
        keys.insert(
            key.replace(['ꙗ', 'ѧ', 'ѩ'], "я")
                .replace('є', "е")
                .replace('ѕ', "з")
                .replace('ѯ', "кс")
                .replace('ѱ', "пс")
                .replace('ѳ', "ф")
                .replace('ѵ', "и"),
        );
    }
    keys
}

fn strip_homograph_suffix(value: &str) -> &str {
    value
        .rsplit_once('#')
        .filter(|(_, suffix)| {
            !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
        })
        .map_or(value, |(lemma, _)| lemma)
}

fn lemma_requires_weak_yer_correspondence(value: &str) -> bool {
    let conservative = surface_key(strip_homograph_suffix(value));
    let yerless = conservative.replace(['ъ', 'ь'], "");
    conservative != yerless
        && conservative
            .chars()
            .take(conservative.chars().count().saturating_sub(1))
            .any(|character| matches!(character, 'ъ' | 'ь'))
}

fn lemma_requires_historical_letter_correspondence(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(
            character,
            'ꙗ' | 'ѧ' | 'ѩ' | 'є' | 'ѕ' | 'ѯ' | 'ѱ' | 'ѳ' | 'ѵ'
        )
    })
}

fn map_case(value: &str) -> Option<&'static str> {
    Some(match value {
        "n" => "nominative",
        "g" => "genitive",
        "d" => "dative",
        "a" => "accusative",
        "i" => "instrumental",
        "l" => "locative",
        "v" => "vocative",
        _ => return None,
    })
}

fn map_number(value: &str) -> Option<&'static str> {
    Some(match value {
        "s" => "singular",
        "d" => "dual",
        "p" => "plural",
        _ => return None,
    })
}

fn map_gender(value: &str) -> Option<&'static str> {
    Some(match value {
        "m" => "masculine",
        "f" => "feminine",
        "n" => "neuter",
        _ => return None,
    })
}

fn map_optional_gender(value: &str) -> Option<&'static str> {
    match value {
        "p" | "q" => Some("any"),
        _ => map_gender(value),
    }
}

fn map_person(value: &str) -> Option<&'static str> {
    Some(match value {
        "1" => "first",
        "2" => "second",
        "3" => "third",
        _ => return None,
    })
}

fn gloss_is_animate(gloss: &str) -> bool {
    let words: BTreeSet<_> = gloss
        .split(|character: char| !character.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    [
        "person",
        "man",
        "woman",
        "king",
        "queen",
        "priest",
        "apostle",
        "prophet",
        "ruler",
        "servant",
        "slave",
        "enemy",
        "friend",
        "brother",
        "father",
        "mother",
        "son",
        "daughter",
        "child",
        "children",
        "people",
        "angel",
        "devil",
        "god",
        "lord",
        "leader",
        "bishop",
        "disciple",
        "student",
        "helper",
        "assistant",
        "sinner",
        "saviour",
        "savior",
        "pharaoh",
        "sister",
        "widow",
        "virgin",
        "soldier",
        "human",
        "animal",
        "beast",
        "bird",
        "fish",
        "horse",
        "sheep",
        "goat",
        "bull",
        "calf",
        "lamb",
        "lion",
        "dragon",
        "serpent",
    ]
    .iter()
    .any(|word| words.contains(*word))
}

fn infer_numeral_kind(identity: &SemanticIdentity) -> Option<&'static str> {
    let words: BTreeSet<_> = identity
        .gloss
        .split(|character: char| !character.is_alphabetic())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    let ordinal = [
        "first",
        "second",
        "third",
        "fourth",
        "fifth",
        "sixth",
        "seventh",
        "eighth",
        "ninth",
        "tenth",
        "eleventh",
        "twelfth",
        "hundredth",
        "thousandth",
    ];
    if ordinal.iter().any(|word| words.contains(*word)) {
        Some("ordinal")
    } else if !identity.gloss.is_empty() {
        Some("cardinal")
    } else {
        None
    }
}

fn unsafe_gloss(gloss: &str) -> bool {
    let lower = gloss.to_lowercase();
    [
        "alternative form",
        "variant of",
        "form of",
        "inflection of",
        "old east church slavonic form",
        "indicative of",
        "imperative of",
        "participle of",
        "letter of",
        "rotating part",
        "mechanism",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn stable_new_lexeme_id(identity: &SemanticIdentity) -> String {
    let mut digest = Sha256::new();
    digest.update(identity.candidate_id.as_bytes());
    let hex = format!("{:x}", digest.finalize());
    format!("synodal:{}:v07-{}", identity.part_of_speech, &hex[..16])
}

fn stable_packet_id(source_id: &str, lexeme_id: &str, cell: &str, printed: &str) -> String {
    let mut digest = Sha256::new();
    for value in [source_id, lexeme_id, cell, printed] {
        digest.update(value.as_bytes());
        digest.update([0]);
    }
    let hex = format!("{:x}", digest.finalize());
    format!("v07-exact-{}", &hex[..16])
}

fn strict_threshold(total: usize, basis_points: usize) -> usize {
    total.saturating_mul(basis_points) / 10_000 + 1
}

fn render_tsv(report: &ReviewPacketReport) -> String {
    let mut out = String::from(
        "rank\tpacket_id\tsurface\tnormalized_surface\ttop_k_uncovered_frequency\tdocument_frequency\tlexeme_id\tlemma\tpart_of_speech\tcell\tevidence_lane\tidentity_status\tsemantic_gloss\tsource_morphology\tevidence_roles\tsource_passage\tevaluation_passage\tcontexts\trisk_flags\tprior_decisions\tpredicted_unique_tokens\tdecision\treviewer_note\n",
    );
    for packet in &report.packets {
        let roles = packet
            .evidence_roles
            .iter()
            .map(|role| {
                format!(
                    "{}:{}:{}:{}",
                    role.role, role.source_id, role.candidate_id, role.passage
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            packet.rank,
            tsv(&packet.packet_id),
            tsv(&packet.surface),
            tsv(&packet.normalized_surface),
            packet.top_k_uncovered_frequency,
            packet.document_frequency,
            tsv(&packet.lexeme_id),
            tsv(&packet.lemma),
            tsv(&packet.part_of_speech),
            tsv(&packet.cell),
            tsv(&packet.evidence_lane),
            tsv(&packet.identity_status),
            tsv(&packet.semantic_gloss),
            tsv(&packet.source_morphology),
            tsv(&roles),
            tsv(&packet.source_passage),
            tsv(&packet.evaluation_passage),
            tsv(&packet.contexts.join(" | ")),
            tsv(&packet.risk_flags.join(",")),
            tsv(&packet.prior_decisions.join(",")),
            packet.predicted_unique_tokens,
            tsv(&packet.decision),
            tsv(&packet.reviewer_note),
        ));
    }
    out
}

fn render_markdown(report: &ReviewPacketReport) -> String {
    let mut out = format!(
        "# Synodal v0.7 source-typed review packets\n\nThese packets are acquisition and review aids. No predicted token counts enter coverage until an explicit decision is represented in the v0.7 ledgers and the canonical resolver is regenerated.\n\n- Locked v0.6 baseline top-k: {} of {}\n- Current top-k: {}\n- Strictly-more-than-70% threshold: {}\n- Remaining current deficit: {}\n- Candidate exact-cell rows: {}\n- Unique candidate surfaces: {}\n- Overlap-adjusted candidate tokens: {}\n- Blocked identity conflicts retained for review: {}\n\n| Rank | Surface | Frequency | Lexeme | Cell | Identity | Risks | Decision |\n|---:|---|---:|---|---|---|---|---|\n",
        report.baseline_top_k,
        report.corpus_tokens,
        report.current_top_k,
        report.strictly_more_than_70_percent,
        report.tokens_needed_for_70_percent,
        report.packet_rows,
        report.unique_candidate_surfaces,
        report.overlap_adjusted_candidate_tokens,
        report.identity_conflicts.len(),
    );
    for packet in &report.packets {
        out.push_str(&format!(
            "| {} | `{}` | {} | `{}` | `{}` | `{}` | {} | `{}` |\n",
            packet.rank,
            escape_markdown(&packet.surface),
            packet.top_k_uncovered_frequency,
            escape_markdown(&packet.lexeme_id),
            escape_markdown(&packet.cell),
            packet.identity_status,
            escape_markdown(&packet.risk_flags.join(", ")),
            packet.decision,
        ));
    }
    out
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn read_json_lines<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Vec<T>, Box<dyn Error>> {
    fs::read_to_string(path)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_str(line).map_err(Into::into))
        .collect()
}

fn read_tsv(path: &Path) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| line.split('\t').map(str::to_owned).collect())
        .collect())
}

fn field<'a>(row: &'a [String], index: usize, label: &str) -> Result<&'a str, Box<dyn Error>> {
    row.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("TSV row omits {label}").into())
}

fn sanitize(value: &str) -> String {
    value
        .replace(['\t', '\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tsv(value: &str) -> String {
    sanitize(value)
}

fn escape_markdown(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "\\`")
}

fn write_if_changed(path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
    if fs::read_to_string(path).ok().as_deref() != Some(contents) {
        fs::write(path, contents)?;
    }
    Ok(())
}

fn check_contents(path: &Path, expected: &str) -> Result<(), Box<dyn Error>> {
    if fs::read_to_string(path).ok().as_deref() == Some(expected) {
        Ok(())
    } else {
        Err(format!("stale {}; rerun synodal-v07-review-packets", path.display()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_surface_key_is_bounded() {
        assert_eq!(surface_key("дꙋ́ша"), surface_key("дѫша"));
        assert_ne!(surface_key("душа"), surface_key("дщерь"));
    }

    #[test]
    fn syntacticus_cells_are_typed() {
        let runtime = RuntimeMetadata::default();
        let semantic = SemanticIdentity {
            source_id: "test-source".into(),
            candidate_id: "semantic".into(),
            passage: "test-passage".into(),
            lemma: "душа".into(),
            part_of_speech: "noun".into(),
            gloss: "soul".into(),
            aspectual_tense: None,
        };
        assert_eq!(
            map_syntacticus_cell(
                "noun",
                "NUMBs|GENDf|CASEa",
                "new",
                &runtime,
                Some(&semantic),
            ),
            Some("noun:accusative:singular:inanimate".into())
        );
        assert_eq!(
            map_syntacticus_cell(
                "verb",
                "PERS3|NUMBp|TENSa|MOODi|VOICa",
                "new",
                &runtime,
                Some(&semantic),
            ),
            Some("aorist:third:plural".into())
        );
    }
}
