//! Structured Synodal Church Slavonic numeral composition (Alypy §§61–70).
//!
//! Components remain typed words with independent form sets. Historically
//! fused spellings are represented as one composed token whose evidence and
//! trace retain every input component; genuinely multiword expressions remain
//! multiple tokens.

use std::collections::BTreeSet;

use synodal_church_slavonic_core::{
    AnalyticConstruction, Animacy, AuthorityRole, Case, Confidence, EpistemicRole, Error, Evidence,
    EvidenceId, EvidenceKind, FormSet, FormSource, FormVariant, Gender, GrammarCell, LexemeId,
    MetadataField, NounCell, Number, NumeralCell, NumeralDeclension, NumeralKind, NumeralLexeme,
    OrthographyProfile, PhraseRole, PhraseToken, RealizedPhrase, Recension, Result, RuleId,
    RuleTrace, SourceId, SynodalWord, TraceStep, decline_numeral, normalize_lookup_accentless,
};

use crate::Inflector;

pub const MIN_CARDINAL_VALUE: u32 = 1;
pub const MAX_CARDINAL_VALUE: u32 = 1_000_000;
pub const MIN_COMPOUND_ORDINAL_VALUE: u16 = 11;
pub const MAX_COMPOUND_ORDINAL_VALUE: u16 = 1_000;

/// Case, optional agreement gender, and animacy for a cardinal expression.
/// Gender is present exactly when the final component is one through four
/// (including the agreement analysis of eleven through fourteen).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CompoundNumeralCell {
    pub case: Case,
    pub gender: Option<Gender>,
    pub animacy: Animacy,
}

/// Position of the counted noun relative to a compound numeral (Alypy §66).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NumeralNounPosition {
    Following,
    Preceding,
}

/// Source-licensed relation between a numeral and the counted noun.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NumeralGovernment {
    Agreement {
        number: Number,
    },
    GenitivePlural,
    /// Marked subject, predicate, and appositional syntax from Alypy §67.
    ContextualNominativePlural,
}

/// Structural strategy used by one correlated numeral realization.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NumeralComposition {
    Simple,
    TeenFirstComponentDeclined,
    TeenSecondComponentDeclined,
    TeenBothComponentsDeclined,
    TeenBothComponentsDual,
    TensAgreement,
    TensGovernment,
    TensBothComponentsSingular,
    TensBothComponentsPlural,
    HundredsAgreement,
    HundredsGovernment,
    MagnitudeAgreement,
    MagnitudeGovernment,
    Magnitude,
    AdditiveFinalConjunction,
    AdditiveAllConjunctions,
    AdditiveAsyndetic,
    CompoundOrdinalFused,
    CompoundOrdinalAnalyticTeen,
    CompoundOrdinalAsyndetic,
    CompoundOrdinalConjunction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CardinalPhraseAnalysis {
    pub construction: NumeralComposition,
    pub tokens: Vec<PhraseToken>,
}

