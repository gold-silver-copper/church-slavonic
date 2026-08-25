use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::report_io::{check_contents, write_if_changed};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use synodal_church_slavonic_core::normalize_lookup_accentless;
use synodal_church_slavonic_dictionary::coverage::{
    CoverageReport, GapContext, GapKind, GapRecord,
};
use synodal_church_slavonic_dictionary::{FamilyId, show_family_by_id};

const DICTIONARY_SOURCE: &str = "english-wiktionary-ocs-kaikki-2026-08-07";
const PONOMAR_DICTIONARY_SOURCE: &str = "ponomar-modern-church-slavonic-corpus-2016";
const UD_SOURCE: &str = "ud-ocs-proiel-r2.18";
const REVIEW_HEADER: &str = "candidate_id\tdecision\tlinked_lexeme_id\tadmitted_class\tadmitted_stem\tgender\taspect\tnumber_restriction\tanimacy\tstem_alternants\tprincipal_parts\taccent_metadata\tpositional_metadata\tabbreviation_metadata\tnormative_source\tnormative_citation\ttarget_evidence\tsemantic_evidence\tconfidence_bp\tassumptions\treview_note";

#[derive(Clone, Debug, Deserialize)]
struct CandidateRecord {
    candidate_id: String,
    source_id: String,
    raw_spelling: String,
    normalized_spelling: String,
    part_of_speech: String,
    grammatical_cell: String,
    partition: String,
}

#[derive(Clone, Debug, Deserialize)]
struct DictionaryEntry {
    word: String,
    pos: String,
    #[serde(default)]
    forms: Vec<DictionaryForm>,
    #[serde(default)]
    senses: Vec<DictionarySense>,
}

#[derive(Clone, Debug, Deserialize)]
struct DictionaryForm {
    form: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct DictionarySense {
    #[serde(default)]
    glosses: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct PonomarDictionaryEntry {
    definition: String,
}

#[derive(Clone, Debug)]
struct SupplementalDictionaryEvidence {
    candidate_id: String,
    definition: String,
}

#[derive(Clone, Debug)]
struct DictionaryFamily {
    candidate_id: String,
    partition: String,
    lemma: String,
    part_of_speech: String,
    glosses: Vec<String>,
    forms: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Clone, Debug)]
struct MorphologicalWitness {
    candidate_id: String,
    lemma: String,
    part_of_speech: String,
    cell: String,
}

#[derive(Clone, Debug)]
struct ReviewedLexeme {
    id: String,
    part_of_speech: synodal_church_slavonic::PartOfSpeech,
}

#[derive(Clone, Debug, Default)]
struct ReviewDecision {
    decision: String,
    linked_lexeme_id: String,
    admitted_class: String,
    reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct FamilySurface {
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct FamilyProposal {
    rank: usize,
    candidate_id: String,
    proposed_lemma: String,
    part_of_speech: String,
    surfaces: Vec<FamilySurface>,
    token_frequency: usize,
    document_frequency: usize,
    documents: Vec<String>,
    contexts: Vec<GapContext>,
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

#[derive(Clone, Debug, Default)]
struct FamilyAggregate {
    proposed_lemma: String,
    part_of_speech: String,
    surfaces: Vec<FamilySurface>,
    corpora: BTreeSet<String>,
    source_ids: BTreeSet<String>,
    editions: BTreeSet<String>,
    passages: BTreeSet<String>,
    documents: BTreeSet<String>,
    contexts: BTreeMap<(String, usize, usize), GapContext>,
    partitions: BTreeSet<String>,
    possible_cells: BTreeSet<String>,
    diagnostic_features: BTreeSet<String>,
    compatible_existing_lexemes: BTreeSet<String>,
    dictionary_candidate_ids: BTreeSet<String>,
    supporting_evidence: BTreeSet<String>,
    contradicting_evidence: BTreeSet<String>,
    missing_metadata: BTreeSet<String>,
    assumptions: BTreeSet<String>,
    confidence_basis_points: u16,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut intermediate = root.join("data/intermediate/synodal");
    let mut coverage_path = root.join("reports/synodal-coverage.json");
    let mut limit = usize::MAX;
    let mut check = false;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--intermediate" => {
                intermediate = PathBuf::from(args.next().ok_or("--intermediate needs a path")?);
            }
            "--coverage" => {
                coverage_path = PathBuf::from(args.next().ok_or("--coverage needs a path")?);
            }
            "--limit" => limit = args.next().ok_or("--limit needs a number")?.parse()?,
            "--check" => check = true,
            value => {
                return Err(
                    format!("unknown synodal-family-review-queue argument {value:?}").into(),
                );
            }
        }
    }
    let report: CoverageReport = serde_json::from_str(&fs::read_to_string(coverage_path)?)?;
    let dictionary = load_dictionary(&intermediate.join(format!("{DICTIONARY_SOURCE}.jsonl")))?;
    let supplemental_dictionary =
        load_ponomar_dictionary(&intermediate.join(format!("{PONOMAR_DICTIONARY_SOURCE}.jsonl")))?;
    let morphological_witnesses =
        load_morphological_witnesses(&intermediate.join(format!("{UD_SOURCE}.jsonl")))?;
    let reviewed_lexemes = reviewed_lexemes_by_lemma()?;
    let reviewed_dictionary_lexemes =
        load_reviewed_dictionary_lexemes(&root.join("data/synodal/lexical_reviews.tsv"))?;
    let reviews = load_reviews(&root.join("data/synodal/family_reviews.tsv"))?;
    validate_admitted_families(&reviews)?;
    let proposals = build_proposals(
        &report.gaps,
        &ProposalEvidence {
            dictionary: &dictionary,
            supplemental_dictionary: &supplemental_dictionary,
            morphological_witnesses: &morphological_witnesses,
            reviewed_lexemes: &reviewed_lexemes,
            reviewed_dictionary_lexemes: &reviewed_dictionary_lexemes,
            reviews: &reviews,
        },
        limit,
    );
    let reviewed_top_200 = proposals
        .iter()
        .take(200)
        .filter(|proposal| proposal.review_status != "candidate-unreviewed")
        .count();
    if check && (proposals.len() < 200 || reviewed_top_200 < 200) {
        return Err(format!(
            "family review gate requires decisions for the top 200 proposals; found {reviewed_top_200}"
        )
        .into());
    }
    let json = format!("{}\n", serde_json::to_string_pretty(&proposals)?);
    let tsv = proposal_tsv(&proposals);
    let json_path = root.join("reports/synodal-family-review-queue.json");
    let tsv_path = root.join("reports/synodal-family-review-queue.tsv");
    if check {
        check_contents(&json_path, &json)?;
        check_contents(&tsv_path, &tsv)?;
    } else {
        write_if_changed(&json_path, &json)?;
        write_if_changed(&tsv_path, &tsv)?;
    }
    println!(
        "Synodal family review queue: {} deterministic proposals; {} carry decisions; {reviewed_top_200}/200 highest-impact proposals reviewed",
        proposals.len(),
        proposals
            .iter()
            .filter(|proposal| proposal.review_status != "candidate-unreviewed")
            .count()
    );
    Ok(())
}

fn load_morphological_witnesses(
    path: &Path,
) -> Result<BTreeMap<String, Vec<MorphologicalWitness>>, Box<dyn Error>> {
    let mut index: BTreeMap<String, Vec<MorphologicalWitness>> = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let candidate: CandidateRecord = serde_json::from_str(line)?;
        if candidate.source_id != UD_SOURCE
            || candidate.partition != "source"
            || candidate.grammatical_cell == "_"
        {
            continue;
        }
        index
            .entry(spelling_key(&candidate.raw_spelling))
            .or_default()
            .push(MorphologicalWitness {
                candidate_id: candidate.candidate_id,
                lemma: candidate.normalized_spelling,
                part_of_speech: candidate.part_of_speech,
                cell: candidate.grammatical_cell,
            });
    }
    for witnesses in index.values_mut() {
        witnesses.sort_by(|left, right| {
            left.lemma
                .cmp(&right.lemma)
                .then_with(|| left.cell.cmp(&right.cell))
                .then_with(|| left.candidate_id.cmp(&right.candidate_id))
        });
        witnesses.dedup_by(|left, right| {
            left.lemma == right.lemma
                && left.part_of_speech == right.part_of_speech
                && left.cell == right.cell
        });
    }
    Ok(index)
}

