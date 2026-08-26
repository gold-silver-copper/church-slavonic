//! Phase-2 shared identity layer generator and gate
//! (docs/UNIFIED_IDENTITY.md; docs/UNIFIED_LANGUAGE_PROMPT.md, execution
//! plan step 2 plus the projection-coherence gate).
//!
//! `cargo xtask unified-identity` regenerates, from committed artifacts only:
//!
//! - `data/unified/identity.tsv` — the initial identity table: the
//!   projection study's registered-lexeme matches restricted to the
//!   unambiguous 1:1, pos-compatible tier. One row per abstract lexeme.
//! - `data/unified/identity-candidates.tsv` — the review companion
//!   (defect-candidates precedent): ambiguous projections (2+ candidate
//!   pairings, or many OCS lexemes onto one Synodal lexeme), pos-mismatched
//!   registered matches, oracle-type-only matches, and — since merge phase 6
//!   (docs/UNIFIED_DATA.md) — the Synodal lexical-union's cross-recension
//!   claims (the preserved proposal queue), each carrying its ledger claim id
//!   as provenance. NOT identity claims.
//! - `data/unified/coherence-baseline.tsv` — the projection-coherence gate
//!   baseline: for every identity entry, each recension's attested cells
//!   replayed through the projection rules against the other side, with the
//!   match counts. Full enumeration, no subsetting.
//!
//! `cargo xtask unified-identity --check` (wired into `check-structure` and
//! CI) regenerates all three in memory and demands they match the committed
//! files byte-for-byte; a coherence match count falling below the committed
//! baseline is reported as a projection-coherence REGRESSION (the gate), any
//! other drift as staleness.

use crate::projection_study::{parse_body_rows, project, study_key};
use crate::report_io::read_tsv;
use crate::rewrite_pilot;
use church_slavonic_extractor::ocs::extract::load_registry;
use church_slavonic_orthography::projection::RuleCounts;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const IDENTITY_RELATIVE: &str = "data/unified/identity.tsv";
const CANDIDATES_RELATIVE: &str = "data/unified/identity-candidates.tsv";
const COHERENCE_RELATIVE: &str = "data/unified/coherence-baseline.tsv";
/// The preserved cross-source proposal queue: the input the Synodal
/// lexical-union ledger derives its cross-recension claims from
/// (`crates/xtask/src/synodal_lexical_union.rs`, `read_queue_claims`).
const LEXICAL_UNION_QUEUE_RELATIVE: &str = "reports/synodal-lexical-review-queue.tsv";
/// Provenance tag of every candidate row the projection study produces.
const PROJECTION_STUDY_PROVENANCE: &str = "projection-study";

/// Synodal part-of-speech codes an OCS pos may identify with.
fn compatible_synodal_pos(ocs_pos: &str) -> &'static [&'static str] {
    match ocs_pos {
        "noun" => &["noun", "proper-noun"],
        "adj" => &["adjective"],
        "verb" => &["verb"],
        "pron" => &["pronoun"],
        "num" => &["numeral"],
        "det" => &["determiner"],
        _ => &[],
    }
}

struct SynodalLexeme {
    id: String,
    lemma: String,
    pos: String,
}

struct Artifacts {
    identity: String,
    candidates: String,
    coherence: String,
    identity_rows: usize,
    candidate_rows: usize,
    totals: [usize; 4],
}

