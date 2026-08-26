//! Source-reviewed Old Church Slavonic pronouns.

use crate::{
    Case, ClosedClassCell, Gender, InflectionError, Number, PartOfSpeech, Person, PredictedForm,
    RequestedCell, RuleId, RuleStep,
};

/// Prefixal formatives used to derive negative and indefinite pronominal
/// families from an independently inflected base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PronominalPrefix {
    /// Negative `ни-`.
    Ni,
    /// Indefinite `нѣ-`.
    Ne,
}

impl PronominalPrefix {
    pub const ALL: [Self; 2] = [Self::Ni, Self::Ne];

    pub const fn text(self) -> &'static str {
        match self {
            Self::Ni => "ни",
            Self::Ne => "нѣ",
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Ni => "ni",
            Self::Ne => "ne",
        }
    }
}

/// Source-described postpositive formatives in derived pronominal families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PronominalPostpositive {
    /// Bound `-же`.
    Ze,
    /// Bound `-жде`.
    Zhde,
    /// Bound `-жьдо`.
    Zhydo,
    /// Independently written `любо`.
    Liubo,
}

impl PronominalPostpositive {
    pub const ALL: [Self; 4] = [Self::Ze, Self::Zhde, Self::Zhydo, Self::Liubo];

    pub const fn text(self) -> &'static str {
        match self {
            Self::Ze => "же",
            Self::Zhde => "жде",
            Self::Zhydo => "жьдо",
            Self::Liubo => "любо",
        }
    }

    /// Whether this formative is attached to the pronominal word rather than
    /// realized as an independent phrase token.
    pub const fn is_bound(self) -> bool {
        !matches!(self, Self::Liubo)
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Ze => "ze",
            Self::Zhde => "zhde",
            Self::Zhydo => "zhydo",
            Self::Liubo => "liubo",
        }
    }
}

/// Treatment of the direct-case `-то` in `къто` and `чьто` before another
/// bound postpositive. Both outcomes are source-described.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirectToTreatment {
    Retain,
    Drop,
}

impl DirectToTreatment {
    pub const ALL: [Self; 2] = [Self::Retain, Self::Drop];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Retain => "retain",
            Self::Drop => "drop",
        }
    }
}

/// Explicit composition choices for a derived pronominal family.
///
/// A preposition represents the source-described interposition between a
/// prefixal formative and the inflected pronominal base. It is therefore valid
/// only together with [`Self::prefix`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PronominalFamilySpec {
    pub prefix: Option<PronominalPrefix>,
    pub postpositive: Option<PronominalPostpositive>,
    pub preposition: Option<String>,
    pub direct_to: Option<DirectToTreatment>,
}

/// Compose one surface text of a §316 derived pronominal family from its
/// inflected base text and the explicitly selected bound formatives. This is
/// the pure text-level composition shared by the phrase-building facade and
/// direct kernel callers; it performs no spec validation.
pub fn compose_pronominal_family_text(
    text: &str,
    prefix: Option<PronominalPrefix>,
    postpositive: Option<PronominalPostpositive>,
    direct_to: Option<DirectToTreatment>,
) -> Result<String, InflectionError> {
    let stem = match direct_to {
        Some(DirectToTreatment::Drop) => {
            text.strip_suffix("то")
                .ok_or_else(|| InflectionError::InvalidInput {
                    reason: format!("cannot drop direct-case -то from {text:?}"),
                })?
        }
        Some(DirectToTreatment::Retain) | None => text,
    };
    Ok(format!(
        "{}{}{}",
        prefix.map_or("", |value| value.text()),
        stem,
        postpositive.map_or("", |value| value.text())
    ))
}

/// Validate an explicitly selected §316 derived-family spec against one
/// inflected pronominal base, given the base's lemma and its ordered surface
/// texts. This is the pure validation shared by the phrase-building facades;
/// it commits to nothing about how the base was resolved.
pub fn validate_pronominal_family_spec(
    base_lemma: &str,
    base_texts: &[&str],
    case: Case,
    spec: &PronominalFamilySpec,
) -> Result<(), InflectionError> {
    if spec.prefix.is_none() && spec.postpositive.is_none() {
        return Err(InflectionError::InvalidInput {
            reason: "a derived pronominal family requires a prefix or postpositive".to_string(),
        });
    }
    if spec.preposition.is_some() && spec.prefix.is_none() {
        return Err(InflectionError::InvalidInput {
            reason: "a preposition can be interposed only between a prefixal formative and its pronominal base"
                .to_string(),
        });
    }
    if spec.preposition.is_some() && case == Case::Nominative {
        return Err(InflectionError::InvalidInput {
            reason: "an interposed preposition cannot govern a nominative pronominal form"
                .to_string(),
        });
    }

    let bound_postpositive = spec
        .postpositive
        .is_some_and(|particle| particle.is_bound());
    let direct_case = matches!(case, Case::Nominative | Case::Accusative);
    let all_to = base_lemma.ends_with("то") && base_texts.iter().all(|text| text.ends_with("то"));
    let any_to = base_lemma.ends_with("то") || base_texts.iter().any(|text| text.ends_with("то"));
    let explicit_treatment_is_licensed = direct_case && bound_postpositive && all_to;

    if spec.direct_to.is_some() && !explicit_treatment_is_licensed {
        return Err(InflectionError::InvalidInput {
            reason: "direct-case -то treatment is valid only for a uniformly -то-final nominative or accusative base before a bound postpositive"
                .to_string(),
        });
    }
    if direct_case && bound_postpositive && any_to && spec.direct_to.is_none() {
        return Err(InflectionError::InvalidInput {
            reason: "a direct -то-final base before a bound postpositive requires an explicit retain/drop treatment"
                .to_string(),
        });
    }
    Ok(())
}

/// Canonicalize an interposed preposition: exactly one Cyrillic word.
pub fn canonical_cyrillic_preposition(preposition: &str) -> Result<String, InflectionError> {
    let lemma = crate::orthography::Lemma::parse(preposition)?;
    if lemma.script() != crate::Script::Cyrillic {
        return Err(InflectionError::InvalidInput {
            reason: "the interposed preposition must be one Cyrillic word".to_string(),
        });
    }
    Ok(lemma.to_string())
}

/// Compose the ordered phrase tokens of a §316 derived pronominal family
/// from the inflected base's ordered variant texts (primary first). Each
/// returned token is an ordered variant list. Bound formatives stay inside
/// the pronominal token; `любо` is an independent token; an interposed
/// preposition splits the prefixal formative and the preposition into their
/// own leading tokens, mirroring the construction's intermediate status
/// between a free sequence and a unitary wordform.
pub fn compose_pronominal_family_tokens(
    base_lemma: &str,
    base_texts: &[&str],
    case: Case,
    spec: &PronominalFamilySpec,
) -> Result<Vec<Vec<String>>, InflectionError> {
    validate_pronominal_family_spec(base_lemma, base_texts, case, spec)?;
    let interposed = spec
        .preposition
        .as_deref()
        .map(canonical_cyrillic_preposition)
        .transpose()?;
    let prefix_is_separate = interposed.is_some();
    let bound_postpositive = spec.postpositive.filter(|particle| particle.is_bound());
    let pronoun_prefix = if prefix_is_separate { None } else { spec.prefix };
    let mut pronoun_variants: Vec<String> = Vec::new();
    for text in base_texts {
        let composed = compose_pronominal_family_text(
            text,
            pronoun_prefix,
            bound_postpositive,
            spec.direct_to,
        )?;
        if !pronoun_variants.contains(&composed) {
            pronoun_variants.push(composed);
        }
    }
    if pronoun_variants.is_empty() {
        return Err(InflectionError::InvalidInput {
            reason: "a pronominal base unexpectedly had no surface variants".to_string(),
        });
    }
    let mut tokens: Vec<Vec<String>> = Vec::with_capacity(4);
    if let Some(preposition) = interposed {
        let Some(prefix) = spec.prefix else {
            return Err(InflectionError::InvalidInput {
                reason: "an interposed preposition requires a prefixal formative".to_string(),
            });
        };
        tokens.push(vec![prefix.text().to_string()]);
        tokens.push(vec![preposition]);
    }
    tokens.push(pronoun_variants);
    if spec.postpositive == Some(PronominalPostpositive::Liubo) {
        tokens.push(vec![PronominalPostpositive::Liubo.text().to_string()]);
    }
    Ok(tokens)
}

/// The regular pronominal declensions conventionally grouped as OCS class
/// `2/p`. `J` identifies possessives such as `мои`, whose citation `-и` is the
/// surface result of a stem-final *j* rather than a soft consonant ending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PronominalDeclension {
    Hard,
    Soft,
    J,
}

impl PronominalDeclension {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Hard => "hard",
            Self::Soft => "soft",
            Self::J => "j",
        }
    }

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::Hard => RuleId::PronounPronominalHard,
            Self::Soft => RuleId::PronounPronominalSoft,
            Self::J => RuleId::PronounPronominalJ,
        }
    }
}

/// Explicit lexical metadata sufficient to decline one regular agreeing
/// pronoun without a dictionary lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PronominalLexeme {
    pub lemma: String,
    pub declension: PronominalDeclension,
}

/// Reviewed regular identities routed through the productive `2/p` system.
/// Gendered source pages such as `она` and `оно` are aliases of the single
/// grammatical identity `онъ`; identities absent from the bundled dictionary
/// remain available through the typed grammar API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StandardPronominalIdentity {
    RelativeMannerYak,
    DemonstrativeT,
    DemonstrativeOn,
    DemonstrativeOv,
    UniversalVsak,
    UniversalVsek,
    PossessiveVash,
    PossessiveNash,
    PossessiveMoi,
    PossessiveTvoi,
    PossessiveSvoi,
    NumeralDva,
    NumeralDvak,
    NumeralDvoi,
    SamenessYedinak,
    SamenessYedynak,
    IndefiniteYedin,
    IndefiniteYedyn,
    RelativeQuantityYelik,
    AlternativeInak,
    AlternativeIn,
    InterrogativeMannerKak,
    InterrogativeQuantityKolik,
    NumeralOba,
    NumeralOboyak,
    NumeralOboi,
    EmphaticSam,
    DemonstrativeQuantitySelik,
    DemonstrativeMannerTak,
    DemonstrativeQuantityTolik,
    NumeralTroi,
    InterrogativePossessiveChii,
}

