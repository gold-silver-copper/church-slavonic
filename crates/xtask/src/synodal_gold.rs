//! The Synodal gold gate: full-enumeration replay of both committed gold
//! oracles (`data/synodal/gold_token_oracle.tsv`,
//! `data/synodal/gold_paradigm_oracle.tsv`) against the engine, regenerating
//! the committed gap worklist `reports/synodal-gold-gap.tsv`. The comparison
//! contract is normative in `docs/SYNODAL_GOLD_ORACLE.md`; this module applies
//! it and never extends it silently.
//!
//! `--check` fails iff the regenerated gap is not a subset of the committed
//! one; `--fix` rewrites the committed gap and fails if it would grow. Witness
//! consultation is a source-present-only refinement: with the intermediate
//! witnesses absent (CI), engine-disagreement rows keep their `engine-*`
//! class, and the gap is identical either way because witnesses only feed the
//! separate defect-candidate report, never the gap itself.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::time::Instant;
use synodal_church_slavonic::{GrammarCell, Inflector, LexemeId, OrthographyProfile, abbreviation};
use synodal_church_slavonic_core::{normalize_lookup_accentless, reflexive_base_candidates};
use synodal_church_slavonic_dictionary::coverage::{Analyzer, classify_non_lexical};
use synodal_church_slavonic_dictionary::{Analysis, AnalysisSource};
use unicode_normalization::UnicodeNormalization;

const TOKEN_ORACLE_RELATIVE: &str = "data/synodal/gold_token_oracle.tsv";
const PARADIGM_ORACLE_RELATIVE: &str = "data/synodal/gold_paradigm_oracle.tsv";
const PARADIGM_HEADWORDS_RELATIVE: &str = "data/synodal/gold_paradigm_headwords.tsv";
const LEDGER_RELATIVE: &str = "data/synodal/gold_source_defects.tsv";
const GAP_RELATIVE: &str = "reports/synodal-gold-gap.tsv";
const DEFECT_CANDIDATES_RELATIVE: &str = "reports/synodal-gold-defect-candidates.tsv";
const CROSSWIRE_RELATIVE: &str = "data/intermediate/synodal/crosswire-csl-elizabeth-1.5.2.jsonl";
const WIKISOURCE_RELATIVE: &str =
    "data/intermediate/synodal/wikisource-church-slavonic-bible-2026-08-09.jsonl";

const GAP_COLUMN_HEADER: &str = "oracle\tkey\treason\tengine_output\texpected";

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    crate::synodal::sync_registry_override(root)?;
    let mut fix = false;
    let mut scope = Scope::default();
    let mut peeked = args.peekable();
    match peeked.peek().map(String::as_str) {
        Some("propose") => {
            peeked.next();
            return crate::synodal_gold_burndown::propose(&mut peeked, root);
        }
        Some("admit") => {
            peeked.next();
            return crate::synodal_gold_burndown::admit(&mut peeked, root);
        }
        Some("loop") => {
            peeked.next();
            return crate::synodal_gold_burndown::inner_loop(&mut peeked, root);
        }
        Some("probe") => {
            peeked.next();
            return crate::synodal_gold_burndown::probe(&mut peeked, root);
        }
        Some("cell") => {
            peeked.next();
            return crate::synodal_gold_burndown::cell(&mut peeked, root);
        }
        Some("analyze") => {
            peeked.next();
            return crate::synodal_gold_burndown::analyze(&mut peeked, root);
        }
        Some("refit") => {
            peeked.next();
            return crate::synodal_gold_refit::refit(&mut peeked, root);
        }
        _ => {}
    }
    while let Some(argument) = peeked.next() {
        match argument.as_str() {
            "--check" => fix = false,
            "--fix" => fix = true,
            "--only" => {
                let class = peeked.next().ok_or("--only requires a gap class")?;
                scope.classes.insert(class);
            }
            "--lemma" => {
                let key = peeked
                    .next()
                    .ok_or("--lemma requires a lemma or lexeme id")?;
                scope.lemmas.push(key);
            }
            "--types-from" => {
                let path = peeked.next().ok_or("--types-from requires a file path")?;
                scope.keys.extend(read_type_keys(Path::new(&path))?);
            }
            other => return Err(format!("unknown synodal-gold option: {other}").into()),
        }
    }
    if fix && !scope.is_empty() {
        return Err(
            "synodal-gold --fix replays the full oracles; scope flags apply to --check only".into(),
        );
    }
    if scope.is_empty() {
        gate(root, fix)
    } else {
        scoped_check(root, &scope)
    }
}

/// Reads gap keys from a file: one per line, either a bare key (a token
/// surface or paradigm key) or a full gap row (`oracle<TAB>key...`), from
/// which the key column is taken. Comments and blank lines are skipped.
fn read_type_keys(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(content
        .lines()
        .filter(|line| {
            !line.starts_with('#') && !line.trim().is_empty() && *line != GAP_COLUMN_HEADER
        })
        .map(|line| {
            let mut fields = line.split('\t');
            match (fields.next(), fields.next()) {
                (Some("token" | "paradigm"), Some(key)) => key.to_owned(),
                _ => line.to_owned(),
            }
        })
        .collect())
}

/// What a replay covers. Empty means the full oracles (the gate); otherwise
/// only rows whose committed gap class is in `classes`, whose key is in
/// `keys`, or which a lexeme in `lemmas` analyses/generates. Scope never
/// changes the comparison contract: a selected row replays exactly as it
/// would under the full gate.
#[derive(Clone, Debug, Default)]
pub(crate) struct Scope {
    pub(crate) classes: BTreeSet<String>,
    pub(crate) keys: BTreeSet<String>,
    pub(crate) lemmas: Vec<String>,
}

impl Scope {
    pub(crate) fn is_empty(&self) -> bool {
        self.classes.is_empty() && self.keys.is_empty() && self.lemmas.is_empty()
    }

    pub(crate) fn from_keys(keys: impl IntoIterator<Item = String>) -> Self {
        Self {
            keys: keys.into_iter().collect(),
            ..Self::default()
        }
    }
}

/// The committed gap, keyed by (oracle, key) with its committed reason.
pub(crate) fn committed_gap(
    root: &Path,
) -> Result<BTreeMap<(String, String), String>, Box<dyn Error>> {
    let gap_path = root.join(GAP_RELATIVE);
    let content = fs::read_to_string(&gap_path).map_err(|error| {
        format!(
            "{GAP_RELATIVE}: {error}; run cargo xtask synodal-gold --fix to commit the baseline"
        )
    })?;
    let mut committed = BTreeMap::new();
    for row in data_rows(&content) {
        let fields: Vec<&str> = row.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!("short committed gap row: {row}").into());
        }
        committed.insert(
            (fields[0].to_owned(), fields[1].to_owned()),
            fields[2].to_owned(),
        );
    }
    Ok(committed)
}

