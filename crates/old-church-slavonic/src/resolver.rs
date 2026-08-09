//! Canonical dictionary-backed cell resolvers.
//!
//! Dictionary table cells take precedence. When a known lexeme has sufficient
//! class metadata, missing cells may be produced by the pure core rules. Unknown
//! or ambiguous lexical facts are returned as typed errors.

use crate::{dictionary, lookup, metadata::*, paradigm::*};
use old_church_slavonic_core::adjective::AdjectiveLexeme;
use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_core::*;

pub fn noun(lemma: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Noun)?;
    queried_result(lemma, record, noun_by_id(record.id, cell))
}

pub fn noun_by_id(id: &str, cell: NounCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Noun)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    let record = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Noun)))?;
    let lexeme = noun_lexeme(record)?;
    predicted_noun(&lexeme, cell, true).map_err(|error| error.with_lexeme_id(id))
}

pub fn noun_with(lexeme: &NounLexeme, cell: NounCell) -> Result<FormSet, InflectionError> {
    predicted_noun(lexeme, cell, false)
}

pub fn noun_paradigm_by_id(id: &str) -> Result<NounParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Noun)?;
    let lemma = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Noun)))?
        .lemma;
    Ok(build_noun_paradigm(id, lemma))
}

pub(crate) fn build_noun_paradigm(id: &str, lemma: &str) -> NounParadigm {
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
    NounParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells,
    }
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
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    let record = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Adjective)))?;
    let class = parse_adjective_class(record.class).ok_or_else(|| {
        InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::AdjectiveClass],
        }
    })?;
    let lexeme = AdjectiveLexeme {
        lemma: record.lemma.to_string(),
        class,
    };
    let predicted = old_church_slavonic_core::adjective::decline(&lexeme, cell)
        .map_err(|error| error.with_lexeme_id(id))?;
    Ok(predicted_set(
        record.lemma,
        predicted,
        FormSourceKind::DictionaryMetadata,
    ))
}

pub fn adjective_paradigm_by_id(id: &str) -> Result<AdjectiveParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Adjective)?;
    let lemma = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Adjective)))?
        .lemma;
    Ok(build_adjective_paradigm(id, lemma))
}

pub(crate) fn build_adjective_paradigm(id: &str, lemma: &str) -> AdjectiveParadigm {
    let mut cells = Vec::new();
    for form in AdjectiveForm::ALL {
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    for animacy in Animacy::ALL {
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
    AdjectiveParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells,
    }
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
    queried_result(lemma, record, comparative_citation_by_id(record.id))
}

pub fn comparative_citation_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Adjective)?;
    lookup::table_form(id, "adj:comparative:citation")
        .or_else(|| lookup::override_form(id, "adj:comparative:citation"))
        .ok_or_else(|| InflectionError::unsupported(id, RequestedCell::ComparativeCitation))
}

pub fn finite_verb(lemma: &str, cell: FiniteVerbCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, finite_by_id(record.id, cell))
}

pub fn finite_by_id(id: &str, cell: FiniteVerbCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    let metadata = verb_metadata_by_id(id)?;
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    generate_finite_from_metadata(&metadata, cell).map_err(|error| error.with_lexeme_id(id))
}

/// Generate through the same dictionary-metadata resolver after an offline
/// caller has already constructed and validated a metadata view. This does not
/// consult the bundled dictionary table and is used for leakage-controlled
/// held-cell evaluation.
pub fn finite_verb_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: FiniteVerbCell,
) -> Result<FormSet, InflectionError> {
    generate_finite_from_metadata(metadata, cell)
}

pub fn finite_paradigm_by_id(id: &str) -> Result<FiniteVerbParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let lemma = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?
        .lemma;
    Ok(build_finite_paradigm(id, lemma))
}

pub(crate) fn build_finite_paradigm(id: &str, lemma: &str) -> FiniteVerbParadigm {
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
                    result: finite_by_id(id, cell),
                });
            }
        }
    }
    FiniteVerbParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells,
    }
}

