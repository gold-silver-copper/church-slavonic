//! Dictionary-backed Old Church Slavonic inflection.
//!
//! Dictionary table cells take precedence. When a known lexeme has sufficient
//! class metadata, missing cells may be produced by the pure core rules. Unknown
//! or ambiguous lexical facts are returned as typed errors.

#![forbid(unsafe_code)]

mod dictionary;
mod lookup;
mod paradigm;

pub use lookup::lookup;
pub use old_church_slavonic_core::*;
pub use paradigm::*;

use old_church_slavonic_core::adjective::AdjectiveLexeme;
use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::verb::VerbLexeme;

pub fn noun(lemma: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Noun)?;
    queried_result(lemma, record, noun_by_id(record.id, cell))
}

pub fn noun_by_id(id: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Noun)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    let record = lookup::find_lexeme(id).ok_or(InflectionError::UnknownLemma)?;
    let lexeme = noun_lexeme(record)?;
    predicted_noun(&lexeme, cell, true)
}

pub fn noun_with(lexeme: &NounLexeme, cell: NounCell) -> Result<FormSet, InflectionError> {
    predicted_noun(lexeme, cell, false)
}

pub fn noun_paradigm(id: &str) -> Result<NounParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Noun)?;
    let mut cells = Vec::with_capacity(Case::ALL.len() * Number::ALL.len());
    for number in Number::ALL {
        for case in Case::ALL {
            let cell = NounCell { case, number };
            cells.push(CellOutcome {
                cell,
                result: noun_by_id(id, cell),
            });
        }
    }
    Ok(NounParadigm {
        lexeme_id: id.to_string(),
        cells,
    })
}

pub fn adjective(lemma: &str, cell: AdjectiveCell) -> Result<FormSet, InflectionError> {
    let candidates = lookup(lemma, PartOfSpeech::Adjective)?;
    match candidates.as_slice() {
        [] => {
            let normalized = orthography::lookup_key(lemma)?;
            let class = if normalized.ends_with('ъ') {
                AdjectiveClass::Hard
            } else if normalized.ends_with(['ь', 'и']) {
                AdjectiveClass::Soft
            } else {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![MetadataField::AdjectiveClass],
                });
            };
            let lexeme = AdjectiveLexeme {
                lemma: normalized,
                class,
            };
            let predicted = old_church_slavonic_core::adjective::decline(&lexeme, cell)?;
            Ok(predicted_set(&lexeme.lemma, predicted, FormSourceKind::Oov))
        }
        [one] => {
            let record =
                lookup::find_lexeme(&one.id).ok_or_else(|| InflectionError::InvalidInput {
                    reason: "generated lookup candidate is missing".to_string(),
                })?;
            queried_result(lemma, record, adjective_by_id(&one.id, cell))
        }
        _ => Err(InflectionError::AmbiguousLexeme { candidates }),
    }
}

pub fn adjective_by_id(id: &str, cell: AdjectiveCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Adjective)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    let record = lookup::find_lexeme(id).ok_or(InflectionError::UnknownLemma)?;
    let class = parse_adjective_class(record.class).ok_or_else(|| {
        InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::AdjectiveClass],
        }
    })?;
    let lexeme = AdjectiveLexeme {
        lemma: record.lemma.to_string(),
        class,
    };
    let predicted = old_church_slavonic_core::adjective::decline(&lexeme, cell)?;
    Ok(predicted_set(
        record.lemma,
        predicted,
        FormSourceKind::DictionaryMetadata,
    ))
}

pub fn adjective_paradigm(id: &str) -> Result<AdjectiveParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Adjective)?;
    let mut cells = Vec::new();
    for form in AdjectiveForm::ALL {
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    for animacy in [Animacy::Animate, Animacy::Inanimate] {
                        let cell = AdjectiveCell {
                            case,
                            number,
                            gender,
                            animacy,
                            form,
                        };
                        cells.push(CellOutcome {
                            cell,
                            result: adjective_by_id(id, cell),
                        });
                    }
                }
            }
        }
    }
    Ok(AdjectiveParadigm {
        lexeme_id: id.to_string(),
        cells,
    })
}

