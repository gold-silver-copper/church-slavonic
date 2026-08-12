use synodal_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FiniteTense, FiniteVerbCell,
    FormSet, Gender, GrammarCell, Number, NumeralCell, NumeralKind, ParticipleCell,
    ParticipleTense, ParticipleVoice, Person, PronounCell, Result,
};

use crate::{Inflector, LexemeSummary, PartOfSpeech};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ParadigmStatus {
    Attested,
    IrregularOverride,
    SourcedPrediction,
    CallerSpecifiedPrediction,
    InferredPrediction,
    AmbiguousPrediction,
    HistoricallyInvalid,
    EvidenceIncomplete,
    MissingMetadata,
    OrthographicMetadataRequired,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ParadigmIdentity {
    Registered(LexemeSummary),
    Explicit {
        lemma: String,
        part_of_speech: PartOfSpeech,
    },
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
    identity: ParadigmIdentity,
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
        Self {
            identity: ParadigmIdentity::Registered(lexeme),
            rows,
        }
    }

    #[must_use]
    pub fn identity(&self) -> &ParadigmIdentity {
        &self.identity
    }

    #[must_use]
    pub fn registered_lexeme(&self) -> Option<&LexemeSummary> {
        match &self.identity {
            ParadigmIdentity::Registered(lexeme) => Some(lexeme),
            ParadigmIdentity::Explicit { .. } => None,
        }
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
                    | ParadigmStatus::CallerSpecifiedPrediction
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

    pub(crate) fn build_explicit(
        lemma: String,
        part_of_speech: PartOfSpeech,
        cells: impl IntoIterator<Item = GrammarCell>,
        mut generate: impl FnMut(GrammarCell) -> Result<FormSet>,
    ) -> Self {
        let rows = cells
            .into_iter()
            .map(|cell| {
                let outcome = generate(cell);
                let status = classify(&outcome);
                ParadigmRow {
                    cell,
                    outcome,
                    status,
                }
            })
            .collect();
        Self {
            identity: ParadigmIdentity::Explicit {
                lemma,
                part_of_speech,
            },
            rows,
        }
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
                    let animacies: &[Animacy] = if case == Case::Accusative {
                        &Animacy::ALL
                    } else {
                        &[Animacy::Inanimate]
                    };
                    animacies.iter().copied().map(move |animacy| AdjectiveCell {
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

pub(crate) fn classify(outcome: &Result<FormSet>) -> ParadigmStatus {
    match outcome {
        Ok(forms) if forms.variants().iter().any(|variant| variant.is_attested()) => {
            ParadigmStatus::Attested
        }
        Ok(forms)
            if forms.variants().iter().any(|variant| {
                matches!(
                    variant.source,
                    synodal_church_slavonic_core::FormSource::SynodalIrregularOverride { .. }
                )
            }) =>
        {
            ParadigmStatus::IrregularOverride
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
        Ok(forms)
            if forms.variants().iter().any(|variant| {
                matches!(
                    variant.source,
                    synodal_church_slavonic_core::FormSource::CallerSpecifiedPrediction { .. }
                )
            }) =>
        {
            ParadigmStatus::CallerSpecifiedPrediction
        }
        Ok(_) => ParadigmStatus::SourcedPrediction,
        Err(Error::HistoricallyInvalidCell { .. }) => ParadigmStatus::HistoricallyInvalid,
        Err(Error::EvidenceIncompleteCell { .. }) => ParadigmStatus::EvidenceIncomplete,
        Err(Error::MissingPrincipalPart { .. } | Error::MissingMetadata { .. }) => {
            ParadigmStatus::MissingMetadata
        }
        Err(Error::OrthographicMetadataRequired { .. }) => {
            ParadigmStatus::OrthographicMetadataRequired
        }
        Err(_) => ParadigmStatus::Unsupported,
    }
}
