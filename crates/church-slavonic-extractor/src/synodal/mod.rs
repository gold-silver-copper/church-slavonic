//! The reviews/evidence/registry pipeline for Synodal Russian Church Slavonic
//! (recension: Synodal).
//!
//! Ingests pinned sources from `references/` into candidate records, reviews
//! them against the curated `data/synodal/*.tsv` tables, and generates the
//! Synodal morphology and dictionary registries. See
//! `docs/SYNODAL_DATA_PIPELINE.md`.

pub mod adapters;
pub mod pipeline;

mod emit;
mod evidence;
mod generate;
mod reviews;
mod schema;
mod validate_grammar;
mod validate_registry;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub(crate) use emit::*;
#[allow(unused_imports)]
pub(crate) use evidence::*;
#[allow(unused_imports)]
pub(crate) use generate::*;
#[allow(unused_imports)]
pub(crate) use reviews::*;
#[allow(unused_imports)]
pub(crate) use schema::*;
#[allow(unused_imports)]
pub(crate) use validate_grammar::*;
#[allow(unused_imports)]
pub(crate) use validate_registry::*;

use adapters::materialize_wikisource_export;
use pipeline::{PipelineOptions, run_pipeline};
use std::path::Path;

pub use evidence::validate_candidate_links;
pub use generate::{generate_dictionary_registry, generate_registry};
pub use schema::{
    APPROVED_SOURCE_RECENSIONS, DictionaryGenerationReport, ExtractionError, GenerationReport,
    REGISTRY_SCHEMA_VERSION, Result, source_recension_is_approved,
};

/// Command-line entry of the Synodal pipeline (`church-slavonic-extractor synodal ...`).
///
/// # Errors
///
/// Returns the first pipeline, generation, or argument error.
pub fn run_cli(
    arguments: &mut dyn Iterator<Item = String>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let first = arguments
        .next()
        .ok_or("usage: church-slavonic-extractor synodal DATA_DIR DESTINATION")?;
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
        let source = arguments.next().map(|value| value.to_owned());
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
        .ok_or("usage: church-slavonic-extractor synodal DATA_DIR DESTINATION")?;
    let dictionary_destination = arguments.next();
    if arguments.next().is_some() {
        return Err(
            "usage: church-slavonic-extractor synodal DATA_DIR MORPHOLOGY_DESTINATION [DICTIONARY_DESTINATION]"
                .into(),
        );
    }

    let report = generate_registry(Path::new(&data_directory), Path::new(&destination))?;
    println!(
        "generated {} lexemes, {} principal parts, {} exact forms, {} accents, {} alignments, {} abbreviations, {} defective inventories, {} irregular inventory entries; sha256 {}",
        report.lexemes,
        report.principal_parts,
        report.exact_forms,
        report.accents,
        report.alignments,
        report.abbreviations,
        report.defective_inventories,
        report.irregular_inventory_entries,
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
