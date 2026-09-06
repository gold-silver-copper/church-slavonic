//! The library's errors, named: what a call could not do and why. Absence
//! is not an error (`get`, `find` stay `Option`); a malformed name, a
//! malformed lexicon line and a cell a lexeme cannot inflect are.

use crate::cell::{Cell, Pos};
use std::fmt;

/// A cell name the part of speech's grammar does not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellError {
    pub pos: Pos,
    pub text: String,
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "«{}» is not a {} cell", self.text, self.pos.tag())
    }
}

impl std::error::Error for CellError {}

/// A lexicon line the parser could not read: the line number (1-based)
/// and what was wrong with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexiconError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for LexiconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for LexiconError {}

/// Why a lexeme has no form for a cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InflectError {
    /// The cell belongs to another part of speech (a noun asked for an
    /// aorist).
    NotThisPartOfSpeech { pos: Pos, cell: Cell },
    /// The lexeme's class is not in the class table (a lexicon defect,
    /// or a guessed lexeme's class the table lacks).
    NoClass { class: String },
    /// The class declares no such cell (a short-only adjective asked for
    /// the long series, a verb class without the l-participle).
    NoSuchCell { class: String, cell: Cell },
}

impl fmt::Display for InflectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InflectError::NotThisPartOfSpeech { pos, cell } => write!(f, "{} is not a cell of a {}", cell.name(), pos.tag()),
            InflectError::NoClass { class } => write!(f, "class {class} is not in the table"),
            InflectError::NoSuchCell { class, cell } => write!(f, "class {class} declares no cell {}", cell.name()),
        }
    }
}

impl std::error::Error for InflectError {}
