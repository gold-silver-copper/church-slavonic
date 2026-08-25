//! Admission preflight: every category of mistake that v0.12 discovered only
//! at seal time or via first-failure late checks, detected statically from the
//! committed TSVs in one pass, with every violation reported at once.
//!
//! This command duplicates guards earlier; it never replaces them. The
//! authoritative late checks (`synodal-check`, the sealed floors and
//! ceilings, the family-review gate) all remain exactly as strict.
//!
//! Four categories, each of which caused real rework in v0.12:
//!
//! 1. **Duplicate lexeme identities.** Seven productive admissions collided
//!    with existing reviewed identities (`поразити`, `вѣровати`, `лꙋкавый`,
//!    `гробъ`, `хранити`, `десный`, `ѡставити`) and were caught only by the
//!    `integrity:cross_lexeme_ambiguous` ceiling after a full coverage run.
//!    Every one of those identities already owned an exact form or a lexical
//!    review whose surface the new lexeme also generated, so analyzing those
//!    committed surfaces exposes the collision in seconds. Genuine reviewed
//!    homonymy is recorded in `data/synodal/homonymy_allowlist.tsv`.
//! 2. **New held-out memorisation.** Exact and accent rows whose normalized
//!    type is held out are memorisation; the historical stock is frozen in
//!    `data/synodal/holdout_memorisation_baseline.tsv` and any new entry is a
//!    violation (the `holdout:memorised_analyzed` ceiling still enforces the
//!    token count late).
//! 3. **Evaluation-passage overlap.** An evaluation row may not share a
//!    passage with any runtime evidence citation; `synodal-check` enforces
//!    this first-failure, which cost three or four round-trips per wave.
//! 4. **Generates-nothing lexemes.** A class/lemma/formation mismatch (the
//!    `беззаконный` case) leaves a lexeme present but unable to produce a
//!    single form; probing each lexeme's own lemma surface catches it before
//!    any build.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

use synodal_church_slavonic_core::orthography::normalize_lookup_accentless;

use crate::report_io::{Table, read_tsv};
use crate::synodal_type_holdout;

pub(crate) const ALLOWLIST_PATH: &str = "data/synodal/homonymy_allowlist.tsv";
pub(crate) const BASELINE_PATH: &str = "data/synodal/holdout_memorisation_baseline.tsv";
const ALLOWLIST_HEADER: &str = "left_lexeme_id\tright_lexeme_id\tjustification";
const BASELINE_HEADER: &str = "normalized_type\ttables";

/// The corpus sources whose passages participate in holdout partitioning.
/// Evidence citing any other source (the grammar, the dictionary workbook)
/// carries a section citation, not a passage.
const TARGET_CORPUS_SOURCES: [&str; 2] = [
    "ponomar-elizabeth-bible-2026-08-09",
    "wikisource-church-slavonic-bible-2026-08-09",
];

pub(crate) fn admit_check(root: &Path, write_baseline: bool) -> Result<(), Box<dyn Error>> {
    if write_baseline {
        let entries = memorised_entries(root)?;
        write_baseline_file(root, &entries)?;
        println!(
            "synodal admit check: wrote {} with {} memorised held-out types",
            BASELINE_PATH,
            entries.len()
        );
        return Ok(());
    }
    let mut violations = Vec::new();
    violations.extend(check_duplicate_identities(root)?);
    violations.extend(check_new_holdout_memorisation(root)?);
    violations.extend(check_evaluation_passage_overlap(root)?);
    violations.extend(check_generation_probes(root)?);
    if violations.is_empty() {
        println!(
            "synodal admit check: no duplicate identities, no new held-out memorisation, no evaluation-passage overlap, no generation-dead lexemes"
        );
        return Ok(());
    }
    for violation in &violations {
        eprintln!("admit-check violation: {violation}");
    }
    Err(format!("synodal admit check found {} violations", violations.len()).into())
}

