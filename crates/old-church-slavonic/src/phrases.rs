//! Structured Old Church Slavonic superlative and verbal periphrases.

use old_church_slavonic_core::adjective::{AdjectiveLexeme, ComparativeLexeme};
use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveForm, AnalyticConstruction, Case, ConditionalAuxiliary, CopulaSeries,
    DirectToTreatment, FiniteTense, FormAnalysis, FormSet, FormSource, FormVariant,
    FutureInfinitiveAuxiliary, FutureReferenceTense, Gender, ImpersonalVerbIdentity,
    InflectionError, InterrogativePronounIdentity, Lemma, MetadataEvidence, MetadataProvenance,
    Number, ParticipleKind, PassiveAuxiliary, Person, PhraseOrder, PhraseRole, PhraseToken,
    PluperfectAuxiliary, PronominalFamilySpec, PronominalPostpositive, PronominalPrefix,
    RealizedPhrase, RuleId, RuleStep, Script,
};

use crate::{Verb, resolver};

const PRONOMINAL_FAMILY_AUTHORITY: &str =
    "Polivanova 2023 §§316, 380; postpositive любо examples in §316 n. 61";
const IMPERSONAL_AUTHORITY: &str = "English Wiktionary OCS impersonal sense inventory, pinned 2026-08-07; official LOVe mьněti record; Polivanova 2023 §§455–482 and OSD entries 879 and 550";

/// Build a derived form of numberless `къто` or `чьто` with explicit
/// prefixal, postpositive, direct-case, and preposition-interposition choices.
///
/// Bound formatives remain in the pronominal token. `любо` is independently
/// written. If a preposition is interposed, the prefix and preposition are also
/// independent tokens, faithfully retaining the construction's intermediate
/// status between a free sequence and a unitary wordform.
///
/// ```
/// use old_church_slavonic::{
///     Case, DirectToTreatment, InterrogativePronounIdentity, PronominalFamilySpec,
///     PronominalPostpositive, PronominalPrefix,
/// };
/// use old_church_slavonic::phrases::interrogative_pronoun_family;
///
/// let retained = interrogative_pronoun_family(
///     InterrogativePronounIdentity::Chto,
///     Case::Nominative,
///     PronominalFamilySpec {
///         prefix: Some(PronominalPrefix::Ni),
///         postpositive: Some(PronominalPostpositive::Ze),
///         direct_to: Some(DirectToTreatment::Retain),
///         preposition: None,
///     },
/// )?;
/// assert_eq!(retained.primary_text(), "ничьтоже");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn interrogative_pronoun_family(
    identity: InterrogativePronounIdentity,
    case: Case,
    spec: PronominalFamilySpec,
) -> Result<RealizedPhrase, InflectionError> {
    pronominal_family_with(resolver::interrogative_pronoun(identity, case)?, case, spec)
}

/// Compose a source-backed inflected pronominal base into a §316 derived
/// family. This generic entry point deliberately accepts a `FormSet`: callers
/// can retain the lexical identity and full evidence of any independently
/// resolved `2/p` base without asking this layer to guess its paradigm.
pub fn pronominal_family_with(
    base: FormSet,
    case: Case,
    spec: PronominalFamilySpec,
) -> Result<RealizedPhrase, InflectionError> {
    validate_pronominal_family_spec(&base, case, &spec)?;
    let interposed_preposition = spec
        .preposition
        .as_deref()
        .map(canonical_cyrillic_preposition)
        .transpose()?;
    let prefix_is_separate = interposed_preposition.is_some();
    let bound_postpositive = spec.postpositive.filter(|particle| particle.is_bound());
    let pronoun = compose_pronominal_token(
        base,
        case,
        if prefix_is_separate {
            None
        } else {
            spec.prefix
        },
        bound_postpositive,
        spec.direct_to,
    )?;

    let mut tokens = Vec::with_capacity(4);
    if prefix_is_separate {
        let Some(prefix) = spec.prefix else {
            return Err(InflectionError::InvalidInput {
                reason: "an interposed preposition requires a prefixal formative".to_string(),
            });
        };
        tokens.push(PhraseToken {
            role: PhraseRole::PrefixalFormative,
            forms: resolver::reviewed_grammar_token(
                prefix.text(),
                RuleId::PronounDerivedFamily,
                "pronoun:derived-family:interposed-prefix",
                PRONOMINAL_FAMILY_AUTHORITY,
            )?,
        });
        tokens.push(PhraseToken {
            role: PhraseRole::Preposition,
            forms: resolver::grammar_token(
                interposed_preposition
                    .as_deref()
                    .ok_or_else(|| InflectionError::InvalidInput {
                        reason: "a separated prefix requires an interposed preposition".to_string(),
                    })?,
                RuleId::PronounDerivedFamily,
                "supply the explicitly selected interposed preposition",
            )?,
        });
    }
    tokens.push(PhraseToken {
        role: PhraseRole::Pronoun,
        forms: pronoun,
    });
    if spec.postpositive == Some(PronominalPostpositive::Liubo) {
        tokens.push(PhraseToken {
            role: PhraseRole::Postpositive,
            forms: resolver::reviewed_grammar_token(
                PronominalPostpositive::Liubo.text(),
                RuleId::PronounDerivedFamily,
                "pronoun:derived-family:separate-liubo",
                PRONOMINAL_FAMILY_AUTHORITY,
            )?,
        });
    }
    RealizedPhrase::new(AnalyticConstruction::PronominalFamily, tokens)
}

pub(crate) fn single_token_pronominal_family_with(
    base: FormSet,
    case: Case,
    spec: PronominalFamilySpec,
) -> Result<FormSet, InflectionError> {
    let phrase = pronominal_family_with(base, case, spec)?;
    if phrase.tokens().len() != 1 {
        return Err(InflectionError::InvalidInput {
            reason: "the requested pronominal family is not a single orthographic word".to_string(),
        });
    }
    Ok(phrase.tokens()[0].forms.clone())
}

