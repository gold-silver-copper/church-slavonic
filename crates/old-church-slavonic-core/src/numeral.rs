//! Source-reviewed Old Church Slavonic cardinal-numeral morphology.

use crate::noun::NounLexeme;
use crate::pronoun::StandardPronominalIdentity;
use crate::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, CollectiveNumeralCell,
    CompoundCardinalCell, DistributiveCardinalCell, FormAnalysis, FormSet, FormSource, FormVariant,
    Gender, InflectionError, InflectionWarning, MetadataEvidence, MetadataProvenance, NounCell,
    NounClass, Number, NumberRestriction, NumeralCell, PhraseRole, PhraseToken, PredictedForm,
    RequestedCell, RuleId, RuleStep,
};

/// Lowest integer owned by the structured compound-ordinal API.
///
/// Values one through ten belong to [`OrdinalNumeralIdentity`] instead.
pub const MIN_COMPOUND_ORDINAL_VALUE: u16 = 11;

/// Highest compound ordinal licensed by the declared Old Church Slavonic
/// source profile.
///
/// This is an evidential boundary, not an integer-storage limit. Gorshkov
/// §§118–119, Elkina §§125 and 129, Polivanova's exhaustive grammatical
/// dictionary, and Leuta–Havryliuk pp. 161–164 independently specify ordinal
/// heads through `тꙑсѧщьнъ` “thousandth” but do not determine the stem shape or
/// component inflection of higher magnitude ordinals. The engine therefore
/// rejects larger values instead of importing later Russian formations.
pub const MAX_COMPOUND_ORDINAL_VALUE: u16 = 1_000;

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
    /// An exact form cited from a named primary manuscript witness.
    PrimaryTextAttestation,
    /// A form generated from a historically established but unattested stem.
    ReconstructedRule,
}

impl NumeralVariantStatus {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ReviewedTable => "reviewed-table",
            Self::ProductiveRule => "productive-rule",
            Self::CorpusAttestation => "corpus-attestation",
            Self::PrimaryTextAttestation => "primary-text-attestation",
            Self::ReconstructedRule => "reconstructed-rule",
        }
    }
}

/// The source-bounded OCS indefinite-quantity numeral-noun inventory.
///
/// `несъвѣда` is not an exact integer synonym for `тъма`. Lvov's review of
/// the Suprasliensis and John the Exarch distinguishes exact `тъма` “ten
/// thousand” from `несъвѣда` “an incalculable quantity” and cites the hard
/// a-stem instrumental plural `несъвѣдами`. The lexeme therefore declines as
/// a noun but is deliberately unavailable to integer composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IndefiniteNumeralIdentity {
    Nesveda,
}

impl IndefiniteNumeralIdentity {
    pub const ALL: [Self; 1] = [Self::Nesveda];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::Nesveda => "несъвѣда",
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::Nesveda => &["несъвѣда"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    pub const fn noun_class(self) -> NounClass {
        match self {
            Self::Nesveda => NounClass::AHard,
        }
    }

    pub const fn gender(self) -> Gender {
        match self {
            Self::Nesveda => Gender::Feminine,
        }
    }

    pub const fn rule_id(self) -> RuleId {
        RuleId::NumeralIndefiniteNoun
    }

    pub const fn authority(self) -> &'static str {
        match self {
            Self::Nesveda => {
                "Lvov 1966 pp. 247–249, citing Codex Suprasliensis and John the Exarch"
            }
        }
    }

    fn noun_lexeme(self) -> NounLexeme {
        NounLexeme {
            lemma: self.canonical_lemma().to_string(),
            class: self.noun_class(),
            gender: self.gender(),
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        }
    }
}

/// Decline one source-listed indefinite-quantity numeral noun.
pub fn decline_indefinite(
    identity: IndefiniteNumeralIdentity,
    cell: NounCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    let mut prediction = crate::noun::decline(&identity.noun_lexeme(), cell)?;
    let rule_id = identity.rule_id();
    prediction.trace.push(RuleStep {
        rule_id,
        before: identity.canonical_lemma().to_string(),
        after: prediction.text.clone(),
        reason: "apply the source-identified indefinite numeral noun's inherited declension",
    });
    prediction.rule_id = rule_id;
    let status = match (cell.case, cell.number) {
        (Case::Nominative, Number::Singular) => NumeralVariantStatus::ReviewedTable,
        (Case::Instrumental, Number::Plural) => NumeralVariantStatus::PrimaryTextAttestation,
        _ => NumeralVariantStatus::ProductiveRule,
    };
    Ok(vec![NumeralVariant { prediction, status }])
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

    fn reconstructed(prediction: PredictedForm) -> Self {
        Self {
            prediction,
            status: NumeralVariantStatus::ReconstructedRule,
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

/// One correlated realization of distributive `по` and a dative cardinal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributiveCardinalAnalysis {
    pub tokens: Vec<PhraseToken>,
}

impl DistributiveCardinalAnalysis {
    /// Render each independently sourced component in token order.
    pub fn primary_text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.forms.primary_text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// An OCS distributive-cardinal construction.
///
/// This is a syntactic `по` phrase, not a synthetic numeral paradigm. The
/// preposition fixes the cardinal in the dative; the cell therefore carries
/// only the gender required by an agreeing final unit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizedDistributiveCardinal {
    value: u16,
    cell: DistributiveCardinalCell,
    government: NumeralGovernment,
    analyses: Vec<DistributiveCardinalAnalysis>,
}

/// The source-described outermost construction used by one compound-ordinal
/// analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OrdinalComposition {
    /// An inflected simple ordinal followed by invariant `на десѧте`.
    AnalyticTeen,
    /// One historically compounded ordinal-adjective stem.
    FusedStem,
    /// An outermost join of agreeing ordinal components without a conjunction.
    Asyndetic,
    /// An asyndetic join under the competing account where only its first
    /// component declines and every later component remains in citation form.
    AsyndeticFirstComponent,
    /// An outermost join of agreeing ordinal components by `и`.
    ConjunctionI,
    /// An outermost join of agreeing ordinal components by `ти`.
    ConjunctionTi,
    /// A unit ordinal followed by fixed `междю десетма` “between two tens”.
    BetweenTens,
    /// A unit ordinal followed by fixed genitival `третиаго десѧте` “of the
    /// third ten”; a governing preposition such as attested `въ` is external.
    UnitWithinThirdTen,
}

impl OrdinalComposition {
    /// Whether this construction preserves the source account contradicted by
    /// the same grammar's directly cited all-agreeing manuscript example.
    pub const fn is_disputed(self) -> bool {
        matches!(self, Self::AsyndeticFirstComponent)
    }
}

/// One correlated structural realization of a compound ordinal.
///
/// When an ordinal has more than two chunks, [`Self::construction`] describes
/// the outermost join; nested lower-chunk connectors remain explicit tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdinalPhraseAnalysis {
    pub construction: OrdinalComposition,
    pub tokens: Vec<PhraseToken>,
}

impl OrdinalPhraseAnalysis {
    /// Render each component's source-first form in token order.
    pub fn primary_text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.forms.primary_text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A compound ordinal whose adjective components retain independent evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizedOrdinal {
    value: u16,
    cell: AdjectiveCell,
    analyses: Vec<OrdinalPhraseAnalysis>,
}

impl RealizedOrdinal {
    pub fn new(
        value: u16,
        cell: AdjectiveCell,
        analyses: Vec<OrdinalPhraseAnalysis>,
    ) -> Result<Self, InflectionError> {
        if value == 0 || analyses.is_empty() {
            return Err(InflectionError::InvalidInput {
                reason: "a realized ordinal requires a positive value and an analysis".to_string(),
            });
        }
        if let Some((index, analysis)) = analyses
            .iter()
            .enumerate()
            .find(|(_, analysis)| !valid_ordinal_analysis(value, analysis))
        {
            return Err(InflectionError::InvalidInput {
                reason: format!(
                    "ordinal analysis {index} ({:?}) has an invalid component sequence: {:?}",
                    analysis.construction,
                    analysis
                        .tokens
                        .iter()
                        .map(|token| (token.role, token.forms.primary_text()))
                        .collect::<Vec<_>>()
                ),
            });
        }
        Ok(Self {
            value,
            cell,
            analyses,
        })
    }

    pub const fn value(&self) -> u16 {
        self.value
    }

    pub const fn cell(&self) -> AdjectiveCell {
        self.cell
    }

    pub fn analyses(&self) -> &[OrdinalPhraseAnalysis] {
        &self.analyses
    }

    /// Render the deterministic first structural analysis and token variants.
    pub fn primary_text(&self) -> String {
        self.analyses[0].primary_text()
    }
}

fn valid_ordinal_tokens(tokens: &[PhraseToken]) -> bool {
    if tokens.is_empty()
        || tokens
            .first()
            .is_none_or(|token| token.role != PhraseRole::Numeral)
        || tokens
            .last()
            .is_none_or(|token| token.role != PhraseRole::Numeral)
    {
        return false;
    }

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].role != PhraseRole::Numeral {
            return false;
        }
        index += 1;

        if index < tokens.len() && tokens[index].role == PhraseRole::Preposition {
            index += 1;
            if index >= tokens.len() || tokens[index].role != PhraseRole::Numeral {
                return false;
            }
            index += 1;
        }

        if index == tokens.len() {
            return true;
        }
        if tokens[index].role == PhraseRole::Conjunction {
            index += 1;
            if index == tokens.len() {
                return false;
            }
        } else if tokens[index].role != PhraseRole::Numeral {
            return false;
        }
    }
    true
}