fn read_optional_tsv(path: &Path) -> Result<Option<Table>, Box<dyn Error>> {
    if path.is_file() {
        Ok(Some(read_tsv(path)?))
    } else {
        Ok(None)
    }
}

fn allowlisted_pairs(root: &Path) -> Result<BTreeSet<(String, String)>, Box<dyn Error>> {
    let mut pairs = BTreeSet::new();
    let Some(table) = read_optional_tsv(&root.join(ALLOWLIST_PATH))? else {
        return Ok(pairs);
    };
    if table.header.join("\t") != ALLOWLIST_HEADER {
        return Err(format!("{ALLOWLIST_PATH} has an unexpected header").into());
    }
    for row in &table.rows {
        let (left, right) = (row[0].clone(), row[1].clone());
        if row[2].trim().is_empty() {
            return Err(format!(
                "{ALLOWLIST_PATH}: the pair {left} / {right} carries no justification"
            )
            .into());
        }
        let ordered = if left <= right {
            (left, right)
        } else {
            (right, left)
        };
        pairs.insert(ordered);
    }
    Ok(pairs)
}

fn pair_key(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_owned(), right.to_owned())
    } else {
        (right.to_owned(), left.to_owned())
    }
}

/// Category 1: a committed surface owned by one identity that another lexeme
/// also analyzes to, or two lexemes sharing a normalized lemma and part of
/// speech, is a duplicated identity unless the pair is reviewed homonymy.
pub(crate) fn check_duplicate_identities(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let allowlist = allowlisted_pairs(root)?;
    let mut violations = Vec::new();

    // Owned surfaces: exact rows and reviewed lexical-form targets.
    let mut owned: Vec<(String, String, String)> = Vec::new(); // (surface, owner, origin)
    if let Some(exact) = read_optional_tsv(&root.join("data/synodal/exact_forms.tsv"))? {
        let (lexeme, expanded) = (exact.index("lexeme_id")?, exact.index("expanded")?);
        for row in &exact.rows {
            owned.push((
                row[expanded].clone(),
                row[lexeme].clone(),
                "exact_forms.tsv".into(),
            ));
        }
    }
    if let Some(reviews) = read_optional_tsv(&root.join("data/synodal/lexical_reviews.tsv"))? {
        let (lexeme, expanded, decision) = (
            reviews.index("lexeme_id")?,
            reviews.index("expanded")?,
            reviews.index("decision")?,
        );
        for row in &reviews.rows {
            if row[decision] == "admitted" && !row[expanded].is_empty() {
                owned.push((
                    row[expanded].clone(),
                    row[lexeme].clone(),
                    "lexical_reviews.tsv".into(),
                ));
            }
        }
    }
    let mut flagged = BTreeSet::new();
    for (surface, owner, origin) in &owned {
        let Ok(analyses) = synodal_church_slavonic_dictionary::analyze(surface) else {
            continue;
        };
        for analysis in analyses {
            let other = analysis.lexeme.id().as_str().to_owned();
            if &other == owner {
                continue;
            }
            let key = pair_key(owner, &other);
            if allowlist.contains(&key) || !flagged.insert(key) {
                continue;
            }
            violations.push(format!(
                "surface {surface:?} ({origin}, owner {owner}) is also analyzed by {other}; merge the productive admission onto the existing identity or record reviewed homonymy in {ALLOWLIST_PATH}"
            ));
        }
    }

    // Lemma collisions catch a duplicate before any surface overlaps.
    if let Some(lexemes) = read_optional_tsv(&root.join("data/synodal/lexemes.tsv"))? {
        let (id, lemma, pos) = (
            lexemes.index("id")?,
            lexemes.index("lemma")?,
            lexemes.index("part_of_speech")?,
        );
        let mut by_lemma: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
        for row in &lexemes.rows {
            by_lemma
                .entry((normalize_lookup_accentless(&row[lemma]), row[pos].clone()))
                .or_default()
                .push(row[id].clone());
        }
        for ((lemma, pos), ids) in by_lemma {
            if ids.len() < 2 {
                continue;
            }
            for pair in ids.windows(2) {
                let key = pair_key(&pair[0], &pair[1]);
                if allowlist.contains(&key) || flagged.contains(&key) {
                    continue;
                }
                flagged.insert(key);
                violations.push(format!(
                    "lexemes {} and {} share the normalized lemma {lemma:?} ({pos}); merge or record reviewed homonymy in {ALLOWLIST_PATH}",
                    pair[0], pair[1]
                ));
            }
        }
    }
    Ok(violations)
}

