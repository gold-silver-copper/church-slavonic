use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

const PACKETS: &str = "reports/synodal-v07-review-packets.json";
const REVIEWS: &str = "data/synodal/v07_exact_reviews_wave9.tsv";
const VARIANT_REVIEWS: &str = "data/synodal/v07_variant_reviews.tsv";
const ABBREVIATION_REVIEWS: &str = "data/synodal/v07_abbreviation_reviews.tsv";
const IDENTITY_CORRECTIONS: &str = "data/synodal/v07_identity_corrections.tsv";
const EVIDENCE_CORRECTIONS: &str = "data/synodal/v07_evidence_corrections.tsv";
const PONOMAR: &str = "data/intermediate/synodal/ponomar-elizabeth-bible-2026-08-09.jsonl";
const WIKISOURCE: &str =
    "data/intermediate/synodal/wikisource-church-slavonic-bible-2026-08-09.jsonl";

const REVIEW_HEADER: &str = "packet_id\tdecision\trealized_unique_tokens\tblocker\treview_note";
const VARIANT_REVIEW_HEADER: &str = "review_id\tlane\tlexeme_id\tsense_id\tcell\texpanded\tprinted\tbase_printed\tbase_evidence_id\tpredicted_unique_tokens\tsource_id\tsource_candidate_id\tsource_passage\tevaluation_source_id\tevaluation_candidate_id\tevaluation_passage\tdecision\treview_note";
const ABBREVIATION_REVIEW_HEADER: &str = "review_id\tlexeme_id\tsense_id\tcell\texpanded\tprinted\tbase_printed\tbase_evidence_id\tnormative_evidence_id\tpredicted_unique_tokens\tsource_id\tsource_candidate_id\tsource_passage\tevaluation_source_id\tevaluation_candidate_id\tevaluation_passage\tcontext_restrictions\tambiguity\tdecision\treview_note";
const IDENTITY_CORRECTION_HEADER: &str = "correction_id\tobsolete_lexeme_id\tobsolete_sense_id\tobsolete_review_id\tcanonical_lexeme_id\tcanonical_sense_id\tcanonical_review_id\tsemantic_candidate_id\tdecision\treview_note";
const EVIDENCE_CORRECTION_HEADER: &str = "correction_id\tobsolete_evidence_id\treplacement_evidence_id\tprinted\tsource_id\tcandidate_id\tsource_passage\tdecision\treview_note";
const EXACT_HEADER: &str =
    "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension";
const ABBREVIATION_HEADER: &str = "lexeme_id\tsense_id\tcell\texpanded\tprinted\trule_id\tevidence_id\treversible\trequired_marks\tcontext_restrictions\tambiguity\tsource_recension\ttarget_recension";
const ABBREVIATION_EVALUATION_HEADER: &str = "id\tlexeme_id\tsense_id\tcell\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";
const LEXICAL_HEADER: &str = "review_id\tlexeme_id\tsense_id\tlemma\tpart_of_speech\tcell\texpanded\tprinted\tgloss\tdomains\tsemantic_source_id\tsemantic_candidate_id\tattestation_source_id\tattestation_candidate_id\tcitation\tdecision\ttarget_recension\treview_note";
const EVIDENCE_HEADER: &str =
    "evidence_id\tcandidate_id\tsource_id\tcitation\tdecision\ttarget_recension\treview_note";
const EVALUATION_HEADER: &str = "id\tlexeme_id\tcell\tpolicy\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity";

#[derive(Clone, Debug, Deserialize)]
struct PacketReport {
    packets: Vec<Packet>,
}

#[derive(Clone, Debug, Deserialize)]
struct Packet {
    packet_id: String,
    surface: String,
    normalized_surface: String,
    lexeme_id: String,
    lemma: String,
    part_of_speech: String,
    cell: String,
    identity_status: String,
    semantic_gloss: String,
    evidence_roles: Vec<EvidenceRole>,
}

#[derive(Clone, Debug, Deserialize)]
struct EvidenceRole {
    role: String,
    source_id: String,
    candidate_id: String,
    passage: String,
}

#[derive(Clone, Debug)]
struct Review {
    decision: String,
    realized_unique_tokens: usize,
    blocker: String,
    review_note: String,
}

#[derive(Clone, Debug)]
struct VariantReview {
    review_id: String,
    lane: String,
    lexeme_id: String,
    sense_id: String,
    cell: String,
    expanded: String,
    printed: String,
    base_printed: String,
    base_evidence_id: String,
    predicted_unique_tokens: usize,
    source_id: String,
    source_candidate_id: String,
    source_passage: String,
    evaluation_source_id: String,
    evaluation_candidate_id: String,
    evaluation_passage: String,
    review_note: String,
}

#[derive(Clone, Debug)]
struct AbbreviationReview {
    review_id: String,
    lexeme_id: String,
    sense_id: String,
    cell: String,
    expanded: String,
    printed: String,
    base_printed: String,
    base_evidence_id: String,
    normative_evidence_id: String,
    predicted_unique_tokens: usize,
    source_id: String,
    source_candidate_id: String,
    source_passage: String,
    evaluation_source_id: String,
    evaluation_candidate_id: String,
    evaluation_passage: String,
    context_restrictions: String,
    ambiguity: String,
    review_note: String,
}

#[derive(Clone, Debug)]
struct IdentityCorrection {
    correction_id: String,
    obsolete_lexeme_id: String,
    obsolete_sense_id: String,
    obsolete_review_id: String,
    canonical_lexeme_id: String,
    canonical_sense_id: String,
    canonical_review_id: String,
    semantic_candidate_id: String,
    review_note: String,
}

#[derive(Clone, Debug)]
struct EvidenceCorrection {
    correction_id: String,
    obsolete_evidence_id: String,
    replacement_evidence_id: String,
    printed: String,
    source_id: String,
    candidate_id: String,
    source_passage: String,
    review_note: String,
}

#[derive(Clone, Debug, Deserialize)]
struct TargetCandidate {
    candidate_id: String,
    source_id: String,
    passage: String,
    partition: String,
    normalized_spelling: String,
}

