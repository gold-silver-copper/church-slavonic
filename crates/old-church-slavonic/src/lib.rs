//! Dictionary-backed Old Church Slavonic inflection with a direct typed API.
//!
//! Ordinary calls take a lemma and the grammatical dimensions of one cell. The
//! result stays structured because a source can supply ordered spelling variants,
//! romanizations, competing analyses, warnings, and provenance.
//!
//! ## One direct cell
//!
//! ```
//! use old_church_slavonic::{noun, Case, Number};
//!
//! let forms = noun("обѣдъ", Case::Dative, Number::Dual)?;
//! assert_eq!(forms.primary_text(), "обѣдома");
//! # Ok::<(), old_church_slavonic::InflectionError>(())
//! ```
//!
//! ## Resolve once
//!
//! A handle binds one unambiguous dictionary identity. Its methods and the free
//! functions use the same canonical by-ID resolver.
//!
//! ```
//! use old_church_slavonic::{Number, Person, Verb};
//!
//! let verb = Verb::resolve("благословити")?;
//! assert_eq!(
//!     verb.present(Person::First, Number::Singular)?.primary_text(),
//!     "благословлѭ",
//! );
//! # Ok::<(), old_church_slavonic::InflectionError>(())
//! ```
//!
//! ## Keep or select variants explicitly
//!
//! [`FormSet::primary_text`] means source-first, not linguistically preferred.
//! Use [`FormSet::unique_text`] when multiplicity should be an error, or
//! [`FormSet::select`] with an explicit [`VariantPolicy`].
//!
//! ```
//! use old_church_slavonic::{aorist, Number, Person, VariantPolicy};
//!
//! let forms = aorist("бꙑти", Person::First, Number::Singular)?;
//! assert_eq!(forms.texts().collect::<Vec<_>>(), ["бѣхъ", "бꙑхъ"]);
//! assert!(forms.unique_text().is_err());
//! assert_eq!(
//!     forms.select(VariantPolicy::SourceFirst)?.text,
//!     "бѣхъ",
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Inspect contextual errors
//!
//! ```
//! use old_church_slavonic::{present, InflectionError, Number, PartOfSpeech, Person};
//!
//! let error = present("несуществовати", Person::Third, Number::Singular)
//!     .expect_err("unknown fixture");
//! assert!(matches!(
//!     error,
//!     InflectionError::UnknownLemma { ref lemma, part_of_speech: PartOfSpeech::Verb }
//!         if lemma == "несуществовати"
//! ));
//! ```
//!
//! ## Walk a paradigm
//!
//! Paradigms retain both successful and failed rows. [`ParadigmLookupError`]
//! distinguishes a cell outside a specialized inventory from a represented row
//! whose inflection failed.
//!
//! ```
//! use old_church_slavonic::{noun_paradigm, Case, Number};
//!
//! let paradigm = noun_paradigm("обѣдъ")?;
//! assert_eq!(
//!     paradigm.form(Case::Dative, Number::Dual)?.primary_text(),
//!     "обѣдома",
//! );
//! assert_eq!(paradigm.iter().count(), 21);
//! assert_eq!(paradigm.successes().count() + paradigm.failures().count(), 21);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Supply explicit out-of-vocabulary metadata
//!
//! Productive rules are deliberately under [`advanced::rules`]. The caller owns
//! the lexical class facts and the result records that provenance.
//!
//! ```
//! use old_church_slavonic::advanced::cells::NounCell;
//! use old_church_slavonic::advanced::rules::{
//!     noun_with, NounClass, NounLexeme, NumberRestriction,
//! };
//! use old_church_slavonic::{Animacy, Case, Gender, Number};
//!
//! let lexeme = NounLexeme {
//!     lemma: "роботъ".into(),
//!     class: NounClass::OMasculineHard,
//!     gender: Gender::Masculine,
//!     animacy: Animacy::Inanimate,
//!     number_restriction: NumberRestriction::All,
//! };
//! let forms = noun_with(
//!     &lexeme,
//!     NounCell { case: Case::Locative, number: Number::Plural },
//! )?;
//! assert_eq!(forms.primary_text(), "роботѣхъ");
//! # Ok::<(), old_church_slavonic::InflectionError>(())
//! ```
//!
//! ## Capability and evidence boundary
//!
//! | System | Ordinary API | Evidence behavior |
//! |---|---|---|
//! | Nouns | [`noun`], [`Noun`], [`noun_paradigm`] | Table first; dictionary metadata or explicit rules when supported |
//! | Adjectives | [`long_adjective`], [`short_adjective`], [`long_only_adjective`], [`Adjective`] | Table first; hard/soft metadata rules; exhaustive typed long-only inventory; exact comparative citations plus productive comparison through [`advanced::rules`] |
//! | Determiners | [`determiner`], [`determiner_identity`], [`determiner_paradigm`], [`Determiner`] | Exhaustive 11-identity reviewed inventory over regular `2/p`, exceptional `кꙑи`, and adjectival `2/a`; explicit OOV metadata through [`advanced::rules`] |
//! | Personal/reflexive/anaphoric pronouns | [`personal_pronoun_with`], [`reflexive_pronoun`], [`anaphoric_pronoun`] and compatible ordinary handles | Complete reviewed closed grammar tables with typed clitic/context selection |
//! | Regular class `2/p` identities | [`regular_pronominal`] and compatible adjective, pronoun, determiner, or numeral calls | All 32 regular identities have reviewed lexical ownership, hard/soft/j-stem class, aliases, and typed number restrictions; explicit OOV metadata through [`advanced::rules`] |
//! | Exceptional pronouns/determiner | [`relative_pronoun`], [`interrogative_pronoun`], [`irregular_agreeing`] and compatible ordinary handles | Complete reviewed relative, no-dual, numberless, mixed, and unique grammar tables |
//! | Derived pronominal families | [`phrases::interrogative_pronoun_family`], [`phrases::pronominal_family_with`] | Typed `ни-/нѣ-`, bound and independent postpositives, direct `-то` alternation, and preposition interposition |
//! | Cardinals through 99 | [`numeral`], [`gendered_numeral`], [`cardinal_numeral_identity`], [`cardinal_numeral_paradigm`], [`compound_cardinal`], [`compound_cardinal_paradigm`] | Reviewed simple and structured compound inventory with inherent-number agreement, typed government, correlated multiword alternatives, and source-listed versus productive variants |
//! | Other closed classes | [`pronoun`] and numeral fallbacks through [`Numeral`] | Exact pinned dictionary cells outside the reviewed productive and exceptional systems |
//! | Finite verbs | [`present`], [`imperfect`], [`aorist`], [`finite`] | Table first; independently sourced stem/formation metadata; reviewed overrides |
//! | Imperatives | [`imperative`] | Six historical person-number cells; invalid cells fail explicitly |
//! | Non-finite forms | [`infinitive`], [`supine`], [`verbal_noun`], [`l_participle`] | Table or independently supported productive rule |
//! | Participles | named participle functions and [`Participle`] | Four independently represented systems with adjective agreement |
//! | Analytic constructions | [`phrases`] | Typed tokens retain independent word-level variants and provenance |
//!
//! Open-class exact table cells take precedence over dictionary principal-part
//! rules and reviewed overrides. Reviewed closed-class identities instead own
//! their complete grammar tables; diagnostic raw access preserves copied source
//! tables. Explicit caller metadata is a separate evidence class.
//! Unsupported systems never fall back to another language or a plausible-looking
//! substitute. See [`advanced`] for stable IDs, explicit rules, dictionary metadata,
//! and diagnostic raw-feature access.