fn memorised_entries(root: &Path) -> Result<BTreeMap<String, BTreeSet<String>>, Box<dyn Error>> {
    let held = synodal_type_holdout::load(&root.join(synodal_type_holdout::HOLDOUT_PATH))?;
    let mut entries: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (path, columns) in [
        ("data/synodal/exact_forms.tsv", ["expanded", "printed"]),
        ("data/synodal/accents.tsv", ["expanded", "accented"]),
    ] {
        let Some(table) = read_optional_tsv(&root.join(path))? else {
            continue;
        };
        let indexes = columns
            .iter()
            .map(|column| table.index(column))
            .collect::<Result<Vec<_>, _>>()?;
        for row in &table.rows {
            for &index in &indexes {
                let normalized = normalize_lookup_accentless(&row[index]);
                if held.contains(&normalized) {
                    entries.entry(normalized).or_default().insert(path.into());
                }
            }
        }
    }
    Ok(entries)
}

fn write_baseline_file(
    root: &Path,
    entries: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), Box<dyn Error>> {
    let mut text = format!("{BASELINE_HEADER}\n");
    for (normalized, tables) in entries {
        text.push_str(&format!(
            "{normalized}\t{}\n",
            tables.iter().cloned().collect::<Vec<_>>().join(",")
        ));
    }
    fs::write(root.join(BASELINE_PATH), text)?;
    Ok(())
}

/// Category 2: any held-out normalized type reachable through an exact or
/// accent row that the frozen baseline does not already record.
pub(crate) fn check_new_holdout_memorisation(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let current = memorised_entries(root)?;
    let baseline_path = root.join(BASELINE_PATH);
    let Some(baseline) = read_optional_tsv(&baseline_path)? else {
        return Err(format!(
            "{BASELINE_PATH} is missing; run cargo xtask synodal-admit-check --write-baseline once and review the diff"
        )
        .into());
    };
    if baseline.header.join("\t") != BASELINE_HEADER {
        return Err(format!("{BASELINE_PATH} has an unexpected header").into());
    }
    let frozen: BTreeSet<String> = baseline.rows.iter().map(|row| row[0].clone()).collect();
    let mut violations = Vec::new();
    for (normalized, tables) in &current {
        if !frozen.contains(normalized) {
            violations.push(format!(
                "held-out type {normalized:?} gained a memorising row in {}; use a paradigm or licence instead, or justify a baseline change explicitly",
                tables.iter().cloned().collect::<Vec<_>>().join(",")
            ));
        }
    }
    for normalized in &frozen {
        if !current.contains_key(normalized) {
            violations.push(format!(
                "baseline lists {normalized:?} but no memorising row remains; rerun --write-baseline to ratchet the baseline down"
            ));
        }
    }
    Ok(violations)
}

