use crate::ocs::extract::{canonical_lemma, load_registry};
use crate::ocs::normalize::lookup_key;
use crate::ocs::schema::{DictionaryExampleRow, DictionarySenseRow, Entry, Registry};
use crate::shared::{atomic_write_batch, sha256_file};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

const DICTIONARY_SCHEMA: u32 = 1;
const MAX_PARSE_FAILURE_FRACTION: f64 = 0.001;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct DictionarySourceMetadata {
    schema_version: u32,
    input_file: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct DictionaryCoverage {
    schema_version: u32,
    source_entries: usize,
    ocs_entries: usize,
    parse_failures: usize,
    lexical_senses: usize,
    linked_inflection_senses: usize,
    senses_with_examples: usize,
    examples: usize,
    senses_by_part_of_speech: BTreeMap<String, usize>,
}

impl DictionaryCoverage {
    fn markdown(&self) -> String {
        let mut out = String::from("# OCS dictionary extraction coverage\n\n");
        out.push_str(&format!("- Source entries: {}\n", self.source_entries));
        out.push_str(&format!("- OCS entries: {}\n", self.ocs_entries));
        out.push_str(&format!("- Parse failures: {}\n", self.parse_failures));
        out.push_str(&format!("- Lexical senses: {}\n", self.lexical_senses));
        out.push_str(&format!(
            "- Linked to inflection identities: {}\n",
            self.linked_inflection_senses
        ));
        out.push_str(&format!(
            "- Senses with examples: {}\n",
            self.senses_with_examples
        ));
        out.push_str(&format!("- Examples: {}\n\n", self.examples));
        out.push_str("## Senses by part of speech\n\n");
        for (part_of_speech, count) in &self.senses_by_part_of_speech {
            out.push_str(&format!("- `{part_of_speech}`: {count}\n"));
        }
        out
    }
}

pub fn refresh_dictionary(dump: &Path, root: &Path) -> Result<(), Box<dyn Error>> {
    let morphology = load_registry(&root.join("data/extracted"))?;
    let (senses, coverage) = extract_dictionary_dump(dump, &morphology)?;
    validate_dictionary(&senses, &morphology)?;

    let data_dir = root.join("data/dictionary");
    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(root.join("reports"))?;

    let mut senses_json = serde_json::to_vec_pretty(&senses)?;
    senses_json.push(b'\n');
    let source = source_metadata(dump)?;
    let mut source_json = serde_json::to_vec_pretty(&source)?;
    source_json.push(b'\n');
    let mut coverage_json = serde_json::to_vec_pretty(&coverage)?;
    coverage_json.push(b'\n');
    let coverage_markdown = coverage.markdown();

    atomic_write_batch(&[
        (data_dir.join("senses.json"), senses_json.as_slice()),
        (data_dir.join("source.json"), source_json.as_slice()),
        (
            root.join("reports/dictionary-coverage.json"),
            coverage_json.as_slice(),
        ),
        (
            root.join("reports/dictionary-coverage.md"),
            coverage_markdown.as_bytes(),
        ),
    ])?;

    println!(
        "refreshed {} OCS senses ({} inflection-linked) from {}",
        senses.len(),
        coverage.linked_inflection_senses,
        dump.display()
    );
    Ok(())
}

pub fn check_dictionary(root: &Path) -> Result<(), Box<dyn Error>> {
    let morphology = load_registry(&root.join("data/extracted"))?;
    let senses: Vec<DictionarySenseRow> =
        serde_json::from_slice(&fs::read(root.join("data/dictionary/senses.json"))?)?;
    validate_dictionary(&senses, &morphology)?;

    let source: DictionarySourceMetadata =
        serde_json::from_slice(&fs::read(root.join("data/dictionary/source.json"))?)?;
    let sources_toml = fs::read_to_string(root.join("data/dictionary/SOURCES.toml"))?;
    if !sources_toml.contains(&source.sha256)
        || !sources_toml.contains(&format!("bytes = {}", source.bytes))
    {
        return Err("dictionary SOURCES.toml disagrees with source.json".into());
    }
    let coverage: DictionaryCoverage =
        serde_json::from_slice(&fs::read(root.join("reports/dictionary-coverage.json"))?)?;
    if coverage.lexical_senses != senses.len()
        || coverage.linked_inflection_senses
            != senses
                .iter()
                .filter(|sense| sense.inflection_lexeme_id.is_some())
                .count()
        || fs::read_to_string(root.join("reports/dictionary-coverage.md"))? != coverage.markdown()
    {
        return Err("dictionary coverage reports are stale".into());
    }

    println!(
        "check-dictionary: OK ({} senses, {} inflection-linked)",
        senses.len(),
        coverage.linked_inflection_senses
    );
    Ok(())
}

fn extract_dictionary_dump(
    dump: &Path,
    morphology: &Registry,
) -> Result<(Vec<DictionarySenseRow>, DictionaryCoverage), Box<dyn Error>> {
    let aliases = morphology_aliases(morphology);
    let mut senses = Vec::new();
    let mut seen_sense_ids = BTreeSet::new();
    let mut coverage = DictionaryCoverage {
        schema_version: DICTIONARY_SCHEMA,
        source_entries: 0,
        ocs_entries: 0,
        parse_failures: 0,
        lexical_senses: 0,
        linked_inflection_senses: 0,
        senses_with_examples: 0,
        examples: 0,
        senses_by_part_of_speech: BTreeMap::new(),
    };

    for line in BufReader::new(File::open(dump)?).lines() {
        let line = line?;
        coverage.source_entries += 1;
        let entry: Entry = match serde_json::from_str(&line) {
            Ok(entry) => entry,
            Err(_) => {
                coverage.parse_failures += 1;
                continue;
            }
        };
        if entry.lang_code != "cu" {
            continue;
        }
        coverage.ocs_entries += 1;
        let Some(part_of_speech) = dictionary_part_of_speech(&entry.pos) else {
            continue;
        };
        let inflection_pos = inflection_part_of_speech(&entry.pos);
        let lemma = inflection_pos.map_or(entry.word.as_str(), |pos| canonical_lemma(&entry, pos));
        let key = match lookup_key(lemma) {
            Ok(key) => key,
            Err(_) => continue,
        };
        let page_key = match lookup_key(&entry.word) {
            Ok(key) => key,
            Err(_) => continue,
        };
        let inflection_lexeme_id = inflection_pos.and_then(|pos| {
            let ids = aliases.get(&(key.clone(), pos.to_string()))?;
            (ids.len() == 1).then(|| ids[0].clone())
        });

        for source in &entry.senses {
            if source.tags.iter().any(|tag| tag == "form-of") || source.glosses.is_empty() {
                continue;
            }
            let glosses = cleaned_strings(&source.glosses);
            if glosses.is_empty() {
                continue;
            }
            let examples = source
                .examples
                .iter()
                .filter(|example| !example.text.trim().is_empty())
                .map(|example| {
                    let translation = if example.english.trim().is_empty() {
                        example.translation.trim()
                    } else {
                        example.english.trim()
                    };
                    let translation = if translation.starts_with("(please add") {
                        ""
                    } else {
                        translation
                    };
                    DictionaryExampleRow {
                        text: example.text.trim().to_string(),
                        romanization: example.roman.trim().to_string(),
                        translation: translation.to_string(),
                        reference: example.reference.trim().to_string(),
                    }
                })
                .collect::<Vec<_>>();
            let source_sense_id = source.id.trim().to_string();
            let identity = format!(
                "{lemma}\0{part_of_speech}\0{source_sense_id}\0{}",
                glosses.join("\0")
            );
            let id = if source_sense_id.is_empty() {
                format!("wiktionary-sense-{:016x}", fnv1a(identity.as_bytes()))
            } else {
                format!("{source_sense_id}#{:016x}", fnv1a(identity.as_bytes()))
            };
            if !seen_sense_ids.insert(id.clone()) {
                continue;
            }
            senses.push(DictionarySenseRow {
                id,
                source_sense_id,
                lemma: lemma.to_string(),
                page_word: entry.word.clone(),
                key: key.clone(),
                page_key: page_key.clone(),
                part_of_speech: part_of_speech.to_string(),
                inflection_lexeme_id: inflection_lexeme_id.clone(),
                glosses,
                raw_glosses: cleaned_strings(&source.raw_glosses),
                tags: sorted_strings(&source.tags),
                topics: sorted_strings(&source.topics),
                examples,
            });
        }
    }

    let failure_fraction = coverage.parse_failures as f64 / coverage.source_entries.max(1) as f64;
    if failure_fraction > MAX_PARSE_FAILURE_FRACTION {
        return Err(format!(
            "{} of {} dictionary rows ({:.3}%) failed to parse",
            coverage.parse_failures,
            coverage.source_entries,
            failure_fraction * 100.0
        )
        .into());
    }

    senses.sort_by(|left, right| {
        (&left.key, &left.part_of_speech, &left.id).cmp(&(
            &right.key,
            &right.part_of_speech,
            &right.id,
        ))
    });
    coverage.lexical_senses = senses.len();
    coverage.linked_inflection_senses = senses
        .iter()
        .filter(|sense| sense.inflection_lexeme_id.is_some())
        .count();
    coverage.senses_with_examples = senses
        .iter()
        .filter(|sense| !sense.examples.is_empty())
        .count();
    coverage.examples = senses.iter().map(|sense| sense.examples.len()).sum();
    for sense in &senses {
        *coverage
            .senses_by_part_of_speech
            .entry(sense.part_of_speech.clone())
            .or_default() += 1;
    }
    Ok((senses, coverage))
}

fn validate_dictionary(
    senses: &[DictionarySenseRow],
    morphology: &Registry,
) -> Result<(), Box<dyn Error>> {
    let mut ids = BTreeSet::new();
    let morphology_ids = morphology
        .lexemes
        .iter()
        .map(|lexeme| lexeme.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut previous: Option<(&str, &str, &str)> = None;
    for sense in senses {
        if sense.id.is_empty()
            || sense.lemma.is_empty()
            || sense.key.is_empty()
            || sense.part_of_speech.is_empty()
            || sense.glosses.is_empty()
            || sense.glosses.iter().any(|gloss| gloss.trim().is_empty())
        {
            return Err(format!("incomplete dictionary sense: {}", sense.id).into());
        }
        if !ids.insert(sense.id.as_str()) {
            return Err(format!("duplicate dictionary sense id: {}", sense.id).into());
        }
        if sense.tags.iter().any(|tag| tag == "form-of") {
            return Err(format!("form-of sense entered lexical dictionary: {}", sense.id).into());
        }
        if lookup_key(&sense.lemma)? != sense.key || lookup_key(&sense.page_word)? != sense.page_key
        {
            return Err(format!("stale lookup key for dictionary sense {}", sense.id).into());
        }
        if let Some(id) = &sense.inflection_lexeme_id
            && !morphology_ids.contains(id.as_str())
        {
            return Err(format!("dictionary sense {} has unknown lexeme id {id}", sense.id).into());
        }
        let current = (
            sense.key.as_str(),
            sense.part_of_speech.as_str(),
            sense.id.as_str(),
        );
        if previous.is_some_and(|previous| previous >= current) {
            return Err("dictionary senses are not uniquely source-sorted".into());
        }
        previous = Some(current);
    }
    Ok(())
}

fn morphology_aliases(registry: &Registry) -> BTreeMap<(String, String), Vec<String>> {
    let positions = registry
        .lexemes
        .iter()
        .map(|lexeme| (lexeme.id.as_str(), lexeme.pos.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut out: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for alias in &registry.aliases {
        if let Some(part_of_speech) = positions.get(alias.lexeme_id.as_str()) {
            let ids = out
                .entry((alias.key.clone(), (*part_of_speech).to_string()))
                .or_default();
            if !ids.contains(&alias.lexeme_id) {
                ids.push(alias.lexeme_id.clone());
            }
        }
    }
    out
}

fn dictionary_part_of_speech(source: &str) -> Option<&'static str> {
    match source {
        "adj" => Some("adjective"),
        "adv" => Some("adverb"),
        "conj" => Some("conjunction"),
        "det" => Some("determiner"),
        "intj" => Some("interjection"),
        "name" => Some("proper-name"),
        "noun" => Some("noun"),
        "num" => Some("numeral"),
        "particle" => Some("particle"),
        "prep" => Some("preposition"),
        "pron" => Some("pronoun"),
        "verb" => Some("verb"),
        _ => None,
    }
}

fn inflection_part_of_speech(source: &str) -> Option<&'static str> {
    match source {
        "noun" | "name" => Some("noun"),
        "adj" => Some("adj"),
        "verb" => Some("verb"),
        "pron" => Some("pron"),
        "num" => Some("num"),
        "det" => Some("det"),
        _ => None,
    }
}

fn cleaned_strings(values: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        let value = value.trim();
        if !value.is_empty() && !out.iter().any(|existing| existing == value) {
            out.push(value.to_string());
        }
    }
    out
}

fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut out = cleaned_strings(values);
    out.sort();
    out
}

fn source_metadata(path: &Path) -> Result<DictionarySourceMetadata, Box<dyn Error>> {
    let bytes = fs::metadata(path)?.len();
    let sha256 = sha256_file(path)?;
    Ok(DictionarySourceMetadata {
        schema_version: DICTIONARY_SCHEMA,
        input_file: path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("dictionary.jsonl")
            .to_string(),
        bytes,
        sha256,
    })
}

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_dictionary_parts_of_speech_are_explicit() {
        assert_eq!(dictionary_part_of_speech("noun"), Some("noun"));
        assert_eq!(dictionary_part_of_speech("adv"), Some("adverb"));
        assert_eq!(dictionary_part_of_speech("character"), None);
        assert_eq!(dictionary_part_of_speech("suffix"), None);
    }
}