fn validate_pronominal_family_spec(
    base: &FormSet,
    case: Case,
    spec: &PronominalFamilySpec,
) -> Result<(), InflectionError> {
    if spec.prefix.is_none() && spec.postpositive.is_none() {
        return Err(InflectionError::InvalidInput {
            reason: "a derived pronominal family requires a prefix or postpositive".to_string(),
        });
    }
    if spec.preposition.is_some() && spec.prefix.is_none() {
        return Err(InflectionError::InvalidInput {
            reason: "a preposition can be interposed only between a prefixal formative and its pronominal base"
                .to_string(),
        });
    }
    if spec.preposition.is_some() && case == Case::Nominative {
        return Err(InflectionError::InvalidInput {
            reason: "an interposed preposition cannot govern a nominative pronominal form"
                .to_string(),
        });
    }

    let bound_postpositive = spec
        .postpositive
        .is_some_and(|particle| particle.is_bound());
    let direct_case = matches!(case, Case::Nominative | Case::Accusative);
    let all_to = base.lemma().ends_with("то") && base.texts().all(|text| text.ends_with("то"));
    let any_to = base.lemma().ends_with("то") || base.texts().any(|text| text.ends_with("то"));
    let explicit_treatment_is_licensed = direct_case && bound_postpositive && all_to;

    if spec.direct_to.is_some() && !explicit_treatment_is_licensed {
        return Err(InflectionError::InvalidInput {
            reason: "direct-case -то treatment is valid only for a uniformly -то-final nominative or accusative base before a bound postpositive"
                .to_string(),
        });
    }
    if direct_case && bound_postpositive && any_to && spec.direct_to.is_none() {
        return Err(InflectionError::InvalidInput {
            reason: "a direct -то-final base before a bound postpositive requires an explicit retain/drop treatment"
                .to_string(),
        });
    }
    Ok(())
}

fn canonical_cyrillic_preposition(preposition: &str) -> Result<String, InflectionError> {
    let lemma = Lemma::parse(preposition)?;
    if lemma.script() != Script::Cyrillic {
        return Err(InflectionError::InvalidInput {
            reason: "the interposed preposition must be one Cyrillic word".to_string(),
        });
    }
    Ok(lemma.to_string())
}

fn compose_pronominal_token(
    base: FormSet,
    case: Case,
    prefix: Option<PronominalPrefix>,
    postpositive: Option<PronominalPostpositive>,
    direct_to: Option<DirectToTreatment>,
) -> Result<FormSet, InflectionError> {
    let transform = |text: &str| -> Result<String, InflectionError> {
        let stem = match direct_to {
            Some(DirectToTreatment::Drop) => {
                text.strip_suffix("то")
                    .ok_or_else(|| InflectionError::InvalidInput {
                        reason: format!("cannot drop direct-case -то from {text:?}"),
                    })?
            }
            Some(DirectToTreatment::Retain) | None => text,
        };
        Ok(format!(
            "{}{}{}",
            prefix.map_or("", |value| value.text()),
            stem,
            postpositive.map_or("", |value| value.text())
        ))
    };
    let lemma = transform(base.lemma())?;
    let variants = base
        .variants()
        .map(|variant| {
            Ok(FormVariant {
                text: transform(&variant.text)?,
                // Composition cannot safely infer a romanization for added or
                // removed material from an arbitrary source romanization.
                romanization: None,
            })
        })
        .collect::<Result<Vec<_>, InflectionError>>()?;
    let primary = variants
        .first()
        .cloned()
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: "a pronominal base unexpectedly had no surface variants".to_string(),
        })?;
    let rule_id = RuleId::PronounDerivedFamily;
    let operation = RuleStep {
        rule_id,
        before: base.primary_text().to_string(),
        after: primary.text.clone(),
        reason: "compose the inflected pronominal base with the explicitly selected §316 formatives",
    };
    let evidence = MetadataEvidence {
        field: None,
        provenance: MetadataProvenance::ReviewedGrammarTable,
        source_feature: Some(format!(
            "pronoun:derived-family:{}:{}:{}:{}",
            case.code(),
            prefix.map_or("none", PronominalPrefix::code),
            postpositive.map_or("none", PronominalPostpositive::code),
            direct_to.map_or("none", DirectToTreatment::code),
        )),
        // This licenses the composition, not every generated surface as an
        // independent textual attestation.
        source_form: None,
        crosscheck_features: Vec::new(),
        authority: Some(PRONOMINAL_FAMILY_AUTHORITY.to_string()),
    };
    let source = FormSource::ReviewedGrammarTable { rule_id };
    let analyses = if base.analyses().is_empty() {
        vec![FormAnalysis {
            variants: variants.clone(),
            source: source.clone(),
            evidence: vec![evidence.clone()],
            trace: vec![operation.clone()],
        }]
    } else {
        base.analyses()
            .iter()
            .map(|analysis| {
                let transformed_variants = analysis
                    .variants
                    .iter()
                    .map(|variant| {
                        Ok(FormVariant {
                            text: transform(&variant.text)?,
                            romanization: None,
                        })
                    })
                    .collect::<Result<Vec<_>, InflectionError>>()?;
                let mut analysis_evidence = analysis.evidence.clone();
                analysis_evidence.push(evidence.clone());
                let mut trace = analysis.trace.clone();
                let analysis_before = analysis
                    .variants
                    .first()
                    .map(|variant| variant.text.clone())
                    .unwrap_or_else(|| base.primary_text().to_string());
                let analysis_primary = transformed_variants
                    .first()
                    .map(|variant| variant.text.clone())
                    .unwrap_or_else(|| primary.text.clone());
                trace.push(RuleStep {
                    before: analysis_before,
                    after: analysis_primary,
                    ..operation.clone()
                });
                Ok(FormAnalysis {
                    variants: transformed_variants,
                    source: source.clone(),
                    evidence: analysis_evidence,
                    trace,
                })
            })
            .collect::<Result<Vec<_>, InflectionError>>()?
    };
    let trace = if analyses.len() == 1 {
        let mut trace = base.trace().to_vec();
        trace.push(operation);
        trace
    } else {
        Vec::new()
    };
    Ok(FormSet::new(
        lemma,
        primary,
        variants.into_iter().skip(1).collect(),
        source,
        base.warnings().to_vec(),
        trace,
        analyses,
    ))
}

