//! Fits reusable Synodal accent paradigms to the printed forms that the
//! locked corpus already attests.
//!
//! The `missing-accent-or-orthographic-metadata` frontier is the subset of the
//! coverage gap where identity and cell are already resolved: the accentless
//! surface analyses correctly, but `OrthographyProfile::SynodalLiturgical`
//! cannot realise the printed marks because the lexeme carries no reviewed
//! accent contract. This module derives that contract instead of storing one
//! accented string per cell.
//!
//! Evidence discipline: a placement is fitted **only** against attestations
//! drawn from `source`-partition passages, and every passage sealed into the
//! held-out evaluation contract is excluded by passage name in both editions.
//! An evaluation-partition token that a fitted rule later realises is
//! therefore a genuine generalisation rather than a memorised string.
//!
//! A proposal is emitted only when one placement plus mark reproduces *every*
//! source-partition attestation in its scope, compared on the same
//! `normalize_lookup` key the reverse analyzer indexes, under the engine's own
//! [`AccentParadigm::apply`]. Scope groups are refined from coarse to fine —
//! number, then case, then gender, then animacy — and the coarsest granularity
//! that fits the whole family is the one that is emitted, so genuine accent
//! mobility is stated as mobility rather than smoothed away. A family that
//! still disagrees at the finest expressible granularity is reported as a
//! conflict with counterexamples rather than being forced.
//!
//! Fitting alone is not sufficient, because it only ever sees tokens that are
//! still in the accent gap. Four guards run over every surviving rule before
//! it is written: it must not overlap a cell an existing reviewed paradigm
//! governs, must realise every cell it claims, must not generate a print that
//! a corpus-complete index of source-partition tokens contradicts, and must
//! not put a kamora on a singular-only scope.

use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use synodal_church_slavonic::{GenerationPolicy, Inflector, OrthographyProfile};
use synodal_church_slavonic_core::{
    AccentMark, AccentParadigm, AccentPlacement, AccentRule, AccentScope, Animacy, AuthorityRole,
    Case, Comparison, EpistemicRole, Evidence, EvidenceId, EvidenceKind, Gender, GrammarCell,
    Number, Recension, SourceId, normalize_lookup, normalize_lookup_accentless,
};
use synodal_church_slavonic_dictionary::coverage::{
    Analyzer, CheckTextOptions, GapKind, check_text,
};

use crate::report_io::write_if_changed_atomic;

const DEFAULT_SOURCES: [&str; 2] = [
    "ponomar-elizabeth-bible-2026-08-09",
    "wikisource-church-slavonic-bible-2026-08-09",
];

/// The offsets searched for every placement kind. No stem or ending in this
/// corpus carries more syllables than this.
const MAX_VOWEL_OFFSET: u8 = 9;

const MARKS: [AccentMark; 3] = [AccentMark::Acute, AccentMark::Grave, AccentMark::Kamora];

/// One `source`-partition occurrence of a printed form whose cell is already
/// resolved but whose accent contract is missing.
#[derive(Clone, Debug)]
struct Attestation {
    cell: GrammarCell,
    expanded: String,
    printed: String,
    passage: String,
    source_id: String,
    candidate_id: String,
}

/// The part of a scope string that is invariant across the refinable
/// agreement dimensions.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ScopeFamily {
    Noun,
    Numeral,
    /// `adjective:{form}:{comparison}` — also matches determiner cells.
    Adjective(&'static str, &'static str),
    Pronoun,
    FiniteVerb(&'static str),
    Participle(&'static str, &'static str, &'static str, &'static str),
    Imperative,
    LParticiple,
}

impl ScopeFamily {
    /// The finest granularity this family's scope grammar can express.
    const fn deepest(&self) -> Granularity {
        match self {
            Self::Noun => Granularity::NumberCase,
            Self::Adjective(..) | Self::Pronoun => Granularity::NumberCaseGenderAnimacy,
            Self::Numeral
            | Self::FiniteVerb(_)
            | Self::Participle(..)
            | Self::Imperative
            | Self::LParticiple => Granularity::Number,
        }
    }
}

/// How finely a scope group is partitioned before a placement is fitted.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Granularity {
    Number,
    NumberCase,
    NumberCaseGender,
    NumberCaseGenderAnimacy,
}

impl Granularity {
    const LADDER: [Self; 4] = [
        Self::Number,
        Self::NumberCase,
        Self::NumberCaseGender,
        Self::NumberCaseGenderAnimacy,
    ];
}

/// The agreement coordinates of one attestation, masked to a granularity.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
struct Coordinates {
    number: Option<Number>,
    case: Option<Case>,
    gender: Option<Gender>,
    animacy: Option<Animacy>,
}

impl Coordinates {
    fn masked(self, granularity: Granularity) -> Self {
        Self {
            number: self.number,
            case: (granularity >= Granularity::NumberCase)
                .then_some(self.case)
                .flatten(),
            gender: (granularity >= Granularity::NumberCaseGender)
                .then_some(self.gender)
                .flatten(),
            animacy: (granularity >= Granularity::NumberCaseGenderAnimacy)
                .then_some(self.animacy)
                .flatten(),
        }
    }
}

/// A fitted decision covering one or more coordinate tuples.
#[derive(Clone, Debug)]
struct FittedRule {
    family: ScopeFamily,
    numbers: BTreeSet<Number>,
    cases: BTreeSet<Case>,
    genders: BTreeSet<Gender>,
    animacies: BTreeSet<Animacy>,
    placement: AccentPlacement,
    mark: AccentMark,
    granularity: Granularity,
    attestations: usize,
}

impl FittedRule {
    /// Renders the reviewed scope string consumed by `accent_paradigms.tsv`.
    fn scope_code(&self) -> String {
        let numbers = join(&self.numbers, |value| value.code());
        match (&self.family, self.granularity) {
            (ScopeFamily::Noun, Granularity::Number) => format!("noun:{numbers}"),
            (ScopeFamily::Noun, _) => format!(
                "noun:{numbers}:{}",
                codes(&self.cases, &Case::ALL, |value| value.code())
            ),
            (ScopeFamily::Numeral, _) => format!("numeral:{numbers}"),
            (ScopeFamily::Adjective(form, comparison), Granularity::Number) => {
                format!("adjective:{form}:{comparison}:{numbers}")
            }
            // Every field of `adjective-agreeing` is required, so dimensions
            // the fit did not need to distinguish are stated as the full
            // closed inventory rather than left empty.
            (ScopeFamily::Adjective(form, comparison), _) => format!(
                "adjective-agreeing:{form}:{comparison}:{numbers}:{}:{}:{}",
                codes(&self.cases, &Case::ALL, |value| value.code()),
                codes(&self.genders, &Gender::ALL, |value| value.code()),
                codes(&self.animacies, &Animacy::ALL, |value| value.code()),
            ),
            // A pronoun cell always carries a case, so the coarse `Number`
            // level still uses the three-part `pronoun` scope with the full
            // case inventory rather than a separate grammar.
            (ScopeFamily::Pronoun, Granularity::Number | Granularity::NumberCase) => format!(
                "pronoun:{numbers}:{}",
                codes(&self.cases, &Case::ALL, |value| value.code())
            ),
            (ScopeFamily::Pronoun, _) => format!(
                "pronoun-agreeing:{numbers}:{}:{}:{}",
                codes(&self.cases, &Case::ALL, |value| value.code()),
                codes(&self.genders, &Gender::ALL, |value| value.code()),
                codes(&self.animacies, &Animacy::ALL, |value| value.code()),
            ),
            (ScopeFamily::FiniteVerb(tense), _) => format!("finite:{tense}:{numbers}"),
            (ScopeFamily::Participle(tense, voice, form, comparison), _) => {
                format!("participle:{tense}:{voice}:{form}:{comparison}:{numbers}")
            }
            (ScopeFamily::Imperative, _) => format!("imperative:{numbers}"),
            (ScopeFamily::LParticiple, _) => format!("l-participle:{numbers}"),
        }
    }