impl StandardPronominalIdentity {
    /// Every regular member of Polivanova's 34-lexeme class `2/p`, except the
    /// exceptional anaphoric `*и` and relative `иже`, which have their own
    /// contextual APIs.
    pub const ALL: [Self; 32] = [
        Self::RelativeMannerYak,
        Self::DemonstrativeT,
        Self::DemonstrativeOn,
        Self::DemonstrativeOv,
        Self::UniversalVsak,
        Self::UniversalVsek,
        Self::PossessiveVash,
        Self::PossessiveNash,
        Self::PossessiveMoi,
        Self::PossessiveTvoi,
        Self::PossessiveSvoi,
        Self::NumeralDva,
        Self::NumeralDvak,
        Self::NumeralDvoi,
        Self::SamenessYedinak,
        Self::SamenessYedynak,
        Self::IndefiniteYedin,
        Self::IndefiniteYedyn,
        Self::RelativeQuantityYelik,
        Self::AlternativeInak,
        Self::AlternativeIn,
        Self::InterrogativeMannerKak,
        Self::InterrogativeQuantityKolik,
        Self::NumeralOba,
        Self::NumeralOboyak,
        Self::NumeralOboi,
        Self::EmphaticSam,
        Self::DemonstrativeQuantitySelik,
        Self::DemonstrativeMannerTak,
        Self::DemonstrativeQuantityTolik,
        Self::NumeralTroi,
        Self::InterrogativePossessiveChii,
    ];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::RelativeMannerYak => "ꙗкъ",
            Self::DemonstrativeT => "тъ",
            Self::DemonstrativeOn => "онъ",
            Self::DemonstrativeOv => "овъ",
            Self::UniversalVsak => "вьсакъ",
            Self::UniversalVsek => "вьсѣкъ",
            Self::PossessiveVash => "вашь",
            Self::PossessiveNash => "нашь",
            Self::PossessiveMoi => "мои",
            Self::PossessiveTvoi => "твои",
            Self::PossessiveSvoi => "свои",
            Self::NumeralDva => "дъва",
            Self::NumeralDvak => "дъвакъ",
            Self::NumeralDvoi => "дъвои",
            Self::SamenessYedinak => "ѥдинакъ",
            Self::SamenessYedynak => "ѥдьнакъ",
            Self::IndefiniteYedin => "ѥдинъ",
            Self::IndefiniteYedyn => "ѥдьнъ",
            Self::RelativeQuantityYelik => "ѥликъ",
            Self::AlternativeInak => "инакъ",
            Self::AlternativeIn => "инъ",
            Self::InterrogativeMannerKak => "какъ",
            Self::InterrogativeQuantityKolik => "коликъ",
            Self::NumeralOba => "оба",
            Self::NumeralOboyak => "обоꙗкъ",
            Self::NumeralOboi => "обои",
            Self::EmphaticSam => "самъ",
            Self::DemonstrativeQuantitySelik => "селикъ",
            Self::DemonstrativeMannerTak => "такъ",
            Self::DemonstrativeQuantityTolik => "толикъ",
            Self::NumeralTroi => "трои",
            Self::InterrogativePossessiveChii => "чии",
        }
    }

    pub const fn declension(self) -> PronominalDeclension {
        match self {
            Self::RelativeMannerYak
            | Self::DemonstrativeT
            | Self::DemonstrativeOn
            | Self::DemonstrativeOv
            | Self::UniversalVsak
            | Self::UniversalVsek
            | Self::NumeralDva
            | Self::NumeralDvak
            | Self::SamenessYedinak
            | Self::SamenessYedynak
            | Self::IndefiniteYedin
            | Self::IndefiniteYedyn
            | Self::RelativeQuantityYelik
            | Self::AlternativeInak
            | Self::AlternativeIn
            | Self::InterrogativeMannerKak
            | Self::InterrogativeQuantityKolik
            | Self::NumeralOba
            | Self::NumeralOboyak
            | Self::EmphaticSam
            | Self::DemonstrativeQuantitySelik
            | Self::DemonstrativeMannerTak
            | Self::DemonstrativeQuantityTolik => PronominalDeclension::Hard,
            Self::PossessiveVash | Self::PossessiveNash => PronominalDeclension::Soft,
            Self::PossessiveMoi
            | Self::PossessiveTvoi
            | Self::PossessiveSvoi
            | Self::NumeralDvoi
            | Self::NumeralOboi
            | Self::NumeralTroi
            | Self::InterrogativePossessiveChii => PronominalDeclension::J,
        }
    }

    /// Primary API ownership. This is a semantic routing decision, independent
    /// of the common morphological class. `ѥдинъ` and `самъ` follow LMU's
    /// adjectival analysis; their numeral/pronominal functions remain visible
    /// in lexical evidence rather than creating homographic paradigms.
    pub const fn part_of_speech(self) -> PartOfSpeech {
        match self {
            Self::NumeralDva
            | Self::NumeralDvak
            | Self::NumeralDvoi
            | Self::NumeralOba
            | Self::NumeralOboyak
            | Self::NumeralOboi
            | Self::NumeralTroi => PartOfSpeech::Numeral,
            Self::SamenessYedinak
            | Self::SamenessYedynak
            | Self::IndefiniteYedin
            | Self::IndefiniteYedyn
            | Self::AlternativeInak
            | Self::AlternativeIn
            | Self::EmphaticSam => PartOfSpeech::Adjective,
            Self::RelativeMannerYak
            | Self::RelativeQuantityYelik
            | Self::InterrogativeMannerKak
            | Self::InterrogativeQuantityKolik
            | Self::DemonstrativeQuantitySelik
            | Self::DemonstrativeMannerTak
            | Self::DemonstrativeQuantityTolik
            | Self::InterrogativePossessiveChii => PartOfSpeech::Determiner,
            Self::DemonstrativeT
            | Self::DemonstrativeOn
            | Self::DemonstrativeOv
            | Self::UniversalVsak
            | Self::UniversalVsek
            | Self::PossessiveVash
            | Self::PossessiveNash
            | Self::PossessiveMoi
            | Self::PossessiveTvoi
            | Self::PossessiveSvoi => PartOfSpeech::Pronoun,
        }
    }

    pub const fn number_restriction(self) -> crate::NumberRestriction {
        match self {
            Self::NumeralDva | Self::NumeralOba => crate::NumberRestriction::DualOnly,
            _ => crate::NumberRestriction::All,
        }
    }

    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::RelativeMannerYak => &["ꙗкъ"],
            Self::DemonstrativeOn => &["онъ", "она", "оно"],
            Self::DemonstrativeT => &["тъ"],
            Self::DemonstrativeOv => &["овъ"],
            Self::UniversalVsak => &["вьсакъ"],
            Self::UniversalVsek => &["вьсѣкъ"],
            Self::PossessiveVash => &["вашь"],
            Self::PossessiveNash => &["нашь"],
            Self::PossessiveMoi => &["мои"],
            Self::PossessiveTvoi => &["твои"],
            Self::PossessiveSvoi => &["свои"],
            Self::NumeralDva => &["дъва"],
            Self::NumeralDvak => &["дъвакъ"],
            Self::NumeralDvoi => &["дъвои"],
            Self::SamenessYedinak => &["ѥдинакъ", "единакъ"],
            Self::SamenessYedynak => &["ѥдьнакъ"],
            Self::IndefiniteYedin => &["ѥдинъ", "единъ"],
            Self::IndefiniteYedyn => &["ѥдьнъ"],
            Self::RelativeQuantityYelik => &["ѥликъ"],
            Self::AlternativeInak => &["инакъ"],
            Self::AlternativeIn => &["инъ"],
            Self::InterrogativeMannerKak => &["какъ"],
            Self::InterrogativeQuantityKolik => &["коликъ"],
            Self::NumeralOba => &["оба"],
            Self::NumeralOboyak => &["обоꙗкъ"],
            Self::NumeralOboi => &["обои"],
            Self::EmphaticSam => &["самъ"],
            Self::DemonstrativeQuantitySelik => &["селикъ"],
            Self::DemonstrativeMannerTak => &["такъ"],
            Self::DemonstrativeQuantityTolik => &["толикъ"],
            Self::NumeralTroi => &["трои"],
            Self::InterrogativePossessiveChii => &["чии"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }

    fn stem(self) -> Option<&'static str> {
        match self {
            // These dualia tantum have source citation forms in -а rather than
            // an independently usable masculine singular citation in -ъ.
            Self::NumeralDva => Some("дъв"),
            Self::NumeralOba => Some("об"),
            _ => self
                .canonical_lemma()
                .strip_suffix(match self.declension() {
                    PronominalDeclension::Hard => 'ъ',
                    PronominalDeclension::Soft => 'ь',
                    PronominalDeclension::J => 'и',
                }),
        }
    }
}

