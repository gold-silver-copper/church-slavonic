#[allow(unused_imports)]
use super::*;

/// Enumerates cells that reverse analysis may attempt for a part of speech.
/// Unsupported cells still fail through the facade's typed error contract.
#[must_use]
pub fn candidate_cells(part_of_speech: PartOfSpeech) -> Vec<GrammarCell> {
    const OPTIONAL_GENDERS: [Option<Gender>; 4] = [
        None,
        Some(Gender::Masculine),
        Some(Gender::Feminine),
        Some(Gender::Neuter),
    ];
    const OPTIONAL_PERSONS: [Option<Person>; 4] = [
        None,
        Some(Person::First),
        Some(Person::Second),
        Some(Person::Third),
    ];
    let mut cells = match part_of_speech {
        PartOfSpeech::Adverb
        | PartOfSpeech::Preposition
        | PartOfSpeech::Conjunction
        | PartOfSpeech::Particle
        | PartOfSpeech::Interjection => {
            vec![GrammarCell::Indeclinable]
        }
        PartOfSpeech::Noun | PartOfSpeech::ProperNoun => core::NounCell::inventory(&Animacy::ALL)
            .into_iter()
            .map(GrammarCell::Noun)
            .collect(),
        PartOfSpeech::Adjective => {
            AdjectiveCell::inventory(&AdjectiveForm::ALL, &Comparison::ALL, |_| &Animacy::ALL)
                .into_iter()
                .map(GrammarCell::Adjective)
                .collect()
        }
        PartOfSpeech::Verb => verb_cells(),
        PartOfSpeech::Pronoun => PronounCell::inventory(
            &OPTIONAL_GENDERS
                .into_iter()
                .flat_map(|gender| {
                    OPTIONAL_PERSONS
                        .into_iter()
                        .map(move |person| (gender, person))
                })
                .collect::<Vec<_>>(),
        )
        .into_iter()
        .map(GrammarCell::Pronoun)
        .collect(),
        PartOfSpeech::Numeral => NumeralCell::inventory(&NumeralKind::ALL, &OPTIONAL_GENDERS)
            .into_iter()
            .map(GrammarCell::Numeral)
            .collect(),
        PartOfSpeech::Determiner => {
            AdjectiveCell::inventory(&AdjectiveForm::ALL, &Comparison::ALL, |_| &Animacy::ALL)
                .into_iter()
                .map(GrammarCell::Determiner)
                .collect()
        }
        PartOfSpeech::Participle => Vec::new(),
    };
    cells.push(GrammarCell::LexicalForm);
    cells
}

/// Returns the exact-compatible and productively supported cells that can
/// contribute to reverse analysis for one stable lexeme.
///
/// This is deliberately narrower than [`candidate_cells`], which remains the
/// exhaustive typed inventory used by the independent correctness oracle.
pub fn analysis_cells_by_id(id: &LexemeId, inflector: Inflector) -> Result<Vec<GrammarCell>> {
    let lexeme = morphology::advanced::lookup_by_id(id)?;
    analysis_cells_for_lexeme(&lexeme, inflector)
}

pub(crate) fn analysis_cells_for_lexeme(
    lexeme: &LexemeSummary,
    inflector: Inflector,
) -> Result<Vec<GrammarCell>> {
    let metadata = lexical_metadata(lexeme.id())?;
    let capabilities = capabilities_by_id(lexeme.id(), inflector)?;
    let exact_keys: BTreeSet<&str> = metadata
        .exact_forms
        .iter()
        .map(|form| form.cell.as_str())
        .collect();
    let mut cells = BTreeSet::new();
    for cell in candidate_cells(lexeme.part_of_speech()) {
        if grammar_cell_registry_keys(cell)
            .iter()
            .any(|key| exact_keys.contains(key.as_str()))
            || productive_cell_is_supported(cell, &metadata, &capabilities)
        {
            cells.insert(cell);
        }
    }
    Ok(cells.into_iter().collect())
}

