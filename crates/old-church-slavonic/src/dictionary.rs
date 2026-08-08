//! Static record types filled by the offline generator.

#[derive(Debug, Clone, Copy)]
pub(crate) struct LexemeRecord {
    pub id: &'static str,
    pub lemma: &'static str,
    pub key: &'static str,
    pub pos: &'static str,
    pub class: &'static str,
    pub gender: &'static str,
    pub animacy: &'static str,
    pub number_restriction: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AliasRecord {
    pub key: &'static str,
    pub lexeme_id: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct FormRecord {
    pub lexeme_id: &'static str,
    pub feature: &'static str,
    pub rank: u16,
    pub form: &'static str,
    pub romanization: &'static str,
    pub is_override: bool,
}

include!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/generated/registry.rs"
));
