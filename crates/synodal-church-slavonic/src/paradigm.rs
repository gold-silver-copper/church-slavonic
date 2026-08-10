use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FiniteTense, FiniteVerbCell,
    FormSet, Gender, GrammarCell, Number, NumeralCell, NumeralKind, ParticipleCell,
    ParticipleTense, ParticipleVoice, Person, PronounCell, Result,
};

use crate::{Inflector, LexemeSummary};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParadigmStatus {
    Attested,
    SourcedPrediction,
    InferredPrediction,
    AmbiguousPrediction,
    HistoricallyInvalid,
    Unsupported,
}

#[derive(Clone, Debug)]
pub struct ParadigmRow {
    cell: GrammarCell,
    outcome: Result<FormSet>,
    status: ParadigmStatus,
}

impl ParadigmRow {
    #[must_use]
    pub const fn cell(&self) -> GrammarCell {
        self.cell
    }

    #[must_use]
    pub const fn status(&self) -> ParadigmStatus {
        self.status
    }

    pub fn outcome(&self) -> &Result<FormSet> {
        &self.outcome
    }
}

#[derive(Clone, Debug)]
pub struct Paradigm {
    lexeme: LexemeSummary,
    rows: Vec<ParadigmRow>,
}

impl Paradigm {
    pub(crate) fn build(
        inflector: Inflector,
        lexeme: LexemeSummary,
        cells: impl IntoIterator<Item = GrammarCell>,
    ) -> Self {
        let rows = cells
            .into_iter()
            .map(|cell| {
                let outcome = inflector.form_by_id(lexeme.id(), cell);
                let status = classify(&outcome);
                ParadigmRow {
                    cell,
                    outcome,
                    status,
                }
            })
            .collect();
        Self { lexeme, rows }
    }

    #[must_use]
    pub fn lexeme(&self) -> &LexemeSummary {
        &self.lexeme
    }

    pub fn form(&self, cell: GrammarCell) -> Result<&FormSet> {
        self.rows
            .iter()
            .find(|row| row.cell == cell)
            .ok_or_else(|| Error::UnsupportedCell {
                reason: "cell is outside this specialized paradigm inventory".into(),
            })?
            .outcome
            .as_ref()
            .map_err(Clone::clone)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParadigmRow> {
        self.rows.iter()
    }

    pub fn attested(&self) -> impl Iterator<Item = &ParadigmRow> {
        self.rows
            .iter()
            .filter(|row| row.status == ParadigmStatus::Attested)
    }

    pub fn predicted(&self) -> impl Iterator<Item = &ParadigmRow> {
        self.rows.iter().filter(|row| {
            matches!(
                row.status,
                ParadigmStatus::SourcedPrediction
                    | ParadigmStatus::InferredPrediction
                    | ParadigmStatus::AmbiguousPrediction
            )
        })
    }

    pub fn failures(&self) -> impl Iterator<Item = &ParadigmRow> {
        self.rows.iter().filter(|row| row.outcome.is_err())
    }

    #[must_use]
    pub fn into_rows(self) -> Vec<ParadigmRow> {
        self.rows
    }
}

pub(crate) fn noun_cells(animacy: Animacy) -> Vec<GrammarCell> {
    Number::ALL
        .into_iter()
        .flat_map(|number| {
            Case::ALL.into_iter().map(move |case| {
                GrammarCell::Noun(synodal_church_slavonic_core::NounCell {
                    case,
                    number,
                    animacy,
                })
            })
        })
        .collect()
}

pub(crate) fn finite_cells(tense: FiniteTense) -> Vec<GrammarCell> {
    Number::ALL
        .into_iter()
        .flat_map(|number| {
            Person::ALL.into_iter().map(move |person| {
                GrammarCell::FiniteVerb(FiniteVerbCell {
                    tense,
                    person,
                    number,
                })
            })
        })
        .collect()
}

pub(crate) fn adjective_cells(form: AdjectiveForm) -> Vec<AdjectiveCell> {
    Number::ALL
        .into_iter()
        .flat_map(|number| {
            Case::ALL.into_iter().flat_map(move |case| {
                Gender::ALL.into_iter().flat_map(move |gender| {
                    Animacy::ALL.into_iter().map(move |animacy| AdjectiveCell {
                        case,
                        number,
                        gender,
                        animacy,
                        form,
                        comparison: Comparison::Positive,
                    })
                })
            })
        })
        .collect()
}

pub(crate) fn pronoun_cells() -> Vec<GrammarCell> {
    Number::ALL
        .into_iter()
        .flat_map(|number| {
            Case::ALL.into_iter().flat_map(move |case| {
                Gender::ALL.into_iter().flat_map(move |gender| {
                    Animacy::ALL.into_iter().map(move |animacy| {
                        GrammarCell::Pronoun(PronounCell {
                            case,
                            number,
                            gender: Some(gender),
                            person: None,
                            animacy,
                        })
                    })
                })
            })
        })
        .collect()
}

pub(crate) fn numeral_cells(kind: NumeralKind) -> Vec<GrammarCell> {
    let genders = [
        Some(Gender::Masculine),
        Some(Gender::Feminine),
        Some(Gender::Neuter),
        None,
    ];
    Number::ALL
        .into_iter()
        .flat_map(|number| {
            Case::ALL.into_iter().flat_map(move |case| {
                genders.into_iter().flat_map(move |gender| {
                    Animacy::ALL.into_iter().map(move |animacy| {
                        GrammarCell::Numeral(NumeralCell {
                            kind,
                            case,
                            number,
                            gender,
                            animacy,
                        })
                    })
                })
            })
        })
        .collect()
}

pub(crate) fn participle_cells(
    tense: ParticipleTense,
    voice: ParticipleVoice,
    form: AdjectiveForm,
) -> Vec<GrammarCell> {
    adjective_cells(form)
        .into_iter()
        .map(|agreement| {
            GrammarCell::Participle(ParticipleCell {
                tense,
                voice,
                agreement,
            })
        })
        .collect()
}

fn classify(outcome: &Result<FormSet>) -> ParadigmStatus {
    match outcome {
        Ok(forms) if forms.variants().iter().any(|variant| variant.is_attested()) => {
            ParadigmStatus::Attested
        }
        Ok(forms) if forms.variants().len() > 1 => ParadigmStatus::AmbiguousPrediction,
        Ok(forms)
            if forms
                .variants()
                .iter()
                .any(|variant| variant.recension_mapping.is_some()) =>
        {
            ParadigmStatus::InferredPrediction
        }
        Ok(_) => ParadigmStatus::SourcedPrediction,
        Err(Error::HistoricallyInvalidCell { .. }) => ParadigmStatus::HistoricallyInvalid,
        Err(_) => ParadigmStatus::Unsupported,
    }
}
