//! Source-reviewed Old Church Slavonic cardinal-numeral morphology.

use crate::noun::NounLexeme;
use crate::pronoun::StandardPronominalIdentity;
use crate::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, CompoundCardinalCell, Gender,
    InflectionError, NounCell, NounClass, Number, NumberRestriction, NumeralCell, PhraseRole,
    PhraseToken, PredictedForm, RequestedCell, RuleId, RuleStep,
};

/// The syntactic relation between a simple cardinal and the enumerated noun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumeralGovernment {
    /// The numeral and noun agree in the numeral's inherent grammatical number.
    Agreement { number: Number },
    /// The numeral is substantival and governs a genitive-plural complement.
    GenitivePlural,
}

/// Evidential status of one ordered numeral realization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumeralVariantStatus {
    /// A form explicitly present in the reviewed grammatical paradigm.
    ReviewedTable,
    /// A form licensed by a reviewed productive declensional profile.
    ProductiveRule,
    /// A noncanonical spelling or deformation observed in the pinned corpus.
    CorpusAttestation,
}

impl NumeralVariantStatus {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ReviewedTable => "reviewed-table",
            Self::ProductiveRule => "productive-rule",
            Self::CorpusAttestation => "corpus-attestation",
        }
    }
}

/// One ordered numeral realization and its evidential status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumeralVariant {
    pub prediction: PredictedForm,
    pub status: NumeralVariantStatus,
}

impl NumeralVariant {
    fn reviewed(text: &str, rule_id: RuleId, lemma: &str, reason: &'static str) -> Self {
        Self {
            prediction: PredictedForm {
                text: text.to_string(),
                rule_id,
                trace: vec![RuleStep {
                    rule_id,
                    before: lemma.to_string(),
                    after: text.to_string(),
                    reason,
                }],
            },
            status: NumeralVariantStatus::ReviewedTable,
        }
    }

    fn productive(prediction: PredictedForm) -> Self {
        Self {
            prediction,
            status: NumeralVariantStatus::ProductiveRule,
        }
    }

    fn productive_text(text: &str, rule_id: RuleId, lemma: &str, reason: &'static str) -> Self {
        Self {
            prediction: PredictedForm {
                text: text.to_string(),
                rule_id,
                trace: vec![RuleStep {
                    rule_id,
                    before: lemma.to_string(),
                    after: text.to_string(),
                    reason,
                }],
            },
            status: NumeralVariantStatus::ProductiveRule,
        }
    }

    fn corpus(text: &str, rule_id: RuleId, lemma: &str, reason: &'static str) -> Self {
        Self {
            prediction: PredictedForm {
                text: text.to_string(),
                rule_id,
                trace: vec![RuleStep {
                    rule_id,
                    before: lemma.to_string(),
                    after: text.to_string(),
                    reason,
                }],
            },
            status: NumeralVariantStatus::CorpusAttestation,
        }
    }
}

/// One correlated structural realization of a composed cardinal.
///
/// Separate analyses are used when variants in different words must remain
/// paired, as in `триѥ десѧте` beside `три десѧти`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardinalPhraseAnalysis {
    pub tokens: Vec<PhraseToken>,
}

impl CardinalPhraseAnalysis {
    /// Render each component's source-first form within this correlated
    /// structural analysis.
    pub fn primary_text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.forms.primary_text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A composed cardinal whose component words retain independent morphology,
/// provenance, warnings, and traces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizedCardinal {
    value: u16,
    cell: CompoundCardinalCell,
    government: NumeralGovernment,
    analyses: Vec<CardinalPhraseAnalysis>,
}

impl RealizedCardinal {
    pub fn new(
        value: u16,
        cell: CompoundCardinalCell,
        government: NumeralGovernment,
        analyses: Vec<CardinalPhraseAnalysis>,
    ) -> Result<Self, InflectionError> {
        if value == 0 || analyses.is_empty() {
            return Err(InflectionError::InvalidInput {
                reason: "a realized cardinal requires a positive value and an analysis".to_string(),
            });
        }
        if analyses
            .iter()
            .any(|analysis| !valid_cardinal_tokens(&analysis.tokens))
        {
            return Err(InflectionError::InvalidInput {
                reason: "a cardinal analysis has an invalid component sequence".to_string(),
            });
        }
        Ok(Self {
            value,
            cell,
            government,
            analyses,
        })
    }

    pub const fn value(&self) -> u16 {
        self.value
    }

    pub const fn cell(&self) -> CompoundCardinalCell {
        self.cell
    }

    pub const fn government(&self) -> NumeralGovernment {
        self.government
    }

    pub fn analyses(&self) -> &[CardinalPhraseAnalysis] {
        &self.analyses
    }

    /// Render the deterministic first structural analysis and each token's
    /// source-first form without discarding the full analyses.
    pub fn primary_text(&self) -> String {
        self.analyses[0].primary_text()
    }
}

fn valid_cardinal_tokens(tokens: &[PhraseToken]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    tokens
        .split(|token| token.role == PhraseRole::Conjunction)
        .all(|segment| {
            matches!(
                segment
                    .iter()
                    .map(|token| token.role)
                    .collect::<Vec<_>>()
                    .as_slice(),
                [PhraseRole::Numeral]
                    | [PhraseRole::Numeral, PhraseRole::Numeral]
                    | [
                        PhraseRole::Numeral,
                        PhraseRole::Preposition,
                        PhraseRole::Numeral
                    ]
            )
        })
        && tokens
            .first()
            .is_some_and(|token| token.role == PhraseRole::Numeral)
        && tokens
            .last()
            .is_some_and(|token| token.role == PhraseRole::Numeral)
}

