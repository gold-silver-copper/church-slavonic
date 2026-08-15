//! Full paradigms assembled through the canonical one-cell resolvers.

use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Animacy, CardinalCompositionOptions, CardinalMagnitudeIdentity,
    CardinalNumeralIdentity, Case, CollectiveNumeralCell, CollectiveNumeralIdentity,
    CompoundCardinalCell, DeterminerCell, DeterminerIdentity, FiniteTense, FiniteVerbCell, FormSet,
    FractionalNumeralIdentity, Gender, GenderedCell, ImperativeCell, InflectionError,
    LParticipleCell, NounCell, Number, NumeralCell, OrdinalNumeralIdentity, PartOfSpeech,
    ParticipleCell, ParticipleKind, Person, PersonalPronounCell, RealizedCardinal, RealizedOrdinal,
    UngenderedCell,
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
    ($name:ident $(<$generic:ident>)?, $cell:ty) => {
        impl$(<$generic>)? $name$(<$generic>)? {
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

        impl<'a $(, $generic)?> IntoIterator for &'a $name$(<$generic>)? {
            type Item = &'a CellOutcome<$cell>;
            type IntoIter = std::slice::Iter<'a, CellOutcome<$cell>>;

            fn into_iter(self) -> Self::IntoIter {
                self.cells.iter()
            }
        }

        impl$(<$generic>)? IntoIterator for $name$(<$generic>)? {
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

/// Complete source-reviewed determiner paradigm, including the animacy
/// dimension needed by adjectival members of the lexical inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterminerParadigm {
    pub(crate) identity: DeterminerIdentity,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<DeterminerCell>>,
}

impl DeterminerParadigm {
    /// The stable grammatical identity represented by this paradigm.
    pub const fn identity(&self) -> DeterminerIdentity {
        self.identity
    }

    /// The canonical grammar lemma.
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// Return one determiner form or distinguish an absent row from a failed row.
    pub fn form(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(
            &self.cells,
            &DeterminerCell {
                case,
                number,
                gender,
                animacy,
            },
        )
    }

    /// Iterate in number-case-gender-animacy order.
    pub fn iter(&self) -> std::slice::Iter<'_, CellOutcome<DeterminerCell>> {
        self.cells.iter()
    }

    /// Iterate over successful cells without discarding their grammar.
    pub fn successes(&self) -> impl Iterator<Item = (&DeterminerCell, &FormSet)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|forms| (&outcome.cell, forms))
        })
    }

    /// Iterate over retained typed failures.
    pub fn failures(&self) -> impl Iterator<Item = (&DeterminerCell, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    /// Consume the paradigm into its ordered cell rows.
    pub fn into_rows(self) -> Vec<CellOutcome<DeterminerCell>> {
        self.cells
    }

    /// Number of represented cells, including typed failures.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Determiner paradigms always represent the complete typed inventory.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<'a> IntoIterator for &'a DeterminerParadigm {
    type Item = &'a CellOutcome<DeterminerCell>;
    type IntoIter = std::slice::Iter<'a, CellOutcome<DeterminerCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl IntoIterator for DeterminerParadigm {
    type Item = CellOutcome<DeterminerCell>;
    type IntoIter = std::vec::IntoIter<CellOutcome<DeterminerCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

/// Complete typed inventory for one reviewed simple cardinal.
///
/// The paradigm includes every optional-gender cell so historically impossible
/// shapes remain visible as typed failures instead of silently disappearing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardinalNumeralParadigm {
    pub(crate) identity: CardinalNumeralIdentity,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<NumeralCell>>,
}

impl CardinalNumeralParadigm {
    /// The stable grammatical identity represented by this paradigm.
    pub const fn identity(&self) -> CardinalNumeralIdentity {
        self.identity
    }

    /// The canonical grammar lemma.
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// Return one cardinal form or distinguish an absent row from a failed row.
    pub fn form(
        &self,
        case: Case,
        number: Number,
        gender: Option<Gender>,
    ) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(
            &self.cells,
            &NumeralCell {
                case,
                number,
                gender,
            },
        )
    }

    /// Iterate in number-case-optional-gender order.
    pub fn iter(&self) -> std::slice::Iter<'_, CellOutcome<NumeralCell>> {
        self.cells.iter()
    }

    /// Iterate over licensed cells without discarding their grammar.
    pub fn successes(&self) -> impl Iterator<Item = (&NumeralCell, &FormSet)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|forms| (&outcome.cell, forms))
        })
    }

    /// Iterate over historically impossible cells retained as typed failures.
    pub fn failures(&self) -> impl Iterator<Item = (&NumeralCell, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    /// Consume the paradigm into its ordered cell rows.
    pub fn into_rows(self) -> Vec<CellOutcome<NumeralCell>> {
        self.cells
    }

    /// Number of represented cells, including typed failures.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Cardinal paradigms always represent the complete typed inventory.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<'a> IntoIterator for &'a CardinalNumeralParadigm {
    type Item = &'a CellOutcome<NumeralCell>;
    type IntoIter = std::slice::Iter<'a, CellOutcome<NumeralCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl IntoIterator for CardinalNumeralParadigm {
    type Item = CellOutcome<NumeralCell>;
    type IntoIter = std::vec::IntoIter<CellOutcome<NumeralCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

/// Complete short-and-long agreement inventory for one simple ordinal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinalNumeralParadigm {
    pub(crate) identity: OrdinalNumeralIdentity,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<AdjectiveCell>>,
}

