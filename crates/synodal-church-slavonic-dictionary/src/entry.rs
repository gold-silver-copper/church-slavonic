#[allow(unused_imports)]
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Sense {
    pub id: String,
    pub gloss: String,
    pub domains: Vec<String>,
    pub source_id: String,
    pub source_recension: String,
    pub semantic_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceExample {
    pub id: String,
    pub lexeme_id: LexemeId,
    pub text: String,
    pub translation: String,
    pub source_id: String,
    pub passage: String,
    pub source_recension: String,
    pub target_recension: String,
    pub partition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub lexeme: LexemeSummary,
    pub senses: Vec<Sense>,
    pub examples: Vec<SourceExample>,
    pub metadata: LexicalMetadataSummary,
    pub capabilities: synodal_church_slavonic::Capabilities,
    pub missing_metadata: Vec<MetadataField>,
}

pub(crate) fn entry_for(lexeme: LexemeSummary) -> Result<Entry> {
    let senses = senses_for(lexeme.id());
    if senses.is_empty() {
        return Err(Error::ContradictoryMetadata {
            reason: format!("lexeme {} has no reviewed semantic sense", lexeme.id()),
        });
    }
    let examples = concordance(lexeme.id());
    Ok(Entry {
        metadata: lexical_metadata(lexeme.id())?,
        capabilities: capabilities_by_id(lexeme.id(), Inflector::default())?,
        missing_metadata: missing_metadata_by_id(lexeme.id())?,
        lexeme,
        senses,
        examples,
    })
}

pub(crate) fn senses_for(id: &LexemeId) -> Vec<Sense> {
    SENSES
        .iter()
        .filter(|sense| sense.0[0] == id.as_str())
        .map(|sense| Sense {
            id: sense.0[1].into(),
            gloss: sense.0[2].into(),
            domains: split_list(sense.0[3]),
            source_id: sense.0[4].into(),
            source_recension: sense.0[5].into(),
            semantic_status: sense.0[6].into(),
        })
        .collect()
}

pub(crate) fn source_example(row: &RawExample) -> SourceExample {
    SourceExample {
        id: row.0[0].into(),
        lexeme_id: LexemeId::from(row.0[1]),
        text: row.0[2].into(),
        translation: row.0[3].into(),
        source_id: row.0[4].into(),
        passage: row.0[5].into(),
        source_recension: row.0[6].into(),
        target_recension: row.0[7].into(),
        partition: row.0[8].into(),
    }
}

pub(crate) fn analysis_source(source: &FormSource) -> AnalysisSource {
    match source {
        FormSource::SynodalAttestation { .. } => AnalysisSource::ExactSynodalAttestation,
        FormSource::SynodalIrregularOverride { .. } => AnalysisSource::SynodalIrregularOverride,
        FormSource::SynodalNormativeGeneration { rule }
            if rule.as_str() == "SYN-REGISTRY-NORMATIVE-TABLE" =>
        {
            AnalysisSource::SynodalNormativeTable
        }
        FormSource::SynodalNormativeGeneration { .. } => AnalysisSource::SynodalProductiveRule,
        FormSource::CallerSpecifiedPrediction { .. } => AnalysisSource::CallerSpecifiedPrediction,
        FormSource::InheritedPrediction { .. } => AnalysisSource::InheritedPrediction,
        FormSource::AnalogicalPrediction { .. } => AnalysisSource::AnalogicalPrediction,
    }
}