#![forbid(unsafe_code)]

pub mod advanced;
mod dictionary;
mod handles;
mod lookup;
mod metadata;
mod paradigm;
pub mod phrases;
mod resolver;

pub use handles::{Adjective, Determiner, Noun, Numeral, Participle, Pronoun, Verb};
pub use lookup::lookup;
pub use old_church_slavonic_core::{
    AdjectiveForm, AnalyticConstruction, AnaphoricEnvironment, Animacy, CardinalNumeralIdentity,
    CardinalPhraseAnalysis, Case, CompoundCardinalCell, ConditionalAuxiliary, CopulaSeries,
    DeterminerCell, DeterminerDeclension, DeterminerIdentity, DeterminerLexeme, DirectToTreatment,
    FiniteTense, FormSet, FormSource, FormVariant, FutureInfinitiveAuxiliary, FutureReferenceTense,
    Gender, GenderedCell, InflectionError, InflectionWarning, InterrogativePronounIdentity,
    IrregularAgreeingIdentity, Lemma, LexemeSummary, LongOnlyAdjectiveIdentity, Number,
    NumeralCell, NumeralGovernment, PartOfSpeech, ParticipleKind, PassiveAuxiliary, Person,
    PersonalPronounCell, PersonalPronounIdentity, PhraseOrder, PhraseRole, PhraseToken,
    PluperfectAuxiliary, PronominalFamilySpec, PronominalPostpositive, PronominalPrefix,
    PronounFormSelection, RealizedCardinal, RealizedPhrase, RequestedCell, Script,
    StandardPronominalIdentity, UngenderedCell, VariantPolicy, VariantSelectionError,
};
pub use paradigm::{
    AdjectiveParadigm, CardinalNumeralParadigm, CellOutcome, ClosedClassParadigm,
    ComparativeParadigm, CompoundCardinalOutcome, CompoundCardinalParadigm, DeterminerParadigm,
    FiniteVerbParadigm, GenderedNumeralParadigm, GenderedPronounParadigm, ImperativeParadigm,
    LParticipleParadigm, NounParadigm, NumeralParadigm, ParadigmLookupError, ParticipleParadigm,
    PersonalPronounParadigm, PronounParadigm, VerbParadigm,
};

/// Rule traces and source-evidence diagnostics.
pub mod trace {
    pub use old_church_slavonic_core::{
        FormAnalysis, MetadataEvidence, MetadataField, MetadataProvenance, RuleId, RuleStep,
    };
}