fn valid_ordinal_analysis(value: u16, analysis: &OrdinalPhraseAnalysis) -> bool {
    if !valid_ordinal_tokens(&analysis.tokens) {
        return false;
    }

    let roles = analysis
        .tokens
        .iter()
        .map(|token| token.role)
        .collect::<Vec<_>>();
    match analysis.construction {
        OrdinalComposition::AnalyticTeen => {
            roles
                == [
                    PhraseRole::Numeral,
                    PhraseRole::Preposition,
                    PhraseRole::Numeral,
                ]
        }
        OrdinalComposition::FusedStem => roles == [PhraseRole::Numeral],
        OrdinalComposition::Asyndetic | OrdinalComposition::AsyndeticFirstComponent => {
            roles.get(1) == Some(&PhraseRole::Numeral)
        }
        OrdinalComposition::ConjunctionI => analysis.tokens.get(1).is_some_and(|token| {
            token.role == PhraseRole::Conjunction && token.forms.primary_text() == "и"
        }),
        OrdinalComposition::ConjunctionTi => analysis.tokens.get(1).is_some_and(|token| {
            token.role == PhraseRole::Conjunction && token.forms.primary_text() == "ти"
        }),
        OrdinalComposition::BetweenTens => {
            (21..=29).contains(&value)
                && roles
                    == [
                        PhraseRole::Numeral,
                        PhraseRole::Preposition,
                        PhraseRole::Numeral,
                    ]
                && analysis.tokens[1].forms.primary_text() == "междю"
                && analysis.tokens[2].forms.primary_text() == "десетма"
        }
        OrdinalComposition::UnitWithinThirdTen => {
            (21..=29).contains(&value)
                && roles
                    == [
                        PhraseRole::Numeral,
                        PhraseRole::Numeral,
                        PhraseRole::Numeral,
                    ]
                && analysis.tokens[1].forms.primary_text() == "третиаго"
                && analysis.tokens[2].forms.primary_text() == "десѧте"
        }
    }
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

impl RealizedDistributiveCardinal {
    pub fn new(
        value: u16,
        cell: DistributiveCardinalCell,
        government: NumeralGovernment,
        analyses: Vec<DistributiveCardinalAnalysis>,
    ) -> Result<Self, InflectionError> {
        if value == 0 || analyses.is_empty() {
            return Err(InflectionError::InvalidInput {
                reason:
                    "a realized distributive cardinal requires a positive value and an analysis"
                        .to_string(),
            });
        }
        if analyses.iter().any(|analysis| {
            analysis.tokens.first().is_none_or(|token| {
                token.role != PhraseRole::Preposition || token.forms.primary_text() != "по"
            }) || !valid_cardinal_tokens(&analysis.tokens[1..])
        }) {
            return Err(InflectionError::InvalidInput {
                reason:
                    "a distributive cardinal must contain по followed by a valid cardinal analysis"
                        .to_string(),
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

    pub const fn cell(&self) -> DistributiveCardinalCell {
        self.cell
    }

    /// The agreement or government inherited from the cardinal inside `по`.
    pub const fn government(&self) -> NumeralGovernment {
        self.government
    }

    pub const fn rule_id(&self) -> RuleId {
        RuleId::NumeralCardinalDistributive
    }

    pub fn analyses(&self) -> &[DistributiveCardinalAnalysis] {
        &self.analyses
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CompoundOrdinalStem {
    stem: &'static str,
    source_listed: bool,
}

/// Decline one historically compounded ordinal-adjective head.
///
/// The accepted values are the fused teen stems, exact decades, exact
/// hundreds, and thousand. Values between those heads are assembled by the
/// facade as structured phrases so their independently inflected components
/// retain provenance.
pub fn decline_compound_ordinal_stem(
    value: u16,
    cell: AdjectiveCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    let stems = compound_ordinal_stems(value).ok_or_else(|| InflectionError::InvalidInput {
        reason: "a compound-ordinal stem must be a reviewed teen, decade, hundred, or thousand"
            .to_string(),
    })?;
    let rule_id = compound_ordinal_rule_id(value);
    let citation_cell = cell.form == AdjectiveForm::Short
        && cell.case == Case::Nominative
        && cell.number == Number::Singular
        && cell.gender == Gender::Masculine;

    let mut variants = Vec::with_capacity(stems.len() + 2);
    for stem in stems {
        let citation = format!("{}ъ", stem.stem);
        let mut prediction = crate::adjective::decline_stem(stem.stem, AdjectiveClass::Hard, cell)?;
        prediction.trace.push(RuleStep {
            rule_id,
            before: citation,
            after: prediction.text.clone(),
            reason: "apply the source-reviewed compound-ordinal stem and adjective agreement",
        });
        prediction.rule_id = rule_id;
        variants.push(NumeralVariant {
            prediction,
            status: if stem.source_listed {
                if citation_cell {
                    NumeralVariantStatus::ReviewedTable
                } else {
                    NumeralVariantStatus::ProductiveRule
                }
            } else {
                NumeralVariantStatus::ReconstructedRule
            },
        });
    }

    let lemma = compound_ordinal_lemma(value)?;
    for text in compound_ordinal_corpus_forms(value, cell) {
        variants.push(NumeralVariant::corpus(
            text,
            rule_id,
            &lemma,
            "retain the cell-specific manuscript spelling reported by the reviewed source union",
        ));
    }
    Ok(variants)
}

/// Deterministic source-first citation used for a compound-ordinal head.
pub fn compound_ordinal_lemma(value: u16) -> Result<String, InflectionError> {
    compound_ordinal_stems(value)
        .and_then(|stems| stems.first())
        .map(|stem| format!("{}ъ", stem.stem))
        .ok_or_else(|| InflectionError::InvalidInput {
            reason: "a compound-ordinal lemma requires a reviewed compound head".to_string(),
        })
}

fn compound_ordinal_rule_id(value: u16) -> RuleId {
    match value {
        11..=19 => RuleId::NumeralOrdinalTeen,
        20..=90 => RuleId::NumeralOrdinalDecade,
        100..=900 => RuleId::NumeralOrdinalHundred,
        1_000 => RuleId::NumeralOrdinalThousand,
        _ => RuleId::NumeralOrdinalAdditive,
    }
}

fn compound_ordinal_stems(value: u16) -> Option<&'static [CompoundOrdinalStem]> {
    use CompoundOrdinalStem as Stem;
    match value {
        11 => Some(&[Stem {
            stem: "ѥднонадесѧт",
            source_listed: true,
        }]),
        12 => Some(&[
            Stem {
                stem: "дъванадесѧтьн",
                source_listed: true,
            },
            Stem {
                stem: "дъванадесѧт",
                source_listed: true,
            },
            Stem {
                stem: "дъвонадесѧтьн",
                source_listed: true,
            },
        ]),
        13 => Some(&[Stem {
            stem: "тринадесѧт",
            source_listed: true,
        }]),
        14 => Some(&[Stem {
            stem: "четꙑренадесѧт",
            source_listed: false,
        }]),
        15 => Some(&[Stem {
            stem: "пѧтьнадесѧтьн",
            source_listed: true,
        }]),
        16 => Some(&[Stem {
            stem: "шестонадесѧт",
            source_listed: true,
        }]),
        17 => Some(&[
            Stem {
                stem: "седмънадесѧт",
                source_listed: true,
            },
            Stem {
                stem: "седмънадесѧтьн",
                source_listed: true,
            },
        ]),
        18 => Some(&[
            Stem {
                stem: "осмонадесѧт",
                source_listed: true,
            },
            Stem {
                stem: "осмонадесѧтьн",
                source_listed: true,
            },
        ]),
        19 => Some(&[Stem {
            stem: "девѧтьнадесѧтьн",
            source_listed: true,
        }]),
        20 => Some(&[
            Stem {
                stem: "дъвадесѧтьн",
                source_listed: true,
            },
            Stem {
                stem: "дъвадесѧт",
                source_listed: true,
            },
            Stem {
                stem: "дъводесѧт",
                source_listed: true,
            },
        ]),
        30 => Some(&[Stem {
            stem: "тридесѧтьн",
            source_listed: true,
        }]),
        40 => Some(&[
            Stem {
                stem: "четꙑридесѧтьн",
                source_listed: true,
            },
            Stem {
                stem: "четꙑридесѧт",
                source_listed: true,
            },
        ]),
        50 => Some(&[
            Stem {
                stem: "пѧтьдесѧтьн",
                source_listed: true,
            },
            Stem {
                stem: "пѧтьдесѧт",
                source_listed: true,
            },
        ]),
        60 => Some(&[
            Stem {
                stem: "шестьдесѧт",
                source_listed: true,
            },
            Stem {
                stem: "шестьдесѧтьн",
                source_listed: false,
            },
        ]),
        70 => Some(&[
            Stem {
                stem: "седмьдесѧтьн",
                source_listed: true,
            },
            Stem {
                stem: "седмодесѧтьн",
                source_listed: true,
            },
        ]),
        80 => Some(&[
            Stem {
                stem: "осмьдесѧтьн",
                source_listed: true,
            },
            Stem {
                stem: "осмьдесѧт",
                source_listed: true,
            },
        ]),
        90 => Some(&[
            Stem {
                stem: "девѧтьдесѧтьн",
                source_listed: true,
            },
            Stem {
                stem: "девѧтьдесѧт",
                source_listed: true,
            },
        ]),
        100 => Some(&[Stem {
            stem: "сътьн",
            source_listed: true,
        }]),
        200 => Some(&[
            Stem {
                stem: "дъвосътьн",
                source_listed: true,
            },
            Stem {
                stem: "дроугосътьн",
                source_listed: true,
            },
        ]),
        300 => Some(&[Stem {
            stem: "трисътьн",
            source_listed: true,
        }]),
        400 => Some(&[Stem {
            stem: "четꙑрисътьн",
            source_listed: true,
        }]),
        500 => Some(&[Stem {
            stem: "пѧтьсътьн",
            source_listed: true,
        }]),
        600 => Some(&[Stem {
            stem: "шестосътьн",
            source_listed: true,
        }]),
        700 => Some(&[Stem {
            stem: "седмосътьн",
            source_listed: false,
        }]),
        800 => Some(&[Stem {
            stem: "осмосътьн",
            source_listed: false,
        }]),
        900 => Some(&[Stem {
            stem: "девѧтосътьн",
            source_listed: false,
        }]),
        1_000 => Some(&[Stem {
            stem: "тꙑсѧщьн",
            source_listed: true,
        }]),
        _ => None,
    }
}

fn compound_ordinal_corpus_forms(value: u16, cell: AdjectiveCell) -> &'static [&'static str] {
    use Case::*;
    use Gender::*;
    use Number::*;
    match (
        value,
        cell.form,
        cell.case,
        cell.number,
        cell.gender,
        cell.animacy,
    ) {
        (11, AdjectiveForm::Long, Genitive, Singular, Masculine, _) => &["ѥднонадесѧтааго"],
        (18, AdjectiveForm::Long, Accusative, Singular, Neuter, _) => &["осмонадесѧтоѥ"],
        (19, AdjectiveForm::Long, Locative, Singular, Neuter, _) => &["девꙙтънадесꙙть҆нѣꙿмь"],
        (20, AdjectiveForm::Long, Dative, Singular, Neuter, _) => &["дъвадесꙙтъноуо̑умоу"],
        (20, AdjectiveForm::Long, Accusative, Singular, Neuter, _) => {
            &["двадесꙙтъноѥ", "дводесꙙтъноѥ"]
        }
        (70, AdjectiveForm::Long, Accusative, Singular, Neuter, _) => &["седмь҆десꙙтъноѥ"],
        (100, AdjectiveForm::Long, Genitive, Singular, Neuter, _) => &["сътънааго"],
        (1_000, AdjectiveForm::Short, Dative, Singular, Masculine, _) => &["тꙑсꙙштъноу"],
        _ => &[],
    }
}

/// The two inflectional classes used by Old Church Slavonic collectives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectiveNumeralDeclension {
    /// The `-ој-` members two, both, and three use class `2/p`.
    Pronominal,
    /// Four through ten use the hard class `2/a` `-ер-/-ор-` series.
    Adjectival,
}

impl CollectiveNumeralDeclension {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Pronominal => "pronominal",
            Self::Adjectival => "adjectival",
        }
    }
}

/// The complete inherited collective-numeral series from two through ten.
///
/// `Two`, `Both`, and `Three` are directly listed in Polivanova's class `2/p`.
/// The higher `-ер-/-ор-` series is inherited and productive through ten, but
/// its OCS attestation is uneven. Direct OCS dictionary citations remain
/// distinguished from historically reconstructed members in every result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectiveNumeralIdentity {
    Two,
    Both,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
}

impl CollectiveNumeralIdentity {
    pub const ALL: [Self; 10] = [
        Self::Two,
        Self::Both,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
    ];

    pub const fn value(self) -> u8 {
        match self {
            Self::Two | Self::Both => 2,
            Self::Three => 3,
            Self::Four => 4,
            Self::Five => 5,
            Self::Six => 6,
            Self::Seven => 7,
            Self::Eight => 8,
            Self::Nine => 9,
            Self::Ten => 10,
        }
    }

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::Two => "дъвои",
            Self::Both => "обои",
            Self::Three => "трои",
            Self::Four => "четворъ",
            Self::Five => "пѧтеръ",
            Self::Six => "шестеръ",
            Self::Seven => "седморъ",
            Self::Eight => "осмеръ",
            Self::Nine => "девѧтеръ",
            Self::Ten => "десѧторъ",
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::Two => &["дъвои"],
            Self::Both => &["обои"],
            Self::Three => &["трои"],
            Self::Four => &["четворъ", "четвѣръ"],
            Self::Five => &["пѧтеръ", "пѧторъ"],
            Self::Six => &["шестеръ", "шесторъ"],
            Self::Seven => &["седморъ", "седмеръ"],
            Self::Eight => &["осмеръ", "осморъ"],
            Self::Nine => &["девѧтеръ", "девѧторъ"],
            Self::Ten => &["десѧторъ", "десѧтеръ"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    pub const fn declension(self) -> CollectiveNumeralDeclension {
        match self {
            Self::Two | Self::Both | Self::Three => CollectiveNumeralDeclension::Pronominal,
            Self::Four
            | Self::Five
            | Self::Six
            | Self::Seven
            | Self::Eight
            | Self::Nine
            | Self::Ten => CollectiveNumeralDeclension::Adjectival,
        }
    }

    pub const fn rule_id(self) -> RuleId {
        match self.declension() {
            CollectiveNumeralDeclension::Pronominal => RuleId::NumeralCollectivePronominal,
            CollectiveNumeralDeclension::Adjectival => RuleId::NumeralCollectiveAdjective,
        }
    }

    pub const fn authority(self) -> &'static str {
        "Polivanova 2023 §§285, 287–299, 303–306, 314–316 and OSD spreadsheet; Krys'ko 2020; ESSJa collective-numeral entries"
    }

    const fn pronominal_identity(self) -> Option<StandardPronominalIdentity> {
        match self {
            Self::Two => Some(StandardPronominalIdentity::NumeralDvoi),
            Self::Both => Some(StandardPronominalIdentity::NumeralOboi),
            Self::Three => Some(StandardPronominalIdentity::NumeralTroi),
            _ => None,
        }
    }

    const fn adjective_stems(self) -> &'static [CollectiveAdjectiveStem] {
        match self {
            Self::Four => &COLLECTIVE_FOUR_STEMS,
            Self::Five => &COLLECTIVE_FIVE_STEMS,
            Self::Six => &COLLECTIVE_SIX_STEMS,
            Self::Seven => &COLLECTIVE_SEVEN_STEMS,
            Self::Eight => &COLLECTIVE_EIGHT_STEMS,
            Self::Nine => &COLLECTIVE_NINE_STEMS,
            Self::Ten => &COLLECTIVE_TEN_STEMS,
            Self::Two | Self::Both | Self::Three => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollectiveStemEvidence {
    DirectOcs,
    Reconstructed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CollectiveAdjectiveStem {
    stem: &'static str,
    evidence: CollectiveStemEvidence,
}

impl CollectiveAdjectiveStem {
    const fn new(stem: &'static str, evidence: CollectiveStemEvidence) -> Self {
        Self { stem, evidence }
    }
}

use CollectiveStemEvidence::{DirectOcs, Reconstructed};

const COLLECTIVE_FOUR_STEMS: [CollectiveAdjectiveStem; 2] = [
    CollectiveAdjectiveStem::new("четвор", DirectOcs),
    CollectiveAdjectiveStem::new("четвѣр", DirectOcs),
];
const COLLECTIVE_FIVE_STEMS: [CollectiveAdjectiveStem; 2] = [
    CollectiveAdjectiveStem::new("пѧтер", Reconstructed),
    CollectiveAdjectiveStem::new("пѧтор", Reconstructed),
];
const COLLECTIVE_SIX_STEMS: [CollectiveAdjectiveStem; 2] = [
    CollectiveAdjectiveStem::new("шестер", Reconstructed),
    CollectiveAdjectiveStem::new("шестор", Reconstructed),
];
const COLLECTIVE_SEVEN_STEMS: [CollectiveAdjectiveStem; 2] = [
    CollectiveAdjectiveStem::new("седмор", DirectOcs),
    CollectiveAdjectiveStem::new("седмер", Reconstructed),
];
const COLLECTIVE_EIGHT_STEMS: [CollectiveAdjectiveStem; 2] = [
    CollectiveAdjectiveStem::new("осмер", DirectOcs),
    CollectiveAdjectiveStem::new("осмор", Reconstructed),
];
const COLLECTIVE_NINE_STEMS: [CollectiveAdjectiveStem; 2] = [
    CollectiveAdjectiveStem::new("девѧтер", Reconstructed),
    CollectiveAdjectiveStem::new("девѧтор", Reconstructed),
];
const COLLECTIVE_TEN_STEMS: [CollectiveAdjectiveStem; 2] = [
    CollectiveAdjectiveStem::new("десѧтор", DirectOcs),
    CollectiveAdjectiveStem::new("десѧтер", Reconstructed),
];

/// Decline one collective numeral in its lexically licensed cell system.
pub fn decline_collective(
    identity: CollectiveNumeralIdentity,
    cell: CollectiveNumeralCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    match (identity.pronominal_identity(), cell) {
        (Some(pronominal), CollectiveNumeralCell::Pronominal(cell)) => {
            let requested = CollectiveNumeralCell::Pronominal(cell);
            let mut prediction = crate::pronoun::decline_standard_pronominal(
                pronominal,
                cell.case,
                cell.number,
                cell.gender,
            )
            .map_err(|error| remap_collective_error(error, identity, requested))?;
            let rule_id = identity.rule_id();
            prediction.trace.push(RuleStep {
                rule_id,
                before: identity.canonical_lemma().to_string(),
                after: prediction.text.clone(),
                reason: "apply the reviewed collective numerical-pronoun profile",
            });
            prediction.rule_id = rule_id;
            let status = if cell.case == Case::Nominative
                && cell.number == Number::Singular
                && cell.gender == Gender::Masculine
            {
                NumeralVariantStatus::ReviewedTable
            } else {
                NumeralVariantStatus::ProductiveRule
            };
            Ok(vec![NumeralVariant { prediction, status }])
        }
        (None, CollectiveNumeralCell::Adjectival(cell)) => {
            let requested = CollectiveNumeralCell::Adjectival(cell);
            let rule_id = identity.rule_id();
            let mut variants = identity
                .adjective_stems()
                .iter()
                .map(|stem| {
                    let mut prediction =
                        crate::adjective::decline_stem(stem.stem, AdjectiveClass::Hard, cell)
                            .map_err(|error| remap_collective_error(error, identity, requested))?;
                    prediction.trace.push(RuleStep {
                        rule_id,
                        before: identity.canonical_lemma().to_string(),
                        after: prediction.text.clone(),
                        reason: "apply the inherited collective -ер-/-ор- adjective profile",
                    });
                    prediction.rule_id = rule_id;
                    let citation_cell = cell.form == AdjectiveForm::Short
                        && cell.case == Case::Nominative
                        && cell.number == Number::Singular
                        && cell.gender == Gender::Masculine;
                    Ok(match stem.evidence {
                        CollectiveStemEvidence::DirectOcs if citation_cell => NumeralVariant {
                            prediction,
                            status: NumeralVariantStatus::ReviewedTable,
                        },
                        CollectiveStemEvidence::DirectOcs => NumeralVariant::productive(prediction),
                        CollectiveStemEvidence::Reconstructed => {
                            NumeralVariant::reconstructed(prediction)
                        }
                    })
                })
                .collect::<Result<Vec<_>, InflectionError>>()?;
            if identity == CollectiveNumeralIdentity::Ten
                && cell.form == AdjectiveForm::Short
                && cell.case == Case::Accusative
                && cell.number == Number::Singular
                && cell.gender == Gender::Neuter
            {
                variants.push(NumeralVariant::corpus(
                    "десꙙторо",
                    rule_id,
                    identity.canonical_lemma(),
                    "retain the cell-specific spelling attested in UD OCS PROIEL r2.18",
                ));
            }
            Ok(variants)
        }
        _ => Err(InflectionError::historically_invalid(
            identity.canonical_lemma(),
            RequestedCell::CollectiveNumeral(cell),
        )),
    }
}

fn remap_collective_error(
    error: InflectionError,
    identity: CollectiveNumeralIdentity,
    cell: CollectiveNumeralCell,
) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. } => InflectionError::historically_invalid(
            identity.canonical_lemma(),
            RequestedCell::CollectiveNumeral(cell),
        ),
        InflectionError::UnsupportedCell { .. } => InflectionError::unsupported(
            identity.canonical_lemma(),
            RequestedCell::CollectiveNumeral(cell),
        ),
        other => other,
    }
}

/// The inherited noun declensions used by OCS fractional numerals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FractionalNumeralDeclension {
    UStem,
    AStem,
    IStem,
}