/// The source-exhaustive simple ordinal identities from first through tenth.
///
/// Polivanova's paradigmatic dictionary lists exactly these ten numeral
/// adjectives in class `2/a`. Nine use the hard subtype. `третии` instead has
/// the explicit workstem `трет.ьj` and therefore requires its own boundary
/// realization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OrdinalNumeralIdentity {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
    Ninth,
    Tenth,
}

impl OrdinalNumeralIdentity {
    pub const ALL: [Self; 10] = [
        Self::First,
        Self::Second,
        Self::Third,
        Self::Fourth,
        Self::Fifth,
        Self::Sixth,
        Self::Seventh,
        Self::Eighth,
        Self::Ninth,
        Self::Tenth,
    ];

    pub const fn value(self) -> u8 {
        match self {
            Self::First => 1,
            Self::Second => 2,
            Self::Third => 3,
            Self::Fourth => 4,
            Self::Fifth => 5,
            Self::Sixth => 6,
            Self::Seventh => 7,
            Self::Eighth => 8,
            Self::Ninth => 9,
            Self::Tenth => 10,
        }
    }

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::First => "прьвъ",
            Self::Second => "въторъ",
            Self::Third => "третии",
            Self::Fourth => "четврьтъ",
            Self::Fifth => "пѧтъ",
            Self::Sixth => "шестъ",
            Self::Seventh => "седмъ",
            Self::Eighth => "осмъ",
            Self::Ninth => "девѧтъ",
            Self::Tenth => "десѧтъ",
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::First => &["прьвъ"],
            Self::Second => &["въторъ"],
            // `третии` is the Polivanova citation; `трети` is the dictionary
            // and PROIEL graphic realization of the same ordinal identity.
            Self::Third => &["третии", "трети"],
            Self::Fourth => &["четврьтъ"],
            Self::Fifth => &["пѧтъ"],
            Self::Sixth => &["шестъ"],
            Self::Seventh => &["седмъ"],
            Self::Eighth => &["осмъ"],
            Self::Ninth => &["девѧтъ"],
            Self::Tenth => &["десѧтъ"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::Third => RuleId::NumeralOrdinalJ,
            _ => RuleId::NumeralOrdinalHard,
        }
    }

    pub const fn authority(self) -> &'static str {
        "Polivanova 2023 §§70, 72, 285, 299, 303–306; Polivanova OSD spreadsheet; UD OCS PROIEL r2.18 crosscheck"
    }

    const fn stem(self) -> &'static str {
        match self {
            Self::First => "прьв",
            Self::Second => "вътор",
            Self::Third => "трет",
            Self::Fourth => "четврьт",
            Self::Fifth => "пѧт",
            Self::Sixth => "шест",
            Self::Seventh => "седм",
            Self::Eighth => "осм",
            Self::Ninth => "девѧт",
            Self::Tenth => "десѧт",
        }
    }
}

/// Decline one simple ordinal through the complete adjective agreement space.
pub fn decline_ordinal(
    identity: OrdinalNumeralIdentity,
    cell: AdjectiveCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    let mut prediction = if identity == OrdinalNumeralIdentity::Third {
        crate::adjective::decline_j_stem(identity.stem(), cell)?
    } else {
        crate::adjective::decline_stem(identity.stem(), AdjectiveClass::Hard, cell)?
    };
    let rule_id = identity.rule_id();
    prediction.trace.push(RuleStep {
        rule_id,
        before: identity.canonical_lemma().to_string(),
        after: prediction.text.clone(),
        reason: "apply the reviewed ordinal-adjective class and agreement profile",
    });
    prediction.rule_id = rule_id;
    let status = if cell.form == AdjectiveForm::Short
        && cell.case == Case::Nominative
        && cell.number == Number::Singular
        && cell.gender == Gender::Masculine
    {
        NumeralVariantStatus::ReviewedTable
    } else {
        NumeralVariantStatus::ProductiveRule
    };
    let mut variants = vec![NumeralVariant { prediction, status }];
    if identity == OrdinalNumeralIdentity::Third {
        if let Some(text) = third_ordinal_corpus_variant(cell) {
            variants.push(NumeralVariant::corpus(
                text,
                rule_id,
                identity.canonical_lemma(),
                "retain the cell-specific spelling attested in UD OCS PROIEL r2.18",
            ));
        }
    }
    Ok(variants)
}

fn third_ordinal_corpus_variant(cell: AdjectiveCell) -> Option<&'static str> {
    use Case::*;
    use Gender::*;
    use Number::*;
    match (cell.form, cell.case, cell.number, cell.gender, cell.animacy) {
        (AdjectiveForm::Long, Genitive, Singular, Masculine | Neuter, _) => Some("третиѣаго"),
        (AdjectiveForm::Long, Dative, Singular, Masculine, _) => Some("третию҄моу"),
        (AdjectiveForm::Long, Accusative, Singular, Neuter, _) => Some("третиее"),
        _ => None,
    }
}

/// The source-exhaustive simple cardinal identities from one through ten.
///
/// The two spellings of `ѥдинъ` are separate lexical doublets in Polivanova's
/// paradigmatic dictionary. `оба` is included because it has an independent
/// lexical identity and the same cardinal agreement behavior as `дъва`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CardinalNumeralIdentity {
    OneYedin,
    OneYedyn,
    TwoDva,
    BothOba,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

