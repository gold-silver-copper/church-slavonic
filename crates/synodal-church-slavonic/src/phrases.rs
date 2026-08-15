use std::collections::BTreeSet;

use synodal_church_slavonic_core::{
    AnalyticConstruction, Aspect, AuthorityRole, EpistemicRole, Error, Evidence, EvidenceId,
    EvidenceKind, FormSet, Gender, GrammarCell, LexemeId, NegativePronounBase, Number,
    OrthographyProfile, ParticipleCell, ParticipleVoice, Person, PhraseRole, PhraseToken,
    PronounCell, PronounCliticProsody, PronounFormSelection, PronounPostpositive, RealizedPhrase,
    Recension, Result, RuleId, SourceId, TraceStep, decline_pronoun,
};

use crate::{Inflector, PartOfSpeech, Participle, Pronoun, Verb};

/// Realizes the Alypy §85 compound future: a present form of the auxiliary
/// `имати` followed by the infinitive of an imperfective lexical verb.
pub fn compound_future(lemma: &str, person: Person, number: Number) -> Result<RealizedPhrase> {
    compound_future_with(lemma, person, number, Inflector::default())
}

pub fn compound_future_with(
    lemma: &str,
    person: Person,
    number: Number,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    if verb.aspect()? != Aspect::Imperfective {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §85 restricts the compound future to imperfective verbs".into(),
        });
    }
    let auxiliary = Verb::resolve_with("имати", inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::CompoundFuture,
        vec![
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: auxiliary.present(person, number)?,
            },
            PhraseToken {
                role: PhraseRole::Infinitive,
                forms: verb.infinitive()?,
            },
        ],
    )
}

/// Realizes the Alypy §88 perfect as an l-participle plus the present copula.
pub fn perfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
) -> Result<RealizedPhrase> {
    perfect_with(lemma, person, number, gender, Inflector::default())
}

pub fn perfect_with(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let copula = Verb::resolve_with("быти", inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::Perfect,
        vec![
            PhraseToken {
                role: PhraseRole::LParticiple,
                forms: verb.l_participle(gender, number)?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: copula.present(person, number)?,
            },
        ],
    )
}

/// Realizes the Alypy §89 pluperfect as an l-participle plus an imperfect
/// form of the copula. Alternate copular series remain variants on the token.
pub fn pluperfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
) -> Result<RealizedPhrase> {
    pluperfect_with(lemma, person, number, gender, Inflector::default())
}

pub fn pluperfect_with(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let copula = Verb::resolve_with("быти", inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::Pluperfect,
        vec![
            PhraseToken {
                role: PhraseRole::LParticiple,
                forms: verb.l_participle(gender, number)?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: copula.imperfect(person, number)?,
            },
        ],
    )
}

/// Realizes the Alypy §91 conditional as an l-participle plus the aorist of
/// `быти`; it remains a structured phrase rather than a space-bearing word.
pub fn conditional(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
) -> Result<RealizedPhrase> {
    conditional_with(lemma, person, number, gender, Inflector::default())
}

pub fn conditional_with(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let verb = Verb::resolve_with(lemma, inflector)?;
    let copula = Verb::resolve_with("быти", inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::Conditional,
        vec![
            PhraseToken {
                role: PhraseRole::LParticiple,
                forms: verb.l_participle(gender, number)?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: copula.aorist(person, number)?,
            },
        ],
    )
}

/// Realizes a reviewed passive participle plus the present copula as a typed
/// analytic passive construction (Alypy §§101–102).
pub fn analytic_passive(
    lemma: &str,
    participle_cell: ParticipleCell,
    person: Person,
    number: Number,
) -> Result<RealizedPhrase> {
    analytic_passive_with(lemma, participle_cell, person, number, Inflector::default())
}

pub fn analytic_passive_with(
    lemma: &str,
    participle_cell: ParticipleCell,
    person: Person,
    number: Number,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    if participle_cell.voice != ParticipleVoice::Passive {
        return Err(Error::HistoricallyInvalidCell {
            reason: "analytic passive requires a passive participle".into(),
        });
    }
    let participle = Participle::resolve_with(lemma, inflector)?;
    let copula = Verb::resolve_with("быти", inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::AnalyticPassive,
        vec![
            PhraseToken {
                role: PhraseRole::PassiveParticiple,
                forms: participle.form(participle_cell)?,
            },
            PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: copula.present(person, number)?,
            },
        ],
    )
}

