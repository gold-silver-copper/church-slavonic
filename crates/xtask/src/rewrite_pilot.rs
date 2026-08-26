//! Rewrite pilot slice (phase 3/4): generate the noun residue + metadata
//! tables consumed by the new `church-slavonic` facade crate, and replay the
//! full attested noun oracle through that crate.
//!
//! - `cargo xtask rewrite-derivability --emit-residue` writes
//!   `crates/church-slavonic/generated/noun_residue.rs`.
//! - `cargo xtask rewrite-pilot-accuracy` replays every attested noun cell
//!   from `data/extracted` through `church_slavonic::noun_variants`.
//!
//! Oracle convention (documented in the facade's lib.rs): the oracle is
//! defined at lemma granularity. Homograph noun lexemes sharing a lemma have
//! their per-cell variant lists merged in rank order (stable; first
//! occurrence wins on duplicate surface forms).

use church_slavonic::{NounMeta, cell_code, kernel_noun_variants};
use old_church_slavonic_core::{Animacy, Case, Gender, Number, NumberRestriction};
use old_church_slavonic_extractor::extract::load_registry;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// One lemma's compact encoded metadata: (class, gender, animacy, restriction).
type MetaCodes = (u8, u8, u8, u8);

struct NounOracle {
    /// lemma -> encoded metadata (first lexeme entry wins).
    meta: BTreeMap<String, MetaCodes>,
    /// (lemma, cell code) -> merged variant list in rank order.
    cells: BTreeMap<(String, u8), Vec<String>>,
    /// Per-lexeme cell count (the 41,566 figure) and how many of those
    /// per-lexeme lists differ from the merged lemma-level list.
    lexeme_cells: usize,
    merged_divergent_lexeme_cells: usize,
}

fn class_code(value: &str) -> u8 {
    match value {
        "o-m-hard" => 1,
        "o-n-hard" => 2,
        "a-hard" => 3,
        "jo-m-soft" => 4,
        "jo-n-soft" => 5,
        "ja-soft" => 6,
        "i-f" => 7,
        "i-m" => 8,
        "u-m" => 9,
        "n-m" => 10,
        "n-n" => 11,
        "nt-n" => 12,
        "r-n" => 13,
        "s-n" => 14,
        "v-f" => 15,
        _ => 0,
    }
}

fn gender_code(value: &str) -> u8 {
    match value {
        "m" => 1,
        "f" => 2,
        "n" => 3,
        _ => 0,
    }
}

fn animacy_code(value: &str) -> u8 {
    match value {
        "an" => 1,
        "in" => 2,
        _ => 0,
    }
}

fn restriction_code(value: &str) -> u8 {
    match value {
        "sg" => 1,
        "du" => 2,
        "pl" => 3,
        _ => 0,
    }
}

fn decode_meta(codes: MetaCodes) -> NounMeta {
    NounMeta {
        class: match codes.0 {
            1 => Some(old_church_slavonic_core::NounClass::OMasculineHard),
            2 => Some(old_church_slavonic_core::NounClass::ONeuterHard),
            3 => Some(old_church_slavonic_core::NounClass::AHard),
            4 => Some(old_church_slavonic_core::NounClass::JoMasculineSoft),
            5 => Some(old_church_slavonic_core::NounClass::JoNeuterSoft),
            6 => Some(old_church_slavonic_core::NounClass::JaSoft),
            7 => Some(old_church_slavonic_core::NounClass::IFeminine),
            8 => Some(old_church_slavonic_core::NounClass::IMasculine),
            9 => Some(old_church_slavonic_core::NounClass::UMasculine),
            10 => Some(old_church_slavonic_core::NounClass::NMasculine),
            11 => Some(old_church_slavonic_core::NounClass::NNeuter),
            12 => Some(old_church_slavonic_core::NounClass::NtNeuter),
            13 => Some(old_church_slavonic_core::NounClass::RStem),
            14 => Some(old_church_slavonic_core::NounClass::SNeuter),
            15 => Some(old_church_slavonic_core::NounClass::VFeminine),
            _ => None,
        },
        gender: match codes.1 {
            1 => Some(Gender::Masculine),
            2 => Some(Gender::Feminine),
            3 => Some(Gender::Neuter),
            _ => None,
        },
        animacy: match codes.2 {
            1 => Some(Animacy::Animate),
            2 => Some(Animacy::Inanimate),
            _ => None,
        },
        restriction: match codes.3 {
            1 => NumberRestriction::SingularOnly,
            2 => NumberRestriction::DualOnly,
            3 => NumberRestriction::PluralOnly,
            _ => NumberRestriction::All,
        },
    }
}

