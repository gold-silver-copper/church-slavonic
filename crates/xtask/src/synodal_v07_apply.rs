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
const REVIEW_HISTORY: [&str; 10] = [
    "data/synodal/v07_exact_reviews.tsv",
    "data/synodal/v07_exact_reviews_wave2.tsv",
    "data/synodal/v07_exact_reviews_wave3.tsv",
    "data/synodal/v07_exact_reviews_wave4.tsv",
    "data/synodal/v07_exact_reviews_wave5.tsv",
    "data/synodal/v07_exact_reviews_wave6.tsv",
    "data/synodal/v07_exact_reviews_wave7.tsv",
    "data/synodal/v07_exact_reviews_wave8.tsv",
    "data/synodal/v07_exact_reviews_wave9.tsv",
    "data/synodal/v07_exact_reviews_wave10.tsv",
];
const VARIANT_REVIEWS: &str = "data/synodal/v07_variant_reviews.tsv";
const ABBREVIATION_REVIEWS: &str = "data/synodal/v07_abbreviation_reviews.tsv";
const IDENTITY_CORRECTIONS: &str = "data/synodal/v07_identity_corrections.tsv";
const EVIDENCE_CORRECTIONS: &str = "data/synodal/v07_evidence_corrections.tsv";
const PACKET_OWNERSHIP: &str = "data/synodal/v07_packet_ownership.tsv";
const PACKET_EVIDENCE_OWNERSHIP: &str = "data/synodal/v07_packet_evidence_ownership.tsv";
const PACKET_LEXICAL_OWNERSHIP: &str = "data/synodal/v07_packet_lexical_ownership.tsv";
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
const PACKET_OWNERSHIP_HEADER: &str = "owner_id\tlexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension\tevaluation_id\tpolicy\tevaluation_expanded\tevaluation_printed\tevaluation_source_id\tevaluation_passage\tevaluation_regularity\tsource_candidate_id";
const MANUAL_PACKET_OWNERS: [&str; 1] = ["eval:identity-correction:on-acc-dual-masculine"];
const HISTORICAL_PACKET_OWNER_COUNT: usize = 1_059;
const HISTORICAL_PACKET_OWNER_DIGEST: &str =
    "833758d269ddc3a8e18adf5c742eb9c73453571e810d847fac578c54521561f0";

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

#[derive(Clone, Debug)]
struct PacketOwnership {
    fields: Vec<String>,
}

impl PacketOwnership {
    fn owner_id(&self) -> &str {
        &self.fields[0]
    }

    fn exact_row(&self) -> String {
        self.fields[1..8].join("\t")
    }

    fn evaluation_row(&self) -> String {
        [
            self.fields[8].as_str(),
            self.fields[1].as_str(),
            self.fields[2].as_str(),
            self.fields[9].as_str(),
            self.fields[10].as_str(),
            self.fields[11].as_str(),
            self.fields[12].as_str(),
            self.fields[13].as_str(),
            self.fields[14].as_str(),
        ]
        .join("\t")
    }