    /// The typed scope this rule will compile to. Built directly rather than
    /// by reparsing [`Self::scope_code`], so the overlap guard tests exactly
    /// the cells the emitted row will claim.
    fn accent_scope(&self) -> AccentScope {
        let numbers = self.numbers.iter().copied().collect::<Vec<_>>();
        let cases = if self.granularity >= Granularity::NumberCase && !self.cases.is_empty() {
            self.cases.iter().copied().collect()
        } else {
            Case::ALL.to_vec()
        };
        let genders =
            if self.granularity >= Granularity::NumberCaseGender && !self.genders.is_empty() {
                self.genders.iter().copied().collect()
            } else {
                Gender::ALL.to_vec()
            };
        let animacies = if self.granularity >= Granularity::NumberCaseGenderAnimacy
            && !self.animacies.is_empty()
        {
            self.animacies.iter().copied().collect()
        } else {
            Animacy::ALL.to_vec()
        };
        match &self.family {
            ScopeFamily::Noun if self.granularity == Granularity::Number => {
                AccentScope::Noun { numbers }
            }
            ScopeFamily::Noun => AccentScope::NounCases { numbers, cases },
            ScopeFamily::Pronoun if self.granularity <= Granularity::NumberCase => {
                AccentScope::PronounCases { numbers, cases }
            }
            ScopeFamily::Pronoun => AccentScope::PronounAgreement {
                numbers,
                cases,
                genders,
                animacies,
            },
            ScopeFamily::Numeral => AccentScope::Numeral { numbers },
            ScopeFamily::Adjective(form, comparison) if self.granularity == Granularity::Number => {
                AccentScope::Adjective {
                    form: parse_adjective_form(form),
                    comparison: parse_comparison(comparison),
                    numbers,
                }
            }
            ScopeFamily::Adjective(form, comparison) => AccentScope::AdjectiveAgreement {
                form: parse_adjective_form(form),
                comparison: parse_comparison(comparison),
                numbers,
                cases,
                genders,
                animacies,
            },
            ScopeFamily::FiniteVerb(tense) => AccentScope::FiniteVerb {
                tense: parse_finite_tense(tense),
                numbers,
            },
            ScopeFamily::Participle(tense, voice, form, comparison) => AccentScope::Participle {
                tense: parse_participle_tense(tense),
                voice: parse_participle_voice(voice),
                form: parse_adjective_form(form),
                comparison: parse_comparison(comparison),
                numbers,
            },
            ScopeFamily::Imperative => AccentScope::Imperative { numbers },
            ScopeFamily::LParticiple => AccentScope::LParticiple { numbers },
        }
    }
}

fn join<T: Copy + Ord>(values: &BTreeSet<T>, code: impl Fn(T) -> &'static str) -> String {
    values
        .iter()
        .map(|value| code(*value))
        .collect::<Vec<_>>()
        .join(",")
}

/// Renders a dimension, widening to the full closed inventory when the fit did
/// not need to distinguish that dimension. The scope grammar has no "any"
/// token, so an unrefined dimension is spelled out in full.
fn codes<T: Copy + Ord>(
    values: &BTreeSet<T>,
    all: &[T],
    code: impl Fn(T) -> &'static str + Copy,
) -> String {
    if values.is_empty() {
        return all
            .iter()
            .map(|value| code(*value))
            .collect::<Vec<_>>()
            .join(",");
    }
    join(values, code)
}

/// Splits a typed cell into its scope family and agreement coordinates. Cells
/// with no grammatical number carry no expressible reusable scope.
fn scope_of(cell: GrammarCell) -> Option<(ScopeFamily, Coordinates)> {
    let (family, coordinates) = match cell {
        GrammarCell::Noun(cell) | GrammarCell::VerbalNoun(cell) => (
            ScopeFamily::Noun,
            Coordinates {
                number: Some(cell.number),
                case: Some(cell.case),
                gender: None,
                animacy: Some(cell.animacy),
            },
        ),
        GrammarCell::Numeral(cell) => (
            ScopeFamily::Numeral,
            Coordinates {
                number: Some(cell.number),
                ..Coordinates::default()
            },
        ),
        GrammarCell::Adjective(cell) | GrammarCell::Determiner(cell) => (
            ScopeFamily::Adjective(cell.form.code(), cell.comparison.code()),
            Coordinates {
                number: Some(cell.number),
                case: Some(cell.case),
                gender: Some(cell.gender),
                animacy: Some(cell.animacy),
            },
        ),
        GrammarCell::FiniteVerb(cell) => (
            ScopeFamily::FiniteVerb(cell.tense.code()),
            Coordinates {
                number: Some(cell.number),
                ..Coordinates::default()
            },
        ),
        GrammarCell::Participle(cell) => (
            ScopeFamily::Participle(
                cell.tense.code(),
                cell.voice.code(),
                cell.agreement.form.code(),
                cell.agreement.comparison.code(),
            ),
            Coordinates {
                number: Some(cell.agreement.number),
                ..Coordinates::default()
            },
        ),
        GrammarCell::Imperative(cell) => (
            ScopeFamily::Imperative,
            Coordinates {
                number: Some(cell.number),
                ..Coordinates::default()
            },
        ),
        GrammarCell::LParticiple(cell) => (
            ScopeFamily::LParticiple,
            Coordinates {
                number: Some(cell.number),
                ..Coordinates::default()
            },
        ),
        GrammarCell::Pronoun(cell) => (
            ScopeFamily::Pronoun,
            Coordinates {
                number: Some(cell.number),
                case: Some(cell.case),
                // A pronoun cell's gender is optional; the personal and
                // reflexive paradigms have none. Leaving it absent keeps such
                // cells on the case-only scope, which is the only one that can
                // match them.
                gender: cell.gender,
                animacy: Some(cell.animacy),
            },
        ),
        GrammarCell::Infinitive
        | GrammarCell::Supine
        | GrammarCell::LexicalForm
        | GrammarCell::Indeclinable => return None,
    };
    Some((family, coordinates))
}

fn placement_code(placement: AccentPlacement) -> String {
    match placement {
        AccentPlacement::StemVowelFromStart(offset) => format!("stem-vowel-from-start:{offset}"),
        AccentPlacement::WordVowelFromStart(offset) => format!("word-vowel-from-start:{offset}"),
        AccentPlacement::EndingVowelFromEnd(offset) => format!("ending-vowel-from-end:{offset}"),
    }
}

const fn mark_code(mark: AccentMark) -> &'static str {
    match mark {
        AccentMark::Acute => "acute",
        AccentMark::Grave => "grave",
        AccentMark::Kamora => "kamora",
    }
}

