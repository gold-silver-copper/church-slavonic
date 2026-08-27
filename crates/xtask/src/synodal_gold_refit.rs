//! `synodal-gold refit`: accent-paradigm admissions for lexemes that are
//! already registered but whose Alypy table cells fail because no reviewed
//! accent rule reaches them (`unreviewed-cell`: the liturgical profile
//! refuses a cell without an accent rule). The printed table is the
//! evidence: for every such cell the accent position the table prints is
//! read off against the engine's accentless expanded form, the smallest
//! reviewable scope that covers exactly the proved cells is chosen, and the
//! rule is verified through the registry path before it is written. Cells an
//! existing (Bible-fitted) rule already reaches are never touched: a
//! disagreement there is a two-source question for the human-review file,
//! not a refit.
//!
//! Cells whose printed surface is a comma-separated alternate list
//! (`нес-е́-ва, -ѣ`) are contract-blocked (`docs/SYNODAL_GOLD_ORACLE.md` §7
//! admits parenthesised alternates only; the question is filed in the
//! human-review file). Their accents are still fitted from the first
//! alternate so the paradigm is complete, but they are never counted as
//! cleared and never decide an admission.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::time::Instant;

use sha2::{Digest, Sha256};
use synodal_church_slavonic::core::MetadataField;
use synodal_church_slavonic::{GrammarCell, Inflector, LexemeId, OrthographyProfile};
use unicode_normalization::UnicodeNormalization;

use crate::synodal_gold::{
    self, ParadigmOracleRow, candidate_cell_keys, committed_rows, load_paradigm_headwords,
    load_paradigm_oracle, paradigm_expected_variants, resolve_paradigm_lexeme, strip_accents,
    surfaces_match,
};
use crate::synodal_gold_burndown::{
    AccentRuleSpec, Admission, ArtifactBuilder, ClassSpec, Cluster, ClusterStatus,
    HUMAN_REVIEW_RELATIVE, candidate_index, land_admissions, loose_key, print_admit_summary,
};

const ALYPY_SOURCE: &str = "alypy-gamanovich-grammar-web-2023";
const REFIT_REPORT_RELATIVE: &str = "reports/synodal-gold-refit.tsv";

struct RefitOptions {
    classes: BTreeSet<String>,
    lemmas: BTreeSet<String>,
    take: Option<usize>,
    dry_run: bool,
    file_review: bool,
}

fn parse_options(args: &mut impl Iterator<Item = String>) -> Result<RefitOptions, Box<dyn Error>> {
    let mut options = RefitOptions {
        classes: BTreeSet::new(),
        lemmas: BTreeSet::new(),
        take: None,
        dry_run: false,
        file_review: false,
    };
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--only" | "--class" => {
                options
                    .classes
                    .insert(args.next().ok_or("--only requires a gap class")?);
            }
            "--lemma" => {
                options
                    .lemmas
                    .insert(args.next().ok_or("--lemma requires a lemma or id")?);
            }
            "--take" => {
                options.take = Some(args.next().ok_or("--take requires a number")?.parse()?);
            }
            "--dry-run" => options.dry_run = true,
            "--file-review" => options.file_review = true,
            other => return Err(format!("unknown synodal-gold refit option: {other}").into()),
        }
    }
    if options.classes.is_empty() {
        options.classes.extend(
            [
                "unreviewed-cell",
                "unregistered-lemma",
                "engine-wrong-accent",
                "engine-wrong-form",
            ]
            .map(str::to_owned),
        );
    }
    Ok(options)
}

/// One failing paradigm row of a registered lexeme, with what the refit
/// learned about it.
struct TargetRow {
    key: String,
    section: String,
    expected: Vec<String>,
    /// The printed surface is a comma-separated alternate list (contract §7
    /// question): fitted from its first alternate, never counted as cleared.
    blocked: bool,
    /// An existing rule already generates the printed base form: only the
    /// contract question stands between the row and a pass.
    reproduced_by_existing: bool,
    /// Candidate cells no existing accent rule reaches, with the rules the
    /// printed accent proves for each (preference order).
    refittable: Vec<(GrammarCell, Vec<AccentRuleSpec>)>,
    /// Why the row cannot be refitted, when it cannot.
    note: String,
}

struct Target {
    id: String,
    lemma: String,
    rows: Vec<TargetRow>,
}

