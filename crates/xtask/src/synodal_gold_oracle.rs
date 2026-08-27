//! Derives the committed type-level token oracle for the Synodal gold gate
//! (`data/synodal/gold_token_oracle.tsv`) from the pinned Ponomar Elizabeth
//! Bible intermediate, and hosts the witness loaders used for adjudication.
//! The comparison contract is normative in `docs/SYNODAL_GOLD_ORACLE.md`.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use synodal_church_slavonic::Inflector;
use synodal_church_slavonic_dictionary::coverage::{Analyzer, classify_non_lexical, tokenize};
use unicode_normalization::UnicodeNormalization;

const SOURCE_RELATIVE: &str = "data/intermediate/synodal/ponomar-elizabeth-bible-2026-08-09.jsonl";
const ORACLE_RELATIVE: &str = "data/synodal/gold_token_oracle.tsv";
const LEDGER_RELATIVE: &str = "data/synodal/gold_source_defects.tsv";
const EVALUATION_RELATIVE: &str = "data/synodal/evaluation.tsv";
const CROSSWIRE_RELATIVE: &str = "data/intermediate/synodal/crosswire-csl-elizabeth-1.5.2.jsonl";
const WIKISOURCE_RELATIVE: &str =
    "data/intermediate/synodal/wikisource-church-slavonic-bible-2026-08-09.jsonl";

/// Bounded evidence-pointer list per oracle row; the cap keeps the committed
/// artifact small (types are never trimmed, only references).
const REFERENCE_CAP: usize = 8;

const COLUMN_HEADER: &str = "surface\tcount\tnon_lexical\treferences\tconfirmed_readings";

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args.by_ref() {
        match argument.as_str() {
            "--check" => check = true,
            other => return Err(format!("unknown synodal-gold-oracle option: {other}").into()),
        }
    }
    if check {
        check_oracle(root)
    } else {
        generate(root)
    }
}

#[derive(serde::Deserialize)]
struct VerseRecord {
    passage: String,
    normalized_spelling: String,
}

/// The canonical comparison key for a printed token surface: NFC over the
/// tokenizer's surface slice. Anything beyond NFC (titlo expansion,
/// initial-uk presentation, verse-initial case) is an enumerated equivalence
/// class applied at comparison time by the gate, never baked into the oracle
/// key — the oracle records the printed form.
pub(crate) fn canonical_surface(token: &str) -> String {
    token.nfc().collect()
}

/// Strips non-scriptural apparatus from a verse before tokenization, per the
/// normative contract:
/// - parenthesized zachalo markers — `(Зача́ло N)` and the abbreviated
///   `(Заⷱ҇ N)`/`(Заⷱ҇)` — are lectionary navigation, not verse text;
/// - square-bracketed segments are versification cross-references
///   (e.g. `[Быт 17:15]`).
///
/// All other parenthesized material (supplied readings such as `(и҆)`) is
/// verse text and is kept; the parentheses themselves are ordinary
/// punctuation and never enter any token.
pub(crate) fn strip_apparatus(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find(['(', '[']) {
        let (head, tail) = rest.split_at(open);
        output.push_str(head);
        let opener = tail.chars().next().expect("split at an opener");
        let closer = if opener == '(' { ')' } else { ']' };
        let Some(close) = tail.find(closer) else {
            output.push_str(tail);
            return output;
        };
        let inner = &tail[opener.len_utf8()..close];
        let apparatus =
            opener == '[' || inner.starts_with("Зач") || inner.starts_with("За\u{2df1}");
        if apparatus {
            output.push(' ');
        } else {
            output.push_str(&tail[..=close]);
        }
        rest = &tail[close + closer.len_utf8()..];
    }
    output.push_str(rest);
    output
}

/// A record is scriptural when its passage is a Bible verse. The eight
/// `Composite.*` records are Ponomar lectionary composites (rubric headings
/// plus verses duplicated from their home books) and are excluded from the
/// token oracle per the normative contract.
fn is_scriptural(passage: &str) -> bool {
    !passage.starts_with("Composite.")
}

#[derive(Default)]
struct TypeEntry {
    count: u64,
    references: Vec<String>,
}