fn feature_cell_code(feature: &str) -> Option<u8> {
    let cell = crate::parse_noun_cell(feature)?;
    Some(cell_code(cell.case, cell.number))
}

fn cell_from_code(code: u8) -> (Case, Number) {
    let case = match code / 3 {
        0 => Case::Nominative,
        1 => Case::Genitive,
        2 => Case::Dative,
        3 => Case::Accusative,
        4 => Case::Instrumental,
        5 => Case::Locative,
        _ => match code / 3 {
            6 => Case::Vocative,
            _ => Case::Vocative,
        },
    };
    let number = match code % 3 {
        0 => Number::Singular,
        1 => Number::Dual,
        _ => Number::Plural,
    };
    (case, number)
}

fn load_noun_oracle(root: &Path) -> Result<NounOracle, Box<dyn Error>> {
    let registry = load_registry(&root.join("data/extracted"))?;
    let mut meta: BTreeMap<String, MetaCodes> = BTreeMap::new();
    let mut lemma_by_id: BTreeMap<&str, &str> = BTreeMap::new();
    for lexeme in &registry.lexemes {
        if lexeme.pos != "noun" {
            continue;
        }
        lemma_by_id.insert(&lexeme.id, &lexeme.lemma);
        meta.entry(lexeme.lemma.clone()).or_insert((
            class_code(&lexeme.class),
            gender_code(&lexeme.gender),
            animacy_code(&lexeme.animacy),
            restriction_code(&lexeme.number_restriction),
        ));
    }
    // Per-lexeme rows in file order, rank-stable within a cell.
    let mut per_lexeme: BTreeMap<(String, u8), Vec<(u16, String)>> = BTreeMap::new();
    let mut merged_rows: BTreeMap<(String, u8), Vec<(u16, String)>> = BTreeMap::new();
    for row in &registry.forms {
        let Some(lemma) = lemma_by_id.get(row.lexeme_id.as_str()) else {
            continue;
        };
        let Some(code) = feature_cell_code(&row.feature) else {
            return Err(format!("unparsed noun feature {}", row.feature).into());
        };
        per_lexeme
            .entry((row.lexeme_id.clone(), code))
            .or_default()
            .push((row.rank, row.form.clone()));
        merged_rows
            .entry(((*lemma).to_string(), code))
            .or_default()
            .push((row.rank, row.form.clone()));
    }
    let dedupe = |rows: &mut Vec<(u16, String)>| -> Vec<String> {
        rows.sort_by_key(|(rank, _)| *rank);
        let mut texts: Vec<String> = Vec::new();
        for (_, form) in rows.iter() {
            if !texts.contains(form) {
                texts.push(form.clone());
            }
        }
        texts
    };
    let mut cells: BTreeMap<(String, u8), Vec<String>> = BTreeMap::new();
    for (key, mut rows) in merged_rows {
        cells.insert(key, dedupe(&mut rows));
    }
    let mut lexeme_cells = 0usize;
    let mut merged_divergent_lexeme_cells = 0usize;
    for ((lexeme_id, code), mut rows) in per_lexeme {
        lexeme_cells += 1;
        let lemma = lemma_by_id[lexeme_id.as_str()];
        let own = dedupe(&mut rows);
        if cells[&(lemma.to_string(), code)] != own {
            merged_divergent_lexeme_cells += 1;
        }
    }
    Ok(NounOracle {
        meta,
        cells,
        lexeme_cells,
        merged_divergent_lexeme_cells,
    })
}