/// Build the usual relative superlative: a declined comparative together with
/// an independently inflected genitive reference. The caller supplies the
/// reference `FormSet`, so its dictionary identity, variants, and evidence stay
/// intact and no noun/adjective/pronoun distinction is guessed here.
pub fn relative_superlative_with(
    comparative: &ComparativeLexeme,
    cell: AdjectiveCell,
    genitive_reference: FormSet,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let head = PhraseToken {
        role: PhraseRole::ComparativeAdjective,
        forms: resolver::comparative_with(comparative, cell)?,
    };
    let dependent = PhraseToken {
        role: PhraseRole::ComparisonReference,
        forms: genitive_reference,
    };
    RealizedPhrase::new(
        AnalyticConstruction::RelativeSuperlative,
        ordered(dependent, head, order),
    )
}

/// Build the source-described absolute superlative with invariant `ѕѣло` and a
/// declined positive adjective. Both attested modifier orders are representable.
pub fn absolute_superlative_adverb_with(
    positive: &AdjectiveLexeme,
    cell: AdjectiveCell,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let head = PhraseToken {
        role: PhraseRole::PositiveAdjective,
        forms: resolver::adjective_with(positive, cell)?,
    };
    let dependent = PhraseToken {
        role: PhraseRole::Adverb,
        forms: resolver::grammar_token(
            "ѕѣло",
            RuleId::PhraseAbsoluteSuperlativeAdverb,
            "supply the invariant absolute-superlative adverb",
        )?,
    };
    RealizedPhrase::new(
        AnalyticConstruction::AbsoluteSuperlativeAdverb,
        ordered(dependent, head, order),
    )
}

/// Build a `да` + present imperative/optative for any person-number cell.
///
/// This is deliberately distinct from the six-cell synthetic imperative. OCS
/// sources use the periphrasis for missing first/third-person commands and also
/// for persons that possess a synthetic imperative when its modal force is
/// appropriate.
pub fn da_imperative(
    lemma: &str,
    person: Person,
    number: Number,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    RealizedPhrase::new(
        AnalyticConstruction::DaImperative,
        vec![
            PhraseToken {
                role: PhraseRole::Particle,
                forms: resolver::grammar_token(
                    "да",
                    RuleId::PhraseDaImperative,
                    "supply the proclitic imperative/optative particle",
                )?,
            },
            PhraseToken {
                role: PhraseRole::FiniteVerb,
                forms: verb.present(person, number)?,
            },
        ],
    )
}

/// Resolve one source-reviewed copular series. This intentionally distinguishes
/// present `ѥс-` from future `бѫд-` and both from the past and modal
/// series that dictionary headwords often aggregate under `бꙑти`.
pub fn copula(
    series: CopulaSeries,
    person: Person,
    number: Number,
) -> Result<FormSet, InflectionError> {
    resolver::copula(series, person, number)
}

/// Build the OCS perfect from an agreeing l-participle and present `ѥс-`
/// copula.
pub fn perfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    paired_phrase(
        AnalyticConstruction::Perfect,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(CopulaSeries::PresentEs, person, number)?,
        },
        PhraseToken {
            role: PhraseRole::LParticiple,
            forms: verb.l_participle(gender, number)?,
        },
        order,
    )
}

/// Build any of the three source-described pluperfect formations: l-participle
/// plus imperfect or aorist `be`, or the three-token l-participle + perfect of
/// `be` construction.
pub fn pluperfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    auxiliary: PluperfectAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    let head = PhraseToken {
        role: PhraseRole::LParticiple,
        forms: verb.l_participle(gender, number)?,
    };
    match auxiliary {
        PluperfectAuxiliary::Imperfect | PluperfectAuxiliary::Aorist => {
            let series = if auxiliary == PluperfectAuxiliary::Imperfect {
                CopulaSeries::ImperfectBe
            } else {
                CopulaSeries::AoristBe
            };
            paired_phrase(
                AnalyticConstruction::Pluperfect,
                PhraseToken {
                    role: PhraseRole::Auxiliary,
                    forms: copula(series, person, number)?,
                },
                head,
                order,
            )
        }
        PluperfectAuxiliary::Perfect => {
            let copular_verb = Verb::resolve("бꙑти")?;
            let auxiliary_participle = PhraseToken {
                role: PhraseRole::AuxiliaryParticiple,
                forms: copular_verb.l_participle(gender, number)?,
            };
            let auxiliary = PhraseToken {
                role: PhraseRole::Auxiliary,
                forms: copula(CopulaSeries::PresentEs, person, number)?,
            };
            let tokens = match order {
                PhraseOrder::HeadFirst => vec![head, auxiliary_participle, auxiliary],
                PhraseOrder::DependentFirst => vec![auxiliary_participle, auxiliary, head],
            };
            RealizedPhrase::new(AnalyticConstruction::Pluperfect, tokens)
        }
    }
}

/// Build an infinitival future. Present auxiliaries are licensed for all four
/// source-listed verbs; past-reference futures are restricted to `имѣти` and
/// `хотѣти`, as stated by the reviewed grammar.
pub fn infinitival_future(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    reference_tense: FutureReferenceTense,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    if reference_tense != FutureReferenceTense::Present
        && !matches!(
            auxiliary,
            FutureInfinitiveAuxiliary::Imeti | FutureInfinitiveAuxiliary::Khoteti
        )
    {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "{auxiliary:?} is not source-licensed as a past-reference future auxiliary"
            ),
        });
    }
    let verb = Verb::resolve(lemma)?;
    let auxiliary_verb = Verb::resolve(auxiliary.lemma())?;
    let tense = match reference_tense {
        FutureReferenceTense::Present => FiniteTense::Present,
        FutureReferenceTense::Imperfect => FiniteTense::Imperfect,
        FutureReferenceTense::Aorist => FiniteTense::Aorist,
    };
    paired_phrase(
        AnalyticConstruction::FutureInfinitive,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: auxiliary_verb.finite(tense, person, number)?,
        },
        PhraseToken {
            role: PhraseRole::Infinitive,
            forms: verb.infinitive()?,
        },
        order,
    )
}