/// The finest reviewable accent scope of a cell (the scope grammar of
/// `data/synodal/accent_paradigms.tsv`), or `None` for a cell kind the
/// reviewed data cannot scope.
fn finest_scope(cell: GrammarCell) -> Option<ScopeShape> {
    let shape = |kind: &'static str, fixed: Vec<&'static str>, number: &'static str| ScopeShape {
        kind,
        fixed,
        number,
        cases: vec![],
        genders: vec![],
        animacies: vec![],
        persons: vec![],
    };
    let animacy = |code: &'static str| match code {
        "animate" => Some("animate"),
        "inanimate" => Some("inanimate"),
        _ => None,
    };
    Some(match cell {
        GrammarCell::Noun(cell) => ScopeShape {
            cases: vec![cell.case.code()],
            ..shape("noun", vec![], cell.number.code())
        },
        GrammarCell::Adjective(cell) | GrammarCell::Determiner(cell) => ScopeShape {
            cases: vec![cell.case.code()],
            genders: vec![cell.gender.code()],
            animacies: vec![animacy(cell.animacy.code())?],
            ..shape(
                "adjective-agreeing",
                vec![cell.form.code(), cell.comparison.code()],
                cell.number.code(),
            )
        },
        GrammarCell::Pronoun(cell) => match cell.gender {
            Some(gender) => ScopeShape {
                cases: vec![cell.case.code()],
                genders: vec![gender.code()],
                animacies: vec![animacy(cell.animacy.code())?],
                ..shape("pronoun-agreeing", vec![], cell.number.code())
            },
            None => ScopeShape {
                cases: vec![cell.case.code()],
                ..shape("pronoun", vec![], cell.number.code())
            },
        },
        GrammarCell::Numeral(cell) => ScopeShape {
            cases: vec![cell.case.code()],
            ..shape("numeral", vec![], cell.number.code())
        },
        GrammarCell::FiniteVerb(cell) => ScopeShape {
            persons: vec![cell.person.code()],
            ..shape("finite", vec![cell.tense.code()], cell.number.code())
        },
        GrammarCell::Imperative(cell) => ScopeShape {
            persons: vec![cell.person.code()],
            ..shape("imperative", vec![], cell.number.code())
        },
        GrammarCell::LParticiple(cell) => ScopeShape {
            genders: vec![cell.gender.code()],
            ..shape("l-participle", vec![], cell.number.code())
        },
        GrammarCell::Participle(cell) => shape(
            "participle",
            vec![
                cell.tense.code(),
                cell.voice.code(),
                cell.agreement.form.code(),
                cell.agreement.comparison.code(),
            ],
            cell.agreement.number.code(),
        ),
        _ => return None,
    })
}

/// A reviewable scope as its dimensions, so proved cells with one rule can
/// be merged along a dimension into one row.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ScopeShape {
    kind: &'static str,
    fixed: Vec<&'static str>,
    number: &'static str,
    cases: Vec<&'static str>,
    genders: Vec<&'static str>,
    animacies: Vec<&'static str>,
    persons: Vec<&'static str>,
}

impl ScopeShape {
    fn code(&self) -> String {
        let mut parts: Vec<String> = vec![self.kind.to_owned()];
        parts.extend(self.fixed.iter().map(|part| (*part).to_owned()));
        parts.push(self.number.to_owned());
        match self.kind {
            "noun" | "pronoun" | "numeral" => parts.push(self.cases.join(",")),
            "adjective-agreeing" | "pronoun-agreeing" => {
                parts.push(self.cases.join(","));
                parts.push(self.genders.join(","));
                parts.push(self.animacies.join(","));
            }
            "finite" | "imperative" => parts.push(self.persons.join(",")),
            "l-participle" => parts.push(self.genders.join(",")),
            _ => {}
        }
        parts.join(":")
    }

    /// Merges two shapes that differ in exactly one list dimension.
    fn merge(&self, other: &Self) -> Option<Self> {
        if self.kind != other.kind || self.fixed != other.fixed || self.number != other.number {
            return None;
        }
        let differing = usize::from(self.cases != other.cases)
            + usize::from(self.genders != other.genders)
            + usize::from(self.animacies != other.animacies)
            + usize::from(self.persons != other.persons);
        if differing != 1 {
            return None;
        }
        let union = |left: &[&'static str], right: &[&'static str]| -> Vec<&'static str> {
            let mut merged: Vec<&'static str> = left.to_vec();
            for item in right {
                if !merged.contains(item) {
                    merged.push(item);
                }
            }
            merged
        };
        Some(Self {
            kind: self.kind,
            fixed: self.fixed.clone(),
            number: self.number,
            cases: union(&self.cases, &other.cases),
            genders: union(&self.genders, &other.genders),
            animacies: union(&self.animacies, &other.animacies),
            persons: union(&self.persons, &other.persons),
        })
    }
}

