use crate::{
    AuthorityRole, EpistemicRole, Error, Evidence, EvidenceId, EvidenceKind, FiniteTense,
    FiniteVerbCell, FormSet, FormSource, FormVariant, Gender, ImperativeCell, LParticipleCell,
    MetadataField, Number, OrthographyProfile, ParticipleTense, ParticipleVoice, Person, Recension,
    Result, RuleId, RuleTrace, SourceId, SynodalWord, TraceStep, VerbSystem,
};

use super::*;

/// Source-bounded formation used to connect a verb to a completely specified
/// derived noun.
///
/// Alypy §27 makes only the `-їе` family mechanically recoverable from a
/// past-passive base. Its other deverbal suffixes are lexical choices, so they
/// remain productive only after the resulting noun itself has been supplied.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum VerbalNounFormation {
    PastPassiveIe,
    ExplicitLexicalNoun,
}

/// Complete typed metadata for one Synodal verbal noun.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct VerbalNounPrincipalPart {
    noun: NounLexeme,
    formation: VerbalNounFormation,
}

impl VerbalNounPrincipalPart {
    /// Forms the productive abstract noun in `-їе` from the complete
    /// short past-passive platform, for example `молен- : моленїе`.
    pub fn past_passive_ie(platform: impl Into<String>) -> Result<Self> {
        let platform = SynodalWord::parse(platform)?;
        if !matches!(platform.canonical().chars().last(), Some('н' | 'т')) {
            return Err(Error::ContradictoryMetadata {
                reason: "a productive verbal noun in -їе requires a short past-passive platform ending in н or т"
                    .into(),
            });
        }
        let lemma = SynodalWord::parse(format!("{}їе", platform.canonical()))?;
        let stem = SynodalWord::parse(format!("{}ї", platform.canonical()))?;
        let noun = NounLexeme::new(
            lemma,
            stem,
            Gender::Neuter,
            NounDeclension::FirstSoftNeuterIe,
        );
        let principal_part = Self {
            noun,
            formation: VerbalNounFormation::PastPassiveIe,
        };
        principal_part.validate()?;
        Ok(principal_part)
    }

    /// Admits one of Alypy §27's lexical suffix families only after the
    /// caller has supplied its complete noun identity and declensional class.
    pub fn explicit_lexical(noun: NounLexeme) -> Result<Self> {
        let principal_part = Self {
            noun,
            formation: VerbalNounFormation::ExplicitLexicalNoun,
        };
        principal_part.validate()?;
        Ok(principal_part)
    }

    #[must_use]
    pub const fn noun(&self) -> &NounLexeme {
        &self.noun
    }

    #[must_use]
    pub const fn formation(&self) -> VerbalNounFormation {
        self.formation
    }

    /// Rejects deserialized or internally assembled metadata that does not
    /// satisfy the selected source-bounded formation.
    pub fn validate(&self) -> Result<()> {
        validate_noun_lexeme(&self.noun)?;
        match self.formation {
            VerbalNounFormation::PastPassiveIe => {
                let stem = self.noun.stem.canonical();
                let platform =
                    stem.strip_suffix('ї')
                        .ok_or_else(|| Error::ContradictoryMetadata {
                            reason: "a productive verbal noun in -їе requires a stem ending in ї"
                                .into(),
                        })?;
                if self.noun.declension != NounDeclension::FirstSoftNeuterIe
                    || self.noun.gender != Gender::Neuter
                    || self.noun.number_inventory != NounNumberInventory::All
                    || self.noun.lemma.canonical() != format!("{stem}е")
                    || !matches!(platform.chars().last(), Some('н' | 'т'))
                {
                    return Err(Error::ContradictoryMetadata {
                        reason: "a productive verbal noun in -їе requires a neuter all-number -їе noun built on a short past-passive platform ending in н or т"
                            .into(),
                    });
                }
            }
            VerbalNounFormation::ExplicitLexicalNoun => {
                let lemma = strip_presentation_marks(self.noun.lemma.canonical()).to_lowercase();
                if ![
                    "ота", "ета", "ба", "ежъ", "нь", "снь", "знь", "тва", "ть", "изна",
                ]
                .into_iter()
                .any(|suffix| lemma.ends_with(suffix))
                {
                    return Err(Error::ContradictoryMetadata {
                        reason: "an explicit lexical verbal noun must belong to one of Alypy §27's -ота/-ета/-ба/-ежъ/-нь/-снь/-знь/-тва/-ть/-изна families"
                            .into(),
                    });
                }
            }
        }
        Ok(())
    }
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
    /// Alypy §86: the closed list `ꙗти, начати, вити, пити, клѧти` (and
    /// their prefixed compounds) takes the personal ending `-тъ` in the
    /// second and third singular beside the bare vowel stem (`приѧтъ`,
    /// `начатъ`, `клѧтъ`). Both prints are ordered variants; every other
    /// cell follows the plain vowel-stem series.
    VowelStemWithT,
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
    /// Alypy §93: after a vowel-final present stem the suffix `-и-` passes
    /// into `-й-` in the singular and second plural (`по́й`, `по́йте`,
    /// `сто́й(те)`, `бо́йсѧ`/`бо́йтесѧ`), and the first dual/plural take the
    /// same j-suffix on such second-conjugation j-stems.
    JSeries,
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
    /// Optional independently suppletive base for the simple future. When this
    /// complete triple is absent, perfective verbs reuse the present triple.
    pub future_stem: Option<SynodalWord>,
    /// Complete suppletive simple-future first-person singular.
    pub future_first_singular: Option<SynodalWord>,
    /// Complete suppletive simple-future third-person plural.
    pub future_third_plural: Option<SynodalWord>,
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
    /// Optional masculine-singular base before `л` when the citation form
    /// preserves a mobile vowel absent from the rest of the paradigm (Alypy
    /// §104 `шелъ : шли`).
    pub l_participle_masculine_singular_stem: Option<SynodalWord>,
    pub present_active_participle: Option<ParticiplePrincipalPart>,
    pub past_active_participle: Option<ParticiplePrincipalPart>,
    pub present_passive_participle: Option<ParticiplePrincipalPart>,
    pub past_passive_participle: Option<ParticiplePrincipalPart>,
    pub verbal_noun: Option<VerbalNounPrincipalPart>,
}