#[allow(clippy::too_many_lines)]
fn render(root: &Path) -> Result<Artifacts, Box<dyn Error>> {
    let registry = load_registry(&root.join("data/extracted"))?;
    let support = rewrite_pilot::dictionary_support(root)?;

    // ---- Synodal registered lexemes, bucketed by study key ---------------
    let mut registered: BTreeMap<String, Vec<SynodalLexeme>> = BTreeMap::new();
    for summary in &synodal_church_slavonic::lexemes()? {
        registered
            .entry(study_key(summary.lemma()))
            .or_default()
            .push(SynodalLexeme {
                id: summary.id().as_str().to_owned(),
                lemma: summary.lemma().to_owned(),
                pos: summary.part_of_speech().code().to_owned(),
            });
    }
    for bucket in registered.values_mut() {
        bucket.sort_by(|a, b| a.id.cmp(&b.id));
    }

    // ---- Synodal evidence keys (for the oracle-type tier and coherence) --
    let mut evidence_keys: HashSet<String> = HashSet::new();
    let token_text = fs::read_to_string(root.join("data/synodal/gold_token_oracle.tsv"))?;
    for row in parse_body_rows(&token_text, 5, "gold_token_oracle.tsv")? {
        evidence_keys.insert(study_key(row[0]));
    }
    let paradigm_text = fs::read_to_string(root.join("data/synodal/gold_paradigm_oracle.tsv"))?;
    for row in parse_body_rows(&paradigm_text, 13, "gold_paradigm_oracle.tsv")? {
        for variant in row[12].split(',').map(str::trim) {
            if variant.starts_with('-') || variant.contains('(') || variant.is_empty() {
                continue;
            }
            let surface: String = variant.chars().filter(|&c| c != '-').collect();
            evidence_keys.insert(study_key(&surface));
        }
    }
    let exact_text = fs::read_to_string(root.join("data/synodal/exact_forms.tsv"))?;
    let mut exact_by_lexeme: HashMap<String, Vec<String>> = HashMap::new();
    for row in parse_body_rows(&exact_text, 7, "exact_forms.tsv")? {
        evidence_keys.insert(study_key(row[3]));
        exact_by_lexeme
            .entry(row[0].to_owned())
            .or_default()
            .push(row[3].to_owned());
    }

    // ---- OCS attested cells ----------------------------------------------
    let mut forms_by_lexeme: HashMap<&str, Vec<&str>> = HashMap::new();
    for form in &registry.forms {
        forms_by_lexeme
            .entry(form.lexeme_id.as_str())
            .or_default()
            .push(form.form.as_str());
    }

    // ---- pairing ---------------------------------------------------------
    struct Pairing {
        ocs_id: String,
        ocs_pos: String,
        ocs_lemma: String,
        matched_key: String,
        synodal_id: String,
    }
    struct CandidateRow {
        ocs_id: String,
        ocs_pos: String,
        ocs_lemma: String,
        kind: &'static str,
        detail: String,
        provenance: String,
    }
    let mut pairings: Vec<Pairing> = Vec::new();
    let mut candidate_rows: Vec<CandidateRow> = Vec::new();
    let mut counts = RuleCounts::default();

    let mut lexemes: Vec<_> = registry.lexemes.iter().collect();
    lexemes.sort_by(|a, b| a.id.cmp(&b.id));
    // Every projected candidate key of every OCS lexeme, for the
    // lexical-union ingestion below (same projection as the pairing).
    let mut ocs_by_projected_key: BTreeMap<String, Vec<&str>> = BTreeMap::new();

    for lexeme in &lexemes {
        let Some(cands) = project(&lexeme.lemma, &mut counts).into_candidates() else {
            continue;
        };
        for candidate in &cands {
            ocs_by_projected_key
                .entry(candidate.clone())
                .or_default()
                .push(lexeme.id.as_str());
        }
        // Every (candidate key, registered Synodal lexeme) pairing, split by
        // pos compatibility. Candidate order is the deterministic projection
        // enumeration order; bucket order is sorted by Synodal id.
        let compatible_pos = compatible_synodal_pos(&lexeme.pos);
        let mut compatible: Vec<(&str, &SynodalLexeme)> = Vec::new();
        let mut incompatible: Vec<(&str, &SynodalLexeme)> = Vec::new();
        for candidate in &cands {
            if let Some(bucket) = registered.get(candidate) {
                for synodal in bucket {
                    if compatible_pos.contains(&synodal.pos.as_str()) {
                        compatible.push((candidate, synodal));
                    } else {
                        incompatible.push((candidate, synodal));
                    }
                }
            }
        }
        let describe = |pairs: &[(&str, &SynodalLexeme)]| {
            pairs
                .iter()
                .map(|(_, synodal)| format!("{}={}", synodal.id, synodal.lemma))
                .collect::<Vec<_>>()
                .join(";")
        };
        match compatible.len() {
            1 => {
                let (key, synodal) = compatible[0];
                pairings.push(Pairing {
                    ocs_id: lexeme.id.clone(),
                    ocs_pos: lexeme.pos.clone(),
                    ocs_lemma: lexeme.lemma.clone(),
                    matched_key: key.to_owned(),
                    synodal_id: synodal.id.clone(),
                });
            }
            0 if !incompatible.is_empty() => candidate_rows.push(CandidateRow {
                ocs_id: lexeme.id.clone(),
                ocs_pos: lexeme.pos.clone(),
                ocs_lemma: lexeme.lemma.clone(),
                kind: "registered-pos-mismatch",
                detail: describe(&incompatible),
                provenance: PROJECTION_STUDY_PROVENANCE.to_owned(),
            }),
            0 => {
                if let Some(candidate) = cands.iter().find(|c| evidence_keys.contains(*c)) {
                    candidate_rows.push(CandidateRow {
                        ocs_id: lexeme.id.clone(),
                        ocs_pos: lexeme.pos.clone(),
                        ocs_lemma: lexeme.lemma.clone(),
                        kind: "oracle-type",
                        detail: candidate.clone(),
                        provenance: PROJECTION_STUDY_PROVENANCE.to_owned(),
                    });
                }
            }
            _ => candidate_rows.push(CandidateRow {
                ocs_id: lexeme.id.clone(),
                ocs_pos: lexeme.pos.clone(),
                ocs_lemma: lexeme.lemma.clone(),
                kind: "ambiguous",
                detail: describe(&compatible),
                provenance: PROJECTION_STUDY_PROVENANCE.to_owned(),
            }),
        }
    }

    // Demote many-to-one pairings (2+ OCS lexemes onto one Synodal lexeme):
    // identity is 1:1 by the stability rule; the whole cluster goes to review.
    let mut synodal_use: BTreeMap<String, usize> = BTreeMap::new();
    for pairing in &pairings {
        *synodal_use.entry(pairing.synodal_id.clone()).or_default() += 1;
    }
    let (unique, shared): (Vec<Pairing>, Vec<Pairing>) = pairings
        .into_iter()
        .partition(|pairing| synodal_use[&pairing.synodal_id] == 1);
    for pairing in shared {
        candidate_rows.push(CandidateRow {
            ocs_id: pairing.ocs_id,
            ocs_pos: pairing.ocs_pos,
            ocs_lemma: pairing.ocs_lemma,
            kind: "ambiguous-many-to-one",
            detail: format!("{}={}", pairing.synodal_id, pairing.matched_key),
            provenance: PROJECTION_STUDY_PROVENANCE.to_owned(),
        });
    }
    candidate_rows.sort_by(|a, b| a.ocs_id.cmp(&b.ocs_id));

    // ---- lexical-union cross-recension claims (merge phase 6) ------------
    // The Synodal lexical-union ledger's `disputed`/`ambiguous` queue claims
    // assert an *unconfirmed* cross-recension identity between a mixed- or
    // OCS-recension headword and a Synodal Bible attestation. They are
    // ingested here as review candidates (never identities) with the ledger
    // claim id as provenance, so the identity layer is the single review
    // queue for cross-recension identity (docs/UNIFIED_DATA.md §4). The OCS
    // side is resolved by the same projection as the pairing above: every
    // OCS lexeme whose projected candidate keys include the claim lemma's
    // projection-normal key is listed (ids sorted), `-` when none does.
    let queue = read_tsv(&root.join(LEXICAL_UNION_QUEUE_RELATIVE))?;
    let queue_lemma = queue.index("lemma")?;
    let queue_printed = queue.index("printed")?;
    let queue_pos = queue.index("part_of_speech")?;
    let queue_semantic = queue.index("semantic_candidate_id")?;
    let queue_attestation = queue.index("attestation_candidate_id")?;
    let queue_passage = queue.index("passage")?;
    let queue_decision = queue.index("decision")?;
    for row in &queue.rows {
        let kind = match row[queue_decision].as_str() {
            "candidate-unreviewed" => "lexical-union-proposal",
            "blocked-ambiguous-homograph" => "lexical-union-homograph",
            other => {
                return Err(format!(
                    "{LEXICAL_UNION_QUEUE_RELATIVE} has decision {other:?}; the identity \
                     ingestion mirrors synodal_lexical_union::read_queue_claims"
                )
                .into());
            }
        };
        let ocs_id = ocs_by_projected_key
            .get(&study_key(&row[queue_lemma]))
            .map_or_else(|| "-".to_owned(), |ids| ids.join(";"));
        candidate_rows.push(CandidateRow {
            ocs_id,
            ocs_pos: row[queue_pos].clone(),
            ocs_lemma: row[queue_lemma].clone(),
            kind,
            detail: format!("{}@{}", row[queue_printed], row[queue_passage]),
            provenance: format!(
                "synodal-lexical-union:queue:{}:{}",
                row[queue_semantic], row[queue_attestation]
            ),
        });
    }

    // ---- abstract keys with deterministic homograph suffixes -------------
    // Base key: pos + projection-normal matched key. Suffix assignment is
    // ONCE per abstract lexeme: within a colliding base, order by OCS lexeme
    // id (stable across refreshes; ids embed the content signature).
    let mut by_base: BTreeMap<String, Vec<Pairing>> = BTreeMap::new();
    for pairing in unique {
        let base = format!("{}:{}", pairing.ocs_pos, pairing.matched_key);
        by_base.entry(base).or_default().push(pairing);
    }
    struct IdentityRow {
        abstract_key: String,
        pairing: Pairing,
    }
    let mut rows: Vec<IdentityRow> = Vec::new();
    for (base, mut group) in by_base {
        group.sort_by(|a, b| a.ocs_id.cmp(&b.ocs_id));
        for (index, pairing) in group.into_iter().enumerate() {
            let abstract_key = if index == 0 {
                base.clone()
            } else {
                format!("{base}_{}", index + 1)
            };
            rows.push(IdentityRow {
                abstract_key,
                pairing,
            });
        }
    }
    rows.sort_by(|a, b| a.abstract_key.cmp(&b.abstract_key));

    // ---- identity.tsv ----------------------------------------------------
    let mut identity = String::from(
        "# identity.tsv — the shared lexeme-identity table (docs/UNIFIED_IDENTITY.md).\n\
         # generated-by: cargo xtask unified-identity\n\
         # High-confidence tier only: unambiguous 1:1 pos-compatible registered-lexeme\n\
         # matches from the projection study. Review candidates live in\n\
         # identity-candidates.tsv and are NOT identity claims.\n\
         abstract_key\tpos\tocs_lexeme_id\tocs_citation\tocs_lemma_key\tsynodal_lexeme_id\tsynodal_citation\n",
    );
    let synodal_lemma_by_id: HashMap<&str, &SynodalLexeme> = registered
        .values()
        .flatten()
        .map(|synodal| (synodal.id.as_str(), synodal))
        .collect();
    for row in &rows {
        let synodal = synodal_lemma_by_id[row.pairing.synodal_id.as_str()];
        writeln!(
            identity,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.abstract_key,
            row.pairing.ocs_pos,
            row.pairing.ocs_id,
            row.pairing.ocs_lemma,
            support
                .key_by_lexeme
                .get(&row.pairing.ocs_id)
                .map_or("", String::as_str),
            synodal.id,
            synodal.lemma,
        )?;
    }

    // ---- identity-candidates.tsv -----------------------------------------
    let mut candidates = String::from(
        "# identity-candidates.tsv — projection pairings needing review before any\n\
         # identity claim (docs/UNIFIED_IDENTITY.md; defect-candidates precedent).\n\
         # generated-by: cargo xtask unified-identity\n\
         # kind: ambiguous (2+ pos-compatible registered pairings) |\n\
         #       ambiguous-many-to-one (several OCS lexemes project onto one Synodal lexeme) |\n\
         #       registered-pos-mismatch (registered match, incompatible part of speech) |\n\
         #       oracle-type (lemma surface attested in Synodal evidence, no registered lexeme) |\n\
         #       lexical-union-proposal (Synodal lexical-union queue claim, cross-recension identity unconfirmed) |\n\
         #       lexical-union-homograph (Synodal lexical-union queue claim blocked by a cross-source homograph)\n\
         # provenance: projection-study, or the Synodal lexical-union ledger claim id\n\
         #       (synodal-lexical-union:<claim_id>; lexical-union rows list every OCS lexeme\n\
         #       sharing the projection-normal key, `-` when none, and printed@passage as candidate)\n\
         ocs_lexeme_id\tpos\tocs_citation\tkind\tcandidates\tprovenance\n",
    );
    for row in &candidate_rows {
        writeln!(
            candidates,
            "{}\t{}\t{}\t{}\t{}\t{}",
            row.ocs_id, row.ocs_pos, row.ocs_lemma, row.kind, row.detail, row.provenance
        )?;
    }

    // ---- coherence-baseline.tsv ------------------------------------------
    // OCS side: every attested OCS cell of the entry, matched when some
    // projected candidate hits the Synodal evidence keys (study semantics).
    // Synodal side: every attested printed surface bound to the entry's
    // Synodal lexeme in exact_forms.tsv, matched when its study key is
    // reachable from some projected OCS attested cell of the same entry.
    let mut coherence = String::from(
        "# coherence-baseline.tsv — projection-coherence gate baseline\n\
         # (docs/UNIFIED_IDENTITY.md, gate section). Full enumeration over the\n\
         # identity table; match counts must never regress.\n\
         # generated-by: cargo xtask unified-identity\n\
         abstract_key\tocs_cells\tocs_matched\tsynodal_cells\tsynodal_matched\n",
    );
    let mut totals = [0usize; 4];
    for row in &rows {
        let mut ocs_cells = 0usize;
        let mut ocs_matched = 0usize;
        let mut projected_keys: HashSet<String> = HashSet::new();
        for surface in forms_by_lexeme
            .get(row.pairing.ocs_id.as_str())
            .into_iter()
            .flatten()
        {
            ocs_cells += 1;
            if let Some(cands) = project(surface, &mut counts).into_candidates() {
                if cands.iter().any(|c| evidence_keys.contains(c)) {
                    ocs_matched += 1;
                }
                projected_keys.extend(cands);
            }
        }
        let mut synodal_cells = 0usize;
        let mut synodal_matched = 0usize;
        for printed in exact_by_lexeme
            .get(row.pairing.synodal_id.as_str())
            .into_iter()
            .flatten()
        {
            synodal_cells += 1;
            if projected_keys.contains(&study_key(printed)) {
                synodal_matched += 1;
            }
        }
        totals[0] += ocs_cells;
        totals[1] += ocs_matched;
        totals[2] += synodal_cells;
        totals[3] += synodal_matched;
        writeln!(
            coherence,
            "{}\t{ocs_cells}\t{ocs_matched}\t{synodal_cells}\t{synodal_matched}",
            row.abstract_key
        )?;
    }

    let identity_rows = rows.len();
    let candidate_count = candidate_rows.len();
    Ok(Artifacts {
        identity,
        candidates,
        coherence,
        identity_rows,
        candidate_rows: candidate_count,
        totals,
    })
}

