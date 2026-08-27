//! Polyakov's corpus-based *Grammatical dictionary of Church Slavonic*
//! (tagged web edition): the HTML mechanics that turn its entry paragraphs
//! into records, the one-time filter into `data/intermediate/polyakov.jsonl`,
//! and the reading of a form's tag string into typed features. Which entries
//! and which cells reach the tables is [`crate::extract`]'s decision.
//!
//! # The artifact
//!
//! 43 UTF-8 pages: `flexslav.htm` (the paradigm-class legend — `N1t`, `N2i`,
//! `N3c`, `N41`, `N5en`, `A1t`, `A2j`, `V11a`, `V21n`, `Vbyt`, `PNja`… with an
//! exemplar paradigm per class), `index1.htm`/`indexnav.htm`, and the letter
//! pages `1/*.htm` (direct alphabetical order) and `2/*.htm` (reverse order — a
//! subset of the same entries). Every `<p><big>` paragraph on every page is an
//! entry; byte-identical paragraphs are deduplicated, so the reverse index adds
//! nothing. Each entry is one lexeme: a headword printed as the Synodal
//! citation form (accented), its lexeme tags, its paradigm class, its corpus
//! frequency, then one line per attested form with that form's analysis and
//! frequency:
//!
//! ```text
//! <p><big><a href="?req=…">аарѡ́нъ</a></big>  <i>S,m,anim,persn</i>  <a href="…flexslav.htm#N1t">N1t</a> (369)<br>
//! <a href="?req=…">аарѡ́на</a> <i>sg,gen/acc</i> (90)<br> …</p>
//! ```
//!
//! # Tag vocabulary (as found in the data)
//!
//! Lexeme tags: part of speech `S` (noun; a dozen entries print `N`), `A`,
//! `V`, `SPRO`, `APRO`, `ADVPRO`, `NUM`, `ANUM`, `ADV`, `PR`, `CONJ`, `PART`,
//! `INTJ`, `PRED`, `PARENTH`; noun gender `m`/`f`/`n` (`m/f` when either),
//! `anim`/`inan`, name classes `persn`/`topn`/`famn`/`patrn`, `pl` (plurale
//! tantum); adjective `poss`, `comp`, `brev`; verb aspect `pf`/`ipf`
//! (`pf/ipf`), `tran`/`intr`, `med` (the `-ся` middle). A few entries carry
//! two alternatives separated by `|` or a trailing `?`.
//!
//! Form tags, comma-separated: number `sg`/`du`/`pl`; case `nom`/`gen`/`dat`/
//! `acc`/`ins`/`loc`/`voc`; gender `m`/`f`/`n`; adjective and participle series
//! `brev` (short) / `plen` (long); degree `comp`; verb `inf`, mood `indic`/
//! `imper`, tense `praes`/`fut`/`aor`/`imperf`/`perf`/`praet`, `partcp` with
//! voice `act`/`pass`, person `1p`/`2p`/`3p`; `clit` (an enclitic pronoun).
//! `9^` and `9` (also glued to a case, `gen9`) mark a spelling under a titlo
//! and carry no cell information. An empty `<i></i>` is an unanalysed form.
//!
//! A slash joins alternatives inside one dimension (`gen/acc`, `m/n`, `2p/3p`,
//! `plen/brev`): the form attests every cell of the expansion; `|` separates
//! whole alternative analyses; a second case in one analysis (`nom,loc`) is a
//! second cell. [`expand`] turns a tag string into that list of flat tag sets
//! and [`features`] reads one set into typed grammar.

use church_slavonic_core::grammar::{Case, Gender, Number, Person};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Intermediate schema: one JSON line per entry.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The headword as printed (accented).
    pub lemma: String,
    /// The lexeme tags (`S,m,anim,persn`), split on commas.
    pub tags: Vec<String>,
    /// The paradigm class of `flexslav.htm` (`N1t`), or empty.
    pub class: String,
    /// The lexeme's corpus frequency.
    pub count: u64,
    pub forms: Vec<FormEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormEntry {
    /// The form as printed (accented).
    pub form: String,
    /// The analysis as printed (`brev,sg,f,nom|brev,sg,m/n,gen/acc`).
    pub tags: String,
    pub count: u64,
    /// The expanded analyses: one flat tag set per attested cell group
    /// ([`expand`]); empty for an unanalysed form.
    pub cells: Vec<Vec<String>>,
}

/// Reduce the pages to one record per distinct entry paragraph.
pub fn filter(source_dir: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
    let mut pages = Vec::new();
    collect_pages(source_dir, &mut pages)?;
    pages.sort();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut writer = BufWriter::new(fs::File::create(out)?);
    let (mut printed, mut kept, mut forms, mut unanalysed) = (0usize, 0usize, 0usize, 0usize);
    for page in &pages {
        let html = fs::read_to_string(page)?;
        for paragraph in paragraphs(&html) {
            printed += 1;
            if !seen.insert(paragraph.to_string()) {
                continue;
            }
            let Some(entry) = parse_entry(paragraph) else {
                continue;
            };
            kept += 1;
            forms += entry.forms.len();
            unanalysed += entry.forms.iter().filter(|f| f.cells.is_empty()).count();
            serde_json::to_writer(&mut writer, &entry)?;
            writer.write_all(b"\n")?;
        }
    }
    writer.flush()?;
    println!(
        "Filtered Polyakov dictionary: {kept} distinct entries ({printed} printed on {} pages), {forms} forms ({unanalysed} unanalysed) into {}",
        pages.len(),
        out.display()
    );
    Ok(())
}