pub(crate) fn productive_cell_is_supported(
    cell: GrammarCell,
    metadata: &LexicalMetadataSummary,
    capabilities: &morphology::Capabilities,
) -> bool {
    let principal_part = |system: &str| {
        metadata
            .principal_parts
            .iter()
            .find(|part| part.system == system)
    };
    match cell {
        GrammarCell::Noun(cell) => {
            capabilities.productive_noun
                && metadata
                    .noun_restriction
                    .as_ref()
                    .is_none_or(|restriction| {
                        number_is_licensed(&restriction.number_inventory, cell.number)
                            && animacy_is_licensed(&restriction.animacy_inventory, cell.animacy)
                    })
        }
        GrammarCell::Adjective(cell) => {
            capabilities.productive_adjective
                && adjectival_cell_is_supported(
                    cell,
                    metadata.class.as_deref(),
                    principal_part("comparative-stem").is_some(),
                )
        }
        GrammarCell::Determiner(cell) => {
            capabilities.productive_determiner
                && determiner_cell_is_supported(cell, metadata.class.as_deref())
        }
        GrammarCell::Pronoun(cell) => {
            capabilities.productive_pronoun
                && pronoun_cell_is_supported(cell, metadata.class.as_deref())
        }
        GrammarCell::Numeral(cell) => {
            capabilities.productive_numeral
                && numeral_cell_is_supported(cell, metadata.class.as_deref())
        }
        GrammarCell::FiniteVerb(cell) if productive_verb_class(metadata.class.as_deref()) => {
            match cell.tense {
                FiniteTense::Present => {
                    metadata.stem.is_some()
                        && (["present-first-singular", "present-third-plural"]
                            .into_iter()
                            .all(|system| principal_part(system).is_some())
                            || (metadata.aspect.as_deref() == Some("perfective")
                                && [
                                    "future-stem",
                                    "future-first-singular",
                                    "future-third-plural",
                                ]
                                .into_iter()
                                .all(|system| principal_part(system).is_some())))
                }
                FiniteTense::Future => {
                    metadata.aspect.as_deref() == Some("perfective")
                        && ((metadata.stem.is_some()
                            && ["present-first-singular", "present-third-plural"]
                                .into_iter()
                                .all(|system| principal_part(system).is_some()))
                            || [
                                "future-stem",
                                "future-first-singular",
                                "future-third-plural",
                            ]
                            .into_iter()
                            .all(|system| principal_part(system).is_some()))
                }
                FiniteTense::Imperfect => {
                    matches!(
                        metadata.aspect.as_deref(),
                        Some("imperfective" | "biaspectual")
                    ) && principal_part("imperfect-stem")
                        .and_then(|part| part.formation.as_deref())
                        .is_some_and(|formation| formation != "irregular")
                }
                FiniteTense::Aorist => principal_part("aorist-stem")
                    .and_then(|part| part.formation.as_deref())
                    .is_some_and(|formation| formation != "irregular"),
                FiniteTense::Past => false,
            }
        }
        GrammarCell::Imperative(_) if productive_verb_class(metadata.class.as_deref()) => {
            principal_part("imperative-stem")
                .and_then(|part| part.formation.as_deref())
                .is_some_and(|formation| formation != "irregular")
        }
        GrammarCell::Infinitive => productive_verb_class(metadata.class.as_deref()),
        GrammarCell::LParticiple(_) if productive_verb_class(metadata.class.as_deref()) => {
            principal_part("l-participle-stem").is_some()
        }
        GrammarCell::Participle(cell) if productive_verb_class(metadata.class.as_deref()) => {
            if cell.agreement.comparison != Comparison::Positive
                || (cell.tense == ParticipleTense::Present
                    && !matches!(
                        metadata.aspect.as_deref(),
                        Some("imperfective" | "biaspectual")
                    ))
            {
                return false;
            }
            let system = format!(
                "{}-{}-participle-{}-stem",
                cell.tense.code(),
                cell.voice.code(),
                cell.agreement.form.code()
            );
            let Some(part) = principal_part(&system) else {
                return false;
            };
            cell.voice != ParticipleVoice::Active
                || cell.agreement.form != AdjectiveForm::Short
                || part.formation.is_some()
        }
        GrammarCell::VerbalNoun(_) if productive_verb_class(metadata.class.as_deref()) => {
            capabilities.verbal_noun
        }
        GrammarCell::LexicalForm
        | GrammarCell::Indeclinable
        | GrammarCell::Supine
        | GrammarCell::VerbalNoun(_)
        | GrammarCell::FiniteVerb(_)
        | GrammarCell::Imperative(_)
        | GrammarCell::LParticiple(_)
        | GrammarCell::Participle(_) => false,
    }
}

