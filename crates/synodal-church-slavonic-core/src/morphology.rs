//! Productive Synodal rules admitted from Alypy (Gamanovich) §§33–44, 53,
//! 57, 79–80, 86–87, 93, and 97.

use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, AuthorityRole, Case, Comparison, Confidence,
    EpistemicRole, Error, Evidence, EvidenceId, EvidenceKind, FiniteTense, FiniteVerbCell, FormSet,
    FormSource, FormVariant, Gender, GenerationPolicy, ImperativeCell, LParticipleCell,
    MetadataField, Number, OrthographyProfile, Person, Recension, Result, RuleId, RuleTrace,
    SourceId, SynodalWord, TraceStep,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NounDeclension {
    FirstHardMasculine,
    FirstHardNeuter,
    SecondHard,
    SecondSoft,
    ThirdFeminine,
}

impl NounDeclension {
    pub const ALL: [Self; 5] = [
        Self::FirstHardMasculine,
        Self::FirstHardNeuter,
        Self::SecondHard,
        Self::SecondSoft,
        Self::ThirdFeminine,
    ];
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NounLexeme {
    pub lemma: SynodalWord,
    pub stem: SynodalWord,
    pub gender: Gender,
    pub declension: NounDeclension,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AdjectiveClass {
    Hard,
    Soft,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AdjectiveLexeme {
    pub lemma: SynodalWord,
    pub stem: SynodalWord,
    pub class: AdjectiveClass,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Aspect {
    Imperfective,
    Perfective,
    Biaspectual,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum VerbConjugation {
    FirstUnpalatalized,
    FirstPalatalized,
    Second,
    Archaic,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AoristFormation {
    VowelStem,
    ConsonantStem,
    Irregular,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ImperfectFormation {
    H,
    Yah,
    Ah,
    Irregular,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ImperativeFormation {
    FirstUnpalatalized,
    ISeries,
    Irregular,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VerbLexeme {
    pub lemma: SynodalWord,
    pub aspect: Aspect,
    pub conjugation: VerbConjugation,
    /// Base before the `е`/`и` connective vowel.
    pub present_stem: Option<SynodalWord>,
    /// The complete first-person singular, including lexical alternation.
    pub present_first_singular: Option<SynodalWord>,
    /// The complete third-person plural, including `ꙋтъ/ютъ/атъ/ѧтъ` choice.
    pub present_third_plural: Option<SynodalWord>,
    /// Base selected independently for the imperfect marker.
    pub imperfect_stem: Option<SynodalWord>,
    pub imperfect_formation: Option<ImperfectFormation>,
    /// Base selected independently for the aorist marker.
    pub aorist_stem: Option<SynodalWord>,
    pub aorist_formation: Option<AoristFormation>,
    /// Base selected independently for imperative suffixes.
    pub imperative_stem: Option<SynodalWord>,
    pub imperative_formation: Option<ImperativeFormation>,
    /// Base after any lexical consonant deletion, before `л`.
    pub l_participle_stem: Option<SynodalWord>,
}

pub fn decline_noun(
    lexeme: &NounLexeme,
    cell: crate::NounCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    validate_noun_metadata(lexeme)?;
    let ending = noun_ending(lexeme, cell)?;
    let mut expanded = vec![join(lexeme.stem.canonical(), ending)];
    if cell.case == Case::Accusative && cell.animacy == Animacy::Animate {
        let nominative_like = noun_ending(
            lexeme,
            crate::NounCell {
                animacy: Animacy::Inanimate,
                ..cell
            },
        )?;
        let nominative_like = join(lexeme.stem.canonical(), nominative_like);
        if cell.number == Number::Plural {
            expanded.insert(0, nominative_like);
        } else if !expanded.contains(&nominative_like) {
            expanded.push(nominative_like);
        }
        expanded.dedup();
    }
    normative_variants(
        expanded,
        noun_rule(lexeme.declension),
        profile,
        "noun-declension",
        lexeme.lemma.canonical(),
    )
}

pub fn decline_adjective(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if cell.comparison != Comparison::Positive {
        return Err(Error::UnsupportedCell {
            reason: "productive comparison requires a separately sourced comparison stem".into(),
        });
    }
    let ending = match cell.form {
        AdjectiveForm::Short => short_adjective_ending(lexeme.class, cell)?,
        AdjectiveForm::Long => long_adjective_ending(lexeme.class, cell)?,
    };
    let rule = match (lexeme.class, cell.form) {
        (AdjectiveClass::Hard, AdjectiveForm::Short) => "SYN-ADJ-SHORT-HARD-ALYPY-53",
        (AdjectiveClass::Soft, AdjectiveForm::Short) => "SYN-ADJ-SHORT-SOFT-ALYPY-53",
        (AdjectiveClass::Hard, AdjectiveForm::Long) => "SYN-ADJ-LONG-HARD-ALYPY-57",
        (AdjectiveClass::Soft, AdjectiveForm::Long) => "SYN-ADJ-LONG-SOFT-ALYPY-57",
    };
    let mut expanded = vec![join(lexeme.stem.canonical(), ending)];
    if cell.case == Case::Accusative && cell.animacy == Animacy::Animate {
        let nominative_cell = AdjectiveCell {
            animacy: Animacy::Inanimate,
            ..cell
        };
        let nominative_ending = match cell.form {
            AdjectiveForm::Short => short_adjective_ending(lexeme.class, nominative_cell)?,
            AdjectiveForm::Long => long_adjective_ending(lexeme.class, nominative_cell)?,
        };
        let nominative_like = join(lexeme.stem.canonical(), nominative_ending);
        if cell.number == Number::Plural {
            expanded.insert(0, nominative_like);
        } else if !expanded.contains(&nominative_like) {
            expanded.push(nominative_like);
        }
        expanded.dedup();
    }
    normative_variants(
        expanded,
        rule,
        profile,
        match cell.form {
            AdjectiveForm::Short => "short-adjective-declension",
            AdjectiveForm::Long => "long-adjective-declension",
        },
        lexeme.lemma.canonical(),
    )
}

pub fn present(
    lexeme: &VerbLexeme,
    person: Person,
    number: Number,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let cell = FiniteVerbCell {
        tense: FiniteTense::Present,
        person,
        number,
    };
    let text = match (person, number) {
        (Person::First, Number::Singular) => required(
            lexeme.present_first_singular.as_ref(),
            MetadataField::PresentFirstSingular,
        )?
        .canonical()
        .to_owned(),
        (Person::Third, Number::Plural) => required(
            lexeme.present_third_plural.as_ref(),
            MetadataField::PresentThirdPlural,
        )?
        .canonical()
        .to_owned(),
        _ => {
            let stem = required(lexeme.present_stem.as_ref(), MetadataField::PresentStem)?;
            join(stem.canonical(), present_ending(lexeme.conjugation, cell)?)
        }
    };
    normative(
        text,
        "SYN-VERB-PRESENT-ALYPY-80",
        profile,
        "present",
        lexeme.lemma.canonical(),
    )
}

pub fn aorist(
    lexeme: &VerbLexeme,
    person: Person,
    number: Number,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let formation = lexeme.aorist_formation.ok_or(Error::MissingPrincipalPart {
        field: MetadataField::AoristStem,
    })?;
    if formation == AoristFormation::Irregular {
        return Err(Error::UnsupportedFormation {
            formation: "irregular Synodal aorist requires an exact table".into(),
        });
    }
    let stem = required(lexeme.aorist_stem.as_ref(), MetadataField::AoristStem)?;
    let ending = aorist_ending(formation, person, number);
    let stem_text = if formation == AoristFormation::ConsonantStem
        && matches!(person, Person::Second | Person::Third)
        && number == Number::Singular
    {
        palatalize_final_velar(stem.canonical())
    } else {
        stem.canonical().to_owned()
    };
    normative(
        join(&stem_text, ending),
        match formation {
            AoristFormation::VowelStem => "SYN-VERB-AORIST-VOWEL-ALYPY-86",
            AoristFormation::ConsonantStem => "SYN-VERB-AORIST-CONSONANT-ALYPY-86",
            AoristFormation::Irregular => unreachable!(),
        },
        profile,
        "aorist",
        lexeme.lemma.canonical(),
    )
}

pub fn imperfect(
    lexeme: &VerbLexeme,
    person: Person,
    number: Number,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if lexeme.aspect == Aspect::Perfective {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §87 restricts the productive imperfect to imperfective verbs".into(),
        });
    }
    let formation = lexeme
        .imperfect_formation
        .ok_or(Error::MissingPrincipalPart {
            field: MetadataField::ImperfectStem,
        })?;
    if formation == ImperfectFormation::Irregular {
        return Err(Error::UnsupportedFormation {
            formation: "irregular Synodal imperfect requires an exact table".into(),
        });
    }
    let stem = required(lexeme.imperfect_stem.as_ref(), MetadataField::ImperfectStem)?;
    normative(
        join(
            stem.canonical(),
            imperfect_ending(formation, person, number),
        ),
        match formation {
            ImperfectFormation::H => "SYN-VERB-IMPERFECT-H-ALYPY-87",
            ImperfectFormation::Yah => "SYN-VERB-IMPERFECT-YAH-ALYPY-87",
            ImperfectFormation::Ah => "SYN-VERB-IMPERFECT-AH-ALYPY-87",
            ImperfectFormation::Irregular => unreachable!(),
        },
        profile,
        "imperfect",
        lexeme.lemma.canonical(),
    )
}

pub fn imperative(
    lexeme: &VerbLexeme,
    cell: ImperativeCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if cell.person == Person::First && cell.number == Number::Singular {
        return Err(Error::HistoricallyInvalidCell {
            reason: "the imperative has no first-person singular".into(),
        });
    }
    if cell.person == Person::Third && cell.number != Number::Singular {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §93 excludes third-person dual and plural imperatives".into(),
        });
    }
    let formation = lexeme
        .imperative_formation
        .ok_or(Error::MissingPrincipalPart {
            field: MetadataField::ImperativeStem,
        })?;
    if formation == ImperativeFormation::Irregular {
        return Err(Error::UnsupportedFormation {
            formation: "irregular Synodal imperative requires an exact table".into(),
        });
    }
    let stem = required(
        lexeme.imperative_stem.as_ref(),
        MetadataField::ImperativeStem,
    )?;
    normative(
        join(stem.canonical(), imperative_ending(formation, cell)),
        "SYN-VERB-IMPERATIVE-ALYPY-93",
        profile,
        "imperative",
        lexeme.lemma.canonical(),
    )
}

pub fn infinitive(lexeme: &VerbLexeme, profile: OrthographyProfile) -> Result<FormSet> {
    normative(
        lexeme.lemma.canonical().to_owned(),
        "SYN-VERB-INFINITIVE-LEXICAL",
        profile,
        "infinitive",
        lexeme.lemma.canonical(),
    )
}

pub fn l_participle(
    lexeme: &VerbLexeme,
    cell: LParticipleCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let stem = required(
        lexeme.l_participle_stem.as_ref(),
        MetadataField::LParticipleStem,
    )?;
    let ending = match (cell.number, cell.gender) {
        (Number::Singular, Gender::Masculine) => "лъ",
        (Number::Singular, Gender::Feminine) => "ла",
        (Number::Singular, Gender::Neuter) => "ло",
        (Number::Dual, Gender::Masculine) => "ла",
        (Number::Dual, Gender::Feminine | Gender::Neuter) => "ли",
        (Number::Plural, _) => "ли",
    };
    normative(
        join(stem.canonical(), ending),
        "SYN-VERB-LPART-ALYPY-97",
        profile,
        "l-participle",
        lexeme.lemma.canonical(),
    )
}

fn validate_noun_metadata(lexeme: &NounLexeme) -> Result<()> {
    let valid = matches!(
        (lexeme.declension, lexeme.gender),
        (NounDeclension::FirstHardMasculine, Gender::Masculine)
            | (NounDeclension::FirstHardNeuter, Gender::Neuter)
            | (NounDeclension::SecondHard | NounDeclension::SecondSoft, _)
            | (NounDeclension::ThirdFeminine, Gender::Feminine)
    );
    if valid {
        Ok(())
    } else {
        Err(Error::ContradictoryMetadata {
            reason: "declension and lexical gender are incompatible".into(),
        })
    }
}

fn noun_rule(declension: NounDeclension) -> &'static str {
    match declension {
        NounDeclension::FirstHardMasculine => "SYN-NOUN-I-HARD-M-ALYPY-34",
        NounDeclension::FirstHardNeuter => "SYN-NOUN-I-HARD-N-ALYPY-34",
        NounDeclension::SecondHard => "SYN-NOUN-II-HARD-ALYPY-39",
        NounDeclension::SecondSoft => "SYN-NOUN-II-SOFT-ALYPY-39",
        NounDeclension::ThirdFeminine => "SYN-NOUN-III-F-ALYPY-41",
    }
}

fn noun_ending(lexeme: &NounLexeme, cell: crate::NounCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let animate_acc = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    let ending = match (lexeme.declension, cell.number, cell.case) {
        (NounDeclension::FirstHardMasculine, Sg, Nom) => "ъ",
        (NounDeclension::FirstHardMasculine, Sg, Gen) => "а",
        (NounDeclension::FirstHardMasculine, Sg, Dat) => "ꙋ",
        (NounDeclension::FirstHardMasculine, Sg, Acc) => animate_acc("ъ", "а"),
        (NounDeclension::FirstHardMasculine, Sg, Ins) => "омъ",
        (NounDeclension::FirstHardMasculine, Sg, Loc) => "ѣ",
        (NounDeclension::FirstHardMasculine, Sg, Voc) => "е",
        (NounDeclension::FirstHardMasculine, Du, Nom | Acc | Voc) => "а",
        (NounDeclension::FirstHardMasculine, Du, Gen | Loc) => "ꙋ",
        (NounDeclension::FirstHardMasculine, Du, Dat | Ins) => "ома",
        (NounDeclension::FirstHardMasculine, Pl, Nom | Voc) => "и",
        (NounDeclension::FirstHardMasculine, Pl, Gen) => "овъ",
        (NounDeclension::FirstHardMasculine, Pl, Dat) => "омъ",
        (NounDeclension::FirstHardMasculine, Pl, Acc) => animate_acc("ы", "овъ"),
        (NounDeclension::FirstHardMasculine, Pl, Ins) => "ы",
        (NounDeclension::FirstHardMasculine, Pl, Loc) => "ѣхъ",

        (NounDeclension::FirstHardNeuter, Sg, Nom | Acc | Voc) => "о",
        (NounDeclension::FirstHardNeuter, Sg, Gen) => "а",
        (NounDeclension::FirstHardNeuter, Sg, Dat) => "ꙋ",
        (NounDeclension::FirstHardNeuter, Sg, Ins) => "омъ",
        (NounDeclension::FirstHardNeuter, Sg, Loc) => "ѣ",
        (NounDeclension::FirstHardNeuter, Du, Nom | Acc | Voc) => "а",
        (NounDeclension::FirstHardNeuter, Du, Gen | Loc) => "ꙋ",
        (NounDeclension::FirstHardNeuter, Du, Dat | Ins) => "ома",
        (NounDeclension::FirstHardNeuter, Pl, Nom | Acc | Voc) => "а",
        (NounDeclension::FirstHardNeuter, Pl, Gen) => "ъ",
        (NounDeclension::FirstHardNeuter, Pl, Dat) => "омъ",
        (NounDeclension::FirstHardNeuter, Pl, Ins) => "ы",
        (NounDeclension::FirstHardNeuter, Pl, Loc) => "ѣхъ",

        (NounDeclension::SecondHard, Sg, Nom) => "а",
        (NounDeclension::SecondHard, Sg, Gen) => "ы",
        (NounDeclension::SecondHard, Sg, Dat | Loc) => "ѣ",
        (NounDeclension::SecondHard, Sg, Acc) => "ꙋ",
        (NounDeclension::SecondHard, Sg, Ins) => "ою",
        (NounDeclension::SecondHard, Sg, Voc) => "о",
        (NounDeclension::SecondHard, Du, Nom | Acc | Voc) => "ѣ",
        (NounDeclension::SecondHard, Du, Gen | Loc) => "ꙋ",
        (NounDeclension::SecondHard, Du, Dat | Ins) => "ама",
        (NounDeclension::SecondHard, Pl, Nom | Voc) => "ы",
        (NounDeclension::SecondHard, Pl, Gen) => "ъ",
        (NounDeclension::SecondHard, Pl, Dat) => "амъ",
        (NounDeclension::SecondHard, Pl, Acc) => animate_acc("ы", "ъ"),
        (NounDeclension::SecondHard, Pl, Ins) => "ами",
        (NounDeclension::SecondHard, Pl, Loc) => "ахъ",

        (NounDeclension::SecondSoft, Sg, Nom) => "ѧ",
        (NounDeclension::SecondSoft, Sg, Gen | Dat | Loc) => "и",
        (NounDeclension::SecondSoft, Sg, Acc) => "ю",
        (NounDeclension::SecondSoft, Sg, Ins) => "ею",
        (NounDeclension::SecondSoft, Sg, Voc) => "е",
        (NounDeclension::SecondSoft, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::SecondSoft, Du, Gen | Loc) => "ю",
        (NounDeclension::SecondSoft, Du, Dat | Ins) => "ѧма",
        (NounDeclension::SecondSoft, Pl, Nom | Voc) => "и",
        (NounDeclension::SecondSoft, Pl, Gen) => "ь",
        (NounDeclension::SecondSoft, Pl, Dat) => "ѧмъ",
        (NounDeclension::SecondSoft, Pl, Acc) => animate_acc("и", "ь"),
        (NounDeclension::SecondSoft, Pl, Ins) => "ѧми",
        (NounDeclension::SecondSoft, Pl, Loc) => "ѧхъ",

        (NounDeclension::ThirdFeminine, Sg, Nom | Acc) => "ь",
        (NounDeclension::ThirdFeminine, Sg, Gen | Dat | Loc) => "и",
        (NounDeclension::ThirdFeminine, Sg, Ins) => "їю",
        (NounDeclension::ThirdFeminine, Sg, Voc) => "е",
        (NounDeclension::ThirdFeminine, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::ThirdFeminine, Du, Gen | Loc) => "їю",
        (NounDeclension::ThirdFeminine, Du, Dat | Ins) => "ема",
        (NounDeclension::ThirdFeminine, Pl, Nom | Voc | Acc) => "и",
        (NounDeclension::ThirdFeminine, Pl, Gen) => "ей",
        (NounDeclension::ThirdFeminine, Pl, Dat) => "емъ",
        (NounDeclension::ThirdFeminine, Pl, Ins) => "ьми",
        (NounDeclension::ThirdFeminine, Pl, Loc) => "ехъ",
    };
    Ok(ending)
}

fn short_adjective_ending(class: AdjectiveClass, cell: AdjectiveCell) -> Result<&'static str> {
    if class == AdjectiveClass::Soft {
        return soft_short_adjective_ending(cell);
    }
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, M, Nom) => "ъ",
        (Sg, M, Gen) => "а",
        (Sg, M, Dat) => "ꙋ",
        (Sg, M, Acc) => animate("ъ", "а"),
        (Sg, M, Ins) => "ымъ",
        (Sg, M, Loc) => "ѣ",
        (Sg, M, Voc) => "е",
        (Sg, F, Nom | Voc) => "а",
        (Sg, F, Gen) => "ы",
        (Sg, F, Dat | Loc) => "ѣ",
        (Sg, F, Acc) => "ꙋ",
        (Sg, F, Ins) => "ою",
        (Sg, N, Nom | Acc | Voc) => "о",
        (Sg, N, Gen) => "а",
        (Sg, N, Dat) => "ꙋ",
        (Sg, N, Ins) => "ымъ",
        (Sg, N, Loc) => "ѣ",
        (Du, M, Nom | Acc | Voc) => "а",
        (Du, F | N, Nom | Acc | Voc) => "ѣ",
        (Du, _, Gen | Loc) => "ꙋ",
        (Du, _, Dat | Ins) => "ыма",
        (Pl, M, Nom | Voc) => "и",
        (Pl, F, Nom | Acc | Voc) => "ы",
        (Pl, N, Nom | Acc | Voc) => "а",
        (Pl, _, Gen | Loc) => "ыхъ",
        (Pl, _, Dat) => "ымъ",
        (Pl, M, Acc) => animate("ы", "ыхъ"),
        (Pl, _, Ins) => "ыми",
    })
}