fn collect_pages(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_pages(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "htm" || e == "html") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn read(path: &Path) -> Result<Vec<Entry>, Box<dyn Error>> {
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

/// Every `<p><big>…</p>` paragraph of a page.
fn paragraphs(html: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<p><big>") {
        let body = &rest[start..];
        let Some(end) = body.find("</p>") else {
            break;
        };
        out.push(&body[..end]);
        rest = &body[end + 4..];
    }
    out
}

/// The text of the first `<tag>…</tag>` element in `s`, with the remainder
/// after its close.
fn element<'a>(s: &'a str, tag: &str) -> Option<(&'a str, &'a str)> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let at = s.find(&open)?;
    let after_open = &s[at..];
    let text_start = after_open.find('>')? + 1;
    let inner = &after_open[text_start..];
    let end = inner.find(&close)?;
    Some((inner[..end].trim(), &inner[end + close.len()..]))
}

/// `(n)` — the last parenthesised integer in `s`.
fn frequency(s: &str) -> Option<u64> {
    let open = s.rfind('(')?;
    let close = s[open..].find(')')? + open;
    s[open + 1..close].trim().parse().ok()
}

pub fn parse_entry(paragraph: &str) -> Option<Entry> {
    let mut lines = paragraph.split("<br>");
    let head = lines.next()?;
    let (lemma, rest) = element(head, "a")?;
    let (tags, rest) = element(rest, "i")?;
    let class = rest
        .split_once('#')
        .and_then(|(_, after)| after.split('"').next())
        .unwrap_or("")
        .trim()
        .to_string();
    let count = frequency(rest)?;
    let mut forms = Vec::new();
    for line in lines {
        let Some((form, rest)) = element(line, "a") else {
            continue;
        };
        let Some((tags, rest)) = element(rest, "i") else {
            continue;
        };
        let Some(count) = frequency(rest) else {
            continue;
        };
        forms.push(FormEntry {
            form: form.to_string(),
            tags: tags.to_string(),
            count,
            cells: expand(tags),
        });
    }
    Some(Entry {
        lemma: lemma.to_string(),
        tags: tags
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect(),
        class,
        count,
        forms,
    })
}

// ---------------------------------------------------------------------------
// Tag vocabulary
// ---------------------------------------------------------------------------