pub(crate) fn pronoun_cell_is_supported(cell: PronounCell, class: Option<&str>) -> bool {
    if cell.case == synodal_church_slavonic::Case::Vocative {
        return false;
    }
    match class {
        Some("pronoun-personal-first") => {
            cell.gender.is_none() && cell.person == Some(Person::First)
        }
        Some("pronoun-personal-second") => {
            cell.gender.is_none() && cell.person == Some(Person::Second)
        }
        Some("pronoun-reflexive") => {
            cell.gender.is_none()
                && cell.person.is_none()
                && cell.number == Number::Singular
                && cell.case != synodal_church_slavonic::Case::Nominative
        }
        Some("pronoun-reflexive-clitic") => {
            cell.gender.is_none()
                && cell.person.is_none()
                && cell.number == Number::Singular
                && matches!(cell.case, Case::Dative | Case::Accusative)
        }
        Some("pronoun-third-person") => cell.gender.is_some() && cell.person == Some(Person::Third),
        Some("pronoun-third-person-demonstrative") => {
            cell.gender.is_some() && matches!(cell.person, None | Some(Person::Third))
        }
        Some("pronoun-relative-izhe")
        | Some(
            "pronoun-proximal-sei"
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
            | "pronoun-indefinite-kii"
            | "pronoun-negative-kii"
            | "pronoun-negative-full-hard"
            | "pronoun-kii-zhdo",
        ) => cell.gender.is_some() && cell.person.is_none(),
        Some(
            "pronoun-interrogative-who"
            | "pronoun-interrogative-what"
            | "pronoun-indefinite-who"
            | "pronoun-indefinite-what"
            | "pronoun-negative-who"
            | "pronoun-negative-what"
            | "pronoun-negative-who-zhe"
            | "pronoun-negative-what-zhe",
        ) => cell.gender.is_none() && cell.person.is_none() && cell.number == Number::Singular,
        _ => false,
    }
}

pub(crate) fn adjectival_cell_is_supported(
    cell: AdjectiveCell,
    class: Option<&str>,
    has_comparative_stem: bool,
) -> bool {
    match class {
        Some("possessive-hard-short" | "possessive-soft-short" | "possessive-j-short") => {
            return cell.comparison == Comparison::Positive && cell.form == AdjectiveForm::Short;
        }
        Some("possessive-in" | "possessive-sk" | "possessive-ii") => {
            return cell.comparison == Comparison::Positive;
        }
        _ => {}
    }
    match (cell.comparison, cell.form) {
        (Comparison::Positive, _) => true,
        (Comparison::Comparative, _) => has_comparative_stem,
        (Comparison::Superlative, AdjectiveForm::Long) => has_comparative_stem,
        (Comparison::Superlative, AdjectiveForm::Short) => {
            has_comparative_stem && cell.case == Case::Nominative
        }
    }
}

pub(crate) fn determiner_cell_is_supported(cell: AdjectiveCell, class: Option<&str>) -> bool {
    if cell.comparison != Comparison::Positive {
        return false;
    }
    match class {
        Some("determiner-pronominal-hard") => true,
        Some("determiner-ves-mixed") => {
            cell.number != Number::Dual && cell.form == AdjectiveForm::Short
        }
        Some("determiner-vsyak-mixed") => cell.number != Number::Dual,
        Some("determiner-full-sk") => cell.form == AdjectiveForm::Long,
        _ => false,
    }
}

pub(crate) fn numeral_cell_is_supported(cell: NumeralCell, class: Option<&str>) -> bool {
    let nonvocative = cell.case != Case::Vocative;
    match class {
        Some("numeral-cardinal-one") => {
            cell.kind == NumeralKind::Cardinal
                && cell.number == Number::Singular
                && cell.gender.is_some()
                && nonvocative
        }
        Some("numeral-cardinal-two" | "numeral-cardinal-both") => {
            cell.kind == NumeralKind::Cardinal
                && cell.number == Number::Dual
                && cell.gender.is_some()
                && nonvocative
        }
        Some("numeral-cardinal-three" | "numeral-cardinal-four") => {
            cell.kind == NumeralKind::Cardinal
                && cell.number == Number::Plural
                && cell.gender.is_some()
                && nonvocative
        }
        Some("numeral-cardinal-i-stem") => {
            cell.kind == NumeralKind::Cardinal
                && cell.gender.is_none()
                && nonvocative
                && (cell.number == Number::Singular
                    || cell.number == Number::Plural
                        && matches!(cell.case, Case::Genitive | Case::Dative | Case::Locative))
        }
        Some(
            "numeral-cardinal-ten"
            | "numeral-cardinal-hundred"
            | "numeral-cardinal-second-hard"
            | "numeral-cardinal-second-mixed"
            | "numeral-cardinal-first-hard-m"
            | "numeral-cardinal-third-f",
        ) => cell.kind == NumeralKind::Cardinal && cell.gender.is_none() && nonvocative,
        Some("ordinal-hard" | "ordinal-ii") => {
            cell.kind == NumeralKind::Ordinal && cell.gender.is_some()
        }
        Some("numeral-collective-agreeing" | "numeral-collective-hard-plural") => {
            cell.kind == NumeralKind::Collective
                && cell.number == Number::Plural
                && cell.gender.is_some()
        }
        Some("numeral-collective-governing-neuter") => {
            cell.kind == NumeralKind::Collective
                && cell.number == Number::Singular
                && cell.gender == Some(Gender::Neuter)
                && nonvocative
        }
        Some("numeral-multiplicative-hard" | "numeral-multiplicative-soft") => {
            cell.kind == NumeralKind::Multiplicative && cell.gender.is_some()
        }
        Some("numeral-fractional-hard") => {
            cell.kind == NumeralKind::Fractional && cell.gender.is_some()
        }
        Some(
            "numeral-fractional-first-u"
            | "numeral-fractional-second-hard"
            | "numeral-fractional-third-f",
        ) => cell.kind == NumeralKind::Fractional && cell.gender.is_none() && nonvocative,
        _ => false,
    }
}

