use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_synodal-dict"))
        .args(arguments)
        .output()
        .expect("run synodal-dict")
}

fn run_stdin(arguments: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_synodal-dict"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run synodal-dict");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write stdin");
    child.wait_with_output().expect("read synodal-dict")
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn golden(name: &str) -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden")
            .join(name),
    )
    .expect("read golden output")
}

#[test]
fn search_show_and_analyze_are_deterministic() {
    let search_human = run(&["search", "city", "--limit", "1"]);
    assert!(search_human.status.success());
    assert_eq!(search_human.stdout, golden("search-city.txt"));

    let search = run(&["search", "city", "--limit", "1", "--json"]);
    assert!(search.status.success());
    let search_json: serde_json::Value =
        serde_json::from_slice(&search.stdout).expect("search JSON");
    assert_eq!(search_json[0]["entry"]["lexeme"]["lemma"], "градъ");

    let show = run(&["show", "synodal:noun:grad", "--json"]);
    assert!(show.status.success());
    let show_json: serde_json::Value = serde_json::from_slice(&show.stdout).expect("show JSON");
    assert_eq!(show_json[0]["metadata"]["class"], "inherited-first-hard-m");

    let analyze = run(&["analyze", "є҆́смь", "--profile", "printed", "--json"]);
    assert!(analyze.status.success());
    let analyze_json: serde_json::Value =
        serde_json::from_slice(&analyze.stdout).expect("analysis JSON");
    assert_eq!(analyze_json[0]["lexeme"]["lemma"], "быти");
}

#[test]
fn show_rejects_unknown_options_and_extra_queries() {
    let unknown = run(&["show", "не", "--bogus"]);
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown show option"));

    let extra = run(&["show", "не", "ли"]);
    assert!(!extra.status.success());
    assert!(String::from_utf8_lossy(&extra.stderr).contains("show accepts exactly one"));
}