fn soft_short_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, M, Nom | Voc) => "ь",
        (Sg, M, Gen) => "ѧ",
        (Sg, M, Dat) => "ю",
        (Sg, M, Acc) => animate("ь", "ѧ"),
        (Sg, M, Ins) => "имъ",
        (Sg, M, Loc) => "и",
        (Sg, F, Nom | Voc) => "ѧ",
        (Sg, F, Gen | Dat | Loc) => "и",
        (Sg, F, Acc) => "ю",
        (Sg, F, Ins) => "ею",
        (Sg, N, Nom | Acc | Voc) => "е",
        (Sg, N, Gen) => "ѧ",
        (Sg, N, Dat) => "ю",
        (Sg, N, Ins) => "имъ",
        (Sg, N, Loc) => "и",
        (Du, M, Nom | Acc | Voc) => "ѧ",
        (Du, F | N, Nom | Acc | Voc) => "и",
        (Du, _, Gen | Loc) => "ю",
        (Du, _, Dat | Ins) => "има",
        (Pl, M | F, Nom | Voc) => "и",
        (Pl, N, Nom | Acc | Voc) => "ѧ",
        (Pl, _, Gen | Loc) => "ихъ",
        (Pl, _, Dat) => "имъ",
        (Pl, M | F, Acc) => animate("и", "ихъ"),
        (Pl, _, Ins) => "ими",
    })
}