/// Every committed gap row as (oracle, key, reason). Paradigm keys are not
/// unique (headword-less tables share keys), so counts must be taken over
/// rows, never over the deduplicated key map.
pub(crate) type CommittedRow = (String, String, String);

pub(crate) fn committed_rows(root: &Path) -> Result<Vec<CommittedRow>, Box<dyn Error>> {
    let content = fs::read_to_string(root.join(GAP_RELATIVE))
        .map_err(|error| format!("{GAP_RELATIVE}: {error}"))?;
    Ok(data_rows(&content)
        .filter_map(|row| {
            let fields: Vec<&str> = row.split('\t').collect();
            (fields.len() == 5).then(|| {
                (
                    fields[0].to_owned(),
                    fields[1].to_owned(),
                    fields[2].to_owned(),
                )
            })
        })
        .collect())
}

/// The result of a (full or scoped) replay.
pub(crate) struct Replay {
    pub(crate) gap: Vec<GapRow>,
    /// The (oracle, key) pairs the replay covered.
    pub(crate) covered: BTreeSet<(String, String)>,
    pub(crate) token_total: usize,
    pub(crate) paradigm_total: usize,
    pub(crate) seconds: f64,
}

/// Replays the selected rows of both oracles with a freshly built analyzer
/// (so an installed registry override is honoured) and returns the failing
/// rows. Witness consultation and ledger validation are the full gate's
/// source-present refinements and are not part of a scoped replay.
pub(crate) fn replay(root: &Path, scope: &Scope) -> Result<Replay, Box<dyn Error>> {
    let started = Instant::now();
    let token_rows = load_token_oracle(root)?;
    let paradigm_rows = load_paradigm_oracle(root)?;
    let headwords = load_paradigm_headwords(root)?;
    let ledger = load_defect_ledger(root)?;
    let committed = committed_gap(root)?;
    let analyzer =
        Analyzer::new(Inflector::default()).map_err(|error| format!("build analyzer: {error}"))?;
    let liturgical = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    let lemma_keys: BTreeSet<String> = scope
        .lemmas
        .iter()
        .map(|lemma| normalize_lookup_accentless(lemma))
        .collect();
    let selects = |oracle: &str, key: &str, lexeme_hits: &dyn Fn() -> bool| -> bool {
        if scope.keys.contains(key) {
            return true;
        }
        if !scope.classes.is_empty()
            && committed
                .get(&(oracle.to_owned(), key.to_owned()))
                .is_some_and(|reason| scope.classes.contains(reason))
        {
            return true;
        }
        !lemma_keys.is_empty() && lexeme_hits()
    };
    let mut gap: Vec<GapRow> = Vec::new();
    let mut covered = BTreeSet::new();
    for row in &token_rows {
        if ledger
            .iter()
            .any(|defect| defect.ponomar_surface == row.surface)
        {
            continue;
        }
        let surface = nfc(&row.surface);
        let hits = || {
            analyzer
                .analyze_profile(&surface, OrthographyProfile::SynodalLiturgical)
                .unwrap_or_default()
                .iter()
                .any(|analysis| {
                    lemma_keys.contains(&normalize_lookup_accentless(analysis.lexeme.lemma()))
                        || lemma_keys.contains(analysis.lexeme.id().as_str())
                })
        };
        if !selects("token", &row.surface, &hits) {
            continue;
        }
        covered.insert(("token".to_owned(), row.surface.clone()));
        if let Some(failure) = replay_token(&analyzer, liturgical, row) {
            gap.push(GapRow {
                oracle: "token",
                key: row.surface.clone(),
                reason: failure.reason,
                engine_output: failure.engine_output,
                expected: row.surface.clone(),
            });
        }
    }
    for row in &paradigm_rows {
        let hits = || {
            paradigm_lemma(&row.headword)
                .is_some_and(|lemma| lemma_keys.contains(&normalize_lookup_accentless(&lemma)))
                || resolve_paradigm_lexeme(&headwords, row).is_some_and(|lexeme| {
                    lemma_keys.contains(lexeme.id().as_str())
                        || lemma_keys.contains(&normalize_lookup_accentless(lexeme.lemma()))
                })
        };
        if !selects("paradigm", &row.key, &hits) {
            continue;
        }
        covered.insert(("paradigm".to_owned(), row.key.clone()));
        if let Some(gap_row) = replay_paradigm(liturgical, &headwords, row) {
            gap.push(gap_row);
        }
    }
    gap.sort();
    gap.dedup();
    Ok(Replay {
        gap,
        covered,
        token_total: token_rows.len(),
        paradigm_total: paradigm_rows.len(),
        seconds: started.elapsed().as_secs_f64(),
    })
}

/// Per-class delta of a scoped replay against the committed gap, over the
/// covered keys only. Returns (class -> (committed rows, regenerated rows)).
pub(crate) fn class_delta(
    replay: &Replay,
    committed_rows: &[CommittedRow],
) -> BTreeMap<(String, String), (usize, usize)> {
    let mut delta: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
    for (oracle, key, reason) in committed_rows {
        if replay.covered.contains(&(oracle.clone(), key.clone())) {
            delta.entry((oracle.clone(), reason.clone())).or_default().0 += 1;
        }
    }
    for row in &replay.gap {
        delta
            .entry((row.oracle.to_owned(), row.reason.to_owned()))
            .or_default()
            .1 += 1;
    }
    delta
}

/// `--check` restricted to a scope: prints the per-class delta over the
/// covered keys and fails on any covered row that is not in the committed
/// gap. Never writes the gap; the full replay stays the authority for --fix.
fn scoped_check(root: &Path, scope: &Scope) -> Result<(), Box<dyn Error>> {
    let committed = committed_gap(root)?;
    let rows = committed_rows(root)?;
    let result = replay(root, scope)?;
    let delta = class_delta(&result, &rows);
    println!(
        "synodal-gold --check (scoped): {} of {} token types and paradigm cells replayed, {} in gap",
        result.covered.len(),
        result.token_total + result.paradigm_total,
        result.gap.len()
    );
    for ((oracle, reason), (before, after)) in &delta {
        println!(
            "  {oracle}\t{reason}\t{before} -> {after}\t({:+})",
            *after as i64 - *before as i64
        );
    }
    println!("  replay runtime: {:.1}s", result.seconds);
    let regressed: Vec<&GapRow> = result
        .gap
        .iter()
        .filter(|row| {
            committed
                .get(&(row.oracle.to_owned(), row.key.clone()))
                .is_none_or(|reason| reason != row.reason)
        })
        .collect();
    if !regressed.is_empty() {
        for row in regressed.iter().take(20) {
            eprintln!("  regressed row: {}", row.render());
        }
        return Err(format!(
            "synodal-gold --check (scoped): {} covered rows fail with a class absent from the committed gap",
            regressed.len()
        )
        .into());
    }
    Ok(())
}

/// One row of the regenerated gap worklist.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct GapRow {
    pub(crate) oracle: &'static str,
    pub(crate) key: String,
    pub(crate) reason: &'static str,
    pub(crate) engine_output: String,
    pub(crate) expected: String,
}

