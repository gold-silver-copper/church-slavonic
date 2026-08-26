//! Extracts the Alypy (Gamanovich) grammar's printed paradigm tables into the
//! committed gold paradigm oracle (`data/synodal/gold_paradigm_oracle.tsv`)
//! plus a human-auditable review report
//! (`reports/alypy-extraction-review.md`).
//!
//! Source: the pinned `alypy-gamanovich-grammar-web-2023` HTML artifacts under
//! `references/downloads/alypy-grammar/` (the same artifacts the §104
//! irregular-verb inventory was curated from). The pre-extracted JSONL
//! intermediate flattens tables into individual `DSText` spans and loses the
//! cell structure, so this extractor parses the HTML tables directly.
//!
//! Extraction is a curated act (docs/SYNODAL_GOLD_ORACLE_PROMPT.md, phase 1):
//! every `<table>` in every artifact is classified, and each `Decline` table
//! carries an explicit per-table disposition — extract with a POS and optional
//! feature defaults, or exclude with a written reason. A `Decline` table with
//! no disposition is an error, so nothing can be skipped silently. Printed
//! surfaces are recorded verbatim (accents, hyphenated morpheme boundaries,
//! pre-reform letter choices intact); the normative comparison contract
//! decides equivalence, not this extractor.

use std::{collections::BTreeMap, error::Error, fmt::Write as _, fs, path::Path};

use crate::report_io::write_if_changed_atomic;

pub(crate) const ORACLE_PATH: &str = "data/synodal/gold_paradigm_oracle.tsv";
pub(crate) const REVIEW_PATH: &str = "reports/alypy-extraction-review.md";
const SOURCE_DIR: &str = "references/downloads/alypy-grammar";
const SOURCE_ID: &str = "alypy-gamanovich-grammar-web-2023";

const ORACLE_COLUMNS: &str = "section\tartifact\ttable_index\tpos\theadword\tcolumn_label\tcase\tnumber\tgender\tperson\ttense\tform\tsurface";

/// Curated disposition for one `Decline`-classed table, keyed by
/// (artifact file name, zero-based index among that file's `Decline` tables).
struct Disposition {
    artifact: &'static str,
    index: usize,
    action: Action,
}

enum Action {
    Extract {
        pos: &'static str,
        /// Number assumed when the table prints no number banner and the row
        /// label carries none (e.g. the §44 dual-only fragments).
        default_number: Option<&'static str>,
        /// Tense assumed when no column header carries one.
        default_tense: Option<&'static str>,
        /// Form applied to every cell (e.g. `imperative`, `l-participle`).
        default_form: Option<&'static str>,
    },
    Exclude {
        reason: &'static str,
    },
}

const fn extract(artifact: &'static str, index: usize, pos: &'static str) -> Disposition {
    Disposition {
        artifact,
        index,
        action: Action::Extract {
            pos,
            default_number: None,
            default_tense: None,
            default_form: None,
        },
    }
}

const fn extract_with(
    artifact: &'static str,
    index: usize,
    pos: &'static str,
    default_number: Option<&'static str>,
    default_tense: Option<&'static str>,
    default_form: Option<&'static str>,
) -> Disposition {
    Disposition {
        artifact,
        index,
        action: Action::Extract {
            pos,
            default_number,
            default_tense,
            default_form,
        },
    }
}

const fn exclude(artifact: &'static str, index: usize, reason: &'static str) -> Disposition {
    Disposition {
        artifact,
        index,
        action: Action::Exclude { reason },
    }
}