/// Common ordinary morphology imports. Specialist metadata and raw features are
/// intentionally excluded; use [`advanced`] for those APIs.
pub mod prelude {
    pub use crate::{
        Adjective, AdjectiveForm, AdjectiveParadigm, AnaphoricEnvironment, Animacy,
        CardinalNumeralIdentity, CardinalNumeralParadigm, Case, CompoundCardinalParadigm,
        Determiner, DeterminerCell, DeterminerIdentity, DeterminerParadigm, FiniteTense,
        FiniteVerbParadigm, FormSet, FormSource, FormVariant, Gender, GenderedNumeralParadigm,
        GenderedPronounParadigm, ImperativeParadigm, InflectionError, InflectionResult,
        InflectionWarning, InterrogativePronounIdentity, IrregularAgreeingIdentity,
        LParticipleParadigm, Lemma, LongOnlyAdjectiveIdentity, Noun, NounParadigm, Number, Numeral,
        NumeralParadigm, ParadigmLookupError, PartOfSpeech, Participle, ParticipleKind,
        ParticipleParadigm, Person, PersonalPronounIdentity, PersonalPronounParadigm, Pronoun,
        PronounFormSelection, PronounParadigm, Script, StandardPronominalIdentity, VariantPolicy,
        VariantSelectionError, Verb, VerbParadigm, adjective_paradigm, anaphoric_pronoun, aorist,
        cardinal_numeral_identity, cardinal_numeral_paradigm, comparative_citation,
        compound_cardinal, compound_cardinal_paradigm, compound_cardinal_paradigm_with_one,
        compound_cardinal_with_one, determiner, determiner_identity, determiner_paradigm, finite,
        finite_paradigm, gendered_numeral, gendered_numeral_paradigm, gendered_pronoun,
        gendered_pronoun_paradigm, imperative, imperative_paradigm, imperfect, infinitive,
        interrogative_pronoun, irregular_agreeing, l_participle, l_participle_paradigm,
        long_adjective, long_only_adjective, lookup, noun, noun_paradigm, numeral,
        numeral_paradigm, participle_paradigm, past_active_participle, past_passive_participle,
        personal_pronoun, personal_pronoun_paradigm, personal_pronoun_with, present,
        present_active_participle, present_paradigm, present_passive_participle, pronoun,
        pronoun_paradigm, reflexive_pronoun, regular_pronominal, relative_pronoun, short_adjective,
        supine, verbal_noun,
    };
}

/// The structured result returned by ordinary inflection calls.
pub type InflectionResult = Result<FormSet, InflectionError>;

/// Decline one dictionary noun cell.
///
/// ```
/// use old_church_slavonic::{noun, Case, Number};
/// assert_eq!(noun("обѣдъ", Case::Dative, Number::Dual)?.primary_text(), "обѣдома");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn noun(lemma: &str, case: Case, number: Number) -> InflectionResult {
    resolver::noun(lemma, old_church_slavonic_core::NounCell { case, number })
}

/// Decline one long/compound adjective cell.
///
/// ```
/// use old_church_slavonic::{long_adjective, Animacy, Case, Gender, Number};
/// assert_eq!(
///     long_adjective(
///         "добръ", Case::Nominative, Number::Singular,
///         Gender::Masculine, Animacy::Inanimate,
///     )?.primary_text(),
///     "добрꙑи",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn long_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    adjective_form(lemma, AdjectiveForm::Long, case, number, gender, animacy)
}

/// Decline one of the three source-listed adjectives that has only long forms.
///
/// This typed entry point carries the lexical defectivity explicitly. The
/// ordinary [`long_adjective`] call recognizes the same canonical lemmas and
/// source-spelling aliases.
///
/// ```
/// use old_church_slavonic::{
///     Animacy, Case, Gender, LongOnlyAdjectiveIdentity, Number,
///     long_only_adjective,
/// };
/// assert_eq!(
///     long_only_adjective(
///         LongOnlyAdjectiveIdentity::OtherProchii,
///         Case::Nominative,
///         Number::Singular,
///         Gender::Neuter,
///         Animacy::Inanimate,
///     )?
///     .primary_text(),
///     "прочеѥ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn long_only_adjective(
    identity: LongOnlyAdjectiveIdentity,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    resolver::long_only_adjective(
        identity,
        old_church_slavonic_core::AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form: AdjectiveForm::Long,
        },
    )
}

/// Decline one source-reviewed determiner cell by lemma.
///
/// The exhaustive inventory spans regular pronominal, exceptional `кꙑи`,
/// short adjectival `ѥтеръ`, and long-only adjectival `которꙑи` profiles.
///
/// ```
/// use old_church_slavonic::{determiner, Animacy, Case, Gender, Number};
/// assert_eq!(
///     determiner(
///         "кꙑи", Case::Accusative, Number::Singular, Gender::Feminine,
///         Animacy::Inanimate,
///     )?
///         .primary_text(),
///     "кѫѭ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn determiner(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    resolver::determiner(
        lemma,
        DeterminerCell {
            case,
            number,
            gender,
            animacy,
        },
    )
}

