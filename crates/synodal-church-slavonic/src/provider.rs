use std::collections::BTreeMap;

use synodal_church_slavonic_core::{
    Animacy, Error, ErrorCode, FormSet, GrammarCell, LexemeId, MetadataField, Result, SynodalWord,
    VerbSystem, normalize_lookup_accentless,
};

use crate::{
    Capabilities, Inflector, LexemeSpec, LexemeSummary, Paradigm, PartOfSpeech,
    paradigm::{noun_cells, verb_cells},
    registry,
    spec::LexemeSpecInner,
};

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProviderLexemeKind {
    Builtin,
    Supplied { spec: LexemeSpec },
}

/// One stable target-recension identity exposed by a [`LexemeProvider`].
///
/// Supplied entries carry typed lexical metadata and resolve through the
/// productive rule kernel; exact surface forms live only in the generated
/// data-side irregular table. Built-in entries are adapters over the
/// generated registry and therefore enter the same `Lexicon` composition and
/// conflict checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderLexeme {
    summary: LexemeSummary,
    kind: ProviderLexemeKind,
}

impl ProviderLexeme {
    pub fn new(
        id: impl Into<LexemeId>,
        source_id: impl Into<String>,
        spec: LexemeSpec,
    ) -> Result<Self> {
        let id = id.into();
        let source_id = source_id.into();
        if id.as_str().trim().is_empty() || source_id.trim().is_empty() {
            return Err(Error::ContradictoryMetadata {
                reason: "a provider lexeme requires nonempty identity and source IDs".into(),
            });
        }
        spec.validate()?;
        Ok(Self {
            summary: LexemeSummary::new(id, spec.lemma().into(), spec.part_of_speech(), source_id),
            kind: ProviderLexemeKind::Supplied { spec },
        })
    }

    fn builtin(summary: LexemeSummary) -> Self {
        Self {
            summary,
            kind: ProviderLexemeKind::Builtin,
        }
    }

    #[must_use]
    pub fn summary(&self) -> &LexemeSummary {
        &self.summary
    }

    #[must_use]
    pub fn spec(&self) -> Option<&LexemeSpec> {
        match &self.kind {
            ProviderLexemeKind::Builtin => None,
            ProviderLexemeKind::Supplied { spec, .. } => Some(spec),
        }
    }
}

/// Read-only source of already reviewed, fully typed lexical entries.
///
/// The contract intentionally contains no filesystem, network, or storage
/// methods. Applications may load their data however they choose and expose a
/// deterministic snapshot through `lexemes`.
pub trait LexemeProvider {
    fn lexemes(&self) -> Result<Vec<ProviderLexeme>>;
}

/// Adapter exposing the generated static registry through [`LexemeProvider`].
#[derive(Clone, Copy, Debug, Default)]
pub struct StaticLexemeProvider;

impl LexemeProvider for StaticLexemeProvider {
    fn lexemes(&self) -> Result<Vec<ProviderLexeme>> {
        registry::all_lexemes()
            .map(|entries| entries.into_iter().map(ProviderLexeme::builtin).collect())
    }
}

/// An application-owned provider suitable for small in-memory lexicons.
#[derive(Clone, Debug, Default)]
pub struct InMemoryLexemeProvider {
    entries: Vec<ProviderLexeme>,
}

impl InMemoryLexemeProvider {
    pub fn new(entries: impl IntoIterator<Item = ProviderLexeme>) -> Result<Self> {
        let entries = entries.into_iter().collect::<Vec<_>>();
        validate_entries(&entries)?;
        Ok(Self { entries })
    }
}

impl LexemeProvider for InMemoryLexemeProvider {
    fn lexemes(&self) -> Result<Vec<ProviderLexeme>> {
        Ok(self.entries.clone())
    }
}

/// Deterministic composition of one or more lexical providers.
///
/// Entries are sorted by stable ID. No provider shadows another: duplicate IDs
/// are a typed conflict, while distinct homographic IDs remain an explicit
/// `AmbiguousLexeme` at lookup time.
#[derive(Clone, Debug)]
pub struct Lexicon {
    inflector: Inflector,
    entries: Vec<ProviderLexeme>,
}

impl Lexicon {
    pub fn builtin(inflector: Inflector) -> Result<Self> {
        Self::from_provider(inflector, &StaticLexemeProvider)
    }