/// The intermediate corpus record, retaining the candidate identity that a
/// reviewed evidence row has to cite.
#[derive(Clone, Debug, Deserialize)]
struct CandidateRecord {
    candidate_id: String,
    source_id: String,
    target_recension: Option<String>,
    passage: String,
    normalized_spelling: String,
    grammatical_cell: String,
    partition: String,
    parse_status: String,
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    crate::synodal_admit_check::ensure_registry_current(root)?;
    let mut intermediate = root.join("data/intermediate/synodal");
    let mut check = false;
    let mut apply = false;
    let mut suggest = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--intermediate" => {
                intermediate = PathBuf::from(args.next().ok_or("--intermediate needs a path")?);
            }
            "--check" => check = true,
            "--apply" => apply = true,
            "--suggest" => {
                let lexeme = args.next().ok_or("--suggest needs a lexeme id")?;
                let cell = args.next().ok_or("--suggest needs a cell key")?;
                suggest = Some((lexeme, cell));
            }
            value => return Err(format!("unknown synodal-accent-fit argument {value:?}").into()),
        }
    }
    if check && apply {
        return Err("--check and --apply are mutually exclusive".into());
    }
    if suggest.is_some() && (check || apply) {
        return Err("--suggest is read-only and excludes --check/--apply".into());
    }

    let held_out = load_held_out_passages(root)?;
    let records = load_source_partition(&intermediate, &held_out)?;
    let printed_index = index_printed_tokens(&records);
    let attestations = collect_attestations(&records)?;
    if let Some((lexeme, cell)) = suggest {
        return suggest_row(root, &lexeme, &cell, &attestations, &printed_index);
    }
    let mut outcome = fit(&attestations);
    validate_rules(&mut outcome, &printed_index)?;
    let tsv = render_tsv(&outcome, &attestations);
    let markdown = render_markdown(&outcome, &attestations);

    let tsv_path = root.join("reports/synodal-accent-fit.tsv");
    let markdown_path = root.join("reports/synodal-accent-fit.md");
    if check {
        require_current(&tsv_path, &tsv)?;
        require_current(&markdown_path, &markdown)?;
        println!("synodal accent fit: current");
        return Ok(());
    }
    write_if_changed_atomic(&tsv_path, &tsv)?;
    write_if_changed_atomic(&markdown_path, &markdown)?;
    if apply {
        let (paradigms, evidence) = apply_to_registry(root, &outcome, &attestations)?;
        println!(
            "synodal accent fit: applied {paradigms} paradigm rows and {evidence} evidence rows"
        );
    }
    println!(
        "synodal accent fit: {} rows over {} lexemes, {} conflicts, {} lexemes with unscopable cells",
        outcome.fitted.values().map(Vec::len).sum::<usize>(),
        outcome.fitted.len(),
        outcome.conflicts.len(),
        outcome.unscopable.len()
    );
    Ok(())
}

/// Appends the fitted contracts to the reviewed seed registries.
///
/// Writes one `reviewed_evidence.tsv` row per lexeme — cited by every accent
/// row of that lexeme, as `registry::accent_paradigm_for` requires a single
/// paradigm ID to carry uniform evidence — and one `accent_paradigms.tsv` row
/// per fitted scope. Existing rows are never rewritten, so the operation is
/// idempotent and re-running after a corpus refresh only adds what is new.
///
/// The write is deliberately append-only, and that has a consequence worth
/// stating: once a rule takes effect its cells leave the accent gap, so the
/// next run no longer proposes it. "Not proposed now" therefore cannot be read
/// as "rejected now", and this function must not prune on that basis. To
/// re-derive the table under a stricter guard, reset `accent_paradigms.tsv`
/// and the `*-accent-fit` evidence rows to their committed state, regenerate,
/// and re-apply from scratch.
fn apply_to_registry(
    root: &Path,
    outcome: &FitOutcome,
    attestations: &BTreeMap<String, Vec<Attestation>>,
) -> Result<(usize, usize), Box<dyn Error>> {
    let paradigm_path = root.join("data/synodal/accent_paradigms.tsv");
    let evidence_path = root.join("data/synodal/reviewed_evidence.tsv");
    let mut paradigm_lines: Vec<String> = fs::read_to_string(&paradigm_path)?
        .lines()
        .map(str::to_owned)
        .collect();
    let mut evidence = fs::read_to_string(&evidence_path)?;
    let existing_paradigms: BTreeSet<String> = paradigm_lines.iter().cloned().collect();
    // Every row of one paradigm must carry the same evidence, and the
    // registry reads a paradigm as one contiguous block. A lexeme fitted in an
    // earlier run therefore keeps that run's witness for every later row, and
    // later rows are inserted at the end of its block rather than appended.
    let existing_evidence: BTreeMap<String, (String, String)> = evidence
        .lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            (fields.len() >= 4).then(|| {
                (
                    fields[0].to_owned(),
                    (fields[2].to_owned(), fields[3].to_owned()),
                )
            })
        })
        .collect();

    let mut added_paradigms = 0;
    let mut added_evidence = 0;
    for (lexeme, rules) in &outcome.fitted {
        let Some(found) = witness_for(attestations, lexeme) else {
            continue;
        };
        let evidence_id = format!("{}-accent-fit", short_id(lexeme));
        let mut witness = found.clone();
        if let Some((source_id, passage)) = existing_evidence.get(&evidence_id) {
            witness.source_id = source_id.clone();
            witness.passage = passage.clone();
        }
        let witness = &witness;
        if !existing_evidence.contains_key(&evidence_id) {
            evidence.push_str(&format!(
                "{evidence_id}\t{}\t{}\t{}\treviewed\tsynodal-russian\t{}\n",
                witness.candidate_id,
                witness.source_id,
                witness.passage,
                "Source-partition print fixes this lexeme's accent placement; the rule was \
                 accepted only after reproducing every source-partition print in scope."
            ));
            added_evidence += 1;
        }
        for rule in rules {
            let row = paradigm_row(lexeme, rule, witness);
            if existing_paradigms.contains(&row) {
                continue;
            }
            let block_prefix = format!("{lexeme}\tsynodal-accent:{}-fitted\t", short_id(lexeme));
            let position = paradigm_lines
                .iter()
                .rposition(|line| line.starts_with(&block_prefix))
                .map_or(paradigm_lines.len(), |index| index + 1);
            paradigm_lines.insert(position, row);
            added_paradigms += 1;
        }
    }
    let mut paradigms = paradigm_lines.join("\n");
    paradigms.push('\n');
    write_if_changed_atomic(&paradigm_path, &paradigms)?;
    write_if_changed_atomic(&evidence_path, &evidence)?;
    Ok((added_paradigms, added_evidence))
}

/// The exact `accent_paradigms.tsv` line one fitted rule compiles to.
fn paradigm_row(lexeme: &str, rule: &FittedRule, witness: &Attestation) -> String {
    format!(
        "{lexeme}\tsynodal-accent:{}-fitted\t{}\t{}\t{}\t\t{}-accent-fit\t{}\t{}\tsynodal-russian\tsynodal-russian",
        short_id(lexeme),
        rule.scope_code(),
        placement_code(rule.placement),
        mark_code(rule.mark),
        short_id(lexeme),
        witness.source_id,
        witness.passage,
    )
}

/// Indexes every printed token of the readable source partition by its
/// accentless lookup key.
///
/// Fitting only sees tokens that are currently in the accent gap. A printed
/// form of the same lexeme that already resolves by some other route — an
/// exact row, or a cell a different rule governs — never reaches the fitter,
/// so a rule can be "consistent with every attestation it saw" and still
/// contradict the corpus. This index is the corpus-complete counter-evidence
/// the validation pass checks against.
fn index_printed_tokens(records: &[CandidateRecord]) -> BTreeMap<String, BTreeSet<String>> {
    let mut index: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for record in records {
        for token in record.normalized_spelling.split_whitespace() {
            let token = token.trim_matches(|character: char| {
                !character.is_alphabetic() && !is_combining(character)
            });
            if token.is_empty() {
                continue;
            }
            index
                .entry(normalize_lookup_accentless(token))
                .or_default()
                .insert(normalize_lookup(token));
        }
    }
    index
}

