//! Source-reviewed Old Church Slavonic personal and reflexive pronouns.

use crate::{Case, Gender, Number, Person, RuleId};

/// One closed personal-pronoun identity. First- and second-person identities
/// carry number but have intrinsic person; the reflexive is numberless; the
/// third-person anaphoric identity carries gender and is defective in the
/// nominative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PersonalPronounIdentity {
    First,
    Second,
    Reflexive,
    AnaphoricThird,
}

impl PersonalPronounIdentity {
    pub const ALL: [Self; 4] = [
        Self::First,
        Self::Second,
        Self::Reflexive,
        Self::AnaphoricThird,
    ];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::First => "азъ",
            Self::Second => "тꙑ",
            Self::Reflexive => "сѧ",
            // Polivanova uses fictional *и as the dictionary identity. The
            // engine omits the metalinguistic asterisk from the valid lemma.
            Self::AnaphoricThird => "и",
        }
    }

    pub const fn person(self) -> Option<Person> {
        match self {
            Self::First => Some(Person::First),
            Self::Second => Some(Person::Second),
            Self::Reflexive => None,
            Self::AnaphoricThird => Some(Person::Third),
        }
    }

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::First => RuleId::PronounPersonalFirst,
            Self::Second => RuleId::PronounPersonalSecond,
            Self::Reflexive => RuleId::PronounReflexive,
            Self::AnaphoricThird => RuleId::PronounAnaphoricThird,
        }
    }

    /// Dictionary-page spellings classified as forms of this grammatical
    /// identity rather than independent complete paradigms.
    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::First => &["азъ", "вѣ", "мꙑ", "наю"],
            Self::Second => &["тꙑ", "ва", "вꙑ", "ваю"],
            Self::Reflexive => &["сѧ"],
            Self::AnaphoricThird => &["и", "ѥ", "ѭ", "ими"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }
}

/// Select table-primary forms, explicitly marked clitic variants, or both in
/// grammar-table order. This does not guess the prosody of unmarked forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PronounFormSelection {
    All,
    TablePrimary,
    MarkedClitic,
}

impl PronounFormSelection {
    pub const fn code(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::TablePrimary => "table-primary",
            Self::MarkedClitic => "marked-clitic",
        }
    }
}

/// The conditioned allomorph of the third-person anaphoric pronoun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnaphoricEnvironment {
    Free,
    AfterPreposition,
}

impl AnaphoricEnvironment {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::AfterPreposition => "after-preposition",
        }
    }
}

/// Evidential and syntactic status of a reviewed pronoun form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PronounVariantStatus {
    TablePrimary,
    MarkedClitic,
    /// UT lists the form in its OCS table, while Polivanova finds no OCS
    /// attestation and compares the later Church Slavonic form.
    DisputedMarkedClitic,
    FreeAnaphoric,
    Adprepositional,
}

impl PronounVariantStatus {
    pub const fn is_marked_clitic(self) -> bool {
        matches!(self, Self::MarkedClitic | Self::DisputedMarkedClitic)
    }