fn reviewed_lexemes_by_lemma() -> Result<BTreeMap<String, Vec<ReviewedLexeme>>, Box<dyn Error>> {
    let mut index: BTreeMap<String, Vec<ReviewedLexeme>> = BTreeMap::new();
    for lexeme in synodal_church_slavonic::lexemes()? {
        index
            .entry(spelling_key(lexeme.lemma()))
            .or_default()
            .push(ReviewedLexeme {
                id: lexeme.id().to_string(),
                part_of_speech: lexeme.part_of_speech(),
            });
    }
    for lexemes in index.values_mut() {
        lexemes.sort_by(|left, right| left.id.cmp(&right.id));
        lexemes.dedup_by(|left, right| left.id == right.id);
    }
    Ok(index)
}

fn load_dictionary(path: &Path) -> Result<BTreeMap<String, Vec<DictionaryFamily>>, Box<dyn Error>> {
    let mut index: BTreeMap<String, Vec<DictionaryFamily>> = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let candidate: CandidateRecord = serde_json::from_str(line)?;
        if candidate.source_id != DICTIONARY_SOURCE {
            continue;
        }
        let entry: DictionaryEntry = serde_json::from_str(&candidate.raw_spelling)?;
        let mut forms: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for form in std::iter::once(DictionaryForm {
            form: entry.word.clone(),
            tags: vec!["lemma".into()],
        })
        .chain(entry.forms.iter().cloned())
        {
            if !form.form.chars().any(char::is_alphabetic)
                || form.tags.iter().any(|tag| tag == "romanization")
            {
                continue;
            }
            forms
                .entry(spelling_key(&form.form))
                .or_default()
                .extend(form.tags);
        }
        let family = DictionaryFamily {
            candidate_id: candidate.candidate_id,
            partition: candidate.partition,
            lemma: entry.word,
            part_of_speech: map_pos(&entry.pos).unwrap_or(&entry.pos).to_owned(),
            glosses: entry
                .senses
                .into_iter()
                .flat_map(|sense| sense.glosses)
                .map(|gloss| sanitize(&gloss))
                .filter(|gloss| !gloss.is_empty())
                .collect(),
            forms: forms.clone(),
        };
        for key in forms.keys() {
            index.entry(key.clone()).or_default().push(family.clone());
        }
    }
    for families in index.values_mut() {
        families.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
        families.dedup_by(|left, right| left.candidate_id == right.candidate_id);
    }
    Ok(index)
}

fn load_ponomar_dictionary(
    path: &Path,
) -> Result<BTreeMap<String, Vec<SupplementalDictionaryEvidence>>, Box<dyn Error>> {
    let mut index: BTreeMap<String, Vec<SupplementalDictionaryEvidence>> = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let candidate: CandidateRecord = serde_json::from_str(line)?;
        if candidate.source_id != PONOMAR_DICTIONARY_SOURCE
            || candidate.part_of_speech != "dictionary-entry"
            || candidate.grammatical_cell != "headword-with-definition"
        {
            continue;
        }
        let entry: PonomarDictionaryEntry = serde_json::from_str(&candidate.raw_spelling)?;
        let definition = sanitize(&entry.definition);
        if definition.is_empty() {
            continue;
        }
        index
            .entry(spelling_key(&candidate.normalized_spelling))
            .or_default()
            .push(SupplementalDictionaryEvidence {
                candidate_id: candidate.candidate_id,
                definition,
            });
    }
    for evidence in index.values_mut() {
        evidence.sort_by(|left, right| left.candidate_id.cmp(&right.candidate_id));
        evidence.dedup_by(|left, right| left.candidate_id == right.candidate_id);
    }
    Ok(index)
}

fn load_reviewed_dictionary_lexemes(
    path: &Path,
) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    const HEADER: &str = "review_id\tlexeme_id\tsense_id\tlemma\tpart_of_speech\tcell\texpanded\tprinted\tgloss\tdomains\tsemantic_source_id\tsemantic_candidate_id\tattestation_source_id\tattestation_candidate_id\tcitation\tdecision\ttarget_recension\treview_note";
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err(format!("{} has an invalid lexical-review header", path.display()).into());
    }
    let mut reviewed = BTreeMap::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 18 {
            return Err(format!(
                "{}:{}: expected 18 fields, found {}",
                path.display(),
                offset + 2,
                fields.len()
            )
            .into());
        }
        if fields[15] != "reviewed" || fields[10] != DICTIONARY_SOURCE {
            continue;
        }
        if let Some(previous) = reviewed.insert(fields[11].into(), fields[1].into())
            && previous != fields[1]
        {
            return Err(format!(
                "{}:{}: dictionary candidate {} maps to multiple reviewed lexemes",
                path.display(),
                offset + 2,
                fields[11]
            )
            .into());
        }
    }
    Ok(reviewed)
}

