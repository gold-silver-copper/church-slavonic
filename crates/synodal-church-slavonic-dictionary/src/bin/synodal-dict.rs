use std::{
    error::Error,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use synodal_church_slavonic_dictionary::{
    Analysis, Entry, FamilyId, FamilySummary, SearchOptions, VocabularyManifest,
    core::{GenerationPolicy, LexemeId, OrthographyProfile, normalize_lookup_accentless},
    coverage::{
        Analyzer, AnalyzerCache, CheckTextOptions, CoveragePassage, EvidenceReadiness, GapKind,
        MarginalRecoveryReport, check_text, coverage,
    },
    families, lint_vocabulary_with, lookup_all, lookup_by_id, search, show_family_by_id,
};

const DEFAULT_FAMILY_PROPOSALS: &str = "reports/synodal-family-review-queue.json";
const DEFAULT_MARGINAL_RECOVERY: &str = "reports/synodal-marginal-recovery.json";

#[cfg(not(test))]
fn main() {
    let context = CliContext::new();
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    let mut diagnostics = io::stderr().lock();
    if let Err(error) = run(
        std::env::args().skip(1),
        &mut input,
        &mut output,
        &mut diagnostics,
        &context,
    ) {
        let _ = writeln!(diagnostics, "error: {error}");
        std::process::exit(1);
    }
}

#[derive(Debug, Default)]
pub struct CliContext {
    analyzers: AnalyzerCache,
}

impl CliContext {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            analyzers: AnalyzerCache::new(),
        }
    }

    #[must_use]
    pub fn analyzer_construction_count(&self) -> usize {
        self.analyzers.construction_count()
    }
}

pub fn run(
    args: impl IntoIterator<Item = String>,
    input: &mut dyn Read,
    output: &mut dyn io::Write,
    diagnostics: &mut dyn io::Write,
    context: &CliContext,
) -> Result<(), Box<dyn Error>> {
    let mut args = args.into_iter();
    match args.next().as_deref() {
        Some("search") => search_command(args, output),
        Some("show") => show_command(args, output),
        Some("families") => families_command(args, output),
        Some("show-family") => show_family_command(args, output),
        Some("analyze") => analyze_command(args, output, context),
        Some("lint") => lint_command(args, input, output, context),
        Some("check-text") => check_text_command(args, input, output, context),
        Some("analyze-text") => analyze_text_command(args, input, output),
        Some("coverage") => coverage_command(args, input, output, context),
        Some("marginal-recovery") => marginal_recovery_command(args, output),
        Some("help") | Some("-h") | Some("--help") | None => {
            help(diagnostics)?;
            Ok(())
        }
        Some(other) => Err(format!("unknown synodal-dict command {other:?}").into()),
    }
}

fn families_command(
    args: impl Iterator<Item = String>,
    output: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let mut query = Vec::new();
    let mut json = false;
    let mut reviewed_only = false;
    let mut proposal_path = None;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--json" => json = true,
            "--reviewed-only" => reviewed_only = true,
            "--proposals" => {
                proposal_path = Some(PathBuf::from(
                    args.next().ok_or("--proposals requires a JSON path")?,
                ));
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown families option {value:?}").into());
            }
            _ => query.push(argument),
        }
    }
    let query = query.join(" ");
    if query.is_empty() {
        return Err("families requires a lemma, surface, or gloss query".into());
    }
    let reviewed = families(&query)?;
    let proposal_path = proposal_path.or_else(|| {
        let default = PathBuf::from(DEFAULT_FAMILY_PROPOSALS);
        default.exists().then_some(default)
    });
    let proposed = if reviewed_only {
        Vec::new()
    } else if let Some(path) = &proposal_path {
        read_family_proposals(path)?
            .into_iter()
            .filter(|proposal| proposal_matches(proposal, &query))
            .collect()
    } else {
        Vec::new()
    };
    if json {
        writeln!(
            output,
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "query": query,
                "reviewed": reviewed,
                "proposed": proposed,
                "proposal_report": proposal_path,
            }))?
        )?;
    } else if reviewed.is_empty() && proposed.is_empty() {
        writeln!(
            output,
            "No reviewed or proposed Synodal family matched {query:?}."
        )?;
    } else {
        for family in &reviewed {
            print_family(output, family)?;
        }
        for proposal in &proposed {
            print_proposed_family(output, proposal)?;
        }
    }
    Ok(())
}

