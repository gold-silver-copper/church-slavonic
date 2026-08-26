use super::*;

pub(crate) fn noun_lexeme(id: &LexemeId) -> Result<NounLexeme> {
    let row = require_pos(id, PartOfSpeech::Noun)?;
    let restriction = NOUN_RESTRICTIONS
        .iter()
        .find(|restriction| restriction.0[0] == id.as_str());
    let number_inventory = restriction.map_or(Ok(NounNumberInventory::All), |restriction| {
        parse_noun_number_inventory(restriction.0[1])
    })?;
    let animacy_inventory = restriction.map_or(Ok(NounAnimacyInventory::All), |restriction| {
        parse_noun_animacy_inventory(restriction.0[2])
    })?;
    Ok(NounLexeme {
        lemma: SynodalWord::parse(row.0[1])?,
        stem: SynodalWord::parse(row.0[4])?,
        gender: parse_gender(row.0[5])?,
        declension: match row.0[3] {
            "first-hard-m" | "inherited-first-hard-m" => NounDeclension::FirstHardMasculine,
            "first-hard-u-stem-m" => NounDeclension::FirstHardMasculineUStem,
            "first-hard-in-ethnonym-m" => NounDeclension::FirstHardMasculineInEthnonym,
            "first-hard-ud-es-m" => NounDeclension::FirstHardMasculineUdEs,
            "first-hard-velar-m" => NounDeclension::FirstHardVelarMasculine,
            "first-mixed-m" => NounDeclension::FirstMixedMasculine,
            "first-mixed-ts-m" => NounDeclension::FirstMixedTsMasculine,
            "first-hard-n" => NounDeclension::FirstHardNeuter,
            "first-soft-m" => NounDeclension::FirstSoftMasculine,
            "first-soft-agent-tel-m" => NounDeclension::FirstSoftMasculineAgentTel,
            "first-soft-lord-m" => NounDeclension::FirstSoftMasculineLord,
            "first-soft-j-m" => NounDeclension::FirstSoftMasculineJ,
            "first-soft-ey-m" => NounDeclension::FirstSoftMasculineEy,
            "first-soft-n" => NounDeclension::FirstSoftNeuter,
            "first-soft-ishche-n" => NounDeclension::FirstSoftNeuterIshche,
            "first-soft-ie-n" => NounDeclension::FirstSoftNeuterIe,
            "second-hard" => NounDeclension::SecondHard,
            "second-hard-velar" => NounDeclension::SecondHardVelar,
            "second-soft" => NounDeclension::SecondSoft,
            "second-soft-postvocalic-ancient-pl" => {
                NounDeclension::SecondSoftPostvocalicAncientPlural
            }
            "second-soft-m-ia" => NounDeclension::SecondSoftMasculineIa,
            "second-soft-f-ia" => NounDeclension::SecondSoftFeminineIa,
            "second-mixed" => NounDeclension::SecondMixed,
            "third-f" => NounDeclension::ThirdFeminine,
            "third-m" => NounDeclension::ThirdMasculine,
            "fourth-neuter-en" => NounDeclension::FourthNeuterEn,
            "fourth-neuter-es" => NounDeclension::FourthNeuterEs,
            "fourth-neuter-es-alt-first" => NounDeclension::FourthNeuterEsAlternatingFirst,
            "fourth-neuter-es-paired-dual" => NounDeclension::FourthNeuterEsPairedDual,
            "fourth-neuter-at" => NounDeclension::FourthNeuterAt,
            "fourth-feminine-er" => NounDeclension::FourthFeminineEr,
            "fourth-feminine-er-daughter" => NounDeclension::FourthFeminineErDaughter,
            "fourth-feminine-ov" => NounDeclension::FourthFeminineOv,
            "fourth-feminine-ov-syncopating" => NounDeclension::FourthFeminineOvSyncopating,
            "fourth-masculine-en" => NounDeclension::FourthMasculineEn,
            "fourth-masculine-en-day" => NounDeclension::FourthMasculineEnDay,
            "fourth-masculine-en-kamen" => NounDeclension::FourthMasculineEnKamen,
            "indeclinable" => NounDeclension::Indeclinable,
            value => return invalid_metadata("noun class", value),
        },
        number_inventory,
        animacy_inventory,
    })
}

