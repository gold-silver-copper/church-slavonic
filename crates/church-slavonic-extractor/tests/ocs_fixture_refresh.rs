use church_slavonic_extractor::ocs::extract::{load_registry, refresh, registry_with_overrides};
use std::fs;
use std::path::PathBuf;

#[test]
fn fixture_refresh_fails_closed_and_is_deterministic() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schema.jsonl");
    let root = std::env::temp_dir().join(format!(
        "church-slavonic-extractor-ocs-test-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture output");
    }
    fs::create_dir_all(root.join("data/ocs")).expect("fixture data directory");
    fs::write(
        root.join("data/ocs/overrides.tsv"),
        "lemma\tpos\tfeature\tvariants\treason\tsource\treview_status\n",
    )
    .expect("empty override registry");
    fs::write(
        root.join("data/ocs/citation-exemptions.tsv"),
        "lexeme_id\treason\tsource\n",
    )
    .expect("empty citation exemption registry");
    refresh(&fixture, &root).expect("fixture refresh");
    let first = fs::read(root.join("data/extracted/forms.tsv")).expect("generated fixture");
    refresh(&fixture, &root).expect("second fixture refresh");
    let second = fs::read(root.join("data/extracted/forms.tsv")).expect("regenerated fixture");
    assert_eq!(first, second);

    let registry = load_registry(&root.join("data/extracted")).expect("fixture registry");
    assert!(
        registry
            .forms
            .iter()
            .any(|row| { row.feature == "verb:finite:present:1:sg" && row.form == "тестѭ" })
    );
    assert!(!registry.forms.iter().any(|row| row.form == "guess"));

    fs::write(
        root.join("data/ocs/overrides.tsv"),
        concat!(
            "lemma\tpos\tfeature\tvariants\treason\tsource\treview_status\n",
            "тестъ\tnoun\tnoun:dat:sg\tтестоу :: testou || тестови :: testovi\t",
            "fixture correction\tfixture citation\tapproved\n"
        ),
    )
    .expect("write approved override");
    let overridden =
        registry_with_overrides(registry.clone(), &root.join("data/ocs/overrides.tsv"))
            .expect("apply approved override");
    let override_forms = overridden
        .overrides
        .iter()
        .filter(|row| row.feature == "noun:dat:sg")
        .collect::<Vec<_>>();
    assert_eq!(override_forms.len(), 2);
    assert_eq!(override_forms[0].form, "тестоу");
    assert_eq!(override_forms[0].rank, 0);
    assert_eq!(override_forms[1].form, "тестови");
    assert_eq!(override_forms[1].rank, 1);
    assert!(
        override_forms
            .iter()
            .all(|row| row.reason == "fixture correction" && row.authority == "fixture citation")
    );

    fs::write(
        root.join("data/ocs/overrides.tsv"),
        concat!(
            "lemma\tpos\tfeature\tvariants\treason\tsource\treview_status\n",
            "тестъ\tnoun\tnoun:dat:sg\tдва слова\tbad fixture\tfixture citation\tapproved\n"
        ),
    )
    .expect("write invalid override");
    assert!(
        registry_with_overrides(registry.clone(), &root.join("data/ocs/overrides.tsv"))
            .expect_err("word-level override must fail")
            .to_string()
            .contains("whitespace")
    );

    fs::write(
        root.join("data/ocs/overrides.tsv"),
        concat!(
            "lemma\tpos\tfeature\tvariants\treason\tsource\treview_status\n",
            "тестъ\tnoun\tnoun:unknown:sg\tтестоу\tbad feature\tfixture citation\tapproved\n"
        ),
    )
    .expect("write invalid feature override");
    assert!(
        registry_with_overrides(registry.clone(), &root.join("data/ocs/overrides.tsv"))
            .expect_err("unknown override feature must fail")
            .to_string()
            .contains("invalid feature")
    );

    fs::write(
        root.join("data/ocs/overrides.tsv"),
        concat!(
            "lemma\tpos\tfeature\tvariants\treason\tsource\treview_status\n",
            "тестъ\tnoun\tnoun:gen:sg\tтестоу\tshadow\tfixture citation\tapproved\n"
        ),
    )
    .expect("write shadowing override");
    assert!(
        registry_with_overrides(registry, &root.join("data/ocs/overrides.tsv"))
            .expect_err("source-cell shadowing must fail")
            .to_string()
            .contains("shadow an exact source cell")
    );
    fs::remove_dir_all(&root).expect("clean fixture output");
}
