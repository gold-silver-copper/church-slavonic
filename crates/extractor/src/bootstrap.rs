//! Table emission and parsing.
//!
//! [`generate_tables`] is the single candidate -> PHF path, byte-deterministic
//! because `check-registry` and the accuracy harness both read the committed
//! tables back with [`parse_phf_pairs`]. There is no lockfile: the generated
//! tables ARE the committed artifact.

use crate::assign::split_key;
use crate::cells::{PRONOUN_KEY, Pos, recension_of_tag, rule_matches};
use crate::extract::{Table, Tables};
use crate::file_generation::write_phf;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

/// A parsed table row: a key and its cells.
pub type KeyForms = (String, Vec<String>);

/// Extract `(key, cells)` pairs from a generated `*_phf.rs` file by reading
/// each `"key" => &[(cell, "form"), …]` line back into a dense row of the
/// map's arity (the file header states it). This is the emitter-independent
/// reader `check-registry` and the accuracy harness use.
pub fn parse_phf_pairs(path: impl AsRef<Path>) -> Result<Vec<KeyForms>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let arity: usize = text
        .lines()
        .find_map(|l| l.split("pairs of a ").nth(1)?.split("-cell").next())
        .ok_or("no arity in the table header")?
        .parse()?;
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('"') || !line.contains("=>") {
            continue;
        }
        let fields = quoted_fields(line);
        let Some((key, forms)) = fields.split_first() else {
            continue;
        };
        let mut cells = vec![String::new(); arity];
        let indices = line
            .split("=>")
            .nth(1)
            .unwrap_or("")
            .split('(')
            .skip(1)
            .filter_map(|part| part.split(',').next()?.trim().parse::<usize>().ok());
        for (i, form) in indices.zip(forms) {
            if i >= arity {
                return Err(format!("cell {i} of `{key}` is past the arity {arity}").into());
            }
            cells[i] = form.clone();
        }
        out.push((key.clone(), cells));
    }
    Ok(out)
}

fn quoted_fields(line: &str) -> Vec<String> {
    line.split('"')
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, s)| s.to_string())
        .collect()
}

/// Emit the four PHF tables. The single source-of-truth -> tables path; a
/// duplicate key is refused here rather than surfacing as an opaque
/// `phf_map!` compile error downstream.
pub fn generate_tables(tables: &Tables, generated_dir: &Path) -> Result<(), Box<dyn Error>> {
    for pos in Pos::ALL {
        let table = tables.get(pos);
        reject_duplicate_keys(table, pos)?;
        let rows: Vec<KeyForms> = table
            .iter()
            .map(|(k, f)| {
                let mut cells = f.clone();
                cells.resize(pos.arity(), String::new());
                (k.clone(), cells)
            })
            .collect();
        write_phf(pos, rows, generated_dir.join(pos.file_name()))?;
    }
    Ok(())
}

fn reject_duplicate_keys(table: &Table, pos: Pos) -> Result<(), Box<dyn Error>> {
    let mut seen: HashSet<&str> = HashSet::new();
    let dups: Vec<&str> = table
        .iter()
        .filter_map(|(k, _)| (!seen.insert(k)).then_some(k.as_str()))
        .collect();
    if dups.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "refusing to generate the {} table — duplicate key(s): {}",
            pos.label(),
            dups.join(", ")
        )
        .into())
    }
}

/// The source-free consistency gate behind `cargo xtask check-registry`. Reads
/// the committed PHF tables back with [`parse_phf_pairs`] and verifies, WITHOUT
/// any lockfile or source:
/// - keys are well-formed and unique (`<ocs|syn>:<lemma>` or
///   `<ocs|syn>:<lemma>_<n>` with `n >= 2`; the pronoun map's lemma is
///   `personal`);
/// - every row has the right arity and at least one non-empty cell;
/// - rule/table layering holds — no cell may equal the regular rule's answer
///   for its lemma (the generator blanks such cells; one that survives means a
///   core rule changed without regenerating), except in a `_n` row where the
///   bare row holds a different form at that cell (the runtime reads a `_n`
///   blank from the bare row, so the rule's form must be spelled out).
///
/// It does NOT verify that a row's VALUES are correct — those are attested
/// data, not derivable without the sources; `cargo xtask accuracy` is the
/// authoritative value check. Returns human-readable violations; empty = OK.
pub fn audit_tables(generated_dir: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut v = Vec::new();
    for pos in Pos::ALL {
        let pairs = parse_phf_pairs(generated_dir.join(pos.file_name()))?;
        if pairs.is_empty() {
            v.push(format!(
                "{}: no table entries parsed (missing or malformed table?)",
                pos.file_name()
            ));
            continue;
        }
        audit_rows(&mut v, pos, &pairs);
    }
    Ok(v)
}