#[derive(Default)]
struct DerivedRows {
    exact: BTreeSet<String>,
    abbreviation: BTreeSet<String>,
    lexical: BTreeMap<String, String>,
    evidence: BTreeMap<String, String>,
    evaluation: BTreeSet<String>,
    abbreviation_evaluation: BTreeSet<String>,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            value => return Err(format!("unknown synodal-v07-apply argument {value:?}").into()),
        }
    }

    let report: PacketReport = serde_json::from_slice(&fs::read(root.join(PACKETS))?)?;
    let reviews = load_reviews(&root.join(REVIEWS))?;
    validate_review_coverage(&report, &reviews)?;
    let variant_reviews = load_variant_reviews(&root.join(VARIANT_REVIEWS))?;
    let abbreviation_reviews = load_abbreviation_reviews(&root.join(ABBREVIATION_REVIEWS))?;
    let identity_corrections = load_identity_corrections(&root.join(IDENTITY_CORRECTIONS))?;
    let evidence_corrections = load_evidence_corrections(&root.join(EVIDENCE_CORRECTIONS))?;
    validate_identity_corrections(root, &identity_corrections)?;
    let candidates = load_target_candidates(root)?;
    validate_evidence_corrections(root, &evidence_corrections, &candidates)?;
    validate_variant_reviews(root, &variant_reviews, &candidates)?;
    validate_abbreviation_reviews(root, &abbreviation_reviews, &candidates)?;
    let mut derived = derive_rows(&report, &reviews)?;
    derive_variant_rows(root, &variant_reviews, &mut derived)?;
    derive_abbreviation_rows(root, &abbreviation_reviews, &mut derived)?;
    derive_evidence_correction_rows(&evidence_corrections, &mut derived)?;
    let outputs = [
        (
            "data/synodal/exact_forms.tsv",
            EXACT_HEADER,
            derived.exact.into_iter().collect::<Vec<_>>(),
        ),
        (
            "data/synodal/abbreviations.tsv",
            ABBREVIATION_HEADER,
            derived.abbreviation.into_iter().collect::<Vec<_>>(),
        ),
        (
            "data/synodal/lexical_reviews.tsv",
            LEXICAL_HEADER,
            derived.lexical.into_values().collect::<Vec<_>>(),
        ),
        (
            "data/synodal/reviewed_evidence.tsv",
            EVIDENCE_HEADER,
            derived.evidence.into_values().collect::<Vec<_>>(),
        ),
        (
            "data/synodal/evaluation.tsv",
            EVALUATION_HEADER,
            derived.evaluation.into_iter().collect::<Vec<_>>(),
        ),
        (
            "data/synodal/abbreviation_evaluation.tsv",
            ABBREVIATION_EVALUATION_HEADER,
            derived
                .abbreviation_evaluation
                .into_iter()
                .collect::<Vec<_>>(),
        ),
    ];

    for (relative, header, rows) in outputs {
        let path = root.join(relative);
        let desired = synchronized_table(
            &path,
            header,
            &rows,
            &identity_corrections,
            &evidence_corrections,
        )?;
        if check {
            if fs::read_to_string(&path)? != desired {
                return Err(format!("stale {relative}; run cargo xtask synodal-v07-apply").into());
            }
        } else if fs::read_to_string(&path)? != desired {
            fs::write(&path, desired)?;
        }
    }

    let admitted = reviews
        .values()
        .filter(|review| review.decision == "admitted")
        .count();
    let variant_predicted: usize = variant_reviews
        .iter()
        .map(|review| review.predicted_unique_tokens)
        .sum();
    let abbreviation_predicted: usize = abbreviation_reviews
        .iter()
        .map(|review| review.predicted_unique_tokens)
        .sum();
    println!(
        "Synodal v0.7 admission materialization: {admitted} admitted packets, {} adjudicated packets, {} admitted explicit variants ({variant_predicted} predicted tokens), {} admitted typed abbreviations ({abbreviation_predicted} predicted tokens), {} reviewed identity corrections, {} reviewed evidence corrections",
        reviews.len(),
        variant_reviews.len(),
        abbreviation_reviews.len(),
        identity_corrections.len(),
        evidence_corrections.len()
    );
    Ok(())
}

fn load_identity_corrections(path: &Path) -> Result<Vec<IdentityCorrection>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(IDENTITY_CORRECTION_HEADER) {
        return Err(format!(
            "invalid v0.7 identity-correction header in {}",
            path.display()
        )
        .into());
    }
    let mut correction_ids = BTreeSet::new();
    let mut obsolete_lexemes = BTreeSet::new();
    let mut corrections = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 10 {
            return Err(format!("invalid v0.7 identity-correction row {}", offset + 2).into());
        }
        if fields[8] != "merged" || fields[9].trim().is_empty() {
            return Err(
                format!("incomplete v0.7 identity correction at row {}", offset + 2).into(),
            );
        }
        if !correction_ids.insert(fields[0]) {
            return Err(format!("duplicate v0.7 identity correction {:?}", fields[0]).into());
        }
        if !obsolete_lexemes.insert(fields[1]) || fields[1] == fields[4] {
            return Err(format!("invalid obsolete identity at row {}", offset + 2).into());
        }
        if fields[..8].iter().any(|field| field.is_empty()) {
            return Err(
                format!("incomplete v0.7 identity correction at row {}", offset + 2).into(),
            );
        }
        corrections.push(IdentityCorrection {
            correction_id: fields[0].into(),
            obsolete_lexeme_id: fields[1].into(),
            obsolete_sense_id: fields[2].into(),
            obsolete_review_id: fields[3].into(),
            canonical_lexeme_id: fields[4].into(),
            canonical_sense_id: fields[5].into(),
            canonical_review_id: fields[6].into(),
            semantic_candidate_id: fields[7].into(),
            review_note: fields[9].into(),
        });
    }
    Ok(corrections)
}

fn load_evidence_corrections(path: &Path) -> Result<Vec<EvidenceCorrection>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(EVIDENCE_CORRECTION_HEADER) {
        return Err(format!(
            "invalid v0.7 evidence-correction header in {}",
            path.display()
        )
        .into());
    }
    let mut correction_ids = BTreeSet::new();
    let mut obsolete_ids = BTreeSet::new();
    let mut replacement_ids = BTreeSet::new();
    let mut corrections = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 9 || fields[..7].iter().any(|field| field.is_empty()) {
            return Err(format!("invalid v0.7 evidence-correction row {}", offset + 2).into());
        }
        if fields[7] != "replaced" || fields[8].trim().is_empty() {
            return Err(
                format!("incomplete v0.7 evidence correction at row {}", offset + 2).into(),
            );
        }
        if !correction_ids.insert(fields[0])
            || !obsolete_ids.insert(fields[1])
            || !replacement_ids.insert(fields[2])
            || fields[1] == fields[2]
        {
            return Err(format!("duplicate v0.7 evidence correction at row {}", offset + 2).into());
        }
        corrections.push(EvidenceCorrection {
            correction_id: fields[0].into(),
            obsolete_evidence_id: fields[1].into(),
            replacement_evidence_id: fields[2].into(),
            printed: fields[3].into(),
            source_id: fields[4].into(),
            candidate_id: fields[5].into(),
            source_passage: fields[6].into(),
            review_note: fields[8].into(),
        });
    }
    Ok(corrections)
}