/// Realizes Alypy §48 negative-pronoun interposition as three independently
/// evidenced tokens: `ни + preposition + inflected interrogative base`.
pub fn negative_pronoun_prepositional(
    preposition: &str,
    base: NegativePronounBase,
    postpositive: Option<PronounPostpositive>,
    cell: PronounCell,
) -> Result<RealizedPhrase> {
    negative_pronoun_prepositional_with(preposition, base, postpositive, cell, Inflector::default())
}

pub fn negative_pronoun_prepositional_with(
    preposition: &str,
    base: NegativePronounBase,
    postpositive: Option<PronounPostpositive>,
    cell: PronounCell,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let negative_id = match (base, postpositive) {
        (NegativePronounBase::Who, None) => "synodal:pronoun:nikto",
        (NegativePronounBase::Who, Some(PronounPostpositive::Zhe)) => {
            "synodal:pronoun:v06-niktozhe"
        }
        (NegativePronounBase::What, None) => "synodal:pronoun:nichto",
        (NegativePronounBase::What, Some(PronounPostpositive::Zhe)) => {
            "synodal:pronoun:v06-nichtozhe"
        }
        (NegativePronounBase::Kii, None) => "synodal:pronoun:nikii",
        (NegativePronounBase::Kotoryi, None) => "synodal:pronoun:nikotoryi",
        (_, Some(_)) => {
            return Err(Error::ContradictoryMetadata {
                reason:
                    "preposition-interposed negative pronouns license only optional -же on кто/что"
                        .into(),
            });
        }
    };

    let preposition_summary = inflector.resolve(preposition)?;
    if preposition_summary.part_of_speech() != PartOfSpeech::Preposition {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "negative-pronoun interposition requires a preposition, not {}",
                preposition_summary.part_of_speech().code()
            ),
        });
    }
    let negative = Pronoun::from_id_with(&LexemeId::from(negative_id), inflector)?;
    let base_forms = separated_negative_base(negative.form(cell)?)?;
    RealizedPhrase::new(
        AnalyticConstruction::NegativePronounPrepositional,
        vec![
            PhraseToken {
                role: PhraseRole::Particle,
                forms: inflector.form_by_id(
                    &LexemeId::from("synodal:particle:negative-ni"),
                    GrammarCell::Indeclinable,
                )?,
            },
            PhraseToken {
                role: PhraseRole::Preposition,
                forms: inflector.form_by_id(preposition_summary.id(), GrammarCell::Indeclinable)?,
            },
            PhraseToken {
                role: PhraseRole::Pronoun,
                forms: base_forms,
            },
        ],
    )
}

/// Joins a previously realized host and one source-licensed short personal or
/// reflexive pronoun as a typed enclitic construction (Alypy §47).
pub fn pronoun_enclitic_after_host(
    host: FormSet,
    pronoun_id: &LexemeId,
    cell: PronounCell,
    prosody: PronounCliticProsody,
) -> Result<RealizedPhrase> {
    pronoun_enclitic_after_host_with(host, pronoun_id, cell, prosody, Inflector::default())
}

pub fn pronoun_enclitic_after_host_with(
    host: FormSet,
    pronoun_id: &LexemeId,
    cell: PronounCell,
    prosody: PronounCliticProsody,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let mut lexeme = crate::registry::pronoun_lexeme(pronoun_id)?;
    lexeme.selection = PronounFormSelection::Enclitic;
    let enclitic_expanded = decline_pronoun(&lexeme, cell, OrthographyProfile::Expanded)?
        .variants()
        .iter()
        .map(|variant| variant.expanded.clone())
        .collect::<BTreeSet<_>>();
    let registered = inflector.form_by_id(pronoun_id, GrammarCell::Pronoun(cell))?;
    let variants = registered
        .variants()
        .iter()
        .filter(|variant| enclitic_expanded.contains(&variant.expanded))
        .cloned()
        .collect::<Vec<_>>();
    if variants.is_empty() {
        return Err(Error::ContradictoryMetadata {
            reason: "the registered pronoun cell did not realize its licensed enclitic form".into(),
        });
    }
    let clitic = FormSet::try_from_variants(variants)?;
    let (host, clitic) = match prosody {
        PronounCliticProsody::AfterFinalVowelStress => {
            (stress_final_host_vowel(host)?, unaccent_enclitic(clitic)?)
        }
        PronounCliticProsody::LogicallyStressed => (
            trace_pronoun_clitic(host, "logically-stressed-host")?,
            trace_pronoun_clitic(clitic, "logically-stressed-short-pronoun")?,
        ),
    };
    RealizedPhrase::new(
        AnalyticConstruction::EncliticPronoun,
        vec![
            PhraseToken {
                role: PhraseRole::Host,
                forms: host,
            },
            PhraseToken {
                role: PhraseRole::Pronoun,
                forms: clitic,
            },
        ],
    )
}

