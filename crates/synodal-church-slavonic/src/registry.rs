use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope,
    ActiveParticipleShortFormation, AdjectiveClass, AdjectiveForm, AdjectiveLexeme, Animacy,
    AoristFormation, Aspect, AuthorityRole, BreathingMark, BreathingRule, Case, Comparison,
    ComparisonFormation, Confidence, DeterminerDeclension, DeterminerLexeme, EpistemicRole, Error,
    Evidence, EvidenceId, EvidenceKind, FiniteTense, Gender, GenerationPolicy, GrammarCell,
    ImperativeFormation, ImperfectFormation, InitialPresentation, LetterOccurrence, LexemeId,
    NounAnimacyInventory, NounDeclension, NounLexeme, NounNumberInventory, Number,
    NumeralDeclension, NumeralLexeme, ParticiplePrincipalPart, ParticipleTense, ParticipleVoice,
    PositionalOperation, PositionalParadigm, PositionalReplacement, PositionalRule,
    PronounDeclension, PronounEnvironment, PronounFormSelection, PronounLexeme,
    PronounPostpositive, PronounPrefix, Recension, RecensionMappingId, Result,
    ShortMasculineStemFormation, SourceId, SynodalWord, VerbConjugation, VerbLexeme,
    VerbalNounPrincipalPart, normalize_lookup_accentless, validate_adjective_lexeme,
    validate_determiner_lexeme, validate_numeral_lexeme, validate_pronoun_lexeme,
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawLexeme(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawNounRestriction(pub [&'static str; 5]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawPrincipalPart(pub [&'static str; 6]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawExactForm(pub [&'static str; 7]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAlignment(pub [&'static str; 11]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAbbreviation(pub [&'static str; 13]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAbbreviationFamily(pub [&'static str; 12]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAccent(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawAccentParadigm(pub [&'static str; 11]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawPositionalRule(pub [&'static str; 7]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawPositionalParadigm(pub [&'static str; 9]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawTransformationRule(pub [&'static str; 6]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawConflict(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawIrregularOverride(pub [&'static str; 5]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawDefectiveInventory(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawIrregularVerbInventory(pub [&'static str; 8]);
#[derive(Clone, Copy, Debug)]
pub(crate) struct RawReviewedEvidence(pub [&'static str; 6]);

include!("../generated/registry.rs");

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PartOfSpeech {
    Adverb,
    Preposition,
    Conjunction,
    Particle,
    Interjection,
    ProperNoun,
    Noun,
    Adjective,
    Verb,
    Pronoun,
    Determiner,
    Numeral,
    Participle,
}

impl PartOfSpeech {
    pub const ALL: [Self; 13] = [
        Self::Adverb,
        Self::Preposition,
        Self::Conjunction,
        Self::Particle,
        Self::Interjection,
        Self::ProperNoun,
        Self::Noun,
        Self::Adjective,
        Self::Verb,
        Self::Pronoun,
        Self::Determiner,
        Self::Numeral,
        Self::Participle,
    ];

    /// Returns the stable code used by reviewed registries and CLI output.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Adverb => "adverb",
            Self::Preposition => "preposition",
            Self::Conjunction => "conjunction",
            Self::Particle => "particle",
            Self::Interjection => "interjection",
            Self::ProperNoun => "proper-noun",
            Self::Noun => "noun",
            Self::Adjective => "adjective",
            Self::Verb => "verb",
            Self::Pronoun => "pronoun",
            Self::Determiner => "determiner",
            Self::Numeral => "numeral",
            Self::Participle => "participle",
        }
    }

    /// Parses an exact stable registry code.
    #[must_use]
    pub fn from_code(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|part| part.code() == value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LexemeSummary {
    id: LexemeId,
    lemma: String,
    part_of_speech: PartOfSpeech,
    source_id: String,
}

impl LexemeSummary {
    pub(crate) fn new(
        id: LexemeId,
        lemma: String,
        part_of_speech: PartOfSpeech,
        source_id: String,
    ) -> Self {
        Self {
            id,
            lemma,
            part_of_speech,
            source_id,
        }
    }

    #[must_use]
    pub fn id(&self) -> &LexemeId {
        &self.id
    }

    #[must_use]
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    #[must_use]
    pub const fn part_of_speech(&self) -> PartOfSpeech {
        self.part_of_speech
    }

    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AlignmentSummary {
    pub mapping_id: String,
    pub source_lexeme_id: String,
    pub target_lexeme_id: String,
    pub relation: String,
    pub status: String,
    pub morphology: String,
    pub semantics: String,
    pub confidence_basis_points: u16,
    pub transformations: Vec<String>,
    pub review_note: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TransformationRuleSummary {
    pub rule_id: String,
    pub source_recension: String,
    pub target_recension: String,
    pub operation: String,
    pub status: String,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RecensionConflictSummary {
    pub conflict_id: String,
    pub source_lexeme_id: String,
    pub target_lexeme_id: String,
    pub kind: String,
    pub status: String,
    pub supporting_evidence: String,
    pub contradicting_evidence: String,
    pub resolution: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PositionalRuleSummary {
    pub rule_id: String,
    pub input: String,
    pub context: String,
    pub output: String,
    pub exceptions: String,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IrregularOverrideSummary {
    pub lexeme_id: String,
    pub system: String,
    pub cell_set: String,
    pub evidence_id: String,
}

/// One exhaustively reviewed entry from Alypy §104's irregular-verb inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IrregularVerbInventorySummary {
    pub source_order: u8,
    pub headword: String,
    pub systems: Vec<String>,
    pub strategy: String,
    pub implementation_status: String,
    pub evidence_id: String,
    pub note: String,
}

/// Reviewable lexical metadata exposed without leaking the generated registry
/// representation. Empty source fields stay `None` rather than being guessed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LexicalMetadataSummary {
    pub lexeme_id: LexemeId,
    pub class: Option<String>,
    pub stem: Option<String>,
    pub gender: Option<String>,
    pub aspect: Option<String>,
    pub source_id: String,
    pub target_recension: String,
    pub noun_restriction: Option<NounRestrictionSummary>,
    pub principal_parts: Vec<PrincipalPartSummary>,
    pub exact_forms: Vec<ExactFormSummary>,
    pub accents: Vec<AccentSummary>,
    pub accent_paradigms: Vec<AccentParadigmSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NounRestrictionSummary {
    pub number_inventory: String,
    pub animacy_inventory: String,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PrincipalPartSummary {
    pub system: String,
    pub value: String,
    pub formation: Option<String>,
    pub evidence_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ExactFormSummary {
    pub cell: String,
    pub expanded: String,
    pub printed: String,
    pub evidence_id: String,
    pub source_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AccentSummary {
    pub cell: String,
    pub expanded: String,
    pub accented: String,
    pub evidence_id: String,
    pub source_id: String,
    pub source_recension: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AccentParadigmSummary {
    pub paradigm_id: String,
    pub scope: String,
    pub placement: String,
    pub mark: String,
    pub breathing: Option<String>,
    pub evidence_id: String,
    pub source_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExactFormRecord {
    pub expanded: &'static str,
    pub printed: &'static str,
    pub evidence_id: &'static str,
    pub source_kind: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReviewedEvidenceRecord {
    pub id: &'static str,
    pub source_id: &'static str,
    pub source_recension: Recension,
    pub citation: &'static str,
    pub role: &'static str,
    pub note: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AbbreviationRecord {
    pub lexeme_id: &'static str,
    pub sense_id: &'static str,
    pub cell: &'static str,
    pub expanded: &'static str,
    pub printed: &'static str,
    pub rule_id: &'static str,
    pub evidence_id: &'static str,
    pub reversible: bool,
    pub required_marks: &'static str,
    pub context_restrictions: &'static str,
    pub ambiguity: &'static str,
    pub source_recension: &'static str,
    pub target_recension: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AbbreviationFamilyRecord {
    pub lexeme_id: &'static str,
    pub sense_id: &'static str,
    pub expanded_prefix: &'static str,
    pub printed_prefix: &'static str,
    pub rule_id: &'static str,
    pub evidence_id: &'static str,
    pub reversible: bool,
    pub required_marks: &'static str,
    pub context_restrictions: &'static str,
    pub ambiguity: &'static str,
    pub source_recension: &'static str,
    pub target_recension: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AccentRecord {
    pub accented: &'static str,
    pub evidence_id: &'static str,
    pub source_id: &'static str,
    pub source_recension: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InheritedAlignment {
    pub mapping_id: RecensionMappingId,
    pub source_lexeme_id: LexemeId,
    pub confidence: Confidence,
    pub evidence_ids: Vec<String>,
    pub transformations: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DefectiveInventoryRecord {
    pub kind: crate::DefectKind,
    pub field: synodal_church_slavonic_core::MetadataField,
    pub reason: &'static str,
}

pub(crate) fn resolve(lemma: &SynodalWord) -> Result<LexemeSummary> {
    let lookup = normalize_lookup_accentless(lemma.canonical());
    let matches: Vec<&RawLexeme> = LEXEMES
        .iter()
        .filter(|row| normalize_lookup_accentless(row.0[1]) == lookup)
        .collect();
    match matches.as_slice() {
        [] => Err(Error::UnknownLemma { lookup }),
        [row] => summary(row),
        rows => Err(Error::AmbiguousLexeme {
            lexemes: rows.iter().map(|row| LexemeId::from(row.0[0])).collect(),
        }),
    }
}

pub(crate) fn from_id(id: &LexemeId) -> Result<LexemeSummary> {
    let row = raw_by_id(id).ok_or_else(|| Error::UnknownLemma {
        lookup: id.to_string(),
    })?;
    summary(row)
}

/// Returns the contiguous run of rows whose first column equals `id`.
///
/// Every generated table used here is emitted sorted by its first column, so
/// rows sharing a lexeme are adjacent and a binary search replaces a full scan
/// without changing the order the resolver observes. That contract is not
/// implicit: `generated_lexeme_tables_are_sorted` fails if the generator ever
/// stops sorting, because an unsorted table would make these lookups silently
/// return the wrong rows rather than merely being slow.
fn rows_for<T>(rows: &'static [T], key: impl Fn(&T) -> &'static str, id: &str) -> &'static [T] {
    let start = rows.partition_point(|row| key(row) < id);
    let length = rows[start..].partition_point(|row| key(row) == id);
    &rows[start..start + length]
}

pub(crate) fn raw_by_id(id: &LexemeId) -> Option<&'static RawLexeme> {
    rows_for(LEXEMES, |row| row.0[0], id.as_str()).first()
}

pub(crate) fn exact_forms(id: &LexemeId, cell: &str) -> Vec<ExactFormRecord> {
    rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
        .iter()
        .filter(|row| row.0[1] == cell)
        .map(|row| ExactFormRecord {
            expanded: row.0[2],
            printed: row.0[3],
            evidence_id: row.0[4],
            source_kind: row.0[5],
        })
        .collect()
}

pub(crate) fn defect_for(id: &LexemeId, cell: &str) -> Result<Option<DefectiveInventoryRecord>> {
    let row = DEFECTIVE_INVENTORIES.iter().find(|row| {
        if row.0[0] != id.as_str() {
            return false;
        }
        match row.0[1] {
            "outside-inventory" => !row.0[2]
                .split(',')
                .map(str::trim)
                .any(|allowed| allowed == cell),
            "cell-prefix" => cell.starts_with(row.0[2]),
            _ => true,
        }
    });
    let Some(row) = row else {
        return Ok(None);
    };
    let kind = match row.0[3] {
        "historically-absent" => crate::DefectKind::HistoricallyAbsent,
        "evidence-incomplete" => crate::DefectKind::EvidenceIncomplete,
        value => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("generated defect inventory has unknown kind {value:?}"),
            });
        }
    };
    let field = parse_metadata_field(row.0[4])?;
    Ok(Some(DefectiveInventoryRecord {
        kind,
        field,
        reason: row.0[5],
    }))
}

fn parse_metadata_field(value: &str) -> Result<synodal_church_slavonic_core::MetadataField> {
    use synodal_church_slavonic_core::MetadataField;
    let field = match value {
        "present-stem" => MetadataField::PresentStem,
        "present-first-singular" => MetadataField::PresentFirstSingular,
        "present-third-plural" => MetadataField::PresentThirdPlural,
        "future-stem" => MetadataField::FutureStem,
        "future-first-singular" => MetadataField::FutureFirstSingular,
        "future-third-plural" => MetadataField::FutureThirdPlural,
        "imperfect-stem" => MetadataField::ImperfectStem,
        "aorist-stem" => MetadataField::AoristStem,
        "aorist-formation" => MetadataField::AoristFormation,
        "imperative-stem" => MetadataField::ImperativeStem,
        "imperative-formation" => MetadataField::ImperativeFormation,
        "imperfect-formation" => MetadataField::ImperfectFormation,
        "infinitive" => MetadataField::Infinitive,
        "supine-stem" => MetadataField::SupineStem,
        "l-participle-stem" => MetadataField::LParticipleStem,
        "participle-stem" => MetadataField::ParticipleStem,
        "participle-formation" => MetadataField::ParticipleFormation,
        "verbal-noun-stem" => MetadataField::VerbalNounStem,
        "aspect" => MetadataField::Aspect,
        "formation" => MetadataField::Formation,
        "regular-background" => MetadataField::RegularBackground,
        "irregular-override" => MetadataField::IrregularOverride,
        value => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("generated defect inventory has unknown metadata field {value:?}"),
            });
        }
    };
    Ok(field)
}

pub(crate) fn reviewed_evidence(evidence_ids: &str) -> Result<Vec<ReviewedEvidenceRecord>> {
    evidence_ids
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(|id| {
            let row = REVIEWED_EVIDENCE
                .iter()
                .find(|row| row.0[0] == id)
                .ok_or_else(|| Error::ContradictoryMetadata {
                    reason: format!("generated evidence provenance is missing {id}"),
                })?;
            let source_recension = match row.0[2] {
                "old-church-slavonic" => Recension::OldChurchSlavonic,
                "synodal-russian" => Recension::SynodalRussian,
                "mixed" => Recension::Mixed,
                value => {
                    return Err(Error::ContradictoryMetadata {
                        reason: format!("generated evidence {id} has unknown recension {value}"),
                    });
                }
            };
            Ok(ReviewedEvidenceRecord {
                id: row.0[0],
                source_id: row.0[1],
                source_recension,
                citation: row.0[3],
                role: row.0[4],
                note: row.0[5],
            })
        })
        .collect()
}

pub(crate) fn has_exact_forms(id: &LexemeId) -> bool {
    !rows_for(EXACT_FORMS, |row| row.0[0], id.as_str()).is_empty()
}

pub(crate) fn pronoun_profiles(
    id: &LexemeId,
) -> Vec<(Option<Gender>, Option<synodal_church_slavonic_core::Person>)> {
    rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
        .iter()
        .filter(|row| row.0[1].starts_with("pronoun:"))
        .filter_map(|row| {
            let GrammarCell::Pronoun(cell) = row.0[1].parse::<GrammarCell>().ok()? else {
                return None;
            };
            Some((cell.gender, cell.person))
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn is_exact_only(id: &LexemeId) -> bool {
    raw_by_id(id).is_some_and(|row| {
        (row.0[3].is_empty() || matches!(row.0[3], "exact" | "exact-complete-pronoun-table"))
            && has_exact_forms(id)
    })
}

pub(crate) fn has_exact_system(id: &LexemeId, prefix: &str) -> bool {
    rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
        .iter()
        .any(|row| row.0[1].starts_with(prefix))
}

pub(crate) fn has_principal_part(id: &LexemeId, system: &str) -> bool {
    rows_for(PRINCIPAL_PARTS, |row| row.0[0], id.as_str())
        .iter()
        .any(|row| row.0[1] == system)
}

pub(crate) fn has_principal_part_prefix(id: &LexemeId, prefix: &str) -> bool {
    rows_for(PRINCIPAL_PARTS, |row| row.0[0], id.as_str())
        .iter()
        .any(|row| row.0[1].starts_with(prefix))
}

pub(crate) fn has_accent_data(id: &LexemeId) -> bool {
    !rows_for(ACCENTS, |row| row.0[0], id.as_str()).is_empty()
        || !rows_for(ACCENT_PARADIGMS, |row| row.0[0], id.as_str()).is_empty()
        || rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
            .iter()
            .any(|row| row.0[2] != row.0[3])
}

pub(crate) fn irregular_evidence_for(id: &LexemeId, cell_key: &str) -> Option<&'static str> {
    IRREGULAR_OVERRIDES
        .iter()
        .find(|row| {
            if row.0[0] != id.as_str() {
                return false;
            }
            match row.0[1] {
                "present" => cell_key.starts_with("present:"),
                "future" => cell_key.starts_with("future:"),
                "aorist" => cell_key.starts_with("aorist:"),
                "imperfect" => cell_key.starts_with("imperfect:"),
                "imperative" => cell_key.starts_with("imperative:"),
                "noun-singular-dative-and-plural" => {
                    cell_key.starts_with("noun:dative:singular:") || cell_key.contains(":plural:")
                }
                _ => false,
            }
        })
        .map(|row| row.0[3])
}

pub(crate) fn accent_for(id: &LexemeId, cell: &str, expanded: &str) -> Option<AccentRecord> {
    ACCENTS
        .iter()
        .find(|row| row.0[0] == id.as_str() && row.0[1] == cell && row.0[2] == expanded)
        .map(|row| AccentRecord {
            accented: row.0[3],
            evidence_id: row.0[4],
            source_id: row.0[5],
            source_recension: row.0[6],
        })
}

pub(crate) fn accent_paradigm_for(
    id: &LexemeId,
    cell: synodal_church_slavonic_core::GrammarCell,
) -> Result<Option<AccentParadigm>> {
    let rows: Vec<&RawAccentParadigm> = rows_for(ACCENT_PARADIGMS, |row| row.0[0], id.as_str())
        .iter()
        .collect();
    let mut applicable_ids = Vec::new();
    for row in &rows {
        if parse_accent_scope(row.0[2])?.applies_to(cell) {
            applicable_ids.push(row.0[1]);
        }
    }
    applicable_ids.sort_unstable();
    applicable_ids.dedup();
    let Some(paradigm_id) = applicable_ids.first().copied() else {
        return Ok(None);
    };
    if applicable_ids.len() > 1 {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "multiple accent paradigms apply to {} in cell {cell:?}",
                id.as_str()
            ),
        });
    }
    let selected: Vec<&RawAccentParadigm> = rows
        .into_iter()
        .filter(|row| row.0[1] == paradigm_id)
        .collect();
    let first = selected[0];
    for row in &selected {
        if row.0[6..] != first.0[6..] {
            return Err(Error::ContradictoryMetadata {
                reason: format!("accent paradigm {paradigm_id} has inconsistent evidence"),
            });
        }
    }
    let accent_rules = selected
        .iter()
        .map(|row| {
            Ok(AccentRule {
                scope: parse_accent_scope(row.0[2])?,
                placement: parse_accent_placement(row.0[3])?,
                mark: parse_accent_mark(row.0[4])?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let breathing_rules = selected
        .iter()
        .filter(|row| !row.0[5].is_empty())
        .map(|row| {
            let placement =
                row.0[5]
                    .strip_prefix("psili@")
                    .ok_or_else(|| Error::ContradictoryMetadata {
                        reason: format!("invalid breathing rule {:?}", row.0[5]),
                    })?;
            Ok(BreathingRule {
                scope: parse_accent_scope(row.0[2])?,
                placement: parse_accent_placement(placement)?,
                mark: BreathingMark::Psili,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(AccentParadigm {
        id: paradigm_id.into(),
        accent_rules,
        breathing_rules,
        evidence: Evidence {
            id: EvidenceId::from(first.0[6]),
            source: SourceId::from(first.0[7]),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::AccentParadigm,
            authority_roles: vec![AuthorityRole::Accentual, AuthorityRole::Orthographic],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: first.0[8].into(),
            note: Some("reviewed reusable Synodal accent paradigm".into()),
        },
    }))
}

/// Builds the reviewed positional-spelling contract governing one cell.
///
/// Mirrors `accent_paradigm_for`: at most one paradigm may apply to a cell, and
/// every row of the selected paradigm must carry identical evidence, so a
/// lexeme cannot accumulate contradictory positional decisions. Rows sharing a
/// scope accumulate their operations in generated order.
pub(crate) fn positional_paradigm_for(
    id: &LexemeId,
    cell: synodal_church_slavonic_core::GrammarCell,
) -> Result<Option<PositionalParadigm>> {
    let rows: Vec<&RawPositionalParadigm> =
        rows_for(POSITIONAL_PARADIGMS, |row| row.0[0], id.as_str())
            .iter()
            .collect();
    let mut applicable_ids = Vec::new();
    for row in &rows {
        if parse_accent_scope(row.0[2])?.applies_to(cell) {
            applicable_ids.push(row.0[1]);
        }
    }
    applicable_ids.sort_unstable();
    applicable_ids.dedup();
    let Some(paradigm_id) = applicable_ids.first().copied() else {
        return Ok(None);
    };
    if applicable_ids.len() > 1 {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "multiple positional paradigms apply to {} in cell {cell:?}",
                id.as_str()
            ),
        });
    }
    let selected: Vec<&RawPositionalParadigm> = rows
        .into_iter()
        .filter(|row| row.0[1] == paradigm_id)
        .collect();
    let first = selected[0];
    for row in &selected {
        if row.0[4..] != first.0[4..] {
            return Err(Error::ContradictoryMetadata {
                reason: format!("positional paradigm {paradigm_id} has inconsistent evidence"),
            });
        }
    }
    let mut grouped: std::collections::BTreeMap<&str, Vec<PositionalOperation>> =
        std::collections::BTreeMap::new();
    for row in &selected {
        grouped
            .entry(row.0[2])
            .or_default()
            .push(parse_positional_operation(row.0[3])?);
    }
    let mut rules = Vec::with_capacity(grouped.len());
    for (scope, operations) in grouped {
        rules.push(PositionalRule {
            scope: parse_accent_scope(scope)?,
            operations: operations
                .into_iter()
                .filter(|operation| {
                    !matches!(
                        operation,
                        PositionalOperation::Initial(InitialPresentation::Preserve)
                    )
                })
                .collect(),
        });
    }
    Ok(Some(PositionalParadigm {
        id: paradigm_id.into(),
        rules,
        evidence: Evidence {
            id: EvidenceId::from(first.0[4]),
            source: SourceId::from(first.0[5]),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::OrthographicParadigm,
            authority_roles: vec![AuthorityRole::Orthographic],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: first.0[6].into(),
            note: Some("reviewed lexical positional spelling contract".into()),
        },
    }))
}

fn parse_positional_operation(value: &str) -> Result<PositionalOperation> {
    if value == "preserve" {
        return Ok(PositionalOperation::Initial(InitialPresentation::Preserve));
    }
    if value == "decimal-i-before-vowel" {
        return Ok(PositionalOperation::DecimalIBeforeVowel);
    }
    if value == "wide-plural-ending" {
        return Ok(PositionalOperation::WidePluralEnding);
    }
    if let Some(presentation) = value.strip_prefix("initial:") {
        let presentation = match presentation {
            "preserve" => InitialPresentation::Preserve,
            "wide-e" => InitialPresentation::WideE,
            "broad-on" => InitialPresentation::BroadOn,
            "iotated-ya" => InitialPresentation::IotatedYa,
            "digraph-uk" => InitialPresentation::DigraphUk,
            other => return invalid_metadata("positional initial presentation", other),
        };
        return Ok(PositionalOperation::Initial(presentation));
    }
    if let Some(rest) = value.strip_prefix("replace:") {
        let Some((replacement, occurrence)) = rest.split_once('@') else {
            return invalid_metadata("positional replacement", value);
        };
        let replacement = match replacement {
            "wide-e" => PositionalReplacement::WideE,
            "broad-on" => PositionalReplacement::BroadOn,
            "omega" => PositionalReplacement::Omega,
            "decimal-i" => PositionalReplacement::DecimalI,
            "iotated-ya" => PositionalReplacement::IotatedYa,
            "yeri" => PositionalReplacement::Yeri,
            "little-yus" => PositionalReplacement::LittleYus,
            other => return invalid_metadata("positional replacement letter", other),
        };
        let Some((direction, index)) = occurrence.split_once(':') else {
            return invalid_metadata("positional occurrence", occurrence);
        };
        let Ok(index) = index.parse::<u8>() else {
            return invalid_metadata("positional occurrence index", index);
        };
        let occurrence = match direction {
            "from-start" => LetterOccurrence::FromStart(index),
            "from-end" => LetterOccurrence::FromEnd(index),
            other => return invalid_metadata("positional occurrence direction", other),
        };
        return Ok(PositionalOperation::Replace {
            replacement,
            occurrence,
        });
    }
    invalid_metadata("positional operation", value)
}

fn parse_accent_scope(value: &str) -> Result<AccentScope> {
    let parts: Vec<&str> = value.split(':').collect();
    match parts.as_slice() {
        ["all"] => Ok(AccentScope::All),
        ["noun", numbers] => Ok(AccentScope::Noun {
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["noun", numbers, cases] => Ok(AccentScope::NounCases {
            numbers: parse_accent_numbers(numbers)?,
            cases: parse_accent_cases(cases)?,
        }),
        ["pronoun", numbers, cases] => Ok(AccentScope::PronounCases {
            numbers: parse_accent_numbers(numbers)?,
            cases: parse_accent_cases(cases)?,
        }),
        ["pronoun-agreeing", numbers, cases, genders, animacies] => {
            Ok(AccentScope::PronounAgreement {
                numbers: parse_accent_numbers(numbers)?,
                cases: parse_accent_cases(cases)?,
                genders: parse_accent_genders(genders)?,
                animacies: parse_accent_animacies(animacies)?,
            })
        }
        ["numeral", numbers] => Ok(AccentScope::Numeral {
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["adjective", form, comparison, numbers] => Ok(AccentScope::Adjective {
            form: match *form {
                "short" => AdjectiveForm::Short,
                "long" => AdjectiveForm::Long,
                value => return invalid_metadata("accent adjective form", value),
            },
            comparison: match *comparison {
                "positive" => Comparison::Positive,
                "comparative" => Comparison::Comparative,
                "superlative" => Comparison::Superlative,
                value => return invalid_metadata("accent comparison", value),
            },
            numbers: parse_accent_numbers(numbers)?,
        }),
        [
            "adjective-agreeing",
            form,
            comparison,
            numbers,
            cases,
            genders,
            animacies,
        ] => Ok(AccentScope::AdjectiveAgreement {
            form: match *form {
                "short" => AdjectiveForm::Short,
                "long" => AdjectiveForm::Long,
                value => return invalid_metadata("accent adjective form", value),
            },
            comparison: match *comparison {
                "positive" => Comparison::Positive,
                "comparative" => Comparison::Comparative,
                "superlative" => Comparison::Superlative,
                value => return invalid_metadata("accent comparison", value),
            },
            numbers: parse_accent_numbers(numbers)?,
            cases: parse_accent_cases(cases)?,
            genders: parse_accent_genders(genders)?,
            animacies: parse_accent_animacies(animacies)?,
        }),
        ["finite", tense, numbers] => Ok(AccentScope::FiniteVerb {
            tense: match *tense {
                "present" => FiniteTense::Present,
                "future" => FiniteTense::Future,
                "past" => FiniteTense::Past,
                "imperfect" => FiniteTense::Imperfect,
                "aorist" => FiniteTense::Aorist,
                value => return invalid_metadata("accent finite tense", value),
            },
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["participle", tense, voice, form, comparison, numbers] => Ok(AccentScope::Participle {
            tense: ParticipleTense::from_code(tense).ok_or_else(|| {
                Error::ContradictoryMetadata {
                    reason: format!("invalid accent participle tense {tense:?}"),
                }
            })?,
            voice: ParticipleVoice::from_code(voice).ok_or_else(|| {
                Error::ContradictoryMetadata {
                    reason: format!("invalid accent participle voice {voice:?}"),
                }
            })?,
            form: AdjectiveForm::from_code(form).ok_or_else(|| Error::ContradictoryMetadata {
                reason: format!("invalid accent participle form {form:?}"),
            })?,
            comparison: Comparison::from_code(comparison).ok_or_else(|| {
                Error::ContradictoryMetadata {
                    reason: format!("invalid accent participle comparison {comparison:?}"),
                }
            })?,
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["imperative", numbers] => Ok(AccentScope::Imperative {
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["l-participle", numbers] => Ok(AccentScope::LParticiple {
            numbers: parse_accent_numbers(numbers)?,
        }),
        _ => invalid_metadata("accent scope", value),
    }
}

fn parse_accent_cases(value: &str) -> Result<Vec<Case>> {
    value
        .split(',')
        .map(|case| match case {
            "nominative" => Ok(Case::Nominative),
            "genitive" => Ok(Case::Genitive),
            "dative" => Ok(Case::Dative),
            "accusative" => Ok(Case::Accusative),
            "instrumental" => Ok(Case::Instrumental),
            "locative" => Ok(Case::Locative),
            "vocative" => Ok(Case::Vocative),
            value => invalid_metadata("accent case", value),
        })
        .collect()
}

fn parse_accent_numbers(value: &str) -> Result<Vec<Number>> {
    value
        .split(',')
        .map(|number| match number {
            "singular" => Ok(Number::Singular),
            "dual" => Ok(Number::Dual),
            "plural" => Ok(Number::Plural),
            value => invalid_metadata("accent number", value),
        })
        .collect()
}

fn parse_accent_genders(value: &str) -> Result<Vec<Gender>> {
    value
        .split(',')
        .map(|gender| match gender {
            "masculine" => Ok(Gender::Masculine),
            "feminine" => Ok(Gender::Feminine),
            "neuter" => Ok(Gender::Neuter),
            value => invalid_metadata("accent gender", value),
        })
        .collect()
}

fn parse_accent_animacies(value: &str) -> Result<Vec<Animacy>> {
    value
        .split(',')
        .map(|animacy| match animacy {
            "animate" => Ok(Animacy::Animate),
            "inanimate" => Ok(Animacy::Inanimate),
            value => invalid_metadata("accent animacy", value),
        })
        .collect()
}

fn parse_accent_placement(value: &str) -> Result<AccentPlacement> {
    let (kind, offset) = value
        .rsplit_once(':')
        .ok_or_else(|| Error::ContradictoryMetadata {
            reason: format!("invalid accent placement {value:?}"),
        })?;
    let offset = offset
        .parse::<u8>()
        .map_err(|_| Error::ContradictoryMetadata {
            reason: format!("invalid accent placement offset {offset:?}"),
        })?;
    match kind {
        "stem-vowel-from-start" => Ok(AccentPlacement::StemVowelFromStart(offset)),
        "word-vowel-from-start" => Ok(AccentPlacement::WordVowelFromStart(offset)),
        "ending-vowel-from-end" => Ok(AccentPlacement::EndingVowelFromEnd(offset)),
        value => invalid_metadata("accent placement", value),
    }
}

fn parse_accent_mark(value: &str) -> Result<AccentMark> {
    match value {
        "acute" => Ok(AccentMark::Acute),
        "grave" => Ok(AccentMark::Grave),
        "kamora" => Ok(AccentMark::Kamora),
        value => invalid_metadata("accent mark", value),
    }
}

pub(crate) fn noun_lexeme(id: &LexemeId) -> Result<NounLexeme> {
    let row = require_pos(id, PartOfSpeech::Noun)?;
    let restriction = NOUN_RESTRICTIONS
        .iter()
        .find(|restriction| restriction.0[0] == id.as_str());
    let number_inventory = restriction.map_or(Ok(NounNumberInventory::All), |restriction| {
        parse_noun_number_inventory(restriction.0[1])
    })?;
    let animacy_inventory = restriction.map_or(Ok(NounAnimacyInventory::All), |restriction| {
        parse_noun_animacy_inventory(restriction.0[2])
    })?;
    Ok(NounLexeme {
        lemma: SynodalWord::parse(row.0[1])?,
        stem: SynodalWord::parse(row.0[4])?,
        gender: parse_gender(row.0[5])?,
        declension: match row.0[3] {
            "first-hard-m" | "inherited-first-hard-m" => NounDeclension::FirstHardMasculine,
            "first-hard-u-stem-m" => NounDeclension::FirstHardMasculineUStem,
            "first-hard-in-ethnonym-m" => NounDeclension::FirstHardMasculineInEthnonym,
            "first-hard-ud-es-m" => NounDeclension::FirstHardMasculineUdEs,
            "first-hard-velar-m" => NounDeclension::FirstHardVelarMasculine,
            "first-mixed-m" => NounDeclension::FirstMixedMasculine,
            "first-mixed-ts-m" => NounDeclension::FirstMixedTsMasculine,
            "first-hard-n" => NounDeclension::FirstHardNeuter,
            "first-soft-m" => NounDeclension::FirstSoftMasculine,
            "first-soft-agent-tel-m" => NounDeclension::FirstSoftMasculineAgentTel,
            "first-soft-lord-m" => NounDeclension::FirstSoftMasculineLord,
            "first-soft-j-m" => NounDeclension::FirstSoftMasculineJ,
            "first-soft-ey-m" => NounDeclension::FirstSoftMasculineEy,
            "first-soft-n" => NounDeclension::FirstSoftNeuter,
            "first-soft-ishche-n" => NounDeclension::FirstSoftNeuterIshche,
            "first-soft-ie-n" => NounDeclension::FirstSoftNeuterIe,
            "second-hard" => NounDeclension::SecondHard,
            "second-hard-velar" => NounDeclension::SecondHardVelar,
            "second-soft" => NounDeclension::SecondSoft,
            "second-soft-postvocalic-ancient-pl" => {
                NounDeclension::SecondSoftPostvocalicAncientPlural
            }
            "second-soft-m-ia" => NounDeclension::SecondSoftMasculineIa,
            "second-soft-f-ia" => NounDeclension::SecondSoftFeminineIa,
            "second-mixed" => NounDeclension::SecondMixed,
            "third-f" => NounDeclension::ThirdFeminine,
            "third-m" => NounDeclension::ThirdMasculine,
            "fourth-neuter-en" => NounDeclension::FourthNeuterEn,
            "fourth-neuter-es" => NounDeclension::FourthNeuterEs,
            "fourth-neuter-es-alt-first" => NounDeclension::FourthNeuterEsAlternatingFirst,
            "fourth-neuter-es-paired-dual" => NounDeclension::FourthNeuterEsPairedDual,
            "fourth-neuter-at" => NounDeclension::FourthNeuterAt,
            "fourth-feminine-er" => NounDeclension::FourthFeminineEr,
            "fourth-feminine-er-daughter" => NounDeclension::FourthFeminineErDaughter,
            "fourth-feminine-ov" => NounDeclension::FourthFeminineOv,
            "fourth-feminine-ov-syncopating" => NounDeclension::FourthFeminineOvSyncopating,
            "fourth-masculine-en" => NounDeclension::FourthMasculineEn,
            "fourth-masculine-en-day" => NounDeclension::FourthMasculineEnDay,
            "fourth-masculine-en-kamen" => NounDeclension::FourthMasculineEnKamen,
            "indeclinable" => NounDeclension::Indeclinable,
            value => return invalid_metadata("noun class", value),
        },
        number_inventory,
        animacy_inventory,
    })
}

fn parse_noun_number_inventory(value: &str) -> Result<NounNumberInventory> {
    match value {
        "all" => Ok(NounNumberInventory::All),
        "singular-only" => Ok(NounNumberInventory::SingularOnly),
        "dual-only" => Ok(NounNumberInventory::DualOnly),
        "plural-only" => Ok(NounNumberInventory::PluralOnly),
        "singular-and-dual" => Ok(NounNumberInventory::SingularAndDual),
        "singular-and-plural" => Ok(NounNumberInventory::SingularAndPlural),
        "dual-and-plural" => Ok(NounNumberInventory::DualAndPlural),
        value => invalid_metadata("noun number inventory", value),
    }
}

fn parse_noun_animacy_inventory(value: &str) -> Result<NounAnimacyInventory> {
    match value {
        "any" => Ok(NounAnimacyInventory::All),
        "animate" => Ok(NounAnimacyInventory::AnimateOnly),
        "inanimate" => Ok(NounAnimacyInventory::InanimateOnly),
        value => invalid_metadata("noun animacy inventory", value),
    }
}

pub(crate) fn adjective_lexeme(id: &LexemeId) -> Result<AdjectiveLexeme> {
    adjectival_lexeme(id, PartOfSpeech::Adjective)
}

pub(crate) fn determiner_lexeme(id: &LexemeId) -> Result<DeterminerLexeme> {
    let row = require_pos(id, PartOfSpeech::Determiner)?;
    let lexeme = DeterminerLexeme::new(
        SynodalWord::parse(row.0[1])?,
        SynodalWord::parse(row.0[4])?,
        match row.0[3] {
            "determiner-pronominal-hard" => DeterminerDeclension::PronominalHard,
            "determiner-ves-mixed" => DeterminerDeclension::VesMixed,
            "determiner-vsyak-mixed" => DeterminerDeclension::VsyakMixed,
            "determiner-full-sk" => DeterminerDeclension::FullSk,
            value => return invalid_metadata("determiner class", value),
        },
    );
    validate_determiner_lexeme(&lexeme)?;
    Ok(lexeme)
}

pub(crate) fn numeral_lexeme(id: &LexemeId) -> Result<NumeralLexeme> {
    let row = require_pos(id, PartOfSpeech::Numeral)?;
    let lexeme = NumeralLexeme::new(
        SynodalWord::parse(row.0[1])?,
        SynodalWord::parse(row.0[4])?,
        match row.0[3] {
            "numeral-cardinal-one" => NumeralDeclension::CardinalOne,
            "numeral-cardinal-two" => NumeralDeclension::CardinalTwo,
            "numeral-cardinal-both" => NumeralDeclension::CardinalBoth,
            "numeral-cardinal-three" => NumeralDeclension::CardinalThree,
            "numeral-cardinal-four" => NumeralDeclension::CardinalFour,
            "numeral-cardinal-i-stem" => NumeralDeclension::CardinalIStem,
            "numeral-cardinal-ten" => NumeralDeclension::CardinalTen,
            "numeral-cardinal-hundred" => NumeralDeclension::CardinalHundred,
            "numeral-cardinal-second-hard" => NumeralDeclension::CardinalSecondHard,
            "numeral-cardinal-second-mixed" => NumeralDeclension::CardinalSecondMixed,
            "numeral-cardinal-first-hard-m" => NumeralDeclension::CardinalFirstHardMasculine,
            "numeral-cardinal-third-f" => NumeralDeclension::CardinalThirdFeminine,
            "ordinal-hard" => NumeralDeclension::OrdinalHard,
            "ordinal-ii" => NumeralDeclension::OrdinalIi,
            "numeral-collective-agreeing" => NumeralDeclension::CollectiveAgreeing,
            "numeral-collective-governing-neuter" => NumeralDeclension::CollectiveGoverningNeuter,
            "numeral-collective-hard-plural" => NumeralDeclension::CollectiveHardPlural,
            "numeral-multiplicative-hard" => NumeralDeclension::MultiplicativeHard,
            "numeral-multiplicative-soft" => NumeralDeclension::MultiplicativeSoft,
            "numeral-fractional-hard" => NumeralDeclension::FractionalHard,
            "numeral-fractional-first-u" => NumeralDeclension::FractionalFirstHardUStem,
            "numeral-fractional-second-hard" => NumeralDeclension::FractionalSecondHard,
            "numeral-fractional-third-f" => NumeralDeclension::FractionalThirdFeminine,
            value => return invalid_metadata("numeral class", value),
        },
    );
    validate_numeral_lexeme(&lexeme)?;
    Ok(lexeme)
}

pub(crate) fn pronoun_lexeme(id: &LexemeId) -> Result<PronounLexeme> {
    let row = require_pos(id, PartOfSpeech::Pronoun)?;
    let lemma = SynodalWord::parse(row.0[1])?;
    let class = row.0[3];
    let lexeme = match class {
        "pronoun-personal-first" => PronounLexeme::closed(lemma, PronounDeclension::PersonalFirst),
        "pronoun-personal-second" => {
            PronounLexeme::closed(lemma, PronounDeclension::PersonalSecond)
        }
        "pronoun-reflexive" => PronounLexeme::closed(lemma, PronounDeclension::Reflexive),
        "pronoun-reflexive-clitic" => PronounLexeme::closed(lemma, PronounDeclension::Reflexive)
            .with_selection(PronounFormSelection::Enclitic),
        "pronoun-third-person" => PronounLexeme::closed(lemma, PronounDeclension::ThirdPerson)
            .with_environment(PronounEnvironment::ContextualVariants),
        "pronoun-third-person-demonstrative" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::ThirdPersonAndDemonstrative,
        )
        .with_environment(PronounEnvironment::ContextualVariants),
        "pronoun-relative-izhe" => PronounLexeme::closed(lemma, PronounDeclension::ThirdPerson)
            .with_environment(PronounEnvironment::ContextualVariants)
            .with_postpositive(PronounPostpositive::Zhe),
        "pronoun-proximal-sei" => PronounLexeme::closed(lemma, PronounDeclension::ProximalSei),
        "pronoun-soft" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::Soft,
        ),
        "pronoun-soft-i-alternating" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::SoftIAlternating,
        ),
        "pronoun-hard" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::Hard,
        ),
        "pronoun-mixed-possessive" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::MixedPossessive,
        ),
        "pronoun-short-hard" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::ShortHard,
        ),
        "pronoun-short-ov-mixed" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::ShortOvMixed,
        ),
        "pronoun-short-velar" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::ShortVelar,
        ),
        "pronoun-quantity-velar" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::QuantityVelar,
        ),
        "pronoun-full-hard" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::FullHard,
        ),
        "pronoun-full-soft" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::FullSoft,
        ),
        "pronoun-full-velar" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::FullVelar,
        ),
        "pronoun-interrogative-kii" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeKii)
        }
        "pronoun-interrogative-who" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWho)
        }
        "pronoun-interrogative-what" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWhat)
        }
        "pronoun-indefinite-who" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWho)
                .with_prefix(PronounPrefix::IndefiniteNe)
        }
        "pronoun-indefinite-what" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWhat)
                .with_prefix(PronounPrefix::IndefiniteNe)
        }
        "pronoun-indefinite-kii" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeKii)
                .with_prefix(PronounPrefix::IndefiniteNe)
        }
        "pronoun-negative-who" => PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWho)
            .with_prefix(PronounPrefix::NegativeNi),
        "pronoun-negative-what" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWhat)
                .with_prefix(PronounPrefix::NegativeNi)
        }
        "pronoun-negative-kii" => PronounLexeme::closed(lemma, PronounDeclension::InterrogativeKii)
            .with_prefix(PronounPrefix::NegativeNi),
        "pronoun-negative-full-hard" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::FullHard,
        )
        .with_prefix(PronounPrefix::NegativeNi),
        "pronoun-kii-zhdo" => PronounLexeme::closed(lemma, PronounDeclension::InterrogativeKii)
            .with_postpositive(PronounPostpositive::Zhdo),
        "pronoun-negative-who-zhe" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWho)
                .with_prefix(PronounPrefix::NegativeNi)
                .with_postpositive(PronounPostpositive::Zhe)
        }
        "pronoun-negative-what-zhe" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWhat)
                .with_prefix(PronounPrefix::NegativeNi)
                .with_postpositive(PronounPostpositive::Zhe)
        }
        value => return invalid_metadata("pronoun class", value),
    };
    validate_pronoun_lexeme(&lexeme)?;
    Ok(lexeme)
}