fn validate_identity_corrections(
    root: &Path,
    corrections: &[IdentityCorrection],
) -> Result<(), Box<dyn Error>> {
    let lexical = fs::read_to_string(root.join("data/synodal/lexical_reviews.tsv"))?;
    let mut lines = lexical.lines();
    if lines.next() != Some(LEXICAL_HEADER) {
        return Err(
            "invalid lexical review table while validating v0.7 identity corrections".into(),
        );
    }
    let rows = lines
        .filter(|line| !line.is_empty())
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    for correction in corrections {
        let obsolete = rows.iter().find(|fields| {
            fields.first() == Some(&correction.obsolete_review_id.as_str())
                && fields.get(1) == Some(&correction.obsolete_lexeme_id.as_str())
                && fields.get(2) == Some(&correction.obsolete_sense_id.as_str())
        });
        let canonical = rows.iter().find(|fields| {
            fields.first() == Some(&correction.canonical_review_id.as_str())
                && fields.get(1) == Some(&correction.canonical_lexeme_id.as_str())
                && fields.get(2) == Some(&correction.canonical_sense_id.as_str())
        });
        let Some(canonical) = canonical else {
            return Err(format!(
                "{} does not resolve a canonical reviewed lexical identity",
                correction.correction_id
            )
            .into());
        };
        let invalid_obsolete = obsolete.is_some_and(|obsolete| {
            obsolete.get(11) != Some(&correction.semantic_candidate_id.as_str())
                || !reviewed_glosses_overlap(obsolete.get(8), canonical.get(8))
                || obsolete.get(4) != canonical.get(4)
                || obsolete.get(15) != Some(&"reviewed")
        });
        if invalid_obsolete
            || canonical.get(11) != Some(&correction.semantic_candidate_id.as_str())
            || canonical.get(15) != Some(&"reviewed")
        {
            return Err(format!(
                "{} does not prove a duplicate reviewed semantic identity",
                correction.correction_id
            )
            .into());
        }
        if correction.review_note.trim().is_empty() {
            return Err(format!("{} has no reviewer rationale", correction.correction_id).into());
        }
    }
    Ok(())
}

fn reviewed_glosses_overlap(left: Option<&&str>, right: Option<&&str>) -> bool {
    let (Some(left), Some(right)) = (left, right) else {
        return false;
    };
    let left = left.to_lowercase();
    let right = right.to_lowercase();
    left == right || left.contains(&right) || right.contains(&left)
}

fn validate_evidence_corrections(
    root: &Path,
    corrections: &[EvidenceCorrection],
    candidates: &BTreeMap<String, TargetCandidate>,
) -> Result<(), Box<dyn Error>> {
    let evidence = fs::read_to_string(root.join("data/synodal/reviewed_evidence.tsv"))?;
    for correction in corrections {
        let obsolete_exists = evidence
            .lines()
            .skip(1)
            .any(|line| line.split('\t').next() == Some(correction.obsolete_evidence_id.as_str()));
        let replacement_exists = evidence.lines().skip(1).any(|line| {
            line.split('\t').next() == Some(correction.replacement_evidence_id.as_str())
        });
        if !obsolete_exists && !replacement_exists {
            return Err(format!(
                "{} cites neither obsolete nor replacement evidence",
                correction.correction_id
            )
            .into());
        }
        let candidate = candidates.get(&correction.candidate_id).ok_or_else(|| {
            format!(
                "{} cites unknown replacement candidate",
                correction.correction_id
            )
        })?;
        if candidate.source_id != correction.source_id
            || candidate.passage != correction.source_passage
            || candidate.partition != "source"
            || !contains_whole_token(&candidate.normalized_spelling, &correction.printed)
        {
            return Err(format!(
                "{} has invalid replacement provenance",
                correction.correction_id
            )
            .into());
        }
        if correction.printed.nfc().collect::<String>() != correction.printed {
            return Err(
                format!("{} contains non-NFC printed text", correction.correction_id).into(),
            );
        }
    }
    Ok(())
}

fn derive_evidence_correction_rows(
    corrections: &[EvidenceCorrection],
    rows: &mut DerivedRows,
) -> Result<(), Box<dyn Error>> {
    for correction in corrections {
        let role = EvidenceRole {
            role: "target-source-orthography".into(),
            source_id: correction.source_id.clone(),
            candidate_id: correction.candidate_id.clone(),
            passage: correction.source_passage.clone(),
        };
        insert_evidence(
            &mut rows.evidence,
            &correction.replacement_evidence_id,
            &role,
            &correction.review_note,
        )?;
    }
    Ok(())
}

fn load_reviews(path: &Path) -> Result<BTreeMap<String, Review>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(REVIEW_HEADER) {
        return Err(format!("invalid v0.7 exact-review header in {}", path.display()).into());
    }
    let mut reviews = BTreeMap::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!("invalid v0.7 exact-review row {}", offset + 2).into());
        }
        let decision = fields[1].to_owned();
        if !matches!(decision.as_str(), "admitted" | "deferred" | "rejected") {
            return Err(format!("invalid v0.7 decision {decision:?} at row {}", offset + 2).into());
        }
        if decision == "admitted" && !fields[3].is_empty() {
            return Err(format!("admitted v0.7 row {} has a blocker", offset + 2).into());
        }
        if decision != "admitted" && fields[3].is_empty() {
            return Err(format!("non-admitted v0.7 row {} has no blocker", offset + 2).into());
        }
        let review = Review {
            decision,
            realized_unique_tokens: fields[2].parse()?,
            blocker: fields[3].to_owned(),
            review_note: fields[4].to_owned(),
        };
        if reviews.insert(fields[0].to_owned(), review).is_some() {
            return Err(format!("duplicate v0.7 packet decision {:?}", fields[0]).into());
        }
    }
    Ok(reviews)
}