impl GapRow {
    pub(crate) fn render(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}",
            self.oracle, self.key, self.reason, self.engine_output, self.expected
        )
    }
}

fn gate(root: &Path, fix: bool) -> Result<(), Box<dyn Error>> {
    let started = Instant::now();
    let token_rows = load_token_oracle(root)?;
    let paradigm_rows = load_paradigm_oracle(root)?;
    let headwords = load_paradigm_headwords(root)?;
    let ledger = load_defect_ledger(root)?;

    let analyzer =
        Analyzer::new(Inflector::default()).map_err(|error| format!("build analyzer: {error}"))?;
    let liturgical = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();

    let mut gap: Vec<GapRow> = Vec::new();
    let mut token_failures: Vec<TokenFailure> = Vec::new();
    for row in &token_rows {
        if ledger
            .iter()
            .any(|defect| defect.ponomar_surface == row.surface)
        {
            continue; // adjudicated source defect, excluded from the gap
        }
        if let Some(failure) = replay_token(&analyzer, liturgical, row) {
            if failure.reason.starts_with("engine-wrong-") {
                token_failures.push(TokenFailure {
                    surface: row.surface.clone(),
                    references: row.references.clone(),
                    engine_output: failure.engine_output.clone(),
                });
            }
            gap.push(GapRow {
                oracle: "token",
                key: row.surface.clone(),
                reason: failure.reason,
                engine_output: failure.engine_output,
                expected: row.surface.clone(),
            });
        }
    }
    let token_gap = gap.len();
    for row in &paradigm_rows {
        if let Some(gap_row) = replay_paradigm(liturgical, &headwords, row) {
            gap.push(gap_row);
        }
    }
    gap.sort();
    gap.dedup();

    validate_defect_ledger(root, &ledger)?;
    let candidate_count = witness_sweep(root, &token_failures)?;

    let content = render_gap(&gap);
    let gap_path = root.join(GAP_RELATIVE);
    let committed = fs::read_to_string(&gap_path).ok();
    let regenerated_rows: BTreeSet<String> = gap.iter().map(GapRow::render).collect();
    let committed_rows: Option<BTreeSet<String>> = committed
        .as_deref()
        .map(|content| data_rows(content).map(str::to_owned).collect());

    print_summary(
        &gap,
        token_gap,
        token_rows.len(),
        paradigm_rows.len(),
        candidate_count,
        started.elapsed().as_secs_f64(),
    );

    if fix {
        if let Some(committed_rows) = &committed_rows {
            let committed_keys: BTreeSet<(&str, &str)> = committed_rows
                .iter()
                .filter_map(|row| row_key(row))
                .collect();
            let new_keys: Vec<&GapRow> = gap
                .iter()
                .filter(|row| !committed_keys.contains(&(row.oracle, row.key.as_str())))
                .collect();
            if !new_keys.is_empty() {
                for row in new_keys.iter().take(20) {
                    eprintln!("  new gap key: {}", row.render());
                }
                return Err(format!(
                    "synodal-gold --fix refuses to grow the gap: {} new failing keys",
                    new_keys.len()
                )
                .into());
            }
        }
        fs::write(&gap_path, content)?;
        println!(
            "synodal-gold --fix: wrote {GAP_RELATIVE} ({} rows)",
            gap.len()
        );
        return Ok(());
    }

    let Some(committed_rows) = committed_rows else {
        return Err(format!(
            "{GAP_RELATIVE} is missing; run cargo xtask synodal-gold --fix to commit the baseline"
        )
        .into());
    };
    let new_rows: Vec<&String> = regenerated_rows
        .iter()
        .filter(|row| !committed_rows.contains(*row))
        .collect();
    if !new_rows.is_empty() {
        for row in new_rows.iter().take(20) {
            eprintln!("  regressed row: {row}");
        }
        return Err(format!(
            "synodal-gold --check: the regenerated gap has {} rows absent from the committed {GAP_RELATIVE}; \
             fix the regression or, if the change is intended and shrinking, run cargo xtask synodal-gold --fix",
            new_rows.len()
        )
        .into());
    }
    println!(
        "synodal-gold --check: regenerated gap ({} rows) is a subset of the committed gap ({} rows)",
        regenerated_rows.len(),
        committed_rows.len()
    );
    Ok(())
}

fn row_key(row: &str) -> Option<(&str, &str)> {
    let mut fields = row.split('\t');
    let oracle = match fields.next()? {
        "token" => "token",
        "paradigm" => "paradigm",
        _ => return None,
    };
    Some((oracle, fields.next()?))
}

pub(crate) fn data_rows(content: &str) -> impl Iterator<Item = &str> {
    content
        .lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty() && *line != GAP_COLUMN_HEADER)
}

fn render_gap(gap: &[GapRow]) -> String {
    let mut output = String::new();
    output.push_str(
        "# synodal-gold-gap.tsv — the committed gap worklist of the Synodal gold gate.\n\
         # generated-by: cargo xtask synodal-gold --fix\n\
         # contract: docs/SYNODAL_GOLD_ORACLE.md; one row per failing token type or paradigm cell.\n\
         # The gate (synodal-gold --check) fails if the regenerated gap is not a subset of this file.\n",
    );
    output.push_str(GAP_COLUMN_HEADER);
    output.push('\n');
    for row in gap {
        output.push_str(&row.render());
        output.push('\n');
    }
    output
}

fn print_summary(
    gap: &[GapRow],
    token_gap: usize,
    token_total: usize,
    paradigm_total: usize,
    candidate_count: Option<usize>,
    seconds: f64,
) {
    let mut by_class: BTreeMap<(&str, &str), usize> = BTreeMap::new();
    for row in gap {
        *by_class.entry((row.oracle, row.reason)).or_default() += 1;
    }
    println!(
        "synodal-gold: token oracle {token_total} types, {token_gap} in gap; paradigm oracle \
         {paradigm_total} cells, {} in gap; total gap {} rows",
        gap.len() - token_gap,
        gap.len()
    );
    for ((oracle, reason), count) in &by_class {
        println!("  {oracle}\t{reason}\t{count}");
    }
    match candidate_count {
        Some(count) => println!("  defect candidates proposed (witness sweep): {count}"),
        None => println!("  witness sweep skipped: intermediate witnesses absent (CI mode)"),
    }
    println!("  replay runtime: {seconds:.1}s");
}

// ---------------------------------------------------------------------------
// Token oracle replay
// ---------------------------------------------------------------------------

pub(crate) struct TokenOracleRow {
    pub(crate) surface: String,
    pub(crate) non_lexical: String,
    pub(crate) references: Vec<String>,
    pub(crate) confirmed_readings: Vec<(LexemeId, String)>,
}

struct TokenFailure {
    surface: String,
    references: Vec<String>,
    engine_output: String,
}

struct Failure {
    reason: &'static str,
    engine_output: String,
}

