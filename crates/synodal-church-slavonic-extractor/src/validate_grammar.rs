use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use synodal_church_slavonic_core::{GrammarCell, RenderedText, SynodalWord};

use super::*;

pub(crate) fn validate_grammar_cell(path: &Path, line: usize, value: &str) -> Result<()> {
    value
        .parse::<GrammarCell>()
        .map(|_| ())
        .map_err(|error| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line,
            reason: error.to_string(),
        })
}

pub(crate) fn grammar_cell_matches_part_of_speech(cell: GrammarCell, part_of_speech: &str) -> bool {
    matches!(
        (cell, part_of_speech),
        (GrammarCell::LexicalForm, _)
            | (
                GrammarCell::Indeclinable,
                "adverb" | "preposition" | "conjunction" | "particle" | "interjection"
            )
            | (GrammarCell::Noun(_), "noun" | "proper-noun")
            | (GrammarCell::Adjective(_), "adjective")
            | (GrammarCell::Determiner(_), "determiner")
            | (GrammarCell::Pronoun(_), "pronoun")
            | (GrammarCell::Numeral(_), "numeral")
            | (GrammarCell::Participle(_), "verb" | "participle")
            | (
                GrammarCell::FiniteVerb(_)
                    | GrammarCell::Imperative(_)
                    | GrammarCell::Infinitive
                    | GrammarCell::LParticiple(_)
                    | GrammarCell::Supine
                    | GrammarCell::VerbalNoun(_),
                "verb"
            )
    )
}

pub(crate) fn validate_cell_lexeme_pos(
    path: &Path,
    line: usize,
    lexeme_id: &str,
    value: &str,
    lexemes: &Table,
) -> Result<()> {
    let part_of_speech = lexemes
        .rows
        .iter()
        .find(|row| row[0] == lexeme_id)
        .map(|row| row[2].as_str())
        .ok_or_else(|| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line,
            reason: format!("grammar cell references unknown lexeme {lexeme_id}"),
        })?;
    let cell = value
        .parse::<GrammarCell>()
        .map_err(|error| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line,
            reason: error.to_string(),
        })?;
    let compatible = grammar_cell_matches_part_of_speech(cell, part_of_speech);
    if compatible {
        Ok(())
    } else {
        invalid(
            path,
            line,
            &format!(
                "grammar cell {value} is incompatible with {part_of_speech} lexeme {lexeme_id}"
            ),
        )
    }
}

pub(crate) fn validate_accents(path: &Path, table: &Table) -> Result<()> {
    let mut keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[7])?;
        if row[5].is_empty() || row[6] != TARGET {
            return invalid(
                path,
                offset + 2,
                "accent evidence requires a source ID and Synodal source recension",
            );
        }
        validate_word(path, offset + 2, &row[2], "expanded accent form")?;
        validate_word(path, offset + 2, &row[3], "accented form")?;
        if !keys.insert((row[0].clone(), row[1].clone(), row[2].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}:{}", row[0], row[1], row[2]),
            });
        }
        if row[2] == row[3] {
            return invalid(
                path,
                offset + 2,
                "accent metadata must add a presentation mark",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_accent_paradigms(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[10])?;
        if row[1].is_empty()
            || row[6].is_empty()
            || row[7].is_empty()
            || row[8].is_empty()
            || row[9] != TARGET
        {
            return invalid(
                path,
                offset + 2,
                "accent paradigm requires stable IDs, evidence, a source, and Synodal source recension",
            );
        }
        if !ids.insert((row[0].clone(), row[1].clone(), row[2].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}", row[0], row[1]),
            });
        }
        if !matches!(row[4].as_str(), "acute" | "grave" | "kamora") {
            return invalid(path, offset + 2, "unknown accent-paradigm mark");
        }
        if !row[3].starts_with("stem-vowel-from-start:")
            && !row[3].starts_with("word-vowel-from-start:")
            && !row[3].starts_with("ending-vowel-from-end:")
        {
            return invalid(path, offset + 2, "unknown accent-paradigm placement");
        }
        validate_accent_placement_code(path, offset + 2, &row[3])?;
        validate_accent_scope_code(path, offset + 2, &row[2])?;
        if !row[5].is_empty() && !row[5].starts_with("psili@") {
            return invalid(path, offset + 2, "unknown accent-paradigm breathing rule");
        }
        if let Some(placement) = row[5].strip_prefix("psili@") {
            validate_accent_placement_code(path, offset + 2, placement)?;
        }
    }
    Ok(())
}