fn long_adjective_ending(class: AdjectiveClass, cell: AdjectiveCell) -> Result<&'static str> {
    if class == AdjectiveClass::Soft {
        return soft_long_adjective_ending(cell);
    }
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, M, Nom | Voc) => "ый",
        (Sg, M, Gen) => "агѡ",
        (Sg, M, Dat) => "омꙋ",
        (Sg, M, Acc) => animate("ый", "аго"),
        (Sg, M, Ins) => "ымъ",
        (Sg, M, Loc) => "ѣмъ",
        (Sg, F, Nom | Voc) => "аѧ",
        (Sg, F, Gen) => "ыѧ",
        (Sg, F, Dat | Loc) => "ѣй",
        (Sg, F, Acc) => "ꙋю",
        (Sg, F, Ins) => "ою",
        (Sg, N, Nom | Acc | Voc) => "ое",
        (Sg, N, Gen) => "агѡ",
        (Sg, N, Dat) => "омꙋ",
        (Sg, N, Ins) => "ымъ",
        (Sg, N, Loc) => "ѣмъ",
        (Du, M, Nom | Acc | Voc) => "аѧ",
        (Du, F | N, Nom | Acc | Voc) => "ѣи",
        (Du, _, Gen | Loc) => "ꙋю",
        (Du, _, Dat | Ins) => "ыма",
        (Pl, M, Nom | Voc) => "їи",
        (Pl, F, Nom | Voc) => "ыѧ",
        (Pl, N, Nom | Acc | Voc) => "аѧ",
        (Pl, _, Gen | Loc) => "ыхъ",
        (Pl, _, Dat) => "ымъ",
        (Pl, M | F, Acc) => animate("ыѧ", "ыхъ"),
        (Pl, _, Ins) => "ыми",
    })
}