fn show_family_command(
    mut args: impl Iterator<Item = String>,
    output: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let id = args
        .next()
        .ok_or("show-family requires a stable family ID")?;
    let mut json = false;
    let mut proposal_path = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--json" => json = true,
            "--proposals" => {
                proposal_path = Some(PathBuf::from(
                    args.next().ok_or("--proposals requires a JSON path")?,
                ));
            }
            value => return Err(format!("unknown show-family option {value:?}").into()),
        }
    }
    if id.starts_with("synodal:family-candidate:") {
        let path = proposal_path.unwrap_or_else(|| PathBuf::from(DEFAULT_FAMILY_PROPOSALS));
        let proposal = read_family_proposals(&path)?
            .into_iter()
            .find(|proposal| proposal["candidate_id"].as_str() == Some(id.as_str()))
            .ok_or_else(|| format!("unknown proposed family ID {id:?} in {}", path.display()))?;
        if json {
            writeln!(output, "{}", serde_json::to_string_pretty(&proposal)?)?;
        } else {
            print_proposed_family(output, &proposal)?;
        }
    } else {
        let family = show_family_by_id(&FamilyId::from(id.as_str()))?;
        if json {
            writeln!(output, "{}", serde_json::to_string_pretty(&family)?)?;
        } else {
            print_family(output, &family)?;
        }
    }
    Ok(())
}

fn read_family_proposals(path: &Path) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
    let proposals: Vec<serde_json::Value> = serde_json::from_str(&fs::read_to_string(path)?)?;
    if proposals.iter().any(|proposal| {
        proposal["candidate_id"].as_str().is_none()
            || proposal["proposed_lemma"].as_str().is_none()
            || proposal["review_status"].as_str().is_none()
    }) {
        return Err(format!("{} is not a family-review queue", path.display()).into());
    }
    Ok(proposals)
}

fn proposal_matches(proposal: &serde_json::Value, query: &str) -> bool {
    let query = normalize_lookup_accentless(query);
    let values = std::iter::once(proposal["candidate_id"].as_str())
        .chain(std::iter::once(proposal["proposed_lemma"].as_str()))
        .chain(
            proposal["surfaces"]
                .as_array()
                .into_iter()
                .flatten()
                .flat_map(|surface| [surface["original"].as_str(), surface["normalized"].as_str()]),
        )
        .flatten();
    values
        .map(normalize_lookup_accentless)
        .any(|value| value.contains(&query))
}

fn search_command(
    args: impl Iterator<Item = String>,
    output: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let mut query = Vec::new();
    let mut options = SearchOptions::default();
    let mut json = false;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--pos" => {
                options.part_of_speech = Some(parse_part_of_speech(
                    &args.next().ok_or("--pos requires a value")?,
                )?);
            }
            "--limit" => options.limit = args.next().ok_or("--limit requires a number")?.parse()?,
            "--exact" => options.fuzzy = false,
            "--json" => json = true,
            value if value.starts_with('-') => {
                return Err(format!("unknown search option {value:?}").into());
            }
            _ => query.push(argument),
        }
    }
    let query = query.join(" ");
    let results = search(&query, &options)?;
    if json {
        writeln!(output, "{}", serde_json::to_string_pretty(&results)?)?;
    } else if results.is_empty() {
        writeln!(output, "No reviewed Synodal entry matched {query:?}.")?;
    } else {
        for (index, result) in results.iter().enumerate() {
            writeln!(
                output,
                "{}. {} [{}] — {}\n   id: {} · score: {} · match: {:?}",
                index + 1,
                result.entry.lexeme.lemma(),
                pos_label(result.entry.lexeme.part_of_speech()),
                result
                    .entry
                    .senses
                    .iter()
                    .map(|sense| sense.gloss.as_str())
                    .collect::<Vec<_>>()
                    .join("; "),
                result.entry.lexeme.id(),
                result.score,
                result.matched_on,
            )?;
        }
    }
    Ok(())
}

