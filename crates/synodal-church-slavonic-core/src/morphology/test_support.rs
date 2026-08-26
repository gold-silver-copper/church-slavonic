use crate::{
    Animacy, Case, EvidenceKind, FormSet, FormSource, Number, OrthographyProfile, Recension,
    SynodalWord,
};

use super::*;

use crate::NounCell;

pub(crate) fn word(value: &str) -> SynodalWord {
    SynodalWord::parse(value).expect("test spelling")
}

pub(crate) fn assert_noun_paradigm(lexeme: &NounLexeme, animacy: Animacy, expected: &[&[&str]]) {
    assert_eq!(expected.len(), Number::ALL.len() * Case::ALL.len());
    for (index, (number, case)) in Number::ALL
        .into_iter()
        .flat_map(|number| Case::ALL.into_iter().map(move |case| (number, case)))
        .enumerate()
    {
        let forms = decline_noun(
            lexeme,
            NounCell {
                case,
                number,
                animacy,
            },
            OrthographyProfile::Expanded,
        )
        .unwrap_or_else(|error| panic!("{number:?} {case:?}: {error}"));
        let actual = forms
            .variants()
            .iter()
            .map(|variant| variant.printed.as_str())
            .collect::<Vec<_>>();
        assert_eq!(actual.as_slice(), expected[index], "{number:?} {case:?}");
    }
}

pub(crate) fn assert_productive_contract(forms: &FormSet) {
    assert!(forms.variants().iter().all(|variant| {
        matches!(
            &variant.source,
            FormSource::SynodalNormativeGeneration { rule } if !rule.to_string().is_empty()
        ) && variant.target_recension == Recension::SynodalRussian
            && variant.source_recension == Some(Recension::SynodalRussian)
            && !variant.evidence.is_empty()
            && variant.evidence.iter().all(|evidence| {
                evidence.kind == EvidenceKind::NormativeRule
                    && evidence.source_recension == Recension::SynodalRussian
                    && !evidence.citation.is_empty()
            })
            && !variant.rule_trace.steps().is_empty()
    }));
}

pub(crate) fn regular_verb() -> VerbLexeme {
    VerbLexeme {
        lemma: word("нести"),
        aspect: Aspect::Imperfective,
        conjugation: VerbConjugation::FirstUnpalatalized,
        present_stem: Some(word("нес")),
        present_first_singular: Some(word("несꙋ")),
        present_third_plural: Some(word("несꙋтъ")),
        future_stem: None,
        future_first_singular: None,
        future_third_plural: None,
        imperfect_stem: Some(word("нес")),
        imperfect_formation: Some(ImperfectFormation::Yah),
        aorist_stem: Some(word("нес")),
        aorist_formation: Some(AoristFormation::ConsonantStem),
        imperative_stem: Some(word("нес")),
        imperative_formation: Some(ImperativeFormation::FirstUnpalatalized),
        l_participle_stem: Some(word("нес")),
        l_participle_masculine_singular_stem: None,
        present_active_participle: Some(ParticiplePrincipalPart {
            short_stem: Some(word("несꙋщ")),
            short_formation: Some(ActiveParticipleShortFormation::PresentFirstUnpalatalized),
            long_stem: Some(word("несꙋщ")),
            class: AdjectiveClass::Hard,
        }),
        past_active_participle: Some(ParticiplePrincipalPart {
            short_stem: Some(word("несш")),
            short_formation: Some(ActiveParticipleShortFormation::PastConsonant),
            long_stem: Some(word("несш")),
            class: AdjectiveClass::Hard,
        }),
        present_passive_participle: Some(ParticiplePrincipalPart {
            short_stem: Some(word("несом")),
            short_formation: None,
            long_stem: Some(word("несом")),
            class: AdjectiveClass::Hard,
        }),
        past_passive_participle: Some(ParticiplePrincipalPart {
            short_stem: Some(word("несен")),
            short_formation: None,
            long_stem: Some(word("несенн")),
            class: AdjectiveClass::Hard,
        }),
        verbal_noun: None,
    }
}