pub(crate) fn build_present_paradigm(id: &str, lemma: &str) -> VerbParadigm {
    let mut cells = Vec::with_capacity(Person::ALL.len() * Number::ALL.len());
    for number in Number::ALL {
        for person in Person::ALL {
            let cell = FiniteVerbCell {
                tense: FiniteTense::Present,
                person,
                number,
            };
            cells.push(CellOutcome {
                cell,
                result: finite_by_id(id, cell),
            });
        }
    }
    VerbParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells,
    }
}

pub fn present_paradigm_by_id(id: &str) -> Result<VerbParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let lemma = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?
        .lemma;
    Ok(build_present_paradigm(id, lemma))
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
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    let metadata = verb_metadata_by_id(id)?;
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    generate_imperative_from_metadata(&metadata, cell).map_err(|error| error.with_lexeme_id(id))
}

pub fn imperative_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: ImperativeCell,
) -> Result<FormSet, InflectionError> {
    generate_imperative_from_metadata(metadata, cell)
}

pub fn imperative_with(
    lexeme: &VerbLexeme,
    cell: ImperativeCell,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::imperative(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn imperative_paradigm_by_id(id: &str) -> Result<ImperativeParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let lemma = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?
        .lemma;
    Ok(build_imperative_paradigm(id, lemma))
}

pub(crate) fn build_imperative_paradigm(id: &str, lemma: &str) -> ImperativeParadigm {
    let mut cells = Vec::new();
    for cell in ImperativeCell::SUPPORTED {
        cells.push(CellOutcome {
            cell,
            result: imperative_by_id(id, cell),
        });
    }
    ImperativeParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells,
    }
}

pub fn l_participle(lemma: &str, cell: LParticipleCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, l_participle_by_id(record.id, cell))
}

pub fn l_participle_by_id(id: &str, cell: LParticipleCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    let metadata = verb_metadata_by_id(id)?;
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    generate_l_participle_from_metadata(&metadata, cell).map_err(|error| error.with_lexeme_id(id))
}

pub fn l_participle_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: LParticipleCell,
) -> Result<FormSet, InflectionError> {
    generate_l_participle_from_metadata(metadata, cell)
}

pub fn l_participle_with(
    lexeme: &VerbLexeme,
    cell: LParticipleCell,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::l_participle(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn l_participle_paradigm_by_id(id: &str) -> Result<LParticipleParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let lemma = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?
        .lemma;
    Ok(build_l_participle_paradigm(id, lemma))
}

pub(crate) fn build_l_participle_paradigm(id: &str, lemma: &str) -> LParticipleParadigm {
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
    LParticipleParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        cells,
    }
}

pub fn participle(lemma: &str, cell: ParticipleCell) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, participle_by_id(record.id, cell))
}

pub fn participle_by_id(id: &str, cell: ParticipleCell) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    if let Some(form) = lookup::table_form(id, &cell.key()) {
        return Ok(form);
    }
    let metadata = verb_metadata_by_id(id)?;
    if let Some(form) = lookup::override_form(id, &cell.key()) {
        return Ok(form);
    }
    generate_participle_from_metadata(&metadata, cell).map_err(|error| error.with_lexeme_id(id))
}

pub fn participle_from_dictionary_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: ParticipleCell,
) -> Result<FormSet, InflectionError> {
    generate_participle_from_metadata(metadata, cell)
}

pub fn participle_with(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
) -> Result<FormSet, InflectionError> {
    let predicted = old_church_slavonic_core::verb::participle(lexeme, cell)?;
    let lemma = orthography::canonical_display(&lexeme.lemma)?;
    Ok(predicted_set(&lemma, predicted, FormSourceKind::Explicit))
}

pub fn participle_paradigm_by_id(
    id: &str,
    kind: ParticipleKind,
) -> Result<ParticipleParadigm, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    let lemma = lookup::find_lexeme(id)
        .ok_or_else(|| InflectionError::unknown_id(id, Some(PartOfSpeech::Verb)))?
        .lemma;
    Ok(build_participle_paradigm(id, lemma, kind))
}