impl FractionalNumeralDeclension {
    pub const fn code(self) -> &'static str {
        match self {
            Self::UStem => "u-stem",
            Self::AStem => "a-stem",
            Self::IStem => "i-stem",
        }
    }
}

/// The source-bounded Old Church Slavonic fractional-numeral lexicon.
///
/// These are substantival numerals, not a productive arithmetic denominator
/// system. Leuta and Havryliuk distinguish the four OCS noun identities from
/// later Church Slavonic `третина` and compounds such as `полътора`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FractionalNumeralIdentity {
    /// The u-stem word `полъ` “one half”.
    HalfPol,
    /// The a-stem synonym `половина` “one half”.
    HalfPolovina,
    /// The feminine i-stem `четврьть` “one quarter”.
    Quarter,
    /// The feminine a-stem `десѧтина` “one tenth”.
    Tenth,
}

impl FractionalNumeralIdentity {
    pub const ALL: [Self; 4] = [
        Self::HalfPol,
        Self::HalfPolovina,
        Self::Quarter,
        Self::Tenth,
    ];

    pub const fn numerator(self) -> u8 {
        1
    }

    pub const fn denominator(self) -> u8 {
        match self {
            Self::HalfPol | Self::HalfPolovina => 2,
            Self::Quarter => 4,
            Self::Tenth => 10,
        }
    }

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::HalfPol => "полъ",
            Self::HalfPolovina => "половина",
            Self::Quarter => "четврьть",
            Self::Tenth => "десѧтина",
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::HalfPol => &["полъ"],
            Self::HalfPolovina => &["половина"],
            Self::Quarter => &["четврьть"],
            Self::Tenth => &["десѧтина"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    pub const fn declension(self) -> FractionalNumeralDeclension {
        match self {
            Self::HalfPol => FractionalNumeralDeclension::UStem,
            Self::HalfPolovina | Self::Tenth => FractionalNumeralDeclension::AStem,
            Self::Quarter => FractionalNumeralDeclension::IStem,
        }
    }

    pub const fn gender(self) -> Gender {
        match self {
            Self::HalfPol => Gender::Masculine,
            Self::HalfPolovina | Self::Quarter | Self::Tenth => Gender::Feminine,
        }
    }

    pub const fn rule_id(self) -> RuleId {
        RuleId::NumeralFractionalNoun
    }

    pub const fn authority(self) -> &'static str {
        match self {
            Self::HalfPol => {
                "Leuta and Havryliuk 2018 p. 162; Gorshkov 2002 §86; \
                Polivanova OSD spreadsheet; UD OCS PROIEL r2.18"
            }
            Self::Tenth => {
                "Leuta and Havryliuk 2018 p. 162; Polivanova OSD spreadsheet; \
                UD OCS PROIEL r2.18"
            }
            Self::HalfPolovina | Self::Quarter => "Leuta and Havryliuk 2018 p. 162",
        }
    }