/// Build the occasional active-participle future with future `бѫд-`.
pub fn participial_future(
    lemma: &str,
    kind: ParticipleKind,
    cell: AdjectiveCell,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    validate_predicative_participle(kind, cell, number, true)?;
    let participle = Verb::resolve(lemma)?.participle(kind)?;
    paired_phrase(
        AnalyticConstruction::FutureParticiple,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(CopulaSeries::FutureBud, person, number)?,
        },
        PhraseToken {
            role: PhraseRole::ActiveParticiple,
            forms: participle.short(cell.case, cell.number, cell.gender, cell.animacy)?,
        },
        order,
    )
}

/// Build the future perfect from an agreeing l-participle and future `бѫд-`.
pub fn future_perfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    paired_phrase(
        AnalyticConstruction::FuturePerfect,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(CopulaSeries::FutureBud, person, number)?,
        },
        PhraseToken {
            role: PhraseRole::LParticiple,
            forms: verb.l_participle(gender, number)?,
        },
        order,
    )
}

/// Build the conditional-optative from an l-participle and either the dedicated
/// conditional series or its source-described aorist replacement.
pub fn conditional_optative(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    auxiliary: ConditionalAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    conditional_optative_tokens(lemma, person, number, gender, auxiliary, order, false)
}

/// Build the independently described `да`-marked optative construction.
pub fn da_conditional_optative(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    auxiliary: ConditionalAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    conditional_optative_tokens(lemma, person, number, gender, auxiliary, order, true)
}

/// Represent the source-described ellipsis of the conditional auxiliary while
/// retaining the agreeing l-participle as a structured construction.
pub fn elliptical_conditional_optative(
    lemma: &str,
    number: Number,
    gender: Gender,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    RealizedPhrase::new(
        AnalyticConstruction::EllipticalConditionalOptative,
        vec![PhraseToken {
            role: PhraseRole::LParticiple,
            forms: verb.l_participle(gender, number)?,
        }],
    )
}

/// Build the source-attested conditional with a passive rather than
/// l-participle predicate.
pub fn conditional_passive(
    lemma: &str,
    kind: ParticipleKind,
    cell: AdjectiveCell,
    person: Person,
    number: Number,
    auxiliary: ConditionalAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    validate_predicative_participle(kind, cell, number, false)?;
    let participle = Verb::resolve(lemma)?.participle(kind)?;
    paired_phrase(
        AnalyticConstruction::ConditionalOptativePassive,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(conditional_series(auxiliary), person, number)?,
        },
        PhraseToken {
            role: PhraseRole::PassiveParticiple,
            forms: participle.short(cell.case, cell.number, cell.gender, cell.animacy)?,
        },
        order,
    )
}

/// Build a present, past, future, or modal analytic passive from an agreeing
/// passive participle and the explicitly selected copular series.
pub fn analytic_passive(
    lemma: &str,
    kind: ParticipleKind,
    cell: AdjectiveCell,
    person: Person,
    number: Number,
    auxiliary: PassiveAuxiliary,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    validate_predicative_participle(kind, cell, number, false)?;
    let participle = Verb::resolve(lemma)?.participle(kind)?;
    paired_phrase(
        AnalyticConstruction::AnalyticPassive,
        PhraseToken {
            role: PhraseRole::Auxiliary,
            forms: copula(passive_series(auxiliary), person, number)?,
        },
        PhraseToken {
            role: PhraseRole::PassiveParticiple,
            forms: participle.short(cell.case, cell.number, cell.gender, cell.animacy)?,
        },
        order,
    )
}

/// Build the finite predicate of a source-identified impersonal construction.
///
/// The construction always selects third-person singular. `достоꙗти` is a
/// one-token lexically impersonal predicate; impersonal `мьнѣти` retains the
/// independently written reflexive particle `сѧ`. Dictionary cells keep their
/// exact provenance, while a missing but regular aorist is reconstructed from
/// the reviewed lexical profile.
pub fn impersonal_predicate(
    identity: ImpersonalVerbIdentity,
    tense: FiniteTense,
) -> Result<RealizedPhrase, InflectionError> {
    let cell = identity.predicate_cell(tense);
    let forms = match resolver::finite_verb(identity.lemma(), cell) {
        Ok(forms)
            if matches!(
                forms.source(),
                FormSource::DictionaryTable | FormSource::ManualOverride
            ) =>
        {
            forms
        }
        // A generic source profile for the same spelling may belong to a
        // distinct personal sense. Once no exact cell exists, the typed
        // impersonal identity owns the productive principal parts.
        Ok(_) => {
            resolver::reviewed_finite_verb_with(&identity.lexeme(), cell, IMPERSONAL_AUTHORITY)?
        }
        Err(
            InflectionError::MissingLexicalMetadata { .. }
            | InflectionError::UnsupportedFormation { .. }
            | InflectionError::UnsupportedCell { .. }
            | InflectionError::UnattestedUnreconstructableCell { .. },
        ) => resolver::reviewed_finite_verb_with(&identity.lexeme(), cell, IMPERSONAL_AUTHORITY)?,
        Err(error) => return Err(error),
    };
    let mut tokens = vec![PhraseToken {
        role: PhraseRole::FiniteVerb,
        forms,
    }];
    if let Some(particle) = identity.reflexive_particle() {
        tokens.push(PhraseToken {
            role: PhraseRole::Particle,
            forms: resolver::reviewed_grammar_token(
                particle,
                RuleId::PhraseImpersonalPredicate,
                "verb:impersonal:reflexive-particle",
                IMPERSONAL_AUTHORITY,
            )?,
        });
    }
    RealizedPhrase::new(AnalyticConstruction::ImpersonalPredicate, tokens)
}

fn conditional_optative_tokens(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    auxiliary: ConditionalAuxiliary,
    order: PhraseOrder,
    with_da: bool,
) -> Result<RealizedPhrase, InflectionError> {
    let verb = Verb::resolve(lemma)?;
    let dependent = PhraseToken {
        role: PhraseRole::Auxiliary,
        forms: copula(conditional_series(auxiliary), person, number)?,
    };
    let head = PhraseToken {
        role: PhraseRole::LParticiple,
        forms: verb.l_participle(gender, number)?,
    };
    let mut tokens = ordered(dependent, head, order);
    let construction = if with_da {
        tokens.insert(
            0,
            PhraseToken {
                role: PhraseRole::Particle,
                forms: resolver::grammar_token(
                    "да",
                    RuleId::PhraseConditionalOptativeDa,
                    "supply the optative particle",
                )?,
            },
        );
        AnalyticConstruction::DaConditionalOptative
    } else {
        AnalyticConstruction::ConditionalOptative
    };
    RealizedPhrase::new(construction, tokens)
}