/// One entry per `Decline` table in the source. Reviewed once; the committed
/// oracle is then frozen behind the checksum machinery.
const DISPOSITIONS: &[Disposition] = &[
    // §34 first declension exemplars.
    extract("p034.htm", 0, "noun"),
    extract("p034.htm", 1, "noun"),
    // §37 collective бра́тїѧ: the print states no number dimension; number is
    // left blank rather than inferred.
    extract("p037.htm", 0, "noun"),
    extract("p039.htm", 0, "noun"),
    extract("p041.htm", 0, "noun"),
    extract("p043.htm", 0, "noun"),
    extract("p043.htm", 1, "noun"),
    extract("p043.htm", 2, "noun"),
    // §44 dual-only irregular fragments (ѻ҆́чи, ᲂу҆́ши): the surrounding prose
    // states the dual; the table itself prints no number banner.
    extract_with("p044.htm", 0, "noun", Some("dual"), None, None),
    extract("p047.htm", 0, "pronoun"),
    extract("p047.htm", 1, "pronoun"),
    extract("p047.htm", 2, "pronoun"),
    // §48 кто̀/что̀ decline without a number dimension in print.
    extract("p048.htm", 0, "pronoun"),
    extract("p048.htm", 1, "pronoun"),
    extract("p048.htm", 2, "pronoun"),
    extract("p048.htm", 3, "pronoun"),
    extract("p048.htm", 4, "pronoun"),
    extract("p053.htm", 0, "adjective"),
    extract("p053.htm", 1, "adjective"),
    extract("p056.htm", 0, "adjective"),
    extract("p057.htm", 0, "adjective"),
    extract("p057.htm", 1, "adjective"),
    extract("p057.htm", 2, "adjective"),
    extract("p057.htm", 3, "adjective"),
    exclude(
        "p058.htm",
        0,
        "comparative-degree formation table (stems and suffixes, not a paradigm)",
    ),
    extract("p060.htm", 0, "adjective"),
    extract("p062.htm", 0, "numeral"),
    extract("p062.htm", 1, "numeral"),
    extract("p062.htm", 2, "numeral"),
    extract("p062.htm", 3, "numeral"),
    extract("p062.htm", 4, "numeral"),
    exclude(
        "p064.htm",
        0,
        "compound-numeral inflection: per-part dual/singular dimensions have no kernel feature vocabulary",
    ),
    exclude(
        "p064.htm",
        1,
        "compound-numeral inflection: per-part dual/singular dimensions have no kernel feature vocabulary",
    ),
    exclude(
        "p068app.htm",
        0,
        "cardinal/ordinal numeral inventory list, not an inflection paradigm",
    ),
    exclude(
        "p069.htm",
        0,
        "collective-numeral formation list, not an inflection paradigm",
    ),
    exclude(
        "p074-077.htm",
        0,
        "verbal-aspect pair illustration, not an inflection paradigm",
    ),
    exclude(
        "p080.htm",
        0,
        "conjugation-endings schema (hyphen-initial endings, no full surfaces)",
    ),
    // §81 overview of бы́ти-based tense forms.
    extract("p081.htm", 0, "verb"),
    extract("p081.htm", 1, "verb"),
    exclude(
        "p081.htm",
        2,
        "perfect/pluperfect are periphrastic tenses outside the kernel FiniteTense vocabulary",
    ),
    exclude(
        "p081.htm",
        3,
        "optative/subjunctive moods are outside the kernel form vocabulary (imperative column is re-attested by the §93 tables)",
    ),
    exclude(
        "p081.htm",
        4,
        "participle formation summary (stems), not a cell-structured paradigm",
    ),
    extract_with("p082.htm", 0, "verb", None, Some("present"), None),
    exclude(
        "p084-085.htm",
        0,
        "periphrastic future auxiliary construction (и҆́мамъ + infinitive), not a single-lexeme paradigm",
    ),
    exclude(
        "p086.htm",
        0,
        "aorist-endings schema (hyphen-initial endings, no full surfaces)",
    ),
    extract_with("p086.htm", 1, "verb", None, Some("aorist"), None),
    exclude(
        "p087.htm",
        0,
        "imperfect-endings schema (hyphen-initial endings, no full surfaces)",
    ),
    extract_with("p087.htm", 1, "verb", None, Some("imperfect"), None),
    extract_with("p087.htm", 2, "verb", None, Some("imperfect"), None),
    exclude(
        "p088.htm",
        0,
        "perfect is a periphrastic tense outside the kernel FiniteTense vocabulary",
    ),
    exclude(
        "p089.htm",
        0,
        "pluperfect is a periphrastic tense outside the kernel FiniteTense vocabulary",
    ),
    exclude(
        "p091.htm",
        0,
        "subjunctive mood is outside the kernel form vocabulary",
    ),
    exclude(
        "p093.htm",
        0,
        "imperative-endings schema (hyphen-initial endings, no full surfaces)",
    ),
    extract_with("p093.htm", 1, "verb", None, None, Some("imperative")),
    extract_with("p093.htm", 2, "verb", None, None, Some("imperative")),
    exclude(
        "p095.htm",
        0,
        "present-active-participle formation summary: printed cells are nominative exemplars whose case/number is implicit, not printed",
    ),
    exclude(
        "p095.htm",
        1,
        "participle stem-formation illustration, not a paradigm",
    ),
    exclude(
        "p096.htm",
        0,
        "past-active-participle formation summary: case/number implicit, not printed",
    ),
    exclude(
        "p096.htm",
        1,
        "participle stem-formation illustration, not a paradigm",
    ),
    exclude(
        "p096.htm",
        2,
        "passive-participle formation summary: case/number implicit, not printed",
    ),
    exclude(
        "p096.htm",
        3,
        "passive-participle formation summary: case/number implicit, not printed",
    ),
    extract_with("p097.htm", 0, "verb", None, None, Some("l-participle")),
    extract("p098.htm", 0, "participle"),
    extract("p098.htm", 1, "participle"),
    exclude(
        "p102.htm",
        0,
        "periphrastic passive tense/mood summary (auxiliary constructions), not a cell-structured paradigm",
    ),
    extract("p103.htm", 0, "verb"),
    extract_with("p103.htm", 1, "verb", None, None, Some("imperative")),
    exclude(
        "p103.htm",
        2,
        "archaic-verb participle inventory list, not a cell-structured paradigm",
    ),
];

// ---------------------------------------------------------------------------
// HTML mechanics
// ---------------------------------------------------------------------------

/// Decodes the entity repertoire the source actually uses and strips markup.
fn clean_text(html: &str) -> String {
    let mut no_tags = String::new();
    let mut chars = html.char_indices().peekable();
    let bytes = html.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] == b'<' {
            let rest = &html[position..];
            let close = rest.find('>').map_or(rest.len(), |offset| offset + 1);
            let tag = rest[..close].to_ascii_lowercase();
            if tag.starts_with("<br") {
                no_tags.push(' ');
            }
            position += close;
        } else {
            let rest = &html[position..];
            let next = rest.find('<').unwrap_or(rest.len());
            no_tags.push_str(&rest[..next]);
            position += next;
        }
    }
    let _ = &mut chars;
    decode_entities(&no_tags)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let tail = &rest[start..];
        let end = tail
            .char_indices()
            .take(12)
            .find(|(_, ch)| *ch == ';')
            .map(|(offset, _)| offset);
        if let Some(end) = end {
            let entity = &tail[1..end];
            let decoded = match entity {
                "nbsp" => Some(' '),
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "ndash" => Some('\u{2013}'),
                "mdash" => Some('\u{2014}'),
                _ => entity.strip_prefix('#').and_then(|digits| {
                    let value = if let Some(hex) = digits.strip_prefix(['x', 'X']) {
                        u32::from_str_radix(hex, 16).ok()
                    } else {
                        digits.parse::<u32>().ok()
                    };
                    value.and_then(char::from_u32)
                }),
            };
            if let Some(ch) = decoded {
                out.push(ch);
                rest = &tail[end + 1..];
                continue;
            }
        }
        out.push('&');
        rest = &tail[1..];
    }
    out.push_str(rest);
    out
}

