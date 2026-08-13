//! Productive Synodal rules admitted from Alypy (Gamanovich) §§33–44, 53,
//! 57, 79–80, 86–87, 93, and 97.

use crate::{
    AdjectiveCell, AdjectiveForm, Animacy, AuthorityRole, Case, Comparison, Confidence,
    EpistemicRole, Error, Evidence, EvidenceId, EvidenceKind, FiniteTense, FiniteVerbCell, FormSet,
    FormSource, FormVariant, Gender, GenerationPolicy, ImperativeCell, LParticipleCell,
    MetadataField, Number, OrthographyProfile, ParticipleCell, ParticipleTense, ParticipleVoice,
    Person, Recension, Result, RuleId, RuleTrace, SourceId, SynodalWord, TraceStep, VerbSystem,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NounDeclension {
    FirstHardMasculine,
    /// First declension with a final velar and the reviewed first/second
    /// palatalizations of Alypy §34.
    FirstHardVelarMasculine,
    /// First-declension masculine with a sibilant stem and mixed endings.
    FirstMixedMasculine,
    FirstHardNeuter,
    FirstSoftMasculine,
    FirstSoftNeuter,
    SecondHard,
    SecondSoft,
    ThirdFeminine,
    ThirdMasculine,
    /// Fourth-declension neuter whose citation form in `-ѧ` has an oblique
    /// stem in `-ен-`, for example `имѧ : имен-`.
    FourthNeuterEn,
    /// Fourth-declension neuter whose citation form in `-о` has an oblique
    /// stem in `-ес-`, for example `небо : небес-`.
    FourthNeuterEs,
    /// Fourth-declension neuter with an independently supplied extended stem
    /// in `-ат-`, for example `ѻтроча : ѻтрочат-`.
    FourthNeuterAt,
    /// Fourth-declension feminine whose citation form in `-и` has an oblique
    /// stem in `-ер-`, for example `мати : матер-`.
    FourthFeminineEr,
    /// Fourth-declension feminine with an independently supplied oblique stem
    /// in `-ов-` or `-в-`, for example `свекры : свекров-`.
    FourthFeminineOv,
    /// Fourth-declension masculine with an independently supplied stem in
    /// `-ен-`, for example `степень : степен-`.
    FourthMasculineEn,
    /// The lexeme-specific `камень` contract: the ordinary masculine `-ен-`
    /// paradigm plus only the alternatives cited in Alypy §43. The separate
    /// collective `каменїе` is never emitted by this contract.
    FourthMasculineEnKamen,
}

impl NounDeclension {
    pub const ALL: [Self; 17] = [
        Self::FirstHardMasculine,
        Self::FirstHardVelarMasculine,
        Self::FirstMixedMasculine,
        Self::FirstHardNeuter,
        Self::FirstSoftMasculine,
        Self::FirstSoftNeuter,
        Self::SecondHard,
        Self::SecondSoft,
        Self::ThirdFeminine,
        Self::ThirdMasculine,
        Self::FourthNeuterEn,
        Self::FourthNeuterEs,
        Self::FourthNeuterAt,
        Self::FourthFeminineEr,
        Self::FourthFeminineOv,
        Self::FourthMasculineEn,
        Self::FourthMasculineEnKamen,
    ];
}

/// Numbers in which a noun is lexically licensed. This is lexical metadata,
/// not a request filter: asking for an absent number returns a historical-cell
/// error and remains visible in a complete paradigm.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NounNumberInventory {
    #[default]
    All,
    SingularOnly,
    DualOnly,
    PluralOnly,
    SingularAndDual,
    SingularAndPlural,
    DualAndPlural,
}

impl NounNumberInventory {
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NounLexeme {
    pub lemma: SynodalWord,
    /// Productive stem. For fourth-declension classes this is the independently
    /// supplied extended oblique stem, not a stem inferred from the citation.
    pub stem: SynodalWord,
    pub gender: Gender,
    pub declension: NounDeclension,
    pub number_inventory: NounNumberInventory,
}

impl NounLexeme {
    #[must_use]
    pub const fn new(
        lemma: SynodalWord,
        stem: SynodalWord,
        gender: Gender,
        declension: NounDeclension,
    ) -> Self {
        Self {
            lemma,
            stem,
            gender,
            declension,
            number_inventory: NounNumberInventory::All,
        }
    }

    #[must_use]
    pub const fn with_number_inventory(mut self, inventory: NounNumberInventory) -> Self {
        self.number_inventory = inventory;
        self
    }
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
    /// Fully specified stem before comparison endings (for example `мꙋдрѣйш`).
    pub comparative_stem: Option<SynodalWord>,
    /// Formation of the independently supplied comparison stem. This is
    /// required only for short comparison, whose masculine and neuter
    /// nominative edges delete part of the comparison suffix (Alypy §58).
    pub comparison_formation: Option<ComparisonFormation>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ComparisonFormation {
    AncientHard,
    AncientSoft,
    LaterYat,
    LaterAi,
}

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

/// The three independently reviewed inputs required by a complete productive
/// present system. Keeping the edge forms explicit prevents a generic stem
/// template from inventing lexical consonant alternations or ending series.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PresentPrincipalParts {
    pub stem: SynodalWord,
    pub first_singular: SynodalWord,
    pub third_plural: SynodalWord,
}

impl PresentPrincipalParts {
    #[must_use]
    pub const fn new(
        stem: SynodalWord,
        first_singular: SynodalWord,
        third_plural: SynodalWord,
    ) -> Self {
        Self {
            stem,
            first_singular,
            third_plural,
        }
    }

    pub fn parse(
        stem: impl Into<String>,
        first_singular: impl Into<String>,
        third_plural: impl Into<String>,
    ) -> Result<Self> {
        Ok(Self::new(
            SynodalWord::parse(stem)?,
            SynodalWord::parse(first_singular)?,
            SynodalWord::parse(third_plural)?,
        ))
    }
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
    pub present_active_participle: Option<ParticiplePrincipalPart>,
    pub past_active_participle: Option<ParticiplePrincipalPart>,
    pub present_passive_participle: Option<ParticiplePrincipalPart>,
    pub past_passive_participle: Option<ParticiplePrincipalPart>,
    pub verbal_noun: Option<(SynodalWord, NounDeclension, Gender)>,
}

impl VerbLexeme {
    /// Reports principal parts absent from the productive background for one
    /// system. Exact overrides can still satisfy individual registered cells.
    #[must_use]
    pub fn missing_principal_parts(&self, system: VerbSystem) -> Vec<MetadataField> {
        let mut missing = Vec::new();
        match system {
            VerbSystem::Finite(crate::FiniteTense::Present) => {
                if self.present_stem.is_none() {
                    missing.push(MetadataField::PresentStem);
                }
                if self.present_first_singular.is_none() {
                    missing.push(MetadataField::PresentFirstSingular);
                }
                if self.present_third_plural.is_none() {
                    missing.push(MetadataField::PresentThirdPlural);
                }
            }
            VerbSystem::Finite(crate::FiniteTense::Imperfect) => {
                if self.imperfect_stem.is_none() {
                    missing.push(MetadataField::ImperfectStem);
                }
                if self.imperfect_formation.is_none() {
                    missing.push(MetadataField::ImperfectFormation);
                }
            }
            VerbSystem::Finite(crate::FiniteTense::Aorist) => {
                if self.aorist_stem.is_none() {
                    missing.push(MetadataField::AoristStem);
                }
                if self.aorist_formation.is_none() {
                    missing.push(MetadataField::AoristFormation);
                }
            }
            VerbSystem::Finite(crate::FiniteTense::Future | crate::FiniteTense::Past)
            | VerbSystem::Infinitive => {}
            VerbSystem::Imperative => {
                if self.imperative_stem.is_none() {
                    missing.push(MetadataField::ImperativeStem);
                }
                if self.imperative_formation.is_none() {
                    missing.push(MetadataField::ImperativeFormation);
                }
            }
            VerbSystem::LParticiple => {
                if self.l_participle_stem.is_none() {
                    missing.push(MetadataField::LParticipleStem);
                }
            }
            VerbSystem::Participle { tense, voice, form } => {
                let part = match (tense, voice) {
                    (ParticipleTense::Present, ParticipleVoice::Active) => {
                        self.present_active_participle.as_ref()
                    }
                    (ParticipleTense::Past, ParticipleVoice::Active) => {
                        self.past_active_participle.as_ref()
                    }
                    (ParticipleTense::Present, ParticipleVoice::Passive) => {
                        self.present_passive_participle.as_ref()
                    }
                    (ParticipleTense::Past, ParticipleVoice::Passive) => {
                        self.past_passive_participle.as_ref()
                    }
                };
                let has_requested_stem = part.is_some_and(|part| match form {
                    crate::AdjectiveForm::Short => part.short_stem.is_some(),
                    crate::AdjectiveForm::Long => part.long_stem.is_some(),
                });
                if !has_requested_stem {
                    missing.push(MetadataField::ParticipleStem);
                }
                if voice == ParticipleVoice::Active
                    && form == crate::AdjectiveForm::Short
                    && part.is_some_and(|part| {
                        part.short_stem.is_some() && part.short_formation.is_none()
                    })
                {
                    missing.push(MetadataField::ParticipleFormation);
                }
            }
            VerbSystem::Supine => missing.push(MetadataField::SupineStem),
            VerbSystem::VerbalNoun { .. } => {
                if self.verbal_noun.is_none() {
                    missing.push(MetadataField::VerbalNounStem);
                }
            }
        }
        missing
    }
}

pub fn decline_noun(
    lexeme: &NounLexeme,
    cell: crate::NounCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    validate_noun_lexeme(lexeme)?;
    if !lexeme.number_inventory.contains(cell.number) {
        return Err(Error::HistoricallyInvalidCell {
            reason: format!(
                "noun {:?} is not licensed in {:?}",
                lexeme.lemma.canonical(),
                cell.number
            ),
        });
    }
    let mut expanded = noun_surfaces(lexeme, cell)?;
    if cell.case == Case::Accusative && cell.animacy == Animacy::Animate {
        let nominative_like = noun_surfaces(
            lexeme,
            crate::NounCell {
                animacy: Animacy::Inanimate,
                ..cell
            },
        )?;
        if cell.number == Number::Plural
            && !matches!(
                lexeme.declension,
                NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineOv
            )
        {
            let mut ordered = nominative_like;
            ordered.extend(expanded);
            expanded = ordered;
        } else {
            for form in nominative_like {
                if !expanded.contains(&form) {
                    expanded.push(form);
                }
            }
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

fn noun_surfaces(lexeme: &NounLexeme, cell: crate::NounCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Nominative as Nom, Vocative as Voc};
    use Number::Singular as Sg;

    let citation_form = matches!(
        (lexeme.declension, cell.number, cell.case),
        (
            NounDeclension::FourthNeuterEn
                | NounDeclension::FourthNeuterEs
                | NounDeclension::FourthNeuterAt,
            Sg,
            Nom | Acc | Voc
        ) | (
            NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineOv,
            Sg,
            Nom | Voc
        ) | (
            NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen,
            Sg,
            Nom | Voc
        )
    );
    if citation_form {
        return Ok(vec![lexeme.lemma.canonical().to_owned()]);
    }

    let stem = noun_stem(lexeme, cell);
    let mut surfaces = noun_endings(lexeme, cell)?
        .into_iter()
        .map(|ending| join(&stem, ending))
        .collect::<Vec<_>>();
    if lexeme.declension == NounDeclension::FourthMasculineEnKamen {
        let lexical_stem = lexeme.stem.canonical();
        let alternative = match (cell.number, cell.case, cell.animacy) {
            (Sg, crate::Case::Genitive, _) => Some(join(lexical_stem, "ѧ")),
            (Sg, crate::Case::Dative, _) => Some(join(lexical_stem, "ю")),
            (Number::Plural, Nom | Voc | Acc, Animacy::Inanimate) => Some(join(lexical_stem, "їѧ")),
            (Number::Plural, crate::Case::Locative, _) => Some(join(lexical_stem, "їѧхъ")),
            _ => None,
        };
        if let Some(alternative) = alternative {
            surfaces.push(alternative);
        }
    }
    Ok(surfaces)
}

fn noun_stem(lexeme: &NounLexeme, cell: crate::NounCell) -> String {
    use Case::{Accusative as Acc, Nominative as Nom, Vocative as Voc};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};

    let stem = lexeme.stem.canonical();
    match lexeme.declension {
        NounDeclension::FirstHardVelarMasculine => match (cell.number, cell.case) {
            (Sg, Voc) => palatalize_final_velar(stem),
            (Sg, crate::Case::Locative) | (Pl, Nom | Voc | crate::Case::Locative) => {
                second_palatalize_final_velar(stem)
            }
            _ => stem.to_owned(),
        },
        NounDeclension::FourthNeuterEn | NounDeclension::FourthNeuterEs
            if matches!((cell.number, cell.case), (Du, Nom | Acc | Voc)) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FourthNeuterAt
            if matches!((cell.number, cell.case), (Du, Nom | Acc | Voc)) =>
        {
            last_o_as_omega(stem)
        }
        NounDeclension::FourthFeminineEr
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, Nom | Voc)
            ) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FourthFeminineOv
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, Nom | Voc)
            ) =>
        {
            last_o_as_omega(stem)
        }
        NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen
            if matches!(
                (cell.number, cell.case),
                (Du, Nom | Acc | Voc) | (Pl, Nom | Voc)
            ) || matches!(
                (cell.number, cell.case, cell.animacy),
                (Pl, Acc, Animacy::Inanimate)
            ) =>
        {
            last_e_as_wide_e(stem)
        }
        _ => stem.to_owned(),
    }
}