fn load_reviews(path: &Path) -> Result<BTreeMap<String, ReviewDecision>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(REVIEW_HEADER) {
        return Err(format!("{} has an invalid family-review header", path.display()).into());
    }
    let mut reviews = BTreeMap::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 21 {
            return Err(format!(
                "{}:{}: expected 21 fields, found {}",
                path.display(),
                offset + 2,
                fields.len()
            )
            .into());
        }
        if !matches!(fields[1], "admitted" | "deferred" | "rejected") {
            return Err(format!("{}:{}: invalid decision", path.display(), offset + 2).into());
        }
        if fields[20].is_empty() {
            return Err(
                format!("{}:{}: review note is required", path.display(), offset + 2).into(),
            );
        }
        if fields[0].is_empty() {
            return Err(format!(
                "{}:{}: candidate ID is required",
                path.display(),
                offset + 2
            )
            .into());
        }
        let confidence = fields[18].parse::<u16>().map_err(|_| {
            format!(
                "{}:{}: confidence_bp must be an integer",
                path.display(),
                offset + 2
            )
        })?;
        if confidence > 10_000 {
            return Err(format!(
                "{}:{}: confidence_bp exceeds 10000",
                path.display(),
                offset + 2
            )
            .into());
        }
        if fields[14].is_empty() != fields[15].is_empty() {
            return Err(format!(
                "{}:{}: normative source and citation must be supplied together",
                path.display(),
                offset + 2
            )
            .into());
        }
        if fields[1] == "admitted"
            && (fields[2].is_empty()
                || fields[3].is_empty()
                || fields[16].is_empty()
                || fields[17].is_empty())
        {
            return Err(format!(
                "{}:{}: admitted reviews require a linked lexeme, admitted class, target evidence, and semantic evidence",
                path.display(),
                offset + 2
            )
            .into());
        }
        if reviews
            .insert(
                fields[0].into(),
                ReviewDecision {
                    decision: fields[1].into(),
                    linked_lexeme_id: fields[2].into(),
                    admitted_class: fields[3].into(),
                    reason: fields[20].into(),
                },
            )
            .is_some()
        {
            return Err(
                format!("{}:{}: duplicate candidate ID", path.display(), offset + 2).into(),
            );
        }
    }
    Ok(reviews)
}

fn validate_admitted_families(
    reviews: &BTreeMap<String, ReviewDecision>,
) -> Result<(), Box<dyn Error>> {
    for (candidate_id, review) in reviews
        .iter()
        .filter(|(_, review)| review.decision == "admitted")
    {
        let lexeme_id =
            synodal_church_slavonic_core::LexemeId::from(review.linked_lexeme_id.as_str());
        let family = show_family_by_id(&FamilyId::for_lexeme(&lexeme_id)).map_err(|error| {
            format!(
                "admitted family {candidate_id} does not resolve {}: {error}",
                review.linked_lexeme_id
            )
        })?;
        let valid = match review.admitted_class.as_str() {
            "second-soft" => {
                family.fully_classed
                    && family.class.as_deref() == Some("second-soft")
                    && family.members.len() >= 21
            }
            "first-hard-m"
            | "first-hard-n"
            | "first-hard-velar-m"
            | "first-mixed-ts-m"
            | "first-soft-m"
            | "first-soft-ie-n"
            | "fourth-feminine-er-daughter"
            | "fourth-neuter-at"
            | "second-hard" => {
                family.fully_classed
                    && family.class.as_deref() == Some(review.admitted_class.as_str())
                    && !family.members.is_empty()
            }
            "possessive-j-short" | "possessive-in" | "possessive-sk" | "hard-short"
            | "velar-short" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some(review.admitted_class.as_str())
                    && family
                        .supported_systems
                        .iter()
                        .any(|system| system == "adjective")
            }
            "numeral-cardinal-one" | "numeral-cardinal-both" | "ordinal-hard" | "ordinal-ii" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some(review.admitted_class.as_str())
                    && family
                        .supported_systems
                        .iter()
                        .any(|system| system == "numeral")
            }
            "indeclinable" => {
                family.fully_classed
                    && family.class.as_deref() == Some("indeclinable")
                    && !family.members.is_empty()
            }
            "exact-typed-positional-cells" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some("second-hard")
                    && family.members.len() >= 2
            }
            "exact-irregular-cells-only" => family.exact_only && family.members.len() >= 7,
            "exact-cell-table" => family.exact_only && family.members.len() >= 5,
            "determiner-ves-mixed-with-exact-overrides" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some("determiner-ves-mixed")
                    && family.members.len() >= 5
            }
            "pronoun-reflexive-with-exact-overrides" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some("pronoun-reflexive")
                    && family.members.len() >= 7
            }
            "pronoun-proximal-sei-with-exact-overrides" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some("pronoun-proximal-sei")
                    && family.members.len() >= 5
            }
            "pronoun-relative-izhe-with-exact-overrides" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some("pronoun-relative-izhe")
                    && family.members.len() >= 5
            }
            "exact-complete-pronoun-table" => {
                family.fully_classed
                    && matches!(
                        (family.class.as_deref(), family.exact_only),
                        (Some("exact-complete-pronoun-table"), true)
                            | (Some("pronoun-soft" | "pronoun-mixed-possessive"), false)
                    )
                    && family.members.len() == 57
            }
            "first-hard-u-stem-m-with-exact-consonantal-overrides" => {
                family.class.as_deref() == Some("first-hard-u-stem-m") && family.members.len() >= 5
            }
            // A verb admitted purely productively has no exact rows at all, so
            // the "with-exact-overrides" classes above cannot describe it: they
            // require a member count this family will never have. What is
            // checkable instead is that the class is reviewed, generation is
            // enabled, and the systems the principal parts claim are the ones
            // the engine actually supports.
            "second-verb-system-productive-only"
            | "first-unpalatalized-verb-system-productive-only"
            | "first-palatalized-verb-system-productive-only" => {
                let expected = review
                    .admitted_class
                    .trim_end_matches("-verb-system-productive-only");
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some(expected)
                    && family.lexeme.part_of_speech() == synodal_church_slavonic::PartOfSpeech::Verb
                    && ["future", "aorist", "imperative", "l-participle"]
                        .iter()
                        .all(|system| {
                            family
                                .supported_systems
                                .iter()
                                .any(|supported| supported == system)
                        })
            }
            // An imperfective verb admitted purely productively claims the
            // present, imperfect, and imperative systems; the simple future,
            // aorist, and l-participle of the perfective contract above are
            // not what its principal parts license.
            "second-imperfective-verb-system-productive-only"
            | "first-unpalatalized-imperfective-verb-system-productive-only"
            | "first-palatalized-imperfective-verb-system-productive-only" => {
                let expected = review
                    .admitted_class
                    .trim_end_matches("-imperfective-verb-system-productive-only");
                let supports = |system: &str| {
                    family
                        .supported_systems
                        .iter()
                        .any(|supported| supported == system)
                };
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some(expected)
                    && family.lexeme.part_of_speech() == synodal_church_slavonic::PartOfSpeech::Verb
                    && supports("present")
                    && supports("imperative")
                    // An imperfective verb carries at least one attested past
                    // system; which one varies (клѧтисѧ has an aorist and no
                    // imperfect print, боѧтисѧ the reverse).
                    && (supports("imperfect") || supports("aorist"))
            }
            "second-verb-system-with-exact-overrides" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some("second")
                    && family.members.len() >= 15
            }
            "first-palatalized-verb-system-with-exact-overrides" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some("first-palatalized")
                    && family.members.len() >= 5
            }
            "first-unpalatalized-verb-system-with-exact-overrides" => {
                family.fully_classed
                    && !family.exact_only
                    && family.class.as_deref() == Some("first-unpalatalized")
                    && family.members.len() >= 5
            }
            "archaic-verb-participle-system-with-exact-overrides" => {
                !family.exact_only
                    && family.lexeme.part_of_speech() == synodal_church_slavonic::PartOfSpeech::Verb
                    && family.class.as_deref() == Some("archaic")
                    && family.members.len() >= 15
                    && family
                        .supported_systems
                        .iter()
                        .any(|system| system == "participle")
                    && family
                        .principal_parts
                        .get("present-active-participle-short-stem")
                        .is_some_and(|stem| stem == "сꙋщ")
                    && family
                        .principal_parts
                        .get("present-active-participle-long-stem")
                        .is_some_and(|stem| stem == "сꙋщ")
            }
            "exact-typed-abbreviation-cells" => {
                family
                    .members
                    .iter()
                    .filter(|member| member.source_kind == "abbreviation")
                    .count()
                    >= 7
            }
            "exact-indeclinable-abbreviation" => {
                family
                    .members
                    .iter()
                    .filter(|member| member.source_kind == "abbreviation")
                    .count()
                    >= 1
            }
            "exact-indeclinable-adverb" => exact_indeclinable_family_matches(
                &family,
                synodal_church_slavonic::PartOfSpeech::Adverb,
            ),
            "exact-indeclinable-conjunction" => exact_indeclinable_family_matches(
                &family,
                synodal_church_slavonic::PartOfSpeech::Conjunction,
            ),
            "exact-indeclinable-interjection" => exact_indeclinable_family_matches(
                &family,
                synodal_church_slavonic::PartOfSpeech::Interjection,
            ),
            "exact-indeclinable-preposition" => exact_indeclinable_family_matches(
                &family,
                synodal_church_slavonic::PartOfSpeech::Preposition,
            ),
            "exact-abbreviation-variant" => family
                .members
                .iter()
                .any(|member| member.source_kind == "abbreviation"),
            _ => false,
        };
        if !valid {
            return Err(format!(
                "admitted family {candidate_id} disagrees with its generated runtime family {} ({})",
                review.linked_lexeme_id, review.admitted_class
            )
            .into());
        }
    }
    Ok(())
}