struct RawTable {
    attrs: String,
    body: String,
}

/// Finds every `<table ...>...</table>` element in document order.
fn find_tables(html: &str) -> Vec<RawTable> {
    let lower = html.to_ascii_lowercase();
    let mut tables = Vec::new();
    let mut search = 0;
    while let Some(offset) = lower[search..].find("<table") {
        let start = search + offset;
        let Some(open_end) = lower[start..].find('>') else {
            break;
        };
        let attrs = html[start + 6..start + open_end].trim().to_string();
        let body_start = start + open_end + 1;
        let Some(close) = lower[body_start..].find("</table") else {
            break;
        };
        tables.push(RawTable {
            attrs,
            body: html[body_start..body_start + close].to_string(),
        });
        search = body_start + close + 8;
    }
    tables
}

#[derive(Clone)]
struct SourceCell {
    text: String,
    header: bool,
    class: String,
    colspan: usize,
    rowspan: usize,
}

/// Splits a `<tr>` body into cells, tolerating the source's stray `</td>`
/// fragments: a cell runs from its opening tag to the next cell opening or the
/// row's end.
fn parse_row(row: &str) -> Vec<SourceCell> {
    let lower = row.to_ascii_lowercase();
    let mut openings = Vec::new();
    for tag in ["<td", "<th"] {
        let mut search = 0;
        while let Some(offset) = lower[search..].find(tag) {
            let at = search + offset;
            let after = lower.as_bytes().get(at + 3).copied().unwrap_or(b' ');
            if after == b' ' || after == b'>' || after == b'\n' || after == b'\r' || after == b'\t'
            {
                openings.push((at, tag == "<th"));
            }
            search = at + 3;
        }
    }
    openings.sort_by_key(|(at, _)| *at);
    let mut cells = Vec::new();
    for (index, &(start, header)) in openings.iter().enumerate() {
        let end = openings.get(index + 1).map_or(row.len(), |&(next, _)| next);
        let fragment = &row[start..end];
        let Some(open_close) = fragment.find('>') else {
            continue;
        };
        let attrs = &fragment[..open_close];
        let mut content = &fragment[open_close + 1..];
        for closer in ["</td>", "</th>"] {
            if let Some(at) = content.to_ascii_lowercase().find(closer) {
                content = &content[..at];
            }
        }
        // Footnote markers are apparatus, not part of the printed surface.
        let mut owned = content.to_string();
        while let Some(at) = owned.find("<span class=\"FootRef\"") {
            if let Some(end) = owned[at..].find("</span>") {
                owned.replace_range(at..at + end + 7, "");
            } else {
                break;
            }
        }
        cells.push(SourceCell {
            text: clean_text(&owned),
            header,
            class: attr_value(attrs, "class").unwrap_or_default(),
            colspan: attr_value(attrs, "colspan")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1),
            rowspan: attr_value(attrs, "rowspan")
                .and_then(|value| value.parse().ok())
                .unwrap_or(1),
        });
    }
    cells
}

fn attr_value(attrs: &str, name: &str) -> Option<String> {
    let lower = attrs.to_ascii_lowercase();
    let at = lower.find(name)?;
    let rest = &attrs[at + name.len()..];
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    if let Some(stripped) = rest.strip_prefix('"') {
        stripped.split('"').next().map(str::to_string)
    } else {
        rest.split([' ', '>', '\t']).next().map(str::to_string)
    }
}

fn parse_rows(body: &str) -> Vec<Vec<SourceCell>> {
    let lower = body.to_ascii_lowercase();
    let mut rows = Vec::new();
    let mut search = 0;
    while let Some(offset) = lower[search..].find("<tr") {
        let start = search + offset;
        let Some(open_end) = lower[start..].find('>') else {
            break;
        };
        let body_start = start + open_end + 1;
        let close = lower[body_start..]
            .find("</tr")
            .map_or(body.len(), |at| body_start + at);
        rows.push(parse_row(&body[body_start..close]));
        search = close.max(body_start) + 1;
    }
    rows
}

// ---------------------------------------------------------------------------
// Feature vocabulary mapping (kernel `church-slavonic-core` codes)
// ---------------------------------------------------------------------------

fn map_case_letter(letter: &str) -> Option<&'static str> {
    match letter {
        "И" => Some("nominative"),
        "Р" => Some("genitive"),
        "Д" => Some("dative"),
        "В" => Some("accusative"),
        "Т" => Some("instrumental"),
        "П" => Some("locative"),
        "З" | "Зв" => Some("vocative"),
        _ => None,
    }
}

/// Parses a printed row label like `И.`, `Р.П`, `И.В.З.`, `1-е`, `2-е и 3-е`,
/// `единственное число`, `краткие`.
#[derive(Default, Clone)]
struct RowLabel {
    cases: Vec<&'static str>,
    persons: Vec<&'static str>,
    number: Option<&'static str>,
    form: Option<&'static str>,
}

fn parse_row_label(label: &str) -> Option<RowLabel> {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Some(RowLabel::default());
    }
    let mut parsed = RowLabel::default();
    if let Some(number) = map_number_word(trimmed) {
        parsed.number = Some(number);
        return Some(parsed);
    }
    match trimmed.to_lowercase().as_str() {
        "краткие" | "краткое" => {
            parsed.form = Some("short");
            return Some(parsed);
        }
        "полные" | "полное" => {
            parsed.form = Some("long");
            return Some(parsed);
        }
        _ => {}
    }
    if trimmed.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        for part in trimmed.split([',', 'и']) {
            let part = part.trim().trim_end_matches("-е").trim();
            match part {
                "1" => parsed.persons.push("first"),
                "2" => parsed.persons.push("second"),
                "3" => parsed.persons.push("third"),
                "" => {}
                _ => return None,
            }
        }
        if parsed.persons.is_empty() {
            return None;
        }
        return Some(parsed);
    }
    for part in trimmed.split('.') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        parsed.cases.push(map_case_letter(part)?);
    }
    if parsed.cases.is_empty() {
        return None;
    }
    Some(parsed)
}