fn is_vowel(character: char) -> bool {
    matches!(
        character,
        'а' | 'е'
            | 'є'
            | 'ё'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѻ'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
            | 'э'
            | 'ю'
            | 'я'
            | 'ꙗ'
            | 'ѧ'
            | 'ѩ'
            | 'ѣ'
            | 'ѥ'
            | 'ѫ'
            | 'ѭ'
            | 'ѵ'
    )
}

/// Reads the printed accent off an expected surface: the vowel index (from
/// the word start, the initial uk digraph lead excluded as in the engine)
/// and the mark. `None` unless exactly one accent mark is printed.
fn printed_accent(expected: &str) -> Option<(usize, usize, &'static str)> {
    let folded = synodal_gold::fold_uk(&synodal_gold::nfc(expected));
    let mut letters: Vec<char> = Vec::new();
    let mut accents: Vec<(usize, &'static str)> = Vec::new();
    for character in folded.nfd() {
        match character {
            '\u{0301}' => accents.push((letters.len(), "acute")),
            '\u{0300}' => accents.push((letters.len(), "grave")),
            '\u{0311}' => accents.push((letters.len(), "kamora")),
            '\u{0484}' | '\u{0486}' | '\u{0483}' => {}
            other if unicode_normalization::char::is_combining_mark(other) => {}
            other => letters.push(other.to_lowercase().next().unwrap_or(other)),
        }
    }
    let [(after, mark)] = accents.as_slice() else {
        return None;
    };
    let accented = after.checked_sub(1)?;
    let lead_uk = letters.first() == Some(&'о') && letters.get(1) == Some(&'у');
    let vowels: Vec<usize> = letters
        .iter()
        .enumerate()
        .filter(|(index, character)| is_vowel(**character) && !(lead_uk && *index == 0))
        .map(|(index, _)| index)
        .collect();
    let position = vowels.iter().position(|index| *index == accented)?;
    Some((position, vowels.len(), mark))
}

/// The rules the printed accent proves for one cell, in preference order:
/// stem-relative placement for a stem vowel, ending-relative for an ending
/// vowel, word-relative as the fallback; a printed final grave is the
/// language-wide varia of an acute rule, so acute is tried first.
fn proved_rules(expected: &str, stem_vowels: Option<usize>) -> Vec<(String, String)> {
    let Some((position, total, mark)) = printed_accent(expected) else {
        return Vec::new();
    };
    let mut placements: Vec<String> = Vec::new();
    match stem_vowels {
        Some(stem) if position < stem => {
            placements.push(format!("stem-vowel-from-start:{position}"));
        }
        Some(_) => placements.push(format!("ending-vowel-from-end:{}", total - 1 - position)),
        None => {}
    }
    placements.push(format!("word-vowel-from-start:{position}"));
    let marks: Vec<&str> = match mark {
        "grave" if position + 1 == total => vec!["acute", "grave"],
        other => vec![other],
    };
    let mut rules = Vec::new();
    for placement in &placements {
        for mark in &marks {
            rules.push((placement.clone(), (*mark).to_owned()));
        }
    }
    rules
}

/// A comma-separated alternate list outside parentheses (contract §7 admits
/// parenthesised alternates only).
fn is_comma_alternate(surface: &str) -> bool {
    let mut depth = 0i32;
    let mut previous = ' ';
    for character in surface.chars() {
        match character {
            '(' => depth += 1,
            ')' => depth -= 1,
            ' ' if previous == ',' && depth == 0 => return true,
            _ => {}
        }
        previous = character;
    }
    false
}

/// The expected variants a fit is judged against: the contract's variants,
/// plus (for a blocked comma list) the words of the list so the accent can
/// be fitted from the printed base form.
fn fitting_variants(surface: &str) -> (Vec<String>, bool) {
    let variants = paradigm_expected_variants(surface);
    if !is_comma_alternate(surface) {
        return (variants, false);
    }
    let mut widened = variants;
    for word in surface.split(", ") {
        let word = word.trim().trim_end_matches(',');
        if word.starts_with('-') || word.is_empty() {
            continue;
        }
        widened.extend(paradigm_expected_variants(word));
    }
    widened.sort();
    widened.dedup();
    (widened, true)
}

fn lexeme_stems(root: &Path) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let content = fs::read_to_string(root.join("data/synodal/lexemes.tsv"))?;
    Ok(content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            (fields.len() >= 5).then(|| (fields[0].to_owned(), fields[4].to_owned()))
        })
        .collect())
}