pub fn decline_adjective(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if cell.form == AdjectiveForm::Short && cell.comparison == Comparison::Comparative {
        let stem = lexeme
            .comparative_stem
            .as_ref()
            .ok_or(Error::MissingPrincipalPart {
                field: MetadataField::ComparisonStem,
            })?;
        let formation = lexeme.comparison_formation.ok_or(Error::MissingMetadata {
            field: MetadataField::ComparisonFormation,
        })?;
        return decline_short_comparison(lexeme, stem, formation, cell, profile);
    }
    if cell.form == AdjectiveForm::Short && cell.comparison == Comparison::Superlative {
        return Err(Error::UnsupportedFormation {
            formation: "short superlative requires an independently reviewed Synodal formation"
                .into(),
        });
    }
    let (stem, ending, rule) = match cell.comparison {
        Comparison::Positive => (
            &lexeme.stem,
            match cell.form {
                AdjectiveForm::Short => short_adjective_ending(lexeme.class, cell)?,
                AdjectiveForm::Long => long_adjective_ending(lexeme.class, cell)?,
            },
            match (lexeme.class, cell.form) {
                (AdjectiveClass::Hard, AdjectiveForm::Short) => "SYN-ADJ-SHORT-HARD-ALYPY-53",
                (AdjectiveClass::Soft, AdjectiveForm::Short) => "SYN-ADJ-SHORT-SOFT-ALYPY-53",
                (AdjectiveClass::Hard, AdjectiveForm::Long) => "SYN-ADJ-LONG-HARD-ALYPY-57",
                (AdjectiveClass::Soft, AdjectiveForm::Long) => "SYN-ADJ-LONG-SOFT-ALYPY-57",
            },
        ),
        Comparison::Comparative | Comparison::Superlative => {
            let stem = lexeme
                .comparative_stem
                .as_ref()
                .ok_or(Error::MissingPrincipalPart {
                    field: MetadataField::ComparisonStem,
                })?;
            (
                stem,
                comparison_long_adjective_ending(cell)?,
                match cell.comparison {
                    Comparison::Comparative => "SYN-ADJ-COMPARATIVE-LONG-ALYPY-58",
                    Comparison::Superlative => "SYN-ADJ-SUPERLATIVE-LONG-ALYPY-59",
                    Comparison::Positive => unreachable!(),
                },
            )
        }
    };
    let mut expanded = vec![join(stem.canonical(), ending)];
    if cell.case == Case::Accusative && cell.animacy == Animacy::Animate {
        let nominative_cell = AdjectiveCell {
            animacy: Animacy::Inanimate,
            ..cell
        };
        let nominative_ending = match cell.comparison {
            Comparison::Positive => match cell.form {
                AdjectiveForm::Short => short_adjective_ending(lexeme.class, nominative_cell)?,
                AdjectiveForm::Long => long_adjective_ending(lexeme.class, nominative_cell)?,
            },
            Comparison::Comparative | Comparison::Superlative => {
                comparison_long_adjective_ending(nominative_cell)?
            }
        };
        let nominative_like = join(stem.canonical(), nominative_ending);
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
    let formation = lexeme.aorist_formation.ok_or(Error::MissingMetadata {
        field: MetadataField::AoristFormation,
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
    if lexeme.aspect == Aspect::Unknown {
        return Err(Error::MissingMetadata {
            field: MetadataField::Aspect,
        });
    }
    if lexeme.aspect == Aspect::Perfective {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §87 restricts the productive imperfect to imperfective verbs".into(),
        });
    }
    let formation = lexeme.imperfect_formation.ok_or(Error::MissingMetadata {
        field: MetadataField::ImperfectFormation,
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
    let formation = lexeme.imperative_formation.ok_or(Error::MissingMetadata {
        field: MetadataField::ImperativeFormation,
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
    decline_adjectival_stem(
        stem,
        principal_part.class,
        cell.agreement,
        rule,
        "participle-declension",
        lexeme.lemma.canonical(),
        profile,
    )
}

fn decline_short_comparison(
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
        "SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-98",
        profile,
        "short-comparison-declension",
        lexeme.lemma.canonical(),
    )
}

fn decline_short_active_participle(
    lexeme: &VerbLexeme,
    stem: &SynodalWord,
    formation: ActiveParticipleShortFormation,
    cell: ParticipleCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let citation =
        active_participle_citation_variants(stem.canonical(), formation, cell.agreement.gender)?;
    let expanded = decline_short_comparison_stem(stem.canonical(), cell.agreement, citation)?;
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

fn decline_short_comparison_stem(
    stem: &str,
    cell: AdjectiveCell,
    citation: Option<Vec<String>>,
) -> Result<Vec<String>> {
    if cell.case == Case::Vocative {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §98 gives no vocative in the short-comparison declension".into(),
        });
    }
    if cell.number == Number::Singular && cell.case == Case::Nominative {
        return citation.ok_or_else(|| Error::ContradictoryMetadata {
            reason: "short comparison citation variants are missing".into(),
        });
    }
    let primary = join(stem, short_comparison_ending(cell)?);
    let mut variants = vec![primary];
    if cell.case == Case::Accusative
        && cell.animacy == Animacy::Animate
        && (cell.gender == Gender::Masculine
            || (cell.gender == Gender::Feminine && cell.number == Number::Plural))
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
        && cell.case == Case::Nominative
        && cell.gender == Gender::Feminine
    {
        variants.push(join(stem, "е"));
    }
    Ok(variants)
}

fn short_comparison_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    Ok(match (cell.number, cell.gender, cell.case) {
        (Sg, _, Nom) => {
            return Err(Error::ContradictoryMetadata {
                reason: "short comparison nominatives require a typed citation edge".into(),
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
                reason: "short comparison has no vocative cell".into(),
            });
        }
    })
}

fn comparison_citation_variants(
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

fn comparison_edge_without_suffix(
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

fn active_participle_citation_variants(
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

fn decline_adjectival_stem(
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

/// Validates the closed class/gender contract and the independently supplied
/// stem shape required by productive alternation rules.
pub fn validate_noun_lexeme(lexeme: &NounLexeme) -> Result<()> {
    let valid = matches!(
        (lexeme.declension, lexeme.gender),
        (NounDeclension::FirstHardMasculine, Gender::Masculine)
            | (NounDeclension::FirstHardVelarMasculine, Gender::Masculine)
            | (NounDeclension::FirstMixedMasculine, Gender::Masculine)
            | (NounDeclension::FirstHardNeuter, Gender::Neuter)
            | (NounDeclension::FirstSoftMasculine, Gender::Masculine)
            | (NounDeclension::FirstSoftNeuter, Gender::Neuter)
            | (
                NounDeclension::SecondHard | NounDeclension::SecondSoft,
                Gender::Feminine
            )
            | (NounDeclension::ThirdFeminine, Gender::Feminine)
            | (NounDeclension::ThirdMasculine, Gender::Masculine)
            | (
                NounDeclension::FourthNeuterEn
                    | NounDeclension::FourthNeuterEs
                    | NounDeclension::FourthNeuterAt,
                Gender::Neuter
            )
            | (
                NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineOv,
                Gender::Feminine
            )
            | (
                NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen,
                Gender::Masculine
            )
    );
    if !valid {
        return Err(Error::ContradictoryMetadata {
            reason: "declension and lexical gender are incompatible".into(),
        });
    }
    let lemma = lexeme.lemma.canonical();
    let stem = lexeme.stem.canonical();
    let valid_shape = match lexeme.declension {
        NounDeclension::FirstHardVelarMasculine => {
            lemma.ends_with('ъ')
                && stem
                    .chars()
                    .last()
                    .is_some_and(|final_char| matches!(final_char, 'г' | 'к' | 'х'))
        }
        NounDeclension::FirstMixedMasculine => {
            lemma.ends_with('ъ')
                && stem
                    .chars()
                    .last()
                    .is_some_and(|final_char| matches!(final_char, 'ж' | 'ч' | 'ш' | 'щ' | 'ц'))
        }
        NounDeclension::FourthNeuterEn => lemma.ends_with('ѧ') && stem.ends_with("ен"),
        NounDeclension::FourthNeuterEs => lemma.ends_with('о') && stem.ends_with("ес"),
        NounDeclension::FourthNeuterAt => {
            (lemma.ends_with('а') || lemma.ends_with('ѧ')) && stem.ends_with("ат")
        }
        NounDeclension::FourthFeminineEr => lemma.ends_with('и') && stem.ends_with("ер"),
        NounDeclension::FourthFeminineOv => {
            (lemma.ends_with('ы') || lemma.ends_with('ь'))
                && (stem.ends_with("ов") || stem.ends_with('в'))
                && !matches!(lemma, "любовь" | "любы")
        }
        NounDeclension::FourthMasculineEn => {
            lemma.ends_with("ень") && stem.ends_with("ен") && lemma != "камень"
        }
        NounDeclension::FourthMasculineEnKamen => lemma == "камень" && stem == "камен",
        _ => true,
    };
    if !valid_shape {
        return Err(Error::ContradictoryMetadata {
            reason: format!(
                "lemma {lemma:?} and stem {stem:?} do not satisfy {:?}",
                lexeme.declension
            ),
        });
    }
    Ok(())
}

fn noun_rule(declension: NounDeclension) -> &'static str {
    match declension {
        NounDeclension::FirstHardMasculine => "SYN-NOUN-I-HARD-M-ALYPY-34",
        NounDeclension::FirstHardVelarMasculine => "SYN-NOUN-I-HARD-VELAR-M-ALYPY-34",
        NounDeclension::FirstMixedMasculine => "SYN-NOUN-I-MIXED-M-ALYPY-33-34",
        NounDeclension::FirstHardNeuter => "SYN-NOUN-I-HARD-N-ALYPY-34",
        NounDeclension::FirstSoftMasculine => "SYN-NOUN-I-SOFT-M-ALYPY-34",
        NounDeclension::FirstSoftNeuter => "SYN-NOUN-I-SOFT-N-ALYPY-34",
        NounDeclension::SecondHard => "SYN-NOUN-II-HARD-ALYPY-39",
        NounDeclension::SecondSoft => "SYN-NOUN-II-SOFT-ALYPY-39",
        NounDeclension::ThirdFeminine => "SYN-NOUN-III-F-ALYPY-41",
        NounDeclension::ThirdMasculine => "SYN-NOUN-III-M-ALYPY-41",
        NounDeclension::FourthNeuterEn => "SYN-NOUN-IV-N-EN-ALYPY-42-43",
        NounDeclension::FourthNeuterEs => "SYN-NOUN-IV-N-ES-ALYPY-42-43",
        NounDeclension::FourthNeuterAt => "SYN-NOUN-IV-N-AT-ALYPY-42-43",
        NounDeclension::FourthFeminineEr => "SYN-NOUN-IV-F-ER-ALYPY-42-43",
        NounDeclension::FourthFeminineOv => "SYN-NOUN-IV-F-OV-ALYPY-42-44",
        NounDeclension::FourthMasculineEn => "SYN-NOUN-IV-M-EN-ALYPY-42-44",
        NounDeclension::FourthMasculineEnKamen => "SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43",
    }
}

fn noun_endings(lexeme: &NounLexeme, cell: crate::NounCell) -> Result<Vec<&'static str>> {
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

        (NounDeclension::FirstHardVelarMasculine, Sg, Nom) => "ъ",
        (NounDeclension::FirstHardVelarMasculine, Sg, Gen) => "а",
        (NounDeclension::FirstHardVelarMasculine, Sg, Dat) => "ꙋ",
        (NounDeclension::FirstHardVelarMasculine, Sg, Acc) => animate_acc("ъ", "а"),
        (NounDeclension::FirstHardVelarMasculine, Sg, Ins) => "омъ",
        (NounDeclension::FirstHardVelarMasculine, Sg, Loc) => "ѣ",
        (NounDeclension::FirstHardVelarMasculine, Sg, Voc) => "е",
        (NounDeclension::FirstHardVelarMasculine, Du, Nom | Acc | Voc) => "а",
        (NounDeclension::FirstHardVelarMasculine, Du, Gen | Loc) => "ꙋ",
        (NounDeclension::FirstHardVelarMasculine, Du, Dat | Ins) => "ома",
        (NounDeclension::FirstHardVelarMasculine, Pl, Nom | Voc) => {
            if lexeme.stem.canonical().ends_with('к') {
                "ы"
            } else {
                "и"
            }
        }
        (NounDeclension::FirstHardVelarMasculine, Pl, Gen) => "овъ",
        (NounDeclension::FirstHardVelarMasculine, Pl, Dat) => "омъ",
        (NounDeclension::FirstHardVelarMasculine, Pl, Acc) => animate_acc("и", "овъ"),
        (NounDeclension::FirstHardVelarMasculine, Pl, Ins) => "и",
        (NounDeclension::FirstHardVelarMasculine, Pl, Loc) => "ѣхъ",

        (NounDeclension::FirstMixedMasculine, Sg, Nom) => "ъ",
        (NounDeclension::FirstMixedMasculine, Sg, Gen) => "а",
        (NounDeclension::FirstMixedMasculine, Sg, Dat) => "ꙋ",
        (NounDeclension::FirstMixedMasculine, Sg, Acc) => animate_acc("ъ", "а"),
        (NounDeclension::FirstMixedMasculine, Sg, Ins) => "емъ",
        (NounDeclension::FirstMixedMasculine, Sg, Loc) => "и",
        (NounDeclension::FirstMixedMasculine, Sg, Voc) => "ꙋ",
        (NounDeclension::FirstMixedMasculine, Du, Nom | Acc | Voc) => "а",
        (NounDeclension::FirstMixedMasculine, Du, Gen | Loc) => "ꙋ",
        (NounDeclension::FirstMixedMasculine, Du, Dat | Ins) => "ема",
        (NounDeclension::FirstMixedMasculine, Pl, Nom | Voc) => "и",
        (NounDeclension::FirstMixedMasculine, Pl, Gen) => "ей",
        (NounDeclension::FirstMixedMasculine, Pl, Dat) => "емъ",
        (NounDeclension::FirstMixedMasculine, Pl, Acc) => animate_acc("ы", "ей"),
        (NounDeclension::FirstMixedMasculine, Pl, Ins) => "ы",
        (NounDeclension::FirstMixedMasculine, Pl, Loc) => "ахъ",

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

        (NounDeclension::FirstSoftMasculine, Sg, Nom) => "ь",
        (NounDeclension::FirstSoftMasculine, Sg, Gen) => "ѧ",
        (NounDeclension::FirstSoftMasculine, Sg, Dat) => "ю",
        (NounDeclension::FirstSoftMasculine, Sg, Acc) => animate_acc("ь", "ѧ"),
        (NounDeclension::FirstSoftMasculine, Sg, Ins) => "емъ",
        (NounDeclension::FirstSoftMasculine, Sg, Loc) => "и",
        (NounDeclension::FirstSoftMasculine, Sg, Voc) => "ю",
        (NounDeclension::FirstSoftMasculine, Du, Nom | Acc | Voc) => "ѧ",
        (NounDeclension::FirstSoftMasculine, Du, Gen | Loc) => "ю",
        (NounDeclension::FirstSoftMasculine, Du, Dat | Ins) => "ема",
        (NounDeclension::FirstSoftMasculine, Pl, Nom | Voc) => "и",
        (NounDeclension::FirstSoftMasculine, Pl, Gen) => "ей",
        (NounDeclension::FirstSoftMasculine, Pl, Dat) => "емъ",
        (NounDeclension::FirstSoftMasculine, Pl, Acc) => animate_acc("и", "ей"),
        (NounDeclension::FirstSoftMasculine, Pl, Ins) => "и",
        (NounDeclension::FirstSoftMasculine, Pl, Loc) => "ехъ",

        (NounDeclension::FirstSoftNeuter, Sg, Nom | Acc | Voc) => "е",
        (NounDeclension::FirstSoftNeuter, Sg, Gen) => "ѧ",
        (NounDeclension::FirstSoftNeuter, Sg, Dat) => "ю",
        (NounDeclension::FirstSoftNeuter, Sg, Ins) => "емъ",
        (NounDeclension::FirstSoftNeuter, Sg, Loc) => "и",
        (NounDeclension::FirstSoftNeuter, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::FirstSoftNeuter, Du, Gen | Loc) => "ю",
        (NounDeclension::FirstSoftNeuter, Du, Dat | Ins) => "ема",
        (NounDeclension::FirstSoftNeuter, Pl, Nom | Acc | Voc) => "ѧ",
        (NounDeclension::FirstSoftNeuter, Pl, Gen) => "ей",
        (NounDeclension::FirstSoftNeuter, Pl, Dat) => "емъ",
        (NounDeclension::FirstSoftNeuter, Pl, Ins) => "и",
        (NounDeclension::FirstSoftNeuter, Pl, Loc) => "ѧхъ",

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

        (NounDeclension::ThirdMasculine, Sg, Nom | Acc) => "ь",
        (NounDeclension::ThirdMasculine, Sg, Gen | Dat | Loc) => "и",
        (NounDeclension::ThirdMasculine, Sg, Ins) => "емъ",
        (NounDeclension::ThirdMasculine, Sg, Voc) => "ь",
        (NounDeclension::ThirdMasculine, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::ThirdMasculine, Du, Gen | Loc) => "їю",
        (NounDeclension::ThirdMasculine, Du, Dat | Ins) => "ьма",
        (NounDeclension::ThirdMasculine, Pl, Nom | Voc) => "їе",
        (NounDeclension::ThirdMasculine, Pl, Gen) => "ій",
        (NounDeclension::ThirdMasculine, Pl, Dat) => "ємъ",
        (NounDeclension::ThirdMasculine, Pl, Acc) => animate_acc("и", "ій"),
        (NounDeclension::ThirdMasculine, Pl, Ins) => "ьми",
        (NounDeclension::ThirdMasculine, Pl, Loc) => "ехъ",

        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Sg,
            Gen,
        ) => "е",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Sg,
            Dat | Loc,
        ) => "и",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Sg,
            Ins,
        ) => "емъ",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Du,
            Nom | Acc | Voc,
        ) => "и",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Du,
            Gen | Loc,
        ) => "ꙋ",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Du,
            Dat | Ins,
        ) => "ема",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Pl,
            Nom | Acc | Voc,
        ) => "а",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Pl,
            Gen,
        ) => "ъ",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Pl,
            Dat,
        ) => "ємъ",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Pl,
            Ins,
        ) => "ы",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Pl,
            Loc,
        ) => "ѣхъ",

        (NounDeclension::FourthFeminineEr, Sg, Gen) => "е",
        (NounDeclension::FourthFeminineEr, Sg, Dat | Loc) => "и",
        (NounDeclension::FourthFeminineEr, Sg, Acc) => "ь",
        (NounDeclension::FourthFeminineEr, Sg, Ins) => "їю",
        (NounDeclension::FourthFeminineEr, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::FourthFeminineEr, Du, Gen | Loc) => "їю",
        (NounDeclension::FourthFeminineEr, Du, Dat | Ins) => "ема",
        (NounDeclension::FourthFeminineEr, Pl, Nom | Voc) => "и",
        (NounDeclension::FourthFeminineEr, Pl, Gen) => "їй",
        (NounDeclension::FourthFeminineEr, Pl, Dat) => "емъ",
        (NounDeclension::FourthFeminineEr, Pl, Acc) => animate_acc("и", "ей"),
        (NounDeclension::FourthFeminineEr, Pl, Ins) => "ьми",
        (NounDeclension::FourthFeminineEr, Pl, Loc) => "ехъ",

        (NounDeclension::FourthFeminineOv, Sg, Gen) => "е",
        (NounDeclension::FourthFeminineOv, Sg, Dat | Loc) => "и",
        (NounDeclension::FourthFeminineOv, Sg, Acc) => "ь",
        (NounDeclension::FourthFeminineOv, Sg, Ins) => "їю",
        (NounDeclension::FourthFeminineOv, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::FourthFeminineOv, Du, Gen | Loc) => "їю",
        (NounDeclension::FourthFeminineOv, Du, Dat | Ins) => "ама",
        (NounDeclension::FourthFeminineOv, Pl, Nom | Voc) => "и",
        (NounDeclension::FourthFeminineOv, Pl, Gen) => "ей",
        (NounDeclension::FourthFeminineOv, Pl, Dat) => "амъ",
        (NounDeclension::FourthFeminineOv, Pl, Acc) => animate_acc("и", "ей"),
        (NounDeclension::FourthFeminineOv, Pl, Ins) => "ами",
        (NounDeclension::FourthFeminineOv, Pl, Loc) => "ахъ",

        (NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen, Sg, Gen) => {
            "е"
        }
        (
            NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen,
            Sg,
            Dat | Loc,
        ) => "и",
        (NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen, Sg, Acc) => {
            animate_acc("ь", "е")
        }
        (NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen, Sg, Ins) => {
            "емъ"
        }
        (
            NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen,
            Du,
            Nom | Acc | Voc,
        ) => "и",
        (
            NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen,
            Du,
            Gen | Loc,
        ) => "ꙋ",
        (
            NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen,
            Du,
            Dat | Ins,
        ) => "ьма",
        (
            NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen,
            Pl,
            Nom | Voc,
        ) => "и",
        (NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen, Pl, Gen) => {
            "їй"
        }
        (NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen, Pl, Dat) => {
            "ємъ"
        }
        (NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen, Pl, Acc) => {
            animate_acc("и", "їй")
        }
        (NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen, Pl, Ins) => {
            "ьми"
        }
        (NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen, Pl, Loc) => {
            "ехъ"
        }

        // Citation forms of fourth-declension nouns are emitted directly from
        // the independently supplied lemma before this table is consulted.
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterAt,
            Sg,
            Nom | Acc | Voc,
        )
        | (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineOv, Sg, Nom | Voc)
        | (
            NounDeclension::FourthMasculineEn | NounDeclension::FourthMasculineEnKamen,
            Sg,
            Nom | Voc,
        ) => {
            return Err(Error::UnsupportedCell {
                reason: "fourth-declension citation cells must be emitted from the supplied lemma"
                    .into(),
            });
        }
    };
    let mut endings = vec![ending];
    match (lexeme.declension, cell.number, cell.case) {
        (NounDeclension::FirstMixedMasculine, Pl, Nom | Voc) => endings.push("їе"),
        (NounDeclension::ThirdFeminine, Du, Dat | Ins) => endings.push("ьма"),
        (NounDeclension::ThirdMasculine, Sg, Voc) => endings.push("ю"),
        (NounDeclension::ThirdMasculine, Pl, Gen) => endings.push("ей"),
        (NounDeclension::FourthNeuterEn, Du, Dat | Ins) => endings.push("ама"),
        (NounDeclension::FourthNeuterEn, Pl, Dat) => endings.push("ѡмъ"),
        (NounDeclension::FourthNeuterAt, Du, Dat | Ins) => endings.push("ама"),
        (NounDeclension::FourthNeuterAt, Pl, Dat) => endings.push("ѡмъ"),
        (NounDeclension::FourthFeminineEr, Pl, Gen) => endings.push("ей"),
        (NounDeclension::FourthFeminineEr, Pl, Acc) if cell.animacy == Animacy::Animate => {
            endings.push("и");
        }
        (NounDeclension::FourthFeminineOv, Pl, Acc) if cell.animacy == Animacy::Animate => {
            endings.push("и");
        }
        (NounDeclension::FourthMasculineEnKamen, Du, Dat | Ins) => endings.push("ема"),
        _ => {}
    }
    Ok(endings)
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

