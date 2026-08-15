//! Source-reviewed Old Church Slavonic cardinal-numeral morphology.

use crate::noun::NounLexeme;
use crate::pronoun::StandardPronominalIdentity;
use crate::{
    Animacy, Case, CompoundCardinalCell, Gender, InflectionError, NounCell, NounClass, Number,
    NumberRestriction, NumeralCell, PhraseRole, PhraseToken, PredictedForm, RequestedCell, RuleId,
    RuleStep,
};

/// The syntactic relation between a simple cardinal and the enumerated noun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumeralGovernment {
    /// The numeral and noun agree in the numeral's inherent grammatical number.
    Agreement { number: Number },
    /// The numeral is substantival and governs a genitive-plural complement.
    GenitivePlural,
}

/// Evidential status of one ordered cardinal realization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumeralVariantStatus {
    /// A form explicitly present in the reviewed grammatical paradigm.
    ReviewedTable,
    /// A form licensed by a reviewed productive declensional profile.
    ProductiveRule,
}

impl NumeralVariantStatus {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ReviewedTable => "reviewed-table",
            Self::ProductiveRule => "productive-rule",
        }
    }
}

/// One ordered cardinal realization and its evidential status.
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