fn show_command(
    mut args: impl Iterator<Item = String>,
    output: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let query = args
        .next()
        .ok_or("show requires a lemma or stable lexeme ID")?;
    let mut json = false;
    for argument in args {
        match argument.as_str() {
            "--json" => json = true,
            value if value.starts_with('-') => {
                return Err(format!("unknown show option {value:?}").into());
            }
            _ => return Err("show accepts exactly one lemma or stable lexeme ID".into()),
        }
    }
    let entries = if query.starts_with("synodal:") {
        vec![lookup_by_id(&LexemeId::from(query.as_str()))?]
    } else {
        lookup_all(&query)?
    };
    if json {
        writeln!(output, "{}", serde_json::to_string_pretty(&entries)?)?;
    } else if entries.is_empty() {
        writeln!(output, "No reviewed Synodal entry matched {query:?}.")?;
    } else {
        for entry in &entries {
            print_entry(output, entry)?;
        }
    }
    Ok(())
}

fn analyze_command(
    args: impl Iterator<Item = String>,
    output: &mut dyn Write,
    context: &CliContext,
) -> Result<(), Box<dyn Error>> {
    let mut words = Vec::new();
    let mut policy = GenerationPolicy::Strict;
    let mut profile = OrthographyProfile::Expanded;
    let mut json = false;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--policy" => policy = parse_policy(&args.next().ok_or("--policy needs a value")?)?,
            "--profile" => profile = parse_profile(&args.next().ok_or("--profile needs a value")?)?,
            "--json" => json = true,
            value if value.starts_with('-') => {
                return Err(format!("unknown analyze option {value:?}").into());
            }
            _ => words.push(argument),
        }
    }
    if words.len() != 1 {
        return Err("analyze requires exactly one Synodal word".into());
    }
    let analyzer = analyzer(policy, profile, context)?;
    let analyses = analyzer.analyze_profile(&words[0], profile)?;
    if json {
        writeln!(output, "{}", serde_json::to_string_pretty(&analyses)?)?;
    } else if analyses.is_empty() {
        writeln!(output, "No analysis under {policy:?}/{profile:?}.")?;
    } else {
        for (index, analysis) in analyses.iter().enumerate() {
            print_analysis(output, index + 1, analysis)?;
        }
    }
    Ok(())
}

fn analyze_text_command(
    args: impl Iterator<Item = String>,
    input: &mut dyn Read,
    output: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let mut texts = Vec::new();
    let mut policy = GenerationPolicy::Strict;
    let mut profile = OrthographyProfile::SynodalLiturgical;
    let mut json = false;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--policy" => policy = parse_policy(&args.next().ok_or("--policy needs a value")?)?,
            "--profile" => profile = parse_profile(&args.next().ok_or("--profile needs a value")?)?,
            "--json" => json = true,
            value if value.starts_with("--") => {
                return Err(format!("unknown analyze-text option {value:?}").into());
            }
            _ => texts.push(argument),
        }
    }
    if texts.len() != 1 {
        return Err("analyze-text requires exactly one TEXT argument (or - for stdin)".into());
    }
    let text = read_input(&texts[0], input)?;
    let inflector = synodal_church_slavonic::Inflector::builder()
        .generation_policy(policy)
        .orthography(profile)
        .build();
    let analysis = synodal_church_slavonic_dictionary::analyze_text(&text, inflector)?;
    if json {
        writeln!(output, "{}", serde_json::to_string_pretty(&analysis)?)?;
        return Ok(());
    }
    for token in &analysis.tokens {
        writeln!(output, "{}", token.token.original)?;
        if token.readings.is_empty() && token.predictions.is_empty() {
            writeln!(output, "    (no reading under {policy:?}/{profile:?})")?;
        }
        for (index, reading) in token.readings.iter().enumerate() {
            print_analysis(output, index + 1, reading)?;
        }
        for prediction in &token.predictions {
            writeln!(
                output,
                "    ? {} + -{} => {:?} [{} bp, {}]",
                prediction.stem,
                prediction.ending,
                prediction.cell,
                prediction.confidence_bp,
                prediction.model,
            )?;
        }
    }
    Ok(())
}