fn adjectival_lexeme(id: &LexemeId, expected: PartOfSpeech) -> Result<AdjectiveLexeme> {
    let row = require_pos(id, expected)?;
    let short_masculine = PRINCIPAL_PARTS
        .iter()
        .find(|part| part.0[0] == id.as_str() && part.0[1] == "short-masculine-stem");
    let lexeme = AdjectiveLexeme {
        lemma: SynodalWord::parse(row.0[1])?,
        stem: SynodalWord::parse(row.0[4])?,
        class: match row.0[3] {
            "hard-short" => AdjectiveClass::Hard,
            "soft-short" => AdjectiveClass::Soft,
            "velar-short" => AdjectiveClass::Velar,
            "possessive-hard-short" => AdjectiveClass::PossessiveHard,
            "possessive-soft-short" => AdjectiveClass::PossessiveSoft,
            "possessive-j-short" => AdjectiveClass::PossessiveJ,
            "possessive-in" => AdjectiveClass::PossessiveIn,
            "possessive-sk" => AdjectiveClass::PossessiveSk,
            "possessive-ii" => AdjectiveClass::PossessiveIi,
            value => return invalid_metadata("adjective class", value),
        },
        short_masculine_stem: short_masculine
            .map(|part| SynodalWord::parse(part.0[2]))
            .transpose()?,
        short_masculine_formation: short_masculine
            .map(|part| parse_short_masculine_formation(part.0[3]))
            .transpose()?,
        comparative_stem: PRINCIPAL_PARTS
            .iter()
            .find(|part| part.0[0] == id.as_str() && part.0[1] == "comparative-stem")
            .map(|part| SynodalWord::parse(part.0[2]))
            .transpose()?,
        comparison_formation: PRINCIPAL_PARTS
            .iter()
            .find(|part| part.0[0] == id.as_str() && part.0[1] == "comparative-stem")
            .map(|part| parse_comparison_formation(part.0[3]))
            .transpose()?,
    };
    validate_adjective_lexeme(&lexeme)?;
    Ok(lexeme)
}