/// Category 3: every evaluation passage that any runtime-evidence citation
/// shares, reported exhaustively instead of first-failure.
pub(crate) fn check_evaluation_passage_overlap(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let Some(evaluation) = read_optional_tsv(&root.join("data/synodal/evaluation.tsv"))? else {
        return Ok(Vec::new());
    };
    let (eval_id, source, passage) = (
        evaluation.index("id")?,
        evaluation.index("source_id")?,
        evaluation.index("passage")?,
    );
    let mut eval_passages: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for row in &evaluation.rows {
        eval_passages
            .entry((row[source].clone(), row[passage].clone()))
            .or_default()
            .push(row[eval_id].clone());
    }

    // Only evidence that runtime-shipping tables actually reference counts,
    // matching the authoritative extractor predicate exactly.
    let runtime_ids = runtime_evidence_ids(root)?;

    let mut violations = Vec::new();
    if let Some(evidence) = read_optional_tsv(&root.join("data/synodal/reviewed_evidence.tsv"))? {
        let (id, source, citation) = (
            evidence.index("evidence_id")?,
            evidence.index("source_id")?,
            evidence.index("citation")?,
        );
        for row in &evidence.rows {
            if !runtime_ids.contains(&row[id])
                || !TARGET_CORPUS_SOURCES.contains(&row[source].as_str())
            {
                continue;
            }
            if let Some(eval_ids) = eval_passages.get(&(row[source].clone(), row[citation].clone()))
            {
                violations.push(format!(
                    "runtime evidence {} cites passage {} which held-out evaluation rows {} also target; move the evaluation row to a clean passage",
                    row[id],
                    row[citation],
                    eval_ids.join(",")
                ));
            }
        }
    }
    if let Some(reviews) = read_optional_tsv(&root.join("data/synodal/lexical_reviews.tsv"))? {
        let (id, source, citation, decision) = (
            reviews.index("review_id")?,
            reviews.index("attestation_source_id")?,
            reviews.index("citation")?,
            reviews.index("decision")?,
        );
        for row in &reviews.rows {
            if row[decision] != "admitted" || !TARGET_CORPUS_SOURCES.contains(&row[source].as_str())
            {
                continue;
            }
            if let Some(eval_ids) = eval_passages.get(&(row[source].clone(), row[citation].clone()))
            {
                violations.push(format!(
                    "reviewed lexical attestation {} cites passage {} which held-out evaluation rows {} also target; move the evaluation row to a clean passage",
                    row[id],
                    row[citation],
                    eval_ids.join(",")
                ));
            }
        }
    }
    violations.sort();
    violations.dedup();
    Ok(violations)
}

/// The evidence ids referenced by runtime-shipping tables, mirroring the
/// extractor's `runtime_evidence_ids` specification.
fn runtime_evidence_ids(root: &Path) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let specifications: [(&str, usize); 14] = [
        ("principal_parts.tsv", 4),
        ("exact_forms.tsv", 4),
        ("alignments.tsv", 8),
        ("abbreviations.tsv", 6),
        ("abbreviation_families.tsv", 5),
        ("accents.tsv", 4),
        ("accent_paradigms.tsv", 6),
        ("positional_paradigms.tsv", 4),
        ("noun_restrictions.tsv", 3),
        ("positional_rules.tsv", 5),
        ("transformation_rules.tsv", 5),
        ("irregular_overrides.tsv", 3),
        ("verb_defectiveness.tsv", 6),
        ("irregular_verb_inventory.tsv", 5),
    ];
    let mut ids = BTreeSet::new();
    for (file_name, column) in specifications {
        let Some(table) = read_optional_tsv(&root.join("data/synodal").join(file_name))? else {
            continue;
        };
        for row in &table.rows {
            for id in row[column].split(',') {
                if !id.is_empty() {
                    ids.insert(id.to_owned());
                }
            }
        }
    }
    Ok(ids)
}

