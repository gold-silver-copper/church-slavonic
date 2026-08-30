//! The Kaikki/Wiktextract Old Church Slavonic dump: the entry schema, the
//! one-time filter into `data/intermediate/kaikki.jsonl`, and the mechanical
//! reading of an entry's inflection tables (which cell a form's tags name).
//! Editorial decisions — what is a lemma, which table is which, how an
//! unlabelled finite block is read — live in [`crate::extract`].

use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use church_slavonic_core::grammar::{Case, Gender, Number, Person, Tense};

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub word: String,
    pub pos: String,
    #[serde(default)]
    pub forms: Vec<FormEntry>,
    #[serde(default)]
    pub senses: Vec<Sense>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormEntry {
    pub form: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Sense {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub form_of: Vec<FormOf>,
}

/// The target of a `form-of` sense: the lemma the headword is a form of.
#[derive(Debug, Clone, Deserialize)]
pub struct FormOf {
    pub word: String,
}

/// Is this sense a `form-of` pointer with a named target?
pub fn is_form_of(sense: &Sense) -> bool {
    has(&sense.tags, "form-of") && !sense.form_of.is_empty()
}

/// The parts of speech the filter keeps, in the dump's own vocabulary.
pub const KEPT_POS: [&str; 4] = ["noun", "adj", "verb", "pron"];

/// Reduce the dump to the entries the extractor reads: kept parts of speech
/// with at least one inflection-table form. Lines are copied verbatim — no
/// parsing decisions happen here.
pub fn filter(dump: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
    let reader = BufReader::new(File::open(dump)?);
    let mut writer = BufWriter::new(File::create(out)?);
    let (mut kept, mut total) = (0usize, 0usize);
    for line in reader.lines() {
        let line = line?;
        total += 1;
        let Ok(entry) = serde_json::from_str::<Entry>(&line) else {
            continue;
        };
        if KEPT_POS.contains(&entry.pos.as_str())
            && (entry.forms.iter().any(is_table_form) || entry.senses.iter().any(is_form_of))
        {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
            kept += 1;
        }
    }
    writer.flush()?;
    println!(
        "Filtered Kaikki dump: kept {kept} of {total} entries into {}",
        out.display()
    );
    Ok(())
}

/// A dump pass may tolerate at most this fraction of lines failing to
/// deserialize; a spike means wiktextract changed its schema.
const MAX_PARSE_FAILURE_RATIO: f64 = 0.01;

/// Read every entry of a (filtered) dump. Parse failures are counted and
/// reported, never silent; past the threshold the pass hard-errors.
pub fn read(path: &Path) -> Result<Vec<Entry>, Box<dyn Error>> {
    let reader = BufReader::new(File::open(path)?);
    let mut entries = Vec::new();
    let (mut failed, mut total) = (0usize, 0usize);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        total += 1;
        match serde_json::from_str::<Entry>(&line) {
            Ok(entry) => entries.push(entry),
            Err(_) => failed += 1,
        }
    }
    if failed > 0 {
        let ratio = failed as f64 / total.max(1) as f64;
        if ratio > MAX_PARSE_FAILURE_RATIO {
            return Err(format!(
                "{failed}/{total} lines of {} failed to parse — the wiktextract schema changed?",
                path.display()
            )
            .into());
        }
        eprintln!(
            "warning: {failed}/{total} lines of {} failed to parse",
            path.display()
        );
    }
    Ok(entries)
}

pub fn is_table_form(form: &FormEntry) -> bool {
    matches!(
        form.source.as_deref(),
        Some("declension") | Some("conjugation")
    )
}

/// A cell of an inflection table is a real form unless it is a template
/// sentinel or a printed dash.
pub fn is_sentinel(form: &FormEntry) -> bool {
    matches!(form.form.as_str(), "-" | "—" | "no-table-tags")
        || form
            .tags
            .iter()
            .any(|t| matches!(t.as_str(), "table-tags" | "class" | "inflection-template"))
}

/// One inflection table of an entry: the `table-tags` banner value and the
/// forms that follow it, in listing order.
pub struct Table<'a> {
    pub banner: String,
    pub forms: Vec<&'a FormEntry>,
}

/// Split an entry's table forms at each `table-tags` banner.
pub fn tables(entry: &Entry) -> Vec<Table<'_>> {
    let mut out: Vec<Table<'_>> = Vec::new();
    for form in entry.forms.iter().filter(|f| is_table_form(f)) {
        if form.tags.iter().any(|t| t == "table-tags") {
            out.push(Table {
                banner: form.form.clone(),
                forms: Vec::new(),
            });
            continue;
        }
        if is_sentinel(form) && form.form != "-" {
            continue;
        }
        match out.last_mut() {
            Some(table) => table.forms.push(form),
            None => out.push(Table {
                banner: String::new(),
                forms: vec![form],
            }),
        }
    }
    out
}