pub(crate) fn parse_noun_number_inventory(value: &str) -> Result<NounNumberInventory> {
    match value {
        "all" => Ok(NounNumberInventory::All),
        "singular-only" => Ok(NounNumberInventory::SingularOnly),
        "dual-only" => Ok(NounNumberInventory::DualOnly),
        "plural-only" => Ok(NounNumberInventory::PluralOnly),
        "singular-and-dual" => Ok(NounNumberInventory::SingularAndDual),
        "singular-and-plural" => Ok(NounNumberInventory::SingularAndPlural),
        "dual-and-plural" => Ok(NounNumberInventory::DualAndPlural),
        value => invalid_metadata("noun number inventory", value),
    }
}

pub(crate) fn parse_noun_animacy_inventory(value: &str) -> Result<NounAnimacyInventory> {
    match value {
        "any" => Ok(NounAnimacyInventory::All),
        "animate" => Ok(NounAnimacyInventory::AnimateOnly),
        "inanimate" => Ok(NounAnimacyInventory::InanimateOnly),
        value => invalid_metadata("noun animacy inventory", value),
    }
}

pub(crate) fn adjective_lexeme(id: &LexemeId) -> Result<AdjectiveLexeme> {
    adjectival_lexeme(id, PartOfSpeech::Adjective)
}

pub(crate) fn determiner_lexeme(id: &LexemeId) -> Result<DeterminerLexeme> {
    let row = require_pos(id, PartOfSpeech::Determiner)?;
    let lexeme = DeterminerLexeme::new(
        SynodalWord::parse(row.0[1])?,
        SynodalWord::parse(row.0[4])?,
        match row.0[3] {
            "determiner-pronominal-hard" => DeterminerDeclension::PronominalHard,
            "determiner-ves-mixed" => DeterminerDeclension::VesMixed,
            "determiner-vsyak-mixed" => DeterminerDeclension::VsyakMixed,
            "determiner-full-sk" => DeterminerDeclension::FullSk,
            value => return invalid_metadata("determiner class", value),
        },
    );
    validate_determiner_lexeme(&lexeme)?;
    Ok(lexeme)
}

pub(crate) fn numeral_lexeme(id: &LexemeId) -> Result<NumeralLexeme> {
    let row = require_pos(id, PartOfSpeech::Numeral)?;
    let lexeme = NumeralLexeme::new(
        SynodalWord::parse(row.0[1])?,
        SynodalWord::parse(row.0[4])?,
        match row.0[3] {
            "numeral-cardinal-one" => NumeralDeclension::CardinalOne,
            "numeral-cardinal-two" => NumeralDeclension::CardinalTwo,
            "numeral-cardinal-both" => NumeralDeclension::CardinalBoth,
            "numeral-cardinal-three" => NumeralDeclension::CardinalThree,
            "numeral-cardinal-four" => NumeralDeclension::CardinalFour,
            "numeral-cardinal-i-stem" => NumeralDeclension::CardinalIStem,
            "numeral-cardinal-ten" => NumeralDeclension::CardinalTen,
            "numeral-cardinal-hundred" => NumeralDeclension::CardinalHundred,
            "numeral-cardinal-second-hard" => NumeralDeclension::CardinalSecondHard,
            "numeral-cardinal-second-mixed" => NumeralDeclension::CardinalSecondMixed,
            "numeral-cardinal-first-hard-m" => NumeralDeclension::CardinalFirstHardMasculine,
            "numeral-cardinal-third-f" => NumeralDeclension::CardinalThirdFeminine,
            "ordinal-hard" => NumeralDeclension::OrdinalHard,
            "ordinal-ii" => NumeralDeclension::OrdinalIi,
            "numeral-collective-agreeing" => NumeralDeclension::CollectiveAgreeing,
            "numeral-collective-governing-neuter" => NumeralDeclension::CollectiveGoverningNeuter,
            "numeral-collective-hard-plural" => NumeralDeclension::CollectiveHardPlural,
            "numeral-multiplicative-hard" => NumeralDeclension::MultiplicativeHard,
            "numeral-multiplicative-soft" => NumeralDeclension::MultiplicativeSoft,
            "numeral-fractional-hard" => NumeralDeclension::FractionalHard,
            "numeral-fractional-first-u" => NumeralDeclension::FractionalFirstHardUStem,
            "numeral-fractional-second-hard" => NumeralDeclension::FractionalSecondHard,
            "numeral-fractional-third-f" => NumeralDeclension::FractionalThirdFeminine,
            value => return invalid_metadata("numeral class", value),
        },
    );
    validate_numeral_lexeme(&lexeme)?;
    Ok(lexeme)
}