/// Decline one determiner by its stable reviewed grammatical identity.
///
/// ```
/// use old_church_slavonic::{
///     determiner_identity, Animacy, Case, DeterminerIdentity, Gender, Number,
/// };
/// assert_eq!(
///     determiner_identity(
///         DeterminerIdentity::DemonstrativeMannerTak,
///         Case::Nominative,
///         Number::Plural,
///         Gender::Masculine,
///         Animacy::Inanimate,
///     )?
///     .primary_text(),
///     "таци",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn determiner_identity(
    identity: DeterminerIdentity,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    resolver::reviewed_determiner(
        identity,
        DeterminerCell {
            case,
            number,
            gender,
            animacy,
        },
    )
}

/// Decline an case-number-only dictionary pronoun cell.
///
/// ```
/// use old_church_slavonic::{pronoun, Case, Number};
/// assert_eq!(pronoun("сѧ", Case::Genitive, Number::Singular)?.primary_text(), "себе");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn pronoun(lemma: &str, case: Case, number: Number) -> InflectionResult {
    resolver::closed_class(
        lemma,
        PartOfSpeech::Pronoun,
        UngenderedCell { case, number }.closed_class(),
    )
}

/// Decline one person-indexed personal-pronoun cell.
///
/// ```
/// use old_church_slavonic::{personal_pronoun, Case, Number, Person};
/// assert_eq!(
///     personal_pronoun("азъ", Case::Dative, Number::Singular, Person::First)?
///         .primary_text(),
///     "мьнѣ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn personal_pronoun(
    lemma: &str,
    case: Case,
    number: Number,
    person: Person,
) -> InflectionResult {
    resolver::closed_class(
        lemma,
        PartOfSpeech::Pronoun,
        PersonalPronounCell {
            case,
            number,
            person,
        }
        .closed_class(),
    )
}

/// Resolve the canonical first- or second-person paradigm with an intrinsic
/// lexical person and an explicit table-primary/marked-clitic selection.
///
/// ```
/// use old_church_slavonic::{
///     Case, Number, PersonalPronounIdentity, PronounFormSelection,
///     personal_pronoun_with,
/// };
///
/// let forms = personal_pronoun_with(
///     PersonalPronounIdentity::First,
///     Case::Dative,
///     Number::Singular,
///     PronounFormSelection::All,
/// )?;
/// assert_eq!(forms.texts().collect::<Vec<_>>(), ["мьнѣ", "ми"]);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn personal_pronoun_with(
    identity: PersonalPronounIdentity,
    case: Case,
    number: Number,
    selection: PronounFormSelection,
) -> InflectionResult {
    resolver::personal_pronoun_with(identity, case, number, selection)
}

/// Resolve the numberless reflexive pronoun with an explicit
/// table-primary/marked-clitic
/// selection. Nominative and vocative requests are historically invalid.
///
/// ```
/// use old_church_slavonic::{Case, PronounFormSelection, reflexive_pronoun};
///
/// let forms = reflexive_pronoun(Case::Dative, PronounFormSelection::MarkedClitic)?;
/// assert_eq!(forms.primary_text(), "си");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn reflexive_pronoun(case: Case, selection: PronounFormSelection) -> InflectionResult {
    resolver::reflexive_pronoun(case, selection)
}

/// Resolve the defective third-person anaphoric pronoun, including its
/// obligatorily conditioned post-prepositional `н҄-` series.
///
/// ```
/// use old_church_slavonic::{
///     AnaphoricEnvironment, Case, Gender, Number, anaphoric_pronoun,
/// };
///
/// let form = anaphoric_pronoun(
///     Case::Accusative,
///     Number::Singular,
///     Gender::Masculine,
///     AnaphoricEnvironment::AfterPreposition,
/// )?;
/// assert_eq!(form.primary_text(), "н҄ь");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn anaphoric_pronoun(
    case: Case,
    number: Number,
    gender: Gender,
    environment: AnaphoricEnvironment,
) -> InflectionResult {
    resolver::anaphoric_pronoun(case, number, gender, environment)
}

/// Resolve the complete relative pronoun `иже` in a free or post-prepositional
/// environment.
///
/// ```
/// use old_church_slavonic::{
///     AnaphoricEnvironment, Case, Gender, Number, relative_pronoun,
/// };
/// let form = relative_pronoun(
///     Case::Dative,
///     Number::Singular,
///     Gender::Feminine,
///     AnaphoricEnvironment::AfterPreposition,
/// )?;
/// assert_eq!(form.primary_text(), "н҄ѥиже");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn relative_pronoun(
    case: Case,
    number: Number,
    gender: Gender,
    environment: AnaphoricEnvironment,
) -> InflectionResult {
    resolver::relative_pronoun(case, number, gender, environment)
}

/// Resolve one agreeing closed irregular identity. This covers no-dual
/// `вьсь/сиць`, unique `сь`, and the syncopated/expanded determiner `кꙑи`.
///
/// ```
/// use old_church_slavonic::{
///     Case, Gender, IrregularAgreeingIdentity, Number, irregular_agreeing,
/// };
/// let forms = irregular_agreeing(
///     IrregularAgreeingIdentity::TotalVes,
///     Case::Nominative,
///     Number::Singular,
///     Gender::Feminine,
/// )?;
/// assert_eq!(forms.texts().collect::<Vec<_>>(), ["вьса", "вьсѣ"]);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn irregular_agreeing(
    identity: IrregularAgreeingIdentity,
    case: Case,
    number: Number,
    gender: Gender,
) -> InflectionResult {
    resolver::irregular_agreeing(identity, case, number, gender)
}