pub(crate) fn build_participle_paradigm(
    id: &str,
    lemma: &str,
    kind: ParticipleKind,
) -> ParticipleParadigm {
    let mut cells = Vec::new();
    for form in AdjectiveForm::ALL {
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    for animacy in Animacy::ALL {
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
    ParticipleParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        kind,
        cells,
    }
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
    let feature = format!("verb:participle:{}:citation", kind.code());
    if let Some(form) = lookup::table_form(id, &feature) {
        return Ok(form);
    }
    let metadata = verb_metadata_by_id(id)?;
    if let Some(form) = lookup::override_form(id, &feature) {
        return Ok(form);
    }
    generate_participle_from_metadata(
        &metadata,
        ParticipleCell {
            kind,
            adjective: AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
            },
        },
    )
    .map_err(|error| error.with_lexeme_id(id))
}

pub fn infinitive(lemma: &str) -> Result<FormSet, InflectionError> {
    let record = lookup::resolve_one(lemma, PartOfSpeech::Verb)?;
    queried_result(lemma, record, infinitive_by_id(record.id))
}

pub fn infinitive_by_id(id: &str) -> Result<FormSet, InflectionError> {
    ensure_pos(id, PartOfSpeech::Verb)?;
    lookup::table_form(id, "verb:infinitive")
        .or_else(|| lookup::override_form(id, "verb:infinitive"))
        .ok_or_else(|| InflectionError::unsupported(id, RequestedCell::Infinitive))
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
    lookup::table_form(id, "verb:supine")
        .ok_or_else(|| InflectionError::unsupported(id, RequestedCell::Supine))
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
    lookup::table_form(id, "verb:verbal-noun")
        .ok_or_else(|| InflectionError::unsupported(id, RequestedCell::VerbalNoun))
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
    lookup::table_form(id, &cell.key(part_of_speech)).ok_or_else(|| {
        InflectionError::unsupported(
            id,
            RequestedCell::ClosedClass {
                part_of_speech,
                cell,
            },
        )
    })
}

pub fn determiner_by_id(id: &str, cell: GenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Determiner, cell.closed_class())
}

pub fn pronoun_by_id(id: &str, cell: UngenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Pronoun, cell.closed_class())
}

pub fn personal_pronoun_by_id(
    id: &str,
    cell: PersonalPronounCell,
) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Pronoun, cell.closed_class())
}

pub fn gendered_pronoun_by_id(id: &str, cell: GenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Pronoun, cell.closed_class())
}

pub fn numeral_by_id(id: &str, cell: UngenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Numeral, cell.closed_class())
}

pub fn gendered_numeral_by_id(id: &str, cell: GenderedCell) -> Result<FormSet, InflectionError> {
    closed_class_by_id(id, PartOfSpeech::Numeral, cell.closed_class())
}

fn closed_class_identity(
    id: &str,
    part_of_speech: PartOfSpeech,
) -> Result<&'static dictionary::LexemeRecord, InflectionError> {
    ensure_pos(id, part_of_speech)?;
    lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, Some(part_of_speech)))
}

pub fn determiner_paradigm_by_id(id: &str) -> Result<DeterminerParadigm, InflectionError> {
    let record = closed_class_identity(id, PartOfSpeech::Determiner)?;
    Ok(build_gendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Determiner,
    ))
}

pub fn pronoun_paradigm_by_id(id: &str) -> Result<PronounParadigm, InflectionError> {
    let record = closed_class_identity(id, PartOfSpeech::Pronoun)?;
    Ok(build_ungendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Pronoun,
    ))
}

pub fn personal_pronoun_paradigm_by_id(
    id: &str,
) -> Result<PersonalPronounParadigm, InflectionError> {
    let record = closed_class_identity(id, PartOfSpeech::Pronoun)?;
    Ok(build_personal_pronoun_paradigm(id, record.lemma))
}

pub fn gendered_pronoun_paradigm_by_id(
    id: &str,
) -> Result<GenderedPronounParadigm, InflectionError> {
    let record = closed_class_identity(id, PartOfSpeech::Pronoun)?;
    Ok(build_gendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Pronoun,
    ))
}

pub fn numeral_paradigm_by_id(id: &str) -> Result<NumeralParadigm, InflectionError> {
    let record = closed_class_identity(id, PartOfSpeech::Numeral)?;
    Ok(build_ungendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Numeral,
    ))
}