    fn is_active(&self, latest_reviews: &BTreeMap<String, Review>) -> bool {
        !self.owner_id().starts_with("v07-exact-")
            || latest_reviews
                .get(self.owner_id())
                .is_some_and(|review| review.decision == "admitted")
    }
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
    let mut refresh_ownership = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            "--refresh-ownership" => refresh_ownership = true,
            value => return Err(format!("unknown synodal-v07-apply argument {value:?}").into()),
        }
    }

    let report: PacketReport = serde_json::from_slice(&fs::read(root.join(PACKETS))?)?;
    let reviews = load_reviews(&root.join(REVIEWS))?;
    validate_review_coverage(&report, &reviews)?;
    let latest_reviews = load_latest_reviews(root)?;
    let variant_reviews = load_variant_reviews(&root.join(VARIANT_REVIEWS))?;
    let abbreviation_reviews = load_abbreviation_reviews(&root.join(ABBREVIATION_REVIEWS))?;
    let identity_corrections = load_identity_corrections(&root.join(IDENTITY_CORRECTIONS))?;
    let evidence_corrections = load_evidence_corrections(&root.join(EVIDENCE_CORRECTIONS))?;
    let initial_derived = derive_rows(&report, &reviews)?;
    let packet_evidence_ownership =
        load_owned_rows(&root.join(PACKET_EVIDENCE_OWNERSHIP), EVIDENCE_HEADER)?;
    let packet_lexical_ownership =
        load_owned_rows(&root.join(PACKET_LEXICAL_OWNERSHIP), LEXICAL_HEADER)?;
    if refresh_ownership {
        if check {
            return Err("--refresh-ownership and --check are mutually exclusive".into());
        }
        let ownership = render_packet_ownership(
            root,
            &latest_reviews,
            &initial_derived,
            &identity_corrections,
            &evidence_corrections,
        )?;
        fs::write(root.join(PACKET_OWNERSHIP), ownership)?;
        fs::write(
            root.join(PACKET_EVIDENCE_OWNERSHIP),
            merge_owned_rows(
                EVIDENCE_HEADER,
                &packet_evidence_ownership,
                initial_derived.evidence.values().cloned(),
            )?,
        )?;
        fs::write(
            root.join(PACKET_LEXICAL_OWNERSHIP),
            merge_owned_rows(
                LEXICAL_HEADER,
                &packet_lexical_ownership,
                initial_derived.lexical.values().cloned(),
            )?,
        )?;
        println!("refreshed durable v0.7 packet ownership");
        return Ok(());
    }
    let packet_ownership = load_packet_ownership(&root.join(PACKET_OWNERSHIP))?;
    let candidates = load_target_candidates(root)?;
    validate_packet_ownership(
        root,
        &packet_ownership,
        &latest_reviews,
        &identity_corrections,
        &packet_evidence_ownership,
        &packet_lexical_ownership,
        &candidates,
    )?;
    validate_identity_corrections(root, &identity_corrections)?;
    validate_evidence_corrections(root, &evidence_corrections, &candidates)?;
    validate_variant_reviews(root, &variant_reviews, &candidates)?;
    validate_abbreviation_reviews(root, &abbreviation_reviews, &candidates)?;
    let mut derived = initial_derived;
    derive_evidence_correction_rows(&evidence_corrections, &mut derived)?;
    activate_owned_dependencies(
        &packet_ownership,
        &latest_reviews,
        &packet_evidence_ownership,
        &packet_lexical_ownership,
        &mut derived,
    )?;
    derive_variant_rows(root, &variant_reviews, &mut derived)?;
    derive_abbreviation_rows(root, &abbreviation_reviews, &mut derived)?;
    derived.evaluation.extend(
        packet_ownership
            .iter()
            .filter(|row| row.is_active(&latest_reviews))
            .map(PacketOwnership::evaluation_row),
    );
    derived.exact.extend(
        packet_ownership
            .iter()
            .filter(|row| row.is_active(&latest_reviews))
            .map(PacketOwnership::exact_row),
    );
    let evaluation_base = filter_table(
        &fs::read_to_string(root.join("data/synodal/evaluation.tsv"))?,
        EVALUATION_HEADER,
        |fields| !is_packet_evaluation_id(fields[0]),
    )?;
    let evaluation = synchronized_table_text(
        &root.join("data/synodal/evaluation.tsv"),
        &evaluation_base,
        EVALUATION_HEADER,
        &derived.evaluation.into_iter().collect::<Vec<_>>(),
        &identity_corrections,
        &evidence_corrections,
    )?;
    let exact_base = filter_table(
        &fs::read_to_string(root.join("data/synodal/exact_forms.tsv"))?,
        EXACT_HEADER,
        |fields| !is_packet_exact_row(fields),
    )?;
    let exact = synchronized_table_text(
        &root.join("data/synodal/exact_forms.tsv"),
        &exact_base,
        EXACT_HEADER,
        &derived.exact.into_iter().collect::<Vec<_>>(),
        &identity_corrections,
        &evidence_corrections,
    )?;
    let exact = merge_duplicate_exact_rows(&exact)?;
    let abbreviation = synchronized_table(
        &root.join("data/synodal/abbreviations.tsv"),
        ABBREVIATION_HEADER,
        &derived.abbreviation.into_iter().collect::<Vec<_>>(),
        &identity_corrections,
        &evidence_corrections,
    )?;
    let referenced_evidence = referenced_evidence_ids(&exact, &abbreviation)?;
    let lexical = synchronized_table(
        &root.join("data/synodal/lexical_reviews.tsv"),
        LEXICAL_HEADER,
        &derived.lexical.into_values().collect::<Vec<_>>(),
        &identity_corrections,
        &evidence_corrections,
    )?;
    let lexical = retain_referenced_v07_rows(&lexical, LEXICAL_HEADER, &referenced_evidence)?;
    let evidence = synchronized_table(
        &root.join("data/synodal/reviewed_evidence.tsv"),
        EVIDENCE_HEADER,
        &derived.evidence.into_values().collect::<Vec<_>>(),
        &identity_corrections,
        &evidence_corrections,
    )?;
    let evidence = retain_referenced_v07_rows(&evidence, EVIDENCE_HEADER, &referenced_evidence)?;
    let abbreviation_evaluation = synchronized_table(
        &root.join("data/synodal/abbreviation_evaluation.tsv"),
        ABBREVIATION_EVALUATION_HEADER,
        &derived
            .abbreviation_evaluation
            .into_iter()
            .collect::<Vec<_>>(),
        &identity_corrections,
        &evidence_corrections,
    )?;
    let outputs = [
        ("data/synodal/exact_forms.tsv", exact),
        ("data/synodal/abbreviations.tsv", abbreviation),
        ("data/synodal/lexical_reviews.tsv", lexical),
        ("data/synodal/reviewed_evidence.tsv", evidence),
        ("data/synodal/evaluation.tsv", evaluation),
        (
            "data/synodal/abbreviation_evaluation.tsv",
            abbreviation_evaluation,
        ),
    ];

    for (relative, desired) in outputs {
        let path = root.join(relative);
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
    let lexeme_text = fs::read_to_string(root.join("data/synodal/lexemes.tsv"))?;
    let lexemes = table_rows(
        &lexeme_text,
        "id\tlemma\tpart_of_speech\tclass\tstem\tgender\taspect\tsource_id\ttarget_recension",
    )?;
    let sense_text = fs::read_to_string(root.join("data/synodal/senses.tsv"))?;
    let senses = table_rows(
        &sense_text,
        "lexeme_id\tsense_id\tgloss\tdomains\tsource_id\tsource_recension\tsemantic_status",
    )?;
    let evidence_text = fs::read_to_string(root.join("data/synodal/reviewed_evidence.tsv"))?;
    let reviewed_evidence = table_rows(&evidence_text, EVIDENCE_HEADER)?;
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
        let canonical_base_lexeme = lexemes
            .iter()
            .find(|fields| fields.first() == Some(&correction.canonical_lexeme_id.as_str()));
        let canonical_base_sense = senses.iter().find(|fields| {
            fields.first() == Some(&correction.canonical_lexeme_id.as_str())
                && fields.get(1) == Some(&correction.canonical_sense_id.as_str())
        });
        let canonical_base_evidence = reviewed_evidence.iter().any(|fields| {
            fields.first() == Some(&correction.canonical_review_id.as_str())
                && fields.get(4) == Some(&"reviewed")
        });
        let invalid_obsolete = obsolete.is_some_and(|obsolete| {
            let matches_review_overlay = canonical.is_some_and(|canonical| {
                canonical.get(11) == Some(&correction.semantic_candidate_id.as_str())
                    && canonical.get(15) == Some(&"reviewed")
                    && obsolete.get(4) == canonical.get(4)
                    && reviewed_glosses_overlap(obsolete.get(8), canonical.get(8))
            });
            let matches_base_identity = canonical_base_lexeme
                .zip(canonical_base_sense)
                .is_some_and(|(lexeme, sense)| {
                    canonical_base_evidence
                        && obsolete.get(4) == lexeme.get(2)
                        && reviewed_glosses_overlap(obsolete.get(8), sense.get(2))
                });
            obsolete.get(11) != Some(&correction.semantic_candidate_id.as_str())
                || obsolete.get(15) != Some(&"reviewed")
                || !(matches_review_overlay || matches_base_identity)
        });
        if invalid_obsolete || (canonical.is_none() && canonical_base_lexeme.is_none()) {
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
    left == right
        || left.contains(&right)
        || right.contains(&left)
        || left
            .split(|character: char| !character.is_alphabetic())
            .filter(|word| word.len() > 2)
            .any(|word| {
                right
                    .split(|character: char| !character.is_alphabetic())
                    .any(|other| word == other)
            })
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

fn load_latest_reviews(root: &Path) -> Result<BTreeMap<String, Review>, Box<dyn Error>> {
    let mut latest = BTreeMap::new();
    for relative in REVIEW_HISTORY {
        for (packet_id, review) in load_reviews(&root.join(relative))? {
            latest.insert(packet_id, review);
        }
    }
    Ok(latest)
}

type ExactTuple = (String, String, String, String);

fn render_packet_ownership(
    root: &Path,
    latest_reviews: &BTreeMap<String, Review>,
    derived: &DerivedRows,
    identity_corrections: &[IdentityCorrection],
    evidence_corrections: &[EvidenceCorrection],
) -> Result<String, Box<dyn Error>> {
    let existing_ownership = load_packet_ownership(&root.join(PACKET_OWNERSHIP))?;
    let exact_path = root.join("data/synodal/exact_forms.tsv");
    let evaluation_path = root.join("data/synodal/evaluation.tsv");
    let exact_current = fs::read_to_string(&exact_path)?;
    let evaluation_current = fs::read_to_string(&evaluation_path)?;
    let exact_text = synchronized_table_text(
        &exact_path,
        &exact_current,
        EXACT_HEADER,
        &derived.exact.iter().cloned().collect::<Vec<_>>(),
        identity_corrections,
        evidence_corrections,
    )?;
    let evaluation_text = synchronized_table_text(
        &evaluation_path,
        &evaluation_current,
        EVALUATION_HEADER,
        &derived.evaluation.iter().cloned().collect::<Vec<_>>(),
        identity_corrections,
        evidence_corrections,
    )?;
    let refreshed = render_packet_ownership_from_tables(
        &exact_text,
        &evaluation_text,
        latest_reviews,
        &MANUAL_PACKET_OWNERS,
        root,
    )?;
    merge_packet_ownership(&existing_ownership, &refreshed, latest_reviews)
}

fn render_packet_ownership_from_tables(
    exact_text: &str,
    evaluation_text: &str,
    latest_reviews: &BTreeMap<String, Review>,
    manual_owners: &[&str],
    root: &Path,
) -> Result<String, Box<dyn Error>> {
    let exact_rows = table_rows(exact_text, EXACT_HEADER)?;
    let evaluation_rows = table_rows(evaluation_text, EVALUATION_HEADER)?;
    let evaluations = evaluation_rows
        .into_iter()
        .map(|fields| (fields[0].to_owned(), fields))
        .collect::<BTreeMap<_, _>>();
    let mut ownership = BTreeMap::new();

    for (packet_id, review) in latest_reviews {
        if review.decision != "admitted" {
            continue;
        }
        let evaluation_id = format!("eval:v07:{}", &stable_hex(&[packet_id])[..16]);
        let evaluation = evaluations.get(&evaluation_id).ok_or_else(|| {
            format!("admitted packet {packet_id} has no materialized evaluation row")
        })?;
        let exact = exact_row_for_evaluation(&exact_rows, evaluation)?;
        ownership.insert(
            packet_id.clone(),
            ownership_row(root, packet_id, exact, evaluation)?,
        );
    }

    for &owner_id in manual_owners {
        let evaluation = evaluations.get(owner_id).ok_or_else(|| {
            format!("manual v0.7 ownership {owner_id} has no materialized evaluation row")
        })?;
        let exact = exact_row_for_evaluation(&exact_rows, evaluation)?;
        ownership.insert(
            owner_id.to_owned(),
            ownership_row(root, owner_id, exact, evaluation)?,
        );
    }

    let mut output = String::from(PACKET_OWNERSHIP_HEADER);
    output.push('\n');
    for row in ownership.into_values() {
        output.push_str(&row);
        output.push('\n');
    }
    Ok(output)
}

fn exact_row_for_evaluation<'a>(
    exact_rows: &'a [Vec<&'a str>],
    evaluation: &[&str],
) -> Result<&'a Vec<&'a str>, Box<dyn Error>> {
    let matches = exact_rows
        .iter()
        .filter(|fields| {
            fields[0] == evaluation[1]
                && fields[1] == evaluation[2]
                && fields[2] == evaluation[4]
                && fields[3] == evaluation[5]
        })
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [row] => Ok(row),
        [] => Err(format!(
            "v0.7 evaluation {:?} has no materialized exact row",
            evaluation[0]
        )
        .into()),
        _ => Err(format!(
            "v0.7 evaluation {:?} has multiple materialized exact rows",
            evaluation[0]
        )
        .into()),
    }
}

