use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, Case, Comparison, Error, FormSet, Gender, MetadataField,
    Number, OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice, Result,
    SynodalWord,
};

use super::*;

/// Independently reviewed stems for one tense/voice participial system.
///
/// Synodal participles cannot be reconstructed from one generic verb stem. In
/// particular, full past-passive forms normally use a doubled `нн` stem while
/// the corresponding short forms use `н`. Keeping both stems explicit prevents
/// the inflector from guessing this lexical choice.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ParticiplePrincipalPart {
    pub short_stem: Option<SynodalWord>,
    /// Typed formation of the short active citation edges. Passive short
    /// participles use ordinary short-adjective endings and leave this empty.
    pub short_formation: Option<ActiveParticipleShortFormation>,
    pub long_stem: Option<SynodalWord>,
    pub class: AdjectiveClass,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ActiveParticipleShortFormation {
    PresentFirstUnpalatalized,
    PresentFirstPalatalized,
    PresentSecond,
    PresentAfterSibilant,
    PastConsonant,
    PastVowel,
    PastIotated,
}

/// Declines a participle only when its tense/voice-specific short and/or full
/// stem has been independently recorded for the lexeme.
pub fn decline_participle(
    lexeme: &VerbLexeme,
    cell: ParticipleCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if cell.agreement.comparison != Comparison::Positive {
        return Err(Error::HistoricallyInvalidCell {
            reason: "participles do not take comparative or superlative agreement".into(),
        });
    }
    if cell.tense == ParticipleTense::Present {
        match lexeme.aspect {
            Aspect::Unknown => {
                return Err(Error::MissingMetadata {
                    field: MetadataField::Aspect,
                });
            }
            Aspect::Perfective => {
                return Err(Error::HistoricallyInvalidCell {
                    reason: "productive present participles require a non-perfective aspect".into(),
                });
            }
            Aspect::Imperfective | Aspect::Biaspectual => {}
        }
    }
    let (principal_part, rule) = match (cell.tense, cell.voice) {
        (ParticipleTense::Present, ParticipleVoice::Active) => (
            lexeme.present_active_participle.as_ref(),
            "SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-ALYPY-95",
        ),
        (ParticipleTense::Past, ParticipleVoice::Active) => (
            lexeme.past_active_participle.as_ref(),
            "SYN-VERB-PARTICIPLE-PAST-ACTIVE-ALYPY-96",
        ),
        (ParticipleTense::Present, ParticipleVoice::Passive) => (
            lexeme.present_passive_participle.as_ref(),
            "SYN-VERB-PARTICIPLE-PRESENT-PASSIVE-ALYPY-99",
        ),
        (ParticipleTense::Past, ParticipleVoice::Passive) => (
            lexeme.past_passive_participle.as_ref(),
            "SYN-VERB-PARTICIPLE-PAST-PASSIVE-ALYPY-100",
        ),
    };
    let principal_part = principal_part.ok_or(Error::MissingPrincipalPart {
        field: MetadataField::ParticipleStem,
    })?;
    if cell.voice == ParticipleVoice::Active && cell.agreement.form == AdjectiveForm::Short {
        let stem = principal_part
            .short_stem
            .as_ref()
            .ok_or(Error::MissingPrincipalPart {
                field: MetadataField::ParticipleStem,
            })?;
        let formation = principal_part
            .short_formation
            .ok_or(Error::MissingMetadata {
                field: MetadataField::ParticipleFormation,
            })?;
        return decline_short_active_participle(lexeme, stem, formation, cell, profile);
    }
    let stem = match cell.agreement.form {
        AdjectiveForm::Short => principal_part.short_stem.as_ref(),
        AdjectiveForm::Long => principal_part.long_stem.as_ref(),
    }
    .ok_or(Error::MissingPrincipalPart {
        field: MetadataField::ParticipleStem,
    })?;
    let class = if principal_part.class == AdjectiveClass::Hard
        && stem
            .canonical()
            .chars()
            .last()
            .is_some_and(|last| matches!(last, 'ш' | 'щ' | 'ж' | 'ч'))
    {
        AdjectiveClass::HardSibilant
    } else {
        principal_part.class
    };
    let declined = decline_adjectival_stem(
        stem,
        class,
        cell.agreement,
        rule,
        "participle-declension",
        lexeme.lemma.canonical(),
        profile,
    )?;
    // Alypy §95: the present active long participle contracts its masculine
    // nominative singular: unpalatalized first-conjugation stems in -ꙋщ- give
    // -ый (сы́й, несы́й, грѧды́й), while -ѧщ-/-ющ- stems give -ѧй (боѧ́й,
    // хранѧ́й, вѣ́рꙋѧй). The uncontracted adjectival print stays available as a
    // later variant because both appear in the target recension.
    if cell.tense == ParticipleTense::Present
        && cell.voice == ParticipleVoice::Active
        && cell.agreement.form == AdjectiveForm::Long
        && cell.agreement.case == Case::Nominative
        && cell.agreement.number == Number::Singular
        && cell.agreement.gender == Gender::Masculine
    {
        let canonical = stem.canonical();
        let contracted = canonical
            .strip_suffix("ѧщ")
            .or_else(|| canonical.strip_suffix("ющ"))
            .map(|base| format!("{base}ѧй"))
            .or_else(|| {
                canonical
                    .strip_suffix("ꙋщ")
                    .or_else(|| canonical.strip_suffix("ущ"))
                    .map(|base| format!("{base}ый"))
            });
        if let Some(contracted) = contracted {
            let contracted_set = normative_variants(
                vec![contracted],
                rule,
                profile,
                "participle-declension",
                lexeme.lemma.canonical(),
            )?;
            let mut variants = contracted_set.variants().to_vec();
            variants.extend(declined.variants().iter().cloned());
            return FormSet::try_from_variants(variants);
        }
    }
    Ok(declined)
}

