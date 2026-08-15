//! Structured Old Church Slavonic superlative and verbal periphrases.

use old_church_slavonic_core::adjective::{AdjectiveLexeme, ComparativeLexeme};
use old_church_slavonic_core::{
    AdjectiveCell, AnalyticConstruction, FormSet, InflectionError, Number, Person, PhraseOrder,
    PhraseRole, PhraseToken, RealizedPhrase, RuleId,
};

use crate::{Verb, resolver};

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
    use old_church_slavonic_core::{AdjectiveClass, AdjectiveForm, Animacy, Case, Gender, Number};

    fn nominative_masculine_short() -> AdjectiveCell {
        AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
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
}