/// Resolve the numberless and genderless interrogative `къто` or `чьто`.
///
/// ```
/// use old_church_slavonic::{
///     Case, InterrogativePronounIdentity, interrogative_pronoun,
/// };
/// let forms = interrogative_pronoun(InterrogativePronounIdentity::Chto, Case::Genitive)?;
/// assert_eq!(
///     forms.texts().collect::<Vec<_>>(),
///     ["чесо", "чьсо", "чесого"],
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn interrogative_pronoun(
    identity: InterrogativePronounIdentity,
    case: Case,
) -> InflectionResult {
    resolver::interrogative_pronoun(identity, case)
}

/// Decline one gender-indexed pronoun cell.
///
/// ```
/// use old_church_slavonic::{gendered_pronoun, Case, Gender, Number};
/// assert_eq!(
///     gendered_pronoun("онъ", Case::Dative, Number::Singular, Gender::Feminine)?
///         .primary_text(),
///     "онои",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn gendered_pronoun(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> InflectionResult {
    resolver::closed_class(
        lemma,
        PartOfSpeech::Pronoun,
        GenderedCell {
            case,
            number,
            gender,
        }
        .closed_class(),
    )
}

/// Decline any reviewed regular identity in Polivanova's class `2/p`.
///
/// The identity carries its reviewed morphological class, primary part of
/// speech, and any number restriction, so this also covers class members that
/// are absent from the bundled dictionary.
///
/// ```
/// use old_church_slavonic::{
///     Case, Gender, Number, StandardPronominalIdentity, regular_pronominal,
/// };
/// assert_eq!(
///     regular_pronominal(
///         StandardPronominalIdentity::NumeralDva,
///         Case::Nominative,
///         Number::Dual,
///         Gender::Feminine,
///     )?.primary_text(),
///     "дъвѣ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn regular_pronominal(
    identity: StandardPronominalIdentity,
    case: Case,
    number: Number,
    gender: Gender,
) -> InflectionResult {
    resolver::regular_pronominal(identity, case, number, gender)
}

/// Decline an case-number-only dictionary numeral cell.
///
/// ```
/// use old_church_slavonic::{numeral, Case, Number};
/// assert_eq!(numeral("девѧть", Case::Genitive, Number::Singular)?.primary_text(), "девѧти");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn numeral(lemma: &str, case: Case, number: Number) -> InflectionResult {
    resolver::numeral(
        lemma,
        NumeralCell {
            case,
            number,
            gender: None,
        },
    )
}

/// Decline a gender-indexed dictionary numeral cell.
///
/// ```
/// use old_church_slavonic::{gendered_numeral, Case, Gender, Number};
/// let forms = gendered_numeral("прьвъ", Case::Nominative, Number::Singular, Gender::Feminine)?;
/// assert_eq!(forms.primary_text(), "прьва");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn gendered_numeral(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> InflectionResult {
    resolver::numeral(
        lemma,
        NumeralCell {
            case,
            number,
            gender: Some(gender),
        },
    )
}

/// Decline one simple cardinal by its stable reviewed grammatical identity.
///
/// Gender is required for agreeing cardinals (`one` through `four`) and absent
/// for substantival cardinals (`five` through `ten`). Invalid combinations are
/// reported as [`InflectionError::HistoricallyInvalidCell`].
///
/// ```
/// use old_church_slavonic::{
///     cardinal_numeral_identity, CardinalNumeralIdentity, Case, Gender, Number,
/// };
/// assert_eq!(
///     cardinal_numeral_identity(
///         CardinalNumeralIdentity::Three,
///         Case::Instrumental,
///         Number::Plural,
///         Some(Gender::Feminine),
///     )?
///     .primary_text(),
///     "трьми",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn cardinal_numeral_identity(
    identity: CardinalNumeralIdentity,
    case: Case,
    number: Number,
    gender: Option<Gender>,
) -> InflectionResult {
    resolver::reviewed_cardinal_numeral(
        identity,
        NumeralCell {
            case,
            number,
            gender,
        },
    )
}

/// Inflect a composed cardinal from 11 through 99.
///
/// Gender follows the final unit: it is required when that unit is one through
/// four and absent when the compound governs a genitive plural. This default
/// route uses `ѥдинъ`; use [`compound_cardinal_with_one`] to select its lexical
/// doublet `ѥдьнъ`.
///
/// ```
/// use old_church_slavonic::{compound_cardinal, Case, Gender};
/// assert_eq!(
///     compound_cardinal(12, Case::Genitive, Some(Gender::Masculine))?.primary_text(),
///     "дъвою на десѧте",
/// );
/// assert_eq!(
///     compound_cardinal(50, Case::Genitive, None)?.primary_text(),
///     "пѧти десѧтъ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn compound_cardinal(
    value: u16,
    case: Case,
    gender: Option<Gender>,
) -> Result<RealizedCardinal, InflectionError> {
    compound_cardinal_with_one(value, CardinalNumeralIdentity::OneYedin, case, gender)
}