pub(crate) fn decline_short_comparison(
    lexeme: &AdjectiveLexeme,
    stem: &SynodalWord,
    formation: ComparisonFormation,
    cell: AdjectiveCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let citation = comparison_citation_variants(stem.canonical(), formation, cell.gender)?;
    let expanded = decline_short_comparison_stem(stem.canonical(), cell, citation)?;
    normative_variants(
        expanded,
        "SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-60",
        profile,
        "short-comparison-declension",
        lexeme.lemma.canonical(),
    )
}

pub(crate) fn decline_short_superlative_predicate(
    lexeme: &AdjectiveLexeme,
    stem: &SynodalWord,
    formation: ComparisonFormation,
    cell: AdjectiveCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if cell.case != Case::Nominative {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §59 licenses the exceptional short superlative only as the nominal part of a compound predicate, and §125 assigns that predicate complement to the nominative"
                .into(),
        });
    }

    let citation = comparison_citation_variants(stem.canonical(), formation, cell.gender)?;
    let mut expanded = decline_short_comparison_stem(stem.canonical(), cell, citation)?;
    if cell.number == Number::Singular && cell.gender == Gender::Masculine {
        // Alypy §59 directly attests и҆́стиннѣйшъ, with the comparison suffix
        // retained before the short masculine ending. Keep that source-defined
        // predicate pattern first, followed by the ordinary short-comparison
        // citation form that §59 also permits to carry superlative semantics.
        expanded.insert(0, join(stem.canonical(), "ъ"));
        expanded.dedup();
    }
    normative_variants(
        expanded,
        "SYN-ADJ-SUPERLATIVE-SHORT-PREDICATE-ALYPY-59-60-125-128",
        profile,
        "short-superlative-predicate",
        lexeme.lemma.canonical(),
    )
}