/// Decline one reviewed regular identity in Polivanova's class `2/p`.
///
/// Unlike [`decline_pronominal`], this entry point also represents dual-only
/// citation forms such as `дъва` and `оба`, whose stems cannot be recovered by
/// stripping a masculine singular citation ending.
pub fn decline_standard_pronominal(
    identity: StandardPronominalIdentity,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<PredictedForm, InflectionError> {
    let lemma = crate::orthography::canonical_display(identity.canonical_lemma())?;
    if identity.number_restriction() == crate::NumberRestriction::DualOnly && number != Number::Dual
    {
        return Err(InflectionError::historically_invalid(
            lemma,
            pronominal_requested_cell(identity.part_of_speech(), case, number, gender),
        ));
    }
    let stem = identity.stem().ok_or_else(|| {
        InflectionError::invalid_lemma(
            &lemma,
            format!(
                "reviewed {} pronominal identity lacks its citation ending",
                identity.declension().code()
            ),
        )
    })?;
    decline_pronominal_stem(
        lemma,
        stem.to_string(),
        identity.declension(),
        identity.part_of_speech(),
        case,
        number,
        gender,
    )
}

/// Decline one complete gendered cell of the regular `2/p` pronominal system.
/// The source paradigm has no vocative; such requests return a typed
/// historically-invalid result rather than silently copying the nominative.
pub fn decline_pronominal(
    lexeme: &PronominalLexeme,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<PredictedForm, InflectionError> {
    decline_pronominal_for_part_of_speech(lexeme, PartOfSpeech::Pronoun, case, number, gender)
}

/// Decline a regular `2/p` lexeme while retaining the owning public part of
/// speech in typed failures. Determiners share these terminals with pronouns.
pub(crate) fn decline_pronominal_for_part_of_speech(
    lexeme: &PronominalLexeme,
    part_of_speech: PartOfSpeech,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<PredictedForm, InflectionError> {
    if !matches!(
        part_of_speech,
        PartOfSpeech::Pronoun | PartOfSpeech::Determiner
    ) {
        return Err(InflectionError::InvalidInput {
            reason: "the pronominal declension requires pronoun or determiner ownership"
                .to_string(),
        });
    }
    let lemma = crate::orthography::canonical_display(&lexeme.lemma)?;
    let citation_ending = match lexeme.declension {
        PronominalDeclension::Hard => 'ъ',
        PronominalDeclension::Soft => 'ь',
        PronominalDeclension::J => 'и',
    };
    let stem = lemma
        .strip_suffix(citation_ending)
        .filter(|stem| !stem.is_empty())
        .ok_or_else(|| {
            InflectionError::invalid_lemma(
                &lemma,
                format!(
                    "the {} pronominal declension requires a nonempty stem and citation -{}",
                    lexeme.declension.code(),
                    citation_ending
                ),
            )
        })?;
    decline_pronominal_stem(
        lemma.clone(),
        stem.to_string(),
        lexeme.declension,
        part_of_speech,
        case,
        number,
        gender,
    )
}

fn pronominal_requested_cell(
    part_of_speech: PartOfSpeech,
    case: Case,
    number: Number,
    gender: Gender,
) -> RequestedCell {
    RequestedCell::ClosedClass {
        part_of_speech,
        cell: ClosedClassCell {
            case,
            number,
            gender: Some(gender),
            person: None,
        },
    }
}

fn decline_pronominal_stem(
    lemma: String,
    mut stem: String,
    declension: PronominalDeclension,
    part_of_speech: PartOfSpeech,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<PredictedForm, InflectionError> {
    if case == Case::Vocative {
        return Err(InflectionError::historically_invalid(
            lemma,
            pronominal_requested_cell(part_of_speech, case, number, gender),
        ));
    }
    let Some(ending) = pronominal_ending(declension, case, number, gender) else {
        return Err(InflectionError::historically_invalid(
            lemma,
            pronominal_requested_cell(part_of_speech, case, number, gender),
        ));
    };
    let rule_id = declension.rule_id();
    let mut trace = Vec::with_capacity(2);

    if declension == PronominalDeclension::Hard && ending.starts_with(['и', 'ѣ']) {
        if let Some(palatalized) = palatalize_final_velar(&stem) {
            trace.push(RuleStep {
                rule_id: RuleId::PronounPronominalVelar,
                before: stem,
                after: palatalized.clone(),
                reason: "palatalize a final velar before a pronominal ending beginning in и or ѣ",
            });
            stem = palatalized;
        }
    }

    let text = format!("{stem}{ending}");
    trace.push(RuleStep {
        rule_id,
        before: stem,
        after: text.clone(),
        reason: "attach the regular pronominal ending to the reviewed pronominal stem",
    });
    Ok(PredictedForm {
        text,
        rule_id,
        trace,
    })
}

fn pronominal_ending(
    declension: PronominalDeclension,
    case: Case,
    number: Number,
    gender: Gender,
) -> Option<&'static str> {
    if declension == PronominalDeclension::J {
        return j_pronominal_ending(case, number, gender);
    }
    let soft = declension == PronominalDeclension::Soft;
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    Some(match (case, number, gender, soft) {
        (Nominative, Singular, Masculine, false) => "ъ",
        (Nominative, Singular, Masculine, true) => "ь",
        (Nominative, Singular, Feminine, _) => "а",
        (Nominative, Singular, Neuter, false) => "о",
        (Nominative, Singular, Neuter, true) => "е",
        (Accusative, Singular, Masculine, false) => "ъ",
        (Accusative, Singular, Masculine, true) => "ь",
        (Accusative, Singular, Feminine, _) => "ѫ",
        (Accusative, Singular, Neuter, false) => "о",
        (Accusative, Singular, Neuter, true) => "е",
        (Genitive, Singular, Masculine | Neuter, false) => "ого",
        (Genitive, Singular, Masculine | Neuter, true) => "его",
        (Genitive, Singular, Feminine, false) => "оѩ",
        (Genitive, Singular, Feminine, true) => "еѩ",
        (Dative, Singular, Masculine | Neuter, false) => "ому",
        (Dative, Singular, Masculine | Neuter, true) => "ему",
        (Dative | Locative, Singular, Feminine, false) => "ои",
        (Dative | Locative, Singular, Feminine, true) => "еи",
        (Instrumental, Singular, Masculine | Neuter, false) => "ѣмь",
        (Instrumental, Singular, Masculine | Neuter, true) => "имь",
        (Instrumental, Singular, Feminine, false) => "оѭ",
        (Instrumental, Singular, Feminine, true) => "еѭ",
        (Locative, Singular, Masculine | Neuter, false) => "омь",
        (Locative, Singular, Masculine | Neuter, true) => "емь",

        (Nominative | Accusative, Dual, Masculine, _) => "а",
        (Nominative | Accusative, Dual, Feminine | Neuter, false) => "ѣ",
        (Nominative | Accusative, Dual, Feminine | Neuter, true) => "и",
        (Genitive | Locative, Dual, _, false) => "ою",
        (Genitive | Locative, Dual, _, true) => "ею",
        (Dative | Instrumental, Dual, _, false) => "ѣма",
        (Dative | Instrumental, Dual, _, true) => "има",

        (Nominative, Plural, Masculine, _) => "и",
        (Nominative, Plural, Feminine, false) => "ы",
        (Nominative, Plural, Feminine, true) => "ѧ",
        (Nominative, Plural, Neuter, _) => "а",
        (Accusative, Plural, Masculine | Feminine, false) => "ы",
        (Accusative, Plural, Masculine | Feminine, true) => "ѧ",
        (Accusative, Plural, Neuter, _) => "а",
        (Genitive | Locative, Plural, _, false) => "ѣхъ",
        (Genitive | Locative, Plural, _, true) => "ихъ",
        (Dative, Plural, _, false) => "ѣмъ",
        (Dative, Plural, _, true) => "имъ",
        (Instrumental, Plural, _, false) => "ѣми",
        (Instrumental, Plural, _, true) => "ими",
        (Case::Vocative, _, _, _) => return None,
    })
}

fn j_pronominal_ending(case: Case, number: Number, gender: Gender) -> Option<&'static str> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    Some(match (case, number, gender) {
        (Nominative, Singular, Masculine) => "и",
        (Nominative, Singular, Feminine) => "ꙗ",
        (Nominative, Singular, Neuter) => "ѥ",
        (Accusative, Singular, Masculine) => "и",
        (Accusative, Singular, Feminine) => "ѭ",
        (Accusative, Singular, Neuter) => "ѥ",
        (Genitive, Singular, Masculine | Neuter) => "ѥго",
        (Genitive, Singular, Feminine) => "ѥѩ",
        (Dative, Singular, Masculine | Neuter) => "ѥму",
        (Dative | Locative, Singular, Feminine) => "ѥи",
        (Instrumental, Singular, Masculine | Neuter) => "имь",
        (Instrumental, Singular, Feminine) => "ѥѭ",
        (Locative, Singular, Masculine | Neuter) => "ѥмь",

        (Nominative | Accusative, Dual, Masculine) => "ꙗ",
        (Nominative | Accusative, Dual, Feminine | Neuter) => "и",
        (Genitive | Locative, Dual, _) => "ѥю",
        (Dative | Instrumental, Dual, _) => "има",

        (Nominative, Plural, Masculine) => "и",
        (Nominative, Plural, Feminine) => "ѩ",
        (Nominative, Plural, Neuter) => "ꙗ",
        (Accusative, Plural, Masculine | Feminine) => "ѩ",
        (Accusative, Plural, Neuter) => "ꙗ",
        (Genitive | Locative, Plural, _) => "ихъ",
        (Dative, Plural, _) => "имъ",
        (Instrumental, Plural, _) => "ими",
        (Case::Vocative, _, _) => return None,
    })
}

fn palatalize_final_velar(stem: &str) -> Option<String> {
    let (base, replacement) = if let Some(base) = stem.strip_suffix('к') {
        (base, "ц")
    } else if let Some(base) = stem.strip_suffix('г') {
        (base, "ѕ")
    } else {
        let base = stem.strip_suffix('х')?;
        (base, "с")
    };
    Some(format!("{base}{replacement}"))
}

/// Closed agreeing paradigms whose stem distribution or terminal mixture is
/// not reducible to the regular `2/p` generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IrregularAgreeingIdentity {
    /// Totalizing `вьсь`, with mixed terminals, two canonical doublets, and no dual.
    TotalVes,
    /// Demonstrative `сиць`, with mixed terminals and no dual.
    DemonstrativeSic,
    /// Proximal demonstrative `сь`, with a unique complete paradigm.
    ProximalSi,
    /// Interrogative determiner `кꙑи`, with syncopated and expanded stems.
    InterrogativeKyi,
}

impl IrregularAgreeingIdentity {
    pub const ALL: [Self; 4] = [
        Self::TotalVes,
        Self::DemonstrativeSic,
        Self::ProximalSi,
        Self::InterrogativeKyi,
    ];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::TotalVes => "вьсь",
            Self::DemonstrativeSic => "сиць",
            Self::ProximalSi => "сь",
            Self::InterrogativeKyi => "кꙑи",
        }
    }

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::TotalVes => RuleId::PronounSpecialVes,
            Self::DemonstrativeSic => RuleId::PronounSpecialSic,
            Self::ProximalSi => RuleId::PronounUniqueSi,
            Self::InterrogativeKyi => RuleId::DeterminerInterrogativeKyi,
        }
    }

    pub const fn part_of_speech(self) -> PartOfSpeech {
        match self {
            Self::InterrogativeKyi => PartOfSpeech::Determiner,
            Self::TotalVes | Self::DemonstrativeSic | Self::ProximalSi => PartOfSpeech::Pronoun,
        }
    }
}