    fn noun_lexeme(self) -> NounLexeme {
        NounLexeme {
            lemma: self.canonical_lemma().to_string(),
            class: match self.declension() {
                FractionalNumeralDeclension::UStem => NounClass::UMasculine,
                FractionalNumeralDeclension::AStem => NounClass::AHard,
                FractionalNumeralDeclension::IStem => NounClass::IFeminine,
            },
            gender: self.gender(),
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        }
    }
}

/// Decline one source-listed fractional noun through all noun cells.
pub fn decline_fractional(
    identity: FractionalNumeralIdentity,
    cell: NounCell,
) -> Result<Vec<NumeralVariant>, InflectionError> {
    let mut prediction = crate::noun::decline(&identity.noun_lexeme(), cell)?;
    let rule_id = identity.rule_id();
    prediction.trace.push(RuleStep {
        rule_id,
        before: identity.canonical_lemma().to_string(),
        after: prediction.text.clone(),
        reason: "apply the source-listed fractional noun's inherited declension",
    });
    prediction.rule_id = rule_id;
    let status = if cell.case == Case::Nominative && cell.number == Number::Singular {
        NumeralVariantStatus::ReviewedTable
    } else {
        NumeralVariantStatus::ProductiveRule
    };
    let mut variants = vec![NumeralVariant { prediction, status }];

    match (identity, cell.case, cell.number) {
        (FractionalNumeralIdentity::HalfPol, Case::Accusative, Number::Singular) => {
            variants.push(NumeralVariant::corpus(
                "полъ",
                rule_id,
                identity.canonical_lemma(),
                "retain the fractional use attested in Codex Marianus Luke 19:8",
            ));
        }
        (FractionalNumeralIdentity::Tenth, Case::Accusative, Number::Singular) => {
            variants.push(NumeralVariant::corpus(
                "десѧтинѫ",
                rule_id,
                identity.canonical_lemma(),
                "retain the tithe uses attested in Codex Marianus Luke 11:42 and 18:12",
            ));
        }
        _ => {}
    }

    Ok(variants)
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

// ---------------------------------------------------------------------------
// Value-driven cardinal composition (moved down from the facade resolver so
// the rewrite-pilot facade can compose numerals without depending on the fat
// `old-church-slavonic` crate; the facade now delegates here).
// ---------------------------------------------------------------------------

/// Lowest integer served by the value-driven cardinal composer.
pub const MIN_CARDINAL_VALUE: u16 = 1;

/// Highest integer licensed by the declared Old Church Slavonic source
/// profile for cardinal composition (`тъма` / `десѧть тꙑсѫшть`).
///
/// This is an evidential boundary, not an integer-storage limit: the reviewed
/// sources specify the construction of cardinals through the myriad but do
/// not determine higher magnitude heads.
pub const MAX_CARDINAL_VALUE: u16 = 10_000;

/// Lowest integer owned by the structured compound-cardinal API; values one
/// through ten belong to [`CardinalNumeralIdentity`] instead.
pub const MIN_COMPOUND_CARDINAL_VALUE: u16 = 11;

pub(crate) const COMPOUND_CARDINAL_AUTHORITY: &str = "UT OCS Online §44.11–10,000; Polivanova 2023 \
    §§321–322, 345–351, 373–374, 383–384";

pub(crate) const DISTRIBUTIVE_CARDINAL_AUTHORITY: &str = "Leuta and Havryliuk 2018 pp. 154, \
    156, 164; UD OCS PROIEL r2.18 and native Syntacticus: Codex Zographensis \
    Mark 14:19 and 6:40, Codex Marianus Luke 9:14 and 10:1, John 8:9 and \
    21:25, and Codex Suprasliensis sentences 245344 and 253762";

/// Wrap one invariant reviewed grammar word (a connector, preposition, or
/// frozen numeral form) as a single-variant [`FormSet`] with its provenance.
pub fn grammar_token(
    text: &str,
    rule_id: RuleId,
    source_feature: &str,
    authority: &'static str,
) -> Result<FormSet, InflectionError> {
    let text = crate::orthography::canonical_display(text)?;
    let form = FormVariant {
        text: text.clone(),
        romanization: None,
    };
    let source = FormSource::ReviewedGrammarTable { rule_id };
    Ok(FormSet::new(
        text.clone(),
        form.clone(),
        Vec::new(),
        source.clone(),
        Vec::new(),
        Vec::new(),
        vec![FormAnalysis {
            variants: vec![form],
            source,
            evidence: vec![MetadataEvidence {
                field: None,
                provenance: MetadataProvenance::ReviewedGrammarTable,
                source_feature: Some(source_feature.to_string()),
                source_form: Some(text),
                crosscheck_features: Vec::new(),
                authority: Some(authority.to_string()),
            }],
            trace: Vec::new(),
        }],
    ))
}

/// Resolve one lexically compatible cell of a reviewed simple cardinal as a
/// provenance-preserving [`FormSet`].
pub fn cardinal_form_set(
    identity: CardinalNumeralIdentity,
    cell: NumeralCell,
) -> Result<FormSet, InflectionError> {
    numeral_variants_form_set(
        identity.canonical_lemma(),
        identity.authority(),
        format!(
            "numeral:cardinal:{}:{}:{}",
            cell.case.abbrev(),
            cell.number.abbrev(),
            cell.gender.map_or("none", Gender::abbrev),
        ),
        decline_cardinal(identity, cell)?,
    )
}

/// Resolve one cell of a reviewed cardinal-magnitude head as a
/// provenance-preserving [`FormSet`].
pub fn magnitude_form_set(
    identity: CardinalMagnitudeIdentity,
    cell: NumeralCell,
) -> Result<FormSet, InflectionError> {
    numeral_variants_form_set(
        identity.canonical_lemma(),
        identity.authority(),
        format!(
            "numeral:cardinal:magnitude:{}:{}:{}:{}",
            identity.rule_id().code(),
            cell.case.abbrev(),
            cell.number.abbrev(),
            cell.gender.map_or("none", Gender::abbrev),
        ),
        decline_magnitude(identity, cell)?,
    )
}

/// Convert an ordered [`NumeralVariant`] list into a provenance-preserving
/// [`FormSet`] keyed by the canonical lemma.
pub fn numeral_variants_form_set(
    lemma: &str,
    authority: &str,
    source_feature: String,
    variants: Vec<NumeralVariant>,
) -> Result<FormSet, InflectionError> {
    let includes_reconstructed = variants
        .iter()
        .any(|variant| variant.status == NumeralVariantStatus::ReconstructedRule);
    let variants = variants
        .into_iter()
        .map(|variant| {
            let rule_id = variant.prediction.rule_id;
            let trace = variant.prediction.trace;
            let form = FormVariant {
                text: crate::orthography::canonical_display(&variant.prediction.text)?,
                romanization: None,
            };
            let source = FormSource::ReviewedGrammarTable { rule_id };
            let analysis = FormAnalysis {
                variants: vec![form.clone()],
                source: source.clone(),
                evidence: vec![MetadataEvidence {
                    field: None,
                    provenance: match variant.status {
                        NumeralVariantStatus::ReviewedTable => {
                            MetadataProvenance::ReviewedGrammarTable
                        }
                        NumeralVariantStatus::ProductiveRule => {
                            MetadataProvenance::ProductiveRuleOutput
                        }
                        NumeralVariantStatus::CorpusAttestation => {
                            MetadataProvenance::CorpusEvaluationObservation
                        }
                        NumeralVariantStatus::PrimaryTextAttestation => {
                            MetadataProvenance::PrimaryTextAttestation
                        }
                        NumeralVariantStatus::ReconstructedRule => {
                            MetadataProvenance::ProductiveRuleOutput
                        }
                    },
                    source_feature: Some(format!("{source_feature}:{}", variant.status.code())),
                    source_form: matches!(
                        variant.status,
                        NumeralVariantStatus::ReviewedTable
                            | NumeralVariantStatus::CorpusAttestation
                            | NumeralVariantStatus::PrimaryTextAttestation
                    )
                    .then(|| form.text.clone()),
                    crosscheck_features: Vec::new(),
                    authority: Some(authority.to_string()),
                }],
                trace: trace.clone(),
            };
            Ok((form, analysis, trace))
        })
        .collect::<Result<Vec<_>, InflectionError>>()?;
    let (primary, primary_analysis, primary_trace) =
        variants
            .first()
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: format!("the {lemma:?} numeral cell has no reviewed forms"),
            })?;
    let mut surface_forms = Vec::with_capacity(variants.len());
    for (form, _, _) in &variants {
        if !surface_forms.contains(form) {
            surface_forms.push(form.clone());
        }
    }
    Ok(FormSet::new(
        crate::orthography::canonical_display(lemma)?,
        primary.clone(),
        surface_forms.into_iter().skip(1).collect(),
        primary_analysis.source.clone(),
        includes_reconstructed
            .then_some(InflectionWarning::IncludesReconstructedForms)
            .into_iter()
            .collect(),
        primary_trace.clone(),
        variants
            .into_iter()
            .map(|(_, analysis, _)| analysis)
            .collect(),
    ))
}