pub fn gendered_numeral_paradigm_by_id(
    id: &str,
) -> Result<GenderedNumeralParadigm, InflectionError> {
    let record = closed_class_identity(id, PartOfSpeech::Numeral)?;
    Ok(build_gendered_closed_class_paradigm(
        id,
        record.lemma,
        PartOfSpeech::Numeral,
    ))
}

pub(crate) fn build_ungendered_closed_class_paradigm(
    id: &str,
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> ClosedClassParadigm<UngenderedCell> {
    let mut cells = Vec::with_capacity(Case::ALL.len() * Number::ALL.len());
    for number in Number::ALL {
        for case in Case::ALL {
            let cell = UngenderedCell { case, number };
            cells.push(CellOutcome {
                cell,
                result: closed_class_by_id(id, part_of_speech, cell.closed_class()),
            });
        }
    }
    ClosedClassParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        part_of_speech,
        cells,
    }
}

pub(crate) fn build_gendered_closed_class_paradigm(
    id: &str,
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> ClosedClassParadigm<GenderedCell> {
    let mut cells = Vec::with_capacity(Case::ALL.len() * Number::ALL.len() * Gender::ALL.len());
    for number in Number::ALL {
        for case in Case::ALL {
            for gender in Gender::ALL {
                let cell = GenderedCell {
                    case,
                    number,
                    gender,
                };
                cells.push(CellOutcome {
                    cell,
                    result: closed_class_by_id(id, part_of_speech, cell.closed_class()),
                });
            }
        }
    }
    ClosedClassParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        part_of_speech,
        cells,
    }
}

pub(crate) fn build_personal_pronoun_paradigm(id: &str, lemma: &str) -> PersonalPronounParadigm {
    let mut cells = Vec::with_capacity(Case::ALL.len() * Number::ALL.len() * Person::ALL.len());
    for number in Number::ALL {
        for case in Case::ALL {
            for person in Person::ALL {
                let cell = PersonalPronounCell {
                    case,
                    number,
                    person,
                };
                cells.push(CellOutcome {
                    cell,
                    result: personal_pronoun_by_id(id, cell),
                });
            }
        }
    }
    ClosedClassParadigm {
        lexeme_id: id.to_string(),
        lemma: lemma.to_string(),
        part_of_speech: PartOfSpeech::Pronoun,
        cells,
    }
}

pub fn dictionary_paradigm_by_id(id: &str) -> Result<DictionaryParadigm, InflectionError> {
    let record = lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, None))?;
    let part_of_speech =
        lookup::parse_pos(record.pos).ok_or_else(|| InflectionError::InvalidInput {
            reason: "generated lexeme has an invalid part of speech".to_string(),
        })?;
    let cells = lookup::table_paradigm(id).ok_or_else(|| InflectionError::unknown_id(id, None))?;
    Ok(DictionaryParadigm {
        lexeme_id: id.to_string(),
        lemma: record.lemma.to_string(),
        part_of_speech,
        cells,
    })
}

pub fn dictionary_form_by_id(id: &str, feature: &str) -> Result<FormSet, InflectionError> {
    lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, None))?;
    lookup::table_form(id, feature)
        .or_else(|| lookup::override_form(id, feature))
        .ok_or_else(|| {
            InflectionError::unsupported(
                id,
                RequestedCell::RawFeature {
                    feature: feature.to_string(),
                },
            )
        })
}

