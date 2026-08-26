use super::*;

pub(crate) fn resolve(lemma: &SynodalWord) -> Result<LexemeSummary> {
    let lookup = normalize_lookup_accentless(lemma.canonical());
    let matches: Vec<&RawLexeme> = LEXEMES
        .iter()
        .filter(|row| normalize_lookup_accentless(row.0[1]) == lookup)
        .collect();
    match matches.as_slice() {
        [] => Err(Error::UnknownLemma { lookup }),
        [row] => summary(row),
        rows => Err(Error::AmbiguousLexeme {
            lexemes: rows.iter().map(|row| LexemeId::from(row.0[0])).collect(),
        }),
    }
}

pub(crate) fn from_id(id: &LexemeId) -> Result<LexemeSummary> {
    let row = raw_by_id(id).ok_or_else(|| Error::UnknownLemma {
        lookup: id.to_string(),
    })?;
    summary(row)
}

/// Returns the contiguous run of rows whose first column equals `id`.
///
/// Every generated table used here is emitted sorted by its first column, so
/// rows sharing a lexeme are adjacent and a binary search replaces a full scan
/// without changing the order the resolver observes. That contract is not
/// implicit: `generated_lexeme_tables_are_sorted` fails if the generator ever
/// stops sorting, because an unsorted table would make these lookups silently
/// return the wrong rows rather than merely being slow.
pub(crate) fn rows_for<T>(
    rows: &'static [T],
    key: impl Fn(&T) -> &'static str,
    id: &str,
) -> &'static [T] {
    let start = rows.partition_point(|row| key(row) < id);
    let length = rows[start..].partition_point(|row| key(row) == id);
    &rows[start..start + length]
}

pub(crate) fn raw_by_id(id: &LexemeId) -> Option<&'static RawLexeme> {
    rows_for(LEXEMES, |row| row.0[0], id.as_str()).first()
}

pub(crate) fn exact_forms(id: &LexemeId, cell: &str) -> Vec<ExactFormRecord> {
    rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
        .iter()
        .filter(|row| row.0[1] == cell)
        .map(|row| ExactFormRecord {
            expanded: row.0[2],
            printed: row.0[3],
            evidence_id: row.0[4],
            source_kind: row.0[5],
        })
        .collect()
}

pub(crate) fn defect_for(id: &LexemeId, cell: &str) -> Result<Option<DefectiveInventoryRecord>> {
    let row = DEFECTIVE_INVENTORIES.iter().find(|row| {
        if row.0[0] != id.as_str() {
            return false;
        }
        match row.0[1] {
            "outside-inventory" => !row.0[2]
                .split(',')
                .map(str::trim)
                .any(|allowed| allowed == cell),
            "cell-prefix" => cell.starts_with(row.0[2]),
            _ => true,
        }
    });
    let Some(row) = row else {
        return Ok(None);
    };
    let kind = match row.0[3] {
        "historically-absent" => crate::DefectKind::HistoricallyAbsent,
        "evidence-incomplete" => crate::DefectKind::EvidenceIncomplete,
        value => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("generated defect inventory has unknown kind {value:?}"),
            });
        }
    };
    let field = parse_metadata_field(row.0[4])?;
    Ok(Some(DefectiveInventoryRecord {
        kind,
        field,
        reason: row.0[5],
    }))
}

