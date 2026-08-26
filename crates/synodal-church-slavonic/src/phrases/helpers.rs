use synodal_church_slavonic_core::{
    AuthorityRole, CompoundAuxiliaryOrder, CompoundFutureAuxiliary, EpistemicRole, Error, Evidence,
    EvidenceId, EvidenceKind, FormSet, GrammarCell, LexemeId, Number, PassiveAgentGovernment,
    Person, PhraseFormation, PhraseOrder, PhraseToken, RealizedPhrase, Recension, Result, RuleId,
    SourceId, TraceStep,
};

#[allow(unused_imports)]
use super::*;
use crate::{Inflector, PartOfSpeech, Verb};

pub(super) fn byti(inflector: Inflector) -> Result<Verb> {
    Verb::from_id_with(&LexemeId::from("synodal:verb:byti"), inflector)
}

pub(super) fn copula_be(person: Person, number: Number, inflector: Inflector) -> Result<FormSet> {
    select_evidence(
        byti(inflector)?.imperfect(person, number)?,
        "alypy-81-byti-imperfect-be",
    )
}

pub(super) fn copula_bya(person: Person, number: Number, inflector: Inflector) -> Result<FormSet> {
    select_evidence(
        byti(inflector)?.imperfect(person, number)?,
        "alypy-81-byti-imperfect-bya",
    )
}

pub(super) fn invariant_by(inflector: Inflector) -> Result<FormSet> {
    let forms = byti(inflector)?.aorist(Person::Second, Number::Singular)?;
    let variants = forms
        .variants()
        .iter()
        .filter(|variant| variant.expanded == "бы")
        .cloned()
        .collect();
    FormSet::try_from_variants(variants)
}

pub(super) fn select_evidence(forms: FormSet, evidence_id: &str) -> Result<FormSet> {
    let variants = forms
        .variants()
        .iter()
        .filter(|variant| {
            variant
                .evidence
                .iter()
                .any(|evidence| evidence.id.as_ref() == evidence_id)
        })
        .cloned()
        .collect();
    FormSet::try_from_variants(variants)
}

pub(super) fn indeclinable(
    lemma: &str,
    expected: PartOfSpeech,
    inflector: Inflector,
) -> Result<FormSet> {
    let summary = inflector.resolve(lemma)?;
    if summary.part_of_speech() != expected {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "analytic token {lemma:?} must be {}, not {}",
                expected.code(),
                summary.part_of_speech().code()
            ),
        });
    }
    inflector.form_by_id(summary.id(), GrammarCell::Indeclinable)
}

pub(super) fn ordered_pair(
    predicate: PhraseToken,
    auxiliary: PhraseToken,
    order: PhraseOrder,
) -> Vec<PhraseToken> {
    match order {
        PhraseOrder::AuxiliaryFirst => vec![auxiliary, predicate],
        PhraseOrder::PredicateFirst => vec![predicate, auxiliary],
    }
}

pub(super) fn ordered_compound(
    predicate: PhraseToken,
    participle: PhraseToken,
    finite: PhraseToken,
    order: CompoundAuxiliaryOrder,
) -> Vec<PhraseToken> {
    match order {
        CompoundAuxiliaryOrder::PredicateParticipleFinite => {
            vec![predicate, participle, finite]
        }
        CompoundAuxiliaryOrder::PredicateFiniteParticiple => {
            vec![predicate, finite, participle]
        }
        CompoundAuxiliaryOrder::ParticipleFinitePredicate => {
            vec![participle, finite, predicate]
        }
        CompoundAuxiliaryOrder::FiniteParticiplePredicate => {
            vec![finite, participle, predicate]
        }
    }
}

pub(super) fn typed_phrase(
    formation: PhraseFormation,
    tokens: Vec<PhraseToken>,
) -> Result<RealizedPhrase> {
    typed_phrase_inner(formation, tokens, None)
}

pub(super) fn typed_phrase_with_government(
    formation: PhraseFormation,
    tokens: Vec<PhraseToken>,
    government: PassiveAgentGovernment,
) -> Result<RealizedPhrase> {
    typed_phrase_inner(formation, tokens, Some(government))
}