/// The numberless, genderless interrogative pronouns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InterrogativePronounIdentity {
    Kto,
    Chto,
}

impl InterrogativePronounIdentity {
    pub const ALL: [Self; 2] = [Self::Kto, Self::Chto];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::Kto => "къто",
            Self::Chto => "чьто",
        }
    }

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::Kto => RuleId::PronounInterrogativeKto,
            Self::Chto => RuleId::PronounInterrogativeChto,
        }
    }
}

/// Return the complete relative-pronoun cell. Free forms are `*и` plus `же`;
/// forms governed by a preposition use the conditioned `н҄-` allomorph. A
/// preposition cannot govern a nominative, and no vocative exists.
pub fn relative_izhe_form(
    case: Case,
    number: Number,
    gender: Gender,
    environment: AnaphoricEnvironment,
) -> Option<String> {
    use AnaphoricEnvironment::AfterPreposition;
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};

    if case == Case::Vocative {
        return None;
    }
    if case == Case::Nominative {
        if environment == AfterPreposition {
            return None;
        }
        return Some(
            match (number, gender) {
                (Singular, Masculine) => "иже",
                (Singular, Neuter) => "ѥже",
                (Singular, Feminine) => "ꙗже",
                (Dual, Masculine) => "ꙗже",
                (Dual, Feminine | Neuter) => "иже",
                (Plural, Masculine) => "иже",
                (Plural, Neuter) => "ꙗже",
                (Plural, Feminine) => "ѩже",
            }
            .to_string(),
        );
    }
    let anaphoric = anaphoric_form(case, number, gender, environment)?;
    let mut text = anaphoric.text.to_string();
    text.push_str("же");
    Some(text)
}

/// Return every source-ordered form in one closed irregular agreeing cell.
/// Empty output denotes a historically invalid vocative or, for `вьсь` and
/// `сиць`, any dual cell.
pub fn irregular_agreeing_forms(
    identity: IrregularAgreeingIdentity,
    case: Case,
    number: Number,
    gender: Gender,
) -> Vec<PronounVariant> {
    match identity {
        IrregularAgreeingIdentity::TotalVes => ves_forms(case, number, gender),
        IrregularAgreeingIdentity::DemonstrativeSic => sic_forms(case, number, gender),
        IrregularAgreeingIdentity::ProximalSi => si_forms(case, number, gender),
        IrregularAgreeingIdentity::InterrogativeKyi => kyi_forms(case, number, gender),
    }
}

/// Return every source-ordered form in one numberless and genderless
/// interrogative cell. Nominative and accusative are syncretic; vocative is
/// historically invalid.
pub fn interrogative_forms(
    identity: InterrogativePronounIdentity,
    case: Case,
) -> Vec<PronounVariant> {
    use PronounVariantStatus::{TablePrimary, TableVariant};
    match (identity, case) {
        (InterrogativePronounIdentity::Kto, Case::Nominative | Case::Accusative) => {
            vec![PronounVariant::new("къто", TablePrimary)]
        }
        (InterrogativePronounIdentity::Kto, Case::Genitive) => {
            vec![PronounVariant::new("кого", TablePrimary)]
        }
        (InterrogativePronounIdentity::Kto, Case::Locative) => {
            vec![PronounVariant::new("комь", TablePrimary)]
        }
        (InterrogativePronounIdentity::Kto, Case::Dative) => {
            vec![PronounVariant::new("кому", TablePrimary)]
        }
        (InterrogativePronounIdentity::Kto, Case::Instrumental) => {
            vec![PronounVariant::new("цѣмь", TablePrimary)]
        }
        (InterrogativePronounIdentity::Chto, Case::Nominative | Case::Accusative) => {
            vec![PronounVariant::new("чьто", TablePrimary)]
        }
        (InterrogativePronounIdentity::Chto, Case::Genitive) => vec![
            PronounVariant::new("чесо", TablePrimary),
            PronounVariant::new("чьсо", TableVariant),
            PronounVariant::new("чесого", TableVariant),
        ],
        (InterrogativePronounIdentity::Chto, Case::Locative) => vec![
            PronounVariant::new("чемь", TablePrimary),
            PronounVariant::new("чесомь", TableVariant),
        ],
        (InterrogativePronounIdentity::Chto, Case::Dative) => vec![
            PronounVariant::new("чему", TablePrimary),
            PronounVariant::new("чесому", TableVariant),
            PronounVariant::new("чьсому", TableVariant),
        ],
        (InterrogativePronounIdentity::Chto, Case::Instrumental) => {
            vec![PronounVariant::new("чимь", TablePrimary)]
        }
        (_, Case::Vocative) => Vec::new(),
    }
}

fn form(text: &'static str) -> Vec<PronounVariant> {
    vec![PronounVariant::new(
        text,
        PronounVariantStatus::TablePrimary,
    )]
}

fn ves_forms(case: Case, number: Number, gender: Gender) -> Vec<PronounVariant> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Plural, Singular};
    use PronounVariantStatus::{TablePrimary, TableVariant};
    match (case, number, gender) {
        (Nominative, Singular, Masculine) | (Accusative, Singular, Masculine) => form("вьсь"),
        (Nominative, Singular, Neuter) | (Accusative, Singular, Neuter) => form("вьсе"),
        (Nominative, Singular, Feminine) => vec![
            PronounVariant::new("вьса", TablePrimary),
            PronounVariant::new("вьсѣ", TableVariant),
        ],
        (Accusative, Singular, Feminine) => form("вьсѫ"),
        (Genitive, Singular, Masculine | Neuter) => form("вьсего"),
        (Genitive, Singular, Feminine) => form("вьсеѩ"),
        (Locative, Singular, Masculine | Neuter) => form("вьсемь"),
        (Dative | Locative, Singular, Feminine) => form("вьсеи"),
        (Dative, Singular, Masculine | Neuter) => form("вьсему"),
        (Instrumental, Singular, Masculine | Neuter) => form("вьсѣмь"),
        (Instrumental, Singular, Feminine) => form("вьсеѭ"),
        (Nominative, Plural, Masculine) => form("вьси"),
        (Nominative | Accusative, Plural, Neuter) => vec![
            PronounVariant::new("вьса", TablePrimary),
            PronounVariant::new("вьсѣ", TableVariant),
        ],
        (Nominative | Accusative, Plural, Feminine) | (Accusative, Plural, Masculine) => {
            form("вьсѧ")
        }
        (Genitive | Locative, Plural, _) => form("вьсѣхъ"),
        (Dative, Plural, _) => form("вьсѣмъ"),
        (Instrumental, Plural, _) => form("вьсѣми"),
        (Case::Vocative, _, _) | (_, Number::Dual, _) => Vec::new(),
    }
}

fn sic_forms(case: Case, number: Number, gender: Gender) -> Vec<PronounVariant> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Plural, Singular};
    let text = match (case, number, gender) {
        (Nominative, Singular, Masculine) | (Accusative, Singular, Masculine) => "сиць",
        (Nominative, Singular, Neuter) | (Accusative, Singular, Neuter) => "сице",
        (Nominative, Singular, Feminine) => "сица",
        (Accusative, Singular, Feminine) => "сицѫ",
        (Genitive, Singular, Masculine | Neuter) => "сицего",
        (Genitive, Singular, Feminine) => "сицеѩ",
        (Locative, Singular, Masculine | Neuter) => "сицемь",
        (Dative | Locative, Singular, Feminine) => "сицеи",
        (Dative, Singular, Masculine | Neuter) => "сицему",
        (Instrumental, Singular, Masculine | Neuter) => "сицѣмь",
        (Instrumental, Singular, Feminine) => "сицеѭ",
        (Nominative, Plural, Masculine) => "сици",
        (Nominative | Accusative, Plural, Neuter) => "сица",
        (Nominative | Accusative, Plural, Feminine) | (Accusative, Plural, Masculine) => "сицѧ",
        (Genitive | Locative, Plural, _) => "сицѣхъ",
        (Dative, Plural, _) => "сицѣмъ",
        (Instrumental, Plural, _) => "сицѣми",
        (Case::Vocative, _, _) | (_, Number::Dual, _) => return Vec::new(),
    };
    form(text)
}

fn si_forms(case: Case, number: Number, gender: Gender) -> Vec<PronounVariant> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    let text = match (case, number, gender) {
        (Nominative, Singular, Masculine) | (Accusative, Singular, Masculine) => "сь",
        (Nominative, Singular, Neuter) | (Accusative, Singular, Neuter) => "се",
        (Nominative, Singular, Feminine) => "си",
        (Accusative, Singular, Feminine) => "сиѭ",
        (Genitive, Singular, Masculine | Neuter) => "сего",
        (Genitive, Singular, Feminine) => "сеѩ",
        (Locative, Singular, Masculine | Neuter) => "семь",
        (Dative | Locative, Singular, Feminine) => "сеи",
        (Dative, Singular, Masculine | Neuter) => "сему",
        (Instrumental, Singular, Masculine | Neuter) => "симь",
        (Instrumental, Singular, Feminine) => "сеѭ",
        (Nominative | Accusative, Dual, Masculine) => "сиꙗ",
        (Nominative | Accusative, Dual, Feminine | Neuter) => "си",
        (Genitive | Locative, Dual, _) => "сею",
        (Dative | Instrumental, Dual, _) => "сима",
        (Nominative, Plural, Masculine) => "сии",
        (Nominative | Accusative, Plural, Neuter) => "си",
        (Nominative | Accusative, Plural, Feminine) | (Accusative, Plural, Masculine) => "сиѩ",
        (Genitive | Locative, Plural, _) => "сихъ",
        (Dative, Plural, _) => "симъ",
        (Instrumental, Plural, _) => "сими",
        (Case::Vocative, _, _) => return Vec::new(),
    };
    form(text)
}