fn parse_short_masculine_formation(value: &str) -> Result<ShortMasculineStemFormation> {
    match value {
        "double-n-reduction" => Ok(ShortMasculineStemFormation::DoubleNReduction),
        "mobile-e-insertion" => Ok(ShortMasculineStemFormation::MobileEInsertion),
        "mobile-o-insertion" => Ok(ShortMasculineStemFormation::MobileOInsertion),
        value => invalid_metadata("short masculine formation", value),
    }
}

fn parse_comparison_formation(value: &str) -> Result<ComparisonFormation> {
    match value {
        "ancient-hard" => Ok(ComparisonFormation::AncientHard),
        "ancient-soft" => Ok(ComparisonFormation::AncientSoft),
        "later-yat" => Ok(ComparisonFormation::LaterYat),
        "later-ai" => Ok(ComparisonFormation::LaterAi),
        value => invalid_metadata("comparison formation", value),
    }
}

fn parse_active_participle_short_formation(value: &str) -> Result<ActiveParticipleShortFormation> {
    match value {
        "present-first-unpalatalized" => {
            Ok(ActiveParticipleShortFormation::PresentFirstUnpalatalized)
        }
        "present-first-palatalized" => Ok(ActiveParticipleShortFormation::PresentFirstPalatalized),
        "present-second" => Ok(ActiveParticipleShortFormation::PresentSecond),
        "present-after-sibilant" => Ok(ActiveParticipleShortFormation::PresentAfterSibilant),
        "past-consonant" => Ok(ActiveParticipleShortFormation::PastConsonant),
        "past-vowel" => Ok(ActiveParticipleShortFormation::PastVowel),
        "past-iotated" => Ok(ActiveParticipleShortFormation::PastIotated),
        value => invalid_metadata("active participle short formation", value),
    }
}