fn ownership_row(
    root: &Path,
    owner_id: &str,
    exact: &[&str],
    evaluation: &[&str],
) -> Result<String, Box<dyn Error>> {
    let source_candidate_id = source_candidate_for_exact(root, exact)?;
    Ok([
        owner_id,
        exact[0],
        exact[1],
        exact[2],
        exact[3],
        exact[4],
        exact[5],
        exact[6],
        evaluation[0],
        evaluation[3],
        evaluation[4],
        evaluation[5],
        evaluation[6],
        evaluation[7],
        evaluation[8],
        source_candidate_id.as_str(),
    ]
    .join("\t"))
}

fn source_candidate_for_exact(root: &Path, exact: &[&str]) -> Result<String, Box<dyn Error>> {
    let evidence = fs::read_to_string(root.join("data/synodal/reviewed_evidence.tsv"))?;
    let candidates = table_rows(&evidence, EVIDENCE_HEADER)?
        .into_iter()
        .map(|fields| (fields[0], fields[1]))
        .collect::<BTreeMap<_, _>>();
    exact[4]
        .split(',')
        .find_map(|id| {
            id.starts_with("v07-source-")
                .then(|| candidates.get(id).copied())
                .flatten()
        })
        .map(str::to_owned)
        .ok_or_else(|| format!("durable row for {} has no source candidate", exact[0]).into())
}