fn soft_long_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, M, Nom | Voc) => "їй",
        (Sg, M, Gen) => "ѧгѡ",
        (Sg, M, Dat) => "емꙋ",
        (Sg, M, Acc) => animate("їй", "ѧго"),
        (Sg, M, Ins) => "имъ",
        (Sg, M, Loc) => "емъ",
        (Sg, F, Nom | Voc) => "ѧѧ",
        (Sg, F, Gen) => "їѧ",
        (Sg, F, Dat | Loc) => "ей",
        (Sg, F, Acc) => "юю",
        (Sg, F, Ins) => "ею",
        (Sg, N, Nom | Acc | Voc) => "ее",
        (Sg, N, Gen) => "ѧгѡ",
        (Sg, N, Dat) => "емꙋ",
        (Sg, N, Ins) => "имъ",
        (Sg, N, Loc) => "емъ",
        (Du, M, Nom | Acc | Voc) => "ѧѧ",
        (Du, F | N, Nom | Acc | Voc) => "їи",
        (Du, _, Gen | Loc) => "юю",
        (Du, _, Dat | Ins) => "има",
        (Pl, M, Nom | Voc) => "їи",
        (Pl, F, Nom | Voc) => "їѧ",
        (Pl, N, Nom | Acc | Voc) => "ѧѧ",
        (Pl, _, Gen | Loc) => "ихъ",
        (Pl, _, Dat) => "имъ",
        (Pl, M | F, Acc) => animate("їѧ", "ихъ"),
        (Pl, _, Ins) => "ими",
    })
}

