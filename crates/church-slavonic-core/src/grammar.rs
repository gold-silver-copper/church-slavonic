//! Closed grammatical category vocabulary shared by both crate families.

macro_rules! closed_enum {
    ($name:ident { $($variant:ident => $code:literal / $abbrev:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const ALL: [Self; closed_enum!(@count $($variant),+)] = [$(Self::$variant),+];

            /// The long spelling used by the Synodal data pipeline.
            #[must_use]
            pub const fn code(self) -> &'static str {
                match self {
                    $(Self::$variant => $code),+
                }
            }

            /// The short spelling used by the OCS data pipeline.
            #[must_use]
            pub const fn abbrev(self) -> &'static str {
                match self {
                    $(Self::$variant => $abbrev),+
                }
            }

            #[must_use]
            pub fn from_code(value: &str) -> Option<Self> {
                match value {
                    $($code => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
    (@count $($variant:ident),+) => {
        <[()]>::len(&[$(closed_enum!(@unit $variant)),+])
    };
    (@unit $variant:ident) => { () };
}

closed_enum!(Case {
    Nominative => "nominative" / "nom",
    Genitive => "genitive" / "gen",
    Dative => "dative" / "dat",
    Accusative => "accusative" / "acc",
    Instrumental => "instrumental" / "ins",
    Locative => "locative" / "loc",
    Vocative => "vocative" / "voc",
});
closed_enum!(Number {
    Singular => "singular" / "sg",
    Dual => "dual" / "du",
    Plural => "plural" / "pl",
});
closed_enum!(Gender {
    Masculine => "masculine" / "m",
    Feminine => "feminine" / "f",
    Neuter => "neuter" / "n",
});
// Variant order follows the Synodal declaration; the OCS family's historical
// `ALL` order (Animate first) lives in its own compatibility shim until its
// enumeration-order consumers are migrated.
closed_enum!(Animacy {
    Inanimate => "inanimate" / "in",
    Animate => "animate" / "an",
});
closed_enum!(Person {
    First => "first" / "1",
    Second => "second" / "2",
    Third => "third" / "3",
});
closed_enum!(AdjectiveForm {
    Short => "short" / "short",
    Long => "long" / "long",
});
closed_enum!(Comparison {
    Positive => "positive" / "positive",
    Comparative => "comparative" / "comparative",
    Superlative => "superlative" / "superlative",
});
closed_enum!(Voice {
    Active => "active" / "active",
    Middle => "middle" / "middle",
    Passive => "passive" / "passive",
});
// `Past` is retained only as a source-adapter normalization tag for
// caller-supplied exact specifications. The audited Synodal target registry
// contains no `past:*` cells: target forms are classified as aorist or
// imperfect.
closed_enum!(FiniteTense {
    Present => "present" / "present",
    Future => "future" / "future",
    Past => "past" / "past",
    Imperfect => "imperfect" / "imperfect",
    Aorist => "aorist" / "aorist",
});
closed_enum!(ParticipleTense {
    Present => "present" / "present",
    Past => "past" / "past",
});
closed_enum!(ParticipleVoice {
    Active => "active" / "active",
    Passive => "passive" / "passive",
});
closed_enum!(NumeralKind {
    Cardinal => "cardinal" / "cardinal",
    Ordinal => "ordinal" / "ordinal",
    Collective => "collective" / "collective",
    Multiplicative => "multiplicative" / "multiplicative",
    Fractional => "fractional" / "fractional",
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_round_trips() {
        for case in Case::ALL {
            assert_eq!(Case::from_code(case.code()), Some(case));
        }
        for number in Number::ALL {
            assert_eq!(Number::from_code(number.code()), Some(number));
        }
    }

    #[test]
    fn abbrevs_match_the_ocs_registry_spellings() {
        assert_eq!(Case::Nominative.abbrev(), "nom");
        assert_eq!(Number::Dual.abbrev(), "du");
        assert_eq!(Gender::Neuter.abbrev(), "n");
        assert_eq!(Person::Third.abbrev(), "3");
        assert_eq!(Animacy::Animate.abbrev(), "an");
    }
}