pub(crate) fn load_token_oracle(root: &Path) -> Result<Vec<TokenOracleRow>, Box<dyn Error>> {
    let path = root.join(TOKEN_ORACLE_RELATIVE);
    let content =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    crate::synodal_gold_oracle::validate_committed(&content)
        .map_err(|error| format!("{TOKEN_ORACLE_RELATIVE}: {error}"))?;
    let mut rows = Vec::new();
    let mut saw_header = false;
    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if !saw_header {
            saw_header = true; // column header
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(format!("short token-oracle row: {line}").into());
        }
        let confirmed_readings = fields[4]
            .split(';')
            .filter(|pair| !pair.is_empty())
            .map(|pair| {
                let (lexeme, cell) = pair
                    .split_once('|')
                    .ok_or_else(|| format!("malformed confirmed reading {pair:?}"))?;
                Ok::<_, String>((LexemeId::from(lexeme), cell.to_owned()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(TokenOracleRow {
            surface: fields[0].to_owned(),
            non_lexical: fields[2].to_owned(),
            references: fields[3]
                .split(',')
                .filter(|reference| !reference.is_empty())
                .map(str::to_owned)
                .collect(),
            confirmed_readings,
        });
    }
    Ok(rows)
}

/// Replays one oracle surface type. Returns `None` on pass, or the typed
/// failure. Non-lexical rows gate on classification stability; lexical rows
/// need (a) at least one attested/normative reading and (b) a generation
/// round trip through one of them — through a confirmed reading specifically
/// when the row carries any.
fn replay_token(
    analyzer: &Analyzer,
    liturgical: Inflector,
    row: &TokenOracleRow,
) -> Option<Failure> {
    let surface = nfc(&row.surface);
    let tag = classify_non_lexical(analyzer, &surface);
    if !row.non_lexical.is_empty() {
        return if tag == Some(row.non_lexical.as_str()) {
            None
        } else {
            Some(Failure {
                reason: "non-lexical-unclassified",
                engine_output: tag.unwrap_or("").to_owned(),
            })
        };
    }
    if let Some(tag) = tag {
        return Some(Failure {
            reason: "non-lexical-unclassified",
            engine_output: tag.to_owned(),
        });
    }
    let analyses = analyzer
        .analyze_profile(&surface, OrthographyProfile::SynodalLiturgical)
        .unwrap_or_default();
    if analyses.is_empty() {
        return Some(Failure {
            reason: if has_titlo(&surface) {
                "abbreviation-unexpanded"
            } else {
                "unregistered-lemma"
            },
            engine_output: String::new(),
        });
    }
    let attested: Vec<&Analysis> = analyses
        .iter()
        .filter(|analysis| !is_prediction(analysis.source))
        .collect();
    if attested.is_empty() {
        return Some(Failure {
            reason: "unreviewed-cell",
            engine_output: analyses[0].matched_text.clone(),
        });
    }
    let candidates: Vec<&Analysis> = if row.confirmed_readings.is_empty() {
        attested.clone()
    } else {
        let confirmed: Vec<&Analysis> = attested
            .iter()
            .copied()
            .filter(|analysis| {
                analysis.cell.is_some_and(|cell| {
                    // Compatible-wildcard cell identity: the evaluation rows
                    // and the analyzer may record the same cell with `any` in
                    // place of a concrete animacy (or vice versa); the
                    // registry's own compatibility keys decide equivalence.
                    let keys = synodal_church_slavonic::grammar_cell_registry_keys(cell);
                    row.confirmed_readings.iter().any(|(id, cell_key)| {
                        id == analysis.lexeme.id()
                            && (keys.contains(cell_key)
                                || cell_key.parse::<GrammarCell>().is_ok_and(|confirmed| {
                                    synodal_church_slavonic::grammar_cell_registry_keys(confirmed)
                                        .iter()
                                        .any(|key| keys.contains(key))
                                }))
                    })
                })
            })
            .collect();
        if confirmed.is_empty() {
            // The confirmed reading is not among the analyses: generate it
            // directly so the failure names the engine's output for it.
            return Some(confirmed_reading_failure(liturgical, row, &surface));
        }
        confirmed
    };
    let mut best_output = String::new();
    let mut accent_near_miss = false;
    for analysis in &candidates {
        match round_trip(liturgical, analysis, &surface) {
            RoundTrip::Match => return None,
            RoundTrip::AccentMismatch(output) => {
                accent_near_miss = true;
                if best_output.is_empty() {
                    best_output = output;
                }
            }
            RoundTrip::Mismatch(output) => {
                if best_output.is_empty() {
                    best_output = output;
                }
            }
        }
    }
    Some(Failure {
        reason: if accent_near_miss {
            "engine-wrong-accent"
        } else {
            "engine-wrong-form"
        },
        engine_output: best_output,
    })
}

fn confirmed_reading_failure(
    liturgical: Inflector,
    row: &TokenOracleRow,
    surface: &str,
) -> Failure {
    let mut best_output = String::new();
    let mut accent_near_miss = false;
    let mut generated_any = false;
    for (lexeme_id, cell_key) in &row.confirmed_readings {
        let Ok(cell) = cell_key.parse::<GrammarCell>() else {
            continue;
        };
        let Ok(forms) = liturgical.form_by_id(lexeme_id, cell) else {
            continue;
        };
        for variant in forms.variants() {
            generated_any = true;
            if surfaces_match(surface, &variant.printed) {
                // The reading generates the surface but the analyzer does not
                // return it: an analysis-path bug, still an engine class.
                return Failure {
                    reason: "engine-wrong-form",
                    engine_output: variant.printed.clone(),
                };
            }
            if accentless_match(surface, &variant.printed) {
                accent_near_miss = true;
            }
            if best_output.is_empty() {
                best_output = variant.printed.clone();
            }
        }
    }
    if !generated_any {
        return Failure {
            reason: "unreviewed-cell",
            engine_output: String::new(),
        };
    }
    Failure {
        reason: if accent_near_miss {
            "engine-wrong-accent"
        } else {
            "engine-wrong-form"
        },
        engine_output: best_output,
    }
}

enum RoundTrip {
    Match,
    AccentMismatch(String),
    Mismatch(String),
}

/// Inflects one reading's lemma+cell under the liturgical profile and
/// compares the result against the printed surface within the contract's
/// equivalence classes (§3).
fn round_trip(liturgical: Inflector, analysis: &Analysis, surface: &str) -> RoundTrip {
    if analysis.source == AnalysisSource::AbbreviationExpansion {
        // §3.1: the engine's expanded reading is contracted for the attested
        // cell and compared exactly against the printed titlo surface.
        return abbreviation_round_trip(liturgical, surface);
    }
    let Some(cell) = analysis.cell else {
        return RoundTrip::Mismatch(analysis.matched_text.clone());
    };
    let Ok(forms) = liturgical.form_by_id(analysis.lexeme.id(), cell) else {
        return RoundTrip::Mismatch(analysis.matched_text.clone());
    };
    let mut accent_near_miss = false;
    let mut best = String::new();
    for variant in forms.variants() {
        let matched = if analysis.reflexive {
            // Alypy §73: the surface is host form + enclitic сѧ; the host must
            // round-trip against one of its reconstructible base candidates.
            reflexive_base_candidates(surface)
                .iter()
                .any(|host| surfaces_match(host, &variant.printed))
        } else {
            surfaces_match(surface, &variant.printed)
        };
        if matched {
            return RoundTrip::Match;
        }
        if !analysis.reflexive && accentless_match(surface, &variant.printed) {
            accent_near_miss = true;
        }
        if best.is_empty() {
            best = variant.printed.clone();
        }
    }
    if accent_near_miss {
        RoundTrip::AccentMismatch(best)
    } else {
        RoundTrip::Mismatch(best)
    }
}

fn abbreviation_round_trip(liturgical: Inflector, surface: &str) -> RoundTrip {
    let Ok(expansions) = abbreviation::expand(surface) else {
        return RoundTrip::Mismatch(String::new());
    };
    let mut best = String::new();
    for expansion in &expansions {
        let Ok(contractions) = abbreviation::contract_variants_for_cell_by_id_with(
            &expansion.lexeme_id,
            &expansion.sense_id,
            expansion.cell,
            liturgical,
        ) else {
            continue;
        };
        for contraction in contractions {
            if surfaces_match(surface, &contraction.printed) {
                return RoundTrip::Match;
            }
            if best.is_empty() {
                best = contraction.printed;
            }
        }
    }
    RoundTrip::Mismatch(best)
}

fn is_prediction(source: AnalysisSource) -> bool {
    matches!(
        source,
        AnalysisSource::CallerSpecifiedPrediction
            | AnalysisSource::InheritedPrediction
            | AnalysisSource::AnalogicalPrediction
    )
}

fn has_titlo(surface: &str) -> bool {
    surface
        .chars()
        .any(|character| matches!(character, '\u{0483}' | '\u{0487}' | '\u{2de0}'..='\u{2dff}'))
}

// ---------------------------------------------------------------------------
// Comparison within the contract's equivalence classes (§3)
// ---------------------------------------------------------------------------

pub(crate) fn nfc(value: &str) -> String {
    value.nfc().collect()
}

/// §3.3: the uk presentation fold — the printed digraph half `ᲂ` and the
/// monograph `ѹ`/`Ѹ` are presentations of the letter pair `оу`. Nothing else
/// is folded here.
pub(crate) fn fold_uk(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\u{1c82}' => output.push('о'),
            'ѹ' => output.push_str("оу"),
            'Ѹ' => output.push_str("Оу"),
            other => output.push(other),
        }
    }
    output
}

/// Exact NFC comparison, then the uk presentation fold (§3.3), then — only
/// for a surface printed capitalized (verse-initial presentation, §3.2) —
/// case-insensitively.
pub(crate) fn surfaces_match(oracle: &str, engine: &str) -> bool {
    let oracle = nfc(oracle);
    let engine = nfc(engine);
    if oracle == engine {
        return true;
    }
    let oracle_folded = fold_uk(&oracle);
    let engine_folded = fold_uk(&engine);
    if oracle_folded == engine_folded {
        return true;
    }
    if oracle.chars().next().is_some_and(char::is_uppercase) {
        return lowercase(&oracle_folded) == lowercase(&engine_folded);
    }
    false
}

pub(crate) fn lowercase(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).nfc().collect()
}