fn present_ending(conjugation: VerbConjugation, cell: FiniteVerbCell) -> Result<&'static str> {
    if conjugation == VerbConjugation::Archaic {
        return Err(Error::UnsupportedFormation {
            formation: "archaic present requires an exact lexeme table".into(),
        });
    }
    let vowel = if conjugation == VerbConjugation::Second {
        "и"
    } else {
        "е"
    };
    Ok(match (cell.person, cell.number, vowel) {
        (Person::Second, Number::Singular, "е") => "еши",
        (Person::Second, Number::Singular, "и") => "иши",
        (Person::Third, Number::Singular, "е") => "етъ",
        (Person::Third, Number::Singular, "и") => "итъ",
        (Person::First, Number::Dual, "е") => "ева",
        (Person::First, Number::Dual, "и") => "ива",
        (Person::Second | Person::Third, Number::Dual, "е") => "ета",
        (Person::Second | Person::Third, Number::Dual, "и") => "ита",
        (Person::First, Number::Plural, "е") => "емъ",
        (Person::First, Number::Plural, "и") => "имъ",
        (Person::Second, Number::Plural, "е") => "ете",
        (Person::Second, Number::Plural, "и") => "ите",
        (Person::First, Number::Singular, _) | (Person::Third, Number::Plural, _) => {
            return Err(Error::ContradictoryMetadata {
                reason: "suppletive present edge cells must use their explicit principal part"
                    .into(),
            });
        }
        (_, _, _) => {
            return Err(Error::HistoricallyInvalidCell {
                reason: "invalid present cell".into(),
            });
        }
    })
}