/// Parses a coherence baseline into per-key `(ocs_matched, synodal_matched)`.
fn parse_coherence(text: &str) -> Result<BTreeMap<String, (usize, usize)>, Box<dyn Error>> {
    let mut map = BTreeMap::new();
    for row in parse_body_rows(text, 5, "coherence-baseline.tsv")? {
        map.insert(row[0].to_owned(), (row[2].parse()?, row[4].parse()?));
    }
    Ok(map)
}

pub(crate) fn run(
    args: &mut dyn Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    match args.next().as_deref() {
        Some("--check") => check(root),
        Some(other) => Err(format!("unknown unified-identity flag: {other}").into()),
        None => {
            let artifacts = render(root)?;
            fs::create_dir_all(root.join("data/unified"))?;
            fs::write(root.join(IDENTITY_RELATIVE), &artifacts.identity)?;
            fs::write(root.join(CANDIDATES_RELATIVE), &artifacts.candidates)?;
            fs::write(root.join(COHERENCE_RELATIVE), &artifacts.coherence)?;
            report(&artifacts);
            Ok(())
        }
    }
}

fn report(artifacts: &Artifacts) {
    println!(
        "unified-identity: {} identity entries, {} review candidates",
        artifacts.identity_rows, artifacts.candidate_rows
    );
    println!(
        "unified-identity: coherence baseline OCS {}/{} matched, Synodal {}/{} matched",
        artifacts.totals[1], artifacts.totals[0], artifacts.totals[3], artifacts.totals[2]
    );
}