fn kyi_forms(case: Case, number: Number, gender: Gender) -> Vec<PronounVariant> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    let text = match (case, number, gender) {
        (Nominative | Accusative, Singular, Masculine) => "кꙑи",
        (Nominative | Accusative, Singular, Neuter) => "коѥ",
        (Nominative, Singular, Feminine) => "каꙗ",
        (Accusative, Singular, Feminine) => "кѫѭ",
        (Genitive, Singular, Masculine | Neuter) => "коѥго",
        (Genitive, Singular, Feminine) => "коѥѩ",
        (Locative, Singular, Masculine | Neuter) => "коѥмь",
        (Dative | Locative, Singular, Feminine) => "коѥи",
        (Dative, Singular, Masculine | Neuter) => "коѥму",
        (Instrumental, Singular, Masculine | Neuter) => "кꙑимь",
        (Instrumental, Singular, Feminine) => "коѥѭ",
        (Nominative | Accusative, Dual, Masculine) => "каꙗ",
        (Nominative | Accusative, Dual, Feminine | Neuter) => "цѣи",
        (Genitive | Locative, Dual, _) => "коѥю",
        (Dative | Instrumental, Dual, _) => "кꙑима",
        (Nominative, Plural, Masculine) => "ции",
        (Nominative | Accusative, Plural, Neuter) => "каꙗ",
        (Nominative | Accusative, Plural, Feminine) | (Accusative, Plural, Masculine) => "кꙑѩ",
        (Genitive | Locative, Plural, _) => "кꙑихъ",
        (Dative, Plural, _) => "кꙑимъ",
        (Instrumental, Plural, _) => "кꙑими",
        (Case::Vocative, _, _) => return Vec::new(),
    };
    form(text)
}

/// One closed personal-pronoun identity. First- and second-person identities
/// carry number but have intrinsic person; the reflexive is numberless; the
/// third-person anaphoric identity carries gender and is defective in the
/// nominative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PersonalPronounIdentity {
    First,
    Second,
    Reflexive,
    AnaphoricThird,
}

impl PersonalPronounIdentity {
    pub const ALL: [Self; 4] = [
        Self::First,
        Self::Second,
        Self::Reflexive,
        Self::AnaphoricThird,
    ];

    pub const fn canonical_lemma(self) -> &'static str {
        match self {
            Self::First => "азъ",
            Self::Second => "тꙑ",
            Self::Reflexive => "сѧ",
            // Polivanova uses fictional *и as the dictionary identity. The
            // engine omits the metalinguistic asterisk from the valid lemma.
            Self::AnaphoricThird => "и",
        }
    }

    pub const fn person(self) -> Option<Person> {
        match self {
            Self::First => Some(Person::First),
            Self::Second => Some(Person::Second),
            Self::Reflexive => None,
            Self::AnaphoricThird => Some(Person::Third),
        }
    }

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::First => RuleId::PronounPersonalFirst,
            Self::Second => RuleId::PronounPersonalSecond,
            Self::Reflexive => RuleId::PronounReflexive,
            Self::AnaphoricThird => RuleId::PronounAnaphoricThird,
        }
    }

    /// Dictionary-page spellings classified as forms of this grammatical
    /// identity rather than independent complete paradigms.
    pub const fn source_union_aliases(self) -> &'static [&'static str] {
        match self {
            Self::First => &["азъ", "вѣ", "мꙑ", "наю"],
            Self::Second => &["тꙑ", "ва", "вꙑ", "ваю"],
            Self::Reflexive => &["сѧ"],
            Self::AnaphoricThird => &["и", "ѥ", "ѭ", "ими"],
        }
    }

    pub fn classify_source_union_lemma(lemma: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|identity| identity.source_union_aliases().contains(&lemma))
    }
}

/// Select table-primary forms, explicitly marked clitic variants, or both in
/// grammar-table order. This does not guess the prosody of unmarked forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PronounFormSelection {
    All,
    TablePrimary,
    MarkedClitic,
}

impl PronounFormSelection {
    pub const fn code(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::TablePrimary => "table-primary",
            Self::MarkedClitic => "marked-clitic",
        }
    }
}

/// The conditioned allomorph of the third-person anaphoric pronoun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnaphoricEnvironment {
    Free,
    AfterPreposition,
}

impl AnaphoricEnvironment {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::AfterPreposition => "after-preposition",
        }
    }
}

/// Evidential and syntactic status of a reviewed pronoun form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PronounVariantStatus {
    TablePrimary,
    /// A co-listed grammar-table realization after the source-ordered primary.
    TableVariant,
    MarkedClitic,
    /// UT lists the form in its OCS table, while Polivanova finds no OCS
    /// attestation and compares the later Church Slavonic form.
    DisputedMarkedClitic,
    FreeAnaphoric,
    Adprepositional,
}

impl PronounVariantStatus {
    pub const fn is_marked_clitic(self) -> bool {
        matches!(self, Self::MarkedClitic | Self::DisputedMarkedClitic)
    }

    pub const fn is_disputed(self) -> bool {
        matches!(self, Self::DisputedMarkedClitic)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PronounVariant {
    pub text: &'static str,
    pub status: PronounVariantStatus,
}

impl PronounVariant {
    pub const fn new(text: &'static str, status: PronounVariantStatus) -> Self {
        Self { text, status }
    }
}

/// Return the complete first- or second-person cell, optionally restricted to
/// the source-identified table-primary or explicitly marked clitic realization.
pub fn personal_forms(
    identity: PersonalPronounIdentity,
    case: Case,
    number: Number,
    selection: PronounFormSelection,
) -> Vec<PronounVariant> {
    let forms = match identity {
        PersonalPronounIdentity::First => first_person_forms(case, number),
        PersonalPronounIdentity::Second => second_person_forms(case, number),
        PersonalPronounIdentity::Reflexive | PersonalPronounIdentity::AnaphoricThird => Vec::new(),
    };
    select(&forms, selection)
}

/// Return the numberless reflexive-pronoun cell. Nominative and vocative are
/// historically invalid and therefore return no forms.
pub fn reflexive_forms(case: Case, selection: PronounFormSelection) -> Vec<PronounVariant> {
    use PronounVariantStatus::{MarkedClitic, TablePrimary};
    let forms: &[PronounVariant] = match case {
        Case::Nominative | Case::Vocative => &[],
        Case::Accusative => &[PronounVariant::new("сѧ", TablePrimary)],
        Case::Genitive => &[PronounVariant::new("себе", TablePrimary)],
        Case::Locative => &[PronounVariant::new("себѣ", TablePrimary)],
        Case::Dative => &[
            PronounVariant::new("себѣ", TablePrimary),
            PronounVariant::new("си", MarkedClitic),
        ],
        Case::Instrumental => &[PronounVariant::new("собоѭ", TablePrimary)],
    };
    select(forms, selection)
}

/// Return one conditioned third-person anaphoric form. Every nominative and
/// vocative cell is historically invalid; demonstratives such as `тъ` and
/// `онъ`, not reconstructed `*и`, fill nominative syntax.
pub fn anaphoric_form(
    case: Case,
    number: Number,
    gender: Gender,
    environment: AnaphoricEnvironment,
) -> Option<PronounVariant> {
    use AnaphoricEnvironment::{AfterPreposition, Free};
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative};
    use Gender::{Feminine, Masculine, Neuter};
    use Number::{Dual, Plural, Singular};
    use PronounVariantStatus::{Adprepositional, FreeAnaphoric};

    let (free, adprepositional) = match (case, number, gender) {
        (Accusative, Singular, Masculine) => ("и", "н҄ь"),
        (Accusative, Singular, Neuter) => ("ѥ", "н҄ѥ"),
        (Accusative, Singular, Feminine) => ("ѭ", "н҄ѭ"),
        (Genitive, Singular, Masculine | Neuter) => ("ѥго", "н҄ѥго"),
        (Genitive, Singular, Feminine) => ("ѥѩ", "н҄ѥѩ"),
        (Locative, Singular, Masculine | Neuter) => ("ѥмь", "н҄ѥмь"),
        (Locative, Singular, Feminine) => ("ѥи", "н҄ѥи"),
        (Dative, Singular, Masculine | Neuter) => ("ѥму", "н҄ѥму"),
        (Dative, Singular, Feminine) => ("ѥи", "н҄ѥи"),
        (Instrumental, Singular, Masculine | Neuter) => ("имь", "н҄имь"),
        (Instrumental, Singular, Feminine) => ("ѥѭ", "н҄ѥѭ"),

        (Accusative, Dual, Masculine) => ("ꙗ", "н҄ꙗ"),
        (Accusative, Dual, Neuter | Feminine) => ("и", "н҄и"),
        (Genitive | Locative, Dual, _) => ("ѥю", "н҄ѥю"),
        (Dative | Instrumental, Dual, _) => ("има", "н҄има"),

        (Accusative, Plural, Masculine | Feminine) => ("ѩ", "н҄ѩ"),
        (Accusative, Plural, Neuter) => ("ꙗ", "н҄ꙗ"),
        (Genitive | Locative, Plural, _) => ("ихъ", "н҄ихъ"),
        (Dative, Plural, _) => ("имъ", "н҄имъ"),
        (Instrumental, Plural, _) => ("ими", "н҄ими"),
        (Case::Nominative | Case::Vocative, _, _) => return None,
    };
    Some(match environment {
        Free => PronounVariant::new(free, FreeAnaphoric),
        AfterPreposition => PronounVariant::new(adprepositional, Adprepositional),
    })
}

fn select(forms: &[PronounVariant], selection: PronounFormSelection) -> Vec<PronounVariant> {
    forms
        .iter()
        .copied()
        .filter(|form| match selection {
            PronounFormSelection::All => true,
            PronounFormSelection::TablePrimary => !form.status.is_marked_clitic(),
            PronounFormSelection::MarkedClitic => form.status.is_marked_clitic(),
        })
        .collect()
}

fn first_person_forms(case: Case, number: Number) -> Vec<PronounVariant> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Number::{Dual, Plural, Singular};
    use PronounVariantStatus::{DisputedMarkedClitic, MarkedClitic, TablePrimary};

