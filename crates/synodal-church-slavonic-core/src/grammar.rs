macro_rules! closed_enum {
    ($name:ident { $($variant:ident => $code:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const ALL: [Self; closed_enum!(@count $($variant),+)] = [$(Self::$variant),+];

            #[must_use]
            pub const fn code(self) -> &'static str {
                match self {
                    $(Self::$variant => $code),+
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
    Nominative => "nominative",
    Genitive => "genitive",
    Dative => "dative",
    Accusative => "accusative",
    Instrumental => "instrumental",
    Locative => "locative",
    Vocative => "vocative",
});
closed_enum!(Number {
    Singular => "singular",
    Dual => "dual",
    Plural => "plural",
});
closed_enum!(Gender {
    Masculine => "masculine",
    Feminine => "feminine",
    Neuter => "neuter",
});
closed_enum!(Animacy {
    Inanimate => "inanimate",
    Animate => "animate",
});
closed_enum!(Person {
    First => "first",
    Second => "second",
    Third => "third",
});
closed_enum!(AdjectiveForm {
    Short => "short",
    Long => "long",
});
closed_enum!(Comparison {
    Positive => "positive",
    Comparative => "comparative",
    Superlative => "superlative",
});
closed_enum!(Voice {
    Active => "active",
    Middle => "middle",
    Passive => "passive",
});
// `Past` represents a source-typed finite past whose evidence does not
// distinguish aorist from imperfect. It is exact-only and is never
// productively generated.
closed_enum!(FiniteTense {
    Present => "present",
    Future => "future",
    Past => "past",
    Imperfect => "imperfect",
    Aorist => "aorist",
});
closed_enum!(ParticipleTense {
    Present => "present",
    Past => "past",
});
closed_enum!(ParticipleVoice {
    Active => "active",
    Passive => "passive",
});
closed_enum!(NumeralKind {
    Cardinal => "cardinal",
    Ordinal => "ordinal",
    Collective => "collective",
});

/// A complete verb-system inventory understood by the public paradigm API.
/// Exact-only systems remain represented here so their unsupported or missing
/// cells stay visible instead of disappearing from a partial table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum VerbSystem {
    Finite(FiniteTense),
    Imperative,
    Infinitive,
    LParticiple,
    Participle {
        tense: ParticipleTense,
        voice: ParticipleVoice,
        form: AdjectiveForm,
    },
    Supine,
    VerbalNoun {
        animacy: Animacy,
    },
}

impl VerbSystem {
    pub const ALL: [Self; 19] = [
        Self::Finite(FiniteTense::Present),
        Self::Finite(FiniteTense::Future),
        Self::Finite(FiniteTense::Past),
        Self::Finite(FiniteTense::Imperfect),
        Self::Finite(FiniteTense::Aorist),
        Self::Imperative,
        Self::Infinitive,
        Self::LParticiple,
        Self::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Short,
        },
        Self::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Long,
        },
        Self::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Short,
        },
        Self::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Long,
        },
        Self::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Short,
        },
        Self::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Long,
        },
        Self::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Short,
        },
        Self::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Long,
        },
        Self::Supine,
        Self::VerbalNoun {
            animacy: Animacy::Inanimate,
        },
        Self::VerbalNoun {
            animacy: Animacy::Animate,
        },
    ];
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NounCell {
    pub case: Case,
    pub number: Number,
    pub animacy: Animacy,
}

impl NounCell {
    /// Enumerates the nominal case/number inventory for the supplied animacies.
    #[must_use]
    pub fn inventory(animacies: &[Animacy]) -> Vec<Self> {
        let mut cells = Vec::new();
        for number in Number::ALL {
            for case in Case::ALL {
                for &animacy in animacies {
                    cells.push(Self {
                        case,
                        number,
                        animacy,
                    });
                }
            }
        }
        cells
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AdjectiveCell {
    pub case: Case,
    pub number: Number,
    pub gender: Gender,
    pub animacy: Animacy,
    pub form: AdjectiveForm,
    pub comparison: Comparison,
}

impl AdjectiveCell {
    /// Enumerates agreement cells while leaving the caller's animacy policy
    /// explicit. This keeps exhaustive analysis distinct from canonical
    /// paradigm presentation.
    #[must_use]
    pub fn inventory<'a>(
        forms: &[AdjectiveForm],
        comparisons: &[Comparison],
        animacies_for_case: impl Fn(Case) -> &'a [Animacy],
    ) -> Vec<Self> {
        let mut cells = Vec::new();
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    for &animacy in animacies_for_case(case) {
                        for &form in forms {
                            for &comparison in comparisons {
                                cells.push(Self {
                                    case,
                                    number,
                                    gender,
                                    animacy,
                                    form,
                                    comparison,
                                });
                            }
                        }
                    }
                }
            }
        }
        cells
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FiniteVerbCell {
    pub tense: FiniteTense,
    pub person: Person,
    pub number: Number,
}