/// Validates the reviewed lexical positional-spelling contract.
///
/// The scope grammar is shared with accent paradigms, because both answer the
/// same question — which cells does this reviewed decision govern — and a
/// second grammar would drift. The operation vocabulary is closed so that a row
/// can never rewrite an unrelated character.
pub(crate) fn validate_positional_paradigms(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        validate_target(path, line, &row[8])?;
        if row[1].is_empty() || row[4].is_empty() || row[5].is_empty() || row[6].is_empty() {
            return invalid(
                path,
                line,
                "positional paradigm requires stable IDs, evidence, a source, and a citation",
            );
        }
        if row[7] != TARGET {
            return invalid(
                path,
                line,
                "positional evidence requires a Synodal source recension",
            );
        }
        if !ids.insert((
            row[0].clone(),
            row[1].clone(),
            row[2].clone(),
            row[3].clone(),
        )) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}:{}", row[0], row[1], row[2]),
            });
        }
        validate_accent_scope_code(path, line, &row[2])?;
        validate_positional_operation_code(path, line, &row[3])?;
    }
    Ok(())
}

pub(crate) fn validate_positional_operation_code(
    path: &Path,
    line: usize,
    value: &str,
) -> Result<()> {
    if matches!(
        value,
        "preserve" | "decimal-i-before-vowel" | "wide-plural-ending"
    ) {
        return Ok(());
    }
    if let Some(presentation) = value.strip_prefix("initial:") {
        if matches!(
            presentation,
            "preserve" | "wide-e" | "broad-on" | "iotated-ya" | "digraph-uk"
        ) {
            return Ok(());
        }
        return invalid(path, line, "unknown initial positional presentation");
    }
    if let Some(rest) = value.strip_prefix("replace:") {
        let Some((replacement, occurrence)) = rest.split_once('@') else {
            return invalid(path, line, "positional replacement needs an occurrence");
        };
        if !matches!(
            replacement,
            "wide-e" | "broad-on" | "omega" | "decimal-i" | "iotated-ya" | "yeri" | "little-yus"
        ) {
            return invalid(path, line, "unknown positional replacement letter");
        }
        let Some((direction, index)) = occurrence.split_once(':') else {
            return invalid(
                path,
                line,
                "positional occurrence needs a direction and index",
            );
        };
        if !matches!(direction, "from-start" | "from-end") || index.parse::<u8>().is_err() {
            return invalid(path, line, "invalid positional occurrence");
        }
        return Ok(());
    }
    invalid(path, line, "unknown positional operation")
}

pub(crate) fn validate_accent_scope_code(path: &Path, line: usize, value: &str) -> Result<()> {
    let parts = value.split(':').collect::<Vec<_>>();
    if let [
        "adjective-agreeing",
        form,
        comparison,
        numbers,
        cases,
        genders,
        animacies,
    ] = parts.as_slice()
    {
        if !matches!(*form, "short" | "long")
            || !matches!(*comparison, "positive" | "comparative" | "superlative")
            || numbers
                .split(',')
                .any(|number| !matches!(number, "singular" | "dual" | "plural"))
            || cases.split(',').any(|case| {
                !matches!(
                    case,
                    "nominative"
                        | "genitive"
                        | "dative"
                        | "accusative"
                        | "instrumental"
                        | "locative"
                        | "vocative"
                )
            })
            || genders
                .split(',')
                .any(|gender| !matches!(gender, "masculine" | "feminine" | "neuter"))
            || animacies
                .split(',')
                .any(|animacy| !matches!(animacy, "animate" | "inanimate"))
        {
            return invalid(path, line, "invalid adjective-agreeing accent scope");
        }
        return Ok(());
    }
    if let ["pronoun-agreeing", numbers, cases, genders, animacies] = parts.as_slice() {
        if genders
            .split(',')
            .any(|gender| !matches!(gender, "masculine" | "feminine" | "neuter"))
            || animacies
                .split(',')
                .any(|animacy| !matches!(animacy, "animate" | "inanimate"))
        {
            return invalid(path, line, "invalid pronoun-agreeing accent scope");
        }
        return validate_accent_numbers_and_cases(path, line, numbers, Some(cases));
    }
    let (numbers, cases) = match parts.as_slice() {
        ["all"] => return Ok(()),
        ["noun", numbers] => (*numbers, None),
        ["noun", numbers, cases] => (*numbers, Some(*cases)),
        // The runtime registry parses reusable pronoun accent scopes into
        // `AccentScope::PronounCases` and `AccentScope::PronounAgreement`, so
        // the reviewed data layer accepts the same two shapes. Without these
        // arms a pronoun accent contract could be compiled but never authored.
        ["pronoun", numbers, cases] => (*numbers, Some(*cases)),
        ["numeral", numbers] => (*numbers, None),
        ["adjective", form, comparison, numbers]
            if matches!(*form, "short" | "long")
                && matches!(*comparison, "positive" | "comparative" | "superlative") =>
        {
            (*numbers, None)
        }
        ["finite", tense, numbers]
            if matches!(
                *tense,
                "present" | "future" | "past" | "imperfect" | "aorist"
            ) =>
        {
            (*numbers, None)
        }
        ["participle", tense, voice, form, comparison, numbers]
            if matches!(*tense, "present" | "past")
                && matches!(*voice, "active" | "passive")
                && matches!(*form, "short" | "long")
                && matches!(*comparison, "positive" | "comparative" | "superlative") =>
        {
            (*numbers, None)
        }
        ["imperative" | "l-participle", numbers] => (*numbers, None),
        _ => return invalid(path, line, "unknown accent-paradigm scope"),
    };
    validate_accent_numbers_and_cases(path, line, numbers, cases)
}