/// Every case the tags name (a `form-of` sense may cover several cells:
/// "nominative/accusative dual").
pub fn cases(tags: &[String]) -> Vec<Case> {
    tags.iter()
        .filter_map(|t| match t.as_str() {
            "nominative" => Some(Case::Nominative),
            "genitive" => Some(Case::Genitive),
            "dative" => Some(Case::Dative),
            "accusative" => Some(Case::Accusative),
            "instrumental" => Some(Case::Instrumental),
            "locative" => Some(Case::Locative),
            "vocative" => Some(Case::Vocative),
            _ => None,
        })
        .collect()
}

pub fn numbers(tags: &[String]) -> Vec<Number> {
    tags.iter()
        .filter_map(|t| match t.as_str() {
            "singular" => Some(Number::Singular),
            "dual" => Some(Number::Dual),
            "plural" => Some(Number::Plural),
            _ => None,
        })
        .collect()
}

pub fn persons(tags: &[String]) -> Vec<Person> {
    tags.iter()
        .filter_map(|t| match t.as_str() {
            "first-person" => Some(Person::First),
            "second-person" => Some(Person::Second),
            "third-person" => Some(Person::Third),
            _ => None,
        })
        .collect()
}

pub fn has(tags: &[String], wanted: &str) -> bool {
    tags.iter().any(|t| t == wanted)
}

pub fn case(tags: &[String]) -> Option<Case> {
    let mut found = None;
    for t in tags {
        let c = match t.as_str() {
            "nominative" => Case::Nominative,
            "genitive" => Case::Genitive,
            "dative" => Case::Dative,
            "accusative" => Case::Accusative,
            "instrumental" => Case::Instrumental,
            "locative" => Case::Locative,
            "vocative" => Case::Vocative,
            _ => continue,
        };
        if found.is_some() {
            return None;
        }
        found = Some(c);
    }
    found
}

pub fn number(tags: &[String]) -> Option<Number> {
    let mut found = None;
    for t in tags {
        let n = match t.as_str() {
            "singular" => Number::Singular,
            "dual" => Number::Dual,
            "plural" => Number::Plural,
            _ => continue,
        };
        if found.is_some() {
            return None;
        }
        found = Some(n);
    }
    found
}

pub fn person(tags: &[String]) -> Option<Person> {
    let mut found = None;
    for t in tags {
        let p = match t.as_str() {
            "first-person" => Person::First,
            "second-person" => Person::Second,
            "third-person" => Person::Third,
            _ => continue,
        };
        if found.is_some() {
            return None;
        }
        found = Some(p);
    }
    found
}

pub fn tense(tags: &[String]) -> Option<Tense> {
    let mut found = None;
    for t in tags {
        let x = match t.as_str() {
            "present" => Tense::Present,
            "imperfect" => Tense::Imperfect,
            "aorist" => Tense::Aorist,
            _ => continue,
        };
        if found.is_some() {
            return None;
        }
        found = Some(x);
    }
    found
}

/// Every gender named by the tags (a shared cell is tagged with all of them).
pub fn genders(tags: &[String]) -> Vec<Gender> {
    let mut out = Vec::new();
    for t in tags {
        match t.as_str() {
            "masculine" => out.push(Gender::Masculine),
            "feminine" => out.push(Gender::Feminine),
            "neuter" => out.push(Gender::Neuter),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(json: &str) -> Entry {
        serde_json::from_str(json).expect("fixture parses")
    }

    #[test]
    fn tables_split_at_banners_and_keep_printed_dashes() {
        let e = entry(
            r#"{"word":"x","pos":"verb","forms":[
              {"form":"x","tags":["romanization"]},
              {"form":"no-table-tags","tags":["table-tags"],"source":"conjugation"},
              {"form":"top","tags":["inflection-template"],"source":"conjugation"},
              {"form":"a","tags":["present","singular"],"source":"conjugation"},
              {"form":"-","tags":["imperative","singular"],"source":"conjugation"},
              {"form":"present","tags":["table-tags"],"source":"conjugation"},
              {"form":"b","tags":["nominative","short-form"],"source":"conjugation"}
            ]}"#,
        );
        let t = tables(&e);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].banner, "no-table-tags");
        let forms: Vec<&str> = t[0].forms.iter().map(|f| f.form.as_str()).collect();
        assert_eq!(forms, ["a", "-"]);
        assert_eq!(t[1].banner, "present");
        assert_eq!(t[1].forms[0].form, "b");
    }

    #[test]
    fn tag_readers_reject_ambiguous_tags() {
        let tags = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(case(&tags(&["dual", "genitive"])), Some(Case::Genitive));
        assert_eq!(case(&tags(&["genitive", "locative"])), None);
        assert_eq!(number(&tags(&["dual", "genitive"])), Some(Number::Dual));
        assert_eq!(
            genders(&tags(&["masculine", "neuter", "dative"])),
            [Gender::Masculine, Gender::Neuter]
        );
        assert_eq!(person(&tags(&["first-person"])), Some(Person::First));
        assert_eq!(tense(&tags(&["aorist", "plural"])), Some(Tense::Aorist));
    }
}
