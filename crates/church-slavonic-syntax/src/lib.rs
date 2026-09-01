//! Syntax trees that round-trip the Church Slavonic Bible.
//!
//! **The round-trip invariant** — the crate's one standing rule, never
//! weakened: for every verse that has a tree, `render(tree)` equals the
//! pinned print byte-for-byte. There is no other definition of correct.
//! Free generation (a new sentence from a new tree) is what the invariant
//! EARNS: a renderer that reproduces the attested verses can be trusted
//! with a tree it has never seen.
//!
//! The crate owns NO morphology — it is a consumer of `church-slavonic`,
//! exactly as the vertograd game is. It owns: the S-expression reader
//! ([`sexpr`]), the tree model and renderer ([`node`]), the closed-class
//! table ([`closed`]), the linter ([`lint`]), and the treebank pipeline
//! over the pinned Bible ([`bible`], [`lift`]).
//!
//! The escape hatch that makes the whole Bible reachable TODAY: the
//! `(w "…")` verbatim leaf. Every verse starts fully verbatim (round-trip
//! holds by construction) and progress is LIFTING leaves into analyzed
//! nodes — which succeeds only when the crate's output matches the
//! attested surface exactly. Analyzed-token coverage is the burn-down
//! number; a lift that fails where the crate should agree is a
//! consumer-found crate defect, fixed upstream in its proper layer,
//! never patched here.

pub mod bible;
pub mod closed;
pub mod lift;
pub mod lint;
pub mod node;
pub mod sexpr;
pub mod titlo;
pub mod treebank;
