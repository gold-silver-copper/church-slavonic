//! Re-export of the shared [`Recension`] axis.
//!
//! The type moved into `church-slavonic-core` (docs/UNIFIED_LANGUAGE_PROMPT.md,
//! phase-2 early task): the recension axis belongs to the shared kernel, not
//! to one family. This module keeps the public path
//! `synodal_church_slavonic_core::recension::Recension` byte-identical.

pub use church_slavonic_core::recension::Recension;