pub(crate) fn pronoun_lexeme(id: &LexemeId) -> Result<PronounLexeme> {
    let row = require_pos(id, PartOfSpeech::Pronoun)?;
    let lemma = SynodalWord::parse(row.0[1])?;
    let class = row.0[3];
    let lexeme = match class {
        "pronoun-personal-first" => PronounLexeme::closed(lemma, PronounDeclension::PersonalFirst),
        "pronoun-personal-second" => {
            PronounLexeme::closed(lemma, PronounDeclension::PersonalSecond)
        }
        "pronoun-reflexive" => PronounLexeme::closed(lemma, PronounDeclension::Reflexive),
        "pronoun-reflexive-clitic" => PronounLexeme::closed(lemma, PronounDeclension::Reflexive)
            .with_selection(PronounFormSelection::Enclitic),
        "pronoun-third-person" => PronounLexeme::closed(lemma, PronounDeclension::ThirdPerson)
            .with_environment(PronounEnvironment::ContextualVariants),
        "pronoun-third-person-demonstrative" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::ThirdPersonAndDemonstrative,
        )
        .with_environment(PronounEnvironment::ContextualVariants),
        "pronoun-relative-izhe" => PronounLexeme::closed(lemma, PronounDeclension::ThirdPerson)
            .with_environment(PronounEnvironment::ContextualVariants)
            .with_postpositive(PronounPostpositive::Zhe),
        "pronoun-proximal-sei" => PronounLexeme::closed(lemma, PronounDeclension::ProximalSei),
        "pronoun-soft" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::Soft,
        ),
        "pronoun-soft-i-alternating" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::SoftIAlternating,
        ),
        "pronoun-hard" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::Hard,
        ),
        "pronoun-mixed-possessive" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::MixedPossessive,
        ),
        "pronoun-short-hard" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::ShortHard,
        ),
        "pronoun-short-ov-mixed" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::ShortOvMixed,
        ),
        "pronoun-short-velar" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::ShortVelar,
        ),
        "pronoun-quantity-velar" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::QuantityVelar,
        ),
        "pronoun-full-hard" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::FullHard,
        ),
        "pronoun-full-soft" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::FullSoft,
        ),
        "pronoun-full-velar" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::FullVelar,
        ),
        "pronoun-interrogative-kii" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeKii)
        }
        "pronoun-interrogative-who" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWho)
        }
        "pronoun-interrogative-what" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWhat)
        }
        "pronoun-indefinite-who" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWho)
                .with_prefix(PronounPrefix::IndefiniteNe)
        }
        "pronoun-indefinite-what" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWhat)
                .with_prefix(PronounPrefix::IndefiniteNe)
        }
        "pronoun-indefinite-kii" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeKii)
                .with_prefix(PronounPrefix::IndefiniteNe)
        }
        "pronoun-negative-who" => PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWho)
            .with_prefix(PronounPrefix::NegativeNi),
        "pronoun-negative-what" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWhat)
                .with_prefix(PronounPrefix::NegativeNi)
        }
        "pronoun-negative-kii" => PronounLexeme::closed(lemma, PronounDeclension::InterrogativeKii)
            .with_prefix(PronounPrefix::NegativeNi),
        "pronoun-negative-full-hard" => PronounLexeme::regular(
            lemma,
            SynodalWord::parse(row.0[4])?,
            PronounDeclension::FullHard,
        )
        .with_prefix(PronounPrefix::NegativeNi),
        "pronoun-kii-zhdo" => PronounLexeme::closed(lemma, PronounDeclension::InterrogativeKii)
            .with_postpositive(PronounPostpositive::Zhdo),
        "pronoun-negative-who-zhe" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWho)
                .with_prefix(PronounPrefix::NegativeNi)
                .with_postpositive(PronounPostpositive::Zhe)
        }
        "pronoun-negative-what-zhe" => {
            PronounLexeme::closed(lemma, PronounDeclension::InterrogativeWhat)
                .with_prefix(PronounPrefix::NegativeNi)
                .with_postpositive(PronounPostpositive::Zhe)
        }
        value => return invalid_metadata("pronoun class", value),
    };
    validate_pronoun_lexeme(&lexeme)?;
    Ok(lexeme)
}