fn lint_command(
    args: impl Iterator<Item = String>,
    input: &mut dyn Read,
    output: &mut dyn Write,
    context: &CliContext,
) -> Result<(), Box<dyn Error>> {
    let mut path = None;
    let mut json = false;
    for argument in args {
        match argument.as_str() {
            "--json" => json = true,
            value if value.starts_with('-') => {
                return Err(format!("unknown lint option {value:?}").into());
            }
            _ if path.is_none() => path = Some(argument),
            _ => return Err("lint accepts exactly one JSON manifest".into()),
        }
    }
    let path = path.ok_or("lint requires a JSON vocabulary manifest")?;
    let manifest: VocabularyManifest = serde_json::from_str(&read_input(&path, input)?)?;
    let analyzer = analyzer(
        GenerationPolicy::Strict,
        OrthographyProfile::Expanded,
        context,
    )?;
    let issues = lint_vocabulary_with(&analyzer, &manifest);
    if json {
        writeln!(output, "{}", serde_json::to_string_pretty(&issues)?)?;
    } else {
        for issue in &issues {
            writeln!(
                output,
                "entry {} {:?} {:?}: {}",
                issue.index, issue.text, issue.kind, issue.detail
            )?;
        }
        writeln!(
            output,
            "vocabulary: {} entries, {} issue(s)",
            manifest.entries.len(),
            issues.len()
        )?;
    }
    if issues.is_empty() {
        Ok(())
    } else {
        Err(format!("{} vocabulary issue(s)", issues.len()).into())
    }
}

fn check_text_command(
    args: impl Iterator<Item = String>,
    input: &mut dyn Read,
    output: &mut dyn Write,
    context: &CliContext,
) -> Result<(), Box<dyn Error>> {
    let mut path = None;
    let mut policy = GenerationPolicy::Strict;
    let mut profile = OrthographyProfile::Expanded;
    let mut max_unknown = 0_usize;
    let mut max_ambiguous = usize::MAX;
    let mut json = false;
    let mut summary_only = false;
    let mut strict = false;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--policy" => policy = parse_policy(&args.next().ok_or("--policy needs a value")?)?,
            "--profile" => profile = parse_profile(&args.next().ok_or("--profile needs a value")?)?,
            "--max-unknown" => {
                max_unknown = args.next().ok_or("--max-unknown needs a number")?.parse()?;
            }
            "--max-ambiguous" => {
                max_ambiguous = args
                    .next()
                    .ok_or("--max-ambiguous needs a number")?
                    .parse()?;
            }
            "--strict" => strict = true,
            "--summary" => summary_only = true,
            "--json" => json = true,
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown check-text option {value:?}").into());
            }
            _ if path.is_none() => path = Some(argument),
            _ => return Err("check-text accepts exactly one input path".into()),
        }
    }
    if strict {
        policy = GenerationPolicy::Strict;
        max_unknown = 0;
        max_ambiguous = 0;
    }
    let path = path.ok_or("check-text requires a text path or - for stdin")?;
    let analyzer = analyzer(policy, profile, context)?;
    let report = check_text(
        &analyzer,
        &read_input(&path, input)?,
        CheckTextOptions {
            generation_policy: policy,
            orthography_profile: profile,
        },
    );
    if json {
        writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
    } else {
        if !summary_only {
            for analysis in &report.tokens {
                if let Some(gap) = &analysis.gap {
                    writeln!(
                        output,
                        "{}:{} {:?}: {} ({})",
                        analysis.token.line,
                        analysis.token.column,
                        analysis.token.original,
                        gap.kind.label(),
                        gap.detail,
                    )?;
                }
            }
        }
        writeln!(
            output,
            "text: {} tokens, {} types, {} top-k, {} ambiguous, {} unresolved",
            report.summary.total_tokens,
            report.summary.unique_tokens,
            report.summary.top_k_analyzed,
            report.summary.ambiguous_tokens,
            report.summary.unresolved_tokens,
        )?;
    }
    let unknown_tokens = report
        .summary
        .by_gap
        .get(GapKind::UnknownLexeme.label())
        .copied()
        .unwrap_or_default();
    if unknown_tokens > max_unknown {
        return Err(format!("{unknown_tokens} unknown token(s), maximum is {max_unknown}").into());
    }
    if report.summary.ambiguous_tokens > max_ambiguous {
        return Err(format!(
            "{} ambiguous token(s), maximum is {max_ambiguous}",
            report.summary.ambiguous_tokens
        )
        .into());
    }
    let top_k_uncovered = report
        .summary
        .total_tokens
        .saturating_sub(report.summary.top_k_analyzed);
    if strict && top_k_uncovered > 0 {
        return Err(format!("{top_k_uncovered} top-k-uncovered token(s) under --strict").into());
    }
    Ok(())
}