fn merge_packet_ownership(
    existing: &[PacketOwnership],
    refreshed: &str,
    latest_reviews: &BTreeMap<String, Review>,
) -> Result<String, Box<dyn Error>> {
    let mut rows = existing
        .iter()
        .map(|row| (row.owner_id().to_owned(), row.fields.join("\t")))
        .collect::<BTreeMap<_, _>>();
    for fields in table_rows(refreshed, PACKET_OWNERSHIP_HEADER)? {
        rows.insert(fields[0].to_owned(), fields.join("\t"));
    }
    rows.retain(|owner_id, _| {
        !owner_id.starts_with("v07-exact-") || latest_reviews.contains_key(owner_id)
    });
    let mut output = String::from(PACKET_OWNERSHIP_HEADER);
    output.push('\n');
    for row in rows.into_values() {
        output.push_str(&row);
        output.push('\n');
    }
    Ok(output)
}

fn load_packet_ownership(path: &Path) -> Result<Vec<PacketOwnership>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(PACKET_OWNERSHIP_HEADER) {
        return Err(format!("invalid v0.7 packet-ownership header in {}", path.display()).into());
    }
    let mut owners = BTreeSet::new();
    let mut ownership = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields = line.split('\t').map(str::to_owned).collect::<Vec<_>>();
        if fields.len() != 16 || fields.iter().any(String::is_empty) {
            return Err(format!("invalid v0.7 packet-ownership row {}", offset + 2).into());
        }
        if !owners.insert(fields[0].clone()) {
            return Err(format!("duplicate v0.7 packet owner {:?}", fields[0]).into());
        }
        ownership.push(PacketOwnership { fields });
    }
    Ok(ownership)
}

fn load_owned_rows(path: &Path, header: &str) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut rows = BTreeMap::new();
    let expected_fields = header.split('\t').count();
    for fields in table_rows(&text, header)? {
        if fields.len() != expected_fields {
            return Err(format!(
                "invalid durable ownership row width in {}: expected {expected_fields}, found {}",
                path.display(),
                fields.len()
            )
            .into());
        }
        if fields.iter().any(|field| field.is_empty()) {
            return Err(format!("empty durable ownership field in {}", path.display()).into());
        }
        let id = fields[0].to_owned();
        if rows.insert(id.clone(), fields.join("\t")).is_some() {
            return Err(format!(
                "duplicate durable ownership row {id:?} in {}",
                path.display()
            )
            .into());
        }
    }
    Ok(rows)
}

fn merge_owned_rows(
    header: &str,
    existing: &BTreeMap<String, String>,
    derived: impl IntoIterator<Item = String>,
) -> Result<String, Box<dyn Error>> {
    let mut rows = existing.clone();
    for row in derived {
        let id = row
            .split('\t')
            .next()
            .ok_or("empty durable ownership row")?;
        if rows
            .insert(id.to_owned(), row.clone())
            .is_some_and(|previous| previous != row)
        {
            return Err(format!("conflicting durable ownership row {id:?}").into());
        }
    }
    let mut output = String::from(header);
    output.push('\n');
    for row in rows.into_values() {
        output.push_str(&row);
        output.push('\n');
    }
    Ok(output)
}

fn activate_owned_dependencies(
    ownership: &[PacketOwnership],
    latest_reviews: &BTreeMap<String, Review>,
    evidence_ownership: &BTreeMap<String, String>,
    lexical_ownership: &BTreeMap<String, String>,
    derived: &mut DerivedRows,
) -> Result<(), Box<dyn Error>> {
    for row in ownership.iter().filter(|row| row.is_active(latest_reviews)) {
        for id in row.fields[5].split(',') {
            if let Some(owned) = evidence_ownership.get(id) {
                if let Some(previous) = derived.evidence.get(id) {
                    if previous != owned
                        && !previous.split('\t').take(6).eq(owned.split('\t').take(6))
                    {
                        return Err(format!("conflicting active evidence ownership {id:?}").into());
                    }
                } else {
                    derived.evidence.insert(id.to_owned(), owned.clone());
                }
            } else if let Some(owned) = lexical_ownership.get(id) {
                let fields = owned.split('\t').collect::<Vec<_>>();
                let lexeme_id = fields
                    .get(1)
                    .ok_or_else(|| format!("invalid active lexical ownership {id:?}"))?;
                if derived
                    .lexical
                    .insert((*lexeme_id).to_owned(), owned.clone())
                    .is_some_and(|previous| previous != *owned)
                {
                    return Err(format!("conflicting active lexical ownership {id:?}").into());
                }
            }
        }
    }
    Ok(())
}

