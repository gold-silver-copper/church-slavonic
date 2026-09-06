//! The Bible treebank: S-expression trees that round-trip the pinned
//! print byte-for-byte (the invariant). Ported from the 1.x syntax crate;
//! every analyzed leaf carries a lexeme id and a cell of the 2.0 lexicon
//! (`(n землѧ.n :case acc :num sg)`, `(v рещи.v :t aor :p 3 :num sg)`).

pub mod bible;
pub mod closed;
pub mod corpus;
pub mod disambiguate;
pub mod export;
pub mod tag;
pub mod lift;
pub mod lint;
pub mod node;
pub mod runner;
pub mod sexpr;
pub mod titlo;