fn coverage_command(
    args: impl Iterator<Item = String>,
    input_reader: &mut dyn Read,
    output: &mut dyn Write,
    context: &CliContext,
) -> Result<(), Box<dyn Error>> {
    let mut input = None;
    let mut policy = GenerationPolicy::Strict;
    let mut profile = OrthographyProfile::SynodalLiturgical;
    let mut json = false;
    let mut markdown_path = None;
    let mut json_path = None;
    let mut tsv_path = None;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--policy" => policy = parse_policy(&args.next().ok_or("--policy needs a value")?)?,
            "--profile" => profile = parse_profile(&args.next().ok_or("--profile needs a value")?)?,
            // Family dimensions are always included in the serialized report;
            // this flag documents intent and keeps the CLI forward-compatible.
            "--by-family" => {}
            "--json" => json = true,
            "--markdown-out" => {
                markdown_path = Some(PathBuf::from(
                    args.next().ok_or("--markdown-out needs a path")?,
                ));
            }
            "--json-out" => {
                json_path = Some(PathBuf::from(args.next().ok_or("--json-out needs a path")?));
            }
            "--tsv-out" => {
                tsv_path = Some(PathBuf::from(args.next().ok_or("--tsv-out needs a path")?));
            }
            value if value.starts_with('-') && value != "-" => {
                return Err(format!("unknown coverage option {value:?}").into());
            }
            _ if input.is_none() => input = Some(argument),
            _ => return Err("coverage accepts exactly one JSONL or TSV input".into()),
        }
    }
    let input = input.ok_or("coverage requires a JSONL or TSV input")?;
    let passages = parse_passages(&read_input(&input, input_reader)?)?;
    let analyzer = analyzer(policy, profile, context)?;
    let report = coverage(
        &analyzer,
        &passages,
        CheckTextOptions {
            generation_policy: policy,
            orthography_profile: profile,
        },
    );
    let markdown = report.markdown();
    let json_text = format!("{}\n", serde_json::to_string_pretty(&report)?);
    let tsv = report.gaps_tsv();
    if let Some(path) = markdown_path {
        write_output(&path, &markdown)?;
    }
    if let Some(path) = json_path {
        write_output(&path, &json_text)?;
    }
    if let Some(path) = tsv_path {
        write_output(&path, &tsv)?;
    }
    if json {
        write!(output, "{json_text}")?;
    } else {
        write!(output, "{markdown}")?;
    }
    Ok(())
}

