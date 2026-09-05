//! The Old Church Slavonic class tables (`lexicon/classes/ocs/*.tsv`),
//! seeded from Kaikki's own paradigm tables by
//! `scripts/kaikki-to-classes.py` and hand-maintained since; the pronoun
//! table is hand-written there. See the module docs of [`super`].

pub const NOUN: &str = include_str!("../../lexicon/classes/ocs/noun.tsv");
pub const ADJ: &str = include_str!("../../lexicon/classes/ocs/adj.tsv");
pub const VERB: &str = include_str!("../../lexicon/classes/ocs/verb.tsv");
pub const PRONOUN: &str = include_str!("../../lexicon/classes/ocs/pronoun.tsv");