/// Validate the lexical-doublet axes of [`CardinalCompositionOptions`].
pub fn validate_cardinal_composition_options(
    options: CardinalCompositionOptions,
) -> Result<(), InflectionError> {
    if !matches!(
        options.one_identity,
        CardinalNumeralIdentity::OneYedin | CardinalNumeralIdentity::OneYedyn
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "compound one selection must be OneYedin or OneYedyn".to_string(),
        });
    }
    if !matches!(
        options.thousand_identity,
        CardinalMagnitudeIdentity::ThousandBackYus | CardinalMagnitudeIdentity::ThousandLittleYus
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "compound thousand selection must be ThousandBackYus or ThousandLittleYus"
                .to_string(),
        });
    }
    Ok(())
}

/// The final agreeing-or-governing unit digit of a composed cardinal, if any.
pub fn final_cardinal_digit(value: u16) -> Option<u8> {
    let remainder = value % 100;
    if (11..=19).contains(&remainder) {
        Some((remainder - 10) as u8)
    } else {
        match remainder % 10 {
            0 => None,
            digit => Some(digit as u8),
        }
    }
}

/// Map one unit digit onto its cardinal identity under the selected doublet
/// for "one".
pub fn digit_identity(
    digit: u8,
    one_identity: CardinalNumeralIdentity,
) -> Result<CardinalNumeralIdentity, InflectionError> {
    match digit {
        1 => Ok(one_identity),
        2 => Ok(CardinalNumeralIdentity::TwoDva),
        3 => Ok(CardinalNumeralIdentity::Three),
        4 => Ok(CardinalNumeralIdentity::Four),
        5 => Ok(CardinalNumeralIdentity::Five),
        6 => Ok(CardinalNumeralIdentity::Six),
        7 => Ok(CardinalNumeralIdentity::Seven),
        8 => Ok(CardinalNumeralIdentity::Eight),
        9 => Ok(CardinalNumeralIdentity::Nine),
        _ => Err(InflectionError::InvalidInput {
            reason: "a compound-cardinal digit must be between one and nine".to_string(),
        }),
    }
}

fn composed_cardinal_government(
    value: u16,
    one_identity: CardinalNumeralIdentity,
) -> Result<NumeralGovernment, InflectionError> {
    Ok(final_cardinal_digit(value)
        .map(|digit| digit_identity(digit, one_identity))
        .transpose()?
        .map_or(NumeralGovernment::GenitivePlural, |identity| {
            identity.government()
        }))
}

/// Compose a reviewed cardinal from one through 10,000 while retaining
/// correlated multiword analyses and each word's own provenance.
///
/// Values one through ten are the simple reviewed cardinals; eleven upward
/// are the compound constructions. The cell's gender must be present exactly
/// when the final unit agrees (final digit one through four, including the
/// analytic teens).
pub fn cardinal(
    value: u16,
    cell: CompoundCardinalCell,
    options: CardinalCompositionOptions,
) -> Result<RealizedCardinal, InflectionError> {
    if !(MIN_CARDINAL_VALUE..=MAX_CARDINAL_VALUE).contains(&value) {
        return Err(InflectionError::InvalidInput {
            reason: "the reviewed value-driven cardinal range is 1 through 10,000".to_string(),
        });
    }
    validate_cardinal_composition_options(options)?;

    let government = composed_cardinal_government(value, options.one_identity)?;
    let expects_gender = matches!(government, NumeralGovernment::Agreement { .. });
    if cell.gender.is_some() != expects_gender {
        return Err(compound_cardinal_cell_error(value, cell));
    }

    let analyses = compose_cardinal_analyses(value, cell, options)
        .map_err(|error| remap_compound_cardinal_error(error, value, cell))?;

    RealizedCardinal::new(value, cell, government, analyses)
        .map_err(|error| remap_compound_cardinal_error(error, value, cell))
}

/// Compose a reviewed compound cardinal from 11 through 10,000; simple values
/// one through ten are rejected (use [`decline_cardinal`] or [`cardinal`]).
pub fn compound_cardinal(
    value: u16,
    cell: CompoundCardinalCell,
    options: CardinalCompositionOptions,
) -> Result<RealizedCardinal, InflectionError> {
    if !(MIN_COMPOUND_CARDINAL_VALUE..=MAX_CARDINAL_VALUE).contains(&value) {
        return Err(InflectionError::InvalidInput {
            reason: "the reviewed compound-cardinal range is 11 through 10,000".to_string(),
        });
    }
    cardinal(value, cell, options)
}

/// Compose source-backed distributive `по` with a dative cardinal from one
/// through 10,000. The construction is syntactic; all cardinal components
/// retain their ordinary inflection and provenance.
pub fn distributive_cardinal(
    value: u16,
    cell: DistributiveCardinalCell,
    options: CardinalCompositionOptions,
) -> Result<RealizedDistributiveCardinal, InflectionError> {
    if !(MIN_CARDINAL_VALUE..=MAX_CARDINAL_VALUE).contains(&value) {
        return Err(InflectionError::InvalidInput {
            reason: "the reviewed distributive-cardinal range is 1 through 10,000".to_string(),
        });
    }
    validate_cardinal_composition_options(options)?;

    let government = composed_cardinal_government(value, options.one_identity)?;
    if cell.gender.is_some() != matches!(government, NumeralGovernment::Agreement { .. }) {
        return Err(distributive_cardinal_cell_error(value, cell));
    }

    let cardinal_analyses = compose_cardinal_analyses(
        value,
        CompoundCardinalCell {
            case: Case::Dative,
            gender: cell.gender,
        },
        options,
    )
    .map_err(|error| remap_distributive_cardinal_error(error, value, cell))?;

    let preposition = grammar_token(
        "по",
        RuleId::NumeralCardinalDistributive,
        "numeral:cardinal:distributive:po-dative",
        DISTRIBUTIVE_CARDINAL_AUTHORITY,
    )?;
    let analyses = cardinal_analyses
        .into_iter()
        .map(|analysis| {
            let mut tokens = Vec::with_capacity(analysis.tokens.len() + 1);
            tokens.push(PhraseToken {
                role: PhraseRole::Preposition,
                forms: preposition.clone(),
            });
            tokens.extend(analysis.tokens);
            DistributiveCardinalAnalysis { tokens }
        })
        .collect();

    RealizedDistributiveCardinal::new(value, cell, government, analyses)
        .map_err(|error| remap_distributive_cardinal_error(error, value, cell))
}