fn marginal_recovery_command(
    args: impl Iterator<Item = String>,
    output: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let mut input = None;
    let mut json = false;
    let mut limit = 50_usize;
    let mut route = None;
    let mut readiness = None;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--json" => json = true,
            "--limit" => limit = args.next().ok_or("--limit needs a number")?.parse()?,
            "--route" => route = Some(args.next().ok_or("--route needs a value")?),
            "--readiness" => {
                readiness = Some(parse_evidence_readiness(
                    &args.next().ok_or("--readiness needs a value")?,
                )?);
            }
            value if value.starts_with('-') => {
                return Err(format!("unknown marginal-recovery option {value:?}").into());
            }
            _ if input.is_none() => input = Some(PathBuf::from(argument)),
            _ => return Err("marginal-recovery accepts at most one JSON report".into()),
        }
    }
    let path = input.unwrap_or_else(|| PathBuf::from(DEFAULT_MARGINAL_RECOVERY));
    let mut report: MarginalRecoveryReport = serde_json::from_str(&fs::read_to_string(&path)?)?;
    report.batches.retain(|batch| {
        route
            .as_ref()
            .is_none_or(|expected| batch.recovery_route == *expected)
            && readiness.is_none_or(|expected| batch.evidence_readiness == expected)
    });
    report.batches.truncate(limit);
    if json {
        writeln!(output, "{}", serde_json::to_string_pretty(&report)?)?;
    } else {
        writeln!(
            output,
            "Synodal marginal recovery: current top-k {}, target {}, {} token(s) still needed. Showing {} batch(es) from {}.",
            report.current_top_k,
            report.target_top_k,
            report.tokens_needed_for_target,
            report.batches.len(),
            path.display(),
        )?;
        for batch in &report.batches {
            writeln!(
                output,
                "{}. {} [{}] — {} marginal token(s), {} cumulative; {:?} readiness / {:?} effort; {}",
                batch.rank,
                batch.label,
                batch.recovery_route,
                batch.overlap_adjusted_tokens,
                batch.cumulative_overlap_adjusted_tokens,
                batch.evidence_readiness,
                batch.review_effort,
                batch.review_status,
            )?;
            if !batch.missing_evidence.is_empty() {
                writeln!(output, "   missing: {}", batch.missing_evidence.join(", "))?;
            }
        }
    }
    Ok(())
}

fn parse_evidence_readiness(value: &str) -> Result<EvidenceReadiness, Box<dyn Error>> {
    match value {
        "blocked" => Ok(EvidenceReadiness::Blocked),
        "weak" => Ok(EvidenceReadiness::Weak),
        "partial" => Ok(EvidenceReadiness::Partial),
        "ready" => Ok(EvidenceReadiness::Ready),
        _ => Err(format!("unknown evidence readiness {value:?}").into()),
    }
}

fn analyzer(
    policy: GenerationPolicy,
    profile: OrthographyProfile,
    context: &CliContext,
) -> Result<std::sync::Arc<Analyzer>, Box<dyn Error>> {
    Ok(context.analyzers.get(
        synodal_church_slavonic_dictionary::morphology::Inflector::builder()
            .generation_policy(policy)
            .orthography(profile)
            .build(),
    )?)
}

fn parse_passages(input: &str) -> Result<Vec<CoveragePassage>, Box<dyn Error>> {
    let first = input
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    if first.trim_start().starts_with('{') {
        input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).map_err(Into::into))
            .collect()
    } else {
        parse_passage_tsv(input)
    }
}

fn parse_passage_tsv(input: &str) -> Result<Vec<CoveragePassage>, Box<dyn Error>> {
    const HEADER: &str =
        "corpus\tsource_id\twork\tedition\tpassage\tpartition\tsource_recension\ttext";
    let mut lines = input.lines();
    if lines.next() != Some(HEADER) {
        return Err(format!("coverage TSV header must be {HEADER:?}").into());
    }
    lines
        .enumerate()
        .filter(|(_, line)| !line.is_empty())
        .map(|(index, line)| {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != 8 {
                return Err(format!(
                    "coverage TSV line {} has {} fields",
                    index + 2,
                    fields.len()
                )
                .into());
            }
            Ok(CoveragePassage {
                corpus: fields[0].into(),
                source_id: fields[1].into(),
                work: fields[2].into(),
                edition: fields[3].into(),
                passage: fields[4].into(),
                partition: fields[5].into(),
                source_recension: fields[6].into(),
                text: fields[7].replace("\\n", "\n"),
            })
        })
        .collect()
}