fn conditional_series(auxiliary: ConditionalAuxiliary) -> CopulaSeries {
    match auxiliary {
        ConditionalAuxiliary::Conditional => CopulaSeries::ConditionalBi,
        ConditionalAuxiliary::AoristReplacement => CopulaSeries::ConditionalAoristBy,
    }
}

fn passive_series(auxiliary: PassiveAuxiliary) -> CopulaSeries {
    match auxiliary {
        PassiveAuxiliary::Present => CopulaSeries::PresentEs,
        PassiveAuxiliary::Imperfect => CopulaSeries::ImperfectBe,
        PassiveAuxiliary::Aorist => CopulaSeries::AoristBe,
        PassiveAuxiliary::Future => CopulaSeries::FutureBud,
        PassiveAuxiliary::Conditional => CopulaSeries::ConditionalBi,
        PassiveAuxiliary::ConditionalAoristReplacement => CopulaSeries::ConditionalAoristBy,
    }
}

fn validate_predicative_participle(
    kind: ParticipleKind,
    cell: AdjectiveCell,
    subject_number: Number,
    active: bool,
) -> Result<(), InflectionError> {
    let valid_kind = if active {
        matches!(
            kind,
            ParticipleKind::PresentActive | ParticipleKind::PastActive
        )
    } else {
        matches!(
            kind,
            ParticipleKind::PresentPassive | ParticipleKind::PastPassive
        )
    };
    if !valid_kind {
        return Err(InflectionError::InvalidInput {
            reason: format!(
                "the requested analytic construction requires an {} participle",
                if active { "active" } else { "passive" }
            ),
        });
    }
    if cell.case != Case::Nominative
        || cell.form != AdjectiveForm::Short
        || cell.number != subject_number
    {
        return Err(InflectionError::InvalidInput {
            reason: "a predicative participle must be short nominative and agree with the subject number"
                .to_string(),
        });
    }
    Ok(())
}

fn paired_phrase(
    construction: AnalyticConstruction,
    dependent: PhraseToken,
    head: PhraseToken,
    order: PhraseOrder,
) -> Result<RealizedPhrase, InflectionError> {
    RealizedPhrase::new(construction, ordered(dependent, head, order))
}

