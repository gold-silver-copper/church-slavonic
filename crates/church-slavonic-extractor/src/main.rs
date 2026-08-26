#![forbid(unsafe_code)]

use std::env;

fn main() {
    let mut arguments = env::args().skip(1);
    let result = match arguments.next().as_deref() {
        Some("ocs") => church_slavonic_extractor::ocs::extract::run_cli(&mut arguments),
        Some("synodal") => church_slavonic_extractor::synodal::run_cli(&mut arguments),
        _ => {
            eprintln!("usage: church-slavonic-extractor <ocs|synodal> ...");
            eprintln!(
                "  ocs <refresh --dump PATH|check|report|dictionary-refresh --dump PATH|dictionary-check>"
            );
            eprintln!("  synodal DATA_DIR MORPHOLOGY_DESTINATION [DICTIONARY_DESTINATION]");
            eprintln!("  synodal candidates WORKSPACE_ROOT CACHE INTERMEDIATE QUARANTINE [SOURCE]");
            eprintln!("  synodal wikisource-split EXPORT REVISION_LOCK DESTINATION");
            std::process::exit(2);
        }
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
