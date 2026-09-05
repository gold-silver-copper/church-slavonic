//! The Alypy (Gamanovich) *Grammar of the Church Slavonic Language* web
//! edition: the HTML mechanics that turn its printed paradigm tables into
//! cell grids, the one-time filter into `data/intermediate/alypy.jsonl`, and
//! the reading of a grid's printed labels (`И.В.З.`, `2-е и 3-е`,
//! `двойственное число`, `ж. и ср. р.`) into typed features. Which tables are
//! paradigms of which part of speech is the importer's decision.
//!
//! Printed surfaces are kept as printed — accents, breathings, titla and the
//! pre-reform letter choices — minus the typographic apparatus: hyphenated
//! morpheme boundaries (`ра́б-ъ`) are joined, footnote markers dropped, and
//! the print's variant notation (`и҆́мава, -ѣ`; `мꙋ́др-ъ (-а)`; `бы́сть (бы̀)`)
//! expanded into a list of alternatives by [`alternatives`].

use church_slavonic::grammar::Recension;
use church_slavonic::grammar::{Case, Gender, Number, Person};
use church_slavonic::orthography::{realise, strip_marks};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use unicode_normalization::UnicodeNormalization;

// ---------------------------------------------------------------------------
// Intermediate schema: one JSON line per `Decline`-classed table.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub text: String,
    pub header: bool,
    pub class: String,
    pub colspan: usize,
    pub rowspan: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub artifact: String,
    /// Index among the artifact's `Decline` tables, in document order.
    pub index: usize,
    pub rows: Vec<Vec<Cell>>,
}

/// Reduce the HTML artifacts to their paradigm-classed tables, one JSON line
/// each, cell structure intact. No feature interpretation happens here.
pub fn filter(source_dir: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
    let mut names: Vec<String> = fs::read_dir(source_dir)?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".htm") || n.ends_with(".html"))
        .collect();
    names.sort();
    let mut writer = BufWriter::new(fs::File::create(out)?);
    let mut count = 0usize;
    for name in &names {
        let html = fs::read_to_string(source_dir.join(name))?;
        let mut index = 0;
        for raw in find_tables(&html) {
            if !raw.attrs.to_ascii_lowercase().contains("decline") {
                continue;
            }
            let table = Table {
                artifact: name.clone(),
                index,
                rows: parse_rows(&raw.body),
            };
            serde_json::to_writer(&mut writer, &table)?;
            writer.write_all(b"\n")?;
            index += 1;
            count += 1;
        }
    }
    writer.flush()?;
    println!(
        "Filtered Alypy grammar: {count} paradigm tables from {} artifacts into {}",
        names.len(),
        out.display()
    );
    Ok(())
}

pub fn read(path: &Path) -> Result<Vec<Table>, Box<dyn Error>> {
    let reader = BufReader::new(fs::File::open(path)?);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line)?);
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// HTML mechanics
// ---------------------------------------------------------------------------