/// The per-row structural + layering checks, separated from file I/O so they
/// are unit-testable with in-memory rows.
fn audit_rows(v: &mut Vec<String>, pos: Pos, pairs: &[KeyForms]) {
    let name = pos.file_name();
    let bare_rows: std::collections::HashMap<&str, &Vec<String>> = pairs
        .iter()
        .filter(|(k, _)| {
            k.split_once(':')
                .is_some_and(|(_, l)| split_key(l).is_none())
        })
        .map(|(k, c)| (k.as_str(), c))
        .collect();
    let mut seen: HashSet<&str> = HashSet::new();
    for (key, cells) in pairs {
        if !seen.insert(key) {
            v.push(format!("{name}: duplicate key `{key}`"));
        }
        if cells.len() != pos.arity() {
            v.push(format!(
                "{name}: key `{key}` has {} cell(s), expected {}",
                cells.len(),
                pos.arity()
            ));
            continue;
        }
        if cells.iter().all(|c| c.is_empty()) {
            v.push(format!("{name}: key `{key}` has no attested cell"));
        }
        let Some((tag, lemma_key)) = key.split_once(':') else {
            v.push(format!("{name}: key `{key}` lacks a recension prefix"));
            continue;
        };
        let Some(recension) = recension_of_tag(tag) else {
            v.push(format!("{name}: key `{key}` has unknown recension `{tag}`"));
            continue;
        };
        let lemma = match split_key(lemma_key) {
            Some((_, suffix)) if suffix < 2 => {
                v.push(format!(
                    "{name}: numbered key `{key}` has suffix {suffix} (numbered keys start at 2)"
                ));
                continue;
            }
            Some((base, _)) => base,
            None => lemma_key,
        };
        if lemma.is_empty() || lemma.contains(':') || lemma.contains(' ') {
            v.push(format!("{name}: key `{key}` has a malformed lemma"));
            continue;
        }
        if pos == Pos::Pronoun && lemma != PRONOUN_KEY {
            v.push(format!(
                "{name}: key `{key}` is not the lemma-less `{PRONOUN_KEY}` row"
            ));
            continue;
        }
        let predicted = pos.predict(lemma, &recension);
        let shadowing = (lemma != lemma_key)
            .then(|| bare_rows.get(format!("{tag}:{lemma}").as_str()).copied())
            .flatten();
        for (i, cell) in cells.iter().enumerate() {
            let shadowed = shadowing.is_some_and(|bare| !bare[i].is_empty() && bare[i] != *cell);
            if !cell.is_empty() && !shadowed && rule_matches(&recension, cell, &predicted[i]) {
                v.push(format!(
                    "{name}: key `{key}` cell {i} `{cell}` equals the regular-rule prediction — dead weight; \
                     a core rule changed without regenerating (`cargo xtask refresh-data`)"
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::noun_cell;
    use church_slavonic_core::grammar::{Case, Number};
    use std::path::PathBuf;

    fn generated_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../church-slavonic/generated")
    }

    /// The emitter is a stable round-trip: parsing the committed tables and
    /// re-emitting them must reproduce them byte-for-byte.
    #[test]
    fn tables_round_trip_committed_output_byte_for_byte() {
        let gen_dir = generated_dir();
        let mut tables = Tables::default();
        for pos in Pos::ALL {
            let rows = parse_phf_pairs(gen_dir.join(pos.file_name())).expect("parses");
            match pos {
                Pos::Noun => tables.noun = rows,
                Pos::Adj => tables.adj = rows,
                Pos::Verb => tables.verb = rows,
                Pos::Pronoun => tables.pronoun = rows,
            }
        }
        let tmp = std::env::temp_dir().join(format!("cs_tables_roundtrip_{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("tmp dir");
        generate_tables(&tables, &tmp).expect("emits");
        for pos in Pos::ALL {
            let committed = fs::read(gen_dir.join(pos.file_name())).expect("committed");
            let regenerated = fs::read(tmp.join(pos.file_name())).expect("regenerated");
            assert!(
                committed == regenerated,
                "{} differs after a parse -> emit round-trip",
                pos.file_name()
            );
        }
    }

    fn row(key: &str, cells: &[(usize, &str)]) -> KeyForms {
        let mut forms = vec![String::new(); Pos::Noun.arity()];
        for (i, f) in cells {
            forms[*i] = f.to_string();
        }
        (key.to_string(), forms)
    }

    #[test]
    fn audit_rows_flags_structural_and_layering_problems() {
        let dat = noun_cell(&Case::Dative, &Number::Singular);
        let genitive = noun_cell(&Case::Genitive, &Number::Singular);
        // Clean: an irregular dative, plus a numbered variant.
        let mut out = Vec::new();
        audit_rows(
            &mut out,
            Pos::Noun,
            &[
                row("ocs:рабъ", &[(dat, "рабови")]),
                row("ocs:рабъ_2", &[(dat, "рабъви")]),
            ],
        );
        assert!(out.is_empty(), "clean table flagged: {out:?}");

        // A rule-equal cell is dead weight.
        let mut out = Vec::new();
        audit_rows(
            &mut out,
            Pos::Noun,
            &[row("ocs:рабъ", &[(genitive, "раба")])],
        );
        assert!(out.iter().any(|m| m.contains("dead weight")), "{out:?}");

        // Duplicate key, wrong arity, empty row, suffix < 2, bad prefix.
        let mut out = Vec::new();
        audit_rows(
            &mut out,
            Pos::Noun,
            &[row("ocs:x", &[(dat, "а")]), row("ocs:x", &[(dat, "б")])],
        );
        assert!(out.iter().any(|m| m.contains("duplicate key")), "{out:?}");
        let mut out = Vec::new();
        audit_rows(
            &mut out,
            Pos::Noun,
            &[("ocs:x".to_string(), vec!["а".to_string()])],
        );
        assert!(out.iter().any(|m| m.contains("expected 21")), "{out:?}");
        let mut out = Vec::new();
        audit_rows(&mut out, Pos::Noun, &[row("ocs:x", &[])]);
        assert!(
            out.iter().any(|m| m.contains("no attested cell")),
            "{out:?}"
        );
        let mut out = Vec::new();
        audit_rows(&mut out, Pos::Noun, &[row("ocs:x_1", &[(dat, "а")])]);
        assert!(out.iter().any(|m| m.contains("suffix 1")), "{out:?}");
        let mut out = Vec::new();
        audit_rows(&mut out, Pos::Noun, &[row("x", &[(dat, "а")])]);
        assert!(
            out.iter().any(|m| m.contains("recension prefix")),
            "{out:?}"
        );
    }

    #[test]
    fn parse_phf_pairs_restores_the_blank_cells() {
        let dir = std::env::temp_dir().join(format!("cs_parse_phf_{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("noun_phf.rs");
        fs::write(
            &path,
            "use phf::phf_map;\n/// pairs of a 3-cell row\npub static NOUN_MAP: phf::Map<&'static str, &'static [(u8, &'static str)]> = phf_map! {\n    \
             \"ocs:x_2\" => &[(1, \"б\")],\n};\n",
        )
        .expect("write");
        let pairs = parse_phf_pairs(&path).expect("parses");
        assert_eq!(
            pairs,
            vec![(
                "ocs:x_2".to_string(),
                vec!["".to_string(), "б".to_string(), "".to_string()]
            )]
        );
    }
}