pub(crate) fn verb_lexeme(id: &LexemeId) -> Result<VerbLexeme> {
    let row = require_pos(id, PartOfSpeech::Verb)?;
    let conjugation = match row.0[3] {
        "first-unpalatalized" => VerbConjugation::FirstUnpalatalized,
        "first-palatalized" => VerbConjugation::FirstPalatalized,
        "second" => VerbConjugation::Second,
        "archaic" => VerbConjugation::Archaic,
        value => return invalid_metadata("verb conjugation", value),
    };
    let aspect = match row.0[6] {
        "imperfective" => Aspect::Imperfective,
        "perfective" => Aspect::Perfective,
        "biaspectual" => Aspect::Biaspectual,
        "" | "unknown" => Aspect::Unknown,
        value => return invalid_metadata("aspect", value),
    };
    let part = |system: &str| {
        PRINCIPAL_PARTS
            .iter()
            .find(|part| part.0[0] == id.as_str() && part.0[1] == system)
    };
    let parsed_part = |system: &str| -> Result<Option<SynodalWord>> {
        part(system)
            .map(|entry| SynodalWord::parse(entry.0[2]))
            .transpose()
    };
    let participle_part = |prefix: &str| -> Result<Option<ParticiplePrincipalPart>> {
        let short = part(&format!("{prefix}-short-stem"));
        let long = part(&format!("{prefix}-long-stem"));
        if short.is_none() && long.is_none() {
            return Ok(None);
        }
        let short_metadata = short.map(|entry| entry.0[3]);
        let long_metadata = long.map(|entry| entry.0[3]);
        let class_code = long_metadata
            .or(short_metadata)
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        let class = match class_code {
            "hard" => AdjectiveClass::Hard,
            "soft" => AdjectiveClass::Soft,
            value => return invalid_metadata("participial adjective class", value),
        };
        for entry in [short, long].into_iter().flatten() {
            if entry.0[3].split(':').next() != Some(class_code) {
                return Err(Error::ContradictoryMetadata {
                    reason: format!(
                        "participial stems for {} use inconsistent classes",
                        id.as_str()
                    ),
                });
            }
        }
        Ok(Some(ParticiplePrincipalPart {
            short_stem: short
                .map(|entry| SynodalWord::parse(entry.0[2]))
                .transpose()?,
            short_formation: short_metadata
                .and_then(|metadata| metadata.split_once(':').map(|(_, value)| value))
                .map(parse_active_participle_short_formation)
                .transpose()?,
            long_stem: long
                .map(|entry| SynodalWord::parse(entry.0[2]))
                .transpose()?,
            class,
        }))
    };

    let lexeme = VerbLexeme {
        lemma: SynodalWord::parse(row.0[1])?,
        aspect,
        conjugation,
        present_stem: nonempty_word(row.0[4])?,
        present_first_singular: parsed_part("present-first-singular")?,
        present_third_plural: parsed_part("present-third-plural")?,
        future_stem: parsed_part("future-stem")?,
        future_first_singular: parsed_part("future-first-singular")?,
        future_third_plural: parsed_part("future-third-plural")?,
        imperfect_stem: parsed_part("imperfect-stem")?,
        imperfect_formation: part("imperfect-stem")
            .map(|entry| parse_imperfect(entry.0[3]))
            .transpose()?,
        aorist_stem: parsed_part("aorist-stem")?,
        aorist_formation: part("aorist-stem")
            .map(|entry| parse_aorist(entry.0[3]))
            .transpose()?,
        imperative_stem: parsed_part("imperative-stem")?,
        imperative_formation: part("imperative-stem")
            .map(|entry| parse_imperative(entry.0[3]))
            .transpose()?,
        l_participle_stem: parsed_part("l-participle-stem")?,
        l_participle_masculine_singular_stem: parsed_part("l-participle-masculine-singular-stem")?,
        present_active_participle: participle_part("present-active-participle")?,
        past_active_participle: participle_part("past-active-participle")?,
        present_passive_participle: participle_part("present-passive-participle")?,
        past_passive_participle: participle_part("past-passive-participle")?,
        verbal_noun: part("verbal-noun-ie-platform")
            .map(|entry| VerbalNounPrincipalPart::past_passive_ie(entry.0[2]))
            .transpose()?,
    };
    let future_part_count = [
        lexeme.future_stem.is_some(),
        lexeme.future_first_singular.is_some(),
        lexeme.future_third_plural.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if !matches!(future_part_count, 0 | 3) {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "independent future principal parts for {} must be supplied as a complete triple",
                id.as_str()
            ),
        });
    }
    Ok(lexeme)
}