/// Compose the correlated multiword analyses of a cardinal from one through
/// 10,000 without wrapping them in a [`RealizedCardinal`].
pub fn compose_cardinal_analyses(
    value: u16,
    cell: CompoundCardinalCell,
    options: CardinalCompositionOptions,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    if value == 10_000 {
        return myriad_analyses(cell.case, options.thousand_identity);
    }

    let mut chunks = Vec::new();
    let thousands = (value / 1_000) as u8;
    if thousands != 0 {
        chunks.push(magnitude_analyses(
            thousands,
            options.thousand_identity,
            cell.case,
        )?);
    }
    let hundreds = ((value % 1_000) / 100) as u8;
    if hundreds != 0 {
        chunks.push(magnitude_analyses(
            hundreds,
            CardinalMagnitudeIdentity::HundredSto,
            cell.case,
        )?);
    }
    let remainder = (value % 100) as u8;
    if remainder != 0 {
        chunks.push(lower_cardinal_analyses(
            remainder,
            cell.case,
            cell.gender,
            options.one_identity,
        )?);
    }

    combine_cardinal_chunks(chunks)
}

fn combine_cardinal_chunks(
    chunks: Vec<Vec<CardinalPhraseAnalysis>>,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    let mut combined = vec![CardinalPhraseAnalysis { tokens: Vec::new() }];
    for chunk in chunks {
        let mut next = Vec::with_capacity(combined.len() * chunk.len());
        for prefix in &combined {
            for suffix in &chunk {
                let mut tokens = prefix.tokens.clone();
                if !tokens.is_empty() {
                    tokens.push(PhraseToken {
                        role: PhraseRole::Conjunction,
                        forms: additive_connector()?,
                    });
                }
                tokens.extend(suffix.tokens.clone());
                next.push(CardinalPhraseAnalysis { tokens });
            }
        }
        combined = next;
    }
    Ok(combined)
}

fn lower_cardinal_analyses(
    value: u8,
    case: Case,
    gender: Option<Gender>,
    one_identity: CardinalNumeralIdentity,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    match value {
        1..=9 => Ok(vec![CardinalPhraseAnalysis {
            tokens: vec![numeral_token(digit_component(
                value,
                one_identity,
                case,
                gender,
            )?)],
        }]),
        10 => Ok(vec![CardinalPhraseAnalysis {
            tokens: vec![numeral_token(cardinal_form_set(
                CardinalNumeralIdentity::Ten,
                NumeralCell {
                    case,
                    number: Number::Singular,
                    gender: None,
                },
            )?)],
        }]),
        11..=19 => {
            let unit = digit_component(value - 10, one_identity, case, gender)?;
            Ok(vec![CardinalPhraseAnalysis {
                tokens: vec![
                    numeral_token(unit),
                    PhraseToken {
                        role: PhraseRole::Preposition,
                        forms: grammar_token(
                            "на",
                            RuleId::NumeralCardinalTeen,
                            "numeral:cardinal:teen:preposition",
                            COMPOUND_CARDINAL_AUTHORITY,
                        )?,
                    },
                    PhraseToken {
                        role: PhraseRole::Numeral,
                        forms: grammar_token(
                            "десѧте",
                            RuleId::NumeralCardinalTeen,
                            "numeral:cardinal:teen:invariant-ten",
                            COMPOUND_CARDINAL_AUTHORITY,
                        )?,
                    },
                ],
            }])
        }
        20..=99 => {
            let tens_digit = value / 10;
            let final_digit = value % 10;
            let mut analyses = tens_analyses(tens_digit, case)?;
            if final_digit != 0 {
                let unit = digit_component(final_digit, one_identity, case, gender)?;
                let connector = additive_connector()?;
                for analysis in &mut analyses {
                    analysis.tokens.push(PhraseToken {
                        role: PhraseRole::Conjunction,
                        forms: connector.clone(),
                    });
                    analysis.tokens.push(numeral_token(unit.clone()));
                }
            }
            Ok(analyses)
        }
        _ => Err(InflectionError::InvalidInput {
            reason: "a lower cardinal chunk must be between one and ninety-nine".to_string(),
        }),
    }
}

fn magnitude_analyses(
    multiplier: u8,
    magnitude: CardinalMagnitudeIdentity,
    case: Case,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    let magnitude_number = match multiplier {
        1 | 5..=9 => Number::Singular,
        2 => Number::Dual,
        3 | 4 => Number::Plural,
        _ => {
            return Err(InflectionError::InvalidInput {
                reason: "a magnitude multiplier must be between one and nine".to_string(),
            });
        }
    };
    let magnitude_case = if multiplier >= 5 {
        Case::Genitive
    } else {
        case
    };
    let magnitude_form = magnitude_form_set(
        magnitude,
        NumeralCell {
            case: magnitude_case,
            number: if multiplier >= 5 {
                Number::Plural
            } else {
                magnitude_number
            },
            gender: None,
        },
    )?;
    let tokens = if multiplier == 1 {
        vec![numeral_token(magnitude_form)]
    } else {
        let leading = digit_component(
            multiplier,
            CardinalNumeralIdentity::OneYedin,
            case,
            (multiplier <= 4).then_some(Gender::Feminine),
        )?;
        vec![numeral_token(leading), numeral_token(magnitude_form)]
    };
    Ok(vec![CardinalPhraseAnalysis { tokens }])
}

fn myriad_analyses(
    case: Case,
    thousand_identity: CardinalMagnitudeIdentity,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    let ten_thousand = CardinalPhraseAnalysis {
        tokens: vec![
            numeral_token(cardinal_form_set(
                CardinalNumeralIdentity::Ten,
                NumeralCell {
                    case,
                    number: Number::Singular,
                    gender: None,
                },
            )?),
            numeral_token(magnitude_form_set(
                thousand_identity,
                NumeralCell {
                    case: Case::Genitive,
                    number: Number::Plural,
                    gender: None,
                },
            )?),
        ],
    };
    let tma = CardinalPhraseAnalysis {
        tokens: vec![numeral_token(magnitude_form_set(
            CardinalMagnitudeIdentity::MyriadTma,
            NumeralCell {
                case,
                number: Number::Singular,
                gender: None,
            },
        )?)],
    };
    Ok(vec![ten_thousand, tma])
}

fn additive_connector() -> Result<FormSet, InflectionError> {
    grammar_token(
        "и",
        RuleId::NumeralCardinalAdditive,
        "numeral:cardinal:additive-connector",
        COMPOUND_CARDINAL_AUTHORITY,
    )
}

fn tens_analyses(
    multiplier: u8,
    case: Case,
) -> Result<Vec<CardinalPhraseAnalysis>, InflectionError> {
    match multiplier {
        2 => {
            let leading = digit_component(
                2,
                CardinalNumeralIdentity::OneYedin,
                case,
                Some(Gender::Masculine),
            )?;
            let ten = numeral_variants_form_set(
                CardinalNumeralIdentity::Ten.canonical_lemma(),
                COMPOUND_CARDINAL_AUTHORITY,
                format!("numeral:cardinal:tens:twenty:{}", case.abbrev()),
                decline_counted_ten(case, Number::Dual)?,
            )?;
            Ok(vec![CardinalPhraseAnalysis {
                tokens: vec![numeral_token(leading), numeral_token(ten)],
            }])
        }
        3 | 4 => {
            let identity = digit_identity(multiplier, CardinalNumeralIdentity::OneYedin)?;
            let variants = decline_counted_ten(case, Number::Plural)?;
            if case == Case::Nominative {
                let mut variants = variants.into_iter();
                let primary_ten = variants
                    .next()
                    .ok_or_else(|| InflectionError::InvalidInput {
                        reason: "counted ten has no nominative plural form".to_string(),
                    })?;
                let primary = CardinalPhraseAnalysis {
                    tokens: vec![
                        numeral_token(digit_component(
                            multiplier,
                            CardinalNumeralIdentity::OneYedin,
                            case,
                            Some(Gender::Masculine),
                        )?),
                        numeral_token(numeral_variants_form_set(
                            CardinalNumeralIdentity::Ten.canonical_lemma(),
                            COMPOUND_CARDINAL_AUTHORITY,
                            format!(
                                "numeral:cardinal:tens:{multiplier}:{}:primary",
                                case.abbrev()
                            ),
                            vec![primary_ten],
                        )?),
                    ],
                };
                let mut analyses = vec![primary];
                if let Some(alternative_ten) = variants.next() {
                    analyses.push(CardinalPhraseAnalysis {
                        tokens: vec![
                            numeral_token(cardinal_form_set(
                                identity,
                                NumeralCell {
                                    case,
                                    number: Number::Plural,
                                    gender: Some(Gender::Feminine),
                                },
                            )?),
                            numeral_token(numeral_variants_form_set(
                                CardinalNumeralIdentity::Ten.canonical_lemma(),
                                COMPOUND_CARDINAL_AUTHORITY,
                                format!(
                                    "numeral:cardinal:tens:{multiplier}:{}:alternative",
                                    case.abbrev()
                                ),
                                vec![alternative_ten],
                            )?),
                        ],
                    });
                }
                Ok(analyses)
            } else {
                let leading = cardinal_form_set(
                    identity,
                    NumeralCell {
                        case,
                        number: Number::Plural,
                        gender: Some(Gender::Masculine),
                    },
                )?;
                let ten = numeral_variants_form_set(
                    CardinalNumeralIdentity::Ten.canonical_lemma(),
                    COMPOUND_CARDINAL_AUTHORITY,
                    format!("numeral:cardinal:tens:{multiplier}:{}", case.abbrev()),
                    variants,
                )?;
                Ok(vec![CardinalPhraseAnalysis {
                    tokens: vec![numeral_token(leading), numeral_token(ten)],
                }])
            }
        }
        5..=9 => {
            let leading =
                digit_component(multiplier, CardinalNumeralIdentity::OneYedin, case, None)?;
            let ten = grammar_token(
                "десѧтъ",
                RuleId::NumeralCardinalTens,
                "numeral:cardinal:tens:invariant-genitive-plural-ten",
                COMPOUND_CARDINAL_AUTHORITY,
            )?;
            Ok(vec![CardinalPhraseAnalysis {
                tokens: vec![numeral_token(leading), numeral_token(ten)],
            }])
        }
        _ => Err(InflectionError::InvalidInput {
            reason: "a decimal tens multiplier must be between two and nine".to_string(),
        }),
    }
}