/// Whether the two surfaces agree once presentation accents and breathing are
/// removed, under the same case rule as [`surfaces_match`] (used only to
/// split `engine-wrong-accent` from `engine-wrong-form`; never a pass).
pub(crate) fn accentless_match(oracle: &str, engine: &str) -> bool {
    let stripped_oracle = strip_accents(oracle);
    let stripped_engine = strip_accents(engine);
    if stripped_oracle == stripped_engine {
        return true;
    }
    nfc(oracle).chars().next().is_some_and(char::is_uppercase)
        && lowercase(&stripped_oracle) == lowercase(&stripped_engine)
}

pub(crate) fn strip_accents(value: &str) -> String {
    fold_uk(&nfc(value))
        .nfd()
        .filter(|character| {
            !matches!(
                character,
                '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}' | '\u{0486}'
            )
        })
        .nfc()
        .collect()
}

// ---------------------------------------------------------------------------
// Paradigm oracle replay
// ---------------------------------------------------------------------------

pub(crate) struct ParadigmOracleRow {
    pub(crate) key: String,
    pub(crate) section: String,
    pub(crate) artifact: String,
    pub(crate) table_index: String,
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

pub(crate) fn load_paradigm_oracle(root: &Path) -> Result<Vec<ParadigmOracleRow>, Box<dyn Error>> {
    let path = root.join(PARADIGM_ORACLE_RELATIVE);
    let content =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut rows = Vec::new();
    let mut saw_header = false;
    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if !saw_header {
            saw_header = true;
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 13 {
            return Err(format!("short paradigm-oracle row: {line}").into());
        }
        rows.push(ParadigmOracleRow {
            key: format!(
                "{}|{}|{}|{}|{}|{}",
                fields[0],
                fields[1],
                fields[2],
                fields[4],
                fields[5],
                fields[6..12].join(":")
            ),
            section: fields[0].to_owned(),
            artifact: fields[1].to_owned(),
            table_index: fields[2].to_owned(),
            pos: fields[3].to_owned(),
            headword: fields[4].to_owned(),
            column_label: fields[5].to_owned(),
            case: fields[6].to_owned(),
            number: fields[7].to_owned(),
            gender: fields[8].to_owned(),
            person: fields[9].to_owned(),
            tense: fields[10].to_owned(),
            form: fields[11].to_owned(),
            surface: fields[12].to_owned(),
        });
    }
    Ok(rows)
}

/// The Alypy-typography equivalence classes of the normative contract's
/// paradigm section: hyphenated morpheme boundaries are stripped, and
/// parenthesised variants are alternates (a full-word alternate stands on its
/// own; a `-suffix` alternate replaces the matching ending of the base form).
/// A multi-word printed cell (periphrases, prepositional demonstrations,
/// adjective-plus-noun demonstration pairs) accepts the whole phrase or any
/// single word of it.
pub(crate) fn paradigm_expected_variants(surface: &str) -> Vec<String> {
    // §7.1 strips the stem-ending hyphen; the hyphen that opens a `-suffix`
    // alternate inside a parenthesis (§7.2) is part of that notation and
    // must survive to the alternate parser below.
    let mut dehyphenated = String::with_capacity(surface.len());
    let mut previous = None;
    for character in surface.chars() {
        if character != '-' || previous == Some('(') {
            dehyphenated.push(character);
        }
        previous = Some(character);
    }
    let mut variants: BTreeSet<String> = BTreeSet::new();
    let mut base = dehyphenated.clone();
    while let Some(open) = base.find('(') {
        let Some(close) = base[open..].find(')').map(|offset| open + offset) else {
            break;
        };
        let inner = base[open + 1..close].trim().to_owned();
        let head = base[..open].trim_end().to_owned();
        let tail = base[close + 1..].trim_start().to_owned();
        for alternate in inner.split(" или ") {
            let alternate = alternate.trim();
            if let Some(suffix) = alternate
                .strip_prefix('‑')
                .or_else(|| alternate.strip_prefix('-'))
            {
                variants.extend(apply_suffix_alternate(&head, suffix));
            } else if !alternate.is_empty() && !alternate.contains(' ') {
                variants.insert(alternate.to_owned());
                // An optional letter printed in place — `(н)е́мъ` — is the
                // same "either side matches" alternate: the word with the
                // parenthesised letters kept is accepted beside the word
                // without them.
                if !tail.is_empty() {
                    variants.insert(format!("{head}{alternate}{tail}"));
                }
            }
        }
        base = format!("{head}{tail}").trim().to_owned();
    }
    variants.insert(base.clone());
    for word in base.split_whitespace() {
        variants.insert(word.to_owned());
    }
    variants
        .into_iter()
        .filter(|variant| !variant.is_empty())
        .collect()
}

/// Applies a printed `-suffix` alternate to the last word of the base form.
/// The printed table does not say where the suffix attaches, so every split
/// point of the base word yields a candidate; the caller only ever uses these
/// as *additional* accepted variants, so an imperfect reconstruction can only
/// fail to accept, never falsely pass an unrelated form.
fn apply_suffix_alternate(head: &str, suffix: &str) -> Vec<String> {
    let Some(last_word) = head.split_whitespace().last() else {
        return Vec::new();
    };
    let characters: Vec<char> = last_word.chars().collect();
    (1..=characters.len())
        .map(|keep| {
            let stem: String = characters[..keep].iter().collect();
            format!("{stem}{suffix}")
        })
        .collect()
}

fn split_feature(value: &str) -> Vec<String> {
    value.split('+').map(str::to_owned).collect()
}

fn feature_or(value: &str, guesses: &[&str]) -> Vec<String> {
    if value.is_empty() {
        guesses.iter().map(|guess| (*guess).to_owned()).collect()
    } else {
        split_feature(value)
    }
}

/// Maps one Alypy row's feature codes onto candidate engine cell keys. The
/// oracle's tables do not encode every engine dimension (noun animacy,
/// adjective length, participle voice, numeral kind), so the unencoded
/// dimensions fan out over their closed value sets; a cell passes when any
/// candidate generates the printed surface.
pub(crate) fn candidate_cell_keys(row: &ParadigmOracleRow) -> Vec<String> {
    let mut keys = Vec::new();
    let cases = split_feature(&row.case);
    let numbers = feature_or(&row.number, &["singular", "dual", "plural"]);
    match row.pos.as_str() {
        "noun" => {
            for case in &cases {
                for number in &numbers {
                    for animacy in ["inanimate", "animate"] {
                        keys.push(format!("noun:{case}:{number}:{animacy}"));
                    }
                }
            }
        }
        "adjective" => {
            let genders = feature_or(&row.gender, &["masculine", "feminine", "neuter"]);
            for case in &cases {
                for number in &numbers {
                    for gender in &genders {
                        for animacy in ["any", "inanimate", "animate"] {
                            for length in ["long", "short"] {
                                for degree in ["positive", "comparative", "superlative"] {
                                    keys.push(format!(
                                        "adjective:{case}:{number}:{gender}:{animacy}:{length}:{degree}"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        "pronoun" => {
            let genders = feature_or(&row.gender, &["any", "masculine", "feminine", "neuter"]);
            for case in &cases {
                for number in &numbers {
                    for gender in &genders {
                        for person in ["none", "first", "second", "third"] {
                            for animacy in ["any", "inanimate", "animate"] {
                                keys.push(format!(
                                    "pronoun:{case}:{number}:{gender}:{person}:{animacy}"
                                ));
                            }
                        }
                    }
                }
            }
        }
        "numeral" => {
            let genders = feature_or(&row.gender, &["any", "masculine", "feminine", "neuter"]);
            for kind in ["cardinal", "ordinal"] {
                for case in &cases {
                    for number in &numbers {
                        for gender in &genders {
                            for animacy in ["any", "inanimate", "animate"] {
                                keys.push(format!(
                                    "numeral:{kind}:{case}:{number}:{gender}:{animacy}"
                                ));
                            }
                        }
                    }
                }
            }
        }
        "participle" => {
            let genders = feature_or(&row.gender, &["masculine", "feminine", "neuter"]);
            let tenses = feature_or(&row.tense, &["present", "past"]);
            for tense in &tenses {
                for voice in ["active", "passive"] {
                    for case in &cases {
                        for number in &numbers {
                            for gender in &genders {
                                for animacy in ["any", "inanimate", "animate"] {
                                    for length in ["long", "short"] {
                                        keys.push(format!(
                                            "participle:{tense}:{voice}:{case}:{number}:{gender}:{animacy}:{length}:positive"
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        "verb" => match row.form.as_str() {
            "imperative" => {
                for person in split_feature(&row.person) {
                    for number in &numbers {
                        keys.push(format!("imperative:{person}:{number}"));
                    }
                }
            }
            "l-participle" => {
                let genders = feature_or(&row.gender, &["masculine", "feminine", "neuter"]);
                for gender in &genders {
                    for number in &numbers {
                        keys.push(format!("l-participle:{gender}:{number}"));
                    }
                }
            }
            _ => {
                for person in split_feature(&row.person) {
                    for number in &numbers {
                        keys.push(format!("{}:{person}:{number}", row.tense));
                    }
                }
            }
        },
        _ => {}
    }
    keys
}

/// The Alypy headword, dehyphenated, with a parenthesised alternate dropped
/// and only the first word of a multi-word demonstration kept.
pub(crate) fn paradigm_lemma(headword: &str) -> Option<String> {
    let first = headword.split_whitespace().next()?;
    let first = first.split('(').next()?.trim();
    let lemma: String = first
        .chars()
        .filter(|character| *character != '-')
        .collect();
    if lemma.is_empty() { None } else { Some(lemma) }
}

/// One reviewed headword assignment for an Alypy table that prints no
/// resolvable exemplar (`data/synodal/gold_paradigm_headwords.tsv`): the
/// table's own printed forms identify the lexeme, and the assignment is
/// review data with the section as provenance. `column_label` empty matches
/// every column; `surface_prefix` (accentless, dehyphenated) separates the
/// lexemes of a table that interleaves several paradigms (§103).
struct HeadwordAssignment {
    section: String,
    artifact: String,
    table_index: String,
    headword: String,
    column_label: String,
    surface_prefix: String,
    lexeme: String,
}

#[derive(Default)]
pub(crate) struct ParadigmHeadwords {
    assignments: Vec<HeadwordAssignment>,
}

pub(crate) fn load_paradigm_headwords(root: &Path) -> Result<ParadigmHeadwords, Box<dyn Error>> {
    let path = root.join(PARADIGM_HEADWORDS_RELATIVE);
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ParadigmHeadwords::default());
        }
        Err(error) => return Err(format!("read {}: {error}", path.display()).into()),
    };
    let mut assignments = Vec::new();
    let mut saw_header = false;
    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if !saw_header {
            saw_header = true;
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 8 {
            return Err(format!("short paradigm-headword row: {line}").into());
        }
        assignments.push(HeadwordAssignment {
            section: fields[0].to_owned(),
            artifact: fields[1].to_owned(),
            table_index: fields[2].to_owned(),
            headword: fields[3].to_owned(),
            column_label: fields[4].to_owned(),
            surface_prefix: fields[5].to_owned(),
            lexeme: fields[6].to_owned(),
        });
    }
    Ok(ParadigmHeadwords { assignments })
}

impl ParadigmHeadwords {
    /// The reviewed lexeme (lemma or id) assigned to a row, if any.
    pub(crate) fn assigned(&self, row: &ParadigmOracleRow) -> Option<&str> {
        let surface = strip_accents(&row.surface.replace('-', ""));
        self.assignments
            .iter()
            .find(|assignment| {
                assignment.section == row.section
                    && assignment.artifact == row.artifact
                    && assignment.table_index == row.table_index
                    && assignment.headword == row.headword
                    && (assignment.column_label.is_empty()
                        || assignment.column_label == row.column_label)
                    && (assignment.surface_prefix.is_empty()
                        || surface.starts_with(&assignment.surface_prefix))
            })
            .map(|assignment| assignment.lexeme.as_str())
    }
}

/// The registered lexeme an Alypy row's headword resolves to: the reviewed
/// headword assignment for the table when one exists (an explicit review
/// outranks a lemma coincidence such as ка́ѧ resolving to its own exact-form
/// entry instead of the кі́й column it heads), else the printed headword as
/// a lemma (accented or accentless).
pub(crate) fn resolve_paradigm_lexeme(
    headwords: &ParadigmHeadwords,
    row: &ParadigmOracleRow,
) -> Option<synodal_church_slavonic::LexemeSummary> {
    if let Some(assigned) = headwords.assigned(row) {
        return if assigned.starts_with("synodal:") {
            synodal_church_slavonic::advanced::lookup_by_id(&LexemeId::from(assigned)).ok()
        } else {
            synodal_church_slavonic::lookup(assigned)
                .or_else(|_| synodal_church_slavonic::lookup(&strip_accents(assigned)))
                .ok()
        };
    }
    let lemma = paradigm_lemma(&row.headword)?;
    synodal_church_slavonic::lookup(&lemma)
        .or_else(|_| synodal_church_slavonic::lookup(&strip_accents(&lemma)))
        .ok()
}

fn replay_paradigm(
    liturgical: Inflector,
    headwords: &ParadigmHeadwords,
    row: &ParadigmOracleRow,
) -> Option<GapRow> {
    let gap = |reason: &'static str, engine_output: String| GapRow {
        oracle: "paradigm",
        key: row.key.clone(),
        reason,
        engine_output,
        expected: row.surface.clone(),
    };
    let Some(lexeme) = resolve_paradigm_lexeme(headwords, row) else {
        return Some(gap("unregistered-lemma", String::new()));
    };
    let expected = paradigm_expected_variants(&row.surface);
    let mut generated_any = false;
    let mut best = String::new();
    let mut accent_near_miss = false;
    for key in candidate_cell_keys(row) {
        let Ok(cell) = key.parse::<GrammarCell>() else {
            continue;
        };
        let Ok(forms) = liturgical.form_by_id(lexeme.id(), cell) else {
            continue;
        };
        for variant in forms.variants() {
            generated_any = true;
            for candidate in [&variant.printed, &variant.expanded] {
                if expected
                    .iter()
                    .any(|expectation| surfaces_match(expectation, candidate))
                {
                    return None;
                }
            }
            if expected
                .iter()
                .any(|expectation| accentless_match(expectation, &variant.printed))
            {
                accent_near_miss = true;
            }
            if best.is_empty() {
                best = variant.printed.clone();
            }
        }
    }
    if !generated_any {
        return Some(gap("unreviewed-cell", String::new()));
    }
    Some(if accent_near_miss {
        gap("engine-wrong-accent", best)
    } else {
        gap("engine-wrong-form", best)
    })
}

// ---------------------------------------------------------------------------
// Source-defect ledger and witness adjudication
// ---------------------------------------------------------------------------

struct DefectRow {
    passage: String,
    ponomar_surface: String,
}

fn load_defect_ledger(root: &Path) -> Result<Vec<DefectRow>, Box<dyn Error>> {
    let path = root.join(LEDGER_RELATIVE);
    let content =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut rows = Vec::new();
    let mut saw_header = false;
    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if !saw_header {
            saw_header = true;
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 6 {
            return Err(format!("short defect-ledger row: {line}").into());
        }
        rows.push(DefectRow {
            passage: fields[0].to_owned(),
            ponomar_surface: fields[1].to_owned(),
        });
    }
    Ok(rows)
}

fn witnesses_present(root: &Path) -> bool {
    root.join(CROSSWIRE_RELATIVE).exists() && root.join(WIKISOURCE_RELATIVE).exists()
}

/// Re-validates every committed ledger row's two-witness disagreement (the
/// ledger cannot quietly absorb engine bugs). Source-present-only: with the
/// witnesses absent the committed ledger is trusted as reviewed data.
fn validate_defect_ledger(root: &Path, ledger: &[DefectRow]) -> Result<(), Box<dyn Error>> {
    if ledger.is_empty() || !witnesses_present(root) {
        return Ok(());
    }
    let passages: BTreeSet<&str> = ledger.iter().map(|row| row.passage.as_str()).collect();
    let crosswire = witness_verses(&root.join(CROSSWIRE_RELATIVE), &passages)?;
    let wikisource = witness_verses(&root.join(WIKISOURCE_RELATIVE), &passages)?;
    for row in ledger {
        let sides_against = |verses: &BTreeMap<String, Vec<String>>| {
            verses.get(&row.passage).is_some_and(|tokens| {
                !tokens
                    .iter()
                    .any(|token| surfaces_match(&row.ponomar_surface, token))
            })
        };
        if !(sides_against(&crosswire) && sides_against(&wikisource)) {
            return Err(format!(
                "defect-ledger row for {} {:?} no longer shows two-witness disagreement",
                row.passage, row.ponomar_surface
            )
            .into());
        }
    }
    Ok(())
}

/// Loads every requested passage from one witness in a single streaming pass.
fn witness_verses(
    path: &Path,
    passages: &BTreeSet<&str>,
) -> Result<BTreeMap<String, Vec<String>>, Box<dyn Error>> {
    #[derive(serde::Deserialize)]
    struct VerseRecord {
        passage: String,
        normalized_spelling: String,
    }
    let file = fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut verses: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        let Some(passage) = extract_passage(&line) else {
            continue;
        };
        if !passages.contains(passage) {
            continue;
        }
        let record: VerseRecord = serde_json::from_str(&line)
            .map_err(|error| format!("parse {}: {error}", path.display()))?;
        let text = crate::synodal_gold_oracle::strip_apparatus(&record.normalized_spelling);
        let tokens = synodal_church_slavonic_dictionary::coverage::tokenize(&text)
            .into_iter()
            .map(|token| nfc(&token.original));
        verses.entry(record.passage).or_default().extend(tokens);
    }
    Ok(verses)
}

fn extract_passage(line: &str) -> Option<&str> {
    let start = line.find("\"passage\":\"")? + "\"passage\":\"".len();
    let end = line[start..].find('"')? + start;
    Some(&line[start..end])
}

/// Sweeps the engine-disagreement token rows against both witnesses and
/// proposes source-defect ledger candidates for human review. Per the
/// contract's two-of-three rule the sweep never auto-adds ledger rows and
/// never changes the gap; the CrossWire witness is modernized civil spelling,
/// so it adjudicates word presence only, and the proposal criterion is that
/// the Wikisource witness attests the engine's surface and not Ponomar's.
/// Returns `None` when the witnesses are absent (CI).
fn witness_sweep(root: &Path, failures: &[TokenFailure]) -> Result<Option<usize>, Box<dyn Error>> {
    let candidates_path = root.join(DEFECT_CANDIDATES_RELATIVE);
    let header = "# synodal-gold-defect-candidates.tsv — witness-sweep proposals for the source-defect\n\
                  # ledger (data/synodal/gold_source_defects.tsv). Human review required before any row\n\
                  # moves to the ledger; this report is regenerated by cargo xtask synodal-gold only\n\
                  # where the intermediate witnesses are present, and is left untouched otherwise (CI).\n\
                  passage\tponomar_surface\tengine_surface\tcrosswire_reading\twikisource_reading\tnote\n";
    if !witnesses_present(root) {
        // Leave the committed report untouched: rewriting it without witness
        // data would dirty the generated tree in environments (CI) that
        // cannot reproduce the sweep.
        return Ok(None);
    }
    let mut passages: BTreeSet<&str> = BTreeSet::new();
    for failure in failures {
        if failure.engine_output.is_empty() {
            continue;
        }
        passages.extend(failure.references.iter().map(String::as_str));
    }
    let crosswire = witness_verses(&root.join(CROSSWIRE_RELATIVE), &passages)?;
    let wikisource = witness_verses(&root.join(WIKISOURCE_RELATIVE), &passages)?;
    let mut candidates = Vec::new();
    for failure in failures {
        if failure.engine_output.is_empty() {
            continue;
        }
        for reference in &failure.references {
            let Some(wiki_tokens) = wikisource.get(reference) else {
                continue;
            };
            let wiki_has_ponomar = wiki_tokens
                .iter()
                .any(|token| surfaces_match(&failure.surface, token));
            let wiki_has_engine = wiki_tokens
                .iter()
                .any(|token| surfaces_match(&failure.engine_output, token));
            if wiki_has_ponomar || !wiki_has_engine {
                continue;
            }
            let crosswire_reading = crosswire
                .get(reference)
                .map(|tokens| tokens.join(" "))
                .unwrap_or_default();
            candidates.push(format!(
                "{reference}\t{}\t{}\t{}\t{}\twikisource attests the engine surface and not Ponomar's; crosswire is civil-spelling word-presence evidence only",
                failure.surface,
                failure.engine_output,
                crosswire_reading,
                wiki_tokens.join(" ")
            ));
            break;
        }
    }
    candidates.sort();
    candidates.dedup();
    let mut content = header.to_owned();
    for candidate in &candidates {
        content.push_str(candidate);
        content.push('\n');
    }
    fs::write(&candidates_path, content)?;
    Ok(Some(candidates.len()))
}

/// The check_structure hook: the gate in check mode.
pub(crate) fn check(root: &Path) -> Result<(), Box<dyn Error>> {
    gate(root, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_matching_applies_only_enumerated_equivalences() {
        assert!(surfaces_match("сло́во", "сло́во"));
        // §3.3 initial-uk presentation
        assert!(surfaces_match("ᲂу҆слы́ша", "ѹ҆слы́ша"));
        assert!(surfaces_match("ᲂу҆слы́ша", "оу҆слы́ша"));
        // §3.2 verse-initial capitalization folds only for capitalized surfaces
        assert!(surfaces_match("Сло́во", "сло́во"));
        assert!(!surfaces_match("сло́во", "Сло́во"));
        // accents and positional letter choices compare exactly
        assert!(!surfaces_match("слово", "сло́во"));
        assert!(!surfaces_match("є҆стество̀", "е҆стество̀"));
        assert!(accentless_match("слово", "сло́во"));
        assert!(!accentless_match("слово", "словъ"));
    }

    #[test]
    fn paradigm_in_place_parenthetical_accepts_both_readings() {
        let variants = paradigm_expected_variants("(н)е́мъ");
        assert!(variants.iter().any(|variant| variant == "не́мъ"));
        assert!(variants.iter().any(|variant| variant == "е́мъ"));
        let suffix = paradigm_expected_variants("цар-ѝ (-е́й)");
        assert!(suffix.iter().any(|variant| nfc(variant) == nfc("царе́й")));
        assert!(suffix.iter().any(|variant| nfc(variant) == nfc("царѝ")));
    }

    #[test]
    fn paradigm_variants_strip_hyphens_and_accept_alternates() {
        assert_eq!(paradigm_expected_variants("ра́б-ъ"), vec!["ра́бъ".to_owned()]);
        let with_alternate = paradigm_expected_variants("цар-ѝ (-їе)");
        assert!(with_alternate.contains(&"царѝ".to_owned()));
        assert!(with_alternate.iter().any(|variant| variant.ends_with("їе")));
        let phrase = paradigm_expected_variants("хощꙋ̀ бы́ти");
        assert!(phrase.contains(&"хощꙋ̀ бы́ти".to_owned()));
        assert!(phrase.contains(&"бы́ти".to_owned()));
    }

    #[test]
    fn paradigm_lemma_takes_first_dehyphenated_word() {
        assert_eq!(paradigm_lemma("бж҃ї-й кра́-й"), Some("бж҃їй".to_owned()));
        assert_eq!(paradigm_lemma("ра́б-ъ"), Some("ра́бъ".to_owned()));
        assert_eq!(paradigm_lemma(""), None);
    }

    #[test]
    fn gap_rows_render_deterministically() {
        let row = GapRow {
            oracle: "token",
            key: "сло́во".into(),
            reason: "unregistered-lemma",
            engine_output: String::new(),
            expected: "сло́во".into(),
        };
        assert_eq!(row.render(), "token\tсло́во\tunregistered-lemma\t\tсло́во");
    }
}
