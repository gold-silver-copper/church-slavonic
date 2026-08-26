mod context;
mod lexeme;
mod nominal;
mod validate;
mod verb;

#[cfg(test)]
mod tests;

pub(crate) use context::SpecContext;
pub use context::{DefectKind, DefectiveCell, SpecificationSource};
pub use lexeme::LexemeSpec;
pub(crate) use lexeme::LexemeSpecInner;
pub use nominal::{AdjectiveSpec, DeterminerSpec, NounSpec, NumeralSpec, PronounSpec};
pub(crate) use validate::{validate_context_cells, validate_pair, validate_participle};
pub use verb::{VerbSpec, VerbSpecBuilder};