fn row_passes(
    liturgical: Inflector,
    id: &LexemeId,
    cells: &[GrammarCell],
    expected: &[String],
) -> bool {
    cells.iter().any(|cell| {
        liturgical.form_by_id(id, *cell).is_ok_and(|forms| {
            forms.variants().iter().any(|variant| {
                [&variant.printed, &variant.expanded].iter().any(|output| {
                    expected
                        .iter()
                        .any(|expectation| surfaces_match(expectation, output))
                })
            })
        })
    })
}

/// Collects the refit targets: every failing paradigm row (in the selected
/// classes) whose headword resolves to a registered lexeme, grouped by
/// lexeme, with the cells the printed accent can prove.
fn collect_targets(
    root: &Path,
    options: &RefitOptions,
) -> Result<Vec<Target>, Box<dyn Error>> {
    let committed = committed_rows(root)?;
    let failing: BTreeMap<&str, &str> = committed
        .iter()
        .filter(|(oracle, _, reason)| oracle == "paradigm" && options.classes.contains(reason))
        .map(|(_, key, reason)| (key.as_str(), reason.as_str()))
        .collect();
    let paradigm_rows = load_paradigm_oracle(root)?;
    let headwords = load_paradigm_headwords(root)?;
    let stems = lexeme_stems(root)?;
    let expanded = Inflector::builder()
        .orthography(OrthographyProfile::Expanded)
        .build();
    let liturgical = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    let mut targets: BTreeMap<String, Target> = BTreeMap::new();
    for row in &paradigm_rows {
        if !failing.contains_key(row.key.as_str()) {
            continue;
        }
        let Some(lexeme) = resolve_paradigm_lexeme(&headwords, row) else {
            continue;
        };
        let id = lexeme.id().as_str().to_owned();
        if !options.lemmas.is_empty()
            && !options.lemmas.contains(&id)
            && !options.lemmas.contains(lexeme.lemma())
            && !options
                .lemmas
                .iter()
                .any(|key| strip_accents(key) == lexeme.lemma())
        {
            continue;
        }
        let stem_vowels = stems
            .get(&id)
            .filter(|stem| !stem.is_empty())
            .map(|stem| stem.chars().filter(|character| is_vowel(*character)).count());
        let target = targets.entry(id.clone()).or_insert_with(|| Target {
            id: id.clone(),
            lemma: lexeme.lemma().to_owned(),
            rows: Vec::new(),
        });
        target
            .rows
            .push(inspect_row(expanded, liturgical, lexeme.id(), row, stem_vowels));
    }
    let mut targets: Vec<Target> = targets.into_values().collect();
    if let Some(take) = options.take {
        targets.truncate(take);
    }
    Ok(targets)
}