pub(crate) fn all_lexemes() -> Result<Vec<LexemeSummary>> {
    LEXEMES.iter().map(summary).collect()
}

pub(crate) fn lexical_metadata(id: &LexemeId) -> Result<LexicalMetadataSummary> {
    let row = raw_by_id(id).ok_or_else(|| Error::UnknownLemma {
        lookup: id.to_string(),
    })?;
    let optional = |value: &str| (!value.is_empty()).then(|| value.to_owned());
    Ok(LexicalMetadataSummary {
        lexeme_id: id.clone(),
        class: optional(row.0[3]),
        stem: optional(row.0[4]),
        gender: optional(row.0[5]),
        aspect: optional(row.0[6]),
        source_id: row.0[7].into(),
        target_recension: row.0[8].into(),
        noun_restriction: NOUN_RESTRICTIONS
            .iter()
            .find(|restriction| restriction.0[0] == id.as_str())
            .map(|restriction| NounRestrictionSummary {
                number_inventory: restriction.0[1].into(),
                animacy_inventory: restriction.0[2].into(),
                evidence_id: restriction.0[3].into(),
            }),
        principal_parts: rows_for(PRINCIPAL_PARTS, |row| row.0[0], id.as_str())
            .iter()
            .map(|part| PrincipalPartSummary {
                system: part.0[1].into(),
                value: part.0[2].into(),
                formation: optional(part.0[3]),
                evidence_id: part.0[4].into(),
            })
            .collect(),
        exact_forms: rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
            .iter()
            .map(|form| ExactFormSummary {
                cell: form.0[1].into(),
                expanded: form.0[2].into(),
                printed: form.0[3].into(),
                evidence_id: form.0[4].into(),
                source_kind: form.0[5].into(),
            })
            .collect(),
        accents: rows_for(ACCENTS, |row| row.0[0], id.as_str())
            .iter()
            .map(|accent| AccentSummary {
                cell: accent.0[1].into(),
                expanded: accent.0[2].into(),
                accented: accent.0[3].into(),
                evidence_id: accent.0[4].into(),
                source_id: accent.0[5].into(),
                source_recension: accent.0[6].into(),
            })
            .collect(),
        accent_paradigms: rows_for(ACCENT_PARADIGMS, |row| row.0[0], id.as_str())
            .iter()
            .map(|accent| AccentParadigmSummary {
                paradigm_id: accent.0[1].into(),
                scope: accent.0[2].into(),
                placement: accent.0[3].into(),
                mark: accent.0[4].into(),
                breathing: optional(accent.0[5]),
                evidence_id: accent.0[6].into(),
                source_id: accent.0[7].into(),
            })
            .collect(),
    })
}