pub fn adjective_with(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::adjective::decline(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn adjective_comparatives(lemma: &str) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Adjective)?;
    queried_result(lemma, record, adjective_comparatives_by_id(record.id))
}

pub fn adjective_comparatives_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Adjective)?;
    lookup::table_form(id, "adj:comparative:citation").ok_or(InflectionError::UnsupportedCell)
}

pub fn finite_verb(lemma: &str, cell: FiniteVerbCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, finite_verb_by_id(record.id, cell))
}

pub fn finite_verb_by_id(id: &str, cell: FiniteVerbCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    Err(InflectionError::UnsupportedCell)
}

pub fn finite_verb_paradigm(id: &str) -> Result<FiniteVerbParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let mut cells = Vec::new();
    for tense in FiniteTense::ALL {
        for number in Number::ALL {
            for person in Person::ALL {
                let cell = FiniteVerbCell {
                    tense,
                    person,
                    number,
                };
                cells.push(CellOutcome {
                    cell,
                    result: finite_verb_by_id(id, cell),
                });
            }
        }
    }
    Ok(FiniteVerbParadigm {
        lexeme_id: id.to_string(),
        cells,
    })
}

pub fn finite_verb_with(
    lexeme: &VerbLexeme,
    cell: FiniteVerbCell,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::finite(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn imperative(lemma: &str, cell: ImperativeCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, imperative_by_id(record.id, cell))
}

pub fn imperative_by_id(id: &str, cell: ImperativeCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, &cell.key()).ok_or(InflectionError::UnsupportedCell)
}

pub fn imperative_with(
    lexeme: &VerbLexeme,
    cell: ImperativeCell,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::imperative(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn imperative_paradigm(id: &str) -> Result<ImperativeParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let mut cells = Vec::new();
    for cell in ImperativeCell::SUPPORTED {
        cells.push(CellOutcome {
            cell,
            result: imperative_by_id(id, cell),
        });
    }
    Ok(ImperativeParadigm {
        lexeme_id: id.to_string(),
        cells,
    })
}

pub fn l_participle(lemma: &str, cell: LParticipleCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, l_participle_by_id(record.id, cell))
}

pub fn l_participle_by_id(id: &str, cell: LParticipleCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, &cell.key()).ok_or(InflectionError::UnsupportedCell)
}

pub fn l_participle_with(
    lexeme: &VerbLexeme,
    cell: LParticipleCell,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::l_participle(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn l_participle_paradigm(id: &str) -> Result<LParticipleParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let mut cells = Vec::new();
    for number in Number::ALL {
        for gender in Gender::ALL {
            let cell = LParticipleCell { gender, number };
            cells.push(CellOutcome {
                cell,
                result: l_participle_by_id(id, cell),
            });
        }
    }
    Ok(LParticipleParadigm {
        lexeme_id: id.to_string(),
        cells,
    })
}

pub fn participle(lemma: &str, cell: ParticipleCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, participle_by_id(record.id, cell))
}

pub fn participle_by_id(id: &str, cell: ParticipleCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, &cell.key()).ok_or(InflectionError::UnsupportedCell)
}

pub fn participle_with(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::participle(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn participle_paradigm(
    id: &str,
    kind: ParticipleKind,
) -> Result<ParticipleParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let mut cells = Vec::new();
    for form in AdjectiveForm::ALL {
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    for animacy in [Animacy::Animate, Animacy::Inanimate] {
                        let cell = ParticipleCell {
                            kind,
                            adjective: AdjectiveCell {
                                case,
                                number,
                                gender,
                                animacy,
                                form,
                            },
                        };
                        cells.push(CellOutcome {
                            cell,
                            result: participle_by_id(id, cell),
                        });
                    }
                }
            }
        }
    }
    Ok(ParticipleParadigm {
        lexeme_id: id.to_string(),
        kind,
        cells,
    })
}

pub fn participle_citation(lemma: &str, kind: ParticipleKind) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, participle_citation_by_id(record.id, kind))
}