/// Inflect a composed cardinal while explicitly selecting the lexical doublet
/// used wherever the value contains a final or teen component one.
///
/// ```
/// use old_church_slavonic::{
///     compound_cardinal_with_one, CardinalNumeralIdentity, Case, Gender,
/// };
/// assert_eq!(
///     compound_cardinal_with_one(
///         21,
///         CardinalNumeralIdentity::OneYedyn,
///         Case::Dative,
///         Some(Gender::Feminine),
///     )?
///     .primary_text(),
///     "дъвѣма десѧтьма и ѥдьнои",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn compound_cardinal_with_one(
    value: u16,
    one_identity: CardinalNumeralIdentity,
    case: Case,
    gender: Option<Gender>,
) -> Result<RealizedCardinal, InflectionError> {
    resolver::compound_cardinal(value, CompoundCardinalCell { case, gender }, one_identity)
}

/// Enumerate every case and optional-gender request for a composed cardinal
/// from 11 through 99, retaining invalid shapes as typed failures.
///
/// ```
/// use old_church_slavonic::{compound_cardinal_paradigm, Case, Gender};
/// let paradigm = compound_cardinal_paradigm(53)?;
/// assert_eq!(paradigm.len(), 28);
/// assert_eq!(
///     paradigm.form(Case::Nominative, Some(Gender::Masculine))?.primary_text(),
///     "пѧть десѧтъ и триѥ",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compound_cardinal_paradigm(value: u16) -> Result<CompoundCardinalParadigm, InflectionError> {
    compound_cardinal_paradigm_with_one(value, CardinalNumeralIdentity::OneYedin)
}

/// Enumerate a composed cardinal while explicitly selecting its lexical
/// doublet of one.
///
/// ```
/// use old_church_slavonic::{
///     compound_cardinal_paradigm_with_one, CardinalNumeralIdentity, Case, Gender,
/// };
/// let paradigm = compound_cardinal_paradigm_with_one(
///     91,
///     CardinalNumeralIdentity::OneYedyn,
/// )?;
/// assert_eq!(
///     paradigm.form(Case::Dative, Some(Gender::Feminine))?.primary_text(),
///     "девѧти десѧтъ и ѥдьнои",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compound_cardinal_paradigm_with_one(
    value: u16,
    one_identity: CardinalNumeralIdentity,
) -> Result<CompoundCardinalParadigm, InflectionError> {
    resolver::build_compound_cardinal_paradigm(value, one_identity)
}

/// Decline one short/simple adjective cell.
///
/// ```
/// use old_church_slavonic::{short_adjective, Animacy, Case, Gender, Number};
/// assert_eq!(
///     short_adjective(
///         "добръ", Case::Nominative, Number::Singular,
///         Gender::Masculine, Animacy::Inanimate,
///     )?.primary_text(),
///     "добръ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn short_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    adjective_form(lemma, AdjectiveForm::Short, case, number, gender, animacy)
}

fn adjective_form(
    lemma: &str,
    form: AdjectiveForm,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    resolver::adjective(
        lemma,
        old_church_slavonic_core::AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
        },
    )
}

/// Inflect a present-indicative verb cell.
///
/// ```
/// use old_church_slavonic::{present, Number, Person};
/// assert_eq!(
///     present("благословити", Person::First, Number::Singular)?.primary_text(),
///     "благословлѭ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn present(lemma: &str, person: Person, number: Number) -> InflectionResult {
    finite(lemma, FiniteTense::Present, person, number)
}

/// Inflect one imperfect person-number cell.
///
/// ```
/// use old_church_slavonic::{imperfect, Number, Person};
/// let forms = imperfect("бꙑти", Person::First, Number::Singular)?;
/// assert!(!forms.primary_text().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn imperfect(lemma: &str, person: Person, number: Number) -> InflectionResult {
    finite(lemma, FiniteTense::Imperfect, person, number)
}

/// Inflect one aorist person-number cell.
///
/// ```
/// use old_church_slavonic::{aorist, Number, Person};
/// let forms = aorist("бꙑти", Person::First, Number::Singular)?;
/// assert_eq!(forms.texts().collect::<Vec<_>>(), ["бѣхъ", "бꙑхъ"]);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn aorist(lemma: &str, person: Person, number: Number) -> InflectionResult {
    finite(lemma, FiniteTense::Aorist, person, number)
}

/// Inflect one cell in an explicitly selected synthetic finite tense.
///
/// ```
/// use old_church_slavonic::{finite, FiniteTense, Number, Person};
/// let forms = finite(
///     "благословити", FiniteTense::Present, Person::First, Number::Singular,
/// )?;
/// assert_eq!(forms.primary_text(), "благословлѭ");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn finite(lemma: &str, tense: FiniteTense, person: Person, number: Number) -> InflectionResult {
    resolver::finite_verb(
        lemma,
        old_church_slavonic_core::FiniteVerbCell {
            tense,
            person,
            number,
        },
    )
}