pub(crate) fn alignments() -> Result<Vec<AlignmentSummary>> {
    ALIGNMENTS
        .iter()
        .map(|row| {
            let confidence_basis_points =
                row.0[7]
                    .parse::<u16>()
                    .map_err(|_| Error::ContradictoryMetadata {
                        reason: format!("invalid mapping confidence {}", row.0[7]),
                    })?;
            Ok(AlignmentSummary {
                mapping_id: row.0[0].into(),
                source_lexeme_id: row.0[1].into(),
                target_lexeme_id: row.0[2].into(),
                relation: row.0[3].into(),
                status: row.0[4].into(),
                morphology: row.0[5].into(),
                semantics: row.0[6].into(),
                confidence_basis_points,
                transformations: split_list(row.0[9]),
                review_note: row.0[10].into(),
            })
        })
        .collect()
}

pub(crate) fn transformation_rules() -> Vec<TransformationRuleSummary> {
    TRANSFORMATION_RULES
        .iter()
        .map(|row| TransformationRuleSummary {
            rule_id: row.0[0].into(),
            source_recension: row.0[1].into(),
            target_recension: row.0[2].into(),
            operation: row.0[3].into(),
            status: row.0[4].into(),
            evidence_id: row.0[5].into(),
        })
        .collect()
}

