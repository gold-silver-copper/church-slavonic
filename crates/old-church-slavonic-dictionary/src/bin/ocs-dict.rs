use old_church_slavonic_dictionary::{
    MatchKind, SOURCE_MANIFEST, SearchOptions, Sense, TextTokenStatus, check_text, lookup, search,
    validate_vocabulary_tsv,
};
use serde_json::json;
use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("search") => search_command(args),
        Some("show") => show_command(args),
        Some("compare") => compare_command(args),
        Some("lint") => lint_command(args),
        Some("check-text") => check_text_command(args),
        Some("source") => {
            print!("{SOURCE_MANIFEST}");
            Ok(())
        }
        Some("help") | Some("-h") | Some("--help") | None => {
            help();
            Ok(())
        }
        Some(other) => Err(format!("unknown ocs-dict command: {other}").into()),
    }
}

fn search_command(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut query = Vec::new();
    let mut options = SearchOptions::default();
    let mut json_output = false;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--limit" => {
                options.limit = args.next().ok_or("--limit needs a number")?.parse()?;
            }
            "--pos" => options.part_of_speech = Some(args.next().ok_or("--pos needs a value")?),
            "--topic" => options.topic = Some(args.next().ok_or("--topic needs a value")?),
            "--json" => json_output = true,
            _ if argument.starts_with('-') => {
                return Err(format!("unknown search option: {argument}").into());
            }
            _ => query.push(argument),
        }
    }
    let query = query.join(" ");
    let results = search(&query, &options)?;
    if json_output {
        let values = results
            .iter()
            .map(|result| {
                let sense = result.sense();
                json!({
                    "score": result.score(),
                    "matched_on": result.matched_on(),
                    "sense": sense_json(sense),
                })
            })
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&values)?);
    } else if results.is_empty() {
        println!("No source-backed OCS senses matched {query:?}.");
    } else {
        for (index, result) in results.iter().enumerate() {
            let sense = result.sense();
            println!(
                "{}. {} [{}] — {}\n   sense: {} · score: {} · match: {}{}",
                index + 1,
                sense.lemma(),
                sense.part_of_speech(),
                sense.glosses().join("; "),
                sense.id(),
                result.score(),
                match_label(result.matched_on()),
                if sense.inflection_lexeme_id().is_some() {
                    " · inflection-linked"
                } else {
                    ""
                }
            );
        }
    }
    Ok(())
}

fn show_command(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let lemma = args.next().ok_or("show needs an OCS lemma")?;
    let json_output = args.next().as_deref() == Some("--json");
    let senses = lookup(&lemma)?;
    if json_output {
        let values = senses.iter().copied().map(sense_json).collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&values)?);
    } else if senses.is_empty() {
        println!("No lexical OCS sense found for {lemma}.");
    } else {
        print_senses(&senses);
    }
    Ok(())
}

fn compare_command(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let lemmas = args.collect::<Vec<_>>();
    if lemmas.len() < 2 {
        return Err("compare needs at least two OCS lemmas".into());
    }
    for lemma in lemmas {
        println!("# {lemma}");
        let senses = lookup(&lemma)?;
        if senses.is_empty() {
            println!("No lexical sense.\n");
        } else {
            print_senses(&senses);
            println!();
        }
    }
    Ok(())
}

fn lint_command(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let path = PathBuf::from(args.next().ok_or("lint needs a vocabulary TSV path")?);
    let json_output = args.next().as_deref() == Some("--json");
    let report = validate_vocabulary_tsv(&fs::read_to_string(&path)?);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        for issue in &report.issues {
            println!("line {}: {:?}: {}", issue.line, issue.level, issue.message);
        }
        println!(
            "vocabulary: {} rows ({} attested, {} thematic, {} proper names)",
            report.rows, report.attested, report.thematic, report.proper_names
        );
    }
    if report.is_ok() {
        Ok(())
    } else {
        Err(format!("{} vocabulary issue(s)", report.issues.len()).into())
    }
}

