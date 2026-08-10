use std::{env, path::Path};

use synodal_church_slavonic_extractor::{
    adapters::materialize_wikisource_export,
    generate_dictionary_registry, generate_registry,
    pipeline::{PipelineOptions, run_pipeline},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("synodal extraction failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    let first = arguments
        .next()
        .ok_or("usage: synodal-church-slavonic-extractor DATA_DIR DESTINATION")?;
    if first == "candidates" {
        let workspace_root = arguments
            .next()
            .ok_or("candidates requires WORKSPACE_ROOT CACHE INTERMEDIATE QUARANTINE [SOURCE]")?;
        let cache = arguments
            .next()
            .ok_or("candidates requires WORKSPACE_ROOT CACHE INTERMEDIATE QUARANTINE [SOURCE]")?;
        let intermediate = arguments
            .next()
            .ok_or("candidates requires WORKSPACE_ROOT CACHE INTERMEDIATE QUARANTINE [SOURCE]")?;
        let quarantine = arguments
            .next()
            .ok_or("candidates requires WORKSPACE_ROOT CACHE INTERMEDIATE QUARANTINE [SOURCE]")?;
        let source = arguments
            .next()
            .map(|value| value.to_string_lossy().into_owned());
        if arguments.next().is_some() {
            return Err(
                "candidates requires WORKSPACE_ROOT CACHE INTERMEDIATE QUARANTINE [SOURCE]".into(),
            );
        }
        let report = run_pipeline(&PipelineOptions {
            workspace_root: workspace_root.into(),
            cache: cache.into(),
            intermediate: intermediate.into(),
            quarantine: quarantine.into(),
            source,
            failure_ceiling: 10_000,
            keep_work: false,
        })?;
        println!(
            "accepted {}, quarantined {}, skipped {} candidate records",
            report.accepted_records, report.quarantined_records, report.skipped_records
        );
        return Ok(());
    }
    if first == "wikisource-split" {
        let export = arguments
            .next()
            .ok_or("wikisource-split requires EXPORT REVISION_LOCK DESTINATION")?;
        let revision_lock = arguments
            .next()
            .ok_or("wikisource-split requires EXPORT REVISION_LOCK DESTINATION")?;
        let destination = arguments
            .next()
            .ok_or("wikisource-split requires EXPORT REVISION_LOCK DESTINATION")?;
        if arguments.next().is_some() {
            return Err(
                "wikisource-split requires exactly EXPORT REVISION_LOCK DESTINATION".into(),
            );
        }
        let report = materialize_wikisource_export(
            Path::new(&export),
            Path::new(&revision_lock),
            Path::new(&destination),
        )?;
        println!(
            "materialized {} exact Wikisource revisions",
            report.accepted_rows
        );
        return Ok(());
    }
    let data_directory = first;
    let destination = arguments
        .next()
        .ok_or("usage: synodal-church-slavonic-extractor DATA_DIR DESTINATION")?;
    let dictionary_destination = arguments.next();
    if arguments.next().is_some() {
        return Err(
            "usage: synodal-church-slavonic-extractor DATA_DIR MORPHOLOGY_DESTINATION [DICTIONARY_DESTINATION]"
                .into(),
        );
    }

    let report = generate_registry(Path::new(&data_directory), Path::new(&destination))?;
    println!(
        "generated {} lexemes, {} principal parts, {} exact forms, {} accents, {} alignments, {} abbreviations; sha256 {}",
        report.lexemes,
        report.principal_parts,
        report.exact_forms,
        report.accents,
        report.alignments,
        report.abbreviations,
        report.output_sha256
    );
    if let Some(dictionary_destination) = dictionary_destination {
        let dictionary_report = generate_dictionary_registry(
            Path::new(&data_directory),
            Path::new(&dictionary_destination),
        )?;
        println!(
            "generated {} senses and {} examples; sha256 {}",
            dictionary_report.senses, dictionary_report.examples, dictionary_report.output_sha256
        );
    }
    Ok(())
}