impl CardinalPhraseAnalysis {
    #[must_use]
    pub fn primary_text(&self) -> String {
        render_tokens(&self.tokens)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RealizedCardinal {
    value: u32,
    cell: CompoundNumeralCell,
    following_government: Vec<NumeralGovernment>,
    preceding_government: Vec<NumeralGovernment>,
    government_evidence: Vec<Evidence>,
    analyses: Vec<CardinalPhraseAnalysis>,
}

impl RealizedCardinal {
    fn new(
        value: u32,
        cell: CompoundNumeralCell,
        following_government: Vec<NumeralGovernment>,
        preceding_government: Vec<NumeralGovernment>,
        government_evidence: Vec<Evidence>,
        analyses: Vec<CardinalPhraseAnalysis>,
    ) -> Result<Self> {
        if !(MIN_CARDINAL_VALUE..=MAX_CARDINAL_VALUE).contains(&value) {
            return Err(Error::OutOfRange {
                value,
                maximum: MAX_CARDINAL_VALUE,
            });
        }
        if analyses.is_empty() || analyses.iter().any(|analysis| analysis.tokens.is_empty()) {
            return Err(Error::EmptyFormSet);
        }
        Ok(Self {
            value,
            cell,
            following_government,
            preceding_government,
            government_evidence,
            analyses,
        })
    }

    #[must_use]
    pub const fn value(&self) -> u32 {
        self.value
    }

    #[must_use]
    pub const fn cell(&self) -> CompoundNumeralCell {
        self.cell
    }

    #[must_use]
    pub fn government(&self, position: NumeralNounPosition) -> &[NumeralGovernment] {
        match position {
            NumeralNounPosition::Following => &self.following_government,
            NumeralNounPosition::Preceding => &self.preceding_government,
        }
    }

    /// Normative evidence for the agreement/government alternatives exposed
    /// by [`Self::government`].
    #[must_use]
    pub fn government_evidence(&self) -> &[Evidence] {
        &self.government_evidence
    }

    #[must_use]
    pub fn analyses(&self) -> &[CardinalPhraseAnalysis] {
        &self.analyses
    }

    #[must_use]
    pub fn primary_text(&self) -> String {
        self.analyses[0].primary_text()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct OrdinalPhraseAnalysis {
    pub construction: NumeralComposition,
    pub tokens: Vec<PhraseToken>,
}

impl OrdinalPhraseAnalysis {
    #[must_use]
    pub fn primary_text(&self) -> String {
        render_tokens(&self.tokens)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RealizedOrdinal {
    value: u16,
    cell: NumeralCell,
    analyses: Vec<OrdinalPhraseAnalysis>,
}

impl RealizedOrdinal {
    fn new(value: u16, cell: NumeralCell, analyses: Vec<OrdinalPhraseAnalysis>) -> Result<Self> {
        if value == 0 || value > MAX_COMPOUND_ORDINAL_VALUE || analyses.is_empty() {
            return Err(Error::OutOfRange {
                value: u32::from(value),
                maximum: u32::from(MAX_COMPOUND_ORDINAL_VALUE),
            });
        }
        Ok(Self {
            value,
            cell,
            analyses,
        })
    }

    #[must_use]
    pub const fn value(&self) -> u16 {
        self.value
    }

    #[must_use]
    pub const fn cell(&self) -> NumeralCell {
        self.cell
    }

    #[must_use]
    pub fn analyses(&self) -> &[OrdinalPhraseAnalysis] {
        &self.analyses
    }

    #[must_use]
    pub fn primary_text(&self) -> String {
        self.analyses[0].primary_text()
    }
}

/// Realizes every source-licensed structural strategy for a cardinal from one
/// through the largest exact simple value listed by Alypy, one million.
pub fn cardinal(value: u32, cell: CompoundNumeralCell) -> Result<RealizedCardinal> {
    cardinal_with(value, cell, Inflector::default())
}

pub fn cardinal_with(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<RealizedCardinal> {
    if !(MIN_CARDINAL_VALUE..=MAX_CARDINAL_VALUE).contains(&value) {
        return Err(Error::OutOfRange {
            value,
            maximum: MAX_CARDINAL_VALUE,
        });
    }
    if cell.case == Case::Vocative {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §§62–64 do not license a cardinal vocative".into(),
        });
    }
    let needs_gender = cardinal_requires_gender(value);
    if cell.gender.is_some() != needs_gender {
        return Err(Error::HistoricallyInvalidCell {
            reason: if needs_gender {
                "the final agreeing cardinal component requires gender"
            } else {
                "a substantival or magnitude-final cardinal has no agreement-gender dimension"
            }
            .into(),
        });
    }

    let analyses = compose_cardinal(value, cell, inflector)?;
    RealizedCardinal::new(
        value,
        cell,
        following_government(value, cell.case),
        preceding_government(value, cell.case),
        vec![numeral_evidence(
            "SYN-NUMERAL-GOVERNMENT-ALYPY-65-67",
            "Alypy (Gamanovich), §§65–67 numeral agreement, government, position, and contextual nominative",
        )],
        analyses,
    )
}

/// Realizes simple and compound ordinals through Alypy's last explicitly
/// supplied ordinal head, `тысѧщный` “thousandth”.
pub fn ordinal(value: u16, cell: NumeralCell) -> Result<RealizedOrdinal> {
    ordinal_with(value, cell, Inflector::default())
}

pub fn ordinal_with(
    value: u16,
    cell: NumeralCell,
    inflector: Inflector,
) -> Result<RealizedOrdinal> {
    if value == 0 || value > MAX_COMPOUND_ORDINAL_VALUE {
        return Err(Error::OutOfRange {
            value: u32::from(value),
            maximum: u32::from(MAX_COMPOUND_ORDINAL_VALUE),
        });
    }
    if cell.kind != NumeralKind::Ordinal || cell.gender.is_none() {
        return Err(Error::HistoricallyInvalidCell {
            reason: "a compound ordinal requires an ordinal agreement cell with gender".into(),
        });
    }
    let analyses = compose_ordinal(value, cell, inflector)?;
    RealizedOrdinal::new(value, cell, analyses)
}

/// Repeats a fully inflected cardinal as a distributive expression (`два
/// два`, Alypy §61). The cited value two is exact construction evidence; the
/// same transparent repetition is available productively for every cardinal
/// in the source-bounded range without labeling an unattested phrase attested.
pub fn repeated_distributive(value: u32, cell: CompoundNumeralCell) -> Result<Vec<RealizedPhrase>> {
    repeated_distributive_with(value, cell, Inflector::default())
}

pub fn repeated_distributive_with(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let cardinal = cardinal_with(value, cell, inflector)?;
    let mut phrases = Vec::new();
    for analysis in cardinal.analyses() {
        let first = tag_tokens(
            &analysis.tokens,
            "SYN-NUMERAL-DISTRIBUTIVE-REPETITION-ALYPY-61",
            "Alypy (Gamanovich), §61; Mark 6:7 два два",
        )?;
        let mut tokens = first.clone();
        tokens.extend(first);
        phrases.push(RealizedPhrase::new(
            AnalyticConstruction::RepeatedDistributive,
            tokens,
        )?);
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

/// Realizes a quantitative multiplicative: an inflected cardinal followed by
/// invariant `кратъ` (Alypy §70).
pub fn multiplicative_krat(value: u32, cell: CompoundNumeralCell) -> Result<Vec<RealizedPhrase>> {
    multiplicative_krat_with(value, cell, Inflector::default())
}

pub fn multiplicative_krat_with(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let cardinal = cardinal_with(value, cell, inflector)?;
    let krat = PhraseToken {
        role: PhraseRole::MultiplicativeUnit,
        forms: grammar_form(
            "кратъ",
            Some("кра́тъ"),
            "SYN-NUMERAL-MULTIPLICATIVE-KRAT-ALYPY-70",
            "Alypy (Gamanovich), §70 invariant кратъ multiplicatives",
            inflector.orthography(),
        )?,
    };
    let mut phrases = Vec::new();
    for analysis in cardinal.analyses() {
        let mut tokens = tag_tokens(
            &analysis.tokens,
            "SYN-NUMERAL-MULTIPLICATIVE-KRAT-ALYPY-70",
            "Alypy (Gamanovich), §70 invariant кратъ multiplicatives",
        )?;
        tokens.push(krat.clone());
        phrases.push(RealizedPhrase::new(
            AnalyticConstruction::MultiplicativeKrat,
            tokens,
        )?);
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

/// Realizes cardinal expressions with inflected `часть`: `єдина часть`,
/// `двѣ части`, `три части`, and their productive case/number extensions.
pub fn fractional_cardinal_parts(
    count: u32,
    case: Case,
    animacy: Animacy,
) -> Result<Vec<RealizedPhrase>> {
    fractional_cardinal_parts_with(count, case, animacy, Inflector::default())
}

pub fn fractional_cardinal_parts_with(
    count: u32,
    case: Case,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let cell = CompoundNumeralCell {
        case,
        gender: cardinal_requires_gender(count).then_some(Gender::Feminine),
        animacy,
    };
    let cardinal = cardinal_with(count, cell, inflector)?;
    fractional_cardinal_phrases(&cardinal, animacy, inflector)
}

/// Realizes an ordinal denominator agreeing with inflected `часть`, such as
/// `десѧтаѧ часть` (Alypy §70).
pub fn fractional_ordinal_parts(denominator: u16, cell: NounCell) -> Result<Vec<RealizedPhrase>> {
    fractional_ordinal_parts_with(denominator, cell, Inflector::default())
}

pub fn fractional_ordinal_parts_with(
    denominator: u16,
    cell: NounCell,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let ordinal = ordinal_with(
        denominator,
        NumeralCell {
            kind: NumeralKind::Ordinal,
            case: cell.case,
            number: cell.number,
            gender: Some(Gender::Feminine),
            animacy: cell.animacy,
        },
        inflector,
    )?;
    let part = fraction_noun_form(cell, inflector)?;
    let mut phrases = Vec::new();
    for analysis in ordinal.analyses() {
        let mut tokens = tag_tokens(
            &analysis.tokens,
            "SYN-NUMERAL-FRACTION-ORDINAL-PART-ALYPY-70",
            "Alypy (Gamanovich), §70 ordinal + inflected часть fractions",
        )?;
        tokens.push(PhraseToken {
            role: PhraseRole::FractionNoun,
            forms: tag_form_set(
                &part,
                "SYN-NUMERAL-FRACTION-ORDINAL-PART-ALYPY-70",
                "Alypy (Gamanovich), §70 ordinal + inflected часть fractions",
            )?,
        });
        phrases.push(RealizedPhrase::new(
            AnalyticConstruction::FractionalPart,
            tokens,
        )?);
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

/// Realizes the directly Synodal fractional adjective `полдесѧтый` with
/// inflected `часть`. III Esdras 14:11–12 directly supplies the feminine
/// genitive singular; the remaining agreement cells are transparent
/// applications of the ordinary hard-adjective paradigm.
pub fn fractional_half_tenth_parts(cell: NounCell) -> Result<RealizedPhrase> {
    fractional_half_tenth_parts_with(cell, Inflector::default())
}

pub fn fractional_half_tenth_parts_with(
    cell: NounCell,
    inflector: Inflector,
) -> Result<RealizedPhrase> {
    let fractional = inflector.form_by_id(
        &LexemeId::from("synodal:numeral:fractional-poludesyatyi"),
        GrammarCell::Numeral(NumeralCell {
            kind: NumeralKind::Fractional,
            case: cell.case,
            number: cell.number,
            gender: Some(Gender::Feminine),
            animacy: cell.animacy,
        }),
    )?;
    let part = fraction_noun_form(cell, inflector)?;
    RealizedPhrase::new(
        AnalyticConstruction::FractionalPart,
        vec![
            numeral_token(fractional),
            PhraseToken {
                role: PhraseRole::FractionNoun,
                forms: part,
            },
        ],
    )
}

/// Realizes a rational expression whose numerator governs an ordinally
/// qualified `часть`. For example, two fifth parts use dual agreement, while
/// five fifth parts use the source-licensed genitive-plural construction.
pub fn fraction(
    numerator: u32,
    denominator: u16,
    case: Case,
    animacy: Animacy,
) -> Result<Vec<RealizedPhrase>> {
    fraction_with(numerator, denominator, case, animacy, Inflector::default())
}

pub fn fraction_with(
    numerator: u32,
    denominator: u16,
    case: Case,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let cardinal = cardinal_with(
        numerator,
        CompoundNumeralCell {
            case,
            gender: cardinal_requires_gender(numerator).then_some(Gender::Feminine),
            animacy,
        },
        inflector,
    )?;
    let noun_cells = governed_fraction_noun_cells(&cardinal, animacy);
    let mut phrases = Vec::new();
    for noun_cell in noun_cells {
        let ordinals = ordinal_with(
            denominator,
            NumeralCell {
                kind: NumeralKind::Ordinal,
                case: noun_cell.case,
                number: noun_cell.number,
                gender: Some(Gender::Feminine),
                animacy: noun_cell.animacy,
            },
            inflector,
        )?;
        let part = fraction_noun_form(noun_cell, inflector)?;
        for cardinal_analysis in cardinal.analyses() {
            for ordinal_analysis in ordinals.analyses() {
                let mut tokens = tag_tokens(
                    &cardinal_analysis.tokens,
                    "SYN-NUMERAL-FRACTION-CARDINAL-ORDINAL-PART-ALYPY-70",
                    "Alypy (Gamanovich), §70 cardinal/ordinal + часть fractions",
                )?;
                tokens.extend(tag_tokens(
                    &ordinal_analysis.tokens,
                    "SYN-NUMERAL-FRACTION-CARDINAL-ORDINAL-PART-ALYPY-70",
                    "Alypy (Gamanovich), §70 cardinal/ordinal + часть fractions",
                )?);
                tokens.push(PhraseToken {
                    role: PhraseRole::FractionNoun,
                    forms: tag_form_set(
                        &part,
                        "SYN-NUMERAL-FRACTION-CARDINAL-ORDINAL-PART-ALYPY-70",
                        "Alypy (Gamanovich), §70 cardinal/ordinal + часть fractions",
                    )?,
                });
                phrases.push(RealizedPhrase::new(
                    AnalyticConstruction::FractionalPart,
                    tokens,
                )?);
            }
        }
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

fn fractional_cardinal_phrases(
    cardinal: &RealizedCardinal,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<Vec<RealizedPhrase>> {
    let noun_cells = governed_fraction_noun_cells(cardinal, animacy);
    let mut phrases = Vec::new();
    for noun_cell in noun_cells {
        let part = fraction_noun_form(noun_cell, inflector)?;
        for analysis in cardinal.analyses() {
            let mut tokens = tag_tokens(
                &analysis.tokens,
                "SYN-NUMERAL-FRACTION-CARDINAL-PART-ALYPY-70",
                "Alypy (Gamanovich), §70 cardinal + inflected часть fractions",
            )?;
            tokens.push(PhraseToken {
                role: PhraseRole::FractionNoun,
                forms: tag_form_set(
                    &part,
                    "SYN-NUMERAL-FRACTION-CARDINAL-PART-ALYPY-70",
                    "Alypy (Gamanovich), §70 cardinal + inflected часть fractions",
                )?,
            });
            phrases.push(RealizedPhrase::new(
                AnalyticConstruction::FractionalPart,
                tokens,
            )?);
        }
    }
    deduplicate_phrases(&mut phrases);
    Ok(phrases)
}

fn governed_fraction_noun_cells(cardinal: &RealizedCardinal, animacy: Animacy) -> Vec<NounCell> {
    let mut cells = Vec::new();
    for government in cardinal.government(NumeralNounPosition::Following) {
        let (case, number) = match government {
            NumeralGovernment::Agreement { number } => (cardinal.cell().case, *number),
            NumeralGovernment::GenitivePlural => (Case::Genitive, Number::Plural),
            NumeralGovernment::ContextualNominativePlural => (Case::Nominative, Number::Plural),
        };
        let cell = NounCell {
            case,
            number,
            animacy,
        };
        if !cells.contains(&cell) {
            cells.push(cell);
        }
    }
    cells
}

fn fraction_noun_form(cell: NounCell, inflector: Inflector) -> Result<FormSet> {
    inflector.form_by_id(
        &LexemeId::from("synodal:noun:v07-6ef4c1b12b34ac8c"),
        GrammarCell::Noun(cell),
    )
}

fn compose_cardinal(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if value <= 999 {
        return sub_thousand_cardinal(value as u16, cell, inflector);
    }

    let mut ordinary = if value == MAX_CARDINAL_VALUE {
        magnitude_chunk(1_000, Magnitude::Thousand, cell, inflector)?
    } else {
        let thousands = value / 1_000;
        let remainder = (value % 1_000) as u16;
        let mut chunks = vec![magnitude_chunk(
            thousands,
            Magnitude::Thousand,
            cell,
            inflector,
        )?];
        if remainder != 0 {
            chunks.push(sub_thousand_cardinal(remainder, cell, inflector)?);
        }
        combine_chunks(chunks, inflector.orthography())?
    };

    if value < MAX_CARDINAL_VALUE {
        ordinary.extend(distributed_thousands_cardinal(value, cell, inflector)?);
    }

    let named = named_magnitude_cardinal(value, cell, inflector)?;
    if let Some(magnitude) = exact_magnitude(value) {
        let exact = single_cardinal_analysis(
            NumeralComposition::Magnitude,
            magnitude_form(magnitude, cell.case, Number::Singular, inflector)?,
        );
        ordinary.retain(|analysis| analysis.primary_text() != exact.primary_text());
        ordinary.insert(0, exact);
    } else {
        ordinary.extend(named);
    }
    deduplicate_analyses(&mut ordinary);
    Ok(ordinary)
}

/// Synodal biblical usage can repeat `тысѧща` after the hundreds and lower
/// parts of a multiplier: 603,000 is printed as `ше́сть сѡ́тъ ты́сѧщъ и҆ трѝ
/// ты́сѧщы`. Keep that analysis correlated rather than flattening it into a
/// list of unrelated word variants.
fn distributed_thousands_cardinal(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    let thousands = value / 1_000;
    if thousands < 101 {
        return Ok(Vec::new());
    }
    let high = (thousands / 100) * 100;
    let low = thousands % 100;
    if low == 0 {
        return Ok(Vec::new());
    }
    let mut chunks = vec![magnitude_chunk(high, Magnitude::Thousand, cell, inflector)?];
    chunks.push(magnitude_chunk(low, Magnitude::Thousand, cell, inflector)?);
    let remainder = (value % 1_000) as u16;
    if remainder != 0 {
        chunks.push(sub_thousand_cardinal(remainder, cell, inflector)?);
    }
    combine_chunks(chunks, inflector.orthography())
}

fn sub_thousand_cardinal(
    value: u16,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if value <= 99 {
        return lower_cardinal(value as u8, cell, inflector);
    }
    if value == 100 {
        return Ok(vec![single_cardinal_analysis(
            NumeralComposition::Magnitude,
            magnitude_form(Magnitude::Hundred, cell.case, Number::Singular, inflector)?,
        )]);
    }

    let mut chunks = Vec::new();
    let hundreds = u32::from(value / 100);
    let remainder = (value % 100) as u8;
    if hundreds != 0 {
        chunks.push(magnitude_chunk(
            hundreds,
            Magnitude::Hundred,
            cell,
            inflector,
        )?);
    }
    if remainder != 0 {
        chunks.push(lower_cardinal(remainder, cell, inflector)?);
    }
    combine_chunks(chunks, inflector.orthography())
}

/// Builds the source-listed named-magnitude analysis alongside ordinary
/// decimal thousands. Thus 54,000 is primarily `пѧтьдесѧтъ и четыре тысѧщы`,
/// but the semantically equivalent `пѧть темъ и четыре тысѧщы` remains
/// available under Alypy's тьма = 10,000 inventory.
fn named_magnitude_cardinal(
    value: u32,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if value < 10_000 || value == MAX_CARDINAL_VALUE {
        return Ok(Vec::new());
    }
    let mut chunks = Vec::new();
    let places = [
        (100_000, Magnitude::Legion),
        (10_000, Magnitude::Myriad),
        (1_000, Magnitude::Thousand),
        (100, Magnitude::Hundred),
    ];
    let mut remainder = value;
    for (place, magnitude) in places {
        let digit = remainder / place;
        if digit != 0 {
            chunks.push(magnitude_chunk(digit, magnitude, cell, inflector)?);
            remainder %= place;
        }
    }
    if remainder != 0 {
        chunks.push(lower_cardinal(remainder as u8, cell, inflector)?);
    }
    combine_chunks(chunks, inflector.orthography())
}

fn lower_cardinal(
    value: u8,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    match value {
        1..=9 => Ok(vec![single_cardinal_analysis(
            NumeralComposition::Simple,
            digit_form(value, cell.case, cell.gender, cell.animacy, inflector)?,
        )]),
        10 => Ok(vec![single_cardinal_analysis(
            NumeralComposition::Simple,
            ten_form(cell.case, Number::Singular, cell.animacy, inflector)?,
        )]),
        11..=19 => teen_analyses(value - 10, cell, inflector),
        20..=99 => {
            let tens = value / 10;
            let unit = value % 10;
            let analyses = tens_analyses(tens, cell.case, cell.animacy, inflector)?;
            if unit == 0 {
                return Ok(analyses);
            }
            let unit = numeral_token(digit_form(
                unit,
                cell.case,
                cell.gender,
                cell.animacy,
                inflector,
            )?);
            let connector = PhraseToken {
                role: PhraseRole::Conjunction,
                forms: grammar_form(
                    "и",
                    Some("и҆"),
                    "SYN-NUMERAL-CARDINAL-ADDITIVE-ALYPY-63",
                    "Alypy (Gamanovich), §63 multi-component cardinals",
                    inflector.orthography(),
                )?,
            };
            let mut combined = Vec::with_capacity(analyses.len() * 2);
            for analysis in analyses {
                let mut with_i = analysis.tokens.clone();
                with_i.push(connector.clone());
                with_i.push(unit.clone());
                combined.push(CardinalPhraseAnalysis {
                    construction: NumeralComposition::AdditiveFinalConjunction,
                    tokens: with_i,
                });
                let mut asyndetic = analysis.tokens;
                asyndetic.push(unit.clone());
                combined.push(CardinalPhraseAnalysis {
                    construction: NumeralComposition::AdditiveAsyndetic,
                    tokens: asyndetic,
                });
            }
            Ok(combined)
        }
        _ => Err(Error::OutOfRange {
            value: u32::from(value),
            maximum: 99,
        }),
    }
}

fn teen_analyses(
    unit: u8,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    let profile = inflector.orthography();
    if profile == OrthographyProfile::SynodalLiturgical {
        return Err(Error::OrthographicMetadataRequired {
            field: MetadataField::AccentParadigm,
        });
    }
    let prefix_gender = if unit <= 4 { cell.gender } else { None };
    let declined_unit = digit_form(unit, cell.case, prefix_gender, cell.animacy, inflector)?;
    let citation_unit = digit_form(
        unit,
        Case::Nominative,
        if unit <= 4 {
            Some(if unit == 1 {
                Gender::Neuter
            } else {
                Gender::Masculine
            })
        } else {
            None
        },
        Animacy::Inanimate,
        inflector,
    )?;
    let fixed_ten = fixed_ten_accusative(profile)?;
    let singular_ten = ten_form(cell.case, Number::Singular, cell.animacy, inflector)?;
    let plural_ten = ten_form(cell.case, Number::Plural, cell.animacy, inflector)?;
    let na = grammar_form(
        "на",
        Some("на́"),
        "SYN-NUMERAL-CARDINAL-TEEN-ALYPY-63-64",
        "Alypy (Gamanovich), §§63–64 teen formation and inflection",
        profile,
    )?;
    let mut analyses = Vec::new();
    push_fused_teen(
        &mut analyses,
        NumeralComposition::TeenFirstComponentDeclined,
        &declined_unit,
        &na,
        &fixed_ten,
        profile,
    )?;
    push_fused_teen(
        &mut analyses,
        NumeralComposition::TeenSecondComponentDeclined,
        &citation_unit,
        &na,
        &singular_ten,
        profile,
    )?;
    push_fused_teen(
        &mut analyses,
        NumeralComposition::TeenSecondComponentDeclined,
        &citation_unit,
        &na,
        &plural_ten,
        profile,
    )?;
    push_fused_teen(
        &mut analyses,
        NumeralComposition::TeenBothComponentsDeclined,
        &declined_unit,
        &na,
        &singular_ten,
        profile,
    )?;
    if unit == 2 {
        let dual_ten = ten_form(cell.case, Number::Dual, cell.animacy, inflector)?;
        push_fused_teen(
            &mut analyses,
            NumeralComposition::TeenBothComponentsDual,
            &declined_unit,
            &na,
            &dual_ten,
            profile,
        )?;
    }
    deduplicate_analyses(&mut analyses);
    Ok(analyses)
}

fn push_fused_teen(
    analyses: &mut Vec<CardinalPhraseAnalysis>,
    construction: NumeralComposition,
    unit: &FormSet,
    na: &FormSet,
    ten: &FormSet,
    profile: OrthographyProfile,
) -> Result<()> {
    let fused = fuse_form_sets(
        &[unit, na, ten],
        "SYN-NUMERAL-CARDINAL-TEEN-ALYPY-63-64",
        "Alypy (Gamanovich), §§63–64 teen formation and inflection",
        profile,
    )?;
    analyses.push(single_cardinal_analysis(construction, fused));
    if construction != NumeralComposition::TeenSecondComponentDeclined {
        analyses.push(CardinalPhraseAnalysis {
            construction,
            tokens: vec![
                numeral_token(unit.clone()),
                PhraseToken {
                    role: PhraseRole::Preposition,
                    forms: na.clone(),
                },
                numeral_token(ten.clone()),
            ],
        });
    }
    Ok(())
}

fn tens_analyses(
    multiplier: u8,
    case: Case,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    let profile = inflector.orthography();
    if profile == OrthographyProfile::SynodalLiturgical {
        return Err(Error::OrthographicMetadataRequired {
            field: MetadataField::AccentParadigm,
        });
    }
    let citation = digit_form(
        multiplier,
        Case::Nominative,
        if multiplier <= 4 {
            Some(Gender::Masculine)
        } else {
            None
        },
        Animacy::Inanimate,
        inflector,
    )?;
    let mut results = Vec::new();
    if multiplier <= 4 {
        for number in [Number::Singular, Number::Plural] {
            let ten = ten_form(case, number, animacy, inflector)?;
            let fused = fuse_form_sets(
                &[&citation, &ten],
                "SYN-NUMERAL-CARDINAL-TENS-AGREEMENT-ALYPY-63-64",
                "Alypy (Gamanovich), §§63–64 twenty through forty",
                profile,
            )?;
            results.push(single_cardinal_analysis(
                NumeralComposition::TensAgreement,
                fused,
            ));
        }
    } else {
        let declined = digit_form(multiplier, case, None, animacy, inflector)?;
        let governed_ten = fixed_genitive_plural_ten(profile)?;
        results.push(single_cardinal_analysis(
            NumeralComposition::TensGovernment,
            fuse_form_sets(
                &[&declined, &governed_ten],
                "SYN-NUMERAL-CARDINAL-TENS-GOVERNMENT-ALYPY-63-64",
                "Alypy (Gamanovich), §§63–64 fifty through ninety",
                profile,
            )?,
        ));
        for (number, construction) in [
            (
                Number::Singular,
                NumeralComposition::TensBothComponentsSingular,
            ),
            (Number::Plural, NumeralComposition::TensBothComponentsPlural),
        ] {
            let ten = ten_form(case, number, animacy, inflector)?;
            results.push(single_cardinal_analysis(
                construction,
                fuse_form_sets(
                    &[&declined, &ten],
                    "SYN-NUMERAL-CARDINAL-TENS-BOTH-ALYPY-64",
                    "Alypy (Gamanovich), §64 both-component alternative for fifty through ninety",
                    profile,
                )?,
            ));
        }
    }
    deduplicate_analyses(&mut results);
    Ok(results)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Magnitude {
    Hundred,
    Thousand,
    Myriad,
    Legion,
    Leodr,
}

impl Magnitude {
    const fn id(self) -> &'static str {
        match self {
            Self::Hundred => "synodal:numeral:v06-sto",
            Self::Thousand => "synodal:numeral:tysiascha",
            Self::Myriad => "synodal:numeral:tma",
            Self::Legion => "synodal:numeral:legeon",
            Self::Leodr => "synodal:numeral:leodr",
        }
    }

    const fn gender(self) -> Gender {
        match self {
            Self::Hundred => Gender::Neuter,
            Self::Thousand | Self::Myriad => Gender::Feminine,
            Self::Legion | Self::Leodr => Gender::Masculine,
        }
    }
}

fn exact_magnitude(value: u32) -> Option<Magnitude> {
    match value {
        100 => Some(Magnitude::Hundred),
        1_000 => Some(Magnitude::Thousand),
        10_000 => Some(Magnitude::Myriad),
        100_000 => Some(Magnitude::Legion),
        1_000_000 => Some(Magnitude::Leodr),
        _ => None,
    }
}

fn magnitude_chunk(
    multiplier: u32,
    magnitude: Magnitude,
    cell: CompoundNumeralCell,
    inflector: Inflector,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if multiplier == 1 {
        return Ok(vec![single_cardinal_analysis(
            NumeralComposition::Magnitude,
            magnitude_form(magnitude, cell.case, Number::Singular, inflector)?,
        )]);
    }
    if magnitude == Magnitude::Hundred && multiplier > 9 {
        return Err(Error::InvalidNumeral {
            reason: "a hundreds multiplier must be one through nine".into(),
        });
    }

    let multiplier_cell = CompoundNumeralCell {
        case: cell.case,
        gender: cardinal_requires_gender(multiplier).then_some(magnitude.gender()),
        animacy: cell.animacy,
    };
    let leading_analyses = if multiplier <= 9 {
        vec![single_cardinal_analysis(
            NumeralComposition::Simple,
            digit_form(
                multiplier as u8,
                multiplier_cell.case,
                multiplier_cell.gender,
                multiplier_cell.animacy,
                inflector,
            )?,
        )]
    } else {
        cardinal_with(multiplier, multiplier_cell, inflector)?.analyses
    };
    let magnitude_cells = following_government(multiplier, cell.case);
    let mut results = Vec::new();
    let (composition_rule, composition_citation) = if magnitude == Magnitude::Hundred {
        (
            "SYN-NUMERAL-CARDINAL-HUNDREDS-ALYPY-63-64",
            "Alypy (Gamanovich), §§63–64 hundreds formation and spelling",
        )
    } else {
        (
            "SYN-NUMERAL-CARDINAL-MAGNITUDE-COMPOSITION-ALYPY-63",
            "Alypy (Gamanovich), §63 separate magnitude composition by agreement or government",
        )
    };
    for leading in leading_analyses {
        let leading_tokens = tag_tokens(&leading.tokens, composition_rule, composition_citation)?;
        for government in &magnitude_cells {
            let (case, number, construction) = match government {
                NumeralGovernment::Agreement { number } => (
                    cell.case,
                    *number,
                    if magnitude == Magnitude::Hundred {
                        NumeralComposition::HundredsAgreement
                    } else {
                        NumeralComposition::MagnitudeAgreement
                    },
                ),
                NumeralGovernment::GenitivePlural => (
                    Case::Genitive,
                    Number::Plural,
                    if magnitude == Magnitude::Hundred {
                        NumeralComposition::HundredsGovernment
                    } else {
                        NumeralComposition::MagnitudeGovernment
                    },
                ),
                NumeralGovernment::ContextualNominativePlural => (
                    Case::Nominative,
                    Number::Plural,
                    if magnitude == Magnitude::Hundred {
                        NumeralComposition::HundredsAgreement
                    } else {
                        NumeralComposition::MagnitudeAgreement
                    },
                ),
            };
            let magnitude_forms = tag_form_set(
                &magnitude_form(magnitude, case, number, inflector)?,
                composition_rule,
                composition_citation,
            )?;
            let mut tokens = leading_tokens.clone();
            tokens.push(numeral_token(magnitude_forms.clone()));
            let spaced = CardinalPhraseAnalysis {
                construction,
                tokens,
            };
            if magnitude != Magnitude::Hundred || leading_tokens.len() != 1 {
                results.push(spaced);
                continue;
            }
            let fused = single_cardinal_analysis(
                construction,
                fuse_form_sets(
                    &[&leading_tokens[0].forms, &magnitude_forms],
                    "SYN-NUMERAL-CARDINAL-HUNDREDS-ALYPY-63-64",
                    "Alypy (Gamanovich), §§63–64 hundreds formation and spelling",
                    inflector.orthography(),
                )?,
            );
            if cell.case == Case::Nominative && multiplier <= 4 {
                results.extend([fused, spaced]);
            } else {
                results.extend([spaced, fused]);
            }
        }
    }
    deduplicate_analyses(&mut results);
    Ok(results)
}

fn combine_chunks(
    mut chunks: Vec<Vec<CardinalPhraseAnalysis>>,
    profile: OrthographyProfile,
) -> Result<Vec<CardinalPhraseAnalysis>> {
    if chunks.len() == 1 {
        return Ok(chunks.pop().unwrap_or_default());
    }
    let mut products = vec![Vec::<CardinalPhraseAnalysis>::new()];
    for chunk in chunks {
        let mut next = Vec::new();
        for prefix in &products {
            for suffix in &chunk {
                let mut combined = prefix.clone();
                combined.push(suffix.clone());
                next.push(combined);
            }
        }
        products = next;
    }

    let conjunction = PhraseToken {
        role: PhraseRole::Conjunction,
        forms: grammar_form(
            "и",
            Some("и҆"),
            "SYN-NUMERAL-CARDINAL-ADDITIVE-ALYPY-63",
            "Alypy (Gamanovich), §63 multi-component cardinal connectors",
            profile,
        )?,
    };
    let mut results = Vec::new();
    for product in products {
        for mode in [
            NumeralComposition::AdditiveFinalConjunction,
            NumeralComposition::AdditiveAllConjunctions,
            NumeralComposition::AdditiveAsyndetic,
        ] {
            let mut tokens = Vec::new();
            for (index, chunk) in product.iter().enumerate() {
                if index != 0
                    && (mode == NumeralComposition::AdditiveAllConjunctions
                        || (mode == NumeralComposition::AdditiveFinalConjunction
                            && index + 1 == product.len()))
                {
                    tokens.push(conjunction.clone());
                }
                tokens.extend(chunk.tokens.clone());
            }
            results.push(CardinalPhraseAnalysis {
                construction: mode,
                tokens,
            });
        }
    }
    deduplicate_analyses(&mut results);
    Ok(results)
}

fn compose_ordinal(
    value: u16,
    cell: NumeralCell,
    inflector: Inflector,
) -> Result<Vec<OrdinalPhraseAnalysis>> {
    if value <= 10 {
        return Ok(vec![OrdinalPhraseAnalysis {
            construction: NumeralComposition::Simple,
            tokens: vec![numeral_token(simple_ordinal_form(
                value as u8,
                cell,
                inflector,
            )?)],
        }]);
    }
    if (11..=19).contains(&value) {
        let unit = (value - 10) as u8;
        let ordinal_unit = simple_ordinal_form(unit, cell, inflector)?;
        let tail = grammar_form(
            "надесѧть",
            None,
            "SYN-NUMERAL-ORDINAL-TEEN-ALYPY-68",
            "Alypy (Gamanovich), §68 and appendix, ordinal teens",
            inflector.orthography(),
        )?;
        let analytic = fuse_form_sets(
            &[&ordinal_unit, &tail],
            "SYN-NUMERAL-ORDINAL-TEEN-ALYPY-68",
            "Alypy (Gamanovich), §68 ordinal ending on the first teen component",
            inflector.orthography(),
        )?;
        let (lemma, stem) = ordinal_head(value)?;
        return Ok(vec![
            OrdinalPhraseAnalysis {
                construction: NumeralComposition::CompoundOrdinalAnalyticTeen,
                tokens: vec![numeral_token(analytic)],
            },
            OrdinalPhraseAnalysis {
                construction: NumeralComposition::CompoundOrdinalFused,
                tokens: vec![numeral_token(dynamic_ordinal_form(
                    lemma, stem, cell, inflector,
                )?)],
            },
        ]);
    }
    if let Ok((lemma, stem)) = ordinal_head(value) {
        return Ok(vec![OrdinalPhraseAnalysis {
            construction: NumeralComposition::CompoundOrdinalFused,
            tokens: vec![numeral_token(dynamic_ordinal_form(
                lemma, stem, cell, inflector,
            )?)],
        }]);
    }

    let (prefix, final_value) = ordinal_prefix_and_head(value)?;
    let final_forms = if final_value <= 10 {
        simple_ordinal_form(final_value as u8, cell, inflector)?
    } else {
        let (lemma, stem) = ordinal_head(final_value)?;
        dynamic_ordinal_form(lemma, stem, cell, inflector)?
    };
    let final_token = numeral_token(final_forms);
    let prefix_cell = CompoundNumeralCell {
        case: Case::Nominative,
        gender: None,
        animacy: Animacy::Inanimate,
    };
    let cardinal_prefix = cardinal_with(u32::from(prefix), prefix_cell, inflector)?;
    let conjunction = PhraseToken {
        role: PhraseRole::Conjunction,
        forms: grammar_form(
            "и",
            Some("и҆"),
            "SYN-NUMERAL-ORDINAL-COMPOUND-ALYPY-68",
            "Alypy (Gamanovich), §68 multi-component ordinals",
            inflector.orthography(),
        )?,
    };
    let mut analyses = Vec::new();
    for prefix_analysis in cardinal_prefix.analyses() {
        let mut asyndetic = prefix_analysis.tokens.clone();
        asyndetic.push(final_token.clone());
        analyses.push(OrdinalPhraseAnalysis {
            construction: NumeralComposition::CompoundOrdinalAsyndetic,
            tokens: asyndetic,
        });
        let mut connected = prefix_analysis.tokens.clone();
        connected.push(conjunction.clone());
        connected.push(final_token.clone());
        analyses.push(OrdinalPhraseAnalysis {
            construction: NumeralComposition::CompoundOrdinalConjunction,
            tokens: connected,
        });
    }
    deduplicate_ordinal_analyses(&mut analyses);
    Ok(analyses)
}

fn ordinal_prefix_and_head(value: u16) -> Result<(u16, u16)> {
    let final_value = if value % 10 != 0 {
        value % 10
    } else if value % 100 != 0 {
        value % 100
    } else if value % 1_000 != 0 {
        value % 1_000
    } else {
        value
    };
    let prefix = value - final_value;
    if prefix == 0 || (final_value > 10 && ordinal_head(final_value).is_err()) {
        return Err(Error::UnsupportedFormation {
            formation: format!("compound ordinal {value}"),
        });
    }
    Ok((prefix, final_value))
}

fn ordinal_head(value: u16) -> Result<(&'static str, &'static str)> {
    let head = match value {
        11 => ("єдинонадесѧтый", "єдинонадесѧт"),
        12 => ("дванадесѧтый", "дванадесѧт"),
        13 => ("тринадесѧтый", "тринадесѧт"),
        14 => ("четыренадесѧтый", "четыренадесѧт"),
        15 => ("пѧтьнадесѧтый", "пѧтьнадесѧт"),
        16 => ("шестьнадесѧтый", "шестьнадесѧт"),
        17 => ("седмьнадесѧтый", "седмьнадесѧт"),
        18 => ("осмьнадесѧтый", "осмьнадесѧт"),
        19 => ("девѧтьнадесѧтый", "девѧтьнадесѧт"),
        20 => ("двадесѧтый", "двадесѧт"),
        30 => ("тридесѧтый", "тридесѧт"),
        40 => ("четыредесѧтый", "четыредесѧт"),
        50 => ("пѧтьдесѧтый", "пѧтьдесѧт"),
        60 => ("шестьдесѧтый", "шестьдесѧт"),
        70 => ("седмьдесѧтый", "седмьдесѧт"),
        80 => ("осмьдесѧтый", "осмьдесѧт"),
        90 => ("девѧтьдесѧтый", "девѧтьдесѧт"),
        100 => ("сотный", "сотн"),
        200 => ("двосотный", "двосотн"),
        300 => ("трисотный", "трисотн"),
        400 => ("четвертосотный", "четвертосотн"),
        500 => ("пѧтьсотный", "пѧтьсотн"),
        600 => ("шестьсотный", "шестьсотн"),
        700 => ("седмьсотный", "седмьсотн"),
        800 => ("осмьсотный", "осмьсотн"),
        900 => ("девѧтьсотный", "девѧтьсотн"),
        1_000 => ("тысѧщный", "тысѧщн"),
        _ => {
            return Err(Error::UnsupportedFormation {
                formation: format!("ordinal head {value}"),
            });
        }
    };
    Ok(head)
}

fn simple_ordinal_form(value: u8, cell: NumeralCell, inflector: Inflector) -> Result<FormSet> {
    let id = match value {
        1 => "synodal:numeral:pervyi",
        2 => "synodal:numeral:vtoryi",
        3 => "synodal:numeral:tretii",
        4 => "synodal:numeral:chetvertyi",
        5 => "synodal:numeral:pyatyi",
        6 => "synodal:numeral:shestyi",
        7 => "synodal:numeral:sedmyi",
        8 => "synodal:numeral:osmyi",
        9 => "synodal:numeral:devyatyi",
        10 => "synodal:numeral:desyatyi",
        _ => {
            return Err(Error::OutOfRange {
                value: u32::from(value),
                maximum: 10,
            });
        }
    };
    inflector.form_by_id(&LexemeId::from(id), GrammarCell::Numeral(cell))
}

fn dynamic_ordinal_form(
    lemma: &str,
    stem: &str,
    cell: NumeralCell,
    inflector: Inflector,
) -> Result<FormSet> {
    let lexeme = NumeralLexeme::new(
        SynodalWord::parse(lemma)?,
        SynodalWord::parse(stem)?,
        NumeralDeclension::OrdinalHard,
    );
    decline_numeral(&lexeme, cell, inflector.orthography())
}

fn digit_form(
    digit: u8,
    case: Case,
    gender: Option<Gender>,
    animacy: Animacy,
    inflector: Inflector,
) -> Result<FormSet> {
    let (id, number) = match digit {
        1 => ("synodal:numeral:edin", Number::Singular),
        2 => ("synodal:numeral:dva", Number::Dual),
        3 => ("synodal:numeral:tri", Number::Plural),
        4 => ("synodal:numeral:chetyre", Number::Plural),
        5 => ("synodal:numeral:wikt-42c5d78bab14", Number::Singular),
        6 => ("synodal:numeral:wikt-58a4f8eb4197", Number::Singular),
        7 => ("synodal:numeral:wikt-2fe80b81eaf8", Number::Singular),
        8 => ("synodal:numeral:v06-7391e80a474691c3", Number::Singular),
        9 => ("synodal:numeral:wikt-04f311cf0bd0", Number::Singular),
        _ => {
            return Err(Error::InvalidNumeral {
                reason: "a cardinal digit component must be one through nine".into(),
            });
        }
    };
    inflector.form_by_id(
        &LexemeId::from(id),
        GrammarCell::Numeral(NumeralCell {
            kind: NumeralKind::Cardinal,
            case,
            number,
            gender,
            animacy,
        }),
    )
}

fn ten_form(case: Case, number: Number, animacy: Animacy, inflector: Inflector) -> Result<FormSet> {
    inflector.form_by_id(
        &LexemeId::from("synodal:numeral:wikt-bc270882d39d"),
        GrammarCell::Numeral(NumeralCell {
            kind: NumeralKind::Cardinal,
            case,
            number,
            gender: None,
            animacy,
        }),
    )
}

fn magnitude_form(
    magnitude: Magnitude,
    case: Case,
    number: Number,
    inflector: Inflector,
) -> Result<FormSet> {
    inflector.form_by_id(
        &LexemeId::from(magnitude.id()),
        GrammarCell::Numeral(NumeralCell {
            kind: NumeralKind::Cardinal,
            case,
            number,
            gender: None,
            animacy: Animacy::Inanimate,
        }),
    )
}

fn fixed_ten_accusative(profile: OrthographyProfile) -> Result<FormSet> {
    grammar_forms(
        &["десѧть", "десѧте"],
        "SYN-NUMERAL-CARDINAL-TEEN-ALYPY-63-64",
        "Alypy (Gamanovich), §§63–64 invariant accusative ten in teens",
        profile,
    )
}

fn fixed_genitive_plural_ten(profile: OrthographyProfile) -> Result<FormSet> {
    grammar_form(
        "десѧтъ",
        Some("десѧ́тъ"),
        "SYN-NUMERAL-CARDINAL-TENS-GOVERNMENT-ALYPY-63-64",
        "Alypy (Gamanovich), §§63–64 governed genitive-plural ten",
        profile,
    )
}

fn grammar_forms(
    forms: &[&str],
    rule: &'static str,
    citation: &'static str,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if profile == OrthographyProfile::SynodalLiturgical {
        return Err(Error::OrthographicMetadataRequired {
            field: MetadataField::AccentParadigm,
        });
    }
    let mut variants = Vec::new();
    for form in forms {
        variants.extend(
            grammar_form(form, None, rule, citation, profile)?
                .variants()
                .to_vec(),
        );
    }
    FormSet::try_from_variants(variants)
}

fn grammar_form(
    expanded: &str,
    accented: Option<&str>,
    rule: &'static str,
    citation: &'static str,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    let expanded = SynodalWord::parse(expanded)?.canonical().to_owned();
    let (accented, printed, warnings) = match profile {
        OrthographyProfile::Expanded => (None, expanded.clone(), Vec::new()),
        OrthographyProfile::ExpandedAccentless => {
            let form = normalize_lookup_accentless(&expanded);
            (
                None,
                form.clone(),
                vec!["accent and breathing marks removed".into()],
            )
        }
        OrthographyProfile::SynodalLiturgical => {
            let accented = accented.ok_or(Error::OrthographicMetadataRequired {
                field: MetadataField::AccentParadigm,
            })?;
            let accented = SynodalWord::parse(accented)?.canonical().to_owned();
            (Some(accented.clone()), accented, Vec::new())
        }
    };
    let rule_id = RuleId::from(rule);
    let evidence = numeral_evidence(rule, citation);
    let evidence_id = evidence.id.clone();
    FormSet::new(FormVariant {
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
        assumptions: Vec::new(),
        evidence: vec![evidence],
        contradictions: Vec::new(),
        warnings,
        rule_trace: RuleTrace::new(vec![TraceStep {
            rule: rule_id,
            stage: "numeral-composition-token".into(),
            input: expanded,
            output: printed,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: vec![evidence_id],
        }]),
    })
}

fn fuse_form_sets(
    parts: &[&FormSet],
    rule: &'static str,
    citation: &'static str,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    if profile == OrthographyProfile::SynodalLiturgical {
        return Err(Error::OrthographicMetadataRequired {
            field: MetadataField::AccentParadigm,
        });
    }
    let mut products = vec![Vec::<FormVariant>::new()];
    for forms in parts {
        let mut next = Vec::new();
        for prefix in &products {
            for variant in forms.variants() {
                let mut combination = prefix.clone();
                combination.push(variant.clone());
                next.push(combination);
            }
        }
        products = next;
    }

    let rule_id = RuleId::from(rule);
    let construction_evidence = numeral_evidence(rule, citation);
    let construction_evidence_id = construction_evidence.id.clone();
    let mut variants = Vec::new();
    for product in products {
        let expanded = product
            .iter()
            .map(|item| item.expanded.as_str())
            .collect::<String>();
        let printed = product
            .iter()
            .map(|item| item.printed.as_str())
            .collect::<String>();
        let mut evidence = Vec::new();
        let mut evidence_ids = Vec::new();
        let mut trace = Vec::new();
        let mut assumptions = Vec::new();
        let mut contradictions = Vec::new();
        let mut warnings = Vec::new();
        let mut confidence = Confidence::CERTAIN;
        for item in product {
            confidence = confidence.min(item.confidence);
            assumptions.extend(item.assumptions);
            contradictions.extend(item.contradictions);
            warnings.extend(item.warnings);
            trace.extend(item.rule_trace.steps().iter().cloned());
            for item_evidence in item.evidence {
                if !evidence
                    .iter()
                    .any(|known: &Evidence| known.id == item_evidence.id)
                {
                    evidence_ids.push(item_evidence.id.clone());
                    evidence.push(item_evidence);
                }
            }
        }
        if !evidence
            .iter()
            .any(|known| known.id == construction_evidence_id)
        {
            evidence_ids.push(construction_evidence_id.clone());
            evidence.push(construction_evidence.clone());
        }
        trace.push(TraceStep {
            rule: rule_id.clone(),
            stage: "fuse-numeral-components".into(),
            input: "component form sets".into(),
            output: printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence: evidence_ids,
        });
        variants.push(FormVariant {
            expanded,
            accented: None,
            printed,
            romanization: None,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            recension_mapping: None,
            confidence,
            source: FormSource::SynodalNormativeGeneration {
                rule: rule_id.clone(),
            },
            assumptions,
            evidence,
            contradictions,
            warnings,
            rule_trace: RuleTrace::new(trace),
        });
    }
    FormSet::try_from_variants(variants)
}

fn numeral_evidence(rule: &'static str, citation: &'static str) -> Evidence {
    Evidence {
        id: EvidenceId::from(format!("normative:{rule}")),
        source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
        source_recension: Recension::SynodalRussian,
        kind: EvidenceKind::NormativeRule,
        authority_roles: vec![AuthorityRole::Grammatical, AuthorityRole::Morphological],
        epistemic_role: EpistemicRole::SynodalNormativeAuthority,
        citation: citation.into(),
        note: Some(format!("stable rule {rule}")),
    }
}

fn tag_tokens(
    tokens: &[PhraseToken],
    rule: &'static str,
    citation: &'static str,
) -> Result<Vec<PhraseToken>> {
    tokens
        .iter()
        .map(|token| {
            Ok(PhraseToken {
                role: token.role,
                forms: tag_form_set(&token.forms, rule, citation)?,
            })
        })
        .collect()
}

fn tag_form_set(forms: &FormSet, rule: &'static str, citation: &'static str) -> Result<FormSet> {
    let rule_id = RuleId::from(rule);
    let construction_evidence = numeral_evidence(rule, citation);
    let construction_evidence_id = construction_evidence.id.clone();
    let mut variants = Vec::with_capacity(forms.variants().len());
    for source in forms.variants() {
        let mut variant = source.clone();
        if !variant
            .evidence
            .iter()
            .any(|known| known.id == construction_evidence_id)
        {
            variant.evidence.push(construction_evidence.clone());
        }
        let evidence = variant
            .evidence
            .iter()
            .map(|item| item.id.clone())
            .collect();
        variant.rule_trace.push(TraceStep {
            rule: rule_id.clone(),
            stage: "numeral-phrase-construction".into(),
            input: variant.expanded.clone(),
            output: variant.printed.clone(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: None,
            evidence,
        });
        variants.push(variant);
    }
    FormSet::try_from_variants(variants)
}

fn numeral_token(forms: FormSet) -> PhraseToken {
    PhraseToken {
        role: PhraseRole::Numeral,
        forms,
    }
}

fn single_cardinal_analysis(
    construction: NumeralComposition,
    forms: FormSet,
) -> CardinalPhraseAnalysis {
    CardinalPhraseAnalysis {
        construction,
        tokens: vec![numeral_token(forms)],
    }
}

fn render_tokens(tokens: &[PhraseToken]) -> String {
    tokens
        .iter()
        .map(|token| token.forms.primary_text())
        .collect::<Vec<_>>()
        .join(" ")
}

fn cardinal_requires_gender(value: u32) -> bool {
    let final_two = value % 100;
    let final_digit = value % 10;
    (1..=4).contains(&final_digit) && !(15..=19).contains(&final_two)
}

fn following_government(value: u32, case: Case) -> Vec<NumeralGovernment> {
    let final_two = value % 100;
    let final_digit = value % 10;
    let mut patterns = if (11..=14).contains(&final_two) {
        vec![
            NumeralGovernment::Agreement {
                number: inherent_number(final_digit),
            },
            NumeralGovernment::GenitivePlural,
        ]
    } else if (1..=4).contains(&final_digit) {
        vec![NumeralGovernment::Agreement {
            number: inherent_number(final_digit),
        }]
    } else if matches!(case, Case::Dative | Case::Instrumental | Case::Locative) {
        vec![
            NumeralGovernment::Agreement {
                number: Number::Plural,
            },
            NumeralGovernment::GenitivePlural,
        ]
    } else {
        vec![NumeralGovernment::GenitivePlural]
    };
    if case == Case::Nominative && value >= 5 {
        patterns.push(NumeralGovernment::ContextualNominativePlural);
    }
    patterns
}

fn preceding_government(value: u32, case: Case) -> Vec<NumeralGovernment> {
    let leading = match value {
        1..=10 => value,
        11..=19 => value - 10,
        20..=99 => value / 10,
        100..=999 => {
            let multiplier = value / 100;
            if multiplier == 1 { 100 } else { multiplier }
        }
        1_000..=999_999 => {
            let multiplier = value / 1_000;
            if multiplier == 1 {
                1_000
            } else {
                first_component_value(multiplier)
            }
        }
        1_000_000 => 1_000_000,
        _ => value,
    };
    following_government(leading, case)
}

fn first_component_value(value: u32) -> u32 {
    match value {
        1..=10 => value,
        11..=19 => value - 10,
        20..=99 => value / 10,
        100..=999 => {
            let multiplier = value / 100;
            if multiplier == 1 { 100 } else { multiplier }
        }
        _ => value,
    }
}

const fn inherent_number(digit: u32) -> Number {
    match digit {
        1 => Number::Singular,
        2 => Number::Dual,
        _ => Number::Plural,
    }
}

fn deduplicate_analyses(analyses: &mut Vec<CardinalPhraseAnalysis>) {
    let mut seen = BTreeSet::new();
    analyses.retain(|analysis| seen.insert((analysis.construction, analysis.primary_text())));
}

fn deduplicate_ordinal_analyses(analyses: &mut Vec<OrdinalPhraseAnalysis>) {
    let mut seen = BTreeSet::new();
    analyses.retain(|analysis| seen.insert((analysis.construction, analysis.primary_text())));
}

fn deduplicate_phrases(phrases: &mut Vec<RealizedPhrase>) {
    let mut seen = BTreeSet::new();
    phrases.retain(|phrase| seen.insert(phrase.primary_text()));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cardinal_cell(case: Case, gender: Option<Gender>) -> CompoundNumeralCell {
        CompoundNumeralCell {
            case,
            gender,
            animacy: Animacy::Inanimate,
        }
    }

    fn ordinal_cell(case: Case, gender: Gender) -> NumeralCell {
        NumeralCell {
            kind: NumeralKind::Ordinal,
            case,
            number: Number::Singular,
            gender: Some(gender),
            animacy: Animacy::Inanimate,
        }
    }

    fn realizes_token_surfaces(analysis: &CardinalPhraseAnalysis, expected: &[&str]) -> bool {
        analysis.tokens.len() == expected.len()
            && analysis
                .tokens
                .iter()
                .zip(expected)
                .all(|(token, expected)| {
                    token
                        .forms
                        .variants()
                        .iter()
                        .any(|variant| variant.expanded == *expected)
                })
    }

    #[test]
    fn cardinals_cover_simple_teens_tens_hundreds_and_all_named_magnitudes() {
        assert_eq!(
            cardinal(2, cardinal_cell(Case::Nominative, Some(Gender::Masculine)))
                .expect("masculine nominative two")
                .primary_text(),
            "два"
        );
        let twelve = cardinal(12, cardinal_cell(Case::Genitive, Some(Gender::Masculine)))
            .expect("genitive twelve");
        assert!(twelve.analyses().len() >= 5);
        assert!(
            twelve
                .analyses()
                .iter()
                .any(|analysis| analysis.primary_text().contains("на"))
        );

        let ninety_three = cardinal(93, cardinal_cell(Case::Genitive, Some(Gender::Masculine)))
            .expect("genitive ninety-three");
        assert!(
            ninety_three
                .analyses()
                .iter()
                .any(|analysis| analysis.primary_text().contains(" и "))
        );

        for (value, expected) in [
            (100, "сто"),
            (1_000, "тысѧща"),
            (10_000, "тьма"),
            (100_000, "легеѡнъ"),
            (1_000_000, "леѡдръ"),
        ] {
            assert_eq!(
                cardinal(value, cardinal_cell(Case::Nominative, None))
                    .expect("named magnitude")
                    .primary_text(),
                expected
            );
        }
    }

    #[test]
    fn cardinal_government_is_case_and_position_typed() {
        let five = cardinal(5, cardinal_cell(Case::Dative, None)).expect("dative five");
        assert_eq!(
            five.government(NumeralNounPosition::Following),
            [
                NumeralGovernment::Agreement {
                    number: Number::Plural
                },
                NumeralGovernment::GenitivePlural
            ]
        );
        assert_eq!(
            five.government_evidence()[0].id,
            EvidenceId::from("normative:SYN-NUMERAL-GOVERNMENT-ALYPY-65-67")
        );
        let twelve = cardinal(12, cardinal_cell(Case::Nominative, Some(Gender::Feminine)))
            .expect("nominative twelve");
        assert!(twelve.government(NumeralNounPosition::Following).contains(
            &NumeralGovernment::Agreement {
                number: Number::Dual
            }
        ));
        assert!(
            twelve
                .government(NumeralNounPosition::Following)
                .contains(&NumeralGovernment::GenitivePlural)
        );
        assert_eq!(
            twelve.government(NumeralNounPosition::Preceding),
            [NumeralGovernment::Agreement {
                number: Number::Dual
            }]
        );

        for value in [100, 1_000] {
            let magnitude =
                cardinal(value, cardinal_cell(Case::Nominative, None)).expect("exact magnitude");
            assert!(
                magnitude
                    .government(NumeralNounPosition::Preceding)
                    .contains(&NumeralGovernment::GenitivePlural)
            );
            assert!(
                !magnitude
                    .government(NumeralNounPosition::Preceding)
                    .contains(&NumeralGovernment::Agreement {
                        number: Number::Singular
                    })
            );
        }
    }

    #[test]
    fn locked_synodal_bible_compound_numerals_are_reproduced() {
        // 1 Chronicles 7:5, locked Wikisource revision 1355550, line 59.
        let fifty_four_thousand_four_hundred =
            cardinal(54_400, cardinal_cell(Case::Nominative, None)).expect("54,400");
        assert!(
            fifty_four_thousand_four_hundred
                .analyses()
                .iter()
                .any(|analysis| realizes_token_surfaces(
                    analysis,
                    &["пѧтьдесѧтъ", "и", "четыри", "тысѧщы", "и", "четыре", "ста"]
                ))
        );

        // 1 Chronicles 21:25, locked Wikisource revision 1355550, line 78.
        let six_hundred_three_thousand_five_hundred_fifty =
            cardinal(603_550, cardinal_cell(Case::Nominative, None)).expect("603,550");
        assert!(
            six_hundred_three_thousand_five_hundred_fifty
                .analyses()
                .iter()
                .any(|analysis| realizes_token_surfaces(
                    analysis,
                    &[
                        "шесть",
                        "сотъ",
                        "тысѧщъ",
                        "и",
                        "три",
                        "тысѧщы",
                        "и",
                        "пѧть",
                        "сотъ",
                        "и",
                        "пѧтьдесѧтъ"
                    ]
                ))
        );

        // 3 Kingdoms 14:20, locked Wikisource revision 1355056, line 619.
        let twenty_two = cardinal(22, cardinal_cell(Case::Nominative, Some(Gender::Neuter)))
            .expect("twenty-two");
        assert!(
            twenty_two
                .analyses()
                .iter()
                .any(|analysis| realizes_token_surfaces(analysis, &["двадесѧть", "два"]))
        );

        // 1 Chronicles 24:17–18, locked revision 1350049, lines 904–951.
        for (value, expected) in [(21, "двадесѧть первый"), (22, "двадесѧть вторый")]
        {
            let realized = ordinal(value, ordinal_cell(Case::Nominative, Gender::Masculine))
                .expect("compound ordinal");
            assert!(
                realized
                    .analyses()
                    .iter()
                    .any(|analysis| analysis.primary_text() == expected)
            );
        }
    }

    #[test]
    fn ordinals_cover_both_teen_placements_and_compounds_through_thousand() {
        let thirteenth = ordinal(13, ordinal_cell(Case::Nominative, Gender::Masculine))
            .expect("masculine thirteenth");
        assert_eq!(thirteenth.analyses().len(), 2);
        assert!(
            thirteenth
                .analyses()
                .iter()
                .any(|analysis| analysis.primary_text().starts_with("трет"))
        );
        assert!(
            thirteenth
                .analyses()
                .iter()
                .any(|analysis| analysis.primary_text().starts_with("тринадесѧт"))
        );

        let one_seventy_second = ordinal(172, ordinal_cell(Case::Accusative, Gender::Neuter))
            .expect("neuter accusative 172nd");
        assert!(
            one_seventy_second
                .analyses()
                .iter()
                .any(|analysis| analysis.primary_text().ends_with(" второе"))
        );
        assert!(ordinal(1_000, ordinal_cell(Case::Nominative, Gender::Masculine)).is_ok());
        assert!(matches!(
            ordinal(1_001, ordinal_cell(Case::Nominative, Gender::Masculine)),
            Err(Error::OutOfRange { .. })
        ));
    }

    #[test]
    fn structurally_invalid_gender_vocative_and_accent_guesses_fail_typed() {
        assert!(matches!(
            cardinal(21, cardinal_cell(Case::Nominative, None)),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
        assert!(matches!(
            cardinal(50, cardinal_cell(Case::Nominative, Some(Gender::Masculine))),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
        assert!(matches!(
            cardinal(12, cardinal_cell(Case::Vocative, Some(Gender::Masculine))),
            Err(Error::HistoricallyInvalidCell { .. })
        ));
        let liturgical = Inflector::builder()
            .orthography(OrthographyProfile::SynodalLiturgical)
            .build();
        assert!(matches!(
            cardinal_with(20, cardinal_cell(Case::Nominative, None), liturgical),
            Err(Error::OrthographicMetadataRequired { .. })
        ));
    }

    #[test]
    fn distributive_multiplicative_and_fractional_constructions_remain_typed_tokens() {
        let two = cardinal_cell(Case::Nominative, Some(Gender::Masculine));
        let repeated = repeated_distributive(2, two).expect("two by two");
        assert_eq!(repeated[0].primary_text(), "два два");
        assert_eq!(
            repeated[0].construction(),
            AnalyticConstruction::RepeatedDistributive
        );
        assert!(
            repeated[0]
                .tokens()
                .iter()
                .all(|token| token.role == PhraseRole::Numeral)
        );

        let seven_times =
            multiplicative_krat(7, cardinal_cell(Case::Genitive, None)).expect("seven times");
        assert_eq!(seven_times[0].primary_text(), "седми кратъ");
        assert_eq!(
            seven_times[0].tokens().last().map(|token| token.role),
            Some(PhraseRole::MultiplicativeUnit)
        );

        let two_parts =
            fractional_cardinal_parts(2, Case::Nominative, Animacy::Inanimate).expect("two parts");
        assert_eq!(two_parts[0].primary_text(), "двѣ части");
        assert_eq!(
            two_parts[0].tokens().last().map(|token| token.role),
            Some(PhraseRole::FractionNoun)
        );

        let tenth_part = fractional_ordinal_parts(
            10,
            NounCell {
                case: Case::Nominative,
                number: Number::Singular,
                animacy: Animacy::Inanimate,
            },
        )
        .expect("tenth part");
        assert_eq!(tenth_part[0].primary_text(), "десѧтаѧ часть");

        let half_tenth = fractional_half_tenth_parts(NounCell {
            case: Case::Genitive,
            number: Number::Singular,
            animacy: Animacy::Inanimate,
        })
        .expect("half-tenth part");
        assert_eq!(half_tenth.primary_text(), "полдесѧтыѧ части");
        assert!(matches!(
            half_tenth.tokens()[0].forms.primary().source,
            FormSource::SynodalAttestation { .. }
        ));
        let predicted_half_tenth = fractional_half_tenth_parts(NounCell {
            case: Case::Dative,
            number: Number::Dual,
            animacy: Animacy::Inanimate,
        })
        .expect("productive half-tenth agreement");
        assert!(matches!(
            predicted_half_tenth.tokens()[0].forms.primary().source,
            FormSource::SynodalNormativeGeneration { .. }
        ));

        let two_fifths =
            fraction(2, 5, Case::Nominative, Animacy::Inanimate).expect("two fifth parts");
        assert_eq!(two_fifths[0].primary_text(), "двѣ пѧтѣи части");
    }

    #[test]
    fn every_compositional_equivalence_class_covers_its_complete_cell_product() {
        let cardinal_values = [
            1, 2, 3, 4, 5, 10, 11, 12, 14, 15, 19, 20, 30, 40, 50, 90, 21, 55, 99, 100, 200, 400,
            500, 900, 101, 111, 114, 115, 120, 121, 999, 1_000, 2_000, 9_000, 10_000, 90_000,
            100_000, 900_000, 1_000_000,
        ];
        for value in cardinal_values {
            let genders: &[Option<Gender>] = if cardinal_requires_gender(value) {
                &[
                    Some(Gender::Masculine),
                    Some(Gender::Feminine),
                    Some(Gender::Neuter),
                ]
            } else {
                &[None]
            };
            for case in Case::ALL.into_iter().filter(|case| *case != Case::Vocative) {
                for &gender in genders {
                    for animacy in Animacy::ALL {
                        let realized = cardinal(
                            value,
                            CompoundNumeralCell {
                                case,
                                gender,
                                animacy,
                            },
                        )
                        .unwrap_or_else(|error| {
                            panic!("cardinal {value} {case:?} {gender:?} {animacy:?}: {error}")
                        });
                        assert!(!realized.analyses().is_empty());
                        assert!(realized.analyses().iter().all(|analysis| {
                            !analysis.tokens.is_empty()
                                && analysis
                                    .tokens
                                    .iter()
                                    .all(|token| !token.forms.variants().is_empty())
                        }));
                    }
                }
            }
        }

        let ordinal_values = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 30, 40, 50, 60,
            70, 80, 90, 21, 99, 100, 200, 300, 400, 500, 600, 700, 800, 900, 101, 110, 111, 172,
            999, 1_000,
        ];
        for value in ordinal_values {
            for number in Number::ALL {
                for case in Case::ALL {
                    for gender in Gender::ALL {
                        for animacy in Animacy::ALL {
                            let cell = NumeralCell {
                                kind: NumeralKind::Ordinal,
                                case,
                                number,
                                gender: Some(gender),
                                animacy,
                            };
                            let realized = ordinal(value, cell).unwrap_or_else(|error| {
                                panic!(
                                    "ordinal {value} {case:?} {number:?} {gender:?} {animacy:?}: {error}"
                                )
                            });
                            assert!(!realized.analyses().is_empty());
                            assert!(
                                realized
                                    .analyses()
                                    .iter()
                                    .all(|analysis| !analysis.tokens.is_empty())
                            );
                        }
                    }
                }
            }
        }
    }
}
