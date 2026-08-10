use std::{env, path::Path};

use synodal_church_slavonic_extractor::{generate_dictionary_registry, generate_registry};

fn main() {
    if let Err(error) = run() {
        eprintln!("synodal extraction failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args_os().skip(1);
    let data_directory = arguments
        .next()
        .ok_or("usage: synodal-church-slavonic-extractor DATA_DIR DESTINATION")?;
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
