//! Deterministic source adapters for the Synodal candidate pipeline.
//!
//! These adapters are offline: they consume only checksum-admitted cache bytes
//! and emit review candidates. They never promote rows into runtime registries.

use calamine::{Data, Reader as CalamineReader, Xls, open_workbook};
use quick_xml::{Reader as XmlReader, events::Event};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

pub type PipelineResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
pub struct PipelineOptions {
    pub workspace_root: PathBuf,
    pub cache: PathBuf,
    pub intermediate: PathBuf,
    pub quarantine: PathBuf,
    pub source: Option<String>,
    pub failure_ceiling: usize,
    pub keep_work: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PipelineReport {
    pub schema_version: u8,
    pub accepted_records: usize,
    pub quarantined_records: usize,
    pub skipped_records: usize,
    pub source_reports: BTreeMap<String, SourceReport>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceReport {
    pub accepted_records: usize,
    pub quarantined_records: usize,
    pub skipped_records: usize,
    pub rejection_reasons: BTreeMap<String, usize>,
    pub output_sha256: String,
}

#[derive(Clone, Debug, Serialize)]
struct Candidate {
    schema_version: u8,
    candidate_id: String,
    source_record_id: String,
    source_id: String,
    source_revision: String,
    artifact_sha256: String,
    source_recension: String,
    target_recension: Option<String>,
    work: String,
    edition: String,
    passage: String,
    source_order: usize,
    raw_spelling: String,
    normalized_spelling: String,
    part_of_speech: String,
    grammatical_cell: String,
    authority_roles: Vec<String>,
    epistemic_roles: Vec<String>,
    upstream_lineage: Vec<String>,
    license: String,
    redistribution: String,
    confidence_basis_points: u16,
    partition: String,
    parse_status: String,
    review_status: String,
    transformations: Vec<String>,
}

#[derive(Serialize)]
struct QuarantineRecord {
    schema_version: u8,
    source_id: String,
    source_record_id: String,
    source_order: usize,
    reason: String,
    raw: String,
}

#[derive(Clone, Debug)]
struct SourceSpec {
    id: &'static str,
    revision: &'static str,
    recension: &'static str,
    target: Option<&'static str>,
    work: &'static str,
    edition: &'static str,
    license: &'static str,
    redistribution: &'static str,
    authority: &'static [&'static str],
    epistemic: &'static [&'static str],
    lineage: &'static [&'static str],
}

#[derive(Clone, Debug)]
struct LockedArtifact {
    path: String,
    sha256: String,
}

#[derive(Clone, Debug)]
struct LockedSource {
    aggregate_sha256: String,
    artifacts: Vec<LockedArtifact>,
}

struct Sink {
    spec: SourceSpec,
    artifact_hash: String,
    accepted: BufWriter<File>,
    quarantine: BufWriter<File>,
    report: SourceReport,
    ceiling: usize,
}

impl Sink {
    #[allow(clippy::too_many_arguments)]
    fn accept(
        &mut self,
        record_key: &str,
        order: usize,
        passage: &str,
        raw: &str,
        normalized: &str,
        part_of_speech: &str,
        cell: &str,
        transformations: Vec<String>,
    ) -> PipelineResult<()> {
        if normalized.trim().is_empty() {
            return self.reject(record_key, order, "empty-normalized-record", raw);
        }
        let source_record_id = stable_id("record", &[self.spec.id, record_key]);
        let candidate_id = stable_id(
            "candidate",
            &[
                self.spec.id,
                self.spec.revision,
                &self.artifact_hash,
                &source_record_id,
                raw,
                normalized,
                cell,
            ],
        );
        let candidate = Candidate {
            schema_version: 1,
            candidate_id,
            source_record_id,
            source_id: self.spec.id.into(),
            source_revision: self.spec.revision.into(),
            artifact_sha256: self.artifact_hash.clone(),
            source_recension: self.spec.recension.into(),
            target_recension: self.spec.target.map(str::to_owned),
            work: self.spec.work.into(),
            edition: self.spec.edition.into(),
            passage: passage.into(),
            source_order: order,
            raw_spelling: raw.into(),
            normalized_spelling: normalized.into(),
            part_of_speech: part_of_speech.into(),
            grammatical_cell: cell.into(),
            authority_roles: strings(self.spec.authority),
            epistemic_roles: strings(self.spec.epistemic),
            upstream_lineage: strings(self.spec.lineage),
            license: self.spec.license.into(),
            redistribution: self.spec.redistribution.into(),
            confidence_basis_points: if self.spec.target.is_some() {
                8000
            } else {
                5500
            },
            partition: passage_partition(self.spec.id, passage),
            parse_status: "parsed".into(),
            review_status: "candidate-unreviewed".into(),
            transformations,
        };
        serde_json::to_writer(&mut self.accepted, &candidate)?;
        self.accepted.write_all(b"\n")?;
        self.report.accepted_records += 1;
        Ok(())
    }

    fn reject(
        &mut self,
        record_key: &str,
        order: usize,
        reason: &str,
        raw: &str,
    ) -> PipelineResult<()> {
        serde_json::to_writer(
            &mut self.quarantine,
            &QuarantineRecord {
                schema_version: 1,
                source_id: self.spec.id.into(),
                source_record_id: stable_id("record", &[self.spec.id, record_key]),
                source_order: order,
                reason: reason.into(),
                raw: raw.into(),
            },
        )?;
        self.quarantine.write_all(b"\n")?;
        self.report.quarantined_records += 1;
        *self
            .report
            .rejection_reasons
            .entry(reason.into())
            .or_default() += 1;
        if self.report.quarantined_records > self.ceiling {
            return Err(format!(
                "{} exceeded its parse-failure ceiling: {} > {}",
                self.spec.id, self.report.quarantined_records, self.ceiling
            )
            .into());
        }
        Ok(())
    }

    fn skip(&mut self) {
        self.report.skipped_records += 1;
    }

    fn finish(mut self, output: &Path) -> PipelineResult<SourceReport> {
        self.accepted.flush()?;
        self.quarantine.flush()?;
        self.report.output_sha256 = sha256_file(output)?;
        Ok(self.report)
    }
}