pub(crate) fn decline_short_active_participle(
    lexeme: &VerbLexeme,
    stem: &SynodalWord,
    formation: ActiveParticipleShortFormation,
    cell: ParticipleCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let citation =
        active_participle_citation_variants(stem.canonical(), formation, cell.agreement.gender)?;
    let expanded =
        decline_short_active_participle_stem(stem.canonical(), cell.agreement, citation)?;
    let rule = match cell.tense {
        ParticipleTense::Present => "SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-SHORT-ALYPY-95-98",
        ParticipleTense::Past => "SYN-VERB-PARTICIPLE-PAST-ACTIVE-SHORT-ALYPY-96-98",
    };
    normative_variants(
        expanded,
        rule,
        profile,
        "short-active-participle-declension",
        lexeme.lemma.canonical(),
    )
}

pub(crate) fn decline_short_comparison_stem(
    stem: &str,
    cell: AdjectiveCell,
    citation: Option<Vec<String>>,
) -> Result<Vec<String>> {
    if cell.number == Number::Singular
        && (matches!(cell.case, Case::Nominative | Case::Vocative)
            || (cell.gender == Gender::Neuter && cell.case == Case::Accusative))
    {
        return citation.ok_or_else(|| Error::ContradictoryMetadata {
            reason: "short comparison citation variants are missing".into(),
        });
    }
    let primary = join(stem, short_comparison_ending(cell)?);
    let mut variants = vec![primary];
    if cell.case == Case::Accusative
        && cell.animacy == Animacy::Animate
        && cell.number == Number::Singular
        && cell.gender == Gender::Masculine
    {
        let genitive = join(
            stem,
            short_comparison_ending(AdjectiveCell {
                case: Case::Genitive,
                ..cell
            })?,
        );
        if !variants.contains(&genitive) {
            variants.push(genitive);
        }
    }
    if cell.number == Number::Plural
        && matches!(cell.case, Case::Nominative | Case::Vocative)
        && cell.gender == Gender::Masculine
    {
        variants.push(join(stem, "и"));
    }
    Ok(variants)
}

pub(crate) fn short_comparison_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, _, Nom | Voc) | (Sg, N, Acc) => {
            return Err(Error::ContradictoryMetadata {
                reason: "short comparison citation cells require a typed citation edge".into(),
            });
        }
        (Sg, M | N, Gen) => "а",
        (Sg, F, Gen | Dat | Loc) => "и",
        (Sg, M | N, Dat) => "ꙋ",
        (Sg, M, Acc) => "ъ",
        (Sg, F, Acc) => "ꙋ",
        (Sg, M | N, Ins) => "имъ",
        (Sg, F, Ins) => "ею",
        (Sg, M | N, Loc) => "и",
        (Du, M, Nom | Acc | Voc) => "а",
        (Du, F | N, Nom | Acc | Voc) => "и",
        (Du, _, Gen | Loc) => "ꙋ",
        (Du, _, Dat | Ins) => "има",
        (Pl, M, Nom | Voc) => "е",
        (Pl, F, Nom | Voc) => "ѧ",
        (Pl, N, Nom | Acc | Voc) => "а",
        (Pl, _, Gen | Loc) => "ихъ",
        (Pl, _, Dat) => "ымъ",
        (Pl, M | F, Acc) => "ѧ",
        (Pl, _, Ins) => "ими",
    })
}

pub(crate) fn decline_short_active_participle_stem(
    stem: &str,
    cell: AdjectiveCell,
    citation: Option<Vec<String>>,
) -> Result<Vec<String>> {
    if cell.case == Case::Vocative {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §98 gives no vocative in the short active-participle declension".into(),
        });
    }
    if cell.number == Number::Singular && cell.case == Case::Nominative {
        return citation.ok_or_else(|| Error::ContradictoryMetadata {
            reason: "short active-participle citation variants are missing".into(),
        });
    }
    let primary = join(stem, short_active_participle_ending(cell)?);
    let mut variants = vec![primary];
    if cell.case == Case::Accusative
        && cell.animacy == Animacy::Animate
        && (cell.gender == Gender::Masculine
            || (cell.gender == Gender::Feminine && cell.number == Number::Plural))
    {
        let genitive = join(
            stem,
            short_active_participle_ending(AdjectiveCell {
                case: Case::Genitive,
                ..cell
            })?,
        );
        if !variants.contains(&genitive) {
            variants.push(genitive);
        }
    }
    if cell.number == Number::Plural
        && cell.case == Case::Nominative
        && cell.gender == Gender::Feminine
    {
        variants.push(join(stem, "е"));
    }
    Ok(variants)
}