fn inspect_row(
    expanded: Inflector,
    liturgical: Inflector,
    id: &LexemeId,
    row: &ParadigmOracleRow,
    stem_vowels: Option<usize>,
) -> TargetRow {
    let (expected, blocked) = fitting_variants(&row.surface);
    let mut refittable: Vec<(GrammarCell, Vec<AccentRuleSpec>)> = Vec::new();
    let mut reached_by_existing = false;
    let mut reproduced_by_existing = false;
    let mut generated_nothing = true;
    for key in candidate_cell_keys(row) {
        let Ok(cell) = key.parse::<GrammarCell>() else {
            continue;
        };
        match liturgical.form_by_id(id, cell) {
            Ok(forms) => {
                reached_by_existing = true;
                if forms.variants().iter().any(|variant| {
                    [&variant.printed, &variant.expanded].iter().any(|output| {
                        expected
                            .iter()
                            .any(|expectation| surfaces_match(expectation, output))
                    })
                }) {
                    reproduced_by_existing = true;
                }
                continue;
            }
            Err(synodal_church_slavonic::Error::OrthographicMetadataRequired {
                field: MetadataField::AccentParadigm,
            }) => {}
            Err(_) => continue,
        }
        let Ok(forms) = expanded.form_by_id(id, cell) else {
            continue;
        };
        generated_nothing = false;
        let Some(shape) = finest_scope(cell) else {
            continue;
        };
        let mut rules: Vec<AccentRuleSpec> = Vec::new();
        for variant in forms.variants() {
            for expectation in &expected {
                if loose_key(expectation) != loose_key(&variant.expanded) {
                    continue;
                }
                for (placement, mark) in proved_rules(expectation, stem_vowels) {
                    let rule = AccentRuleSpec {
                        scope: shape.code(),
                        placement,
                        mark,
                    };
                    if !rules.contains(&rule) {
                        rules.push(rule);
                    }
                }
            }
        }
        if !rules.is_empty() {
            refittable.push((cell, rules));
        }
    }
    let note = if !refittable.is_empty() || (blocked && reproduced_by_existing) {
        String::new()
    } else if reached_by_existing {
        "an existing accent rule reaches the cell and disagrees with the print".into()
    } else if generated_nothing {
        "the class generates no candidate cell".into()
    } else {
        "no candidate cell's accentless form matches the print".into()
    };
    TargetRow {
        key: row.key.clone(),
        section: row.section.clone(),
        expected,
        blocked,
        reproduced_by_existing,
        refittable,
        note,
    }
}

/// One scope group under fit: the proved cells, their rows, and the rules
/// still to try.
struct Group {
    shape: ScopeShape,
    cells: Vec<GrammarCell>,
    rows: Vec<usize>,
    candidates: Vec<AccentRuleSpec>,
    chosen: Option<AccentRuleSpec>,
}

/// The fit for one lexeme: rules per merged scope, the rows they clear, the
/// rows left as residue with a reason.
struct Fit {
    rules: Vec<AccentRuleSpec>,
    cleared_rows: Vec<usize>,
    blocked_rows: Vec<usize>,
    residue: Vec<(usize, String)>,
}