fn load_variant_reviews(path: &Path) -> Result<Vec<VariantReview>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(VARIANT_REVIEW_HEADER) {
        return Err(format!("invalid v0.7 variant-review header in {}", path.display()).into());
    }
    let mut review_ids = BTreeSet::new();
    let mut runtime_keys = BTreeSet::new();
    let mut reviews = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 18 {
            return Err(format!("invalid v0.7 variant-review row {}", offset + 2).into());
        }
        if !review_ids.insert(fields[0]) {
            return Err(format!("duplicate v0.7 variant review {:?}", fields[0]).into());
        }
        if !matches!(fields[1], "exact-form" | "abbreviation") {
            return Err(format!("invalid v0.7 variant lane at row {}", offset + 2).into());
        }
        if fields[16] != "admitted" || fields[17].trim().is_empty() {
            return Err(format!("incomplete v0.7 variant decision at row {}", offset + 2).into());
        }
        if fields[1] == "exact-form" && !fields[3].is_empty() {
            return Err(format!("exact-form variant has a sense ID at row {}", offset + 2).into());
        }
        if fields[1] == "abbreviation" && fields[3].is_empty() {
            return Err(format!(
                "abbreviation variant lacks a sense ID at row {}",
                offset + 2
            )
            .into());
        }
        let predicted_unique_tokens = fields[9].parse()?;
        if predicted_unique_tokens == 0 {
            return Err(format!("zero-return v0.7 variant at row {}", offset + 2).into());
        }
        let runtime_key = (fields[1], fields[2], fields[4], fields[5], fields[6]);
        if !runtime_keys.insert(runtime_key) {
            return Err(format!("duplicate v0.7 variant runtime row {}", offset + 2).into());
        }
        reviews.push(VariantReview {
            review_id: fields[0].into(),
            lane: fields[1].into(),
            lexeme_id: fields[2].into(),
            sense_id: fields[3].into(),
            cell: fields[4].into(),
            expanded: fields[5].into(),
            printed: fields[6].into(),
            base_printed: fields[7].into(),
            base_evidence_id: fields[8].into(),
            predicted_unique_tokens,
            source_id: fields[10].into(),
            source_candidate_id: fields[11].into(),
            source_passage: fields[12].into(),
            evaluation_source_id: fields[13].into(),
            evaluation_candidate_id: fields[14].into(),
            evaluation_passage: fields[15].into(),
            review_note: fields[17].into(),
        });
    }
    Ok(reviews)
}

fn load_abbreviation_reviews(path: &Path) -> Result<Vec<AbbreviationReview>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(ABBREVIATION_REVIEW_HEADER) {
        return Err(format!(
            "invalid v0.7 abbreviation-review header in {}",
            path.display()
        )
        .into());
    }
    let mut review_ids = BTreeSet::new();
    let mut runtime_keys = BTreeSet::new();
    let mut reviews = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 20 {
            return Err(format!("invalid v0.7 abbreviation-review row {}", offset + 2).into());
        }
        if !review_ids.insert(fields[0]) {
            return Err(format!("duplicate v0.7 abbreviation review {:?}", fields[0]).into());
        }
        if fields[18] != "admitted" || fields[19].trim().is_empty() {
            return Err(format!(
                "incomplete v0.7 abbreviation decision at row {}",
                offset + 2
            )
            .into());
        }
        if fields[1].is_empty()
            || fields[2].is_empty()
            || fields[3].is_empty()
            || fields[4].is_empty()
            || fields[5].is_empty()
            || fields[6].is_empty()
            || fields[7].is_empty()
            || fields[8].is_empty()
            || fields[16].is_empty()
            || fields[17].is_empty()
        {
            return Err(format!("incomplete v0.7 abbreviation row {}", offset + 2).into());
        }
        let predicted_unique_tokens = fields[9].parse()?;
        if predicted_unique_tokens == 0 {
            return Err(format!("zero-return v0.7 abbreviation at row {}", offset + 2).into());
        }
        let runtime_key = (fields[1], fields[2], fields[3], fields[4], fields[5]);
        if !runtime_keys.insert(runtime_key) {
            return Err(format!("duplicate v0.7 abbreviation runtime row {}", offset + 2).into());
        }
        reviews.push(AbbreviationReview {
            review_id: fields[0].into(),
            lexeme_id: fields[1].into(),
            sense_id: fields[2].into(),
            cell: fields[3].into(),
            expanded: fields[4].into(),
            printed: fields[5].into(),
            base_printed: fields[6].into(),
            base_evidence_id: fields[7].into(),
            normative_evidence_id: fields[8].into(),
            predicted_unique_tokens,
            source_id: fields[10].into(),
            source_candidate_id: fields[11].into(),
            source_passage: fields[12].into(),
            evaluation_source_id: fields[13].into(),
            evaluation_candidate_id: fields[14].into(),
            evaluation_passage: fields[15].into(),
            context_restrictions: fields[16].into(),
            ambiguity: fields[17].into(),
            review_note: fields[19].into(),
        });
    }
    Ok(reviews)
}

fn load_target_candidates(
    root: &Path,
) -> Result<BTreeMap<String, TargetCandidate>, Box<dyn Error>> {
    let mut candidates = BTreeMap::new();
    for relative in [PONOMAR, WIKISOURCE] {
        for (offset, line) in fs::read_to_string(root.join(relative))?.lines().enumerate() {
            let candidate: TargetCandidate = serde_json::from_str(line).map_err(|error| {
                format!(
                    "invalid target candidate at {relative}:{}: {error}",
                    offset + 1
                )
            })?;
            let candidate_id = candidate.candidate_id.clone();
            if candidates.insert(candidate_id.clone(), candidate).is_some() {
                return Err(format!("duplicate target candidate ID {candidate_id}").into());
            }
        }
    }
    Ok(candidates)
}