impl OrdinalNumeralParadigm {
    /// The stable grammatical identity represented by this paradigm.
    pub const fn identity(&self) -> OrdinalNumeralIdentity {
        self.identity
    }

    /// The canonical grammar lemma.
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// Return one ordinal-adjective form.
    pub fn form(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(
            &self.cells,
            &AdjectiveCell {
                case,
                number,
                gender,
                animacy,
                form,
            },
        )
    }

    /// Iterate in form-number-case-gender-animacy order.
    pub fn iter(&self) -> std::slice::Iter<'_, CellOutcome<AdjectiveCell>> {
        self.cells.iter()
    }

    /// Iterate over successful cells without discarding their grammar.
    pub fn successes(&self) -> impl Iterator<Item = (&AdjectiveCell, &FormSet)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|forms| (&outcome.cell, forms))
        })
    }

    /// Iterate over retained typed failures.
    pub fn failures(&self) -> impl Iterator<Item = (&AdjectiveCell, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    /// Consume the paradigm into its ordered cell rows.
    pub fn into_rows(self) -> Vec<CellOutcome<AdjectiveCell>> {
        self.cells
    }

    /// Number of represented cells, including typed failures.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Ordinal paradigms always represent the complete typed inventory.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<'a> IntoIterator for &'a OrdinalNumeralParadigm {
    type Item = &'a CellOutcome<AdjectiveCell>;
    type IntoIter = std::slice::Iter<'a, CellOutcome<AdjectiveCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl IntoIterator for OrdinalNumeralParadigm {
    type Item = CellOutcome<AdjectiveCell>;
    type IntoIter = std::vec::IntoIter<CellOutcome<AdjectiveCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

/// Complete licensed cell inventory for one collective numeral.
///
/// Pronominal collectives contain 63 case-number-gender rows. Adjectival
/// collectives contain all 252 short/long agreement rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectiveNumeralParadigm {
    pub(crate) identity: CollectiveNumeralIdentity,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<CollectiveNumeralCell>>,
}