pub(crate) fn conflicts() -> Vec<RecensionConflictSummary> {
    CONFLICTS
        .iter()
        .map(|row| RecensionConflictSummary {
            conflict_id: row.0[0].into(),
            source_lexeme_id: row.0[1].into(),
            target_lexeme_id: row.0[2].into(),
            kind: row.0[3].into(),
            status: row.0[4].into(),
            supporting_evidence: row.0[5].into(),
            contradicting_evidence: row.0[6].into(),
            resolution: row.0[7].into(),
        })
        .collect()
}

pub(crate) fn positional_rules() -> Vec<PositionalRuleSummary> {
    POSITIONAL_RULES
        .iter()
        .map(|row| PositionalRuleSummary {
            rule_id: row.0[0].into(),
            input: row.0[1].into(),
            context: row.0[2].into(),
            output: row.0[3].into(),
            exceptions: row.0[4].into(),
            evidence_id: row.0[5].into(),
        })
        .collect()
}

pub(crate) fn irregular_overrides() -> Vec<IrregularOverrideSummary> {
    IRREGULAR_OVERRIDES
        .iter()
        .map(|row| IrregularOverrideSummary {
            lexeme_id: row.0[0].into(),
            system: row.0[1].into(),
            cell_set: row.0[2].into(),
            evidence_id: row.0[3].into(),
        })
        .collect()
}

pub(crate) fn irregular_verb_inventory() -> Result<Vec<IrregularVerbInventorySummary>> {
    IRREGULAR_VERB_INVENTORY
        .iter()
        .map(|row| {
            let source_order =
                row.0[0]
                    .parse::<u8>()
                    .map_err(|_| Error::ContradictoryMetadata {
                        reason: format!("invalid Alypy §104 source order {:?}", row.0[0]),
                    })?;
            Ok(IrregularVerbInventorySummary {
                source_order,
                headword: row.0[1].into(),
                systems: split_list(row.0[2]),
                strategy: row.0[3].into(),
                implementation_status: row.0[4].into(),
                evidence_id: row.0[5].into(),
                note: row.0[6].into(),
            })
        })
        .collect()
}

pub(crate) fn abbreviations_for(id: &LexemeId, sense_id: &str) -> Vec<AbbreviationRecord> {
    ABBREVIATIONS
        .iter()
        .filter(|row| row.0[0] == id.as_str() && row.0[1] == sense_id)
        .map(|row| AbbreviationRecord {
            lexeme_id: row.0[0],
            sense_id: row.0[1],
            cell: row.0[2],
            expanded: row.0[3],
            printed: row.0[4],
            rule_id: row.0[5],
            evidence_id: row.0[6],
            reversible: row.0[7] == "true",
            required_marks: row.0[8],
            context_restrictions: row.0[9],
            ambiguity: row.0[10],
            source_recension: row.0[11],
            target_recension: row.0[12],
        })
        .collect()
}

pub(crate) fn abbreviations_for_printed(printed: &str) -> Vec<AbbreviationRecord> {
    ABBREVIATIONS
        .iter()
        .filter(|row| row.0[4] == printed)
        .map(|row| AbbreviationRecord {
            lexeme_id: row.0[0],
            sense_id: row.0[1],
            cell: row.0[2],
            expanded: row.0[3],
            printed: row.0[4],
            rule_id: row.0[5],
            evidence_id: row.0[6],
            reversible: row.0[7] == "true",
            required_marks: row.0[8],
            context_restrictions: row.0[9],
            ambiguity: row.0[10],
            source_recension: row.0[11],
            target_recension: row.0[12],
        })
        .collect()
}

pub(crate) fn abbreviation_families_for(
    id: &LexemeId,
    sense_id: &str,
) -> Vec<AbbreviationFamilyRecord> {
    ABBREVIATION_FAMILIES
        .iter()
        .filter(|row| row.0[0] == id.as_str() && row.0[1] == sense_id)
        .map(abbreviation_family_record)
        .collect()
}

pub(crate) fn abbreviation_family_records() -> Vec<AbbreviationFamilyRecord> {
    ABBREVIATION_FAMILIES
        .iter()
        .map(abbreviation_family_record)
        .collect()
}

fn abbreviation_family_record(row: &RawAbbreviationFamily) -> AbbreviationFamilyRecord {
    AbbreviationFamilyRecord {
        lexeme_id: row.0[0],
        sense_id: row.0[1],
        expanded_prefix: row.0[2],
        printed_prefix: row.0[3],
        rule_id: row.0[4],
        evidence_id: row.0[5],
        reversible: row.0[6] == "true",
        required_marks: row.0[7],
        context_restrictions: row.0[8],
        ambiguity: row.0[9],
        source_recension: row.0[10],
        target_recension: row.0[11],
    }
}

pub(crate) fn noun_uses_inherited_class(id: &LexemeId) -> bool {
    raw_by_id(id).is_some_and(|row| row.0[3].starts_with("inherited-"))
}

