use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use synodal_church_slavonic_core::{Animacy, GrammarCell, Number};
use unicode_normalization::UnicodeNormalization;

use super::*;

pub(crate) fn validate_lexemes(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        validate_target(path, line, &row[8])?;
        validate_word(path, line, &row[1], "lemma")?;
        if !row[4].is_empty() {
            validate_word(path, line, &row[4], "stem")?;
        }
        if !row[0].starts_with("synodal:") {
            return invalid(
                path,
                line,
                "target lexeme IDs must use the synodal namespace",
            );
        }
        if !matches!(
            row[2].as_str(),
            "adverb"
                | "preposition"
                | "conjunction"
                | "particle"
                | "interjection"
                | "proper-noun"
                | "noun"
                | "adjective"
                | "verb"
                | "pronoun"
                | "determiner"
                | "numeral"
                | "participle"
        ) {
            return invalid(path, line, "unknown lexeme part of speech");
        }
        let valid_class = matches!(
            (row[2].as_str(), row[3].as_str()),
            (_, "" | "exact")
                | (
                    "noun",
                    "first-hard-m"
                        | "inherited-first-hard-m"
                        | "first-hard-u-stem-m"
                        | "first-hard-in-ethnonym-m"
                        | "first-hard-ud-es-m"
                        | "first-hard-velar-m"
                        | "first-mixed-m"
                        | "first-mixed-ts-m"
                        | "first-hard-n"
                        | "first-soft-m"
                        | "first-soft-agent-tel-m"
                        | "first-soft-lord-m"
                        | "first-soft-j-m"
                        | "first-soft-ey-m"
                        | "first-soft-n"
                        | "first-soft-ishche-n"
                        | "first-soft-ie-n"
                        | "second-hard"
                        | "second-hard-velar"
                        | "second-soft"
                        | "second-soft-postvocalic-ancient-pl"
                        | "second-soft-m-ia"
                        | "second-soft-f-ia"
                        | "second-mixed"
                        | "third-f"
                        | "third-m"
                        | "fourth-neuter-en"
                        | "fourth-neuter-es"
                        | "fourth-neuter-es-alt-first"
                        | "fourth-neuter-es-paired-dual"
                        | "fourth-neuter-at"
                        | "fourth-feminine-er"
                        | "fourth-feminine-er-daughter"
                        | "fourth-feminine-ov"
                        | "fourth-feminine-ov-syncopating"
                        | "fourth-masculine-en"
                        | "fourth-masculine-en-day"
                        | "fourth-masculine-en-kamen"
                        | "indeclinable",
                )
                | (
                    "adjective",
                    "hard-short"
                        | "soft-short"
                        | "velar-short"
                        | "possessive-hard-short"
                        | "possessive-soft-short"
                        | "possessive-j-short"
                        | "possessive-in"
                        | "possessive-sk"
                        | "possessive-ii"
                )
                | (
                    "determiner",
                    "determiner-pronominal-hard"
                        | "determiner-ves-mixed"
                        | "determiner-vsyak-mixed"
                        | "determiner-full-sk"
                )
                | (
                    "numeral",
                    "numeral-cardinal-one"
                        | "numeral-cardinal-two"
                        | "numeral-cardinal-both"
                        | "numeral-cardinal-three"
                        | "numeral-cardinal-four"
                        | "numeral-cardinal-i-stem"
                        | "numeral-cardinal-ten"
                        | "numeral-cardinal-hundred"
                        | "numeral-cardinal-second-hard"
                        | "numeral-cardinal-second-mixed"
                        | "numeral-cardinal-first-hard-m"
                        | "numeral-cardinal-third-f"
                        | "ordinal-hard"
                        | "ordinal-ii"
                        | "numeral-collective-agreeing"
                        | "numeral-collective-governing-neuter"
                        | "numeral-collective-hard-plural"
                        | "numeral-multiplicative-hard"
                        | "numeral-multiplicative-soft"
                        | "numeral-fractional-hard"
                        | "numeral-fractional-first-u"
                        | "numeral-fractional-second-hard"
                        | "numeral-fractional-third-f"
                )
                | (
                    "pronoun",
                    "exact-complete-pronoun-table"
                        | "pronoun-personal-first"
                        | "pronoun-personal-second"
                        | "pronoun-reflexive"
                        | "pronoun-reflexive-clitic"
                        | "pronoun-third-person"
                        | "pronoun-third-person-demonstrative"
                        | "pronoun-relative-izhe"
                        | "pronoun-proximal-sei"
                        | "pronoun-soft"
                        | "pronoun-soft-i-alternating"
                        | "pronoun-hard"
                        | "pronoun-mixed-possessive"
                        | "pronoun-short-hard"
                        | "pronoun-short-ov-mixed"
                        | "pronoun-short-velar"
                        | "pronoun-quantity-velar"
                        | "pronoun-full-hard"
                        | "pronoun-full-soft"
                        | "pronoun-full-velar"
                        | "pronoun-interrogative-kii"
                        | "pronoun-interrogative-who"
                        | "pronoun-interrogative-what"
                        | "pronoun-indefinite-kii"
                        | "pronoun-indefinite-who"
                        | "pronoun-indefinite-what"
                        | "pronoun-negative-kii"
                        | "pronoun-negative-full-hard"
                        | "pronoun-negative-who"
                        | "pronoun-negative-what"
                        | "pronoun-kii-zhdo"
                        | "pronoun-negative-who-zhe"
                        | "pronoun-negative-what-zhe",
                )
                | (
                    "verb",
                    "first-unpalatalized" | "first-palatalized" | "second" | "archaic"
                )
        );
        if !valid_class {
            return invalid(path, line, "unknown class for lexeme part of speech");
        }
        if row[2] == "verb" && row[1].ends_with("сѧ") && row[4].ends_with("сѧ") {
            return invalid(
                path,
                line,
                "reflexive verb stems are stored without the enclitic -сѧ; the resolver attaches it",
            );
        }
        if !matches!(row[5].as_str(), "" | "masculine" | "feminine" | "neuter") {
            return invalid(path, line, "unknown lexical gender");
        }
        if !matches!(
            row[6].as_str(),
            "" | "unknown" | "imperfective" | "perfective" | "biaspectual"
        ) {
            return invalid(path, line, "unknown lexical aspect");
        }
        if row[7].is_empty() {
            return invalid(path, line, "a lexeme requires a source ID");
        }
    }
    Ok(())
}

