//! The noun class table (`lexicon/classes/noun.tsv`), seeded from
//! Polyakov's paradigm legend by `scripts/polyakov-legend-to-classes.py`
//! and hand-maintained since. See the module docs of [`super`].

pub const TABLE: &str = include_str!("../../lexicon/classes/noun.tsv");