fn aorist_ending(formation: AoristFormation, person: Person, number: Number) -> &'static str {
    let consonant = formation == AoristFormation::ConsonantStem;
    match (person, number, consonant) {
        (Person::First, Number::Singular, false) => "хъ",
        (Person::First, Number::Singular, true) => "охъ",
        (Person::Second | Person::Third, Number::Singular, false) => "",
        (Person::Second | Person::Third, Number::Singular, true) => "е",
        (Person::First, Number::Dual, false) => "хова",
        (Person::First, Number::Dual, true) => "охова",
        (Person::Second | Person::Third, Number::Dual, false) => "ста",
        (Person::Second | Person::Third, Number::Dual, true) => "оста",
        (Person::First, Number::Plural, false) => "хомъ",
        (Person::First, Number::Plural, true) => "охомъ",
        (Person::Second, Number::Plural, false) => "сте",
        (Person::Second, Number::Plural, true) => "осте",
        (Person::Third, Number::Plural, false) => "ша",
        (Person::Third, Number::Plural, true) => "оша",
    }
}

fn imperfect_ending(formation: ImperfectFormation, person: Person, number: Number) -> &'static str {
    match (formation, person, number) {
        (ImperfectFormation::H, Person::First, Number::Singular) => "хъ",
        (ImperfectFormation::H, Person::Second | Person::Third, Number::Singular) => "ше",
        (ImperfectFormation::H, Person::First, Number::Dual) => "хова",
        (ImperfectFormation::H, Person::Second | Person::Third, Number::Dual) => "ста",
        (ImperfectFormation::H, Person::First, Number::Plural) => "хомъ",
        (ImperfectFormation::H, Person::Second, Number::Plural) => "сте",
        (ImperfectFormation::H, Person::Third, Number::Plural) => "хꙋ",
        (ImperfectFormation::Yah, Person::First, Number::Singular) => "ѧхъ",
        (ImperfectFormation::Yah, Person::Second | Person::Third, Number::Singular) => "ѧше",
        (ImperfectFormation::Yah, Person::First, Number::Dual) => "ѧхова",
        (ImperfectFormation::Yah, Person::Second | Person::Third, Number::Dual) => "ѧста",
        (ImperfectFormation::Yah, Person::First, Number::Plural) => "ѧхомъ",
        (ImperfectFormation::Yah, Person::Second, Number::Plural) => "ѧсте",
        (ImperfectFormation::Yah, Person::Third, Number::Plural) => "ѧхꙋ",
        (ImperfectFormation::Ah, Person::First, Number::Singular) => "ахъ",
        (ImperfectFormation::Ah, Person::Second | Person::Third, Number::Singular) => "аше",
        (ImperfectFormation::Ah, Person::First, Number::Dual) => "ахова",
        (ImperfectFormation::Ah, Person::Second | Person::Third, Number::Dual) => "аста",
        (ImperfectFormation::Ah, Person::First, Number::Plural) => "ахомъ",
        (ImperfectFormation::Ah, Person::Second, Number::Plural) => "асте",
        (ImperfectFormation::Ah, Person::Third, Number::Plural) => "ахꙋ",
        (ImperfectFormation::Irregular, _, _) => unreachable!(),
    }
}