fn is_combining(character: char) -> bool {
    matches!(character, '\u{0300}'..='\u{036f}' | '\u{0483}'..='\u{0489}' | '\u{2de0}'..='\u{2dff}')
}

/// Drops any fitted rule whose scope would reach a cell that an existing
/// reviewed paradigm already governs.
///
/// The registry admits at most one accent paradigm per cell
/// (`registry::accent_paradigm_for`), so a coarse scope such as
/// `noun:singular` is unusable when the lexeme already carries a rule for one
/// of its singular cells: the two paradigm IDs would both apply and the lexeme
/// would fail with `ContradictoryMetadata`, silently losing coverage it
/// already had. Rejected rules are reported rather than narrowed, because
/// narrowing here would restate an existing reviewed decision without its
/// evidence.
fn validate_rules(
    outcome: &mut FitOutcome,
    printed_index: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), Box<dyn Error>> {
    let expanded = Inflector::builder()
        .generation_policy(GenerationPolicy::Strict)
        .orthography(OrthographyProfile::Expanded)
        .build();
    let liturgical = Inflector::builder()
        .generation_policy(GenerationPolicy::Strict)
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();

    let mut rejected = Vec::new();
    for (lexeme, rules) in &mut outcome.fitted {
        let id = synodal_church_slavonic_core::LexemeId::from(lexeme.as_str());
        let cells = synodal_church_slavonic_dictionary::analysis_cells_by_id(&id, expanded)
            .unwrap_or_default();
        rules.retain(|rule| {
            // Kamora is this print tradition's number-disambiguating mark: it
            // is what separates a dual or plural form from the singular it is
            // otherwise homographic with. The reverse analyzer offers every
            // syncretic reading of a token, so a kamora-marked dual or plural
            // print is also offered as a singular reading and can fit a
            // singular-only partition. Emitting that rule would both invent an
            // unattested singular accent and erase the distinction the print
            // exists to state, so a singular-only kamora contract is reported
            // for explicit review instead of admitted.
            if rule.mark == AccentMark::Kamora
                && rule.numbers.len() == 1
                && rule.numbers.contains(&Number::Singular)
            {
                rejected.push(format!(
                    "{lexeme}\t{}\tsingular-only kamora needs explicit review: the mark \
                     disambiguates dual and plural, so a syncretic reading may have driven it",
                    rule.scope_code()
                ));
                return false;
            }
            let scope = rule.accent_scope();
            let in_scope: Vec<_> = cells
                .iter()
                .copied()
                .filter(|cell| scope.applies_to(*cell))
                .collect();
            if let Some(cell) = in_scope
                .iter()
                .find(|cell| liturgical.form_by_id(&id, **cell).is_ok())
            {
                rejected.push(format!(
                    "{lexeme}\t{}\toverlaps an existing reviewed paradigm at {}",
                    rule.scope_code(),
                    cell.key()
                ));
                return false;
            }
            // A rule generalises to cells the corpus never attests, which is
            // legitimate for a reviewed productive contract but only if the
            // placement actually exists in those forms. A stem-vowel offset
            // beyond a shorter form in the same scope would raise
            // `ContradictoryMetadata` at runtime and take the whole lexeme
            // down with it, so the rule must realise every cell it claims.
            let probe = AccentParadigm {
                id: "synodal-accent:realizability-probe".to_owned(),
                accent_rules: vec![AccentRule {
                    scope: scope.clone(),
                    placement: rule.placement,
                    mark: rule.mark,
                }],
                breathing_rules: Vec::new(),
                evidence: probe_evidence(),
            };
            for cell in in_scope {
                let Ok(forms) = expanded.form_by_id(&id, cell) else {
                    continue;
                };
                for variant in forms.variants() {
                    let Ok(printed) = probe.apply(cell, &variant.expanded) else {
                        rejected.push(format!(
                            "{lexeme}\t{}\tcannot realise {} ({})",
                            rule.scope_code(),
                            cell.key(),
                            variant.expanded
                        ));
                        return false;
                    };
                    // Corpus-complete counter-evidence. Fitting only saw the
                    // prints that were still in the accent gap; a print of the
                    // same accentless key that already resolved by another
                    // route was invisible to it. If the corpus prints such a
                    // form and this rule can produce none of the attested
                    // marks for that key, the rule contradicts the corpus and
                    // must not ship, however well it explained the gap subset.
                    let key = normalize_lookup_accentless(&variant.expanded);
                    if let Some(attested) = printed_index.get(&key)
                        && !attested.contains(&normalize_lookup(&printed))
                    {
                        rejected.push(format!(
                            "{lexeme}\t{}\tgenerates {} for {} but the source partition prints {}",
                            rule.scope_code(),
                            printed,
                            cell.key(),
                            attested.iter().cloned().collect::<Vec<_>>().join("/")
                        ));
                        return false;
                    }
                }
            }
            true
        });
    }
    outcome.fitted.retain(|_, rules| !rules.is_empty());
    outcome.conflicts.extend(rejected);
    Ok(())
}

fn require_current(path: &Path, expected: &str) -> Result<(), Box<dyn Error>> {
    let actual = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if actual != expected {
        return Err(format!("stale {}; rerun synodal-accent-fit", path.display()).into());
    }
    Ok(())
}

/// The passages sealed into the held-out evaluation contract.
///
/// This is a second, stricter boundary than the corpus `partition` field: a
/// passage can be `source`-partition for coverage and still be the sealed
/// witness of an evaluation row.
///
/// The set is keyed by passage alone, deliberately dropping the source ID the
/// evaluation rows record. The two direct target corpora are verse-parallel,
/// and their `source`/`evaluation` partition assignments are independent, so
/// the same verse is frequently held out in one edition while the other
/// edition carries the same sentence — usually including the very token under
/// review — as `source`. Keying on `(source_id, passage)` would let that
/// sibling record back in and make fitting a held-out cell against a
/// near-identical printing possible. Keying on the passage excludes both
/// editions of every sealed verse.
fn load_held_out_passages(root: &Path) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let mut held_out = BTreeSet::new();
    for name in ["evaluation.tsv", "abbreviation_evaluation.tsv"] {
        let path = root.join("data/synodal").join(name);
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        for line in contents.lines().skip(1).filter(|line| !line.is_empty()) {
            if let Some(passage) = line.split('\t').nth(7) {
                held_out.insert(passage.to_owned());
            }
        }
    }
    Ok(held_out)
}

/// Loads only the `source`-partition verse records of the two direct target
/// corpora. Held-out passages are excluded at the input boundary so that no
/// later stage can fit against them.
fn load_source_partition(
    intermediate: &Path,
    held_out: &BTreeSet<String>,
) -> Result<Vec<CandidateRecord>, Box<dyn Error>> {
    let mut records = Vec::new();
    for source in DEFAULT_SOURCES {
        let path = intermediate.join(format!("{source}.jsonl"));
        let contents = fs::read_to_string(&path).map_err(|error| {
            format!(
                "cannot read {}: {error}; run synodal-bootstrap first",
                path.display()
            )
        })?;
        for line in contents.lines().filter(|line| !line.trim().is_empty()) {
            let record: CandidateRecord = serde_json::from_str(line)?;
            if record.partition != "source"
                || record.parse_status != "parsed"
                || record.grammatical_cell != "verse"
                || record.target_recension.as_deref() != Some("synodal-russian")
                || held_out.contains(&record.passage)
            {
                continue;
            }
            records.push(record);
        }
    }
    if records.is_empty() {
        return Err("no source-partition Synodal passages were loaded".into());
    }
    Ok(records)
}

