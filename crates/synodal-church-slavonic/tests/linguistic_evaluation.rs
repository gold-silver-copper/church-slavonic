use std::collections::BTreeSet;

use synodal_church_slavonic::{
    Animacy, Aspect, Case, FormSource, GrammarCell, Inflector, LexemeId, NounCell, Number,
    SpecificationSource, VerbConjugation, VerbSpec,
};

const FIXTURE: &str = include_str!("../../../data/synodal/linguistic_evaluation.tsv");

#[test]
fn curated_linguistic_contracts_pass_without_frequency_weighting() {
    let mut rows = FIXTURE.lines();
    assert_eq!(
        rows.next(),
        Some(
            "contract_id\tlexeme_id\tcell\texpected_variants\texpected_error\texpected_source\tnormative_source\tnormative_citation"
        )
    );
    let inflector = Inflector::default();
    let mut checked = 0;
    let mut contract_ids = BTreeSet::new();
    for line in rows {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 8, "malformed evaluation row {line:?}");
        let [
            contract_id,
            lexeme_id,
            cell,
            expected_variants,
            expected_error,
            expected_source,
            source,
            citation,
        ] = fields.as_slice()
        else {
            unreachable!("column count checked above")
        };
        assert!(!contract_id.is_empty());
        assert!(contract_ids.insert(*contract_id), "duplicate {contract_id}");
        assert!(!source.is_empty());
        assert!(!citation.is_empty());
        let outcome = inflector.form_by_id(&LexemeId::from(*lexeme_id), parse_cell(cell));
        if expected_error.is_empty() {
            let forms = outcome.unwrap_or_else(|error| panic!("{contract_id}: {error}"));
            assert_eq!(
                forms.texts().collect::<Vec<_>>(),
                expected_variants.split('|').collect::<Vec<_>>(),
                "{contract_id}"
            );
            let actual_source = match forms.primary().source {
                FormSource::SynodalAttestation { .. } => "attested",
                FormSource::SynodalIrregularOverride { .. } => "irregular",
                FormSource::SynodalNormativeGeneration { .. } => "productive",
                FormSource::CallerSpecifiedPrediction { .. } => "caller-specified",
                FormSource::InheritedPrediction { .. } => "inherited",
                FormSource::AnalogicalPrediction { .. } => "analogical",
            };
            assert_eq!(actual_source, *expected_source, "{contract_id}");
        } else {
            let error = outcome.expect_err(contract_id);
            assert_eq!(
                format!("{:?}", error.code()),
                *expected_error,
                "{contract_id}"
            );
            assert_eq!(*expected_source, "error", "{contract_id}");
        }
        checked += 1;
    }
    assert_eq!(checked, 27);
}

#[test]
fn alypy_27_molen_ie_matches_the_locked_target_attestation() {
    const EXACT_FORMS: &str = include_str!("../../../data/synodal/exact_forms.tsv");
    assert!(EXACT_FORMS.lines().any(|line| {
        line.starts_with(
            "synodal:noun:v06-4b5409194023ee78\tnoun:accusative:singular:inanimate\tмоленїе\t",
        )
    }));

    let source = SpecificationSource::new(
        "alypy-27-verbal-noun",
        "alypy-gamanovich-grammar-web-2023",
        "§27 молен-ї-е; independently locked target моленїе",
    )
    .expect("reviewed source");
    let verb = VerbSpec::builder(
        "молити",
        Aspect::Imperfective,
        VerbConjugation::Second,
        source,
    )
    .expect("typed verb")
    .verbal_noun_ie("молен")
    .expect("Alypy platform")
    .build()
    .expect("complete derived-noun metadata");
    assert_eq!(
        verb.form(GrammarCell::VerbalNoun(NounCell {
            case: Case::Accusative,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        }))
        .expect("productive verbal noun")
        .primary_text(),
        "моленїе"
    );
}

fn parse_cell(value: &str) -> GrammarCell {
    value.parse().expect("valid typed evaluation cell")
}