/// Inflect one historically represented imperative cell.
///
/// ```
/// use old_church_slavonic::{imperative, Number, Person};
/// assert_eq!(
///     imperative("благословити", Person::Second, Number::Singular)?.primary_text(),
///     "благослови",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn imperative(lemma: &str, person: Person, number: Number) -> InflectionResult {
    resolver::imperative(
        lemma,
        old_church_slavonic_core::ImperativeCell { person, number },
    )
}

/// Inflect one gender-number l-participle cell.
///
/// ```
/// use old_church_slavonic::{l_participle, Gender, Number};
/// assert_eq!(
///     l_participle("благословити", Gender::Feminine, Number::Dual)?.primary_text(),
///     "благословилѣ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn l_participle(lemma: &str, gender: Gender, number: Number) -> InflectionResult {
    resolver::l_participle(
        lemma,
        old_church_slavonic_core::LParticipleCell { gender, number },
    )
}

/// Return the dictionary infinitive.
///
/// ```
/// assert_eq!(
///     old_church_slavonic::infinitive("благословити")?.primary_text(),
///     "благословити",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn infinitive(lemma: &str) -> InflectionResult {
    resolver::infinitive(lemma)
}

/// Return a dictionary or safely generated supine.
///
/// ```
/// assert_eq!(old_church_slavonic::supine("бости")?.primary_text(), "бостъ");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn supine(lemma: &str) -> InflectionResult {
    resolver::supine(lemma)
}

/// Return a dictionary-listed verbal noun.
///
/// ```
/// let forms = old_church_slavonic::verbal_noun("благословити")?;
/// assert_eq!(forms.primary_text(), "благословлѥниѥ");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn verbal_noun(lemma: &str) -> InflectionResult {
    resolver::verbal_noun(lemma)
}

/// Return dictionary-listed comparative citations.
///
/// ```
/// let result = old_church_slavonic::comparative_citation("добръ");
/// assert!(result.is_ok() || matches!(
///     result,
///     Err(old_church_slavonic::InflectionError::UnsupportedCell { .. })
/// ));
/// ```
pub fn comparative_citation(lemma: &str) -> InflectionResult {
    resolver::adjective_comparatives(lemma)
}

/// Bind the present active participle system of one verb.
///
/// ```
/// let participle = old_church_slavonic::present_active_participle("благословити")?;
/// assert!(!participle.citation()?.primary_text().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn present_active_participle(lemma: &str) -> Result<Participle, InflectionError> {
    Verb::resolve(lemma)?.present_active_participle()
}

/// Bind the present passive participle system of one verb.
///
/// ```
/// let participle = old_church_slavonic::present_passive_participle("благословити")?;
/// assert!(!participle.citation()?.primary_text().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn present_passive_participle(lemma: &str) -> Result<Participle, InflectionError> {
    Verb::resolve(lemma)?.present_passive_participle()
}

/// Bind the past active participle system of one verb.
///
/// ```
/// let participle = old_church_slavonic::past_active_participle("благословити")?;
/// assert_eq!(participle.citation()?.texts().count(), 2);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn past_active_participle(lemma: &str) -> Result<Participle, InflectionError> {
    Verb::resolve(lemma)?.past_active_participle()
}

/// Bind the past passive participle system of one verb.
///
/// ```
/// let participle = old_church_slavonic::past_passive_participle("благословити")?;
/// assert!(!participle.citation()?.primary_text().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn past_passive_participle(lemma: &str) -> Result<Participle, InflectionError> {
    Verb::resolve(lemma)?.past_passive_participle()
}

/// Enumerate all noun cells after resolving one dictionary lemma.
///
/// ```
/// let paradigm = old_church_slavonic::noun_paradigm("обѣдъ")?;
/// assert_eq!(paradigm.len(), 21);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn noun_paradigm(lemma: &str) -> Result<NounParadigm, InflectionError> {
    Ok(Noun::resolve(lemma)?.paradigm())
}

/// Enumerate long and short adjective cells after resolving one lemma.
///
/// ```
/// let paradigm = old_church_slavonic::adjective_paradigm("добръ")?;
/// assert!(!paradigm.is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn adjective_paradigm(lemma: &str) -> Result<AdjectiveParadigm, InflectionError> {
    Ok(Adjective::resolve(lemma)?.paradigm())
}

/// Enumerate the complete source-reviewed determiner inventory.
///
/// ```
/// let paradigm = old_church_slavonic::determiner_paradigm("кꙑи")?;
/// assert!(!paradigm.successes().collect::<Vec<_>>().is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn determiner_paradigm(lemma: &str) -> Result<DeterminerParadigm, InflectionError> {
    resolver::determiner_paradigm(lemma)
}

/// Enumerate case-number-only pronoun cells, retaining unsupported rows.
///
/// ```
/// use old_church_slavonic::{pronoun_paradigm, Case, Number};
/// let paradigm = pronoun_paradigm("сѧ")?;
/// assert_eq!(paradigm.form(Case::Genitive, Number::Singular)?.primary_text(), "себе");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn pronoun_paradigm(lemma: &str) -> Result<PronounParadigm, InflectionError> {
    Ok(Pronoun::resolve(lemma)?.paradigm())
}