pub(crate) fn short_active_participle_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, _, Nom) => {
            return Err(Error::ContradictoryMetadata {
                reason: "short active-participle nominatives require a typed citation edge".into(),
            });
        }
        (Sg, M | N, Gen) => "а",
        (Sg, F, Gen | Dat | Loc) => "и",
        (Sg, M | N, Dat) => "ꙋ",
        (Sg, M, Acc) => "ъ",
        (Sg, F, Acc) => "ꙋ",
        (Sg, N, Acc) => "е",
        (Sg, M | N, Ins) => "имъ",
        (Sg, F, Ins) => "ею",
        (Sg, M | N, Loc) => "емъ",
        (Du, M, Nom | Acc) => "а",
        (Du, F | N, Nom | Acc) => "ѣ",
        (Du, _, Gen | Loc) => "ꙋ",
        (Du, _, Dat | Ins) => "ема",
        (Pl, M, Nom) => "е",
        (Pl, F, Nom) => "ѧ",
        (Pl, N, Nom | Acc) => "а",
        (Pl, _, Gen | Loc) => "ихъ",
        (Pl, _, Dat) => "ымъ",
        (Pl, M | F, Acc) => "ѧ",
        (Pl, _, Ins) => "ими",
        (_, _, Voc) => {
            return Err(Error::HistoricallyInvalidCell {
                reason: "short active participle has no vocative cell".into(),
            });
        }
    })
}

pub(crate) fn comparison_citation_variants(
    stem: &str,
    formation: ComparisonFormation,
    gender: Gender,
) -> Result<Option<Vec<String>>> {
    let variants = match gender {
        Gender::Feminine => vec![join(stem, "и")],
        Gender::Masculine => vec![comparison_edge_without_suffix(stem, formation, false)?],
        Gender::Neuter => vec![
            comparison_edge_without_suffix(stem, formation, true)?,
            join(stem, "е"),
        ],
    };
    Ok(Some(variants))
}

pub(crate) fn comparison_edge_without_suffix(
    stem: &str,
    formation: ComparisonFormation,
    neuter: bool,
) -> Result<String> {
    let (suffix, ending) = match (formation, neuter) {
        (ComparisonFormation::AncientHard, false) => ("ш", "їй"),
        (ComparisonFormation::AncientHard, true) => ("ш", "е"),
        (ComparisonFormation::AncientSoft, false) => ("ьш", "їй"),
        (ComparisonFormation::AncientSoft, true) => ("ьш", "е"),
        (ComparisonFormation::LaterYat | ComparisonFormation::LaterAi, false) => ("ш", ""),
        (ComparisonFormation::LaterYat | ComparisonFormation::LaterAi, true) => ("йш", "е"),
    };
    let base = stem
        .strip_suffix(suffix)
        .ok_or_else(|| Error::ContradictoryMetadata {
            reason: format!("comparison stem {stem:?} does not match {formation:?}"),
        })?;
    Ok(join(base, ending))
}

