//! Streaming adapters for the large, locally pinned source files.

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    error, fmt, fs,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterConfig {
    pub source_id: String,
    pub source_recension: String,
    pub parse_failure_ceiling: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct StreamingReport {
    pub input_lines: usize,
    pub accepted_rows: usize,
    pub skipped_rows: usize,
    pub quarantined_rows: usize,
    pub reasons: BTreeMap<String, usize>,
}

#[derive(Debug)]
pub enum AdapterError {
    Io(io::Error),
    Json(serde_json::Error),
    Xml(quick_xml::Error),
    InvalidConfiguration(String),
    FailureCeiling { failures: usize, ceiling: usize },
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::Json(error) => error.fmt(formatter),
            Self::Xml(error) => error.fmt(formatter),
            Self::InvalidConfiguration(reason) => formatter.write_str(reason),
            Self::FailureCeiling { failures, ceiling } => write!(
                formatter,
                "source parse-failure ceiling exceeded: {failures} failures, ceiling {ceiling}"
            ),
        }
    }
}

impl error::Error for AdapterError {}

impl From<io::Error> for AdapterError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for AdapterError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<quick_xml::Error> for AdapterError {
    fn from(value: quick_xml::Error) -> Self {
        Self::Xml(value)
    }
}

pub type Result<T> = std::result::Result<T, AdapterError>;

#[derive(Serialize)]
struct QuarantineRow<'a> {
    source_id: &'a str,
    line: usize,
    reason: &'a str,
    raw: &'a str,
}

#[derive(Serialize)]
struct PonomarVerse<'a> {
    source_id: &'a str,
    source_recension: &'static str,
    target_recension: &'static str,
    chapter: u32,
    verse: u32,
    source_order: usize,
    raw_source: &'a str,
    normalized_text: String,
    epistemic_role: &'static str,
}

#[derive(Serialize)]
struct KaikkiOcsRow<'a> {
    source_id: &'a str,
    source_recension: &'static str,
    target_recension: Option<&'static str>,
    source_order: usize,
    source_lexeme_id: String,
    lemma: &'a str,
    part_of_speech: &'a str,
    raw_source: &'a str,
    forms: &'a Value,
    epistemic_role: &'static str,
}

/// Streams a Ponomar Elizabeth-Bible `.text` file to reviewable JSONL.
///
/// Chapter and verse identity, raw source spelling, and source order remain
/// explicit. Malformed rows go to a separate quarantine stream and count
/// against a strict caller-selected ceiling.
pub fn stream_ponomar_text(
    reader: impl BufRead,
    normalized: impl Write,
    quarantine: impl Write,
    config: &AdapterConfig,
) -> Result<StreamingReport> {
    validate_config(config, "synodal-russian")?;
    let mut normalized = BufWriter::new(normalized);
    let mut quarantine = BufWriter::new(quarantine);
    let mut report = StreamingReport::default();
    let mut chapter = None;

    for (offset, line) in reader.lines().enumerate() {
        let line_number = offset + 1;
        report.input_lines += 1;
        let line = line?;
        if let Some(value) = line.strip_prefix('#') {
            match value.parse::<u32>() {
                Ok(value) => {
                    chapter = Some(value);
                    report.skipped_rows += 1;
                }
                Err(_) => quarantine_row(
                    &mut quarantine,
                    &mut report,
                    config,
                    line_number,
                    "invalid-chapter-marker",
                    &line,
                )?,
            }
            ensure_ceiling(&report, config)?;
            continue;
        }
        if line.trim().is_empty() {
            report.skipped_rows += 1;
            continue;
        }
        let Some((verse, raw_source)) = line.split_once('|') else {
            quarantine_row(
                &mut quarantine,
                &mut report,
                config,
                line_number,
                "missing-verse-separator",
                &line,
            )?;
            ensure_ceiling(&report, config)?;
            continue;
        };
        let Some(chapter) = chapter else {
            quarantine_row(
                &mut quarantine,
                &mut report,
                config,
                line_number,
                "verse-before-chapter",
                &line,
            )?;
            ensure_ceiling(&report, config)?;
            continue;
        };
        let Ok(verse) = verse.parse::<u32>() else {
            quarantine_row(
                &mut quarantine,
                &mut report,
                config,
                line_number,
                "invalid-verse-number",
                &line,
            )?;
            ensure_ceiling(&report, config)?;
            continue;
        };
        let record = PonomarVerse {
            source_id: &config.source_id,
            source_recension: "synodal-russian",
            target_recension: "synodal-russian",
            chapter,
            verse,
            source_order: line_number,
            raw_source,
            normalized_text: raw_source.trim().replace("**", ""),
            epistemic_role: "evaluation-only-evidence",
        };
        serde_json::to_writer(&mut normalized, &record)?;
        normalized.write_all(b"\n")?;
        report.accepted_rows += 1;
    }
    normalized.flush()?;
    quarantine.flush()?;
    Ok(report)
}