/// Enumerate person-indexed pronoun cells, retaining unsupported rows.
///
/// ```
/// use old_church_slavonic::{personal_pronoun_paradigm, Case, Number, Person};
/// let paradigm = personal_pronoun_paradigm("азъ")?;
/// assert_eq!(
///     paradigm.form(Case::Dative, Number::Singular, Person::First)?.primary_text(),
///     "мьнѣ",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn personal_pronoun_paradigm(lemma: &str) -> Result<PersonalPronounParadigm, InflectionError> {
    Ok(Pronoun::resolve(lemma)?.personal_paradigm())
}

/// Enumerate gender-indexed pronoun cells, retaining unsupported rows.
///
/// ```
/// use old_church_slavonic::{gendered_pronoun_paradigm, Case, Gender, Number};
/// let paradigm = gendered_pronoun_paradigm("онъ")?;
/// assert_eq!(
///     paradigm.form(Case::Dative, Number::Singular, Gender::Feminine)?.primary_text(),
///     "онои",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn gendered_pronoun_paradigm(lemma: &str) -> Result<GenderedPronounParadigm, InflectionError> {
    Ok(Pronoun::resolve(lemma)?.gendered_paradigm())
}

/// Enumerate case-number-only numeral cells, retaining unsupported rows.
///
/// ```
/// use old_church_slavonic::{numeral_paradigm, Case, Number};
/// let paradigm = numeral_paradigm("девѧть")?;
/// assert_eq!(paradigm.form(Case::Genitive, Number::Singular)?.primary_text(), "девѧти");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn numeral_paradigm(lemma: &str) -> Result<NumeralParadigm, InflectionError> {
    Ok(Numeral::resolve(lemma)?.paradigm())
}

/// Enumerate all typed cells for one reviewed simple cardinal.
///
/// ```
/// use old_church_slavonic::{
///     cardinal_numeral_paradigm, CardinalNumeralIdentity, Case, Number,
/// };
/// let paradigm = cardinal_numeral_paradigm(CardinalNumeralIdentity::Ten);
/// assert_eq!(paradigm.len(), 84);
/// assert_eq!(
///     paradigm.form(Case::Nominative, Number::Plural, None)?.primary_text(),
///     "десѧте",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn cardinal_numeral_paradigm(identity: CardinalNumeralIdentity) -> CardinalNumeralParadigm {
    resolver::build_cardinal_numeral_paradigm(identity)
}

/// Enumerate gender-indexed numeral cells, retaining unsupported rows.
///
/// ```
/// use old_church_slavonic::{gendered_numeral_paradigm, Case, Gender, Number};
/// let paradigm = gendered_numeral_paradigm("прьвъ")?;
/// assert_eq!(
///     paradigm.form(Case::Nominative, Number::Singular, Gender::Feminine)?.primary_text(),
///     "прьва",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn gendered_numeral_paradigm(lemma: &str) -> Result<GenderedNumeralParadigm, InflectionError> {
    Ok(Numeral::resolve(lemma)?.gendered_paradigm())
}

/// Enumerate the nine present-indicative person-number cells.
///
/// ```
/// let paradigm = old_church_slavonic::present_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 9);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn present_paradigm(lemma: &str) -> Result<VerbParadigm, InflectionError> {
    Ok(Verb::resolve(lemma)?.present_paradigm())
}

/// Enumerate present, imperfect, and aorist cells after resolving one lemma.
///
/// ```
/// let paradigm = old_church_slavonic::finite_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 27);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn finite_paradigm(lemma: &str) -> Result<FiniteVerbParadigm, InflectionError> {
    Ok(Verb::resolve(lemma)?.finite_paradigm())
}

/// Enumerate the six historically represented imperative cells.
///
/// ```
/// let paradigm = old_church_slavonic::imperative_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 6);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn imperative_paradigm(lemma: &str) -> Result<ImperativeParadigm, InflectionError> {
    Ok(Verb::resolve(lemma)?.imperative_paradigm())
}

/// Enumerate every gender-number l-participle cell.
///
/// ```
/// let paradigm = old_church_slavonic::l_participle_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 9);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn l_participle_paradigm(lemma: &str) -> Result<LParticipleParadigm, InflectionError> {
    Ok(Verb::resolve(lemma)?.l_participle_paradigm())
}

/// Enumerate both agreement paradigms for one declined participle system.
///
/// ```
/// use old_church_slavonic::{participle_paradigm, ParticipleKind};
/// let paradigm = participle_paradigm("благословити", ParticipleKind::PastActive)?;
/// assert_eq!(paradigm.kind(), ParticipleKind::PastActive);
/// assert!(!paradigm.is_empty());
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn participle_paradigm(
    lemma: &str,
    kind: ParticipleKind,
) -> Result<ParticipleParadigm, InflectionError> {
    Ok(Verb::resolve(lemma)?.participle(kind)?.paradigm())
}