pub(crate) fn parse_metadata_field(
    value: &str,
) -> Result<synodal_church_slavonic_core::MetadataField> {
    use synodal_church_slavonic_core::MetadataField;
    let field = match value {
        "present-stem" => MetadataField::PresentStem,
        "present-first-singular" => MetadataField::PresentFirstSingular,
        "present-third-plural" => MetadataField::PresentThirdPlural,
        "future-stem" => MetadataField::FutureStem,
        "future-first-singular" => MetadataField::FutureFirstSingular,
        "future-third-plural" => MetadataField::FutureThirdPlural,
        "imperfect-stem" => MetadataField::ImperfectStem,
        "aorist-stem" => MetadataField::AoristStem,
        "aorist-formation" => MetadataField::AoristFormation,
        "imperative-stem" => MetadataField::ImperativeStem,
        "imperative-formation" => MetadataField::ImperativeFormation,
        "imperfect-formation" => MetadataField::ImperfectFormation,
        "infinitive" => MetadataField::Infinitive,
        "supine-stem" => MetadataField::SupineStem,
        "l-participle-stem" => MetadataField::LParticipleStem,
        "participle-stem" => MetadataField::ParticipleStem,
        "participle-formation" => MetadataField::ParticipleFormation,
        "verbal-noun-stem" => MetadataField::VerbalNounStem,
        "aspect" => MetadataField::Aspect,
        "formation" => MetadataField::Formation,
        "regular-background" => MetadataField::RegularBackground,
        "irregular-override" => MetadataField::IrregularOverride,
        value => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("generated defect inventory has unknown metadata field {value:?}"),
            });
        }
    };
    Ok(field)
}

pub(crate) fn reviewed_evidence(evidence_ids: &str) -> Result<Vec<ReviewedEvidenceRecord>> {
    evidence_ids
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(|id| {
            let row = REVIEWED_EVIDENCE
                .iter()
                .find(|row| row.0[0] == id)
                .ok_or_else(|| Error::ContradictoryMetadata {
                    reason: format!("generated evidence provenance is missing {id}"),
                })?;
            let source_recension = match row.0[2] {
                "old-church-slavonic" => Recension::OldChurchSlavonic,
                "synodal-russian" => Recension::SynodalRussian,
                "mixed" => Recension::Mixed,
                value => {
                    return Err(Error::ContradictoryMetadata {
                        reason: format!("generated evidence {id} has unknown recension {value}"),
                    });
                }
            };
            Ok(ReviewedEvidenceRecord {
                id: row.0[0],
                source_id: row.0[1],
                source_recension,
                citation: row.0[3],
                role: row.0[4],
                note: row.0[5],
            })
        })
        .collect()
}

pub(crate) fn has_exact_forms(id: &LexemeId) -> bool {
    !rows_for(EXACT_FORMS, |row| row.0[0], id.as_str()).is_empty()
}

