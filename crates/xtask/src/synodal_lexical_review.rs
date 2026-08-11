use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use synodal_church_slavonic_core::normalize_lookup_accentless;
use synodal_church_slavonic_dictionary::coverage::tokenize;
use unicode_normalization::UnicodeNormalization;

const SEMANTIC_SOURCE: &str = "english-wiktionary-ocs-kaikki-2026-08-07";
const ATTESTATION_SOURCE: &str = "ponomar-elizabeth-bible-2026-08-09";
type FrequencyIndex = (BTreeMap<String, usize>, BTreeMap<String, String>);

#[derive(Clone, Debug, Deserialize)]
struct Candidate {
    candidate_id: String,
    source_id: String,
    target_recension: Option<String>,
    passage: String,
    normalized_spelling: String,
    partition: String,
    raw_spelling: String,
}

#[derive(Clone, Debug, Deserialize)]
struct WiktionaryEntry {
    word: String,
    pos: String,
    #[serde(default)]
    senses: Vec<WiktionarySense>,
    #[serde(default)]
    forms: Vec<WiktionaryForm>,
}

#[derive(Clone, Debug, Deserialize)]
struct WiktionarySense {
    #[serde(default)]
    glosses: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct WiktionaryForm {
    form: String,
}

#[derive(Clone, Debug)]
struct Attestation {
    candidate_id: String,
    passage: String,
    printed: String,
    preferred: bool,
    context: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Proposal {
    rank: usize,
    frequency: usize,
    lemma: String,
    printed: String,
    part_of_speech: String,
    cell: String,
    gloss: String,
    semantic_candidate_id: String,
    attestation_candidate_id: String,
    passage: String,
    context: String,
    decision: String,
    review_reason: String,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut intermediate = root.join("data/intermediate/synodal");
    let mut check = false;
    let mut limit = 1_000_usize;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--intermediate" => {
                intermediate = PathBuf::from(args.next().ok_or("--intermediate needs a path")?);
            }
            "--limit" => limit = args.next().ok_or("--limit needs a number")?.parse()?,
            "--check" => check = true,
            value => {
                return Err(
                    format!("unknown synodal-lexical-review-queue argument {value:?}").into(),
                );
            }
        }
    }
    let proposals = proposals(root, &intermediate, limit)?;
    let tsv = proposal_tsv(&proposals);
    let json = format!("{}\n", serde_json::to_string_pretty(&proposals)?);
    let tsv_path = root.join("reports/synodal-lexical-review-queue.tsv");
    let json_path = root.join("reports/synodal-lexical-review-queue.json");
    if check {
        check_contents(&tsv_path, &tsv)?;
        check_contents(&json_path, &json)?;
    } else {
        write_if_changed(&tsv_path, &tsv)?;
        write_if_changed(&json_path, &json)?;
    }
    println!(
        "Synodal lexical review queue: {} preserved cross-source proposals",
        proposals.len()
    );
    Ok(())
}

