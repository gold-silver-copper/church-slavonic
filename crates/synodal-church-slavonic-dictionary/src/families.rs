#[allow(unused_imports)]
use super::*;

/// Stable identifier for a reviewed morphological family.
///
/// Reviewed families currently have a one-to-one relationship with a stable
/// Synodal lexeme identity.  Candidate family IDs produced by `xtask` are
/// deliberately not accepted by the runtime dictionary.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FamilyId(String);

impl FamilyId {
    #[must_use]
    pub fn for_lexeme(id: &LexemeId) -> Self {
        Self(format!("family:{}", id.as_str()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn lexeme_id(&self) -> Option<LexemeId> {
        self.0.strip_prefix("family:").map(LexemeId::from)
    }
}

impl std::fmt::Display for FamilyId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&str> for FamilyId {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FamilyMember {
    /// Stable grammar-cell key. This remains a string so the review registry
    /// can expose exact cells not yet supported by productive generation.
    pub cell: String,
    pub expanded: String,
    pub printed: String,
    pub evidence_id: String,
    pub source_kind: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FamilySummary {
    pub id: FamilyId,
    pub lexeme: LexemeSummary,
    pub senses: Vec<Sense>,
    pub members: Vec<FamilyMember>,
    pub class: Option<String>,
    pub stem: Option<String>,
    pub principal_parts: BTreeMap<String, String>,
    pub supported_systems: Vec<String>,
    pub missing_metadata: Vec<MetadataField>,
    /// Family-level requirements not represented by the low-level
    /// `MetadataField` enum (for example a nominal declension class).
    pub missing_family_metadata: Vec<String>,
    pub exact_only: bool,
    pub fully_classed: bool,
}

/// Finds reviewed morphological families by lemma or gloss. Ambiguous
/// homographs are returned independently and never collapsed to one identity.
pub fn families(query: &str) -> Result<Vec<FamilySummary>> {
    let options = SearchOptions {
        limit: usize::MAX,
        fuzzy: false,
        ..SearchOptions::default()
    };
    let mut ids: BTreeSet<LexemeId> = search(query, &options)?
        .into_iter()
        .map(|matched| matched.entry.lexeme.id().clone())
        .collect();
    if let Ok(exact) = lookup_all(query) {
        ids.extend(exact.into_iter().map(|entry| entry.lexeme.id().clone()));
    }
    if let Ok(analyses) = analyze(query) {
        ids.extend(
            analyses
                .into_iter()
                .map(|analysis| analysis.lexeme.id().clone()),
        );
    }
    ids.iter().map(family_for_lexeme).collect()
}

/// Returns one reviewed family by stable ID. Proposed family IDs remain part
/// of the review tooling and fail explicitly here.
pub fn show_family_by_id(id: &FamilyId) -> Result<FamilySummary> {
    let lexeme_id = id.lexeme_id().ok_or_else(|| Error::UnknownLemma {
        lookup: id.as_str().into(),
    })?;
    family_for_lexeme(&lexeme_id)
}

pub(crate) fn family_for_lexeme(id: &LexemeId) -> Result<FamilySummary> {
    let entry = lookup_by_id(id)?;
    let metadata = &entry.metadata;
    let mut members: Vec<_> = metadata
        .exact_forms
        .iter()
        .map(|form| FamilyMember {
            cell: form.cell.clone(),
            expanded: form.expanded.clone(),
            printed: form.printed.clone(),
            evidence_id: form.evidence_id.clone(),
            source_kind: form.source_kind.clone(),
        })
        .collect();
    for accent in &metadata.accents {
        let member = FamilyMember {
            cell: accent.cell.clone(),
            expanded: accent.expanded.clone(),
            printed: accent.accented.clone(),
            evidence_id: accent.evidence_id.clone(),
            source_kind: "accent-table".into(),
        };
        if !members.contains(&member) {
            members.push(member);
        }
    }
    for sense in &entry.senses {
        for contraction in abbreviation::contractions_by_id(id, &sense.id)? {
            members.push(FamilyMember {
                cell: contraction.cell_key,
                expanded: contraction.expanded,
                printed: contraction.printed,
                evidence_id: contraction
                    .evidence_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                source_kind: "abbreviation".into(),
            });
        }
    }
    members.sort_by(|left, right| {
        left.cell
            .cmp(&right.cell)
            .then_with(|| left.printed.cmp(&right.printed))
            .then_with(|| left.evidence_id.cmp(&right.evidence_id))
    });
    let capabilities = &entry.capabilities;
    let supported_systems = capabilities
        .supported_systems()
        .map(str::to_owned)
        .collect();
    let exact_complete_table = metadata.class.as_deref() == Some("exact-complete-pronoun-table");
    let exact_only = metadata
        .class
        .as_deref()
        .is_none_or(|class| matches!(class, "exact" | "exact-complete-pronoun-table"))
        && metadata.principal_parts.is_empty();
    let fully_classed = exact_complete_table
        || (!exact_only
            && match entry.lexeme.part_of_speech() {
                PartOfSpeech::Noun | PartOfSpeech::ProperNoun => {
                    metadata.class.is_some() && metadata.stem.is_some() && metadata.gender.is_some()
                }
                PartOfSpeech::Adjective | PartOfSpeech::Determiner => {
                    metadata.class.is_some() && metadata.stem.is_some()
                }
                PartOfSpeech::Verb => {
                    metadata.class.is_some() && metadata.stem.is_some() && metadata.aspect.is_some()
                }
                _ => true,
            });
    let mut missing_family_metadata = BTreeSet::new();
    if exact_only
        && !exact_complete_table
        && matches!(
            entry.lexeme.part_of_speech(),
            PartOfSpeech::Noun
                | PartOfSpeech::ProperNoun
                | PartOfSpeech::Adjective
                | PartOfSpeech::Determiner
                | PartOfSpeech::Verb
                | PartOfSpeech::Pronoun
                | PartOfSpeech::Numeral
        )
    {
        missing_family_metadata.insert("reviewed-inflection-class-or-exact-complete-table".into());
    }
    if metadata.stem.is_none()
        && matches!(
            entry.lexeme.part_of_speech(),
            PartOfSpeech::Noun
                | PartOfSpeech::ProperNoun
                | PartOfSpeech::Adjective
                | PartOfSpeech::Determiner
                | PartOfSpeech::Verb
        )
    {
        missing_family_metadata.insert("reviewed-stem-and-alternants".into());
    }
    if entry.lexeme.part_of_speech() == PartOfSpeech::Verb && metadata.principal_parts.is_empty() {
        missing_family_metadata.insert("independent-verb-principal-parts".into());
    }
    Ok(FamilySummary {
        id: FamilyId::for_lexeme(id),
        lexeme: entry.lexeme,
        senses: entry.senses,
        members,
        class: metadata.class.clone(),
        stem: metadata.stem.clone(),
        principal_parts: metadata
            .principal_parts
            .iter()
            .map(|part| (part.system.clone(), part.value.clone()))
            .collect(),
        supported_systems,
        missing_metadata: entry.missing_metadata,
        missing_family_metadata: missing_family_metadata.into_iter().collect(),
        exact_only,
        fully_classed,
    })
}