/// Realizes the source-listed masculine singular inanimate accusative
/// contractions `на(н)и → нань` and `въ(н)и → вонь` (Alypy §47).
pub fn contracted_third_person_accusative(preposition: &str) -> Result<RealizedPhrase> {
    contracted_third_person_accusative_with(preposition, Inflector::default())
}

pub fn contracted_third_person_accusative_with(
    preposition: &str,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let preposition = inflector.resolve(preposition)?;
    if preposition.part_of_speech() != PartOfSpeech::Preposition {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "third-person contraction requires a preposition, not {}",
                preposition.part_of_speech().code()
            ),
        });
    }
    let contraction_id = match preposition.lemma() {
        "на" => "synodal:pronoun:v07-2f384f6138f1b4bd",
        "въ" => "synodal:pronoun:v07-61cdc8112ee912fe",
        other => {
            return Err(Error::HistoricallyInvalidCell {
                reason: format!(
                    "Alypy §47 explicitly licenses contracted -нь only after на and въ, not {other}"
                ),
            });
        }
    };
    let forms = append_phrase_trace(
        inflector.form_by_id(&LexemeId::from(contraction_id), GrammarCell::LexicalForm)?,
        "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47",
        "third-person-preposition-contraction",
    )?;
    RealizedPhrase::new(
        AnalyticConstruction::ThirdPersonPrepositionalContraction,
        vec![PhraseToken {
            role: PhraseRole::FusedPrepositionPronoun,
            forms,
        }],
    )
}

fn stress_final_host_vowel(forms: FormSet) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, "final-vowel-stressed-host", |text| {
        let unaccented = strip_stress_marks(text);
        let final_base = unaccented.chars().rev().find(|character| {
            !matches!(
                *character as u32,
                0x0300..=0x036f | 0x0483..=0x0489 | 0x2de0..=0x2dff | 0xfe20..=0xfe2f
            )
        });
        if !final_base.is_some_and(is_synodal_vowel) {
            return Err(Error::ContradictoryMetadata {
                reason: "AfterFinalVowelStress requires a host ending in a vowel".into(),
            });
        }
        Ok(format!("{unaccented}\u{0301}"))
    })
}

fn unaccent_enclitic(forms: FormSet) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, "unaccented-enclitic", |text| {
        Ok(strip_stress_marks(text))
    })
}

fn trace_pronoun_clitic(forms: FormSet, stage: &'static str) -> Result<FormSet> {
    transform_pronoun_clitic_forms(forms, stage, |text| Ok(text.to_owned()))
}

fn transform_pronoun_clitic_forms(
    forms: FormSet,
    stage: &'static str,
    transform: impl Fn(&str) -> Result<String>,
) -> Result<FormSet> {
    let rule = RuleId::from("SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47");
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let input = variant.printed.clone();
        variant.printed = transform(&variant.printed)?;
        variant.accented = match &variant.accented {
            Some(accented) => Some(transform(accented)?),
            None if stage == "final-vowel-stressed-host" => Some(variant.printed.clone()),
            None => None,
        };
        variant.romanization = None;
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: stage.into(),
            input,
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn strip_stress_marks(text: &str) -> String {
    text.chars()
        .filter(|character| !matches!(character, '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}'))
        .collect()
}

fn is_synodal_vowel(character: char) -> bool {
    matches!(
        character.to_lowercase().next().unwrap_or(character),
        'а' | 'е'
            | 'є'
            | 'ё'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
            | 'э'
            | 'ю'
            | 'я'
            | 'ѧ'
            | 'ѩ'
            | 'ѣ'
    )
}

fn append_phrase_trace(forms: FormSet, rule: &'static str, stage: &'static str) -> Result<FormSet> {
    let rule = RuleId::from(rule);
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: stage.into(),
            input: variant.expanded.clone(),
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn pronoun_phrase_evidence(rule: &str) -> Result<Evidence> {
    let (id, citation, note, roles) = match rule {
        "SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47" => (
            "alypy-47-pronoun-enclisis",
            "Alypy (Gamanovich), §47 note 2",
            "short-pronoun enclisis and accent",
            vec![AuthorityRole::Grammatical, AuthorityRole::Accentual],
        ),
        "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47" => (
            "alypy-47-third-person-contraction",
            "Alypy (Gamanovich), §47 note 1",
            "third-person prepositional contraction",
            vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        ),
        "SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48" => (
            "alypy-48-negative-pronoun-interposition",
            "Alypy (Gamanovich), §48",
            "negative-pronoun preposition interposition",
            vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        ),
        _ => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("unknown pronoun phrase rule {rule}"),
            });
        }
    };
    Ok(Evidence {
        id: EvidenceId::from(id),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: roles,
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: citation.into(),
        note: Some(note.into()),
    })
}