/// Resolve an accepted normalized verb feature through the same table-first
/// public APIs as the typed entry points. Non-verb exact table/override keys
/// remain available, but productive normalized-key dispatch is intentionally
/// verb-only. Ordinary callers should prefer the typed cell APIs.
pub fn form_by_id(id: &str, feature: &str) -> Result<FormSet, InflectionError> {
    let record = lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, None))?;
    if let Some(form) = lookup::table_form(id, feature) {
        return Ok(form);
    }
    let parts = feature.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        ["verb", "finite", tense, person, number] if record.pos == "verb" => finite_by_id(
            id,
            FiniteVerbCell {
                tense: parse_feature_tense(tense)?,
                person: parse_feature_person(person)?,
                number: parse_feature_number(number)?,
            },
        ),
        ["verb", "imperative", person, number] if record.pos == "verb" => imperative_by_id(
            id,
            ImperativeCell {
                person: parse_feature_person(person)?,
                number: parse_feature_number(number)?,
            },
        ),
        ["verb", "l-participle", gender, number] if record.pos == "verb" => l_participle_by_id(
            id,
            LParticipleCell {
                gender: parse_feature_gender(gender)?,
                number: parse_feature_number(number)?,
            },
        ),
        [
            "verb",
            "participle",
            kind,
            "adj",
            form,
            case,
            number,
            gender,
            animacy,
        ] if record.pos == "verb" => participle_by_id(
            id,
            ParticipleCell {
                kind: parse_feature_participle_kind(kind)?,
                adjective: AdjectiveCell {
                    case: parse_feature_case(case)?,
                    number: parse_feature_number(number)?,
                    gender: parse_feature_gender(gender)?,
                    animacy: parse_feature_animacy(animacy)?,
                    form: parse_feature_adjective_form(form)?,
                },
            },
        ),
        ["verb", "participle", kind, "citation"] if record.pos == "verb" => {
            participle_citation_by_id(id, parse_feature_participle_kind(kind)?)
        }
        ["verb", "infinitive"] if record.pos == "verb" => infinitive_by_id(id),
        ["verb", "supine"] if record.pos == "verb" => supine_by_id(id),
        ["verb", "verbal-noun"] if record.pos == "verb" => verbal_noun_by_id(id),
        _ => lookup::override_form(id, feature).ok_or_else(|| {
            InflectionError::unsupported(
                id,
                RequestedCell::RawFeature {
                    feature: feature.to_string(),
                },
            )
        }),
    }
}

fn invalid_feature(segment: &str) -> InflectionError {
    InflectionError::InvalidInput {
        reason: format!("invalid normalized feature segment: {segment}"),
    }
}