/// Category 4: a registered lexeme whose own lemma surface no longer analyzes
/// to it generates nothing at all — the shape of the `беззаконный`
/// double-n/mobile-е formation mismatch. Exact-only and archaic identities
/// are exercised through their exact rows instead, so only productive classes
/// are probed.
pub(crate) fn check_generation_probes(root: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let Some(lexemes) = read_optional_tsv(&root.join("data/synodal/lexemes.tsv"))? else {
        return Ok(Vec::new());
    };
    let (id, lemma, class) = (
        lexemes.index("id")?,
        lexemes.index("lemma")?,
        lexemes.index("class")?,
    );
    // A lexeme counts as alive if any of its own surfaces analyzes back to it:
    // its lemma, an owned exact form, an owned reviewed lexical form, or an
    // evaluation expectation. An OCS-spelled lemma (кънязь) or a fluent-vowel
    // nominative (лакоть) legitimately differs from every generated print, so
    // the probe never relies on the lemma alone.
    let mut probes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &lexemes.rows {
        probes
            .entry(row[id].clone())
            .or_default()
            .push(row[lemma].clone());
    }
    // A handful of stem-derived candidate surfaces covers lexemes whose lemma
    // is not itself generable (fluent-vowel лакоть, OCS-spelled lemmas).
    if let Some(lexeme_table) = read_optional_tsv(&root.join("data/synodal/lexemes.tsv"))? {
        let (id_column, stem_column) = (lexeme_table.index("id")?, lexeme_table.index("stem")?);
        for row in &lexeme_table.rows {
            if row[stem_column].is_empty() {
                continue;
            }
            if let Some(list) = probes.get_mut(&row[id_column]) {
                for ending in ["ъ", "ь", "а", "о", "е", "и", "ы", "ѧ", "ю", "ти"] {
                    list.push(format!("{}{}", row[stem_column], ending));
                }
            }
        }
    }
    for (path, id_column, surface_column) in [
        ("data/synodal/exact_forms.tsv", "lexeme_id", "expanded"),
        ("data/synodal/lexical_reviews.tsv", "lexeme_id", "expanded"),
        (
            "data/synodal/evaluation.tsv",
            "lexeme_id",
            "expected_expanded",
        ),
    ] {
        let Some(table) = read_optional_tsv(&root.join(path))? else {
            continue;
        };
        let (owner, surface) = (table.index(id_column)?, table.index(surface_column)?);
        for row in &table.rows {
            if !row[surface].is_empty() {
                if let Some(list) = probes.get_mut(&row[owner]) {
                    list.push(row[surface].clone());
                }
            }
        }
    }
    let mut violations = Vec::new();
    for row in &lexemes.rows {
        if matches!(row[class].as_str(), "" | "archaic" | "exact") {
            continue;
        }
        let reaches_itself = probes.get(&row[id]).into_iter().flatten().any(|surface| {
            synodal_church_slavonic_dictionary::analyze(surface)
                .map(|analyses| {
                    analyses
                        .iter()
                        .any(|analysis| analysis.lexeme.id().as_str() == row[id])
                })
                .unwrap_or(false)
        });
        if !reaches_itself {
            violations.push(format!(
                "lexeme {} ({}, class {}) analyzes none of its own surfaces; its class, stem, or a principal-part formation is inconsistent and the lexeme likely generates nothing",
                row[id], row[lemma], row[class]
            ));
        }
    }
    Ok(violations)
}

/// FNV-1a fingerprint matching the runtime crates' build scripts.
fn registry_fingerprint(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}-{}", bytes.len())
}