pub(crate) fn inherited_alignments(
    id: &LexemeId,
    policy: GenerationPolicy,
    threshold_basis_points: u16,
) -> Result<Vec<InheritedAlignment>> {
    let candidates: Vec<&RawAlignment> = ALIGNMENTS
        .iter()
        .filter(|row| {
            row.0[2] == id.as_str()
                && row.0[4] != "rejected"
                && row.0[6] != "false-friend"
                && (policy == GenerationPolicy::Exploratory
                    || matches!(row.0[4], "reviewed" | "automatically-validated"))
        })
        .collect();
    if candidates.is_empty() {
        return Err(Error::MissingRecensionMapping { source: id.clone() });
    }
    if policy != GenerationPolicy::Exploratory && candidates.len() > 1 {
        return Err(Error::AmbiguousRecensionMapping {
            mappings: candidates
                .iter()
                .map(|row| RecensionMappingId::from(row.0[0]))
                .collect(),
        });
    }
    candidates
        .into_iter()
        .map(|row| {
            let confidence_basis_points =
                row.0[7]
                    .parse::<u16>()
                    .map_err(|_| Error::ContradictoryMetadata {
                        reason: format!("invalid mapping confidence {}", row.0[7]),
                    })?;
            if policy == GenerationPolicy::Productive
                && confidence_basis_points < threshold_basis_points
            {
                return Err(Error::MissingRecensionMapping { source: id.clone() });
            }
            let confidence =
                Confidence::from_basis_points(confidence_basis_points).ok_or_else(|| {
                    Error::ContradictoryMetadata {
                        reason: "mapping confidence exceeds 10000 basis points".into(),
                    }
                })?;
            Ok(InheritedAlignment {
                mapping_id: RecensionMappingId::from(row.0[0]),
                source_lexeme_id: LexemeId::from(row.0[1]),
                confidence,
                evidence_ids: split_list(row.0[8]),
                transformations: split_list(row.0[9]),
            })
        })
        .collect()
}

fn require_pos(id: &LexemeId, expected: PartOfSpeech) -> Result<&'static RawLexeme> {
    let row = raw_by_id(id).ok_or_else(|| Error::UnknownLemma {
        lookup: id.to_string(),
    })?;
    let actual = parse_pos(row.0[2])?;
    if actual == expected {
        Ok(row)
    } else {
        Err(Error::ContradictoryMetadata {
            reason: format!("lexeme {id} is {actual:?}, not {expected:?}"),
        })
    }
}

fn summary(row: &RawLexeme) -> Result<LexemeSummary> {
    Ok(LexemeSummary {
        id: LexemeId::from(row.0[0]),
        lemma: row.0[1].into(),
        part_of_speech: parse_pos(row.0[2])?,
        source_id: row.0[7].into(),
    })
}

fn parse_pos(value: &str) -> Result<PartOfSpeech> {
    PartOfSpeech::from_code(value).ok_or_else(|| Error::ContradictoryMetadata {
        reason: format!("unknown part of speech code {value:?}"),
    })
}

fn parse_gender(value: &str) -> Result<Gender> {
    match value {
        "masculine" => Ok(Gender::Masculine),
        "feminine" => Ok(Gender::Feminine),
        "neuter" => Ok(Gender::Neuter),
        other => invalid_metadata("gender", other),
    }
}

fn parse_imperfect(value: &str) -> Result<ImperfectFormation> {
    match value {
        "h" => Ok(ImperfectFormation::H),
        "yah" => Ok(ImperfectFormation::Yah),
        "ah" => Ok(ImperfectFormation::Ah),
        "irregular" => Ok(ImperfectFormation::Irregular),
        other => invalid_metadata("imperfect formation", other),
    }
}

fn parse_aorist(value: &str) -> Result<AoristFormation> {
    match value {
        "vowel" => Ok(AoristFormation::VowelStem),
        "consonant" => Ok(AoristFormation::ConsonantStem),
        "irregular" => Ok(AoristFormation::Irregular),
        other => invalid_metadata("aorist formation", other),
    }
}

fn parse_imperative(value: &str) -> Result<ImperativeFormation> {
    match value {
        "first-unpalatalized" => Ok(ImperativeFormation::FirstUnpalatalized),
        "i-series" => Ok(ImperativeFormation::ISeries),
        "irregular" => Ok(ImperativeFormation::Irregular),
        other => invalid_metadata("imperative formation", other),
    }
}

fn nonempty_word(value: &str) -> Result<Option<SynodalWord>> {
    if value.is_empty() {
        Ok(None)
    } else {
        SynodalWord::parse(value).map(Some)
    }
}

fn invalid_metadata<T>(field: &str, value: &str) -> Result<T> {
    Err(Error::ContradictoryMetadata {
        reason: format!("unknown {field} code {value:?}"),
    })
}

fn split_list(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(str::to_owned).collect()
    }
}

#[cfg(test)]
mod binary_search_contract {
    use super::*;

    fn assert_sorted<T>(label: &str, rows: &'static [T], key: impl Fn(&T) -> &'static str) {
        let mut previous = "";
        for row in rows {
            let current = key(row);
            assert!(
                previous <= current,
                "{label} must stay sorted by its first column for `rows_for` to be correct, \
                 but {previous:?} precedes {current:?}"
            );
            previous = current;
        }
    }

    /// `rows_for` binary-searches these tables, so an unsorted one would make
    /// registry lookups silently return the wrong rows rather than merely being
    /// slow. This pins the generator's ordering contract.
    #[test]
    fn generated_lexeme_tables_are_sorted() {
        assert_sorted("LEXEMES", LEXEMES, |row| row.0[0]);
        assert_sorted("EXACT_FORMS", EXACT_FORMS, |row| row.0[0]);
        assert_sorted("PRINCIPAL_PARTS", PRINCIPAL_PARTS, |row| row.0[0]);
        assert_sorted("ACCENTS", ACCENTS, |row| row.0[0]);
        assert_sorted("ACCENT_PARADIGMS", ACCENT_PARADIGMS, |row| row.0[0]);
    }

    /// The searched range must be exactly the rows a full scan would keep, for
    /// every lexeme in the registry and in the same order.
    #[test]
    fn searched_ranges_match_an_exhaustive_scan() {
        for lexeme in LEXEMES {
            let id = lexeme.0[0];
            let searched: Vec<&str> = rows_for(EXACT_FORMS, |row| row.0[0], id)
                .iter()
                .map(|row| row.0[1])
                .collect();
            let scanned: Vec<&str> = EXACT_FORMS
                .iter()
                .filter(|row| row.0[0] == id)
                .map(|row| row.0[1])
                .collect();
            assert_eq!(searched, scanned, "exact forms diverged for {id}");

            let searched: Vec<&str> = rows_for(ACCENT_PARADIGMS, |row| row.0[0], id)
                .iter()
                .map(|row| row.0[2])
                .collect();
            let scanned: Vec<&str> = ACCENT_PARADIGMS
                .iter()
                .filter(|row| row.0[0] == id)
                .map(|row| row.0[2])
                .collect();
            assert_eq!(searched, scanned, "accent paradigms diverged for {id}");
        }
    }

    /// An identifier with no rows, and one ordering before and after every
    /// stored key, must all yield an empty range rather than panicking.
    #[test]
    fn absent_identifiers_yield_an_empty_range() {
        for id in ["", "synodal:noun:absent-lexeme", "\u{10ffff}"] {
            assert!(rows_for(EXACT_FORMS, |row| row.0[0], id).is_empty());
            assert!(rows_for(LEXEMES, |row| row.0[0], id).is_empty());
        }
    }
}

#[cfg(test)]
mod positional_contract {
    use super::*;

    #[test]
    fn every_reviewed_operation_code_parses_to_its_closed_variant() {
        assert_eq!(
            parse_positional_operation("wide-plural-ending").expect("wide plural"),
            PositionalOperation::WidePluralEnding
        );
        assert_eq!(
            parse_positional_operation("decimal-i-before-vowel").expect("decimal i"),
            PositionalOperation::DecimalIBeforeVowel
        );
        assert_eq!(
            parse_positional_operation("initial:wide-e").expect("initial"),
            PositionalOperation::Initial(InitialPresentation::WideE)
        );
        assert_eq!(
            parse_positional_operation("replace:omega@from-end:1").expect("replace"),
            PositionalOperation::Replace {
                replacement: PositionalReplacement::Omega,
                occurrence: LetterOccurrence::FromEnd(1),
            }
        );
        assert_eq!(
            parse_positional_operation("replace:wide-e@from-start:2").expect("replace"),
            PositionalOperation::Replace {
                replacement: PositionalReplacement::WideE,
                occurrence: LetterOccurrence::FromStart(2),
            }
        );
    }

    /// The operation vocabulary is closed precisely so a row cannot silently
    /// rewrite an unrelated character.
    #[test]
    fn unknown_or_malformed_operations_are_rejected() {
        for code in [
            "",
            "widen-everything",
            "initial:tilde",
            "replace:omega",
            "replace:tilde@from-end:1",
            "replace:omega@sideways:1",
            "replace:omega@from-end:x",
        ] {
            assert!(
                parse_positional_operation(code).is_err(),
                "{code:?} must not parse"
            );
        }
    }

    /// A lexeme with no reviewed positional contract must resolve to `None`
    /// rather than erroring, so the registry path stays a no-op until reviewed
    /// rows exist.
    #[test]
    fn absent_positional_contracts_are_not_an_error() {
        let id = LexemeId::from("synodal:noun:grad");
        let cell = GrammarCell::Noun(synodal_church_slavonic_core::NounCell {
            case: Case::Dative,
            number: Number::Plural,
            animacy: Animacy::Inanimate,
        });
        assert!(
            positional_paradigm_for(&id, cell)
                .expect("absent contract is not an error")
                .is_none()
        );
    }
}