fn check_text_command(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut path = None;
    let mut allow_path = None;
    let mut max_unknown = 0_usize;
    let mut json_output = false;
    let mut summary_only = false;
    let mut args = args.peekable();
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--allow" => {
                allow_path = Some(PathBuf::from(args.next().ok_or("--allow needs a path")?))
            }
            "--max-unknown" => {
                max_unknown = args.next().ok_or("--max-unknown needs a number")?.parse()?;
            }
            "--json" => json_output = true,
            "--summary" => summary_only = true,
            _ if argument.starts_with('-') => {
                return Err(format!("unknown check-text option: {argument}").into());
            }
            _ if path.is_none() => path = Some(PathBuf::from(argument)),
            _ => return Err("check-text accepts exactly one text path".into()),
        }
    }
    let path = path.ok_or("check-text needs a text path")?;
    let allowlisted = allow_path.map_or_else(
        || Ok::<_, Box<dyn Error>>(BTreeSet::new()),
        |path| read_allowlist(&path),
    )?;
    let report = check_text(&fs::read_to_string(&path)?, &allowlisted);
    if json_output {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        if !summary_only {
            for analysis in &report.analyses {
                if analysis.status == TextTokenStatus::Unknown {
                    println!(
                        "{}:{}: unknown OCS token {:?}",
                        analysis.line, analysis.column, analysis.token
                    );
                }
            }
        }
        println!(
            "text: {} tokens, {} distinct, {} unknown",
            report.total_tokens, report.unique_tokens, report.unknown_tokens
        );
    }
    if report.unknown_tokens <= max_unknown {
        Ok(())
    } else {
        Err(format!(
            "{} unknown token(s), maximum is {max_unknown}",
            report.unknown_tokens
        )
        .into())
    }
}

fn read_allowlist(path: &Path) -> Result<BTreeSet<String>, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .map(|line| line.split('#').next().unwrap_or_default().trim())
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

fn print_senses(senses: &[Sense]) {
    for sense in senses {
        println!(
            "{} [{}]\n  {}\n  sense: {}{}",
            sense.lemma(),
            sense.part_of_speech(),
            sense.glosses().join("; "),
            sense.id(),
            sense
                .inflection_lexeme_id()
                .map_or(String::new(), |id| format!("\n  inflection: {id}"))
        );
        for example in sense.examples().take(2) {
            println!("  example: {}", example.text().replace('\n', " "));
            if let Some(translation) = example.translation() {
                println!("           {translation}");
            }
            if let Some(reference) = example.reference() {
                println!("           — {reference}");
            }
        }
    }
}

fn sense_json(sense: Sense) -> serde_json::Value {
    let examples = sense
        .examples()
        .map(|example| {
            json!({
                "text": example.text(),
                "romanization": example.romanization(),
                "translation": example.translation(),
                "reference": example.reference(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "id": sense.id(),
        "source_sense_id": sense.source_sense_id(),
        "lemma": sense.lemma(),
        "source_spelling": sense.source_spelling(),
        "part_of_speech": sense.part_of_speech(),
        "glosses": sense.glosses(),
        "raw_glosses": sense.raw_glosses(),
        "tags": sense.tags(),
        "topics": sense.topics(),
        "examples": examples,
        "inflection_lexeme_id": sense.inflection_lexeme_id(),
        "source": old_church_slavonic_dictionary::SOURCE_NAME,
        "license": old_church_slavonic_dictionary::SOURCE_LICENSE,
    })
}

fn match_label(kind: MatchKind) -> &'static str {
    match kind {
        MatchKind::Lemma => "lemma",
        MatchKind::ExactGloss => "exact gloss",
        MatchKind::GlossPhrase => "gloss phrase",
        MatchKind::GlossWords => "gloss words",
        MatchKind::Topic => "topic",
    }
}

fn help() {
    eprintln!("ocs-dict <command>");
    eprintln!("  search QUERY [--pos POS] [--topic TOPIC] [--limit N] [--json]");
    eprintln!("  show LEMMA [--json]");
    eprintln!("  compare LEMMA LEMMA [...]");
    eprintln!("  lint VOCABULARY.tsv [--json]");
    eprintln!("  check-text TEXT [--allow FILE] [--max-unknown N] [--summary] [--json]");
    eprintln!("  source");
}