pub(crate) fn adjectival_lexeme(id: &LexemeId, expected: PartOfSpeech) -> Result<AdjectiveLexeme> {
    let row = require_pos(id, expected)?;
    let short_masculine = PRINCIPAL_PARTS
        .iter()
        .find(|part| part.0[0] == id.as_str() && part.0[1] == "short-masculine-stem");
    let lexeme = AdjectiveLexeme {
        lemma: SynodalWord::parse(row.0[1])?,
        stem: SynodalWord::parse(row.0[4])?,
        class: match row.0[3] {
            "hard-short" => AdjectiveClass::Hard,
            "soft-short" => AdjectiveClass::Soft,
            "velar-short" => AdjectiveClass::Velar,
            "possessive-hard-short" => AdjectiveClass::PossessiveHard,
            "possessive-soft-short" => AdjectiveClass::PossessiveSoft,
            "possessive-j-short" => AdjectiveClass::PossessiveJ,
            "possessive-in" => AdjectiveClass::PossessiveIn,
            "possessive-sk" => AdjectiveClass::PossessiveSk,
            "possessive-ii" => AdjectiveClass::PossessiveIi,
            value => return invalid_metadata("adjective class", value),
        },
        short_masculine_stem: short_masculine
            .map(|part| SynodalWord::parse(part.0[2]))
            .transpose()?,
        short_masculine_formation: short_masculine
            .map(|part| parse_short_masculine_formation(part.0[3]))
            .transpose()?,
        comparative_stem: PRINCIPAL_PARTS
            .iter()
            .find(|part| part.0[0] == id.as_str() && part.0[1] == "comparative-stem")
            .map(|part| SynodalWord::parse(part.0[2]))
            .transpose()?,
        comparison_formation: PRINCIPAL_PARTS
            .iter()
            .find(|part| part.0[0] == id.as_str() && part.0[1] == "comparative-stem")
            .map(|part| parse_comparison_formation(part.0[3]))
            .transpose()?,
    };
    validate_adjective_lexeme(&lexeme)?;
    Ok(lexeme)
}

pub(crate) fn parse_short_masculine_formation(value: &str) -> Result<ShortMasculineStemFormation> {
    match value {
        "double-n-reduction" => Ok(ShortMasculineStemFormation::DoubleNReduction),
        "mobile-e-insertion" => Ok(ShortMasculineStemFormation::MobileEInsertion),
        "mobile-o-insertion" => Ok(ShortMasculineStemFormation::MobileOInsertion),
        value => invalid_metadata("short masculine formation", value),
    }
}

pub(crate) fn parse_comparison_formation(value: &str) -> Result<ComparisonFormation> {
    match value {
        "ancient-hard" => Ok(ComparisonFormation::AncientHard),
        "ancient-soft" => Ok(ComparisonFormation::AncientSoft),
        "later-yat" => Ok(ComparisonFormation::LaterYat),
        "later-ai" => Ok(ComparisonFormation::LaterAi),
        value => invalid_metadata("comparison formation", value),
    }
}

pub(crate) fn parse_active_participle_short_formation(
    value: &str,
) -> Result<ActiveParticipleShortFormation> {
    match value {
        "present-first-unpalatalized" => {
            Ok(ActiveParticipleShortFormation::PresentFirstUnpalatalized)
        }
        "present-first-palatalized" => Ok(ActiveParticipleShortFormation::PresentFirstPalatalized),
        "present-second" => Ok(ActiveParticipleShortFormation::PresentSecond),
        "present-after-sibilant" => Ok(ActiveParticipleShortFormation::PresentAfterSibilant),
        "past-consonant" => Ok(ActiveParticipleShortFormation::PastConsonant),
        "past-vowel" => Ok(ActiveParticipleShortFormation::PastVowel),
        "past-iotated" => Ok(ActiveParticipleShortFormation::PastIotated),
        value => invalid_metadata("active participle short formation", value),
    }
}

/// A verb registered under a lemma in `-сѧ` (Alypy §73: verbs used only
/// with the reflexive enclitic, or admitted in their reflexive voice). Its
/// stems and principal parts are stored bare; the resolver appends the
/// enclitic to every generated cell.
pub(crate) fn is_reflexive_verb(id: &LexemeId) -> bool {
    raw_by_id(id).is_some_and(|row| row.0[2] == "verb" && row.0[1].ends_with("сѧ"))
}