fn exact_indeclinable_family_matches(
    family: &synodal_church_slavonic_dictionary::FamilySummary,
    expected_part_of_speech: synodal_church_slavonic::PartOfSpeech,
) -> bool {
    family.exact_only
        && family.lexeme.part_of_speech() == expected_part_of_speech
        && matches!(family.members.as_slice(), [member] if member.cell == "indeclinable")
}

/// The reviewed and source-derived evidence indexes consulted while grouping
/// gap surfaces into family proposals.
struct ProposalEvidence<'a> {
    dictionary: &'a BTreeMap<String, Vec<DictionaryFamily>>,
    supplemental_dictionary: &'a BTreeMap<String, Vec<SupplementalDictionaryEvidence>>,
    morphological_witnesses: &'a BTreeMap<String, Vec<MorphologicalWitness>>,
    reviewed_lexemes: &'a BTreeMap<String, Vec<ReviewedLexeme>>,
    reviewed_dictionary_lexemes: &'a BTreeMap<String, String>,
    reviews: &'a BTreeMap<String, ReviewDecision>,
}

fn build_proposals(
    gaps: &[GapRecord],
    evidence: &ProposalEvidence<'_>,
    limit: usize,
) -> Vec<FamilyProposal> {
    let ProposalEvidence {
        dictionary,
        supplemental_dictionary,
        morphological_witnesses,
        reviewed_lexemes,
        reviewed_dictionary_lexemes,
        reviews,
    } = *evidence;
    let mut groups: BTreeMap<String, FamilyAggregate> = BTreeMap::new();
    for gap in gaps.iter().filter(|gap| gap.top_k_uncovered_frequency > 0) {
        let family_key = diagnostic_family_key(gap, dictionary);
        let dictionary_matches = dictionary
            .get(&spelling_key(&gap.normalized))
            .cloned()
            .unwrap_or_default();
        let supplemental_dictionary_matches = supplemental_dictionary
            .get(&spelling_key(&gap.normalized))
            .cloned()
            .unwrap_or_default();
        let morphological_matches = morphological_witnesses
            .get(&spelling_key(&gap.original))
            .cloned()
            .unwrap_or_default();
        let (lemma, part_of_speech, confidence) =
            proposal_identity(gap, &dictionary_matches, &family_key);
        let aggregate = groups.entry(family_key.clone()).or_default();
        if aggregate.proposed_lemma.is_empty() {
            aggregate.proposed_lemma = lemma;
            aggregate.part_of_speech = part_of_speech;
            aggregate.confidence_basis_points = confidence;
        } else {
            aggregate.confidence_basis_points = aggregate.confidence_basis_points.min(confidence);
        }
        let mut cells = possible_cells(gap, &dictionary_matches);
        cells.extend(morphological_matches.iter().map(|witness| {
            format!(
                "ud:{}:{}",
                witness.part_of_speech,
                witness.cell.replace('|', "+")
            )
        }));
        cells.sort();
        cells.dedup();
        aggregate.surfaces.push(FamilySurface {
            original: gap.original.clone(),
            normalized: gap.normalized.clone(),
            frequency: gap.top_k_uncovered_frequency,
            document_frequency: gap.top_k_uncovered_documents.len(),
            possible_cells: cells.clone(),
            corpus: gap.corpus.clone(),
            source_id: gap.source_id.clone(),
            edition: gap.edition.clone(),
            passage: gap.passage.clone(),
            partition: gap.partition.clone(),
        });
        aggregate.corpora.extend(gap.corpora.iter().cloned());
        aggregate.source_ids.extend(gap.source_ids.iter().cloned());
        aggregate.editions.extend(gap.editions.iter().cloned());
        aggregate.passages.insert(gap.passage.clone());
        if gap.top_k_uncovered_documents.is_empty() {
            aggregate
                .documents
                .insert(format!("{}:{}", gap.source_id, gap.passage));
        } else {
            aggregate
                .documents
                .extend(gap.top_k_uncovered_documents.iter().cloned());
        }
        aggregate.partitions.extend(gap.partitions.iter().cloned());
        for context in &gap.contexts {
            aggregate.contexts.insert(
                (context.document.clone(), context.line, context.column),
                context.clone(),
            );
        }
        aggregate.possible_cells.extend(cells);
        aggregate
            .diagnostic_features
            .extend(diagnostic_features(gap, &dictionary_matches));
        aggregate
            .compatible_existing_lexemes
            .extend(gap.candidate_lexeme_ids.iter().map(ToString::to_string));
        match family_key.as_str() {
            "probable-reviewed-stem:synodal:noun:syn" => {
                aggregate
                    .compatible_existing_lexemes
                    .insert("synodal:noun:syn".into());
            }
            "probable-reviewed-stem:synodal:noun:zemlya" => {
                aggregate
                    .compatible_existing_lexemes
                    .insert("synodal:noun:zemlya".into());
            }
            "abbreviation-family:synodal:noun:wikt-ed67a3345df1" => {
                aggregate
                    .compatible_existing_lexemes
                    .insert("synodal:noun:wikt-ed67a3345df1".into());
            }
            "probable-irregular-family:synodal:verb:wikt-06af096688df" => {
                aggregate
                    .compatible_existing_lexemes
                    .insert("synodal:verb:wikt-06af096688df".into());
            }
            "probable-determiner-family:весь" => {
                aggregate
                    .compatible_existing_lexemes
                    .insert("synodal:determiner:ves".into());
            }
            _ => {}
        }
        for family in &dictionary_matches {
            aggregate
                .dictionary_candidate_ids
                .insert(family.candidate_id.clone());
            if let Some(lexeme_id) = reviewed_dictionary_lexemes.get(&family.candidate_id) {
                aggregate
                    .compatible_existing_lexemes
                    .insert(lexeme_id.clone());
            }
            aggregate.supporting_evidence.insert(format!(
                "{}:{} supplies an OCS dictionary identity/form candidate in the {} partition ({})",
                DICTIONARY_SOURCE,
                family.candidate_id,
                family.partition,
                family.glosses.join("; ")
            ));
            if family.partition != "source" {
                aggregate.missing_metadata.insert(
                    "source-partition-exact-cell-evidence; evaluation candidates cannot license runtime facts"
                        .into(),
                );
            }
        }
        for evidence in &supplemental_dictionary_matches {
            aggregate
                .dictionary_candidate_ids
                .insert(evidence.candidate_id.clone());
            aggregate.supporting_evidence.insert(format!(
                "{PONOMAR_DICTIONARY_SOURCE}:{} supplies a mixed-recension Church Slavonic headword and semantic candidate ({})",
                evidence.candidate_id, evidence.definition
            ));
        }
        if !supplemental_dictionary_matches.is_empty() {
            aggregate.contradicting_evidence.insert(
                "the SCI Ponomar dictionary is mixed-recension semantic evidence and does not type a target Synodal cell"
                    .into(),
            );
        }
        for witness in &morphological_matches {
            if let Some(lexemes) = reviewed_lexemes.get(&spelling_key(&witness.lemma)) {
                aggregate.compatible_existing_lexemes.extend(
                    lexemes
                        .iter()
                        .filter(|lexeme| {
                            ud_pos_matches_runtime(&witness.part_of_speech, lexeme.part_of_speech)
                        })
                        .map(|lexeme| lexeme.id.clone()),
                );
            }
            aggregate.possible_cells.insert(format!(
                "ud:{}:{}",
                witness.part_of_speech,
                witness.cell.replace('|', "+")
            ));
            aggregate.supporting_evidence.insert(format!(
                "{UD_SOURCE}:{} supplies a source-partition typed OCS token witness for lemma {} ({})",
                witness.candidate_id, witness.lemma, witness.cell
            ));
        }
        if morphological_matches.len() > 1 {
            aggregate.contradicting_evidence.insert(
                "more than one source-partition treebank lemma or cell matches the target surface"
                    .into(),
            );
        }
        if !morphological_matches.is_empty() {
            aggregate.contradicting_evidence.insert(
                "an OCS treebank cell requires an independent exact Synodal surface witness".into(),
            );
        }
        aggregate.supporting_evidence.insert(format!(
            "{}:{}:{} supplies a {}-partition target-recension surface occurrence",
            gap.source_id, gap.edition, gap.passage, gap.partition
        ));
        if dictionary_matches.len() > 1 || gap.candidate_lexeme_ids.len() > 1 {
            aggregate.contradicting_evidence.insert(
                "more than one lexical identity is compatible with at least one surface".into(),
            );
        }
        if !dictionary_matches.is_empty() {
            aggregate.contradicting_evidence.insert(
                "OCS dictionary morphology is candidate evidence, not a Synodal paradigm".into(),
            );
        }
        aggregate.missing_metadata.extend(missing_metadata(gap));
        aggregate.assumptions.extend(grouping_assumptions(gap));
    }

    let mut proposals: Vec<_> = groups
        .into_iter()
        .map(|(group_key, mut aggregate)| {
            aggregate.surfaces.sort_by(|left, right| {
                Reverse(left.frequency)
                    .cmp(&Reverse(right.frequency))
                    .then_with(|| left.normalized.cmp(&right.normalized))
            });
            let token_frequency = aggregate.surfaces.iter().map(|surface| surface.frequency).sum();
            let document_frequency = aggregate.documents.len();
            let candidate_id = stable_candidate_id(&group_key, &aggregate.surfaces);
            let review = reviews.get(&candidate_id);
            if let Some(review) = review {
                if !review.linked_lexeme_id.is_empty() {
                    aggregate
                        .compatible_existing_lexemes
                        .insert(review.linked_lexeme_id.clone());
                }
            }
            FamilyProposal {
                rank: 0,
                candidate_id,
                proposed_lemma: aggregate.proposed_lemma,
                part_of_speech: aggregate.part_of_speech,
                surfaces: aggregate.surfaces,
                token_frequency,
                document_frequency,
                documents: aggregate.documents.into_iter().take(64).collect(),
                contexts: aggregate.contexts.into_values().take(12).collect(),
                corpora: aggregate.corpora.into_iter().collect(),
                source_ids: aggregate.source_ids.into_iter().collect(),
                editions: aggregate.editions.into_iter().collect(),
                passages: aggregate.passages.into_iter().collect(),
                partitions: aggregate.partitions.into_iter().collect(),
                possible_cells: aggregate.possible_cells.into_iter().collect(),
                diagnostic_features: aggregate.diagnostic_features.into_iter().collect(),
                compatible_existing_lexemes: aggregate
                    .compatible_existing_lexemes
                    .into_iter()
                    .collect(),
                dictionary_candidate_ids: aggregate
                    .dictionary_candidate_ids
                    .into_iter()
                    .collect(),
                supporting_evidence: aggregate.supporting_evidence.into_iter().collect(),
                contradicting_evidence: aggregate
                    .contradicting_evidence
                    .into_iter()
                    .collect(),
                missing_metadata: aggregate.missing_metadata.into_iter().collect(),
                confidence_basis_points: aggregate.confidence_basis_points,
                assumptions: aggregate.assumptions.into_iter().collect(),
                review_status: review.map_or("candidate-unreviewed", |review| &review.decision).into(),
                review_reason: review.map_or(
                    "surface grouping is diagnostic only; target identity and complete morphology require review",
                    |review| review.reason.as_str(),
                ).into(),
            }
        })
        .collect();
    proposals.sort_by(|left, right| {
        Reverse(left.token_frequency)
            .cmp(&Reverse(right.token_frequency))
            .then_with(|| Reverse(left.document_frequency).cmp(&Reverse(right.document_frequency)))
            .then_with(|| left.candidate_id.cmp(&right.candidate_id))
    });
    proposals.truncate(limit);
    for (index, proposal) in proposals.iter_mut().enumerate() {
        proposal.rank = index + 1;
    }
    proposals
}