pub fn participle_citation_by_id(
    id: &str,
    kind: ParticipleKind,
) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, &format!("verb:participle:{}:citation", kind.code()))
        .ok_or(InflectionError::UnsupportedCell)
}

pub fn infinitive(lemma: &str) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, infinitive_by_id(record.id))
}

pub fn infinitive_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, "verb:infinitive").ok_or(InflectionError::UnsupportedCell)
}

pub fn infinitive_with(lexeme: &VerbLexeme) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::infinitive(lexeme)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn supine(lemma: &str) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, supine_by_id(record.id))
}

pub fn supine_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, "verb:supine").ok_or(InflectionError::UnsupportedCell)
}

pub fn supine_with(lexeme: &VerbLexeme) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::supine(lexeme)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn verbal_noun(lemma: &str) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, verbal_noun_by_id(record.id))
}

pub fn verbal_noun_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, "verb:verbal-noun").ok_or(InflectionError::UnsupportedCell)
}

pub fn closed_class(
    lemma: &str,
    part_of_speech: PartOfSpeech,
    cell: ClosedClassCell,
) -> Result<FormSet, InflectionError> {
    if !matches!(
        part_of_speech,
        PartOfSpeech::Pronoun | PartOfSpeech::Numeral | PartOfSpeech::Determiner
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "closed_class accepts pronoun, numeral, or determiner".to_string(),
        });
    }
    let record = lookup::resolve_one(lemma, part_of_speech)?;
    queried_result(
        lemma,
        record,
        closed_class_by_id(record.id, part_of_speech, cell),
    )
}

pub fn closed_class_by_id(
    id: &str,
    part_of_speech: PartOfSpeech,
    cell: ClosedClassCell,
) -> Result<FormSet, InflectionError> {
    if !matches!(
        part_of_speech,
        PartOfSpeech::Pronoun | PartOfSpeech::Numeral | PartOfSpeech::Determiner
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "closed_class_by_id accepts pronoun, numeral, or determiner".to_string(),
        });
    }
    ensure_pos(id, part_of_speech)?;
    lookup::table_form(id, &cell.key(part_of_speech)).ok_or(InflectionError::UnsupportedCell)
}

pub fn dictionary_paradigm_by_id(id: &str) -> Result<DictionaryParadigm, InflectionError> {
    let record = lookup::find_lexeme(id).ok_or(InflectionError::UnknownLemma)?;
    let part_of_speech =
        lookup::parse_pos(record.pos).ok_or_else(|| InflectionError::InvalidInput {
            reason: "generated lexeme has an invalid part of speech".to_string(),
        })?;
    let cells = lookup::table_paradigm(id).ok_or(InflectionError::UnknownLemma)?;
    Ok(DictionaryParadigm {
        lexeme_id: id.to_string(),
        part_of_speech,
        cells,
    })
}

pub fn dictionary_form_by_id(id: &str, feature: &str) -> Result<FormSet, InflectionError> {
    lookup::find_lexeme(id).ok_or(InflectionError::UnknownLemma)?;
    lookup::table_form(id, feature).ok_or(InflectionError::UnsupportedCell)
}

fn ensure_pos(id: &str, expected: PartOfSpeech) -> Result<(), InflectionError> {
    let record = lookup::find_lexeme(id).ok_or(InflectionError::UnknownLemma)?;
    if record.pos == expected.code() {
        Ok(())
    } else {
        Err(InflectionError::InvalidInput {
            reason: format!("lexeme {id} is {}, not {expected}", record.pos),
        })
    }
}

fn add_alias_warning(
    query: &str,
    record: &dictionary::LexemeRecord,
    result: &mut FormSet,
) -> Result<(), InflectionError> {
    if orthography::lookup_key(query)? != record.key {
        result
            .warnings
            .push(InflectionWarning::OrthographicAliasUsed {
                canonical: record.lemma.to_string(),
            });
    }
    Ok(())
}

fn queried_result(
    query: &str,
    record: &dictionary::LexemeRecord,
    result: Result<FormSet, InflectionError>,
) -> Result<FormSet, InflectionError> {
    let mut result = result?;
    add_alias_warning(query, record, &mut result)?;
    Ok(result)
}