fn validate_variant_reviews(
    root: &Path,
    reviews: &[VariantReview],
    candidates: &BTreeMap<String, TargetCandidate>,
) -> Result<(), Box<dyn Error>> {
    let exact = fs::read_to_string(root.join("data/synodal/exact_forms.tsv"))?;
    let abbreviation = fs::read_to_string(root.join("data/synodal/abbreviations.tsv"))?;
    for review in reviews {
        if review.printed == review.base_printed
            || accent_case_skeleton(&review.printed) != accent_case_skeleton(&review.base_printed)
        {
            return Err(format!(
                "{} is not a bounded accent/case variant of its base row",
                review.review_id
            )
            .into());
        }
        if review.expanded.nfc().collect::<String>() != review.expanded
            || review.printed.nfc().collect::<String>() != review.printed
            || review.base_printed.nfc().collect::<String>() != review.base_printed
        {
            return Err(format!("{} contains non-NFC registry text", review.review_id).into());
        }
        let source = validate_variant_candidate(
            review,
            candidates,
            &review.source_candidate_id,
            &review.source_id,
            &review.source_passage,
            "source",
        )?;
        let evaluation = validate_variant_candidate(
            review,
            candidates,
            &review.evaluation_candidate_id,
            &review.evaluation_source_id,
            &review.evaluation_passage,
            "evaluation",
        )?;
        if source.passage == evaluation.passage {
            return Err(format!(
                "{} reuses its source passage for evaluation",
                review.review_id
            )
            .into());
        }
        if !contains_whole_token(&source.normalized_spelling, &review.printed)
            || !contains_whole_token(&evaluation.normalized_spelling, &review.printed)
        {
            return Err(format!("{} lacks an exact whole-token witness", review.review_id).into());
        }
        let base_found = if review.lane == "exact-form" {
            table_has_base_row(
                &exact,
                EXACT_HEADER,
                review,
                &[0, 1, 2, 3, 4],
                &[
                    &review.lexeme_id,
                    &review.cell,
                    &review.expanded,
                    &review.base_printed,
                    &review.base_evidence_id,
                ],
            )?
        } else {
            table_has_base_row(
                &abbreviation,
                ABBREVIATION_HEADER,
                review,
                &[0, 1, 2, 3, 4, 6],
                &[
                    &review.lexeme_id,
                    &review.sense_id,
                    &review.cell,
                    &review.expanded,
                    &review.base_printed,
                    &review.base_evidence_id,
                ],
            )?
        };
        if !base_found {
            return Err(format!("{} has no exact reviewed base row", review.review_id).into());
        }
    }
    Ok(())
}

fn validate_abbreviation_reviews(
    root: &Path,
    reviews: &[AbbreviationReview],
    candidates: &BTreeMap<String, TargetCandidate>,
) -> Result<(), Box<dyn Error>> {
    let exact = fs::read_to_string(root.join("data/synodal/exact_forms.tsv"))?;
    let evidence = fs::read_to_string(root.join("data/synodal/reviewed_evidence.tsv"))?;
    let senses = fs::read_to_string(root.join("data/synodal/senses.tsv"))?;
    let lexical = fs::read_to_string(root.join("data/synodal/lexical_reviews.tsv"))?;
    for review in reviews {
        for value in [
            &review.expanded,
            &review.printed,
            &review.base_printed,
            &review.context_restrictions,
            &review.ambiguity,
        ] {
            if value.nfc().collect::<String>() != *value {
                return Err(format!("{} contains non-NFC registry text", review.review_id).into());
            }
        }
        if !review.printed.chars().any(|character| {
            character == '\u{483}' || ('\u{2de0}'..='\u{2dff}').contains(&character)
        }) {
            return Err(format!("{} has no titlo or letter-titlo", review.review_id).into());
        }
        find_base_fields(
            &exact,
            EXACT_HEADER,
            &[0, 1, 2, 3, 4],
            &[
                &review.lexeme_id,
                &review.cell,
                &review.expanded,
                &review.base_printed,
                &review.base_evidence_id,
            ],
        )?;
        if !has_reviewed_sense(&senses, &lexical, &review.lexeme_id, &review.sense_id) {
            return Err(format!("{} has no reviewed lexeme-sense pair", review.review_id).into());
        }
        if !evidence.lines().skip(1).any(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            fields.first() == Some(&review.normative_evidence_id.as_str())
                && fields.get(4) == Some(&"reviewed")
                && fields.get(5) == Some(&"synodal-russian")
        }) {
            return Err(format!(
                "{} lacks reviewed normative abbreviation evidence",
                review.review_id
            )
            .into());
        }
        let source = validate_abbreviation_candidate(
            review,
            candidates,
            &review.source_candidate_id,
            &review.source_id,
            &review.source_passage,
            "source",
        )?;
        let evaluation = validate_abbreviation_candidate(
            review,
            candidates,
            &review.evaluation_candidate_id,
            &review.evaluation_source_id,
            &review.evaluation_passage,
            "evaluation",
        )?;
        if source.passage == evaluation.passage {
            return Err(format!(
                "{} reuses its source passage for evaluation",
                review.review_id
            )
            .into());
        }
        if !contains_whole_token(&source.normalized_spelling, &review.printed)
            || !contains_whole_token(&evaluation.normalized_spelling, &review.printed)
        {
            return Err(format!("{} lacks an exact whole-token witness", review.review_id).into());
        }
    }
    Ok(())
}

fn has_reviewed_sense(senses: &str, lexical: &str, lexeme_id: &str, sense_id: &str) -> bool {
    senses.lines().skip(1).any(|line| {
        let fields: Vec<_> = line.split('\t').collect();
        fields.first() == Some(&lexeme_id) && fields.get(1) == Some(&sense_id)
    }) || lexical.lines().skip(1).any(|line| {
        let fields: Vec<_> = line.split('\t').collect();
        fields.get(1) == Some(&lexeme_id)
            && fields.get(2) == Some(&sense_id)
            && fields.get(15) == Some(&"reviewed")
    })
}

fn validate_abbreviation_candidate<'a>(
    review: &AbbreviationReview,
    candidates: &'a BTreeMap<String, TargetCandidate>,
    candidate_id: &str,
    source_id: &str,
    passage: &str,
    partition: &str,
) -> Result<&'a TargetCandidate, Box<dyn Error>> {
    let candidate = candidates.get(candidate_id).ok_or_else(|| {
        format!(
            "{} cites unknown abbreviation candidate {candidate_id}",
            review.review_id
        )
    })?;
    if candidate.source_id != source_id
        || candidate.passage != passage
        || candidate.partition != partition
    {
        return Err(format!(
            "{} has mismatched abbreviation {partition} provenance",
            review.review_id
        )
        .into());
    }
    Ok(candidate)
}

fn validate_variant_candidate<'a>(
    review: &VariantReview,
    candidates: &'a BTreeMap<String, TargetCandidate>,
    candidate_id: &str,
    source_id: &str,
    passage: &str,
    partition: &str,
) -> Result<&'a TargetCandidate, Box<dyn Error>> {
    let candidate = candidates.get(candidate_id).ok_or_else(|| {
        format!(
            "{} cites unknown candidate {candidate_id}",
            review.review_id
        )
    })?;
    if candidate.source_id != source_id
        || candidate.passage != passage
        || candidate.partition != partition
    {
        return Err(format!("{} has mismatched {partition} provenance", review.review_id).into());
    }
    Ok(candidate)
}