fn proposals(
    root: &Path,
    intermediate: &Path,
    limit: usize,
) -> Result<Vec<Proposal>, Box<dyn Error>> {
    let (frequencies, preferred) = load_frequencies(&intermediate.join("ponomar-frequency.tsv"))?;
    let attestations = load_attestations(
        &intermediate.join(format!("{ATTESTATION_SOURCE}.jsonl")),
        &frequencies,
        &preferred,
    )?;
    let mut existing = existing_lexemes(&root.join("data/synodal/lexemes.tsv"))?;
    existing.extend(reviewed_lexemes(
        &root.join("data/synodal/lexical_reviews.tsv"),
    )?);
    let semantic_text = fs::read_to_string(intermediate.join(format!("{SEMANTIC_SOURCE}.jsonl")))?;
    let mut entries = Vec::new();
    let mut form_owners = BTreeMap::<String, BTreeSet<(String, String)>>::new();
    for line in semantic_text.lines() {
        let candidate: Candidate = serde_json::from_str(line)?;
        let entry: WiktionaryEntry = serde_json::from_str(&candidate.raw_spelling)?;
        let owner = (spelling_key(&entry.word), entry.pos.clone());
        for form in std::iter::once(entry.word.as_str())
            .chain(entry.forms.iter().map(|form| form.form.as_str()))
            .filter(|form| form.chars().any(char::is_alphabetic))
        {
            form_owners
                .entry(spelling_key(form))
                .or_default()
                .insert(owner.clone());
        }
        entries.push((candidate, entry));
    }
    let mut selected = BTreeMap::<(String, String), Proposal>::new();
    for (candidate, entry) in entries {
        let Some(part_of_speech) = map_pos(&entry.pos) else {
            continue;
        };
        let key = spelling_key(&entry.word);
        let (Some(&frequency), Some(attestation)) = (frequencies.get(&key), attestations.get(&key))
        else {
            continue;
        };
        let glosses: BTreeSet<_> = entry
            .senses
            .iter()
            .flat_map(|sense| &sense.glosses)
            .map(|gloss| sanitize(gloss))
            .filter(|gloss| !gloss.is_empty() && !is_derived_form_or_unsafe_homograph(gloss))
            .collect();
        if glosses.is_empty() {
            continue;
        }
        let gloss = glosses.into_iter().collect::<Vec<_>>().join("; ");
        let lemma = normalize_lookup_accentless(&attestation.printed);
        if existing.contains(&(spelling_key(&lemma), part_of_speech.to_owned())) {
            continue;
        }
        let group = (spelling_key(&lemma), part_of_speech.to_owned());
        let ambiguous_homograph = form_owners.get(&key).is_some_and(|owners| owners.len() > 1);
        let (decision, review_reason) = if ambiguous_homograph && !is_closed_class(part_of_speech) {
            (
                "blocked-ambiguous-homograph".into(),
                "surface belongs to multiple OCS lexeme paradigms; target identity must be resolved manually".into(),
            )
        } else {
            (
                "candidate-unreviewed".into(),
                "requires human confirmation of target identity, sense continuity, and morphological status".into(),
            )
        };
        let proposal = Proposal {
            rank: 0,
            frequency,
            lemma,
            printed: attestation.printed.clone(),
            part_of_speech: part_of_speech.into(),
            cell: if is_closed_class(part_of_speech) {
                "indeclinable".into()
            } else {
                "lexical-form".into()
            },
            gloss,
            semantic_candidate_id: candidate.candidate_id,
            attestation_candidate_id: attestation.candidate_id.clone(),
            passage: attestation.passage.clone(),
            context: sanitize(&attestation.context),
            decision,
            review_reason,
        };
        let replace = selected.get(&group).is_none_or(|current| {
            (proposal.gloss.len(), &proposal.semantic_candidate_id)
                < (current.gloss.len(), &current.semantic_candidate_id)
        });
        if replace {
            selected.insert(group, proposal);
        }
    }
    let mut proposals: Vec<_> = selected.into_values().collect();
    proposals.sort_by_key(|proposal| {
        (
            Reverse(proposal.frequency),
            proposal.part_of_speech.clone(),
            proposal.lemma.clone(),
            proposal.semantic_candidate_id.clone(),
        )
    });
    proposals.truncate(limit);
    for (index, proposal) in proposals.iter_mut().enumerate() {
        proposal.rank = index + 1;
    }
    Ok(proposals)
}

fn load_frequencies(path: &Path) -> Result<FrequencyIndex, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut frequencies = BTreeMap::new();
    let mut variants = BTreeMap::<String, Vec<(usize, String)>>::new();
    for line in text.lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 3 {
            return Err(format!("invalid frequency row {line:?}").into());
        }
        let count = fields[2].parse::<usize>()?;
        let key = spelling_key(fields[1]);
        *frequencies.entry(key.clone()).or_default() += count;
        variants
            .entry(key)
            .or_default()
            .push((count, fields[1].into()));
    }
    let preferred = variants
        .into_iter()
        .map(|(key, mut variants)| {
            variants.sort_by_key(|(count, text)| (Reverse(*count), text.clone()));
            (key, variants.remove(0).1)
        })
        .collect();
    Ok((frequencies, preferred))
}

