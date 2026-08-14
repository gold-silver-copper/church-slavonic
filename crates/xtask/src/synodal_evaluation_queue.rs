use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

use crate::report_io::{check_contents, write_if_changed};
use serde::{Deserialize, Serialize};
use synodal_church_slavonic::{
    FiniteTense, FormSource, GenerationPolicy, GrammarCell, Inflector, OrthographyProfile,
    ParticipleTense, ParticipleVoice, grammar_cell_key, lexemes,
};
use synodal_church_slavonic_dictionary::{candidate_cells, coverage::tokenize};
use unicode_normalization::UnicodeNormalization;

const SOURCE: &str = "ponomar-elizabeth-bible-2026-08-09";

#[derive(Clone, Debug, Deserialize)]
struct PassageCandidate {
    source_id: String,
    target_recension: Option<String>,
    passage: String,
    normalized_spelling: String,
    partition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Proposal {
    rank: usize,
    lexeme_id: String,
    cell: String,
    system: String,
    policy: String,
    expected_expanded: String,
    expected_printed: String,
    source_id: String,
    passage: String,
    provenance: String,
    decision: String,
    review_reason: String,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut limit = 2_000_usize;
    let mut check = false;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--limit" => limit = args.next().ok_or("--limit needs a number")?.parse()?,
            "--check" => check = true,
            value => {
                return Err(format!("unknown synodal-evaluation-queue argument {value:?}").into());
            }
        }
    }
    let proposals = build(root, limit)?;
    let tsv = to_tsv(&proposals);
    let json = format!("{}\n", serde_json::to_string_pretty(&proposals)?);
    let tsv_path = root.join("reports/synodal-evaluation-review-queue.tsv");
    let json_path = root.join("reports/synodal-evaluation-review-queue.json");
    if check {
        check_contents(&tsv_path, &tsv)?;
        check_contents(&json_path, &json)?;
    } else {
        write_if_changed(&tsv_path, &tsv)?;
        write_if_changed(&json_path, &json)?;
    }
    println!(
        "Synodal evaluation review queue: {} passage-disjoint cells",
        proposals.len()
    );
    Ok(())
}

fn build(root: &Path, limit: usize) -> Result<Vec<Proposal>, Box<dyn Error>> {
    let forbidden = source_passages(root)?;
    let token_passages = evaluation_tokens(
        &root.join(format!("data/intermediate/synodal/{SOURCE}.jsonl")),
        &forbidden,
    )?;
    let existing = existing_cells(&root.join("data/synodal/evaluation.tsv"))?;
    let policies = [
        (GenerationPolicy::Strict, "strict"),
        (GenerationPolicy::Productive, "productive"),
        (GenerationPolicy::Exploratory, "exploratory"),
    ];
    let mut proposals = Vec::new();
    for lexeme in lexemes()? {
        if lexeme.id().as_str().contains(":wikt-") {
            continue;
        }
        for cell in candidate_cells(lexeme.part_of_speech()) {
            if matches!(cell, GrammarCell::LexicalForm | GrammarCell::Indeclinable) {
                continue;
            }
            let cell_key = grammar_cell_key(cell);
            if existing.contains(&(lexeme.id().to_string(), cell_key.clone())) {
                continue;
            }
            let mut admitted = None;
            for (policy, policy_name) in policies {
                let result = Inflector::builder()
                    .generation_policy(policy)
                    .orthography(OrthographyProfile::SynodalLiturgical)
                    .build()
                    .form_by_id(lexeme.id(), cell);
                let Ok(forms) = result else {
                    continue;
                };
                for variant in forms.variants() {
                    let key: String = variant.printed.nfc().collect();
                    let Some(passages) = token_passages.get(&key) else {
                        continue;
                    };
                    let Some(passage) = passages.iter().next() else {
                        continue;
                    };
                    admitted = Some(Proposal {
                        rank: 0,
                        lexeme_id: lexeme.id().to_string(),
                        cell: cell_key.clone(),
                        system: morphological_system(cell).into(),
                        policy: policy_name.into(),
                        expected_expanded: variant.expanded.clone(),
                        expected_printed: variant.printed.clone(),
                        source_id: SOURCE.into(),
                        passage: passage.clone(),
                        provenance: provenance(&variant.source).into(),
                        decision: "candidate-unreviewed".into(),
                        review_reason: "requires context review and confirmation that the surface uniquely identifies this cell".into(),
                    });
                    break;
                }
                if admitted.is_some() {
                    break;
                }
            }
            if let Some(proposal) = admitted {
                proposals.push(proposal);
            }
        }
    }
    let competing_cells: BTreeMap<_, BTreeSet<_>> = proposals
        .iter()
        .map(|proposal| {
            (
                (
                    proposal.lexeme_id.clone(),
                    proposal.expected_printed.clone(),
                ),
                proposal.cell.clone(),
            )
        })
        .fold(BTreeMap::new(), |mut groups, (key, cell)| {
            groups.entry(key).or_default().insert(cell);
            groups
        });
    for proposal in &mut proposals {
        let cells = &competing_cells[&(
            proposal.lexeme_id.clone(),
            proposal.expected_printed.clone(),
        )];
        if cells.len() > 1 {
            proposal.decision = "blocked-cell-ambiguity".into();
            proposal.review_reason = format!(
                "the surface is generated for multiple cells: {}",
                cells.iter().cloned().collect::<Vec<_>>().join(",")
            );
        }
    }
    proposals.sort_by_key(|proposal| {
        (
            proposal.system.clone(),
            proposal.lexeme_id.clone(),
            proposal.cell.clone(),
            Reverse(proposal.policy.clone()),
            proposal.passage.clone(),
        )
    });
    proposals.truncate(limit);
    for (index, proposal) in proposals.iter_mut().enumerate() {
        proposal.rank = index + 1;
    }
    Ok(proposals)
}

