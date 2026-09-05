//! The Russian Wiktionary's Church Slavonic section via Kaikki/Wiktextract:
//! the one-time filter into `data/intermediate/ruwiktionary.jsonl` and the
//! reading of an entry's structured forms. The dump shares the English
//! Kaikki schema ([`crate::kaikki::Entry`]), but of its 7,249 entries only
//! 39 carry a `forms` list (Wiktextract structured no other table), so the
//! source is a small accented Synodal supplement to Polyakov and Alypy —
//! which entry becomes which lemma is [`crate::extract`]'s decision.
//!
//! # Form tags (as found in the data)
//!
//! Nouns: `singular`/`dual`/`plural` with `nominative`… `vocative`. Verbs:
//! `first-person`/`second-person`/`third-person` with `singular`/`plural`
//! and `present` (or `future`, a perfective's present) or `imperative`; the
//! dual is printed with a gender note instead of a number tag (`(м.)`,
//! `(ж.)`) and the `perfect` rows are the periphrastic l-participle
//! paradigm (`писа́лъ є҆́смь`), outside the schema. A cell printing two forms
//! joins them with `/`; the form is the accented Synodal print with the
//! breathing.

use crate::kaikki::{self, Entry, FormEntry};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// Keep the entries with structured forms in a part of speech the tables
/// cover. Lines are copied verbatim.
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
        if kaikki::KEPT_POS.contains(&entry.pos.as_str()) && !entry.forms.is_empty() {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
            kept += 1;
        }
    }
    writer.flush()?;
    println!(
        "Filtered ru.wiktionary dump: kept {kept} of {total} entries into {}",
        out.display()
    );
    Ok(())
}

pub fn read(path: &Path) -> Result<Vec<Entry>, Box<dyn Error>> {
    kaikki::read(path)
}

/// The printed alternatives of a cell (`ча̑дъ/ча́дѡвъ`), the gender note of a
/// dual row stripped (`пи́шева (м.)`); a multi-word form (a periphrasis) is
/// nothing.
pub fn alternatives(form: &FormEntry) -> Vec<String> {
    let text = match form.form.split_once(" (") {
        Some((main, _)) => main,
        None => form.form.as_str(),
    };
    if text.contains(' ') {
        return Vec::new();
    }
    text.split('/')
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .map(str::to_string)
        .collect()
}

/// The dual rows carry no number tag: `(м.)`/`(ж.)` after the form is one.
pub fn is_dual_note(form: &FormEntry) -> bool {
    form.form.ends_with("(м.)") || form.form.ends_with("(ж.)") || form.form.ends_with("(ср.)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(text: &str, tags: &[&str]) -> FormEntry {
        FormEntry {
            form: text.to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            source: None,
        }
    }

    #[test]
    fn alternatives_split_on_the_slash_and_drop_periphrases() {
        assert_eq!(
            alternatives(&form("ча̑дъ/ча́дѡвъ", &["plural", "genitive"])),
            ["ча̑дъ", "ча́дѡвъ"]
        );
        assert_eq!(alternatives(&form("пи́шева (м.)", &["present"])), ["пи́шева"]);
        assert!(is_dual_note(&form("пи́шева (м.)", &["present"])));
        assert!(alternatives(&form("писа́лъ є҆́смь (м.)", &["perfect"])).is_empty());
    }
}
