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

#[derive(Debug, Default, Clone)]
pub struct Registry {
    pub lexemes: Vec<LexemeRow>,
    pub aliases: Vec<AliasRow>,
    pub forms: Vec<FormRow>,
}