fn map_number_word(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    if lower.contains("единственное") {
        Some("singular")
    } else if lower.contains("двойственное") {
        Some("dual")
    } else if lower.contains("множественное") {
        Some("plural")
    } else {
        None
    }
}

fn map_gender_header(text: &str) -> Option<Vec<&'static str>> {
    let lower = text.to_lowercase();
    // Reject headers that merely mention a gender inside longer prose.
    if lower.chars().count() > 24 {
        return None;
    }
    let mut masculine = false;
    let mut feminine = false;
    let mut neuter = false;
    for part in lower.split([',', '.']).flat_map(|part| part.split(" и ")) {
        let part = part.trim().trim_end_matches('.');
        if part.is_empty() || part == "р" || part == "род" {
            continue;
        }
        if part.starts_with("муж") || part == "м" {
            masculine = true;
        } else if part.starts_with("жен") || part == "ж" {
            feminine = true;
        } else if part.starts_with("сред") || part.starts_with("ср") {
            neuter = true;
        } else {
            return None;
        }
    }
    if !(masculine || feminine || neuter) {
        return None;
    }
    let mut genders = Vec::new();
    if masculine {
        genders.push("masculine");
    }
    if feminine {
        genders.push("feminine");
    }
    if neuter {
        genders.push("neuter");
    }
    Some(genders)
}

fn map_tense_header(text: &str) -> Option<&'static str> {
    let lower = text.to_lowercase();
    if lower.contains("имперфект") {
        Some("imperfect")
    } else if lower.contains("аорист") {
        Some("aorist")
    } else if lower.contains("плюсквамперфект") || lower.contains("перфект") {
        None
    } else if lower.contains("настоящее") {
        Some("present")
    } else if lower.contains("будущее") {
        Some("future")
    } else {
        None
    }
}