#[test]
fn lint_and_check_text_enforce_exit_thresholds() {
    let vocabulary = fixture("vocabulary.json");
    let lint = run(&["lint", vocabulary.to_str().expect("UTF-8 path"), "--json"]);
    assert!(
        lint.status.success(),
        "{}",
        String::from_utf8_lossy(&lint.stderr)
    );
    let issues: serde_json::Value = serde_json::from_slice(&lint.stdout).expect("lint JSON");
    assert_eq!(issues, serde_json::json!([]));
    assert_eq!(lint.stdout, golden("lint-empty.json"));

    let rendered = fixture("rendered.txt");
    let check = run(&[
        "check-text",
        rendered.to_str().expect("UTF-8 path"),
        "--profile",
        "printed",
        "--strict",
        "--json",
    ]);
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("check JSON");
    assert_eq!(report["summary"]["unresolved_tokens"], 0);

    let orthographic_gap = run_stdin(
        &[
            "check-text",
            "-",
            "--profile",
            "printed",
            "--max-unknown",
            "0",
            "--json",
        ],
        "и\u{301}",
    );
    assert!(
        orthographic_gap.status.success(),
        "{}",
        String::from_utf8_lossy(&orthographic_gap.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&orthographic_gap.stdout).expect("orthographic gap JSON");
    assert_eq!(report["summary"]["unresolved_tokens"], 1);
    assert_eq!(
        report["summary"]["by_gap"]["missing-accent-or-orthographic-metadata"],
        1
    );

    let strict_gap = run_stdin(
        &["check-text", "-", "--profile", "printed", "--strict"],
        "и\u{301}",
    );
    assert!(!strict_gap.status.success());
    assert!(String::from_utf8_lossy(&strict_gap.stderr).contains("under --strict"));

    let strict_spelling_variant = run_stdin(
        &["check-text", "-", "--profile", "printed", "--strict"],
        "ѽ",
    );
    assert!(!strict_spelling_variant.status.success());
    assert!(
        String::from_utf8_lossy(&strict_spelling_variant.stderr)
            .contains("top-k-uncovered token(s) under --strict")
    );
}

#[test]
fn coverage_accepts_passage_tsv_and_emits_ranked_json() {
    let corpus =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/synodal/coverage_passages.tsv");
    let output = run(&[
        "coverage",
        corpus.to_str().expect("UTF-8 path"),
        "--profile",
        "printed",
        "--by-family",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("coverage JSON");
    assert_eq!(report["passages"], 10);
    assert!(report["by_family"].is_object());
    assert!(report["unresolved_by_probable_family"].is_object());
    assert!(report["estimated_recovery_by_route"].is_object());
    assert!(
        report["review_queue"]
            .as_array()
            .is_some_and(|rows| !rows.is_empty())
    );
}

#[test]
fn family_commands_separate_reviewed_and_proposed_data() {
    let proposals = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../reports/synodal-family-review-queue.json");
    let reviewed = run(&["families", "ꙗкѡ", "--reviewed-only", "--json"]);
    assert!(
        reviewed.status.success(),
        "{}",
        String::from_utf8_lossy(&reviewed.stderr)
    );
    let reviewed_json: serde_json::Value =
        serde_json::from_slice(&reviewed.stdout).expect("reviewed family JSON");
    assert_eq!(reviewed_json["reviewed"].as_array().map(Vec::len), Some(1));
    assert_eq!(reviewed_json["proposed"], serde_json::json!([]));

    let proposal_rows: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&proposals).expect("family proposal report"))
            .expect("family proposal JSON");
    let expected_candidate_id = proposal_rows[0]["candidate_id"]
        .as_str()
        .expect("first candidate ID");
    let proposed = run(&[
        "families",
        expected_candidate_id,
        "--proposals",
        proposals.to_str().expect("UTF-8 path"),
        "--json",
    ]);
    assert!(
        proposed.status.success(),
        "{}",
        String::from_utf8_lossy(&proposed.stderr)
    );
    let proposed_json: serde_json::Value =
        serde_json::from_slice(&proposed.stdout).expect("proposed family JSON");
    let candidate_id = proposed_json["proposed"]
        .as_array()
        .and_then(|rows| {
            rows.iter().find_map(|row| {
                (row["candidate_id"].as_str() == Some(expected_candidate_id))
                    .then(|| row["candidate_id"].as_str())
                    .flatten()
            })
        })
        .expect("candidate ID");

    let shown = run(&[
        "show-family",
        candidate_id,
        "--proposals",
        proposals.to_str().expect("UTF-8 path"),
        "--json",
    ]);
    assert!(shown.status.success());
    let shown_json: serde_json::Value =
        serde_json::from_slice(&shown.stdout).expect("shown proposal JSON");
    assert_eq!(shown_json["candidate_id"], candidate_id);

    let bad_option = run(&["show-family", "family:synodal:determiner:ves", "--bogus"]);
    assert!(!bad_option.status.success());
}

#[test]
fn marginal_recovery_exposes_ranked_review_readiness() {
    let report = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../reports/synodal-marginal-recovery.json");
    let output = run(&[
        "marginal-recovery",
        report.to_str().expect("UTF-8 path"),
        "--readiness",
        "ready",
        "--limit",
        "2",
        "--json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("marginal recovery JSON");
    assert_eq!(value["current_top_k"], 919_436);
    assert_eq!(value["target_top_k"], 919_341);
    assert_eq!(value["tokens_needed_for_target"], 0);
    assert_eq!(value["milestones"].as_array().map(Vec::len), Some(5));
    assert_eq!(value["milestones"][4]["percent"], 70);
    assert_eq!(value["milestones"][4]["margin"], 95);
    let batches = value["batches"].as_array().expect("batches");
    assert_eq!(batches.len(), 2);
    assert!(
        batches
            .iter()
            .all(|batch| batch["evidence_readiness"] == "ready")
    );
    assert!(batches[0]["overlap_adjusted_tokens"].as_u64().is_some());

    let bad = run(&[
        "marginal-recovery",
        report.to_str().expect("UTF-8 path"),
        "--readiness",
        "invented",
    ]);
    assert!(!bad.status.success());
}
