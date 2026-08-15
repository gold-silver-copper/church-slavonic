//! Source-reviewed Synodal Church Slavonic determiner morphology.

use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FormSet, Gender, Number,
    OrthographyProfile, Result, SynodalWord,
    morphology::{long_adjective_ending, normative_variants},
};

/// Productive determiner paradigms described by Alypy §§45, 48, and 57.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum DeterminerDeclension {
    /// The short pronominal and full adjectival paradigms of `самъ`/`самый`.
    PronominalHard,
    /// The mixed hard/soft, dual-less paradigm of `весь`.
    VesMixed,
    /// The short and full velar, dual-less paradigms of `всѧкъ`/`всѧкїй`.
    VsyakMixed,
    /// Full `-скїй` determiners with `-ск-`/`-ст-` alternation before `ѣ`.
    FullSk,
}

impl DeterminerDeclension {
    pub const ALL: [Self; 4] = [
        Self::PronominalHard,
        Self::VesMixed,
        Self::VsyakMixed,
        Self::FullSk,
    ];
}

/// Lexically licensed grammatical numbers for an agreeing determiner.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum DeterminerNumberInventory {
    #[default]
    All,
    SingularOnly,
    DualOnly,
    PluralOnly,
    SingularAndDual,
    SingularAndPlural,
    DualAndPlural,
}

impl DeterminerNumberInventory {
    #[must_use]
    pub const fn contains(self, number: Number) -> bool {
        matches!(
            (self, number),
            (Self::All, _)
                | (Self::SingularOnly, Number::Singular)
                | (Self::DualOnly, Number::Dual)
                | (Self::PluralOnly, Number::Plural)
                | (Self::SingularAndDual, Number::Singular | Number::Dual)
                | (Self::SingularAndPlural, Number::Singular | Number::Plural)
                | (Self::DualAndPlural, Number::Dual | Number::Plural)
        )
    }
}

/// Complete typed metadata for one productive Synodal determiner.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DeterminerLexeme {
    pub lemma: SynodalWord,
    pub stem: SynodalWord,
    pub declension: DeterminerDeclension,
    pub number_inventory: DeterminerNumberInventory,
}

impl DeterminerLexeme {
    #[must_use]
    pub const fn new(
        lemma: SynodalWord,
        stem: SynodalWord,
        declension: DeterminerDeclension,
    ) -> Self {
        let number_inventory = match declension {
            DeterminerDeclension::VesMixed | DeterminerDeclension::VsyakMixed => {
                DeterminerNumberInventory::SingularAndPlural
            }
            DeterminerDeclension::PronominalHard | DeterminerDeclension::FullSk => {
                DeterminerNumberInventory::All
            }
        };
        Self {
            lemma,
            stem,
            declension,
            number_inventory,
        }
    }

    #[must_use]
    pub const fn with_number_inventory(mut self, inventory: DeterminerNumberInventory) -> Self {
        self.number_inventory = inventory;
        self
    }
}

/// Validates class-specific lexical metadata without generating a form.
pub fn validate_determiner_lexeme(lexeme: &DeterminerLexeme) -> Result<()> {
    let stem = lexeme.stem.canonical();
    match lexeme.declension {
        DeterminerDeclension::PronominalHard if stem.is_empty() => {
            return contradictory("a pronominal-hard determiner requires a nonempty stem");
        }
        DeterminerDeclension::VesMixed if stem != "вс" => {
            return contradictory("the весь mixed paradigm requires the reviewed stem вс");
        }
        DeterminerDeclension::VsyakMixed if !stem.ends_with('к') => {
            return contradictory("the всѧкъ mixed paradigm requires a stem ending in к");
        }
        DeterminerDeclension::FullSk if !stem.ends_with("ск") => {
            return contradictory("a full -скїй determiner requires a stem ending in -ск");
        }
        DeterminerDeclension::PronominalHard
        | DeterminerDeclension::VesMixed
        | DeterminerDeclension::VsyakMixed
        | DeterminerDeclension::FullSk => {}
    }
    if matches!(
        lexeme.declension,
        DeterminerDeclension::VesMixed | DeterminerDeclension::VsyakMixed
    ) && lexeme.number_inventory.contains(Number::Dual)
    {
        return contradictory("Alypy §48 explicitly excludes the dual from весь and всѧкъ");
    }
    Ok(())
}