impl CardinalNumeralIdentity {
    pub const ALL: [Self; 12] = [
        Self::OneYedin,
        Self::OneYedyn,
        Self::TwoDva,
        Self::BothOba,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
    ];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::OneYedin => "ѥдинъ",
            Self::OneYedyn => "ѥдьнъ",
            Self::TwoDva => "дъва",
            Self::BothOba => "оба",
            Self::Three => "триѥ",
            Self::Four => "четыре",
            Self::Five => "пѧть",
            Self::Six => "шесть",
            Self::Seven => "седмь",
            Self::Eight => "осмь",
            Self::Nine => "девѧть",
            Self::Ten => "десѧть",
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::OneYedin => &["ѥдинъ", "единъ"],
            Self::OneYedyn => &["ѥдьнъ", "едьнъ"],
            Self::TwoDva => &["дъва"],
            Self::BothOba => &["оба"],
            Self::Three => &["триѥ", "трьѥ"],
            Self::Four => &["четыре", "четꙑре", "чєтꙑрє"],
            Self::Five => &["пѧть"],
            Self::Six => &["шесть", "шєсть"],
            Self::Seven => &["седмь", "сєдмь"],
            Self::Eight => &["осмь"],
            Self::Nine => &["девѧть", "дєвѧть"],
            Self::Ten => &["десѧть", "дєсѧть"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    pub const fn government(self) -> NumeralGovernment {
        match self {
            Self::OneYedin | Self::OneYedyn => NumeralGovernment::Agreement {
                number: Number::Singular,
            },
            Self::TwoDva | Self::BothOba => NumeralGovernment::Agreement {
                number: Number::Dual,
            },
            Self::Three | Self::Four => NumeralGovernment::Agreement {
                number: Number::Plural,
            },
            Self::Five | Self::Six | Self::Seven | Self::Eight | Self::Nine | Self::Ten => {
                NumeralGovernment::GenitivePlural
            }
        }
    }

    pub const fn authority(self) -> &'static str {
        match self {
            Self::OneYedin | Self::OneYedyn | Self::TwoDva | Self::BothOba => {
                "Polivanova 2023 §§314–316; UT OCS Online §44.1–2"
            }
            Self::Three => "Polivanova 2023 §§321–322; UT OCS Online §44.3",
            Self::Four => "Polivanova 2023 §§383–384; UT OCS Online §44.4",
            Self::Five | Self::Six | Self::Seven | Self::Eight | Self::Nine => {
                "Polivanova 2023 §§349–351; UT OCS Online §44.5–10"
            }
            Self::Ten => "Polivanova 2023 §§373–374; UT OCS Online §44.5–10",
        }
    }
}

/// Lexical magnitude heads used by higher cardinal constructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CardinalMagnitudeIdentity {
    HundredSto,
    ThousandBackYus,
    ThousandLittleYus,
    MyriadTma,
}

impl CardinalMagnitudeIdentity {
    pub const ALL: [Self; 4] = [
        Self::HundredSto,
        Self::ThousandBackYus,
        Self::ThousandLittleYus,
        Self::MyriadTma,
    ];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::HundredSto => "съто",
            Self::ThousandBackYus => "тꙑсѫщи",
            Self::ThousandLittleYus => "тꙑсѧщи",
            Self::MyriadTma => "тъма",
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::HundredSto => &["съто"],
            Self::ThousandBackYus => &["тꙑсѫщи", "тысѫщи", "тꙑсѫшти", "тысѫшти"],
            Self::ThousandLittleYus => &["тꙑсѧщи", "тысѧщи", "тꙑсѧшти", "тысѧшти"],
            Self::MyriadTma => &["тъма"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    pub const fn government(self) -> NumeralGovernment {
        NumeralGovernment::GenitivePlural
    }

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::HundredSto => RuleId::NumeralCardinalHundred,
            Self::ThousandBackYus | Self::ThousandLittleYus => RuleId::NumeralCardinalThousand,
            Self::MyriadTma => RuleId::NumeralCardinalMyriad,
        }
    }

    pub const fn authority(self) -> &'static str {
        match self {
            Self::HundredSto => "UT OCS Online §44.100",
            Self::ThousandBackYus | Self::ThousandLittleYus => {
                "Polivanova 2023 §§345–348; UT OCS Online §44.1000"
            }
            Self::MyriadTma => "UT OCS Online §44.10,000",
        }
    }
}

/// Lexical choices that remain independent of the integer being composed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardinalCompositionOptions {
    pub one_identity: CardinalNumeralIdentity,
    pub thousand_identity: CardinalMagnitudeIdentity,
}

impl CardinalCompositionOptions {
    pub const DEFAULT: Self = Self {
        one_identity: CardinalNumeralIdentity::OneYedin,
        thousand_identity: CardinalMagnitudeIdentity::ThousandBackYus,
    };
}

/// Decline one magnitude head independently of its multiplicative context.
pub fn decline_magnitude(
    identity: CardinalMagnitudeIdentity,
    cell: NumeralCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    if cell.gender.is_some() {
        return Err(InflectionError::historically_invalid(
            identity.canonical_lemma(),
            RequestedCell::Numeral(cell),
        ));
    }
    let noun_cell = NounCell {
        case: cell.case,
        number: cell.number,
    };
    let result = match identity {
        CardinalMagnitudeIdentity::HundredSto => {
            decline_regular_magnitude(identity, noun_cell, NounClass::ONeuterHard, Gender::Neuter)
        }
        CardinalMagnitudeIdentity::MyriadTma => {
            decline_regular_magnitude(identity, noun_cell, NounClass::AHard, Gender::Feminine)
        }
        CardinalMagnitudeIdentity::ThousandBackYus => decline_thousand(identity, noun_cell, 'ѫ'),
        CardinalMagnitudeIdentity::ThousandLittleYus => decline_thousand(identity, noun_cell, 'ѧ'),
    };
    result.map_err(|error| remap_cell_error(error, identity.canonical_lemma(), cell))
}