    pub fn from_provider(inflector: Inflector, provider: &dyn LexemeProvider) -> Result<Self> {
        Self::compose(inflector, &[provider])
    }

    pub fn compose(inflector: Inflector, providers: &[&dyn LexemeProvider]) -> Result<Self> {
        let mut entries = Vec::new();
        for provider in providers {
            entries.extend(provider.lexemes()?);
        }
        validate_entries(&entries)?;
        entries.sort_by(|left, right| left.summary.id().cmp(right.summary.id()));
        Ok(Self { inflector, entries })
    }

    #[must_use]
    pub const fn inflector(&self) -> Inflector {
        self.inflector
    }

    pub fn lexemes(&self) -> impl Iterator<Item = &LexemeSummary> {
        self.entries.iter().map(ProviderLexeme::summary)
    }

    pub fn resolve(&self, lemma: &str) -> Result<LexemeSummary> {
        let parsed = SynodalWord::parse(lemma)?;
        let lookup = normalize_lookup_accentless(parsed.canonical());
        let matches = self
            .entries
            .iter()
            .filter(|entry| normalize_lookup_accentless(entry.summary.lemma()) == lookup)
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Err(Error::UnknownLemma { lookup }),
            [entry] => Ok(entry.summary.clone()),
            entries => Err(Error::AmbiguousLexeme {
                lexemes: entries
                    .iter()
                    .map(|entry| entry.summary.id().clone())
                    .collect(),
            }),
        }
    }

    pub fn from_id(&self, id: &LexemeId) -> Result<LexemeSummary> {
        self.entry(id)
            .map(|entry| entry.summary.clone())
            .ok_or_else(|| Error::UnknownLemma {
                lookup: id.to_string(),
            })
    }

    pub fn form(&self, lemma: &str, cell: GrammarCell) -> Result<FormSet> {
        let summary = self.resolve(lemma)?;
        self.form_by_id(summary.id(), cell)
    }

    pub fn form_by_id(&self, id: &LexemeId, cell: GrammarCell) -> Result<FormSet> {
        let entry = self.entry(id).ok_or_else(|| Error::UnknownLemma {
            lookup: id.to_string(),
        })?;
        match &entry.kind {
            ProviderLexemeKind::Builtin => self.inflector.form_by_id(id, cell),
            ProviderLexemeKind::Supplied { spec } => self.inflector.form_spec(spec, cell),
        }
    }

    #[must_use]
    pub fn batch(&self, requests: impl IntoIterator<Item = BatchRequest>) -> BatchResult {
        let rows = requests
            .into_iter()
            .map(|request| {
                let outcome = match &request.lexeme {
                    BatchLexeme::Lemma(lemma) => self.form(lemma, request.cell),
                    BatchLexeme::Id(id) => self.form_by_id(id, request.cell),
                };
                BatchRow { request, outcome }
            })
            .collect();
        BatchResult { rows }
    }

    pub fn noun_paradigm(&self, id: &LexemeId, animacy: Animacy) -> Result<Paradigm> {
        let summary = self.from_id(id)?;
        require_pos(&summary, PartOfSpeech::Noun)?;
        Ok(Paradigm::build_with(summary, noun_cells(animacy), |cell| {
            self.form_by_id(id, cell)
        }))
    }

    pub fn verb_system_paradigm(&self, id: &LexemeId, system: VerbSystem) -> Result<Paradigm> {
        let summary = self.from_id(id)?;
        require_pos(&summary, PartOfSpeech::Verb)?;
        Ok(Paradigm::build_with(summary, verb_cells(system), |cell| {
            self.form_by_id(id, cell)
        }))
    }

    pub fn capabilities_by_id(&self, id: &LexemeId) -> Result<Capabilities> {
        let entry = self.entry(id).ok_or_else(|| Error::UnknownLemma {
            lookup: id.to_string(),
        })?;
        match &entry.kind {
            ProviderLexemeKind::Builtin => {
                Ok(Capabilities::for_summary(&entry.summary, self.inflector))
            }
            ProviderLexemeKind::Supplied { spec } => Ok(spec_capabilities(spec)),
        }
    }

    pub fn missing_principal_parts(
        &self,
        id: &LexemeId,
        system: VerbSystem,
    ) -> Result<Vec<MetadataField>> {
        let entry = self.entry(id).ok_or_else(|| Error::UnknownLemma {
            lookup: id.to_string(),
        })?;
        require_pos(&entry.summary, PartOfSpeech::Verb)?;
        match &entry.kind {
            ProviderLexemeKind::Builtin => {
                Ok(registry::verb_lexeme(id)?.missing_principal_parts(system))
            }
            ProviderLexemeKind::Supplied { spec, .. } => match spec.inner() {
                LexemeSpecInner::Verb(verb) => Ok(verb.missing_principal_parts(system)),
                _ => Err(Error::ContradictoryMetadata {
                    reason: "provider part of speech and specification disagree".into(),
                }),
            },
        }
    }

    fn entry(&self, id: &LexemeId) -> Option<&ProviderLexeme> {
        self.entries
            .binary_search_by(|entry| entry.summary.id().cmp(id))
            .ok()
            .map(|index| &self.entries[index])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BatchLexeme {
    Lemma(String),
    Id(LexemeId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BatchRequest {
    pub lexeme: BatchLexeme,
    pub cell: GrammarCell,
}

impl BatchRequest {
    #[must_use]
    pub fn lemma(lemma: impl Into<String>, cell: GrammarCell) -> Self {
        Self {
            lexeme: BatchLexeme::Lemma(lemma.into()),
            cell,
        }
    }

    #[must_use]
    pub fn id(id: impl Into<LexemeId>, cell: GrammarCell) -> Self {
        Self {
            lexeme: BatchLexeme::Id(id.into()),
            cell,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BatchRow {
    pub request: BatchRequest,
    pub outcome: Result<FormSet>,
}

impl BatchRow {
    #[must_use]
    pub fn error_code(&self) -> Option<ErrorCode> {
        self.outcome.as_ref().err().map(Error::code)
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BatchResult {
    rows: Vec<BatchRow>,
}

impl BatchResult {
    pub fn iter(&self) -> impl Iterator<Item = &BatchRow> {
        self.rows.iter()
    }

    pub fn successes(&self) -> impl Iterator<Item = &BatchRow> {
        self.rows.iter().filter(|row| row.outcome.is_ok())
    }

    pub fn failures(&self) -> impl Iterator<Item = &BatchRow> {
        self.rows.iter().filter(|row| row.outcome.is_err())
    }

    pub fn with_error_code(&self, code: ErrorCode) -> impl Iterator<Item = &BatchRow> {
        self.rows
            .iter()
            .filter(move |row| row.error_code() == Some(code))
    }

    #[must_use]
    pub fn into_rows(self) -> Vec<BatchRow> {
        self.rows
    }
}

fn validate_entries(entries: &[ProviderLexeme]) -> Result<()> {
    let mut ids = BTreeMap::new();
    for entry in entries {
        if let Some(previous) = ids.insert(entry.summary.id().clone(), entry.summary.lemma()) {
            return Err(Error::ProviderConflict {
                lexeme: entry.summary.id().clone(),
                reason: format!(
                    "duplicate stable identity for lemmas {previous:?} and {:?}",
                    entry.summary.lemma()
                ),
            });
        }
        if entry.summary.id().as_str().trim().is_empty()
            || entry.summary.lemma().trim().is_empty()
            || entry.summary.source_id().trim().is_empty()
        {
            return Err(Error::ProviderConflict {
                lexeme: entry.summary.id().clone(),
                reason: "identity, lemma, and source ID must be nonempty".into(),
            });
        }
        if let ProviderLexemeKind::Supplied { spec, .. } = &entry.kind {
            spec.validate()?;
            if spec.lemma() != entry.summary.lemma()
                || spec.part_of_speech() != entry.summary.part_of_speech()
            {
                return Err(Error::ProviderConflict {
                    lexeme: entry.summary.id().clone(),
                    reason: "identity metadata conflicts with its lexical specification".into(),
                });
            }
        }
    }
    Ok(())
}

fn require_pos(summary: &LexemeSummary, expected: PartOfSpeech) -> Result<()> {
    if summary.part_of_speech() == expected {
        Ok(())
    } else {
        Err(Error::ContradictoryMetadata {
            reason: format!(
                "lexeme {} is {:?}, not {expected:?}",
                summary.id(),
                summary.part_of_speech()
            ),
        })
    }
}

fn spec_capabilities(spec: &LexemeSpec) -> Capabilities {
    match spec.inner() {
        LexemeSpecInner::Noun(_) => Capabilities {
            productive_noun: true,
            ..Capabilities::default()
        },
        LexemeSpecInner::Adjective(_) => Capabilities {
            productive_adjective: true,
            ..Capabilities::default()
        },
        LexemeSpecInner::Determiner(_) => Capabilities {
            productive_determiner: true,
            ..Capabilities::default()
        },
        LexemeSpecInner::Numeral(_) => Capabilities {
            productive_numeral: true,
            ..Capabilities::default()
        },
        LexemeSpecInner::Pronoun(_) => Capabilities {
            productive_pronoun: true,
            ..Capabilities::default()
        },
        LexemeSpecInner::Verb(verb) => {
            let complete = |system| verb.missing_principal_parts(system).is_empty();
            Capabilities {
                present: complete(VerbSystem::Finite(
                    synodal_church_slavonic_core::FiniteTense::Present,
                )),
                future: verb.lexeme.aspect == synodal_church_slavonic_core::Aspect::Perfective
                    && complete(VerbSystem::Finite(
                        synodal_church_slavonic_core::FiniteTense::Future,
                    )),
                imperfect: complete(VerbSystem::Finite(
                    synodal_church_slavonic_core::FiniteTense::Imperfect,
                )),
                aorist: complete(VerbSystem::Finite(
                    synodal_church_slavonic_core::FiniteTense::Aorist,
                )),
                imperative: complete(VerbSystem::Imperative),
                infinitive: true,
                l_participle: complete(VerbSystem::LParticiple),
                participle: VerbSystem::ALL
                    .into_iter()
                    .filter(|system| matches!(system, VerbSystem::Participle { .. }))
                    .any(complete),
                verbal_noun: complete(VerbSystem::VerbalNoun {
                    animacy: synodal_church_slavonic_core::Animacy::Inanimate,
                }),
                ..Capabilities::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use synodal_church_slavonic_core::{
        Animacy, Case, Error, ErrorCode, Gender, GrammarCell, NounCell, NounDeclension,
        NounNumberInventory, Number,
    };

    use super::*;
    use crate::{NounSpec, SpecificationSource};

    fn source(label: &str) -> SpecificationSource {
        SpecificationSource::new(
            format!("evidence:{label}"),
            "application-test-lexicon",
            format!("reviewed application fixture {label}"),
        )
        .expect("valid test provenance")
    }

    fn noun_cell(case: Case, number: Number) -> GrammarCell {
        GrammarCell::Noun(NounCell {
            case,
            number,
            animacy: Animacy::Inanimate,
        })
    }

    fn supplied_noun(id: &str, lemma: &str) -> ProviderLexeme {
        let stem = lemma.strip_suffix('ъ').unwrap_or(lemma);
        let spec = NounSpec::new(
            lemma,
            stem,
            Gender::Masculine,
            NounDeclension::FirstHardMasculine,
            source(id),
        )
        .expect("valid supplied noun");
        ProviderLexeme::new(id, "application-test-lexicon", LexemeSpec::from(spec))
            .expect("valid provider entry")
    }

    #[test]
    fn composition_rejects_duplicate_identity_and_keeps_homographs_ambiguous() {
        let left = InMemoryLexemeProvider::new([supplied_noun("application:noun:left", "даръ")])
            .expect("left provider");
        let duplicate =
            InMemoryLexemeProvider::new([supplied_noun("application:noun:left", "гласъ")])
                .expect("duplicate provider is valid alone");
        let error = Lexicon::compose(Inflector::default(), &[&left, &duplicate])
            .expect_err("duplicate IDs must fail closed");
        assert_eq!(error.code(), ErrorCode::ProviderConflict);

        let right = InMemoryLexemeProvider::new([supplied_noun("application:noun:right", "даръ")])
            .expect("right provider");
        let lexicon = Lexicon::compose(Inflector::default(), &[&right, &left])
            .expect("homographs retain distinct IDs");
        assert_eq!(
            lexicon
                .lexemes()
                .map(|entry| entry.id().as_str())
                .collect::<Vec<_>>(),
            ["application:noun:left", "application:noun:right"]
        );
        assert!(matches!(
            lexicon.resolve("да́ръ"),
            Err(Error::AmbiguousLexeme { lexemes }) if lexemes.len() == 2
        ));
    }

    #[test]
    fn static_and_application_providers_compose_without_shadowing() {
        let application =
            InMemoryLexemeProvider::new([supplied_noun("application:noun:putnik", "пꙋтникъ")])
                .expect("application provider");
        let lexicon =
            Lexicon::compose(Inflector::default(), &[&StaticLexemeProvider, &application])
                .expect("composed lexicon");
        assert_eq!(
            lexicon
                .form("пꙋтникъ", noun_cell(Case::Genitive, Number::Singular),)
                .expect("application noun")
                .primary_text(),
            "пꙋтника"
        );
        assert_eq!(
            lexicon
                .form("рабъ", noun_cell(Case::Genitive, Number::Singular))
                .expect("built-in noun")
                .primary_text(),
            "раба"
        );
    }

    #[test]
    fn batch_retains_order_failures_and_error_codes() {
        let provider =
            InMemoryLexemeProvider::new([supplied_noun("application:noun:putnik", "пꙋтникъ")])
                .expect("application provider");
        let lexicon = Lexicon::from_provider(Inflector::default(), &provider).expect("lexicon");
        let genitive = noun_cell(Case::Genitive, Number::Singular);
        let batch = lexicon.batch([
            BatchRequest::lemma("пꙋтникъ", genitive),
            BatchRequest::lemma("неизвѣстенъ", genitive),
            BatchRequest::id("application:noun:putnik", genitive),
        ]);
        let rows = batch.iter().collect::<Vec<_>>();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[0].outcome.as_ref().expect("first").primary_text(),
            "пꙋтника"
        );
        assert_eq!(rows[1].error_code(), Some(ErrorCode::UnknownLemma));
        assert_eq!(
            rows[2].outcome.as_ref().expect("third").primary_text(),
            "пꙋтника"
        );
        assert_eq!(batch.successes().count(), 2);
        assert_eq!(batch.failures().count(), 1);
        assert_eq!(batch.with_error_code(ErrorCode::UnknownLemma).count(), 1);
    }

    #[test]
    fn provider_paradigms_retain_number_restriction_failures() {
        let spec = NounSpec::new(
            "людїе",
            "люд",
            Gender::Masculine,
            NounDeclension::ThirdMasculine,
            source("people"),
        )
        .expect("valid noun")
        .with_number_inventory(NounNumberInventory::PluralOnly)
        .expect("valid restriction");
        let id = LexemeId::from("application:noun:people");
        let provider = InMemoryLexemeProvider::new([ProviderLexeme::new(
            id.clone(),
            "application-test-lexicon",
            LexemeSpec::from(spec),
        )
        .expect("entry")])
        .expect("provider");
        let lexicon = Lexicon::from_provider(Inflector::default(), &provider).expect("lexicon");
        let paradigm = lexicon
            .noun_paradigm(&id, Animacy::Animate)
            .expect("provider paradigm");
        assert_eq!(paradigm.iter().count(), 21);
        assert_eq!(
            paradigm
                .iter()
                .filter(|row| row.error_code() == Some(ErrorCode::HistoricallyInvalidCell))
                .count(),
            14
        );
        assert!(
            lexicon
                .capabilities_by_id(&id)
                .expect("capabilities")
                .productive_noun
        );
    }

    #[test]
    fn provider_validation_is_panic_free_for_hostile_input() {
        assert!(matches!(
            Lexicon::from_provider(Inflector::default(), &InMemoryLexemeProvider::default())
                .expect("empty provider")
                .resolve("а\u{301}\u{486}\u{200d}"),
            Err(Error::InvalidUnicode { .. } | Error::InvalidOrthography { .. })
        ));
        assert!(matches!(
            ProviderLexeme::new(
                "",
                "application-test-lexicon",
                LexemeSpec::from(
                    NounSpec::new(
                        "даръ",
                        "дар",
                        Gender::Masculine,
                        NounDeclension::FirstHardMasculine,
                        source("invalid-id"),
                    )
                    .expect("valid spec")
                )
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}