fn print_entry(output: &mut dyn Write, entry: &Entry) -> io::Result<()> {
    writeln!(
        output,
        "{} [{}]\n  id: {}\n  source: {}\n  class: {}\n  stem: {}\n  gender: {}\n  aspect: {}",
        entry.lexeme.lemma(),
        pos_label(entry.lexeme.part_of_speech()),
        entry.lexeme.id(),
        entry.lexeme.source_id(),
        entry.metadata.class.as_deref().unwrap_or("—"),
        entry.metadata.stem.as_deref().unwrap_or("—"),
        entry.metadata.gender.as_deref().unwrap_or("—"),
        entry.metadata.aspect.as_deref().unwrap_or("—"),
    )?;
    for sense in &entry.senses {
        writeln!(
            output,
            "  sense: {} — {} [{}; {}]",
            sense.id, sense.gloss, sense.source_recension, sense.semantic_status
        )?;
    }
    for part in &entry.metadata.principal_parts {
        writeln!(
            output,
            "  principal part: {} = {}{} — {}",
            part.system,
            part.value,
            part.formation
                .as_deref()
                .map_or_else(String::new, |formation| format!(" ({formation})")),
            part.evidence_id,
        )?;
    }
    writeln!(
        output,
        "  exact forms: {} · accent rows: {} · missing metadata: {:?}",
        entry.metadata.exact_forms.len(),
        entry.metadata.accents.len(),
        entry.missing_metadata,
    )?;
    for example in &entry.examples {
        writeln!(
            output,
            "  example: {} — {} ({}, {})",
            example.text, example.translation, example.source_id, example.passage
        )?;
    }
    Ok(())
}

fn print_family(output: &mut dyn Write, family: &FamilySummary) -> io::Result<()> {
    writeln!(
        output,
        "{} [{}]\n  id: {}\n  lexeme: {}\n  class: {} · stem: {}\n  exact-only: {} · fully-classed: {}\n  systems: {}\n  missing metadata: {:?} {}",
        family.lexeme.lemma(),
        pos_label(family.lexeme.part_of_speech()),
        family.id,
        family.lexeme.id(),
        family.class.as_deref().unwrap_or("—"),
        family.stem.as_deref().unwrap_or("—"),
        family.exact_only,
        family.fully_classed,
        family.supported_systems.join(", "),
        family.missing_metadata,
        family.missing_family_metadata.join(", "),
    )?;
    for member in &family.members {
        writeln!(
            output,
            "  {}: {} / {} [{}; {}]",
            member.cell, member.expanded, member.printed, member.source_kind, member.evidence_id,
        )?;
    }
    Ok(())
}