    let forms: &[PronounVariant] = match (case, number) {
        (Nominative, Singular) => &[PronounVariant::new("азъ", TablePrimary)],
        (Accusative, Singular) => &[PronounVariant::new("мѧ", TablePrimary)],
        (Genitive, Singular) => &[PronounVariant::new("мене", TablePrimary)],
        (Locative, Singular) => &[PronounVariant::new("мьнѣ", TablePrimary)],
        (Dative, Singular) => &[
            PronounVariant::new("мьнѣ", TablePrimary),
            PronounVariant::new("ми", MarkedClitic),
        ],
        (Instrumental, Singular) => &[PronounVariant::new("мъноѭ", TablePrimary)],

        (Nominative, Dual) => &[PronounVariant::new("вѣ", TablePrimary)],
        (Accusative, Dual) => &[
            PronounVariant::new("на", TablePrimary),
            PronounVariant::new("нꙑ", MarkedClitic),
        ],
        (Genitive | Locative, Dual) => &[PronounVariant::new("наю", TablePrimary)],
        (Dative, Dual) => &[
            PronounVariant::new("нама", TablePrimary),
            PronounVariant::new("на", DisputedMarkedClitic),
        ],
        (Instrumental, Dual) => &[PronounVariant::new("нама", TablePrimary)],

        (Nominative, Plural) => &[PronounVariant::new("мꙑ", TablePrimary)],
        (Accusative, Plural) => &[PronounVariant::new("нꙑ", TablePrimary)],
        (Genitive | Locative, Plural) => &[PronounVariant::new("насъ", TablePrimary)],
        (Dative, Plural) => &[
            PronounVariant::new("намъ", TablePrimary),
            PronounVariant::new("нꙑ", MarkedClitic),
        ],
        (Instrumental, Plural) => &[PronounVariant::new("нами", TablePrimary)],
        (Case::Vocative, _) => &[],
    };
    forms.to_vec()
}