    pub const fn is_disputed(self) -> bool {
        matches!(self, Self::DisputedMarkedClitic)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PronounVariant {
    pub text: &'static str,
    pub status: PronounVariantStatus,
}

impl PronounVariant {
    pub const fn new(text: &'static str, status: PronounVariantStatus) -> Self {
        Self { text, status }
    }
}

/// Return the complete first- or second-person cell, optionally restricted to
/// the source-identified table-primary or explicitly marked clitic realization.
pub fn personal_forms(
    identity: PersonalPronounIdentity,
    case: Case,
    number: Number,
    selection: PronounFormSelection,
) -> Vec<PronounVariant> {
    let forms = match identity {
        PersonalPronounIdentity::First => first_person_forms(case, number),
        PersonalPronounIdentity::Second => second_person_forms(case, number),
        PersonalPronounIdentity::Reflexive | PersonalPronounIdentity::AnaphoricThird => Vec::new(),
    };
    select(&forms, selection)
}

/// Return the numberless reflexive-pronoun cell. Nominative and vocative are
/// historically invalid and therefore return no forms.
pub fn reflexive_forms(case: Case, selection: PronounFormSelection) -> Vec<PronounVariant> {
    use PronounVariantStatus::{MarkedClitic, TablePrimary};
    let forms: &[PronounVariant] = match case {
        Case::Nominative | Case::Vocative => &[],
        Case::Accusative => &[PronounVariant::new("сѧ", TablePrimary)],
        Case::Genitive => &[PronounVariant::new("себе", TablePrimary)],
        Case::Locative => &[PronounVariant::new("себѣ", TablePrimary)],
        Case::Dative => &[
            PronounVariant::new("себѣ", TablePrimary),
            PronounVariant::new("си", MarkedClitic),
        ],
        Case::Instrumental => &[PronounVariant::new("собоѭ", TablePrimary)],
    };
    select(forms, selection)
}

/// Return one conditioned third-person anaphoric form. Every nominative and
/// vocative cell is historically invalid; demonstratives such as `тъ` and
/// `онъ`, not reconstructed `*и`, fill nominative syntax.
pub fn anaphoric_form(
    case: Case,
    number: Number,
    gender: Gender,
    environment: AnaphoricEnvironment,
) -> Option<PronounVariant> {
    use AnaphoricEnvironment::{AfterPreposition, Free};
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    use PronounVariantStatus::{Adprepositional, FreeAnaphoric};

    let (free, adprepositional) = match (case, number, gender) {
        (Accusative, Singular, Masculine) => ("и", "н҄ь"),
        (Accusative, Singular, Neuter) => ("ѥ", "н҄ѥ"),
        (Accusative, Singular, Feminine) => ("ѭ", "н҄ѭ"),
        (Genitive, Singular, Masculine | Neuter) => ("ѥго", "н҄ѥго"),
        (Genitive, Singular, Feminine) => ("ѥѩ", "н҄ѥѩ"),
        (Locative, Singular, Masculine | Neuter) => ("ѥмь", "н҄ѥмь"),
        (Locative, Singular, Feminine) => ("ѥи", "н҄ѥи"),
        (Dative, Singular, Masculine | Neuter) => ("ѥму", "н҄ѥму"),
        (Dative, Singular, Feminine) => ("ѥи", "н҄ѥи"),
        (Instrumental, Singular, Masculine | Neuter) => ("имь", "н҄имь"),
        (Instrumental, Singular, Feminine) => ("ѥѭ", "н҄ѥѭ"),

        (Accusative, Dual, Masculine) => ("ꙗ", "н҄ꙗ"),
        (Accusative, Dual, Neuter | Feminine) => ("и", "н҄и"),
        (Genitive | Locative, Dual, _) => ("ѥю", "н҄ѥю"),
        (Dative | Instrumental, Dual, _) => ("има", "н҄има"),

        (Accusative, Plural, Masculine | Feminine) => ("ѩ", "н҄ѩ"),
        (Accusative, Plural, Neuter) => ("ꙗ", "н҄ꙗ"),
        (Genitive | Locative, Plural, _) => ("ихъ", "н҄ихъ"),
        (Dative, Plural, _) => ("имъ", "н҄имъ"),
        (Instrumental, Plural, _) => ("ими", "н҄ими"),
        (Case::Nominative | Case::Vocative, _, _) => return None,
    };
    Some(match environment {
        Free => PronounVariant::new(free, FreeAnaphoric),
        AfterPreposition => PronounVariant::new(adprepositional, Adprepositional),
    })
}

fn select(forms: &[PronounVariant], selection: PronounFormSelection) -> Vec<PronounVariant> {
    forms
        .iter()
        .copied()
        .filter(|form| match selection {
            PronounFormSelection::All => true,
            PronounFormSelection::TablePrimary => !form.status.is_marked_clitic(),
            PronounFormSelection::MarkedClitic => form.status.is_marked_clitic(),
        })
        .collect()
}

fn first_person_forms(case: Case, number: Number) -> Vec<PronounVariant> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Number::{Dual, Plural, Singular};
    use PronounVariantStatus::{DisputedMarkedClitic, MarkedClitic, TablePrimary};

    let forms: &[PronounVariant] = match (case, number) {
        (Nominative, Singular) => &[PronounVariant::new("азъ", TablePrimary)],
        (Accusative, Singular) => &[PronounVariant::new("мѧ", TablePrimary)],
        (Genitive, Singular) => &[PronounVariant::new("мене", TablePrimary)],
        (Locative, Singular) => &[PronounVariant::new("мьнѣ", TablePrimary)],
        (Dative, Singular) => &[
            PronounVariant::new("мьнѣ", TablePrimary),
            PronounVariant::new("ми", MarkedClitic),
        ],
        (Instrumental, Singular) => &[PronounVariant::new("мъноѭ", TablePrimary)],

        (Nominative, Dual) => &[PronounVariant::new("вѣ", TablePrimary)],
        (Accusative, Dual) => &[
            PronounVariant::new("на", TablePrimary),
            PronounVariant::new("нꙑ", MarkedClitic),
        ],
        (Genitive | Locative, Dual) => &[PronounVariant::new("наю", TablePrimary)],
        (Dative, Dual) => &[
            PronounVariant::new("нама", TablePrimary),
            PronounVariant::new("на", DisputedMarkedClitic),
        ],
        (Instrumental, Dual) => &[PronounVariant::new("нама", TablePrimary)],

        (Nominative, Plural) => &[PronounVariant::new("мꙑ", TablePrimary)],
        (Accusative, Plural) => &[PronounVariant::new("нꙑ", TablePrimary)],
        (Genitive | Locative, Plural) => &[PronounVariant::new("насъ", TablePrimary)],
        (Dative, Plural) => &[
            PronounVariant::new("намъ", TablePrimary),
            PronounVariant::new("нꙑ", MarkedClitic),
        ],
        (Instrumental, Plural) => &[PronounVariant::new("нами", TablePrimary)],
        (Case::Vocative, _) => &[],
    };
    forms.to_vec()
}

fn second_person_forms(case: Case, number: Number) -> Vec<PronounVariant> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Number::{Dual, Plural, Singular};
    use PronounVariantStatus::{MarkedClitic, TablePrimary};

