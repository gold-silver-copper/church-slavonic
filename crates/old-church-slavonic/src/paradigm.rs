//! Full paradigms assembled through the canonical one-cell resolvers.

use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, FiniteTense, FiniteVerbCell, FormSet, Gender,
    GenderedCell, ImperativeCell, InflectionError, LParticipleCell, NounCell, Number, PartOfSpeech,
    ParticipleCell, ParticipleKind, Person, PersonalPronounCell, UngenderedCell,
};
use std::fmt;

/// One requested cell and its typed outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellOutcome<C> {
    pub cell: C,
    pub result: Result<FormSet, InflectionError>,
}

impl<C> CellOutcome<C> {
    /// The grammatical cell requested by this row.
    pub fn cell(&self) -> &C {
        &self.cell
    }

    /// The successful forms or the typed failure retained for this cell.
    pub fn forms(&self) -> Result<&FormSet, &InflectionError> {
        self.result.as_ref()
    }

    /// The retained error, if generation failed.
    pub fn error(&self) -> Option<&InflectionError> {
        self.result.as_ref().err()
    }

    /// Consume the row into its cell and outcome.
    pub fn into_parts(self) -> (C, Result<FormSet, InflectionError>) {
        (self.cell, self.result)
    }
}

/// Why a typed paradigm could not return successful forms for a requested cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParadigmLookupError {
    /// This specialized paradigm does not enumerate the requested cell.
    NotRepresented,
    /// The cell is represented and retains a typed inflection failure.
    Failed(InflectionError),
}