fn validate_packet_ownership(
    root: &Path,
    ownership: &[PacketOwnership],
    latest_reviews: &BTreeMap<String, Review>,
    identity_corrections: &[IdentityCorrection],
    evidence_ownership: &BTreeMap<String, String>,
    lexical_ownership: &BTreeMap<String, String>,
    candidates: &BTreeMap<String, TargetCandidate>,
) -> Result<(), Box<dyn Error>> {
    let admitted = latest_reviews
        .iter()
        .filter(|(_, review)| review.decision == "admitted")
        .map(|(packet_id, _)| packet_id.as_str())
        .collect::<BTreeSet<_>>();
    let packet_owners = ownership
        .iter()
        .filter(|row| row.owner_id().starts_with("v07-exact-"))
        .map(PacketOwnership::owner_id)
        .collect::<BTreeSet<_>>();
    let owner_digest = stable_hex(&packet_owners.iter().copied().collect::<Vec<_>>());
    if packet_owners.len() != HISTORICAL_PACKET_OWNER_COUNT
        || owner_digest != HISTORICAL_PACKET_OWNER_DIGEST
    {
        return Err(format!(
            "durable v0.7 ownership inventory drifted: found {} rows with digest {owner_digest}; expected {HISTORICAL_PACKET_OWNER_COUNT} rows with digest {HISTORICAL_PACKET_OWNER_DIGEST}",
            packet_owners.len(),
        )
        .into());
    }
    if !packet_owners.is_superset(&admitted) {
        let missing = admitted
            .difference(&packet_owners)
            .take(5)
            .collect::<Vec<_>>();
        return Err(format!(
            "durable v0.7 ownership does not cover admitted review history; missing={missing:?}"
        )
        .into());
    }
    if let Some(stale) = packet_owners
        .iter()
        .find(|owner_id| !latest_reviews.contains_key(**owner_id))
    {
        return Err(format!("durable v0.7 ownership has unknown packet {stale}").into());
    }

    let manual = ownership
        .iter()
        .filter(|row| !row.owner_id().starts_with("v07-exact-"))
        .map(PacketOwnership::owner_id)
        .collect::<BTreeSet<_>>();
    let expected_manual = MANUAL_PACKET_OWNERS.into_iter().collect::<BTreeSet<_>>();
    if manual != expected_manual {
        return Err(format!(
            "durable v0.7 ownership has invalid manual owners {manual:?}; expected {expected_manual:?}"
        )
        .into());
    }

    let evidence_text = fs::read_to_string(root.join("data/synodal/reviewed_evidence.tsv"))?;
    let lexical_text = fs::read_to_string(root.join("data/synodal/lexical_reviews.tsv"))?;
    let evidence_rows = table_rows(&evidence_text, EVIDENCE_HEADER)?;
    let lexical_rows = table_rows(&lexical_text, LEXICAL_HEADER)?;
    let evidence_candidates = evidence_rows
        .iter()
        .map(|fields| (fields[0], fields[1]))
        .collect::<BTreeMap<_, _>>();
    let known_evidence = evidence_candidates
        .keys()
        .copied()
        .chain(lexical_rows.iter().map(|fields| fields[0]))
        .chain(evidence_ownership.keys().map(String::as_str))
        .chain(lexical_ownership.keys().map(String::as_str))
        .chain(
            identity_corrections
                .iter()
                .map(|correction| correction.obsolete_review_id.as_str()),
        )
        .collect::<BTreeSet<_>>();

    for row in ownership {
        let fields = &row.fields;
        if fields[3] != fields[10] || fields[4] != fields[11] {
            return Err(
                format!("{} has mismatched exact/evaluation tuples", row.owner_id()).into(),
            );
        }
        if fields[6] != "synodal-attestation" || fields[7] != "synodal-russian" {
            return Err(format!("{} has invalid exact provenance", row.owner_id()).into());
        }
        for id in fields[5].split(',') {
            if id.starts_with("v07-source-") || id.starts_with("v07-target-") {
                let owned = evidence_ownership.get(id).ok_or_else(|| {
                    format!(
                        "{} lacks durable evidence ownership for {id:?}",
                        row.owner_id()
                    )
                })?;
                let evidence = owned.split('\t').collect::<Vec<_>>();
                if evidence[4] != "reviewed" || evidence[5] != "synodal-russian" {
                    return Err(format!(
                        "{} has unreviewed or wrong-recension durable evidence {id:?}",
                        row.owner_id()
                    )
                    .into());
                }
                let expected = if id.starts_with("v07-source-") {
                    if evidence[1] != fields[15] {
                        return Err(format!(
                            "{} source ownership candidate disagrees with {id:?}",
                            row.owner_id()
                        )
                        .into());
                    }
                    stable_id("v07-source", &[evidence[1], &fields[2]])
                } else {
                    if !matches!(
                        evidence[2],
                        "ponomar-elizabeth-bible-2026-08-09"
                            | "wikisource-church-slavonic-bible-2026-08-09"
                    ) {
                        return Err(format!(
                            "{} target ownership uses a non-target source {id:?}",
                            row.owner_id()
                        )
                        .into());
                    }
                    stable_id("v07-target", &[evidence[1], &fields[4]])
                };
                if id != expected {
                    return Err(format!(
                        "{} durable evidence ID {id:?} is not bound to its stored candidate and tuple",
                        row.owner_id()
                    )
                    .into());
                }
            } else if id.starts_with("review:v07:") {
                let owned = lexical_ownership.get(id).ok_or_else(|| {
                    format!(
                        "{} lacks durable lexical ownership for {id:?}",
                        row.owner_id()
                    )
                })?;
                validate_owned_lexical(
                    row.owner_id(),
                    &fields[1],
                    id,
                    owned,
                    row.is_active(latest_reviews),
                )?;
            }
        }
        let evaluation_candidates = candidates
            .values()
            .filter(|candidate| {
                candidate.source_id == fields[12]
                    && candidate.passage == fields[13]
                    && contains_whole_token(&candidate.normalized_spelling, &fields[11])
            })
            .take(2)
            .collect::<Vec<_>>();
        if evaluation_candidates.len() != 1 {
            return Err(format!(
                "{} does not bind to exactly one held-out evaluation candidate",
                row.owner_id()
            )
            .into());
        }
        if row.is_active(latest_reviews)
            && fields[5].split(',').any(|id| !known_evidence.contains(id))
        {
            return Err(format!("{} cites missing durable evidence", row.owner_id()).into());
        }
        if row.owner_id().starts_with("v07-exact-") {
            let expected_evaluation = format!("eval:v07:{}", &stable_hex(&[row.owner_id()])[..16]);
            if fields[8] != expected_evaluation {
                return Err(
                    format!("{} has a non-deterministic evaluation ID", row.owner_id()).into(),
                );
            }
            let mut lexeme_ids = vec![fields[1].as_str()];
            lexeme_ids.extend(
                identity_corrections
                    .iter()
                    .filter(|correction| correction.canonical_lexeme_id == fields[1])
                    .map(|correction| correction.obsolete_lexeme_id.as_str()),
            );
            let source_candidate_id = fields[15].as_str();
            let expected_source_id = stable_id("v07-source", &[source_candidate_id, &fields[2]]);
            let matches_owner = fields[5].split(',').any(|id| id == expected_source_id)
                && lexeme_ids.iter().any(|lexeme_id| {
                    stable_id(
                        "v07-exact",
                        &[source_candidate_id, lexeme_id, &fields[2], &fields[4]],
                    ) == row.owner_id()
                });
            if !matches_owner {
                return Err(format!(
                    "{} cannot be reconstructed from its source evidence and exact tuple",
                    row.owner_id()
                )
                .into());
            }
        } else if fields[8] != fields[0] || fields[14] != "v07-reviewed-identity-correction" {
            return Err(format!("{} has invalid manual ownership metadata", row.owner_id()).into());
        }
    }
    Ok(())
}