impl VerbLexeme {
    fn verbal_noun_is_complete(&self) -> bool {
        match &self.verbal_noun {
            Some(principal_part) => principal_part.validate().is_ok(),
            None => self
                .past_passive_participle
                .as_ref()
                .and_then(|part| part.short_stem.as_ref())
                .is_some_and(|platform| {
                    VerbalNounPrincipalPart::past_passive_ie(platform.canonical()).is_ok()
                }),
        }
    }

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
            VerbSystem::Finite(crate::FiniteTense::Future) => {
                let has_independent_future = self.future_stem.is_some()
                    || self.future_first_singular.is_some()
                    || self.future_third_plural.is_some();
                if has_independent_future {
                    if self.future_stem.is_none() {
                        missing.push(MetadataField::FutureStem);
                    }
                    if self.future_first_singular.is_none() {
                        missing.push(MetadataField::FutureFirstSingular);
                    }
                    if self.future_third_plural.is_none() {
                        missing.push(MetadataField::FutureThirdPlural);
                    }
                } else {
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
            VerbSystem::Finite(crate::FiniteTense::Past) | VerbSystem::Infinitive => {}
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
            // The Russian/Synodal target has no productive supine. An empty
            // list means there is no principal part a caller can add to enable
            // the historically absent category.
            VerbSystem::Supine => {}
            VerbSystem::VerbalNoun { .. } => {
                if !self.verbal_noun_is_complete() {
                    missing.push(MetadataField::VerbalNounStem);
                }
            }
        }
        missing
    }
}

/// Forms and completely declines a source-bounded Synodal verbal noun.
///
/// Alypy §27 licenses `-їе` on a past-passive base; §34 supplies the
/// full soft-neuter paradigm. Other §27 suffix families are accepted only as
/// explicitly specified noun lexemes, preventing arbitrary suffix guessing.
pub fn decline_verbal_noun(
    lexeme: &VerbLexeme,
    cell: crate::NounCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let principal_part = if let Some(principal_part) = &lexeme.verbal_noun {
        principal_part.clone()
    } else if let Some(platform) = lexeme
        .past_passive_participle
        .as_ref()
        .and_then(|part| part.short_stem.as_ref())
    {
        VerbalNounPrincipalPart::past_passive_ie(platform.canonical())?
    } else {
        return Err(Error::MissingPrincipalPart {
            field: MetadataField::VerbalNounStem,
        });
    };
    principal_part.validate()?;

    let (rule, stage, citation) = match principal_part.formation() {
        VerbalNounFormation::PastPassiveIe => (
            "SYN-VERB-VERBAL-NOUN-IE-ALYPY-27",
            "verbal-noun-formation-past-passive-ie",
            "Alypy (Gamanovich), §27 `-їе` from past-passive bases; §34 declension",
        ),
        VerbalNounFormation::ExplicitLexicalNoun => (
            "SYN-VERB-VERBAL-NOUN-LEXICAL-ALYPY-27",
            "verbal-noun-explicit-lexical-formation",
            "Alypy (Gamanovich), §27 lexical deverbal suffix families; noun declension §§34–44",
        ),
    };
    let rule_id = RuleId::from(rule);
    let evidence_id = EvidenceId::from(format!("normative:{rule}"));
    let evidence = Evidence {
        id: evidence_id.clone(),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: citation.into(),
        note: Some(format!("stable rule {rule}")),
    };
    let citation_form = principal_part.noun().lemma.canonical().to_owned();
    let declined = decline_noun(principal_part.noun(), cell, profile)?;
    let mut variants = Vec::<FormVariant>::from(declined);
    for variant in &mut variants {
        variant.source = FormSource::SynodalNormativeGeneration {
            rule: rule_id.clone(),
        };
        variant.evidence.insert(0, evidence.clone());
        let mut steps = vec![TraceStep {
            rule: rule_id.clone(),
            stage: stage.into(),
            input: lexeme.lemma.canonical().into(),
            output: citation_form.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: vec![evidence_id.clone()],
        }];
        steps.extend(variant.rule_trace.steps().iter().cloned());
        variant.rule_trace = RuleTrace::new(steps);
    }
    FormSet::try_from_variants(variants)
}