pub(crate) fn validate_noun_restrictions(path: &Path, table: &Table) -> Result<()> {
    let mut lexeme_ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !lexeme_ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        if !matches!(
            row[1].as_str(),
            "all"
                | "singular-only"
                | "dual-only"
                | "plural-only"
                | "singular-and-dual"
                | "singular-and-plural"
                | "dual-and-plural"
        ) {
            return invalid(path, offset + 2, "unknown noun number inventory");
        }
        if !matches!(row[2].as_str(), "any" | "animate" | "inanimate") {
            return invalid(path, offset + 2, "unknown noun animacy inventory");
        }
        if row[3].is_empty() {
            return invalid(
                path,
                offset + 2,
                "a noun restriction requires normative evidence",
            );
        }
        validate_target(path, offset + 2, &row[4])?;
    }
    Ok(())
}

pub(crate) fn validate_noun_restriction_lexemes(
    path: &Path,
    restrictions: &Table,
    lexemes: &Table,
) -> Result<()> {
    let lexemes_by_id = lexemes
        .rows
        .iter()
        .map(|row| (row[0].as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for (offset, restriction) in restrictions.rows.iter().enumerate() {
        let Some(lexeme) = lexemes_by_id.get(restriction[0].as_str()) else {
            return invalid(
                path,
                offset + 2,
                "noun restriction references an unknown lexeme",
            );
        };
        if lexeme[2] != "noun" {
            return invalid(
                path,
                offset + 2,
                "noun restriction references a non-noun lexeme",
            );
        }
        if restriction[4] != lexeme[8] {
            return invalid(
                path,
                offset + 2,
                "noun restriction and lexeme target recensions disagree",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_noun_restriction_exact_forms(
    _restriction_path: &Path,
    restrictions: &Table,
    exact_path: &Path,
    exact_forms: &Table,
) -> Result<()> {
    let restrictions_by_lexeme = restrictions
        .rows
        .iter()
        .map(|row| (row[0].as_str(), (row[1].as_str(), row[2].as_str())))
        .collect::<BTreeMap<_, _>>();
    for (offset, row) in exact_forms.rows.iter().enumerate() {
        let Some((number_inventory, animacy_inventory)) =
            restrictions_by_lexeme.get(row[0].as_str()).copied()
        else {
            continue;
        };
        let cell = row[1]
            .parse::<GrammarCell>()
            .map_err(|error| ExtractionError::InvalidRow {
                file: exact_path.to_owned(),
                line: offset + 2,
                reason: error.to_string(),
            })?;
        let cell = match cell {
            GrammarCell::Noun(cell) => cell,
            GrammarCell::LexicalForm => {
                return invalid(
                    exact_path,
                    offset + 2,
                    "restricted nouns require typed exact noun cells",
                );
            }
            _ => continue,
        };
        let number_allowed = matches!(
            (number_inventory, cell.number),
            ("all", _)
                | ("singular-only", Number::Singular)
                | ("dual-only", Number::Dual)
                | ("plural-only", Number::Plural)
                | ("singular-and-dual", Number::Singular | Number::Dual)
                | ("singular-and-plural", Number::Singular | Number::Plural)
                | ("dual-and-plural", Number::Dual | Number::Plural)
        );
        let wildcard_animacy = row[1].ends_with(":any");
        let animacy_allowed = matches!(
            (animacy_inventory, cell.animacy, wildcard_animacy),
            ("any", _, _)
                | ("animate", Animacy::Animate, false)
                | ("inanimate", Animacy::Inanimate, false)
        );
        if !number_allowed || !animacy_allowed {
            return invalid(
                exact_path,
                offset + 2,
                "exact noun cell is outside the lexeme's licensed number or animacy inventory",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_principal_parts(path: &Path, table: &Table) -> Result<()> {
    for (offset, row) in table.rows.iter().enumerate() {
        validate_target(path, offset + 2, &row[5])?;
        validate_word(path, offset + 2, &row[2], "principal part")?;
        if row[1].is_empty() || row[4].is_empty() {
            return invalid(
                path,
                offset + 2,
                "principal parts require a system and normative evidence",
            );
        }
        if row[1] == "comparative-stem"
            && !matches!(
                row[3].as_str(),
                "ancient-hard" | "ancient-soft" | "later-yat" | "later-ai"
            )
        {
            return invalid(path, offset + 2, "unknown typed comparison formation");
        }
        if row[1] == "short-masculine-stem"
            && !matches!(
                row[3].as_str(),
                "double-n-reduction" | "mobile-e-insertion" | "mobile-o-insertion"
            )
        {
            return invalid(path, offset + 2, "unknown typed short-masculine formation");
        }
        if row[1] == "verbal-noun-ie-platform" && row[3] != "past-passive-ie" {
            return invalid(
                path,
                offset + 2,
                "verbal-noun -їе platforms require the typed past-passive-ie formation",
            );
        }
        if row[1] == "verbal-noun-ie-platform" && !matches!(row[2].chars().last(), Some('н' | 'т'))
        {
            return invalid(
                path,
                offset + 2,
                "a verbal-noun -їе platform must end in н or т",
            );
        }
        if row[1] == "l-participle-masculine-singular-stem"
            && !table
                .rows
                .iter()
                .any(|candidate| candidate[0] == row[0] && candidate[1] == "l-participle-stem")
        {
            return invalid(
                path,
                offset + 2,
                "an l-participle masculine-singular stem requires the general stem",
            );
        }
        if row[1].ends_with("active-participle-short-stem") {
            let valid = matches!(
                row[3].as_str(),
                "hard:present-first-unpalatalized"
                    | "soft:present-first-unpalatalized"
                    | "hard:present-first-palatalized"
                    | "soft:present-first-palatalized"
                    | "hard:present-second"
                    | "soft:present-second"
                    | "hard:present-after-sibilant"
                    | "soft:present-after-sibilant"
                    | "hard:past-consonant"
                    | "soft:past-consonant"
                    | "hard:past-vowel"
                    | "soft:past-vowel"
                    | "hard:past-iotated"
                    | "soft:past-iotated"
            );
            if !valid {
                return invalid(
                    path,
                    offset + 2,
                    "active short participles require a class and closed typed formation",
                );
            }
        }
    }
    let future_systems = [
        "future-stem",
        "future-first-singular",
        "future-third-plural",
    ];
    let future_lexemes = table
        .rows
        .iter()
        .filter(|row| future_systems.contains(&row[1].as_str()))
        .map(|row| row[0].as_str())
        .collect::<BTreeSet<_>>();
    for lexeme_id in future_lexemes {
        let supplied = future_systems
            .iter()
            .filter(|system| {
                table
                    .rows
                    .iter()
                    .any(|row| row[0] == lexeme_id && row[1] == **system)
            })
            .count();
        if supplied != future_systems.len() {
            return invalid(
                path,
                1,
                "independent future stem, first singular, and third plural must be supplied together",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_exact_forms(path: &Path, table: &Table, lexemes: &Table) -> Result<()> {
    let mut runtime_keys = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !runtime_keys.insert((
            row[0].clone(),
            row[1].clone(),
            row[2].clone(),
            row[3].clone(),
        )) {
            return invalid(
                path,
                offset + 2,
                "duplicate lexeme/cell/expanded/printed exact-form tuple",
            );
        }
        validate_grammar_cell(path, offset + 2, &row[1])?;
        validate_cell_lexeme_pos(path, offset + 2, &row[0], &row[1], lexemes)?;
        validate_target(path, offset + 2, &row[6])?;
        validate_word(path, offset + 2, &row[2], "expanded form")?;
        validate_word(path, offset + 2, &row[3], "printed form")?;
        if !matches!(
            row[5].as_str(),
            "normative-table" | "normative-variant" | "synodal-attestation"
        ) {
            return invalid(path, offset + 2, "unknown exact-form source kind");
        }
        if row[5] == "synodal-attestation" && row[4].is_empty() {
            return invalid(
                path,
                offset + 2,
                "attestation rows require passage evidence",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_alignments(path: &Path, table: &Table) -> Result<()> {
    let mut ids = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        if !ids.insert(row[0].clone()) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: row[0].clone(),
            });
        }
        if !row[2].starts_with("synodal:") {
            return invalid(
                path,
                offset + 2,
                "mapping target must be a Synodal lexeme ID",
            );
        }
        if !row[1].starts_with("ocs:") {
            return invalid(
                path,
                offset + 2,
                "mapping source must be a stable OCS lexeme ID",
            );
        }
        if !matches!(
            row[4].as_str(),
            "reviewed" | "automatically-validated" | "exploratory" | "rejected"
        ) {
            return invalid(path, offset + 2, "unknown mapping review status");
        }
        let confidence = row[7]
            .parse::<u16>()
            .map_err(|_| ExtractionError::InvalidRow {
                file: path.to_owned(),
                line: offset + 2,
                reason: "confidence must be integer basis points".into(),
            })?;
        if confidence > 10_000 {
            return invalid(
                path,
                offset + 2,
                "confidence cannot exceed 10000 basis points",
            );
        }
        if row[4] == "rejected" && confidence != 0 {
            return invalid(
                path,
                offset + 2,
                "rejected mappings must have zero confidence",
            );
        }
        if row[4] != "rejected" && (confidence == 0 || row[8].is_empty() || row[9].is_empty()) {
            return invalid(
                path,
                offset + 2,
                "admitted mappings require confidence, evidence, and explicit transformations",
            );
        }
    }
    Ok(())
}

pub(crate) fn validate_abbreviations(path: &Path, table: &Table, lexemes: &Table) -> Result<()> {
    for (offset, row) in table.rows.iter().enumerate() {
        validate_grammar_cell(path, offset + 2, &row[2])?;
        validate_cell_lexeme_pos(path, offset + 2, &row[0], &row[2], lexemes)?;
        validate_target(path, offset + 2, &row[12])?;
        validate_word(path, offset + 2, &row[3], "expanded abbreviation")?;
        validate_word(path, offset + 2, &row[4], "printed abbreviation")?;
        if row[1].is_empty() {
            return invalid(
                path,
                offset + 2,
                "abbreviation rows require a semantic sense ID",
            );
        }
        if row[2].is_empty()
            || row[5].is_empty()
            || row[6].is_empty()
            || row[8].is_empty()
            || row[9].is_empty()
            || row[10].is_empty()
            || row[11] != TARGET
        {
            return invalid(
                path,
                offset + 2,
                "abbreviations require a cell, rule, evidence, marks, context, ambiguity, and Synodal source recension",
            );
        }
        if !matches!(row[7].as_str(), "true" | "false") {
            return invalid(path, offset + 2, "reversible must be true or false");
        }
    }
    Ok(())
}

pub(crate) fn validate_abbreviation_families(
    path: &Path,
    table: &Table,
    abbreviations: &Table,
    lexemes: &Table,
) -> Result<()> {
    let mut patterns = BTreeSet::new();
    let mut metadata = BTreeMap::<(String, String), Vec<String>>::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        if !lexemes.rows.iter().any(|lexeme| lexeme[0] == row[0]) {
            return invalid(
                path,
                line,
                &format!("abbreviation family references unknown lexeme {}", row[0]),
            );
        }
        validate_word(path, line, &row[2], "expanded abbreviation-family prefix")?;
        validate_word(path, line, &row[3], "printed abbreviation-family prefix")?;
        validate_target(path, line, &row[11])?;
        if row[1].is_empty()
            || row[2].is_empty()
            || row[3].is_empty()
            || row[4].is_empty()
            || row[5].is_empty()
            || row[7].is_empty()
            || row[8].is_empty()
            || row[9].is_empty()
            || row[10] != TARGET
        {
            return invalid(
                path,
                line,
                "abbreviation families require identity, nonempty prefixes, rule, evidence, marks, context, ambiguity, and Synodal recensions",
            );
        }
        if !matches!(row[6].as_str(), "true" | "false") {
            return invalid(path, line, "reversible must be true or false");
        }
        validate_abbreviation_family_marks(path, line, &row[3], &row[7])?;
        if !patterns.insert((row[0].clone(), row[1].clone(), row[2].clone())) {
            return Err(ExtractionError::DuplicateId {
                file: path.to_owned(),
                id: format!("{}:{}:{}", row[0], row[1], row[2]),
            });
        }
        let key = (row[0].clone(), row[1].clone());
        let family_metadata = row[4..].to_vec();
        if let Some(first) = metadata.get(&key) {
            if first != &family_metadata {
                return invalid(
                    path,
                    line,
                    "all patterns in one abbreviation family must share rule and review metadata",
                );
            }
        } else {
            metadata.insert(key, family_metadata);
        }
    }

    for ((lexeme_id, sense_id), _) in metadata {
        let family_patterns = table
            .rows
            .iter()
            .filter(|row| row[0] == lexeme_id && row[1] == sense_id)
            .collect::<Vec<_>>();
        let exact = abbreviations
            .rows
            .iter()
            .filter(|row| row[0] == lexeme_id && row[1] == sense_id)
            .collect::<Vec<_>>();
        if exact.is_empty() {
            return invalid(
                path,
                1,
                &format!(
                    "abbreviation family {lexeme_id}:{sense_id} requires at least one reviewed exact cell"
                ),
            );
        }
        for exact_row in exact {
            let expected = normalize_abbreviation_family_shape(&exact_row[4]);
            let covered = family_patterns.iter().any(|pattern| {
                exact_row[3]
                    .strip_prefix(&pattern[2])
                    .map(|suffix| format!("{}{suffix}", pattern[3]))
                    .is_some_and(|generated| {
                        normalize_abbreviation_family_shape(&generated) == expected
                    })
            });
            if !covered {
                return invalid(
                    path,
                    1,
                    &format!(
                        "abbreviation family {lexeme_id}:{sense_id} does not reproduce reviewed exact shape {:?} -> {:?}",
                        exact_row[3], exact_row[4]
                    ),
                );
            }
        }
        for pattern in family_patterns {
            if !abbreviations.rows.iter().any(|exact_row| {
                exact_row[0] == lexeme_id
                    && exact_row[1] == sense_id
                    && abbreviation_pattern_covers(&exact_row[3], &exact_row[4], pattern)
            }) {
                return invalid(
                    path,
                    1,
                    &format!(
                        "abbreviation-family pattern {:?} -> {:?} has no reviewed exact witness",
                        pattern[2], pattern[3]
                    ),
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_abbreviation_family_marks(
    path: &Path,
    line: usize,
    printed_prefix: &str,
    required_marks: &str,
) -> Result<()> {
    let characters = printed_prefix.nfd().collect::<Vec<_>>();
    for mark in required_marks.split(',') {
        let present = match mark {
            "titlo" => characters.iter().any(|character| {
                *character == '\u{0483}' || ('\u{2de0}'..='\u{2dff}').contains(character)
            }),
            "initial-breathing" => characters.contains(&'\u{0486}'),
            "pokrytie" => characters.contains(&'\u{0487}'),
            "superscript-s" => characters.contains(&'\u{2ded}'),
            "superscript-g" => characters.contains(&'\u{2de2}'),
            "superscript-o" => characters.contains(&'\u{2dea}'),
            "superscript-d" => characters.contains(&'\u{2de3}'),
            _ => return invalid(path, line, "unknown required abbreviation-family mark"),
        };
        if !present {
            return invalid(
                path,
                line,
                &format!("printed family prefix is missing required mark {mark:?}"),
            );
        }
    }
    Ok(())
}

pub(crate) fn abbreviation_pattern_covers(
    expanded: &str,
    printed: &str,
    pattern: &[String],
) -> bool {
    expanded
        .strip_prefix(&pattern[2])
        .map(|suffix| format!("{}{suffix}", pattern[3]))
        .is_some_and(|generated| {
            normalize_abbreviation_family_shape(&generated)
                == normalize_abbreviation_family_shape(printed)
        })
}

pub(crate) fn normalize_abbreviation_family_shape(value: &str) -> String {
    value
        .nfd()
        .filter(|character| !matches!(character, '\u{0300}' | '\u{0301}' | '\u{0308}' | '\u{0311}'))
        .flat_map(char::to_lowercase)
        .map(|character| match character {
            'ѡ' | 'ѻ' | 'ꙍ' => 'о',
            'і' | 'ї' => 'и',
            'є' => 'е',
            'ꙋ' => 'у',
            'ꙗ' | 'я' => 'ѧ',
            other => other,
        })
        .nfc()
        .collect()
}

pub(crate) fn validate_abbreviation_inventory(
    path: &Path,
    table: &Table,
    families: &Table,
) -> Result<()> {
    if table.rows.len() != 48 {
        return invalid(
            path,
            1,
            "Alypy §3.c abbreviation inventory must classify all 48 named entries",
        );
    }
    let mut orders = BTreeSet::new();
    for (offset, row) in table.rows.iter().enumerate() {
        let line = offset + 2;
        let order = row[0]
            .parse::<u8>()
            .map_err(|_| ExtractionError::InvalidRow {
                file: path.to_owned(),
                line,
                reason: "source_order must be an integer from 1 through 48".into(),
            })?;
        if !(1..=48).contains(&order) || !orders.insert(order) {
            return invalid(path, line, "source_order must uniquely cover 1 through 48");
        }
        validate_word(path, line, &row[1], "source abbreviation head")?;
        validate_word(path, line, &row[2], "source abbreviation expansion")?;
        validate_target(path, line, &row[10])?;
        if row[3].is_empty() || row[8].is_empty() || row[9].is_empty() {
            return invalid(
                path,
                line,
                "every abbreviation inventory row requires semantic scope, evidence, and a review note",
            );
        }
        match row[4].as_str() {
            "productive-family" => {
                if row[5].is_empty() || row[6].is_empty() || row[7].is_empty() {
                    return invalid(
                        path,
                        line,
                        "productive abbreviation decisions require lexeme, sense, and rule IDs",
                    );
                }
                if !families.rows.iter().any(|family| {
                    family[0] == row[5]
                        && family[1] == row[6]
                        && family[4] == row[7]
                        && abbreviation_inventory_pattern_covers(&row[2], &row[1], family)
                }) {
                    return invalid(
                        path,
                        line,
                        "productive abbreviation decision does not structurally match a generated family",
                    );
                }
            }
            "implementation-missing" => {
                if row[5..8].iter().any(|value| !value.is_empty()) {
                    return invalid(
                        path,
                        line,
                        "missing abbreviation decisions cannot claim runtime IDs",
                    );
                }
            }
            _ => {
                return invalid(
                    path,
                    line,
                    "abbreviation decision must be productive-family or implementation-missing",
                );
            }
        }
    }
    if orders != (1_u8..=48).collect() {
        return invalid(
            path,
            1,
            "source_order does not exhaustively cover 1 through 48",
        );
    }
    Ok(())
}

pub(crate) fn abbreviation_inventory_pattern_covers(
    expanded_head: &str,
    printed_head: &str,
    family: &[String],
) -> bool {
    let expanded = normalize_abbreviation_family_shape(expanded_head);
    let expanded_prefix = normalize_abbreviation_family_shape(&family[2]);
    expanded
        .strip_prefix(&expanded_prefix)
        .map(|suffix| {
            format!(
                "{}{suffix}",
                normalize_abbreviation_family_shape(&family[3])
            )
        })
        .is_some_and(|generated| generated == normalize_abbreviation_family_shape(printed_head))
}