/// Expand a printed analysis into flat tag sets: `|` separates analyses, a
/// slash inside a tag multiplies the set by its alternatives. An empty
/// analysis expands to nothing.
pub fn expand(tags: &str) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    for analysis in tags.split('|') {
        let analysis = analysis.trim();
        if analysis.is_empty() {
            continue;
        }
        let mut sets: Vec<Vec<String>> = vec![Vec::new()];
        for token in analysis.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            let alternatives: Vec<&str> = token.split('/').collect();
            let mut next = Vec::with_capacity(sets.len() * alternatives.len());
            for set in &sets {
                for alt in &alternatives {
                    let mut grown = set.clone();
                    grown.push((*alt).to_string());
                    next.push(grown);
                }
            }
            sets = next;
        }
        out.extend(sets);
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Series {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TenseTag {
    Present,
    Future,
    Aorist,
    Imperfect,
    /// The `perf` tag: the l-participle (`бы́лъ`).
    Perfect,
    /// The `praet` tag: the past active participle's tense.
    Past,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Indicative,
    Imperative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Voice {
    Active,
    Passive,
}

/// One flat tag set read into grammar. Every field is what the set names,
/// nothing is inferred.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Features {
    pub number: Option<Number>,
    pub cases: Vec<Case>,
    pub gender: Option<Gender>,
    pub person: Option<Person>,
    pub series: Option<Series>,
    pub comparative: bool,
    pub tense: Option<TenseTag>,
    pub mood: Option<Mood>,
    pub voice: Option<Voice>,
    pub participle: bool,
    pub infinitive: bool,
    pub clitic: bool,
    /// Tags outside the vocabulary above (none in the pinned artifact; kept
    /// so a revision cannot silently pass a new dimension).
    pub unknown: Vec<String>,
}

pub fn features(tags: &[String]) -> Features {
    let mut f = Features::default();
    for tag in tags {
        // `9`/`9^` mark a spelling under a titlo, alone or glued to a case.
        let tag = tag.trim_end_matches('^').trim_end_matches('9');
        match tag {
            "" => {}
            "sg" => f.number = Some(Number::Singular),
            "du" => f.number = Some(Number::Dual),
            "pl" => f.number = Some(Number::Plural),
            "nom" => f.cases.push(Case::Nominative),
            "gen" => f.cases.push(Case::Genitive),
            "dat" => f.cases.push(Case::Dative),
            "acc" => f.cases.push(Case::Accusative),
            "ins" => f.cases.push(Case::Instrumental),
            "loc" => f.cases.push(Case::Locative),
            "voc" => f.cases.push(Case::Vocative),
            "m" => f.gender = Some(Gender::Masculine),
            "f" => f.gender = Some(Gender::Feminine),
            "n" => f.gender = Some(Gender::Neuter),
            "1p" => f.person = Some(Person::First),
            "2p" => f.person = Some(Person::Second),
            "3p" => f.person = Some(Person::Third),
            "brev" => f.series = Some(Series::Short),
            "plen" => f.series = Some(Series::Long),
            "comp" => f.comparative = true,
            "praes" => f.tense = Some(TenseTag::Present),
            "fut" => f.tense = Some(TenseTag::Future),
            "aor" => f.tense = Some(TenseTag::Aorist),
            "imperf" => f.tense = Some(TenseTag::Imperfect),
            "perf" => f.tense = Some(TenseTag::Perfect),
            "praet" => f.tense = Some(TenseTag::Past),
            "indic" => f.mood = Some(Mood::Indicative),
            "imper" => f.mood = Some(Mood::Imperative),
            "act" => f.voice = Some(Voice::Active),
            "pass" => f.voice = Some(Voice::Passive),
            "partcp" => f.participle = true,
            "inf" => f.infinitive = true,
            "clit" => f.clitic = true,
            other => f.unknown.push(other.to_string()),
        }
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: &str = concat!(
        r#"<p><big><a href="?req=аарѡнъ">аарѡ́нъ</a></big>  <i>S,m,anim,persn</i>  "#,
        r#"<a href="http://dic.feb-web.ru/slavonic/dicgram/flexslav.htm#N1t">N1t</a> (369)<br>"#,
        r#"<a href="?req=аарѡнъ">аарѡ́нъ</a> <i>sg,nom</i> (180)<br> "#,
        r#"<a href="?req=аарѡна">аарѡ́на</a> <i>sg,gen/acc</i> (90)<br> "#,
        r#"<a href="?req=аарѡнѣхъ">аарѡ́нѣхъ</a> <i></i> (2)</p>"#
    );

    #[test]
    fn an_entry_paragraph_parses_into_a_record() {
        let html = format!(
            "<title>x</title>\n{ENTRY}\n<p><big><a href=\"?req=а\">а</a></big>  <i>CONJ</i>  (2463)<br><a href=\"?req=а\">а</a> <i></i> (2463)</p>"
        );
        let paragraphs = paragraphs(&html);
        assert_eq!(paragraphs.len(), 2);
        let e = parse_entry(paragraphs[0]).expect("parses");
        assert_eq!(e.lemma, "аарѡ́нъ");
        assert_eq!(e.tags, ["S", "m", "anim", "persn"]);
        assert_eq!(e.class, "N1t");
        assert_eq!(e.count, 369);
        assert_eq!(e.forms.len(), 3);
        assert_eq!(e.forms[1].form, "аарѡ́на");
        assert_eq!(e.forms[1].count, 90);
        assert_eq!(e.forms[1].cells, [["sg", "gen"], ["sg", "acc"]]);
        assert!(e.forms[2].cells.is_empty());
        let conj = parse_entry(paragraphs[1]).expect("parses");
        assert_eq!(conj.class, "");
        assert_eq!(conj.tags, ["CONJ"]);
    }

    #[test]
    fn slashes_multiply_and_bars_separate_analyses() {
        let cells = expand("brev,sg,f,nom|brev,sg,m/n,gen/acc");
        assert_eq!(cells.len(), 5);
        assert_eq!(cells[0], ["brev", "sg", "f", "nom"]);
        assert_eq!(cells[4], ["brev", "sg", "n", "acc"]);
        assert!(expand("").is_empty());
        let f = features(&cells[4]);
        assert_eq!(f.series, Some(Series::Short));
        assert_eq!(f.number, Some(Number::Singular));
        assert_eq!(f.gender, Some(Gender::Neuter));
        assert_eq!(f.cases, [Case::Accusative]);
    }

    #[test]
    fn titlo_markers_carry_no_cell_and_two_cases_are_two_cells() {
        let f = features(&expand("9^,brev,sg,m/n,nom,loc")[0]);
        assert!(f.unknown.is_empty());
        assert_eq!(f.cases, [Case::Nominative, Case::Locative]);
        let f = features(&expand("sg,gen9")[0]);
        assert_eq!(f.cases, [Case::Genitive]);
        let f = features(&expand("partcp,praet,act,brev,sg,m,nom")[0]);
        assert!(f.participle);
        assert_eq!(f.tense, Some(TenseTag::Past));
        assert_eq!(f.voice, Some(Voice::Active));
        let f = features(&expand("indic,fut,du,2p/3p")[1]);
        assert_eq!(f.person, Some(Person::Third));
        assert_eq!(f.mood, Some(Mood::Indicative));
        assert_eq!(features(&expand("xyz")[0]).unknown, ["xyz"]);
    }
}