fn separated_negative_base(forms: FormSet) -> Result<FormSet> {
    let rule = RuleId::from("SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48");
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        let input = variant.printed.clone();
        variant.expanded = strip_negative_prefix(&variant.expanded)?;
        variant.printed = strip_negative_prefix(&variant.printed)?;
        variant.accented = variant
            .accented
            .as_deref()
            .map(strip_negative_prefix)
            .transpose()?;
        let construction_evidence = pronoun_phrase_evidence(rule.as_ref())?;
        if !variant
            .evidence
            .iter()
            .any(|item| item.id == construction_evidence.id)
        {
            variant.evidence.push(construction_evidence);
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: "negative-pronoun-preposition-interposition".into(),
            input,
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn strip_negative_prefix(text: &str) -> Result<String> {
    text.strip_prefix("ни")
        .or_else(|| text.strip_prefix("Ни"))
        .map(str::to_owned)
        .ok_or_else(|| Error::ContradictoryMetadata {
            reason: "negative-pronoun interposition requires a surface ни- prefix".into(),
        })
}

/// Constructs a typed analytic phrase from already inflected tokens. This is the
/// escape hatch for reviewed future, pluperfect, conditional, passive, and
/// periphrastic combinations whose auxiliary lexeme is not yet in the registry.
pub fn from_tokens(
    construction: AnalyticConstruction,
    tokens: Vec<PhraseToken>,
) -> Result<RealizedPhrase> {
    RealizedPhrase::new(construction, tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use synodal_church_slavonic_core::{
        AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, ParticipleTense,
    };

    #[test]
    fn perfect_is_structured_not_a_fake_word() {
        let phrase = perfect("нести", Person::First, Number::Singular, Gender::Masculine)
            .expect("supported phrase");
        assert_eq!(phrase.tokens().len(), 2);
        assert_eq!(phrase.primary_text(), "неслъ єсмь");
    }

    #[test]
    fn compound_future_uses_reviewed_auxiliary_and_infinitive() {
        let expanded = compound_future("нести", Person::Third, Number::Plural)
            .expect("supported compound future");
        assert_eq!(expanded.primary_text(), "имꙋтъ нести");

        let liturgical = compound_future_with(
            "нести",
            Person::Third,
            Number::Plural,
            Inflector::builder()
                .orthography(synodal_church_slavonic_core::OrthographyProfile::SynodalLiturgical)
                .build(),
        )
        .expect("accented compound future");
        assert_eq!(liturgical.primary_text(), "и҆́мꙋтъ нестѝ");
    }

    #[test]
    fn pluperfect_and_conditional_use_independent_copular_systems() {
        let pluperfect = pluperfect("писати", Person::Third, Number::Singular, Gender::Masculine)
            .expect("supported pluperfect");
        assert_eq!(pluperfect.primary_text(), "писалъ бѣ");
        let copulas = pluperfect.tokens()[1].forms.variants();
        assert_eq!(copulas.len(), 3);
        assert_eq!(
            copulas
                .iter()
                .filter(|variant| variant.is_attested())
                .count(),
            1
        );
        assert_eq!(
            copulas
                .iter()
                .filter(|variant| variant.is_predicted())
                .count(),
            2
        );

        let conditional = conditional("писати", Person::First, Number::Singular, Gender::Masculine)
            .expect("supported conditional");
        assert_eq!(conditional.primary_text(), "писалъ быхъ");
    }

    #[test]
    fn analytic_passive_is_structured_and_voice_checked() {
        let phrase = analytic_passive(
            "нести",
            ParticipleCell {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Passive,
                agreement: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            },
            Person::Third,
            Number::Singular,
        )
        .expect("reviewed passive cell");
        assert_eq!(phrase.primary_text(), "несенъ єсть");
    }

    #[test]
    fn negative_pronoun_preposition_is_a_typed_three_token_construction() {
        let phrase = negative_pronoun_prepositional(
            "ѡ",
            NegativePronounBase::Who,
            Some(PronounPostpositive::Zhe),
            PronounCell {
                case: Case::Locative,
                number: Number::Singular,
                gender: None,
                person: None,
                animacy: Animacy::Animate,
            },
        )
        .expect("Alypy §48 interposed negative pronoun");
        assert_eq!(phrase.primary_text(), "ни ѡ комъже");
        assert_eq!(
            phrase
                .tokens()
                .iter()
                .map(|token| token.role)
                .collect::<Vec<_>>(),
            [
                PhraseRole::Particle,
                PhraseRole::Preposition,
                PhraseRole::Pronoun
            ]
        );
        assert!(phrase.tokens()[2].forms.rule_traces().all(|trace| {
            trace
                .steps()
                .iter()
                .any(|step| step.rule.as_ref() == "SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48")
        }));

        let kii = negative_pronoun_prepositional(
            "въ",
            NegativePronounBase::Kii,
            None,
            PronounCell {
                case: Case::Locative,
                number: Number::Singular,
                gender: Some(Gender::Neuter),
                person: None,
                animacy: Animacy::Inanimate,
            },
        )
        .expect("negative кій interposition");
        assert_eq!(kii.primary_text(), "ни въ коемъ");

        let kotoryi = negative_pronoun_prepositional(
            "въ",
            NegativePronounBase::Kotoryi,
            None,
            PronounCell {
                case: Case::Locative,
                number: Number::Singular,
                gender: Some(Gender::Feminine),
                person: None,
                animacy: Animacy::Inanimate,
            },
        )
        .expect("negative который interposition");
        assert_eq!(kotoryi.primary_text(), "ни въ которой");

        for (base, postpositive) in [
            (NegativePronounBase::Who, PronounPostpositive::Zhdo),
            (NegativePronounBase::Kii, PronounPostpositive::Zhe),
        ] {
            assert!(matches!(
                negative_pronoun_prepositional(
                    "въ",
                    base,
                    Some(postpositive),
                    PronounCell {
                        case: Case::Locative,
                        number: Number::Singular,
                        gender: None,
                        person: None,
                        animacy: Animacy::Inanimate,
                    },
                ),
                Err(Error::ContradictoryMetadata { .. })
            ));
        }
    }

    #[test]
    fn short_pronoun_enclisis_selects_the_clitic_and_realizes_source_prosody() {
        let cell = PronounCell {
            case: Case::Accusative,
            number: Number::Singular,
            gender: None,
            person: Some(Person::First),
            animacy: Animacy::Animate,
        };
        let host = Verb::resolve("писати")
            .expect("registered verb")
            .imperative(Person::Second, Number::Singular)
            .expect("imperative host");
        let enclitic = pronoun_enclitic_after_host(
            host,
            &LexemeId::from("synodal:pronoun:az"),
            cell,
            PronounCliticProsody::AfterFinalVowelStress,
        )
        .expect("Alypy §47 final-vowel enclisis");
        assert_eq!(enclitic.primary_text(), "пиши\u{0301} мѧ");
        assert_eq!(
            enclitic.construction(),
            AnalyticConstruction::EncliticPronoun
        );
        assert_eq!(
            enclitic
                .tokens()
                .iter()
                .map(|token| token.role)
                .collect::<Vec<_>>(),
            [PhraseRole::Host, PhraseRole::Pronoun]
        );
        assert_eq!(
            enclitic.tokens()[1].forms.texts().collect::<BTreeSet<_>>(),
            BTreeSet::from(["мѧ"])
        );
        assert!(enclitic.tokens().iter().all(|token| {
            token.forms.rule_traces().all(|trace| {
                trace
                    .steps()
                    .iter()
                    .any(|step| step.rule.as_ref() == "SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47")
            })
        }));

        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let logical_host = Verb::resolve("писати")
            .expect("registered verb")
            .imperative(Person::Second, Number::Singular)
            .expect("imperative host");
        let logical = pronoun_enclitic_after_host_with(
            logical_host,
            &LexemeId::from("synodal:pronoun:az"),
            cell,
            PronounCliticProsody::LogicallyStressed,
            liturgical,
        )
        .expect("logically stressed short pronoun");
        assert!(logical.primary_text().ends_with(" мѧ̀"));
    }

    #[test]
    fn third_person_prepositional_contractions_are_typed_exact_forms() {
        let na = contracted_third_person_accusative("на").expect("нань contraction");
        assert_eq!(na.primary_text(), "нань");
        assert_eq!(
            na.construction(),
            AnalyticConstruction::ThirdPersonPrepositionalContraction
        );
        assert_eq!(na.tokens()[0].role, PhraseRole::FusedPrepositionPronoun);
        assert!(na.tokens()[0].forms.primary().is_attested());

        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        let vo = contracted_third_person_accusative_with("въ", liturgical)
            .expect("accented вонь contraction");
        assert_eq!(vo.primary_text(), "во́нь");
        assert!(vo.tokens()[0].forms.rule_traces().all(|trace| {
            trace.steps().iter().any(|step| {
                step.rule.as_ref() == "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47"
            })
        }));

        assert!(matches!(
            contracted_third_person_accusative("за"),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }
}