fn decline_regular_magnitude(
    identity: CardinalMagnitudeIdentity,
    cell: NounCell,
    class: NounClass,
    gender: Gender,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    let prediction = crate::noun::decline(
        &NounLexeme {
            lemma: identity.canonical_lemma().to_string(),
            class,
            gender,
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        },
        cell,
    )?;
    if cell.case == Case::Nominative && cell.number == Number::Singular {
        Ok(vec![NumeralVariant::reviewed(
            &prediction.text,
            identity.rule_id(),
            identity.canonical_lemma(),
            "select the source-listed cardinal-magnitude citation form",
        )])
    } else {
        Ok(vec![productive_magnitude(identity, prediction)])
    }
}

fn decline_thousand(
    identity: CardinalMagnitudeIdentity,
    cell: NounCell,
    nasal: char,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    let canonical = identity.canonical_lemma();
    let canonical_form = if cell.case == Case::Nominative && cell.number == Number::Singular {
        NumeralVariant::reviewed(
            canonical,
            RuleId::NumeralCardinalThousand,
            canonical,
            "select the exceptional source-listed thousand nominative singular",
        )
    } else {
        productive_magnitude(
            identity,
            crate::noun::decline(
                &NounLexeme {
                    lemma: canonical.to_string(),
                    class: NounClass::JaSoft,
                    gender: Gender::Feminine,
                    animacy: Animacy::Inanimate,
                    number_restriction: NumberRestriction::All,
                },
                cell,
            )?,
        )
    };
    let expanded_lemma = format!("тꙑс{nasal}шти");
    let expanded_text = if cell.case == Case::Nominative && cell.number == Number::Singular {
        expanded_lemma.clone()
    } else if matches!(
        cell.case,
        Case::Nominative | Case::Accusative | Case::Vocative
    ) && cell.number == Number::Plural
    {
        format!("тꙑс{nasal}штѧ")
    } else {
        crate::noun::decline(
            &NounLexeme {
                lemma: expanded_lemma.clone(),
                class: NounClass::JaSoft,
                gender: Gender::Feminine,
                animacy: Animacy::Inanimate,
                number_restriction: NumberRestriction::All,
            },
            cell,
        )?
        .text
    };
    let expanded = if cell.case == Case::Nominative
        && matches!(cell.number, Number::Singular | Number::Plural)
    {
        NumeralVariant::reviewed(
            &expanded_text,
            RuleId::NumeralCardinalThousand,
            canonical,
            "retain the UT source spelling of the thousand profile",
        )
    } else {
        NumeralVariant::productive_text(
            &expanded_text,
            RuleId::NumeralCardinalThousand,
            canonical,
            "apply the reviewed ja-stem oblique profile to the UT thousand spelling",
        )
    };
    Ok(vec![canonical_form, expanded])
}

fn productive_magnitude(
    identity: CardinalMagnitudeIdentity,
    mut prediction: PredictedForm,
) -> NumeralVariant {
    let rule_id = identity.rule_id();
    prediction.trace.push(RuleStep {
        rule_id,
        before: identity.canonical_lemma().to_string(),
        after: prediction.text.clone(),
        reason: "apply the reviewed declensional class of the cardinal magnitude",
    });
    prediction.rule_id = rule_id;
    NumeralVariant::productive(prediction)
}

/// Return every source-ordered realization of one simple-cardinal cell.
pub fn decline_cardinal(
    identity: CardinalNumeralIdentity,
    cell: NumeralCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    validate_cell_shape(identity, cell)?;
    let forms = match identity {
        CardinalNumeralIdentity::OneYedin => {
            decline_pronominal(StandardPronominalIdentity::IndefiniteYedin, identity, cell)
        }
        CardinalNumeralIdentity::OneYedyn => {
            decline_pronominal(StandardPronominalIdentity::IndefiniteYedyn, identity, cell)
        }
        CardinalNumeralIdentity::TwoDva => {
            decline_pronominal(StandardPronominalIdentity::NumeralDva, identity, cell)
        }
        CardinalNumeralIdentity::BothOba => {
            decline_pronominal(StandardPronominalIdentity::NumeralOba, identity, cell)
        }
        CardinalNumeralIdentity::Three => decline_three(cell),
        CardinalNumeralIdentity::Four => decline_four(cell),
        CardinalNumeralIdentity::Five
        | CardinalNumeralIdentity::Six
        | CardinalNumeralIdentity::Seven
        | CardinalNumeralIdentity::Eight
        | CardinalNumeralIdentity::Nine => decline_i_stem(identity, cell),
        CardinalNumeralIdentity::Ten => decline_ten(cell),
    };
    forms.map_err(|error| remap_cell_error(error, identity.canonical_lemma(), cell))
}