impl fmt::Display for ParadigmLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRepresented => {
                f.write_str("the specialized paradigm does not represent this cell")
            }
            Self::Failed(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ParadigmLookupError {}

fn forms_for<'a, C: PartialEq>(
    cells: &'a [CellOutcome<C>],
    requested: &C,
) -> Result<&'a FormSet, ParadigmLookupError> {
    let outcome = cells
        .iter()
        .find(|candidate| candidate.cell == *requested)
        .ok_or(ParadigmLookupError::NotRepresented)?;
    outcome
        .forms()
        .map_err(|error| ParadigmLookupError::Failed(error.clone()))
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

            /// Iterate over successful cells without discarding their grammar.
            pub fn successes(&self) -> impl Iterator<Item = (&$cell, &FormSet)> {
                self.cells.iter().filter_map(|outcome| {
                    outcome
                        .result
                        .as_ref()
                        .ok()
                        .map(|forms| (&outcome.cell, forms))
                })
            }

            /// Iterate over retained typed failures.
            pub fn failures(&self) -> impl Iterator<Item = (&$cell, &InflectionError)> {
                self.cells.iter().filter_map(|outcome| {
                    outcome
                        .result
                        .as_ref()
                        .err()
                        .map(|error| (&outcome.cell, error))
                })
            }

            /// Consume the paradigm into its ordered cell rows.
            pub fn into_rows(self) -> Vec<CellOutcome<$cell>> {
                self.cells
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
    /// Return one noun form or distinguish an absent row from a failed row.
    pub fn form(&self, case: Case, number: Number) -> Result<&FormSet, ParadigmLookupError> {
        let cell = NounCell { case, number };
        forms_for(&self.cells, &cell)
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
    /// Return one adjective form or distinguish an absent row from a failed row.
    pub fn form(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<&FormSet, ParadigmLookupError> {
        let cell = AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
        };
        forms_for(&self.cells, &cell)
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
    /// Return one finite form or distinguish an absent row from a failed row.
    pub fn form(
        &self,
        tense: FiniteTense,
        person: Person,
        number: Number,
    ) -> Result<&FormSet, ParadigmLookupError> {
        let cell = FiniteVerbCell {
            tense,
            person,
            number,
        };
        forms_for(&self.cells, &cell)
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
    /// Return one present form or distinguish an absent row from a failed row.
    pub fn form(&self, person: Person, number: Number) -> Result<&FormSet, ParadigmLookupError> {
        let cell = FiniteVerbCell {
            tense: FiniteTense::Present,
            person,
            number,
        };
        forms_for(&self.cells, &cell)
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
    /// Return one imperative or distinguish an absent row from a failed row.
    pub fn form(&self, person: Person, number: Number) -> Result<&FormSet, ParadigmLookupError> {
        let cell = ImperativeCell { person, number };
        forms_for(&self.cells, &cell)
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
    /// Return one l-participle or distinguish an absent row from a failed row.
    pub fn form(&self, gender: Gender, number: Number) -> Result<&FormSet, ParadigmLookupError> {
        let cell = LParticipleCell { gender, number };
        forms_for(&self.cells, &cell)
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
    pub fn form(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<&FormSet, ParadigmLookupError> {
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
        forms_for(&self.cells, &cell)
    }
}

paradigm_common!(ParticipleParadigm, ParticipleCell);

/// A source-backed closed-class paradigm with one explicit cell shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedClassParadigm<C> {
    pub(crate) lexeme_id: String,
    pub(crate) lemma: String,
    pub(crate) part_of_speech: PartOfSpeech,
    pub(crate) cells: Vec<CellOutcome<C>>,
}

impl<C> ClosedClassParadigm<C> {
    /// The canonical dictionary lemma.
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// The stable dictionary lexeme ID.
    pub fn id(&self) -> &str {
        &self.lexeme_id
    }

    /// The closed class represented by this table.
    pub fn part_of_speech(&self) -> PartOfSpeech {
        self.part_of_speech
    }

    /// Iterate over represented rows in stable grammatical order.
    pub fn iter(&self) -> std::slice::Iter<'_, CellOutcome<C>> {
        self.cells.iter()
    }

    /// Iterate over successful cells without discarding their grammar.
    pub fn successes(&self) -> impl Iterator<Item = (&C, &FormSet)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|forms| (&outcome.cell, forms))
        })
    }

    /// Iterate over represented cells that retained a typed failure.
    pub fn failures(&self) -> impl Iterator<Item = (&C, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    /// Number of represented rows, including typed failures.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Whether this typed inventory has no represented rows.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Consume the paradigm into its ordered cell rows.
    pub fn into_rows(self) -> Vec<CellOutcome<C>> {
        self.cells
    }
}

impl ClosedClassParadigm<UngenderedCell> {
    /// Return one case-number form or distinguish an absent row from a failed row.
    pub fn form(&self, case: Case, number: Number) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(&self.cells, &UngenderedCell { case, number })
    }
}

impl ClosedClassParadigm<GenderedCell> {
    /// Return one agreeing form or distinguish an absent row from a failed row.
    pub fn form(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(
            &self.cells,
            &GenderedCell {
                case,
                number,
                gender,
            },
        )
    }
}

impl ClosedClassParadigm<PersonalPronounCell> {
    /// Return one personal-pronoun form or distinguish absence from failure.
    pub fn form(
        &self,
        case: Case,
        number: Number,
        person: Person,
    ) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(
            &self.cells,
            &PersonalPronounCell {
                case,
                number,
                person,
            },
        )
    }
}

impl<'a, C> IntoIterator for &'a ClosedClassParadigm<C> {
    type Item = &'a CellOutcome<C>;
    type IntoIter = std::slice::Iter<'a, CellOutcome<C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl<C> IntoIterator for ClosedClassParadigm<C> {
    type Item = CellOutcome<C>;
    type IntoIter = std::vec::IntoIter<CellOutcome<C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

pub type DeterminerParadigm = ClosedClassParadigm<GenderedCell>;
pub type PronounParadigm = ClosedClassParadigm<UngenderedCell>;
pub type PersonalPronounParadigm = ClosedClassParadigm<PersonalPronounCell>;
pub type GenderedPronounParadigm = ClosedClassParadigm<GenderedCell>;
pub type NumeralParadigm = ClosedClassParadigm<UngenderedCell>;
pub type GenderedNumeralParadigm = ClosedClassParadigm<GenderedCell>;

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