/// The staleness + projection-coherence gate (wired into `check-structure`).
pub(crate) fn check(root: &Path) -> Result<(), Box<dyn Error>> {
    let artifacts = render(root)?;
    // Parse gate: the committed table must load through the kernel registry.
    let committed_identity = fs::read_to_string(root.join(IDENTITY_RELATIVE)).map_err(|error| {
        format!("{IDENTITY_RELATIVE}: {error} (run cargo xtask unified-identity)")
    })?;
    church_slavonic_core::IdentityRegistry::parse(&committed_identity)
        .map_err(|error| format!("{IDENTITY_RELATIVE}: {error}"))?;
    let committed_coherence =
        fs::read_to_string(root.join(COHERENCE_RELATIVE)).map_err(|error| {
            format!("{COHERENCE_RELATIVE}: {error} (run cargo xtask unified-identity)")
        })?;

    // Coherence gate first: a regression gets its own diagnosis, staleness
    // alone a different one.
    let baseline = parse_coherence(&committed_coherence)?;
    let current = parse_coherence(&artifacts.coherence)?;
    let mut regressions = Vec::new();
    for (key, &(ocs, synodal)) in &baseline {
        match current.get(key) {
            None => regressions.push(format!("{key}: entry disappeared from the identity table")),
            Some(&(current_ocs, current_synodal)) => {
                if current_ocs < ocs || current_synodal < synodal {
                    regressions.push(format!(
                        "{key}: matched {ocs}/{synodal} -> {current_ocs}/{current_synodal}"
                    ));
                }
            }
        }
    }
    if !regressions.is_empty() {
        return Err(format!(
            "projection-coherence REGRESSION against {COHERENCE_RELATIVE} ({} entries):\n  {}",
            regressions.len(),
            regressions.join("\n  ")
        )
        .into());
    }

    let committed_candidates =
        fs::read_to_string(root.join(CANDIDATES_RELATIVE)).map_err(|error| {
            format!("{CANDIDATES_RELATIVE}: {error} (run cargo xtask unified-identity)")
        })?;
    for (relative, committed, current) in [
        (IDENTITY_RELATIVE, &committed_identity, &artifacts.identity),
        (
            CANDIDATES_RELATIVE,
            &committed_candidates,
            &artifacts.candidates,
        ),
        (
            COHERENCE_RELATIVE,
            &committed_coherence,
            &artifacts.coherence,
        ),
    ] {
        if committed != current {
            return Err(format!(
                "{relative} is stale; run cargo xtask unified-identity and commit the result"
            )
            .into());
        }
    }
    report(&artifacts);
    Ok(())
}
