use std::collections::BTreeSet;

use synodal_church_slavonic::{FormSource, GrammarCell, Inflector, LexemeId};

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
    assert_eq!(checked, 11);
}

fn parse_cell(value: &str) -> GrammarCell {
    value.parse().expect("valid typed evaluation cell")
}