fn table_has_base_row(
    text: &str,
    header: &str,
    review: &VariantReview,
    indexes: &[usize],
    expected: &[&String],
) -> Result<bool, Box<dyn Error>> {
    let mut lines = text.lines();
    if lines.next() != Some(header) {
        return Err(format!("invalid base table for {}", review.review_id).into());
    }
    Ok(lines.filter(|line| !line.is_empty()).any(|line| {
        let fields: Vec<_> = line.split('\t').collect();
        indexes.iter().zip(expected).all(|(index, value)| {
            fields
                .get(*index)
                .is_some_and(|field| *field == value.as_str())
        })
    }))
}

fn accent_case_skeleton(value: &str) -> String {
    value
        .nfd()
        .filter(|character| !matches!(*character, '\u{300}' | '\u{301}' | '\u{311}'))
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .nfc()
        .collect()
}

fn contains_whole_token(text: &str, token: &str) -> bool {
    text.match_indices(token).any(|(start, matched)| {
        let end = start + matched.len();
        let previous = text[..start].chars().next_back();
        let next = text[end..].chars().next();
        !previous.is_some_and(is_token_character) && !next.is_some_and(is_token_character)
    })
}

fn is_token_character(character: char) -> bool {
    character.is_alphabetic() || is_combining_mark(character)
}

fn validate_review_coverage(
    report: &PacketReport,
    reviews: &BTreeMap<String, Review>,
) -> Result<(), Box<dyn Error>> {
    let packet_ids: BTreeSet<_> = report
        .packets
        .iter()
        .map(|packet| packet.packet_id.as_str())
        .collect();
    if packet_ids.len() != report.packets.len() {
        return Err("v0.7 packet report contains duplicate packet IDs".into());
    }
    let review_ids: BTreeSet<_> = reviews.keys().map(String::as_str).collect();
    if packet_ids != review_ids {
        let missing = packet_ids
            .difference(&review_ids)
            .take(5)
            .collect::<Vec<_>>();
        let stale = review_ids
            .difference(&packet_ids)
            .take(5)
            .collect::<Vec<_>>();
        return Err(format!(
            "v0.7 decisions do not cover the current packets; missing={missing:?}, stale={stale:?}"
        )
        .into());
    }
    for (packet_id, review) in reviews {
        if review.decision == "admitted" && review.realized_unique_tokens != 0 {
            return Err(format!(
                "{packet_id} records realized gain before canonical recomputation"
            )
            .into());
        }
        if review.review_note.trim().is_empty() {
            return Err(format!("{packet_id} has no reviewer rationale").into());
        }
        if review.decision != "admitted" && review.blocker.trim().is_empty() {
            return Err(format!("{packet_id} has no exact blocker").into());
        }
    }
    Ok(())
}

fn derive_rows(
    report: &PacketReport,
    reviews: &BTreeMap<String, Review>,
) -> Result<DerivedRows, Box<dyn Error>> {
    let mut rows = DerivedRows::default();
    for packet in &report.packets {
        if reviews[&packet.packet_id].decision != "admitted" {
            continue;
        }
        let morphology = packet
            .evidence_roles
            .iter()
            .find(|role| {
                matches!(
                    role.role.as_str(),
                    "source-typed-morphology" | "source-lexical-identity"
                )
            })
            .ok_or_else(|| format!("{} has no source morphology role", packet.packet_id))?;
        let target = role(packet, "target-source-orthography")?;
        let evaluation = role(packet, "held-out-target-evaluation")?;
        let semantic = packet
            .evidence_roles
            .iter()
            .find(|role| role.role == "semantic-identity");

        let morphology_id = stable_id("v07-source", &[&morphology.candidate_id, &packet.cell]);
        let target_id = stable_id("v07-target", &[&target.candidate_id, &packet.surface]);
        insert_evidence(
            &mut rows.evidence,
            &morphology_id,
            morphology,
            "Reviewer confirmed that the governed source supports this exact surface and typed cell; no sibling cell or productive rule is implied.",
        )?;
        insert_evidence(
            &mut rows.evidence,
            &target_id,
            target,
            "Reviewer confirmed the exact NFC whole-token Synodal source occurrence and printed orthography; the held-out passage is separate.",
        )?;

        let mut evidence_ids = Vec::new();
        if packet.identity_status == "new-source-semantic-identity" {
            let semantic = semantic.ok_or_else(|| {
                format!(
                    "{} introduces an identity without semantic evidence",
                    packet.packet_id
                )
            })?;
            let suffix = stable_hex(&[&packet.lexeme_id])[..16].to_owned();
            let review_id = format!("review:v07:{suffix}");
            let sense_id = format!("sense:v07:{suffix}");
            let reviewed_lemma = if packet
                .lemma
                .chars()
                .filter(|character| character.is_alphabetic())
                .all(|character| !matches!(character, '\u{2c00}'..='\u{2cff}'))
            {
                packet.lemma.as_str()
            } else {
                packet.normalized_surface.as_str()
            };
            let lexical = [
                review_id.as_str(),
                packet.lexeme_id.as_str(),
                sense_id.as_str(),
                reviewed_lemma,
                packet.part_of_speech.as_str(),
                if matches!(packet.part_of_speech.as_str(), "adverb" | "preposition") {
                    "indeclinable"
                } else {
                    "lexical-form"
                },
                packet.normalized_surface.as_str(),
                packet.surface.as_str(),
                packet.semantic_gloss.as_str(),
                if packet.part_of_speech == "proper-noun" {
                    "person,name"
                } else {
                    "general"
                },
                semantic.source_id.as_str(),
                semantic.candidate_id.as_str(),
                target.source_id.as_str(),
                target.candidate_id.as_str(),
                target.passage.as_str(),
                "reviewed",
                "synodal-russian",
                "Reviewer confirmed the source-partition lexical and semantic identity, the independent exact Synodal whole-token witness, and the bounded historical spelling continuity. Only the separately listed exact cells are admitted.",
            ]
            .join("\t");
            rows.lexical
                .entry(packet.lexeme_id.clone())
                .or_insert(lexical);
            evidence_ids.push(review_id);
        }
        evidence_ids.push(morphology_id);
        evidence_ids.push(target_id);
        rows.exact.insert(
            [
                packet.lexeme_id.as_str(),
                packet.cell.as_str(),
                packet.normalized_surface.as_str(),
                packet.surface.as_str(),
                evidence_ids.join(",").as_str(),
                "synodal-attestation",
                "synodal-russian",
            ]
            .join("\t"),
        );
        rows.evaluation.insert(
            [
                format!("eval:v07:{}", &stable_hex(&[&packet.packet_id])[..16]).as_str(),
                packet.lexeme_id.as_str(),
                packet.cell.as_str(),
                "strict",
                packet.normalized_surface.as_str(),
                packet.surface.as_str(),
                evaluation.source_id.as_str(),
                evaluation.passage.as_str(),
                "v07-held-out-exact-cell",
            ]
            .join("\t"),
        );
    }
    Ok(rows)
}