fn validate_cell_shape(
    identity: CardinalNumeralIdentity,
    cell: NumeralCell,
) -> Result<(), InflectionError> {
    let valid = match identity.government() {
        NumeralGovernment::Agreement { number } => cell.number == number && cell.gender.is_some(),
        NumeralGovernment::GenitivePlural if identity == CardinalNumeralIdentity::Ten => {
            cell.gender.is_none()
        }
        NumeralGovernment::GenitivePlural => {
            cell.number == Number::Singular && cell.gender.is_none()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(InflectionError::historically_invalid(
            identity.canonical_lemma(),
            RequestedCell::Numeral(cell),
        ))
    }
}

fn decline_pronominal(
    pronominal: StandardPronominalIdentity,
    identity: CardinalNumeralIdentity,
    cell: NumeralCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    let gender = cell.gender.ok_or_else(|| {
        InflectionError::historically_invalid(
            identity.canonical_lemma(),
            RequestedCell::Numeral(cell),
        )
    })?;
    crate::pronoun::decline_standard_pronominal(pronominal, cell.case, cell.number, gender)
        .map(|prediction| vec![NumeralVariant::productive(prediction)])
}

fn decline_three(cell: NumeralCell) -> Result<Vec<NumeralVariant>, InflectionError> {
    let gender = required_gender(CardinalNumeralIdentity::Three, cell)?;
    let text = match (cell.case, gender) {
        (Case::Nominative, Gender::Masculine) => "триѥ",
        (Case::Nominative, Gender::Neuter | Gender::Feminine) | (Case::Accusative, _) => "три",
        (Case::Genitive, _) => "трии",
        (Case::Locative, _) => "трьхъ",
        (Case::Dative, _) => "трьмъ",
        (Case::Instrumental, _) => "трьми",
        (Case::Vocative, _) => {
            return Err(InflectionError::historically_invalid(
                CardinalNumeralIdentity::Three.canonical_lemma(),
                RequestedCell::Numeral(cell),
            ));
        }
    };
    Ok(vec![NumeralVariant::reviewed(
        text,
        RuleId::NumeralCardinalThree,
        CardinalNumeralIdentity::Three.canonical_lemma(),
        "select the unique plural-only cardinal-three cell",
    )])
}

fn decline_four(cell: NumeralCell) -> Result<Vec<NumeralVariant>, InflectionError> {
    let gender = required_gender(CardinalNumeralIdentity::Four, cell)?;
    let text = match (cell.case, gender) {
        (Case::Nominative, Gender::Masculine) => "четыре",
        (Case::Nominative, Gender::Neuter | Gender::Feminine) | (Case::Accusative, _) => "четыри",
        (Case::Genitive, _) => "четыръ",
        (Case::Locative, _) => "четырехъ",
        (Case::Dative, _) => "четыремъ",
        (Case::Instrumental, _) => "четырьми",
        (Case::Vocative, _) => {
            return Err(InflectionError::historically_invalid(
                CardinalNumeralIdentity::Four.canonical_lemma(),
                RequestedCell::Numeral(cell),
            ));
        }
    };
    Ok(vec![NumeralVariant::reviewed(
        text,
        RuleId::NumeralCardinalFour,
        CardinalNumeralIdentity::Four.canonical_lemma(),
        "select the unique plural-only cardinal-four cell",
    )])
}

fn decline_i_stem(
    identity: CardinalNumeralIdentity,
    cell: NumeralCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    crate::noun::decline(
        &NounLexeme {
            lemma: identity.canonical_lemma().to_string(),
            class: NounClass::IFeminine,
            gender: Gender::Feminine,
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::SingularOnly,
        },
        NounCell {
            case: cell.case,
            number: cell.number,
        },
    )
    .map(|prediction| vec![NumeralVariant::productive(prediction)])
}

fn decline_ten(cell: NumeralCell) -> Result<Vec<NumeralVariant>, InflectionError> {
    let lemma = CardinalNumeralIdentity::Ten.canonical_lemma();
    let productive = crate::noun::decline(
        &NounLexeme {
            lemma: lemma.to_string(),
            class: NounClass::IFeminine,
            gender: Gender::Feminine,
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        },
        NounCell {
            case: cell.case,
            number: cell.number,
        },
    )?;
    let primary = match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => Some("десѧть"),
        (Case::Locative, Number::Singular) => Some("десѧти"),
        (Case::Instrumental, Number::Singular) => Some("десѧтиѭ"),
        (Case::Nominative | Case::Accusative, Number::Dual) => Some("десѧти"),
        (Case::Dative | Case::Instrumental, Number::Dual) => Some("десѧтьма"),
        (Case::Nominative, Number::Plural) => Some("десѧте"),
        (Case::Accusative, Number::Plural) => Some("десѧти"),
        (Case::Genitive, Number::Plural) => Some("десѧтъ"),
        (Case::Locative, Number::Plural) => Some("десѧтехъ"),
        (Case::Dative, Number::Plural) => Some("десѧтемъ"),
        (Case::Instrumental, Number::Plural) => Some("десѧты"),
        _ => None,
    };
    let mut forms = Vec::new();
    if let Some(text) = primary {
        forms.push(NumeralVariant::reviewed(
            text,
            RuleId::NumeralCardinalTen,
            lemma,
            "select the attested mixed-declension cardinal-ten cell",
        ));
    }
    let secondary = match (cell.case, cell.number) {
        (Case::Locative, Number::Singular) => Some("десѧте"),
        (Case::Nominative | Case::Accusative, Number::Dual) => Some("десѧтѣ"),
        (Case::Dative, Number::Plural) => Some("десѧтьмъ"),
        _ => None,
    };
    if let Some(text) = secondary {
        forms.push(NumeralVariant::reviewed(
            text,
            RuleId::NumeralCardinalTen,
            lemma,
            "retain the source-listed secondary cardinal-ten realization",
        ));
    }
    if forms
        .iter()
        .all(|variant| variant.prediction.text != productive.text)
    {
        forms.push(NumeralVariant::productive(productive));
    }
    Ok(forms)
}