    let forms: &[PronounVariant] = match (case, number) {
        (Nominative, Singular) => &[PronounVariant::new("тꙑ", TablePrimary)],
        (Accusative, Singular) => &[PronounVariant::new("тѧ", TablePrimary)],
        (Genitive, Singular) => &[PronounVariant::new("тебе", TablePrimary)],
        (Locative, Singular) => &[PronounVariant::new("тебѣ", TablePrimary)],
        (Dative, Singular) => &[
            PronounVariant::new("тебѣ", TablePrimary),
            PronounVariant::new("ти", MarkedClitic),
        ],
        (Instrumental, Singular) => &[PronounVariant::new("тобоѭ", TablePrimary)],

        (Nominative | Accusative, Dual) => &[
            PronounVariant::new("ва", TablePrimary),
            PronounVariant::new("вꙑ", MarkedClitic),
        ],
        (Genitive | Locative, Dual) => &[PronounVariant::new("ваю", TablePrimary)],
        (Dative, Dual) => &[
            PronounVariant::new("вама", TablePrimary),
            PronounVariant::new("ва", MarkedClitic),
        ],
        (Instrumental, Dual) => &[PronounVariant::new("вама", TablePrimary)],

        (Nominative | Accusative, Plural) => &[PronounVariant::new("вꙑ", TablePrimary)],
        (Genitive | Locative, Plural) => &[PronounVariant::new("васъ", TablePrimary)],
        (Dative, Plural) => &[
            PronounVariant::new("вамъ", TablePrimary),
            PronounVariant::new("вꙑ", MarkedClitic),
        ],
        (Instrumental, Plural) => &[PronounVariant::new("вами", TablePrimary)],
        (Case::Vocative, _) => &[],
    };
    forms.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_and_second_person_goldens_cover_every_nonvocative_cell() {
        let expected = [
            (
                PersonalPronounIdentity::First,
                [
                    "азъ",
                    "мене",
                    "мьнѣ || ми",
                    "мѧ",
                    "мъноѭ",
                    "мьнѣ",
                    "вѣ",
                    "наю",
                    "нама || на",
                    "на || нꙑ",
                    "нама",
                    "наю",
                    "мꙑ",
                    "насъ",
                    "намъ || нꙑ",
                    "нꙑ",
                    "нами",
                    "насъ",
                ],
            ),
            (
                PersonalPronounIdentity::Second,
                [
                    "тꙑ",
                    "тебе",
                    "тебѣ || ти",
                    "тѧ",
                    "тобоѭ",
                    "тебѣ",
                    "ва || вꙑ",
                    "ваю",
                    "вама || ва",
                    "ва || вꙑ",
                    "вама",
                    "ваю",
                    "вꙑ",
                    "васъ",
                    "вамъ || вꙑ",
                    "вꙑ",
                    "вами",
                    "васъ",
                ],
            ),
        ];
        let cases = [
            Case::Nominative,
            Case::Genitive,
            Case::Dative,
            Case::Accusative,
            Case::Instrumental,
            Case::Locative,
        ];

        for (identity, expected_cells) in expected {
            let actual_cells = Number::ALL
                .into_iter()
                .flat_map(|number| {
                    cases.into_iter().map(move |case| {
                        personal_forms(identity, case, number, PronounFormSelection::All)
                            .iter()
                            .map(|form| form.text)
                            .collect::<Vec<_>>()
                            .join(" || ")
                    })
                })
                .collect::<Vec<_>>();
            assert_eq!(actual_cells, expected_cells, "{identity:?}");
            for number in Number::ALL {
                assert!(
                    personal_forms(identity, Case::Vocative, number, PronounFormSelection::All)
                        .is_empty()
                );
            }
        }
    }