pub(crate) fn verb_lexeme(id: &LexemeId) -> Result<VerbLexeme> {
    let row = require_pos(id, PartOfSpeech::Verb)?;
    // The bare infinitive is the host the enclitic attaches to.
    let lemma = row.0[1].strip_suffix("сѧ").unwrap_or(row.0[1]);
    let conjugation = match row.0[3] {
        "first-unpalatalized" => VerbConjugation::FirstUnpalatalized,
        "first-palatalized" => VerbConjugation::FirstPalatalized,
        "second" => VerbConjugation::Second,
        "archaic" => VerbConjugation::Archaic,
        value => return invalid_metadata("verb conjugation", value),
    };
    let aspect = match row.0[6] {
        "imperfective" => Aspect::Imperfective,
        "perfective" => Aspect::Perfective,
        "biaspectual" => Aspect::Biaspectual,
        "" | "unknown" => Aspect::Unknown,
        value => return invalid_metadata("aspect", value),
    };
    let part = |system: &str| {
        PRINCIPAL_PARTS
            .iter()
            .find(|part| part.0[0] == id.as_str() && part.0[1] == system)
    };
    let parsed_part = |system: &str| -> Result<Option<SynodalWord>> {
        part(system)
            .map(|entry| SynodalWord::parse(entry.0[2]))
            .transpose()
    };
    let participle_part = |prefix: &str| -> Result<Option<ParticiplePrincipalPart>> {
        let short = part(&format!("{prefix}-short-stem"));
        let long = part(&format!("{prefix}-long-stem"));
        if short.is_none() && long.is_none() {
            return Ok(None);
        }
        let short_metadata = short.map(|entry| entry.0[3]);
        let long_metadata = long.map(|entry| entry.0[3]);
        let class_code = long_metadata
            .or(short_metadata)
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        let class = match class_code {
            "hard" => AdjectiveClass::Hard,
            "soft" => AdjectiveClass::Soft,
            value => return invalid_metadata("participial adjective class", value),
        };
        for entry in [short, long].into_iter().flatten() {
            if entry.0[3].split(':').next() != Some(class_code) {
                return Err(Error::ContradictoryMetadata {
                    reason: format!(
                        "participial stems for {} use inconsistent classes",
                        id.as_str()
                    ),
                });
            }
        }
        Ok(Some(ParticiplePrincipalPart {
            short_stem: short
                .map(|entry| SynodalWord::parse(entry.0[2]))
                .transpose()?,
            short_formation: short_metadata
                .and_then(|metadata| metadata.split_once(':').map(|(_, value)| value))
                .map(parse_active_participle_short_formation)
                .transpose()?,
            long_stem: long
                .map(|entry| SynodalWord::parse(entry.0[2]))
                .transpose()?,
            class,
        }))
    };

    let lexeme = VerbLexeme {
        lemma: SynodalWord::parse(lemma)?,
        aspect,
        conjugation,
        present_stem: nonempty_word(row.0[4])?,
        present_first_singular: parsed_part("present-first-singular")?,
        present_third_plural: parsed_part("present-third-plural")?,
        future_stem: parsed_part("future-stem")?,
        future_first_singular: parsed_part("future-first-singular")?,
        future_third_plural: parsed_part("future-third-plural")?,
        imperfect_stem: parsed_part("imperfect-stem")?,
        imperfect_formation: part("imperfect-stem")
            .map(|entry| parse_imperfect(entry.0[3]))
            .transpose()?,
        aorist_stem: parsed_part("aorist-stem")?,
        aorist_formation: part("aorist-stem")
            .map(|entry| parse_aorist(entry.0[3]))
            .transpose()?,
        imperative_stem: parsed_part("imperative-stem")?,
        imperative_formation: part("imperative-stem")
            .map(|entry| parse_imperative(entry.0[3]))
            .transpose()?,
        l_participle_stem: parsed_part("l-participle-stem")?,
        l_participle_masculine_singular_stem: parsed_part("l-participle-masculine-singular-stem")?,
        present_active_participle: participle_part("present-active-participle")?,
        past_active_participle: participle_part("past-active-participle")?,
        present_passive_participle: participle_part("present-passive-participle")?,
        past_passive_participle: participle_part("past-passive-participle")?,
        verbal_noun: part("verbal-noun-ie-platform")
            .map(|entry| VerbalNounPrincipalPart::past_passive_ie(entry.0[2]))
            .transpose()?,
    };
    let future_part_count = [
        lexeme.future_stem.is_some(),
        lexeme.future_first_singular.is_some(),
        lexeme.future_third_plural.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if !matches!(future_part_count, 0 | 3) {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "independent future principal parts for {} must be supplied as a complete triple",
                id.as_str()
            ),
        });
    }
    Ok(lexeme)
}

pub(crate) fn all_lexemes() -> Result<Vec<LexemeSummary>> {
    LEXEMES.iter().map(summary).collect()
}