/// Generates one source-licensed Synodal determiner agreement cell.
pub fn decline_determiner(
    lexeme: &DeterminerLexeme,
    cell: AdjectiveCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    validate_determiner_lexeme(lexeme)?;
    if cell.comparison != Comparison::Positive {
        return Err(Error::HistoricallyInvalidCell {
            reason: "determiners do not take comparative or superlative agreement".into(),
        });
    }
    if !lexeme.number_inventory.contains(cell.number) {
        return Err(Error::HistoricallyInvalidCell {
            reason: "the requested grammatical number is absent from this determiner paradigm"
                .into(),
        });
    }
    let (forms, rule) = match lexeme.declension {
        DeterminerDeclension::PronominalHard => (
            match cell.form {
                AdjectiveForm::Short => short_hard_forms(lexeme.stem.canonical(), cell),
                AdjectiveForm::Long => full_hard_forms(lexeme.stem.canonical(), cell)?,
            },
            "SYN-DETERMINER-HARD-ALYPY-45-48",
        ),
        DeterminerDeclension::VesMixed => {
            if cell.form != AdjectiveForm::Short {
                return Err(Error::HistoricallyInvalidCell {
                    reason: "весь has only the short pronominal paradigm".into(),
                });
            }
            (ves_forms(cell), "SYN-DETERMINER-VES-MIXED-ALYPY-45-48")
        }
        DeterminerDeclension::VsyakMixed => (
            match cell.form {
                AdjectiveForm::Short => vsyak_short_forms(lexeme.stem.canonical(), cell),
                AdjectiveForm::Long => vsyak_long_forms(lexeme.stem.canonical(), cell),
            },
            "SYN-DETERMINER-VSYAK-MIXED-ALYPY-45-48-57",
        ),
        DeterminerDeclension::FullSk => {
            if cell.form != AdjectiveForm::Long {
                return Err(Error::HistoricallyInvalidCell {
                    reason: "the -скїй determiner class has only full adjectival forms".into(),
                });
            }
            (
                full_sk_forms(lexeme.stem.canonical(), cell)?,
                "SYN-DETERMINER-FULL-SK-ALYPY-45-57",
            )
        }
    };
    normative_variants(
        forms,
        rule,
        profile,
        "determiner-declension",
        lexeme.lemma.canonical(),
    )
}

fn contradictory<T>(reason: &str) -> Result<T> {
    Err(Error::ContradictoryMetadata {
        reason: reason.into(),
    })
}

fn join(stem: &str, ending: &str) -> String {
    let mut result = String::with_capacity(stem.len() + ending.len());
    result.push_str(stem);
    result.push_str(ending);
    result
}

fn nominative_cell(cell: AdjectiveCell) -> AdjectiveCell {
    AdjectiveCell {
        case: Case::Nominative,
        animacy: Animacy::Inanimate,
        ..cell
    }
}

fn vocative_as_nominative(cell: AdjectiveCell) -> AdjectiveCell {
    if cell.case == Case::Vocative {
        nominative_cell(cell)
    } else {
        cell
    }
}

fn short_hard_forms(stem: &str, cell: AdjectiveCell) -> Vec<String> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let cell = vocative_as_nominative(cell);
    let endings: &[&str] = match (cell.number, cell.gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => &["ъ"],
        (Sg, F, Nom, _) => &["а"],
        (Sg, N, Nom, _) => &["о"],
        (Sg, M | N, Gen, _) => &["огѡ", "ого"],
        (Sg, F, Gen, _) => &["оѧ"],
        (Sg, M | N, Dat, _) => &["омꙋ"],
        (Sg, F, Dat | Loc, _) => &["ой", "ѣй"],
        (Sg, M, Acc, Animacy::Inanimate) => &["ъ"],
        (Sg, M, Acc, Animacy::Animate) => &["ого", "огѡ"],
        (Sg, F, Acc, _) => &["ꙋ"],
        (Sg, N, Acc, _) => &["о"],
        (Sg, M | N, Ins, _) => &["ѣмъ"],
        (Sg, F, Ins, _) => &["ою"],
        (Sg, M | N, Loc, _) => &["омъ", "ѣмъ"],
        (Du, M, Nom | Acc, _) => &["а"],
        (Du, F | N, Nom | Acc, _) => &["ѣ"],
        (Du, _, Gen | Loc, _) => &["ѡю"],
        (Du, _, Dat | Ins, _) => &["ѣма"],
        (Pl, M, Nom, _) => &["и"],
        (Pl, F, Nom, _) => &["и"],
        (Pl, N, Nom, _) => &["а"],
        (Pl, _, Gen | Loc, _) => &["ѣхъ"],
        (Pl, _, Dat, _) => &["ѣмъ"],
        (Pl, M | F, Acc, Animacy::Inanimate) => &["и"],
        (Pl, M | F, Acc, Animacy::Animate) => &["ѣхъ"],
        (Pl, N, Acc, _) => &["а"],
        (Pl, _, Ins, _) => &["ѣми"],
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    endings.iter().map(|ending| join(stem, ending)).collect()
}