fn map_form_header(text: &str) -> Option<&'static str> {
    match text.to_lowercase().as_str() {
        "краткие" | "краткое" => Some("short"),
        "полные" | "полное" => Some("long"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct OracleRow {
    pub(crate) section: String,
    pub(crate) artifact: String,
    pub(crate) table_index: usize,
    pub(crate) pos: String,
    pub(crate) headword: String,
    pub(crate) column_label: String,
    pub(crate) case: String,
    pub(crate) number: String,
    pub(crate) gender: String,
    pub(crate) person: String,
    pub(crate) tense: String,
    pub(crate) form: String,
    pub(crate) surface: String,
}

pub(crate) struct TableReport {
    pub(crate) artifact: String,
    pub(crate) table_index: usize,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) detail: String,
    pub(crate) cells: usize,
}

pub(crate) struct Extraction {
    pub(crate) rows: Vec<OracleRow>,
    pub(crate) tables: Vec<TableReport>,
    pub(crate) sections: BTreeMap<String, String>,
}

#[derive(Clone, Default)]
struct ColumnContext {
    genders: Vec<&'static str>,
    number: Option<&'static str>,
    tense: Option<&'static str>,
    form: Option<&'static str>,
    labels: Vec<String>,
    headword: Option<String>,
}

fn join(values: &[&str]) -> String {
    values.join("+")
}

fn is_empty_surface(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.is_empty() || trimmed == "\u{2013}" || trimmed == "\u{2014}" || trimmed == "-"
}

/// Extracts one curated `Decline` table into oracle rows.
#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
fn extract_table(
    section: &str,
    artifact: &str,
    table_index: usize,
    body: &str,
    pos: &str,
    default_number: Option<&'static str>,
    default_tense: Option<&'static str>,
    default_form: Option<&'static str>,
) -> Result<(Vec<OracleRow>, usize), String> {
    let source_rows = parse_rows(body);
    // Expand the grid, tracking rowspans so column indices stay aligned.
    let mut columns: Vec<ColumnContext> = Vec::new();
    let mut pending: BTreeMap<usize, (usize, SourceCell)> = BTreeMap::new();
    let mut current_number: Option<&'static str> = None;
    let mut current_gender_banner: Option<Vec<&'static str>> = None;
    let mut seen_banner = false;
    let mut out = Vec::new();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut endings_skipped = 0usize;

    for row in &source_rows {
        // Lay the row out on the grid.
        struct Placed {
            cell: SourceCell,
            start: usize,
            span: usize,
        }
        let mut placed: Vec<Placed> = Vec::new();
        let mut column = 0;
        let mut queue = row.iter();
        loop {
            if let Some((remaining, cell)) = pending.get(&column).cloned() {
                // A rowspan continuation occupies this column; it is not a
                // freshly printed cell, so it emits nothing.
                if remaining > 1 {
                    pending.insert(column, (remaining - 1, cell.clone()));
                } else {
                    pending.remove(&column);
                }
                column += cell.colspan;
                continue;
            }
            let Some(cell) = queue.next() else {
                break;
            };
            if cell.rowspan > 1 {
                pending.insert(column, (cell.rowspan - 1, cell.clone()));
            }
            placed.push(Placed {
                cell: cell.clone(),
                start: column,
                span: cell.colspan,
            });
            column += cell.colspan;
        }
        let width = column;
        if columns.len() < width {
            columns.resize(width, ColumnContext::default());
        }
        if placed.is_empty() {
            continue;
        }

        let all_headers = placed.iter().all(|entry| entry.cell.header);
        if all_headers {
            // Banner rows: a single header spanning the full grid sets the
            // number (or gender) context for the rows that follow.
            let banner = placed
                .iter()
                .find(|entry| !entry.cell.text.trim().is_empty());
            let banner_count = placed
                .iter()
                .filter(|entry| !entry.cell.text.trim().is_empty())
                .count();
            if banner_count == 1 {
                let banner = banner
                    .map(|entry| entry.cell.text.clone())
                    .unwrap_or_default();
                if let Some(number) = map_number_word(&banner) {
                    current_number = Some(number);
                    seen_banner = true;
                    continue;
                }
                if let Some(genders) = map_gender_header(&banner) {
                    current_gender_banner = Some(genders);
                    seen_banner = true;
                    continue;
                }
            }
            // Column-header row: distribute header text across the spanned
            // columns and fold recognized dimensions into per-column context.
            for entry in &placed {
                let text = entry.cell.text.trim();
                if text.is_empty() {
                    continue;
                }
                for target in entry.start..entry.start + entry.span {
                    let Some(context) = columns.get_mut(target) else {
                        continue;
                    };
                    if let Some(genders) = map_gender_header(text) {
                        context.genders = genders;
                    } else if let Some(number) = map_number_word(text) {
                        context.number = Some(number);
                        context.labels.push(text.to_string());
                    } else if let Some(tense) = map_tense_header(text) {
                        context.tense = Some(tense);
                        context.labels.push(text.to_string());
                    } else if let Some(form) = map_form_header(text) {
                        context.form = Some(form);
                    } else {
                        context.labels.push(text.to_string());
                    }
                }
            }
            continue;
        }

        // Data row. Walk cells left to right; label cells (DeclName class or
        // header cells) open a labelled run, so the §37 paired layout
        // (label, form, label, form) parses without special cases.
        let mut label: Option<RowLabel> = None;
        let mut label_text = String::new();
        let mut saw_label_cell = false;
        let mut data_cells: Vec<Placed> = Vec::new();
        let mut runs: Vec<(RowLabel, String, Vec<Placed>)> = Vec::new();
        for entry in placed {
            let is_label = entry.cell.class.contains("DeclName") || entry.cell.header;
            if is_label {
                if saw_label_cell {
                    runs.push((
                        label.clone().unwrap_or_default(),
                        label_text.clone(),
                        std::mem::take(&mut data_cells),
                    ));
                }
                saw_label_cell = true;
                label_text = entry.cell.text.clone();
                label = parse_row_label(&entry.cell.text);
                if label.is_none() {
                    return Err(format!(
                        "{artifact} table {table_index}: unparseable row label {:?}",
                        entry.cell.text
                    ));
                }
            } else {
                data_cells.push(entry);
            }
        }
        if !saw_label_cell {
            // A leading data row with no label at all: treat as a headword
            // row when it appears before any banner (e.g. the infinitive row
            // of the §82 conjugation table).
            if !seen_banner {
                for entry in data_cells {
                    if is_empty_surface(&entry.cell.text) {
                        continue;
                    }
                    for target in entry.start..entry.start + entry.span {
                        if let Some(context) = columns.get_mut(target) {
                            context.headword = Some(entry.cell.text.clone());
                        }
                    }
                }
                continue;
            }
            return Err(format!(
                "{artifact} table {table_index}: data row without a label after a banner"
            ));
        }
        runs.push((label.unwrap_or_default(), label_text, data_cells));

        for (row_label, printed_label, cells) in runs {
            // An empty label with data before any banner is a headword row.
            if row_label.cases.is_empty()
                && row_label.persons.is_empty()
                && row_label.number.is_none()
                && row_label.form.is_none()
                && printed_label.trim().is_empty()
            {
                if !seen_banner {
                    for entry in cells {
                        if is_empty_surface(&entry.cell.text) {
                            continue;
                        }
                        for target in entry.start..entry.start + entry.span {
                            if let Some(context) = columns.get_mut(target) {
                                context.headword = Some(entry.cell.text.clone());
                            }
                        }
                    }
                    continue;
                }
                return Err(format!(
                    "{artifact} table {table_index}: empty row label after a banner"
                ));
            }
            for entry in cells {
                if is_empty_surface(&entry.cell.text) {
                    continue;
                }
                if entry.cell.text.trim().starts_with('-') {
                    // A bare ending is a formation schema fragment, not a
                    // printed surface; counted, reported, never silently lost.
                    endings_skipped += 1;
                    continue;
                }
                let mut genders: Vec<&'static str> = Vec::new();
                let mut column_number: Option<&'static str> = None;
                let mut tense: Option<&'static str> = None;
                let mut form = row_label.form.or(default_form);
                let mut labels: Vec<String> = Vec::new();
                let mut headwords: Vec<String> = Vec::new();
                for target in entry.start..entry.start + entry.span {
                    let Some(context) = columns.get(target) else {
                        continue;
                    };
                    for gender in &context.genders {
                        if !genders.contains(gender) {
                            genders.push(gender);
                        }
                    }
                    if column_number.is_none() {
                        column_number = context.number;
                    }
                    if tense.is_none() {
                        tense = context.tense;
                    }
                    if form.is_none() {
                        form = context.form;
                    }
                    for text in &context.labels {
                        if !labels.contains(text) {
                            labels.push(text.clone());
                        }
                    }
                    if let Some(headword) = &context.headword {
                        if !headwords.contains(headword) {
                            headwords.push(headword.clone());
                        }
                    }
                }
                if genders.is_empty() {
                    if let Some(banner) = &current_gender_banner {
                        genders = banner.clone();
                    }
                }
                let number = row_label
                    .number
                    .or(current_number)
                    .or(column_number)
                    .or(default_number)
                    .map(str::to_string)
                    .unwrap_or_default();
                out.push(OracleRow {
                    section: section.to_string(),
                    artifact: artifact.to_string(),
                    table_index,
                    pos: pos.to_string(),
                    headword: headwords.join(" | "),
                    column_label: labels.join(" | "),
                    case: join(&row_label.cases),
                    number,
                    gender: join(&genders),
                    person: join(&row_label.persons),
                    tense: tense
                        .or(default_tense)
                        .map(str::to_string)
                        .unwrap_or_default(),
                    form: form.map(str::to_string).unwrap_or_default(),
                    surface: entry.cell.text.clone(),
                });
                spans.push((entry.start, entry.span));
            }
        }
    }

    // Backfill headwords for declension tables: per grid column, the first
    // printed nominative cell (the exemplar of the column's first number
    // section) names the column.
    let mut nominative_by_column: BTreeMap<usize, String> = BTreeMap::new();
    for (row, (start, span)) in out.iter().zip(&spans) {
        if row.case.starts_with("nominative") && *span == 1 {
            nominative_by_column
                .entry(*start)
                .or_insert_with(|| row.surface.clone());
        }
    }
    for (row, (start, span)) in out.iter_mut().zip(&spans) {
        if row.headword.is_empty() && !row.case.is_empty() {
            let mut headwords: Vec<&String> = Vec::new();
            for target in *start..*start + *span {
                if let Some(headword) = nominative_by_column.get(&target) {
                    if !headwords.contains(&headword) {
                        headwords.push(headword);
                    }
                }
            }
            row.headword = headwords
                .iter()
                .map(|headword| headword.as_str())
                .collect::<Vec<_>>()
                .join(" | ");
        }
    }
    Ok((out, endings_skipped))
}

/// Classifies every table in every artifact and extracts the curated ones.
pub(crate) fn extract_all(root: &Path) -> Result<Extraction, Box<dyn Error>> {
    let source_dir = root.join(SOURCE_DIR);
    let mut file_names: Vec<String> = fs::read_dir(&source_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".htm") || name.ends_with(".html"))
        .collect();
    file_names.sort();

    let mut dispositions: BTreeMap<(String, usize), &Disposition> = BTreeMap::new();
    for disposition in DISPOSITIONS {
        if dispositions
            .insert(
                (disposition.artifact.to_string(), disposition.index),
                disposition,
            )
            .is_some()
        {
            return Err(format!(
                "duplicate disposition for {} table {}",
                disposition.artifact, disposition.index
            )
            .into());
        }
    }
    let mut consumed: BTreeMap<(String, usize), bool> = dispositions
        .keys()
        .map(|key| (key.clone(), false))
        .collect();

    let mut extraction = Extraction {
        rows: Vec::new(),
        tables: Vec::new(),
        sections: BTreeMap::new(),
    };

    for name in &file_names {
        let html = fs::read_to_string(source_dir.join(name))?;
        let section = html
            .find("<h3>")
            .and_then(|start| {
                html[start..]
                    .find("</h3>")
                    .map(|end| clean_text(&html[start + 4..start + end]))
            })
            .unwrap_or_else(|| name.trim_end_matches(".htm").to_string());
        extraction.sections.insert(name.clone(), section.clone());

        let mut decline_index = 0;
        for (table_index, table) in find_tables(&html).into_iter().enumerate() {
            let attrs = table.attrs.to_ascii_lowercase();
            if attrs.contains("decline") {
                let key = (name.clone(), decline_index);
                let disposition = dispositions.get(&key).ok_or_else(|| {
                    format!("no curated disposition for {name} Decline table {decline_index}")
                })?;
                consumed.insert(key, true);
                match &disposition.action {
                    Action::Extract {
                        pos,
                        default_number,
                        default_tense,
                        default_form,
                    } => {
                        let (rows, endings_skipped) = extract_table(
                            &section,
                            name,
                            table_index,
                            &table.body,
                            pos,
                            *default_number,
                            *default_tense,
                            *default_form,
                        )
                        .map_err(|message| -> Box<dyn Error> { message.into() })?;
                        let detail = if endings_skipped == 0 {
                            format!("pos={pos}")
                        } else {
                            format!(
                                "pos={pos}; {endings_skipped} bare-ending cells skipped (schema fragments, not surfaces)"
                            )
                        };
                        extraction.tables.push(TableReport {
                            artifact: name.clone(),
                            table_index,
                            kind: "Decline".into(),
                            status: "extracted".into(),
                            detail,
                            cells: rows.len(),
                        });
                        extraction.rows.extend(rows);
                    }
                    Action::Exclude { reason } => {
                        extraction.tables.push(TableReport {
                            artifact: name.clone(),
                            table_index,
                            kind: "Decline".into(),
                            status: "excluded".into(),
                            detail: (*reason).to_string(),
                            cells: 0,
                        });
                    }
                }
                decline_index += 1;
            } else if attrs.contains("wordtable") {
                extraction.tables.push(TableReport {
                    artifact: name.clone(),
                    table_index,
                    kind: "WordTable".into(),
                    status: "excluded".into(),
                    detail: "word-formation/orthography/syntax illustration (WordTable class)"
                        .into(),
                    cells: 0,
                });
            } else if attrs.contains("align=\"center\"") {
                extraction.tables.push(TableReport {
                    artifact: name.clone(),
                    table_index,
                    kind: "navigation".into(),
                    status: "excluded".into(),
                    detail: "page navigation strip".into(),
                    cells: 0,
                });
            } else {
                let snippet: String = clean_text(&table.body).chars().take(60).collect();
                extraction.tables.push(TableReport {
                    artifact: name.clone(),
                    table_index,
                    kind: "unclassed".into(),
                    status: "excluded".into(),
                    detail: format!("layout/illustration table (no paradigm class): {snippet}"),
                    cells: 0,
                });
            }
        }
    }

    for (key, used) in &consumed {
        if !used {
            return Err(format!(
                "disposition for {} table {} matched no Decline table in the source",
                key.0, key.1
            )
            .into());
        }
    }
    Ok(extraction)
}