fn load_attestations(
    path: &Path,
    frequencies: &BTreeMap<String, usize>,
    preferred: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Attestation>, Box<dyn Error>> {
    let mut attestations = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let candidate: Candidate = serde_json::from_str(line)?;
        if candidate.source_id != ATTESTATION_SOURCE
            || candidate.partition != "source"
            || candidate.target_recension.as_deref() != Some("synodal-russian")
        {
            continue;
        }
        for token in tokenize(&candidate.normalized_spelling) {
            let key = spelling_key(&token.original);
            if !frequencies.contains_key(&key) {
                continue;
            }
            let preferred_match = preferred
                .get(&key)
                .is_some_and(|value| value.nfc().eq(token.original.nfc()));
            let attestation = Attestation {
                candidate_id: candidate.candidate_id.clone(),
                passage: candidate.passage.clone(),
                printed: token.original,
                preferred: preferred_match,
                context: candidate.normalized_spelling.clone(),
            };
            if attestations
                .get(&key)
                .is_none_or(|current: &Attestation| !current.preferred && preferred_match)
            {
                attestations.insert(key, attestation);
            }
        }
    }
    Ok(attestations)
}

fn existing_lexemes(path: &Path) -> Result<BTreeSet<(String, String)>, Box<dyn Error>> {
    let mut existing = BTreeSet::new();
    for line in fs::read_to_string(path)?.lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() >= 3 {
            existing.insert((spelling_key(fields[1]), fields[2].into()));
        }
    }
    Ok(existing)
}

fn reviewed_lexemes(path: &Path) -> Result<BTreeSet<(String, String)>, Box<dyn Error>> {
    let mut existing = BTreeSet::new();
    for line in fs::read_to_string(path)?.lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() == 18 && fields[15] == "reviewed" {
            existing.insert((spelling_key(fields[3]), fields[4].into()));
        }
    }
    Ok(existing)
}

fn spelling_key(value: &str) -> String {
    normalize_lookup_accentless(value)
        .to_lowercase()
        .replace(['і', 'ї'], "и")
        .replace('ѡ', "о")
        .replace('ѿ', "от")
        .replace("ᲂу", "у")
        .replace('ꙋ', "у")
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

fn is_closed_class(value: &str) -> bool {
    matches!(
        value,
        "adverb" | "preposition" | "conjunction" | "particle" | "interjection"
    )
}

fn sanitize(value: &str) -> String {
    value
        .replace(['\t', '\r', '\n'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_derived_form_or_unsafe_homograph(gloss: &str) -> bool {
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
        || ([
            "nominative",
            "accusative",
            "genitive",
            "dative",
            "instrumental",
            "locative",
            "vocative",
            "singular",
            "plural",
            "dual",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
            && lower.contains(" of "))
}

fn proposal_tsv(proposals: &[Proposal]) -> String {
    let mut output = String::from(
        "rank\tfrequency\tlemma\tprinted\tpart_of_speech\tcell\tgloss\tsemantic_candidate_id\tattestation_candidate_id\tpassage\tcontext\tdecision\treview_reason\n",
    );
    for proposal in proposals {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            proposal.rank,
            proposal.frequency,
            proposal.lemma,
            proposal.printed,
            proposal.part_of_speech,
            proposal.cell,
            proposal.gloss,
            proposal.semantic_candidate_id,
            proposal.attestation_candidate_id,
            proposal.passage,
            proposal.context,
            proposal.decision,
            proposal.review_reason,
        ));
    }
    output
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
        Err(format!("{} is stale", path.display()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_inflected_entries_but_not_ordinary_glosses() {
        assert!(is_derived_form_or_unsafe_homograph(
            "genitive plural of слово"
        ));
        assert!(is_derived_form_or_unsafe_homograph(
            "alternative form of отъ"
        ));
        assert!(!is_derived_form_or_unsafe_homograph("word, speech"));
    }

    #[test]
    fn spelling_key_normalizes_marks_and_positional_letters() {
        assert_eq!(spelling_key("ꙗ҆́кѡ"), spelling_key("ꙗко"));
        assert_eq!(spelling_key("и҆"), spelling_key("і"));
    }
}