/// Runs every selected source adapter and atomically replaces the local layers.
pub fn run_pipeline(options: &PipelineOptions) -> PipelineResult<PipelineReport> {
    let staging = options
        .intermediate
        .with_extension(format!("staging-{}", std::process::id()));
    let quarantine_staging = options
        .quarantine
        .with_extension(format!("staging-{}", std::process::id()));
    remove_if_exists(&staging)?;
    remove_if_exists(&quarantine_staging)?;
    fs::create_dir_all(&staging)?;
    fs::create_dir_all(&quarantine_staging)?;
    let work = staging.join("work");
    fs::create_dir_all(&work)?;
    let lock = load_source_lock(&options.workspace_root)?;

    let result = run_selected(options, &staging, &quarantine_staging, &work, &lock);
    let mut report = match result {
        Ok(report) => report,
        Err(error) => {
            let _ = remove_if_exists(&staging);
            let _ = remove_if_exists(&quarantine_staging);
            return Err(error);
        }
    };
    if !options.keep_work {
        remove_if_exists(&work)?;
    }
    atomic_replace_directory(&staging, &options.intermediate)?;
    atomic_replace_directory(&quarantine_staging, &options.quarantine)?;
    report.schema_version = 1;
    fs::write(
        options.intermediate.join("adapter-reports.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}

type Adapter =
    fn(&PipelineOptions, &Path, &Path, &Path, &LockedSource) -> PipelineResult<SourceReport>;

fn run_selected(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    work: &Path,
    lock: &BTreeMap<String, LockedSource>,
) -> PipelineResult<PipelineReport> {
    let adapters: &[(&str, Adapter)] = &[
        ("ponomar-elizabeth-bible-2026-08-09", adapt_ponomar),
        ("alypy-gamanovich-grammar-web-2023", adapt_alypy),
        ("dyachenko-1900-scan", adapt_dyachenko),
        (
            "wikisource-church-slavonic-bible-2026-08-09",
            adapt_wikisource,
        ),
        ("crosswire-csl-elizabeth-1.5.2", adapt_crosswire),
        ("polivanova-osd-source", adapt_polivanova_osd),
        ("polivanova-fup-2023", adapt_polivanova_tei),
        ("ud-ocs-proiel-r2.18", adapt_ud),
        ("syntacticus-20230428", adapt_syntacticus),
        ("ccmh-2021-04-23", adapt_ccmh),
        ("diacu-1.0", adapt_diacu),
        ("english-wiktionary-ocs-kaikki-2026-08-07", adapt_wiktionary),
        (
            "ponomar-modern-church-slavonic-corpus-2016",
            adapt_modern_dictionary_and_frequency,
        ),
    ];
    let mut report = PipelineReport::default();
    let mut matched = false;
    for (source_id, adapter) in adapters {
        if options
            .source
            .as_deref()
            .is_some_and(|selected| selected != *source_id)
        {
            continue;
        }
        matched = true;
        let locked = lock
            .get(*source_id)
            .ok_or_else(|| format!("source lock has no artifacts for {source_id}"))?;
        let source_report = adapter(options, output, quarantine, work, locked)?;
        report.accepted_records += source_report.accepted_records;
        report.quarantined_records += source_report.quarantined_records;
        report.skipped_records += source_report.skipped_records;
        report
            .source_reports
            .insert((*source_id).into(), source_report);
    }
    if !matched {
        return Err(format!(
            "source {:?} has no candidate adapter",
            options.source.as_deref().unwrap_or("<all>")
        )
        .into());
    }
    Ok(report)
}

fn load_source_lock(root: &Path) -> PipelineResult<BTreeMap<String, LockedSource>> {
    let text = fs::read_to_string(root.join("references/SOURCE_LOCK.tsv"))?;
    let mut grouped = BTreeMap::<String, Vec<LockedArtifact>>::new();
    for (offset, line) in text.lines().enumerate().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 10 {
            return Err(format!("invalid source-lock row {}", offset + 1).into());
        }
        grouped
            .entry(fields[0].into())
            .or_default()
            .push(LockedArtifact {
                path: fields[4].into(),
                sha256: fields[5].into(),
            });
    }
    Ok(grouped
        .into_iter()
        .map(|(id, mut artifacts)| {
            artifacts.sort_by(|left, right| left.path.cmp(&right.path));
            let mut hasher = Sha256::new();
            for artifact in &artifacts {
                hasher.update(artifact.sha256.as_bytes());
                hasher.update(b"  ");
                hasher.update(artifact.path.as_bytes());
                hasher.update(b"\n");
            }
            let aggregate_sha256 = hasher
                .finalize()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            (
                id,
                LockedSource {
                    aggregate_sha256,
                    artifacts,
                },
            )
        })
        .collect())
}

fn source_path(cache: &Path, locked: &LockedSource, suffix: &str) -> PipelineResult<PathBuf> {
    let artifact = locked
        .artifacts
        .iter()
        .find(|artifact| artifact.path.ends_with(suffix))
        .ok_or_else(|| format!("locked source has no artifact ending in {suffix:?}"))?;
    let relative = Path::new(&artifact.path)
        .strip_prefix("downloads")
        .unwrap_or_else(|_| Path::new(&artifact.path));
    Ok(cache.join(relative))
}

fn new_sink(
    output: &Path,
    quarantine: &Path,
    spec: SourceSpec,
    locked: &LockedSource,
    ceiling: usize,
) -> PipelineResult<(Sink, PathBuf)> {
    let output_path = output.join(format!("{}.jsonl", spec.id));
    let quarantine_path = quarantine.join(format!("{}.jsonl", spec.id));
    let sink = Sink {
        spec,
        artifact_hash: locked.aggregate_sha256.clone(),
        accepted: BufWriter::new(File::create(&output_path)?),
        quarantine: BufWriter::new(File::create(quarantine_path)?),
        report: SourceReport::default(),
        ceiling,
    };
    Ok((sink, output_path))
}

fn adapt_ponomar(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = synodal_spec(
        "ponomar-elizabeth-bible-2026-08-09",
        "0af645f438856f45c22026912d2e4a9ce495e531",
        "Elizabeth Bible",
        "Ponomar Unicode text",
        "GPL-3.0-or-later; individual Bible-file status under review",
        "evaluation-only",
        &["lexical", "orthographic", "accentual", "evaluation"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let archive = source_path(&options.cache, locked, ".tar.gz")?;
    let directory = work.join("ponomar");
    extract_tar(&archive, &directory)?;
    let mut files = recursive_files(&directory, Some("text"))?;
    files.retain(|path| path.to_string_lossy().contains("/languages/cu/bible/elis/"));
    if files.len() != 78 {
        return Err(format!(
            "Ponomar archive contains {} Elizabeth .text files, expected 78",
            files.len()
        )
        .into());
    }
    let mut frequency = BTreeMap::<String, usize>::new();
    let mut order = 0;
    for file in files {
        let book = file
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or("non-UTF-8 Ponomar book")?;
        let mut chapter = None;
        for line in BufReader::new(File::open(&file)?).lines() {
            let line = line?;
            if let Some(value) = line.strip_prefix('#') {
                chapter = value.parse::<u32>().ok();
                if chapter.is_none() {
                    sink.reject(
                        &format!("{book}:chapter:{order}"),
                        order,
                        "invalid-chapter-marker",
                        &line,
                    )?;
                }
                continue;
            }
            if line.trim().is_empty() {
                sink.skip();
                continue;
            }
            order += 1;
            let Some((verse, raw)) = line.split_once('|') else {
                sink.reject(
                    &format!("{book}:line:{order}"),
                    order,
                    "missing-verse-separator",
                    &line,
                )?;
                continue;
            };
            let Some(chapter) = chapter else {
                sink.reject(
                    &format!("{book}:line:{order}"),
                    order,
                    "verse-before-chapter",
                    &line,
                )?;
                continue;
            };
            if verse.parse::<u32>().is_err() {
                sink.reject(
                    &format!("{book}:{chapter}:{verse}"),
                    order,
                    "invalid-verse-number",
                    &line,
                )?;
                continue;
            }
            let normalized = strip_ponomar_markup(raw);
            for token in word_tokens(&normalized) {
                *frequency.entry(token).or_default() += 1;
            }
            let passage = format!("{book}.{chapter}.{verse}");
            sink.accept(
                &passage,
                order,
                &passage,
                raw,
                &normalized,
                "corpus-passage",
                "verse",
                vec!["remove-editorial-markup".into()],
            )?;
        }
    }
    let mut rows: Vec<(String, usize)> = frequency.into_iter().collect();
    rows.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    let mut writer = BufWriter::new(File::create(output.join("ponomar-frequency.tsv"))?);
    writer.write_all(b"rank\ttoken\tcount\n")?;
    for (offset, (token, count)) in rows.into_iter().enumerate() {
        writeln!(writer, "{}\t{}\t{}", offset + 1, token, count)?;
    }
    writer.flush()?;
    sink.finish(&output_path)
}

fn adapt_alypy(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    _work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = synodal_spec(
        "alypy-gamanovich-grammar-web-2023",
        "web-corrections-through-2023-12-10",
        "Grammar of the Church Slavonic Language",
        "Ponomar corrected web text based on 1991 edition",
        "Ponomar site terms and underlying edition rights",
        "evaluation-only",
        &["morphological", "orthographic", "accentual", "numeral"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let mut order = 0;
    for artifact in &locked.artifacts {
        let relative = Path::new(&artifact.path)
            .strip_prefix("downloads")
            .unwrap_or_else(|_| Path::new(&artifact.path));
        let path = options.cache.join(relative);
        let html = fs::read_to_string(&path)?;
        let section = first_between(&html, "<h3>", "</h3>")
            .map(|value| strip_html(&value))
            .unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            });
        let page = path.file_name().unwrap_or_default().to_string_lossy();
        order += 1;
        sink.accept(
            &format!("{page}:section"),
            order,
            &section,
            &html,
            &section,
            "grammar-rule",
            "section",
            vec!["html-text-extraction".into()],
        )?;
        for (index, witness) in class_spans(&html, "DSText").into_iter().enumerate() {
            order += 1;
            let normalized = decode_html(&strip_html(&witness));
            sink.accept(
                &format!("{page}:witness:{index}"),
                order,
                &section,
                &witness,
                &normalized,
                "paradigm-witness",
                "untyped-review-candidate",
                vec![
                    "DSText-span-extraction".into(),
                    "html-entity-decoding".into(),
                ],
            )?;
        }
    }
    sink.finish(&output_path)
}

fn adapt_dyachenko(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = SourceSpec {
        id: "dyachenko-1900-scan",
        revision: "1900-scan-locked-by-sha256",
        recension: "mixed",
        target: None,
        work: "Complete Church Slavonic Dictionary",
        edition: "Grigory D’yachenko, 1900 scan",
        license: "public-domain scan; host terms apply",
        redistribution: "candidate-metadata-only",
        authority: &["lexical", "semantic"],
        epistemic: &["mixed-dictionary-candidate"],
        lineage: &[],
    };
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    require_tool(
        "djvutxt",
        "install DjVuLibre (for example: brew install djvulibre)",
    )?;
    let source = source_path(&options.cache, locked, ".djvu")?;
    let ocr = work.join("dyachenko.txt");
    if !Command::new("djvutxt")
        .arg(&source)
        .arg(&ocr)
        .status()?
        .success()
    {
        return Err("djvutxt failed while extracting D’yachenko".into());
    }
    let mut order = 0;
    if fs::metadata(&ocr)?.len() > 0 {
        let mut page = 1;
        for line in BufReader::new(File::open(ocr)?).lines() {
            let line = line?;
            if line.contains('\u{c}') {
                page += line.matches('\u{c}').count();
                continue;
            }
            if line.trim().is_empty() {
                sink.skip();
                continue;
            }
            order += 1;
            sink.accept(
                &format!("page:{page}:line:{order}"),
                order,
                &format!("page {page}"),
                &line,
                line.trim(),
                "dictionary-entry-candidate",
                "embedded-ocr-line;correction-status=uncorrected",
                vec!["embedded-djvu-ocr".into()],
            )?;
        }
    } else {
        require_tool(
            "ddjvu",
            "install DjVuLibre (for example: brew install djvulibre)",
        )?;
        require_tool(
            "tesseract",
            "install Tesseract 5 to run the checksum-pinned OCR model",
        )?;
        let tessdata = source_path(&options.cache, locked, "rus.traineddata")?;
        let pages = djvu_page_count(&source)?;
        let page_output = work.join("dyachenko-ocr-pages");
        run_pinned_ocr(
            &source,
            tessdata.parent().ok_or("OCR model has no parent")?,
            &page_output,
            pages,
        )?;
        for page in 1..=pages {
            for line in read_tesseract_lines(&page_output.join(format!("page-{page}.tsv")))? {
                order += 1;
                let passage = format!(
                    "page {page}; bbox={},{},{},{}",
                    line.left, line.top, line.width, line.height
                );
                sink.accept(
                    &format!("page:{page}:line:{}", line.line_key),
                    order,
                    &passage,
                    &line.text,
                    &line.text,
                    "dictionary-entry-candidate",
                    &format!(
                        "external-ocr-line;mean-confidence={};correction-status=uncorrected",
                        line.mean_confidence
                    ),
                    vec![
                        "DjVuLibre-page-render".into(),
                        "tesseract-5-tessdata-fast-rus-4.1.0".into(),
                    ],
                )?;
            }
        }
    }
    if order == 0 {
        return Err("D’yachenko adapter produced no OCR records".into());
    }
    sink.finish(&output_path)
}

fn adapt_wikisource(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    _work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = synodal_spec(
        "wikisource-church-slavonic-bible-2026-08-09",
        "78-exact-revisions-WIKISOURCE_REVISIONS.tsv",
        "Church Slavonic Bible",
        "Wikisource exact revisions",
        "CC BY-SA 4.0",
        "allowed",
        &["lexical", "orthographic", "accentual", "evaluation"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let book_codes = load_wikisource_book_codes(&options.workspace_root)?;
    let mut order = 0;
    for artifact in &locked.artifacts {
        let relative = Path::new(&artifact.path)
            .strip_prefix("downloads")
            .unwrap_or_else(|_| Path::new(&artifact.path));
        let revision = Path::new(&artifact.path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let book = book_codes
            .get(revision.as_ref())
            .ok_or_else(|| format!("Wikisource revision {revision} has no reviewed book code"))?;
        if book == "-" {
            continue;
        }
        let mut chapter = None::<u32>;
        let mut current_verse = None::<u32>;
        let mut awaiting_first_verse = false;
        for (line_offset, line) in BufReader::new(File::open(options.cache.join(relative))?)
            .lines()
            .enumerate()
        {
            let line = line?;
            if let Some(anchor) = template_argument(&line, "Anchor|") {
                chapter = anchor.split_whitespace().next_back().and_then(|number| {
                    synodal_church_slavonic_core::parse_cyrillic_numeral(number).ok()
                });
                if chapter.is_none() {
                    sink.reject(
                        &format!("revision:{revision}:line:{}", line_offset + 1),
                        line_offset + 1,
                        "invalid-chapter-anchor",
                        &line,
                    )?;
                }
                current_verse = None;
                awaiting_first_verse = chapter.is_some();
                continue;
            }
            let Some(chapter) = chapter else {
                sink.skip();
                continue;
            };
            if is_wikisource_paratext(&line) {
                sink.skip();
                continue;
            }
            let normalized = strip_wikitext(&line);
            if normalized.is_empty() {
                sink.skip();
                continue;
            }
            let marked_verse = wikisource_verse_number(&line);
            if let Some(verse) = marked_verse {
                current_verse = Some(verse);
                awaiting_first_verse = false;
            } else if awaiting_first_verse && normalized.chars().any(char::is_alphabetic) {
                current_verse = Some(1);
                awaiting_first_verse = false;
            }
            let Some(verse) = current_verse else {
                sink.skip();
                continue;
            };
            order += 1;
            let passage = format!("{book}.{chapter}.{verse}");
            let record = format!("{passage}:line:{}", line_offset + 1);
            sink.accept(
                &record,
                order,
                &passage,
                &line,
                &normalized,
                "community-transcription",
                "verse-or-paratext",
                vec![
                    "preserve-raw-template-lineage".into(),
                    "deterministic-wikitext-strip".into(),
                    "reviewed-book-code-and-cyrillic-verse-alignment".into(),
                ],
            )?;
        }
    }
    sink.finish(&output_path)
}

fn adapt_crosswire(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = synodal_spec(
        "crosswire-csl-elizabeth-1.5.2",
        "1.5.2-2011-08-17",
        "Elizabeth Bible",
        "CrossWire CSlElizabeth 1.5.2, modernized spelling",
        "Public Domain",
        "allowed",
        &["lexical", "evaluation", "orthographic-contrast"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    require_tool(
        "mod2imp",
        "install SWORD utilities (for example: brew install sword) to extract CSlElizabeth",
    )?;
    let archive = source_path(&options.cache, locked, ".zip")?;
    let module = work.join("crosswire");
    extract_zip(&archive, &module)?;
    let mut child = Command::new("mod2imp")
        .arg("CSlElizabeth")
        .env("SWORD_PATH", &module)
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or("mod2imp stdout unavailable")?;
    let mut reference = String::new();
    let mut text = String::new();
    let mut order = 0;
    for line in BufReader::new(stdout).lines() {
        let line = line?;
        if let Some(next) = line.strip_prefix("$$$") {
            if !reference.is_empty() {
                order += 1;
                let normalized = strip_osis(&text);
                let passage = canonical_crosswire_passage(&reference);
                if normalized.is_empty() || passage.is_none() {
                    sink.skip();
                    reference = next.trim().into();
                    text.clear();
                    continue;
                }
                let passage = passage.ok_or("checked CrossWire passage")?;
                sink.accept(
                    &reference,
                    order,
                    &passage,
                    &text,
                    &normalized,
                    "modernized-biblical-text",
                    "verse",
                    vec!["SWORD-mod2imp".into(), "OSIS-strip".into()],
                )?;
            }
            reference = next.trim().into();
            text.clear();
        } else {
            if !text.is_empty() {
                text.push('\n');
            }
            text.push_str(&line);
        }
    }
    if !reference.is_empty() {
        order += 1;
        let normalized = strip_osis(&text);
        let passage = canonical_crosswire_passage(&reference);
        if normalized.is_empty() || passage.is_none() {
            sink.skip();
        } else {
            let passage = passage.ok_or("checked CrossWire passage")?;
            sink.accept(
                &reference,
                order,
                &passage,
                &text,
                &normalized,
                "modernized-biblical-text",
                "verse",
                vec!["SWORD-mod2imp".into(), "OSIS-strip".into()],
            )?;
        }
    }
    if !child.wait()?.success() {
        return Err("mod2imp failed while extracting CSlElizabeth".into());
    }
    sink.finish(&output_path)
}

fn adapt_polivanova_osd(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = ocs_spec(
        "polivanova-osd-source",
        "osd.zip-last-modified-2020-01-10",
        "Old Church Slavic: Grammar and Dictionaries",
        "OSD spreadsheet",
        "exact spreadsheet notice requires review",
        "unresolved",
        &["lexical", "morphological"],
        &["Anna Polivanova"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let archive = source_path(&options.cache, locked, ".zip")?;
    let directory = work.join("polivanova-osd");
    extract_zip(&archive, &directory)?;
    let xls = recursive_files(&directory, Some("xls"))?
        .into_iter()
        .find(|path| path.file_name() == Some(OsStr::new("osd_data.xls")))
        .ok_or("Polivanova archive has no XLS")?;
    let mut workbook: Xls<_> = open_workbook(&xls)?;
    let mut order = 0;
    for sheet in workbook.sheet_names().to_vec() {
        if let Ok(range) = workbook.worksheet_range(&sheet) {
            for (row_index, row) in range.rows().enumerate() {
                let cells: Vec<String> = row.iter().map(data_to_string).collect();
                if cells.iter().all(|cell| cell.trim().is_empty()) {
                    sink.skip();
                    continue;
                }
                order += 1;
                let raw = cells.join("\t");
                sink.accept(
                    &format!("{sheet}:{}", row_index + 1),
                    order,
                    &format!("sheet:{sheet}:row:{}", row_index + 1),
                    &raw,
                    &raw,
                    "ocs-dictionary-row",
                    "untyped",
                    vec!["XLS-cell-extraction".into()],
                )?;
            }
        }
    }
    sink.finish(&output_path)
}

fn adapt_polivanova_tei(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    _work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = ocs_spec(
        "polivanova-fup-2023",
        "eISBN-XML-979-12-215-0105-6",
        "Old Church Slavic",
        "Firenze University Press TEI XML, 2023",
        "CC BY 4.0",
        "allowed",
        &["morphological", "orthographic"],
        &["Anna Polivanova", "polivanova-osd-source"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let xml = source_path(&options.cache, locked, ".xml")?;
    stream_xml_text(&xml, &mut sink, "tei-text-node")?;
    sink.finish(&output_path)
}

fn adapt_ud(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = ocs_spec(
        "ud-ocs-proiel-r2.18",
        "64eddf87abfaa51e7f5acf0bef1bebcdaca1559f",
        "UD Old Church Slavonic PROIEL",
        "release r2.18",
        "CC BY-NC-SA 4.0",
        "noncommercial-only",
        &["morphological", "evaluation"],
        &["PROIEL", "Syntacticus"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let archive = source_path(&options.cache, locked, ".tar.gz")?;
    let directory = work.join("ud");
    extract_tar(&archive, &directory)?;
    for file in recursive_files(&directory, Some("conllu"))? {
        stream_conllu(&file, &mut sink)?;
    }
    sink.finish(&output_path)
}

fn adapt_syntacticus(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = ocs_spec(
        "syntacticus-20230428",
        "525cee4fb40590d7d514376c11acaed1bdd91c15",
        "Syntacticus PROIEL/TOROT",
        "native treebank release",
        "CC BY-NC-SA 4.0",
        "noncommercial-only",
        &["morphological", "evaluation"],
        &["PROIEL", "TOROT"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let archive = source_path(&options.cache, locked, ".tar.gz")?;
    let directory = work.join("syntacticus");
    extract_tar(&archive, &directory)?;
    let files = recursive_files(&directory, Some("conll"))?;
    for file in files.into_iter().filter(|path| {
        let path = path.to_string_lossy();
        path.contains("/proiel/marianus.") || path.contains("/torot/")
    }) {
        stream_conllu(&file, &mut sink)?;
    }
    sink.finish(&output_path)
}

fn adapt_ccmh(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    _work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = ocs_spec(
        "ccmh-2021-04-23",
        "files-last-modified-2021-04-23",
        "Corpus Cyrillo-Methodianum Helsingiense",
        "locked text/XML witnesses",
        "version-specific license unresolved",
        "evaluation-only",
        &["orthographic", "evaluation"],
        &[],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    for artifact in &locked.artifacts {
        let relative = Path::new(&artifact.path)
            .strip_prefix("downloads")
            .unwrap_or_else(|_| Path::new(&artifact.path));
        let path = options.cache.join(relative);
        match path.extension().and_then(OsStr::to_str) {
            Some("xml") => stream_xml_text(&path, &mut sink, "ccmh-xml-text")?,
            Some("txt") => stream_lines(&path, &mut sink, "historical-witness-line")?,
            _ => sink.skip(),
        }
    }
    sink.finish(&output_path)
}

fn adapt_diacu(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = SourceSpec {
        id: "diacu-1.0",
        revision: "d4b00baa0b63b9ed4c60eb998670986a072294a0",
        recension: "mixed",
        target: None,
        work: "DIACU diachronic dataset",
        edition: "1.0",
        license: "no explicit top-level data license found",
        redistribution: "evaluation-only",
        authority: &["evaluation", "recension-classification"],
        epistemic: &["contamination-control"],
        lineage: &[],
    };
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let archive = source_path(&options.cache, locked, ".tar.gz")?;
    let directory = work.join("diacu");
    extract_tar(&archive, &directory)?;
    let json = recursive_files(&directory, Some("json"))?
        .into_iter()
        .find(|path| path.file_name() == Some(OsStr::new("DIACU_1.0.json")))
        .ok_or("DIACU archive lacks DIACU_1.0.json")?;
    let value: Value = serde_json::from_reader(BufReader::new(File::open(json)?))?;
    let documents = value
        .get("Documents")
        .and_then(Value::as_array)
        .ok_or("DIACU Documents is not an array")?;
    let mut order = 0;
    for (document_index, document) in documents.iter().enumerate() {
        let title = document
            .get("Title")
            .and_then(Value::as_str)
            .unwrap_or("untitled");
        let language = document
            .get("Language")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let Some(content) = document.get("Content").and_then(Value::as_str) else {
            sink.reject(
                &format!("document:{document_index}"),
                order,
                "missing-document-content",
                &document.to_string(),
            )?;
            continue;
        };
        for (line_index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                sink.skip();
                continue;
            }
            order += 1;
            sink.accept(
                &format!("document:{document_index}:line:{line_index}"),
                order,
                &format!("{title}:line:{}", line_index + 1),
                line,
                line.trim(),
                "diachronic-control-text",
                language,
                vec!["DIACU-document-segmentation".into()],
            )?;
        }
    }
    sink.finish(&output_path)
}

fn adapt_wiktionary(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    _work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = ocs_spec(
        "english-wiktionary-ocs-kaikki-2026-08-07",
        "Kaikki-last-modified-2026-08-07-from-enwiktionary-20260801",
        "English Wiktionary OCS",
        "Kaikki/Wiktextract",
        "CC BY-SA 4.0 and GFDL 1.1-or-later",
        "allowed",
        &["lexical", "morphological"],
        &["English Wiktionary", "Wiktextract d9fa233"],
    );
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let path = source_path(&options.cache, locked, ".jsonl")?;
    for (offset, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        let value: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => {
                sink.reject(
                    &format!("line:{}", offset + 1),
                    offset + 1,
                    "invalid-json",
                    &line,
                )?;
                continue;
            }
        };
        if value.get("lang_code").and_then(Value::as_str) != Some("cu") {
            sink.skip();
            continue;
        }
        let Some(lemma) = value.get("word").and_then(Value::as_str) else {
            sink.reject(
                &format!("line:{}", offset + 1),
                offset + 1,
                "missing-lemma",
                &line,
            )?;
            continue;
        };
        let pos = value
            .get("pos")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        sink.accept(
            &format!("line:{}:{lemma}:{pos}", offset + 1),
            offset + 1,
            lemma,
            &line,
            lemma,
            pos,
            "lexeme-and-forms",
            vec!["Wiktextract-JSON-selection".into()],
        )?;
    }
    sink.finish(&output_path)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct PonomarDictionaryEntry {
    id: String,
    word: String,
    transcription: String,
    definition: String,
}

fn adapt_modern_dictionary_and_frequency(
    options: &PipelineOptions,
    output: &Path,
    quarantine: &Path,
    _work: &Path,
    locked: &LockedSource,
) -> PipelineResult<SourceReport> {
    let spec = SourceSpec {
        id: "ponomar-modern-church-slavonic-corpus-2016",
        revision: "wordlist-2016-02-25;dictionary-2016-04-01",
        recension: "mixed",
        target: None,
        work: "Modern Church Slavonic corpus frequency list and dictionary",
        edition: "SCI Ponomar 2016 wordlist and dictionary",
        license: "component lineage requires per-work audit",
        redistribution: "evaluation-only",
        authority: &[
            "frequency",
            "lexical",
            "semantic",
            "orthographic",
            "accentual",
        ],
        epistemic: &["mixed-recension-candidate"],
        lineage: &["Ponomar modern Church Slavonic corpus"],
    };
    let (mut sink, output_path) =
        new_sink(output, quarantine, spec, locked, options.failure_ceiling)?;
    let path = source_path(&options.cache, locked, "wordlist.tsv")?;
    let mut lines = BufReader::new(File::open(&path)?).lines();
    if lines.next().transpose()?.as_deref() != Some("Word\tFreq") {
        return Err("unexpected Ponomar frequency-list header".into());
    }
    for (offset, line) in lines.enumerate() {
        let row_number = offset + 2;
        let line = line?;
        match parse_ponomar_frequency_row(&line) {
            Ok((word, frequency)) => sink.accept(
                &format!("wordlist.tsv:{row_number}"),
                row_number,
                &format!("wordlist.tsv:line:{row_number}"),
                &line,
                &word,
                "frequency-row",
                &format!("frequency={frequency}"),
                vec!["TSV-field-extraction".into()],
            )?,
            Err(reason) => sink.reject(
                &format!("wordlist.tsv:{row_number}"),
                row_number,
                &reason,
                &line,
            )?,
        }
    }

    let dictionary = source_path(&options.cache, locked, "dictout.xls")?;
    let mut workbook: Xls<_> = open_workbook(&dictionary)?;
    let sheet = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or("Ponomar dictionary workbook has no sheets")?;
    let range = workbook.worksheet_range(&sheet)?;
    let mut rows = range.rows();
    let header = rows
        .next()
        .ok_or("Ponomar dictionary workbook has no header")?
        .iter()
        .map(data_to_string)
        .collect::<Vec<_>>();
    if header != ["Id", "Word", "Transcription", "Definition"] {
        return Err(format!("unexpected Ponomar dictionary header {header:?}").into());
    }
    for (offset, row) in rows.enumerate() {
        let row_number = offset + 2;
        let cells = row.iter().map(data_to_string).collect::<Vec<_>>();
        match parse_ponomar_dictionary_entry(&cells) {
            Ok(entry) => {
                let raw = serde_json::to_string(&entry)?;
                sink.accept(
                    &format!("dictout.xls:{sheet}:{row_number}:{}", entry.id),
                    1_000_000 + row_number,
                    &format!("dictout.xls:{sheet}:row:{row_number};entry:{}", entry.id),
                    &raw,
                    &entry.word,
                    "dictionary-entry",
                    "headword-with-definition",
                    vec![
                        "XLS-cell-extraction".into(),
                        "structured-dictionary-entry".into(),
                    ],
                )?;
            }
            Err(reason) => sink.reject(
                &format!("dictout.xls:{sheet}:{row_number}"),
                1_000_000 + row_number,
                &reason,
                &cells.join("\t"),
            )?,
        }
    }
    sink.finish(&output_path)
}

fn parse_ponomar_frequency_row(line: &str) -> Result<(String, u64), String> {
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() != 2 {
        return Err("frequency-row-width".into());
    }
    let word = fields[0].trim();
    if !word.chars().any(char::is_alphabetic) {
        return Err("frequency-missing-word".into());
    }
    let frequency = fields[1]
        .parse::<u64>()
        .map_err(|_| "frequency-invalid-count".to_owned())?;
    Ok((word.into(), frequency))
}

fn parse_ponomar_dictionary_entry(cells: &[String]) -> Result<PonomarDictionaryEntry, String> {
    if cells.len() != 4 {
        return Err("dictionary-row-width".into());
    }
    let entry = PonomarDictionaryEntry {
        id: cells[0].trim().to_owned(),
        word: cells[1].trim().to_owned(),
        transcription: cells[2].trim().to_owned(),
        definition: cells[3].split_whitespace().collect::<Vec<_>>().join(" "),
    };
    if entry.id.parse::<u64>().is_err() {
        return Err("dictionary-invalid-id".into());
    }
    if !entry.word.chars().any(char::is_alphabetic) {
        return Err("dictionary-missing-headword".into());
    }
    if entry.definition.is_empty() {
        return Err("dictionary-missing-definition".into());
    }
    Ok(entry)
}

#[allow(clippy::too_many_arguments)]
fn synodal_spec(
    id: &'static str,
    revision: &'static str,
    work: &'static str,
    edition: &'static str,
    license: &'static str,
    redistribution: &'static str,
    authority: &'static [&'static str],
) -> SourceSpec {
    SourceSpec {
        id,
        revision,
        recension: "synodal-russian",
        target: Some("synodal-russian"),
        work,
        edition,
        license,
        redistribution,
        authority,
        epistemic: &["direct-synodal-candidate"],
        lineage: &[],
    }
}

#[allow(clippy::too_many_arguments)]
fn ocs_spec(
    id: &'static str,
    revision: &'static str,
    work: &'static str,
    edition: &'static str,
    license: &'static str,
    redistribution: &'static str,
    authority: &'static [&'static str],
    lineage: &'static [&'static str],
) -> SourceSpec {
    SourceSpec {
        id,
        revision,
        recension: "old-church-slavonic",
        target: None,
        work,
        edition,
        license,
        redistribution,
        authority,
        epistemic: &["inherited-ocs-evidence"],
        lineage,
    }
}

fn stream_conllu(path: &Path, sink: &mut Sink) -> PipelineResult<()> {
    let file = path.file_name().unwrap_or_default().to_string_lossy();
    let mut sentence = "unknown".to_owned();
    for (offset, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        if let Some(value) = line.strip_prefix("# sent_id = ") {
            sentence = value.into();
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            sink.skip();
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 6 || fields[0].contains(['-', '.']) {
            sink.reject(
                &format!("{file}:{}", offset + 1),
                offset + 1,
                "invalid-conllu-row",
                &line,
            )?;
            continue;
        }
        sink.accept(
            &format!("{file}:{sentence}:{}", fields[0]),
            offset + 1,
            &format!("{file}:{sentence}"),
            fields[1],
            fields[2],
            fields[3],
            fields[5],
            vec!["CoNLL-U-column-mapping".into()],
        )?;
    }
    Ok(())
}

fn stream_xml_text(path: &Path, sink: &mut Sink, pos: &str) -> PipelineResult<()> {
    let mut reader = XmlReader::from_reader(BufReader::new(File::open(path)?));
    reader.config_mut().trim_text(true);
    let mut buffer = Vec::new();
    let mut order = 0;
    loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Text(text) => {
                let raw = text.unescape()?.into_owned();
                if raw.trim().is_empty() {
                    sink.skip();
                } else {
                    order += 1;
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    sink.accept(
                        &format!("{name}:{order}"),
                        order,
                        &format!("{name}:text:{order}"),
                        &raw,
                        raw.trim(),
                        pos,
                        "untyped",
                        vec!["streaming-XML-text-node".into()],
                    )?;
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    Ok(())
}

fn stream_lines(path: &Path, sink: &mut Sink, pos: &str) -> PipelineResult<()> {
    for (offset, line) in BufReader::new(File::open(path)?).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            sink.skip();
        } else {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            sink.accept(
                &format!("{name}:{}", offset + 1),
                offset + 1,
                &format!("{name}:line:{}", offset + 1),
                &line,
                line.trim(),
                pos,
                "untyped",
                vec!["line-preserving-extraction".into()],
            )?;
        }
    }
    Ok(())
}

#[derive(Debug)]
struct OcrLine {
    line_key: String,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
    mean_confidence: u16,
    text: String,
}

fn djvu_page_count(source: &Path) -> PipelineResult<usize> {
    let output = Command::new("djvused")
        .arg(source)
        .args(["-e", "n"])
        .output()?;
    if !output.status.success() {
        return Err("djvused failed to count D’yachenko pages".into());
    }
    let pages = String::from_utf8(output.stdout)?.trim().parse::<usize>()?;
    if pages != 1_158 {
        return Err(format!("D’yachenko scan has {pages} pages; expected 1158").into());
    }
    Ok(pages)
}

fn run_pinned_ocr(
    source: &Path,
    tessdata: &Path,
    output: &Path,
    pages: usize,
) -> PipelineResult<()> {
    remove_if_exists(output)?;
    fs::create_dir_all(output)?;
    let source = Arc::new(source.to_owned());
    let tessdata = Arc::new(tessdata.to_owned());
    let output = Arc::new(output.to_owned());
    let next_page = Arc::new(AtomicUsize::new(1));
    let errors = Arc::new(Mutex::new(Vec::<String>::new()));
    let workers = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1)
        .min(8);
    let mut handles = Vec::new();
    for _ in 0..workers {
        let source = Arc::clone(&source);
        let tessdata = Arc::clone(&tessdata);
        let output = Arc::clone(&output);
        let next_page = Arc::clone(&next_page);
        let errors = Arc::clone(&errors);
        handles.push(thread::spawn(move || {
            loop {
                let page = next_page.fetch_add(1, Ordering::Relaxed);
                if page > pages {
                    break;
                }
                if let Err(error) = ocr_page(&source, &tessdata, &output, page) {
                    if let Ok(mut errors) = errors.lock() {
                        errors.push(format!("page {page}: {error}"));
                    }
                    break;
                }
            }
        }));
    }
    for handle in handles {
        handle
            .join()
            .map_err(|_| "D’yachenko OCR worker panicked")?;
    }
    let errors = errors.lock().map_err(|_| "OCR error mutex poisoned")?;
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("D’yachenko OCR failed: {}", errors.join("; ")).into())
    }
}

fn ocr_page(source: &Path, tessdata: &Path, output: &Path, page: usize) -> PipelineResult<()> {
    let image = output.join(format!("page-{page}.pbm"));
    let tsv = output.join(format!("page-{page}.tsv"));
    let render = Command::new("ddjvu")
        .arg("-format=pnm")
        .arg(format!("-page={page}"))
        .arg(source)
        .arg(&image)
        .status()?;
    if !render.success() {
        return Err("ddjvu rendering failed".into());
    }
    let recognition = Command::new("tesseract")
        .arg("--tessdata-dir")
        .arg(tessdata)
        .args(["-l", "rus", "-c", "tessedit_create_tsv=1"])
        .arg(&image)
        .arg("stdout")
        .stdout(File::create(&tsv)?)
        .stderr(Stdio::null())
        .status()?;
    fs::remove_file(image)?;
    if !recognition.success() {
        return Err("Tesseract recognition failed".into());
    }
    Ok(())
}

fn read_tesseract_lines(path: &Path) -> PipelineResult<Vec<OcrLine>> {
    #[derive(Default)]
    struct PendingLine {
        left: u32,
        top: u32,
        right: u32,
        bottom: u32,
        confidence_sum: f32,
        confidence_count: usize,
        words: Vec<String>,
    }
    let mut lines = BTreeMap::<String, PendingLine>::new();
    for (offset, row) in BufReader::new(File::open(path)?).lines().enumerate() {
        let row = row?;
        if offset == 0 {
            continue;
        }
        let fields: Vec<&str> = row.split('\t').collect();
        if fields.len() != 12 || fields[0] != "5" || fields[11].trim().is_empty() {
            continue;
        }
        let key = [fields[2], fields[3], fields[4]].join(":");
        let left = fields[6].parse::<u32>()?;
        let top = fields[7].parse::<u32>()?;
        let width = fields[8].parse::<u32>()?;
        let height = fields[9].parse::<u32>()?;
        let confidence = fields[10].parse::<f32>()?.max(0.0);
        let line = lines.entry(key).or_default();
        if line.words.is_empty() {
            line.left = left;
            line.top = top;
        } else {
            line.left = line.left.min(left);
            line.top = line.top.min(top);
        }
        line.right = line.right.max(left + width);
        line.bottom = line.bottom.max(top + height);
        line.confidence_sum += confidence;
        line.confidence_count += 1;
        line.words.push(fields[11].into());
    }
    Ok(lines
        .into_iter()
        .map(|(line_key, line)| OcrLine {
            line_key,
            left: line.left,
            top: line.top,
            width: line.right - line.left,
            height: line.bottom - line.top,
            mean_confidence: (line.confidence_sum / line.confidence_count as f32).round() as u16,
            text: line.words.join(" "),
        })
        .collect())
}

fn extract_tar(archive: &Path, destination: &Path) -> PipelineResult<()> {
    remove_if_exists(destination)?;
    fs::create_dir_all(destination)?;
    let status = Command::new("tar")
        .args(["-xzf"])
        .arg(archive)
        .arg("-C")
        .arg(destination)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("tar extraction failed for {}", archive.display()).into())
    }
}

fn extract_zip(archive: &Path, destination: &Path) -> PipelineResult<()> {
    remove_if_exists(destination)?;
    fs::create_dir_all(destination)?;
    let status = Command::new("unzip")
        .arg("-qq")
        .arg(archive)
        .arg("-d")
        .arg(destination)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ZIP extraction failed for {}", archive.display()).into())
    }
}

fn recursive_files(root: &Path, extension: Option<&str>) -> PipelineResult<Vec<PathBuf>> {
    let mut pending = vec![root.to_owned()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(fs::DirEntry::file_name);
        for entry in entries.into_iter().rev() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if extension
                .is_none_or(|value| path.extension().and_then(OsStr::to_str) == Some(value))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn strip_ponomar_markup(value: &str) -> String {
    let mut output = String::new();
    let mut brace_depth = 0;
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '{' => brace_depth += 1,
            '}' if brace_depth > 0 => brace_depth -= 1,
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
            }
            _ if brace_depth == 0 => output.push(ch),
            _ => {}
        }
    }
    output.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn strip_wikitext(value: &str) -> String {
    let without_refs = value.split("<ref").next().unwrap_or(value);
    strip_html(&strip_templates(&expand_bukvitsa(without_refs)))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn expand_bukvitsa(value: &str) -> String {
    let mut output = String::new();
    let mut remaining = value;
    while let Some((before, tail)) = remaining.split_once("{{Буквица|") {
        output.push_str(before);
        let Some((arguments, after)) = tail.split_once("}}") else {
            output.push_str("{{Буквица|");
            output.push_str(tail);
            return output;
        };
        if let Some(letter) = arguments.rsplit('|').next() {
            output.push_str(letter.trim());
        }
        remaining = after;
    }
    output.push_str(remaining);
    output
}

fn is_wikisource_paratext(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty()
        || trimmed.starts_with("[[")
        || trimmed.starts_with("====")
        || trimmed.starts_with("----")
        || trimmed.starts_with("<br")
        || trimmed.starts_with("<div")
        || trimmed.starts_with("</div")
        || trimmed.starts_with("__")
        || trimmed.starts_with("{{Навигация")
}

fn wikisource_verse_number(line: &str) -> Option<u32> {
    let (_, colors) = line.split_once("{{Colors|")?;
    let (_, value) = colors.split_once("||")?;
    let number = value.split_once("}}").map_or(value, |(number, _)| number);
    synodal_church_slavonic_core::parse_cyrillic_numeral(number.trim()).ok()
}

fn load_wikisource_book_codes(root: &Path) -> PipelineResult<BTreeMap<String, String>> {
    let books_path = root.join("references/WIKISOURCE_BOOKS.tsv");
    let books = fs::read_to_string(&books_path)?;
    let mut lines = books.lines();
    if lines.next() != Some("title\tbook_code") {
        return Err(format!("invalid {} header", books_path.display()).into());
    }
    let mut by_title = BTreeMap::new();
    for (offset, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 2 || fields[0].is_empty() || fields[1].is_empty() {
            return Err(format!("invalid {} row {}", books_path.display(), offset + 2).into());
        }
        if by_title.insert(fields[0], fields[1]).is_some() {
            return Err(format!("duplicate Wikisource title {:?}", fields[0]).into());
        }
    }

    let revisions_path = root.join("references/WIKISOURCE_REVISIONS.tsv");
    let revisions = fs::read_to_string(&revisions_path)?;
    let mut lines = revisions.lines();
    if lines.next() != Some("title\tpage_id\trevision_id\ttimestamp\tmediawiki_sha1") {
        return Err(format!("invalid {} header", revisions_path.display()).into());
    }
    let mut by_revision = BTreeMap::new();
    for (offset, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!("invalid {} row {}", revisions_path.display(), offset + 2).into());
        }
        let book = by_title.get(fields[0]).ok_or_else(|| {
            format!(
                "Wikisource title {:?} has no reviewed book-code mapping",
                fields[0]
            )
        })?;
        if by_revision
            .insert(fields[2].to_owned(), (*book).to_owned())
            .is_some()
        {
            return Err(format!("duplicate Wikisource revision {}", fields[2]).into());
        }
    }
    if by_revision.len() != by_title.len() {
        return Err("Wikisource revision and reviewed book-code inventories differ".into());
    }
    Ok(by_revision)
}

fn strip_templates(value: &str) -> String {
    let mut output = String::new();
    let mut depth = 0_u32;
    let chars: Vec<char> = value.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if chars.get(index) == Some(&'{') && chars.get(index + 1) == Some(&'{') {
            depth += 1;
            index += 2;
            continue;
        }
        if chars.get(index) == Some(&'}') && chars.get(index + 1) == Some(&'}') && depth > 0 {
            depth -= 1;
            index += 2;
            continue;
        }
        if depth == 0 {
            output.push(chars[index]);
        }
        index += 1;
    }
    output
}

fn strip_osis(value: &str) -> String {
    strip_html(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn canonical_crosswire_passage(reference: &str) -> Option<String> {
    let (book, location) = reference.rsplit_once(' ')?;
    let (chapter, verse) = location.split_once(':')?;
    let chapter = chapter.parse::<u32>().ok()?;
    let verse = verse.parse::<u32>().ok()?;
    if chapter == 0 || verse == 0 {
        return None;
    }
    let book = match book {
        "Genesis" => "Gen",
        "Exodus" => "Ex",
        "Leviticus" => "Lev",
        "Numbers" => "Num",
        "Deuteronomy" => "Deut",
        "Joshua" => "Josh",
        "Judges" => "Judg",
        "Ruth" => "Ruth",
        "I Samuel" => "I_Kings",
        "II Samuel" => "II_Kings",
        "I Kings" => "III_Kings",
        "II Kings" => "IV_Kings",
        "I Chronicles" => "I_Paral",
        "II Chronicles" => "II_Paral",
        "I Esdras" => "I_Esdra",
        "Ezra" => "II_Esdra",
        "Nehemiah" => "Nehem",
        "Tobit" => "Tobit",
        "Judith" => "Judith",
        "Esther" => "Esther",
        "I Maccabees" => "I_Macc",
        "II Maccabees" => "II_Macc",
        "Job" => "Job",
        "Psalms" => "Psalm",
        "Proverbs" => "Prov",
        "Ecclesiastes" => "Eccles",
        "Song of Solomon" => "Song",
        "Wisdom" => "Wisd",
        "Sirach" => "Sirach",
        "Isaiah" => "Isa",
        "Jeremiah" => "Jerem",
        "Lamentations" => "Lamen",
        "Epistle of Jeremiah" => "Epistle",
        "Baruch" => "Baruch",
        "Ezekiel" => "Ezek",
        "Daniel" => "Dan",
        "Hosea" => "Hos",
        "Joel" => "Joel",
        "Amos" => "Amos",
        "Obadiah" => "Obad",
        "Jonah" => "Jona",
        "Micah" => "Mica",
        "Nahum" => "Nahum",
        "Habakkuk" => "Habak",
        "Zephaniah" => "Zeph",
        "Haggai" => "Hagg",
        "Zechariah" => "Zech",
        "Malachi" => "Mal",
        "Prayer of Manasses" => "Composite",
        "Matthew" => "Mt",
        "Mark" => "Mk",
        "Luke" => "Lk",
        "John" => "Jn",
        "Acts" => "Acts",
        "Romans" => "Rom",
        "I Corinthians" => "I_Cor",
        "II Corinthians" => "II_Cor",
        "Galatians" => "Gal",
        "Ephesians" => "Eph",
        "Philippians" => "Philip",
        "Colossians" => "Col",
        "I Thessalonians" => "I_Thess",
        "II Thessalonians" => "II_Thess",
        "I Timothy" => "I_Tim",
        "II Timothy" => "II_Tim",
        "Titus" => "Tit",
        "Philemon" => "Philemon",
        "Hebrews" => "Heb",
        "James" => "Jas",
        "I Peter" => "I_Pet",
        "II Peter" => "II_Pet",
        "I John" => "I_Jn",
        "II John" => "II_Jn",
        "III John" => "III_Jn",
        "Jude" => "Jude",
        "Revelation of John" => "Apoc",
        _ => return None,
    };
    Some(format!("{book}.{chapter}.{verse}"))
}

fn strip_html(value: &str) -> String {
    let mut output = String::new();
    let mut tag = false;
    for ch in value.chars() {
        match ch {
            '<' => tag = true,
            '>' => tag = false,
            _ if !tag => output.push(ch),
            _ => {}
        }
    }
    decode_html(output.trim())
}

fn decode_html(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn first_between(value: &str, start: &str, end: &str) -> Option<String> {
    let (_, tail) = value.split_once(start)?;
    let (body, _) = tail.split_once(end)?;
    Some(body.into())
}

fn class_spans(html: &str, class: &str) -> Vec<String> {
    let mut values = Vec::new();
    let marker = format!("class=\"{class}\"");
    let mut tail = html;
    while let Some((_, after)) = tail.split_once(&marker) {
        let Some((_, body)) = after.split_once('>') else {
            break;
        };
        let Some((value, rest)) = body.split_once("</span>") else {
            break;
        };
        values.push(value.into());
        tail = rest;
    }
    values
}

fn template_argument(line: &str, marker: &str) -> Option<String> {
    let (_, tail) = line.split_once(marker)?;
    let end = tail.find(['|', '}']).unwrap_or(tail.len());
    Some(tail[..end].trim().into())
}

fn word_tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| {
            !(ch.is_alphabetic() || matches!(ch, '\u{0300}'..='\u{036f}' | '\u{0483}'..='\u{0489}'))
        })
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn data_to_string(value: &Data) -> String {
    match value {
        Data::Empty => String::new(),
        _ => value.to_string(),
    }
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn stable_id(namespace: &str, values: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for value in values {
        hasher.update(value.as_bytes());
        hasher.update([0]);
    }
    let digest = hasher.finalize();
    let hash: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("synodal:{namespace}:{}", &hash[..24])
}

fn passage_partition(source: &str, passage: &str) -> String {
    let digest = Sha256::digest(format!("{source}\0{passage}").as_bytes());
    if digest[0] % 5 == 0 {
        "evaluation".into()
    } else {
        "source".into()
    }
}

fn sha256_file(path: &Path) -> PipelineResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn require_tool(tool: &str, remediation: &str) -> PipelineResult<()> {
    match Command::new(tool)
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(_) => Ok(()),
        Err(_) => {
            Err(format!("required adapter tool {tool:?} is unavailable; {remediation}").into())
        }
    }
}

fn remove_if_exists(path: &Path) -> PipelineResult<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn atomic_replace_directory(source: &Path, destination: &Path) -> PipelineResult<()> {
    let backup = destination.with_extension(format!("backup-{}", std::process::id()));
    remove_if_exists(&backup)?;
    if destination.exists() {
        fs::rename(destination, &backup)?;
    }
    match fs::rename(source, destination) {
        Ok(()) => {
            remove_if_exists(&backup)?;
            Ok(())
        }
        Err(error) => {
            if backup.exists() {
                fs::rename(backup, destination)?;
            }
            Err(error.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("synodal-pipeline-{label}-{}", std::process::id()))
    }

    #[test]
    fn markup_is_removed_without_losing_church_slavonic_text() {
        assert_eq!(
            strip_ponomar_markup(" **Кни́га** {note} родства̀ "),
            "Кни́га родства̀"
        );
        assert_eq!(word_tokens("Кни́га родства̀"), vec!["кни́га", "родства̀"]);
        assert_eq!(strip_wikitext("{{Буквица|font-size=5em|С}}ло́во"), "Сло́во");
        assert_eq!(
            wikisource_verse_number("{{Smaller|{{Colors|#B22222||а҃і}}}} text"),
            Some(11)
        );
        assert_eq!(
            canonical_crosswire_passage("Genesis 1:1").as_deref(),
            Some("Gen.1.1")
        );
        assert_eq!(canonical_crosswire_passage("Genesis 0:0"), None);
    }

    #[test]
    fn every_locked_wikisource_revision_has_a_reviewed_book_code() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let codes = load_wikisource_book_codes(&root).expect("book codes");
        assert_eq!(codes.len(), 78);
        assert_eq!(
            codes.values().filter(|code| code.as_str() == "-").count(),
            1
        );
    }

    #[test]
    fn partitions_and_ids_are_stable() {
        assert_eq!(stable_id("x", &["a", "b"]), stable_id("x", &["a", "b"]));
        assert_eq!(
            passage_partition("source", "passage"),
            passage_partition("source", "passage")
        );
    }

    #[test]
    fn ponomar_dictionary_rows_preserve_identity_marks_and_meaning() {
        let entry = parse_ponomar_dictionary_entry(&[
            "18040".into(),
            "А҆ба́че".into(),
            "абаче".into(),
            "впрочем,   однако, но".into(),
        ])
        .expect("dictionary row");
        assert_eq!(entry.id, "18040");
        assert_eq!(entry.word, "А҆ба́че");
        assert_eq!(entry.transcription, "абаче");
        assert_eq!(entry.definition, "впрочем, однако, но");
        assert_eq!(
            serde_json::from_str::<PonomarDictionaryEntry>(
                &serde_json::to_string(&entry).expect("serialize dictionary row")
            )
            .expect("deserialize dictionary row"),
            entry
        );

        assert_eq!(
            parse_ponomar_dictionary_entry(&[
                "not-an-id".into(),
                "слово".into(),
                "слово".into(),
                "meaning".into(),
            ]),
            Err("dictionary-invalid-id".into())
        );

        assert_eq!(
            parse_ponomar_frequency_row("є҆сѝ\t50272"),
            Ok(("є҆сѝ".into(), 50_272))
        );
        assert_eq!(
            parse_ponomar_frequency_row("Word\tFreq"),
            Err("frequency-invalid-count".into())
        );
    }

    #[test]
    fn alypy_witness_extraction_preserves_source_order() {
        let html =
            r#"<h3>§34</h3><span class="DSText">ра́б-ъ</span><span class="DSText">сел-о̀</span>"#;
        assert_eq!(class_spans(html, "DSText"), vec!["ра́б-ъ", "сел-о̀"]);
    }

    #[test]
    fn wikitext_stripping_does_not_promote_template_content() {
        assert_eq!(
            strip_wikitext("{{Marker|а҃}} Сло́во <ref>note</ref>"),
            "Сло́во"
        );
    }

    #[test]
    fn tesseract_tsv_fixture_preserves_lines_boxes_confidence_and_order() {
        let fixture = fixture_directory("tesseract-tsv");
        remove_if_exists(&fixture).expect("clean fixture");
        fs::create_dir_all(&fixture).expect("fixture directory");
        let path = fixture.join("page-1.tsv");
        fs::write(
            &path,
            "level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext\n\
             5\t1\t1\t1\t2\t1\t40\t30\t12\t8\t80.0\tсло́во\n\
             5\t1\t1\t1\t1\t1\t10\t20\t8\t9\t90.0\tа҆́зъ\n\
             5\t1\t1\t1\t1\t2\t20\t19\t15\t11\t70.0\tє҆́смь\n",
        )
        .expect("TSV fixture");
        let lines = read_tesseract_lines(&path).expect("parsed TSV");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].line_key, "1:1:1");
        assert_eq!(lines[0].text, "а҆́зъ є҆́смь");
        assert_eq!(
            (lines[0].left, lines[0].top, lines[0].width, lines[0].height),
            (10, 19, 25, 11)
        );
        assert_eq!(lines[0].mean_confidence, 80);
        assert_eq!(lines[1].line_key, "1:1:2");
        assert_eq!(lines[1].text, "сло́во");
        remove_if_exists(&fixture).expect("fixture cleanup");
    }

    #[test]
    fn alypy_fixture_adapter_is_byte_deterministic() {
        let fixture = fixture_directory("alypy");
        remove_if_exists(&fixture).expect("clean fixture");
        let cache = fixture.join("cache");
        let page = cache.join("alypy-grammar/p034.htm");
        fs::create_dir_all(page.parent().expect("fixture parent")).expect("fixture parent");
        let html = b"<html><h3>\xc2\xa734</h3><span class=\"DSText\">\xd1\x80\xd0\xb0\xcc\x81\xd0\xb1-\xd1\x8a</span></html>";
        fs::write(&page, html).expect("fixture page");
        let sha = Sha256::digest(html)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        fs::create_dir_all(fixture.join("references")).expect("fixture references");
        fs::write(
            fixture.join("references/SOURCE_LOCK.tsv"),
            format!(
                "source_id\tartifact_id\ttransport\turl\tpath\tsha256\tsize_bytes\tformat\tsignature\tcontent_types\n\
                 alypy-gamanovich-grammar-web-2023\tfixture-page\tdirect\thttps://example.invalid/p034.htm\tdownloads/alypy-grammar/p034.htm\t{sha}\t{}\thtml\thtml\ttext/html\n",
                html.len()
            ),
        )
        .expect("fixture lock");

        let run = |suffix: &str| {
            let intermediate = fixture.join(format!("intermediate-{suffix}"));
            let quarantine = fixture.join(format!("quarantine-{suffix}"));
            let report = run_pipeline(&PipelineOptions {
                workspace_root: fixture.clone(),
                cache: cache.clone(),
                intermediate: intermediate.clone(),
                quarantine,
                source: Some("alypy-gamanovich-grammar-web-2023".into()),
                failure_ceiling: 0,
                keep_work: false,
            })
            .expect("fixture adapter");
            let bytes = fs::read(intermediate.join("alypy-gamanovich-grammar-web-2023.jsonl"))
                .expect("fixture candidates");
            (report, bytes)
        };
        let first = run("one");
        let second = run("two");
        assert_eq!(first, second);
        assert_eq!(
            first
                .0
                .source_reports
                .get("alypy-gamanovich-grammar-web-2023")
                .expect("source report")
                .output_sha256,
            "03227728dbfe472f42db76e42bdefad6f6f7ddd89e6d6650ede2470383c02cb7"
        );
        remove_if_exists(&fixture).expect("fixture cleanup");
    }
}
