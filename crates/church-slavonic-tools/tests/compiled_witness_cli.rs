#![allow(clippy::unwrap_used)]
//! Actual executable controls for source validation and compiled witness analysis.
//! Constructed source/annotations, not historical gold.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    path::PathBuf,
    process::{Command, Output},
};
struct Fixture {
    _dir: tempfile::TempDir,
    model: PathBuf,
    source: PathBuf,
    index: PathBuf,
    raw: String,
}
fn fixture(upper: bool) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("model with spaces.json");
    let source = dir.path().join("original source.html");
    let index = dir.path().join("compiled index.idx");
    let word = if upper { "ГРАДА" } else { "града" };
    let witness = format!("  {word}\t");
    let raw = format!("<h>title</h><p>{witness}</p>");
    let start = raw.find(&witness).unwrap();
    let mut data: Value = serde_json::from_slice(include_bytes!(
        "../../../data/rewrite/ocs-hard-noun-model.json"
    ))
    .unwrap();
    let evidence = data["input"]["evidence"][0]["id"].clone();
    data["input"]["observations"] = json!({
        "sources":[{"id":"s","uri":"test:constructed","sha256":format!("{:x}",Sha256::digest(raw.as_bytes()))}],
        "witnesses":[{"id":"w","source":"s","start":start,"end":start+witness.len(),"location":"constructed text node"}],
        "observations":[{"id":"o","witness":"w","start":2,"end":2+word.len(),"surface":word,"kind":"PedagogicalExample"}],
        "annotations":[
            {"id":"a-gen","observation":"o","original":{"case":"genitive","method":"constructed control"},"target":{"lexeme":"l-000001","cell":["n","gen.sg"],"orthography":"o-lrc-cyrillic"},"evidence":[evidence]},
            {"id":"a-unmapped","observation":"o","original":{"source_tag":"unsupported-tag"},"target":null,"evidence":[evidence]}
        ]
    });
    std::fs::write(&model, serde_json::to_vec(&data).unwrap()).unwrap();
    std::fs::write(&source, &raw).unwrap();
    Fixture {
        _dir: dir,
        model,
        source,
        index,
        raw,
    }
}
fn run(args: Vec<OsString>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .output()
        .unwrap()
}
fn compile(f: &Fixture, policy: &str) -> String {
    let out = run(vec![
        "compile-index".into(),
        f.model.clone().into(),
        "o-lrc-cyrillic".into(),
        policy.into(),
        f.index.clone().into(),
        f.source.clone().into(),
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice::<Value>(&out.stdout).unwrap()["sha256"]
        .as_str()
        .unwrap()
        .into()
}
fn witness_args(f: &Fixture) -> Vec<OsString> {
    vec![
        "analyze-witness".into(),
        f.model.clone().into(),
        "o-lrc-cyrillic".into(),
        "w".into(),
        f.source.clone().into(),
    ]
}
fn indexed_args(f: &Fixture, digest: &str) -> Vec<OsString> {
    let mut args = witness_args(f);
    args.extend([
        "--index".into(),
        f.index.clone().into(),
        "--index-sha256".into(),
        digest.into(),
    ]);
    args
}
fn good(output: Output) -> Vec<u8> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
fn restored_witness_keeps_source_addresses_annotations_and_actual_ambiguity() {
    let f = fixture(false);
    let cold = good(run(witness_args(&f)));
    let digest = compile(&f, "exact");
    let restored = good(run(indexed_args(&f, &digest)));
    assert_eq!(restored, cold);
    let doc: Value = serde_json::from_slice(&restored).unwrap();
    assert_eq!(doc["registered_witness"], "w");
    assert_eq!(doc["source"], "  града\t");
    assert_eq!(
        doc["witness_spec"]["start"],
        f.raw.find("  града\t").unwrap()
    );
    assert_eq!(doc["witness_annotations"].as_array().unwrap().len(), 2);
    let candidates: Vec<_> = doc["segments"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| s["candidates"].as_array().unwrap())
        .collect();
    assert_eq!(candidates.len(), 4);
    for c in candidates {
        let links = c["derivation"]["source_annotations"].as_array().unwrap();
        if c["cell"] == "gen.sg" {
            assert_eq!(links, &vec![json!("a-gen")]);
        } else {
            assert!(links.is_empty());
        }
    }
}

#[test]
fn compiled_witness_never_bypasses_original_source_validation() {
    let f = fixture(false);
    let digest = compile(&f, "exact");
    // Change bytes outside the witness: its visible text and offsets are still
    // identical, but it is no longer the registered source snapshot.
    let altered = f.raw.clone() + " ";
    std::fs::write(&f.source, &altered).unwrap();
    let out = run(indexed_args(&f, &digest));
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("MissingSource"));
    // Updating the source declaration does not make an old compiled index valid.
    let mut data: Value = serde_json::from_slice(&std::fs::read(&f.model).unwrap()).unwrap();
    data["input"]["observations"]["sources"][0]["sha256"] =
        format!("{:x}", Sha256::digest(altered.as_bytes())).into();
    std::fs::write(&f.model, serde_json::to_vec(&data).unwrap()).unwrap();
    let out = run(indexed_args(&f, &digest));
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("compiled index model"));
}

#[test]
fn explicit_matching_policy_and_paired_unique_index_options_are_enforced() {
    let f = fixture(true);
    let digest = compile(&f, "case-insensitive");
    let wrong = run(indexed_args(&f, &digest));
    assert!(!wrong.status.success());
    assert!(wrong.stdout.is_empty());
    assert!(String::from_utf8_lossy(&wrong.stderr).contains("profile/policy"));
    let mut cold = witness_args(&f);
    cold.extend(["--match".into(), "case-insensitive".into()]);
    let mut restored = indexed_args(&f, &digest);
    restored.extend(["--match".into(), "case-insensitive".into()]);
    let expected = good(run(cold));
    let actual = good(run(restored));
    assert_eq!(actual, expected);
    let doc: Value = serde_json::from_slice(&actual).unwrap();
    assert_eq!(doc["source"], "  ГРАДА\t");
    assert_eq!(
        doc["segments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["candidates"].as_array().unwrap().len())
            .sum::<usize>(),
        4
    );
    let mut missing = witness_args(&f);
    missing.extend(["--index".into(), f.index.clone().into()]);
    let out = run(missing);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("supplied together"));
    let mut duplicate = indexed_args(&f, &digest);
    duplicate.extend(["--index".into(), f.index.clone().into()]);
    let out = run(duplicate);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("duplicate option --index"));
    let ordinary = run(vec![
        "analyze".into(),
        "--model".into(),
        f.model.clone().into(),
        "--index".into(),
        f.index.clone().into(),
        "--index".into(),
        f.index.clone().into(),
        "--index-sha256".into(),
        digest.into(),
        "text".into(),
    ]);
    assert!(!ordinary.status.success());
    assert!(String::from_utf8_lossy(&ordinary.stderr).contains("duplicate option --index"));
}