pub(super) fn typed_phrase_inner(
    formation: PhraseFormation,
    tokens: Vec<PhraseToken>,
    government: Option<PassiveAgentGovernment>,
) -> Result<RealizedPhrase> {
    let (rule, evidence) = analytic_phrase_evidence(formation);
    let tokens = tokens
        .into_iter()
        .map(|mut token| {
            token.forms = append_analytic_trace(token.forms, rule, evidence.clone())?;
            Ok(token)
        })
        .collect::<Result<Vec<_>>>()?;
    RealizedPhrase::new_typed_with_government(formation, tokens, government)
}

pub(super) fn append_analytic_trace(
    forms: FormSet,
    rule: &'static str,
    evidence: Evidence,
) -> Result<FormSet> {
    let rule = RuleId::from(rule);
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        if !variant.evidence.iter().any(|item| item.id == evidence.id) {
            variant.evidence.push(evidence.clone());
        }
        let evidence_ids = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule.clone(),
            stage: "compose-typed-analytic-phrase".into(),
            input: variant.expanded.clone(),
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: evidence_ids,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

pub(super) fn analytic_phrase_evidence(formation: PhraseFormation) -> (&'static str, Evidence) {
    let (rule, id, source, citation) = match formation {
        PhraseFormation::CompoundFuture(CompoundFutureAuxiliary::Byti) => (
            "SYN-PHRASE-FUTURE-ALYPY-85-PK-13",
            "syn-phrase-future-alypy-85-pk-13",
            "pletneva-kravetsky-church-slavonic-2005",
            "Pletneva–Kravetsky, lesson 13, table 36",
        ),
        PhraseFormation::CompoundFuture(_) => (
            "SYN-PHRASE-FUTURE-ALYPY-85-PK-13",
            "syn-phrase-future-alypy-85-pk-13",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §85; Pletneva–Kravetsky lesson 13",
        ),
        PhraseFormation::Perfect(_) => (
            "SYN-PHRASE-PERFECT-ALYPY-88",
            "syn-phrase-perfect-alypy-88",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §88",
        ),
        PhraseFormation::Pluperfect(_) => (
            "SYN-PHRASE-PLUPERFECT-ALYPY-89-168",
            "syn-phrase-pluperfect-alypy-89-168",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §§89 and 168",
        ),
        PhraseFormation::FutureAnterior => (
            "SYN-PHRASE-FUTURE-ANTERIOR-ALYPY-162",
            "syn-phrase-future-anterior-alypy-162",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §162",
        ),
        PhraseFormation::Conditional(_) => (
            "SYN-PHRASE-CONDITIONAL-ALYPY-91",
            "syn-phrase-conditional-alypy-91",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §91",
        ),
        PhraseFormation::Optative(_) => (
            "SYN-PHRASE-OPTATIVE-ALYPY-92",
            "syn-phrase-optative-alypy-92",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §92",
        ),
        PhraseFormation::AnalyticPassive(_) => (
            "SYN-PHRASE-PASSIVE-ALYPY-101-102",
            "syn-phrase-passive-alypy-101-102",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §§101–102",
        ),
        PhraseFormation::PeriphrasticTense(_) => (
            "SYN-PHRASE-PERIPHRASTIC-ALYPY-90-163",
            "syn-phrase-periphrastic-alypy-90-163",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §§90 and 163",
        ),
        PhraseFormation::CopulaEllipsis(_) => (
            "SYN-PHRASE-COPULA-ELLIPSIS-ALYPY-123-124",
            "syn-phrase-copula-ellipsis-alypy-123-124",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §§123–124",
        ),
        PhraseFormation::CompositeAdverbialParticiple(_) => (
            "SYN-PHRASE-ADVERBIAL-PARTICIPLE-ALYPY-146",
            "syn-phrase-adverbial-participle-alypy-146",
            "alypy-gamanovich-grammar-web-2023",
            "Alypy §146",
        ),
    };
    (
        rule,
        Evidence {
            id: EvidenceId::from(id),
            source: SourceId::from(source),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::NormativeRule,
            authority_roles: vec![AuthorityRole::Grammatical],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: citation.into(),
            note: Some("typed analytic-construction contract".into()),
        },
    )
}