pub fn present(
    lexeme: &VerbLexeme,
    person: Person,
    number: Number,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let text = present_shape(lexeme, person, number)?;
    normative(
        text,
        "SYN-VERB-PRESENT-ALYPY-80",
        profile,
        "present",
        lexeme.lemma.canonical(),
    )
}

/// Realizes the Alypy §84 simple future. Its morphology is the complete
/// present-shaped person × number paradigm, but only independently classified
/// perfective verbs license that temporal interpretation productively.
pub fn future(
    lexeme: &VerbLexeme,
    person: Person,
    number: Number,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    match lexeme.aspect {
        Aspect::Unknown => {
            return Err(Error::MissingMetadata {
                field: MetadataField::Aspect,
            });
        }
        Aspect::Imperfective | Aspect::Biaspectual => {
            return Err(Error::EvidenceIncompleteCell {
                field: MetadataField::Aspect,
                reason: "Alypy §84 and Pletneva–Kravetsky lesson 13 require contextual or exact evidence before a non-perfective present-shaped form can be typed as future"
                    .into(),
            });
        }
        Aspect::Perfective => {}
    }
    normative(
        future_shape(lexeme, person, number)?,
        "SYN-VERB-FUTURE-PERFECTIVE-ALYPY-84",
        profile,
        "simple-future",
        lexeme.lemma.canonical(),
    )
}

pub(crate) fn present_shape(lexeme: &VerbLexeme, person: Person, number: Number) -> Result<String> {
    finite_shape(
        lexeme,
        person,
        number,
        lexeme.present_stem.as_ref(),
        lexeme.present_first_singular.as_ref(),
        lexeme.present_third_plural.as_ref(),
        [
            MetadataField::PresentStem,
            MetadataField::PresentFirstSingular,
            MetadataField::PresentThirdPlural,
        ],
    )
}

pub(crate) fn future_shape(lexeme: &VerbLexeme, person: Person, number: Number) -> Result<String> {
    let has_independent_future = lexeme.future_stem.is_some()
        || lexeme.future_first_singular.is_some()
        || lexeme.future_third_plural.is_some();
    if has_independent_future {
        finite_shape(
            lexeme,
            person,
            number,
            lexeme.future_stem.as_ref(),
            lexeme.future_first_singular.as_ref(),
            lexeme.future_third_plural.as_ref(),
            [
                MetadataField::FutureStem,
                MetadataField::FutureFirstSingular,
                MetadataField::FutureThirdPlural,
            ],
        )
    } else {
        present_shape(lexeme, person, number)
    }
}

