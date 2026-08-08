use old_church_slavonic_extractor::extract::{load_registry, refresh, registry_with_overrides};
use std::fs;
use std::path::PathBuf;

#[test]
fn fixture_refresh_fails_closed_and_is_deterministic() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schema.jsonl");
    let root = std::env::temp_dir().join(format!(
        "old-church-slavonic-extractor-test-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove stale fixture output");
    }
    fs::create_dir_all(root.join("data")).expect("fixture data directory");
    fs::write(
        root.join("data/overrides.tsv"),
        "lemma\tpos\tfeature\tvariants\treason\tsource\treview_status\n",
    )
    .expect("empty override registry");
    fs::write(
        root.join("data/citation-exemptions.tsv"),
        "lexeme_id\treason\tsource\n",
    )
    .expect("empty citation exemption registry");
    refresh(&fixture, &root).expect("fixture refresh");
    let first = fs::read(root.join("crates/old-church-slavonic/generated/registry.rs"))
        .expect("generated fixture");
    refresh(&fixture, &root).expect("second fixture refresh");
    let second = fs::read(root.join("crates/old-church-slavonic/generated/registry.rs"))
        .expect("regenerated fixture");
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
        root.join("data/overrides.tsv"),
        concat!(
            "lemma\tpos\tfeature\tvariants\treason\tsource\treview_status\n",
            "тестъ\tnoun\tnoun:gen:sg\tтестоу :: testou || теста :: testa\t",
            "fixture correction\tfixture citation\tapproved\n"
        ),
    )
    .expect("write approved override");
    let overridden = registry_with_overrides(registry.clone(), &root.join("data/overrides.tsv"))
        .expect("apply approved override");
    let override_forms = overridden
        .forms
        .iter()
        .filter(|row| row.feature == "noun:gen:sg")
        .collect::<Vec<_>>();
    assert_eq!(override_forms.len(), 2);
    assert_eq!(override_forms[0].form, "тестоу");
    assert_eq!(override_forms[0].rank, 0);
    assert_eq!(override_forms[1].form, "теста");
    assert_eq!(override_forms[1].rank, 1);
    assert!(
        override_forms
            .iter()
            .all(|row| row.source_tags == "manual-override")
    );

    fs::write(
        root.join("data/overrides.tsv"),
        concat!(
            "lemma\tpos\tfeature\tvariants\treason\tsource\treview_status\n",
            "тестъ\tnoun\tnoun:gen:sg\tдва слова\tbad fixture\tfixture citation\tapproved\n"
        ),
    )
    .expect("write invalid override");
    assert!(
        registry_with_overrides(registry, &root.join("data/overrides.tsv"))
            .expect_err("word-level override must fail")
            .to_string()
            .contains("whitespace")
    );
    fs::remove_dir_all(&root).expect("clean fixture output");
}