pub(crate) fn pronoun_profiles(
    id: &LexemeId,
) -> Vec<(Option<Gender>, Option<synodal_church_slavonic_core::Person>)> {
    rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
        .iter()
        .filter(|row| row.0[1].starts_with("pronoun:"))
        .filter_map(|row| {
            let GrammarCell::Pronoun(cell) = row.0[1].parse::<GrammarCell>().ok()? else {
                return None;
            };
            Some((cell.gender, cell.person))
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn is_exact_only(id: &LexemeId) -> bool {
    raw_by_id(id).is_some_and(|row| {
        (row.0[3].is_empty() || matches!(row.0[3], "exact" | "exact-complete-pronoun-table"))
            && has_exact_forms(id)
    })
}

pub(crate) fn has_exact_system(id: &LexemeId, prefix: &str) -> bool {
    rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
        .iter()
        .any(|row| row.0[1].starts_with(prefix))
}

pub(crate) fn has_principal_part(id: &LexemeId, system: &str) -> bool {
    rows_for(PRINCIPAL_PARTS, |row| row.0[0], id.as_str())
        .iter()
        .any(|row| row.0[1] == system)
}

pub(crate) fn has_principal_part_prefix(id: &LexemeId, prefix: &str) -> bool {
    rows_for(PRINCIPAL_PARTS, |row| row.0[0], id.as_str())
        .iter()
        .any(|row| row.0[1].starts_with(prefix))
}

pub(crate) fn has_accent_data(id: &LexemeId) -> bool {
    !rows_for(ACCENTS, |row| row.0[0], id.as_str()).is_empty()
        || !rows_for(ACCENT_PARADIGMS, |row| row.0[0], id.as_str()).is_empty()
        || rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
            .iter()
            .any(|row| row.0[2] != row.0[3])
}

pub(crate) fn irregular_evidence_for(id: &LexemeId, cell_key: &str) -> Option<&'static str> {
    IRREGULAR_OVERRIDES
        .iter()
        .find(|row| {
            if row.0[0] != id.as_str() {
                return false;
            }
            match row.0[1] {
                "present" => cell_key.starts_with("present:"),
                "future" => cell_key.starts_with("future:"),
                "aorist" => cell_key.starts_with("aorist:"),
                "imperfect" => cell_key.starts_with("imperfect:"),
                "imperative" => cell_key.starts_with("imperative:"),
                "noun-singular-dative-and-plural" => {
                    cell_key.starts_with("noun:dative:singular:") || cell_key.contains(":plural:")
                }
                _ => false,
            }
        })
        .map(|row| row.0[3])
}

pub(crate) fn accent_for(id: &LexemeId, cell: &str, expanded: &str) -> Option<AccentRecord> {
    ACCENTS
        .iter()
        .find(|row| row.0[0] == id.as_str() && row.0[1] == cell && row.0[2] == expanded)
        .map(|row| AccentRecord {
            accented: row.0[3],
            evidence_id: row.0[4],
            source_id: row.0[5],
            source_recension: row.0[6],
        })
}

pub(crate) fn accent_paradigm_for(
    id: &LexemeId,
    cell: synodal_church_slavonic_core::GrammarCell,
) -> Result<Option<AccentParadigm>> {
    let rows: Vec<&RawAccentParadigm> = rows_for(ACCENT_PARADIGMS, |row| row.0[0], id.as_str())
        .iter()
        .collect();
    let mut applicable_ids = Vec::new();
    for row in &rows {
        if parse_accent_scope(row.0[2])?.applies_to(cell) {
            applicable_ids.push(row.0[1]);
        }
    }
    applicable_ids.sort_unstable();
    applicable_ids.dedup();
    let Some(paradigm_id) = applicable_ids.first().copied() else {
        return Ok(None);
    };
    if applicable_ids.len() > 1 {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "multiple accent paradigms apply to {} in cell {cell:?}",
                id.as_str()
            ),
        });
    }
    let selected: Vec<&RawAccentParadigm> = rows
        .into_iter()
        .filter(|row| row.0[1] == paradigm_id)
        .collect();
    let first = selected[0];
    for row in &selected {
        if row.0[6..] != first.0[6..] {
            return Err(Error::ContradictoryMetadata {
                reason: format!("accent paradigm {paradigm_id} has inconsistent evidence"),
            });
        }
    }
    let accent_rules = selected
        .iter()
        .map(|row| {
            Ok(AccentRule {
                scope: parse_accent_scope(row.0[2])?,
                placement: parse_accent_placement(row.0[3])?,
                mark: parse_accent_mark(row.0[4])?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let breathing_rules = selected
        .iter()
        .filter(|row| !row.0[5].is_empty())
        .map(|row| {
            let placement =
                row.0[5]
                    .strip_prefix("psili@")
                    .ok_or_else(|| Error::ContradictoryMetadata {
                        reason: format!("invalid breathing rule {:?}", row.0[5]),
                    })?;
            Ok(BreathingRule {
                scope: parse_accent_scope(row.0[2])?,
                placement: parse_accent_placement(placement)?,
                mark: BreathingMark::Psili,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Some(AccentParadigm {
        id: paradigm_id.into(),
        accent_rules,
        breathing_rules,
        evidence: Evidence {
            id: EvidenceId::from(first.0[6]),
            source: SourceId::from(first.0[7]),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::AccentParadigm,
            authority_roles: vec![AuthorityRole::Accentual, AuthorityRole::Orthographic],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: first.0[8].into(),
            note: Some("reviewed reusable Synodal accent paradigm".into()),
        },
    }))
}

/// Builds the reviewed positional-spelling contract governing one cell.
///
/// Mirrors `accent_paradigm_for`: at most one paradigm may apply to a cell, and
/// every row of the selected paradigm must carry identical evidence, so a
/// lexeme cannot accumulate contradictory positional decisions. Rows sharing a
/// scope accumulate their operations in generated order.
#[allow(dead_code)]
pub(crate) fn positional_paradigm_for(
    id: &LexemeId,
    cell: synodal_church_slavonic_core::GrammarCell,
) -> Result<Option<PositionalParadigm>> {
    let rows: Vec<&RawPositionalParadigm> =
        rows_for(POSITIONAL_PARADIGMS, |row| row.0[0], id.as_str())
            .iter()
            .collect();
    let mut applicable_ids = Vec::new();
    for row in &rows {
        if parse_accent_scope(row.0[2])?.applies_to(cell) {
            applicable_ids.push(row.0[1]);
        }
    }
    applicable_ids.sort_unstable();
    applicable_ids.dedup();
    let Some(paradigm_id) = applicable_ids.first().copied() else {
        return Ok(None);
    };
    if applicable_ids.len() > 1 {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "multiple positional paradigms apply to {} in cell {cell:?}",
                id.as_str()
            ),
        });
    }
    let selected: Vec<&RawPositionalParadigm> = rows
        .into_iter()
        .filter(|row| row.0[1] == paradigm_id)
        .collect();
    let first = selected[0];
    for row in &selected {
        if row.0[4..] != first.0[4..] {
            return Err(Error::ContradictoryMetadata {
                reason: format!("positional paradigm {paradigm_id} has inconsistent evidence"),
            });
        }
    }
    let mut grouped: std::collections::BTreeMap<&str, Vec<PositionalOperation>> =
        std::collections::BTreeMap::new();
    for row in &selected {
        grouped
            .entry(row.0[2])
            .or_default()
            .push(parse_positional_operation(row.0[3])?);
    }
    let mut rules = Vec::with_capacity(grouped.len());
    for (scope, operations) in grouped {
        rules.push(PositionalRule {
            scope: parse_accent_scope(scope)?,
            operations: operations
                .into_iter()
                .filter(|operation| {
                    !matches!(
                        operation,
                        PositionalOperation::Initial(InitialPresentation::Preserve)
                    )
                })
                .collect(),
        });
    }
    Ok(Some(PositionalParadigm {
        id: paradigm_id.into(),
        rules,
        evidence: Evidence {
            id: EvidenceId::from(first.0[4]),
            source: SourceId::from(first.0[5]),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::OrthographicParadigm,
            authority_roles: vec![AuthorityRole::Orthographic],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: first.0[6].into(),
            note: Some("reviewed lexical positional spelling contract".into()),
        },
    }))
}

#[allow(dead_code)]
pub(crate) fn parse_positional_operation(value: &str) -> Result<PositionalOperation> {
    if value == "preserve" {
        return Ok(PositionalOperation::Initial(InitialPresentation::Preserve));
    }
    if value == "decimal-i-before-vowel" {
        return Ok(PositionalOperation::DecimalIBeforeVowel);
    }
    if value == "wide-plural-ending" {
        return Ok(PositionalOperation::WidePluralEnding);
    }
    if let Some(presentation) = value.strip_prefix("initial:") {
        let presentation = match presentation {
            "preserve" => InitialPresentation::Preserve,
            "wide-e" => InitialPresentation::WideE,
            "broad-on" => InitialPresentation::BroadOn,
            "iotated-ya" => InitialPresentation::IotatedYa,
            "digraph-uk" => InitialPresentation::DigraphUk,
            other => return invalid_metadata("positional initial presentation", other),
        };
        return Ok(PositionalOperation::Initial(presentation));
    }
    if let Some(rest) = value.strip_prefix("replace:") {
        let Some((replacement, occurrence)) = rest.split_once('@') else {
            return invalid_metadata("positional replacement", value);
        };
        let replacement = match replacement {
            "wide-e" => PositionalReplacement::WideE,
            "broad-on" => PositionalReplacement::BroadOn,
            "omega" => PositionalReplacement::Omega,
            "decimal-i" => PositionalReplacement::DecimalI,
            "iotated-ya" => PositionalReplacement::IotatedYa,
            "yeri" => PositionalReplacement::Yeri,
            "little-yus" => PositionalReplacement::LittleYus,
            other => return invalid_metadata("positional replacement letter", other),
        };
        let Some((direction, index)) = occurrence.split_once(':') else {
            return invalid_metadata("positional occurrence", occurrence);
        };
        let Ok(index) = index.parse::<u8>() else {
            return invalid_metadata("positional occurrence index", index);
        };
        let occurrence = match direction {
            "from-start" => LetterOccurrence::FromStart(index),
            "from-end" => LetterOccurrence::FromEnd(index),
            other => return invalid_metadata("positional occurrence direction", other),
        };
        return Ok(PositionalOperation::Replace {
            replacement,
            occurrence,
        });
    }
    invalid_metadata("positional operation", value)
}

pub(crate) fn parse_accent_scope(value: &str) -> Result<AccentScope> {
    let parts: Vec<&str> = value.split(':').collect();
    match parts.as_slice() {
        ["all"] => Ok(AccentScope::All),
        ["noun", numbers] => Ok(AccentScope::Noun {
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["noun", numbers, cases] => Ok(AccentScope::NounCases {
            numbers: parse_accent_numbers(numbers)?,
            cases: parse_accent_cases(cases)?,
        }),
        ["pronoun", numbers, cases] => Ok(AccentScope::PronounCases {
            numbers: parse_accent_numbers(numbers)?,
            cases: parse_accent_cases(cases)?,
        }),
        ["pronoun-agreeing", numbers, cases, genders, animacies] => {
            Ok(AccentScope::PronounAgreement {
                numbers: parse_accent_numbers(numbers)?,
                cases: parse_accent_cases(cases)?,
                genders: parse_accent_genders(genders)?,
                animacies: parse_accent_animacies(animacies)?,
            })
        }
        ["numeral", numbers] => Ok(AccentScope::Numeral {
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["adjective", form, comparison, numbers] => Ok(AccentScope::Adjective {
            form: match *form {
                "short" => AdjectiveForm::Short,
                "long" => AdjectiveForm::Long,
                value => return invalid_metadata("accent adjective form", value),
            },
            comparison: match *comparison {
                "positive" => Comparison::Positive,
                "comparative" => Comparison::Comparative,
                "superlative" => Comparison::Superlative,
                value => return invalid_metadata("accent comparison", value),
            },
            numbers: parse_accent_numbers(numbers)?,
        }),
        [
            "adjective-agreeing",
            form,
            comparison,
            numbers,
            cases,
            genders,
            animacies,
        ] => Ok(AccentScope::AdjectiveAgreement {
            form: match *form {
                "short" => AdjectiveForm::Short,
                "long" => AdjectiveForm::Long,
                value => return invalid_metadata("accent adjective form", value),
            },
            comparison: match *comparison {
                "positive" => Comparison::Positive,
                "comparative" => Comparison::Comparative,
                "superlative" => Comparison::Superlative,
                value => return invalid_metadata("accent comparison", value),
            },
            numbers: parse_accent_numbers(numbers)?,
            cases: parse_accent_cases(cases)?,
            genders: parse_accent_genders(genders)?,
            animacies: parse_accent_animacies(animacies)?,
        }),
        ["finite", tense, numbers] => Ok(AccentScope::FiniteVerb {
            tense: match *tense {
                "present" => FiniteTense::Present,
                "future" => FiniteTense::Future,
                "past" => FiniteTense::Past,
                "imperfect" => FiniteTense::Imperfect,
                "aorist" => FiniteTense::Aorist,
                value => return invalid_metadata("accent finite tense", value),
            },
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["participle", tense, voice, form, comparison, numbers] => Ok(AccentScope::Participle {
            tense: ParticipleTense::from_code(tense).ok_or_else(|| {
                Error::ContradictoryMetadata {
                    reason: format!("invalid accent participle tense {tense:?}"),
                }
            })?,
            voice: ParticipleVoice::from_code(voice).ok_or_else(|| {
                Error::ContradictoryMetadata {
                    reason: format!("invalid accent participle voice {voice:?}"),
                }
            })?,
            form: AdjectiveForm::from_code(form).ok_or_else(|| Error::ContradictoryMetadata {
                reason: format!("invalid accent participle form {form:?}"),
            })?,
            comparison: Comparison::from_code(comparison).ok_or_else(|| {
                Error::ContradictoryMetadata {
                    reason: format!("invalid accent participle comparison {comparison:?}"),
                }
            })?,
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["imperative", numbers] => Ok(AccentScope::Imperative {
            numbers: parse_accent_numbers(numbers)?,
        }),
        ["l-participle", numbers] => Ok(AccentScope::LParticiple {
            numbers: parse_accent_numbers(numbers)?,
        }),
        _ => invalid_metadata("accent scope", value),
    }
}

pub(crate) fn parse_accent_cases(value: &str) -> Result<Vec<Case>> {
    value
        .split(',')
        .map(|case| match case {
            "nominative" => Ok(Case::Nominative),
            "genitive" => Ok(Case::Genitive),
            "dative" => Ok(Case::Dative),
            "accusative" => Ok(Case::Accusative),
            "instrumental" => Ok(Case::Instrumental),
            "locative" => Ok(Case::Locative),
            "vocative" => Ok(Case::Vocative),
            value => invalid_metadata("accent case", value),
        })
        .collect()
}

pub(crate) fn parse_accent_numbers(value: &str) -> Result<Vec<Number>> {
    value
        .split(',')
        .map(|number| match number {
            "singular" => Ok(Number::Singular),
            "dual" => Ok(Number::Dual),
            "plural" => Ok(Number::Plural),
            value => invalid_metadata("accent number", value),
        })
        .collect()
}

pub(crate) fn parse_accent_genders(value: &str) -> Result<Vec<Gender>> {
    value
        .split(',')
        .map(|gender| match gender {
            "masculine" => Ok(Gender::Masculine),
            "feminine" => Ok(Gender::Feminine),
            "neuter" => Ok(Gender::Neuter),
            value => invalid_metadata("accent gender", value),
        })
        .collect()
}

pub(crate) fn parse_accent_animacies(value: &str) -> Result<Vec<Animacy>> {
    value
        .split(',')
        .map(|animacy| match animacy {
            "animate" => Ok(Animacy::Animate),
            "inanimate" => Ok(Animacy::Inanimate),
            value => invalid_metadata("accent animacy", value),
        })
        .collect()
}

pub(crate) fn parse_accent_placement(value: &str) -> Result<AccentPlacement> {
    let (kind, offset) = value
        .rsplit_once(':')
        .ok_or_else(|| Error::ContradictoryMetadata {
            reason: format!("invalid accent placement {value:?}"),
        })?;
    let offset = offset
        .parse::<u8>()
        .map_err(|_| Error::ContradictoryMetadata {
            reason: format!("invalid accent placement offset {offset:?}"),
        })?;
    match kind {
        "stem-vowel-from-start" => Ok(AccentPlacement::StemVowelFromStart(offset)),
        "word-vowel-from-start" => Ok(AccentPlacement::WordVowelFromStart(offset)),
        "ending-vowel-from-end" => Ok(AccentPlacement::EndingVowelFromEnd(offset)),
        value => invalid_metadata("accent placement", value),
    }
}

pub(crate) fn parse_accent_mark(value: &str) -> Result<AccentMark> {
    match value {
        "acute" => Ok(AccentMark::Acute),
        "grave" => Ok(AccentMark::Grave),
        "kamora" => Ok(AccentMark::Kamora),
        value => invalid_metadata("accent mark", value),
    }
}

#[cfg(test)]
mod binary_search_contract {
    use super::*;

    fn assert_sorted<T>(label: &str, rows: &'static [T], key: impl Fn(&T) -> &'static str) {
        let mut previous = "";
        for row in rows {
            let current = key(row);
            assert!(
                previous <= current,
                "{label} must stay sorted by its first column for `rows_for` to be correct, \
                 but {previous:?} precedes {current:?}"
            );
            previous = current;
        }
    }

    /// `rows_for` binary-searches these tables, so an unsorted one would make
    /// registry lookups silently return the wrong rows rather than merely being
    /// slow. This pins the generator's ordering contract.
    #[test]
    fn generated_lexeme_tables_are_sorted() {
        assert_sorted("LEXEMES", LEXEMES, |row| row.0[0]);
        assert_sorted("EXACT_FORMS", EXACT_FORMS, |row| row.0[0]);
        assert_sorted("PRINCIPAL_PARTS", PRINCIPAL_PARTS, |row| row.0[0]);
        assert_sorted("ACCENTS", ACCENTS, |row| row.0[0]);
        assert_sorted("ACCENT_PARADIGMS", ACCENT_PARADIGMS, |row| row.0[0]);
    }

    /// The searched range must be exactly the rows a full scan would keep, for
    /// every lexeme in the registry and in the same order.
    #[test]
    fn searched_ranges_match_an_exhaustive_scan() {
        for lexeme in LEXEMES {
            let id = lexeme.0[0];
            let searched: Vec<&str> = rows_for(EXACT_FORMS, |row| row.0[0], id)
                .iter()
                .map(|row| row.0[1])
                .collect();
            let scanned: Vec<&str> = EXACT_FORMS
                .iter()
                .filter(|row| row.0[0] == id)
                .map(|row| row.0[1])
                .collect();
            assert_eq!(searched, scanned, "exact forms diverged for {id}");

            let searched: Vec<&str> = rows_for(ACCENT_PARADIGMS, |row| row.0[0], id)
                .iter()
                .map(|row| row.0[2])
                .collect();
            let scanned: Vec<&str> = ACCENT_PARADIGMS
                .iter()
                .filter(|row| row.0[0] == id)
                .map(|row| row.0[2])
                .collect();
            assert_eq!(searched, scanned, "accent paradigms diverged for {id}");
        }
    }

    /// An identifier with no rows, and one ordering before and after every
    /// stored key, must all yield an empty range rather than panicking.
    #[test]
    fn absent_identifiers_yield_an_empty_range() {
        for id in ["", "synodal:noun:absent-lexeme", "\u{10ffff}"] {
            assert!(rows_for(EXACT_FORMS, |row| row.0[0], id).is_empty());
            assert!(rows_for(LEXEMES, |row| row.0[0], id).is_empty());
        }
    }
}

#[cfg(test)]
mod positional_contract {
    use super::*;

    #[test]
    fn every_reviewed_operation_code_parses_to_its_closed_variant() {
        assert_eq!(
            parse_positional_operation("wide-plural-ending").expect("wide plural"),
            PositionalOperation::WidePluralEnding
        );
        assert_eq!(
            parse_positional_operation("decimal-i-before-vowel").expect("decimal i"),
            PositionalOperation::DecimalIBeforeVowel
        );
        assert_eq!(
            parse_positional_operation("initial:wide-e").expect("initial"),
            PositionalOperation::Initial(InitialPresentation::WideE)
        );
        assert_eq!(
            parse_positional_operation("replace:omega@from-end:1").expect("replace"),
            PositionalOperation::Replace {
                replacement: PositionalReplacement::Omega,
                occurrence: LetterOccurrence::FromEnd(1),
            }
        );
        assert_eq!(
            parse_positional_operation("replace:wide-e@from-start:2").expect("replace"),
            PositionalOperation::Replace {
                replacement: PositionalReplacement::WideE,
                occurrence: LetterOccurrence::FromStart(2),
            }
        );
    }

    /// The operation vocabulary is closed precisely so a row cannot silently
    /// rewrite an unrelated character.
    #[test]
    fn unknown_or_malformed_operations_are_rejected() {
        for code in [
            "",
            "widen-everything",
            "initial:tilde",
            "replace:omega",
            "replace:tilde@from-end:1",
            "replace:omega@sideways:1",
            "replace:omega@from-end:x",
        ] {
            assert!(
                parse_positional_operation(code).is_err(),
                "{code:?} must not parse"
            );
        }
    }

    /// A lexeme with no reviewed positional contract must resolve to `None`
    /// rather than erroring, so the registry path stays a no-op until reviewed
    /// rows exist.
    #[test]
    fn absent_positional_contracts_are_not_an_error() {
        let id = LexemeId::from("synodal:noun:grad");
        let cell = GrammarCell::Noun(synodal_church_slavonic_core::NounCell {
            case: Case::Dative,
            number: Number::Plural,
            animacy: Animacy::Inanimate,
        });
        assert!(
            positional_paradigm_for(&id, cell)
                .expect("absent contract is not an error")
                .is_none()
        );
    }
}