pub(crate) fn validate_accent_numbers_and_cases(
    path: &Path,
    line: usize,
    numbers: &str,
    cases: Option<&str>,
) -> Result<()> {
    if !numbers
        .split(',')
        .all(|number| matches!(number, "singular" | "dual" | "plural"))
    {
        return invalid(path, line, "unknown number in accent-paradigm scope");
    }
    if cases.is_some_and(|cases| {
        cases.split(',').any(|case| {
            !matches!(
                case,
                "nominative"
                    | "genitive"
                    | "dative"
                    | "accusative"
                    | "instrumental"
                    | "locative"
                    | "vocative"
            )
        })
    }) {
        return invalid(path, line, "unknown case in accent-paradigm scope");
    }
    Ok(())
}

pub(crate) fn validate_accent_placement_code(path: &Path, line: usize, value: &str) -> Result<()> {
    let Some((kind, offset)) = value.rsplit_once(':') else {
        return invalid(path, line, "invalid accent-paradigm placement");
    };
    if !matches!(
        kind,
        "stem-vowel-from-start" | "word-vowel-from-start" | "ending-vowel-from-end"
    ) || offset.parse::<u8>().is_err()
    {
        return invalid(path, line, "invalid accent-paradigm placement");
    }
    Ok(())
}

pub(crate) fn validate_positional_rules(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[6])?;
        validate_word(path, offset + 2, &row[1], "positional input")?;
        validate_word(path, offset + 2, &row[3], "positional output")?;
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn validate_transformation_rules(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if row[1] != "old-church-slavonic" || row[2] != TARGET {
            return invalid(
                path,
                offset + 2,
                "recension transformations must explicitly map OCS to Synodal Russian",
            );
        }
        if !matches!(row[4].as_str(), "reviewed" | "automatically-validated") {
            return invalid(path, offset + 2, "unknown transformation review status");
        }
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn validate_conflicts(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !row[1].starts_with("ocs:") || !row[2].starts_with("synodal:") {
            return invalid(
                path,
                offset + 2,
                "conflicts require stable OCS source and Synodal target IDs",
            );
        }
        if row[5].is_empty() || row[6].is_empty() || row[7].is_empty() {
            return invalid(
                path,
                offset + 2,
                "conflicts must retain support, contradiction, and resolution",
            );
        }
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn validate_conflict_evidence(
    path: &Path,
    conflicts: &Table,
    evidence: &Table,
) -> Result<()> {
    let known = evidence
        .rows
        .iter()
        .map(|row| row[0].as_str())
        .collect::<BTreeSet<_>>();
    for (offset, row) in conflicts.rows.iter().enumerate() {
        for column in [5_usize, 6_usize] {
            for evidence_id in row[column].split(',').filter(|value| !value.is_empty()) {
                if !known.contains(evidence_id) {
                    return invalid(
                        path,
                        offset + 2,
                        &format!("conflict has unregistered evidence {evidence_id:?}"),
                    );
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_irregular_overrides(path: &Path, table: &Table) -> Result<()> {
    let mut keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[4])?;
        if !keys.insert((row[0].clone(), row[1].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}", row[0], row[1]),
            });
        }
        if row[2] != "data/synodal/exact_forms.tsv" || row[3].is_empty() {
            return invalid(
                path,
                offset + 2,
                "irregular overrides must point to the exact-form registry and evidence",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_defective_inventories(
    path: &Path,
    table: &Table,
    lexemes: &Table,
) -> Result<()> {
    let mut keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        validate_target(path, line, &row[7])?;
        if !keys.insert((row[0].clone(), row[1].clone(), row[2].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}:{}", row[0], row[1], row[2]),
            });
        }
        let Some(lexeme) = lexemes.rows.iter().find(|lexeme| lexeme[0] == row[0]) else {
            return invalid(
                path,
                line,
                "defective inventory references an unknown lexeme",
            );
        };
        if lexeme[2] != "verb" {
            return invalid(
                path,
                line,
                "defective inventory references a non-verb lexeme",
            );
        }
        match row[1].as_str() {
            "outside-inventory" => {
                let mut cells = BTreeSet::new();
                for cell in row[2]
                    .split(',')
                    .map(str::trim)
                    .filter(|cell| !cell.is_empty())
                {
                    validate_grammar_cell(path, line, cell)?;
                    validate_cell_lexeme_pos(path, line, &row[0], cell, lexemes)?;
                    if !cells.insert(cell) {
                        return invalid(path, line, "defective inventory repeats an allowed cell");
                    }
                }
                if cells.is_empty() {
                    return invalid(
                        path,
                        line,
                        "defective inventory requires at least one allowed cell",
                    );
                }
            }
            "cell-prefix" => {
                if !matches!(
                    row[2].as_str(),
                    "present:"
                        | "future:"
                        | "past:"
                        | "imperfect:"
                        | "aorist:"
                        | "imperative:"
                        | "l-participle:"
                        | "participle:present:active:"
                        | "participle:present:passive:"
                        | "participle:past:active:"
                        | "participle:past:passive:"
                        | "verbal-noun:"
                ) {
                    return invalid(path, line, "unknown defective cell-system prefix");
                }
            }
            _ => return invalid(path, line, "unknown defective inventory mode"),
        }
        if !matches!(
            row[3].as_str(),
            "historically-absent" | "evidence-incomplete"
        ) {
            return invalid(path, line, "unknown defect kind");
        }
        if !matches!(
            row[4].as_str(),
            "present-stem"
                | "present-first-singular"
                | "present-third-plural"
                | "future-stem"
                | "future-first-singular"
                | "future-third-plural"
                | "imperfect-stem"
                | "aorist-stem"
                | "aorist-formation"
                | "imperative-stem"
                | "imperative-formation"
                | "imperfect-formation"
                | "infinitive"
                | "supine-stem"
                | "l-participle-stem"
                | "participle-stem"
                | "participle-formation"
                | "verbal-noun-stem"
                | "aspect"
                | "formation"
                | "regular-background"
                | "irregular-override"
        ) {
            return invalid(path, line, "unknown defect metadata field");
        }
        if row[5].trim().is_empty() || row[6].trim().is_empty() {
            return invalid(
                path,
                line,
                "defective inventory requires a reason and evidence",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_irregular_verb_inventory(path: &Path, table: &Table) -> Result<()> {
    let expected_orders = (2_u8..=100)
        .filter(|order| *order != 97)
        .collect::<BTreeSet<_>>();
    let mut orders = BTreeSet::new();
    let mut headword_orders = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        validate_target(path, line, &row[7])?;
        let source_order = row[0]
            .parse::<u8>()
            .map_err(|_| ExtractionError::InvalidRow {
                file: path.to_owned(),
                line,
                reason: "irregular inventory source_order must be an integer".into(),
            })?;
        if !orders.insert(source_order) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        if row[1].trim().is_empty() || !headword_orders.insert((row[1].clone(), source_order)) {
            return invalid(path, line, "irregular inventory requires a source headword");
        }
        let systems = row[2]
            .split(',')
            .map(str::trim)
            .filter(|system| !system.is_empty())
            .collect::<BTreeSet<_>>();
        if systems.is_empty()
            || systems.iter().any(|system| {
                !matches!(
                    *system,
                    "present"
                        | "future"
                        | "aorist"
                        | "imperfect"
                        | "imperative"
                        | "l-participle"
                        | "present-active-participle"
                        | "present-passive-participle"
                        | "past-active-participle"
                        | "past-passive-participle"
                        | "stem-alternation"
                        | "defectiveness"
                )
            })
        {
            return invalid(path, line, "irregular inventory has an unknown system code");
        }
        if !matches!(
            row[3].as_str(),
            "bundled-exact-and-productive"
                | "bundled-exact-and-defective"
                | "caller-exact-principal-parts"
                | "typed-defective-inventory"
        ) {
            return invalid(path, line, "irregular inventory has an unknown strategy");
        }
        if !matches!(
            row[4].as_str(),
            "implemented-bundled"
                | "implemented-by-metadata-contract"
                | "source-evidence-incomplete"
        ) {
            return invalid(
                path,
                line,
                "irregular inventory has an unknown implementation status",
            );
        }
        if row[5].trim().is_empty() || row[6].trim().is_empty() {
            return invalid(
                path,
                line,
                "irregular inventory requires evidence and a note",
            );
        }
    }
    if orders != expected_orders {
        let missing = expected_orders
            .difference(&orders)
            .copied()
            .collect::<Vec<_>>();
        let extra = orders
            .difference(&expected_orders)
            .copied()
            .collect::<Vec<_>>();
        return invalid(
            path,
            1,
            &format!(
                "irregular inventory must cover all 98 Alypy §104 verb entries; missing {missing:?}, extra {extra:?}"
            ),
        );
    }
    Ok(())
}

pub(crate) fn validate_morphology_references<const N: usize>(
    lexeme_path: &Path,
    lexemes: &Table,
    tables: [(&Path, &Table, usize); N],
) -> Result<()> {
    let ids: BTreeSet<&str> = lexemes.rows.iter().map(|row| row[0].as_str()).collect();
    for (path, table, id_column) in tables {
        for (offset, row) in table.rows.iter().enumerate() {
            if !ids.contains(row[id_column].as_str()) {
                return invalid(
                    path,
                    offset + 2,
                    &format!(
                        "unknown lexeme ID {:?}; target registry is {}",
                        row[id_column],
                        lexeme_path.display()
                    ),
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_semantic_alignments(path: &Path, table: &Table) -> Result<()> {
    let mut mappings = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if row[1].is_empty() || row[2].is_empty() || row[4].is_empty() || row[5].is_empty() {
            return invalid(
                path,
                offset + 2,
                "semantic alignments require both sense IDs, evidence, and a review note",
            );
        }
        if !matches!(
            row[3].as_str(),
            "established" | "false-friend" | "uncertain"
        ) {
            return invalid(path, offset + 2, "unknown semantic alignment status");
        }
        if !mappings.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn validate_semantic_alignment_evidence(
    path: &Path,
    alignments: &Table,
    evidence: &Table,
) -> Result<()> {
    let decisions = evidence
        .rows
        .iter()
        .map(|row| (row[0].as_str(), row[4].as_str()))
        .collect::<BTreeMap<_, _>>();
    for (offset, row) in alignments.rows.iter().enumerate() {
        let required_decision = if row[3] == "false-friend" {
            "rejected"
        } else {
            "reviewed"
        };
        if decisions.get(row[4].as_str()) != Some(&required_decision) {
            return invalid(
                path,
                offset + 2,
                &format!(
                    "semantic alignment status {:?} requires {required_decision} evidence",
                    row[3]
                ),
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_alignment_references(
    alignment_path: &Path,
    alignments: &Table,
    lexemes: &Table,
    transformation_rules: &Table,
    conflict_path: &Path,
    conflicts: &Table,
) -> Result<()> {
    let target_ids: BTreeSet<&str> = lexemes.rows.iter().map(|row| row[0].as_str()).collect();
    let transformations: BTreeSet<&str> = transformation_rules
        .rows
        .iter()
        .map(|row| row[0].as_str())
        .collect();
    for (offset, row) in alignments.rows.iter().enumerate() {
        if !row[1].starts_with("ocs:") || !target_ids.contains(row[2].as_str()) {
            return invalid(
                alignment_path,
                offset + 2,
                "alignment requires an OCS source ID and an existing Synodal target ID",
            );
        }
        for transformation in row[9].split(',').filter(|value| !value.is_empty()) {
            if !transformations.contains(transformation) {
                return invalid(
                    alignment_path,
                    offset + 2,
                    &format!("unknown recension transformation {transformation:?}"),
                );
            }
        }
    }
    for (offset, row) in conflicts.rows.iter().enumerate() {
        if !target_ids.contains(row[2].as_str()) {
            return invalid(
                conflict_path,
                offset + 2,
                "conflict target is absent from the Synodal lexeme registry",
            );
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_dictionary_references(
    sense_path: &Path,
    senses: &Table,
    example_path: &Path,
    examples: &Table,
    semantic_path: &Path,
    semantic_alignments: &Table,
    lexemes: &Table,
    morphology_alignments: &Table,
) -> Result<()> {
    let lexeme_ids: BTreeSet<&str> = lexemes.rows.iter().map(|row| row[0].as_str()).collect();
    let sense_ids: BTreeSet<&str> = senses.rows.iter().map(|row| row[1].as_str()).collect();
    let mapping_ids: BTreeSet<&str> = morphology_alignments
        .rows
        .iter()
        .map(|row| row[0].as_str())
        .collect();
    for (offset, row) in senses.rows.iter().enumerate() {
        if !lexeme_ids.contains(row[0].as_str()) {
            return invalid(
                sense_path,
                offset + 2,
                "sense refers to an unknown Synodal lexeme",
            );
        }
    }
    for (offset, row) in examples.rows.iter().enumerate() {
        if !lexeme_ids.contains(row[1].as_str()) {
            return invalid(
                example_path,
                offset + 2,
                "example refers to an unknown Synodal lexeme",
            );
        }
    }
    for (offset, row) in semantic_alignments.rows.iter().enumerate() {
        if !mapping_ids.contains(row[0].as_str()) || !sense_ids.contains(row[2].as_str()) {
            return invalid(
                semantic_path,
                offset + 2,
                "semantic decision requires an existing mapping and target sense",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_senses(
    path: &Path,
    table: &Table,
    source_recensions: &BTreeMap<String, String>,
) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !ids.insert((row[0].clone(), row[1].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}", row[0], row[1]),
            });
        }
        if !row[0].starts_with("synodal:") || row[1].is_empty() || row[2].is_empty() {
            return invalid(
                path,
                offset + 2,
                "sense requires a Synodal lexeme ID, stable sense ID, and gloss",
            );
        }
        let registered_recension =
            source_recensions
                .get(&row[4])
                .ok_or_else(|| ExtractionError::InvalidRow {
                    file: path.to_owned(),
                    line: offset + 2,
                    reason: format!("sense source {:?} is not registered", row[4]),
                })?;
        if registered_recension != &row[5] {
            return invalid(
                path,
                offset + 2,
                "sense source recension disagrees with the source inventory",
            );
        }
        let valid_status = match row[5].as_str() {
            "mixed" => row[6] == "reviewed-with-synodal-corpus",
            "old-church-slavonic" => matches!(
                row[6].as_str(),
                "reviewed-ocs-inheritance" | "reviewed-with-synodal-corpus"
            ),
            "synodal-russian" => row[6] == "normative",
            _ => false,
        };
        if !valid_status {
            return invalid(
                path,
                offset + 2,
                "sense semantic status is incompatible with its source recension",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_examples(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        RenderedText::parse(&row[2]).map_err(|error| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line: offset + 2,
            reason: format!("invalid example text: {error}"),
        })?;
        validate_target(path, offset + 2, &row[7])?;
        if row[6] != TARGET {
            return invalid(
                path,
                offset + 2,
                "target dictionary examples require Synodal source recension",
            );
        }
        if row[8] != "source" && row[8] != "evaluation" {
            return invalid(path, offset + 2, "unknown source/evaluation partition");
        }
    }
    Ok(())
}

pub(crate) fn validate_target(path: &Path, line: usize, value: &str) -> Result<()> {
    if value == TARGET {
        Ok(())
    } else {
        invalid(path, line, "target_recension must be synodal-russian")
    }
}

pub(crate) fn validate_word(path: &Path, line: usize, value: &str, label: &str) -> Result<()> {
    SynodalWord::parse(value)
        .map(|_| ())
        .map_err(|error| ExtractionError::InvalidRow {
            file: path.to_owned(),
            line,
            reason: format!("invalid {label}: {error}"),
        })
}

pub(crate) fn reject_forbidden_authority(path: &Path, line: usize, value: &str) -> Result<()> {
    let lower = value.to_lowercase();
    if lower.contains("slovowiki") {
        invalid(path, line, "Slovowiki is a forbidden linguistic authority")
    } else {
        Ok(())
    }
}

pub(crate) fn invalid<T>(path: &Path, line: usize, reason: &str) -> Result<T> {
    Err(ExtractionError::InvalidRow {
        file: path.to_owned(),
        line,
        reason: reason.into(),
    })
}
