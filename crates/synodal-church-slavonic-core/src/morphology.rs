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
    /// Historical `u`-stem members of the first declension, retaining the
    /// ordered `-ꙋ`, `-ови`, `-ове`, `-ов-`, and `-ми` alternatives described
    /// in Alypy §§37–38 (for example `сынъ` and `домъ`).
    FirstHardMasculineUStem,
    /// Ethnonyms in `-инъ`, whose plural drops `-ин-` and has nominative and
    /// vocative `-е`, for example `галїлеанинъ : галїлеане` (Alypy §37).
    FirstHardMasculineInEthnonym,
    /// The lexeme-specific mixed historical profile of `ꙋдъ`, which keeps
    /// its ordinary first-declension forms and can additionally use an
    /// extended `ꙋдес-` stem after the fourth-declension neuter analogy.
    FirstHardMasculineUdEs,
    /// First declension with a final velar and the reviewed first/second
    /// palatalizations of Alypy §34.
    FirstHardVelarMasculine,
    /// First-declension masculine with a sibilant stem and mixed endings.
    FirstMixedMasculine,
    /// First-declension masculine with a `ц`-final oblique stem and an
    /// independently supplied citation form, including mobile-`е` nouns such
    /// as `младенецъ : младенц-`. Synodal `ц` combines with `ы`/`ъ` but not
    /// `о` (Alypy §8.c); the remaining mixed endings follow §§33–37.
    FirstMixedTsMasculine,
    FirstHardNeuter,
    FirstSoftMasculine,
    /// Agent nouns in `-тель`, retaining the ordinary soft paradigm plus the
    /// `-е` / `-їе` nominative-vocative plural variants of Alypy §37.
    FirstSoftMasculineAgentTel,
    /// The mixed `господь` profile: hard singular obliques, historical
    /// i-stem dual/plural endings, and the lexical vocative `господи`.
    FirstSoftMasculineLord,
    /// Masculine `j`-stems with a surface citation in `-й`, for example
    /// `край : кра-` and `славїй : славї-`.
    FirstSoftMasculineJ,
    /// Masculine nouns in `-ей` with the distinct `їерей` pattern from Alypy
    /// §§34 and 37, including genitive singular `-а` and plural `-є`.
    FirstSoftMasculineEy,
    FirstSoftNeuter,
    /// Soft neuters in `-ище`, whose locative plural admits ordered `-ахъ`,
    /// `-ихъ`, and `-ехъ` variants (Alypy §37).
    FirstSoftNeuterIshche,
    /// Soft neuters in `-їе`, whose dual/plural spelling and endings differ
    /// from the ordinary `море` pattern (for example `знаменїе`).
    FirstSoftNeuterIe,
    SecondHard,
    /// Second declension with a final velar and the §39 palatalization before
    /// `ѣ` in singular dative/locative and dual citation cells.
    SecondHardVelar,
    SecondSoft,
    /// Soft nouns in `-ѧ` after a vowel that retain the ancient `-ѧ`
    /// nominative/accusative plural, for example `молнїѧ` and `ѕмїѧ`.
    SecondSoftPostvocalicAncientPlural,
    /// Masculine names in `-їа`, with the §40 instrumental singular `-емъ`.
    SecondSoftMasculineIa,
    /// Feminine names in `-іа`, with the feminine instrumental singular
    /// `-іею`, for example `маріа : марі-` (Alypy §§32, 39–40).
    SecondSoftFeminineIa,
    /// Second-declension stems ending in a sibilant, with the mixed endings
    /// printed in Alypy §§39–40 (for example `юноша`).
    SecondMixed,
    ThirdFeminine,
    ThirdMasculine,
    /// Fourth-declension neuter whose citation form in `-ѧ` has an oblique
    /// stem in `-ен-`, for example `имѧ : имен-`.
    FourthNeuterEn,
    /// Fourth-declension neuter whose citation form in `-о` has an oblique
    /// stem in `-ес-`, for example `небо : небес-`.
    FourthNeuterEs,
    /// Extended `-ес-` neuters in `-о` that also admit a complete ordinary
    /// first-declension background without `-ес-` (Alypy §44).
    FourthNeuterEsAlternatingFirst,
    /// The paired-body `ѻко` / `ꙋхо` contract from Alypy §44: singular and
    /// plural use the independently supplied `-ес-` stem, while every dual
    /// cell uses its corresponding short `-ч-` / `-ш-` stem.
    FourthNeuterEsPairedDual,
    /// Fourth-declension neuter with an independently supplied extended stem
    /// in `-ат-`, for example `ѻтроча : ѻтрочат-`.
    FourthNeuterAt,
    /// Fourth-declension feminine whose citation form in `-и` has an oblique
    /// stem in `-ер-`, for example `мати : матер-`.
    FourthFeminineEr,
    /// The lexeme-specific modern `дщерь` identity with the historical
    /// nominative/vocative citation `дщи` and oblique `дщер-` stem.
    FourthFeminineErDaughter,
    /// Fourth-declension feminine with an independently supplied oblique stem
    /// in `-ов-` or `-в-`, for example `свекры : свекров-`.
    FourthFeminineOv,
    /// Modern `-овь` members of the `свекры` family whose full `-ов-` and
    /// syncopated `-в-` stems are distributed by cell, for example
    /// `церковь : церков- / церкв-` and `любовь : любов- / любв-`.
    /// `stem` is the independently supplied short `-в-` stem; the full stem
    /// is recoverable without ambiguity from the validated citation form.
    FourthFeminineOvSyncopating,
    /// Fourth-declension masculine with an independently supplied stem in
    /// `-ен-`, for example `степень : степен-`.
    FourthMasculineEn,
    /// The lexeme-specific syncopating paradigm of `день : дн- / ден-`.
    FourthMasculineEnDay,
    /// The lexeme-specific `камень` contract: the ordinary masculine `-ен-`
    /// paradigm plus only the alternatives cited in Alypy §43. The separate
    /// collective `каменїе` is never emitted by this contract.
    FourthMasculineEnKamen,
    /// Borrowed nouns whose supplied lemma is invariant in every licensed
    /// case and number, including the Hebrew names described in Alypy §37.
    Indeclinable,
}

impl NounDeclension {
    pub const ALL: [Self; 38] = [
        Self::FirstHardMasculine,
        Self::FirstHardMasculineUStem,
        Self::FirstHardMasculineInEthnonym,
        Self::FirstHardMasculineUdEs,
        Self::FirstHardVelarMasculine,
        Self::FirstMixedMasculine,
        Self::FirstMixedTsMasculine,
        Self::FirstHardNeuter,
        Self::FirstSoftMasculine,
        Self::FirstSoftMasculineAgentTel,
        Self::FirstSoftMasculineLord,
        Self::FirstSoftMasculineJ,
        Self::FirstSoftMasculineEy,
        Self::FirstSoftNeuter,
        Self::FirstSoftNeuterIshche,
        Self::FirstSoftNeuterIe,
        Self::SecondHard,
        Self::SecondHardVelar,
        Self::SecondSoft,
        Self::SecondSoftPostvocalicAncientPlural,
        Self::SecondSoftMasculineIa,
        Self::SecondSoftFeminineIa,
        Self::SecondMixed,
        Self::ThirdFeminine,
        Self::ThirdMasculine,
        Self::FourthNeuterEn,
        Self::FourthNeuterEs,
        Self::FourthNeuterEsAlternatingFirst,
        Self::FourthNeuterEsPairedDual,
        Self::FourthNeuterAt,
        Self::FourthFeminineEr,
        Self::FourthFeminineErDaughter,
        Self::FourthFeminineOv,
        Self::FourthFeminineOvSyncopating,
        Self::FourthMasculineEn,
        Self::FourthMasculineEnDay,
        Self::FourthMasculineEnKamen,
        Self::Indeclinable,
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
    /// Hard adjective with a final velar stem. Its complete positive
    /// paradigm has the ending spellings and palatalizations printed
    /// separately in Alypy §57 (`благїй : блазїи`).
    Velar,
    /// Short possessives in `-ов-` / `-ев-` (Alypy §§50 and 53). Any
    /// independently attested compound-form cell remains a lexical exact
    /// override rather than licensing a long paradigm for every member.
    PossessiveHard,
    /// Short possessives in palatal `-ь` / `-ень` (Alypy §§50 and 53).
    /// A mobile vowel is supplied through the ordinary typed short-masculine
    /// principal part rather than inferred from spelling.
    PossessiveSoft,
    /// Possessives in `-їй`, whose `-ї-` belongs to the derivational suffix
    /// and whose complete predominantly short declension is printed
    /// separately in Alypy §56. That section also licenses occasional
    /// compound forms.
    PossessiveIi,
}

impl AdjectiveClass {
    pub const ALL: [Self; 6] = [
        Self::Hard,
        Self::Soft,
        Self::Velar,
        Self::PossessiveHard,
        Self::PossessiveSoft,
        Self::PossessiveIi,
    ];
}

/// Source-defined relation between the ordinary positive stem and the stem
/// used before the short masculine citation ending.
///
/// The alternant remains an independently supplied principal part. The enum
/// validates why it differs, preventing a caller from smuggling an arbitrary
/// irregular stem through either productive rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ShortMasculineStemFormation {
    /// Alypy §52: a full/general `-нн-` stem loses one `н` at the short
    /// masculine citation edge (`блаженн- : блаженъ`).
    DoubleNReduction,
    /// A mobile `е` appears before the final stem consonant at the short
    /// masculine citation edge (`преподобн- : преподобенъ`).
    MobileEInsertion,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AdjectiveLexeme {
    pub lemma: SynodalWord,
    pub stem: SynodalWord,
    pub class: AdjectiveClass,
    /// Independently supplied positive stem used before the short masculine
    /// citation ending. It must be paired with `short_masculine_formation`.
    pub short_masculine_stem: Option<SynodalWord>,
    pub short_masculine_formation: Option<ShortMasculineStemFormation>,
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
                NounDeclension::FourthFeminineEr
                    | NounDeclension::FourthFeminineErDaughter
                    | NounDeclension::FourthFeminineOv
                    | NounDeclension::FourthFeminineOvSyncopating
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

    if lexeme.declension == NounDeclension::Indeclinable {
        return Ok(vec![lexeme.lemma.canonical().to_owned()]);
    }
    if lexeme.declension == NounDeclension::FirstMixedTsMasculine
        && cell.number == Sg
        && matches!(cell.case, Nom | Acc)
        && (cell.case == Nom || cell.animacy == Animacy::Inanimate)
    {
        return Ok(vec![lexeme.lemma.canonical().to_owned()]);
    }
    if lexeme.declension == NounDeclension::FirstMixedTsMasculine
        && cell.number == Sg
        && cell.case == Voc
    {
        let stem = lexeme.stem.canonical();
        let palatalized = stem
            .strip_suffix('ц')
            .map_or_else(|| stem.to_owned(), |prefix| join(prefix, "ч"));
        return Ok(vec![join(&palatalized, "е")]);
    }
    if lexeme.declension == NounDeclension::FirstHardMasculineUdEs {
        let short_stem = lexeme.lemma.canonical().strip_suffix('ъ').ok_or_else(|| {
            Error::ContradictoryMetadata {
                reason: "the ꙋдъ mixed profile requires a citation in -ъ".into(),
            }
        })?;
        let ordinary = NounLexeme::new(
            lexeme.lemma.clone(),
            SynodalWord::parse(short_stem)?,
            Gender::Masculine,
            NounDeclension::FirstHardMasculine,
        );
        let mut surfaces = noun_surfaces(&ordinary, cell)?;
        if !matches!((cell.number, cell.case), (Sg, Nom | Acc | Voc)) {
            let extended = NounLexeme::new(
                SynodalWord::parse(join(short_stem, "о"))?,
                lexeme.stem.clone(),
                Gender::Neuter,
                NounDeclension::FourthNeuterEs,
            );
            surfaces.extend(noun_surfaces(&extended, cell)?);
            surfaces.dedup();
        }
        return Ok(surfaces);
    }
    if lexeme.declension == NounDeclension::FourthMasculineEnDay
        && cell.number == Sg
        && cell.case == Acc
    {
        return Ok(vec![if cell.animacy == Animacy::Animate {
            "дне".to_owned()
        } else {
            lexeme.lemma.canonical().to_owned()
        }]);
    }

    let citation_form = matches!(
        (lexeme.declension, cell.number, cell.case),
        (
            NounDeclension::FourthNeuterEn
                | NounDeclension::FourthNeuterEs
                | NounDeclension::FourthNeuterEsAlternatingFirst
                | NounDeclension::FourthNeuterEsPairedDual
                | NounDeclension::FourthNeuterAt,
            Sg,
            Nom | Acc | Voc
        ) | (
            NounDeclension::FourthFeminineEr
                | NounDeclension::FourthFeminineErDaughter
                | NounDeclension::FourthFeminineOv
                | NounDeclension::FourthFeminineOvSyncopating,
            Sg,
            Nom | Voc
        ) | (
            NounDeclension::FourthMasculineEn
                | NounDeclension::FourthMasculineEnDay
                | NounDeclension::FourthMasculineEnKamen,
            Sg,
            Nom | Voc
        )
    );
    if citation_form {
        if lexeme.declension == NounDeclension::FourthFeminineErDaughter {
            let short_stem = lexeme
                .stem
                .canonical()
                .strip_suffix("ер")
                .unwrap_or_default();
            return Ok(vec![join(short_stem, "и")]);
        }
        if lexeme.declension == NounDeclension::FourthFeminineOvSyncopating && cell.case == Voc {
            return Ok(vec![
                lexeme.lemma.canonical().to_owned(),
                join(lexeme.stem.canonical(), "е"),
            ]);
        }
        return Ok(vec![lexeme.lemma.canonical().to_owned()]);
    }

    let stem = noun_stem(lexeme, cell);
    let mut surfaces = noun_endings(lexeme, cell)?
        .into_iter()
        .map(|ending| join(&stem, ending))
        .collect::<Vec<_>>();
    if lexeme.declension == NounDeclension::FourthNeuterEsAlternatingFirst {
        let short_stem = lexeme.stem.canonical().strip_suffix("ес").ok_or_else(|| {
            Error::ContradictoryMetadata {
                reason: "alternating -ес- neuters require an extended stem in -ес-".into(),
            }
        })?;
        let ordinary = NounLexeme::new(
            lexeme.lemma.clone(),
            SynodalWord::parse(short_stem)?,
            Gender::Neuter,
            NounDeclension::FirstHardNeuter,
        );
        surfaces.extend(noun_surfaces(&ordinary, cell)?);
        surfaces.dedup();
    }
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
    if lexeme.declension == NounDeclension::FourthNeuterEsPairedDual
        && cell.number == Number::Dual
        && matches!(cell.case, Nom | Acc | Voc)
        && lexeme
            .stem
            .canonical()
            .strip_suffix("ес")
            .is_some_and(|stem| stem.ends_with('ч'))
    {
        if let Some(short_stem) = lexeme.stem.canonical().strip_suffix("ес") {
            let mut alternative = short_stem.to_owned();
            alternative.pop();
            alternative.push('ц');
            surfaces.push(join(&alternative, "ѣ"));
        }
    }
    if lexeme.declension == NounDeclension::FourthNeuterEsPairedDual
        && cell.number == Sg
        && cell.case == crate::Case::Locative
    {
        if let Some(short_stem) = lexeme.lemma.canonical().strip_suffix('о') {
            surfaces.push(join(&second_palatalize_final_velar(short_stem), "ѣ"));
        }
    }
    if lexeme.declension == NounDeclension::FirstSoftNeuterIe
        && cell.number == Number::Plural
        && cell.case == crate::Case::Instrumental
    {
        if let Some(short_stem) = lexeme
            .stem
            .canonical()
            .strip_suffix('ї')
            .or_else(|| lexeme.stem.canonical().strip_suffix('і'))
        {
            surfaces.push(join(short_stem, "ьми"));
            surfaces.push(join(short_stem, "ми"));
        }
    }
    surfaces.dedup();
    Ok(surfaces)
}

fn noun_stem(lexeme: &NounLexeme, cell: crate::NounCell) -> String {
    use Case::{Accusative as Acc, Nominative as Nom, Vocative as Voc};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};

