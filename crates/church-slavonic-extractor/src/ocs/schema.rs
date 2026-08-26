use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
pub struct Entry {
    pub word: String,
    #[serde(default)]
    pub lang_code: String,
    pub pos: String,
    #[serde(default)]
    pub forms: Vec<SourceForm>,
    #[serde(default)]
    pub head_templates: Vec<HeadTemplate>,
    #[serde(default)]
    pub senses: Vec<Sense>,
}

#[derive(Debug, Deserialize)]
pub struct SourceForm {
    pub form: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub raw_tags: Vec<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default, alias = "roman")]
    pub romanization: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HeadTemplate {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub args: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct Sense {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub glosses: Vec<String>,
    #[serde(default)]
    pub raw_glosses: Vec<String>,
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default)]
    pub examples: Vec<SourceExample>,
    #[serde(default)]
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SourceExample {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub roman: String,
    #[serde(default)]
    pub translation: String,
    #[serde(default)]
    pub english: String,
    #[serde(default, rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LexemeRow {
    pub id: String,
    pub lemma: String,
    pub page_word: String,
    pub key: String,
    pub pos: String,
    pub class: String,
    pub raw_class: String,
    pub gender: String,
    pub animacy: String,
    pub number_restriction: String,
    pub head_templates: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AliasRow {
    pub key: String,
    pub lexeme_id: String,
    pub source_spellings: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FormRow {
    pub lexeme_id: String,
    pub feature: String,
    pub rank: u16,
    pub form: String,
    pub romanization: String,
    pub source_spelling: String,
    pub source_tags: String,
}

/// One provenance-bearing field in a verb-system analysis.
///
/// The normalized file deliberately stores fields rather than a wide optional
/// record. `system` and `analysis_rank` keep alternative aorists or participles
/// separate, while `field` is validated against a closed vocabulary before the
/// runtime registry is generated.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VerbMetadataRow {
    pub lexeme_id: String,
    pub system: String,
    pub analysis_rank: u16,
    pub field: String,
    pub value: String,
    pub provenance: String,
    pub source_feature: String,
    pub source_form: String,
    pub crosscheck_features: String,
    pub authority: String,
}

/// One reviewed cell-specific correction, kept separate from source table rows
/// so an override can never displace exact dictionary evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OverrideRow {
    pub lexeme_id: String,
    pub feature: String,
    pub rank: u16,
    pub form: String,
    pub romanization: String,
    pub reason: String,
    pub authority: String,
}

#[derive(Debug, Default, Clone)]
pub struct Registry {
    pub lexemes: Vec<LexemeRow>,
    pub aliases: Vec<AliasRow>,
    pub forms: Vec<FormRow>,
    pub verb_metadata: Vec<VerbMetadataRow>,
    pub overrides: Vec<OverrideRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DictionaryExampleRow {
    pub text: String,
    pub romanization: String,
    pub translation: String,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DictionarySenseRow {
    pub id: String,
    pub source_sense_id: String,
    pub lemma: String,
    pub page_word: String,
    pub key: String,
    pub page_key: String,
    pub part_of_speech: String,
    pub inflection_lexeme_id: Option<String>,
    pub glosses: Vec<String>,
    pub raw_glosses: Vec<String>,
    pub tags: Vec<String>,
    pub topics: Vec<String>,
    pub examples: Vec<DictionaryExampleRow>,
}