fn full_hard_forms(stem: &str, cell: AdjectiveCell) -> Result<Vec<String>> {
    let cell = vocative_as_nominative(cell);
    let ending = long_adjective_ending(crate::AdjectiveClass::Hard, cell)?;
    Ok(vec![join(stem, ending)])
}

fn ves_forms(cell: AdjectiveCell) -> Vec<String> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Plural as Pl, Singular as Sg};
    let cell = vocative_as_nominative(cell);
    let forms: &[&str] = match (cell.number, cell.gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => &["весь"],
        (Sg, F, Nom, _) => &["всѧ"],
        (Sg, N, Nom, _) => &["все"],
        (Sg, M | N, Gen, _) => &["всего", "всегѡ"],
        (Sg, F, Gen, _) => &["всеѧ"],
        (Sg, M | N, Dat, _) => &["всемꙋ"],
        (Sg, F, Dat | Loc, _) => &["всей"],
        (Sg, M, Acc, Animacy::Inanimate) => &["весь"],
        (Sg, M, Acc, Animacy::Animate) => &["всего", "всегѡ"],
        (Sg, F, Acc, _) => &["всю"],
        (Sg, N, Acc, _) => &["все"],
        (Sg, M | N, Ins, _) => &["всѣмъ"],
        (Sg, F, Ins, _) => &["всею"],
        (Sg, M | N, Loc, _) => &["всемъ"],
        (Pl, M, Nom, _) => &["вси"],
        (Pl, F | N, Nom, _) => &["всѧ"],
        (Pl, _, Gen | Loc, _) => &["всехъ"],
        (Pl, _, Dat, _) => &["всѣмъ"],
        (Pl, M | F, Acc, Animacy::Animate) => &["всехъ"],
        (Pl, M | F, Acc, Animacy::Inanimate) | (Pl, N, Acc, _) => &["всѧ"],
        (Pl, _, Ins, _) => &["всѣми"],
        (Number::Dual, _, _, _) | (_, _, Case::Vocative, _) => unreachable!(),
    };
    forms.iter().map(|form| (*form).into()).collect()
}

fn vsyak_short_forms(stem: &str, cell: AdjectiveCell) -> Vec<String> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Plural as Pl, Singular as Sg};
    let cell = vocative_as_nominative(cell);
    let palatalized = stem.strip_suffix('к').map_or_else(
        || stem.to_owned(),
        |base| {
            let mut value = base.to_owned();
            value.push('ц');
            value
        },
    );
    let raw = |ending| join(stem, ending);
    let soft = |ending| join(&palatalized, ending);
    match (cell.number, cell.gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => vec![raw("ъ")],
        (Sg, F, Nom, _) => vec![raw("а")],
        (Sg, N, Nom, _) => vec![raw("о")],
        (Sg, M | N, Gen, _) => vec![raw("агѡ"), raw("аго")],
        (Sg, F, Gen, _) => vec![raw("оѧ")],
        (Sg, M | N, Dat, _) => vec![raw("омꙋ")],
        (Sg, F, Dat | Loc, _) => vec![soft("ѣй"), raw("ой")],
        (Sg, M, Acc, Animacy::Inanimate) => vec![raw("ъ")],
        (Sg, M, Acc, Animacy::Animate) => vec![raw("аго"), raw("агѡ")],
        (Sg, F, Acc, _) => vec![raw("ꙋ")],
        (Sg, N, Acc, _) => vec![raw("о")],
        (Sg, M | N, Ins, _) => vec![soft("ѣмъ")],
        (Sg, F, Ins, _) => vec![raw("ою")],
        (Sg, M | N, Loc, _) => vec![raw("омъ"), soft("ѣмъ")],
        (Pl, M, Nom, _) => vec![soft("ы")],
        (Pl, F, Nom, _) => vec![raw("и")],
        (Pl, N, Nom, _) => vec![raw("а")],
        (Pl, _, Gen | Loc, _) => vec![soft("ѣхъ")],
        (Pl, _, Dat, _) => vec![soft("ѣмъ")],
        (Pl, M | F, Acc, Animacy::Animate) => vec![soft("ѣхъ")],
        (Pl, M | F, Acc, Animacy::Inanimate) => vec![raw("и")],
        (Pl, N, Acc, _) => vec![raw("а")],
        (Pl, _, Ins, _) => vec![soft("ѣми")],
        (Number::Dual, _, _, _) | (_, _, Case::Vocative, _) => unreachable!(),
    }
}