fn validate_owned_lexical(
    owner_id: &str,
    owner_lexeme_id: &str,
    review_id: &str,
    owned: &str,
    active: bool,
) -> Result<(), Box<dyn Error>> {
    let lexical = owned.split('\t').collect::<Vec<_>>();
    if lexical.len() != LEXICAL_HEADER.split('\t').count()
        || lexical[0] != review_id
        || lexical[1] != owner_lexeme_id
        || !matches!(lexical[15], "reviewed" | "rejected")
        || (active && lexical[15] != "reviewed")
        || lexical[16] != "synodal-russian"
    {
        return Err(format!(
            "{owner_id} has wrong-identity, unreviewed, or wrong-recension durable lexical ownership {review_id:?}"
        )
        .into());
    }
    Ok(())
}

fn is_packet_evaluation_id(id: &str) -> bool {
    id.strip_prefix("eval:v07:").is_some_and(|suffix| {
        suffix.len() == 16 && suffix.bytes().all(|byte| byte.is_ascii_hexdigit())
    }) || MANUAL_PACKET_OWNERS.contains(&id)
}

fn is_packet_exact_row(fields: &[&str]) -> bool {
    fields.get(4).is_some_and(|evidence| {
        let ids = evidence.split(',').collect::<Vec<_>>();
        ids.iter().any(|id| id.starts_with("v07-source-"))
            && ids.iter().any(|id| id.starts_with("v07-target-"))
    })
}

fn merge_duplicate_exact_rows(exact: &str) -> Result<String, Box<dyn Error>> {
    let mut order = Vec::<ExactTuple>::new();
    let mut rows = BTreeMap::<ExactTuple, Vec<String>>::new();
    for fields in table_rows(exact, EXACT_HEADER)? {
        let key = (
            fields[0].to_owned(),
            fields[1].to_owned(),
            fields[2].to_owned(),
            fields[3].to_owned(),
        );
        if let Some(existing) = rows.get_mut(&key) {
            if existing[5] != fields[5] || existing[6] != fields[6] {
                return Err(format!(
                    "duplicate exact tuple {key:?} disagrees on source kind or target recension"
                )
                .into());
            }
            let mut evidence = existing[4]
                .split(',')
                .map(str::to_owned)
                .collect::<Vec<_>>();
            for id in fields[4].split(',') {
                if !evidence.iter().any(|existing| existing == id) {
                    evidence.push(id.to_owned());
                }
            }
            existing[4] = evidence.join(",");
        } else {
            order.push(key.clone());
            rows.insert(key, fields.into_iter().map(str::to_owned).collect());
        }
    }
    let mut output = String::from(EXACT_HEADER);
    output.push('\n');
    for key in order {
        output.push_str(&rows[&key].join("\t"));
        output.push('\n');
    }
    Ok(output)
}