fn evaluation_tokens(
    path: &Path,
    forbidden: &BTreeSet<(String, String)>,
) -> Result<BTreeMap<String, BTreeSet<String>>, Box<dyn Error>> {
    let mut tokens = BTreeMap::<String, BTreeSet<String>>::new();
    for line in fs::read_to_string(path)?.lines() {
        let candidate: PassageCandidate = serde_json::from_str(line)?;
        if candidate.source_id != SOURCE
            || candidate.partition != "evaluation"
            || candidate.target_recension.as_deref() != Some("synodal-russian")
            || forbidden.contains(&(candidate.source_id.clone(), candidate.passage.clone()))
        {
            continue;
        }
        for token in tokenize(&candidate.normalized_spelling) {
            tokens
                .entry(token.original.nfc().collect())
                .or_default()
                .insert(candidate.passage.clone());
        }
    }
    Ok(tokens)
}

fn source_passages(root: &Path) -> Result<BTreeSet<(String, String)>, Box<dyn Error>> {
    let mut passages = BTreeSet::new();
    for line in fs::read_to_string(root.join("data/synodal/training_passages.tsv"))?
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() == 6 {
            passages.insert((fields[0].into(), fields[1].into()));
        }
    }
    for line in fs::read_to_string(root.join("data/synodal/lexical_reviews.tsv"))?
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() == 18 && fields[15] == "reviewed" {
            passages.insert((fields[12].into(), fields[14].into()));
        }
    }
    Ok(passages)
}

fn existing_cells(path: &Path) -> Result<BTreeSet<(String, String)>, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            (fields.len() == 9).then(|| (fields[1].into(), fields[2].into()))
        })
        .collect())
}

fn morphological_system(cell: GrammarCell) -> &'static str {
    match cell {
        GrammarCell::LexicalForm => "lexical-form",
        GrammarCell::Indeclinable => "indeclinable",
        GrammarCell::Noun(_) => "noun",
        GrammarCell::Adjective(cell) => match cell.comparison {
            synodal_church_slavonic::Comparison::Positive => "adjective",
            synodal_church_slavonic::Comparison::Comparative => "comparison",
            synodal_church_slavonic::Comparison::Superlative => "superlative",
        },
        GrammarCell::FiniteVerb(cell) => match cell.tense {
            FiniteTense::Present => "present",
            FiniteTense::Future => "future",
            FiniteTense::Past => "past",
            FiniteTense::Imperfect => "imperfect",
            FiniteTense::Aorist => "aorist",
        },
        GrammarCell::Imperative(_) => "imperative",
        GrammarCell::Infinitive => "infinitive",
        GrammarCell::Supine => "supine",
        GrammarCell::LParticiple(_) => "l-participle",
        GrammarCell::Participle(cell) => match (cell.tense, cell.voice) {
            (ParticipleTense::Present, ParticipleVoice::Active) => "present-active-participle",
            (ParticipleTense::Past, ParticipleVoice::Active) => "past-active-participle",
            (ParticipleTense::Present, ParticipleVoice::Passive) => "present-passive-participle",
            (ParticipleTense::Past, ParticipleVoice::Passive) => "past-passive-participle",
        },
        GrammarCell::VerbalNoun(_) => "verbal-noun",
        GrammarCell::Pronoun(_) => "pronoun",
        GrammarCell::Determiner(_) => "determiner",
        GrammarCell::Numeral(_) => "numeral",
    }
}

fn provenance(source: &FormSource) -> &'static str {
    match source {
        FormSource::SynodalAttestation { .. } => "exact-attestation",
        FormSource::SynodalIrregularOverride { .. } => "irregular-override",
        FormSource::SynodalNormativeGeneration { .. } => "normative",
        FormSource::CallerSpecifiedPrediction { .. } => "caller-specified",
        FormSource::InheritedPrediction { .. } => "reviewed-inheritance",
        FormSource::AnalogicalPrediction { .. } => "analogical",
    }
}

fn to_tsv(proposals: &[Proposal]) -> String {
    let mut output = String::from(
        "rank\tlexeme_id\tcell\tsystem\tpolicy\texpected_expanded\texpected_printed\tsource_id\tpassage\tprovenance\tdecision\treview_reason\n",
    );
    for row in proposals {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.rank,
            row.lexeme_id,
            row.cell,
            row.system,
            row.policy,
            row.expected_expanded,
            row.expected_printed,
            row.source_id,
            row.passage,
            row.provenance,
            row.decision,
            row.review_reason,
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_supported_systems_without_generic_other_bucket() {
        assert_eq!(morphological_system(GrammarCell::Infinitive), "infinitive");
        assert_eq!(morphological_system(GrammarCell::Supine), "supine");
    }
}