pub(crate) fn finite_shape(
    lexeme: &VerbLexeme,
    person: Person,
    number: Number,
    stem: Option<&SynodalWord>,
    first_singular: Option<&SynodalWord>,
    third_plural: Option<&SynodalWord>,
    fields: [MetadataField; 3],
) -> Result<String> {
    let cell = FiniteVerbCell {
        tense: FiniteTense::Present,
        person,
        number,
    };
    let text = match (person, number) {
        (Person::First, Number::Singular) => {
            required(first_singular, fields[1])?.canonical().to_owned()
        }
        (Person::Third, Number::Plural) => {
            required(third_plural, fields[2])?.canonical().to_owned()
        }
        _ => {
            let stem = required(stem, fields[0])?;
            join(stem.canonical(), present_ending(lexeme.conjugation, cell)?)
        }
    };
    Ok(text)
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
    let rule = match formation {
        AoristFormation::VowelStem => "SYN-VERB-AORIST-VOWEL-ALYPY-86",
        AoristFormation::VowelStemWithT => "SYN-VERB-AORIST-VOWEL-T-ALYPY-86",
        AoristFormation::ConsonantStem => "SYN-VERB-AORIST-CONSONANT-ALYPY-86",
        AoristFormation::Irregular => unreachable!(),
    };
    if formation == AoristFormation::VowelStemWithT
        && matches!(person, Person::Second | Person::Third)
        && number == Number::Singular
    {
        // The bare stem and the -тъ print are both attested; Alypy cites the
        // -тъ shape from the liturgical text, so it leads the ordered pair.
        return normative_variants(
            vec![join(&stem_text, "тъ"), stem_text.clone()],
            rule,
            profile,
            "aorist",
            lexeme.lemma.canonical(),
        );
    }
    normative(
        join(&stem_text, ending),
        rule,
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

/// Rule identifier for the reflexive/passive enclitic (Alypy §73).
pub const REFLEXIVE_RULE_ID: &str = "SYN-VERB-REFLEXIVE-ALYPY-73";

/// Alypy §73: the reflexive, reciprocal, and analytic passive voices are
/// formed "прибавлением к глаголу действительного залога возвратного
/// местоимения -сѧ". The enclitic attaches to every form of the verb, never
/// carries an accent, and a final jer of the host is dropped before it
/// (`клѧ́тъ` + `сѧ` → `клѧ́тсѧ`, `да́стъ` + `сѧ` → `да́стсѧ`; Alypy's own
/// examples `воцари́сѧ`, `ѡ҆блече́сѧ`, `бра́шасѧ`). The rule is purely
/// concatenative on both the expanded and the printed surface, so it is
/// applied after accent realisation and leaves the host's marks untouched.
#[must_use]
pub fn reflexive_surface(base: &str) -> String {
    let host = base.strip_suffix('ъ').unwrap_or(base);
    format!("{host}сѧ")
}

/// The host surfaces a reflexive form could have been built from, most
/// specific first: the form without `сѧ`, and — because the rule deletes a
/// final jer — that form with `ъ` restored when it ends in a consonant.
#[must_use]
pub fn reflexive_base_candidates(surface: &str) -> Vec<String> {
    let Some(host) = surface.strip_suffix("сѧ") else {
        return Vec::new();
    };
    if host.is_empty() {
        return Vec::new();
    }
    let mut candidates = vec![host.to_owned()];
    // Before an enclitic a word-final grave surfaces as an acute
    // (`возвратѝ` in isolation, `возврати́сѧ` with the enclitic), so the
    // isolated host is also sought under the grave.
    if let Some(stripped) = host.strip_suffix('\u{0301}') {
        candidates.push(format!("{stripped}\u{0300}"));
    }
    let ends_in_consonant = host
        .chars()
        .rev()
        .find(|character| {
            !matches!(
                character,
                '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}' | '\u{0486}' | '\u{0485}'
            )
        })
        .is_some_and(|last| {
            !crate::orthography::is_synodal_vowel(last) && last != 'ъ' && last != 'ь' && last != 'й'
        });
    if ends_in_consonant {
        candidates.push(format!("{host}ъ"));
    }
    candidates
}

pub fn l_participle(
    lexeme: &VerbLexeme,
    cell: LParticipleCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let general_stem = required(
        lexeme.l_participle_stem.as_ref(),
        MetadataField::LParticipleStem,
    )?;
    let stem = if cell.number == Number::Singular && cell.gender == Gender::Masculine {
        lexeme
            .l_participle_masculine_singular_stem
            .as_ref()
            .unwrap_or(general_stem)
    } else {
        general_stem
    };
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

pub(crate) fn present_ending(
    conjugation: VerbConjugation,
    cell: FiniteVerbCell,
) -> Result<&'static str> {
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

pub(crate) fn aorist_ending(
    formation: AoristFormation,
    person: Person,
    number: Number,
) -> &'static str {
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

pub(crate) fn imperfect_ending(
    formation: ImperfectFormation,
    person: Person,
    number: Number,
) -> &'static str {
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

pub(crate) fn imperative_ending(
    formation: ImperativeFormation,
    cell: ImperativeCell,
) -> &'static str {
    match (formation, cell.person, cell.number) {
        (ImperativeFormation::JSeries, Person::Second | Person::Third, Number::Singular) => "й",
        (ImperativeFormation::JSeries, Person::First, Number::Dual) => "йва",
        (ImperativeFormation::JSeries, Person::Second, Number::Dual) => "йта",
        (ImperativeFormation::JSeries, Person::First, Number::Plural) => "ймъ",
        (ImperativeFormation::JSeries, Person::Second, Number::Plural) => "йте",
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