    #[test]
    fn reflexive_is_numberless_defective_and_has_a_typed_clitic() {
        assert!(reflexive_forms(Case::Nominative, PronounFormSelection::All).is_empty());
        assert!(reflexive_forms(Case::Vocative, PronounFormSelection::All).is_empty());
        assert_eq!(
            reflexive_forms(Case::Dative, PronounFormSelection::All)
                .iter()
                .map(|form| form.text)
                .collect::<Vec<_>>(),
            ["себѣ", "си"]
        );
        assert_eq!(
            reflexive_forms(Case::Dative, PronounFormSelection::MarkedClitic)[0].status,
            PronounVariantStatus::MarkedClitic
        );
    }

    #[test]
    fn anaphoric_free_and_adprepositional_inventories_are_complete() {
        let goldens = [
            (
                AnaphoricEnvironment::Free,
                [
                    "ѥго", "ѥѩ", "ѥго", "ѥму", "ѥи", "ѥму", "и", "ѭ", "ѥ", "имь", "ѥѭ", "имь",
                    "ѥмь", "ѥи", "ѥмь", "ѥю", "ѥю", "ѥю", "има", "има", "има", "ꙗ", "и", "и",
                    "има", "има", "има", "ѥю", "ѥю", "ѥю", "ихъ", "ихъ", "ихъ", "имъ", "имъ",
                    "имъ", "ѩ", "ѩ", "ꙗ", "ими", "ими", "ими", "ихъ", "ихъ", "ихъ",
                ],
            ),
            (
                AnaphoricEnvironment::AfterPreposition,
                [
                    "н҄ѥго",
                    "н҄ѥѩ",
                    "н҄ѥго",
                    "н҄ѥму",
                    "н҄ѥи",
                    "н҄ѥму",
                    "н҄ь",
                    "н҄ѭ",
                    "н҄ѥ",
                    "н҄имь",
                    "н҄ѥѭ",
                    "н҄имь",
                    "н҄ѥмь",
                    "н҄ѥи",
                    "н҄ѥмь",
                    "н҄ѥю",
                    "н҄ѥю",
                    "н҄ѥю",
                    "н҄има",
                    "н҄има",
                    "н҄има",
                    "н҄ꙗ",
                    "н҄и",
                    "н҄и",
                    "н҄има",
                    "н҄има",
                    "н҄има",
                    "н҄ѥю",
                    "н҄ѥю",
                    "н҄ѥю",
                    "н҄ихъ",
                    "н҄ихъ",
                    "н҄ихъ",
                    "н҄имъ",
                    "н҄имъ",
                    "н҄имъ",
                    "н҄ѩ",
                    "н҄ѩ",
                    "н҄ꙗ",
                    "н҄ими",
                    "н҄ими",
                    "н҄ими",
                    "н҄ихъ",
                    "н҄ихъ",
                    "н҄ихъ",
                ],
            ),
        ];

        for (environment, expected) in goldens {
            let mut valid = 0;
            let mut actual = Vec::new();
            for number in Number::ALL {
                for case in Case::ALL {
                    for gender in Gender::ALL {
                        let form = anaphoric_form(case, number, gender, environment);
                        if matches!(case, Case::Nominative | Case::Vocative) {
                            assert!(form.is_none());
                        } else {
                            assert!(form.is_some(), "{case:?} {number:?} {gender:?}");
                            valid += 1;
                            actual.push(form.expect("valid anaphoric cell").text);
                        }
                    }
                }
            }
            assert_eq!(valid, 45);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn source_union_aliases_are_exhaustive_and_nonoverlapping() {
        let mut aliases = Vec::new();
        for identity in PersonalPronounIdentity::ALL {
            for alias in identity.source_union_aliases() {
                assert_eq!(
                    PersonalPronounIdentity::classify_source_union_lemma(alias),
                    Some(identity)
                );
                aliases.push(*alias);
            }
        }
        aliases.sort_unstable();
        aliases.dedup();
        assert_eq!(aliases.len(), 13);
    }

    #[test]
    fn the_first_dual_dative_clitic_keeps_its_disputed_status() {
        let forms = personal_forms(
            PersonalPronounIdentity::First,
            Case::Dative,
            Number::Dual,
            PronounFormSelection::All,
        );
        assert_eq!(forms[1].text, "на");
        assert_eq!(forms[1].status, PronounVariantStatus::DisputedMarkedClitic);
    }
}