fn digit_component(
    digit: u8,
    one_identity: CardinalNumeralIdentity,
    case: Case,
    gender: Option<Gender>,
) -> Result<FormSet, InflectionError> {
    let identity = digit_identity(digit, one_identity)?;
    let number = match identity.government() {
        NumeralGovernment::Agreement { number } => number,
        NumeralGovernment::GenitivePlural => Number::Singular,
    };
    cardinal_form_set(
        identity,
        NumeralCell {
            case,
            number,
            gender,
        },
    )
}

fn numeral_token(forms: FormSet) -> PhraseToken {
    PhraseToken {
        role: PhraseRole::Numeral,
        forms,
    }
}

fn compound_cardinal_cell_error(value: u16, cell: CompoundCardinalCell) -> InflectionError {
    InflectionError::historically_invalid(
        value.to_string(),
        RequestedCell::CompoundCardinal { value, cell },
    )
}

fn distributive_cardinal_cell_error(value: u16, cell: DistributiveCardinalCell) -> InflectionError {
    InflectionError::historically_invalid(
        value.to_string(),
        RequestedCell::DistributiveCardinal { value, cell },
    )
}

fn remap_compound_cardinal_error(
    error: InflectionError,
    value: u16,
    cell: CompoundCardinalCell,
) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. }
        | InflectionError::UnsupportedCell { .. } => compound_cardinal_cell_error(value, cell),
        other => other,
    }
}