/// Long comparison endings after the independently supplied `-(ь)ш-`,
/// `-ѣйш-`, or `-айш-` stem. Alypy §58 gives a mixed series: for example,
/// masculine `-шїй`, feminine `-шаѧ`, and neuter `-шее`, while the oblique
/// cells combine hard `-шагѡ`/`-шꙋю` with soft `-шемꙋ`/`-шихъ` endings.
fn comparison_long_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
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
        (Sg, M, Gen) => "агѡ",
        (Sg, M, Dat) => "емꙋ",
        (Sg, M, Acc) => animate("їй", "аго"),
        (Sg, M, Ins) => "имъ",
        (Sg, M, Loc) => "емъ",
        (Sg, F, Nom | Voc) => "аѧ",
        (Sg, F, Gen) => "їѧ",
        (Sg, F, Dat | Loc) => "ей",
        (Sg, F, Acc) => "ꙋю",
        (Sg, F, Ins) => "ею",
        (Sg, N, Nom | Acc | Voc) => "ее",
        (Sg, N, Gen) => "агѡ",
        (Sg, N, Dat) => "емꙋ",
        (Sg, N, Ins) => "имъ",
        (Sg, N, Loc) => "емъ",
        (Du, M, Nom | Acc | Voc) => "аѧ",
        (Du, F | N, Nom | Acc | Voc) => "їи",
        (Du, _, Gen | Loc) => "ꙋю",
        (Du, _, Dat | Ins) => "има",
        (Pl, M, Nom | Voc) => "їи",
        (Pl, F, Nom | Voc) => "їѧ",
        (Pl, N, Nom | Acc | Voc) => "аѧ",
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