pub(crate) fn active_participle_citation_variants(
    stem: &str,
    formation: ActiveParticipleShortFormation,
    gender: Gender,
) -> Result<Option<Vec<String>>> {
    if gender == Gender::Feminine {
        return Ok(Some(vec![join(stem, "и")]));
    }
    let primary = match formation {
        ActiveParticipleShortFormation::PresentFirstUnpalatalized => {
            let base = stem
                .strip_suffix("ꙋщ")
                .ok_or_else(|| Error::ContradictoryMetadata {
                    reason: format!(
                        "present first-conjugation participle stem {stem:?} must end in ꙋщ"
                    ),
                })?;
            join(base, "ый")
        }
        ActiveParticipleShortFormation::PresentFirstPalatalized => {
            let base = stem
                .strip_suffix("ющ")
                .ok_or_else(|| Error::ContradictoryMetadata {
                    reason: format!("first-palatalized participle stem {stem:?} must end in ющ"),
                })?;
            join(base, "ѧ")
        }
        ActiveParticipleShortFormation::PresentSecond => stem
            .strip_suffix('щ')
            .ok_or_else(|| Error::ContradictoryMetadata {
                reason: format!("second-conjugation participle stem {stem:?} must end in щ"),
            })?
            .to_owned(),
        ActiveParticipleShortFormation::PresentAfterSibilant => stem
            .strip_suffix('щ')
            .ok_or_else(|| Error::ContradictoryMetadata {
                reason: format!("after-sibilant participle stem {stem:?} must end in щ"),
            })?
            .to_owned(),
        ActiveParticipleShortFormation::PastConsonant
        | ActiveParticipleShortFormation::PastVowel => {
            let base = stem
                .strip_suffix('ш')
                .ok_or_else(|| Error::ContradictoryMetadata {
                    reason: format!("past participle stem {stem:?} must end in ш"),
                })?;
            join(base, "ъ")
        }
        ActiveParticipleShortFormation::PastIotated => stem
            .strip_suffix('ш')
            .ok_or_else(|| Error::ContradictoryMetadata {
                reason: format!("iotated past participle stem {stem:?} must end in ш"),
            })?
            .to_owned(),
    };
    let mut variants = vec![primary];
    if formation == ActiveParticipleShortFormation::PresentAfterSibilant {
        let base = stem
            .strip_suffix("ащ")
            .ok_or_else(|| Error::ContradictoryMetadata {
                reason: format!("after-sibilant participle stem {stem:?} must end in ащ"),
            })?;
        variants.push(join(base, "ѧ"));
    }
    let retained = match formation {
        ActiveParticipleShortFormation::PresentFirstUnpalatalized
        | ActiveParticipleShortFormation::PresentFirstPalatalized
        | ActiveParticipleShortFormation::PresentSecond
        | ActiveParticipleShortFormation::PresentAfterSibilant => Some(join(stem, "ь")),
        ActiveParticipleShortFormation::PastConsonant
        | ActiveParticipleShortFormation::PastVowel => Some(join(stem, "ъ")),
        ActiveParticipleShortFormation::PastIotated => None,
    };
    if gender == Gender::Neuter {
        variants.push(join(stem, "е"));
        if formation != ActiveParticipleShortFormation::PastIotated {
            variants.push(join(stem, "о"));
        }
    } else if let Some(retained) = retained
        && !variants.contains(&retained)
    {
        variants.push(retained);
    }
    Ok(Some(variants))
}

pub(crate) fn decline_adjectival_stem(
    stem: &SynodalWord,
    class: AdjectiveClass,
    cell: AdjectiveCell,
    rule: &'static str,
    stage: &'static str,
    lemma: &str,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let ending = match cell.form {
        AdjectiveForm::Short => short_adjective_ending(class, cell)?,
        AdjectiveForm::Long => long_adjective_ending(class, cell)?,
    };
    let mut expanded = vec![join(stem.canonical(), ending)];
    if cell.case == Case::Accusative && cell.animacy == Animacy::Animate {
        let nominative_cell = AdjectiveCell {
            animacy: Animacy::Inanimate,
            ..cell
        };
        let nominative_ending = match cell.form {
            AdjectiveForm::Short => short_adjective_ending(class, nominative_cell)?,
            AdjectiveForm::Long => long_adjective_ending(class, nominative_cell)?,
        };
        let nominative_like = join(stem.canonical(), nominative_ending);
        if cell.number == Number::Plural {
            expanded.insert(0, nominative_like);
        } else if !expanded.contains(&nominative_like) {
            expanded.push(nominative_like);
        }
        expanded.dedup();
    }
    normative_variants(expanded, rule, profile, stage, lemma)
}