/// Runs the canonical analyzer over the source partition and keeps every token
/// whose gap is exactly "the cell resolves but the marks do not".
fn collect_attestations(
    records: &[CandidateRecord],
) -> Result<BTreeMap<String, Vec<Attestation>>, Box<dyn Error>> {
    let analyzer = Analyzer::new(
        Inflector::builder()
            .generation_policy(GenerationPolicy::Strict)
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build(),
    )?;
    let expanded_inflector = Inflector::builder()
        .generation_policy(GenerationPolicy::Strict)
        .orthography(OrthographyProfile::Expanded)
        .build();
    let liturgical_inflector = Inflector::builder()
        .generation_policy(GenerationPolicy::Strict)
        .orthography(OrthographyProfile::SynodalLiturgical)
        .build();
    let options = CheckTextOptions {
        generation_policy: GenerationPolicy::Strict,
        orthography_profile: OrthographyProfile::SynodalLiturgical,
    };

    let mut by_lexeme: BTreeMap<String, Vec<Attestation>> = BTreeMap::new();
    let mut seen: BTreeSet<(String, String, String)> = BTreeSet::new();
    for record in records {
        let report = check_text(&analyzer, &record.normalized_spelling, options.clone());
        for token in report.tokens {
            let Some(gap) = token.gap.as_ref() else {
                continue;
            };
            if gap.kind != GapKind::MissingAccentOrOrthographicMetadata {
                continue;
            }
            for analysis in &token.analyses {
                let Some(cell) = analysis.cell else {
                    continue;
                };
                let id = analysis.lexeme.id();
                // A reflexive/passive reading derived by Alypy §73 attests the
                // *host* cell of an active verb: the print to fit is the token
                // without its enclitic, with the deleted final jer restored
                // where the accentless host demands it. The enclitic never
                // carries a mark, so the host keeps every mark the token has.
                let hosts: Vec<String> = if analysis.reflexive {
                    synodal_church_slavonic_core::reflexive_base_candidates(&token.token.original)
                } else {
                    vec![token.token.original.clone()]
                };
                // A cell that already resolves under the liturgical profile is
                // governed by an existing rule that merely disagrees with the
                // print. A second paradigm covering the same cell is rejected
                // by the registry outright, so these are left for review.
                if liturgical_inflector.form_by_id(id, cell).is_ok() {
                    continue;
                }
                let Ok(forms) = expanded_inflector.form_by_id(id, cell) else {
                    continue;
                };
                for variant in forms.variants() {
                    // One cell can offer several ordered variants; only the
                    // variant that *is* this token's accentless counterpart may
                    // be paired with its print.
                    let variant_key = normalize_lookup_accentless(&variant.expanded);
                    let Some(printed) = hosts
                        .iter()
                        .find(|host| normalize_lookup_accentless(host) == variant_key)
                    else {
                        continue;
                    };
                    let key = (id.as_str().to_owned(), cell.key(), printed.clone());
                    if !seen.insert(key) {
                        continue;
                    }
                    by_lexeme
                        .entry(id.as_str().to_owned())
                        .or_default()
                        .push(Attestation {
                            cell,
                            expanded: variant.expanded.clone(),
                            printed: printed.clone(),
                            passage: record.passage.clone(),
                            source_id: record.source_id.clone(),
                            candidate_id: record.candidate_id.clone(),
                        });
                }
            }
        }
    }
    Ok(by_lexeme)
}

#[derive(Debug, Default)]
struct FitOutcome {
    fitted: BTreeMap<String, Vec<FittedRule>>,
    conflicts: Vec<String>,
    unscopable: BTreeMap<String, BTreeSet<String>>,
}

/// Fits each `(lexeme, family)` at the coarsest granularity whose every
/// partition is reproduced exactly, escalating only when a coarser level fails.
fn fit(attestations: &BTreeMap<String, Vec<Attestation>>) -> FitOutcome {
    let placements = candidate_placements();
    let mut outcome = FitOutcome::default();

    for (lexeme, rows) in attestations {
        let mut families: BTreeMap<ScopeFamily, Vec<(&Attestation, Coordinates)>> = BTreeMap::new();
        for row in rows {
            match scope_of(row.cell) {
                Some((family, coordinates)) => {
                    families.entry(family).or_default().push((row, coordinates));
                }
                None => {
                    outcome
                        .unscopable
                        .entry(lexeme.clone())
                        .or_default()
                        .insert(format!("{} {}", row.cell.key(), row.printed));
                }
            }
        }

        for (family, group) in families {
            let mut by_granularity: BTreeMap<Granularity, Decisions> = BTreeMap::new();
            let mut unfitted = Vec::new();
            refine(
                &family,
                &group,
                Granularity::Number,
                &placements,
                &mut by_granularity,
                &mut unfitted,
            );
            for (granularity, decisions) in by_granularity {
                let rules = merge(&family, granularity, decisions);
                outcome
                    .fitted
                    .entry(lexeme.clone())
                    .or_default()
                    .extend(rules);
            }
            if !unfitted.is_empty() {
                outcome
                    .conflicts
                    .push(describe_conflict(lexeme, &family, &unfitted, &placements));
            }
        }
    }
    outcome
}

type Decisions = BTreeMap<Coordinates, (AccentPlacement, AccentMark, usize)>;