pub(crate) fn lexical_metadata(id: &LexemeId) -> Result<LexicalMetadataSummary> {
    let row = raw_by_id(id).ok_or_else(|| Error::UnknownLemma {
        lookup: id.to_string(),
    })?;
    let optional = |value: &str| (!value.is_empty()).then(|| value.to_owned());
    Ok(LexicalMetadataSummary {
        lexeme_id: id.clone(),
        class: optional(row.0[3]),
        stem: optional(row.0[4]),
        gender: optional(row.0[5]),
        aspect: optional(row.0[6]),
        source_id: row.0[7].into(),
        target_recension: row.0[8].into(),
        noun_restriction: NOUN_RESTRICTIONS
            .iter()
            .find(|restriction| restriction.0[0] == id.as_str())
            .map(|restriction| NounRestrictionSummary {
                number_inventory: restriction.0[1].into(),
                animacy_inventory: restriction.0[2].into(),
                evidence_id: restriction.0[3].into(),
            }),
        principal_parts: rows_for(PRINCIPAL_PARTS, |row| row.0[0], id.as_str())
            .iter()
            .map(|part| PrincipalPartSummary {
                system: part.0[1].into(),
                value: part.0[2].into(),
                formation: optional(part.0[3]),
                evidence_id: part.0[4].into(),
            })
            .collect(),
        exact_forms: rows_for(EXACT_FORMS, |row| row.0[0], id.as_str())
            .iter()
            .map(|form| ExactFormSummary {
                cell: form.0[1].into(),
                expanded: form.0[2].into(),
                printed: form.0[3].into(),
                evidence_id: form.0[4].into(),
                source_kind: form.0[5].into(),
            })
            .collect(),
        accents: rows_for(ACCENTS, |row| row.0[0], id.as_str())
            .iter()
            .map(|accent| AccentSummary {
                cell: accent.0[1].into(),
                expanded: accent.0[2].into(),
                accented: accent.0[3].into(),
                evidence_id: accent.0[4].into(),
                source_id: accent.0[5].into(),
                source_recension: accent.0[6].into(),
            })
            .collect(),
        accent_paradigms: rows_for(ACCENT_PARADIGMS, |row| row.0[0], id.as_str())
            .iter()
            .map(|accent| AccentParadigmSummary {
                paradigm_id: accent.0[1].into(),
                scope: accent.0[2].into(),
                placement: accent.0[3].into(),
                mark: accent.0[4].into(),
                breathing: optional(accent.0[5]),
                evidence_id: accent.0[6].into(),
                source_id: accent.0[7].into(),
            })
            .collect(),
    })
}

pub(crate) fn alignments() -> Result<Vec<AlignmentSummary>> {
    ALIGNMENTS
        .iter()
        .map(|row| {
            let confidence_basis_points =
                row.0[7]
                    .parse::<u16>()
                    .map_err(|_| Error::ContradictoryMetadata {
                        reason: format!("invalid mapping confidence {}", row.0[7]),
                    })?;
            Ok(AlignmentSummary {
                mapping_id: row.0[0].into(),
                source_lexeme_id: row.0[1].into(),
                target_lexeme_id: row.0[2].into(),
                relation: row.0[3].into(),
                status: row.0[4].into(),
                morphology: row.0[5].into(),
                semantics: row.0[6].into(),
                confidence_basis_points,
                transformations: split_list(row.0[9]),
                review_note: row.0[10].into(),
            })
        })
        .collect()
}

pub(crate) fn transformation_rules() -> Vec<TransformationRuleSummary> {
    TRANSFORMATION_RULES
        .iter()
        .map(|row| TransformationRuleSummary {
            rule_id: row.0[0].into(),
            source_recension: row.0[1].into(),
            target_recension: row.0[2].into(),
            operation: row.0[3].into(),
            status: row.0[4].into(),
            evidence_id: row.0[5].into(),
        })
        .collect()
}

pub(crate) fn conflicts() -> Vec<RecensionConflictSummary> {
    CONFLICTS
        .iter()
        .map(|row| RecensionConflictSummary {
            conflict_id: row.0[0].into(),
            source_lexeme_id: row.0[1].into(),
            target_lexeme_id: row.0[2].into(),
            kind: row.0[3].into(),
            status: row.0[4].into(),
            supporting_evidence: row.0[5].into(),
            contradicting_evidence: row.0[6].into(),
            resolution: row.0[7].into(),
        })
        .collect()
}

pub(crate) fn positional_rules() -> Vec<PositionalRuleSummary> {
    POSITIONAL_RULES
        .iter()
        .map(|row| PositionalRuleSummary {
            rule_id: row.0[0].into(),
            input: row.0[1].into(),
            context: row.0[2].into(),
            output: row.0[3].into(),
            exceptions: row.0[4].into(),
            evidence_id: row.0[5].into(),
        })
        .collect()
}