fn diagnostic_family_key(
    gap: &GapRecord,
    dictionary: &BTreeMap<String, Vec<DictionaryFamily>>,
) -> String {
    let normalized = spelling_key(&gap.normalized);
    if normalized.starts_with("сын") {
        return "probable-reviewed-stem:synodal:noun:syn".into();
    }
    if normalized.starts_with("земл") {
        return "probable-reviewed-stem:synodal:noun:zemlya".into();
    }
    if normalized.starts_with("гдⷭ҇н") {
        return "abbreviation-family-proposed-adjective:господень".into();
    }
    if normalized.starts_with("гдⷭ") {
        return "abbreviation-family:synodal:noun:wikt-ed67a3345df1".into();
    }
    if matches!(normalized.as_str(), "рече" | "речеши" | "речетъ") {
        return "probable-irregular-family:synodal:verb:wikt-06af096688df".into();
    }
    if matches!(
        normalized.as_str(),
        "весь"
            | "всѧ"
            | "все"
            | "вси"
            | "всѣхъ"
            | "всѣмъ"
            | "всѣми"
            | "всю"
            | "всему"
            | "всей"
            | "всеѧ"
            | "всего"
            | "всею"
    ) {
        return "probable-determiner-family:весь".into();
    }
    let matches = dictionary.get(&normalized).cloned().unwrap_or_default();
    if !matches.is_empty() {
        return format!(
            "dictionary-family-set:{}",
            matches
                .iter()
                .map(|family| family.candidate_id.as_str())
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    if !gap.candidate_lexeme_ids.is_empty() {
        return format!(
            "known-family-set:{}",
            gap.candidate_lexeme_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    if let Some(stem) = diagnostic_stem(&normalized) {
        return format!("diagnostic-stem:{stem}");
    }
    format!("ungrouped-surface:{normalized}")
}

fn diagnostic_features(
    gap: &GapRecord,
    dictionary_matches: &[DictionaryFamily],
) -> BTreeSet<String> {
    let normalized = spelling_key(&gap.normalized);
    let mut features = BTreeSet::from([
        format!("accentless-positional-key:{normalized}"),
        format!("context-samples:{}", gap.contexts.len()),
    ]);
    if let Some(prefix) = visible_prefix(&normalized) {
        features.insert(format!("possible-prefix:{prefix}"));
    }
    if let Some((stem, ending)) = split_inflectional_ending(&normalized) {
        features.insert(format!("possible-stem:{stem}"));
        features.insert(format!("possible-ending:{ending}"));
    }
    if has_abbreviation_marks(&gap.original) {
        features.insert("titlo-or-superscript-pattern".into());
    }
    if !dictionary_matches.is_empty() {
        features.insert("dictionary-form-membership".into());
    }
    if !gap.candidate_lexeme_ids.is_empty() {
        features.insert("reviewed-runtime-candidate".into());
    }
    features
}

fn diagnostic_stem(value: &str) -> Option<String> {
    let (stem, _) = split_inflectional_ending(value)?;
    (stem.chars().count() >= 3).then(|| stem.into())
}

fn split_inflectional_ending(value: &str) -> Option<(&str, &str)> {
    const ENDINGS: [&str; 40] = [
        "овѣхъ",
        "овомъ",
        "еви",
        "овъ",
        "ѣхъ",
        "ихъ",
        "ѧми",
        "ами",
        "ьми",
        "ѧмъ",
        "амъ",
        "омъ",
        "емъ",
        "егѡ",
        "ого",
        "аго",
        "емꙋ",
        "омꙋ",
        "иши",
        "еши",
        "итъ",
        "етъ",
        "ѧтъ",
        "утъ",
        "ихъ",
        "иша",
        "оша",
        "ити",
        "ати",
        "ѣти",
        "ти",
        "ма",
        "ми",
        "ѧ",
        "а",
        "ы",
        "и",
        "е",
        "ꙋ",
        "ю",
    ];
    ENDINGS.iter().find_map(|ending| {
        value
            .strip_suffix(ending)
            .and_then(|stem| (stem.chars().count() >= 3).then_some((stem, *ending)))
    })
}

fn visible_prefix(value: &str) -> Option<&'static str> {
    const PREFIXES: [&str; 20] = [
        "пред", "пре", "при", "про", "воз", "вос", "раз", "рас", "из", "ис", "вз", "съ", "со",
        "под", "над", "от", "ѿ", "об", "на", "по",
    ];
    PREFIXES.into_iter().find(|prefix| {
        value
            .strip_prefix(prefix)
            .is_some_and(|rest| rest.chars().count() >= 3)
    })
}

fn has_abbreviation_marks(value: &str) -> bool {
    value.chars().any(|character| {
        matches!(character, '\u{0483}' | '\u{0487}')
            || ('\u{2de0}'..='\u{2dff}').contains(&character)
    })
}

fn proposal_identity(
    gap: &GapRecord,
    dictionary_matches: &[DictionaryFamily],
    family_key: &str,
) -> (String, String, u16) {
    let key = family_key;
    if key.contains("synodal:noun:syn") {
        return ("сынъ".into(), "noun".into(), 7_500);
    }
    if key.contains("synodal:noun:zemlya") {
        return ("землѧ".into(), "noun".into(), 8_500);
    }
    if key.starts_with("abbreviation-family") {
        if key.ends_with("господень") {
            return ("господень".into(), "adjective".into(), 8_500);
        }
        return ("господь".into(), "noun".into(), 9_500);
    }
    if key.contains("synodal:verb:wikt-06af096688df") {
        return ("рещи".into(), "verb".into(), 7_500);
    }
    if key.ends_with("весь") {
        return ("весь".into(), "determiner".into(), 8_000);
    }
    if dictionary_matches.len() == 1 {
        return (
            dictionary_matches[0].lemma.clone(),
            dictionary_matches[0].part_of_speech.clone(),
            5_500,
        );
    }
    if dictionary_matches.len() > 1 {
        return (
            dictionary_matches
                .iter()
                .map(|family| family.lemma.as_str())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(" / "),
            "ambiguous".into(),
            2_500,
        );
    }
    if let Some(stem) = key.strip_prefix("diagnostic-stem:") {
        return (format!("{stem}-"), "unknown".into(), 1_500);
    }
    (gap.normalized.clone(), "unknown".into(), 1_000)
}

fn possible_cells(gap: &GapRecord, dictionary_matches: &[DictionaryFamily]) -> Vec<String> {
    let mut cells = BTreeSet::new();
    for family in dictionary_matches {
        if let Some(tags) = family.forms.get(&spelling_key(&gap.normalized)) {
            if !tags.is_empty() {
                cells.insert(tags.iter().cloned().collect::<Vec<_>>().join("+"));
            }
        }
    }
    let normalized = spelling_key(&gap.normalized);
    for cell in ending_hypotheses(&normalized) {
        cells.insert(cell.into());
    }
    if cells.is_empty() {
        cells.insert("untyped".into());
    }
    cells.into_iter().collect()
}

fn ending_hypotheses(value: &str) -> Vec<&'static str> {
    let mut cells = Vec::new();
    if value.ends_with("ове") || value.ends_with("и") || value.ends_with("ы") {
        cells.push("possible-nominative-plural");
    }
    if value.ends_with("овъ") || value.ends_with("ѣхъ") || value.ends_with("ихъ") {
        cells.push("possible-genitive-or-locative-plural");
    }
    if value.ends_with("омъ") || value.ends_with("ѧмъ") {
        cells.push("possible-dative-plural-or-instrumental-singular");
    }
    if value.ends_with("а") || value.ends_with("ѧ") {
        cells.push("possible-nominative-feminine-or-genitive-accusative");
    }
    if value.ends_with("е") {
        cells.push("possible-vocative-or-finite-verb");
    }
    if value.ends_with("ши") || value.ends_with("тъ") {
        cells.push("possible-finite-verb");
    }
    cells
}

fn missing_metadata(gap: &GapRecord) -> BTreeSet<String> {
    let mut fields: BTreeSet<String> = gap
        .missing_metadata
        .iter()
        .map(|field| format!("{field:?}").to_lowercase())
        .collect();
    match gap.kind {
        GapKind::UnknownLexeme => {
            fields.insert("target-lexeme-identity".into());
            fields.insert("target-morphological-cell".into());
        }
        GapKind::MissingDeclensionOrClass => {
            fields.insert("declension-or-conjugation-class".into());
        }
        GapKind::MissingVerbPrincipalPart => {
            fields.insert("independent-principal-part".into());
        }
        GapKind::UnsupportedFormation => {
            fields.insert("normative-formation-rule-or-exact-table".into());
        }
        GapKind::MissingAccentOrOrthographicMetadata => {
            fields.insert("accent-or-printed-orthography".into());
        }
        GapKind::AmbiguityOrSpellingVariant => {
            fields.insert("identity-or-spelling-variant-decision".into());
        }
    }
    fields
}

fn grouping_assumptions(gap: &GapRecord) -> BTreeSet<String> {
    let mut assumptions = BTreeSet::new();
    assumptions
        .insert("surface similarity is a proposal and does not establish lexical identity".into());
    if gap.normalized.starts_with("гдⷭ") {
        assumptions.insert("titlo pattern is compatible with a господь contraction".into());
    }
    if gap.normalized.starts_with("сын") || gap.normalized.starts_with("земл") {
        assumptions
            .insert("shared visible stem is diagnostic only; alternants require review".into());
    }
    assumptions
}

fn stable_candidate_id(group_key: &str, surfaces: &[FamilySurface]) -> String {
    let mut digest = Sha256::new();
    digest.update(group_key.as_bytes());
    let _ = surfaces;
    let hex = format!("{:x}", digest.finalize());
    format!("synodal:family-candidate:{}", &hex[..24])
}

fn map_pos(value: &str) -> Option<&'static str> {
    Some(match value {
        "noun" => "noun",
        "name" => "proper-noun",
        "adj" => "adjective",
        "verb" => "verb",
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

fn ud_pos_matches_runtime(
    ud_pos: &str,
    runtime_pos: synodal_church_slavonic::PartOfSpeech,
) -> bool {
    use synodal_church_slavonic::PartOfSpeech as RuntimePos;

    match ud_pos {
        "NOUN" => matches!(runtime_pos, RuntimePos::Noun | RuntimePos::ProperNoun),
        "PROPN" => runtime_pos == RuntimePos::ProperNoun,
        "VERB" | "AUX" => runtime_pos == RuntimePos::Verb,
        "ADJ" => matches!(runtime_pos, RuntimePos::Adjective | RuntimePos::Participle),
        // PROIEL uses DET for possessive and demonstrative forms represented
        // by either public runtime category. Keep both visible for review.
        "DET" => matches!(runtime_pos, RuntimePos::Determiner | RuntimePos::Pronoun),
        "PRON" => matches!(runtime_pos, RuntimePos::Pronoun | RuntimePos::Determiner),
        "ADV" => runtime_pos == RuntimePos::Adverb,
        "ADP" => runtime_pos == RuntimePos::Preposition,
        "CCONJ" | "SCONJ" => runtime_pos == RuntimePos::Conjunction,
        "PART" => runtime_pos == RuntimePos::Particle,
        "INTJ" => runtime_pos == RuntimePos::Interjection,
        "NUM" => runtime_pos == RuntimePos::Numeral,
        _ => false,
    }
}

fn spelling_key(value: &str) -> String {
    normalize_lookup_accentless(value)
        .to_lowercase()
        .replace(['і', 'ї'], "и")
        .replace(['ѡ', 'ѻ'], "о")
        .replace('ѿ', "от")
        .replace("ᲂу", "у")
        .replace('ꙋ', "у")
}

fn sanitize(value: &str) -> String {
    value
        .replace(['\t', '\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn proposal_tsv(proposals: &[FamilyProposal]) -> String {
    let mut output = String::from(
        "rank\tcandidate_id\tproposed_lemma\tpart_of_speech\tsurfaces\ttoken_frequency\tdocument_frequency\tdocuments\tcontexts\tcorpora\tsource_ids\teditions\tpassages\tpartitions\tpossible_cells\tdiagnostic_features\tcompatible_existing_lexemes\tdictionary_candidate_ids\tsupporting_evidence\tcontradicting_evidence\tmissing_metadata\tconfidence_bp\tassumptions\treview_status\treview_reason\n",
    );
    for proposal in proposals {
        let surfaces = proposal
            .surfaces
            .iter()
            .map(|surface| format!("{}:{}", surface.original, surface.frequency))
            .collect::<Vec<_>>()
            .join(" | ");
        output.push_str(
            &[
                proposal.rank.to_string(),
                proposal.candidate_id.clone(),
                proposal.proposed_lemma.clone(),
                proposal.part_of_speech.clone(),
                surfaces,
                proposal.token_frequency.to_string(),
                proposal.document_frequency.to_string(),
                proposal.documents.join(","),
                proposal
                    .contexts
                    .iter()
                    .map(|context| {
                        format!(
                            "{}:{}:{} {}",
                            context.passage, context.line, context.column, context.excerpt
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" | "),
                proposal.corpora.join(","),
                proposal.source_ids.join(","),
                proposal.editions.join(","),
                proposal.passages.join(","),
                proposal.partitions.join(","),
                proposal.possible_cells.join(","),
                proposal.diagnostic_features.join(","),
                proposal.compatible_existing_lexemes.join(","),
                proposal.dictionary_candidate_ids.join(","),
                proposal.supporting_evidence.join(" | "),
                proposal.contradicting_evidence.join(" | "),
                proposal.missing_metadata.join(","),
                proposal.confidence_basis_points.to_string(),
                proposal.assumptions.join(" | "),
                proposal.review_status.clone(),
                proposal.review_reason.clone(),
            ]
            .into_iter()
            .map(|value| sanitize(&value))
            .collect::<Vec<_>>()
            .join("\t"),
        );
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_ids_are_stable_and_surface_order_independent() {
        let surface = FamilySurface {
            original: "сы́нове".into(),
            normalized: "сынове".into(),
            frequency: 1,
            document_frequency: 1,
            possible_cells: vec!["possible-nominative-plural".into()],
            corpus: "fixture".into(),
            source_id: "fixture".into(),
            edition: "fixture".into(),
            passage: "fixture".into(),
            partition: "source".into(),
        };
        let changed_frequency = FamilySurface {
            frequency: 99,
            ..surface.clone()
        };
        assert_eq!(
            stable_candidate_id("сынъ", std::slice::from_ref(&surface)),
            stable_candidate_id("сынъ", std::slice::from_ref(&changed_frequency))
        );
        let first = FamilySurface {
            normalized: "сынови".into(),
            ..surface.clone()
        };
        let second = FamilySurface {
            normalized: "сынове".into(),
            ..surface
        };
        assert_eq!(
            stable_candidate_id("сынъ", &[first.clone(), second.clone()]),
            stable_candidate_id("сынъ", &[second, first])
        );
    }

    #[test]
    fn ending_hypotheses_do_not_assert_identity() {
        assert!(ending_hypotheses("сынове").contains(&"possible-nominative-plural"));
        assert!(
            grouping_assumptions(
                &serde_json::from_value(serde_json::json!({
                    "kind": "unknown-lexeme",
                    "original": "сы́нове",
                    "normalized": "сынове",
                    "corpus": "fixture",
                    "source_id": "fixture",
                    "work": "fixture",
                    "edition": "fixture",
                    "passage": "fixture",
                    "partition": "source",
                    "source_recension": "synodal-russian",
                    "corpora": ["fixture"],
                    "source_ids": ["fixture"],
                    "editions": ["fixture"],
                    "partitions": ["source"],
                    "source_recensions": ["synodal-russian"],
                    "byte_start": 0,
                    "byte_end": 1,
                    "line": 1,
                    "column": 1,
                    "candidate_lexeme_ids": [],
                    "requested_morphological_system": null,
                    "generation_policy": "Strict",
                    "orthography_profile": "SynodalLiturgical",
                    "resolver_trace": {"steps": []},
                    "missing_metadata": [],
                    "secondary_reasons": [],
                    "detail": "fixture",
                    "frequency": 1,
                    "document_frequency": 1,
                    "suggested_action": "fixture"
                }))
                .expect("gap")
            )
            .iter()
            .any(|assumption| assumption.contains("does not establish"))
        );
    }

    #[test]
    fn productive_adjective_family_does_not_require_an_exact_member() {
        let family = show_family_by_id(&FamilyId::for_lexeme(
            &synodal_church_slavonic_core::LexemeId::from("synodal:adjective:mertv"),
        ))
        .expect("reviewed productive adjective family");

        assert!(family.fully_classed);
        assert!(!family.exact_only);
        assert_eq!(family.class.as_deref(), Some("hard-short"));
        assert!(family.members.is_empty());
        assert!(
            family
                .supported_systems
                .iter()
                .any(|system| system == "adjective")
        );
    }

    #[test]
    fn exact_indeclinable_family_labels_require_matching_pos_and_cell() {
        let family = show_family_by_id(&FamilyId::for_lexeme(
            &synodal_church_slavonic_core::LexemeId::from("synodal:adverb:dokole"),
        ))
        .expect("reviewed exact indeclinable adverb");
        assert!(exact_indeclinable_family_matches(
            &family,
            synodal_church_slavonic::PartOfSpeech::Adverb
        ));
        assert!(!exact_indeclinable_family_matches(
            &family,
            synodal_church_slavonic::PartOfSpeech::Preposition
        ));

        let mut wrong_cell = family;
        wrong_cell.members[0].cell = "lexical-form".into();
        assert!(!exact_indeclinable_family_matches(
            &wrong_cell,
            synodal_church_slavonic::PartOfSpeech::Adverb
        ));

        let reviews = BTreeMap::from([(
            "fixture:wrong-pos".into(),
            ReviewDecision {
                decision: "admitted".into(),
                linked_lexeme_id: "synodal:adverb:dokole".into(),
                admitted_class: "exact-indeclinable-preposition".into(),
                reason: "fixture".into(),
            },
        )]);
        assert!(validate_admitted_families(&reviews).is_err());
    }
}