pub(crate) fn productive_verb_class(class: Option<&str>) -> bool {
    matches!(
        class,
        Some("first-unpalatalized" | "first-palatalized" | "second" | "archaic")
    )
}

pub(crate) fn number_is_licensed(inventory: &str, number: Number) -> bool {
    match inventory {
        "singular-only" => number == Number::Singular,
        "dual-only" => number == Number::Dual,
        "plural-only" => number == Number::Plural,
        "singular-and-dual" => matches!(number, Number::Singular | Number::Dual),
        "singular-and-plural" => matches!(number, Number::Singular | Number::Plural),
        "dual-and-plural" => matches!(number, Number::Dual | Number::Plural),
        _ => true,
    }
}

pub(crate) fn animacy_is_licensed(inventory: &str, animacy: Animacy) -> bool {
    match inventory {
        "animate" => animacy == Animacy::Animate,
        "inanimate" => animacy == Animacy::Inanimate,
        _ => true,
    }
}

pub(crate) fn verb_cells() -> Vec<GrammarCell> {
    let mut cells: Vec<GrammarCell> = FiniteVerbCell::inventory(&FiniteTense::ALL)
        .into_iter()
        .map(GrammarCell::FiniteVerb)
        .collect();
    cells.push(GrammarCell::Infinitive);
    cells.push(GrammarCell::Supine);
    for number in Number::ALL {
        for gender in Gender::ALL {
            cells.push(GrammarCell::LParticiple(LParticipleCell { gender, number }));
        }
        for person in Person::ALL {
            cells.push(GrammarCell::Imperative(ImperativeCell { person, number }));
        }
    }
    let agreements = AdjectiveCell::inventory(&AdjectiveForm::ALL, &[Comparison::Positive], |_| {
        &Animacy::ALL
    });
    cells.extend(
        ParticipleCell::inventory(&ParticipleTense::ALL, &ParticipleVoice::ALL, &agreements)
            .into_iter()
            .map(GrammarCell::Participle),
    );
    cells.extend(
        core::NounCell::inventory(&Animacy::ALL)
            .into_iter()
            .map(GrammarCell::VerbalNoun),
    );
    cells
}

pub(crate) fn split_list(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(str::to_owned).collect()
    }
}

pub(crate) fn fuzzy_score(query: &str, candidate: &str) -> Option<u16> {
    let candidate_word = candidate
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .min_by_key(|word| levenshtein(query, word))
        .unwrap_or(candidate);
    let distance = levenshtein(query, candidate_word);
    let maximum = query.chars().count().max(candidate_word.chars().count());
    let allowed = 2_usize.max(maximum / 4);
    (distance <= allowed).then(|| {
        let penalty = (distance.saturating_mul(700)).min(3_000) as u16;
        7_000_u16.saturating_sub(penalty)
    })
}

pub(crate) fn levenshtein(left: &str, right: &str) -> usize {
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (left_index, left_character) in left.chars().enumerate() {
        let mut current = Vec::with_capacity(right.len() + 1);
        current.push(left_index + 1);
        for (right_index, right_character) in right.iter().enumerate() {
            current.push(
                (current[right_index] + 1)
                    .min(previous[right_index + 1] + 1)
                    .min(previous[right_index] + usize::from(left_character != *right_character)),
            );
        }
        previous = current;
    }
    previous[right.len()]
}