impl FiniteVerbCell {
    /// Enumerates finite cells for the supplied tenses.
    #[must_use]
    pub fn inventory(tenses: &[FiniteTense]) -> Vec<Self> {
        let mut cells = Vec::new();
        for &tense in tenses {
            for number in Number::ALL {
                for person in Person::ALL {
                    cells.push(Self {
                        tense,
                        person,
                        number,
                    });
                }
            }
        }
        cells
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ImperativeCell {
    pub person: Person,
    pub number: Number,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LParticipleCell {
    pub gender: Gender,
    pub number: Number,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ParticipleCell {
    pub tense: ParticipleTense,
    pub voice: ParticipleVoice,
    pub agreement: AdjectiveCell,
}

impl ParticipleCell {
    /// Combines supplied participial systems with an explicit agreement
    /// inventory, preserving the order of every input dimension.
    #[must_use]
    pub fn inventory(
        tenses: &[ParticipleTense],
        voices: &[ParticipleVoice],
        agreements: &[AdjectiveCell],
    ) -> Vec<Self> {
        let mut cells = Vec::new();
        for &tense in tenses {
            for &voice in voices {
                for &agreement in agreements {
                    cells.push(Self {
                        tense,
                        voice,
                        agreement,
                    });
                }
            }
        }
        cells
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PronounCell {
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
    pub person: Option<Person>,
    pub animacy: Animacy,
}

impl PronounCell {
    /// Enumerates case/number/animacy cells for the caller's validated
    /// gender/person profiles.
    #[must_use]
    pub fn inventory(profiles: &[(Option<Gender>, Option<Person>)]) -> Vec<Self> {
        let mut cells = Vec::new();
        for number in Number::ALL {
            for case in Case::ALL {
                for &(gender, person) in profiles {
                    for animacy in Animacy::ALL {
                        cells.push(Self {
                            case,
                            number,
                            gender,
                            person,
                            animacy,
                        });
                    }
                }
            }
        }
        cells
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NumeralCell {
    pub kind: NumeralKind,
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
    pub animacy: Animacy,
}

impl NumeralCell {
    /// Enumerates numeral cells for explicit kind and optional-gender
    /// inventories.
    #[must_use]
    pub fn inventory(kinds: &[NumeralKind], genders: &[Option<Gender>]) -> Vec<Self> {
        let mut cells = Vec::new();
        for &kind in kinds {
            for number in Number::ALL {
                for case in Case::ALL {
                    for &gender in genders {
                        for animacy in Animacy::ALL {
                            cells.push(Self {
                                kind,
                                case,
                                number,
                                gender,
                                animacy,
                            });
                        }
                    }
                }
            }
        }
        cells
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum GrammarCell {
    /// A reviewed dictionary headword attested in target-recension text when
    /// its inflectional cell has not yet been independently established.
    /// This is lexical evidence only and never enables productive inflection.
    LexicalForm,
    /// Exact lexical form for adverbs, prepositions, conjunctions, particles,
    /// and interjections. This cell never enables productive inflection.
    Indeclinable,
    Noun(NounCell),
    Adjective(AdjectiveCell),
    FiniteVerb(FiniteVerbCell),
    Imperative(ImperativeCell),
    Infinitive,
    Supine,
    LParticiple(LParticipleCell),
    Participle(ParticipleCell),
    VerbalNoun(NounCell),
    Pronoun(PronounCell),
    Determiner(AdjectiveCell),
    Numeral(NumeralCell),
}

impl GrammarCell {
    /// Return the canonical stable key for a fully typed runtime cell.
    ///
    /// Registry-only `any` animacy keys parse to a neutral typed representative.
    /// Callers that need to round-trip such wildcard keys must retain the raw
    /// registry key alongside this value.
    #[must_use]
    pub fn key(self) -> String {
        match self {
            Self::LexicalForm => "lexical-form".into(),
            Self::Indeclinable => "indeclinable".into(),
            Self::Noun(cell) => format!(
                "noun:{}:{}:{}",
                cell.case.code(),
                cell.number.code(),
                cell.animacy.code()
            ),
            Self::Adjective(cell) => adjective_key("adjective", cell),
            Self::FiniteVerb(cell) => format!(
                "{}:{}:{}",
                cell.tense.code(),
                cell.person.code(),
                cell.number.code()
            ),
            Self::Imperative(cell) => {
                format!("imperative:{}:{}", cell.person.code(), cell.number.code())
            }
            Self::Infinitive => "infinitive".into(),
            Self::Supine => "supine".into(),
            Self::LParticiple(cell) => {
                format!("l-participle:{}:{}", cell.gender.code(), cell.number.code())
            }
            Self::Participle(cell) => format!(
                "participle:{}:{}:{}:{}:{}:{}:{}:{}",
                cell.tense.code(),
                cell.voice.code(),
                cell.agreement.case.code(),
                cell.agreement.number.code(),
                cell.agreement.gender.code(),
                cell.agreement.animacy.code(),
                cell.agreement.form.code(),
                cell.agreement.comparison.code()
            ),
            Self::VerbalNoun(cell) => format!(
                "verbal-noun:{}:{}:{}",
                cell.case.code(),
                cell.number.code(),
                cell.animacy.code()
            ),
            Self::Pronoun(cell) => format!(
                "pronoun:{}:{}:{}:{}:{}",
                cell.case.code(),
                cell.number.code(),
                cell.gender.map_or("any", Gender::code),
                cell.person.map_or("none", Person::code),
                cell.animacy.code()
            ),
            Self::Determiner(cell) => adjective_key("determiner", cell),
            Self::Numeral(cell) => format!(
                "numeral:{}:{}:{}:{}:{}",
                cell.kind.code(),
                cell.case.code(),
                cell.number.code(),
                cell.gender.map_or("any", Gender::code),
                cell.animacy.code()
            ),
        }
    }
}

impl std::str::FromStr for GrammarCell {
    type Err = crate::Error;

    fn from_str(value: &str) -> crate::Result<Self> {
        let fields: Vec<_> = value.split(':').collect();
        match fields.as_slice() {
            ["lexical-form"] => Ok(Self::LexicalForm),
            ["indeclinable"] => Ok(Self::Indeclinable),
            ["noun", case, number, animacy] => Ok(Self::Noun(NounCell {
                case: required_code("case", case, Case::from_code)?,
                number: required_code("number", number, Number::from_code)?,
                animacy: required_code("animacy", animacy, Animacy::from_code)?,
            })),
            ["verbal-noun", case, number, animacy] => Ok(Self::VerbalNoun(NounCell {
                case: required_code("case", case, Case::from_code)?,
                number: required_code("number", number, Number::from_code)?,
                animacy: required_code("animacy", animacy, Animacy::from_code)?,
            })),
            ["adjective", case, number, gender, animacy, form, comparison] => Ok(Self::Adjective(
                parse_adjective_cell(case, number, gender, animacy, form, comparison)?,
            )),
            [
                "determiner",
                case,
                number,
                gender,
                animacy,
                form,
                comparison,
            ] => Ok(Self::Determiner(parse_adjective_cell(
                case, number, gender, animacy, form, comparison,
            )?)),
            [
                tense @ ("present" | "future" | "past" | "imperfect" | "aorist"),
                person,
                number,
            ] => Ok(Self::FiniteVerb(FiniteVerbCell {
                tense: required_code("finite tense", tense, FiniteTense::from_code)?,
                person: required_code("person", person, Person::from_code)?,
                number: required_code("number", number, Number::from_code)?,
            })),
            ["imperative", person, number] => Ok(Self::Imperative(ImperativeCell {
                person: required_code("person", person, Person::from_code)?,
                number: required_code("number", number, Number::from_code)?,
            })),
            ["infinitive"] => Ok(Self::Infinitive),
            ["supine"] => Ok(Self::Supine),
            ["l-participle", gender, number] => Ok(Self::LParticiple(LParticipleCell {
                gender: required_code("gender", gender, Gender::from_code)?,
                number: required_code("number", number, Number::from_code)?,
            })),
            [
                "participle",
                tense,
                voice,
                case,
                number,
                gender,
                animacy,
                form,
                comparison,
            ] => Ok(Self::Participle(ParticipleCell {
                tense: required_code("participle tense", tense, ParticipleTense::from_code)?,
                voice: required_code("participle voice", voice, ParticipleVoice::from_code)?,
                agreement: parse_adjective_cell(case, number, gender, animacy, form, comparison)?,
            })),
            ["pronoun", case, number, gender, animacy] => {
                parse_pronoun_cell(case, number, gender, "none", animacy).map(Self::Pronoun)
            }
            ["pronoun", case, number, gender, person, animacy] => {
                parse_pronoun_cell(case, number, gender, person, animacy).map(Self::Pronoun)
            }
            ["numeral", kind, case, number, gender, animacy] => Ok(Self::Numeral(NumeralCell {
                kind: required_code("numeral kind", kind, NumeralKind::from_code)?,
                case: required_code("case", case, Case::from_code)?,
                number: required_code("number", number, Number::from_code)?,
                gender: optional_gender(gender)?,
                animacy: neutral_animacy(animacy)?,
            })),
            _ => Err(invalid_cell_key(value)),
        }
    }
}

fn adjective_key(prefix: &str, cell: AdjectiveCell) -> String {
    format!(
        "{prefix}:{}:{}:{}:{}:{}:{}",
        cell.case.code(),
        cell.number.code(),
        cell.gender.code(),
        cell.animacy.code(),
        cell.form.code(),
        cell.comparison.code()
    )
}

fn parse_adjective_cell(
    case: &str,
    number: &str,
    gender: &str,
    animacy: &str,
    form: &str,
    comparison: &str,
) -> crate::Result<AdjectiveCell> {
    Ok(AdjectiveCell {
        case: required_code("case", case, Case::from_code)?,
        number: required_code("number", number, Number::from_code)?,
        gender: required_code("gender", gender, Gender::from_code)?,
        animacy: neutral_animacy(animacy)?,
        form: required_code("adjective form", form, AdjectiveForm::from_code)?,
        comparison: required_code("comparison", comparison, Comparison::from_code)?,
    })
}

fn parse_pronoun_cell(
    case: &str,
    number: &str,
    gender: &str,
    person: &str,
    animacy: &str,
) -> crate::Result<PronounCell> {
    Ok(PronounCell {
        case: required_code("case", case, Case::from_code)?,
        number: required_code("number", number, Number::from_code)?,
        gender: optional_gender(gender)?,
        person: optional_person(person)?,
        animacy: neutral_animacy(animacy)?,
    })
}

fn required_code<T>(
    kind: &str,
    value: &str,
    parse: impl FnOnce(&str) -> Option<T>,
) -> crate::Result<T> {
    parse(value).ok_or_else(|| crate::Error::ContradictoryMetadata {
        reason: format!("unknown grammar-cell {kind} {value:?}"),
    })
}

fn optional_gender(value: &str) -> crate::Result<Option<Gender>> {
    if value == "any" {
        Ok(None)
    } else {
        required_code("gender", value, Gender::from_code).map(Some)
    }
}

fn optional_person(value: &str) -> crate::Result<Option<Person>> {
    if value == "none" {
        Ok(None)
    } else {
        required_code("person", value, Person::from_code).map(Some)
    }
}

fn neutral_animacy(value: &str) -> crate::Result<Animacy> {
    match value {
        // Registry `any` cells use the neutral typed representative outside
        // the accusative; lookup still considers the explicit wildcard key.
        "any" => Ok(Animacy::Inanimate),
        _ => required_code("animacy", value, Animacy::from_code),
    }
}

fn invalid_cell_key(value: &str) -> crate::Error {
    crate::Error::ContradictoryMetadata {
        reason: format!("unsupported grammar cell key {value:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_inventories_preserve_dimension_order_and_policy() {
        let nouns = NounCell::inventory(&Animacy::ALL);
        assert_eq!(nouns.len(), 42);
        assert_eq!(
            nouns.first(),
            Some(&NounCell {
                case: Case::Nominative,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            })
        );
        assert_eq!(
            nouns.last(),
            Some(&NounCell {
                case: Case::Vocative,
                number: Number::Plural,
                animacy: Animacy::Animate,
            })
        );

        let canonical_agreement =
            AdjectiveCell::inventory(&[AdjectiveForm::Long], &[Comparison::Positive], |case| {
                if case == Case::Accusative {
                    &Animacy::ALL
                } else {
                    &[Animacy::Inanimate]
                }
            });
        assert_eq!(canonical_agreement.len(), 72);
        assert!(
            canonical_agreement.iter().all(|cell| {
                cell.case == Case::Accusative || cell.animacy == Animacy::Inanimate
            })
        );

        let exhaustive_agreement =
            AdjectiveCell::inventory(&AdjectiveForm::ALL, &Comparison::ALL, |_| &Animacy::ALL);
        assert_eq!(exhaustive_agreement.len(), 756);
        assert_eq!(FiniteVerbCell::inventory(&FiniteTense::ALL).len(), 45);
        assert_eq!(
            ParticipleCell::inventory(
                &ParticipleTense::ALL,
                &ParticipleVoice::ALL,
                &exhaustive_agreement,
            )
            .len(),
            3_024,
        );

        let profiles = [(None, None), (Some(Gender::Neuter), Some(Person::Third))];
        assert_eq!(PronounCell::inventory(&profiles).len(), 84);
        let optional_genders = [
            None,
            Some(Gender::Masculine),
            Some(Gender::Feminine),
            Some(Gender::Neuter),
        ];
        assert_eq!(
            NumeralCell::inventory(&NumeralKind::ALL, &optional_genders).len(),
            504
        );
    }

    #[test]
    fn grammar_cell_keys_round_trip_through_one_typed_codec() {
        let cells = [
            GrammarCell::LexicalForm,
            GrammarCell::Indeclinable,
            GrammarCell::Noun(NounCell {
                case: Case::Accusative,
                number: Number::Plural,
                animacy: Animacy::Animate,
            }),
            GrammarCell::Adjective(AdjectiveCell {
                case: Case::Genitive,
                number: Number::Dual,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Comparative,
            }),
            GrammarCell::FiniteVerb(FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person: Person::First,
                number: Number::Singular,
            }),
            GrammarCell::Imperative(ImperativeCell {
                person: Person::Second,
                number: Number::Plural,
            }),
            GrammarCell::Infinitive,
            GrammarCell::Supine,
            GrammarCell::LParticiple(LParticipleCell {
                gender: Gender::Neuter,
                number: Number::Dual,
            }),
            GrammarCell::Participle(ParticipleCell {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Passive,
                agreement: AdjectiveCell {
                    case: Case::Instrumental,
                    number: Number::Plural,
                    gender: Gender::Masculine,
                    animacy: Animacy::Animate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                },
            }),
            GrammarCell::VerbalNoun(NounCell {
                case: Case::Locative,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            }),
            GrammarCell::Pronoun(PronounCell {
                case: Case::Dative,
                number: Number::Plural,
                gender: None,
                person: Some(Person::Third),
                animacy: Animacy::Inanimate,
            }),
            GrammarCell::Determiner(AdjectiveCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Short,
                comparison: Comparison::Positive,
            }),
            GrammarCell::Numeral(NumeralCell {
                kind: NumeralKind::Collective,
                case: Case::Vocative,
                number: Number::Plural,
                gender: None,
                animacy: Animacy::Animate,
            }),
        ];

        for cell in cells {
            let key = cell.key();
            assert_eq!(key.parse::<GrammarCell>(), Ok(cell), "{key}");
        }
    }

    #[test]
    fn grammar_cell_codec_rejects_unknown_closed_codes() {
        assert!(
            "noun:ablative:singular:inanimate"
                .parse::<GrammarCell>()
                .is_err()
        );
        assert!("present:fourth:singular".parse::<GrammarCell>().is_err());
        assert!(
            "participle:future:active:nominative:singular:masculine:inanimate:short:positive"
                .parse::<GrammarCell>()
                .is_err()
        );
    }

    #[test]
    fn grammar_cell_codec_accepts_registry_wildcards_as_neutral_representatives() {
        assert_eq!(
            "pronoun:nominative:singular:any:any"
                .parse::<GrammarCell>()
                .expect("legacy pronoun cell"),
            GrammarCell::Pronoun(PronounCell {
                case: Case::Nominative,
                number: Number::Singular,
                gender: None,
                person: None,
                animacy: Animacy::Inanimate,
            })
        );
        assert_eq!(
            "numeral:cardinal:accusative:plural:any:any"
                .parse::<GrammarCell>()
                .expect("wildcard numeral cell"),
            GrammarCell::Numeral(NumeralCell {
                kind: NumeralKind::Cardinal,
                case: Case::Accusative,
                number: Number::Plural,
                gender: None,
                animacy: Animacy::Inanimate,
            })
        );
        assert!(
            "pronoun:nominative:singular:none:any:inanimate"
                .parse::<GrammarCell>()
                .is_err(),
            "gender accepts `any`, never the person sentinel"
        );
        assert!(
            "pronoun:nominative:singular:any:any:inanimate"
                .parse::<GrammarCell>()
                .is_err(),
            "person accepts `none`, never the gender sentinel"
        );
    }
}