fn ordered(dependent: PhraseToken, head: PhraseToken, order: PhraseOrder) -> Vec<PhraseToken> {
    match order {
        PhraseOrder::DependentFirst => vec![dependent, head],
        PhraseOrder::HeadFirst => vec![head, dependent],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use old_church_slavonic_core::adjective::productive_new_comparative;
    use old_church_slavonic_core::{
        AdjectiveClass, AdjectiveForm, Animacy, Case, DirectToTreatment, Gender, Number,
        PronominalFamilySpec, PronominalPostpositive, PronominalPrefix,
    };

    fn nominative_masculine_short() -> AdjectiveCell {
        AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        }
    }

    fn predicative_cell(number: Number, gender: Gender) -> AdjectiveCell {
        AdjectiveCell {
            case: Case::Nominative,
            number,
            gender,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        }
    }

    #[test]
    fn superlative_strategies_keep_component_provenance_and_order() {
        let positive = AdjectiveLexeme {
            lemma: "свѧтъ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let absolute = absolute_superlative_adverb_with(
            &positive,
            nominative_masculine_short(),
            PhraseOrder::HeadFirst,
        )
        .expect("absolute superlative");
        assert_eq!(absolute.primary_text(), "свѧтъ ѕѣло");
        assert_eq!(absolute.tokens().len(), 2);
        assert_eq!(absolute.rule_id(), RuleId::PhraseAbsoluteSuperlativeAdverb);

        let comparative = productive_new_comparative(&positive).expect("new comparative");
        let reference = resolver::grammar_token(
            "вьсѣхъ",
            RuleId::PhraseRelativeSuperlative,
            "supply an explicit genitive comparison reference",
        )
        .expect("reference token");
        let relative = relative_superlative_with(
            &comparative,
            nominative_masculine_short(),
            reference,
            PhraseOrder::DependentFirst,
        )
        .expect("relative superlative");
        assert_eq!(relative.primary_text(), "вьсѣхъ свѧтѣи");
        assert_eq!(relative.tokens()[0].forms.primary_text(), "вьсѣхъ");
        assert_eq!(relative.tokens()[1].role, PhraseRole::ComparativeAdjective);
    }

    #[test]
    fn derived_interrogative_families_cover_prefixes_and_bound_postpositives() {
        let negative = interrogative_pronoun_family(
            InterrogativePronounIdentity::Kto,
            Case::Dative,
            PronominalFamilySpec {
                prefix: Some(PronominalPrefix::Ni),
                ..PronominalFamilySpec::default()
            },
        )
        .expect("negative family");
        assert_eq!(negative.primary_text(), "никому");
        assert_eq!(negative.tokens().len(), 1);
        assert_eq!(negative.rule_id(), RuleId::PronounDerivedFamily);

        let indefinite = interrogative_pronoun_family(
            InterrogativePronounIdentity::Kto,
            Case::Nominative,
            PronominalFamilySpec {
                prefix: Some(PronominalPrefix::Ne),
                ..PronominalFamilySpec::default()
            },
        )
        .expect("indefinite family");
        assert_eq!(indefinite.primary_text(), "нѣкъто");

        for (postpositive, expected) in [
            (PronominalPostpositive::Ze, "къже"),
            (PronominalPostpositive::Zhde, "къжде"),
            (PronominalPostpositive::Zhydo, "къжьдо"),
        ] {
            let family = interrogative_pronoun_family(
                InterrogativePronounIdentity::Kto,
                Case::Nominative,
                PronominalFamilySpec {
                    postpositive: Some(postpositive),
                    direct_to: Some(DirectToTreatment::Drop),
                    ..PronominalFamilySpec::default()
                },
            )
            .expect("bound postpositive family");
            assert_eq!(family.primary_text(), expected);
        }

        let retained = interrogative_pronoun_family(
            InterrogativePronounIdentity::Chto,
            Case::Accusative,
            PronominalFamilySpec {
                prefix: Some(PronominalPrefix::Ni),
                postpositive: Some(PronominalPostpositive::Ze),
                direct_to: Some(DirectToTreatment::Retain),
                ..PronominalFamilySpec::default()
            },
        )
        .expect("retained direct -to");
        assert_eq!(retained.primary_text(), "ничьтоже");
        let dropped = interrogative_pronoun_family(
            InterrogativePronounIdentity::Chto,
            Case::Accusative,
            PronominalFamilySpec {
                prefix: Some(PronominalPrefix::Ni),
                postpositive: Some(PronominalPostpositive::Ze),
                direct_to: Some(DirectToTreatment::Drop),
                ..PronominalFamilySpec::default()
            },
        )
        .expect("dropped direct -to");
        assert_eq!(dropped.primary_text(), "ничьже");
    }

    #[test]
    fn derived_pronominal_sequences_preserve_token_and_variant_structure() {
        let interposed = interrogative_pronoun_family(
            InterrogativePronounIdentity::Kto,
            Case::Locative,
            PronominalFamilySpec {
                prefix: Some(PronominalPrefix::Ni),
                postpositive: Some(PronominalPostpositive::Ze),
                preposition: Some("о".to_string()),
                direct_to: None,
            },
        )
        .expect("interposed preposition family");
        assert_eq!(interposed.primary_text(), "ни о комьже");
        assert_eq!(
            interposed
                .tokens()
                .iter()
                .map(|token| token.role)
                .collect::<Vec<_>>(),
            [
                PhraseRole::PrefixalFormative,
                PhraseRole::Preposition,
                PhraseRole::Pronoun,
            ]
        );
        assert_eq!(
            interposed.tokens()[0].forms.source(),
            &FormSource::ReviewedGrammarTable {
                rule_id: RuleId::PronounDerivedFamily,
            }
        );
        assert_eq!(
            interposed.tokens()[1].forms.source(),
            &FormSource::ExplicitMetadataRule {
                rule_id: RuleId::PronounDerivedFamily,
            }
        );

        let liubo = interrogative_pronoun_family(
            InterrogativePronounIdentity::Chto,
            Case::Genitive,
            PronominalFamilySpec {
                postpositive: Some(PronominalPostpositive::Liubo),
                ..PronominalFamilySpec::default()
            },
        )
        .expect("independent liubo family");
        assert_eq!(liubo.primary_text(), "чесо любо");
        assert_eq!(liubo.tokens().len(), 2);
        assert_eq!(
            liubo.tokens()[0].forms.texts().collect::<Vec<_>>(),
            ["чесо", "чьсо", "чесого"]
        );
        assert_eq!(liubo.tokens()[1].role, PhraseRole::Postpositive);
        assert_eq!(liubo.tokens()[0].forms.analyses().len(), 3);
        assert!(liubo.tokens()[0].forms.trace().is_empty());
        assert!(liubo.tokens()[0].forms.analyses().iter().all(|analysis| {
            analysis.evidence.len() == 2
                && analysis
                    .trace
                    .last()
                    .is_some_and(|step| step.rule_id == RuleId::PronounDerivedFamily)
        }));
    }

    #[test]
    fn derived_pronominal_family_rejects_underspecified_or_malformed_choices() {
        assert!(matches!(
            interrogative_pronoun_family(
                InterrogativePronounIdentity::Kto,
                Case::Nominative,
                PronominalFamilySpec::default(),
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            interrogative_pronoun_family(
                InterrogativePronounIdentity::Kto,
                Case::Nominative,
                PronominalFamilySpec {
                    prefix: Some(PronominalPrefix::Ni),
                    preposition: Some("о".to_string()),
                    ..PronominalFamilySpec::default()
                },
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            interrogative_pronoun_family(
                InterrogativePronounIdentity::Kto,
                Case::Locative,
                PronominalFamilySpec {
                    postpositive: Some(PronominalPostpositive::Ze),
                    preposition: Some("о".to_string()),
                    ..PronominalFamilySpec::default()
                },
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            interrogative_pronoun_family(
                InterrogativePronounIdentity::Kto,
                Case::Nominative,
                PronominalFamilySpec {
                    postpositive: Some(PronominalPostpositive::Ze),
                    ..PronominalFamilySpec::default()
                },
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            interrogative_pronoun_family(
                InterrogativePronounIdentity::Kto,
                Case::Genitive,
                PronominalFamilySpec {
                    postpositive: Some(PronominalPostpositive::Ze),
                    direct_to: Some(DirectToTreatment::Drop),
                    ..PronominalFamilySpec::default()
                },
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            interrogative_pronoun_family(
                InterrogativePronounIdentity::Kto,
                Case::Dative,
                PronominalFamilySpec {
                    prefix: Some(PronominalPrefix::Ni),
                    preposition: Some("о въ".to_string()),
                    ..PronominalFamilySpec::default()
                },
            ),
            Err(InflectionError::InvalidLemma { .. })
        ));

        let separate = interrogative_pronoun_family(
            InterrogativePronounIdentity::Kto,
            Case::Nominative,
            PronominalFamilySpec {
                postpositive: Some(PronominalPostpositive::Liubo),
                ..PronominalFamilySpec::default()
            },
        )
        .expect("separate postpositive does not trigger -to ambiguity");
        assert_eq!(separate.primary_text(), "къто любо");
    }

    #[test]
    fn prefixed_absolute_superlative_remains_one_inflected_word() {
        let positive = AdjectiveLexeme {
            lemma: "свѧтъ".to_string(),
            class: AdjectiveClass::Hard,
        };
        let form = resolver::pre_superlative_with(&positive, nominative_masculine_short())
            .expect("prefixed superlative");
        assert_eq!(form.primary_text(), "прѣсвѧтъ");
        assert_eq!(
            form.source(),
            &old_church_slavonic_core::FormSource::ExplicitMetadataRule {
                rule_id: RuleId::AdjectiveSuperlativePre,
            }
        );
        assert_eq!(form.trace().len(), 2);
    }

    #[test]
    fn da_imperative_covers_every_person_number_cell() {
        let phrases = Number::ALL
            .into_iter()
            .flat_map(|number| {
                Person::ALL
                    .into_iter()
                    .map(move |person| da_imperative("благословити", person, number))
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("all analytic imperative cells");
        assert_eq!(phrases.len(), 9);
        assert!(
            phrases
                .iter()
                .all(|phrase| phrase.primary_text().starts_with("да "))
        );
        assert_eq!(phrases[0].primary_text(), "да благословлѭ");
        assert_eq!(phrases[8].primary_text(), "да благословѧтъ");
    }

    #[test]
    fn copular_series_are_complete_and_keep_reconstruction_visible() {
        let forms = CopulaSeries::ALL
            .into_iter()
            .flat_map(|series| {
                Number::ALL.into_iter().flat_map(move |number| {
                    Person::ALL
                        .into_iter()
                        .map(move |person| copula(series, person, number))
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .expect("all six complete copular series");
        assert_eq!(forms.len(), 54);
        assert_eq!(forms[0].primary_text(), "ѥсмь");
        assert_eq!(
            copula(CopulaSeries::FutureBud, Person::Third, Number::Plural)
                .expect("future copula")
                .primary_text(),
            "бѫдѫтъ"
        );
        let reconstructed = copula(CopulaSeries::ConditionalBi, Person::First, Number::Dual)
            .expect("reconstructed conditional dual");
        assert_eq!(reconstructed.primary_text(), "бивѣ");
        assert_eq!(
            reconstructed.source(),
            &old_church_slavonic_core::FormSource::ExplicitMetadataRule {
                rule_id: RuleId::VerbCopulaConditionalBi,
            }
        );
        assert!(
            reconstructed
                .warnings()
                .contains(&old_church_slavonic_core::InflectionWarning::IncludesReconstructedForms)
        );
        assert_eq!(reconstructed.analyses()[0].trace.len(), 1);
    }

    #[test]
    fn perfect_pluperfect_future_perfect_and_conditionals_cover_all_cells() {
        let mut perfect_count = 0;
        let mut pluperfect_count = 0;
        let mut future_perfect_count = 0;
        let mut conditional_count = 0;
        let mut da_conditional_count = 0;
        for number in Number::ALL {
            for person in Person::ALL {
                for gender in Gender::ALL {
                    perfect(
                        "благословити",
                        person,
                        number,
                        gender,
                        PhraseOrder::HeadFirst,
                    )
                    .expect("perfect cell");
                    perfect_count += 1;
                    future_perfect(
                        "благословити",
                        person,
                        number,
                        gender,
                        PhraseOrder::HeadFirst,
                    )
                    .expect("future-perfect cell");
                    future_perfect_count += 1;
                    for auxiliary in [
                        PluperfectAuxiliary::Imperfect,
                        PluperfectAuxiliary::Aorist,
                        PluperfectAuxiliary::Perfect,
                    ] {
                        pluperfect(
                            "благословити",
                            person,
                            number,
                            gender,
                            auxiliary,
                            PhraseOrder::HeadFirst,
                        )
                        .expect("pluperfect cell");
                        pluperfect_count += 1;
                    }
                    for auxiliary in [
                        ConditionalAuxiliary::Conditional,
                        ConditionalAuxiliary::AoristReplacement,
                    ] {
                        conditional_optative(
                            "благословити",
                            person,
                            number,
                            gender,
                            auxiliary,
                            PhraseOrder::DependentFirst,
                        )
                        .expect("conditional cell");
                        conditional_count += 1;
                        da_conditional_optative(
                            "благословити",
                            person,
                            number,
                            gender,
                            auxiliary,
                            PhraseOrder::DependentFirst,
                        )
                        .expect("da conditional cell");
                        da_conditional_count += 1;
                    }
                }
            }
        }
        assert_eq!(perfect_count, 27);
        assert_eq!(pluperfect_count, 81);
        assert_eq!(future_perfect_count, 27);
        assert_eq!(conditional_count, 54);
        assert_eq!(da_conditional_count, 54);
        let elliptical_count = Number::ALL
            .into_iter()
            .flat_map(|number| {
                Gender::ALL.into_iter().map(move |gender| {
                    elliptical_conditional_optative("благословити", number, gender)
                        .expect("elliptical conditional cell")
                })
            })
            .count();
        assert_eq!(elliptical_count, 9);
        assert_eq!(
            perfect(
                "благословити",
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::HeadFirst,
            )
            .expect("perfect")
            .primary_text(),
            "благословилъ ѥсмь"
        );
        assert_eq!(
            elliptical_conditional_optative("благословити", Number::Plural, Gender::Feminine,)
                .expect("elliptical conditional")
                .tokens()
                .len(),
            1
        );
    }

    #[test]
    fn all_licensed_infinitival_future_auxiliary_cells_are_realized() {
        let mut count = 0;
        for auxiliary in FutureInfinitiveAuxiliary::ALL {
            for number in Number::ALL {
                for person in Person::ALL {
                    infinitival_future(
                        "благословити",
                        auxiliary,
                        FutureReferenceTense::Present,
                        person,
                        number,
                        PhraseOrder::DependentFirst,
                    )
                    .expect("present-reference future");
                    count += 1;
                    if matches!(
                        auxiliary,
                        FutureInfinitiveAuxiliary::Imeti | FutureInfinitiveAuxiliary::Khoteti
                    ) {
                        for tense in [
                            FutureReferenceTense::Imperfect,
                            FutureReferenceTense::Aorist,
                        ] {
                            infinitival_future(
                                "благословити",
                                auxiliary,
                                tense,
                                person,
                                number,
                                PhraseOrder::DependentFirst,
                            )
                            .expect("past-reference future");
                            count += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(count, 72);
        let imeti_future = infinitival_future(
            "благословити",
            FutureInfinitiveAuxiliary::Imeti,
            FutureReferenceTense::Present,
            Person::Third,
            Number::Plural,
            PhraseOrder::DependentFirst,
        )
        .expect("имѣти future");
        assert_eq!(imeti_future.primary_text(), "имѫтъ благословити");
        assert_eq!(
            imeti_future.tokens()[0].forms.source(),
            &old_church_slavonic_core::FormSource::ManualOverride
        );
        assert_eq!(
            imeti_future.tokens()[0].forms.texts().collect::<Vec<_>>(),
            vec!["имѫтъ", "имѣютъ"]
        );
        let khoteti_past_future = infinitival_future(
            "благословити",
            FutureInfinitiveAuxiliary::Khoteti,
            FutureReferenceTense::Aorist,
            Person::First,
            Number::Dual,
            PhraseOrder::DependentFirst,
        )
        .expect("хотѣти future-in-the-past");
        assert_eq!(
            khoteti_past_future.tokens()[0].forms.source(),
            &old_church_slavonic_core::FormSource::ManualOverride
        );
        assert_eq!(
            khoteti_past_future.tokens()[0].forms.primary_text(),
            "хотѣховѣ"
        );
        assert!(matches!(
            infinitival_future(
                "благословити",
                FutureInfinitiveAuxiliary::Vochati,
                FutureReferenceTense::Imperfect,
                Person::First,
                Number::Singular,
                PhraseOrder::DependentFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
    }

    #[test]
    fn participial_futures_and_passives_enforce_and_cover_agreement() {
        let mut active_count = 0;
        let mut passive_count = 0;
        let mut conditional_passive_count = 0;
        for number in Number::ALL {
            for person in Person::ALL {
                for gender in Gender::ALL {
                    let cell = predicative_cell(number, gender);
                    for kind in [ParticipleKind::PresentActive, ParticipleKind::PastActive] {
                        participial_future(
                            "благословити",
                            kind,
                            cell,
                            person,
                            number,
                            PhraseOrder::HeadFirst,
                        )
                        .expect("active-participle future cell");
                        active_count += 1;
                    }
                    for kind in [ParticipleKind::PresentPassive, ParticipleKind::PastPassive] {
                        for auxiliary in [
                            PassiveAuxiliary::Present,
                            PassiveAuxiliary::Imperfect,
                            PassiveAuxiliary::Aorist,
                            PassiveAuxiliary::Future,
                            PassiveAuxiliary::Conditional,
                            PassiveAuxiliary::ConditionalAoristReplacement,
                        ] {
                            analytic_passive(
                                "благословити",
                                kind,
                                cell,
                                person,
                                number,
                                auxiliary,
                                PhraseOrder::HeadFirst,
                            )
                            .expect("analytic passive cell");
                            passive_count += 1;
                        }
                        for auxiliary in [
                            ConditionalAuxiliary::Conditional,
                            ConditionalAuxiliary::AoristReplacement,
                        ] {
                            conditional_passive(
                                "благословити",
                                kind,
                                cell,
                                person,
                                number,
                                auxiliary,
                                PhraseOrder::HeadFirst,
                            )
                            .expect("conditional passive cell");
                            conditional_passive_count += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(active_count, 54);
        assert_eq!(passive_count, 324);
        assert_eq!(conditional_passive_count, 108);
        assert!(matches!(
            participial_future(
                "благословити",
                ParticipleKind::PastPassive,
                predicative_cell(Number::Singular, Gender::Masculine),
                Person::Third,
                Number::Singular,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PresentActive,
                predicative_cell(Number::Singular, Gender::Masculine),
                Person::Third,
                Number::Singular,
                PassiveAuxiliary::Present,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PastPassive,
                AdjectiveCell {
                    form: AdjectiveForm::Long,
                    ..predicative_cell(Number::Singular, Gender::Masculine)
                },
                Person::Third,
                Number::Singular,
                PassiveAuxiliary::Present,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PastPassive,
                AdjectiveCell {
                    case: Case::Genitive,
                    ..predicative_cell(Number::Singular, Gender::Masculine)
                },
                Person::Third,
                Number::Singular,
                PassiveAuxiliary::Present,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PastPassive,
                predicative_cell(Number::Plural, Gender::Masculine),
                Person::Third,
                Number::Singular,
                PassiveAuxiliary::Present,
                PhraseOrder::HeadFirst,
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
    }

    #[test]
    fn impersonal_predicates_keep_lexical_and_reflexive_structures_distinct() {
        let dostojati =
            impersonal_predicate(ImpersonalVerbIdentity::Dostojati, FiniteTense::Present)
                .expect("lexically impersonal predicate");
        assert_eq!(dostojati.primary_text(), "достоитъ");
        assert_eq!(dostojati.tokens().len(), 1);
        assert_eq!(dostojati.tokens()[0].role, PhraseRole::FiniteVerb);
        assert_eq!(
            dostojati.tokens()[0].forms.source(),
            &FormSource::DictionaryTable
        );

        let mneti =
            impersonal_predicate(ImpersonalVerbIdentity::MnetiReflexive, FiniteTense::Present)
                .expect("reflexive impersonal predicate");
        assert_eq!(mneti.primary_text(), "мьнитъ сѧ");
        assert_eq!(mneti.tokens().len(), 2);
        assert_eq!(mneti.tokens()[0].role, PhraseRole::FiniteVerb);
        assert_eq!(mneti.tokens()[1].role, PhraseRole::Particle);
        assert_eq!(mneti.rule_id(), RuleId::PhraseImpersonalPredicate);
    }

    #[test]
    fn impersonal_predicate_covers_every_finite_tense_with_provenance() {
        for identity in ImpersonalVerbIdentity::ALL {
            for tense in FiniteTense::ALL {
                let phrase = impersonal_predicate(identity, tense)
                    .unwrap_or_else(|error| panic!("{identity:?} {tense:?}: {error:?}"));
                assert_eq!(
                    phrase.tokens().len(),
                    usize::from(identity.reflexive_particle().is_some()) + 1
                );
            }
        }

        let reconstructed =
            impersonal_predicate(ImpersonalVerbIdentity::Dostojati, FiniteTense::Aorist)
                .expect("reviewed reconstructable aorist");
        assert_eq!(reconstructed.primary_text(), "достоꙗ");
        assert!(matches!(
            reconstructed.tokens()[0].forms.source(),
            FormSource::ReviewedGrammarTable { .. }
        ));
        assert_eq!(
            reconstructed.tokens()[0].forms.analyses()[0].evidence[0]
                .authority
                .as_deref(),
            Some(IMPERSONAL_AUTHORITY)
        );
    }
}