fn remap_distributive_cardinal_error(
    error: InflectionError,
    value: u16,
    cell: DistributiveCardinalCell,
) -> InflectionError {
    match error {
        InflectionError::HistoricallyInvalidCell { .. }
        | InflectionError::UnsupportedCell { .. } => distributive_cardinal_cell_error(value, cell),
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
    fn compound_ordinal_heads_cover_every_agreement_cell_with_typed_evidence() {
        let heads = [
            11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 30, 40, 50, 60, 70, 80, 90, 100, 200, 300, 400,
            500, 600, 700, 800, 900, 1_000,
        ];
        for value in heads {
            let outcomes = AdjectiveCell::all()
                .map(|cell| {
                    (
                        cell,
                        decline_compound_ordinal_stem(value, cell)
                            .unwrap_or_else(|error| panic!("{value} {cell:?}: {error}")),
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(outcomes.len(), 252, "{value}");
            assert!(outcomes.iter().all(|(_, variants)| !variants.is_empty()));
            assert!(
                outcomes
                    .iter()
                    .flat_map(|(_, variants)| variants)
                    .all(|variant| variant.prediction.rule_id == compound_ordinal_rule_id(value))
            );
        }

        let citation = AdjectiveCell {
            form: AdjectiveForm::Short,
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
        };
        assert_eq!(
            decline_compound_ordinal_stem(20, citation)
                .expect("twentieth citations")
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["дъвадесѧтьнъ", "дъвадесѧтъ", "дъводесѧтъ"]
        );
        assert!(
            decline_compound_ordinal_stem(20, citation)
                .expect("twentieth citations")
                .iter()
                .all(|variant| variant.status == NumeralVariantStatus::ReviewedTable)
        );
        assert_eq!(
            decline_compound_ordinal_stem(200, citation)
                .expect("two-hundredth citations")
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["дъвосътьнъ", "дроугосътьнъ"]
        );
        assert!(
            decline_compound_ordinal_stem(700, citation)
                .expect("reconstructed seven-hundredth")
                .iter()
                .all(|variant| variant.status == NumeralVariantStatus::ReconstructedRule)
        );
    }

    #[test]
    fn compound_ordinal_heads_keep_cell_specific_manuscript_spellings() {
        let variants = |value, form, case, gender| {
            decline_compound_ordinal_stem(
                value,
                AdjectiveCell {
                    form,
                    case,
                    number: Number::Singular,
                    gender,
                    animacy: Animacy::Inanimate,
                },
            )
            .expect("licensed compound-ordinal head")
        };
        for (value, form, case, gender, expected) in [
            (
                19,
                AdjectiveForm::Long,
                Case::Locative,
                Gender::Neuter,
                "девꙙтънадесꙙть҆нѣꙿмь",
            ),
            (
                20,
                AdjectiveForm::Long,
                Case::Dative,
                Gender::Neuter,
                "дъвадесꙙтъноуо̑умоу",
            ),
            (
                70,
                AdjectiveForm::Long,
                Case::Accusative,
                Gender::Neuter,
                "седмь҆десꙙтъноѥ",
            ),
            (
                100,
                AdjectiveForm::Long,
                Case::Genitive,
                Gender::Neuter,
                "сътънааго",
            ),
            (
                1_000,
                AdjectiveForm::Short,
                Case::Dative,
                Gender::Masculine,
                "тꙑсꙙштъноу",
            ),
        ] {
            let forms = variants(value, form, case, gender);
            let corpus = forms
                .iter()
                .find(|variant| variant.prediction.text == expected)
                .unwrap_or_else(|| panic!("missing {value} corpus form {expected}"));
            assert_eq!(corpus.status, NumeralVariantStatus::CorpusAttestation);
        }

        for (value, case, gender, excluded) in [
            (11, Case::Genitive, Gender::Neuter, "ѥднонадесѧтааго"),
            (18, Case::Nominative, Gender::Neuter, "осмонадесѧтоѥ"),
            (20, Case::Nominative, Gender::Neuter, "двадесꙙтъноѥ"),
            (70, Case::Nominative, Gender::Neuter, "седмь҆десꙙтъноѥ"),
            (100, Case::Genitive, Gender::Masculine, "сътънааго"),
            (500, Case::Accusative, Gender::Neuter, "пѧтьсьтное"),
        ] {
            assert!(
                variants(value, AdjectiveForm::Long, case, gender)
                    .iter()
                    .filter(|variant| variant.prediction.text == excluded)
                    .all(|variant| variant.status != NumeralVariantStatus::CorpusAttestation),
                "ambiguous or context-excluded corpus spelling leaked into {value} {case:?} {gender:?}"
            );
        }
    }

    #[test]
    fn collective_inventory_is_complete_nonoverlapping_and_typed() {
        assert_eq!(CollectiveNumeralIdentity::ALL.len(), 10);
        let mut aliases = std::collections::BTreeSet::new();
        for identity in CollectiveNumeralIdentity::ALL {
            assert_eq!(
                CollectiveNumeralIdentity::classify_source_union_lemma(identity.canonical_lemma()),
                Some(identity)
            );
            for alias in identity.source_union_aliases() {
                assert!(aliases.insert(*alias), "duplicate collective alias {alias}");
            }
        }
        assert_eq!(CollectiveNumeralIdentity::Two.value(), 2);
        assert_eq!(CollectiveNumeralIdentity::Both.value(), 2);
        assert_eq!(CollectiveNumeralIdentity::Ten.value(), 10);
        assert_eq!(
            CollectiveNumeralIdentity::Three.declension(),
            CollectiveNumeralDeclension::Pronominal
        );
        assert_eq!(
            CollectiveNumeralIdentity::Four.declension(),
            CollectiveNumeralDeclension::Adjectival
        );
    }

    #[test]
    fn collective_pronominals_reuse_every_reviewed_class_2_p_cell() {
        for (identity, pronominal) in [
            (
                CollectiveNumeralIdentity::Two,
                StandardPronominalIdentity::NumeralDvoi,
            ),
            (
                CollectiveNumeralIdentity::Both,
                StandardPronominalIdentity::NumeralOboi,
            ),
            (
                CollectiveNumeralIdentity::Three,
                StandardPronominalIdentity::NumeralTroi,
            ),
        ] {
            let outcomes = crate::GenderedCell::all()
                .map(|cell| {
                    let expected = crate::pronoun::decline_standard_pronominal(
                        pronominal,
                        cell.case,
                        cell.number,
                        cell.gender,
                    );
                    let actual =
                        decline_collective(identity, CollectiveNumeralCell::Pronominal(cell));
                    (cell, expected, actual)
                })
                .collect::<Vec<_>>();
            assert_eq!(outcomes.len(), 63);
            assert_eq!(
                outcomes
                    .iter()
                    .filter(|(_, _, result)| result.is_ok())
                    .count(),
                54,
                "{identity:?}"
            );
            for (cell, expected, actual) in outcomes {
                match (expected, actual) {
                    (Ok(expected), Ok(actual)) => {
                        assert_eq!(actual.len(), 1, "{identity:?} {cell:?}");
                        assert_eq!(
                            actual[0].prediction.text, expected.text,
                            "{identity:?} {cell:?}"
                        );
                        assert_eq!(
                            actual[0].prediction.rule_id,
                            RuleId::NumeralCollectivePronominal
                        );
                    }
                    (
                        Err(InflectionError::HistoricallyInvalidCell { .. }),
                        Err(InflectionError::HistoricallyInvalidCell { .. }),
                    ) => {}
                    pair => {
                        panic!("collective/pronominal mismatch for {identity:?} {cell:?}: {pair:?}")
                    }
                }
            }
        }
    }

    #[test]
    fn higher_collectives_cover_every_adjective_cell_and_both_inherited_stems() {
        for identity in CollectiveNumeralIdentity::ALL
            .into_iter()
            .filter(|identity| identity.declension() == CollectiveNumeralDeclension::Adjectival)
        {
            for cell in AdjectiveCell::all() {
                let variants =
                    decline_collective(identity, CollectiveNumeralCell::Adjectival(cell))
                        .unwrap_or_else(|error| panic!("{identity:?} {cell:?}: {error}"));
                let expected_count = if identity == CollectiveNumeralIdentity::Ten
                    && cell.form == AdjectiveForm::Short
                    && cell.case == Case::Accusative
                    && cell.number == Number::Singular
                    && cell.gender == Gender::Neuter
                {
                    3
                } else {
                    2
                };
                assert_eq!(variants.len(), expected_count, "{identity:?} {cell:?}");
                for (variant, stem) in variants.iter().zip(identity.adjective_stems()) {
                    let adjective =
                        crate::adjective::decline_stem(stem.stem, AdjectiveClass::Hard, cell)
                            .expect("licensed hard adjective");
                    assert_eq!(variant.prediction.text, adjective.text);
                    assert_eq!(
                        variant.prediction.rule_id,
                        RuleId::NumeralCollectiveAdjective
                    );
                }
            }
        }
    }

    #[test]
    fn collective_cell_classes_fail_closed_when_crossed() {
        let adjectival = CollectiveNumeralCell::adjectival(
            AdjectiveForm::Short,
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
            Animacy::Inanimate,
        );
        let pronominal = CollectiveNumeralCell::pronominal(
            Case::Nominative,
            Number::Singular,
            Gender::Masculine,
        );
        assert!(matches!(
            decline_collective(CollectiveNumeralIdentity::Two, adjectival),
            Err(InflectionError::HistoricallyInvalidCell {
                cell: RequestedCell::CollectiveNumeral(cell),
                ..
            }) if cell == adjectival
        ));
        assert!(matches!(
            decline_collective(CollectiveNumeralIdentity::Four, pronominal),
            Err(InflectionError::HistoricallyInvalidCell {
                cell: RequestedCell::CollectiveNumeral(cell),
                ..
            }) if cell == pronominal
        ));
    }

    #[test]
    fn collective_goldens_distinguish_direct_reconstructed_and_corpus_forms() {
        let citation = |identity| {
            decline_collective(
                identity,
                CollectiveNumeralCell::adjectival(
                    AdjectiveForm::Short,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine,
                    Animacy::Inanimate,
                ),
            )
            .expect("collective citation")
        };
        assert_eq!(
            citation(CollectiveNumeralIdentity::Four)
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["четворъ", "четвѣръ"]
        );
        assert!(
            citation(CollectiveNumeralIdentity::Four)
                .iter()
                .all(|variant| variant.status == NumeralVariantStatus::ReviewedTable)
        );
        assert_eq!(
            citation(CollectiveNumeralIdentity::Five)
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["пѧтеръ", "пѧторъ"]
        );
        assert!(
            citation(CollectiveNumeralIdentity::Five)
                .iter()
                .all(|variant| variant.status == NumeralVariantStatus::ReconstructedRule)
        );

        let low = decline_collective(
            CollectiveNumeralIdentity::Two,
            CollectiveNumeralCell::pronominal(Case::Accusative, Number::Singular, Gender::Neuter),
        )
        .expect("collective two");
        assert_eq!(low[0].prediction.text, "дъвоѥ");

        let ten = decline_collective(
            CollectiveNumeralIdentity::Ten,
            CollectiveNumeralCell::adjectival(
                AdjectiveForm::Short,
                Case::Accusative,
                Number::Singular,
                Gender::Neuter,
                Animacy::Inanimate,
            ),
        )
        .expect("collective ten corpus cell");
        assert_eq!(
            ten.iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["десѧторо", "десѧтеро", "десꙙторо"]
        );
        assert_eq!(ten[2].status, NumeralVariantStatus::CorpusAttestation);
    }

    #[test]
    fn fractional_inventory_is_closed_nonoverlapping_and_period_bounded() {
        assert_eq!(FractionalNumeralIdentity::ALL.len(), 4);
        let mut aliases = std::collections::BTreeSet::new();
        for identity in FractionalNumeralIdentity::ALL {
            assert_eq!(identity.numerator(), 1);
            assert_eq!(
                FractionalNumeralIdentity::classify_source_union_lemma(identity.canonical_lemma()),
                Some(identity)
            );
            for alias in identity.source_union_aliases() {
                assert!(aliases.insert(*alias), "duplicate fractional alias {alias}");
            }
        }
        assert_eq!(FractionalNumeralIdentity::HalfPol.denominator(), 2);
        assert_eq!(FractionalNumeralIdentity::HalfPolovina.denominator(), 2);
        assert_eq!(FractionalNumeralIdentity::Quarter.denominator(), 4);
        assert_eq!(FractionalNumeralIdentity::Tenth.denominator(), 10);
        assert_eq!(
            FractionalNumeralIdentity::classify_source_union_lemma("третина"),
            None
        );
        assert_eq!(
            FractionalNumeralIdentity::classify_source_union_lemma("полътора"),
            None
        );
    }

    #[test]
    fn fractional_nouns_reuse_every_inherited_declension_cell() {
        for identity in FractionalNumeralIdentity::ALL {
            let mut reviewed = 0;
            for cell in NounCell::all() {
                let variants = decline_fractional(identity, cell)
                    .unwrap_or_else(|error| panic!("{identity:?} {cell:?}: {error}"));
                assert!(!variants.is_empty(), "{identity:?} {cell:?}");
                let actual = &variants[0];
                let expected = crate::noun::decline(&identity.noun_lexeme(), cell)
                    .expect("licensed inherited noun cell");
                assert_eq!(
                    actual.prediction.text, expected.text,
                    "{identity:?} {cell:?}"
                );
                assert_eq!(actual.prediction.rule_id, RuleId::NumeralFractionalNoun);
                assert_eq!(
                    actual.prediction.trace.last().map(|step| step.rule_id),
                    Some(RuleId::NumeralFractionalNoun)
                );
                if actual.status == NumeralVariantStatus::ReviewedTable {
                    reviewed += 1;
                    assert_eq!(cell.case, Case::Nominative);
                    assert_eq!(cell.number, Number::Singular);
                }
            }
            assert_eq!(reviewed, 1, "{identity:?}");
        }
    }

    #[test]
    fn fractional_goldens_keep_source_classes_and_exact_corpus_evidence() {
        let form = |identity, case, number| {
            decline_fractional(identity, NounCell { case, number })
                .expect("licensed fractional cell")
        };
        assert_eq!(
            form(
                FractionalNumeralIdentity::HalfPol,
                Case::Genitive,
                Number::Singular
            )[0]
            .prediction
            .text,
            "полоу"
        );
        assert_eq!(
            form(
                FractionalNumeralIdentity::HalfPolovina,
                Case::Accusative,
                Number::Singular
            )[0]
            .prediction
            .text,
            "половинѫ"
        );
        assert_eq!(
            form(
                FractionalNumeralIdentity::Quarter,
                Case::Instrumental,
                Number::Singular
            )[0]
            .prediction
            .text,
            "четврьтьѭ"
        );
        let half = form(
            FractionalNumeralIdentity::HalfPol,
            Case::Accusative,
            Number::Singular,
        );
        assert_eq!(half.len(), 2);
        assert_eq!(half[1].prediction.text, "полъ");
        assert_eq!(half[1].status, NumeralVariantStatus::CorpusAttestation);
        let tenth = form(
            FractionalNumeralIdentity::Tenth,
            Case::Accusative,
            Number::Singular,
        );
        assert_eq!(tenth.len(), 2);
        assert_eq!(tenth[1].prediction.text, "десѧтинѫ");
        assert_eq!(tenth[1].status, NumeralVariantStatus::CorpusAttestation);

        for identity in FractionalNumeralIdentity::ALL {
            for cell in NounCell::all()
                .filter(|cell| cell.case != Case::Accusative || cell.number != Number::Singular)
            {
                assert!(
                    form(identity, cell.case, cell.number)
                        .iter()
                        .all(|variant| variant.status != NumeralVariantStatus::CorpusAttestation),
                    "corpus evidence leaked into {identity:?} {cell:?}"
                );
            }
        }
    }

    #[test]
    fn indefinite_quantity_inventory_is_closed_and_not_an_exact_magnitude() {
        assert_eq!(
            IndefiniteNumeralIdentity::ALL,
            [IndefiniteNumeralIdentity::Nesveda]
        );
        assert_eq!(
            IndefiniteNumeralIdentity::classify_source_union_lemma("несъвѣда"),
            Some(IndefiniteNumeralIdentity::Nesveda)
        );
        assert_eq!(
            IndefiniteNumeralIdentity::Nesveda.noun_class(),
            NounClass::AHard
        );
        assert_eq!(
            IndefiniteNumeralIdentity::Nesveda.gender(),
            Gender::Feminine
        );
        assert_eq!(
            CardinalMagnitudeIdentity::classify_source_union_lemma("несъвѣда"),
            None,
            "an incalculable quantity must not enter exact integer composition"
        );
    }

    #[test]
    fn nesveda_reuses_every_a_stem_cell_and_keeps_primary_text_evidence_local() {
        let identity = IndefiniteNumeralIdentity::Nesveda;
        let mut reviewed = 0;
        let mut attested = 0;
        for cell in NounCell::all() {
            let variants = decline_indefinite(identity, cell)
                .unwrap_or_else(|error| panic!("{cell:?}: {error}"));
            assert_eq!(variants.len(), 1, "{cell:?}");
            let actual = &variants[0];
            let expected = crate::noun::decline(&identity.noun_lexeme(), cell)
                .expect("every hard a-stem noun cell is licensed");
            assert_eq!(actual.prediction.text, expected.text, "{cell:?}");
            assert_eq!(actual.prediction.rule_id, RuleId::NumeralIndefiniteNoun);
            assert_eq!(
                actual.prediction.trace.last().map(|step| step.rule_id),
                Some(RuleId::NumeralIndefiniteNoun)
            );
            match actual.status {
                NumeralVariantStatus::ReviewedTable => {
                    reviewed += 1;
                    assert_eq!(
                        cell,
                        NounCell {
                            case: Case::Nominative,
                            number: Number::Singular,
                        }
                    );
                }
                NumeralVariantStatus::PrimaryTextAttestation => {
                    attested += 1;
                    assert_eq!(
                        cell,
                        NounCell {
                            case: Case::Instrumental,
                            number: Number::Plural,
                        }
                    );
                    assert_eq!(actual.prediction.text, "несъвѣдами");
                }
                NumeralVariantStatus::ProductiveRule => {}
                NumeralVariantStatus::CorpusAttestation
                | NumeralVariantStatus::ReconstructedRule => {
                    panic!("wrong evidence status in {cell:?}")
                }
            }
        }
        assert_eq!(reviewed, 1);
        assert_eq!(attested, 1);
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