fn imperative_ending(formation: ImperativeFormation, cell: ImperativeCell) -> &'static str {
    match (formation, cell.person, cell.number) {
        (_, Person::Second | Person::Third, Number::Singular) => "и",
        (ImperativeFormation::FirstUnpalatalized, Person::First, Number::Dual) => "ева",
        (ImperativeFormation::FirstUnpalatalized, Person::Second, Number::Dual) => "ита",
        (ImperativeFormation::FirstUnpalatalized, Person::First, Number::Plural) => "емъ",
        (ImperativeFormation::FirstUnpalatalized, Person::Second, Number::Plural) => "ите",
        (ImperativeFormation::ISeries, Person::First, Number::Dual) => "ива",
        (ImperativeFormation::ISeries, Person::Second, Number::Dual) => "ита",
        (ImperativeFormation::ISeries, Person::First, Number::Plural) => "имъ",
        (ImperativeFormation::ISeries, Person::Second, Number::Plural) => "ите",
        _ => unreachable!(),
    }
}

fn required<T>(value: Option<&T>, field: MetadataField) -> Result<&T> {
    value.ok_or(Error::MissingPrincipalPart { field })
}

fn join(stem: &str, ending: &str) -> String {
    let mut text = String::with_capacity(stem.len() + ending.len());
    text.push_str(stem);
    text.push_str(ending);
    text
}

fn palatalize_final_velar(stem: &str) -> String {
    let replacement = match stem.chars().last() {
        Some('к') => Some('ч'),
        Some('г') => Some('ж'),
        Some('х') => Some('ш'),
        _ => None,
    };
    if let Some(replacement) = replacement {
        let mut value = stem.to_owned();
        value.pop();
        value.push(replacement);
        value
    } else {
        stem.to_owned()
    }
}

fn normative(
    expanded: String,
    rule: &'static str,
    profile: OrthographyProfile,
    stage: &'static str,
    input: &str,
) -> Result<FormSet> {
    normative_variants(vec![expanded], rule, profile, stage, input)
}

