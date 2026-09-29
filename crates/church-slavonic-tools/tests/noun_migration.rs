#![allow(clippy::unwrap_used)]
use church_slavonic::lexicon::COLUMNS;
use church_slavonic_tools::{migration, model_artifact};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

struct Fixture {
    _dir: tempfile::TempDir,
    source: PathBuf,
    seed: PathBuf,
    mapping: PathBuf,
    output: PathBuf,
}
impl Fixture {
    fn new(variants: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let f = Self {
            source: dir.path().join("source.tsv"),
            seed: dir.path().join("seed.json"),
            mapping: dir.path().join("mapping.json"),
            output: dir.path().join("output.json"),
            _dir: dir,
        };
        let raw = format!(
            "{}\r\nold.n\tградъ\tn\tm\t-\tuntrusted-class\t-\t-\t-\t{variants}\tK:claim\tuncertain identity\r\nother.n\tградъ\tn\t-\t-\t-\t-\t-\t-\t-\t-\t-\r\n",
            COLUMNS.join("\t")
        );
        fs::write(&f.source, &raw).unwrap();
        fs::write(
            &f.seed,
            include_bytes!("../../../data/rewrite/ocs-hard-noun-model.json"),
        )
        .unwrap();
        let mapping = json!({
            "source_sha256": format!("{:x}", Sha256::digest(raw.as_bytes())),
            "orthography": "o-lrc-cyrillic",
            "entries": [{"legacy_id":"old.n", "targets":["l-000001"],
                "rationale":"Constructed proposed correspondence; not an identity decision"}]
        });
        fs::write(&f.mapping, serde_json::to_vec(&mapping).unwrap()).unwrap();
        fs::write(&f.output, b"previous output").unwrap();
        f
    }
    fn run(&self) -> Result<migration::Report, Box<dyn std::error::Error>> {
        migration::migrate(&self.source, &self.seed, &self.mapping, &self.output)
    }
    fn change_mapping(&self, change: impl FnOnce(&mut Value)) {
        let mut v: Value = serde_json::from_slice(&fs::read(&self.mapping).unwrap()).unwrap();
        change(&mut v);
        fs::write(&self.mapping, serde_json::to_vec(&v).unwrap()).unwrap();
    }
    fn rejects(&self) {
        assert!(self.run().is_err());
        assert_eq!(fs::read(&self.output).unwrap(), b"previous output");
    }
}

#[test]
fn preserves_claims_spans_and_disagreements_without_modifying_grammar() {
    let f = Fixture::new("gen.sg=града×2|несовпадение×0;nom.sg=ГРАДЪ");
    let report = f.run().unwrap();
    assert_eq!(
        (
            report.source_records,
            report.selected_records,
            report.unselected_records
        ),
        (2, 1, 1)
    );
    assert_eq!(
        (
            report.imported_forms,
            report.cell_claims,
            report.exact_compatible,
            report.tolerant_compatible
        ),
        (4, 3, 1, 2)
    );
    assert!(report.claims[0].cell.is_none());
    assert!(report.claims[2].exact.is_empty());
    let raw = fs::read(&f.source).unwrap();
    let output = fs::read(&f.output).unwrap();
    model_artifact::from_bytes_with_sources(&output, vec![raw.clone()]).unwrap();
    let mut migrated: Value = serde_json::from_slice(&output).unwrap();
    let seed_input: model_artifact::Artifact =
        serde_json::from_slice(&fs::read(&f.seed).unwrap()).unwrap();
    let mut seed = serde_json::to_value(seed_input).unwrap();
    let archive = &migrated["input"]["observations"];
    for (w, o) in archive["witnesses"]
        .as_array()
        .unwrap()
        .iter()
        .zip(archive["observations"].as_array().unwrap())
    {
        let start = w["start"].as_u64().unwrap() as usize;
        let end = w["end"].as_u64().unwrap() as usize;
        assert_eq!(&raw[start..end], o["surface"].as_str().unwrap().as_bytes());
        assert_eq!(o["kind"], "Unclassified");
    }
    let annotations = archive["annotations"].as_array().unwrap();
    assert!(annotations[0]["target"].is_null());
    assert_eq!(annotations[1]["original"]["raw_variant_weight"], "2");
    assert_eq!(annotations[2]["original"]["raw_variant_weight"], "0");
    assert_eq!(annotations[1]["original"]["class"], "untrusted-class");
    assert_eq!(annotations[1]["original"]["note"], "uncertain identity");
    assert_eq!(
        annotations[1]["original"]["correspondence_status"],
        "proposed; not adjudicated"
    );
    // Compare the entire grammatical input, not just one generated surface.
    migrated["input"]
        .as_object_mut()
        .unwrap()
        .remove("observations");
    seed["input"]
        .as_object_mut()
        .unwrap()
        .remove("observations");
    let added = migrated["input"]["evidence"]
        .as_array_mut()
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(added["review"], "Unverified");
    assert_eq!(migrated, seed);
}