pub(crate) fn irregular_overrides() -> Vec<IrregularOverrideSummary> {
    // The merged exact-form table is the single irregular table: overrides
    // survive as per-row provenance stamps, so the summary is the distinct
    // set of stamped (lexeme, system, evidence) tuples. `cell_set` names the
    // curated source table the stamps were folded from.
    let mut seen = std::collections::BTreeSet::new();
    EXACT_FORMS
        .iter()
        .filter(|row| !row.0[7].is_empty())
        .filter(|row| seen.insert((row.0[0], row.0[7], row.0[8])))
        .map(|row| IrregularOverrideSummary {
            lexeme_id: row.0[0].into(),
            system: row.0[7].into(),
            cell_set: "data/synodal/exact_forms.tsv".into(),
            evidence_id: row.0[8].into(),
        })
        .collect()
}

pub(crate) fn irregular_verb_inventory() -> Result<Vec<IrregularVerbInventorySummary>> {
    IRREGULAR_VERB_INVENTORY
        .iter()
        .map(|row| {
            let source_order =
                row.0[0]
                    .parse::<u8>()
                    .map_err(|_| Error::ContradictoryMetadata {
                        reason: format!("invalid Alypy §104 source order {:?}", row.0[0]),
                    })?;
            Ok(IrregularVerbInventorySummary {
                source_order,
                headword: row.0[1].into(),
                systems: split_list(row.0[2]),
                strategy: row.0[3].into(),
                implementation_status: row.0[4].into(),
                evidence_id: row.0[5].into(),
                note: row.0[6].into(),
            })
        })
        .collect()
}

pub(crate) fn abbreviations_for(id: &LexemeId, sense_id: &str) -> Vec<AbbreviationRecord> {
    ABBREVIATIONS
        .iter()
        .filter(|row| row.0[0] == id.as_str() && row.0[1] == sense_id)
        .map(|row| AbbreviationRecord {
            lexeme_id: row.0[0],
            sense_id: row.0[1],
            cell: row.0[2],
            expanded: row.0[3],
            printed: row.0[4],
            rule_id: row.0[5],
            evidence_id: row.0[6],
            reversible: row.0[7] == "true",
            required_marks: row.0[8],
            context_restrictions: row.0[9],
            ambiguity: row.0[10],
            source_recension: row.0[11],
            target_recension: row.0[12],
        })
        .collect()
}

pub(crate) fn abbreviations_for_printed(printed: &str) -> Vec<AbbreviationRecord> {
    ABBREVIATIONS
        .iter()
        .filter(|row| row.0[4] == printed)
        .map(|row| AbbreviationRecord {
            lexeme_id: row.0[0],
            sense_id: row.0[1],
            cell: row.0[2],
            expanded: row.0[3],
            printed: row.0[4],
            rule_id: row.0[5],
            evidence_id: row.0[6],
            reversible: row.0[7] == "true",
            required_marks: row.0[8],
            context_restrictions: row.0[9],
            ambiguity: row.0[10],
            source_recension: row.0[11],
            target_recension: row.0[12],
        })
        .collect()
}

pub(crate) fn abbreviation_families_for(
    id: &LexemeId,
    sense_id: &str,
) -> Vec<AbbreviationFamilyRecord> {
    ABBREVIATION_FAMILIES
        .iter()
        .filter(|row| row.0[0] == id.as_str() && row.0[1] == sense_id)
        .map(abbreviation_family_record)
        .collect()
}

pub(crate) fn abbreviation_family_records() -> Vec<AbbreviationFamilyRecord> {
    ABBREVIATION_FAMILIES
        .iter()
        .map(abbreviation_family_record)
        .collect()
}

pub(crate) fn abbreviation_family_record(row: &RawAbbreviationFamily) -> AbbreviationFamilyRecord {
    AbbreviationFamilyRecord {
        lexeme_id: row.0[0],
        sense_id: row.0[1],
        expanded_prefix: row.0[2],
        printed_prefix: row.0[3],
        rule_id: row.0[4],
        evidence_id: row.0[5],
        reversible: row.0[6] == "true",
        required_marks: row.0[7],
        context_restrictions: row.0[8],
        ambiguity: row.0[9],
        source_recension: row.0[10],
        target_recension: row.0[11],
    }
}

pub(crate) fn noun_uses_inherited_class(id: &LexemeId) -> bool {
    raw_by_id(id).is_some_and(|row| row.0[3].starts_with("inherited-"))
}