fn derive_variant_rows(
    root: &Path,
    reviews: &[VariantReview],
    rows: &mut DerivedRows,
) -> Result<(), Box<dyn Error>> {
    let exact_text = fs::read_to_string(root.join("data/synodal/exact_forms.tsv"))?;
    let abbreviation_text = fs::read_to_string(root.join("data/synodal/abbreviations.tsv"))?;
    for review in reviews {
        let target_id = stable_id(
            "v07-variant-target",
            &[&review.source_candidate_id, &review.printed],
        );
        let target_role = EvidenceRole {
            role: "target-source-orthography".into(),
            source_id: review.source_id.clone(),
            candidate_id: review.source_candidate_id.clone(),
            passage: review.source_passage.clone(),
        };
        insert_evidence(
            &mut rows.evidence,
            &target_id,
            &target_role,
            &review.review_note,
        )?;
        let evidence = format!("{},{}", review.base_evidence_id, target_id);
        if review.lane == "exact-form" {
            let base = find_base_fields(
                &exact_text,
                EXACT_HEADER,
                &[0, 1, 2, 3, 4],
                &[
                    &review.lexeme_id,
                    &review.cell,
                    &review.expanded,
                    &review.base_printed,
                    &review.base_evidence_id,
                ],
            )?;
            rows.exact.insert(
                [
                    review.lexeme_id.as_str(),
                    review.cell.as_str(),
                    review.expanded.as_str(),
                    review.printed.as_str(),
                    evidence.as_str(),
                    base[5],
                    base[6],
                ]
                .join("\t"),
            );
            rows.evaluation.insert(
                [
                    stable_id("eval:v07:variant", &[&review.review_id]).as_str(),
                    review.lexeme_id.as_str(),
                    review.cell.as_str(),
                    "strict",
                    review.expanded.as_str(),
                    review.printed.as_str(),
                    review.evaluation_source_id.as_str(),
                    review.evaluation_passage.as_str(),
                    "v07-held-out-explicit-accent-case-variant",
                ]
                .join("\t"),
            );
        } else {
            let base = find_base_fields(
                &abbreviation_text,
                ABBREVIATION_HEADER,
                &[0, 1, 2, 3, 4, 6],
                &[
                    &review.lexeme_id,
                    &review.sense_id,
                    &review.cell,
                    &review.expanded,
                    &review.base_printed,
                    &review.base_evidence_id,
                ],
            )?;
            let rule_id = format!(
                "SYN-ABBR-V07-VARIANT-{}",
                &stable_hex(&[&review.review_id])[..16]
            );
            rows.abbreviation.insert(
                [
                    review.lexeme_id.as_str(),
                    review.sense_id.as_str(),
                    review.cell.as_str(),
                    review.expanded.as_str(),
                    review.printed.as_str(),
                    rule_id.as_str(),
                    evidence.as_str(),
                    base[7],
                    base[8],
                    base[9],
                    base[10],
                    base[11],
                    base[12],
                ]
                .join("\t"),
            );
            rows.abbreviation_evaluation.insert(
                [
                    stable_id("eval:v07:abbr-variant", &[&review.review_id]).as_str(),
                    review.lexeme_id.as_str(),
                    review.sense_id.as_str(),
                    review.cell.as_str(),
                    review.expanded.as_str(),
                    review.printed.as_str(),
                    review.evaluation_source_id.as_str(),
                    review.evaluation_passage.as_str(),
                    "typed-abbreviation-held-out-explicit-capitalization",
                ]
                .join("\t"),
            );
        }
    }
    Ok(())
}

fn derive_abbreviation_rows(
    _root: &Path,
    reviews: &[AbbreviationReview],
    rows: &mut DerivedRows,
) -> Result<(), Box<dyn Error>> {
    for review in reviews {
        let target_id = stable_id(
            "v07-abbreviation-target",
            &[&review.source_candidate_id, &review.printed],
        );
        let target_role = EvidenceRole {
            role: "target-source-orthography".into(),
            source_id: review.source_id.clone(),
            candidate_id: review.source_candidate_id.clone(),
            passage: review.source_passage.clone(),
        };
        insert_evidence(
            &mut rows.evidence,
            &target_id,
            &target_role,
            &review.review_note,
        )?;
        let evidence = format!(
            "{},{},{}",
            review.base_evidence_id, review.normative_evidence_id, target_id
        );
        let rule_id = format!(
            "SYN-ABBR-V07-TYPED-{}",
            &stable_hex(&[&review.review_id])[..16]
        );
        rows.abbreviation.insert(
            [
                review.lexeme_id.as_str(),
                review.sense_id.as_str(),
                review.cell.as_str(),
                review.expanded.as_str(),
                review.printed.as_str(),
                rule_id.as_str(),
                evidence.as_str(),
                "false",
                "titlo",
                review.context_restrictions.as_str(),
                review.ambiguity.as_str(),
                "synodal-russian",
                "synodal-russian",
            ]
            .join("\t"),
        );
        rows.abbreviation_evaluation.insert(
            [
                stable_id("eval:v07:typed-abbreviation", &[&review.review_id]).as_str(),
                review.lexeme_id.as_str(),
                review.sense_id.as_str(),
                review.cell.as_str(),
                review.expanded.as_str(),
                review.printed.as_str(),
                review.evaluation_source_id.as_str(),
                review.evaluation_passage.as_str(),
                "typed-abbreviation-held-out-exact-cell",
            ]
            .join("\t"),
        );
    }
    Ok(())
}

fn find_base_fields<'a>(
    text: &'a str,
    header: &str,
    indexes: &[usize],
    expected: &[&String],
) -> Result<Vec<&'a str>, Box<dyn Error>> {
    let mut lines = text.lines();
    if lines.next() != Some(header) {
        return Err("invalid base registry header".into());
    }
    lines
        .filter(|line| !line.is_empty())
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .find(|fields| {
            indexes.iter().zip(expected).all(|(index, value)| {
                fields
                    .get(*index)
                    .is_some_and(|field| field == &value.as_str())
            })
        })
        .ok_or_else(|| "reviewed variant base row disappeared during derivation".into())
}