fn second_person_forms(case: Case, number: Number) -> Vec<PronounVariant> {
    use Case::{Accusative, Dative, Genitive, Instrumental, Locative, Nominative};
    use Number::{Dual, Plural, Singular};
    use PronounVariantStatus::{MarkedClitic, TablePrimary};

    let forms: &[PronounVariant] = match (case, number) {
        (Nominative, Singular) => &[PronounVariant::new("тꙑ", TablePrimary)],
        (Accusative, Singular) => &[PronounVariant::new("тѧ", TablePrimary)],
        (Genitive, Singular) => &[PronounVariant::new("тебе", TablePrimary)],
        (Locative, Singular) => &[PronounVariant::new("тебѣ", TablePrimary)],
        (Dative, Singular) => &[
            PronounVariant::new("тебѣ", TablePrimary),
            PronounVariant::new("ти", MarkedClitic),
        ],
        (Instrumental, Singular) => &[PronounVariant::new("тобоѭ", TablePrimary)],

        (Nominative | Accusative, Dual) => &[
            PronounVariant::new("ва", TablePrimary),
            PronounVariant::new("вꙑ", MarkedClitic),
        ],
        (Genitive | Locative, Dual) => &[PronounVariant::new("ваю", TablePrimary)],
        (Dative, Dual) => &[
            PronounVariant::new("вама", TablePrimary),
            PronounVariant::new("ва", MarkedClitic),
        ],
        (Instrumental, Dual) => &[PronounVariant::new("вама", TablePrimary)],

        (Nominative | Accusative, Plural) => &[PronounVariant::new("вꙑ", TablePrimary)],
        (Genitive | Locative, Plural) => &[PronounVariant::new("васъ", TablePrimary)],
        (Dative, Plural) => &[
            PronounVariant::new("вамъ", TablePrimary),
            PronounVariant::new("вꙑ", MarkedClitic),
        ],
        (Instrumental, Plural) => &[PronounVariant::new("вами", TablePrimary)],
        (Case::Vocative, _) => &[],
    };
    forms.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn irregular_goldens(
        identity: IrregularAgreeingIdentity,
        expected_valid_cells: usize,
        expected: &[&str],
    ) {
        let actual = Number::ALL
            .into_iter()
            .flat_map(|number| {
                Case::ALL.into_iter().flat_map(move |case| {
                    Gender::ALL.into_iter().filter_map(move |gender| {
                        let forms = irregular_agreeing_forms(identity, case, number, gender);
                        (!forms.is_empty()).then(|| {
                            forms
                                .iter()
                                .map(|form| form.text)
                                .collect::<Vec<_>>()
                                .join(" || ")
                        })
                    })
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(actual.len(), expected_valid_cells, "{identity:?}");
        assert_eq!(actual, expected, "{identity:?}");
    }

    #[test]
    fn relative_izhe_inventory_is_complete_in_both_environments() {
        let expected_free = [
            "иже",
            "ꙗже",
            "ѥже",
            "ѥгоже",
            "ѥѩже",
            "ѥгоже",
            "ѥмуже",
            "ѥиже",
            "ѥмуже",
            "иже",
            "ѭже",
            "ѥже",
            "имьже",
            "ѥѭже",
            "имьже",
            "ѥмьже",
            "ѥиже",
            "ѥмьже",
            "ꙗже",
            "иже",
            "иже",
            "ѥюже",
            "ѥюже",
            "ѥюже",
            "имаже",
            "имаже",
            "имаже",
            "ꙗже",
            "иже",
            "иже",
            "имаже",
            "имаже",
            "имаже",
            "ѥюже",
            "ѥюже",
            "ѥюже",
            "иже",
            "ѩже",
            "ꙗже",
            "ихъже",
            "ихъже",
            "ихъже",
            "имъже",
            "имъже",
            "имъже",
            "ѩже",
            "ѩже",
            "ꙗже",
            "имиже",
            "имиже",
            "имиже",
            "ихъже",
            "ихъже",
            "ихъже",
        ];
        let actual_free = Number::ALL
            .into_iter()
            .flat_map(|number| {
                Case::ALL.into_iter().flat_map(move |case| {
                    Gender::ALL.into_iter().filter_map(move |gender| {
                        relative_izhe_form(case, number, gender, AnaphoricEnvironment::Free)
                    })
                })
            })
            .collect::<Vec<_>>();
        assert_eq!(actual_free.len(), 54);
        assert_eq!(actual_free, expected_free);

        let mut prepositional_count = 0;
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    let form = relative_izhe_form(
                        case,
                        number,
                        gender,
                        AnaphoricEnvironment::AfterPreposition,
                    );
                    if matches!(case, Case::Nominative | Case::Vocative) {
                        assert!(form.is_none());
                    } else {
                        let form = form.expect("valid ad-prepositional relative cell");
                        assert!(form.starts_with("н҄"));
                        assert!(form.ends_with("же"));
                        prepositional_count += 1;
                    }
                }
            }
        }
        assert_eq!(prepositional_count, 45);
        assert_eq!(
            relative_izhe_form(
                Case::Dative,
                Number::Singular,
                Gender::Feminine,
                AnaphoricEnvironment::AfterPreposition,
            )
            .as_deref(),
            Some("н҄ѥиже")
        );
    }

    #[test]
    fn mixed_and_unique_agreeing_pronoun_goldens_are_exhaustive() {
        irregular_goldens(
            IrregularAgreeingIdentity::TotalVes,
            36,
            &[
                "вьсь",
                "вьса || вьсѣ",
                "вьсе",
                "вьсего",
                "вьсеѩ",
                "вьсего",
                "вьсему",
                "вьсеи",
                "вьсему",
                "вьсь",
                "вьсѫ",
                "вьсе",
                "вьсѣмь",
                "вьсеѭ",
                "вьсѣмь",
                "вьсемь",
                "вьсеи",
                "вьсемь",
                "вьси",
                "вьсѧ",
                "вьса || вьсѣ",
                "вьсѣхъ",
                "вьсѣхъ",
                "вьсѣхъ",
                "вьсѣмъ",
                "вьсѣмъ",
                "вьсѣмъ",
                "вьсѧ",
                "вьсѧ",
                "вьса || вьсѣ",
                "вьсѣми",
                "вьсѣми",
                "вьсѣми",
                "вьсѣхъ",
                "вьсѣхъ",
                "вьсѣхъ",
            ],
        );
        irregular_goldens(
            IrregularAgreeingIdentity::DemonstrativeSic,
            36,
            &[
                "сиць",
                "сица",
                "сице",
                "сицего",
                "сицеѩ",
                "сицего",
                "сицему",
                "сицеи",
                "сицему",
                "сиць",
                "сицѫ",
                "сице",
                "сицѣмь",
                "сицеѭ",
                "сицѣмь",
                "сицемь",
                "сицеи",
                "сицемь",
                "сици",
                "сицѧ",
                "сица",
                "сицѣхъ",
                "сицѣхъ",
                "сицѣхъ",
                "сицѣмъ",
                "сицѣмъ",
                "сицѣмъ",
                "сицѧ",
                "сицѧ",
                "сица",
                "сицѣми",
                "сицѣми",
                "сицѣми",
                "сицѣхъ",
                "сицѣхъ",
                "сицѣхъ",
            ],
        );
        for identity in [
            IrregularAgreeingIdentity::TotalVes,
            IrregularAgreeingIdentity::DemonstrativeSic,
        ] {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    assert!(
                        irregular_agreeing_forms(identity, case, Number::Dual, gender).is_empty()
                    );
                }
            }
        }
    }

    #[test]
    fn si_and_kyi_goldens_cover_all_fifty_four_nonvocative_cells() {
        irregular_goldens(
            IrregularAgreeingIdentity::ProximalSi,
            54,
            &[
                "сь", "си", "се", "сего", "сеѩ", "сего", "сему", "сеи", "сему", "сь", "сиѭ", "се",
                "симь", "сеѭ", "симь", "семь", "сеи", "семь", "сиꙗ", "си", "си", "сею", "сею",
                "сею", "сима", "сима", "сима", "сиꙗ", "си", "си", "сима", "сима", "сима", "сею",
                "сею", "сею", "сии", "сиѩ", "си", "сихъ", "сихъ", "сихъ", "симъ", "симъ", "симъ",
                "сиѩ", "сиѩ", "си", "сими", "сими", "сими", "сихъ", "сихъ", "сихъ",
            ],
        );
        irregular_goldens(
            IrregularAgreeingIdentity::InterrogativeKyi,
            54,
            &[
                "кꙑи",
                "каꙗ",
                "коѥ",
                "коѥго",
                "коѥѩ",
                "коѥго",
                "коѥму",
                "коѥи",
                "коѥму",
                "кꙑи",
                "кѫѭ",
                "коѥ",
                "кꙑимь",
                "коѥѭ",
                "кꙑимь",
                "коѥмь",
                "коѥи",
                "коѥмь",
                "каꙗ",
                "цѣи",
                "цѣи",
                "коѥю",
                "коѥю",
                "коѥю",
                "кꙑима",
                "кꙑима",
                "кꙑима",
                "каꙗ",
                "цѣи",
                "цѣи",
                "кꙑима",
                "кꙑима",
                "кꙑима",
                "коѥю",
                "коѥю",
                "коѥю",
                "ции",
                "кꙑѩ",
                "каꙗ",
                "кꙑихъ",
                "кꙑихъ",
                "кꙑихъ",
                "кꙑимъ",
                "кꙑимъ",
                "кꙑимъ",
                "кꙑѩ",
                "кꙑѩ",
                "каꙗ",
                "кꙑими",
                "кꙑими",
                "кꙑими",
                "кꙑихъ",
                "кꙑихъ",
                "кꙑихъ",
            ],
        );
    }

    #[test]
    fn numberless_interrogative_goldens_preserve_ordered_variants() {
        let expected = [
            (
                InterrogativePronounIdentity::Kto,
                ["къто", "кого", "кому", "къто", "цѣмь", "комь"].as_slice(),
            ),
            (
                InterrogativePronounIdentity::Chto,
                [
                    "чьто",
                    "чесо || чьсо || чесого",
                    "чему || чесому || чьсому",
                    "чьто",
                    "чимь",
                    "чемь || чесомь",
                ]
                .as_slice(),
            ),
        ];
        for (identity, expected_cells) in expected {
            let actual = Case::ALL
                .into_iter()
                .filter_map(|case| {
                    let forms = interrogative_forms(identity, case);
                    (!forms.is_empty()).then(|| {
                        forms
                            .iter()
                            .map(|form| form.text)
                            .collect::<Vec<_>>()
                            .join(" || ")
                    })
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, expected_cells, "{identity:?}");
            assert!(interrogative_forms(identity, Case::Vocative).is_empty());
        }
    }

    fn complete_pronominal_goldens(
        identity: StandardPronominalIdentity,
        expected: [&str; 54],
    ) -> Vec<String> {
        let mut actual = Vec::new();
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    let form = decline_standard_pronominal(identity, case, number, gender);
                    if case == Case::Vocative {
                        assert!(
                            matches!(form, Err(InflectionError::HistoricallyInvalidCell { .. })),
                            "{identity:?} {case:?} {number:?} {gender:?}"
                        );
                    } else {
                        actual.push(
                            form.unwrap_or_else(|error| {
                                panic!("{identity:?} {case:?} {number:?} {gender:?}: {error}")
                            })
                            .text,
                        );
                    }
                }
            }
        }
        assert_eq!(actual, expected, "{identity:?}");
        actual
    }

    #[test]
    fn regular_pronominal_goldens_cover_every_nonvocative_cell() {
        complete_pronominal_goldens(
            StandardPronominalIdentity::DemonstrativeT,
            [
                "тъ", "та", "то", "того", "тоѩ", "того", "тому", "тои", "тому", "тъ", "тѫ", "то",
                "тѣмь", "тоѭ", "тѣмь", "томь", "тои", "томь", "та", "тѣ", "тѣ", "тою", "тою",
                "тою", "тѣма", "тѣма", "тѣма", "та", "тѣ", "тѣ", "тѣма", "тѣма", "тѣма", "тою",
                "тою", "тою", "ти", "ты", "та", "тѣхъ", "тѣхъ", "тѣхъ", "тѣмъ", "тѣмъ", "тѣмъ",
                "ты", "ты", "та", "тѣми", "тѣми", "тѣми", "тѣхъ", "тѣхъ", "тѣхъ",
            ],
        );
        complete_pronominal_goldens(
            StandardPronominalIdentity::PossessiveNash,
            [
                "нашь",
                "наша",
                "наше",
                "нашего",
                "нашеѩ",
                "нашего",
                "нашему",
                "нашеи",
                "нашему",
                "нашь",
                "нашѫ",
                "наше",
                "нашимь",
                "нашеѭ",
                "нашимь",
                "нашемь",
                "нашеи",
                "нашемь",
                "наша",
                "наши",
                "наши",
                "нашею",
                "нашею",
                "нашею",
                "нашима",
                "нашима",
                "нашима",
                "наша",
                "наши",
                "наши",
                "нашима",
                "нашима",
                "нашима",
                "нашею",
                "нашею",
                "нашею",
                "наши",
                "нашѧ",
                "наша",
                "нашихъ",
                "нашихъ",
                "нашихъ",
                "нашимъ",
                "нашимъ",
                "нашимъ",
                "нашѧ",
                "нашѧ",
                "наша",
                "нашими",
                "нашими",
                "нашими",
                "нашихъ",
                "нашихъ",
                "нашихъ",
            ],
        );
        complete_pronominal_goldens(
            StandardPronominalIdentity::PossessiveMoi,
            [
                "мои",
                "моꙗ",
                "моѥ",
                "моѥго",
                "моѥѩ",
                "моѥго",
                "моѥму",
                "моѥи",
                "моѥму",
                "мои",
                "моѭ",
                "моѥ",
                "моимь",
                "моѥѭ",
                "моимь",
                "моѥмь",
                "моѥи",
                "моѥмь",
                "моꙗ",
                "мои",
                "мои",
                "моѥю",
                "моѥю",
                "моѥю",
                "моима",
                "моима",
                "моима",
                "моꙗ",
                "мои",
                "мои",
                "моима",
                "моима",
                "моима",
                "моѥю",
                "моѥю",
                "моѥю",
                "мои",
                "моѩ",
                "моꙗ",
                "моихъ",
                "моихъ",
                "моихъ",
                "моимъ",
                "моимъ",
                "моимъ",
                "моѩ",
                "моѩ",
                "моꙗ",
                "моими",
                "моими",
                "моими",
                "моихъ",
                "моихъ",
                "моихъ",
            ],
        );
    }

    #[test]
    fn hard_pronominal_velars_palatalize_only_in_conditioning_cells() {
        let lexeme = PronominalLexeme {
            lemma: "такъ".to_string(),
            declension: PronominalDeclension::Hard,
        };
        let nominative_plural =
            decline_pronominal(&lexeme, Case::Nominative, Number::Plural, Gender::Masculine)
                .expect("regular velar pronoun");
        assert_eq!(nominative_plural.text, "таци");
        assert_eq!(nominative_plural.trace.len(), 2);
        assert_eq!(
            nominative_plural.trace[0].rule_id,
            RuleId::PronounPronominalVelar
        );
        assert_eq!(
            decline_pronominal(&lexeme, Case::Genitive, Number::Singular, Gender::Masculine,)
                .expect("unconditioned hard cell")
                .text,
            "такого"
        );
    }

    #[test]
    fn standard_pronominal_source_aliases_are_exhaustive_and_nonoverlapping() {
        let mut aliases = Vec::new();
        for identity in StandardPronominalIdentity::ALL {
            for alias in identity.source_union_aliases() {
                assert_eq!(
                    StandardPronominalIdentity::classify_source_union_lemma(alias),
                    Some(identity)
                );
                aliases.push(*alias);
            }
        }
        aliases.sort_unstable();
        aliases.dedup();
        assert_eq!(aliases.len(), 36);
    }

    #[test]
    fn standard_pronominal_inventory_covers_every_regular_class_2_p_identity() {
        assert_eq!(StandardPronominalIdentity::ALL.len(), 32);
        let mut actual_lexemes = StandardPronominalIdentity::ALL
            .map(StandardPronominalIdentity::canonical_lemma)
            .into_iter()
            .chain(["*и", "иже"])
            .collect::<Vec<_>>();
        let mut source_lexemes = vec![
            "ꙗкъ",
            "вашь",
            "вьсакъ",
            "вьсѣкъ",
            "дъва",
            "дъвакъ",
            "дъвои",
            "ѥдинакъ",
            "ѥдьнакъ",
            "ѥдинъ",
            "ѥдьнъ",
            "ѥликъ",
            "*и",
            "иже",
            "инакъ",
            "инъ",
            "какъ",
            "коликъ",
            "мои",
            "нашь",
            "оба",
            "обоꙗкъ",
            "обои",
            "овъ",
            "онъ",
            "самъ",
            "свои",
            "селикъ",
            "такъ",
            "твои",
            "толикъ",
            "трои",
            "тъ",
            "чии",
        ];
        actual_lexemes.sort_unstable();
        source_lexemes.sort_unstable();
        assert_eq!(actual_lexemes, source_lexemes);
        assert_eq!(
            StandardPronominalIdentity::ALL
                .into_iter()
                .filter(|identity| identity.part_of_speech() == PartOfSpeech::Pronoun)
                .count(),
            10
        );
        assert_eq!(
            StandardPronominalIdentity::ALL
                .into_iter()
                .filter(|identity| identity.part_of_speech() == PartOfSpeech::Adjective)
                .count(),
            7
        );
        assert_eq!(
            StandardPronominalIdentity::ALL
                .into_iter()
                .filter(|identity| identity.part_of_speech() == PartOfSpeech::Numeral)
                .count(),
            7
        );
        assert_eq!(
            StandardPronominalIdentity::ALL
                .into_iter()
                .filter(|identity| identity.part_of_speech() == PartOfSpeech::Determiner)
                .count(),
            8
        );

        let mut successes = 0;
        for identity in StandardPronominalIdentity::ALL {
            for number in Number::ALL {
                for case in Case::ALL {
                    for gender in Gender::ALL {
                        let result = decline_standard_pronominal(identity, case, number, gender);
                        let supported = case != Case::Vocative
                            && (identity.number_restriction() == crate::NumberRestriction::All
                                || number == Number::Dual);
                        if supported {
                            result.unwrap_or_else(|error| {
                                panic!("{identity:?} {case:?} {number:?} {gender:?}: {error}")
                            });
                            successes += 1;
                        } else {
                            let Err(InflectionError::HistoricallyInvalidCell {
                                cell:
                                    RequestedCell::ClosedClass {
                                        part_of_speech,
                                        cell,
                                    },
                                ..
                            }) = result
                            else {
                                panic!(
                                    "expected typed invalid cell for {identity:?} {case:?} {number:?} {gender:?}"
                                );
                            };
                            assert_eq!(part_of_speech, identity.part_of_speech());
                            assert_eq!(cell.case, case);
                            assert_eq!(cell.number, number);
                            assert_eq!(cell.gender, Some(gender));
                            assert_eq!(cell.person, None);
                        }
                    }
                }
            }
        }
        assert_eq!(successes, 1_656);
    }

    #[test]
    fn standard_pronominal_selected_source_goldens_cover_special_stems_and_classes() {
        let form = |identity, case, number, gender| {
            decline_standard_pronominal(identity, case, number, gender)
                .expect("reviewed class 2/p cell")
                .text
        };
        assert_eq!(
            form(
                StandardPronominalIdentity::UniversalVsak,
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
            ),
            "вьсаци"
        );
        assert_eq!(
            form(
                StandardPronominalIdentity::UniversalVsek,
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
            ),
            "вьсѣци"
        );
        assert_eq!(
            form(
                StandardPronominalIdentity::NumeralDva,
                Case::Nominative,
                Number::Dual,
                Gender::Masculine,
            ),
            "дъва"
        );
        assert_eq!(
            form(
                StandardPronominalIdentity::NumeralDva,
                Case::Nominative,
                Number::Dual,
                Gender::Feminine,
            ),
            "дъвѣ"
        );
        assert_eq!(
            form(
                StandardPronominalIdentity::NumeralOba,
                Case::Nominative,
                Number::Dual,
                Gender::Masculine,
            ),
            "оба"
        );
        assert_eq!(
            form(
                StandardPronominalIdentity::NumeralOba,
                Case::Nominative,
                Number::Dual,
                Gender::Neuter,
            ),
            "обѣ"
        );
        for (identity, expected) in [
            (StandardPronominalIdentity::NumeralDvoi, "дъвои"),
            (StandardPronominalIdentity::NumeralOboi, "обои"),
            (StandardPronominalIdentity::NumeralTroi, "трои"),
            (
                StandardPronominalIdentity::InterrogativePossessiveChii,
                "чии",
            ),
        ] {
            assert_eq!(
                form(
                    identity,
                    Case::Nominative,
                    Number::Plural,
                    Gender::Masculine,
                ),
                expected
            );
        }
        for (identity, expected) in [
            (StandardPronominalIdentity::IndefiniteYedin, "ѥдиного"),
            (StandardPronominalIdentity::EmphaticSam, "самого"),
            (StandardPronominalIdentity::AlternativeIn, "иного"),
        ] {
            assert_eq!(
                form(
                    identity,
                    Case::Genitive,
                    Number::Singular,
                    Gender::Masculine,
                ),
                expected
            );
        }
        assert_eq!(
            form(
                StandardPronominalIdentity::DemonstrativeMannerTak,
                Case::Nominative,
                Number::Plural,
                Gender::Masculine,
            ),
            "таци"
        );
    }

    #[test]
    fn regular_pronominal_lexical_shape_is_validated() {
        for invalid in [
            PronominalLexeme {
                lemma: "мои".to_string(),
                declension: PronominalDeclension::Hard,
            },
            PronominalLexeme {
                lemma: "тъ".to_string(),
                declension: PronominalDeclension::Soft,
            },
            PronominalLexeme {
                lemma: "нашь".to_string(),
                declension: PronominalDeclension::J,
            },
            PronominalLexeme {
                lemma: "ъ".to_string(),
                declension: PronominalDeclension::Hard,
            },
        ] {
            assert!(matches!(
                decline_pronominal(
                    &invalid,
                    Case::Nominative,
                    Number::Singular,
                    Gender::Masculine
                ),
                Err(InflectionError::InvalidLemma { .. })
            ));
        }
    }

    #[test]
    fn first_and_second_person_goldens_cover_every_nonvocative_cell() {
        let expected = [
            (
                PersonalPronounIdentity::First,
                [
                    "азъ",
                    "мене",
                    "мьнѣ || ми",
                    "мѧ",
                    "мъноѭ",
                    "мьнѣ",
                    "вѣ",
                    "наю",
                    "нама || на",
                    "на || нꙑ",
                    "нама",
                    "наю",
                    "мꙑ",
                    "насъ",
                    "намъ || нꙑ",
                    "нꙑ",
                    "нами",
                    "насъ",
                ],
            ),
            (
                PersonalPronounIdentity::Second,
                [
                    "тꙑ",
                    "тебе",
                    "тебѣ || ти",
                    "тѧ",
                    "тобоѭ",
                    "тебѣ",
                    "ва || вꙑ",
                    "ваю",
                    "вама || ва",
                    "ва || вꙑ",
                    "вама",
                    "ваю",
                    "вꙑ",
                    "васъ",
                    "вамъ || вꙑ",
                    "вꙑ",
                    "вами",
                    "васъ",
                ],
            ),
        ];
        let cases = [
            Case::Nominative,
            Case::Genitive,
            Case::Dative,
            Case::Accusative,
            Case::Instrumental,
            Case::Locative,
        ];

        for (identity, expected_cells) in expected {
            let actual_cells = Number::ALL
                .into_iter()
                .flat_map(|number| {
                    cases.into_iter().map(move |case| {
                        personal_forms(identity, case, number, PronounFormSelection::All)
                            .iter()
                            .map(|form| form.text)
                            .collect::<Vec<_>>()
                            .join(" || ")
                    })
                })
                .collect::<Vec<_>>();
            assert_eq!(actual_cells, expected_cells, "{identity:?}");
            for number in Number::ALL {
                assert!(
                    personal_forms(identity, Case::Vocative, number, PronounFormSelection::All)
                        .is_empty()
                );
            }
        }
    }

    #[test]
    fn reflexive_is_numberless_defective_and_has_a_typed_clitic() {
        assert!(reflexive_forms(Case::Nominative, PronounFormSelection::All).is_empty());
        assert!(reflexive_forms(Case::Vocative, PronounFormSelection::All).is_empty());
        assert_eq!(
            reflexive_forms(Case::Dative, PronounFormSelection::All)
                .iter()
                .map(|form| form.text)
                .collect::<Vec<_>>(),
            ["себѣ", "си"]
        );
        assert_eq!(
            reflexive_forms(Case::Dative, PronounFormSelection::MarkedClitic)[0].status,
            PronounVariantStatus::MarkedClitic
        );
    }

    #[test]
    fn anaphoric_free_and_adprepositional_inventories_are_complete() {
        let goldens = [
            (
                AnaphoricEnvironment::Free,
                [
                    "ѥго", "ѥѩ", "ѥго", "ѥму", "ѥи", "ѥму", "и", "ѭ", "ѥ", "имь", "ѥѭ", "имь",
                    "ѥмь", "ѥи", "ѥмь", "ѥю", "ѥю", "ѥю", "има", "има", "има", "ꙗ", "и", "и",
                    "има", "има", "има", "ѥю", "ѥю", "ѥю", "ихъ", "ихъ", "ихъ", "имъ", "имъ",
                    "имъ", "ѩ", "ѩ", "ꙗ", "ими", "ими", "ими", "ихъ", "ихъ", "ихъ",
                ],
            ),
            (
                AnaphoricEnvironment::AfterPreposition,
                [
                    "н҄ѥго",
                    "н҄ѥѩ",
                    "н҄ѥго",
                    "н҄ѥму",
                    "н҄ѥи",
                    "н҄ѥму",
                    "н҄ь",
                    "н҄ѭ",
                    "н҄ѥ",
                    "н҄имь",
                    "н҄ѥѭ",
                    "н҄имь",
                    "н҄ѥмь",
                    "н҄ѥи",
                    "н҄ѥмь",
                    "н҄ѥю",
                    "н҄ѥю",
                    "н҄ѥю",
                    "н҄има",
                    "н҄има",
                    "н҄има",
                    "н҄ꙗ",
                    "н҄и",
                    "н҄и",
                    "н҄има",
                    "н҄има",
                    "н҄има",
                    "н҄ѥю",
                    "н҄ѥю",
                    "н҄ѥю",
                    "н҄ихъ",
                    "н҄ихъ",
                    "н҄ихъ",
                    "н҄имъ",
                    "н҄имъ",
                    "н҄имъ",
                    "н҄ѩ",
                    "н҄ѩ",
                    "н҄ꙗ",
                    "н҄ими",
                    "н҄ими",
                    "н҄ими",
                    "н҄ихъ",
                    "н҄ихъ",
                    "н҄ихъ",
                ],
            ),
        ];

        for (environment, expected) in goldens {
            let mut valid = 0;
            let mut actual = Vec::new();
            for number in Number::ALL {
                for case in Case::ALL {
                    for gender in Gender::ALL {
                        let form = anaphoric_form(case, number, gender, environment);
                        if matches!(case, Case::Nominative | Case::Vocative) {
                            assert!(form.is_none());
                        } else {
                            assert!(form.is_some(), "{case:?} {number:?} {gender:?}");
                            valid += 1;
                            actual.push(form.expect("valid anaphoric cell").text);
                        }
                    }
                }
            }
            assert_eq!(valid, 45);
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn source_union_aliases_are_exhaustive_and_nonoverlapping() {
        let mut aliases = Vec::new();
        for identity in PersonalPronounIdentity::ALL {
            for alias in identity.source_union_aliases() {
                assert_eq!(
                    PersonalPronounIdentity::classify_source_union_lemma(alias),
                    Some(identity)
                );
                aliases.push(*alias);
            }
        }
        aliases.sort_unstable();
        aliases.dedup();
        assert_eq!(aliases.len(), 13);
    }

    #[test]
    fn the_first_dual_dative_clitic_keeps_its_disputed_status() {
        let forms = personal_forms(
            PersonalPronounIdentity::First,
            Case::Dative,
            Number::Dual,
            PronounFormSelection::All,
        );
        assert_eq!(forms[1].text, "на");
        assert_eq!(forms[1].status, PronounVariantStatus::DisputedMarkedClitic);
    }
}