fn print_proposed_family(output: &mut dyn Write, proposal: &serde_json::Value) -> io::Result<()> {
    writeln!(
        output,
        "{} [{}]\n  proposed id: {}\n  status: {} · confidence: {} bp\n  frequency: {} tokens / {} documents\n  compatible reviewed lexemes: {}\n  missing metadata: {}\n  reason: {}",
        proposal["proposed_lemma"].as_str().unwrap_or("—"),
        proposal["part_of_speech"].as_str().unwrap_or("unknown"),
        proposal["candidate_id"].as_str().unwrap_or("—"),
        proposal["review_status"].as_str().unwrap_or("—"),
        proposal["confidence_basis_points"]
            .as_u64()
            .unwrap_or_default(),
        proposal["token_frequency"].as_u64().unwrap_or_default(),
        proposal["document_frequency"].as_u64().unwrap_or_default(),
        proposal["compatible_existing_lexemes"]
            .as_array()
            .map(|values| values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default(),
        proposal["missing_metadata"]
            .as_array()
            .map(|values| values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default(),
        proposal["review_reason"].as_str().unwrap_or("—"),
    )?;
    if let Some(surfaces) = proposal["surfaces"].as_array() {
        for surface in surfaces {
            writeln!(
                output,
                "  surface: {} ({} tokens; cells: {})",
                surface["original"].as_str().unwrap_or("—"),
                surface["frequency"].as_u64().unwrap_or_default(),
                surface["possible_cells"]
                    .as_array()
                    .map(|values| values
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", "))
                    .unwrap_or_default(),
            )?;
        }
    }
    Ok(())
}

fn print_analysis(output: &mut dyn Write, index: usize, analysis: &Analysis) -> io::Result<()> {
    writeln!(
        output,
        "{}. {} [{}] {:?}\n   cell: {:?} · confidence: {} bp · matched: {}",
        index,
        analysis.lexeme.lemma(),
        analysis.lexeme.id(),
        analysis.source,
        analysis.cell,
        analysis.confidence.basis_points(),
        analysis.matched_text,
    )?;
    if !analysis.evidence_ids.is_empty() {
        writeln!(output, "   evidence: {}", analysis.evidence_ids.join(", "))?;
    }
    if let Some(mapping) = &analysis.recension_mapping {
        writeln!(output, "   recension mapping: {mapping}")?;
    }
    for step in analysis.rule_trace.steps() {
        writeln!(
            output,
            "   trace: {} {} → {}",
            step.stage, step.input, step.output
        )?;
    }
    Ok(())
}

fn parse_policy(value: &str) -> Result<GenerationPolicy, Box<dyn Error>> {
    match value {
        "strict" => Ok(GenerationPolicy::Strict),
        "productive" => Ok(GenerationPolicy::Productive),
        "exploratory" => Ok(GenerationPolicy::Exploratory),
        _ => Err(format!("unknown generation policy {value:?}").into()),
    }
}

fn parse_profile(value: &str) -> Result<OrthographyProfile, Box<dyn Error>> {
    match value {
        "expanded" => Ok(OrthographyProfile::Expanded),
        "accentless" | "expanded-accentless" => Ok(OrthographyProfile::ExpandedAccentless),
        "printed" | "liturgical" | "synodal-liturgical" => {
            Ok(OrthographyProfile::SynodalLiturgical)
        }
        _ => Err(format!("unknown orthography profile {value:?}").into()),
    }
}

fn parse_part_of_speech(
    value: &str,
) -> Result<synodal_church_slavonic_dictionary::morphology::PartOfSpeech, Box<dyn Error>> {
    use synodal_church_slavonic_dictionary::morphology::PartOfSpeech;
    let canonical = match value {
        "adv" => "adverb",
        "prep" => "preposition",
        "conj" => "conjunction",
        "intj" => "interjection",
        "name" => "proper-noun",
        "adj" => "adjective",
        "pron" => "pronoun",
        "det" => "determiner",
        "num" => "numeral",
        other => other,
    };
    PartOfSpeech::from_code(canonical)
        .ok_or_else(|| format!("unknown part of speech {value:?}").into())
}

fn pos_label(value: synodal_church_slavonic_dictionary::morphology::PartOfSpeech) -> &'static str {
    value.code()
}

fn read_input(path: &str, input: &mut dyn Read) -> Result<String, Box<dyn Error>> {
    if path == "-" {
        let mut contents = String::new();
        input.read_to_string(&mut contents)?;
        Ok(contents)
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

fn write_output(path: &Path, contents: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

fn help(diagnostics: &mut dyn Write) -> io::Result<()> {
    writeln!(diagnostics, "synodal-dict <command>")?;
    writeln!(
        diagnostics,
        "  search QUERY [--pos POS] [--limit N] [--exact] [--json]"
    )?;
    writeln!(diagnostics, "  show LEMMA_OR_ID [--json]")?;
    writeln!(
        diagnostics,
        "  families QUERY [--reviewed-only] [--proposals FAMILY_QUEUE.json] [--json]"
    )?;
    writeln!(
        diagnostics,
        "  show-family FAMILY_ID [--proposals FAMILY_QUEUE.json] [--json]"
    )?;
    writeln!(
        diagnostics,
        "  analyze WORD [--policy POLICY] [--profile PROFILE] [--json]
  analyze-text TEXT [--policy POLICY] [--profile PROFILE] [--json]"
    )?;
    writeln!(diagnostics, "  lint MANIFEST.json [--json]")?;
    writeln!(
        diagnostics,
        "  check-text TEXT [--policy POLICY] [--profile PROFILE] [--max-unknown N] [--max-ambiguous N] [--strict] [--summary] [--json]"
    )?;
    writeln!(
        diagnostics,
        "  coverage CORPUS.jsonl [--policy POLICY] [--profile PROFILE] [--by-family] [--json] [--markdown-out PATH] [--json-out PATH] [--tsv-out PATH]"
    )?;
    writeln!(
        diagnostics,
        "  marginal-recovery [REPORT.json] [--route ROUTE] [--readiness LEVEL] [--limit N] [--json]"
    )?;
    writeln!(
        diagnostics,
        "  use - as an input path to read UTF-8 from stdin"
    )
}