fn normative_variants(
    expanded: Vec<String>,
    rule: &'static str,
    profile: OrthographyProfile,
    stage: &'static str,
    input: &str,
) -> Result<FormSet> {
    let rule_id = RuleId::from(rule);
    let evidence_id = EvidenceId::from(format!("normative:{rule}"));
    let evidence = Evidence {
        id: evidence_id.clone(),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: rule.into(),
        note: Some("Alypy section is encoded in the stable rule ID".into()),
    };
    let variants = expanded
        .into_iter()
        .map(|expanded| {
            let expanded = SynodalWord::parse(expanded)?.canonical().to_owned();
            let (accented, printed, warnings) = match profile {
                OrthographyProfile::Expanded => (None, expanded.clone(), Vec::new()),
                OrthographyProfile::ExpandedAccentless => {
                    let accentless = strip_presentation_marks(&expanded);
                    (
                        None,
                        accentless.clone(),
                        vec!["accent and breathing marks removed".into()],
                    )
                }
                OrthographyProfile::SynodalLiturgical => {
                    if !expanded.chars().any(is_accent_or_breathing) {
                        return Err(Error::OrthographicMetadataRequired {
                            field: MetadataField::AccentClass,
                        });
                    }
                    (Some(expanded.clone()), expanded.clone(), Vec::new())
                }
            };
            Ok(FormVariant {
                expanded: expanded.clone(),
                accented,
                printed: printed.clone(),
                romanization: None,
                source_recension: Some(Recension::SynodalRussian),
                target_recension: Recension::SynodalRussian,
                recension_mapping: None,
                confidence: Confidence::from_basis_points(9_500).unwrap_or(Confidence::CERTAIN),
                source: FormSource::SynodalNormativeGeneration {
                    rule: rule_id.clone(),
                },
                assumptions: vec![],
                evidence: vec![evidence.clone()],
                contradictions: vec![],
                warnings,
                rule_trace: RuleTrace::new(vec![TraceStep {
                    rule: rule_id.clone(),
                    stage: stage.into(),
                    input: input.into(),
                    output: printed,
                    source_recension: Some(Recension::SynodalRussian),
                    target_recension: Recension::SynodalRussian,
                    mapping: None,
                    evidence: vec![evidence_id.clone()],
                }]),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    FormSet::try_from_variants(variants)
}

fn strip_presentation_marks(value: &str) -> String {
    value
        .chars()
        .filter(|character| !is_accent_or_breathing(*character))
        .collect()
}

fn is_accent_or_breathing(character: char) -> bool {
    matches!(
        character,
        '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}' | '\u{0486}'
    )
}

/// Checks whether a generation policy may use the selected productive rule.
#[must_use]
pub const fn policy_allows_normative_rule(policy: GenerationPolicy) -> bool {
    matches!(
        policy,
        GenerationPolicy::Strict | GenerationPolicy::Productive | GenerationPolicy::Exploratory
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NounCell;

    fn word(value: &str) -> SynodalWord {
        SynodalWord::parse(value).expect("test spelling")
    }

    #[test]
    fn declines_first_hard_noun_from_alypy_34() {
        let lexeme = NounLexeme {
            lemma: word("рабъ"),
            stem: word("раб"),
            gender: Gender::Masculine,
            declension: NounDeclension::FirstHardMasculine,
        };
        let form = decline_noun(
            &lexeme,
            NounCell {
                case: Case::Dative,
                number: Number::Plural,
                animacy: Animacy::Animate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("supported form");
        assert_eq!(form.primary_text(), "рабомъ");
    }

    #[test]
    fn animate_accusatives_retain_alypy_35_variants_in_normative_order() {
        let lexeme = NounLexeme {
            lemma: word("рабъ"),
            stem: word("раб"),
            gender: Gender::Masculine,
            declension: NounDeclension::FirstHardMasculine,
        };
        let singular = decline_noun(
            &lexeme,
            NounCell {
                case: Case::Accusative,
                number: Number::Singular,
                animacy: Animacy::Animate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("supported singular");
        assert_eq!(
            singular
                .variants()
                .iter()
                .map(|variant| variant.printed.as_str())
                .collect::<Vec<_>>(),
            ["раба", "рабъ"]
        );

        let plural = decline_noun(
            &lexeme,
            NounCell {
                case: Case::Accusative,
                number: Number::Plural,
                animacy: Animacy::Animate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("supported plural");
        assert_eq!(
            plural
                .variants()
                .iter()
                .map(|variant| variant.printed.as_str())
                .collect::<Vec<_>>(),
            ["рабы", "рабовъ"]
        );
        assert!(
            plural
                .variants()
                .iter()
                .all(|variant| !variant.evidence.is_empty())
        );
    }

    #[test]
    fn declines_long_hard_adjective_from_alypy_57() {
        let lexeme = AdjectiveLexeme {
            lemma: word("мꙋдръ"),
            stem: word("мꙋдр"),
            class: AdjectiveClass::Hard,
        };
        let form = decline_adjective(
            &lexeme,
            AdjectiveCell {
                case: Case::Genitive,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Animate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            },
            OrthographyProfile::Expanded,
        )
        .expect("supported form");
        assert_eq!(form.primary_text(), "мꙋдрагѡ");
    }

    #[test]
    fn present_uses_independent_edge_principal_parts() {
        let lexeme = regular_verb();
        assert_eq!(
            present(
                &lexeme,
                Person::First,
                Number::Singular,
                OrthographyProfile::Expanded
            )
            .expect("first singular")
            .primary_text(),
            "несꙋ"
        );
        assert_eq!(
            present(
                &lexeme,
                Person::Third,
                Number::Plural,
                OrthographyProfile::Expanded
            )
            .expect("third plural")
            .primary_text(),
            "несꙋтъ"
        );
    }

    #[test]
    fn conjugates_consonant_aorist_from_alypy_86() {
        let lexeme = regular_verb();
        assert_eq!(
            aorist(
                &lexeme,
                Person::First,
                Number::Singular,
                OrthographyProfile::Expanded
            )
            .expect("aorist")
            .primary_text(),
            "несохъ"
        );
        assert_eq!(
            aorist(
                &lexeme,
                Person::Third,
                Number::Singular,
                OrthographyProfile::Expanded
            )
            .expect("aorist")
            .primary_text(),
            "несе"
        );
    }

    #[test]
    fn rejects_perfective_imperfect() {
        let mut lexeme = regular_verb();
        lexeme.aspect = Aspect::Perfective;
        assert!(matches!(
            imperfect(
                &lexeme,
                Person::Third,
                Number::Singular,
                OrthographyProfile::Expanded
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn liturgical_profile_requires_accent_metadata() {
        let lexeme = regular_verb();
        assert!(matches!(
            present(
                &lexeme,
                Person::Second,
                Number::Singular,
                OrthographyProfile::SynodalLiturgical
            ),
            Err(Error::OrthographicMetadataRequired { .. })
        ));
    }

    fn regular_verb() -> VerbLexeme {
        VerbLexeme {
            lemma: word("нести"),
            aspect: Aspect::Imperfective,
            conjugation: VerbConjugation::FirstUnpalatalized,
            present_stem: Some(word("нес")),
            present_first_singular: Some(word("несꙋ")),
            present_third_plural: Some(word("несꙋтъ")),
            imperfect_stem: Some(word("нес")),
            imperfect_formation: Some(ImperfectFormation::Yah),
            aorist_stem: Some(word("нес")),
            aorist_formation: Some(AoristFormation::ConsonantStem),
            imperative_stem: Some(word("нес")),
            imperative_formation: Some(ImperativeFormation::FirstUnpalatalized),
            l_participle_stem: Some(word("нес")),
        }
    }
}