fn vsyak_long_forms(stem: &str, cell: AdjectiveCell) -> Vec<String> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Plural as Pl, Singular as Sg};
    let cell = vocative_as_nominative(cell);
    let palatalized = stem.strip_suffix('к').map_or_else(
        || stem.to_owned(),
        |base| {
            let mut value = base.to_owned();
            value.push('ц');
            value
        },
    );
    let raw = |ending| join(stem, ending);
    let soft = |ending| join(&palatalized, ending);
    match (cell.number, cell.gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => vec![raw("їй")],
        (Sg, F, Nom, _) => vec![raw("аѧ")],
        (Sg, N, Nom, _) => vec![raw("ое")],
        (Sg, M | N, Gen, _) => vec![raw("агѡ")],
        (Sg, F, Gen, _) => vec![raw("їѧ")],
        (Sg, M | N, Dat, _) => vec![raw("омꙋ")],
        (Sg, F, Dat | Loc, _) => vec![soft("ѣй"), raw("ой")],
        (Sg, M, Acc, Animacy::Inanimate) => vec![raw("їй")],
        (Sg, M, Acc, Animacy::Animate) => vec![raw("аго")],
        (Sg, F, Acc, _) => vec![raw("ꙋю")],
        (Sg, N, Acc, _) => vec![raw("ое")],
        (Sg, M | N, Ins, _) => vec![raw("имъ")],
        (Sg, F, Ins, _) => vec![raw("ою")],
        (Sg, M | N, Loc, _) => vec![soft("ѣмъ"), raw("омъ")],
        (Pl, M, Nom, _) => vec![soft("ыи")],
        (Pl, F, Nom, _) => vec![raw("їѧ")],
        (Pl, N, Nom, _) => vec![raw("аѧ")],
        (Pl, _, Gen | Loc, _) => vec![raw("ихъ")],
        (Pl, _, Dat, _) => vec![raw("имъ")],
        (Pl, M | F, Acc, Animacy::Animate) => vec![raw("ихъ")],
        (Pl, M | F, Acc, Animacy::Inanimate) => vec![raw("їѧ")],
        (Pl, N, Acc, _) => vec![raw("аѧ")],
        (Pl, _, Ins, _) => vec![raw("ими")],
        (Number::Dual, _, _, _) | (_, _, Case::Vocative, _) => unreachable!(),
    }
}