    let stem = lexeme.stem.canonical();
    match lexeme.declension {
        NounDeclension::FirstHardMasculineInEthnonym if cell.number == Pl => {
            stem.strip_suffix("ин").unwrap_or(stem).to_owned()
        }
        NounDeclension::FourthMasculineEnDay
            if matches!(
                (cell.number, cell.case),
                (Du, crate::Case::Dative | crate::Case::Instrumental)
                    | (Pl, crate::Case::Instrumental)
            ) =>
        {
            "ден".to_owned()
        }
        NounDeclension::FirstHardVelarMasculine => match (cell.number, cell.case) {
            (Sg, Voc) => palatalize_final_velar(stem),
            (Sg, crate::Case::Locative) | (Pl, Nom | Voc | crate::Case::Locative) => {
                second_palatalize_final_velar(stem)
            }
            _ => stem.to_owned(),
        },
        NounDeclension::SecondHardVelar => match (cell.number, cell.case) {
            (Sg, crate::Case::Dative | crate::Case::Locative) | (Du, Nom | Acc | Voc) => {
                second_palatalize_final_velar(stem)
            }
            _ => stem.to_owned(),
        },
        NounDeclension::SecondMixed
            if matches!((cell.number, cell.case), (Du, Nom | Acc | Voc)) =>
        {
            last_o_as_omega(stem)
        }
        NounDeclension::FirstSoftMasculineEy
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, crate::Case::Genitive | Acc | crate::Case::Instrumental)
            ) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FirstSoftNeuterIe
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, Nom | Acc | Voc)
            ) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FourthNeuterEn
        | NounDeclension::FourthNeuterEs
        | NounDeclension::FourthNeuterEsAlternatingFirst
            if matches!((cell.number, cell.case), (Du, Nom | Acc | Voc)) =>
        {
            last_e_as_wide_e(stem)
        }
        NounDeclension::FourthNeuterAt
            if matches!((cell.number, cell.case), (Du, Nom | Acc | Voc)) =>
        {
            last_o_as_omega(stem)
        }
        NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter
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
        NounDeclension::FourthNeuterEsPairedDual if cell.number == Du => {
            stem.strip_suffix("ес").unwrap_or(stem).to_owned()
        }
        NounDeclension::FourthFeminineOvSyncopating => {
            let use_full_stem = matches!(
                (cell.number, cell.case),
                (Sg, Acc | crate::Case::Instrumental)
                    | (Du, crate::Case::Genitive | crate::Case::Locative)
            );
            let selected = if use_full_stem {
                lexeme
                    .lemma
                    .canonical()
                    .strip_suffix('ь')
                    .unwrap_or(lexeme.lemma.canonical())
            } else {
                stem
            };
            if matches!(
                (cell.number, cell.case),
                (
                    Du,
                    Nom | Acc | Voc | crate::Case::Genitive | crate::Case::Locative
                ) | (Pl, Nom | Voc)
            ) {
                last_e_as_wide_e(selected)
            } else {
                selected.to_owned()
            }
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

fn positive_adjective_surface(lexeme: &AdjectiveLexeme, cell: AdjectiveCell) -> Result<String> {
    if lexeme.class == AdjectiveClass::Velar {
        return velar_positive_adjective_surface(lexeme, cell);
    }
    let ending = match cell.form {
        AdjectiveForm::Short => short_adjective_ending(lexeme.class, cell)?,
        AdjectiveForm::Long => long_adjective_ending(lexeme.class, cell)?,
    };
    let stem = if cell.form == AdjectiveForm::Short
        && cell.number == Number::Singular
        && cell.gender == Gender::Masculine
        && matches!(ending, "ъ" | "ь")
    {
        lexeme.short_masculine_stem.as_ref().unwrap_or(&lexeme.stem)
    } else {
        &lexeme.stem
    };
    Ok(join(stem.canonical(), ending))
}

fn velar_positive_adjective_surface(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
) -> Result<String> {
    let mut stem = lexeme.stem.canonical().to_owned();
    let ending = match cell.form {
        AdjectiveForm::Short => {
            let hard = short_adjective_ending(AdjectiveClass::Hard, cell)?;
            if hard == "ѣ"
                || matches!(
                    (cell.number, cell.gender, cell.case),
                    (
                        Number::Plural,
                        Gender::Masculine,
                        Case::Nominative | Case::Vocative
                    )
                )
            {
                stem = second_palatalize_final_velar(&stem);
            } else if hard == "е" {
                stem = palatalize_final_velar(&stem);
            }
            match hard {
                "ы" => "и",
                "ымъ" => "имъ",
                "ыма" => "има",
                "ыхъ" => "ихъ",
                "ыми" => "ими",
                other => other,
            }
        }
        AdjectiveForm::Long => {
            if matches!(
                (cell.number, cell.gender, cell.case),
                (
                    Number::Singular,
                    Gender::Feminine,
                    Case::Dative | Case::Locative
                ) | (
                    Number::Singular,
                    Gender::Masculine | Gender::Neuter,
                    Case::Locative
                ) | (
                    Number::Dual,
                    Gender::Feminine | Gender::Neuter,
                    Case::Nominative | Case::Accusative | Case::Vocative
                ) | (
                    Number::Plural,
                    Gender::Masculine,
                    Case::Nominative | Case::Vocative
                )
            ) {
                stem = second_palatalize_final_velar(&stem);
            }
            velar_long_adjective_ending(cell)?
        }
    };
    Ok(join(&stem, ending))
}

pub fn decline_adjective(
    lexeme: &AdjectiveLexeme,
    cell: AdjectiveCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    validate_adjective_lexeme(lexeme)?;
    if matches!(
        lexeme.class,
        AdjectiveClass::PossessiveHard
            | AdjectiveClass::PossessiveSoft
            | AdjectiveClass::PossessiveIi
    ) && cell.comparison != Comparison::Positive
    {
        return Err(Error::HistoricallyInvalidCell {
            reason: "possessive adjectives do not license comparison".into(),
        });
    }
    if matches!(
        lexeme.class,
        AdjectiveClass::PossessiveHard | AdjectiveClass::PossessiveSoft
    ) && cell.form == AdjectiveForm::Long
    {
        return Err(Error::HistoricallyInvalidCell {
            reason: "this possessive suffix licenses only the short paradigm; exceptional compound forms require exact lexical evidence".into(),
        });
    }
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
        let stem = lexeme
            .comparative_stem
            .as_ref()
            .ok_or(Error::MissingPrincipalPart {
                field: MetadataField::ComparisonStem,
            })?;
        let formation = lexeme.comparison_formation.ok_or(Error::MissingMetadata {
            field: MetadataField::ComparisonFormation,
        })?;
        return decline_short_superlative_predicate(lexeme, stem, formation, cell, profile);
    }
    let (mut expanded, rule) = match cell.comparison {
        Comparison::Positive => (
            vec![positive_adjective_surface(lexeme, cell)?],
            match (lexeme.class, cell.form) {
                (AdjectiveClass::Hard, AdjectiveForm::Short) => "SYN-ADJ-SHORT-HARD-ALYPY-53",
                (AdjectiveClass::Soft, AdjectiveForm::Short) => "SYN-ADJ-SHORT-SOFT-ALYPY-53",
                (AdjectiveClass::Velar, AdjectiveForm::Short) => "SYN-ADJ-SHORT-VELAR-ALYPY-53-57",
                (AdjectiveClass::Hard, AdjectiveForm::Long) => "SYN-ADJ-LONG-HARD-ALYPY-57",
                (AdjectiveClass::Soft, AdjectiveForm::Long) => "SYN-ADJ-LONG-SOFT-ALYPY-57",
                (AdjectiveClass::Velar, AdjectiveForm::Long) => "SYN-ADJ-LONG-VELAR-ALYPY-57",
                (AdjectiveClass::PossessiveHard, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-OV-EV-SHORT-ALYPY-50-53"
                }
                (AdjectiveClass::PossessiveSoft, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-SOFT-SHORT-ALYPY-50-53"
                }
                (AdjectiveClass::PossessiveIi, AdjectiveForm::Short) => {
                    "SYN-ADJ-POSSESSIVE-II-SHORT-ALYPY-56"
                }
                (AdjectiveClass::PossessiveIi, AdjectiveForm::Long) => {
                    "SYN-ADJ-POSSESSIVE-II-LONG-ALYPY-56"
                }
                (
                    AdjectiveClass::PossessiveHard | AdjectiveClass::PossessiveSoft,
                    AdjectiveForm::Long,
                ) => unreachable!("long possessive cells are rejected above"),
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
                vec![join(
                    stem.canonical(),
                    comparison_long_adjective_ending(cell)?,
                )],
                match cell.comparison {
                    Comparison::Comparative => "SYN-ADJ-COMPARATIVE-LONG-ALYPY-58",
                    Comparison::Superlative => "SYN-ADJ-SUPERLATIVE-LONG-ALYPY-59",
                    Comparison::Positive => unreachable!(),
                },
            )
        }
    };
    if cell.case == Case::Accusative && cell.animacy == Animacy::Animate {
        let nominative_cell = AdjectiveCell {
            animacy: Animacy::Inanimate,
            ..cell
        };
        let nominative_like = match cell.comparison {
            Comparison::Positive => positive_adjective_surface(lexeme, nominative_cell)?,
            Comparison::Comparative | Comparison::Superlative => join(
                lexeme
                    .comparative_stem
                    .as_ref()
                    .ok_or(Error::MissingPrincipalPart {
                        field: MetadataField::ComparisonStem,
                    })?
                    .canonical(),
                comparison_long_adjective_ending(nominative_cell)?,
            ),
        };
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

fn present_shape(lexeme: &VerbLexeme, person: Person, number: Number) -> Result<String> {
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

fn future_shape(lexeme: &VerbLexeme, person: Person, number: Number) -> Result<String> {
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

fn finite_shape(
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
        "SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-60",
        profile,
        "short-comparison-declension",
        lexeme.lemma.canonical(),
    )
}

fn decline_short_superlative_predicate(
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

fn decline_short_active_participle(
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

fn decline_short_comparison_stem(
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

fn short_comparison_ending(cell: AdjectiveCell) -> Result<&'static str> {
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

fn decline_short_active_participle_stem(
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

fn short_active_participle_ending(cell: AdjectiveCell) -> Result<&'static str> {
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
        (
            NounDeclension::FirstHardMasculine
                | NounDeclension::FirstHardMasculineUStem
                | NounDeclension::FirstHardMasculineInEthnonym
                | NounDeclension::FirstHardMasculineUdEs,
            Gender::Masculine,
        ) | (NounDeclension::FirstHardVelarMasculine, Gender::Masculine)
            | (
                NounDeclension::FirstMixedMasculine | NounDeclension::FirstMixedTsMasculine,
                Gender::Masculine,
            )
            | (NounDeclension::FirstHardNeuter, Gender::Neuter)
            | (
                NounDeclension::FirstSoftMasculine
                    | NounDeclension::FirstSoftMasculineAgentTel
                    | NounDeclension::FirstSoftMasculineLord
                    | NounDeclension::FirstSoftMasculineJ
                    | NounDeclension::FirstSoftMasculineEy,
                Gender::Masculine,
            )
            | (
                NounDeclension::FirstSoftNeuter
                    | NounDeclension::FirstSoftNeuterIshche
                    | NounDeclension::FirstSoftNeuterIe,
                Gender::Neuter,
            )
            | (
                NounDeclension::SecondHard
                    | NounDeclension::SecondHardVelar
                    | NounDeclension::SecondSoft
                    | NounDeclension::SecondSoftPostvocalicAncientPlural
                    | NounDeclension::SecondMixed,
                Gender::Feminine | Gender::Masculine
            )
            | (NounDeclension::SecondSoftMasculineIa, Gender::Masculine)
            | (NounDeclension::SecondSoftFeminineIa, Gender::Feminine)
            | (NounDeclension::ThirdFeminine, Gender::Feminine)
            | (NounDeclension::ThirdMasculine, Gender::Masculine)
            | (
                NounDeclension::FourthNeuterEn
                    | NounDeclension::FourthNeuterEs
                    | NounDeclension::FourthNeuterEsAlternatingFirst
                    | NounDeclension::FourthNeuterEsPairedDual
                    | NounDeclension::FourthNeuterAt,
                Gender::Neuter
            )
            | (
                NounDeclension::FourthFeminineEr
                    | NounDeclension::FourthFeminineErDaughter
                    | NounDeclension::FourthFeminineOv
                    | NounDeclension::FourthFeminineOvSyncopating,
                Gender::Feminine
            )
            | (
                NounDeclension::FourthMasculineEn
                    | NounDeclension::FourthMasculineEnDay
                    | NounDeclension::FourthMasculineEnKamen,
                Gender::Masculine
            )
            | (NounDeclension::Indeclinable, _)
    );
    if !valid {
        return Err(Error::ContradictoryMetadata {
            reason: "declension and lexical gender are incompatible".into(),
        });
    }
    let lemma = lexeme.lemma.canonical();
    let stem = lexeme.stem.canonical();
    let valid_shape = match lexeme.declension {
        NounDeclension::FirstHardMasculineUStem => lemma.ends_with('ъ'),
        NounDeclension::FirstHardMasculineInEthnonym => {
            lemma.strip_suffix('ъ').is_some_and(|base| base == stem) && stem.ends_with("ин")
        }
        NounDeclension::FirstHardMasculineUdEs => lemma == "ꙋдъ" && stem == "ꙋдес",
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
        NounDeclension::FirstMixedTsMasculine => {
            let citation_stem = lemma.strip_suffix('ъ').unwrap_or_default();
            let direct = citation_stem == stem;
            let mobile_e = stem
                .strip_suffix('ц')
                .is_some_and(|prefix| citation_stem == format!("{prefix}ец"));
            stem.ends_with('ц') && (direct || mobile_e)
        }
        NounDeclension::FirstSoftMasculineJ => {
            lemma.strip_suffix('й').is_some_and(|prefix| prefix == stem) && !lemma.ends_with("ей")
        }
        NounDeclension::FirstSoftMasculineEy => {
            lemma.strip_suffix('й').is_some_and(|prefix| prefix == stem) && stem.ends_with('е')
        }
        NounDeclension::FirstSoftNeuterIe => {
            lemma.strip_suffix('е').is_some_and(|prefix| prefix == stem)
                && (stem.ends_with('ї') || stem.ends_with('і'))
        }
        NounDeclension::FirstSoftMasculineAgentTel => {
            lemma.strip_suffix('ь').is_some_and(|prefix| prefix == stem) && stem.ends_with("тел")
        }
        NounDeclension::FirstSoftMasculineLord => lemma == "господь" && stem == "господ",
        NounDeclension::FirstSoftNeuterIshche => {
            lemma.strip_suffix('е').is_some_and(|prefix| prefix == stem) && stem.ends_with("ищ")
        }
        NounDeclension::SecondHardVelar => {
            lemma.ends_with('а')
                && stem
                    .chars()
                    .last()
                    .is_some_and(|final_char| matches!(final_char, 'г' | 'к' | 'х'))
        }
        NounDeclension::SecondMixed => {
            lemma.ends_with('а')
                && stem
                    .chars()
                    .last()
                    .is_some_and(|final_char| matches!(final_char, 'ж' | 'ч' | 'ш' | 'щ' | 'ц'))
        }
        NounDeclension::SecondSoftPostvocalicAncientPlural => {
            lemma.ends_with('ѧ')
                && lemma.strip_suffix('ѧ').is_some_and(|prefix| prefix == stem)
                && stem.chars().last().is_some_and(|character| {
                    matches!(
                        character,
                        'а' | 'е'
                            | 'є'
                            | 'и'
                            | 'і'
                            | 'ї'
                            | 'о'
                            | 'ѡ'
                            | 'ꙋ'
                            | 'ѹ'
                            | 'ы'
                            | 'ѣ'
                            | 'ѧ'
                            | 'ю'
                            | 'ѵ'
                    )
                })
        }
        NounDeclension::SecondSoftMasculineIa | NounDeclension::SecondSoftFeminineIa => {
            lemma.strip_suffix('а').is_some_and(|prefix| prefix == stem)
                && (stem.ends_with('ї') || stem.ends_with('і'))
        }
        NounDeclension::FourthNeuterEn => lemma.ends_with('ѧ') && stem.ends_with("ен"),
        NounDeclension::FourthNeuterEs | NounDeclension::FourthNeuterEsAlternatingFirst => {
            lemma.strip_suffix('о').is_some_and(|short| {
                stem.strip_suffix("ес")
                    .is_some_and(|extended_short| extended_short == short)
            })
        }
        NounDeclension::FourthNeuterEsPairedDual => {
            lemma.ends_with('о')
                && stem
                    .strip_suffix("ес")
                    .is_some_and(|short| short.ends_with('ч') || short.ends_with('ш'))
        }
        NounDeclension::FourthNeuterAt => {
            (lemma.ends_with('а') || lemma.ends_with('ѧ')) && stem.ends_with("ат")
        }
        NounDeclension::FourthFeminineEr => lemma.ends_with('и') && stem.ends_with("ер"),
        NounDeclension::FourthFeminineErDaughter => lemma == "дщерь" && stem == "дщер",
        NounDeclension::FourthFeminineOv => {
            (lemma.ends_with('ы') || lemma.ends_with('ь'))
                && (stem.ends_with("ов") || stem.ends_with('в'))
                && !matches!(lemma, "любовь" | "любы")
        }
        NounDeclension::FourthFeminineOvSyncopating => lemma
            .strip_suffix("овь")
            .is_some_and(|prefix| stem == format!("{prefix}в")),
        NounDeclension::FourthMasculineEn => {
            lemma.ends_with("ень") && stem.ends_with("ен") && lemma != "камень"
        }
        NounDeclension::FourthMasculineEnDay => lemma == "день" && stem == "дн",
        NounDeclension::FourthMasculineEnKamen => lemma == "камень" && stem == "камен",
        NounDeclension::Indeclinable => !lemma.is_empty() && lemma == stem,
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

/// Validates source-defined positive-stem allomorphy independently of any
/// lexical registry. This is intentionally available to explicit callers as
/// well as the bundled facade.
pub fn validate_adjective_lexeme(lexeme: &AdjectiveLexeme) -> Result<()> {
    if lexeme.class == AdjectiveClass::Velar
        && !lexeme
            .stem
            .canonical()
            .chars()
            .last()
            .is_some_and(|character| matches!(character, 'г' | 'к' | 'х'))
    {
        return Err(Error::ContradictoryMetadata {
            reason: "a velar adjective requires a stem ending in г, к, or х".into(),
        });
    }
    if lexeme.short_masculine_stem.is_some() != lexeme.short_masculine_formation.is_some() {
        return Err(Error::ContradictoryMetadata {
            reason: "short masculine stem and typed formation must be supplied together".into(),
        });
    }
    if let (Some(short), Some(formation)) = (
        &lexeme.short_masculine_stem,
        lexeme.short_masculine_formation,
    ) {
        let stem = lexeme.stem.canonical();
        let valid = match formation {
            ShortMasculineStemFormation::DoubleNReduction => stem
                .strip_suffix('н')
                .is_some_and(|single_n| stem.ends_with("нн") && single_n == short.canonical()),
            ShortMasculineStemFormation::MobileEInsertion => stem
                .char_indices()
                .last()
                .is_some_and(|(offset, final_character)| {
                    short.canonical() == format!("{}е{final_character}", &stem[..offset])
                }),
        };
        if !valid {
            return Err(Error::ContradictoryMetadata {
                reason: format!(
                    "short masculine stem {:?} does not match {:?} from {:?}",
                    short.canonical(),
                    formation,
                    stem
                ),
            });
        }
    }
    Ok(())
}

fn noun_rule(declension: NounDeclension) -> &'static str {
    match declension {
        NounDeclension::FirstHardMasculine => "SYN-NOUN-I-HARD-M-ALYPY-34",
        NounDeclension::FirstHardMasculineUStem => "SYN-NOUN-I-U-STEM-M-ALYPY-37-38",
        NounDeclension::FirstHardMasculineInEthnonym => "SYN-NOUN-I-HARD-M-IN-ETHNONYM-ALYPY-37",
        NounDeclension::FirstHardMasculineUdEs => "SYN-NOUN-I-M-UD-ES-ALYPY-44",
        NounDeclension::FirstHardVelarMasculine => "SYN-NOUN-I-HARD-VELAR-M-ALYPY-34",
        NounDeclension::FirstMixedMasculine => "SYN-NOUN-I-MIXED-M-ALYPY-33-34",
        NounDeclension::FirstMixedTsMasculine => "SYN-NOUN-I-MIXED-TS-M-ALYPY-8-33-37",
        NounDeclension::FirstHardNeuter => "SYN-NOUN-I-HARD-N-ALYPY-34",
        NounDeclension::FirstSoftMasculine => "SYN-NOUN-I-SOFT-M-ALYPY-34",
        NounDeclension::FirstSoftMasculineAgentTel => "SYN-NOUN-I-SOFT-M-TEL-AGENT-ALYPY-37",
        NounDeclension::FirstSoftMasculineLord => "SYN-NOUN-I-SOFT-M-LORD-ALYPY-35-41",
        NounDeclension::FirstSoftMasculineJ => "SYN-NOUN-I-SOFT-J-M-ALYPY-34-37",
        NounDeclension::FirstSoftMasculineEy => "SYN-NOUN-I-SOFT-EY-M-ALYPY-34-37",
        NounDeclension::FirstSoftNeuter => "SYN-NOUN-I-SOFT-N-ALYPY-34",
        NounDeclension::FirstSoftNeuterIshche => "SYN-NOUN-I-SOFT-N-ISHCHE-ALYPY-37",
        NounDeclension::FirstSoftNeuterIe => "SYN-NOUN-I-SOFT-IE-N-ALYPY-34-37",
        NounDeclension::SecondHard => "SYN-NOUN-II-HARD-ALYPY-39",
        NounDeclension::SecondHardVelar => "SYN-NOUN-II-HARD-VELAR-ALYPY-39-40",
        NounDeclension::SecondSoft => "SYN-NOUN-II-SOFT-ALYPY-39",
        NounDeclension::SecondSoftPostvocalicAncientPlural => {
            "SYN-NOUN-II-SOFT-POSTVOCALIC-ANCIENT-PL-ALYPY-40"
        }
        NounDeclension::SecondSoftMasculineIa => "SYN-NOUN-II-SOFT-M-IA-ALYPY-39-40",
        NounDeclension::SecondSoftFeminineIa => "SYN-NOUN-II-SOFT-F-IA-ALYPY-32-39-40",
        NounDeclension::SecondMixed => "SYN-NOUN-II-MIXED-ALYPY-39-40",
        NounDeclension::ThirdFeminine => "SYN-NOUN-III-F-ALYPY-41",
        NounDeclension::ThirdMasculine => "SYN-NOUN-III-M-ALYPY-41",
        NounDeclension::FourthNeuterEn => "SYN-NOUN-IV-N-EN-ALYPY-42-43",
        NounDeclension::FourthNeuterEs => "SYN-NOUN-IV-N-ES-ALYPY-42-43",
        NounDeclension::FourthNeuterEsAlternatingFirst => "SYN-NOUN-IV-N-ES-ALT-FIRST-ALYPY-42-44",
        NounDeclension::FourthNeuterEsPairedDual => "SYN-NOUN-IV-N-ES-PAIRED-DUAL-ALYPY-44",
        NounDeclension::FourthNeuterAt => "SYN-NOUN-IV-N-AT-ALYPY-42-43",
        NounDeclension::FourthFeminineEr => "SYN-NOUN-IV-F-ER-ALYPY-42-43",
        NounDeclension::FourthFeminineErDaughter => "SYN-NOUN-IV-F-ER-DAUGHTER-ALYPY-42-44",
        NounDeclension::FourthFeminineOv => "SYN-NOUN-IV-F-OV-ALYPY-42-44",
        NounDeclension::FourthFeminineOvSyncopating => "SYN-NOUN-IV-F-OV-SYNCOPATING-ALYPY-42-44",
        NounDeclension::FourthMasculineEn => "SYN-NOUN-IV-M-EN-ALYPY-42-44",
        NounDeclension::FourthMasculineEnDay => "SYN-NOUN-IV-M-EN-DAY-ALYPY-43",
        NounDeclension::FourthMasculineEnKamen => "SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43",
        NounDeclension::Indeclinable => "SYN-NOUN-INDECLINABLE-ALYPY-37",
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
    if lexeme.declension == NounDeclension::FirstMixedTsMasculine {
        return Ok(match (cell.number, cell.case) {
            (Sg, Nom) => vec!["ъ"],
            (Sg, Gen) => vec!["а"],
            (Sg, Dat) => vec!["ꙋ", "еви"],
            (Sg, Acc) => vec![animate_acc("ъ", "а")],
            (Sg, Ins) => vec!["емъ"],
            (Sg, Loc) => vec!["и", "ѣ"],
            (Sg, Voc) => vec!["е"],
            (Du, Nom | Acc | Voc) => vec!["а"],
            (Du, Gen | Loc) => vec!["ꙋ"],
            (Du, Dat | Ins) => vec!["ема"],
            (Pl, Nom | Voc) => vec!["ы"],
            (Pl, Gen) => vec!["євъ"],
            (Pl, Dat) => vec!["ємъ"],
            (Pl, Acc) => vec![animate_acc("ы", "євъ")],
            (Pl, Ins) => vec!["ы", "ьми", "ами"],
            (Pl, Loc) => vec!["ѣхъ"],
        });
    }
    if lexeme.declension == NounDeclension::Indeclinable {
        return Ok(vec![""]);
    }
    if lexeme.declension == NounDeclension::FirstHardMasculineInEthnonym && cell.number == Pl {
        return Ok(vec![match cell.case {
            Nom | Voc => "е",
            Gen => "ъ",
            Dat => "омъ",
            Acc => animate_acc("е", "ъ"),
            Ins => "ы",
            Loc => "ѣхъ",
        }]);
    }
    if lexeme.declension == NounDeclension::FirstSoftMasculineLord {
        return Ok(match (cell.number, cell.case) {
            (Sg, Nom) => vec!["ь"],
            (Sg, Gen) => vec!["а"],
            (Sg, Dat) => vec!["ꙋ", "еви"],
            (Sg, Acc) => vec![animate_acc("ь", "а")],
            (Sg, Ins) => vec!["омъ"],
            (Sg, Loc) => vec!["ѣ"],
            (Sg, Voc) => vec!["и"],
            (Du, Nom | Acc | Voc) => vec!["и"],
            (Du, Gen | Loc) => vec!["їю", "ю"],
            (Du, Dat | Ins) => vec!["ьма"],
            (Pl, Nom | Voc) => vec!["їе"],
            (Pl, Gen) => vec!["ій", "ей"],
            (Pl, Dat) => vec!["ємъ"],
            (Pl, Acc) => vec![animate_acc("и", "ій")],
            (Pl, Ins) => vec!["ьми"],
            (Pl, Loc) => vec!["ехъ"],
        });
    }
    if lexeme.declension == NounDeclension::FirstSoftNeuterIshche
        && matches!((cell.number, cell.case), (Pl, Loc))
    {
        return Ok(vec!["ахъ", "ихъ", "ехъ"]);
    }
    if lexeme.declension == NounDeclension::FourthMasculineEnDay
        && matches!((cell.number, cell.case), (Du, Gen | Loc))
    {
        return Ok(vec!["їю", "ю"]);
    }
    let base_declension = match lexeme.declension {
        NounDeclension::FirstHardMasculineInEthnonym | NounDeclension::FirstHardMasculineUdEs => {
            NounDeclension::FirstHardMasculine
        }
        NounDeclension::FirstMixedTsMasculine => NounDeclension::FirstMixedMasculine,
        NounDeclension::FirstSoftMasculineAgentTel => NounDeclension::FirstSoftMasculine,
        NounDeclension::FirstSoftNeuterIshche => NounDeclension::FirstSoftNeuter,
        NounDeclension::FourthNeuterEsAlternatingFirst => NounDeclension::FourthNeuterEs,
        NounDeclension::FourthMasculineEnDay => NounDeclension::FourthMasculineEn,
        declension => declension,
    };
    let ending = match (base_declension, cell.number, cell.case) {
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Sg, Nom) => {
            "ъ"
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Sg, Gen) => {
            "а"
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Sg, Dat) => {
            "ꙋ"
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Sg, Acc) => {
            animate_acc("ъ", "а")
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Sg, Ins) => {
            "омъ"
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Sg, Loc) => {
            "ѣ"
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Sg, Voc) => {
            "е"
        }
        (
            NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem,
            Du,
            Nom | Acc | Voc,
        ) => "а",
        (
            NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem,
            Du,
            Gen | Loc,
        ) => "ꙋ",
        (
            NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem,
            Du,
            Dat | Ins,
        ) => "ома",
        (
            NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem,
            Pl,
            Nom | Voc,
        ) => "и",
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Pl, Gen) => {
            "овъ"
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Pl, Dat) => {
            "омъ"
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Pl, Acc) => {
            animate_acc("ы", "овъ")
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Pl, Ins) => {
            "ы"
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardMasculineUStem, Pl, Loc) => {
            "ѣхъ"
        }

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

        (NounDeclension::FirstSoftMasculineJ, Sg, Nom) => "й",
        (NounDeclension::FirstSoftMasculineJ, Sg, Gen) => "ѧ",
        (NounDeclension::FirstSoftMasculineJ, Sg, Dat) => "ю",
        (NounDeclension::FirstSoftMasculineJ, Sg, Acc) => animate_acc("й", "ѧ"),
        (NounDeclension::FirstSoftMasculineJ, Sg, Ins) => "емъ",
        (NounDeclension::FirstSoftMasculineJ, Sg, Loc) => "и",
        (NounDeclension::FirstSoftMasculineJ, Sg, Voc) => {
            if lexeme.stem.canonical().ends_with('ї') {
                "е"
            } else {
                "ю"
            }
        }
        (NounDeclension::FirstSoftMasculineJ, Du, Nom | Acc | Voc) => "ѧ",
        (NounDeclension::FirstSoftMasculineJ, Du, Gen | Loc) => "ю",
        (NounDeclension::FirstSoftMasculineJ, Du, Dat | Ins) => "ема",
        (NounDeclension::FirstSoftMasculineJ, Pl, Nom | Voc) => "и",
        (NounDeclension::FirstSoftMasculineJ, Pl, Gen) => "євъ",
        (NounDeclension::FirstSoftMasculineJ, Pl, Dat) => "ємъ",
        (NounDeclension::FirstSoftMasculineJ, Pl, Acc) => animate_acc("и", "євъ"),
        (NounDeclension::FirstSoftMasculineJ, Pl, Ins) => "и",
        (NounDeclension::FirstSoftMasculineJ, Pl, Loc) => "ехъ",

        (NounDeclension::FirstSoftMasculineEy, Sg, Nom) => "й",
        (NounDeclension::FirstSoftMasculineEy, Sg, Gen) => "а",
        (NounDeclension::FirstSoftMasculineEy, Sg, Dat) => "ю",
        (NounDeclension::FirstSoftMasculineEy, Sg, Acc) => animate_acc("й", "а"),
        (NounDeclension::FirstSoftMasculineEy, Sg, Ins) => "емъ",
        (NounDeclension::FirstSoftMasculineEy, Sg, Loc) => "и",
        (NounDeclension::FirstSoftMasculineEy, Sg, Voc) => "ю",
        (NounDeclension::FirstSoftMasculineEy, Du, Nom | Acc | Voc) => "а",
        (NounDeclension::FirstSoftMasculineEy, Du, Gen | Loc) => "ю",
        (NounDeclension::FirstSoftMasculineEy, Du, Dat | Ins) => "ема",
        (NounDeclension::FirstSoftMasculineEy, Pl, Nom | Voc) => "є",
        (NounDeclension::FirstSoftMasculineEy, Pl, Gen) => "й",
        (NounDeclension::FirstSoftMasculineEy, Pl, Dat) => "ємъ",
        (NounDeclension::FirstSoftMasculineEy, Pl, Acc) => animate_acc("и", "й"),
        (NounDeclension::FirstSoftMasculineEy, Pl, Ins) => "и",
        (NounDeclension::FirstSoftMasculineEy, Pl, Loc) => "ехъ",

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

        (NounDeclension::FirstSoftNeuterIe, Sg, Nom | Acc | Voc) => "е",
        (NounDeclension::FirstSoftNeuterIe, Sg, Gen) => "ѧ",
        (NounDeclension::FirstSoftNeuterIe, Sg, Dat) => "ю",
        (NounDeclension::FirstSoftNeuterIe, Sg, Ins) => "емъ",
        (NounDeclension::FirstSoftNeuterIe, Sg, Loc) => "и",
        (NounDeclension::FirstSoftNeuterIe, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::FirstSoftNeuterIe, Du, Gen | Loc) => "ю",
        (NounDeclension::FirstSoftNeuterIe, Du, Dat | Ins) => "ема",
        (NounDeclension::FirstSoftNeuterIe, Pl, Nom | Acc | Voc) => "ѧ",
        (NounDeclension::FirstSoftNeuterIe, Pl, Gen) => "й",
        (NounDeclension::FirstSoftNeuterIe, Pl, Dat) => "ємъ",
        (NounDeclension::FirstSoftNeuterIe, Pl, Ins) => "и",
        (NounDeclension::FirstSoftNeuterIe, Pl, Loc) => "ихъ",

        (NounDeclension::SecondHardVelar, Sg, Gen) => "и",
        (NounDeclension::SecondHardVelar, Pl, Nom | Voc) => "и",
        (NounDeclension::SecondHardVelar, Pl, Acc) => animate_acc("и", "ъ"),
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Sg, Nom) => "а",
        (NounDeclension::SecondHard, Sg, Gen) => "ы",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Sg, Dat | Loc) => "ѣ",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Sg, Acc) => "ꙋ",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Sg, Ins) => "ою",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Sg, Voc) => "о",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Du, Nom | Acc | Voc) => "ѣ",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Du, Gen | Loc) => "ꙋ",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Du, Dat | Ins) => "ама",
        (NounDeclension::SecondHard, Pl, Nom | Voc) => "ы",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Pl, Gen) => "ъ",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Pl, Dat) => "амъ",
        (NounDeclension::SecondHard, Pl, Acc) => animate_acc("ы", "ъ"),
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Pl, Ins) => "ами",
        (NounDeclension::SecondHard | NounDeclension::SecondHardVelar, Pl, Loc) => "ахъ",

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

        (NounDeclension::SecondSoftPostvocalicAncientPlural, Sg, Nom) => "ѧ",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Sg, Gen | Dat | Loc) => "и",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Sg, Acc) => "ю",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Sg, Ins) => "ею",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Sg, Voc) => "е",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Du, Gen | Loc) => "ю",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Du, Dat | Ins) => "ѧма",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Pl, Nom | Voc) => "ѧ",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Pl, Acc) => animate_acc("ѧ", "й"),
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Pl, Gen) => "й",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Pl, Dat) => "ѧмъ",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Pl, Ins) => "ѧми",
        (NounDeclension::SecondSoftPostvocalicAncientPlural, Pl, Loc) => "ѧхъ",

        (NounDeclension::SecondSoftMasculineIa, Sg, Nom) => "а",
        (NounDeclension::SecondSoftMasculineIa, Sg, Gen | Dat | Loc) => "и",
        (NounDeclension::SecondSoftMasculineIa, Sg, Acc) => "ю",
        (NounDeclension::SecondSoftMasculineIa, Sg, Ins) => "емъ",
        (NounDeclension::SecondSoftMasculineIa, Sg, Voc) => "е",
        (NounDeclension::SecondSoftMasculineIa, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::SecondSoftMasculineIa, Du, Gen | Loc) => "ю",
        (NounDeclension::SecondSoftMasculineIa, Du, Dat | Ins) => "ѧма",
        (NounDeclension::SecondSoftMasculineIa, Pl, Nom | Voc) => "и",
        (NounDeclension::SecondSoftMasculineIa, Pl, Gen) => "й",
        (NounDeclension::SecondSoftMasculineIa, Pl, Dat) => "ѧмъ",
        (NounDeclension::SecondSoftMasculineIa, Pl, Acc) => animate_acc("и", "й"),
        (NounDeclension::SecondSoftMasculineIa, Pl, Ins) => "ѧми",
        (NounDeclension::SecondSoftMasculineIa, Pl, Loc) => "ѧхъ",

        (NounDeclension::SecondSoftFeminineIa, Sg, Nom) => "а",
        (NounDeclension::SecondSoftFeminineIa, Sg, Gen | Dat | Loc) => "и",
        (NounDeclension::SecondSoftFeminineIa, Sg, Acc) => "ю",
        (NounDeclension::SecondSoftFeminineIa, Sg, Ins) => "ею",
        (NounDeclension::SecondSoftFeminineIa, Sg, Voc) => "е",
        (NounDeclension::SecondSoftFeminineIa, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::SecondSoftFeminineIa, Du, Gen | Loc) => "ю",
        (NounDeclension::SecondSoftFeminineIa, Du, Dat | Ins) => "ѧма",
        (NounDeclension::SecondSoftFeminineIa, Pl, Nom | Voc) => "и",
        (NounDeclension::SecondSoftFeminineIa, Pl, Gen) => "й",
        (NounDeclension::SecondSoftFeminineIa, Pl, Dat) => "ѧмъ",
        (NounDeclension::SecondSoftFeminineIa, Pl, Acc) => animate_acc("и", "й"),
        (NounDeclension::SecondSoftFeminineIa, Pl, Ins) => "ѧми",
        (NounDeclension::SecondSoftFeminineIa, Pl, Loc) => "ѧхъ",

        (NounDeclension::SecondMixed, Sg, Nom) => "а",
        (NounDeclension::SecondMixed, Sg, Gen | Dat | Loc) => "и",
        (NounDeclension::SecondMixed, Sg, Acc) => "ꙋ",
        (NounDeclension::SecondMixed, Sg, Ins) => "ею",
        (NounDeclension::SecondMixed, Sg, Voc) => "е",
        (NounDeclension::SecondMixed, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::SecondMixed, Du, Gen | Loc) => "ꙋ",
        (NounDeclension::SecondMixed, Du, Dat | Ins) => "ама",
        (NounDeclension::SecondMixed, Pl, Nom | Voc) => "и",
        (NounDeclension::SecondMixed, Pl, Gen) => "ъ",
        (NounDeclension::SecondMixed, Pl, Dat) => "амъ",
        (NounDeclension::SecondMixed, Pl, Acc) => animate_acc("ы", "ъ"),
        (NounDeclension::SecondMixed, Pl, Ins) => "ами",
        (NounDeclension::SecondMixed, Pl, Loc) => "ахъ",

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

        (NounDeclension::FourthNeuterEsPairedDual, Du, Nom | Acc | Voc) => "и",
        (NounDeclension::FourthNeuterEsPairedDual, Du, Gen | Loc) => "їю",
        (NounDeclension::FourthNeuterEsPairedDual, Du, Dat | Ins) => "има",

        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterEsPairedDual
            | NounDeclension::FourthNeuterAt,
            Sg,
            Gen,
        ) => "е",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterEsPairedDual
            | NounDeclension::FourthNeuterAt,
            Sg,
            Dat | Loc,
        ) => "и",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterEsPairedDual
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
            | NounDeclension::FourthNeuterEsPairedDual
            | NounDeclension::FourthNeuterAt,
            Pl,
            Nom | Acc | Voc,
        ) => "а",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterEsPairedDual
            | NounDeclension::FourthNeuterAt,
            Pl,
            Gen,
        ) => "ъ",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterEsPairedDual
            | NounDeclension::FourthNeuterAt,
            Pl,
            Dat,
        ) => "ємъ",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterEsPairedDual
            | NounDeclension::FourthNeuterAt,
            Pl,
            Ins,
        ) => "ы",
        (
            NounDeclension::FourthNeuterEn
            | NounDeclension::FourthNeuterEs
            | NounDeclension::FourthNeuterEsPairedDual
            | NounDeclension::FourthNeuterAt,
            Pl,
            Loc,
        ) => "ѣхъ",

        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Sg, Gen) => {
            "е"
        }
        (
            NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter,
            Sg,
            Dat | Loc,
        ) => "и",
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Sg, Acc) => {
            "ь"
        }
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Sg, Ins) => {
            "їю"
        }
        (
            NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter,
            Du,
            Nom | Acc | Voc,
        ) => "и",
        (
            NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter,
            Du,
            Gen | Loc,
        ) => "їю",
        (
            NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter,
            Du,
            Dat | Ins,
        ) => "ема",
        (
            NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter,
            Pl,
            Nom | Voc,
        ) => "и",
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Pl, Gen) => {
            "їй"
        }
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Pl, Dat) => {
            "емъ"
        }
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Pl, Acc) => {
            animate_acc("и", "ей")
        }
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Pl, Ins) => {
            "ьми"
        }
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Pl, Loc) => {
            "ехъ"
        }

        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Sg,
            Gen,
        ) => "е",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Sg,
            Dat | Loc,
        ) => "и",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Sg,
            Acc,
        ) => "ь",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Sg,
            Ins,
        ) => "їю",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Du,
            Nom | Acc | Voc,
        ) => "и",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Du,
            Gen | Loc,
        ) => "їю",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Du,
            Dat | Ins,
        ) => "ама",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Pl,
            Nom | Voc,
        ) => "и",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Pl,
            Gen,
        ) => "ей",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Pl,
            Dat,
        ) => "амъ",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Pl,
            Acc,
        ) => animate_acc("и", "ей"),
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Pl,
            Ins,
        ) => "ами",
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Pl,
            Loc,
        ) => "ахъ",

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
            | NounDeclension::FourthNeuterEsPairedDual
            | NounDeclension::FourthNeuterAt,
            Sg,
            Nom | Acc | Voc,
        )
        | (
            NounDeclension::FourthFeminineEr
            | NounDeclension::FourthFeminineErDaughter
            | NounDeclension::FourthFeminineOv
            | NounDeclension::FourthFeminineOvSyncopating,
            Sg,
            Nom | Voc,
        )
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
        _ => {
            return Err(Error::ContradictoryMetadata {
                reason: format!("unmapped noun declension base {base_declension:?}"),
            });
        }
    };
    let mut endings = vec![ending];
    match (lexeme.declension, cell.number, cell.case) {
        (
            NounDeclension::FirstHardMasculine
            | NounDeclension::FirstHardMasculineUStem
            | NounDeclension::FirstHardMasculineInEthnonym
            | NounDeclension::FirstHardVelarMasculine,
            Sg,
            Dat,
        ) => endings.push("ови"),
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardVelarMasculine, Pl, Gen) => {
            endings.push("ъ")
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardVelarMasculine, Pl, Ins) => {
            endings.extend(["ми", "ами"])
        }
        (NounDeclension::FirstHardMasculine | NounDeclension::FirstHardVelarMasculine, Pl, Loc) => {
            endings.push("ахъ")
        }
        (NounDeclension::FirstHardNeuter, Pl, Ins) => endings.push("ами"),
        (NounDeclension::FirstHardNeuter, Pl, Loc) => endings.push("ахъ"),
        (NounDeclension::FirstMixedMasculine, Sg, Dat) => endings.push("еви"),
        (NounDeclension::FirstMixedMasculine, Sg, Loc) => endings.push("ѣ"),
        (NounDeclension::FirstMixedMasculine, Pl, Ins) => endings.extend(["ьми", "ами"]),
        (
            NounDeclension::FirstSoftMasculine | NounDeclension::FirstSoftMasculineAgentTel,
            Sg,
            Dat,
        ) => endings.push("еви"),
        (
            NounDeclension::FirstSoftMasculine | NounDeclension::FirstSoftMasculineAgentTel,
            Sg,
            Loc,
        ) => endings.push("ѣ"),
        (NounDeclension::FirstSoftMasculine, Pl, Nom | Voc) => endings.push("їе"),
        (NounDeclension::FirstSoftMasculineAgentTel, Pl, Nom | Voc) => endings.extend(["е", "їе"]),
        (
            NounDeclension::FirstSoftMasculine | NounDeclension::FirstSoftMasculineAgentTel,
            Pl,
            Ins,
        ) => endings.extend(["ьми", "ами"]),
        (
            NounDeclension::FirstSoftMasculine | NounDeclension::FirstSoftMasculineAgentTel,
            Pl,
            Loc,
        ) => endings.push("ѧхъ"),
        (NounDeclension::FirstSoftMasculineJ, Sg, Loc) => endings.push("ѣ"),
        (NounDeclension::FirstSoftMasculineEy, Sg, Dat) => endings.push("ови"),
        (NounDeclension::FirstSoftMasculineEy, Sg, Loc) => endings.push("ѣ"),
        (NounDeclension::FirstSoftNeuter | NounDeclension::FirstSoftNeuterIshche, Pl, Ins) => {
            endings.extend(["ьми", "ами"])
        }
        (NounDeclension::FirstHardMasculineUStem, Sg, Gen | Loc) => endings.push("ꙋ"),
        (NounDeclension::FirstHardMasculineUStem, Pl, Nom | Voc) => endings.push("ове"),
        (NounDeclension::FirstHardMasculineUStem, Pl, Dat) => endings.push("овомъ"),
        (NounDeclension::FirstHardMasculineUStem, Pl, Ins) => endings.push("ми"),
        (NounDeclension::FirstHardMasculineUStem, Pl, Loc) => endings.extend(["овѣхъ", "ахъ"]),
        (NounDeclension::FirstSoftMasculineEy, Sg, Voc) => endings.push("е"),
        (NounDeclension::FirstSoftMasculineEy, Du, Dat | Ins) => endings.push("ома"),
        (NounDeclension::FirstSoftMasculineEy, Pl, Dat) => endings.push("ѡмъ"),
        (NounDeclension::FourthMasculineEnDay, Sg, Dat) => endings.push("еви"),
        (NounDeclension::FourthMasculineEnDay, Pl, Nom | Voc) => endings.push("іе"),
        (NounDeclension::FourthMasculineEnDay, Pl, Gen) => endings.push("ей"),
        (NounDeclension::FirstMixedMasculine, Pl, Nom | Voc) => endings.push("їе"),
        (NounDeclension::SecondMixed, Sg, Dat) => endings.push("ѣ"),
        (NounDeclension::ThirdFeminine, Du, Dat | Ins) => endings.push("ьма"),
        (NounDeclension::ThirdMasculine, Sg, Voc) => endings.push("ю"),
        (NounDeclension::ThirdMasculine, Pl, Gen) => endings.push("ей"),
        (NounDeclension::FourthNeuterEn, Du, Dat | Ins) => endings.push("ама"),
        (NounDeclension::FourthNeuterEn, Pl, Dat) => endings.push("ѡмъ"),
        (NounDeclension::FourthNeuterAt, Du, Dat | Ins) => endings.push("ама"),
        (NounDeclension::FourthNeuterAt, Pl, Dat) => endings.push("ѡмъ"),
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Pl, Gen) => {
            endings.push("ей")
        }
        (NounDeclension::FourthFeminineEr | NounDeclension::FourthFeminineErDaughter, Pl, Acc)
            if cell.animacy == Animacy::Animate =>
        {
            endings.push("и");
        }
        (
            NounDeclension::FourthFeminineOv | NounDeclension::FourthFeminineOvSyncopating,
            Pl,
            Acc,
        ) if cell.animacy == Animacy::Animate => {
            endings.push("и");
        }
        (NounDeclension::FourthMasculineEnKamen, Du, Dat | Ins) => endings.push("ема"),
        _ => {}
    }
    Ok(endings)
}