pub(crate) fn emit_residue(root: &Path) -> Result<(), Box<dyn Error>> {
    let oracle = load_noun_oracle(root)?;
    let mut residue: Vec<(&str, u8, &Vec<String>)> = Vec::new();
    for ((lemma, code), expected) in &oracle.cells {
        let meta = decode_meta(oracle.meta[lemma]);
        let (case, number) = cell_from_code(*code);
        let predicted = kernel_noun_variants(lemma, &meta, case, number);
        if predicted.as_deref() != Some(expected.as_slice()) {
            residue.push((lemma, *code, expected));
        }
    }
    let mut out = String::new();
    out.push_str(
        "// @generated by `cargo xtask rewrite-derivability --emit-residue`. Do not edit.\n\
         //\n\
         // NOUN_META: (lemma, class, gender, animacy, number restriction) codes for\n\
         // every noun lemma in data/extracted (see church-slavonic/src/lib.rs for\n\
         // the code tables). NOUN_RESIDUE: (lemma, cell code, variants) for exactly\n\
         // the attested cells the rule kernel does not reproduce verbatim. Both\n\
         // slices are sorted for binary search.\n",
    );
    let _ = writeln!(
        out,
        "pub static NOUN_META: &[(&str, u8, u8, u8, u8)] = &[ // {} lemmas",
        oracle.meta.len()
    );
    for (lemma, (class, gender, animacy, restriction)) in &oracle.meta {
        let _ = writeln!(out, "    ({lemma:?}, {class}, {gender}, {animacy}, {restriction}),");
    }
    out.push_str("];\n");
    let _ = writeln!(
        out,
        "pub static NOUN_RESIDUE: &[(&str, u8, &[&str])] = &[ // {} cells",
        residue.len()
    );
    for (lemma, code, variants) in &residue {
        let _ = write!(out, "    ({lemma:?}, {code}, &[");
        for (index, variant) in variants.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "{variant:?}");
        }
        out.push_str("]),\n");
    }
    out.push_str("];\n");
    let path = root.join("crates/church-slavonic/generated/noun_residue.rs");
    fs::write(&path, &out)?;
    println!(
        "wrote {} ({} bytes): {} metadata rows, {} residue cells (of {} merged / {} per-lexeme)",
        path.display(),
        out.len(),
        oracle.meta.len(),
        residue.len(),
        oracle.cells.len(),
        oracle.lexeme_cells,
    );
    Ok(())
}

pub(crate) fn accuracy(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    if let Some(extra) = args.next() {
        return Err(format!("rewrite-pilot-accuracy takes no arguments, found {extra}").into());
    }
    let oracle = load_noun_oracle(root)?;
    let mut total = 0usize;
    let mut matched = 0usize;
    let mut mismatches: Vec<String> = Vec::new();
    for ((lemma, code), expected) in &oracle.cells {
        total += 1;
        let (case, number) = cell_from_code(*code);
        match church_slavonic::noun_variants(lemma, case, number) {
            Ok(variants) if &variants == expected => matched += 1,
            other => {
                if mismatches.len() < 20 {
                    mismatches.push(format!(
                        "{lemma} cell {code}: stored {:?} vs facade {other:?}",
                        expected
                    ));
                }
            }
        }
    }
    let generated = root.join("crates/church-slavonic/generated/noun_residue.rs");
    let bytes = fs::metadata(&generated)?.len();
    println!("rewrite pilot accuracy (nouns, lemma-merged oracle)");
    println!("  merged cells matched: {matched}/{total}");
    println!(
        "  per-lexeme cells covered: {} (of which {} homograph cells differ from the \
         merged lemma-level list and are served merged)",
        oracle.lexeme_cells, oracle.merged_divergent_lexeme_cells
    );
    println!("  generated table size: {bytes} bytes ({generated})", generated = generated.display());
    for line in &mismatches {
        println!("  MISMATCH {line}");
    }
    if matched != total {
        return Err(format!("pilot accuracy {matched}/{total}, expected 100%").into());
    }
    Ok(())
}