impl CollectiveNumeralParadigm {
    pub const fn identity(&self) -> CollectiveNumeralIdentity {
        self.identity
    }

    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    pub fn form(&self, cell: CollectiveNumeralCell) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(&self.cells, &cell)
    }

    pub fn pronominal_form(
        &self,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<&FormSet, ParadigmLookupError> {
        self.form(CollectiveNumeralCell::pronominal(case, number, gender))
    }

    pub fn adjectival_form(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<&FormSet, ParadigmLookupError> {
        self.form(CollectiveNumeralCell::adjectival(
            form, case, number, gender, animacy,
        ))
    }

    pub fn iter(&self) -> std::slice::Iter<'_, CellOutcome<CollectiveNumeralCell>> {
        self.cells.iter()
    }

    pub fn successes(&self) -> impl Iterator<Item = (&CollectiveNumeralCell, &FormSet)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|forms| (&outcome.cell, forms))
        })
    }

    pub fn failures(&self) -> impl Iterator<Item = (&CollectiveNumeralCell, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    pub fn into_rows(self) -> Vec<CellOutcome<CollectiveNumeralCell>> {
        self.cells
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<'a> IntoIterator for &'a CollectiveNumeralParadigm {
    type Item = &'a CellOutcome<CollectiveNumeralCell>;
    type IntoIter = std::slice::Iter<'a, CellOutcome<CollectiveNumeralCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl IntoIterator for CollectiveNumeralParadigm {
    type Item = CellOutcome<CollectiveNumeralCell>;
    type IntoIter = std::vec::IntoIter<CellOutcome<CollectiveNumeralCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

/// Complete 21-cell noun inventory for one OCS fractional numeral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FractionalNumeralParadigm {
    pub(crate) identity: FractionalNumeralIdentity,
    pub(crate) lemma: String,
    pub(crate) cells: Vec<CellOutcome<NounCell>>,
}

impl FractionalNumeralParadigm {
    pub const fn identity(&self) -> FractionalNumeralIdentity {
        self.identity
    }

    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    pub fn form(&self, case: Case, number: Number) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(&self.cells, &NounCell { case, number })
    }

    pub fn iter(&self) -> std::slice::Iter<'_, CellOutcome<NounCell>> {
        self.cells.iter()
    }

    pub fn successes(&self) -> impl Iterator<Item = (&NounCell, &FormSet)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|forms| (&outcome.cell, forms))
        })
    }

    pub fn failures(&self) -> impl Iterator<Item = (&NounCell, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    pub fn into_rows(self) -> Vec<CellOutcome<NounCell>> {
        self.cells
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<'a> IntoIterator for &'a FractionalNumeralParadigm {
    type Item = &'a CellOutcome<NounCell>;
    type IntoIter = std::slice::Iter<'a, CellOutcome<NounCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl IntoIterator for FractionalNumeralParadigm {
    type Item = CellOutcome<NounCell>;
    type IntoIter = std::vec::IntoIter<CellOutcome<NounCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

/// One composed-cardinal request and its typed structural outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundCardinalOutcome {
    pub cell: CompoundCardinalCell,
    pub result: Result<RealizedCardinal, InflectionError>,
}

impl CompoundCardinalOutcome {
    pub const fn cell(&self) -> &CompoundCardinalCell {
        &self.cell
    }

    pub fn cardinal(&self) -> Result<&RealizedCardinal, &InflectionError> {
        self.result.as_ref()
    }

    pub fn error(&self) -> Option<&InflectionError> {
        self.result.as_ref().err()
    }

    pub fn into_parts(
        self,
    ) -> (
        CompoundCardinalCell,
        Result<RealizedCardinal, InflectionError>,
    ) {
        (self.cell, self.result)
    }
}

/// Complete optional-gender case inventory for one composed cardinal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundCardinalParadigm {
    pub(crate) value: u16,
    pub(crate) options: CardinalCompositionOptions,
    pub(crate) cells: Vec<CompoundCardinalOutcome>,
}

impl CompoundCardinalParadigm {
    pub const fn value(&self) -> u16 {
        self.value
    }

    pub const fn one_identity(&self) -> CardinalNumeralIdentity {
        self.options.one_identity
    }

    pub const fn thousand_identity(&self) -> CardinalMagnitudeIdentity {
        self.options.thousand_identity
    }

    pub const fn options(&self) -> CardinalCompositionOptions {
        self.options
    }

    pub fn form(
        &self,
        case: Case,
        gender: Option<Gender>,
    ) -> Result<&RealizedCardinal, ParadigmLookupError> {
        let outcome = self
            .cells
            .iter()
            .find(|outcome| outcome.cell == CompoundCardinalCell { case, gender })
            .ok_or(ParadigmLookupError::NotRepresented)?;
        outcome
            .cardinal()
            .map_err(|error| ParadigmLookupError::Failed(error.clone()))
    }

    pub fn iter(&self) -> std::slice::Iter<'_, CompoundCardinalOutcome> {
        self.cells.iter()
    }

    pub fn successes(&self) -> impl Iterator<Item = (&CompoundCardinalCell, &RealizedCardinal)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|cardinal| (&outcome.cell, cardinal))
        })
    }

    pub fn failures(&self) -> impl Iterator<Item = (&CompoundCardinalCell, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    pub fn into_rows(self) -> Vec<CompoundCardinalOutcome> {
        self.cells
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<'a> IntoIterator for &'a CompoundCardinalParadigm {
    type Item = &'a CompoundCardinalOutcome;
    type IntoIter = std::slice::Iter<'a, CompoundCardinalOutcome>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl IntoIterator for CompoundCardinalParadigm {
    type Item = CompoundCardinalOutcome;
    type IntoIter = std::vec::IntoIter<CompoundCardinalOutcome>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

/// One compound-ordinal agreement request and its structured outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundOrdinalOutcome {
    pub cell: AdjectiveCell,
    pub result: Result<RealizedOrdinal, InflectionError>,
}

impl CompoundOrdinalOutcome {
    pub const fn cell(&self) -> &AdjectiveCell {
        &self.cell
    }

    pub fn ordinal(&self) -> Result<&RealizedOrdinal, &InflectionError> {
        self.result.as_ref()
    }

    pub fn error(&self) -> Option<&InflectionError> {
        self.result.as_ref().err()
    }

    pub fn into_parts(self) -> (AdjectiveCell, Result<RealizedOrdinal, InflectionError>) {
        (self.cell, self.result)
    }
}

/// Complete short/long adjective-agreement inventory for one compound ordinal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundOrdinalParadigm {
    pub(crate) value: u16,
    pub(crate) cells: Vec<CompoundOrdinalOutcome>,
}

impl CompoundOrdinalParadigm {
    pub const fn value(&self) -> u16 {
        self.value
    }

    pub fn form(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<&RealizedOrdinal, ParadigmLookupError> {
        let cell = AdjectiveCell {
            form,
            case,
            number,
            gender,
            animacy,
        };
        let outcome = self
            .cells
            .iter()
            .find(|outcome| outcome.cell == cell)
            .ok_or(ParadigmLookupError::NotRepresented)?;
        outcome
            .ordinal()
            .map_err(|error| ParadigmLookupError::Failed(error.clone()))
    }

    pub fn iter(&self) -> std::slice::Iter<'_, CompoundOrdinalOutcome> {
        self.cells.iter()
    }

    pub fn successes(&self) -> impl Iterator<Item = (&AdjectiveCell, &RealizedOrdinal)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|ordinal| (&outcome.cell, ordinal))
        })
    }

    pub fn failures(&self) -> impl Iterator<Item = (&AdjectiveCell, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    pub fn into_rows(self) -> Vec<CompoundOrdinalOutcome> {
        self.cells
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<'a> IntoIterator for &'a CompoundOrdinalParadigm {
    type Item = &'a CompoundOrdinalOutcome;
    type IntoIter = std::slice::Iter<'a, CompoundOrdinalOutcome>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl IntoIterator for CompoundOrdinalParadigm {
    type Item = CompoundOrdinalOutcome;
    type IntoIter = std::vec::IntoIter<CompoundOrdinalOutcome>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

/// A productive old or new comparative's complete agreement inventory.
///
/// Unlike a dictionary-backed [`AdjectiveParadigm`], this value retains
/// explicit caller-supplied principal parts and therefore has no dictionary ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComparativeParadigm {
    pub(crate) lemma: String,
    pub(crate) syncopated_citation: String,
    pub(crate) expanded_citation: String,
    pub(crate) cells: Vec<CellOutcome<AdjectiveCell>>,
}

impl ComparativeParadigm {
    /// The positive adjective lemma associated with this comparison lexeme.
    pub fn lemma(&self) -> &str {
        &self.lemma
    }

    /// The short masculine nominative singular principal part.
    pub fn syncopated_citation(&self) -> &str {
        &self.syncopated_citation
    }

    /// The short feminine nominative singular principal part.
    pub fn expanded_citation(&self) -> &str {
        &self.expanded_citation
    }

    /// Return one comparative form or distinguish an absent row from a failed row.
    pub fn form(
        &self,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Result<&FormSet, ParadigmLookupError> {
        forms_for(
            &self.cells,
            &AdjectiveCell {
                case,
                number,
                gender,
                animacy,
                form,
            },
        )
    }

    /// Iterate in canonical adjective-cell order.
    pub fn iter(&self) -> std::slice::Iter<'_, CellOutcome<AdjectiveCell>> {
        self.cells.iter()
    }

    /// Iterate over successful cells without discarding their grammar.
    pub fn successes(&self) -> impl Iterator<Item = (&AdjectiveCell, &FormSet)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .ok()
                .map(|forms| (&outcome.cell, forms))
        })
    }

    /// Iterate over retained typed failures.
    pub fn failures(&self) -> impl Iterator<Item = (&AdjectiveCell, &InflectionError)> {
        self.cells.iter().filter_map(|outcome| {
            outcome
                .result
                .as_ref()
                .err()
                .map(|error| (&outcome.cell, error))
        })
    }

    /// Consume the paradigm into its ordered cell rows.
    pub fn into_rows(self) -> Vec<CellOutcome<AdjectiveCell>> {
        self.cells
    }

    /// Number of represented cells, including typed failures.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Productive comparative paradigms always represent all adjective cells.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl<'a> IntoIterator for &'a ComparativeParadigm {
    type Item = &'a CellOutcome<AdjectiveCell>;
    type IntoIter = std::slice::Iter<'a, CellOutcome<AdjectiveCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.iter()
    }
}

impl IntoIterator for ComparativeParadigm {
    type Item = CellOutcome<AdjectiveCell>;
    type IntoIter = std::vec::IntoIter<CellOutcome<AdjectiveCell>>;

    fn into_iter(self) -> Self::IntoIter {
        self.cells.into_iter()
    }
}

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
    /// The closed class represented by this table.
    pub fn part_of_speech(&self) -> PartOfSpeech {
        self.part_of_speech
    }
}

paradigm_common!(ClosedClassParadigm<C>, C);

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
