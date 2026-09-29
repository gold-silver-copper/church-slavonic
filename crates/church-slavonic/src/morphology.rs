//! Evidence-bearing declarative morphology with separate grammar and spelling
//! profiles. Rules apply optional explicit stem-final replacement and suffix
//! concatenation; unsupported operations must not be guessed by the engine.

mod abbreviation;
mod acceptance;
mod lexical_acceptance;
mod index;
pub use index::{AnalysisIndexStats, MAX_COMPILED_INDEX_BYTES};
mod stem;
pub mod accent;
pub mod observations;
use crate::Cell;
pub use abbreviation::{AbbreviationRule, AbbreviationTrace};
pub use acceptance::{ClaimReview, GenerationAcceptance, GenerationReview};
pub use lexical_acceptance::{LexicalAcceptance, LexicalClaim, ReviewedLexicalClaim};
pub use stem::{StemReplacement, StemTrace};
pub use accent::{AccentMark, AccentPolicy, AccentTrace, StressInstruction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use unicode_normalization::UnicodeNormalization;

macro_rules! identity {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        pub struct $name(pub String);
    };
}
identity!(LexemeId);
identity!(GrammarId);
identity!(OrthographyId);
identity!(ParadigmId);
identity!(EvidenceId);
identity!(RuleId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EvidenceKind {
    DescriptiveRule,
    EditorialPrescription,
    LexicalAssertion,
    Annotation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ReviewStatus {
    Unverified,
    SourceChecked,
    ExpertReviewed { reviewer: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub id: EvidenceId,
    pub source: String,
    pub source_sha256: String,
    pub method: String,
    pub location: String,
    pub claim: String,
    pub kind: EvidenceKind,
    pub review: ReviewStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrammarProfile {
    pub id: GrammarId,
    pub description: String,
    pub evidence: Vec<EvidenceId>,
}

/// An ordered editorial transformation. The input and output are retained in
/// every generated derivation. Identity rendering uses an empty list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellingRule {
    pub from: String,
    pub to: String,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnicodePolicy {
    #[default]
    Preserve,
    Nfc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccentOverride {
    pub paradigm: ParadigmId,
    #[serde(with = "cell_codec")]
    pub cell: Cell,
    pub mark: AccentMark,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrthographyProfile {
    pub id: OrthographyId,
    pub description: String,
    pub compatible_grammars: Vec<GrammarId>,
    #[serde(default)]
    pub accent: AccentPolicy,
    #[serde(default)]
    pub unicode: UnicodePolicy,
    #[serde(default)]
    pub accent_overrides: Vec<AccentOverride>,
    pub rules: Vec<SpellingRule>,
    #[serde(default)]
    pub abbreviations: Vec<AbbreviationRule>,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CellRule {
    pub id: RuleId,
    #[serde(with = "cell_codec")]
    pub cell: Cell,
    /// Names an explicitly supplied lexical stem or principal part.
    pub stem: String,
    /// Explicit stem-final change before suffixation and stress addressing.
    #[serde(default)]
    pub stem_replacement: Option<StemReplacement>,
    pub suffix: String,
    #[serde(default)]
    /// Vowel indices address the transformed stem/word, after stem replacement.
    pub stress: StressInstruction,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Paradigm {
    pub id: ParadigmId,
    pub grammar: GrammarId,
    #[serde(with = "pos_codec")]
    pub pos: crate::Pos,
    /// Multiple rules for a cell are explicitly licensed alternatives.
    pub rules: Vec<CellRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stem {
    pub name: String,
    pub text: String,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RelationKind {
    DerivedAdverb,
    OtherDerivation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexicalRelation {
    pub kind: RelationKind,
    pub target: LexemeId,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Restriction {
    #[serde(with = "cell_codec")]
    pub cell: Cell,
    pub reason: String,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LexicalEntry {
    pub id: LexemeId,
    pub displayed_lemma: String,
    /// Source-backed alternative classifications, not an inflection class or
    /// a claim about every token's syntactic use.
    pub categories: Vec<CategoryAssertion>,
    /// Lexical gender assertions, independent of a form's agreement gender.
    /// Empty means no assertion supplied; alternatives are not resolved here.
    #[serde(default)]
    pub genders: Vec<GenderAssertion>,
    pub paradigm: ParadigmId,
    pub stems: Vec<Stem>,
    pub unavailable: Vec<Restriction>,
    pub relations: Vec<LexicalRelation>,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LexicalCategory {
    Noun,
    Adjective,
    Verb,
    Pronoun,
    Determiner,
    Numeral,
    Adverb,
    Preposition,
    Conjunction,
    Particle,
    Interjection,
    Unclassified,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenderAssertion {
    #[serde(with = "gender_codec")]
    pub gender: crate::grammar::Gender,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CategoryAssertion {
    pub category: LexicalCategory,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Derivation {
    pub claim_sha256: String,
    pub review: GenerationReview,
    pub underlying: String,
    pub stem_change: Option<StemTrace>,
    pub accent: AccentTrace,
    pub surface: String,
    /// Exact source-annotation links, not an assertion that every generated
    /// cell or every matching token is attested or correctly analyzed.
    pub source_annotations: Vec<observations::AnnotationId>,
    pub abbreviation: Option<AbbreviationTrace>,
    pub grammar: GrammarId,
    pub orthography: OrthographyId,
    pub paradigm: ParadigmId,
    pub rule: RuleId,
    pub spelling_steps: Vec<(String, String)>,
    pub evidence: Vec<EvidenceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    /// Licensed by this executable model; inspect each derivation review.
    Licensed(Vec<Derivation>),
    /// A licensed cell cannot be realized from the available lexical inputs.
    MissingStems(Vec<String>),
    Partial {
        forms: Vec<Derivation>,
        missing_stems: Vec<String>,
    },
    /// A supported claim excludes this cell for this lexeme.
    Unavailable(Restriction),
    UnresolvedRestriction {
        restriction: Restriction,
        otherwise: Box<Availability>,
    },
    /// The selected paradigm provides no rule or explicit restriction.
    UnsupportedCell,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelInput {
    #[serde(default)]
    pub observations: observations::ObservationInput,
    pub evidence: Vec<Evidence>,
    pub grammars: Vec<GrammarProfile>,
    pub orthographies: Vec<OrthographyProfile>,
    pub paradigms: Vec<Paradigm>,
    pub lexemes: Vec<LexicalEntry>,
}

/// Resource policy belongs to the caller, not to an untrusted model artifact.
#[derive(Debug, Clone, Serialize)]
pub struct ModelLimits {
    /// Aggregate logical storage for lazily constructed analysis indexes.
    pub max_index_bytes: usize,
    pub max_index_forms: usize,
    pub max_index_checks: usize,
    pub max_observation_bytes: usize,
    pub max_observation_records: usize,
    pub max_form_bytes: usize,
    pub max_spelling_steps: usize,
    pub max_result_bytes: usize,
    pub max_candidates: usize,
    pub max_rule_checks: usize,
}
impl Default for ModelLimits {
    fn default() -> Self {
        Self {
            max_index_bytes: 64 * 1024 * 1024,
            max_index_forms: 100_000,
            max_index_checks: 1_000_000,
            max_observation_bytes: 64 * 1024 * 1024,
            max_observation_records: 100_000,
            max_form_bytes: 16 * 1024,
            max_spelling_steps: 256,
            max_result_bytes: 64 * 1024 * 1024,
            max_candidates: 100_000,
            max_rule_checks: 1_000_000,
        }
    }
}

/// A caller-verified assertion includes the attachment of evidence to a
/// particular lexeme and restriction. A trusted citation alone is insufficient.
#[derive(Debug, Clone, Serialize)]
pub struct RestrictionAssertion {
    pub lexeme: LexemeId,
    pub restriction: Restriction,
}

pub struct Model {
    analysis_cache: std::sync::RwLock<index::Cache>,
    observations: observations::Archive,
    limits: ModelLimits,
    data_sha256: String,
    generation_policy_sha256: String,
    accepted_generation: BTreeMap<String, GenerationAcceptance>,
    lexical_policy_sha256: String,
    accepted_lexical: BTreeMap<String, LexicalAcceptance>,
    verified_evidence: BTreeSet<EvidenceId>,
    verified_restrictions: BTreeSet<(LexemeId, String)>,
    evidence: BTreeMap<EvidenceId, Evidence>,
    grammars: BTreeMap<GrammarId, GrammarProfile>,
    orthographies: BTreeMap<OrthographyId, OrthographyProfile>,
    paradigms: BTreeMap<ParadigmId, Paradigm>,
    lexemes: BTreeMap<LexemeId, LexicalEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    MissingSource(String),
    InvalidObservation(String),
    InvalidIdentity(String),
    Duplicate(String),
    MissingReference(String),
    MissingEvidence(String),
    InvalidRule(String),
    IncompatibleProfiles,
    ArtifactMismatch(&'static str),
    Encoding(String),
    VerificationMismatch(EvidenceId),
    ResourceLimit(&'static str),
    Accent(accent::AccentError),
}
impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "morphology model: {self:?}")
    }
}
impl std::error::Error for ModelError {}

impl Model {
    pub fn build(input: ModelInput) -> Result<Self, ModelError> {
        Self::build_with_limits(input, ModelLimits::default())
    }

    pub fn build_with_limits(input: ModelInput, limits: ModelLimits) -> Result<Self, ModelError> {
        Self::build_with_verified_evidence(input, limits, Vec::new())
    }

    /// Only the caller can supply independently verified claims. Serialized
    /// review flags are reports, not authority. Every accepted record must
    /// match the input claim in full, including its source hash and wording.
    pub fn build_with_verified_evidence(
        input: ModelInput,
        limits: ModelLimits,
        verified: Vec<Evidence>,
    ) -> Result<Self, ModelError> {
        Self::build_with_verification(input, limits, verified, Vec::new())
    }

    pub fn build_with_verification(
        input: ModelInput,
        limits: ModelLimits,
        verified: Vec<Evidence>,
        restrictions: Vec<RestrictionAssertion>,
    ) -> Result<Self, ModelError> {
        Self::build_with_verification_and_sources(input, limits, verified, restrictions, Vec::new())
    }

    pub fn build_with_sources(
        input: ModelInput,
        limits: ModelLimits,
        sources: Vec<Vec<u8>>,
    ) -> Result<Self, ModelError> {
        Self::build_with_verification_and_sources(input, limits, Vec::new(), Vec::new(), sources)
    }

    pub fn build_with_verification_and_sources(
        input: ModelInput,
        limits: ModelLimits,
        verified: Vec<Evidence>,
        restrictions: Vec<RestrictionAssertion>,
        sources: Vec<Vec<u8>>,
    ) -> Result<Self, ModelError> {
        let encoded = serde_json::to_vec(&(&input, &limits, &verified, &restrictions))
            .map_err(|e| ModelError::Encoding(e.to_string()))?;
        let data_sha256 = format!("{:x}", Sha256::digest(&encoded));
        let observations = observations::Archive::build(input.observations, sources, &limits)?;
        // Each map is built before references are resolved; forward references
        // are valid, duplicate identities never overwrite an earlier record.
        let mut model = Self {
            analysis_cache: std::sync::RwLock::new(index::Cache::default()),
            observations,
            data_sha256,
            generation_policy_sha256: Self::empty_generation_policy_sha256(),
            accepted_generation: BTreeMap::new(),
            lexical_policy_sha256: Self::empty_generation_policy_sha256(),
            accepted_lexical: BTreeMap::new(),
            limits,
            verified_evidence: BTreeSet::new(),
            verified_restrictions: BTreeSet::new(),
            evidence: BTreeMap::new(),
            grammars: BTreeMap::new(),
            orthographies: BTreeMap::new(),
            paradigms: BTreeMap::new(),
            lexemes: BTreeMap::new(),
        };
        macro_rules! add {
            ($records:expr, $map:expr) => {
                for record in $records {
                    let key = record.id.clone();
                    if key.0.trim().is_empty() {
                        return Err(ModelError::InvalidIdentity(key.0));
                    }
                    if $map.insert(key.clone(), record).is_some() {
                        return Err(ModelError::Duplicate(key.0));
                    }
                }
            };
        }
        add!(input.evidence, model.evidence);
        add!(input.grammars, model.grammars);
        add!(input.orthographies, model.orthographies);
        add!(input.paradigms, model.paradigms);
        add!(input.lexemes, model.lexemes);
        for evidence in verified {
            if model.evidence.get(&evidence.id) != Some(&evidence) {
                return Err(ModelError::VerificationMismatch(evidence.id));
            }
            model.verified_evidence.insert(evidence.id);
        }
        for e in model.evidence.values() {
            if e.source.is_empty()
                || e.location.is_empty()
                || e.claim.is_empty()
                || e.method.is_empty()
                || e.source_sha256.len() != 64
                || !e.source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err(ModelError::MissingEvidence(e.id.0.clone()));
            }
        }
        for g in model.grammars.values() {
            model.check_evidence(&g.evidence, &g.id.0)?;
        }
        for o in model.orthographies.values() {
            if o.rules.len() > model.limits.max_spelling_steps {
                return Err(ModelError::ResourceLimit("spelling steps"));
            }
            model.check_evidence(&o.evidence, &o.id.0)?;
            for grammar in &o.compatible_grammars {
                if !model.grammars.contains_key(grammar) {
                    return Err(ModelError::MissingReference(grammar.0.clone()));
                }
            }
            let mut overrides = BTreeSet::new();
            for rule in &o.accent_overrides {
                let p = model
                    .paradigms
                    .get(&rule.paradigm)
                    .ok_or_else(|| ModelError::MissingReference(rule.paradigm.0.clone()))?;
                if !o.compatible_grammars.contains(&p.grammar)
                    || !p.rules.iter().any(|r| r.cell == rule.cell)
                    || p.rules
                        .iter()
                        .any(|r| r.cell == rule.cell && r.stress == StressInstruction::Unspecified)
                    || !overrides.insert((&rule.paradigm, rule.cell.name()))
                {
                    return Err(ModelError::InvalidRule(o.id.0.clone()));
                }
                model.check_evidence(&rule.evidence, &o.id.0)?;
            }
            for rule in &o.rules {
                if rule.from.is_empty() {
                    return Err(ModelError::InvalidRule(o.id.0.clone()));
                }
                model.check_evidence(&rule.evidence, &o.id.0)?;
            }
        }
        for p in model.paradigms.values() {
            if !model.grammars.contains_key(&p.grammar) {
                return Err(ModelError::MissingReference(p.grammar.0.clone()));
            }
            let mut ids = BTreeSet::new();
            for rule in &p.rules {
                if rule.id.0.is_empty() || !ids.insert(&rule.id) {
                    return Err(ModelError::Duplicate(rule.id.0.clone()));
                }
                if rule.stem.is_empty() || rule.cell.pos() != p.pos {
                    return Err(ModelError::InvalidRule(p.id.0.clone()));
                }
                model.check_evidence(&rule.evidence, &p.id.0)?;
                if let Some(change) = &rule.stem_replacement {
                    if change.from.is_empty() || change.from == change.to {
                        return Err(ModelError::InvalidRule(rule.id.0.clone()));
                    }
                    model.check_evidence(&change.evidence, &rule.id.0)?;
                }
            }
        }
        for l in model.lexemes.values() {
            if !model.paradigms.contains_key(&l.paradigm) {
                return Err(ModelError::MissingReference(l.paradigm.0.clone()));
            }
            model.check_evidence(&l.evidence, &l.id.0)?;
            if l.categories.is_empty() {
                return Err(ModelError::InvalidRule(format!(
                    "{}: missing classification",
                    l.id.0
                )));
            }
            let mut categories = BTreeSet::new();
            for assertion in &l.categories {
                if !categories.insert(assertion.category) {
                    return Err(ModelError::Duplicate(format!("{}: category", l.id.0)));
                }
                model.check_evidence(&assertion.evidence, &l.id.0)?;
            }
            let mut genders = BTreeSet::new();
            for assertion in &l.genders {
                if !genders.insert(assertion.gender) {
                    return Err(ModelError::Duplicate(format!("{}: lexical gender", l.id.0)));
                }
                model.check_evidence(&assertion.evidence, &l.id.0)?;
            }
            let mut names = BTreeSet::new();
            for stem in &l.stems {
                if stem.name.is_empty() || !names.insert(&stem.name) {
                    return Err(ModelError::Duplicate(format!(
                        "{}:stem:{}",
                        l.id.0, stem.name
                    )));
                }
                model.check_evidence(&stem.evidence, &l.id.0)?;
            }
            for rule in &model.paradigms[&l.paradigm].rules {
                if let Some(stem) = l.stems.iter().find(|stem| stem.name == rule.stem) {
                    let realized = stem::apply(&stem.text, rule.stem_replacement.as_ref(), &model.limits)?;
                    accent::position(&realized, &rule.suffix, &rule.stress)
                        .map_err(ModelError::Accent)?;
                }
            }
            let mut cells = BTreeSet::new();
            for restriction in &l.unavailable {
                if restriction.cell.pos() != model.paradigms[&l.paradigm].pos
                    || restriction.reason.is_empty()
                    || !cells.insert(restriction.cell.name())
                {
                    return Err(ModelError::InvalidRule(l.id.0.clone()));
                }
                model.check_evidence(&restriction.evidence, &l.id.0)?;
            }
            for relation in &l.relations {
                if !model.lexemes.contains_key(&relation.target) {
                    return Err(ModelError::MissingReference(relation.target.0.clone()));
                }
                model.check_evidence(&relation.evidence, &l.id.0)?;
            }
        }
        for assertion in restrictions {
            let entry = model
                .lexemes
                .get(&assertion.lexeme)
                .ok_or_else(|| ModelError::MissingReference(assertion.lexeme.0.clone()))?;
            if !entry.unavailable.contains(&assertion.restriction) {
                return Err(ModelError::InvalidRule(assertion.lexeme.0));
            }
            for id in &assertion.restriction.evidence {
                if !model.verified_evidence.contains(id) {
                    return Err(ModelError::VerificationMismatch(id.clone()));
                }
            }
            model
                .verified_restrictions
                .insert((assertion.lexeme, assertion.restriction.cell.name()));
        }
        model.validate_abbreviations()?;
        for annotation in model.observations.annotations() {
            model.check_evidence(&annotation.evidence, &annotation.id.0)?;
            if let Some(target) = &annotation.target {
                if !model.lexemes.contains_key(&target.lexeme) {
                    return Err(ModelError::MissingReference(target.lexeme.0.clone()));
                }
                if !model.orthographies.contains_key(&target.orthography) {
                    return Err(ModelError::MissingReference(target.orthography.0.clone()));
                }
                // Do not require agreement with the current generator. An
                // independently asserted analysis may contradict this model.
            }
        }
        Ok(model)
    }

    fn check_evidence(&self, evidence: &[EvidenceId], owner: &str) -> Result<(), ModelError> {
        if evidence.is_empty() {
            return Err(ModelError::MissingEvidence(owner.into()));
        }
        for id in evidence {
            if !self.evidence.contains_key(id) {
                return Err(ModelError::MissingReference(id.0.clone()));
            }
        }
        Ok(())
    }

    /// Identity of the implementation sources used by modeled documents.
    /// Dependency-lock provenance is recorded separately by the artifact pipeline.
    pub fn engine_sha256(&self) -> &'static str {
        static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        HASH.get_or_init(|| {
            let mut digest = Sha256::new();
            for bytes in [
                include_bytes!("morphology.rs").as_slice(),
                include_bytes!("morphology/index.rs").as_slice(),
                include_bytes!("morphology/index/persistence.rs").as_slice(),
                include_bytes!("morphology/accent.rs").as_slice(),
                include_bytes!("morphology/stem.rs").as_slice(),
                include_bytes!("morphology/acceptance.rs").as_slice(),
                include_bytes!("morphology/lexical_acceptance.rs").as_slice(),
                include_bytes!("morphology/abbreviation.rs").as_slice(),
                include_bytes!("morphology/observations.rs").as_slice(),
                include_bytes!("document.rs").as_slice(),
                include_bytes!("witness.rs").as_slice(),
                include_bytes!("matching.rs").as_slice(),
                include_bytes!("orthography.rs").as_slice(),
                include_bytes!("cell.rs").as_slice(),
                include_bytes!("grammar.rs").as_slice(),
            ] {
                digest.update((bytes.len() as u64).to_le_bytes());
                digest.update(bytes);
            }
            format!("{:x}", digest.finalize())
        })
    }

    pub fn limits(&self) -> &ModelLimits {
        &self.limits
    }

    pub fn observations(&self) -> &observations::Archive {
        &self.observations
    }

    pub fn data_sha256(&self) -> &str {
        &self.data_sha256
    }
    pub fn verified_evidence_ids(&self) -> impl Iterator<Item = &EvidenceId> {
        self.verified_evidence.iter()
    }

    pub fn evidence_records(&self) -> impl Iterator<Item = &Evidence> {
        self.evidence.values()
    }
    pub fn orthography(&self, id: &OrthographyId) -> Option<&OrthographyProfile> {
        self.orthographies.get(id)
    }

    pub fn evidence(&self, id: &EvidenceId) -> Option<&Evidence> {
        self.evidence.get(id)
    }
    pub fn lexeme(&self, id: &LexemeId) -> Option<&LexicalEntry> {
        self.lexemes.get(id)
    }
    pub fn grammar(&self, id: &GrammarId) -> Option<&GrammarProfile> {
        self.grammars.get(id)
    }

    pub fn generate(
        &self,
        id: &LexemeId,
        cell: Cell,
        orthography: &OrthographyId,
    ) -> Result<Availability, ModelError> {
        let entry = self
            .lexemes
            .get(id)
            .ok_or_else(|| ModelError::MissingReference(id.0.clone()))?;
        let paradigm = &self.paradigms[&entry.paradigm];
        let spelling = self
            .orthographies
            .get(orthography)
            .ok_or_else(|| ModelError::MissingReference(orthography.0.clone()))?;
        if !spelling.compatible_grammars.contains(&paradigm.grammar) {
            return Err(ModelError::IncompatibleProfiles);
        }
        let restriction = entry.unavailable.iter().find(|r| r.cell == cell);
        if let Some(restriction) = restriction
            && self
                .verified_restrictions
                .contains(&(id.clone(), cell.name()))
        {
            return Ok(Availability::Unavailable(restriction.clone()));
        }
        let mut otherwise = self.realize_with_abbreviations(entry, paradigm, spelling, cell)?;
        if let Availability::Licensed(forms) | Availability::Partial { forms, .. } = &mut otherwise
        {
            let mut bytes = 0usize;
            let mut links = 0usize;
            for form in forms {
                let ids = self
                    .observations
                    .support(id, cell, orthography, &form.surface);
                links = links
                    .checked_add(ids.len())
                    .ok_or(ModelError::ResourceLimit("generation observations"))?;
                bytes = bytes
                    .checked_add(form.retained_bytes())
                    .and_then(|n| ids.iter().try_fold(n, |n, id| n.checked_add(id.0.len())))
                    .ok_or(ModelError::ResourceLimit("generation observations"))?;
                if bytes > self.limits.max_result_bytes || links > self.limits.max_candidates {
                    return Err(ModelError::ResourceLimit("generation observations"));
                }
                form.source_annotations = ids.to_vec();
                bytes += self.qualify_generation(id, cell, form, self.limits.max_result_bytes - bytes)?;
            }
        }
        Ok(match restriction {
            Some(restriction) => Availability::UnresolvedRestriction {
                restriction: restriction.clone(),
                otherwise: Box::new(otherwise),
            },
            None => otherwise,
        })
    }

    fn realize(
        &self,
        entry: &LexicalEntry,
        paradigm: &Paradigm,
        spelling: &OrthographyProfile,
        cell: Cell,
    ) -> Result<Availability, ModelError> {
        let rules: Vec<_> = paradigm
            .rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.cell == cell)
            .collect();
        if rules.is_empty() {
            return Ok(Availability::UnsupportedCell);
        }
        let missing: BTreeSet<_> = rules
            .iter()
            .filter(|(_, r)| !entry.stems.iter().any(|s| s.name == r.stem))
            .map(|(_, r)| r.stem.clone())
            .collect();
        let mut forms = Vec::new();
        let mut result_bytes = 0usize;
        for (_, rule) in rules {
            let Some(stem) = entry.stems.iter().find(|s| s.name == rule.stem) else {
                continue;
            };
            let realized_stem = stem::apply(&stem.text, rule.stem_replacement.as_ref(), &self.limits)?;
            if realized_stem
                .len()
                .checked_add(rule.suffix.len())
                .is_none_or(|n| n > self.limits.max_form_bytes)
            {
                return Err(ModelError::ResourceLimit("form bytes"));
            }
            let underlying = format!("{}{}", realized_stem, rule.suffix);
            let stem_change = rule.stem_replacement.as_ref().map(|_| StemTrace {
                before: stem.text.clone(), after: realized_stem.into_owned(),
            });
            let realized_stem = stem_change.as_ref().map_or(stem.text.as_str(), |t| t.after.as_str());
            let predicted = accent::position(realized_stem, &rule.suffix, &rule.stress)
                .map_err(ModelError::Accent)?;
            let override_rule = spelling
                .accent_overrides
                .iter()
                .find(|r| r.paradigm == paradigm.id && r.cell == cell);
            let accent = accent::render(
                &underlying,
                predicted,
                &spelling.accent,
                override_rule.map(|r| r.mark),
            )
            .map_err(ModelError::Accent)?;
            let mut surface = accent.accented.clone();
            let mut steps = Vec::new();
            let mut evidence = entry.evidence.clone();
            evidence.extend(self.grammars[&paradigm.grammar].evidence.clone());
            evidence.extend(stem.evidence.clone());
            evidence.extend(rule.evidence.clone());
            if let Some(change) = &rule.stem_replacement {
                evidence.extend(change.evidence.clone());
            }
            evidence.extend(spelling.evidence.clone());
            if let Some(rule) = override_rule {
                evidence.extend(rule.evidence.clone());
            }
            for rule in &spelling.rules {
                if rule.to.len() > rule.from.len() {
                    let growth = surface
                        .matches(&rule.from)
                        .count()
                        .checked_mul(rule.to.len() - rule.from.len());
                    if growth
                        .and_then(|n| surface.len().checked_add(n))
                        .is_none_or(|n| n > self.limits.max_form_bytes)
                    {
                        return Err(ModelError::ResourceLimit("form bytes"));
                    }
                }
                let next = surface.replace(&rule.from, &rule.to);
                if next != surface {
                    steps.push((surface, next.clone()));
                    surface = next;
                    evidence.extend(rule.evidence.clone());
                }
            }
            if spelling.unicode == UnicodePolicy::Nfc {
                let normalized: String = surface.nfc().collect();
                if normalized != surface {
                    steps.push((surface, normalized.clone()));
                    surface = normalized;
                }
            }
            evidence.sort();
            evidence.dedup();
            if surface.len() > self.limits.max_form_bytes
                || accent.accented.len() > self.limits.max_form_bytes
            {
                return Err(ModelError::ResourceLimit("form bytes"));
            }
            let size = stem_change.as_ref().map_or(0, StemTrace::retained_bytes)
                + underlying.len()
                + surface.len()
                + accent.accented.len()
                + steps.iter().map(|(a, b)| a.len() + b.len()).sum::<usize>()
                + evidence.iter().map(|e| e.0.len()).sum::<usize>()
                + paradigm.grammar.0.len()
                + spelling.id.0.len()
                + paradigm.id.0.len()
                + rule.id.0.len();
            result_bytes = result_bytes
                .checked_add(size)
                .ok_or(ModelError::ResourceLimit("result bytes"))?;
            if result_bytes > self.limits.max_result_bytes
                || forms.len() >= self.limits.max_candidates
            {
                return Err(ModelError::ResourceLimit("generation results"));
            }
            forms.push(Derivation {
                claim_sha256: String::new(),
                review: GenerationReview::Unreviewed,
                underlying,
                stem_change,
                accent,
                surface,
                source_annotations: Vec::new(),
                abbreviation: None,
                grammar: paradigm.grammar.clone(),
                orthography: spelling.id.clone(),
                paradigm: paradigm.id.clone(),
                rule: rule.id.clone(),
                spelling_steps: steps,
                evidence,
            });
        }
        if missing.is_empty() {
            Ok(Availability::Licensed(forms))
        } else if forms.is_empty() {
            Ok(Availability::MissingStems(missing.into_iter().collect()))
        } else {
            Ok(Availability::Partial {
                forms,
                missing_stems: missing.into_iter().collect(),
            })
        }
    }

    pub fn inventory(&self, id: &LexemeId) -> Result<Vec<Cell>, ModelError> {
        let entry = self
            .lexemes
            .get(id)
            .ok_or_else(|| ModelError::MissingReference(id.0.clone()))?;
        let mut cells: Vec<_> = self.paradigms[&entry.paradigm]
            .rules
            .iter()
            .map(|r| r.cell)
            .chain(entry.unavailable.iter().map(|r| r.cell))
            .collect();
        cells.sort_by_key(Cell::name);
        cells.dedup();
        Ok(cells)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelAnalysis<'a> {
    pub lexeme: &'a LexicalEntry,
    pub cell: Cell,
    pub derivation: Derivation,
    pub lexical_claims: Vec<ReviewedLexicalClaim>,
    pub trace: crate::matching::MatchTrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelAnalysisSet<'a> {
    pub candidates: Vec<ModelAnalysis<'a>>,
    pub unresolved_restrictions: Vec<(&'a LexicalEntry, Restriction)>,
    /// Missing stems may hide further matches; this is not a complete search
    /// merely because all currently generatable forms were examined.
    pub unresolved_cells: Vec<(&'a LexicalEntry, Cell, Vec<String>)>,
}

impl Model {
    pub fn analyze(
        &self,
        surface: &str,
        orthography: &OrthographyId,
        policy: crate::matching::MatchPolicy,
    ) -> Result<ModelAnalysisSet<'_>, ModelError> {
        self.analyze_budgeted(surface, orthography, policy, &mut 0)
    }

    pub(crate) fn analyze_budgeted(
        &self,
        surface: &str,
        orthography: &OrthographyId,
        policy: crate::matching::MatchPolicy,
        checks: &mut usize,
    ) -> Result<ModelAnalysisSet<'_>, ModelError> {
        index::analyze(self, surface, orthography, policy, checks)
    }
}

mod cell_codec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
    pub fn serialize<S: Serializer>(cell: &crate::Cell, serializer: S) -> Result<S::Ok, S::Error> {
        (cell.pos().tag(), cell.name()).serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<crate::Cell, D::Error> {
        let (pos, name) = <(String, String)>::deserialize(deserializer)?;
        let pos = crate::Pos::parse(&pos).ok_or_else(|| D::Error::custom("unknown cell POS"))?;
        crate::Cell::parse(pos, &name).map_err(D::Error::custom)
    }
}

mod pos_codec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
    pub fn serialize<S: Serializer>(pos: &crate::Pos, serializer: S) -> Result<S::Ok, S::Error> {
        pos.tag().serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<crate::Pos, D::Error> {
        let name = String::deserialize(deserializer)?;
        crate::Pos::parse(&name).ok_or_else(|| D::Error::custom("unknown POS"))
    }
}

mod gender_codec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
    use crate::grammar::Gender;
    pub fn serialize<S: Serializer>(gender: &Gender, serializer: S) -> Result<S::Ok, S::Error> {
        match gender { Gender::Masculine => "m", Gender::Feminine => "f", Gender::Neuter => "n" }.serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Gender, D::Error> {
        match String::deserialize(deserializer)?.as_str() {
            "m" => Ok(Gender::Masculine), "f" => Ok(Gender::Feminine), "n" => Ok(Gender::Neuter),
            _ => Err(D::Error::custom("unknown lexical gender")),
        }
    }
}