fn render_oracle(extraction: &Extraction) -> String {
    let mut out = String::new();
    out.push_str("# Gold paradigm oracle: Alypy (Gamanovich) grammar printed paradigm tables.\n");
    let _ = writeln!(out, "# Source: {SOURCE_ID} ({SOURCE_DIR}).");
    out.push_str(
        "# Generator: cargo xtask alypy-paradigm-oracle (crates/xtask/src/alypy_oracle.rs).\n",
    );
    out.push_str("# Surfaces are verbatim as printed (accents, hyphenated morpheme boundaries,\n");
    out.push_str("# variant parentheses intact); the comparison contract decides equivalence.\n");
    out.push_str(ORACLE_COLUMNS);
    out.push('\n');
    for row in &extraction.rows {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.section,
            row.artifact,
            row.table_index,
            row.pos,
            row.headword,
            row.column_label,
            row.case,
            row.number,
            row.gender,
            row.person,
            row.tense,
            row.form,
            row.surface,
        );
    }
    out
}

fn render_review(extraction: &Extraction) -> String {
    let mut out = String::new();
    out.push_str("# Alypy paradigm-table extraction review\n\n");
    let _ = writeln!(
        out,
        "Source: `{SOURCE_ID}` (`{SOURCE_DIR}`). Generated by `cargo xtask alypy-paradigm-oracle`; regenerating is a deliberate curated act (the committed oracle is the frozen artifact)."
    );
    out.push('\n');
    let extracted: Vec<&TableReport> = extraction
        .tables
        .iter()
        .filter(|table| table.status == "extracted")
        .collect();
    let excluded_paradigm: Vec<&TableReport> = extraction
        .tables
        .iter()
        .filter(|table| table.status == "excluded" && table.kind == "Decline")
        .collect();
    let excluded_other = extraction
        .tables
        .iter()
        .filter(|table| table.status == "excluded" && table.kind != "Decline")
        .count();
    let _ = writeln!(
        out,
        "Totals: {} tables found; {} extracted ({} cells); {} `Decline` tables excluded by curated decision; {} non-paradigm tables excluded by class.\n",
        extraction.tables.len(),
        extracted.len(),
        extraction.rows.len(),
        excluded_paradigm.len(),
        excluded_other,
    );

    out.push_str("## Conventions\n\n");
    out.push_str("- Feature values use the kernel `church-slavonic-core` codes (case/number/gender/person/tense); `form` additionally carries `short`/`long`, `imperative`, `l-participle`.\n");
    out.push_str("- Combined printed labels (e.g. `И.В.З.`, `2-е и 3-е`) become `+`-joined feature values on one row: one oracle row per printed cell, never per implied grid position.\n");
    out.push_str("- A cell whose colspan covers several columns (e.g. `для всех родов`) is one row whose gender is the union of the spanned columns.\n");
    out.push_str("- `headword` is the printed exemplar: the infinitive header row where the table prints one, otherwise the nominative cell of the column's first number section; blank where the table prints no exemplar (e.g. the §97 l-participle and §93/§103 imperative tables).\n");
    out.push_str("- `П.` maps to kernel `locative` (the grammar's предложный падеж).\n");
    out.push_str("- Surfaces are verbatim: accents, titla, hyphenated morpheme boundaries (`ра́б-ъ`), parenthesised variants (`і҆ере́-ю (-е)`), and comma-separated alternatives (`и҆́мава, -ѣ`) are preserved exactly as printed. Pre-reform/Synodal orthography differences (broad он/est forms, initial-uk presentation, ѡ/ѻ choices) are NOT normalized here; the normative comparison contract owns equivalence.\n");
    out.push_str("- Dashes (–/—) and empty cells print no form and yield no row; bare hyphen-initial endings inside otherwise-extracted tables are schema fragments, skipped with a per-table count in the detail column.\n");
    out.push_str("- `table_index` is the document-order index among ALL `<table>` elements of the artifact (navigation strips included), so a row can be located in the raw HTML directly; curated dispositions in the extractor are keyed by the Decline-relative index.\n");
    out.push_str("- In the §37 paired layout only the left column prints the nominative, so right-column rows carry a blank headword; §48 кто̀/что̀ and §37 бра́тїѧ print no number dimension, so `number` is blank there rather than inferred.\n\n");

    out.push_str("## Extracted tables\n\n");
    out.push_str("| artifact | table | section | detail | cells |\n|---|---|---|---|---|\n");
    for table in &extracted {
        let section = extraction
            .sections
            .get(&table.artifact)
            .cloned()
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            table.artifact, table.table_index, section, table.detail, table.cells
        );
    }

    out.push_str("\n## Excluded `Decline` tables (curated)\n\n");
    out.push_str("| artifact | table | section | reason |\n|---|---|---|---|\n");
    for table in &excluded_paradigm {
        let section = extraction
            .sections
            .get(&table.artifact)
            .cloned()
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} |",
            table.artifact, table.table_index, section, table.detail
        );
    }

    out.push_str("\n## Excluded non-paradigm tables\n\n");
    out.push_str("Navigation strips (`align=\"center\"`), `WordTable` illustrations (word lists, formation schemata, orthography comparisons, syntax diagrams), and unclassed layout tables (alphabet/letter inventories, consonant-alternation schemata). Every one is listed:\n\n");
    out.push_str("| artifact | table | kind | reason |\n|---|---|---|---|\n");
    for table in &extraction.tables {
        if table.status == "excluded" && table.kind != "Decline" {
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} |",
                table.artifact, table.table_index, table.kind, table.detail
            );
        }
    }

    out.push_str("\n## Per-POS cell counts\n\n");
    let mut by_pos: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &extraction.rows {
        *by_pos.entry(row.pos.as_str()).or_default() += 1;
    }
    out.push_str("| pos | cells |\n|---|---|\n");
    for (pos, count) in &by_pos {
        let _ = writeln!(out, "| {pos} | {count} |");
    }
    out
}