fn confirmed_readings(root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>, Box<dyn Error>> {
    let path = root.join(EVALUATION_RELATIVE);
    let content =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut lines = content.lines();
    let header = lines.next().ok_or("empty evaluation.tsv")?;
    let columns: Vec<&str> = header.split('\t').collect();
    let index_of = |name: &str| {
        columns
            .iter()
            .position(|column| *column == name)
            .ok_or_else(|| format!("evaluation.tsv lacks column {name}"))
    };
    let id_index = index_of("id")?;
    let lexeme_index = index_of("lexeme_id")?;
    let cell_index = index_of("cell")?;
    let printed_index = index_of("expected_printed")?;
    // A reviewed evaluation row retracted through the exact-cell correction
    // ledger is no longer a confirmed reading (§5): the gate must not be held
    // to a cell the ledger has withdrawn.
    let retracted = crate::synodal::load_retracted_evaluation_ids(
        &root.join("data/synodal/v10_exact_cell_corrections.tsv"),
    )?;
    let mut map: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        let (Some(id), Some(lexeme), Some(cell), Some(printed)) = (
            fields.get(id_index),
            fields.get(lexeme_index),
            fields.get(cell_index),
            fields.get(printed_index),
        ) else {
            return Err(format!("short evaluation.tsv row: {line}").into());
        };
        if retracted.contains(*id) {
            continue;
        }
        map.entry(canonical_surface(printed))
            .or_default()
            .insert(format!("{lexeme}|{cell}"));
    }
    Ok(map)
}

/// Builds the full oracle file content (header block + column header + rows)
/// from the pinned source. Deterministic: rows sort by surface byte order.
fn render_oracle(root: &Path) -> Result<String, Box<dyn Error>> {
    let source = root.join(SOURCE_RELATIVE);
    let file =
        fs::File::open(&source).map_err(|error| format!("open {}: {error}", source.display()))?;
    let confirmed = confirmed_readings(root)?;
    let analyzer =
        Analyzer::new(Inflector::default()).map_err(|error| format!("build analyzer: {error}"))?;
    let mut types: BTreeMap<String, TypeEntry> = BTreeMap::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record: VerseRecord = serde_json::from_str(&line)
            .map_err(|error| format!("parse {}: {error}", source.display()))?;
        if !is_scriptural(&record.passage) {
            continue;
        }
        let text = strip_apparatus(&record.normalized_spelling);
        for token in tokenize(&text) {
            let surface = canonical_surface(&token.original);
            debug_assert!(!surface.contains(['\t', '\n']));
            let entry = types.entry(surface).or_default();
            entry.count += 1;
            if entry.references.len() < REFERENCE_CAP && !entry.references.contains(&record.passage)
            {
                entry.references.push(record.passage.clone());
            }
        }
    }
    let mut body = String::new();
    body.push_str(COLUMN_HEADER);
    body.push('\n');
    for (surface, entry) in &types {
        let non_lexical = classify_non_lexical(&analyzer, surface).unwrap_or("");
        let readings = confirmed
            .get(surface)
            .map(|set| set.iter().cloned().collect::<Vec<_>>().join(";"))
            .unwrap_or_default();
        body.push_str(&format!(
            "{surface}\t{}\t{non_lexical}\t{}\t{readings}\n",
            entry.count,
            entry.references.join(",")
        ));
    }
    let sha = hex_digest(body.as_bytes());
    let mut output = String::new();
    output.push_str("# gold_token_oracle.tsv — type-level token oracle for the Elizabeth Bible (Synodal gold gate).\n");
    output.push_str("# generated-by: cargo xtask synodal-gold-oracle\n");
    output.push_str(&format!("# source: {SOURCE_RELATIVE}\n"));
    output.push_str("# contract: docs/SYNODAL_GOLD_ORACLE.md\n");
    output.push_str(&format!("# reference-cap: {REFERENCE_CAP}\n"));
    output.push_str(&format!("# rows: {}\n", types.len()));
    output.push_str(&format!("# body-sha256: {sha}\n"));
    output.push_str(&body);
    Ok(output)
}

fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn generate(root: &Path) -> Result<(), Box<dyn Error>> {
    let started = std::time::Instant::now();
    let content = render_oracle(root)?;
    let path = root.join(ORACLE_RELATIVE);
    fs::write(&path, &content)?;
    ensure_defect_ledger(root)?;
    let rows = committed_row_count(&content)?;
    println!(
        "synodal-gold-oracle: wrote {} ({rows} types, {} bytes) in {:.1}s",
        ORACLE_RELATIVE,
        content.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn committed_row_count(content: &str) -> Result<usize, Box<dyn Error>> {
    let declared: usize = content
        .lines()
        .find_map(|line| line.strip_prefix("# rows: "))
        .ok_or("oracle lacks a # rows: header")?
        .parse()?;
    Ok(declared)
}

/// The staleness contract. With the intermediate source present (local),
/// regenerate and compare byte-for-byte. Without it (CI), validate the
/// committed artifact self-consistently: header block present, declared row
/// count matches the data rows, and the declared body sha matches the body.
fn check_oracle(root: &Path) -> Result<(), Box<dyn Error>> {
    ensure_defect_ledger(root)?;
    let path = root.join(ORACLE_RELATIVE);
    let committed =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    if root.join(SOURCE_RELATIVE).exists() {
        let regenerated = render_oracle(root)?;
        if regenerated != committed {
            return Err(format!(
                "{ORACLE_RELATIVE} is stale against {SOURCE_RELATIVE}; run cargo xtask synodal-gold-oracle"
            )
            .into());
        }
        println!("synodal-gold-oracle --check: regenerated and identical");
        return Ok(());
    }
    validate_committed(&committed)?;
    println!("synodal-gold-oracle --check: source absent; committed artifact self-validates");
    Ok(())
}

pub(crate) fn validate_committed(content: &str) -> Result<(), Box<dyn Error>> {
    if !content
        .lines()
        .any(|line| line == "# generated-by: cargo xtask synodal-gold-oracle")
    {
        return Err("oracle lacks its generator header".into());
    }
    let declared_sha = content
        .lines()
        .find_map(|line| line.strip_prefix("# body-sha256: "))
        .ok_or("oracle lacks a # body-sha256: header")?;
    let body_start = content
        .find(COLUMN_HEADER)
        .ok_or("oracle lacks its column header")?;
    let body = &content[body_start..];
    if hex_digest(body.as_bytes()) != declared_sha {
        return Err("oracle body does not match its declared sha256".into());
    }
    let declared_rows = committed_row_count(content)?;
    let actual_rows = body.lines().count() - 1;
    if declared_rows != actual_rows {
        return Err(
            format!("oracle declares {declared_rows} rows but contains {actual_rows}").into(),
        );
    }
    Ok(())
}

const LEDGER_HEADER: &str = "\
# gold_source_defects.tsv — witness-adjudicated digitization defects in the Ponomar Elizabeth Bible.
# A row may enter only under the two-of-three rule of docs/SYNODAL_GOLD_ORACLE.md: at a
# disagreement between the engine and Ponomar, both other witnesses (CrossWire, Wikisource)
# must side against Ponomar's reading; their readings are recorded as evidence.
passage\tponomar_surface\tadjudicated_surface\tcrosswire_reading\twikisource_reading\tnote
";

fn ensure_defect_ledger(root: &Path) -> Result<(), Box<dyn Error>> {
    let path = root.join(LEDGER_RELATIVE);
    if !path.exists() {
        fs::write(&path, LEDGER_HEADER)?;
    }
    Ok(())
}

/// The two independent digitizations consulted only at disagreement points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // consumed by the forthcoming synodal-gold gate
pub(crate) enum Witness {
    Crosswire,
    Wikisource,
}

impl Witness {
    fn relative_path(self) -> &'static str {
        match self {
            Self::Crosswire => CROSSWIRE_RELATIVE,
            Self::Wikisource => WIKISOURCE_RELATIVE,
        }
    }
}

/// Streams a witness JSONL and returns the canonical token surfaces of every
/// record for `passage` (in source order). Loads lazily: the 50+ MB witness
/// files are scanned line-by-line with a cheap substring pre-filter and are
/// never materialized or committed as artifacts.
#[allow(dead_code)] // consumed by the forthcoming synodal-gold gate
pub(crate) fn witness_token_surfaces(
    root: &Path,
    witness: Witness,
    passage: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    witness_token_surfaces_at(&root.join(witness.relative_path()), passage)
}

pub(crate) fn witness_token_surfaces_at(
    path: &Path,
    passage: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let file = fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let needle = format!("\"passage\":\"{passage}\"");
    let mut surfaces = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if !line.contains(&needle) {
            continue;
        }
        let record: VerseRecord = serde_json::from_str(&line)
            .map_err(|error| format!("parse {}: {error}", path.display()))?;
        if record.passage != passage {
            continue;
        }
        let text = strip_apparatus(&record.normalized_spelling);
        surfaces.extend(
            tokenize(&text)
                .into_iter()
                .map(|token| canonical_surface(&token.original)),
        );
    }
    Ok(surfaces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_apparatus_removes_zachalo_markers_and_cross_references() {
        assert_eq!(strip_apparatus("(Зача́ло 1) Пе́рвое сло́во"), "  Пе́рвое сло́во");
        assert_eq!(strip_apparatus("(За\u{2df1}\u{0487} 5) текст"), "  текст");
        assert_eq!(strip_apparatus("сло́во [Быт 17:15] рѣ́чь"), "сло́во   рѣ́чь");
    }

    #[test]
    fn strip_apparatus_keeps_supplied_readings() {
        assert_eq!(strip_apparatus("гдⷭ҇ь (и҆) речѐ"), "гдⷭ҇ь (и҆) речѐ");
        assert_eq!(
            strip_apparatus("несбала́нсиро (ва́нный"),
            "несбала́нсиро (ва́нный"
        );
    }

    #[test]
    fn canonical_surface_is_nfc() {
        // decomposed е + combining acute composes under NFC
        assert_eq!(canonical_surface("сло\u{0301}во"), "сло́во");
        assert_eq!(
            canonical_surface("сле\u{0301}дъ"),
            "сле\u{0301}дъ".nfc().collect::<String>()
        );
    }

    #[test]
    fn composite_passages_are_not_scriptural() {
        assert!(!is_scriptural("Composite.3.1"));
        assert!(is_scriptural("Gen.1.1"));
        assert!(is_scriptural("I_Macc.13.28"));
    }

    #[test]
    fn committed_oracle_round_trips_validation() {
        let body = format!(
            "{COLUMN_HEADER}\nсло́во\t3\t\tGen.1.1\tsynodal:noun:slovo|noun:nominative:singular:inanimate\n"
        );
        let sha = hex_digest(body.as_bytes());
        let content = format!(
            "# gold_token_oracle.tsv — test\n# generated-by: cargo xtask synodal-gold-oracle\n# reference-cap: 8\n# rows: 1\n# body-sha256: {sha}\n{body}"
        );
        validate_committed(&content).expect("valid");
        let tampered = content.replace("сло́во\t3", "сло́во\t4");
        assert!(validate_committed(&tampered).is_err());
        let wrong_rows = content.replace("# rows: 1", "# rows: 2");
        assert!(validate_committed(&wrong_rows).is_err());
    }

    #[test]
    fn witness_loader_streams_matching_passages() {
        let dir = std::env::temp_dir().join("gold-oracle-witness-test");
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("witness.jsonl");
        fs::write(
            &path,
            concat!(
                "{\"passage\":\"Gen.1.1\",\"normalized_spelling\":\"Въ нача́лѣ сотворѝ бг҃ъ\"}\n",
                "{\"passage\":\"Gen.1.2\",\"normalized_spelling\":\"землѧ́ же бѣ̀\"}\n"
            ),
        )
        .expect("write");
        let surfaces = witness_token_surfaces_at(&path, "Gen.1.1").expect("load");
        assert_eq!(surfaces, ["Въ", "нача́лѣ", "сотворѝ", "бг҃ъ"]);
        assert!(
            witness_token_surfaces_at(&path, "Gen.9.9")
                .expect("load")
                .is_empty()
        );
        fs::remove_file(&path).ok();
        fs::remove_dir(&dir).ok();
    }
}