/// Decline `десѧть` when it is itself counted by two, three, or four in a
/// multiplicative tens construction.
///
/// The dual genitive `десѧту` is explicitly listed by UT §44.20; the locative
/// and vocative extend that dual GL/NA pattern productively. The plural reuses
/// the reviewed mixed simple-ten table.
pub fn decline_counted_ten(
    case: Case,
    number: Number,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    if number == Number::Plural {
        if case == Case::Nominative {
            let lemma = CardinalNumeralIdentity::Ten.canonical_lemma();
            return Ok(vec![
                NumeralVariant::reviewed(
                    "десѧте",
                    RuleId::NumeralCardinalTens,
                    lemma,
                    "select the source-listed masculine nominative plural form of counted ten",
                ),
                NumeralVariant::reviewed(
                    "десѧти",
                    RuleId::NumeralCardinalTens,
                    lemma,
                    "retain the source-listed alternative nominative plural form of counted ten",
                ),
            ]);
        }
        return decline_ten(NumeralCell {
            case,
            number,
            gender: None,
        });
    }
    let lemma = CardinalNumeralIdentity::Ten.canonical_lemma();
    if number != Number::Dual {
        return Err(InflectionError::historically_invalid(
            lemma,
            RequestedCell::Numeral(NumeralCell {
                case,
                number,
                gender: None,
            }),
        ));
    }
    let variant = match case {
        Case::Nominative | Case::Accusative => NumeralVariant::reviewed(
            "десѧти",
            RuleId::NumeralCardinalTens,
            lemma,
            "select the source-listed dual direct-case form of counted ten",
        ),
        Case::Genitive => NumeralVariant::reviewed(
            "десѧту",
            RuleId::NumeralCardinalTens,
            lemma,
            "select the source-listed genitive dual form of counted ten",
        ),
        Case::Dative | Case::Instrumental => NumeralVariant::reviewed(
            "десѧтьма",
            RuleId::NumeralCardinalTens,
            lemma,
            "select the source-listed dual oblique form of counted ten",
        ),
        Case::Locative => NumeralVariant::productive_text(
            "десѧту",
            RuleId::NumeralCardinalTens,
            lemma,
            "extend the dual genitive-locative syncretism to counted ten",
        ),
        Case::Vocative => NumeralVariant::productive_text(
            "десѧти",
            RuleId::NumeralCardinalTens,
            lemma,
            "extend the dual nominative-vocative syncretism to counted ten",
        ),
    };
    Ok(vec![variant])
}

fn required_gender(
    identity: CardinalNumeralIdentity,
    cell: NumeralCell,
) -> Result<Gender, InflectionError> {
    cell.gender.ok_or_else(|| {
        InflectionError::historically_invalid(
            identity.canonical_lemma(),
            RequestedCell::Numeral(cell),
        )
    })
}