#[test]
fn invalid_sources_and_mappings_preserve_previous_output() {
    for variant in [
        "unknown=града",
        "gen.sg=града×bad",
        "gen.sg=града×4294967296",
        "gen.sg=града×1×2",
        "gen.sg=",
        "gen.sg=града|",
    ] {
        Fixture::new(variant).rejects();
    }
    for change in 0..5 {
        let f = Fixture::new("gen.sg=града");
        f.change_mapping(|m| match change {
            0 => m["source_sha256"] = json!("0".repeat(64)),
            1 => m["entries"][0]["legacy_id"] = json!("missing"),
            2 => m["entries"][0]["targets"] = json!(["missing"]),
            3 => m["entries"][0]["targets"] = json!(["l-000001", "l-000001"]),
            _ => {
                let entry = m["entries"][0].clone();
                m["entries"].as_array_mut().unwrap().push(entry);
            }
        });
        f.rejects();
    }
    let f = Fixture::new("gen.sg=града");
    let before = fs::read(&f.source).unwrap();
    assert!(migration::migrate(&f.source, &f.seed, &f.mapping, &f.source).is_err());
    assert_eq!(fs::read(&f.source).unwrap(), before);
}

#[test]
fn repeated_metadata_budget_rejects_before_publication() {
    let variants = format!("gen.sg={}", vec!["града"; 1500].join("|"));
    let f = Fixture::new(&variants);
    let error = f.run().err().unwrap().to_string();
    assert!(error.contains("budget"), "{error}");
    assert_eq!(fs::read(&f.output).unwrap(), b"previous output");
}

#[test]
fn proposed_identity_alternatives_remain_separate() {
    let f = Fixture::new("gen.sg=града");
    let mut seed: Value = serde_json::from_slice(&fs::read(&f.seed).unwrap()).unwrap();
    let mut other = seed["input"]["lexemes"][0].clone();
    other["id"] = json!("constructed-second-identity");
    seed["input"]["lexemes"].as_array_mut().unwrap().push(other);
    fs::write(&f.seed, serde_json::to_vec(&seed).unwrap()).unwrap();
    f.change_mapping(|m| {
        m["entries"][0]["targets"] = json!(["l-000001", "constructed-second-identity"])
    });
    let report = f.run().unwrap();
    assert_eq!(report.cell_claims, 1);
    assert_eq!(report.exact_compatible, 1); // Claim denominator, not number of targets.
    assert_eq!(report.claims[1].exact.len(), 2);
    let output: Value = serde_json::from_slice(&fs::read(&f.output).unwrap()).unwrap();
    let annotations = output["input"]["observations"]["annotations"]
        .as_array()
        .unwrap();
    assert_eq!(annotations.len(), 3);
    assert!(annotations[0]["target"].is_null());
    assert_ne!(
        annotations[1]["target"]["lexeme"],
        annotations[2]["target"]["lexeme"]
    );
    assert_eq!(output["input"]["lexemes"].as_array().unwrap().len(), 2);
}

#[test]
fn malformed_source_records_and_utf8_are_rejected_with_matching_hash() {
    for case in 0..4 {
        let f = Fixture::new("gen.sg=града");
        let raw = fs::read(&f.source).unwrap();
        let mut text = String::from_utf8(raw).unwrap();
        match case {
            0 => text = text.replace("other.n", "old.n"),
            1 => text = text.replace("\tn\t", "\tv\t"),
            2 => text = text.replace("\tK:claim", "\textra\tK:claim"),
            _ => {}
        }
        let mut raw = text.into_bytes();
        if case == 3 {
            raw.push(0xff);
        }
        f.change_mapping(|m| m["source_sha256"] = json!(format!("{:x}", Sha256::digest(&raw))));
        fs::write(&f.source, raw).unwrap();
        f.rejects();
    }
}