fn referenced_evidence_ids(
    exact: &str,
    abbreviation: &str,
) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let mut referenced = BTreeSet::new();
    for fields in table_rows(exact, EXACT_HEADER)? {
        referenced.extend(fields[4].split(',').map(str::to_owned));
    }
    for fields in table_rows(abbreviation, ABBREVIATION_HEADER)? {
        referenced.extend(fields[6].split(',').map(str::to_owned));
    }
    Ok(referenced)
}

fn retain_referenced_v07_rows(
    table: &str,
    header: &str,
    referenced: &BTreeSet<String>,
) -> Result<String, Box<dyn Error>> {
    filter_table(table, header, |fields| {
        let id = fields[0];
        let rejected_lexical_decision =
            header == LEXICAL_HEADER && fields.get(15) == Some(&"rejected");
        !(id.starts_with("v07-") || id.starts_with("review:v07:"))
            || referenced.contains(id)
            || rejected_lexical_decision
    })
}

fn table_rows<'a>(table: &'a str, header: &str) -> Result<Vec<Vec<&'a str>>, Box<dyn Error>> {
    let mut lines = table.lines();
    if lines.next() != Some(header) {
        return Err("invalid synchronized v0.7 table header".into());
    }
    Ok(lines
        .filter(|line| !line.is_empty())
        .map(|line| line.split('\t').collect())
        .collect())
}