fn noun_lexeme(record: &dictionary::LexemeRecord) -> Result<NounLexeme, InflectionError> {
    let class =
        parse_noun_class(record.class).ok_or_else(|| InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::NounClass],
        })?;
    let gender =
        parse_gender(record.gender).ok_or_else(|| InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::Gender],
        })?;
    let animacy =
        parse_animacy(record.animacy).ok_or_else(|| InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::Animacy],
        })?;
    Ok(NounLexeme {
        lemma: record.lemma.to_string(),
        class,
        gender,
        animacy,
        number_restriction: parse_restriction(record.number_restriction),
    })
}

fn predicted_noun(
    lexeme: &NounLexeme,
    cell: NounCell,
    dictionary_metadata: bool,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::noun::decline(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(
        &lemma,
        predicted,
        if dictionary_metadata {
            FormSourceKind::DictionaryMetadata
        } else {
            FormSourceKind::Explicit
        },
    ))
}

enum FormSourceKind {
    DictionaryMetadata,
    Explicit,
    Oov,
}

fn predicted_set(lemma: &str, predicted: PredictedForm, kind: FormSourceKind) -> FormSet {
    let trace = predicted.trace;
    let source = match kind {
        FormSourceKind::DictionaryMetadata => FormSource::DictionaryMetadataRule {
            rule_id: predicted.rule_id,
        },
        FormSourceKind::Explicit => FormSource::ExplicitMetadataRule {
            rule_id: predicted.rule_id,
        },
        FormSourceKind::Oov => FormSource::OovPrediction {
            rule_id: predicted.rule_id,
        },
    };
    FormSet {
        lemma: lemma.to_string(),
        variants: vec![FormVariant {
            text: predicted.text,
            romanization: None,
        }],
        source,
        warnings: vec![InflectionWarning::PredictedNotDictionaryBacked],
        trace,
    }
}

fn parse_noun_class(value: &str) -> Option<NounClass> {
    match value {
        "o-m-hard" | "o-stem:m" => Some(NounClass::OMasculineHard),
        "o-n-hard" | "o-stem:n" => Some(NounClass::ONeuterHard),
        "jo-m-soft" => Some(NounClass::JoMasculineSoft),
        "jo-n-soft" => Some(NounClass::JoNeuterSoft),
        "a-hard" | "a-stem:f" => Some(NounClass::AHard),
        "ja-soft" => Some(NounClass::JaSoft),
        "i-f" => Some(NounClass::IFeminine),
        "i-m" => Some(NounClass::IMasculine),
        "u-m" => Some(NounClass::UMasculine),
        "n-m" => Some(NounClass::NMasculine),
        "n-n" => Some(NounClass::NNeuter),
        "nt-n" => Some(NounClass::NtNeuter),
        "r-n" => Some(NounClass::RStem),
        "s-n" => Some(NounClass::SNeuter),
        "v-f" => Some(NounClass::VFeminine),
        "indeclinable" => Some(NounClass::Indeclinable),
        _ => None,
    }
}

fn parse_adjective_class(value: &str) -> Option<AdjectiveClass> {
    match value {
        "adj-hard" => Some(AdjectiveClass::Hard),
        "adj-soft" => Some(AdjectiveClass::Soft),
        _ => None,
    }
}

fn parse_gender(value: &str) -> Option<Gender> {
    match value {
        "m" => Some(Gender::Masculine),
        "f" => Some(Gender::Feminine),
        "n" => Some(Gender::Neuter),
        _ => None,
    }
}

fn parse_animacy(value: &str) -> Option<Animacy> {
    match value {
        "an" => Some(Animacy::Animate),
        "in" => Some(Animacy::Inanimate),
        _ => None,
    }
}

fn parse_restriction(value: &str) -> NumberRestriction {
    match value {
        "sg" => NumberRestriction::SingularOnly,
        "du" => NumberRestriction::DualOnly,
        "pl" => NumberRestriction::PluralOnly,
        _ => NumberRestriction::All,
    }
}
