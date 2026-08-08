//! Full-paradigm structures assembled through the public cell resolvers.

use old_church_slavonic_core::{
    AdjectiveCell, FiniteVerbCell, FormSet, ImperativeCell, InflectionError, LParticipleCell,
    NounCell, PartOfSpeech,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellOutcome<C> {
    pub cell: C,
    pub result: Result<FormSet, InflectionError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounParadigm {
    pub lexeme_id: String,
    pub cells: Vec<CellOutcome<NounCell>>,
}

impl NounParadigm {
    pub fn get(&self, cell: NounCell) -> Option<&Result<FormSet, InflectionError>> {
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjectiveParadigm {
    pub lexeme_id: String,
    pub cells: Vec<CellOutcome<AdjectiveCell>>,
}

impl AdjectiveParadigm {
    pub fn get(&self, cell: AdjectiveCell) -> Option<&Result<FormSet, InflectionError>> {
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiniteVerbParadigm {
    pub lexeme_id: String,
    pub cells: Vec<CellOutcome<FiniteVerbCell>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImperativeParadigm {
    pub lexeme_id: String,
    pub cells: Vec<CellOutcome<ImperativeCell>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LParticipleParadigm {
    pub lexeme_id: String,
    pub cells: Vec<CellOutcome<LParticipleCell>>,
}

/// Every table-backed cell for a dictionary lexeme, including non-finite forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryParadigm {
    pub lexeme_id: String,
    pub part_of_speech: PartOfSpeech,
    pub cells: Vec<(String, FormSet)>,
}

impl DictionaryParadigm {
    pub fn get(&self, feature_key: &str) -> Option<&FormSet> {
        self.cells
            .iter()
            .find_map(|(feature, forms)| (feature == feature_key).then_some(forms))
    }
}