fn full_sk_forms(stem: &str, cell: AdjectiveCell) -> Result<Vec<String>> {
    let cell = vocative_as_nominative(cell);
    let mut ending = long_adjective_ending(crate::AdjectiveClass::Hard, cell)?;
    if cell.number == Number::Singular
        && cell.gender == Gender::Masculine
        && cell.case == Case::Nominative
    {
        ending = "їй";
    }
    let conditioned_stem = if ending.starts_with('ѣ') {
        let Some(base) = stem.strip_suffix("ск") else {
            return contradictory("a full -скїй determiner requires a stem ending in -ск");
        };
        join(base, "ст")
    } else {
        stem.to_owned()
    };
    Ok(vec![join(&conditioned_stem, ending)])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(value: &str) -> SynodalWord {
        SynodalWord::parse(value).expect("test word")
    }

    fn cell(
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
        form: AdjectiveForm,
    ) -> AdjectiveCell {
        AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
            comparison: Comparison::Positive,
        }
    }

    #[test]
    fn mixed_determiners_cover_source_diagnostic_cells() {
        let ves = DeterminerLexeme::new(word("весь"), word("вс"), DeterminerDeclension::VesMixed);
        let vsyak = DeterminerLexeme::new(
            word("всѧкъ"),
            word("всѧк"),
            DeterminerDeclension::VsyakMixed,
        );
        let examples = [
            (
                &ves,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    AdjectiveForm::Short,
                ),
                vec!["всего", "всегѡ"],
            ),
            (
                &ves,
                cell(
                    Case::Instrumental,
                    Number::Plural,
                    Gender::Neuter,
                    Animacy::Inanimate,
                    AdjectiveForm::Short,
                ),
                vec!["всѣми"],
            ),
            (
                &vsyak,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    AdjectiveForm::Short,
                ),
                vec!["всѧкагѡ", "всѧкаго"],
            ),
            (
                &vsyak,
                cell(
                    Case::Dative,
                    Number::Singular,
                    Gender::Feminine,
                    Animacy::Inanimate,
                    AdjectiveForm::Short,
                ),
                vec!["всѧцѣй", "всѧкой"],
            ),
            (
                &vsyak,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    Gender::Feminine,
                    Animacy::Inanimate,
                    AdjectiveForm::Short,
                ),
                vec!["всѧкоѧ"],
            ),
            (
                &vsyak,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    Gender::Feminine,
                    Animacy::Inanimate,
                    AdjectiveForm::Long,
                ),
                vec!["всѧкїѧ"],
            ),
        ];
        for (lexeme, cell, expected) in examples {
            let forms = decline_determiner(lexeme, cell, OrthographyProfile::Expanded)
                .expect("licensed cell");
            assert_eq!(forms.texts().collect::<Vec<_>>(), expected);
        }
    }

    #[test]
    fn sam_and_full_sk_have_distinct_short_and_long_behavior() {
        let sam = DeterminerLexeme::new(
            word("самъ"),
            word("сам"),
            DeterminerDeclension::PronominalHard,
        );
        let vsyacheskii = DeterminerLexeme::new(
            word("всѧческїй"),
            word("всѧческ"),
            DeterminerDeclension::FullSk,
        );
        assert_eq!(
            decline_determiner(
                &sam,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    AdjectiveForm::Short
                ),
                OrthographyProfile::Expanded,
            )
            .expect("short sam")
            .texts()
            .collect::<Vec<_>>(),
            vec!["самогѡ", "самого"]
        );
        assert_eq!(
            decline_determiner(
                &sam,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    AdjectiveForm::Long
                ),
                OrthographyProfile::Expanded,
            )
            .expect("long sam")
            .primary_text(),
            "самагѡ"
        );
        assert_eq!(
            decline_determiner(
                &vsyacheskii,
                cell(
                    Case::Locative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                    AdjectiveForm::Long
                ),
                OrthographyProfile::Expanded,
            )
            .expect("conditioned -ск- form")
            .primary_text(),
            "всѧчестѣмъ"
        );
    }

    #[test]
    fn invalid_forms_and_numbers_fail_typed() {
        let ves = DeterminerLexeme::new(word("весь"), word("вс"), DeterminerDeclension::VesMixed);
        for invalid in [
            cell(
                Case::Nominative,
                Number::Dual,
                Gender::Masculine,
                Animacy::Inanimate,
                AdjectiveForm::Short,
            ),
            cell(
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Inanimate,
                AdjectiveForm::Long,
            ),
        ] {
            assert!(matches!(
                decline_determiner(&ves, invalid, OrthographyProfile::Expanded),
                Err(Error::HistoricallyInvalidCell { .. })
            ));
        }
    }

    #[test]
    fn exhaustive_inventory_is_total_over_licensed_cells() {
        let lexemes = [
            DeterminerLexeme::new(
                word("самъ"),
                word("сам"),
                DeterminerDeclension::PronominalHard,
            ),
            DeterminerLexeme::new(word("весь"), word("вс"), DeterminerDeclension::VesMixed),
            DeterminerLexeme::new(
                word("всѧкъ"),
                word("всѧк"),
                DeterminerDeclension::VsyakMixed,
            ),
            DeterminerLexeme::new(
                word("всѧческїй"),
                word("всѧческ"),
                DeterminerDeclension::FullSk,
            ),
        ];
        for lexeme in lexemes {
            for cell in
                AdjectiveCell::inventory(&AdjectiveForm::ALL, &[Comparison::Positive], |_| {
                    &Animacy::ALL
                })
            {
                let expected_valid = lexeme.number_inventory.contains(cell.number)
                    && match lexeme.declension {
                        DeterminerDeclension::VesMixed => cell.form == AdjectiveForm::Short,
                        DeterminerDeclension::FullSk => cell.form == AdjectiveForm::Long,
                        DeterminerDeclension::PronominalHard | DeterminerDeclension::VsyakMixed => {
                            true
                        }
                    };
                let result = decline_determiner(&lexeme, cell, OrthographyProfile::Expanded);
                assert_eq!(
                    result.is_ok(),
                    expected_valid,
                    "{:?} {cell:?}",
                    lexeme.declension
                );
            }
        }
    }
}