/// Greedily fits a scope family from coarse to fine.
///
/// Each partition that one placement reproduces is settled at the current
/// granularity, which keeps the widest reviewed generalisation the evidence
/// supports. Only the partitions that disagree are split further, so genuine
/// mobility is localised instead of forcing the whole family down to its
/// finest scope. A refined partition is always a strict subset of the coarse
/// partition that failed, and a failed partition is never emitted, so the
/// emitted scopes remain pairwise disjoint and the registry's one-paradigm-
/// per-cell rule is preserved.
fn refine<'a>(
    family: &ScopeFamily,
    group: &[(&'a Attestation, Coordinates)],
    granularity: Granularity,
    placements: &[AccentPlacement],
    settled: &mut BTreeMap<Granularity, Decisions>,
    unfitted: &mut Vec<(Coordinates, Vec<&'a Attestation>)>,
) {
    let mut partitions: BTreeMap<Coordinates, Vec<(&'a Attestation, Coordinates)>> =
        BTreeMap::new();
    for (row, coordinates) in group {
        partitions
            .entry(coordinates.masked(granularity))
            .or_default()
            .push((row, *coordinates));
    }
    let next = Granularity::LADDER
        .iter()
        .copied()
        .find(|candidate| *candidate > granularity && *candidate <= family.deepest());

    for (coordinates, members) in partitions {
        let rows: Vec<&'a Attestation> = members.iter().map(|(row, _)| *row).collect();
        if let Some((placement, mark)) = best_fit(family, coordinates, &rows, placements) {
            settled
                .entry(granularity)
                .or_default()
                .insert(coordinates, (placement, mark, rows.len()));
            continue;
        }
        match next {
            Some(next) => refine(family, &members, next, placements, settled, unfitted),
            None => unfitted.push((coordinates, rows)),
        }
    }
}

/// The first placement and mark, in deterministic search order, that
/// reproduces every attestation in the partition.
fn best_fit(
    family: &ScopeFamily,
    coordinates: Coordinates,
    rows: &[&Attestation],
    placements: &[AccentPlacement],
) -> Option<(AccentPlacement, AccentMark)> {
    let scope = probe_scope(family, coordinates);
    for placement in placements {
        for mark in MARKS {
            if rows
                .iter()
                .all(|row| reproduces(&scope, *placement, mark, row))
            {
                return Some((*placement, mark));
            }
        }
    }
    None
}

/// Merges partitions that agree on a decision, but only when their union is an
/// exact Cartesian product. Merging a ragged set would silently claim cells the
/// evidence never covered under that decision.
fn merge(family: &ScopeFamily, granularity: Granularity, decisions: Decisions) -> Vec<FittedRule> {
    let mut buckets: BTreeMap<(AccentPlacement, AccentMark), Vec<(Coordinates, usize)>> =
        BTreeMap::new();
    for (coordinates, (placement, mark, count)) in decisions {
        buckets
            .entry((placement, mark))
            .or_default()
            .push((coordinates, count));
    }

    let mut rules = Vec::new();
    for ((placement, mark), entries) in buckets {
        let numbers: BTreeSet<Number> = entries.iter().filter_map(|(c, _)| c.number).collect();
        let cases: BTreeSet<Case> = entries.iter().filter_map(|(c, _)| c.case).collect();
        let genders: BTreeSet<Gender> = entries.iter().filter_map(|(c, _)| c.gender).collect();
        let animacies: BTreeSet<Animacy> = entries.iter().filter_map(|(c, _)| c.animacy).collect();
        let product = numbers.len().max(1)
            * cases.len().max(1)
            * genders.len().max(1)
            * animacies.len().max(1);
        let attestations = entries.iter().map(|(_, count)| *count).sum();
        if product == entries.len() {
            rules.push(FittedRule {
                family: family.clone(),
                numbers,
                cases,
                genders,
                animacies,
                placement,
                mark,
                granularity,
                attestations,
            });
            continue;
        }
        for (coordinates, count) in entries {
            rules.push(FittedRule {
                family: family.clone(),
                numbers: coordinates.number.into_iter().collect(),
                cases: coordinates.case.into_iter().collect(),
                genders: coordinates.gender.into_iter().collect(),
                animacies: coordinates.animacy.into_iter().collect(),
                placement,
                mark,
                granularity,
                attestations: count,
            });
        }
    }
    rules
}

/// Describes the partitions that no reviewed placement reproduces, together
/// with the counterexamples that defeat the best-scoring candidate rule.
fn describe_conflict(
    lexeme: &str,
    family: &ScopeFamily,
    unfitted: &[(Coordinates, Vec<&Attestation>)],
    placements: &[AccentPlacement],
) -> String {
    let mut lines = Vec::new();
    for (coordinates, rows) in unfitted {
        let coordinates = *coordinates;
        let scope = probe_scope(family, coordinates);
        // Report the counterexamples against the placement that explains the
        // most attestations, so a reviewer sees the genuine minority pattern.
        let best = placements
            .iter()
            .flat_map(|placement| MARKS.map(|mark| (*placement, mark)))
            .max_by_key(|(placement, mark)| {
                rows.iter()
                    .filter(|row| reproduces(&scope, *placement, *mark, row))
                    .count()
            });
        let counterexamples = best.map_or_else(
            || {
                rows.iter()
                    .take(4)
                    .map(|row| format!("{}→{}", row.expanded, row.printed))
                    .collect::<Vec<_>>()
            },
            |(placement, mark)| {
                rows.iter()
                    .filter(|row| !reproduces(&scope, placement, mark, row))
                    .take(4)
                    .map(|row| format!("{}→{}", row.expanded, row.printed))
                    .collect::<Vec<_>>()
            },
        );
        lines.push(format!(
            "{lexeme}\t{}\t{}\t{} attestation(s); best rule {} misses: {}",
            describe_coordinates(coordinates),
            format_args!("{family:?}"),
            rows.len(),
            best.map_or_else(
                || "none".to_owned(),
                |(placement, mark)| format!("{} {}", placement_code(placement), mark_code(mark))
            ),
            counterexamples.join(" ")
        ));
    }
    lines.join("\n")
}

fn describe_coordinates(coordinates: Coordinates) -> String {
    let mut parts = Vec::new();
    if let Some(number) = coordinates.number {
        parts.push(number.code());
    }
    if let Some(case) = coordinates.case {
        parts.push(case.code());
    }
    if let Some(gender) = coordinates.gender {
        parts.push(gender.code());
    }
    if let Some(animacy) = coordinates.animacy {
        parts.push(animacy.code());
    }
    parts.join(":")
}

fn candidate_placements() -> Vec<AccentPlacement> {
    let mut placements = Vec::new();
    for offset in 0..=MAX_VOWEL_OFFSET {
        placements.push(AccentPlacement::StemVowelFromStart(offset));
    }
    for offset in 0..=MAX_VOWEL_OFFSET {
        placements.push(AccentPlacement::EndingVowelFromEnd(offset));
    }
    for offset in 0..=MAX_VOWEL_OFFSET {
        placements.push(AccentPlacement::WordVowelFromStart(offset));
    }
    placements
}

/// The in-memory scope used while searching. It always names exactly the
/// coordinates of the partition under test so that `applies_to` is true for
/// every attestation in it and false for everything else.
fn probe_scope(family: &ScopeFamily, coordinates: Coordinates) -> AccentScope {
    let numbers = coordinates.number.into_iter().collect::<Vec<_>>();
    let cases = coordinates
        .case
        .map_or_else(|| Case::ALL.to_vec(), |case| vec![case]);
    let genders = coordinates
        .gender
        .map_or_else(|| Gender::ALL.to_vec(), |gender| vec![gender]);
    let animacies = coordinates
        .animacy
        .map_or_else(|| Animacy::ALL.to_vec(), |animacy| vec![animacy]);
    match family {
        ScopeFamily::Noun => AccentScope::NounCases { numbers, cases },
        // `PronounAgreement::applies_to` requires the cell to carry a gender,
        // so a partition that has not been refined to gender must use the
        // case-only scope or it would match nothing.
        ScopeFamily::Pronoun if coordinates.gender.is_none() => {
            AccentScope::PronounCases { numbers, cases }
        }
        ScopeFamily::Pronoun => AccentScope::PronounAgreement {
            numbers,
            cases,
            genders,
            animacies,
        },
        ScopeFamily::Numeral => AccentScope::Numeral { numbers },
        ScopeFamily::Adjective(form, comparison) => AccentScope::AdjectiveAgreement {
            form: parse_adjective_form(form),
            comparison: parse_comparison(comparison),
            numbers,
            cases,
            genders,
            animacies,
        },
        ScopeFamily::FiniteVerb(tense) => AccentScope::FiniteVerb {
            tense: parse_finite_tense(tense),
            numbers,
        },
        ScopeFamily::Participle(tense, voice, form, comparison) => AccentScope::Participle {
            tense: parse_participle_tense(tense),
            voice: parse_participle_voice(voice),
            form: parse_adjective_form(form),
            comparison: parse_comparison(comparison),
            numbers,
        },
        ScopeFamily::Imperative => AccentScope::Imperative { numbers },
        ScopeFamily::LParticiple => AccentScope::LParticiple { numbers },
    }
}

/// Applies a candidate rule with the engine's own accent implementation and
/// compares it against the printed corpus token.
///
/// The comparison is made on `normalize_lookup`, which is the exact key the
/// reverse analyzer indexes and queries. Comparing raw bytes instead would
/// reject a correct rule whenever the corpus token happens to be sentence
/// initial and therefore capitalised, since generation always produces the
/// lowercase citation shape. Every prosodic and positional mark is significant
/// in this key, so mark sensitivity is fully preserved.
fn reproduces(
    scope: &AccentScope,
    placement: AccentPlacement,
    mark: AccentMark,
    row: &Attestation,
) -> bool {
    let paradigm = AccentParadigm {
        id: "synodal-accent:fit-probe".to_owned(),
        accent_rules: vec![AccentRule {
            scope: scope.clone(),
            placement,
            mark,
        }],
        breathing_rules: Vec::new(),
        evidence: probe_evidence(),
    };
    let expected = normalize_lookup(&row.printed);
    paradigm
        .apply(row.cell, &row.expanded)
        .is_ok_and(|printed| normalize_lookup(&printed) == expected)
}

fn parse_adjective_form(value: &str) -> synodal_church_slavonic_core::AdjectiveForm {
    use synodal_church_slavonic_core::AdjectiveForm;
    AdjectiveForm::from_code(value).unwrap_or(AdjectiveForm::Short)
}

fn parse_comparison(value: &str) -> Comparison {
    Comparison::from_code(value).unwrap_or(Comparison::Positive)
}

fn parse_finite_tense(value: &str) -> synodal_church_slavonic_core::FiniteTense {
    use synodal_church_slavonic_core::FiniteTense;
    FiniteTense::from_code(value).unwrap_or(FiniteTense::Present)
}

fn parse_participle_tense(value: &str) -> synodal_church_slavonic_core::ParticipleTense {
    use synodal_church_slavonic_core::ParticipleTense;
    ParticipleTense::from_code(value).unwrap_or(ParticipleTense::Present)
}

fn parse_participle_voice(value: &str) -> synodal_church_slavonic_core::ParticipleVoice {
    use synodal_church_slavonic_core::ParticipleVoice;
    ParticipleVoice::from_code(value).unwrap_or(ParticipleVoice::Active)
}

/// Evidence used only to satisfy [`AccentParadigm::validate`] during the
/// in-memory search. It never reaches a report or a registry row; emitted
/// proposals carry the real corpus citation instead.
fn probe_evidence() -> Evidence {
    Evidence {
        id: EvidenceId::from("synodal-accent-fit-probe"),
        source: SourceId::from("ponomar-elizabeth-bible-2026-08-09"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::AccentParadigm,
        authority_roles: vec![AuthorityRole::Accentual, AuthorityRole::Orthographic],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: "accent-fit search probe".into(),
        note: None,
    }
}

fn render_tsv(outcome: &FitOutcome, attestations: &BTreeMap<String, Vec<Attestation>>) -> String {
    let mut output = String::from(
        "lexeme_id\tparadigm_id\tscope\tplacement\tmark\tbreathing\tevidence_id\tsource_id\tcitation\tsource_recension\ttarget_recension\tgranularity\tattestations\tcandidate_id\n",
    );
    for (lexeme, rules) in &outcome.fitted {
        let Some(witness) = witness_for(attestations, lexeme) else {
            continue;
        };
        let paradigm_id = format!("synodal-accent:{}-fitted", short_id(lexeme));
        let evidence_id = format!("{}-accent-fit", short_id(lexeme));
        for rule in rules {
            output.push_str(&format!(
                "{lexeme}\t{paradigm_id}\t{}\t{}\t{}\t\t{evidence_id}\t{}\t{}\tsynodal-russian\tsynodal-russian\t{:?}\t{}\t{}\n",
                rule.scope_code(),
                placement_code(rule.placement),
                mark_code(rule.mark),
                witness.source_id,
                witness.passage,
                rule.granularity,
                rule.attestations,
                witness.candidate_id,
            ));
        }
    }
    output
}

/// Picks the deterministic evidence witness for a lexeme: the lexicographically
/// first `(passage, printed)` source-partition attestation.
fn witness_for<'a>(
    attestations: &'a BTreeMap<String, Vec<Attestation>>,
    lexeme: &str,
) -> Option<&'a Attestation> {
    attestations
        .get(lexeme)?
        .iter()
        .min_by(|left, right| (&left.passage, &left.printed).cmp(&(&right.passage, &right.printed)))
}

fn short_id(lexeme: &str) -> String {
    lexeme.replace("synodal:", "").replace(':', "-")
}

fn render_markdown(
    outcome: &FitOutcome,
    attestations: &BTreeMap<String, Vec<Attestation>>,
) -> String {
    let rows: usize = outcome.fitted.values().map(Vec::len).sum();
    let mut output = String::from("# Synodal accent-paradigm fit\n\n");
    output.push_str(
        "Every rule below was fitted **only** against `source`-partition printed forms and is\nkept only when the engine's own accent implementation reproduces each of those\nforms byte-for-byte. Held-out `evaluation` passages were never read while\nfitting, so evaluation-partition tokens that these rules realise are genuine\ngeneralisations rather than memorised strings.\n\nScope groups are refined from coarse to fine (number, case, gender, animacy) and\nthe coarsest granularity that fits the whole family is the one emitted, so real\naccent mobility is stated rather than smoothed away.\n\n",
    );
    output.push_str(&format!(
        "- Lexemes with a fitted contract: {}\n- Accent-paradigm rows proposed: {rows}\n- Scope families with no reproducing placement at the finest granularity: {}\n- Lexemes with cells outside the reusable scope grammar: {}\n\n",
        outcome.fitted.len(),
        outcome.conflicts.len(),
        outcome.unscopable.len()
    ));

    let mut by_granularity: BTreeMap<String, usize> = BTreeMap::new();
    for rules in outcome.fitted.values() {
        for rule in rules {
            *by_granularity
                .entry(format!("{:?}", rule.granularity))
                .or_default() += 1;
        }
    }
    output.push_str("| Granularity | Rows |\n|---|---:|\n");
    for (granularity, count) in &by_granularity {
        output.push_str(&format!("| `{granularity}` | {count} |\n"));
    }

    output.push_str("\n## Fitted contracts\n\n");
    output.push_str("| Lexeme | Scope | Placement | Mark | Attestations |\n");
    output.push_str("|---|---|---|---|---:|\n");
    for (lexeme, rules) in &outcome.fitted {
        for rule in rules {
            output.push_str(&format!(
                "| `{lexeme}` | `{}` | `{}` | `{}` | {} |\n",
                rule.scope_code(),
                placement_code(rule.placement),
                mark_code(rule.mark),
                rule.attestations,
            ));
        }
    }

    if !outcome.conflicts.is_empty() {
        output.push_str("\n## Unfitted scope families\n\n");
        output.push_str(
            "No reviewed placement reproduces every printed form even at the finest scope the\ngrammar can express. These need human review: the usual causes are homography,\nan enclitic environment, or mobility conditioned by something the scope grammar\ndoes not name.\n\n```text\n",
        );
        for conflict in &outcome.conflicts {
            output.push_str(conflict);
            output.push('\n');
        }
        output.push_str("```\n");
    }

    if !outcome.unscopable.is_empty() {
        output.push_str("\n## Cells outside the reusable scope grammar\n\n");
        output.push_str(
            "Infinitive, supine, pronoun, and lexical cells carry no grammatical number, so the\naccent-paradigm scope grammar cannot address them. They need per-cell\n`accents.tsv` evidence instead.\n\n```text\n",
        );
        for (lexeme, cells) in &outcome.unscopable {
            output.push_str(&format!(
                "{lexeme}\t{}\n",
                cells.iter().cloned().collect::<Vec<_>>().join(" | ")
            ));
        }
        output.push_str("```\n");
    }

    let total: usize = attestations.values().map(Vec::len).sum();
    output.push_str(&format!(
        "\n## Inputs\n\n- Distinct source-partition `(lexeme, cell, printed)` attestations read: {total}\n- Lexemes with at least one such attestation: {}\n- Placement space searched per partition: {} placements x {} marks\n",
        attestations.len(),
        candidate_placements().len(),
        MARKS.len()
    ));
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(family: ScopeFamily, granularity: Granularity) -> FittedRule {
        FittedRule {
            family,
            numbers: BTreeSet::from([Number::Singular, Number::Plural]),
            cases: BTreeSet::from([Case::Genitive]),
            genders: BTreeSet::from([Gender::Masculine]),
            animacies: BTreeSet::from([Animacy::Inanimate]),
            placement: AccentPlacement::StemVowelFromStart(0),
            mark: AccentMark::Acute,
            granularity,
            attestations: 1,
        }
    }

    #[test]
    fn coarse_scopes_omit_the_refinable_dimensions() {
        assert_eq!(
            rule(ScopeFamily::Noun, Granularity::Number).scope_code(),
            "noun:singular,plural"
        );
        assert_eq!(
            rule(
                ScopeFamily::Adjective("long", "positive"),
                Granularity::Number
            )
            .scope_code(),
            "adjective:long:positive:singular,plural"
        );
    }

    #[test]
    fn refined_scopes_use_the_case_and_agreement_grammars() {
        assert_eq!(
            rule(ScopeFamily::Noun, Granularity::NumberCase).scope_code(),
            "noun:singular,plural:genitive"
        );
        assert_eq!(
            rule(
                ScopeFamily::Adjective("short", "positive"),
                Granularity::NumberCaseGenderAnimacy
            )
            .scope_code(),
            "adjective-agreeing:short:positive:singular,plural:genitive:masculine:inanimate"
        );
    }

    #[test]
    fn placement_codes_match_the_validator_vocabulary() {
        assert_eq!(
            placement_code(AccentPlacement::StemVowelFromStart(0)),
            "stem-vowel-from-start:0"
        );
        assert_eq!(
            placement_code(AccentPlacement::EndingVowelFromEnd(1)),
            "ending-vowel-from-end:1"
        );
    }

    #[test]
    fn cells_without_grammatical_number_have_no_reusable_scope() {
        assert!(scope_of(GrammarCell::Infinitive).is_none());
        assert!(scope_of(GrammarCell::Supine).is_none());
        assert!(scope_of(GrammarCell::LexicalForm).is_none());
    }

    #[test]
    fn masking_drops_only_the_finer_dimensions() {
        let coordinates = Coordinates {
            number: Some(Number::Dual),
            case: Some(Case::Dative),
            gender: Some(Gender::Neuter),
            animacy: Some(Animacy::Animate),
        };
        let masked = coordinates.masked(Granularity::NumberCase);
        assert_eq!(masked.number, Some(Number::Dual));
        assert_eq!(masked.case, Some(Case::Dative));
        assert_eq!(masked.gender, None);
        assert_eq!(masked.animacy, None);
    }
}

/// Prints the exact `accent_paradigms.tsv` row that would realize one cell's
/// print — with the correct block paradigm ID and block-uniform evidence when
/// the lexeme already carries a fitted block — or explains precisely which
/// witness is missing. A cell whose only corpus witnesses are themselves
/// held-out types is reported as unfittable without memorisation, never as a
/// row.
fn suggest_row(
    root: &Path,
    lexeme: &str,
    cell_key: &str,
    attestations: &BTreeMap<String, Vec<Attestation>>,
    printed_index: &BTreeMap<String, BTreeSet<String>>,
) -> Result<(), Box<dyn Error>> {
    let _cell: GrammarCell = cell_key
        .parse()
        .map_err(|error| format!("invalid cell key {cell_key:?}: {error}"))?;
    let held_types = crate::synodal_type_holdout::load(
        &root.join(crate::synodal_type_holdout::HOLDOUT_PATH),
    )?;
    let mine: Vec<Attestation> = attestations
        .get(lexeme)
        .into_iter()
        .flatten()
        .filter(|attestation| attestation.cell.key() == cell_key)
        .cloned()
        .collect();
    if mine.is_empty() {
        let expanded = Inflector::builder()
            .generation_policy(GenerationPolicy::Strict)
            .orthography(OrthographyProfile::Expanded)
            .build()
            .form_by_id(&synodal_church_slavonic_core::LexemeId::from(lexeme), _cell);
        return Err(match expanded {
            Err(error) => format!(
                "cell {cell_key} of {lexeme} does not expand ({error}); fix the expansion before fitting an accent"
            ),
            Ok(forms) => {
                let keys: Vec<String> = forms
                    .variants()
                    .iter()
                    .map(|variant| normalize_lookup_accentless(&variant.expanded))
                    .collect();
                let witnessed = keys
                    .iter()
                    .any(|key| printed_index.contains_key(key));
                if witnessed {
                    format!(
                        "cell {cell_key} of {lexeme} expands but is not in the accent gap (it already resolves, or its prints attest another lexeme); nothing to fit"
                    )
                } else {
                    format!(
                        "cell {cell_key} of {lexeme} has no source-partition witness at all; no accent can be fitted without new corpus evidence"
                    )
                }
            }
        }
        .into());
    }
    let (usable, memorising): (Vec<_>, Vec<_>) = mine.iter().partition(|attestation| {
        !held_types.contains(&normalize_lookup_accentless(&attestation.printed))
    });
    if usable.is_empty() {
        let prints: BTreeSet<&str> = memorising
            .iter()
            .map(|attestation| attestation.printed.as_str())
            .collect();
        return Err(format!(
            "cell {cell_key} of {lexeme} is unfittable without memorisation: every corpus witness ({}) is itself a held-out type; leave it to a rule generalised from non-held cells or defer",
            prints.into_iter().collect::<Vec<_>>().join(", ")
        )
        .into());
    }
    let mut single = BTreeMap::new();
    single.insert(
        lexeme.to_owned(),
        usable.iter().map(|attestation| (*attestation).clone()).collect::<Vec<_>>(),
    );
    let outcome = fit(&single);
    let Some(rules) = outcome.fitted.get(lexeme) else {
        let prints: BTreeSet<&str> = usable
            .iter()
            .map(|attestation| attestation.printed.as_str())
            .collect();
        return Err(format!(
            "cell {cell_key} of {lexeme} has conflicting witnesses ({}); review the variants before fitting",
            prints.into_iter().collect::<Vec<_>>().join(", ")
        )
        .into());
    };
    let witness = &usable[0];
    for rule in rules {
        let mut row = paradigm_row(lexeme, rule, witness);
        // Reuse the existing block's paradigm ID and uniform evidence when
        // the lexeme already carries one, and name the insertion point.
        let paradigms_path = root.join("data/synodal/accent_paradigms.tsv");
        let existing = fs::read_to_string(&paradigms_path)?;
        let block_rows: Vec<(usize, &str)> = existing
            .lines()
            .enumerate()
            .filter(|(_, line)| line.starts_with(&format!("{lexeme}\t")))
            .collect();
        if let Some((line_number, block_row)) = block_rows.last() {
            let block: Vec<&str> = block_row.split('\t').collect();
            let mut columns: Vec<String> =
                row.split('\t').map(str::to_owned).collect();
            columns[1] = block[1].to_owned();
            for index in [6, 7, 8] {
                columns[index] = block[index].to_owned();
            }
            row = columns.join("\t");
            println!(
                "insert inside the existing block, after data/synodal/accent_paradigms.tsv line {}:",
                line_number + 1
            );
        } else {
            println!(
                "no existing block for {lexeme}; the row cites its witness directly (add the matching reviewed_evidence row {}-accent-fit if absent):",
                short_id(lexeme)
            );
        }
        println!("{row}");
    }
    Ok(())
}