fn remap_cell_error(error: InflectionError, lemma: &str, cell: NumeralCell) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. } => {
            InflectionError::historically_invalid(lemma, RequestedCell::Numeral(cell))
        }
        InflectionError::UnsupportedCell { .. } => {
            InflectionError::unsupported(lemma, RequestedCell::Numeral(cell))
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(case: Case, number: Number, gender: Option<Gender>) -> NumeralCell {
        NumeralCell {
            case,
            number,
            gender,
        }
    }

    #[test]
    fn simple_cardinal_inventory_is_exhaustive_and_nonoverlapping() {
        assert_eq!(CardinalNumeralIdentity::ALL.len(), 12);
        let mut aliases = std::collections::BTreeSet::new();
        for identity in CardinalNumeralIdentity::ALL {
            assert_eq!(
                CardinalNumeralIdentity::classify_source_union_lemma(identity.canonical_lemma()),
                Some(identity)
            );
            for alias in identity.source_union_aliases() {
                assert!(aliases.insert(*alias), "duplicate cardinal alias {alias}");
            }
        }
    }

    #[test]
    fn simple_ordinal_inventory_is_exhaustive_and_nonoverlapping() {
        let citations = [
            "прьвъ",
            "въторъ",
            "третии",
            "четврьтъ",
            "пѧтъ",
            "шестъ",
            "седмъ",
            "осмъ",
            "девѧтъ",
            "десѧтъ",
        ];
        assert_eq!(OrdinalNumeralIdentity::ALL.len(), citations.len());
        let mut aliases = std::collections::BTreeSet::new();
        for (index, (identity, citation)) in OrdinalNumeralIdentity::ALL
            .into_iter()
            .zip(citations)
            .enumerate()
        {
            assert_eq!(identity.value(), (index + 1) as u8);
            assert_eq!(identity.canonical_lemma(), citation);
            assert_eq!(
                OrdinalNumeralIdentity::classify_source_union_lemma(citation),
                Some(identity)
            );
            for alias in identity.source_union_aliases() {
                assert!(aliases.insert(*alias), "duplicate ordinal alias {alias}");
            }
        }
        assert_eq!(
            OrdinalNumeralIdentity::classify_source_union_lemma("трети"),
            Some(OrdinalNumeralIdentity::Third)
        );
    }

    #[test]
    fn every_simple_ordinal_licenses_all_adjective_cells() {
        for identity in OrdinalNumeralIdentity::ALL {
            let outcomes = AdjectiveCell::all()
                .map(|cell| {
                    (
                        cell,
                        decline_ordinal(identity, cell)
                            .unwrap_or_else(|error| panic!("{identity:?} {cell:?}: {error}")),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(outcomes.len(), 252, "{identity:?}");
            assert!(
                outcomes
                    .iter()
                    .flat_map(|(_, variants)| variants)
                    .all(|variant| variant.prediction.rule_id == identity.rule_id())
            );
            assert_eq!(
                outcomes
                    .iter()
                    .flat_map(|(_, variants)| variants)
                    .filter(|variant| variant.status == NumeralVariantStatus::ReviewedTable)
                    .count(),
                2,
                "the two animacy projections of the citation cell are reviewed"
            );
        }
    }

    #[test]
    fn hard_ordinals_reuse_the_adjective_profile_exactly() {
        for identity in OrdinalNumeralIdentity::ALL
            .into_iter()
            .filter(|identity| *identity != OrdinalNumeralIdentity::Third)
        {
            for cell in AdjectiveCell::all() {
                let ordinal = decline_ordinal(identity, cell)
                    .expect("licensed hard ordinal")
                    .remove(0)
                    .prediction
                    .text;
                let adjective =
                    crate::adjective::decline_stem(identity.stem(), AdjectiveClass::Hard, cell)
                        .expect("licensed hard adjective")
                        .text;
                assert_eq!(ordinal, adjective, "{identity:?} {cell:?}");
            }
        }
    }

    #[test]
    fn third_ordinal_uses_the_reviewed_yer_j_profile() {
        let form = |case, number, gender, adjective_form| {
            decline_ordinal(
                OrdinalNumeralIdentity::Third,
                AdjectiveCell {
                    case,
                    number,
                    gender,
                    animacy: Animacy::Inanimate,
                    form: adjective_form,
                },
            )
            .expect("licensed third-ordinal cell")[0]
                .prediction
                .text
                .clone()
        };
        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                AdjectiveForm::Short,
            ),
            "третии"
        );
        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Neuter,
                AdjectiveForm::Short,
            ),
            "третиѥ"
        );
        assert_eq!(
            form(
                Case::Nominative,
                Number::Singular,
                Gender::Feminine,
                AdjectiveForm::Short,
            ),
            "третиꙗ"
        );
        assert_eq!(
            form(
                Case::Accusative,
                Number::Singular,
                Gender::Feminine,
                AdjectiveForm::Short,
            ),
            "третиѭ"
        );
        assert_eq!(
            form(
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                AdjectiveForm::Short,
            ),
            "третиꙗ"
        );
        assert_eq!(
            form(
                Case::Nominative,
                Number::Plural,
                Gender::Feminine,
                AdjectiveForm::Short,
            ),
            "третиѩ"
        );
    }

    #[test]
    fn third_ordinal_retains_cell_specific_proiel_spellings() {
        let variants = |form, case, gender| {
            decline_ordinal(
                OrdinalNumeralIdentity::Third,
                AdjectiveCell {
                    case,
                    number: Number::Singular,
                    gender,
                    animacy: Animacy::Inanimate,
                    form,
                },
            )
            .expect("licensed third-ordinal cell")
        };
        for (case, gender, expected) in [
            (Case::Genitive, Gender::Masculine, "третиѣаго"),
            (Case::Dative, Gender::Masculine, "третию҄моу"),
            (Case::Accusative, Gender::Neuter, "третиее"),
        ] {
            let forms = variants(AdjectiveForm::Long, case, gender);
            assert_eq!(forms[1].prediction.text, expected);
            assert_eq!(forms[1].status, NumeralVariantStatus::CorpusAttestation);
        }
    }

    #[test]
    fn magnitude_inventory_is_exhaustive_nonoverlapping_and_ungendered() {
        assert_eq!(CardinalMagnitudeIdentity::ALL.len(), 4);
        let mut aliases = std::collections::BTreeSet::new();
        for identity in CardinalMagnitudeIdentity::ALL {
            assert_eq!(
                CardinalMagnitudeIdentity::classify_source_union_lemma(identity.canonical_lemma()),
                Some(identity)
            );
            for alias in identity.source_union_aliases() {
                assert!(aliases.insert(*alias), "duplicate magnitude alias {alias}");
            }
            let outcomes = NumeralCell::all()
                .map(|cell| (cell, decline_magnitude(identity, cell)))
                .collect::<Vec<_>>();
            assert_eq!(
                outcomes.iter().filter(|(_, result)| result.is_ok()).count(),
                21,
                "{identity:?}"
            );
            assert!(
                outcomes
                    .iter()
                    .filter(|(cell, _)| cell.gender.is_some())
                    .all(|(_, result)| matches!(
                        result,
                        Err(InflectionError::HistoricallyInvalidCell { .. })
                    ))
            );
        }
    }

    #[test]
    fn magnitude_goldens_cover_hundred_thousand_and_myriad_profiles() {
        let hundred = decline_magnitude(
            CardinalMagnitudeIdentity::HundredSto,
            cell(Case::Genitive, Number::Singular, None),
        )
        .expect("hundred genitive singular");
        assert_eq!(hundred[0].prediction.text, "съта");
        assert_eq!(
            hundred[0].prediction.rule_id,
            RuleId::NumeralCardinalHundred
        );
        assert_eq!(hundred[0].status, NumeralVariantStatus::ProductiveRule);

        let thousand_nominative = decline_magnitude(
            CardinalMagnitudeIdentity::ThousandBackYus,
            cell(Case::Nominative, Number::Singular, None),
        )
        .expect("thousand nominative singular");
        assert_eq!(
            thousand_nominative
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["тꙑсѫщи", "тꙑсѫшти"]
        );
        assert!(
            thousand_nominative
                .iter()
                .all(|variant| variant.status == NumeralVariantStatus::ReviewedTable)
        );

        let thousand_plural = decline_magnitude(
            CardinalMagnitudeIdentity::ThousandLittleYus,
            cell(Case::Nominative, Number::Plural, None),
        )
        .expect("thousand nominative plural");
        assert_eq!(
            thousand_plural
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["тꙑсѧщѩ", "тꙑсѧштѧ"]
        );
        assert_eq!(
            thousand_plural[0].status,
            NumeralVariantStatus::ProductiveRule
        );
        assert_eq!(
            thousand_plural[1].status,
            NumeralVariantStatus::ReviewedTable
        );

        let thousand_accusative = decline_magnitude(
            CardinalMagnitudeIdentity::ThousandBackYus,
            cell(Case::Accusative, Number::Singular, None),
        )
        .expect("thousand accusative singular");
        assert_eq!(
            thousand_accusative
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["тꙑсѫщѭ", "тꙑсѫштѭ"]
        );

        let myriad = decline_magnitude(
            CardinalMagnitudeIdentity::MyriadTma,
            cell(Case::Accusative, Number::Singular, None),
        )
        .expect("myriad accusative singular");
        assert_eq!(myriad[0].prediction.text, "тъмѫ");
        assert_eq!(myriad[0].prediction.rule_id, RuleId::NumeralCardinalMyriad);
    }

    #[test]
    fn one_through_nine_enforce_real_agreement_and_government_shapes() {
        let expected_valid = [18, 18, 18, 18, 18, 18, 7, 7, 7, 7, 7];
        for (identity, expected_valid) in CardinalNumeralIdentity::ALL
            .into_iter()
            .take(11)
            .zip(expected_valid)
        {
            let valid = NumeralCell::all()
                .filter(|cell| decline_cardinal(identity, *cell).is_ok())
                .count();
            assert_eq!(valid, expected_valid, "{identity:?}");
        }
    }

    #[test]
    fn source_goldens_cover_each_simple_cardinal_profile() {
        let goldens = [
            (
                CardinalNumeralIdentity::OneYedin,
                cell(Case::Genitive, Number::Singular, Some(Gender::Feminine)),
                "ѥдиноѩ",
            ),
            (
                CardinalNumeralIdentity::OneYedyn,
                cell(Case::Dative, Number::Singular, Some(Gender::Masculine)),
                "ѥдьному",
            ),
            (
                CardinalNumeralIdentity::TwoDva,
                cell(Case::Genitive, Number::Dual, Some(Gender::Masculine)),
                "дъвою",
            ),
            (
                CardinalNumeralIdentity::BothOba,
                cell(Case::Nominative, Number::Dual, Some(Gender::Feminine)),
                "обѣ",
            ),
            (
                CardinalNumeralIdentity::Three,
                cell(Case::Nominative, Number::Plural, Some(Gender::Masculine)),
                "триѥ",
            ),
            (
                CardinalNumeralIdentity::Four,
                cell(Case::Instrumental, Number::Plural, Some(Gender::Neuter)),
                "четырьми",
            ),
            (
                CardinalNumeralIdentity::Five,
                cell(Case::Genitive, Number::Singular, None),
                "пѧти",
            ),
            (
                CardinalNumeralIdentity::Six,
                cell(Case::Instrumental, Number::Singular, None),
                "шестьѭ",
            ),
            (
                CardinalNumeralIdentity::Seven,
                cell(Case::Locative, Number::Singular, None),
                "седми",
            ),
            (
                CardinalNumeralIdentity::Eight,
                cell(Case::Dative, Number::Singular, None),
                "осми",
            ),
            (
                CardinalNumeralIdentity::Nine,
                cell(Case::Accusative, Number::Singular, None),
                "девѧть",
            ),
        ];
        for (identity, cell, expected) in goldens {
            assert_eq!(
                decline_cardinal(identity, cell).expect("licensed cardinal")[0]
                    .prediction
                    .text,
                expected
            );
        }
    }

    #[test]
    fn ten_preserves_mixed_attested_and_productive_variants() {
        let nominative_plural = decline_cardinal(
            CardinalNumeralIdentity::Ten,
            cell(Case::Nominative, Number::Plural, None),
        )
        .expect("ten nominative plural");
        assert_eq!(
            nominative_plural
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["десѧте", "десѧти"]
        );
        assert_eq!(
            decline_cardinal(
                CardinalNumeralIdentity::Ten,
                cell(Case::Locative, Number::Singular, None),
            )
            .expect("ten locative singular")
            .iter()
            .map(|variant| variant.prediction.text.as_str())
            .collect::<Vec<_>>(),
            ["десѧти", "десѧте"]
        );
        let dative_plural = decline_cardinal(
            CardinalNumeralIdentity::Ten,
            cell(Case::Dative, Number::Plural, None),
        )
        .expect("ten dative plural");
        assert_eq!(
            dative_plural
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["десѧтемъ", "десѧтьмъ"]
        );
        assert!(
            dative_plural
                .iter()
                .all(|variant| variant.status == NumeralVariantStatus::ReviewedTable)
        );
        let nominative_singular = decline_cardinal(
            CardinalNumeralIdentity::Ten,
            cell(Case::Nominative, Number::Singular, None),
        )
        .expect("ten nominative singular");
        assert_eq!(nominative_singular.len(), 1);
        assert_eq!(
            nominative_singular[0].status,
            NumeralVariantStatus::ReviewedTable
        );
        assert_eq!(
            NumeralCell::all()
                .filter(|cell| decline_cardinal(CardinalNumeralIdentity::Ten, *cell).is_ok())
                .count(),
            21
        );
    }

    #[test]
    fn counted_ten_preserves_the_attested_twenty_genitive() {
        let genitive =
            decline_counted_ten(Case::Genitive, Number::Dual).expect("counted ten genitive dual");
        assert_eq!(genitive[0].prediction.text, "десѧту");
        assert_eq!(genitive[0].status, NumeralVariantStatus::ReviewedTable);
        let locative =
            decline_counted_ten(Case::Locative, Number::Dual).expect("counted ten locative dual");
        assert_eq!(locative[0].prediction.text, "десѧту");
        assert_eq!(locative[0].status, NumeralVariantStatus::ProductiveRule);
    }
}