fn short_adjective_ending(class: AdjectiveClass, cell: AdjectiveCell) -> Result<&'static str> {
    match class {
        AdjectiveClass::Soft | AdjectiveClass::PossessiveSoft => {
            return soft_short_adjective_ending(cell);
        }
        AdjectiveClass::PossessiveIi => return possessive_ii_short_ending(cell),
        AdjectiveClass::Hard | AdjectiveClass::Velar | AdjectiveClass::PossessiveHard => {}
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

fn possessive_ii_short_ending(cell: AdjectiveCell) -> Result<&'static str> {
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
        (Sg, M, Gen) => "їѧ",
        (Sg, M, Dat) => "їю",
        (Sg, M, Acc) => animate("їй", "їѧ"),
        (Sg, M, Ins) => "їимъ",
        (Sg, M, Loc) => "їи",
        (Sg, F, Nom | Gen | Voc) => "їѧ",
        (Sg, F, Dat | Loc) => "їи",
        (Sg, F, Acc) => "їю",
        (Sg, F, Ins) => "їею",
        (Sg, N, Nom | Acc | Voc) => "їе",
        (Sg, N, Gen) => "їѧ",
        (Sg, N, Dat) => "їю",
        (Sg, N, Ins) => "їимъ",
        (Sg, N, Loc) => "їи",
        (Du, M, Nom | Acc | Voc) => "їѧ",
        (Du, F | N, Nom | Acc | Voc) => "їи",
        (Du, _, Gen | Loc) => "їю",
        (Du, _, Dat | Ins) => "їима",
        (Pl, M, Nom | Voc) => "їи",
        (Pl, F | N, Nom | Voc) => "їѧ",
        (Pl, _, Gen | Loc) => "їихъ",
        (Pl, _, Dat) => "їимъ",
        (Pl, M | F, Acc) => animate(if cell.gender == M { "їи" } else { "їѧ" }, "їихъ"),
        (Pl, N, Acc) => "їѧ",
        (Pl, M | N, Ins) => "їи",
        (Pl, F, Ins) => "їими",
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

pub(crate) fn long_adjective_ending(
    class: AdjectiveClass,
    cell: AdjectiveCell,
) -> Result<&'static str> {
    match class {
        AdjectiveClass::Soft => return soft_long_adjective_ending(cell),
        AdjectiveClass::Velar => return velar_long_adjective_ending(cell),
        AdjectiveClass::PossessiveIi => return possessive_ii_long_ending(cell),
        AdjectiveClass::PossessiveHard | AdjectiveClass::PossessiveSoft => {
            return Err(Error::HistoricallyInvalidCell {
                reason: "this possessive suffix has no productive long paradigm".into(),
            });
        }
        AdjectiveClass::Hard => {}
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

/// Occasional compound forms of the `-їй` class (Alypy §56). Direct-case,
/// dual, and plural cells are syncretic with the short table; singular
/// obliques take the source-licensed pronominal extensions, including the
/// explicitly cited `божїѧгѡ`.
fn possessive_ii_long_ending(cell: AdjectiveCell) -> Result<&'static str> {
    use Case::{
        Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins, Locative as Loc,
        Nominative as Nom, Vocative as Voc,
    };
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::Singular as Sg;
    if cell.number != Sg {
        return possessive_ii_short_ending(cell);
    }
    let animate = |nominative, genitive| {
        if cell.animacy == Animacy::Animate {
            genitive
        } else {
            nominative
        }
    };
    Ok(match (cell.gender, cell.case) {
        (M, Nom | Voc) => "їй",
        (M, Gen) => "їѧгѡ",
        (M, Dat) => "їемꙋ",
        (M, Acc) => animate("їй", "їѧго"),
        (M, Ins) => "їимъ",
        (M, Loc) => "їемъ",
        (F, Nom | Voc) => "їѧ",
        (F, Gen | Dat | Loc) => "їей",
        (F, Acc) => "їю",
        (F, Ins) => "їею",
        (N, Nom | Acc | Voc) => "їе",
        (N, Gen) => "їѧгѡ",
        (N, Dat) => "їемꙋ",
        (N, Ins) => "їимъ",
        (N, Loc) => "їемъ",
    })
}

fn velar_long_adjective_ending(cell: AdjectiveCell) -> Result<&'static str> {
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
        (Sg, M, Dat) => "омꙋ",
        (Sg, M, Acc) => animate("їй", "аго"),
        (Sg, M, Ins) => "имъ",
        (Sg, M, Loc) => "ѣмъ",
        (Sg, F, Nom | Voc) => "аѧ",
        (Sg, F, Gen) => "їѧ",
        (Sg, F, Dat | Loc) => "ѣй",
        (Sg, F, Acc) => "ꙋю",
        (Sg, F, Ins) => "ою",
        (Sg, N, Nom | Acc | Voc) => "ое",
        (Sg, N, Gen) => "агѡ",
        (Sg, N, Dat) => "омꙋ",
        (Sg, N, Ins) => "имъ",
        (Sg, N, Loc) => "ѣмъ",
        (Du, M, Nom | Acc | Voc) => "аѧ",
        (Du, F | N, Nom | Acc | Voc) => "ѣи",
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

pub(crate) fn normative_variants(
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
        "SYN-NOUN-I-MIXED-TS-M-ALYPY-8-33-37" => "Alypy (Gamanovich), §§8 and 33–37",
        "SYN-NOUN-I-U-STEM-M-ALYPY-37-38" => "Alypy (Gamanovich), §§37–38",
        "SYN-NOUN-I-HARD-M-IN-ETHNONYM-ALYPY-37"
        | "SYN-NOUN-I-SOFT-M-TEL-AGENT-ALYPY-37"
        | "SYN-NOUN-I-SOFT-N-ISHCHE-ALYPY-37"
        | "SYN-NOUN-INDECLINABLE-ALYPY-37" => "Alypy (Gamanovich), §37",
        "SYN-NOUN-I-SOFT-M-LORD-ALYPY-35-41" => "Alypy (Gamanovich), §§35, 38, and 41",
        "SYN-NOUN-I-M-UD-ES-ALYPY-44" => "Alypy (Gamanovich), §44 ꙋдъ : ꙋдес-",
        "SYN-NOUN-I-SOFT-J-M-ALYPY-34-37"
        | "SYN-NOUN-I-SOFT-EY-M-ALYPY-34-37"
        | "SYN-NOUN-I-SOFT-IE-N-ALYPY-34-37" => "Alypy (Gamanovich), §§34–37",
        "SYN-NOUN-II-HARD-ALYPY-39" | "SYN-NOUN-II-SOFT-ALYPY-39" => {
            "Alypy (Gamanovich), §§39–40, 44"
        }
        "SYN-NOUN-II-HARD-VELAR-ALYPY-39-40" | "SYN-NOUN-II-MIXED-ALYPY-39-40" => {
            "Alypy (Gamanovich), §§39–40"
        }
        "SYN-NOUN-II-SOFT-POSTVOCALIC-ANCIENT-PL-ALYPY-40"
        | "SYN-NOUN-II-SOFT-M-IA-ALYPY-39-40" => "Alypy (Gamanovich), §§39–40",
        "SYN-NOUN-II-SOFT-F-IA-ALYPY-32-39-40" => "Alypy (Gamanovich), §§32 and 39–40",
        "SYN-NOUN-III-F-ALYPY-41" | "SYN-NOUN-III-M-ALYPY-41" => "Alypy (Gamanovich), §41",
        "SYN-NOUN-IV-N-EN-ALYPY-42-43"
        | "SYN-NOUN-IV-N-ES-ALYPY-42-43"
        | "SYN-NOUN-IV-N-AT-ALYPY-42-43"
        | "SYN-NOUN-IV-F-ER-ALYPY-42-43" => "Alypy (Gamanovich), §§42–43",
        "SYN-NOUN-IV-N-ES-PAIRED-DUAL-ALYPY-44" => "Alypy (Gamanovich), §44 ѻко/ꙋхо",
        "SYN-NOUN-IV-N-ES-ALT-FIRST-ALYPY-42-44" => {
            "Alypy (Gamanovich), §§42–44 -ес- / first-declension alternatives"
        }
        "SYN-NOUN-IV-F-ER-DAUGHTER-ALYPY-42-44" => "Alypy (Gamanovich), §§42–44 дщи : дщер-",
        "SYN-NOUN-IV-F-OV-ALYPY-42-44"
        | "SYN-NOUN-IV-F-OV-SYNCOPATING-ALYPY-42-44"
        | "SYN-NOUN-IV-M-EN-ALYPY-42-44" => "Alypy (Gamanovich), §§42–44",
        "SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43" => "Alypy (Gamanovich), §43 камень notes",
        "SYN-NOUN-IV-M-EN-DAY-ALYPY-43" => "Alypy (Gamanovich), §43 день table",
        "SYN-ADJ-SHORT-HARD-ALYPY-53" | "SYN-ADJ-SHORT-SOFT-ALYPY-53" => {
            "Alypy (Gamanovich), §§53–55"
        }
        "SYN-ADJ-SHORT-VELAR-ALYPY-53-57" => "Alypy (Gamanovich), §§53–57",
        "SYN-ADJ-LONG-HARD-ALYPY-57" | "SYN-ADJ-LONG-SOFT-ALYPY-57" => {
            "Alypy (Gamanovich), §§56–57"
        }
        "SYN-ADJ-LONG-VELAR-ALYPY-57" => "Alypy (Gamanovich), §57 velar table",
        "SYN-ADJ-COMPARATIVE-LONG-ALYPY-58" => "Alypy (Gamanovich), §58",
        "SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-60" => {
            "Alypy (Gamanovich), §§58 and 60 short-comparison declension"
        }
        "SYN-ADJ-SUPERLATIVE-LONG-ALYPY-59" => "Alypy (Gamanovich), §59",
        "SYN-ADJ-SUPERLATIVE-SHORT-PREDICATE-ALYPY-59-60-125-128" => {
            "Alypy (Gamanovich), §§59–60, 125, and 128"
        }
        "SYN-DETERMINER-HARD-ALYPY-45-48" => {
            "Alypy (Gamanovich), §§45 and 48 short and full determinative pronouns"
        }
        "SYN-DETERMINER-VES-MIXED-ALYPY-45-48" => {
            "Alypy (Gamanovich), §§45 and 48.7 весь mixed paradigm and no-dual restriction"
        }
        "SYN-DETERMINER-VSYAK-MIXED-ALYPY-45-48-57" => {
            "Alypy (Gamanovich), §§45, 48, and 57 всѧкъ/всѧкїй paradigms"
        }
        "SYN-DETERMINER-FULL-SK-ALYPY-45-57" => {
            "Alypy (Gamanovich), §§45 and 57 full -скїй declension and -ск-/-ст- alternation"
        }
        "SYN-NUMERAL-CARDINAL-ONE-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-TWO-BOTH-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-THREE-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-FOUR-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-I-STEM-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-TEN-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-HUNDRED-ALYPY-62"
        | "SYN-NUMERAL-CARDINAL-MAGNITUDE-NOUN-ALYPY-61-62" => {
            "Alypy (Gamanovich), §§61–62 simple cardinal and magnitude paradigms"
        }
        "SYN-NUMERAL-ORDINAL-ADJECTIVAL-ALYPY-68" => {
            "Alypy (Gamanovich), §68 ordinal formation and full-adjective declension"
        }
        "SYN-NUMERAL-COLLECTIVE-AGREEING-ALYPY-69"
        | "SYN-NUMERAL-COLLECTIVE-GOVERNING-ALYPY-69"
        | "SYN-NUMERAL-COLLECTIVE-HARD-PLURAL-ALYPY-69" => {
            "Alypy (Gamanovich), §69 collective numeral inventories and government"
        }
        "SYN-NUMERAL-MULTIPLICATIVE-ADJECTIVAL-ALYPY-61-70"
        | "SYN-NUMERAL-FRACTIONAL-NOUN-ALYPY-61-70" => {
            "Alypy (Gamanovich), §§61 and 70 multiplicative and fractional numerals"
        }
        "SYN-NUMERAL-FRACTIONAL-ADJECTIVAL-ALYPY-51-TARGET" => {
            "Alypy (Gamanovich), §51 full hard-adjective declension; Synodal Bible, III Esdras 14:11–12 полдесѧтый"
        }
        "SYN-PRONOUN-PERSONAL-FIRST-ALYPY-47"
        | "SYN-PRONOUN-PERSONAL-SECOND-ALYPY-47"
        | "SYN-PRONOUN-REFLEXIVE-ALYPY-47" => "Alypy (Gamanovich), §47 first group",
        "SYN-PRONOUN-THIRD-PERSON-ALYPY-46-47" => {
            "Alypy (Gamanovich), §§46–47 third-person paradigm"
        }
        "SYN-PRONOUN-SEI-ALYPY-45-48" => "Alypy (Gamanovich), §§45–48 сей/сій paradigm",
        "SYN-PRONOUN-SOFT-ALYPY-47-48" | "SYN-PRONOUN-HARD-ALYPY-47-48" => {
            "Alypy (Gamanovich), §§47–48 pronominal declension"
        }
        "SYN-PRONOUN-SOFT-I-ALTERNATING-ALYPY-45-48" => {
            "Alypy (Gamanovich), §§45–48 чій paradigm and і/ї spelling"
        }
        "SYN-PRONOUN-MIXED-POSSESSIVE-ALYPY-48" => {
            "Alypy (Gamanovich), §48 mixed possessive declension"
        }
        "SYN-PRONOUN-KII-ALYPY-48" => "Alypy (Gamanovich), §48 two-base кій paradigm",
        "SYN-PRONOUN-SHORT-HARD-ALYPY-48" => "Alypy (Gamanovich), §48 short-pronoun paradigm",
        "SYN-PRONOUN-SHORT-OV-MIXED-ALYPY-48" => {
            "Alypy (Gamanovich), §48 compound -ов- mixed paradigm"
        }
        "SYN-PRONOUN-SHORT-VELAR-ALYPY-48" | "SYN-PRONOUN-QUANTITY-VELAR-ALYPY-48" => {
            "Alypy (Gamanovich), §48 velar and quantity pronouns"
        }
        "SYN-PRONOUN-FULL-HARD-ALYPY-48-57"
        | "SYN-PRONOUN-FULL-SOFT-ALYPY-48-57"
        | "SYN-PRONOUN-FULL-VELAR-ALYPY-48-57" => {
            "Alypy (Gamanovich), §§48 and 57 full adjectival pronouns"
        }
        "SYN-PRONOUN-KTO-ALYPY-48" | "SYN-PRONOUN-CHTO-ALYPY-48" => {
            "Alypy (Gamanovich), §48 interrogative paradigms"
        }
        "SYN-PRONOUN-DERIVED-ALYPY-46-48" => "Alypy (Gamanovich), §§46–48 derived pronouns",
        "SYN-PRONOUN-NEGATIVE-PREPOSITION-ALYPY-48" => {
            "Alypy (Gamanovich), §48 negative-pronoun preposition interposition"
        }
        "SYN-PRONOUN-ENCLITIC-PROSODY-ALYPY-47" => {
            "Alypy (Gamanovich), §47 short-pronoun enclisis and accent"
        }
        "SYN-PRONOUN-THIRD-PREPOSITION-CONTRACTION-ALYPY-47" => {
            "Alypy (Gamanovich), §47 на(н)и/въ(н)и contractions"
        }
        "SYN-VERB-PRESENT-ALYPY-80" => "Alypy (Gamanovich), §§79–80",
        "SYN-VERB-FUTURE-PERFECTIVE-ALYPY-84" => {
            "Alypy (Gamanovich), §84 simple future of perfective verbs"
        }
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
    fn alpy_37_38_complete_u_stem_background_is_bounded() {
        let son = NounLexeme::new(
            word("сынъ"),
            word("сын"),
            Gender::Masculine,
            NounDeclension::FirstHardMasculineUStem,
        );
        assert_noun_paradigm(
            &son,
            Animacy::Animate,
            &[
                &["сынъ"],
                &["сына", "сынꙋ"],
                &["сынꙋ", "сынови"],
                &["сына", "сынъ"],
                &["сыномъ"],
                &["сынѣ", "сынꙋ"],
                &["сыне"],
                &["сына"],
                &["сынꙋ"],
                &["сынома"],
                &["сына"],
                &["сынома"],
                &["сынꙋ"],
                &["сына"],
                &["сыни", "сынове"],
                &["сыновъ"],
                &["сыномъ", "сыновомъ"],
                &["сыны", "сыновъ"],
                &["сыны", "сынми"],
                &["сынѣхъ", "сыновѣхъ", "сынахъ"],
                &["сыни", "сынове"],
            ],
        );
    }

    #[test]
    fn alpy_34_37_j_ey_and_ie_stem_goldens() {
        let kraj = NounLexeme::new(
            word("край"),
            word("кра"),
            Gender::Masculine,
            NounDeclension::FirstSoftMasculineJ,
        );
        assert_noun_paradigm(
            &kraj,
            Animacy::Animate,
            &[
                &["край"],
                &["краѧ"],
                &["краю"],
                &["краѧ", "край"],
                &["краемъ"],
                &["краи", "краѣ"],
                &["краю"],
                &["краѧ"],
                &["краю"],
                &["краема"],
                &["краѧ"],
                &["краема"],
                &["краю"],
                &["краѧ"],
                &["краи"],
                &["краєвъ"],
                &["краємъ"],
                &["краи", "краєвъ"],
                &["краи"],
                &["краехъ"],
                &["краи"],
            ],
        );

        let priest = NounLexeme::new(
            word("їерей"),
            word("їере"),
            Gender::Masculine,
            NounDeclension::FirstSoftMasculineEy,
        );
        assert_noun_paradigm(
            &priest,
            Animacy::Animate,
            &[
                &["їерей"],
                &["їереа"],
                &["їерею", "їереови"],
                &["їереа", "їерей"],
                &["їереемъ"],
                &["їереи", "їереѣ"],
                &["їерею", "їерее"],
                &["їерєа"],
                &["їерєю"],
                &["їереема", "їереома"],
                &["їерєа"],
                &["їереема", "їереома"],
                &["їерєю"],
                &["їерєа"],
                &["їереє"],
                &["їерєй"],
                &["їереємъ", "їереѡмъ"],
                &["їерєи", "їерєй"],
                &["їерєи"],
                &["їереехъ"],
                &["їереє"],
            ],
        );

        let sign = NounLexeme::new(
            word("знаменїе"),
            word("знаменї"),
            Gender::Neuter,
            NounDeclension::FirstSoftNeuterIe,
        );
        assert_noun_paradigm(
            &sign,
            Animacy::Inanimate,
            &[
                &["знаменїе"],
                &["знаменїѧ"],
                &["знаменїю"],
                &["знаменїе"],
                &["знаменїемъ"],
                &["знаменїи"],
                &["знаменїе"],
                &["знамєнїи"],
                &["знамєнїю"],
                &["знаменїема"],
                &["знамєнїи"],
                &["знаменїема"],
                &["знамєнїю"],
                &["знамєнїи"],
                &["знамєнїѧ"],
                &["знаменїй"],
                &["знаменїємъ"],
                &["знамєнїѧ"],
                &["знаменїи", "знаменьми", "знаменми"],
                &["знаменїихъ"],
                &["знамєнїѧ"],
            ],
        );
    }

    #[test]
    fn rejects_second_declension_with_neuter_gender() {
        let lexeme = NounLexeme {
            lemma: word("жена"),
            stem: word("жен"),
            gender: Gender::Neuter,
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
    fn alpy_39_40_velar_and_mixed_second_declension_goldens() {
        let hand = NounLexeme::new(
            word("рꙋка"),
            word("рꙋк"),
            Gender::Feminine,
            NounDeclension::SecondHardVelar,
        );
        assert_noun_paradigm(
            &hand,
            Animacy::Inanimate,
            &[
                &["рꙋка"],
                &["рꙋки"],
                &["рꙋцѣ"],
                &["рꙋкꙋ"],
                &["рꙋкою"],
                &["рꙋцѣ"],
                &["рꙋко"],
                &["рꙋцѣ"],
                &["рꙋкꙋ"],
                &["рꙋкама"],
                &["рꙋцѣ"],
                &["рꙋкама"],
                &["рꙋкꙋ"],
                &["рꙋцѣ"],
                &["рꙋки"],
                &["рꙋкъ"],
                &["рꙋкамъ"],
                &["рꙋки"],
                &["рꙋками"],
                &["рꙋкахъ"],
                &["рꙋки"],
            ],
        );

        let youth = NounLexeme::new(
            word("юноша"),
            word("юнош"),
            Gender::Masculine,
            NounDeclension::SecondMixed,
        );
        assert_noun_paradigm(
            &youth,
            Animacy::Animate,
            &[
                &["юноша"],
                &["юноши"],
                &["юноши", "юношѣ"],
                &["юношꙋ"],
                &["юношею"],
                &["юноши"],
                &["юноше"],
                &["юнѡши"],
                &["юношꙋ"],
                &["юношама"],
                &["юнѡши"],
                &["юношама"],
                &["юношꙋ"],
                &["юнѡши"],
                &["юноши"],
                &["юношъ"],
                &["юношамъ"],
                &["юношы", "юношъ"],
                &["юношами"],
                &["юношахъ"],
                &["юноши"],
            ],
        );
    }

    #[test]
    fn alpy_32_40_postvocalic_and_gendered_ia_boundaries() {
        let lightning = NounLexeme::new(
            word("молнїѧ"),
            word("молнї"),
            Gender::Feminine,
            NounDeclension::SecondSoftPostvocalicAncientPlural,
        );
        for case in [Case::Nominative, Case::Accusative, Case::Vocative] {
            assert_eq!(
                decline_noun(
                    &lightning,
                    NounCell {
                        case,
                        number: Number::Plural,
                        animacy: Animacy::Inanimate,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("ancient postvocalic plural")
                .primary_text(),
                "молнїѧ"
            );
        }
        assert_eq!(
            decline_noun(
                &lightning,
                NounCell {
                    case: Case::Genitive,
                    number: Number::Plural,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("ordinary noncitation plural")
            .primary_text(),
            "молнїй"
        );

        let isaiah = NounLexeme::new(
            word("исаїа"),
            word("исаї"),
            Gender::Masculine,
            NounDeclension::SecondSoftMasculineIa,
        );
        assert_eq!(
            decline_noun(
                &isaiah,
                NounCell {
                    case: Case::Instrumental,
                    number: Number::Singular,
                    animacy: Animacy::Animate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("§40 masculine -їа instrumental")
            .primary_text(),
            "исаїемъ"
        );

        let mary = NounLexeme::new(
            word("маріа"),
            word("марі"),
            Gender::Feminine,
            NounDeclension::SecondSoftFeminineIa,
        );
        assert_noun_paradigm(
            &mary,
            Animacy::Animate,
            &[
                &["маріа"],
                &["маріи"],
                &["маріи"],
                &["марію"],
                &["маріею"],
                &["маріи"],
                &["маріе"],
                &["маріи"],
                &["марію"],
                &["маріѧма"],
                &["маріи"],
                &["маріѧма"],
                &["марію"],
                &["маріи"],
                &["маріи"],
                &["марій"],
                &["маріѧмъ"],
                &["маріи", "марій"],
                &["маріѧми"],
                &["маріѧхъ"],
                &["маріи"],
            ],
        );
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
                &["мꙋжꙋ", "мꙋжеви"],
                &["мꙋжа", "мꙋжъ"],
                &["мꙋжемъ"],
                &["мꙋжи", "мꙋжѣ"],
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
                &["мꙋжы", "мꙋжьми", "мꙋжами"],
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
                word("сынь"),
                word("сын"),
                Gender::Masculine,
                NounDeclension::FirstHardMasculineUStem,
            ),
            NounLexeme::new(
                word("галїлеанъ"),
                word("галїлеан"),
                Gender::Masculine,
                NounDeclension::FirstHardMasculineInEthnonym,
            ),
            NounLexeme::new(
                word("удъ"),
                word("удес"),
                Gender::Masculine,
                NounDeclension::FirstHardMasculineUdEs,
            ),
            NounLexeme::new(
                word("свидѣтель"),
                word("свидѣт"),
                Gender::Masculine,
                NounDeclension::FirstSoftMasculineAgentTel,
            ),
            NounLexeme::new(
                word("господинъ"),
                word("господин"),
                Gender::Masculine,
                NounDeclension::FirstSoftMasculineLord,
            ),
            NounLexeme::new(
                word("краь"),
                word("кра"),
                Gender::Masculine,
                NounDeclension::FirstSoftMasculineJ,
            ),
            NounLexeme::new(
                word("їерей"),
                word("їер"),
                Gender::Masculine,
                NounDeclension::FirstSoftMasculineEy,
            ),
            NounLexeme::new(
                word("знаменїе"),
                word("знамен"),
                Gender::Neuter,
                NounDeclension::FirstSoftNeuterIe,
            ),
            NounLexeme::new(
                word("море"),
                word("мор"),
                Gender::Neuter,
                NounDeclension::FirstSoftNeuterIshche,
            ),
            NounLexeme::new(
                word("домъ"),
                word("дом"),
                Gender::Masculine,
                NounDeclension::FirstMixedMasculine,
            ),
            NounLexeme::new(
                word("жена"),
                word("жен"),
                Gender::Feminine,
                NounDeclension::SecondHardVelar,
            ),
            NounLexeme::new(
                word("жена"),
                word("жен"),
                Gender::Feminine,
                NounDeclension::SecondMixed,
            ),
            NounLexeme::new(
                word("землѧ"),
                word("земл"),
                Gender::Feminine,
                NounDeclension::SecondSoftPostvocalicAncientPlural,
            ),
            NounLexeme::new(
                word("исаїѧ"),
                word("исаї"),
                Gender::Masculine,
                NounDeclension::SecondSoftMasculineIa,
            ),
            NounLexeme::new(
                word("маріѧ"),
                word("марі"),
                Gender::Feminine,
                NounDeclension::SecondSoftFeminineIa,
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
                word("чꙋдо"),
                word("чꙋден"),
                Gender::Neuter,
                NounDeclension::FourthNeuterEsAlternatingFirst,
            ),
            NounLexeme::new(
                word("ѻко"),
                word("очен"),
                Gender::Neuter,
                NounDeclension::FourthNeuterEsPairedDual,
            ),
            NounLexeme::new(
                word("мати"),
                word("матес"),
                Gender::Feminine,
                NounDeclension::FourthFeminineEr,
            ),
            NounLexeme::new(
                word("дщи"),
                word("дщер"),
                Gender::Feminine,
                NounDeclension::FourthFeminineErDaughter,
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
                word("день"),
                word("ден"),
                Gender::Masculine,
                NounDeclension::FourthMasculineEnDay,
            ),
            NounLexeme::new(
                word("адѡнаі"),
                word("адонаи"),
                Gender::Masculine,
                NounDeclension::Indeclinable,
            ),
            NounLexeme::new(
                word("любовь"),
                word("любов"),
                Gender::Feminine,
                NounDeclension::FourthFeminineOv,
            ),
            NounLexeme::new(
                word("любовь"),
                word("любов"),
                Gender::Feminine,
                NounDeclension::FourthFeminineOvSyncopating,
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
    fn alpy_43_44_cell_scoped_irregular_stem_goldens() {
        let eye = NounLexeme::new(
            word("ѻко"),
            word("очес"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEsPairedDual,
        );
        assert_noun_paradigm(
            &eye,
            Animacy::Inanimate,
            &[
                &["ѻко"],
                &["очесе"],
                &["очеси"],
                &["ѻко"],
                &["очесемъ"],
                &["очеси", "ѻцѣ"],
                &["ѻко"],
                &["очи", "оцѣ"],
                &["очїю"],
                &["очима"],
                &["очи", "оцѣ"],
                &["очима"],
                &["очїю"],
                &["очи", "оцѣ"],
                &["очеса"],
                &["очесъ"],
                &["очесємъ"],
                &["очеса"],
                &["очесы"],
                &["очесѣхъ"],
                &["очеса"],
            ],
        );

        let ear = NounLexeme::new(
            word("ꙋхо"),
            word("ушес"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEsPairedDual,
        );
        assert_eq!(
            decline_noun(
                &ear,
                NounCell {
                    case: Case::Genitive,
                    number: Number::Dual,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("Alypy §44 paired dual")
            .primary_text(),
            "ушїю"
        );
        assert_eq!(
            decline_noun(
                &ear,
                NounCell {
                    case: Case::Nominative,
                    number: Number::Plural,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("Alypy §44 extended plural")
            .primary_text(),
            "ушеса"
        );

        let church = NounLexeme::new(
            word("церковь"),
            word("церкв"),
            Gender::Feminine,
            NounDeclension::FourthFeminineOvSyncopating,
        );
        assert_noun_paradigm(
            &church,
            Animacy::Inanimate,
            &[
                &["церковь"],
                &["церкве"],
                &["церкви"],
                &["церковь"],
                &["церковїю"],
                &["церкви"],
                &["церковь", "церкве"],
                &["цєркви"],
                &["цєрковїю"],
                &["церквама"],
                &["цєркви"],
                &["церквама"],
                &["цєрковїю"],
                &["цєркви"],
                &["цєркви"],
                &["церквей"],
                &["церквамъ"],
                &["церкви"],
                &["церквами"],
                &["церквахъ"],
                &["цєркви"],
            ],
        );

        let love = NounLexeme::new(
            word("любовь"),
            word("любв"),
            Gender::Feminine,
            NounDeclension::FourthFeminineOvSyncopating,
        );
        let cells = [
            (Number::Singular, Case::Genitive, "любве"),
            (Number::Singular, Case::Instrumental, "любовїю"),
            (Number::Dual, Case::Genitive, "любовїю"),
            (Number::Dual, Case::Dative, "любвама"),
            (Number::Plural, Case::Genitive, "любвей"),
        ];
        for (number, case, expected) in cells {
            assert_eq!(
                decline_noun(
                    &love,
                    NounCell {
                        case,
                        number,
                        animacy: Animacy::Inanimate,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("cell-scoped любовь stem")
                .primary_text(),
                expected
            );
        }

        let daughter = NounLexeme::new(
            word("дщерь"),
            word("дщер"),
            Gender::Feminine,
            NounDeclension::FourthFeminineErDaughter,
        );
        for (case, expected) in [
            (Case::Nominative, "дщи"),
            (Case::Accusative, "дщерь"),
            (Case::Genitive, "дщере"),
            (Case::Vocative, "дщи"),
        ] {
            assert_eq!(
                decline_noun(
                    &daughter,
                    NounCell {
                        case,
                        number: Number::Singular,
                        animacy: Animacy::Animate,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("Alypy §44 daughter family")
                .primary_text(),
                expected
            );
        }
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
    fn alpy_8_33_37_mobile_e_ts_noun_is_complete() {
        let infant = NounLexeme::new(
            word("младенецъ"),
            word("младенц"),
            Gender::Masculine,
            NounDeclension::FirstMixedTsMasculine,
        );
        validate_noun_lexeme(&infant).expect("source-defined -ецъ : -ц- contract");
        assert_noun_paradigm(
            &infant,
            Animacy::Animate,
            &[
                &["младенецъ"],
                &["младенца"],
                &["младенцꙋ", "младенцеви"],
                &["младенца", "младенецъ"],
                &["младенцемъ"],
                &["младенци", "младенцѣ"],
                &["младенче"],
                &["младенца"],
                &["младенцꙋ"],
                &["младенцема"],
                &["младенца"],
                &["младенцема"],
                &["младенцꙋ"],
                &["младенца"],
                &["младенцы"],
                &["младенцєвъ"],
                &["младенцємъ"],
                &["младенцы", "младенцєвъ"],
                &["младенцы", "младенцьми", "младенцами"],
                &["младенцѣхъ"],
                &["младенцы"],
            ],
        );
    }

    #[test]
    fn alpy_52_short_masculine_stem_formations_are_typed() {
        let blessed = AdjectiveLexeme {
            lemma: word("блаженъ"),
            stem: word("блаженн"),
            class: AdjectiveClass::Hard,
            short_masculine_stem: Some(word("блажен")),
            short_masculine_formation: Some(ShortMasculineStemFormation::DoubleNReduction),
            comparative_stem: None,
            comparison_formation: None,
        };
        let venerable = AdjectiveLexeme {
            lemma: word("преподобенъ"),
            stem: word("преподобн"),
            class: AdjectiveClass::Hard,
            short_masculine_stem: Some(word("преподобен")),
            short_masculine_formation: Some(ShortMasculineStemFormation::MobileEInsertion),
            comparative_stem: None,
            comparison_formation: None,
        };
        for adjective in [&blessed, &venerable] {
            validate_adjective_lexeme(adjective).expect("typed positive principal part");
        }
        let form = |lexeme: &AdjectiveLexeme, gender, adjective_form| {
            decline_adjective(
                lexeme,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: adjective_form,
                    comparison: Comparison::Positive,
                },
                OrthographyProfile::Expanded,
            )
            .expect("productive positive cell")
            .primary_text()
            .to_owned()
        };
        assert_eq!(
            form(&blessed, Gender::Masculine, AdjectiveForm::Short),
            "блаженъ"
        );
        assert_eq!(
            form(&blessed, Gender::Feminine, AdjectiveForm::Short),
            "блаженна"
        );
        assert_eq!(
            form(&blessed, Gender::Masculine, AdjectiveForm::Long),
            "блаженный"
        );
        assert_eq!(
            form(&venerable, Gender::Masculine, AdjectiveForm::Short),
            "преподобенъ"
        );
        assert_eq!(
            form(&venerable, Gender::Feminine, AdjectiveForm::Short),
            "преподобна"
        );

        let mut contradictory = blessed.clone();
        contradictory.short_masculine_stem = Some(word("блаженн"));
        assert!(matches!(
            validate_adjective_lexeme(&contradictory),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }

    #[test]
    fn declines_long_hard_adjective_from_alypy_57() {
        let lexeme = AdjectiveLexeme {
            lemma: word("мꙋдръ"),
            stem: word("мꙋдр"),
            class: AdjectiveClass::Hard,
            short_masculine_stem: None,
            short_masculine_formation: None,
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
    fn alpy_57_velar_adjective_table_controls_endings_and_palatalization() {
        let good = AdjectiveLexeme {
            lemma: word("благъ"),
            stem: word("благ"),
            class: AdjectiveClass::Velar,
            short_masculine_stem: None,
            short_masculine_formation: None,
            comparative_stem: None,
            comparison_formation: None,
        };
        validate_adjective_lexeme(&good).expect("velar stem");
        let form = |number, gender, case, adjective_form, animacy| {
            decline_adjective(
                &good,
                AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy,
                    form: adjective_form,
                    comparison: Comparison::Positive,
                },
                OrthographyProfile::Expanded,
            )
            .expect("Alypy velar cell")
            .variants()
            .iter()
            .map(|variant| variant.expanded.clone())
            .collect::<Vec<_>>()
        };
        use AdjectiveForm::{Long, Short};
        use Case::{
            Accusative as Acc, Dative as Dat, Genitive as Gen, Locative as Loc, Nominative as Nom,
            Vocative as Voc,
        };
        use Gender::{Feminine as F, Masculine as M, Neuter as N};
        use Number::{Dual as Du, Plural as Pl, Singular as Sg};

        for (number, gender, case, adjective_form, expected) in [
            (Sg, M, Nom, Short, &["благъ"][..]),
            (Sg, M, Voc, Short, &["блаже"]),
            (Sg, F, Dat, Short, &["блазѣ"]),
            (Pl, M, Nom, Short, &["блази"]),
            (Pl, F, Nom, Short, &["благи"]),
            (Sg, M, Nom, Long, &["благїй"]),
            (Sg, F, Gen, Long, &["благїѧ"]),
            (Sg, M, Loc, Long, &["блазѣмъ"]),
            (Du, F, Nom, Long, &["блазѣи"]),
            (Pl, M, Nom, Long, &["блазїи"]),
            (Pl, F, Nom, Long, &["благїѧ"]),
            (Pl, N, Nom, Long, &["благаѧ"]),
        ] {
            assert_eq!(
                form(number, gender, case, adjective_form, Animacy::Inanimate),
                expected,
                "{number:?} {gender:?} {case:?} {adjective_form:?}"
            );
        }
        assert_eq!(
            form(Sg, M, Acc, Long, Animacy::Animate),
            ["благаго", "благїй"]
        );
        assert_eq!(
            form(Pl, M, Acc, Long, Animacy::Animate),
            ["благїѧ", "благихъ"]
        );

        let mut contradictory = good;
        contradictory.stem = word("мꙋдр");
        assert!(matches!(
            validate_adjective_lexeme(&contradictory),
            Err(Error::ContradictoryMetadata { .. })
        ));
        assert!(matches!(
            decline_adjective(
                &contradictory,
                AdjectiveCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    gender: Gender::Masculine,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }

    #[test]
    fn declines_comparison_stem_with_alypy_58_mixed_endings() {
        let lexeme = AdjectiveLexeme {
            lemma: word("мꙋдръ"),
            stem: word("мꙋдр"),
            class: AdjectiveClass::Hard,
            short_masculine_stem: None,
            short_masculine_formation: None,
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
    fn short_superlative_is_bounded_to_nominative_predicate_agreement() {
        let lexeme = AdjectiveLexeme {
            lemma: word("истиннъ"),
            stem: word("истинн"),
            class: AdjectiveClass::Hard,
            short_masculine_stem: None,
            short_masculine_formation: None,
            comparative_stem: Some(word("истиннѣйш")),
            comparison_formation: Some(ComparisonFormation::LaterYat),
        };
        let expected = [
            (Number::Singular, Gender::Masculine, "истиннѣйшъ|истиннѣй"),
            (Number::Singular, Gender::Feminine, "истиннѣйши"),
            (Number::Singular, Gender::Neuter, "истиннѣе|истиннѣйше"),
            (Number::Dual, Gender::Masculine, "истиннѣйша"),
            (Number::Dual, Gender::Feminine, "истиннѣйши"),
            (Number::Dual, Gender::Neuter, "истиннѣйши"),
            (Number::Plural, Gender::Masculine, "истиннѣйше|истиннѣйши"),
            (Number::Plural, Gender::Feminine, "истиннѣйшѧ"),
            (Number::Plural, Gender::Neuter, "истиннѣйша"),
        ];
        for (number, gender, expected) in expected {
            let forms = decline_adjective(
                &lexeme,
                AdjectiveCell {
                    case: Case::Nominative,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Superlative,
                },
                OrthographyProfile::Expanded,
            )
            .expect("Alypy §59 predicate short superlative");
            assert_eq!(
                forms.texts().collect::<Vec<_>>(),
                expected.split('|').collect::<Vec<_>>()
            );
            assert_productive_contract(&forms);
        }

        for number in Number::ALL {
            for gender in Gender::ALL {
                for case in Case::ALL {
                    if case == Case::Nominative {
                        continue;
                    }
                    assert!(matches!(
                        decline_adjective(
                            &lexeme,
                            AdjectiveCell {
                                case,
                                number,
                                gender,
                                animacy: Animacy::Inanimate,
                                form: AdjectiveForm::Short,
                                comparison: Comparison::Superlative,
                            },
                            OrthographyProfile::Expanded,
                        ),
                        Err(Error::HistoricallyInvalidCell { .. })
                    ));
                }
            }
        }
    }

    #[test]
    fn alpy_50_56_possessive_adjective_contracts_are_complete_and_bounded() {
        let bozhii = AdjectiveLexeme {
            lemma: word("божїй"),
            stem: word("бож"),
            class: AdjectiveClass::PossessiveIi,
            short_masculine_stem: None,
            short_masculine_formation: None,
            comparative_stem: None,
            comparison_formation: None,
        };
        let short_goldens = [
            [
                "божїй",
                "божїѧ",
                "божїю",
                "божїѧ",
                "божїимъ",
                "божїи",
                "божїй",
                "божїѧ",
                "божїю",
                "божїима",
                "божїѧ",
                "божїима",
                "божїю",
                "божїѧ",
                "божїи",
                "божїихъ",
                "божїимъ",
                "божїи",
                "божїи",
                "божїихъ",
                "божїи",
            ],
            [
                "божїѧ",
                "божїѧ",
                "божїи",
                "божїю",
                "божїею",
                "божїи",
                "божїѧ",
                "божїи",
                "божїю",
                "божїима",
                "божїи",
                "божїима",
                "божїю",
                "божїи",
                "божїѧ",
                "божїихъ",
                "божїимъ",
                "божїѧ",
                "божїими",
                "божїихъ",
                "божїѧ",
            ],
            [
                "божїе",
                "божїѧ",
                "божїю",
                "божїе",
                "божїимъ",
                "божїи",
                "божїе",
                "божїи",
                "божїю",
                "божїима",
                "божїи",
                "божїима",
                "божїю",
                "божїи",
                "божїѧ",
                "божїихъ",
                "божїимъ",
                "божїѧ",
                "божїи",
                "божїихъ",
                "божїѧ",
            ],
        ];
        for (gender, expected) in Gender::ALL.into_iter().zip(short_goldens) {
            for ((number, case), expected) in Number::ALL
                .into_iter()
                .flat_map(|number| Case::ALL.into_iter().map(move |case| (number, case)))
                .zip(expected)
            {
                assert_eq!(
                    decline_adjective(
                        &bozhii,
                        AdjectiveCell {
                            case,
                            number,
                            gender,
                            animacy: Animacy::Animate,
                            form: AdjectiveForm::Short,
                            comparison: Comparison::Positive,
                        },
                        OrthographyProfile::Expanded,
                    )
                    .expect("complete Alypy §56 short table")
                    .primary_text(),
                    expected,
                    "{gender:?} {number:?} {case:?}"
                );
            }
        }
        for (case, gender, expected) in [
            (Case::Genitive, Gender::Masculine, "божїѧгѡ"),
            (Case::Dative, Gender::Masculine, "божїемꙋ"),
            (Case::Genitive, Gender::Feminine, "божїей"),
            (Case::Instrumental, Gender::Feminine, "божїею"),
            (Case::Locative, Gender::Neuter, "божїемъ"),
        ] {
            assert_eq!(
                decline_adjective(
                    &bozhii,
                    AdjectiveCell {
                        case,
                        number: Number::Singular,
                        gender,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Long,
                        comparison: Comparison::Positive,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("Alypy §56 compound possessive")
                .primary_text(),
                expected
            );
        }

        let gospoden = AdjectiveLexeme {
            lemma: word("господень"),
            stem: word("господн"),
            class: AdjectiveClass::PossessiveSoft,
            short_masculine_stem: Some(word("господен")),
            short_masculine_formation: Some(ShortMasculineStemFormation::MobileEInsertion),
            comparative_stem: None,
            comparison_formation: None,
        };
        let israel = AdjectiveLexeme {
            lemma: word("израилевъ"),
            stem: word("израилев"),
            class: AdjectiveClass::PossessiveHard,
            short_masculine_stem: None,
            short_masculine_formation: None,
            comparative_stem: None,
            comparison_formation: None,
        };
        for (lexeme, expected) in [(&gospoden, "господень"), (&israel, "израилевъ")]
        {
            assert_eq!(
                decline_adjective(
                    lexeme,
                    AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Short,
                        comparison: Comparison::Positive,
                    },
                    OrthographyProfile::Expanded,
                )
                .expect("productive short possessive")
                .primary_text(),
                expected
            );
            assert!(matches!(
                decline_adjective(
                    lexeme,
                    AdjectiveCell {
                        case: Case::Nominative,
                        number: Number::Singular,
                        gender: Gender::Masculine,
                        animacy: Animacy::Inanimate,
                        form: AdjectiveForm::Long,
                        comparison: Comparison::Positive,
                    },
                    OrthographyProfile::Expanded,
                ),
                Err(Error::HistoricallyInvalidCell { .. })
            ));
        }
        for lexeme in [&gospoden, &israel] {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    for case in Case::ALL {
                        let cell = AdjectiveCell {
                            case,
                            number,
                            gender,
                            animacy: Animacy::Animate,
                            form: AdjectiveForm::Short,
                            comparison: Comparison::Positive,
                        };
                        assert_productive_contract(
                            &decline_adjective(lexeme, cell, OrthographyProfile::Expanded)
                                .expect("complete short possessive paradigm"),
                        );
                        assert!(matches!(
                            decline_adjective(
                                lexeme,
                                AdjectiveCell {
                                    form: AdjectiveForm::Long,
                                    ..cell
                                },
                                OrthographyProfile::Expanded,
                            ),
                            Err(Error::HistoricallyInvalidCell { .. })
                        ));
                    }
                }
            }
        }
        for number in Number::ALL {
            for gender in Gender::ALL {
                for case in Case::ALL {
                    assert_productive_contract(
                        &decline_adjective(
                            &bozhii,
                            AdjectiveCell {
                                case,
                                number,
                                gender,
                                animacy: Animacy::Animate,
                                form: AdjectiveForm::Long,
                                comparison: Comparison::Positive,
                            },
                            OrthographyProfile::Expanded,
                        )
                        .expect("complete compound -їй possessive paradigm"),
                    );
                }
            }
        }
        assert!(matches!(
            decline_adjective(
                &bozhii,
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
            Err(Error::HistoricallyInvalidCell { .. })
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
    fn simple_future_is_the_complete_perfective_present_shape() {
        let present_lexeme = regular_verb();
        let mut future_lexeme = present_lexeme.clone();
        future_lexeme.lemma = word("понести");
        future_lexeme.aspect = Aspect::Perfective;

        for number in Number::ALL {
            for person in Person::ALL {
                let present_form = present(
                    &present_lexeme,
                    person,
                    number,
                    OrthographyProfile::Expanded,
                )
                .expect("complete present-shaped source paradigm");
                let future_form =
                    future(&future_lexeme, person, number, OrthographyProfile::Expanded)
                        .expect("Alypy §84 perfective simple future");
                assert_eq!(
                    future_form.texts().collect::<Vec<_>>(),
                    present_form.texts().collect::<Vec<_>>()
                );
                assert!(future_form.variants().iter().all(|variant| {
                    matches!(
                        &variant.source,
                        FormSource::SynodalNormativeGeneration { rule }
                            if rule.as_str() == "SYN-VERB-FUTURE-PERFECTIVE-ALYPY-84"
                    )
                }));
            }
        }

        let mut suppletive = future_lexeme.clone();
        suppletive.lemma = word("възѧти");
        suppletive.present_stem = Some(word("вземл"));
        suppletive.present_first_singular = Some(word("вземлю"));
        suppletive.present_third_plural = Some(word("вземлютъ"));
        suppletive.future_stem = Some(word("возм"));
        suppletive.future_first_singular = Some(word("возмꙋ"));
        suppletive.future_third_plural = Some(word("возмꙋтъ"));
        assert_eq!(
            present(
                &suppletive,
                Person::Second,
                Number::Singular,
                OrthographyProfile::Expanded,
            )
            .expect("independent present series")
            .primary_text(),
            "вземлеши"
        );
        assert_eq!(
            future(
                &suppletive,
                Person::Second,
                Number::Singular,
                OrthographyProfile::Expanded,
            )
            .expect("independent future series")
            .primary_text(),
            "возмеши"
        );

        suppletive.future_third_plural = None;
        assert_eq!(
            suppletive.missing_principal_parts(VerbSystem::Finite(FiniteTense::Future)),
            vec![MetadataField::FutureThirdPlural]
        );

        assert!(matches!(
            future(
                &present_lexeme,
                Person::Third,
                Number::Singular,
                OrthographyProfile::Expanded,
            ),
            Err(Error::EvidenceIncompleteCell {
                field: MetadataField::Aspect,
                ..
            })
        ));

        let mut biaspectual = present_lexeme.clone();
        biaspectual.aspect = Aspect::Biaspectual;
        assert!(matches!(
            future(
                &biaspectual,
                Person::Third,
                Number::Singular,
                OrthographyProfile::Expanded,
            ),
            Err(Error::EvidenceIncompleteCell {
                field: MetadataField::Aspect,
                ..
            })
        ));

        let mut unknown = present_lexeme;
        unknown.aspect = Aspect::Unknown;
        assert_eq!(
            future(
                &unknown,
                Person::Third,
                Number::Singular,
                OrthographyProfile::Expanded,
            ),
            Err(Error::MissingMetadata {
                field: MetadataField::Aspect,
            })
        );
    }

    #[test]
    fn alpy_104_mobile_vowel_l_participle_keeps_two_typed_stems() {
        let mut verb = regular_verb();
        verb.lemma = word("изити");
        verb.l_participle_stem = Some(word("изш"));
        verb.l_participle_masculine_singular_stem = Some(word("изше"));

        let expected = [
            (Gender::Masculine, Number::Singular, "изшелъ"),
            (Gender::Feminine, Number::Singular, "изшла"),
            (Gender::Neuter, Number::Singular, "изшло"),
            (Gender::Masculine, Number::Dual, "изшла"),
            (Gender::Feminine, Number::Dual, "изшли"),
            (Gender::Neuter, Number::Dual, "изшли"),
            (Gender::Masculine, Number::Plural, "изшли"),
            (Gender::Feminine, Number::Plural, "изшли"),
            (Gender::Neuter, Number::Plural, "изшли"),
        ];
        for (gender, number, surface) in expected {
            assert_eq!(
                l_participle(
                    &verb,
                    LParticipleCell { gender, number },
                    OrthographyProfile::Expanded,
                )
                .expect("typed two-stem l-participle")
                .primary_text(),
                surface
            );
        }

        verb.l_participle_masculine_singular_stem = None;
        assert_eq!(
            l_participle(
                &verb,
                LParticipleCell {
                    gender: Gender::Masculine,
                    number: Number::Singular,
                },
                OrthographyProfile::Expanded,
            )
            .expect("legacy one-stem l-participle")
            .primary_text(),
            "изшлъ"
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
            future_stem: None,
            future_first_singular: None,
            future_third_plural: None,
            imperfect_stem: Some(word("нес")),
            imperfect_formation: Some(ImperfectFormation::Yah),
            aorist_stem: Some(word("нес")),
            aorist_formation: Some(AoristFormation::ConsonantStem),
            imperative_stem: Some(word("нес")),
            imperative_formation: Some(ImperativeFormation::FirstUnpalatalized),
            l_participle_stem: Some(word("нес")),
            l_participle_masculine_singular_stem: None,
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

    #[test]
    fn verbal_noun_ie_has_the_complete_alypy_27_34_paradigm() {
        let mut verb = regular_verb();
        verb.lemma = word("молити");
        verb.past_passive_participle = Some(ParticiplePrincipalPart {
            short_stem: Some(word("молен")),
            short_formation: None,
            long_stem: Some(word("моленн")),
            class: AdjectiveClass::Hard,
        });
        let expected: &[&[&str]] = &[
            &["моленїе"],
            &["моленїѧ"],
            &["моленїю"],
            &["моленїе"],
            &["моленїемъ"],
            &["моленїи"],
            &["моленїе"],
            &["молєнїи"],
            &["молєнїю"],
            &["моленїема"],
            &["молєнїи"],
            &["моленїема"],
            &["молєнїю"],
            &["молєнїи"],
            &["молєнїѧ"],
            &["моленїй"],
            &["моленїємъ"],
            &["молєнїѧ"],
            &["моленїи", "моленьми", "моленми"],
            &["моленїихъ"],
            &["молєнїѧ"],
        ];

        for animacy in Animacy::ALL {
            for (index, (number, case)) in Number::ALL
                .into_iter()
                .flat_map(|number| Case::ALL.into_iter().map(move |case| (number, case)))
                .enumerate()
            {
                let forms = decline_verbal_noun(
                    &verb,
                    NounCell {
                        case,
                        number,
                        animacy,
                    },
                    OrthographyProfile::Expanded,
                )
                .unwrap_or_else(|error| panic!("{animacy:?} {number:?} {case:?}: {error}"));
                assert_eq!(
                    forms.texts().collect::<Vec<_>>().as_slice(),
                    expected[index],
                    "{animacy:?} {number:?} {case:?}"
                );
                assert!(matches!(
                    &forms.primary().source,
                    FormSource::SynodalNormativeGeneration { rule }
                        if rule.as_str() == "SYN-VERB-VERBAL-NOUN-IE-ALYPY-27"
                ));
                assert_eq!(
                    forms.primary().rule_trace.steps()[0].stage,
                    "verbal-noun-formation-past-passive-ie"
                );
            }
        }
    }

    #[test]
    fn verbal_noun_keeps_lexical_suffix_choice_explicit() {
        let mut verb = regular_verb();
        verb.lemma = word("молитися");
        verb.past_passive_participle = None;
        assert_eq!(
            decline_verbal_noun(
                &verb,
                NounCell {
                    case: Case::Nominative,
                    number: Number::Singular,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::MissingPrincipalPart {
                field: MetadataField::VerbalNounStem,
            })
        );

        verb.verbal_noun = Some(
            VerbalNounPrincipalPart::explicit_lexical(
                NounLexeme::new(
                    word("молитва"),
                    word("молитв"),
                    Gender::Feminine,
                    NounDeclension::SecondHard,
                )
                .with_number_inventory(NounNumberInventory::SingularAndPlural),
            )
            .expect("complete lexical deverbal noun"),
        );
        let nominative = decline_verbal_noun(
            &verb,
            NounCell {
                case: Case::Nominative,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::Expanded,
        )
        .expect("explicit lexical suffix family");
        assert_eq!(nominative.primary_text(), "молитва");
        assert!(matches!(
            &nominative.primary().source,
            FormSource::SynodalNormativeGeneration { rule }
                if rule.as_str() == "SYN-VERB-VERBAL-NOUN-LEXICAL-ALYPY-27"
        ));
        assert!(matches!(
            decline_verbal_noun(
                &verb,
                NounCell {
                    case: Case::Nominative,
                    number: Number::Dual,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
    }

    #[test]
    fn productive_verbal_noun_rejects_a_non_participial_platform() {
        assert!(matches!(
            VerbalNounPrincipalPart::past_passive_ie("моли"),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let mut verb = regular_verb();
        verb.verbal_noun = None;
        verb.past_passive_participle = Some(ParticiplePrincipalPart {
            short_stem: Some(word("моли")),
            short_formation: None,
            long_stem: Some(word("моленн")),
            class: AdjectiveClass::Hard,
        });
        assert_eq!(
            verb.missing_principal_parts(VerbSystem::VerbalNoun {
                animacy: Animacy::Inanimate,
            }),
            [MetadataField::VerbalNounStem]
        );
    }

    #[test]
    fn lexical_verbal_noun_requires_an_alypy_27_suffix_family() {
        for (lemma, stem, gender, declension) in [
            (
                "работа",
                "работ",
                Gender::Feminine,
                NounDeclension::SecondHard,
            ),
            (
                "сꙋета",
                "сꙋет",
                Gender::Feminine,
                NounDeclension::SecondHard,
            ),
            (
                "слꙋжба",
                "слꙋжб",
                Gender::Feminine,
                NounDeclension::SecondHard,
            ),
            (
                "падежъ",
                "падеж",
                Gender::Masculine,
                NounDeclension::FirstMixedMasculine,
            ),
            (
                "дань",
                "дан",
                Gender::Feminine,
                NounDeclension::ThirdFeminine,
            ),
            (
                "пѣснь",
                "пѣсн",
                Gender::Feminine,
                NounDeclension::ThirdFeminine,
            ),
            (
                "жизнь",
                "жизн",
                Gender::Feminine,
                NounDeclension::ThirdFeminine,
            ),
            (
                "молитва",
                "молитв",
                Gender::Feminine,
                NounDeclension::SecondHard,
            ),
            (
                "власть",
                "власт",
                Gender::Feminine,
                NounDeclension::ThirdFeminine,
            ),
            (
                "ꙋкоризна",
                "ꙋкоризн",
                Gender::Feminine,
                NounDeclension::SecondHard,
            ),
        ] {
            VerbalNounPrincipalPart::explicit_lexical(NounLexeme::new(
                word(lemma),
                word(stem),
                gender,
                declension,
            ))
            .unwrap_or_else(|error| panic!("{lemma}: {error}"));
        }

        assert!(matches!(
            VerbalNounPrincipalPart::explicit_lexical(NounLexeme::new(
                word("столъ"),
                word("стол"),
                Gender::Masculine,
                NounDeclension::FirstHardMasculine,
            )),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }

    #[test]
    fn accented_verbal_noun_platform_supports_the_liturgical_profile() {
        let mut verb = regular_verb();
        verb.lemma = word("молити");
        verb.verbal_noun = Some(
            VerbalNounPrincipalPart::past_passive_ie("моле́н").expect("accented source platform"),
        );
        let forms = decline_verbal_noun(
            &verb,
            NounCell {
                case: Case::Genitive,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            },
            OrthographyProfile::SynodalLiturgical,
        )
        .expect("accented productive verbal noun");
        assert_eq!(forms.primary_text(), "моле́нїѧ");
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
                NounDeclension::FirstHardMasculineUStem => ("сынъ", "сын", Gender::Masculine),
                NounDeclension::FirstHardMasculineInEthnonym => {
                    ("галїлеанинъ", "галїлеанин", Gender::Masculine)
                }
                NounDeclension::FirstHardMasculineUdEs => ("ꙋдъ", "ꙋдес", Gender::Masculine),
                NounDeclension::FirstHardVelarMasculine => ("ѻтрокъ", "ѻтрок", Gender::Masculine),
                NounDeclension::FirstMixedMasculine => ("мꙋжъ", "мꙋж", Gender::Masculine),
                NounDeclension::FirstMixedTsMasculine => {
                    ("младенецъ", "младенц", Gender::Masculine)
                }
                NounDeclension::FirstHardNeuter => ("слово", "слов", Gender::Neuter),
                NounDeclension::FirstSoftMasculine => ("царь", "цар", Gender::Masculine),
                NounDeclension::FirstSoftMasculineAgentTel => {
                    ("свидѣтель", "свидѣтел", Gender::Masculine)
                }
                NounDeclension::FirstSoftMasculineLord => ("господь", "господ", Gender::Masculine),
                NounDeclension::FirstSoftMasculineJ => ("край", "кра", Gender::Masculine),
                NounDeclension::FirstSoftMasculineEy => ("їерей", "їере", Gender::Masculine),
                NounDeclension::FirstSoftNeuter => ("море", "мор", Gender::Neuter),
                NounDeclension::FirstSoftNeuterIshche => ("соборище", "соборищ", Gender::Neuter),
                NounDeclension::FirstSoftNeuterIe => ("знаменїе", "знаменї", Gender::Neuter),
                NounDeclension::SecondHard => ("жена", "жен", Gender::Feminine),
                NounDeclension::SecondHardVelar => ("рꙋка", "рꙋк", Gender::Feminine),
                NounDeclension::SecondSoft => ("землѧ", "земл", Gender::Feminine),
                NounDeclension::SecondSoftPostvocalicAncientPlural => {
                    ("молнїѧ", "молнї", Gender::Feminine)
                }
                NounDeclension::SecondSoftMasculineIa => ("исаїа", "исаї", Gender::Masculine),
                NounDeclension::SecondSoftFeminineIa => ("маріа", "марі", Gender::Feminine),
                NounDeclension::SecondMixed => ("юноша", "юнош", Gender::Masculine),
                NounDeclension::ThirdFeminine => ("кость", "кост", Gender::Feminine),
                NounDeclension::ThirdMasculine => ("пꙋть", "пꙋт", Gender::Masculine),
                NounDeclension::FourthNeuterEn => ("имѧ", "имен", Gender::Neuter),
                NounDeclension::FourthNeuterEs => ("небо", "небес", Gender::Neuter),
                NounDeclension::FourthNeuterEsAlternatingFirst => ("чꙋдо", "чꙋдес", Gender::Neuter),
                NounDeclension::FourthNeuterEsPairedDual => ("ѻко", "очес", Gender::Neuter),
                NounDeclension::FourthNeuterAt => ("ѻтроча", "ѻтрочат", Gender::Neuter),
                NounDeclension::FourthFeminineEr => ("мати", "матер", Gender::Feminine),
                NounDeclension::FourthFeminineErDaughter => ("дщерь", "дщер", Gender::Feminine),
                NounDeclension::FourthFeminineOv => ("свекры", "свекров", Gender::Feminine),
                NounDeclension::FourthFeminineOvSyncopating => {
                    ("церковь", "церкв", Gender::Feminine)
                }
                NounDeclension::FourthMasculineEn => ("степень", "степен", Gender::Masculine),
                NounDeclension::FourthMasculineEnDay => ("день", "дн", Gender::Masculine),
                NounDeclension::FourthMasculineEnKamen => ("камень", "камен", Gender::Masculine),
                NounDeclension::Indeclinable => ("адѡнаі", "адѡнаі", Gender::Masculine),
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

        for (class, lemma, stem) in [
            (AdjectiveClass::Hard, "мꙋдръ", "мꙋдр"),
            (AdjectiveClass::Soft, "синь", "син"),
            (AdjectiveClass::Velar, "благъ", "благ"),
        ] {
            let lexeme = AdjectiveLexeme {
                lemma: word(lemma),
                stem: word(stem),
                class,
                short_masculine_stem: None,
                short_masculine_formation: None,
                comparative_stem: Some(word("мꙋдрѣйш")),
                comparison_formation: Some(ComparisonFormation::LaterYat),
            };
            for form in [AdjectiveForm::Short, AdjectiveForm::Long] {
                for comparison in [
                    Comparison::Positive,
                    Comparison::Comparative,
                    Comparison::Superlative,
                ] {
                    for number in Number::ALL {
                        for case in Case::ALL {
                            for gender in Gender::ALL {
                                for animacy in if case == Case::Accusative {
                                    Animacy::ALL.as_slice()
                                } else {
                                    &[Animacy::Inanimate]
                                } {
                                    let outcome = decline_adjective(
                                        &lexeme,
                                        AdjectiveCell {
                                            case,
                                            number,
                                            gender,
                                            animacy: *animacy,
                                            form,
                                            comparison,
                                        },
                                        OrthographyProfile::Expanded,
                                    );
                                    if form == AdjectiveForm::Short
                                        && comparison == Comparison::Superlative
                                        && case != Case::Nominative
                                    {
                                        assert!(matches!(
                                            outcome,
                                            Err(Error::HistoricallyInvalidCell { .. })
                                        ));
                                    } else {
                                        assert_productive_contract(
                                            &outcome.expect("declared adjective inventory"),
                                        );
                                    }
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
    fn alpy_60_complete_short_comparison_golden() {
        struct Row {
            number: Number,
            gender: Gender,
            cells: [&'static str; 7],
        }

        use Gender::{Feminine as F, Masculine as M, Neuter as N};
        use Number::{Dual as Du, Plural as Pl, Singular as Sg};

        // Case order is nominative, genitive, dative, accusative,
        // instrumental, locative, vocative. `@` denotes the typed §58
        // citation edge; the other entries are §60 suffixes.
        let rows = [
            Row {
                number: Sg,
                gender: M,
                cells: ["@", "а", "ꙋ", "ъ", "имъ", "и", "@"],
            },
            Row {
                number: Sg,
                gender: F,
                cells: ["@", "и", "и", "ꙋ", "ею", "и", "@"],
            },
            Row {
                number: Sg,
                gender: N,
                cells: ["@", "а", "ꙋ", "@", "имъ", "и", "@"],
            },
            Row {
                number: Du,
                gender: M,
                cells: ["а", "ꙋ", "има", "а", "има", "ꙋ", "а"],
            },
            Row {
                number: Du,
                gender: F,
                cells: ["и", "ꙋ", "има", "и", "има", "ꙋ", "и"],
            },
            Row {
                number: Du,
                gender: N,
                cells: ["и", "ꙋ", "има", "и", "има", "ꙋ", "и"],
            },
            Row {
                number: Pl,
                gender: M,
                cells: ["е|и", "ихъ", "ымъ", "ѧ", "ими", "ихъ", "е|и"],
            },
            Row {
                number: Pl,
                gender: F,
                cells: ["ѧ", "ихъ", "ымъ", "ѧ", "ими", "ихъ", "ѧ"],
            },
            Row {
                number: Pl,
                gender: N,
                cells: ["а", "ихъ", "ымъ", "а", "ими", "ихъ", "а"],
            },
        ];
        let lexeme = AdjectiveLexeme {
            lemma: word("мꙋдръ"),
            stem: word("мꙋдр"),
            class: AdjectiveClass::Hard,
            short_masculine_stem: None,
            short_masculine_formation: None,
            comparative_stem: Some(word("мꙋдрѣйш")),
            comparison_formation: Some(ComparisonFormation::LaterYat),
        };

        for row in rows {
            for (case, cell_golden) in Case::ALL.into_iter().zip(row.cells) {
                for animacy in if case == Case::Accusative {
                    Animacy::ALL.as_slice()
                } else {
                    &[Animacy::Inanimate]
                } {
                    let forms = decline_adjective(
                        &lexeme,
                        AdjectiveCell {
                            case,
                            number: row.number,
                            gender: row.gender,
                            animacy: *animacy,
                            form: AdjectiveForm::Short,
                            comparison: Comparison::Comparative,
                        },
                        OrthographyProfile::Expanded,
                    )
                    .expect("complete Alypy §60 short-comparison cell");
                    let mut expected = if cell_golden == "@" {
                        match row.gender {
                            M => vec!["мꙋдрѣй".to_owned()],
                            F => vec!["мꙋдрѣйши".to_owned()],
                            N => vec!["мꙋдрѣе".to_owned(), "мꙋдрѣйше".to_owned()],
                        }
                    } else {
                        cell_golden
                            .split('|')
                            .map(|suffix| format!("мꙋдрѣйш{suffix}"))
                            .collect::<Vec<_>>()
                    };
                    if case == Case::Accusative
                        && row.number == Sg
                        && row.gender == M
                        && *animacy == Animacy::Animate
                    {
                        expected.push("мꙋдрѣйша".to_owned());
                    }
                    assert_eq!(
                        forms.texts().collect::<Vec<_>>(),
                        expected.iter().map(String::as_str).collect::<Vec<_>>()
                    );
                    assert!(forms.variants().iter().all(|variant| {
                        matches!(
                            &variant.source,
                            FormSource::SynodalNormativeGeneration { rule }
                                if rule.as_ref() == "SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-60"
                        ) && variant
                            .evidence
                            .iter()
                            .all(|evidence| evidence.citation.contains("§§58 and 60"))
                    }));
                }
            }
        }
    }

    #[test]
    fn alpy_98_complete_short_active_participle_goldens() {
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

        // Alypy §98's complete short-active-participle table, represented as
        // suffixes after the independently supplied participle stem. The three
        // singular nominative citation edges are checked separately.
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
                    comparison: Comparison::Positive,
                };
                let suffixes = if *animacy == Animacy::Animate {
                    golden.animate_variants.unwrap_or(golden.variants)
                } else {
                    golden.variants
                };
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
                    let forms = decline_participle(
                        &verb,
                        ParticipleCell {
                            tense,
                            voice: ParticipleVoice::Active,
                            agreement: adjective_cell,
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
                let agreement = AdjectiveCell {
                    case: Case::Vocative,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: AdjectiveForm::Short,
                    comparison: Comparison::Positive,
                };
                for tense in ParticipleTense::ALL {
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
                short_masculine_stem: None,
                short_masculine_formation: None,
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
            short_masculine_stem: None,
            short_masculine_formation: None,
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

    #[test]
    fn alpy_37_44_remaining_productive_noun_profiles_are_bounded() {
        let ethnonym = NounLexeme::new(
            word("галїлеанинъ"),
            word("галїлеанин"),
            Gender::Masculine,
            NounDeclension::FirstHardMasculineInEthnonym,
        );
        assert_noun_paradigm(
            &ethnonym,
            Animacy::Animate,
            &[
                &["галїлеанинъ"],
                &["галїлеанина"],
                &["галїлеанинꙋ", "галїлеанинови"],
                &["галїлеанина", "галїлеанинъ"],
                &["галїлеаниномъ"],
                &["галїлеанинѣ"],
                &["галїлеанине"],
                &["галїлеанина"],
                &["галїлеанинꙋ"],
                &["галїлеанинома"],
                &["галїлеанина"],
                &["галїлеанинома"],
                &["галїлеанинꙋ"],
                &["галїлеанина"],
                &["галїлеане"],
                &["галїлеанъ"],
                &["галїлеаномъ"],
                &["галїлеане", "галїлеанъ"],
                &["галїлеаны"],
                &["галїлеанѣхъ"],
                &["галїлеане"],
            ],
        );

        let ud = NounLexeme::new(
            word("ꙋдъ"),
            word("ꙋдес"),
            Gender::Masculine,
            NounDeclension::FirstHardMasculineUdEs,
        );
        assert_noun_paradigm(
            &ud,
            Animacy::Inanimate,
            &[
                &["ꙋдъ"],
                &["ꙋда", "ꙋдесе"],
                &["ꙋдꙋ", "ꙋдови", "ꙋдеси"],
                &["ꙋдъ"],
                &["ꙋдомъ", "ꙋдесемъ"],
                &["ꙋдѣ", "ꙋдеси"],
                &["ꙋде"],
                &["ꙋда", "ꙋдєси"],
                &["ꙋдꙋ", "ꙋдесꙋ"],
                &["ꙋдома", "ꙋдесема"],
                &["ꙋда", "ꙋдєси"],
                &["ꙋдома", "ꙋдесема"],
                &["ꙋдꙋ", "ꙋдесꙋ"],
                &["ꙋда", "ꙋдєси"],
                &["ꙋди", "ꙋдеса"],
                &["ꙋдовъ", "ꙋдъ", "ꙋдесъ"],
                &["ꙋдомъ", "ꙋдесємъ"],
                &["ꙋды", "ꙋдеса"],
                &["ꙋды", "ꙋдми", "ꙋдами", "ꙋдесы"],
                &["ꙋдѣхъ", "ꙋдахъ", "ꙋдесѣхъ"],
                &["ꙋди", "ꙋдеса"],
            ],
        );

        let lord = NounLexeme::new(
            word("господь"),
            word("господ"),
            Gender::Masculine,
            NounDeclension::FirstSoftMasculineLord,
        );
        assert_noun_paradigm(
            &lord,
            Animacy::Animate,
            &[
                &["господь"],
                &["господа"],
                &["господꙋ", "господеви"],
                &["господа", "господь"],
                &["господомъ"],
                &["господѣ"],
                &["господи"],
                &["господи"],
                &["господїю", "господю"],
                &["господьма"],
                &["господи"],
                &["господьма"],
                &["господїю", "господю"],
                &["господи"],
                &["господїе"],
                &["господій", "господей"],
                &["господємъ"],
                &["господи", "господій"],
                &["господьми"],
                &["господехъ"],
                &["господїе"],
            ],
        );

        let alternating = NounLexeme::new(
            word("чꙋдо"),
            word("чꙋдес"),
            Gender::Neuter,
            NounDeclension::FourthNeuterEsAlternatingFirst,
        );
        assert_noun_paradigm(
            &alternating,
            Animacy::Inanimate,
            &[
                &["чꙋдо"],
                &["чꙋдесе", "чꙋда"],
                &["чꙋдеси", "чꙋдꙋ"],
                &["чꙋдо"],
                &["чꙋдесемъ", "чꙋдомъ"],
                &["чꙋдеси", "чꙋдѣ"],
                &["чꙋдо"],
                &["чꙋдєси", "чꙋда"],
                &["чꙋдесꙋ", "чꙋдꙋ"],
                &["чꙋдесема", "чꙋдома"],
                &["чꙋдєси", "чꙋда"],
                &["чꙋдесема", "чꙋдома"],
                &["чꙋдесꙋ", "чꙋдꙋ"],
                &["чꙋдєси", "чꙋда"],
                &["чꙋдеса", "чꙋда"],
                &["чꙋдесъ", "чꙋдъ"],
                &["чꙋдесємъ", "чꙋдомъ"],
                &["чꙋдеса", "чꙋда"],
                &["чꙋдесы", "чꙋды", "чꙋдами"],
                &["чꙋдесѣхъ", "чꙋдѣхъ", "чꙋдахъ"],
                &["чꙋдеса", "чꙋда"],
            ],
        );

        let day = NounLexeme::new(
            word("день"),
            word("дн"),
            Gender::Masculine,
            NounDeclension::FourthMasculineEnDay,
        );
        assert_noun_paradigm(
            &day,
            Animacy::Inanimate,
            &[
                &["день"],
                &["дне"],
                &["дни", "дневи"],
                &["день"],
                &["днемъ"],
                &["дни"],
                &["день"],
                &["дни"],
                &["днїю", "дню"],
                &["деньма"],
                &["дни"],
                &["деньма"],
                &["днїю", "дню"],
                &["дни"],
                &["дни", "дніе"],
                &["днїй", "дней"],
                &["днємъ"],
                &["дни"],
                &["деньми"],
                &["днехъ"],
                &["дни", "дніе"],
            ],
        );

        for (lexeme, case, number, expected) in [
            (
                NounLexeme::new(
                    word("свидѣтель"),
                    word("свидѣтел"),
                    Gender::Masculine,
                    NounDeclension::FirstSoftMasculineAgentTel,
                ),
                Case::Nominative,
                Number::Plural,
                vec!["свидѣтели", "свидѣтеле", "свидѣтелїе"],
            ),
            (
                NounLexeme::new(
                    word("соборище"),
                    word("соборищ"),
                    Gender::Neuter,
                    NounDeclension::FirstSoftNeuterIshche,
                ),
                Case::Locative,
                Number::Plural,
                vec!["соборищахъ", "соборищихъ", "соборищехъ"],
            ),
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
            .expect("bounded lexical subclass");
            assert_eq!(
                forms
                    .variants()
                    .iter()
                    .map(|variant| variant.printed.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
        }

        let invariant = NounLexeme::new(
            word("адѡнаі"),
            word("адѡнаі"),
            Gender::Masculine,
            NounDeclension::Indeclinable,
        );
        for number in Number::ALL {
            for case in Case::ALL {
                assert_eq!(
                    decline_noun(
                        &invariant,
                        NounCell {
                            case,
                            number,
                            animacy: Animacy::Animate,
                        },
                        OrthographyProfile::Expanded,
                    )
                    .expect("invariant noun cell")
                    .primary_text(),
                    "адѡнаі"
                );
            }
        }
    }
}