pub(crate) fn inherited_alignments(
    id: &LexemeId,
    policy: GenerationPolicy,
    threshold_basis_points: u16,
) -> Result<Vec<InheritedAlignment>> {
    let candidates: Vec<&RawAlignment> = ALIGNMENTS
        .iter()
        .filter(|row| {
            row.0[2] == id.as_str()
                && row.0[4] != "rejected"
                && row.0[6] != "false-friend"
                && (policy == GenerationPolicy::Exploratory
                    || matches!(row.0[4], "reviewed" | "automatically-validated"))
        })
        .collect();
    if candidates.is_empty() {
        return Err(Error::MissingRecensionMapping { source: id.clone() });
    }
    if policy != GenerationPolicy::Exploratory && candidates.len() > 1 {
        return Err(Error::AmbiguousRecensionMapping {
            mappings: candidates
                .iter()
                .map(|row| RecensionMappingId::from(row.0[0]))
                .collect(),
        });
    }
    candidates
        .into_iter()
        .map(|row| {
            let confidence_basis_points =
                row.0[7]
                    .parse::<u16>()
                    .map_err(|_| Error::ContradictoryMetadata {
                        reason: format!("invalid mapping confidence {}", row.0[7]),
                    })?;
            if policy == GenerationPolicy::Productive
                && confidence_basis_points < threshold_basis_points
            {
                return Err(Error::MissingRecensionMapping { source: id.clone() });
            }
            let confidence =
                Confidence::from_basis_points(confidence_basis_points).ok_or_else(|| {
                    Error::ContradictoryMetadata {
                        reason: "mapping confidence exceeds 10000 basis points".into(),
                    }
                })?;
            Ok(InheritedAlignment {
                mapping_id: RecensionMappingId::from(row.0[0]),
                source_lexeme_id: LexemeId::from(row.0[1]),
                confidence,
                evidence_ids: split_list(row.0[8]),
                transformations: split_list(row.0[9]),
            })
        })
        .collect()
}

pub(crate) fn require_pos(id: &LexemeId, expected: PartOfSpeech) -> Result<&'static RawLexeme> {
    let row = raw_by_id(id).ok_or_else(|| Error::UnknownLemma {
        lookup: id.to_string(),
    })?;
    let actual = parse_pos(row.0[2])?;
    if actual == expected {
        Ok(row)
    } else {
        Err(Error::ContradictoryMetadata {
            reason: format!("lexeme {id} is {actual:?}, not {expected:?}"),
        })
    }
}

pub(crate) fn summary(row: &RawLexeme) -> Result<LexemeSummary> {
    Ok(LexemeSummary {
        id: LexemeId::from(row.0[0]),
        lemma: row.0[1].into(),
        part_of_speech: parse_pos(row.0[2])?,
        source_id: row.0[7].into(),
    })
}

pub(crate) fn parse_pos(value: &str) -> Result<PartOfSpeech> {
    PartOfSpeech::from_code(value).ok_or_else(|| Error::ContradictoryMetadata {
        reason: format!("unknown part of speech code {value:?}"),
    })
}

pub(crate) fn parse_gender(value: &str) -> Result<Gender> {
    match value {
        "masculine" => Ok(Gender::Masculine),
        "feminine" => Ok(Gender::Feminine),
        "neuter" => Ok(Gender::Neuter),
        other => invalid_metadata("gender", other),
    }
}

pub(crate) fn parse_imperfect(value: &str) -> Result<ImperfectFormation> {
    match value {
        "h" => Ok(ImperfectFormation::H),
        "yah" => Ok(ImperfectFormation::Yah),
        "ah" => Ok(ImperfectFormation::Ah),
        "irregular" => Ok(ImperfectFormation::Irregular),
        other => invalid_metadata("imperfect formation", other),
    }
}

pub(crate) fn parse_aorist(value: &str) -> Result<AoristFormation> {
    match value {
        "vowel" => Ok(AoristFormation::VowelStem),
        "vowel-t" => Ok(AoristFormation::VowelStemWithT),
        "consonant" => Ok(AoristFormation::ConsonantStem),
        "irregular" => Ok(AoristFormation::Irregular),
        other => invalid_metadata("aorist formation", other),
    }
}

pub(crate) fn parse_imperative(value: &str) -> Result<ImperativeFormation> {
    match value {
        "first-unpalatalized" => Ok(ImperativeFormation::FirstUnpalatalized),
        "i-series" => Ok(ImperativeFormation::ISeries),
        "j-series" => Ok(ImperativeFormation::JSeries),
        "irregular" => Ok(ImperativeFormation::Irregular),
        other => invalid_metadata("imperative formation", other),
    }
}

pub(crate) fn nonempty_word(value: &str) -> Result<Option<SynodalWord>> {
    if value.is_empty() {
        Ok(None)
    } else {
        SynodalWord::parse(value).map(Some)
    }
}

pub(crate) fn invalid_metadata<T>(field: &str, value: &str) -> Result<T> {
    Err(Error::ContradictoryMetadata {
        reason: format!("unknown {field} code {value:?}"),
    })
}

pub(crate) fn split_list(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(str::to_owned).collect()
    }
}