fn parse_feature_tense(value: &str) -> Result<FiniteTense, InflectionError> {
    match value {
        "present" => Ok(FiniteTense::Present),
        "imperfect" => Ok(FiniteTense::Imperfect),
        "aorist" => Ok(FiniteTense::Aorist),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_person(value: &str) -> Result<Person, InflectionError> {
    match value {
        "1" => Ok(Person::First),
        "2" => Ok(Person::Second),
        "3" => Ok(Person::Third),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_number(value: &str) -> Result<Number, InflectionError> {
    match value {
        "sg" => Ok(Number::Singular),
        "du" => Ok(Number::Dual),
        "pl" => Ok(Number::Plural),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_gender(value: &str) -> Result<Gender, InflectionError> {
    match value {
        "m" => Ok(Gender::Masculine),
        "f" => Ok(Gender::Feminine),
        "n" => Ok(Gender::Neuter),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_case(value: &str) -> Result<Case, InflectionError> {
    match value {
        "nom" => Ok(Case::Nominative),
        "gen" => Ok(Case::Genitive),
        "dat" => Ok(Case::Dative),
        "acc" => Ok(Case::Accusative),
        "ins" => Ok(Case::Instrumental),
        "loc" => Ok(Case::Locative),
        "voc" => Ok(Case::Vocative),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_animacy(value: &str) -> Result<Animacy, InflectionError> {
    match value {
        "an" => Ok(Animacy::Animate),
        "in" => Ok(Animacy::Inanimate),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_adjective_form(value: &str) -> Result<AdjectiveForm, InflectionError> {
    match value {
        "short" => Ok(AdjectiveForm::Short),
        "long" => Ok(AdjectiveForm::Long),
        _ => Err(invalid_feature(value)),
    }
}

fn parse_feature_participle_kind(value: &str) -> Result<ParticipleKind, InflectionError> {
    match value {
        "present-active" => Ok(ParticipleKind::PresentActive),
        "present-passive" => Ok(ParticipleKind::PresentPassive),
        "past-active" => Ok(ParticipleKind::PastActive),
        "past-passive" => Ok(ParticipleKind::PastPassive),
        _ => Err(invalid_feature(value)),
    }
}

#[derive(Clone)]
struct UsedMetadata {
    value: String,
    evidence: MetadataEvidence,
}

fn generate_finite_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: FiniteVerbCell,
) -> Result<FormSet, InflectionError> {
    let mut analyses = Vec::new();
    match cell.tense {
        FiniteTense::Present => {
            if metadata.present.is_empty() {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![MetadataField::VerbClass, MetadataField::PresentStem],
                });
            }
            for analysis in &metadata.present {
                let mut lexeme = VerbLexeme::new(&metadata.lemma, analysis.class.value);
                lexeme.aspect = metadata.aspect.as_ref().map(|aspect| aspect.value);
                lexeme.stems.present = Some(analysis.stem.value.clone());
                lexeme.stems.present_first_singular = analysis
                    .first_singular_stem
                    .as_ref()
                    .map(|stem| stem.value.clone());
                let predicted = old_church_slavonic_core::verb::finite(&lexeme, cell)?;
                let mut selected = vec![used(&analysis.class), used(&analysis.stem)];
                if cell.person == Person::First && cell.number == Number::Singular {
                    if let Some(first) = &analysis.first_singular_stem {
                        selected.push(used(first));
                    }
                }
                analyses.push(metadata_analysis(predicted, selected));
            }
        }
        FiniteTense::Imperfect => {
            if metadata.imperfect.is_empty() {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![
                        MetadataField::ImperfectStem,
                        MetadataField::ImperfectFormation,
                    ],
                });
            }
            for analysis in &metadata.imperfect {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.imperfect = Some(analysis.stem.value.clone());
                lexeme.formations.imperfect = Some(analysis.formation.value);
                lexeme.formations.imperfect_variant_policy = Some(analysis.variant_policy.value);
                let predicted = old_church_slavonic_core::verb::finite(&lexeme, cell)?;
                analyses.push(metadata_analysis(
                    predicted,
                    vec![
                        used(&analysis.stem),
                        used(&analysis.formation),
                        used(&analysis.variant_policy),
                    ],
                ));
            }
        }
        FiniteTense::Aorist => {
            if metadata.aorist.is_empty() {
                return Err(InflectionError::MissingLexicalMetadata {
                    needed: vec![MetadataField::AoristStem, MetadataField::AoristFormation],
                });
            }
            for analysis in &metadata.aorist {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.aorist = Some(analysis.stem.value.clone());
                lexeme.formations.aorist = Some(analysis.formation.value);
                let predicted = old_church_slavonic_core::verb::finite(&lexeme, cell)?;
                analyses.push(metadata_analysis(
                    predicted,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
    }
    metadata_form_set(&metadata.lemma, analyses)
}

fn generate_imperative_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: ImperativeCell,
) -> Result<FormSet, InflectionError> {
    if !cell.is_supported() {
        return Err(InflectionError::historically_invalid(
            &metadata.lexeme_id,
            RequestedCell::Imperative(cell),
        ));
    }
    if metadata.imperative.is_empty() {
        return Err(InflectionError::MissingLexicalMetadata {
            needed: vec![
                MetadataField::ImperativeStem,
                MetadataField::ImperativeFormation,
            ],
        });
    }
    let mut analyses = Vec::new();
    for analysis in &metadata.imperative {
        let mut lexeme = metadata_verb(metadata);
        lexeme.stems.imperative = Some(analysis.stem.value.clone());
        lexeme.formations.imperative = Some(analysis.formation.value);
        let predicted = old_church_slavonic_core::verb::imperative(&lexeme, cell)?;
        analyses.push(metadata_analysis(
            predicted,
            vec![used(&analysis.stem), used(&analysis.formation)],
        ));
    }
    metadata_form_set(&metadata.lemma, analyses)
}

fn generate_l_participle_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: LParticipleCell,
) -> Result<FormSet, InflectionError> {
    if metadata.l_participle.is_empty() {
        return Err(InflectionError::MissingLexicalMetadata {
            needed: vec![MetadataField::LParticipleStem],
        });
    }
    let mut analyses = Vec::new();
    for analysis in &metadata.l_participle {
        let mut lexeme = metadata_verb(metadata);
        lexeme.stems.aorist = Some(analysis.stem.value.clone());
        let predicted = old_church_slavonic_core::verb::l_participle(&lexeme, cell)?;
        analyses.push(metadata_analysis(predicted, vec![used(&analysis.stem)]));
    }
    metadata_form_set(&metadata.lemma, analyses)
}

fn generate_participle_from_metadata(
    metadata: &DictionaryVerbMetadata,
    cell: ParticipleCell,
) -> Result<FormSet, InflectionError> {
    let mut analyses = Vec::new();
    match cell.kind {
        ParticipleKind::PresentActive => {
            if metadata.present_active_participle.is_empty() {
                return missing_participle(
                    MetadataField::PresentActiveParticipleStem,
                    MetadataField::PresentActiveParticipleFormation,
                );
            }
            for analysis in &metadata.present_active_participle {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.present_active_participle = Some(analysis.stem.value.clone());
                lexeme.formations.present_active_participle = Some(analysis.formation.value);
                analyses.push(metadata_analysis(
                    old_church_slavonic_core::verb::participle(&lexeme, cell)?,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
        ParticipleKind::PresentPassive => {
            if metadata.present_passive_participle.is_empty() {
                return missing_participle(
                    MetadataField::PresentPassiveParticipleStem,
                    MetadataField::PresentPassiveParticipleFormation,
                );
            }
            for analysis in &metadata.present_passive_participle {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.present_passive_participle = Some(analysis.stem.value.clone());
                lexeme.formations.present_passive_participle = Some(analysis.formation.value);
                analyses.push(metadata_analysis(
                    old_church_slavonic_core::verb::participle(&lexeme, cell)?,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
        ParticipleKind::PastActive => {
            if metadata.past_active_participle.is_empty() {
                return missing_participle(
                    MetadataField::PastActiveParticipleStem,
                    MetadataField::PastActiveParticipleFormation,
                );
            }
            for analysis in &metadata.past_active_participle {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.past_active_participle = Some(analysis.stem.value.clone());
                lexeme.formations.past_active_participle = Some(analysis.formation.value);
                analyses.push(metadata_analysis(
                    old_church_slavonic_core::verb::participle(&lexeme, cell)?,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
        ParticipleKind::PastPassive => {
            if metadata.past_passive_participle.is_empty() {
                return missing_participle(
                    MetadataField::PastPassiveParticipleStem,
                    MetadataField::PastPassiveParticipleFormation,
                );
            }
            for analysis in &metadata.past_passive_participle {
                let mut lexeme = metadata_verb(metadata);
                lexeme.stems.past_passive_participle = Some(analysis.stem.value.clone());
                lexeme.formations.past_passive_participle = Some(analysis.formation.value);
                analyses.push(metadata_analysis(
                    old_church_slavonic_core::verb::participle(&lexeme, cell)?,
                    vec![used(&analysis.stem), used(&analysis.formation)],
                ));
            }
        }
    }
    metadata_form_set(&metadata.lemma, analyses)
}

fn missing_participle<T>(
    stem: MetadataField,
    formation: MetadataField,
) -> Result<T, InflectionError> {
    Err(InflectionError::MissingLexicalMetadata {
        needed: vec![stem, formation],
    })
}

fn metadata_verb(metadata: &DictionaryVerbMetadata) -> VerbLexeme {
    let class = metadata
        .present
        .first()
        .map_or(VerbClass::Irregular, |present| present.class.value);
    let mut lexeme = VerbLexeme::new(&metadata.lemma, class);
    lexeme.aspect = metadata.aspect.as_ref().map(|aspect| aspect.value);
    lexeme
}

trait TraceMetadataValue {
    fn trace_value(&self) -> String;
}

impl TraceMetadataValue for String {
    fn trace_value(&self) -> String {
        self.clone()
    }
}

macro_rules! trace_debug_value {
    ($($type:ty),+ $(,)?) => {
        $(impl TraceMetadataValue for $type {
            fn trace_value(&self) -> String {
                format!("{self:?}")
            }
        })+
    };
}

trace_debug_value!(
    VerbClass,
    ImperfectFormation,
    ImperfectVariantPolicy,
    AoristFormation,
    ImperativeFormation,
    PresentActiveParticipleFormation,
    PresentPassiveParticipleFormation,
    PastActiveParticipleFormation,
    PastPassiveParticipleFormation,
);

fn used<T: TraceMetadataValue>(metadata: &SourcedMetadata<T>) -> UsedMetadata {
    UsedMetadata {
        value: metadata.value.trace_value(),
        evidence: metadata.evidence.clone(),
    }
}

fn metadata_analysis(predicted: PredictedForm, used: Vec<UsedMetadata>) -> FormAnalysis {
    let source = FormSource::DictionaryMetadataRule {
        rule_id: predicted.rule_id,
    };
    let mut trace = used
        .iter()
        .map(|metadata| RuleStep {
            rule_id: RuleId::VerbDictionaryMetadata,
            before: metadata.evidence.source_form.clone().unwrap_or_default(),
            after: metadata.value.clone(),
            reason: "select a validated dictionary principal-part field",
        })
        .collect::<Vec<_>>();
    trace.extend(predicted.trace);
    let mut evidence = used
        .into_iter()
        .map(|metadata| metadata.evidence)
        .collect::<Vec<_>>();
    evidence.push(MetadataEvidence {
        field: None,
        provenance: MetadataProvenance::ProductiveRuleOutput,
        source_feature: Some(predicted.rule_id.code().to_string()),
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: Some("docs/MORPHOLOGY_SPEC.md".to_string()),
    });
    FormAnalysis {
        variants: vec![FormVariant {
            text: predicted.text,
            romanization: None,
        }],
        source,
        evidence,
        trace,
    }
}

fn metadata_form_set(lemma: &str, analyses: Vec<FormAnalysis>) -> Result<FormSet, InflectionError> {
    let mut variants = Vec::new();
    for analysis in &analyses {
        for variant in &analysis.variants {
            if !variants.contains(variant) {
                variants.push(variant.clone());
            }
        }
    }
    let multiple = analyses.len() > 1;
    let source = if multiple {
        FormSource::DictionaryMetadataAnalyses
    } else {
        analyses
            .first()
            .map_or(FormSource::DictionaryMetadataAnalyses, |analysis| {
                analysis.source.clone()
            })
    };
    let trace = if multiple {
        Vec::new()
    } else {
        analyses
            .first()
            .map_or_else(Vec::new, |analysis| analysis.trace.clone())
    };
    let mut warnings = vec![InflectionWarning::PredictedNotDictionaryBacked];
    if multiple {
        warnings.push(InflectionWarning::MultipleMorphologicalAnalyses);
    }
    if variants.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "metadata generation produced no form analysis".to_string(),
        });
    }
    let primary = variants.remove(0);
    Ok(FormSet::new(
        lemma, primary, variants, source, warnings, trace, analyses,
    ))
}

fn ensure_pos(id: &str, expected: PartOfSpeech) -> Result<(), InflectionError> {
    let record =
        lookup::find_lexeme(id).ok_or_else(|| InflectionError::unknown_id(id, Some(expected)))?;
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
        result.add_warning(InflectionWarning::OrthographicAliasUsed {
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

#[derive(Clone, Copy)]
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
    let primary = FormVariant {
        text: predicted.text,
        romanization: None,
    };
    let evidence = vec![MetadataEvidence {
        field: None,
        provenance: match kind {
            FormSourceKind::DictionaryMetadata => MetadataProvenance::DictionaryPrincipalPart,
            FormSourceKind::Explicit => MetadataProvenance::ExplicitCallerMetadata,
            FormSourceKind::Oov => MetadataProvenance::ProductiveRuleOutput,
        },
        source_feature: None,
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: None,
    }];
    let analyses = vec![FormAnalysis {
        variants: vec![primary.clone()],
        source: source.clone(),
        evidence,
        trace: trace.clone(),
    }];
    FormSet::new(
        lemma,
        primary,
        Vec::new(),
        source,
        vec![InflectionWarning::PredictedNotDictionaryBacked],
        trace,
        analyses,
    )
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