/// Entry point: `cargo xtask alypy-paradigm-oracle [--check]`.
#[allow(dead_code)]
pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mode = args.next();
    let check = match mode.as_deref() {
        None => false,
        Some("--check") => true,
        Some(other) => return Err(format!("unknown flag {other}").into()),
    };
    let extraction = extract_all(root)?;
    let oracle = render_oracle(&extraction);
    let review = render_review(&extraction);
    if check {
        let committed = fs::read_to_string(root.join(ORACLE_PATH))?;
        if committed != oracle {
            return Err(format!(
                "{ORACLE_PATH} is stale: regenerated extraction differs from the committed oracle"
            )
            .into());
        }
        println!(
            "alypy paradigm oracle: {} cells across {} extracted tables (fresh)",
            extraction.rows.len(),
            extraction
                .tables
                .iter()
                .filter(|table| table.status == "extracted")
                .count()
        );
        return Ok(());
    }
    write_if_changed_atomic(&root.join(ORACLE_PATH), &oracle)?;
    write_if_changed_atomic(&root.join(REVIEW_PATH), &review)?;
    println!(
        "alypy paradigm oracle: wrote {} cells across {} extracted tables ({} tables excluded)",
        extraction.rows.len(),
        extraction
            .tables
            .iter()
            .filter(|table| table.status == "extracted")
            .count(),
        extraction
            .tables
            .iter()
            .filter(|table| table.status == "excluded")
            .count()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }

    /// Interim regeneration entry point while the `alypy-paradigm-oracle`
    /// dispatch is not yet wired into `main.rs`:
    /// `cargo test -p xtask regenerate_alypy_oracle -- --ignored`.
    #[test]
    #[ignore = "writes the committed oracle artifacts; run explicitly"]
    fn regenerate_alypy_oracle_artifacts() {
        let root = workspace_root();
        let extraction = extract_all(&root).expect("extraction succeeds");
        write_if_changed_atomic(&root.join(ORACLE_PATH), &render_oracle(&extraction))
            .expect("write oracle");
        write_if_changed_atomic(&root.join(REVIEW_PATH), &render_review(&extraction))
            .expect("write review");
    }

    #[test]
    fn row_label_parsing_covers_printed_vocabulary() {
        let label = parse_row_label("И.В.З.").expect("case combo");
        assert_eq!(label.cases, vec!["nominative", "accusative", "vocative"]);
        let label = parse_row_label("Р.П").expect("trailing dot omitted");
        assert_eq!(label.cases, vec!["genitive", "locative"]);
        let label = parse_row_label("2-е и 3-е").expect("person combo");
        assert_eq!(label.persons, vec!["second", "third"]);
        let label = parse_row_label("двойственное число").expect("number word");
        assert_eq!(label.number, Some("dual"));
        assert!(parse_row_label("основа").is_none());
    }

    #[test]
    fn header_mapping_distinguishes_imperfect_from_perfect() {
        assert_eq!(map_tense_header("имперфект(преходящее)"), Some("imperfect"));
        assert_eq!(map_tense_header("перфект(прошедшее совершенное)"), None);
        assert_eq!(
            map_tense_header("аорист несовершенного вида"),
            Some("aorist")
        );
        assert_eq!(
            map_gender_header("ж. и ср. р."),
            Some(vec!["feminine", "neuter"])
        );
        assert_eq!(map_gender_header("мужской род"), Some(vec!["masculine"]));
    }

    #[test]
    fn broken_header_row_still_parses() {
        // p034 prints `<th>&nbsp;</th></td><th ...>`: a stray closer.
        let cells = parse_row("<th>&nbsp;</th></td><th colspan=\"2\">Твердое склонение</th>");
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[1].text, "Твердое склонение");
        assert_eq!(cells[1].colspan, 2);
    }

    #[test]
    fn full_extraction_is_deterministic_and_complete() {
        let root = workspace_root();
        if !root.join(SOURCE_DIR).is_dir() {
            eprintln!("skipping: {SOURCE_DIR} not present");
            return;
        }
        let first = extract_all(&root).expect("extraction succeeds");
        let second = extract_all(&root).expect("extraction succeeds twice");
        assert_eq!(render_oracle(&first), render_oracle(&second));
        // Every source table is accounted for: 68 Decline dispositions plus
        // classed exclusions cover the full inventory.
        assert_eq!(
            first
                .tables
                .iter()
                .filter(|table| table.kind == "Decline")
                .count(),
            DISPOSITIONS.len()
        );
        assert!(first.rows.len() > 500, "expected a substantive cell count");
        // Spot-check a known printed cell: §34 dative singular of ра́б-ъ.
        assert!(first.rows.iter().any(|row| row.artifact == "p034.htm"
            && row.case == "dative"
            && row.number == "singular"
            && row.surface == "раб-ꙋ̀"));
        // Vocative dual/plural fusion rows carry `+`-joined cases.
        assert!(
            first
                .rows
                .iter()
                .any(|row| row.case == "nominative+accusative+vocative" && row.number == "dual")
        );
    }
}
