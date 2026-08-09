#[derive(Debug, Clone, Copy)]
pub(crate) struct ExampleRecord {
    pub text: &'static str,
    pub romanization: &'static str,
    pub translation: &'static str,
    pub reference: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct SenseRecord {
    pub id: &'static str,
    pub source_sense_id: &'static str,
    pub lemma: &'static str,
    pub page_word: &'static str,
    pub key: &'static str,
    pub page_key: &'static str,
    pub part_of_speech: &'static str,
    pub inflection_lexeme_id: Option<&'static str>,
    pub glosses: &'static [&'static str],
    pub raw_glosses: &'static [&'static str],
    pub tags: &'static [&'static str],
    pub topics: &'static [&'static str],
    pub examples: &'static [ExampleRecord],
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/dictionary.rs"
));