fn second_palatalize_final_velar(stem: &str) -> String {
    let replacement = match stem.chars().last() {
        Some('к') => Some('ц'),
        Some('г') => Some('з'),
        Some('х') => Some('с'),
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

fn last_e_as_wide_e(stem: &str) -> String {
    let mut characters = stem.chars().collect::<Vec<_>>();
    if let Some(index) = characters.iter().rposition(|character| *character == 'е') {
        characters[index] = 'є';
    }
    characters.into_iter().collect()
}

fn last_o_as_omega(stem: &str) -> String {
    let mut characters = stem.chars().collect::<Vec<_>>();
    if let Some(index) = characters.iter().rposition(|character| *character == 'о') {
        characters[index] = 'ѡ';
    }
    characters.into_iter().collect()
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
        citation: normative_citation(rule).into(),
        note: Some(format!("stable rule {rule}")),
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

fn normative_citation(rule: &str) -> &'static str {
    match rule {
        "SYN-NOUN-I-HARD-M-ALYPY-34"
        | "SYN-NOUN-I-HARD-VELAR-M-ALYPY-34"
        | "SYN-NOUN-I-MIXED-M-ALYPY-33-34"
        | "SYN-NOUN-I-HARD-N-ALYPY-34"
        | "SYN-NOUN-I-SOFT-M-ALYPY-34"
        | "SYN-NOUN-I-SOFT-N-ALYPY-34" => "Alypy (Gamanovich), §§34–38",
        "SYN-NOUN-II-HARD-ALYPY-39" | "SYN-NOUN-II-SOFT-ALYPY-39" => {
            "Alypy (Gamanovich), §§39–40, 44"
        }
        "SYN-NOUN-III-F-ALYPY-41" | "SYN-NOUN-III-M-ALYPY-41" => "Alypy (Gamanovich), §41",
        "SYN-NOUN-IV-N-EN-ALYPY-42-43"
        | "SYN-NOUN-IV-N-ES-ALYPY-42-43"
        | "SYN-NOUN-IV-N-AT-ALYPY-42-43"
        | "SYN-NOUN-IV-F-ER-ALYPY-42-43" => "Alypy (Gamanovich), §§42–43",
        "SYN-NOUN-IV-F-OV-ALYPY-42-44" | "SYN-NOUN-IV-M-EN-ALYPY-42-44" => {
            "Alypy (Gamanovich), §§42–44"
        }
        "SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43" => "Alypy (Gamanovich), §43 камень notes",
        "SYN-ADJ-SHORT-HARD-ALYPY-53" | "SYN-ADJ-SHORT-SOFT-ALYPY-53" => {
            "Alypy (Gamanovich), §§53–55"
        }
        "SYN-ADJ-LONG-HARD-ALYPY-57" | "SYN-ADJ-LONG-SOFT-ALYPY-57" => {
            "Alypy (Gamanovich), §§56–57"
        }
        "SYN-ADJ-COMPARATIVE-LONG-ALYPY-58" => "Alypy (Gamanovich), §58",
        "SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-98" => {
            "Alypy (Gamanovich), §58 citation forms and §98 complete declension"
        }
        "SYN-ADJ-SUPERLATIVE-LONG-ALYPY-59" => "Alypy (Gamanovich), §59",
        "SYN-VERB-PRESENT-ALYPY-80" => "Alypy (Gamanovich), §§79–80",
        "SYN-VERB-AORIST-VOWEL-ALYPY-86" | "SYN-VERB-AORIST-CONSONANT-ALYPY-86" => {
            "Alypy (Gamanovich), §86"
        }
        "SYN-VERB-IMPERFECT-H-ALYPY-87"
        | "SYN-VERB-IMPERFECT-YAH-ALYPY-87"
        | "SYN-VERB-IMPERFECT-AH-ALYPY-87" => "Alypy (Gamanovich), §87",
        "SYN-VERB-IMPERATIVE-ALYPY-93" => "Alypy (Gamanovich), §93",
        "SYN-VERB-INFINITIVE-LEXICAL" => "Alypy (Gamanovich), §79; lexical infinitive",
        "SYN-VERB-LPART-ALYPY-97" => "Alypy (Gamanovich), §97",
        "SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-ALYPY-95" => "Alypy (Gamanovich), §95",
        "SYN-VERB-PARTICIPLE-PAST-ACTIVE-ALYPY-96" => "Alypy (Gamanovich), §96",
        "SYN-VERB-PARTICIPLE-PRESENT-PASSIVE-ALYPY-99" => "Alypy (Gamanovich), §99",
        "SYN-VERB-PARTICIPLE-PAST-PASSIVE-ALYPY-100" => "Alypy (Gamanovich), §100",
        "SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-SHORT-ALYPY-95-98" => {
            "Alypy (Gamanovich), §95 citation forms and §98 complete declension"
        }
        "SYN-VERB-PARTICIPLE-PAST-ACTIVE-SHORT-ALYPY-96-98" => {
            "Alypy (Gamanovich), §96 citation forms and §98 complete declension"
        }
        _ => "Synodal normative rule; see stable rule identifier",
    }
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

    fn assert_noun_paradigm(lexeme: &NounLexeme, animacy: Animacy, expected: &[&[&str]]) {
        assert_eq!(expected.len(), Number::ALL.len() * Case::ALL.len());
        for (index, (number, case)) in Number::ALL
            .into_iter()
            .flat_map(|number| Case::ALL.into_iter().map(move |case| (number, case)))
            .enumerate()
        {
            let forms = decline_noun(
                lexeme,
                NounCell {
                    case,
                    number,
                    animacy,
                },
                OrthographyProfile::Expanded,
            )
            .unwrap_or_else(|error| panic!("{number:?} {case:?}: {error}"));
            let actual = forms
                .variants()
                .iter()
                .map(|variant| variant.printed.as_str())
                .collect::<Vec<_>>();
            assert_eq!(actual.as_slice(), expected[index], "{number:?} {case:?}");
        }
    }

    #[test]
    fn declines_first_hard_noun_from_alypy_34() {
        let lexeme = NounLexeme {
            lemma: word("рабъ"),
            stem: word("раб"),
            gender: Gender::Masculine,
            declension: NounDeclension::FirstHardMasculine,
            number_inventory: NounNumberInventory::All,
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
    fn rejects_second_declension_with_non_feminine_gender() {
        let lexeme = NounLexeme {
            lemma: word("жена"),
            stem: word("жен"),
            gender: Gender::Masculine,
            declension: NounDeclension::SecondHard,
            number_inventory: NounNumberInventory::All,
        };
        assert!(matches!(
            decline_noun(
                &lexeme,
                NounCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }

    #[test]
    fn animate_accusatives_retain_alypy_35_variants_in_normative_order() {
        let lexeme = NounLexeme {
            lemma: word("рабъ"),
            stem: word("раб"),
            gender: Gender::Masculine,
            declension: NounDeclension::FirstHardMasculine,
            number_inventory: NounNumberInventory::All,
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
    fn alpy_34_complete_mixed_masculine_golden() {
        let lexeme = NounLexeme::new(
            word("мꙋжъ"),
            word("мꙋж"),
            Gender::Masculine,
            NounDeclension::FirstMixedMasculine,
        );
        assert_noun_paradigm(
            &lexeme,
            Animacy::Animate,
            &[
                &["мꙋжъ"],
                &["мꙋжа"],
                &["мꙋжꙋ"],
                &["мꙋжа", "мꙋжъ"],
                &["мꙋжемъ"],
                &["мꙋжи"],
                &["мꙋжꙋ"],
                &["мꙋжа"],
                &["мꙋжꙋ"],
                &["мꙋжема"],
                &["мꙋжа"],
                &["мꙋжема"],
                &["мꙋжꙋ"],
                &["мꙋжа"],
                &["мꙋжи", "мꙋжїе"],
                &["мꙋжей"],
                &["мꙋжемъ"],
                &["мꙋжы", "мꙋжей"],
                &["мꙋжы"],
                &["мꙋжахъ"],
                &["мꙋжи", "мꙋжїе"],
            ],
        );
    }

    #[test]
    fn alpy_34_velar_alternations_cover_g_k_and_h_boundaries() {
        for (lemma, stem, locative, vocative, nominative_plural) in [
            ("богъ", "бог", "бозѣ", "боже", "бози"),
            ("ѻтрокъ", "ѻтрок", "ѻтроцѣ", "ѻтроче", "ѻтроцы"),
            ("дꙋхъ", "дꙋх", "дꙋсѣ", "дꙋше", "дꙋси"),
        ] {
            let lexeme = NounLexeme::new(
                word(lemma),
                word(stem),
                Gender::Masculine,
                NounDeclension::FirstHardVelarMasculine,
            );
            for (case, number, expected) in [
                (Case::Locative, Number::Singular, locative),
                (Case::Vocative, Number::Singular, vocative),
                (Case::Nominative, Number::Plural, nominative_plural),
            ] {
                let forms = decline_noun(
                    &lexeme,
                    NounCell {
                        case,
                        number,
                        animacy: Animacy::Inanimate,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("reviewed velar cell");
                assert_eq!(
                    forms.primary_text(),
                    expected,
                    "{lemma} {number:?} {case:?}"
                );
            }
        }
    }

    #[test]
    fn productive_noun_classes_reject_incompatible_stem_shapes() {
        for lexeme in [
            NounLexeme::new(
                word("рабъ"),
                word("раб"),
                Gender::Masculine,
                NounDeclension::FirstHardVelarMasculine,
            ),
            NounLexeme::new(
                word("домъ"),
                word("дом"),
                Gender::Masculine,
                NounDeclension::FirstMixedMasculine,
            ),
            NounLexeme::new(
                word("имѧ"),
                word("имес"),
                Gender::Neuter,
                NounDeclension::FourthNeuterEn,
            ),
            NounLexeme::new(
                word("небо"),
                word("небен"),
                Gender::Neuter,
                NounDeclension::FourthNeuterEs,
            ),
            NounLexeme::new(
                word("мати"),
                word("матес"),
                Gender::Feminine,
                NounDeclension::FourthFeminineEr,
            ),
            NounLexeme::new(
                word("ѻтроча"),
                word("ѻтрочен"),
                Gender::Neuter,
                NounDeclension::FourthNeuterAt,
            ),
            NounLexeme::new(
                word("свекры"),
                word("свекрер"),
                Gender::Feminine,
                NounDeclension::FourthFeminineOv,
            ),
            NounLexeme::new(
                word("степень"),
                word("степес"),
                Gender::Masculine,
                NounDeclension::FourthMasculineEn,
            ),
            NounLexeme::new(
                word("камень"),
                word("камен"),
                Gender::Masculine,
                NounDeclension::FourthMasculineEn,
            ),
            NounLexeme::new(
                word("степень"),
                word("степен"),
                Gender::Masculine,
                NounDeclension::FourthMasculineEnKamen,
            ),
            NounLexeme::new(
                word("любовь"),
                word("любов"),
                Gender::Feminine,
                NounDeclension::FourthFeminineOv,
            ),
        ] {
            assert!(matches!(
                validate_noun_lexeme(&lexeme),
                Err(Error::ContradictoryMetadata { .. })
            ));
        }
    }

    #[test]
    fn alpy_41_complete_third_masculine_golden() {
        let lexeme = NounLexeme::new(
            word("пꙋть"),
            word("пꙋт"),
            Gender::Masculine,
            NounDeclension::ThirdMasculine,
        );
        assert_noun_paradigm(
            &lexeme,
            Animacy::Inanimate,
            &[
                &["пꙋть"],
                &["пꙋти"],
                &["пꙋти"],
                &["пꙋть"],
                &["пꙋтемъ"],
                &["пꙋти"],
                &["пꙋть", "пꙋтю"],
                &["пꙋти"],
                &["пꙋтїю"],
                &["пꙋтьма"],
                &["пꙋти"],
                &["пꙋтьма"],
                &["пꙋтїю"],
                &["пꙋти"],
                &["пꙋтїе"],
                &["пꙋтій", "пꙋтей"],
                &["пꙋтємъ"],
                &["пꙋти"],
                &["пꙋтьми"],
                &["пꙋтехъ"],
                &["пꙋтїе"],
            ],
        );
    }

    #[test]
    fn alpy_43_complete_extended_stem_goldens() {
        let imya = NounLexeme::new(
            word("имѧ"),
            word("имен"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEn,
        );
        assert_noun_paradigm(
            &imya,
            Animacy::Inanimate,
            &[
                &["имѧ"],
                &["имене"],
                &["имени"],
                &["имѧ"],
                &["именемъ"],
                &["имени"],
                &["имѧ"],
                &["имєни"],
                &["именꙋ"],
                &["именема", "именама"],
                &["имєни"],
                &["именема", "именама"],
                &["именꙋ"],
                &["имєни"],
                &["имена"],
                &["именъ"],
                &["именємъ", "именѡмъ"],
                &["имена"],
                &["имены"],
                &["именѣхъ"],
                &["имена"],
            ],
        );

        let nebo = NounLexeme::new(
            word("небо"),
            word("небес"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEs,
        );
        assert_noun_paradigm(
            &nebo,
            Animacy::Inanimate,
            &[
                &["небо"],
                &["небесе"],
                &["небеси"],
                &["небо"],
                &["небесемъ"],
                &["небеси"],
                &["небо"],
                &["небєси"],
                &["небесꙋ"],
                &["небесема"],
                &["небєси"],
                &["небесема"],
                &["небесꙋ"],
                &["небєси"],
                &["небеса"],
                &["небесъ"],
                &["небесємъ"],
                &["небеса"],
                &["небесы"],
                &["небесѣхъ"],
                &["небеса"],
            ],
        );

        let mati = NounLexeme::new(
            word("мати"),
            word("матер"),
            Gender::Feminine,
            NounDeclension::FourthFeminineEr,
        );
        assert_noun_paradigm(
            &mati,
            Animacy::Animate,
            &[
                &["мати"],
                &["матере"],
                &["матери"],
                &["матерь"],
                &["матерїю"],
                &["матери"],
                &["мати"],
                &["матєри"],
                &["матєрїю"],
                &["матерема"],
                &["матєри"],
                &["матерема"],
                &["матєрїю"],
                &["матєри"],
                &["матєри"],
                &["матерїй", "матерей"],
                &["матеремъ"],
                &["матерей", "матери"],
                &["матерьми"],
                &["матерехъ"],
                &["матєри"],
            ],
        );
    }

    #[test]
    fn alpy_43_44_additional_extended_stem_goldens() {
        let otrocha = NounLexeme::new(
            word("ѻтроча"),
            word("ѻтрочат"),
            Gender::Neuter,
            NounDeclension::FourthNeuterAt,
        );
        assert_noun_paradigm(
            &otrocha,
            Animacy::Inanimate,
            &[
                &["ѻтроча"],
                &["ѻтрочате"],
                &["ѻтрочати"],
                &["ѻтроча"],
                &["ѻтрочатемъ"],
                &["ѻтрочати"],
                &["ѻтроча"],
                &["ѻтрѡчати"],
                &["ѻтрочатꙋ"],
                &["ѻтрочатема", "ѻтрочатама"],
                &["ѻтрѡчати"],
                &["ѻтрочатема", "ѻтрочатама"],
                &["ѻтрочатꙋ"],
                &["ѻтрѡчати"],
                &["ѻтрочата"],
                &["ѻтрочатъ"],
                &["ѻтрочатємъ", "ѻтрочатѡмъ"],
                &["ѻтрочата"],
                &["ѻтрочаты"],
                &["ѻтрочатѣхъ"],
                &["ѻтрочата"],
            ],
        );

        let svekry = NounLexeme::new(
            word("свекры"),
            word("свекров"),
            Gender::Feminine,
            NounDeclension::FourthFeminineOv,
        );
        assert_noun_paradigm(
            &svekry,
            Animacy::Animate,
            &[
                &["свекры"],
                &["свекрове"],
                &["свекрови"],
                &["свекровь"],
                &["свекровїю"],
                &["свекрови"],
                &["свекры"],
                &["свекрѡви"],
                &["свекрѡвїю"],
                &["свекровама"],
                &["свекрѡви"],
                &["свекровама"],
                &["свекрѡвїю"],
                &["свекрѡви"],
                &["свекрѡви"],
                &["свекровей"],
                &["свекровамъ"],
                &["свекровей", "свекрови"],
                &["свекровами"],
                &["свекровахъ"],
                &["свекрѡви"],
            ],
        );

        let kamen = NounLexeme::new(
            word("камень"),
            word("камен"),
            Gender::Masculine,
            NounDeclension::FourthMasculineEnKamen,
        );
        assert_noun_paradigm(
            &kamen,
            Animacy::Inanimate,
            &[
                &["камень"],
                &["камене", "каменѧ"],
                &["камени", "каменю"],
                &["камень"],
                &["каменемъ"],
                &["камени"],
                &["камень"],
                &["камєни"],
                &["каменꙋ"],
                &["каменьма", "каменема"],
                &["камєни"],
                &["каменьма", "каменема"],
                &["каменꙋ"],
                &["камєни"],
                &["камєни", "каменїѧ"],
                &["каменїй"],
                &["каменємъ"],
                &["камєни", "каменїѧ"],
                &["каменьми"],
                &["каменехъ", "каменїѧхъ"],
                &["камєни", "каменїѧ"],
            ],
        );
        assert!(
            decline_noun(
                &kamen,
                NounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("ordinary plural")
            .variants()
            .iter()
            .all(|variant| variant.expanded != "каменїе")
        );
        assert_eq!(
            decline_noun(
                &kamen,
                NounCell {
                    case: Case::Accusative,
                    number: Number::Plural,
                    animacy: Animacy::Animate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("ordered animate variants")
            .variants()
            .iter()
            .map(|variant| variant.expanded.as_str())
            .collect::<Vec<_>>(),
            ["камєни", "каменїѧ", "каменїй"]
        );
    }

    #[test]
    fn plural_only_nouns_retain_absent_numbers_as_typed_failures() {
        let people = NounLexeme::new(
            word("людїе"),
            word("люд"),
            Gender::Masculine,
            NounDeclension::ThirdMasculine,
        )
        .with_number_inventory(NounNumberInventory::PluralOnly);
        assert_eq!(
            decline_noun(
                &people,
                NounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    animacy: Animacy::Animate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("licensed plural")
            .primary_text(),
            "людїе"
        );
        assert!(matches!(
            decline_noun(
                &people,
                NounCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    animacy: Animacy::Animate,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn declines_long_hard_adjective_from_alypy_57() {
        let lexeme = AdjectiveLexeme {
            lemma: word("мꙋдръ"),
            stem: word("мꙋдр"),
            class: AdjectiveClass::Hard,
            comparative_stem: None,
            comparison_formation: None,
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
    fn declines_comparison_stem_with_alypy_58_mixed_endings() {
        let lexeme = AdjectiveLexeme {
            lemma: word("мꙋдръ"),
            stem: word("мꙋдр"),
            class: AdjectiveClass::Hard,
            comparative_stem: Some(word("мꙋдрѣйш")),
            comparison_formation: Some(ComparisonFormation::LaterYat),
        };
        let form = |case, number, gender, animacy| {
            decline_adjective(
                &lexeme,
                AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Comparative,
                },
                OrthographyProfile::Expanded,
            )
            .expect("reviewed comparison stem")
            .primary_text()
            .to_owned()
        };
        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Feminine,
                Animacy::Inanimate
            ),
            "мꙋдрѣйшаѧ"
        );
        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Neuter,
                Animacy::Inanimate
            ),
            "мꙋдрѣйшее"
        );
        assert_eq!(
            form(
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate
            ),
            "мꙋдрѣйшагѡ"
        );
        assert_eq!(
            form(
                Case::Dative,
                Number::Singular,
                Gender::Masculine,
                Animacy::Animate
            ),
            "мꙋдрѣйшемꙋ"
        );
        assert_eq!(
            form(
                Case::Accusative,
                Number::Singular,
                Gender::Feminine,
                Animacy::Inanimate
            ),
            "мꙋдрѣйшꙋю"
        );
        assert_eq!(
            form(
                Case::Genitive,
                Number::Plural,
                Gender::Masculine,
                Animacy::Animate
            ),
            "мꙋдрѣйшихъ"
        );
    }

    #[test]
    fn short_superlative_remains_explicitly_unsupported() {
        let lexeme = AdjectiveLexeme {
            lemma: word("мꙋдръ"),
            stem: word("мꙋдр"),
            class: AdjectiveClass::Hard,
            comparative_stem: Some(word("мꙋдрѣйш")),
            comparison_formation: Some(ComparisonFormation::LaterYat),
        };
        assert!(matches!(
            decline_adjective(
                &lexeme,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Superlative,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::UnsupportedFormation { .. })
        ));
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

    #[test]
    fn declines_independently_specified_participle_stems() {
        let lexeme = regular_verb();
        let short = decline_participle(
            &lexeme,
            ParticipleCell {
                tense: ParticipleTense::Present,
                voice: ParticipleVoice::Active,
                agreement: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Feminine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
            },
            OrthographyProfile::Expanded,
        )
        .expect("reviewed participial principal part");
        assert_eq!(short.primary_text(), "несꙋщаѧ");

        let long = decline_participle(
            &lexeme,
            ParticipleCell {
                tense: ParticipleTense::Past,
                voice: ParticipleVoice::Passive,
                agreement: AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
            },
            OrthographyProfile::Expanded,
        )
        .expect("separate full-form stem");
        assert_eq!(long.primary_text(), "несенный");
    }

    #[test]
    fn rejects_comparison_for_participles() {
        let cell = AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Long,
            comparison: Comparison::Comparative,
        };
        assert!(matches!(
            decline_participle(
                &regular_verb(),
                ParticipleCell {
                    tense: ParticipleTense::Past,
                    voice: ParticipleVoice::Active,
                    agreement: cell,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn missing_principal_part_diagnostics_include_typed_formations() {
        let mut verb = regular_verb();
        verb.imperfect_formation = None;
        assert_eq!(
            verb.missing_principal_parts(VerbSystem::Finite(FiniteTense::Imperfect)),
            vec![MetadataField::ImperfectFormation]
        );
        verb.aorist_formation = None;
        assert_eq!(
            verb.missing_principal_parts(VerbSystem::Finite(FiniteTense::Aorist)),
            vec![MetadataField::AoristFormation]
        );
        verb.imperative_formation = None;
        assert_eq!(
            verb.missing_principal_parts(VerbSystem::Imperative),
            vec![MetadataField::ImperativeFormation]
        );
        verb.present_active_participle
            .as_mut()
            .expect("test principal part")
            .short_formation = None;
        assert_eq!(
            verb.missing_principal_parts(VerbSystem::Participle {
                tense: ParticipleTense::Present,
                voice: ParticipleVoice::Active,
                form: AdjectiveForm::Short,
            }),
            vec![MetadataField::ParticipleFormation]
        );
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
            present_active_participle: Some(ParticiplePrincipalPart {
                short_stem: Some(word("несꙋщ")),
                short_formation: Some(ActiveParticipleShortFormation::PresentFirstUnpalatalized),
                long_stem: Some(word("несꙋщ")),
                class: AdjectiveClass::Hard,
            }),
            past_active_participle: Some(ParticiplePrincipalPart {
                short_stem: Some(word("несш")),
                short_formation: Some(ActiveParticipleShortFormation::PastConsonant),
                long_stem: Some(word("несш")),
                class: AdjectiveClass::Hard,
            }),
            present_passive_participle: Some(ParticiplePrincipalPart {
                short_stem: Some(word("несом")),
                short_formation: None,
                long_stem: Some(word("несом")),
                class: AdjectiveClass::Hard,
            }),
            past_passive_participle: Some(ParticiplePrincipalPart {
                short_stem: Some(word("несен")),
                short_formation: None,
                long_stem: Some(word("несенн")),
                class: AdjectiveClass::Hard,
            }),
            verbal_noun: None,
        }
    }

    fn assert_productive_contract(forms: &FormSet) {
        assert!(forms.variants().iter().all(|variant| {
            matches!(
                &variant.source,
                FormSource::SynodalNormativeGeneration { rule } if !rule.to_string().is_empty()
            ) && variant.target_recension == Recension::SynodalRussian
                && variant.source_recension == Some(Recension::SynodalRussian)
                && !variant.evidence.is_empty()
                && variant.evidence.iter().all(|evidence| {
                    evidence.kind == EvidenceKind::NormativeRule
                        && evidence.source_recension == Recension::SynodalRussian
                        && !evidence.citation.is_empty()
                })
                && !variant.rule_trace.steps().is_empty()
        }));
    }

    #[test]
    fn productive_rule_inventory_contracts_are_complete() {
        for declension in NounDeclension::ALL {
            let (lemma, stem, gender) = match declension {
                NounDeclension::FirstHardMasculine => ("рабъ", "раб", Gender::Masculine),
                NounDeclension::FirstHardVelarMasculine => ("ѻтрокъ", "ѻтрок", Gender::Masculine),
                NounDeclension::FirstMixedMasculine => ("мꙋжъ", "мꙋж", Gender::Masculine),
                NounDeclension::FirstHardNeuter => ("слово", "слов", Gender::Neuter),
                NounDeclension::FirstSoftMasculine => ("царь", "цар", Gender::Masculine),
                NounDeclension::FirstSoftNeuter => ("море", "мор", Gender::Neuter),
                NounDeclension::SecondHard => ("жена", "жен", Gender::Feminine),
                NounDeclension::SecondSoft => ("землѧ", "земл", Gender::Feminine),
                NounDeclension::ThirdFeminine => ("кость", "кост", Gender::Feminine),
                NounDeclension::ThirdMasculine => ("пꙋть", "пꙋт", Gender::Masculine),
                NounDeclension::FourthNeuterEn => ("имѧ", "имен", Gender::Neuter),
                NounDeclension::FourthNeuterEs => ("небо", "небес", Gender::Neuter),
                NounDeclension::FourthNeuterAt => ("ѻтроча", "ѻтрочат", Gender::Neuter),
                NounDeclension::FourthFeminineEr => ("мати", "матер", Gender::Feminine),
                NounDeclension::FourthFeminineOv => ("свекры", "свекров", Gender::Feminine),
                NounDeclension::FourthMasculineEn => ("степень", "степен", Gender::Masculine),
                NounDeclension::FourthMasculineEnKamen => ("камень", "камен", Gender::Masculine),
            };
            let lexeme = NounLexeme {
                lemma: word(lemma),
                stem: word(stem),
                gender,
                declension,
                number_inventory: NounNumberInventory::All,
            };
            for number in Number::ALL {
                for case in Case::ALL {
                    for animacy in if case == Case::Accusative {
                        Animacy::ALL.as_slice()
                    } else {
                        &[Animacy::Inanimate]
                    } {
                        assert_productive_contract(
                            &decline_noun(
                                &lexeme,
                                crate::NounCell {
                                    case,
                                    number,
                                    animacy: *animacy,
                                },
                                OrthographyProfile::Expanded,
                            )
                            .expect("declared noun inventory"),
                        );
                    }
                }
            }
        }

        for class in [AdjectiveClass::Hard, AdjectiveClass::Soft] {
            let lexeme = AdjectiveLexeme {
                lemma: word("мꙋдръ"),
                stem: word("мꙋдр"),
                class,
                comparative_stem: Some(word("мꙋдрѣйш")),
                comparison_formation: Some(ComparisonFormation::LaterYat),
            };
            for form in [AdjectiveForm::Short, AdjectiveForm::Long] {
                for comparison in if form == AdjectiveForm::Short {
                    &[Comparison::Positive][..]
                } else {
                    &[
                        Comparison::Positive,
                        Comparison::Comparative,
                        Comparison::Superlative,
                    ][..]
                } {
                    for number in Number::ALL {
                        for case in Case::ALL {
                            for gender in Gender::ALL {
                                for animacy in if case == Case::Accusative {
                                    Animacy::ALL.as_slice()
                                } else {
                                    &[Animacy::Inanimate]
                                } {
                                    assert_productive_contract(
                                        &decline_adjective(
                                            &lexeme,
                                            AdjectiveCell {
                                                case,
                                                number,
                                                gender,
                                                animacy: *animacy,
                                                form,
                                                comparison: *comparison,
                                            },
                                            OrthographyProfile::Expanded,
                                        )
                                        .expect("declared adjective inventory"),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let base = regular_verb();
        for number in Number::ALL {
            for person in Person::ALL {
                assert_productive_contract(
                    &present(&base, person, number, OrthographyProfile::Expanded)
                        .expect("declared present inventory"),
                );
                for formation in [AoristFormation::VowelStem, AoristFormation::ConsonantStem] {
                    let mut verb = base.clone();
                    verb.aorist_formation = Some(formation);
                    assert_productive_contract(
                        &aorist(&verb, person, number, OrthographyProfile::Expanded)
                            .expect("declared aorist inventory"),
                    );
                }
                for formation in [
                    ImperfectFormation::H,
                    ImperfectFormation::Yah,
                    ImperfectFormation::Ah,
                ] {
                    let mut verb = base.clone();
                    verb.imperfect_formation = Some(formation);
                    assert_productive_contract(
                        &imperfect(&verb, person, number, OrthographyProfile::Expanded)
                            .expect("declared imperfect inventory"),
                    );
                }
                for formation in [
                    ImperativeFormation::FirstUnpalatalized,
                    ImperativeFormation::ISeries,
                ] {
                    let mut verb = base.clone();
                    verb.imperative_formation = Some(formation);
                    let outcome = imperative(
                        &verb,
                        ImperativeCell { person, number },
                        OrthographyProfile::Expanded,
                    );
                    if (person == Person::First && number == Number::Singular)
                        || (person == Person::Third && number != Number::Singular)
                    {
                        assert!(matches!(
                            outcome,
                            Err(Error::HistoricallyInvalidCell { .. })
                        ));
                    } else {
                        assert_productive_contract(
                            &outcome.expect("declared imperative inventory"),
                        );
                    }
                }
            }
            for gender in Gender::ALL {
                assert_productive_contract(
                    &l_participle(
                        &base,
                        LParticipleCell { gender, number },
                        OrthographyProfile::Expanded,
                    )
                    .expect("declared l-participle inventory"),
                );
            }
        }

        for tense in ParticipleTense::ALL {
            for voice in ParticipleVoice::ALL {
                for form in [AdjectiveForm::Short, AdjectiveForm::Long] {
                    if voice == ParticipleVoice::Active && form == AdjectiveForm::Short {
                        continue;
                    }
                    for number in Number::ALL {
                        for case in Case::ALL {
                            for gender in Gender::ALL {
                                for animacy in if case == Case::Accusative {
                                    Animacy::ALL.as_slice()
                                } else {
                                    &[Animacy::Inanimate]
                                } {
                                    assert_productive_contract(
                                        &decline_participle(
                                            &base,
                                            ParticipleCell {
                                                tense,
                                                voice,
                                                agreement: AdjectiveCell {
                                                    case,
                                                    number,
                                                    gender,
                                                    animacy: *animacy,
                                                    form,
                                                    comparison: Comparison::Positive,
                                                },
                                            },
                                            OrthographyProfile::Expanded,
                                        )
                                        .expect("declared ordinary participle inventory"),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn alpy_58_95_96_98_complete_short_comparison_goldens() {
        #[derive(Clone, Copy)]
        struct Golden {
            number: Number,
            gender: Gender,
            case: Case,
            variants: &'static [&'static str],
            animate_variants: Option<&'static [&'static str]>,
        }

        use Case::{
            Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins,
            Locative as Loc, Nominative as Nom,
        };
        use Gender::{Feminine as F, Masculine as M, Neuter as N};
        use Number::{Dual as Du, Plural as Pl, Singular as Sg};

        // Alypy §98's complete short-comparison table, represented as suffixes
        // after the independently supplied comparison/participle stem. The
        // three singular nominative citation edges are checked separately.
        let goldens = [
            Golden {
                number: Sg,
                gender: M,
                case: Nom,
                variants: &[],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: M,
                case: Gen,
                variants: &["а"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: M,
                case: Dat,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: M,
                case: Acc,
                variants: &["ъ"],
                animate_variants: Some(&["ъ", "а"]),
            },
            Golden {
                number: Sg,
                gender: M,
                case: Ins,
                variants: &["имъ"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: M,
                case: Loc,
                variants: &["емъ"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: F,
                case: Nom,
                variants: &[],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: F,
                case: Gen,
                variants: &["и"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: F,
                case: Dat,
                variants: &["и"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: F,
                case: Acc,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: F,
                case: Ins,
                variants: &["ею"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: F,
                case: Loc,
                variants: &["и"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: N,
                case: Nom,
                variants: &[],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: N,
                case: Gen,
                variants: &["а"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: N,
                case: Dat,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: N,
                case: Acc,
                variants: &["е"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: N,
                case: Ins,
                variants: &["имъ"],
                animate_variants: None,
            },
            Golden {
                number: Sg,
                gender: N,
                case: Loc,
                variants: &["емъ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: M,
                case: Nom,
                variants: &["а"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: M,
                case: Gen,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: M,
                case: Dat,
                variants: &["ема"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: M,
                case: Acc,
                variants: &["а"],
                animate_variants: Some(&["а", "ꙋ"]),
            },
            Golden {
                number: Du,
                gender: M,
                case: Ins,
                variants: &["ема"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: M,
                case: Loc,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: F,
                case: Nom,
                variants: &["ѣ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: F,
                case: Gen,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: F,
                case: Dat,
                variants: &["ема"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: F,
                case: Acc,
                variants: &["ѣ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: F,
                case: Ins,
                variants: &["ема"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: F,
                case: Loc,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: N,
                case: Nom,
                variants: &["ѣ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: N,
                case: Gen,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: N,
                case: Dat,
                variants: &["ема"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: N,
                case: Acc,
                variants: &["ѣ"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: N,
                case: Ins,
                variants: &["ема"],
                animate_variants: None,
            },
            Golden {
                number: Du,
                gender: N,
                case: Loc,
                variants: &["ꙋ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: M,
                case: Nom,
                variants: &["е"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: M,
                case: Gen,
                variants: &["ихъ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: M,
                case: Dat,
                variants: &["ымъ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: M,
                case: Acc,
                variants: &["ѧ"],
                animate_variants: Some(&["ѧ", "ихъ"]),
            },
            Golden {
                number: Pl,
                gender: M,
                case: Ins,
                variants: &["ими"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: M,
                case: Loc,
                variants: &["ихъ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: F,
                case: Nom,
                variants: &["ѧ", "е"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: F,
                case: Gen,
                variants: &["ихъ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: F,
                case: Dat,
                variants: &["ымъ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: F,
                case: Acc,
                variants: &["ѧ"],
                animate_variants: Some(&["ѧ", "ихъ"]),
            },
            Golden {
                number: Pl,
                gender: F,
                case: Ins,
                variants: &["ими"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: F,
                case: Loc,
                variants: &["ихъ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: N,
                case: Nom,
                variants: &["а"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: N,
                case: Gen,
                variants: &["ихъ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: N,
                case: Dat,
                variants: &["ымъ"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: N,
                case: Acc,
                variants: &["а"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: N,
                case: Ins,
                variants: &["ими"],
                animate_variants: None,
            },
            Golden {
                number: Pl,
                gender: N,
                case: Loc,
                variants: &["ихъ"],
                animate_variants: None,
            },
        ];

        let adjective = AdjectiveLexeme {
            lemma: word("мꙋдръ"),
            stem: word("мꙋдр"),
            class: AdjectiveClass::Hard,
            comparative_stem: Some(word("мꙋдрѣйш")),
            comparison_formation: Some(ComparisonFormation::LaterYat),
        };
        let verb = regular_verb();

        for golden in goldens {
            for animacy in if golden.case == Acc {
                Animacy::ALL.as_slice()
            } else {
                &[Animacy::Inanimate]
            } {
                let adjective_cell = AdjectiveCell {
                    case: golden.case,
                    number: golden.number,
                    gender: golden.gender,
                    animacy: *animacy,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Comparative,
                };
                let suffixes = if *animacy == Animacy::Animate {
                    golden.animate_variants.unwrap_or(golden.variants)
                } else {
                    golden.variants
                };
                let comparison_expected = if golden.number == Sg && golden.case == Nom {
                    match golden.gender {
                        M => vec!["мꙋдрѣй".to_owned()],
                        F => vec!["мꙋдрѣйши".to_owned()],
                        N => vec!["мꙋдрѣе".to_owned(), "мꙋдрѣйше".to_owned()],
                    }
                } else {
                    suffixes
                        .iter()
                        .map(|suffix| format!("мꙋдрѣйш{suffix}"))
                        .collect()
                };
                let comparison =
                    decline_adjective(&adjective, adjective_cell, OrthographyProfile::Expanded)
                        .expect("Alypy §§58, 98 comparison cell");
                assert_eq!(
                    comparison.texts().collect::<Vec<_>>(),
                    comparison_expected
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                );
                assert!(comparison.variants().iter().all(|variant| {
                    variant.target_recension == Recension::SynodalRussian
                        && !variant.evidence.is_empty()
                        && variant
                            .evidence
                            .iter()
                            .all(|evidence| evidence.citation.contains("Alypy"))
                        && !variant.rule_trace.steps().is_empty()
                }));

                for (tense, stem, citation) in [
                    (
                        ParticipleTense::Present,
                        "несꙋщ",
                        ["несый|несꙋщь", "несꙋщи", "несый|несꙋще|несꙋщо"],
                    ),
                    (
                        ParticipleTense::Past,
                        "несш",
                        ["несъ|несшъ", "несши", "несъ|несше|несшо"],
                    ),
                ] {
                    let mut agreement = adjective_cell;
                    agreement.comparison = Comparison::Positive;
                    let forms = decline_participle(
                        &verb,
                        ParticipleCell {
                            tense,
                            voice: ParticipleVoice::Active,
                            agreement,
                        },
                        OrthographyProfile::Expanded,
                    )
                    .expect("Alypy §§95–96, 98 active-participle cell");
                    let expected = if golden.number == Sg && golden.case == Nom {
                        citation[match golden.gender {
                            M => 0,
                            F => 1,
                            N => 2,
                        }]
                        .split('|')
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                    } else {
                        suffixes
                            .iter()
                            .map(|suffix| format!("{stem}{suffix}"))
                            .collect()
                    };
                    assert_eq!(
                        forms.texts().collect::<Vec<_>>(),
                        expected.iter().map(String::as_str).collect::<Vec<_>>()
                    );
                    assert!(forms.variants().iter().all(|variant| {
                        variant.target_recension == Recension::SynodalRussian
                            && !variant.evidence.is_empty()
                            && variant
                                .evidence
                                .iter()
                                .all(|evidence| evidence.citation.contains("Alypy"))
                            && !variant.rule_trace.steps().is_empty()
                    }));
                }
            }
        }

        for number in Number::ALL {
            for gender in Gender::ALL {
                let comparison_cell = AdjectiveCell {
                    case: Case::Vocative,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Comparative,
                };
                assert!(matches!(
                    decline_adjective(&adjective, comparison_cell, OrthographyProfile::Expanded),
                    Err(Error::HistoricallyInvalidCell { .. })
                ));
                for tense in ParticipleTense::ALL {
                    let mut agreement = comparison_cell;
                    agreement.comparison = Comparison::Positive;
                    assert!(matches!(
                        decline_participle(
                            &verb,
                            ParticipleCell {
                                tense,
                                voice: ParticipleVoice::Active,
                                agreement,
                            },
                            OrthographyProfile::Expanded
                        ),
                        Err(Error::HistoricallyInvalidCell { .. })
                    ));
                }
            }
        }
    }

    #[test]
    fn active_participle_citation_formation_seams() {
        struct Citation<'a> {
            tense: ParticipleTense,
            formation: ActiveParticipleShortFormation,
            stem: &'a str,
            masculine: &'a [&'a str],
            feminine: &'a [&'a str],
            neuter: &'a [&'a str],
        }
        let citations = [
            Citation {
                tense: ParticipleTense::Present,
                formation: ActiveParticipleShortFormation::PresentFirstPalatalized,
                stem: "дѣлающ",
                masculine: &["дѣлаѧ", "дѣлающь"],
                feminine: &["дѣлающи"],
                neuter: &["дѣлаѧ", "дѣлающе", "дѣлающо"],
            },
            Citation {
                tense: ParticipleTense::Present,
                formation: ActiveParticipleShortFormation::PresentSecond,
                stem: "молѧщ",
                masculine: &["молѧ", "молѧщь"],
                feminine: &["молѧщи"],
                neuter: &["молѧ", "молѧще", "молѧщо"],
            },
            Citation {
                tense: ParticipleTense::Present,
                formation: ActiveParticipleShortFormation::PresentAfterSibilant,
                stem: "молчащ",
                masculine: &["молча", "молчѧ", "молчащь"],
                feminine: &["молчащи"],
                neuter: &["молча", "молчѧ", "молчаще", "молчащо"],
            },
            Citation {
                tense: ParticipleTense::Past,
                formation: ActiveParticipleShortFormation::PastVowel,
                stem: "дѣлавш",
                masculine: &["дѣлавъ", "дѣлавшъ"],
                feminine: &["дѣлавши"],
                neuter: &["дѣлавъ", "дѣлавше", "дѣлавшо"],
            },
            Citation {
                tense: ParticipleTense::Past,
                formation: ActiveParticipleShortFormation::PastIotated,
                stem: "сотворьш",
                masculine: &["сотворь"],
                feminine: &["сотворьши"],
                neuter: &["сотворь", "сотворьше"],
            },
        ];

        for citation in citations {
            let part = ParticiplePrincipalPart {
                short_stem: Some(word(citation.stem)),
                short_formation: Some(citation.formation),
                long_stem: None,
                class: AdjectiveClass::Hard,
            };
            let mut verb = regular_verb();
            match citation.tense {
                ParticipleTense::Present => verb.present_active_participle = Some(part),
                ParticipleTense::Past => verb.past_active_participle = Some(part),
            }
            for (gender, expected) in [
                (Gender::Masculine, citation.masculine),
                (Gender::Feminine, citation.feminine),
                (Gender::Neuter, citation.neuter),
            ] {
                let forms = decline_participle(
                    &verb,
                    ParticipleCell {
                        tense: citation.tense,
                        voice: ParticipleVoice::Active,
                        agreement: AdjectiveCell {
                            case: Case::Nominative,
                            number: Number::Singular,
                            gender,
                            animacy: Animacy::Inanimate,
                            form: AdjectiveForm::Short,
                            comparison: Comparison::Positive,
                        },
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("source-backed citation edge");
                assert_eq!(forms.texts().collect::<Vec<_>>(), expected);
            }
        }
    }

    #[test]
    fn aspect_sensitive_rules_reject_unknown_aspect_as_missing_metadata() {
        let mut verb = regular_verb();
        verb.aspect = Aspect::Unknown;
        assert_eq!(
            imperfect(
                &verb,
                Person::First,
                Number::Singular,
                OrthographyProfile::Expanded,
            ),
            Err(Error::MissingMetadata {
                field: MetadataField::Aspect,
            })
        );
        assert_eq!(
            decline_participle(
                &verb,
                ParticipleCell {
                    tense: ParticipleTense::Present,
                    voice: ParticipleVoice::Active,
                    agreement: AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Long,
                        comparison: Comparison::Positive,
                    },
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::MissingMetadata {
                field: MetadataField::Aspect,
            })
        );
    }

    #[test]
    fn comparison_citation_formation_seams() {
        struct Citation<'a> {
            formation: ComparisonFormation,
            stem: &'a str,
            masculine: &'a [&'a str],
            feminine: &'a [&'a str],
            neuter: &'a [&'a str],
        }
        let citations = [
            Citation {
                formation: ComparisonFormation::AncientHard,
                stem: "вышш",
                masculine: &["вышїй"],
                feminine: &["вышши"],
                neuter: &["выше", "вышше"],
            },
            Citation {
                formation: ComparisonFormation::AncientSoft,
                stem: "глꙋбльш",
                masculine: &["глꙋблїй"],
                feminine: &["глꙋбльши"],
                neuter: &["глꙋбле", "глꙋбльше"],
            },
            Citation {
                formation: ComparisonFormation::LaterAi,
                stem: "высочайш",
                masculine: &["высочай"],
                feminine: &["высочайши"],
                neuter: &["высочае", "высочайше"],
            },
        ];

        for citation in citations {
            let adjective = AdjectiveLexeme {
                lemma: word("высокъ"),
                stem: word("высок"),
                class: AdjectiveClass::Hard,
                comparative_stem: Some(word(citation.stem)),
                comparison_formation: Some(citation.formation),
            };
            for (gender, expected) in [
                (Gender::Masculine, citation.masculine),
                (Gender::Feminine, citation.feminine),
                (Gender::Neuter, citation.neuter),
            ] {
                let forms = decline_adjective(
                    &adjective,
                    AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Comparative,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("source-backed comparison citation edge");
                assert_eq!(forms.texts().collect::<Vec<_>>(), expected);
            }
        }

        let contradictory = AdjectiveLexeme {
            lemma: word("высокъ"),
            stem: word("высок"),
            class: AdjectiveClass::Hard,
            comparative_stem: Some(word("высочайш")),
            comparison_formation: Some(ComparisonFormation::AncientSoft),
        };
        assert!(matches!(
            decline_adjective(
                &contradictory,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Comparative,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}