/// Streams the pinned Kaikki/Wiktextract JSONL and selects OCS source records.
///
/// This adapter never emits a target surface record: every accepted row is
/// explicitly Old Church Slavonic inherited evidence with `target_recension`
/// absent. The raw JSON object is preserved for lossless later review.
pub fn stream_kaikki_ocs(
    reader: impl BufRead,
    normalized: impl Write,
    quarantine: impl Write,
    config: &AdapterConfig,
) -> Result<StreamingReport> {
    validate_config(config, "old-church-slavonic")?;
    let mut normalized = BufWriter::new(normalized);
    let mut quarantine = BufWriter::new(quarantine);
    let mut report = StreamingReport::default();

    for (offset, line) in reader.lines().enumerate() {
        let line_number = offset + 1;
        report.input_lines += 1;
        let line = line?;
        let value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => {
                quarantine_row(
                    &mut quarantine,
                    &mut report,
                    config,
                    line_number,
                    "invalid-json",
                    &line,
                )?;
                ensure_ceiling(&report, config)?;
                continue;
            }
        };
        let lang_code = value.get("lang_code").and_then(Value::as_str);
        let language = value.get("lang").and_then(Value::as_str);
        if lang_code != Some("cu") && language != Some("Old Church Slavonic") {
            report.skipped_rows += 1;
            continue;
        }
        let Some(lemma) = value.get("word").and_then(Value::as_str) else {
            quarantine_row(
                &mut quarantine,
                &mut report,
                config,
                line_number,
                "missing-lemma",
                &line,
            )?;
            ensure_ceiling(&report, config)?;
            continue;
        };
        let Some(part_of_speech) = value.get("pos").and_then(Value::as_str) else {
            quarantine_row(
                &mut quarantine,
                &mut report,
                config,
                line_number,
                "missing-part-of-speech",
                &line,
            )?;
            ensure_ceiling(&report, config)?;
            continue;
        };
        let empty_forms = Value::Array(Vec::new());
        let forms = value.get("forms").unwrap_or(&empty_forms);
        let source_lexeme_id = content_id(&config.source_id, lemma, part_of_speech, &line);
        let record = KaikkiOcsRow {
            source_id: &config.source_id,
            source_recension: "old-church-slavonic",
            target_recension: None,
            source_order: line_number,
            source_lexeme_id,
            lemma,
            part_of_speech,
            raw_source: &line,
            forms,
            epistemic_role: "inherited-ocs-evidence",
        };
        serde_json::to_writer(&mut normalized, &record)?;
        normalized.write_all(b"\n")?;
        report.accepted_rows += 1;
    }
    normalized.flush()?;
    quarantine.flush()?;
    Ok(report)
}

pub fn stream_ponomar_file(
    source: &Path,
    normalized: &Path,
    quarantine: &Path,
    config: &AdapterConfig,
) -> Result<StreamingReport> {
    stream_file(source, normalized, quarantine, config, stream_ponomar_text)
}