fn clean_text(html: &str) -> String {
    let mut no_tags = String::new();
    let bytes = html.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        if bytes[position] == b'<' {
            let rest = &html[position..];
            let close = rest.find('>').map_or(rest.len(), |o| o + 1);
            if rest[..close].to_ascii_lowercase().starts_with("<br") {
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
            .map(|(o, _)| o);
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

/// Splits a `<tr>` body into cells, tolerating the source's stray `</td>`
/// fragments: a cell runs from its opening tag to the next cell opening.
fn parse_row(row: &str) -> Vec<Cell> {
    let lower = row.to_ascii_lowercase();
    let mut openings = Vec::new();
    for tag in ["<td", "<th"] {
        let mut search = 0;
        while let Some(offset) = lower[search..].find(tag) {
            let at = search + offset;
            let after = lower.as_bytes().get(at + 3).copied().unwrap_or(b' ');
            if matches!(after, b' ' | b'>' | b'\n' | b'\r' | b'\t') {
                openings.push((at, tag == "<th"));
            }
            search = at + 3;
        }
    }
    openings.sort_by_key(|(at, _)| *at);
    let mut cells = Vec::new();
    for (i, &(start, header)) in openings.iter().enumerate() {
        let end = openings.get(i + 1).map_or(row.len(), |&(next, _)| next);
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
            match owned[at..].find("</span>") {
                Some(end) => owned.replace_range(at..at + end + 7, ""),
                None => break,
            }
        }
        cells.push(Cell {
            text: clean_text(&owned),
            header,
            class: attr_value(attrs, "class").unwrap_or_default(),
            colspan: attr_value(attrs, "colspan")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            rowspan: attr_value(attrs, "rowspan")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
        });
    }
    cells
}

fn attr_value(attrs: &str, name: &str) -> Option<String> {
    let lower = attrs.to_ascii_lowercase();
    let at = lower.find(name)?;
    let rest = attrs[at + name.len()..].trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    if let Some(stripped) = rest.strip_prefix('"') {
        stripped.split('"').next().map(str::to_string)
    } else {
        rest.split([' ', '>', '\t']).next().map(str::to_string)
    }
}

fn parse_rows(body: &str) -> Vec<Vec<Cell>> {
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
// Printed-label vocabulary
// ---------------------------------------------------------------------------

/// The tense words the grammar prints as column headers. `Future` covers
/// both the perfective present (`да́мъ`) and the `бꙋ́дꙋ` series.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenseWord {
    Present,
    Imperfect,
    Aorist,
    Future,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormWord {
    Short,
    Long,
}

fn case_letter(letter: &str) -> Option<Case> {
    match letter {
        "И" => Some(Case::Nominative),
        "Р" => Some(Case::Genitive),
        "Д" => Some(Case::Dative),
        "В" => Some(Case::Accusative),
        "Т" => Some(Case::Instrumental),
        "П" => Some(Case::Locative),
        "З" | "Зв" => Some(Case::Vocative),
        _ => None,
    }
}

#[derive(Default, Clone)]
struct RowLabel {
    cases: Vec<Case>,
    persons: Vec<Person>,
    number: Option<Number>,
    form: Option<FormWord>,
}

fn parse_row_label(label: &str) -> Option<RowLabel> {
    let trimmed = label.trim();
    let mut parsed = RowLabel::default();
    if trimmed.is_empty() {
        return Some(parsed);
    }
    if let Some(number) = number_word(trimmed) {
        parsed.number = Some(number);
        return Some(parsed);
    }
    if let Some(form) = form_word(trimmed) {
        parsed.form = Some(form);
        return Some(parsed);
    }
    if trimmed.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        for part in trimmed.split([',', 'и']) {
            match part.trim().trim_end_matches("-е").trim() {
                "1" => parsed.persons.push(Person::First),
                "2" => parsed.persons.push(Person::Second),
                "3" => parsed.persons.push(Person::Third),
                "" => {}
                _ => return None,
            }
        }
        return (!parsed.persons.is_empty()).then_some(parsed);
    }
    for part in trimmed.split('.') {
        let part = part.trim();
        if !part.is_empty() {
            parsed.cases.push(case_letter(part)?);
        }
    }
    (!parsed.cases.is_empty()).then_some(parsed)
}

fn number_word(text: &str) -> Option<Number> {
    let lower = text.to_lowercase();
    if lower.contains("единственное") {
        Some(Number::Singular)
    } else if lower.contains("двойственное") {
        Some(Number::Dual)
    } else if lower.contains("множественное") {
        Some(Number::Plural)
    } else {
        None
    }
}

fn gender_header(text: &str) -> Option<Vec<Gender>> {
    let lower = text.to_lowercase();
    if lower.chars().count() > 24 {
        return None;
    }
    let (mut m, mut f, mut n) = (false, false, false);
    for part in lower.split([',', '.']).flat_map(|p| p.split(" и ")) {
        let part = part.trim().trim_end_matches('.');
        if part.is_empty() || part == "р" || part == "род" {
            continue;
        }
        if part.starts_with("муж") || part == "м" {
            m = true;
        } else if part.starts_with("жен") || part == "ж" {
            f = true;
        } else if part.starts_with("сред") || part.starts_with("ср") {
            n = true;
        } else {
            return None;
        }
    }
    if !(m || f || n) {
        return None;
    }
    let mut out = Vec::new();
    if m {
        out.push(Gender::Masculine);
    }
    if f {
        out.push(Gender::Feminine);
    }
    if n {
        out.push(Gender::Neuter);
    }
    Some(out)
}

fn tense_header(text: &str) -> Option<TenseWord> {
    let lower = text.to_lowercase();
    if lower.contains("имперфект") {
        Some(TenseWord::Imperfect)
    } else if lower.contains("аорист") {
        Some(TenseWord::Aorist)
    } else if lower.contains("перфект") {
        None
    } else if lower.contains("настоящее") {
        Some(TenseWord::Present)
    } else if lower.contains("будущее") {
        Some(TenseWord::Future)
    } else {
        None
    }
}

fn form_word(text: &str) -> Option<FormWord> {
    match text.to_lowercase().as_str() {
        "краткие" | "краткое" => Some(FormWord::Short),
        "полные" | "полное" => Some(FormWord::Long),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Grid reading
// ---------------------------------------------------------------------------

/// Assumptions a table's surrounding prose supplies where the print does not.
#[derive(Debug, Clone, Copy, Default)]
pub struct Defaults {
    pub number: Option<Number>,
    pub tense: Option<TenseWord>,
}

/// One printed cell with the features its row label and column headers name.
#[derive(Debug, Clone)]
pub struct Row {
    /// Leftmost grid column the cell occupies.
    pub column: usize,
    /// The column's printed exemplar: the infinitive header row where the
    /// table prints one, otherwise the column's first nominative cell.
    pub headword: String,
    /// Column header texts that named no recognised feature.
    pub labels: Vec<String>,
    pub cases: Vec<Case>,
    pub number: Option<Number>,
    pub genders: Vec<Gender>,
    pub persons: Vec<Person>,
    pub tense: Option<TenseWord>,
    pub form: Option<FormWord>,
    pub surface: String,
}

#[derive(Clone, Default)]
struct ColumnContext {
    genders: Vec<Gender>,
    number: Option<Number>,
    tense: Option<TenseWord>,
    form: Option<FormWord>,
    labels: Vec<String>,
    headword: Option<String>,
}

fn is_empty_surface(text: &str) -> bool {
    let t = text.trim();
    t.is_empty() || t == "\u{2013}" || t == "\u{2014}" || t == "-"
}

struct Placed {
    cell: Cell,
    start: usize,
    span: usize,
}

/// Read one table's grid: banner rows set the number/gender context, header
/// rows the per-column features, labelled data rows the cases/persons.
pub fn rows(table: &Table, defaults: Defaults) -> Result<Vec<Row>, String> {
    let name = format!("{} table {}", table.artifact, table.index);
    let mut columns: Vec<ColumnContext> = Vec::new();
    let mut pending: BTreeMap<usize, (usize, Cell)> = BTreeMap::new();
    let mut current_number: Option<Number> = None;
    let mut gender_banner: Option<Vec<Gender>> = None;
    let mut seen_banner = false;
    let mut out: Vec<Row> = Vec::new();

    for row in &table.rows {
        let mut placed: Vec<Placed> = Vec::new();
        let mut column = 0;
        let mut queue = row.iter();
        loop {
            if let Some((remaining, cell)) = pending.get(&column).cloned() {
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
        if columns.len() < column {
            columns.resize(column, ColumnContext::default());
        }
        if placed.is_empty() {
            continue;
        }

        if placed.iter().all(|p| p.cell.header) {
            let banners: Vec<&Placed> = placed
                .iter()
                .filter(|p| !p.cell.text.trim().is_empty())
                .collect();
            if banners.len() == 1 {
                let text = banners[0].cell.text.clone();
                if let Some(number) = number_word(&text) {
                    current_number = Some(number);
                    seen_banner = true;
                    continue;
                }
                if let Some(genders) = gender_header(&text) {
                    gender_banner = Some(genders);
                    seen_banner = true;
                    continue;
                }
            }
            for p in &placed {
                let text = p.cell.text.trim();
                if text.is_empty() {
                    continue;
                }
                for target in p.start..p.start + p.span {
                    let Some(ctx) = columns.get_mut(target) else {
                        continue;
                    };
                    if let Some(genders) = gender_header(text) {
                        ctx.genders = genders;
                    } else if let Some(number) = number_word(text) {
                        ctx.number = Some(number);
                        ctx.labels.push(text.to_string());
                    } else if let Some(tense) = tense_header(text) {
                        ctx.tense = Some(tense);
                        ctx.labels.push(text.to_string());
                    } else if let Some(form) = form_word(text) {
                        ctx.form = Some(form);
                    } else {
                        ctx.labels.push(text.to_string());
                    }
                }
            }
            continue;
        }

        // Data row: label cells open a labelled run of data cells.
        let mut label: Option<RowLabel> = None;
        let mut label_text = String::new();
        let mut saw_label = false;
        let mut data: Vec<Placed> = Vec::new();
        let mut runs: Vec<(RowLabel, String, Vec<Placed>)> = Vec::new();
        for p in placed {
            if p.cell.class.contains("DeclName") || p.cell.header {
                if saw_label {
                    runs.push((
                        label.clone().unwrap_or_default(),
                        label_text.clone(),
                        std::mem::take(&mut data),
                    ));
                }
                saw_label = true;
                label_text = p.cell.text.clone();
                label = parse_row_label(&p.cell.text);
                if label.is_none() {
                    return Err(format!("{name}: unparseable row label {:?}", p.cell.text));
                }
            } else {
                data.push(p);
            }
        }
        if !saw_label {
            if seen_banner {
                return Err(format!("{name}: data row without a label after a banner"));
            }
            set_headwords(&mut columns, &data);
            continue;
        }
        runs.push((label.unwrap_or_default(), label_text, data));

        for (row_label, printed, cells) in runs {
            let unlabelled = row_label.cases.is_empty()
                && row_label.persons.is_empty()
                && row_label.number.is_none()
                && row_label.form.is_none()
                && printed.trim().is_empty();
            if unlabelled {
                if seen_banner {
                    return Err(format!("{name}: empty row label after a banner"));
                }
                set_headwords(&mut columns, &cells);
                continue;
            }
            for p in cells {
                if is_empty_surface(&p.cell.text) || p.cell.text.trim().starts_with('-') {
                    // A bare ending is a formation schema fragment, not a surface.
                    continue;
                }
                let mut genders: Vec<Gender> = Vec::new();
                let mut column_number = None;
                let mut tense = None;
                let mut form = row_label.form;
                let mut labels: Vec<String> = Vec::new();
                let mut headwords: Vec<String> = Vec::new();
                for target in p.start..p.start + p.span {
                    let Some(ctx) = columns.get(target) else {
                        continue;
                    };
                    for g in &ctx.genders {
                        if !genders.contains(g) {
                            genders.push(*g);
                        }
                    }
                    column_number = column_number.or(ctx.number);
                    tense = tense.or(ctx.tense);
                    form = form.or(ctx.form);
                    for l in &ctx.labels {
                        if !labels.contains(l) {
                            labels.push(l.clone());
                        }
                    }
                    if let Some(h) = &ctx.headword
                        && !headwords.contains(h)
                    {
                        headwords.push(h.clone());
                    }
                }
                if genders.is_empty()
                    && let Some(banner) = &gender_banner
                {
                    genders = banner.clone();
                }
                out.push(Row {
                    column: p.start,
                    headword: headwords.join(" | "),
                    labels,
                    cases: row_label.cases.clone(),
                    number: row_label
                        .number
                        .or(current_number)
                        .or(column_number)
                        .or(defaults.number),
                    genders,
                    persons: row_label.persons.clone(),
                    tense: tense.or(defaults.tense),
                    form,
                    surface: p.cell.text.clone(),
                });
            }
        }
    }

    // Declension tables: the column's first nominative cell names the column.
    let mut nominative_by_column: BTreeMap<usize, String> = BTreeMap::new();
    for row in &out {
        if row.cases.first() == Some(&Case::Nominative) {
            nominative_by_column
                .entry(row.column)
                .or_insert_with(|| row.surface.clone());
        }
    }
    for row in &mut out {
        if row.headword.is_empty()
            && !row.cases.is_empty()
            && let Some(h) = nominative_by_column.get(&row.column)
        {
            row.headword = h.clone();
        }
    }
    Ok(out)
}

fn set_headwords(columns: &mut [ColumnContext], cells: &[Placed]) {
    for p in cells {
        if is_empty_surface(&p.cell.text) {
            continue;
        }
        for target in p.start..p.start + p.span {
            if let Some(ctx) = columns.get_mut(target) {
                ctx.headword = Some(p.cell.text.clone());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Printed surfaces
// ---------------------------------------------------------------------------

/// The lemma key of a printed headword: the part before any ` = `/` — `
/// present-stem gloss or parenthesised variant, morpheme hyphens joined,
/// marks stripped, lowercased. `None` for a multi-word headword.
pub fn lemma_key(headword: &str) -> Option<String> {
    let mut head = headword.trim();
    for sep in [" = ", " — ", " – ", " ("] {
        if let Some((left, _)) = head.split_once(sep) {
            head = left.trim();
        }
    }
    let joined: String = head.chars().filter(|c| *c != '-').collect();
    if joined.is_empty() || joined.contains(' ') || joined.contains(',') {
        return None;
    }
    Some(realise(&joined, &Recension::Synodal))
}

/// Expand a printed cell into its attested alternatives, primary first.
/// `а, -б` and `а (-б)` substitute an ending (the segment after the primary's
/// last hyphen, or as many final letters as the ending has); `а (б)` after a
/// space is a whole alternative word when it shares the primary's initial
/// letter, and an ending otherwise; `(н)е́мъ` is an optional prefix.
pub fn alternatives(printed: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut primary_hyphenated: Option<String> = None;
    for part in split_alternatives(printed) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(rest) = part.strip_prefix('(')
            && let Some((prefix, main)) = rest.split_once(')')
        {
            let main = main.trim();
            push(&mut out, main);
            push(&mut out, &format!("{prefix}{main}"));
            primary_hyphenated = Some(main.to_string());
            continue;
        }
        let (main, parens) = split_parens(part);
        let main = main.trim();
        let base = if let Some(ending) = main.strip_prefix('-') {
            match &primary_hyphenated {
                Some(p) => apply_ending(p, ending),
                None => continue,
            }
        } else {
            main.to_string()
        };
        push(&mut out, &base);
        for (spaced, paren) in parens {
            let paren = paren.trim();
            // A word-initial spelling (a vowel under its breathing: `(ѧ҆̀)`)
            // is a whole alternative word too.
            let initial_vowel = paren
                .nfd()
                .nth(1)
                .is_some_and(|m| matches!(m, '\u{486}' | '\u{485}'));
            let whole = spaced
                && !paren.starts_with('-')
                && (first_letter(paren) == first_letter(&base) || initial_vowel);
            let alt = if whole {
                paren.to_string()
            } else {
                apply_ending(&base, paren.trim_start_matches('-'))
            };
            push(&mut out, &alt);
        }
        primary_hyphenated = Some(base);
    }
    out
}

fn push(out: &mut Vec<String>, hyphenated: &str) {
    let joined: String = hyphenated
        .chars()
        .filter(|c| *c != '-')
        .collect::<String>()
        .trim()
        .to_string();
    if !joined.is_empty() && !out.contains(&joined) {
        out.push(joined);
    }
}

/// Split on top-level `, ` (commas inside parentheses stay).
fn split_alternatives(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
                continue;
            }
            _ => {}
        }
        current.push(c);
    }
    parts.push(current);
    parts
}

/// `main` and every `(…)` group with whether a space preceded it.
fn split_parens(text: &str) -> (String, Vec<(bool, String)>) {
    let mut main = String::new();
    let mut parens = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('(') {
        let before = &rest[..open];
        main.push_str(before);
        let spaced = before.ends_with(' ');
        let after = &rest[open + 1..];
        let Some(close) = after.find(')') else {
            break;
        };
        parens.push((spaced, after[..close].to_string()));
        rest = &after[close + 1..];
    }
    main.push_str(rest);
    (main, parens)
}

fn apply_ending(primary: &str, ending: &str) -> String {
    // A hyphenated primary names its own stem boundary.
    if let Some(at) = primary.rfind('-') {
        return format!("{}{ending}", &primary[..=at]);
    }
    let ending_letters: Vec<char> = ending
        .chars()
        .filter(|c| !is_mark(*c) && *c != '-')
        .collect();
    // A multi-letter ending replaces the primary from the last occurrence of
    // its first letter (`творѧ̀, -ѧ́щ-ь`; `мꙋ̑дрыѧ (ыхъ)`); otherwise as many
    // final letters as the ending has (`и҆́мава, -ѣ`; `бы́хова (-ѣ)`).
    let skeleton: Vec<char> = primary.chars().collect();
    let anchor = ending_letters
        .first()
        .filter(|_| ending_letters.len() >= 2)
        .and_then(|first| skeleton.iter().rposition(|c| c == first));
    let kept: String = match anchor {
        Some(at) => skeleton[..at].iter().collect(),
        None => {
            let mut remaining = ending_letters.len();
            let mut kept: Vec<char> = Vec::new();
            for c in skeleton.iter().rev() {
                if remaining == 0 {
                    kept.push(*c);
                } else if !is_mark(*c) {
                    remaining -= 1;
                }
            }
            kept.reverse();
            kept.into_iter().collect()
        }
    };
    format!("{kept}{ending}")
}

fn first_letter(text: &str) -> Option<char> {
    strip_marks(text).chars().next()
}

fn is_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036f | 0x0483..=0x0489 | 0x2de0..=0x2dff | 0xfe20..=0xfe2f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_labels_cover_the_printed_vocabulary() {
        let l = parse_row_label("И.В.З.").expect("cases");
        assert_eq!(
            l.cases,
            [Case::Nominative, Case::Accusative, Case::Vocative]
        );
        let l = parse_row_label("Р.П").expect("trailing dot omitted");
        assert_eq!(l.cases, [Case::Genitive, Case::Locative]);
        let l = parse_row_label("2-е и 3-е").expect("persons");
        assert_eq!(l.persons, [Person::Second, Person::Third]);
        let l = parse_row_label("двойственное число").expect("number");
        assert_eq!(l.number, Some(Number::Dual));
        assert!(parse_row_label("основа").is_none());
        assert_eq!(
            tense_header("имперфект(преходящее)"),
            Some(TenseWord::Imperfect)
        );
        assert_eq!(tense_header("перфект(прошедшее совершенное)"), None);
        assert_eq!(
            gender_header("ж. и ср. р."),
            Some(vec![Gender::Feminine, Gender::Neuter])
        );
    }

    #[test]
    fn stray_closers_still_parse() {
        let cells = parse_row("<th>&nbsp;</th></td><th colspan=\"2\">Твердое склонение</th>");
        assert_eq!(cells.len(), 2);
        assert_eq!(cells[1].text, "Твердое склонение");
        assert_eq!(cells[1].colspan, 2);
    }

    #[test]
    fn headwords_become_accented_lemma_keys() {
        assert_eq!(lemma_key("ра́б-ъ").as_deref(), Some("ра́бъ"));
        assert_eq!(lemma_key("пис-а́-ти").as_deref(), Some("писа́ти"));
        assert_eq!(lemma_key("клѧ́-ти = клен-ꙋ́тъ").as_deref(), Some("клѧ́ти"));
        assert_eq!(lemma_key("и҆-тѝ — и҆д-ꙋ́тъ").as_deref(), Some("и҆тѝ"));
        assert_eq!(lemma_key("ѻ҆́чи (ѻ҆́цѣ)").as_deref(), Some("ѻ҆́чи"));
        assert_eq!(lemma_key("ᲂу҆́ши").as_deref(), Some("оу҆́ши"));
        assert_eq!(lemma_key("бж҃ї-й кра́-й"), None);
    }

    #[test]
    fn printed_variant_notation_expands() {
        let alts = |s: &str| alternatives(s);
        assert_eq!(alts("раб-ꙋ̀"), ["рабꙋ̀"]);
        assert_eq!(alts("мꙋ́др-ъ (-а)"), ["мꙋ́дръ", "мꙋ́дра"]);
        assert_eq!(alts("и҆́мава, -ѣ"), ["и҆́мава", "и҆́мавѣ"]);
        assert_eq!(alts("и҆́дева(вѣ)"), ["и҆́дева", "и҆́девѣ"]);
        assert_eq!(alts("бы́сть (бы̀)"), ["бы́сть", "бы̀"]);
        assert_eq!(alts("ѻ҆́чи (ѻ҆́цѣ)"), ["ѻ҆́чи", "ѻ҆́цѣ"]);
        assert_eq!(alts("мꙋдр-ѣ́е (-ѣ́йше)"), ["мꙋдрѣ́е", "мꙋдрѣ́йше"]);
        assert_eq!(alts("мнѣ̀, мѝ"), ["мнѣ̀", "мѝ"]);
        assert_eq!(alts("(н)е́мъ"), ["е́мъ", "не́мъ"]);
        assert_eq!(alts("творѧ̀, -ѧ́щ-ь"), ["творѧ̀", "творѧ́щь"]);
        assert_eq!(
            alts("пла́чива(вѣ), -ева(вѣ)"),
            ["пла́чива", "пла́чивѣ", "пла́чева", "пла́чевѣ"]
        );
        assert_eq!(alts("мꙋ̑др-ыѧ (ыхъ)"), ["мꙋ̑дрыѧ", "мꙋ̑дрыхъ"]);
    }
}