fn role<'a>(packet: &'a Packet, name: &str) -> Result<&'a EvidenceRole, Box<dyn Error>> {
    packet
        .evidence_roles
        .iter()
        .find(|role| role.role == name)
        .ok_or_else(|| format!("{} has no {name} role", packet.packet_id).into())
}

fn insert_evidence(
    rows: &mut BTreeMap<String, String>,
    id: &str,
    role: &EvidenceRole,
    note: &str,
) -> Result<(), Box<dyn Error>> {
    let row = [
        id,
        role.candidate_id.as_str(),
        role.source_id.as_str(),
        role.passage.as_str(),
        "reviewed",
        "synodal-russian",
        note,
    ]
    .join("\t");
    if rows
        .insert(id.to_owned(), row.clone())
        .is_some_and(|previous| previous != row)
    {
        return Err(format!("conflicting derived evidence ID {id}").into());
    }
    Ok(())
}

fn stable_id(prefix: &str, values: &[&str]) -> String {
    format!("{prefix}-{}", &stable_hex(values)[..16])
}

fn stable_hex(values: &[&str]) -> String {
    let mut digest = Sha256::new();
    for value in values {
        digest.update(value.as_bytes());
        digest.update([0]);
    }
    format!("{:x}", digest.finalize())
}

fn synchronized_table(
    path: &Path,
    header: &str,
    derived: &[String],
    corrections: &[IdentityCorrection],
    evidence_corrections: &[EvidenceCorrection],
) -> Result<String, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(header) {
        return Err(format!("invalid header in {}", path.display()).into());
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for line in lines.filter(|line| !line.is_empty()) {
        if let Some(corrected) =
            correct_reviewed_row(header, line, corrections, evidence_corrections)?
        {
            if seen.insert(corrected.clone()) {
                rows.push(corrected);
            }
        }
    }
    for row in derived {
        if let Some(corrected) =
            correct_reviewed_row(header, row, corrections, evidence_corrections)?
            && seen.insert(corrected.clone())
        {
            rows.push(corrected);
        }
    }
    let mut output = String::from(header);
    output.push('\n');
    for row in rows {
        output.push_str(&row);
        output.push('\n');
    }
    Ok(output)
}

fn correct_reviewed_row(
    header: &str,
    row: &str,
    corrections: &[IdentityCorrection],
    evidence_corrections: &[EvidenceCorrection],
) -> Result<Option<String>, Box<dyn Error>> {
    let mut fields = row.split('\t').map(str::to_owned).collect::<Vec<_>>();
    for correction in corrections {
        if header == LEXICAL_HEADER
            && (fields.first() == Some(&correction.obsolete_review_id)
                || fields.get(1) == Some(&correction.obsolete_lexeme_id)
                || fields.get(2) == Some(&correction.obsolete_sense_id))
        {
            return Ok(None);
        }
        if matches!(header, EXACT_HEADER | EVALUATION_HEADER)
            && fields.get(if header == EXACT_HEADER { 0 } else { 1 })
                == Some(&correction.obsolete_lexeme_id)
        {
            let index = if header == EXACT_HEADER { 0 } else { 1 };
            fields[index].clone_from(&correction.canonical_lexeme_id);
            if header == EXACT_HEADER {
                let evidence = fields
                    .get_mut(4)
                    .ok_or("invalid exact-form row during identity correction")?;
                *evidence = evidence
                    .split(',')
                    .map(|id| {
                        if id == correction.obsolete_review_id {
                            correction.canonical_review_id.as_str()
                        } else {
                            id
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",");
            }
        }
        if matches!(header, ABBREVIATION_HEADER | ABBREVIATION_EVALUATION_HEADER)
            && fields.first() == Some(&correction.obsolete_lexeme_id)
        {
            fields[0].clone_from(&correction.canonical_lexeme_id);
            if fields.get(1) == Some(&correction.obsolete_sense_id) {
                fields[1].clone_from(&correction.canonical_sense_id);
            }
        }
    }
    for correction in evidence_corrections {
        if header == EVIDENCE_HEADER && fields.first() == Some(&correction.obsolete_evidence_id) {
            return Ok(None);
        }
        let evidence_index = if header == EXACT_HEADER {
            Some(4)
        } else if header == ABBREVIATION_HEADER {
            Some(6)
        } else {
            None
        };
        if let Some(index) = evidence_index
            && let Some(evidence) = fields.get_mut(index)
        {
            *evidence = evidence
                .split(',')
                .map(|id| {
                    if id == correction.obsolete_evidence_id {
                        correction.replacement_evidence_id.as_str()
                    } else {
                        id
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
        }
    }
    Ok(Some(fields.join("\t")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_ids_are_order_sensitive_and_repeatable() {
        assert_eq!(stable_id("v07", &["a", "b"]), stable_id("v07", &["a", "b"]));
        assert_ne!(stable_id("v07", &["a", "b"]), stable_id("v07", &["b", "a"]));
    }

    #[test]
    fn identity_correction_merges_exact_rows_and_drops_duplicate_lexical_review() {
        let correction = IdentityCorrection {
            correction_id: "correction".into(),
            obsolete_lexeme_id: "old-lexeme".into(),
            obsolete_sense_id: "old-sense".into(),
            obsolete_review_id: "old-review".into(),
            canonical_lexeme_id: "canonical-lexeme".into(),
            canonical_sense_id: "canonical-sense".into(),
            canonical_review_id: "canonical-review".into(),
            semantic_candidate_id: "semantic".into(),
            review_note: "reviewed duplicate".into(),
        };
        let exact =
            "old-lexeme\tcell\texpanded\tprinted\told-review,source,target\tkind\trecension";
        assert_eq!(
            correct_reviewed_row(EXACT_HEADER, exact, std::slice::from_ref(&correction), &[])
                .expect("valid exact identity correction"),
            Some("canonical-lexeme\tcell\texpanded\tprinted\tcanonical-review,source,target\tkind\trecension".into())
        );
        let lexical = "old-review\told-lexeme\told-sense";
        assert_eq!(
            correct_reviewed_row(LEXICAL_HEADER, lexical, &[correction], &[])
                .expect("valid lexical identity correction"),
            None
        );
    }
}