pub fn stream_kaikki_file(
    source: &Path,
    normalized: &Path,
    quarantine: &Path,
    config: &AdapterConfig,
) -> Result<StreamingReport> {
    stream_file(source, normalized, quarantine, config, stream_kaikki_ocs)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WikisourceRevision {
    title: String,
    page_id: u64,
    revision_id: u64,
    timestamp: String,
    mediawiki_sha1: String,
}

#[derive(Default)]
struct WikisourcePage {
    title: String,
    page_id: String,
    revision_id: String,
    timestamp: String,
    mediawiki_sha1: String,
    text: String,
}

#[derive(Clone, Copy)]
enum WikisourceField {
    Title,
    PageId,
    RevisionId,
    Timestamp,
    Sha1,
    Text,
}

/// Splits a pinned MediaWiki XML export into exact revision-content files.
///
/// The committed revision manifest is the authority for page identity. Every
/// page, revision, timestamp, and MediaWiki SHA-1 in the export must agree with
/// the manifest before the destination directory is replaced atomically.
pub fn materialize_wikisource_export(
    export: &Path,
    revision_manifest: &Path,
    destination: &Path,
) -> Result<StreamingReport> {
    let expected = load_wikisource_revisions(revision_manifest)?;
    let temporary = destination.with_extension(format!("tmp-{}", std::process::id()));
    if temporary.exists() {
        fs::remove_dir_all(&temporary)?;
    }
    fs::create_dir_all(&temporary)?;
    let result = materialize_wikisource_export_into(export, &temporary, &expected);
    match result {
        Ok(report) => {
            let backup = destination.with_extension(format!("backup-{}", std::process::id()));
            if backup.exists() {
                fs::remove_dir_all(&backup)?;
            }
            if destination.exists() {
                fs::rename(destination, &backup)?;
            }
            if let Err(error) = fs::rename(&temporary, destination) {
                if backup.exists() {
                    let _ = fs::rename(&backup, destination);
                }
                return Err(AdapterError::Io(error));
            }
            if backup.exists() {
                fs::remove_dir_all(backup)?;
            }
            Ok(report)
        }
        Err(error) => {
            let _ = fs::remove_dir_all(temporary);
            Err(error)
        }
    }
}

fn materialize_wikisource_export_into(
    export: &Path,
    destination: &Path,
    expected: &BTreeMap<u64, WikisourceRevision>,
) -> Result<StreamingReport> {
    use quick_xml::{Reader, events::Event};

    let mut reader = Reader::from_reader(BufReader::new(fs::File::open(export)?));
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut page = None::<WikisourcePage>;
    let mut field = None::<WikisourceField>;
    let mut in_revision = false;
    let mut seen = BTreeSet::new();
    let mut report = StreamingReport::default();

    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(event) => match event.local_name().as_ref() {
                b"page" => page = Some(WikisourcePage::default()),
                b"revision" => in_revision = true,
                b"title" => field = Some(WikisourceField::Title),
                b"id"
                    if in_revision
                        && page
                            .as_ref()
                            .is_some_and(|page| page.revision_id.is_empty()) =>
                {
                    field = Some(WikisourceField::RevisionId);
                }
                b"id"
                    if !in_revision
                        && page.as_ref().is_some_and(|page| page.page_id.is_empty()) =>
                {
                    field = Some(WikisourceField::PageId);
                }
                b"timestamp" => field = Some(WikisourceField::Timestamp),
                b"sha1" => field = Some(WikisourceField::Sha1),
                b"text" => field = Some(WikisourceField::Text),
                _ => {}
            },
            Event::Text(event) => {
                if let (Some(page), Some(field)) = (&mut page, field) {
                    let value = event.unescape()?.into_owned();
                    wikimedia_field_mut(page, field).push_str(&value);
                }
            }
            Event::CData(event) => {
                if let (Some(page), Some(field)) = (&mut page, field) {
                    let value = event.decode().map_err(|error| {
                        AdapterError::InvalidConfiguration(format!(
                            "invalid Wikisource CDATA encoding: {error}"
                        ))
                    })?;
                    wikimedia_field_mut(page, field).push_str(&value);
                }
            }
            Event::End(event) => match event.local_name().as_ref() {
                b"title" | b"id" | b"timestamp" | b"sha1" | b"text" => field = None,
                b"revision" => {
                    in_revision = false;
                    field = None;
                }
                b"page" => {
                    let page = page.take().ok_or_else(|| {
                        AdapterError::InvalidConfiguration(
                            "Wikisource export closed an unopened page".into(),
                        )
                    })?;
                    write_wikisource_page(page, destination, expected, &mut seen)?;
                    report.input_lines += 1;
                    report.accepted_rows += 1;
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if seen.len() != expected.len() {
        let missing: Vec<String> = expected
            .keys()
            .filter(|revision| !seen.contains(revision))
            .map(u64::to_string)
            .collect();
        return Err(AdapterError::InvalidConfiguration(format!(
            "Wikisource export omitted locked revisions: {}",
            missing.join(",")
        )));
    }
    Ok(report)
}

fn wikimedia_field_mut(page: &mut WikisourcePage, field: WikisourceField) -> &mut String {
    match field {
        WikisourceField::Title => &mut page.title,
        WikisourceField::PageId => &mut page.page_id,
        WikisourceField::RevisionId => &mut page.revision_id,
        WikisourceField::Timestamp => &mut page.timestamp,
        WikisourceField::Sha1 => &mut page.mediawiki_sha1,
        WikisourceField::Text => &mut page.text,
    }
}

fn write_wikisource_page(
    page: WikisourcePage,
    destination: &Path,
    expected: &BTreeMap<u64, WikisourceRevision>,
    seen: &mut BTreeSet<u64>,
) -> Result<()> {
    let page_id = page.page_id.parse::<u64>().map_err(|_| {
        AdapterError::InvalidConfiguration(format!("invalid Wikisource page ID {:?}", page.page_id))
    })?;
    let revision_id = page.revision_id.parse::<u64>().map_err(|_| {
        AdapterError::InvalidConfiguration(format!(
            "invalid Wikisource revision ID {:?}",
            page.revision_id
        ))
    })?;
    let locked = expected.get(&revision_id).ok_or_else(|| {
        AdapterError::InvalidConfiguration(format!(
            "Wikisource export contains unlocked revision {revision_id}"
        ))
    })?;
    if page.title != locked.title
        || page_id != locked.page_id
        || page.timestamp != locked.timestamp
        || page.mediawiki_sha1 != locked.mediawiki_sha1
    {
        return Err(AdapterError::InvalidConfiguration(format!(
            "Wikisource revision {revision_id} metadata disagrees with its lock"
        )));
    }
    if !seen.insert(revision_id) {
        return Err(AdapterError::InvalidConfiguration(format!(
            "duplicate Wikisource revision {revision_id}"
        )));
    }
    fs::write(
        destination.join(format!("{revision_id}.wikitext")),
        page.text,
    )?;
    Ok(())
}

fn load_wikisource_revisions(path: &Path) -> Result<BTreeMap<u64, WikisourceRevision>> {
    const HEADER: &str = "title\tpage_id\trevision_id\ttimestamp\tmediawiki_sha1";
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err(AdapterError::InvalidConfiguration(
            "invalid Wikisource revision-lock header".into(),
        ));
    }
    let mut revisions = BTreeMap::new();
    for (offset, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(AdapterError::InvalidConfiguration(format!(
                "invalid Wikisource revision-lock row {}",
                offset + 2
            )));
        }
        let revision = WikisourceRevision {
            title: fields[0].into(),
            page_id: fields[1].parse().map_err(|_| {
                AdapterError::InvalidConfiguration(format!(
                    "invalid page ID in Wikisource revision-lock row {}",
                    offset + 2
                ))
            })?,
            revision_id: fields[2].parse().map_err(|_| {
                AdapterError::InvalidConfiguration(format!(
                    "invalid revision ID in Wikisource revision-lock row {}",
                    offset + 2
                ))
            })?,
            timestamp: fields[3].into(),
            mediawiki_sha1: fields[4].into(),
        };
        if revisions.insert(revision.revision_id, revision).is_some() {
            return Err(AdapterError::InvalidConfiguration(format!(
                "duplicate revision ID in Wikisource revision-lock row {}",
                offset + 2
            )));
        }
    }
    if revisions.is_empty() {
        return Err(AdapterError::InvalidConfiguration(
            "Wikisource revision lock is empty".into(),
        ));
    }
    Ok(revisions)
}

fn stream_file(
    source: &Path,
    normalized: &Path,
    quarantine: &Path,
    config: &AdapterConfig,
    adapter: impl FnOnce(
        BufReader<fs::File>,
        BufWriter<fs::File>,
        BufWriter<fs::File>,
        &AdapterConfig,
    ) -> Result<StreamingReport>,
) -> Result<StreamingReport> {
    let normalized_temporary = temporary_path(normalized);
    let quarantine_temporary = temporary_path(quarantine);
    create_parent(normalized)?;
    create_parent(quarantine)?;
    let report = adapter(
        BufReader::new(fs::File::open(source)?),
        BufWriter::new(fs::File::create(&normalized_temporary)?),
        BufWriter::new(fs::File::create(&quarantine_temporary)?),
        config,
    );
    match report {
        Ok(report) => {
            fs::rename(normalized_temporary, normalized)?;
            fs::rename(quarantine_temporary, quarantine)?;
            Ok(report)
        }
        Err(error) => {
            let _ = fs::remove_file(normalized_temporary);
            let _ = fs::remove_file(quarantine_temporary);
            Err(error)
        }
    }
}

fn quarantine_row(
    writer: &mut impl Write,
    report: &mut StreamingReport,
    config: &AdapterConfig,
    line: usize,
    reason: &str,
    raw: &str,
) -> Result<()> {
    serde_json::to_writer(
        &mut *writer,
        &QuarantineRow {
            source_id: &config.source_id,
            line,
            reason,
            raw,
        },
    )?;
    writer.write_all(b"\n")?;
    report.quarantined_rows += 1;
    *report.reasons.entry(reason.into()).or_default() += 1;
    Ok(())
}

fn ensure_ceiling(report: &StreamingReport, config: &AdapterConfig) -> Result<()> {
    if report.quarantined_rows > config.parse_failure_ceiling {
        Err(AdapterError::FailureCeiling {
            failures: report.quarantined_rows,
            ceiling: config.parse_failure_ceiling,
        })
    } else {
        Ok(())
    }
}

fn validate_config(config: &AdapterConfig, expected_recension: &str) -> Result<()> {
    if config.source_id.trim().is_empty() {
        return Err(AdapterError::InvalidConfiguration(
            "adapter source ID cannot be empty".into(),
        ));
    }
    let lower = config.source_id.to_lowercase();
    if lower.contains("slovowiki") || lower.contains("interslavic") {
        return Err(AdapterError::InvalidConfiguration(
            "forbidden linguistic authority in adapter source ID".into(),
        ));
    }
    if config.source_recension != expected_recension {
        return Err(AdapterError::InvalidConfiguration(format!(
            "adapter requires source_recension {expected_recension}, found {}",
            config.source_recension
        )));
    }
    Ok(())
}

fn content_id(source_id: &str, lemma: &str, part_of_speech: &str, raw: &str) -> String {
    let mut hasher = Sha256::new();
    for value in [source_id, lemma, part_of_speech, raw] {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    let digest = hasher.finalize();
    let hash: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("ocs:kaikki:{}", &hash[..24])
}

fn temporary_path(destination: &Path) -> PathBuf {
    destination.with_extension(format!(
        "{}.tmp",
        destination
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("output")
    ))
}

fn create_parent(path: &Path) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        AdapterError::InvalidConfiguration(format!("{} has no parent", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn ponomar_adapter_preserves_raw_and_quarantines_malformed_rows() {
        let input = "#1\n1| **Кни́га** родства̀\nbad row\n";
        let mut normalized = Vec::new();
        let mut quarantine = Vec::new();
        let report = stream_ponomar_text(
            Cursor::new(input),
            &mut normalized,
            &mut quarantine,
            &AdapterConfig {
                source_id: "ponomar-elizabeth-bible-fixture".into(),
                source_recension: "synodal-russian".into(),
                parse_failure_ceiling: 1,
            },
        )
        .expect("one admitted parse failure");
        assert_eq!(report.accepted_rows, 1);
        assert_eq!(report.quarantined_rows, 1);
        let normalized = String::from_utf8(normalized).expect("UTF-8 output");
        assert!(normalized.contains(" **Кни́га** родства̀"));
        assert!(normalized.contains("Кни́га родства̀"));
        assert!(
            String::from_utf8(quarantine)
                .expect("UTF-8 quarantine")
                .contains("missing-verse-separator")
        );
    }

    #[test]
    fn kaikki_adapter_keeps_ocs_separate_from_target_records() {
        let input = concat!(
            "{\"lang_code\":\"cu\",\"lang\":\"Old Church Slavonic\",\"word\":\"градъ\",\"pos\":\"noun\",\"forms\":[]}\n",
            "{\"lang_code\":\"en\",\"word\":\"town\",\"pos\":\"noun\"}\n"
        );
        let mut normalized = Vec::new();
        let report = stream_kaikki_ocs(
            Cursor::new(input),
            &mut normalized,
            Vec::new(),
            &AdapterConfig {
                source_id: "english-wiktionary-ocs-kaikki-fixture".into(),
                source_recension: "old-church-slavonic".into(),
                parse_failure_ceiling: 0,
            },
        )
        .expect("valid stream");
        assert_eq!(report.accepted_rows, 1);
        assert_eq!(report.skipped_rows, 1);
        let output = String::from_utf8(normalized).expect("UTF-8 output");
        assert!(output.contains("\"target_recension\":null"));
        assert!(output.contains("\"source_recension\":\"old-church-slavonic\""));
    }

    #[test]
    fn strict_failure_ceiling_aborts() {
        let error = stream_ponomar_text(
            Cursor::new("not a verse\n"),
            Vec::new(),
            Vec::new(),
            &AdapterConfig {
                source_id: "fixture".into(),
                source_recension: "synodal-russian".into(),
                parse_failure_ceiling: 0,
            },
        )
        .expect_err("ceiling must be enforced");
        assert!(matches!(error, AdapterError::FailureCeiling { .. }));
    }

    #[test]
    fn wikisource_export_is_split_only_after_revision_lock_validation() {
        let directory =
            std::env::temp_dir().join(format!("synodal-wikisource-fixture-{}", std::process::id()));
        if directory.exists() {
            fs::remove_dir_all(&directory).expect("old fixture cleanup");
        }
        fs::create_dir_all(&directory).expect("fixture directory");
        let export = directory.join("export.xml");
        fs::write(
            &export,
            concat!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>",
                "<mediawiki><page><title>Бі́блїа</title><id>10</id><revision>",
                "<id>20</id><timestamp>2026-01-02T03:04:05Z</timestamp>",
                "<contributor><id>999</id></contributor>",
                "<text xml:space=\"preserve\">слово &amp; гласъ</text>",
                "<sha1>lockedsha1</sha1></revision></page></mediawiki>"
            ),
        )
        .expect("export fixture");
        let revisions = directory.join("revisions.tsv");
        fs::write(
            &revisions,
            concat!(
                "title\tpage_id\trevision_id\ttimestamp\tmediawiki_sha1\n",
                "Бі́блїа\t10\t20\t2026-01-02T03:04:05Z\tlockedsha1\n"
            ),
        )
        .expect("revision fixture");
        let destination = directory.join("revisions");
        let report = materialize_wikisource_export(&export, &revisions, &destination)
            .expect("locked export");
        assert_eq!(report.accepted_rows, 1);
        assert_eq!(
            fs::read_to_string(destination.join("20.wikitext")).expect("revision text"),
            "слово & гласъ"
        );
        fs::remove_dir_all(directory).expect("fixture cleanup");
    }
}
