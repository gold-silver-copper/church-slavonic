//! TEMPORARY (Parts 0–2): conversions between the 2.0 grammar enums and
//! the legacy crate's, so the treebank can keep rendering its lemma-keyed
//! leaves through `church-slavonic-legacy` until Part 2 re-lifts it onto
//! lexeme ids. Deleted with the legacy dependency.

use church_slavonic::grammar as new;
use church_slavonic_legacy as old;

pub trait Legacy: Sized {
    type Old;
    fn to_legacy(self) -> Self::Old;
    fn from_legacy(old: Self::Old) -> Self;
}

macro_rules! map_enum {
    ($new:ty, $old:ty, $($v:ident),+) => {
        impl Legacy for $new {
            type Old = $old;
            fn to_legacy(self) -> $old {
                match self { $(<$new>::$v => <$old>::$v),+ }
            }
            fn from_legacy(o: $old) -> Self {
                match o { $(<$old>::$v => <$new>::$v),+ }
            }
        }
    };
}

map_enum!(new::Case, old::Case, Nominative, Genitive, Dative, Accusative, Instrumental, Locative, Vocative);
map_enum!(new::Number, old::Number, Singular, Dual, Plural);
map_enum!(new::Gender, old::Gender, Masculine, Feminine, Neuter);
map_enum!(new::Person, old::Person, First, Second, Third);
map_enum!(new::Tense, old::Tense, Present, Imperfect, Aorist);
map_enum!(new::Form, old::Form, Finite, Participle, Infinitive, Imperative);
map_enum!(new::Voice, old::Voice, Active, Passive);
map_enum!(new::Series, old::Series, Short, Long);
map_enum!(new::Degree, old::Degree, Positive, Comparative, Superlative);
map_enum!(new::Recension, old::Recension, OldChurchSlavonic, Synodal);

/// Shorthand: `l(x)` converts a 2.0 value to its legacy twin.
pub fn l<T: Legacy>(x: T) -> T::Old {
    x.to_legacy()
}
