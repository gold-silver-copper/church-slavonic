use std::collections::BTreeSet;

use synodal_church_slavonic::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, FiniteTense, FiniteVerbCell,
    FormSource, Gender, GrammarCell, Inflector, LexemeId, NounCell, Number, ParticipleCell,
    ParticipleTense, ParticipleVoice, Person,
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
    assert_eq!(checked, 11);
}

fn parse_cell(value: &str) -> GrammarCell {
    let fields = value.split(':').collect::<Vec<_>>();
    match fields.as_slice() {
        ["noun", case, number, animacy] => GrammarCell::Noun(NounCell {
            case: parse_case(case),
            number: parse_number(number),
            animacy: parse_animacy(animacy),
        }),
        [
            tense @ ("present" | "future" | "past" | "imperfect" | "aorist"),
            person,
            number,
        ] => GrammarCell::FiniteVerb(FiniteVerbCell {
            tense: parse_tense(tense),
            person: parse_person(person),
            number: parse_number(number),
        }),
        [
            "participle",
            tense,
            voice,
            case,
            number,
            gender,
            animacy,
            form,
            "positive",
        ] => GrammarCell::Participle(ParticipleCell {
            tense: match *tense {
                "present" => ParticipleTense::Present,
                "past" => ParticipleTense::Past,
                _ => panic!("unknown participle tense {tense}"),
            },
            voice: match *voice {
                "active" => ParticipleVoice::Active,
                "passive" => ParticipleVoice::Passive,
                _ => panic!("unknown participle voice {voice}"),
            },
            agreement: AdjectiveCell {
                case: parse_case(case),
                number: parse_number(number),
                gender: parse_gender(gender),
                animacy: parse_animacy(animacy),
                form: match *form {
                    "short" => AdjectiveForm::Short,
                    "long" => AdjectiveForm::Long,
                    _ => panic!("unknown adjective form {form}"),
                },
                comparison: Comparison::Positive,
            },
        }),
        _ => panic!("unknown fixture cell {value}"),
    }
}

fn parse_case(value: &str) -> Case {
    match value {
        "nominative" => Case::Nominative,
        "genitive" => Case::Genitive,
        "dative" => Case::Dative,
        "accusative" => Case::Accusative,
        "instrumental" => Case::Instrumental,
        "locative" => Case::Locative,
        "vocative" => Case::Vocative,
        _ => panic!("unknown case {value}"),
    }
}

fn parse_number(value: &str) -> Number {
    match value {
        "singular" => Number::Singular,
        "dual" => Number::Dual,
        "plural" => Number::Plural,
        _ => panic!("unknown number {value}"),
    }
}

fn parse_animacy(value: &str) -> Animacy {
    match value {
        "animate" => Animacy::Animate,
        "inanimate" => Animacy::Inanimate,
        _ => panic!("unknown animacy {value}"),
    }
}

fn parse_person(value: &str) -> Person {
    match value {
        "first" => Person::First,
        "second" => Person::Second,
        "third" => Person::Third,
        _ => panic!("unknown person {value}"),
    }
}

fn parse_tense(value: &str) -> FiniteTense {
    match value {
        "present" => FiniteTense::Present,
        "future" => FiniteTense::Future,
        "past" => FiniteTense::Past,
        "imperfect" => FiniteTense::Imperfect,
        "aorist" => FiniteTense::Aorist,
        _ => panic!("unknown finite tense {value}"),
    }
}

fn parse_gender(value: &str) -> Gender {
    match value {
        "masculine" => Gender::Masculine,
        "feminine" => Gender::Feminine,
        "neuter" => Gender::Neuter,
        _ => panic!("unknown gender {value}"),
    }
}
