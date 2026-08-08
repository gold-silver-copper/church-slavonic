//! Full paradigms assembled through the canonical one-cell resolvers.

use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, FiniteTense, FiniteVerbCell, FormSet, Gender,
    ImperativeCell, InflectionError, LParticipleCell, NounCell, Number, PartOfSpeech,
    ParticipleCell, ParticipleKind, Person,
};

/// One requested cell and its typed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellOutcome<C> {
    pub cell: C,
    pub result: Result<FormSet, InflectionError>,
}

macro_rules! paradigm_common {
    ($name:ident, $cell:ty) => {
        impl $name {
            /// The canonical dictionary lemma.
            pub fn lemma(&self) -> &str {
                &self.lemma
            }

            /// The stable dictionary lexeme ID.
            pub fn id(&self) -> &str {
                &self.lexeme_id
            }

            /// Iterate in the documented paradigm order.
            pub fn iter(&self) -> std::slice::Iter<'_, CellOutcome<$cell>> {
                self.cells.iter()
            }

            /// Number of represented cells, including typed failures.
            pub fn len(&self) -> usize {
                self.cells.len()
            }

            /// Paradigms always represent at least one grammatical cell.
            pub fn is_empty(&self) -> bool {
                self.cells.is_empty()
            }
        }

        impl<'a> IntoIterator for &'a $name {
            type Item = &'a CellOutcome<$cell>;
            type IntoIter = std::slice::Iter<'a, CellOutcome<$cell>>;

            fn into_iter(self) -> Self::IntoIter {
                self.cells.iter()
            }
        }

        impl IntoIterator for $name {
            type Item = CellOutcome<$cell>;
            type IntoIter = std::vec::IntoIter<CellOutcome<$cell>>;

            fn into_iter(self) -> Self::IntoIter {
                self.cells.into_iter()
            }
        }
    };
}

/// Seven cases by three numbers, ordered number then case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NounParadigm {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<NounCell>>,
}

impl NounParadigm {
    /// Get one noun outcome by its direct grammatical dimensions.
    pub fn get(&self, case: Case, number: Number) -> Option<&Result<FormSet, InflectionError>> {
        let cell = NounCell { case, number };
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

paradigm_common!(NounParadigm, NounCell);

/// Long and short adjective agreement cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjectiveParadigm {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<AdjectiveCell>>,
}

impl AdjectiveParadigm {
    /// Get one adjective outcome by its direct grammatical dimensions.
    pub fn get(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Option<&Result<FormSet, InflectionError>> {
        let cell = AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
        };
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

paradigm_common!(AdjectiveParadigm, AdjectiveCell);

/// Present, imperfect, and aorist person-number cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiniteVerbParadigm {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<FiniteVerbCell>>,
}

impl FiniteVerbParadigm {
    /// Get one finite outcome by tense, person, and number.
    pub fn get(
        &self,
        tense: FiniteTense,
        person: Person,
        number: Number,
    ) -> Option<&Result<FormSet, InflectionError>> {
        let cell = FiniteVerbCell {
            tense,
            person,
            number,
        };
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

paradigm_common!(FiniteVerbParadigm, FiniteVerbCell);

/// The nine present-indicative person-number cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerbParadigm {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<FiniteVerbCell>>,
}

impl VerbParadigm {
    /// Get one present outcome by person and number.
    pub fn get(&self, person: Person, number: Number) -> Option<&Result<FormSet, InflectionError>> {
        let cell = FiniteVerbCell {
            tense: FiniteTense::Present,
            person,
            number,
        };
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

paradigm_common!(VerbParadigm, FiniteVerbCell);

/// The six historically represented imperative cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImperativeParadigm {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<ImperativeCell>>,
}

impl ImperativeParadigm {
    /// Get one imperative outcome by person and number.
    pub fn get(&self, person: Person, number: Number) -> Option<&Result<FormSet, InflectionError>> {
        let cell = ImperativeCell { person, number };
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

paradigm_common!(ImperativeParadigm, ImperativeCell);

/// Gender-number agreement for the l-participle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LParticipleParadigm {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<LParticipleCell>>,
}

impl LParticipleParadigm {
    /// Get one l-participle outcome by gender and number.
    pub fn get(&self, gender: Gender, number: Number) -> Option<&Result<FormSet, InflectionError>> {
        let cell = LParticipleCell { gender, number };
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

paradigm_common!(LParticipleParadigm, LParticipleCell);

/// Full adjective agreement for one verbal participle kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipleParadigm {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) kind: ParticipleKind,
    pub(crate) cells: Vec<CellOutcome<ParticipleCell>>,
}

impl ParticipleParadigm {
    /// The verbal participle system represented by this paradigm.
    pub fn kind(&self) -> ParticipleKind {
        self.kind
    }

    /// Get one declined participle outcome.
    pub fn get(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Option<&Result<FormSet, InflectionError>> {
        let cell = ParticipleCell {
            kind: self.kind,
            adjective: AdjectiveCell {
                case,
                number,
                gender,
                animacy,
                form,
            },
        };
        self.cells
            .iter()
            .find_map(|candidate| (candidate.cell == cell).then_some(&candidate.result))
    }
}

paradigm_common!(ParticipleParadigm, ParticipleCell);

/// Every table-backed raw feature for a dictionary lexeme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryParadigm {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) part_of_speech: PartOfSpeech,
    pub(crate) cells: Vec<(String, FormSet)>,
}

impl DictionaryParadigm {
    /// The canonical dictionary lemma.
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// The stable dictionary lexeme ID.
    pub fn id(&self) -> &str {
        &self.lexeme_id
    }

    /// The dictionary part of speech.
    pub fn part_of_speech(&self) -> PartOfSpeech {
        self.part_of_speech
    }

    /// Find a raw normalized feature.
    pub fn get(&self, feature_key: &str) -> Option<&FormSet> {
        self.cells
            .iter()
            .find_map(|(feature, forms)| (feature == feature_key).then_some(forms))
    }

    /// Iterate over raw normalized features in registry order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&str, &FormSet)> {
        self.cells
            .iter()
            .map(|(feature, forms)| (feature.as_str(), forms))
    }

    /// Number of table-backed raw features.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether this raw table paradigm contains no feature rows.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}