fn fit_target(root: &Path, target: &Target, liturgical: Inflector) -> Result<Fit, Box<dyn Error>> {
    let id = LexemeId::from(target.id.as_str());
    // Group the proved cells by finest scope; a group's candidates are the
    // rules every one of its cells proves, in preference order.
    let mut groups: BTreeMap<ScopeShape, Group> = BTreeMap::new();
    let mut residue: Vec<(usize, String)> = Vec::new();
    let mut blocked_rows: Vec<usize> = Vec::new();
    for (index, row) in target.rows.iter().enumerate() {
        if row.refittable.is_empty() {
            if row.blocked && row.reproduced_by_existing {
                blocked_rows.push(index);
            } else {
                residue.push((index, row.note.clone()));
            }
            continue;
        }
        for (cell, rules) in &row.refittable {
            let shape = finest_scope(*cell).expect("refittable cells have a scope");
            let group = groups.entry(shape.clone()).or_insert_with(|| Group {
                shape,
                cells: Vec::new(),
                rows: Vec::new(),
                candidates: rules.clone(),
                chosen: None,
            });
            group.cells.push(*cell);
            if !group.rows.contains(&index) {
                group.rows.push(index);
            }
            group
                .candidates
                .retain(|candidate| rules.iter().any(|rule| rule == candidate));
        }
    }
    let mut groups: Vec<Group> = groups.into_values().collect();
    // Rounds: install every pending group's next candidate at once and keep
    // the groups whose rows all pass; a group is settled by the first rule
    // that reproduces every printed cell it covers.
    let mut round = 0usize;
    loop {
        let pending: Vec<usize> = groups
            .iter()
            .enumerate()
            .filter(|(_, group)| group.chosen.is_none() && group.candidates.len() > round)
            .map(|(index, _)| index)
            .collect();
        if pending.is_empty() {
            break;
        }
        let mut builder = ArtifactBuilder::load(root)?;
        for &index in &pending {
            builder.accent(&target.id, &groups[index].candidates[round]);
        }
        builder.install()?;
        for &index in &pending {
            let group = &groups[index];
            let passes = group.rows.iter().all(|row_index| {
                let row = &target.rows[*row_index];
                let cells: Vec<GrammarCell> = row
                    .refittable
                    .iter()
                    .map(|(cell, _)| *cell)
                    .filter(|cell| group.cells.contains(cell))
                    .collect();
                row_passes(liturgical, &id, &cells, &row.expected)
            });
            if passes {
                groups[index].chosen = Some(group.candidates[round].clone());
            }
        }
        round += 1;
    }
    // Merge settled groups sharing one rule along one dimension.
    let mut merged: Vec<(ScopeShape, AccentRuleSpec, Vec<usize>)> = groups
        .iter()
        .filter_map(|group| {
            group
                .chosen
                .clone()
                .map(|rule| (group.shape.clone(), rule, group.rows.clone()))
        })
        .collect();
    let mut changed = true;
    while changed {
        changed = false;
        'outer: for left in 0..merged.len() {
            for right in left + 1..merged.len() {
                let same_rule = merged[left].1.placement == merged[right].1.placement
                    && merged[left].1.mark == merged[right].1.mark;
                if !same_rule {
                    continue;
                }
                if let Some(shape) = merged[left].0.merge(&merged[right].0) {
                    let (_, rule, rows_right) = merged.remove(right);
                    merged[left].0 = shape;
                    for row in rows_right {
                        if !merged[left].2.contains(&row) {
                            merged[left].2.push(row);
                        }
                    }
                    merged[left].1 = AccentRuleSpec {
                        scope: merged[left].0.code(),
                        ..rule
                    };
                    changed = true;
                    break 'outer;
                }
            }
        }
    }
    let rules: Vec<AccentRuleSpec> = merged
        .iter()
        .map(|(shape, rule, _)| AccentRuleSpec {
            scope: shape.code(),
            placement: rule.placement.clone(),
            mark: rule.mark.clone(),
        })
        .collect();
    // Confirm the merged rules together through the registry path.
    let mut builder = ArtifactBuilder::load(root)?;
    for rule in &rules {
        builder.accent(&target.id, rule);
    }
    builder.install()?;
    let mut cleared_rows = Vec::new();
    for (index, row) in target.rows.iter().enumerate() {
        if row.refittable.is_empty() {
            continue;
        }
        let cells: Vec<GrammarCell> = row.refittable.iter().map(|(cell, _)| *cell).collect();
        if !row_passes(liturgical, &id, &cells, &row.expected) {
            let unsettled = groups
                .iter()
                .filter(|group| group.chosen.is_none() && group.rows.contains(&index))
                .map(|group| group.shape.code())
                .collect::<Vec<_>>();
            residue.push((
                index,
                if unsettled.is_empty() {
                    "the proved rules do not combine".to_owned()
                } else {
                    format!(
                        "no single rule reproduces every printed cell of scope {}",
                        unsettled.join(" ")
                    )
                },
            ));
        } else if row.blocked {
            blocked_rows.push(index);
        } else {
            cleared_rows.push(index);
        }
    }
    residue.sort();
    Ok(Fit {
        rules,
        cleared_rows,
        blocked_rows,
        residue,
    })
}

fn refit_cluster_id(target: &Target) -> String {
    let digest = Sha256::digest(format!("refit|{}", target.id).as_bytes());
    let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("gold-refit-{}", &hex[..12])
}