fn filter_table(
    table: &str,
    header: &str,
    mut retain: impl FnMut(&[&str]) -> bool,
) -> Result<String, Box<dyn Error>> {
    let rows = table_rows(table, header)?;
    let mut output = String::from(header);
    output.push('\n');
    for fields in rows {
        if retain(&fields) {
            output.push_str(&fields.join("\t"));
            output.push('\n');
        }
    }
    Ok(output)
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
            // An admitted exact-form variant has its own source and held-out
            // witnesses. Its bounded base spelling need not remain admitted
            // when a later exact-packet wave rejects or defers that base cell.
            true
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
        if review.lane == "exact-form" {
            let base = try_find_base_fields(
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
            let evidence = base.as_ref().map_or_else(
                || target_id.clone(),
                |_| format!("{},{}", review.base_evidence_id, target_id),
            );
            rows.exact.insert(
                [
                    review.lexeme_id.as_str(),
                    review.cell.as_str(),
                    review.expanded.as_str(),
                    review.printed.as_str(),
                    evidence.as_str(),
                    base.as_ref()
                        .map_or("synodal-attestation", |fields| fields[5]),
                    base.as_ref().map_or("synodal-russian", |fields| fields[6]),
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
            let evidence = format!("{},{}", review.base_evidence_id, target_id);
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
    try_find_base_fields(text, header, indexes, expected)?
        .ok_or_else(|| "reviewed variant base row disappeared during derivation".into())
}

fn try_find_base_fields<'a>(
    text: &'a str,
    header: &str,
    indexes: &[usize],
    expected: &[&String],
) -> Result<Option<Vec<&'a str>>, Box<dyn Error>> {
    let mut lines = text.lines();
    if lines.next() != Some(header) {
        return Err("invalid base registry header".into());
    }
    Ok(lines
        .filter(|line| !line.is_empty())
        .map(|line| line.split('\t').collect::<Vec<_>>())
        .find(|fields| {
            indexes.iter().zip(expected).all(|(index, value)| {
                fields
                    .get(*index)
                    .is_some_and(|field| field == &value.as_str())
            })
        }))
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
    synchronized_table_text(
        path,
        &text,
        header,
        derived,
        corrections,
        evidence_corrections,
    )
}

fn synchronized_table_text(
    path: &Path,
    text: &str,
    header: &str,
    derived: &[String],
    corrections: &[IdentityCorrection],
    evidence_corrections: &[EvidenceCorrection],
) -> Result<String, Box<dyn Error>> {
    let mut lines = text.lines();
    if lines.next() != Some(header) {
        return Err(format!("invalid header in {}", path.display()).into());
    }
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    let mut committed_ids = BTreeSet::new();
    for line in lines.filter(|line| !line.is_empty()) {
        if let Some(corrected) =
            correct_reviewed_row(header, line, corrections, evidence_corrections)?
        {
            if let Some(id) = unique_row_id(header, &corrected) {
                committed_ids.insert(id);
            }
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
            // Tables keyed by a unique first-column identifier must not gain a
            // second row for an identifier a previous wave already
            // materialised. A later review can refine the committed row — for
            // example replacing a generic `lexical-form` cell with a typed
            // one — and re-emitting the original shape would both duplicate
            // the stable ID and regress the refinement. Changes to an already
            // materialised row belong in the identity and evidence correction
            // ledgers, which are applied above.
            if unique_row_id(header, &corrected).is_some_and(|id| committed_ids.contains(&id)) {
                continue;
            }
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

/// Returns the row's unique stable identifier for the tables that have one.
///
/// `exact_forms.tsv` and `abbreviations.tsv` are keyed by a composite of
/// several columns rather than a single ID, so they are deliberately excluded
/// and keep whole-row deduplication.
fn unique_row_id(header: &str, row: &str) -> Option<String> {
    if header == LEXICAL_HEADER
        || header == EVIDENCE_HEADER
        || header == EVALUATION_HEADER
        || header == ABBREVIATION_EVALUATION_HEADER
    {
        return row.split('\t').next().map(str::to_owned);
    }
    None
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

    #[test]
    fn durable_ownership_restores_admissions_and_retracts_reintroduced_rows() {
        let admitted = "lexeme\tcell\texpanded\tprinted\tv07-source-admitted,v07-target-admitted\tsynodal-attestation\tsynodal-russian";
        let rejected = "other\tcell\texpanded\tprinted\tv07-source-rejected,v07-target-rejected\tsynodal-attestation\tsynodal-russian";
        let variant = "lexeme\tcell\texpanded\tvariant\tv07-variant-source-id,v07-variant-target-id\tsynodal-attestation\tsynodal-russian";
        let dirty = format!("{EXACT_HEADER}\n{rejected}\n{variant}\n");
        let base = filter_table(&dirty, EXACT_HEADER, |fields| !is_packet_exact_row(fields))
            .expect("packet rows filter cleanly");
        let restored = synchronized_table_text(
            Path::new("exact.tsv"),
            &base,
            EXACT_HEADER,
            &[admitted.into()],
            &[],
            &[],
        )
        .expect("durable ownership materializes cleanly");
        assert!(restored.contains(admitted));
        assert!(restored.contains(variant));
        assert!(!restored.contains(rejected));

        let second_base = filter_table(&restored, EXACT_HEADER, |fields| {
            !is_packet_exact_row(fields)
        })
        .expect("second pass packet rows filter cleanly");
        let second = synchronized_table_text(
            Path::new("exact.tsv"),
            &second_base,
            EXACT_HEADER,
            &[admitted.into()],
            &[],
            &[],
        )
        .expect("second durable materialization is valid");
        assert_eq!(second, restored);
    }

    #[test]
    fn ownership_refresh_bootstraps_a_newly_admitted_packet() {
        let packet_id = "v07-exact-new-admission";
        let evaluation_id = format!("eval:v07:{}", &stable_hex(&[packet_id])[..16]);
        let exact = "lexeme\tcell\texpanded\tprinted\tv07-source-80c65e35ce25632b,v07-target-new\tsynodal-attestation\tsynodal-russian";
        let evaluation = format!(
            "{evaluation_id}\tlexeme\tcell\tstrict\texpanded\tprinted\tsource\tpassage\tv07-held-out-exact-cell"
        );
        let exact_table = synchronized_table_text(
            Path::new("exact.tsv"),
            &format!("{EXACT_HEADER}\n"),
            EXACT_HEADER,
            &[exact.into()],
            &[],
            &[],
        )
        .expect("new exact admission materializes in memory");
        let evaluation_table = synchronized_table_text(
            Path::new("evaluation.tsv"),
            &format!("{EVALUATION_HEADER}\n"),
            EVALUATION_HEADER,
            &[evaluation],
            &[],
            &[],
        )
        .expect("new evaluation admission materializes in memory");
        let latest_reviews = BTreeMap::from([(
            packet_id.into(),
            Review {
                decision: "admitted".into(),
                realized_unique_tokens: 0,
                blocker: String::new(),
                review_note: "newly admitted after review".into(),
            },
        )]);

        let ownership = render_packet_ownership_from_tables(
            &exact_table,
            &evaluation_table,
            &latest_reviews,
            &[],
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        )
        .expect("ownership refresh bootstraps the new admission");
        assert!(ownership.contains(packet_id));
        assert!(ownership.contains(&evaluation_id));
    }

    #[test]
    fn historical_owner_activation_tracks_the_latest_decision() {
        let owner = PacketOwnership {
            fields: [
                "v07-exact-historical",
                "lexeme",
                "cell",
                "expanded",
                "printed",
                "v07-source-id,v07-target-id",
                "synodal-attestation",
                "synodal-russian",
                "eval:v07:0123456789abcdef",
                "strict",
                "expanded",
                "printed",
                "source",
                "passage",
                "v07-held-out-exact-cell",
                "candidate",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        };
        let review = |decision: &str| {
            BTreeMap::from([(
                owner.owner_id().to_owned(),
                Review {
                    decision: decision.into(),
                    realized_unique_tokens: 0,
                    blocker: if decision == "admitted" {
                        String::new()
                    } else {
                        "reviewed blocker".into()
                    },
                    review_note: "reviewed decision".into(),
                },
            )])
        };

        assert!(!owner.is_active(&review("deferred")));
        assert!(owner.is_active(&review("admitted")));
    }

    #[test]
    fn duplicate_exact_rows_merge_their_evidence() {
        let exact = format!(
            "{EXACT_HEADER}\nlexeme\tcell\texpanded\tprinted\treview,source\tsynodal-attestation\tsynodal-russian\nlexeme\tcell\texpanded\tprinted\tsource,target\tsynodal-attestation\tsynodal-russian\n"
        );
        assert_eq!(
            merge_duplicate_exact_rows(&exact).expect("compatible rows merge"),
            format!(
                "{EXACT_HEADER}\nlexeme\tcell\texpanded\tprinted\treview,source,target\tsynodal-attestation\tsynodal-russian\n"
            )
        );
    }

    #[test]
    fn durable_lexical_ownership_is_bound_to_identity_and_active_review() {
        let reviewed = "review:v07:test\tlexeme\tsense\tlemma\tnoun\tlexical-form\texpanded\tprinted\tgloss\tdomain\tsemantic-source\tsemantic-candidate\ttarget-source\ttarget-candidate\tpassage\treviewed\tsynodal-russian\tnote";
        let rejected = reviewed.replacen("\treviewed\t", "\trejected\t", 1);

        validate_owned_lexical("owner", "lexeme", "review:v07:test", reviewed, true)
            .expect("active reviewed ownership is valid");
        validate_owned_lexical("owner", "lexeme", "review:v07:test", &rejected, false)
            .expect("inactive rejected ownership remains durable");
        assert!(
            validate_owned_lexical("owner", "other", "review:v07:test", reviewed, true).is_err()
        );
        assert!(
            validate_owned_lexical("owner", "lexeme", "review:v07:test", &rejected, true).is_err()
        );
    }
}