/// Refuses to measure with a binary compiled against an older registry than
/// the one on disk. `synodal-regenerate` rewrites `generated/registry.rs`,
/// but the registries compile *into* the binaries, so every measurement after
/// a regenerate silently reflects the old data until a rebuild — which cost a
/// full evaluation cycle during v0.12. The message names the exact rebuild.
pub(crate) fn ensure_registry_current(root: &Path) -> Result<(), Box<dyn Error>> {
    for (crate_name, path, compiled) in [
        (
            "synodal-church-slavonic",
            "crates/synodal-church-slavonic/generated/registry.rs",
            synodal_church_slavonic::REGISTRY_FINGERPRINT,
        ),
        (
            "synodal-church-slavonic-dictionary",
            "crates/synodal-church-slavonic-dictionary/generated/registry.rs",
            synodal_church_slavonic_dictionary::REGISTRY_FINGERPRINT,
        ),
    ] {
        let on_disk = registry_fingerprint(&fs::read(root.join(path))?);
        if on_disk != compiled {
            return Err(format!(
                "this binary was compiled against an older {crate_name} registry than {path}; rebuild before measuring: cargo build --release -p xtask -p synodal-church-slavonic-dictionary"
            )
            .into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(label: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "synodal-admit-check-{label}-{}",
            std::process::id()
        ));
        if directory.exists() {
            fs::remove_dir_all(&directory).expect("stale scratch removed");
        }
        fs::create_dir_all(directory.join("data/synodal")).expect("scratch created");
        directory
    }

    /// Replays the v0.12 wave-15 duplicate exactly as the pre-merge tree
    /// presented it: the surface лꙋкавство stood behind one identity while
    /// the registry analyzed it to another. The preflight must name the pair.
    #[test]
    fn replays_the_lukavstvo_duplicate_identity() {
        let root = scratch("duplicate");
        fs::write(
            root.join("data/synodal/exact_forms.tsv"),
            "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension\n\
             synodal:noun:v12-lukavstvo\tnoun:accusative:singular:inanimate\tлꙋкавство\tлꙋка́вство\tguard\tsynodal-attestation\tsynodal-russian\n",
        )
        .expect("exact fixture written");
        let violations = check_duplicate_identities(&root).expect("check ran");
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("synodal:noun:v07-9c2563bd3383fa6d")),
            "expected the existing identity to be named: {violations:?}"
        );
        fs::remove_dir_all(&root).expect("scratch removed");
    }

    /// Replays the v0.12 wave-15 collision: the вѣ́рный evaluation row on
    /// Apoc.1.5 shared its passage with runtime-referenced evidence. All
    /// overlaps must surface in one pass.
    #[test]
    fn replays_the_verny_evaluation_passage_collision() {
        let root = scratch("passage");
        fs::write(
            root.join("data/synodal/evaluation.tsv"),
            "id\tlexeme_id\tcell\tpolicy\texpected_expanded\texpected_printed\tsource_id\tpassage\tregularity\n\
             eval:v12:replay-verny\tsynodal:adjective:v06-a79476be07ef953c\tadjective:nominative:singular:masculine:inanimate:long:positive\tproductive\tвѣрный\tвѣ́рный\tponomar-elizabeth-bible-2026-08-09\tApoc.1.5\tv12-productive-held-out\n",
        )
        .expect("evaluation fixture written");
        fs::write(
            root.join("data/synodal/reviewed_evidence.tsv"),
            "evidence_id\tcandidate_id\tsource_id\tcitation\tdecision\ttarget_recension\treview_note\n\
             v05-source-form-98bdadbf64a10b4f\tsynodal:candidate:replay\tponomar-elizabeth-bible-2026-08-09\tApoc.1.5\treviewed\tsynodal-russian\treplayed fixture\n",
        )
        .expect("evidence fixture written");
        fs::write(
            root.join("data/synodal/exact_forms.tsv"),
            "lexeme_id\tcell\texpanded\tprinted\tevidence_id\tsource_kind\ttarget_recension\n\
             synodal:noun:replay\tlexical-form\tагнецъ\tа҆́гнецъ\tv05-source-form-98bdadbf64a10b4f\tsynodal-attestation\tsynodal-russian\n",
        )
        .expect("runtime reference written");
        let violations = check_evaluation_passage_overlap(&root).expect("check ran");
        assert!(
            violations.iter().any(|violation| {
                violation.contains("Apoc.1.5") && violation.contains("eval:v12:replay-verny")
            }),
            "expected the shared passage to be reported: {violations:?}"
        );
        fs::remove_dir_all(&root).expect("scratch removed");
    }
}