pub(crate) fn refit(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let options = parse_options(args)?;
    let started = Instant::now();
    let targets = collect_targets(root, &options)?;
    let liturgical = Inflector::builder()
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    let candidates = candidate_index(root)?;
    let mut admissions: Vec<Admission> = Vec::new();
    let mut rejected: Vec<(String, Vec<String>)> = Vec::new();
    let mut report = String::from(
        "# synodal-gold-refit.tsv — accent-paradigm refits of registered lexemes from their Alypy\n\
         # tables (cargo xtask synodal-gold refit). One row per lexeme and per residue cell.\n\
         lexeme_id\tlemma\tstatus\trules\tcleared\tblocked\tresidue\tdetail\n",
    );
    let mut review_rows: Vec<String> = Vec::new();
    let filed = fs::read_to_string(root.join(HUMAN_REVIEW_RELATIVE)).unwrap_or_default();
    let mut total_cleared = 0usize;
    let mut total_blocked = 0usize;
    let mut total_residue = 0usize;
    for target in &targets {
        let fit = fit_target(root, target, liturgical)?;
        total_cleared += fit.cleared_rows.len();
        total_blocked += fit.blocked_rows.len();
        total_residue += fit.residue.len();
        let rules: Vec<String> = fit
            .rules
            .iter()
            .map(|rule| format!("{}|{}|{}", rule.scope, rule.placement, rule.mark))
            .collect();
        let status = if fit.rules.is_empty() {
            "unfit"
        } else if fit.residue.is_empty() {
            "fit"
        } else {
            "partial"
        };
        let _ = writeln!(
            report,
            "{}\t{}\t{status}\t{}\t{}\t{}\t{}\t",
            target.id,
            target.lemma,
            rules.join(";"),
            fit.cleared_rows.len(),
            fit.blocked_rows.len(),
            fit.residue.len()
        );
        for (index, note) in &fit.residue {
            let row = &target.rows[*index];
            let _ = writeln!(
                report,
                "{}\t{}\tresidue\t\t\t\t{}\t{note} (printed {})",
                target.id,
                target.lemma,
                row.key,
                row.expected.join(" / ")
            );
        }
        for index in &fit.blocked_rows {
            let row = &target.rows[*index];
            if filed.contains(&format!("paradigm\t{}\t", row.key)) {
                continue;
            }
            review_rows.push(format!(
                "paradigm\t{}\t{}\t{}\t{}\tComma-separated alternate list (see на́шѧ): the accent paradigm now generates the printed base form, but §7.2 admits parenthesised alternates only.",
                row.key,
                row.expected.first().cloned().unwrap_or_default(),
                row.expected.join(" / "),
                row.section
            ));
        }
        if fit.rules.is_empty() {
            continue;
        }
        let sections: BTreeSet<String> = target
            .rows
            .iter()
            .map(|row| row.section.clone())
            .collect();
        let citation = sections.iter().next().cloned().unwrap_or_default();
        let cluster_id = refit_cluster_id(target);
        let Some(candidate_id) = candidates
            .get(&(ALYPY_SOURCE.to_owned(), citation.clone()))
            .cloned()
        else {
            rejected.push((
                cluster_id,
                vec![format!("no {ALYPY_SOURCE} candidate record for {citation:?}")],
            ));
            continue;
        };
        let cluster = Cluster {
            id: cluster_id.clone(),
            status: ClusterStatus::Fit,
            class: ClassSpec {
                pos: "",
                class: "",
                gender: "",
                aspect: "",
            },
            lemma: target.lemma.clone(),
            stem: String::new(),
            cells: Vec::new(),
            token_keys: Vec::new(),
            paradigm_keys: fit
                .cleared_rows
                .iter()
                .map(|index| target.rows[*index].key.clone())
                .collect(),
            accent: fit.rules.clone(),
            evidence: sections.into_iter().collect(),
            note: String::new(),
        };
        admissions.push(Admission {
            cluster,
            lexeme_id: target.id.clone(),
            evidence_id: cluster_id,
            candidate_id,
            source_id: ALYPY_SOURCE.to_owned(),
            citation,
            accent_only: true,
        });
    }
    fs::write(root.join(REFIT_REPORT_RELATIVE), report)?;
    println!(
        "synodal-gold refit: {} lexemes, {} admissions; cells cleared {total_cleared}, contract-blocked {total_blocked}, residue {total_residue}; wrote {REFIT_REPORT_RELATIVE}",
        targets.len(),
        admissions.len()
    );
    if options.file_review && !review_rows.is_empty() {
        let mut content = filed.clone();
        if !content.ends_with('\n') {
            content.push('\n');
        }
        for row in &review_rows {
            content.push_str(row);
            content.push('\n');
        }
        fs::write(root.join(HUMAN_REVIEW_RELATIVE), content)?;
        println!(
            "  filed {} contract-blocked cells in {HUMAN_REVIEW_RELATIVE}",
            review_rows.len()
        );
    } else if !review_rows.is_empty() {
        println!(
            "  {} contract-blocked cells not yet filed (rerun with --file-review)",
            review_rows.len()
        );
    }
    if options.dry_run {
        for admission in &admissions {
            println!(
                "  would admit {} ({} rules, {} cells)",
                admission.lexeme_id,
                admission.cluster.accent.len(),
                admission.cluster.paradigm_keys.len()
            );
        }
        // The probes installed overrides in-process only; nothing was written.
        return Ok(());
    }
    let outcome = land_admissions(root, admissions, rejected)?;
    print_admit_summary(&outcome, started.elapsed().as_secs_f64());
    Ok(())
}
