//! Offline, deterministic source adapters for Synodal Russian Church Slavonic.
#![forbid(unsafe_code)]

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

pub use evidence::validate_candidate_links;
pub use generate::{generate_dictionary_registry, generate_registry};
pub use schema::{
    APPROVED_SOURCE_RECENSIONS, DictionaryGenerationReport, ExtractionError, GenerationReport,
    REGISTRY_SCHEMA_VERSION, Result, source_recension_is_approved,
};
