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
//! | Cardinals and distributives through 10,000, ordinals 1–1,000, collectives 2–10, fractional nouns, and indefinite quantities | [`numeral`], [`gendered_numeral`], [`cardinal_numeral_identity`], [`cardinal_magnitude`], [`ordinal_numeral`], [`ordinal_numeral_identity`], [`ordinal_numeral_paradigm`], [`compound_ordinal`], [`compound_ordinal_paradigm`], [`collective_numeral`], [`collective_numeral_identity`], [`collective_numeral_paradigm`], [`fractional_numeral`], [`fractional_numeral_identity`], [`fractional_numeral_paradigm`], [`indefinite_numeral`], [`indefinite_numeral_identity`], [`indefinite_numeral_paradigm`], [`compound_cardinal`], [`compound_cardinal_paradigm`], [`distributive_cardinal`], [`distributive_cardinal_paradigm`] | Reviewed simple, magnitude, structured compound and `по` + dative distributive, adjective-agreement ordinal, inherited collective, fractional-noun, and non-exact quantity inventories with typed government, correlated multiword alternatives, historical declension classes, and explicit source-listed, productive, reconstructed, disputed, corpus, and primary-text evidence |
//! | Other closed classes | [`pronoun`] and numeral fallbacks through [`Numeral`] | Exact pinned dictionary cells outside the reviewed productive and exceptional systems |
//! | Finite verbs | [`present`], [`imperfect`], [`aorist`], [`finite`] | Exact table, manual override, closed reviewed irregular profile, then independently sourced stem/formation metadata; competing profiles remain separate analyses |
//! | Imperatives | [`imperative`] | Six historical person-number cells; invalid cells fail explicitly |
//! | Non-finite and derived forms | [`infinitive`], [`supine`], [`l_participle`]; [`verbal_noun`], [`verbal_noun_form`], [`verbal_noun_paradigm`] | Table or independently supported productive rule; verbal nouns are complete derived soft-neuter paradigms |
//! | Participles | named participle functions and [`Participle`] | Four independently represented systems with adjective agreement |
//! | Analytic constructions | [`phrases`] | Typed tokens retain independent word-level variants and provenance |
//!
//! Open-class exact table cells take precedence over reviewed manual overrides,
//! closed irregular profiles, and dictionary principal-part rules, in that order.
//! Reviewed-only irregular verb spellings have stable lexical IDs and retain
//! direct versus predictive evidence for every cell. Reviewed closed-class
//! identities instead own their complete grammar tables; diagnostic raw access
//! preserves copied source tables. Explicit caller metadata is a separate evidence class.
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
    AccentEvidence, AccentParadigm, AccentPlacement, AccentReconstructionStatus, AccentRule,
    AccentScope, AdjectiveForm, AnalyticConstruction, AnaphoricEnvironment, Animacy,
    CardinalCompositionOptions, CardinalMagnitudeIdentity, CardinalNumeralIdentity,
    CardinalPhraseAnalysis, Case, CollectiveNumeralCell, CollectiveNumeralDeclension,
    CollectiveNumeralIdentity, CompoundCardinalCell, ConditionalAuxiliary, CopulaSeries,
    DeterminerCell, DeterminerDeclension, DeterminerIdentity, DeterminerLexeme, DirectToTreatment,
    DistributiveCardinalAnalysis, DistributiveCardinalCell, FiniteTense, FormSet, FormSource,
    FormVariant, FractionalNumeralDeclension, FractionalNumeralIdentity, FutureInfinitiveAuxiliary,
    FutureReferenceTense, Gender, GenderedCell, GlagoliticProfile, ImpersonalVerbIdentity,
    ImpersonalVerbStatus, IndefiniteNumeralIdentity, InflectionError, InflectionWarning,
    InterrogativePronounIdentity, IrregularAgreeingIdentity, IrregularVerbAnalysis,
    IrregularVerbFamilyMember, IrregularVerbGroup, Lemma, LexemeSummary, LongOnlyAdjectiveIdentity,
    MAX_COMPOUND_ORDINAL_VALUE, MIN_COMPOUND_ORDINAL_VALUE, Number, NumeralCell, NumeralGovernment,
    OrdinalComposition, OrdinalNumeralIdentity, OrdinalPhraseAnalysis, PartOfSpeech,
    ParticipleKind, PassiveAuxiliary, Person, PersonalPronounCell, PersonalPronounIdentity,
    PhraseOrder, PhraseRole, PhraseToken, PluperfectAuxiliary, PronominalFamilySpec,
    PronominalPostpositive, PronominalPrefix, PronounFormSelection, RealizedCardinal,
    RealizedDistributiveCardinal, RealizedOrdinal, RealizedPhrase, ReconstructedAccent,
    RequestedCell, Script, StandardPronominalIdentity, TransliteratedForm,
    TransliterationDirection, TransliterationFidelity, TransliterationLoss,
    TransliterationLossKind, TransliterationLossPolicy, TwofoldNounFamilyMember, UngenderedCell,
    UniqueVerbFamilyMember, UniqueVerbIdentity, UniqueVerbProfileKind, VariantPolicy,
    VariantSelectionError,
};
pub use paradigm::{
    AdjectiveParadigm, CardinalNumeralParadigm, CellOutcome, ClosedClassParadigm,
    CollectiveNumeralParadigm, ComparativeParadigm, CompoundCardinalOutcome,
    CompoundCardinalParadigm, CompoundOrdinalOutcome, CompoundOrdinalParadigm, DeterminerParadigm,
    DistributiveCardinalOutcome, DistributiveCardinalParadigm, FiniteVerbParadigm,
    FractionalNumeralParadigm, GenderedNumeralParadigm, GenderedPronounParadigm,
    ImperativeParadigm, IndefiniteNumeralParadigm, LParticipleParadigm, NounParadigm,
    NumeralParadigm, OrdinalNumeralParadigm, ParadigmLookupError, ParticipleParadigm,
    PersonalPronounParadigm, PronounParadigm, VerbParadigm, VerbalNounParadigm,
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
        CardinalCompositionOptions, CardinalMagnitudeIdentity, CardinalNumeralIdentity,
        CardinalNumeralParadigm, Case, CollectiveNumeralCell, CollectiveNumeralDeclension,
        CollectiveNumeralIdentity, CollectiveNumeralParadigm, CompoundCardinalParadigm,
        CompoundOrdinalParadigm, Determiner, DeterminerCell, DeterminerIdentity,
        DeterminerParadigm, DistributiveCardinalCell, DistributiveCardinalParadigm, FiniteTense,
        FiniteVerbParadigm, FormSet, FormSource, FormVariant, FractionalNumeralDeclension,
        FractionalNumeralIdentity, FractionalNumeralParadigm, Gender, GenderedNumeralParadigm,
        GenderedPronounParadigm, ImperativeParadigm, IndefiniteNumeralIdentity,
        IndefiniteNumeralParadigm, InflectionError, InflectionResult, InflectionWarning,
        InterrogativePronounIdentity, IrregularAgreeingIdentity, IrregularVerbAnalysis,
        IrregularVerbFamilyMember, IrregularVerbGroup, LParticipleParadigm, Lemma,
        LongOnlyAdjectiveIdentity, Noun, NounParadigm, Number, Numeral, NumeralParadigm,
        OrdinalNumeralIdentity, OrdinalNumeralParadigm, ParadigmLookupError, PartOfSpeech,
        Participle, ParticipleKind, ParticipleParadigm, Person, PersonalPronounIdentity,
        PersonalPronounParadigm, Pronoun, PronounFormSelection, PronounParadigm, Script,
        StandardPronominalIdentity, UniqueVerbFamilyMember, UniqueVerbIdentity,
        UniqueVerbProfileKind, VariantPolicy, VariantSelectionError, Verb, VerbParadigm,
        VerbalNounParadigm, adjective_paradigm, anaphoric_pronoun, aorist, cardinal_magnitude,
        cardinal_numeral_identity, cardinal_numeral_paradigm, collective_numeral,
        collective_numeral_identity, collective_numeral_paradigm,
        collective_numeral_paradigm_identity, comparative_citation, compound_cardinal,
        compound_cardinal_paradigm, compound_cardinal_paradigm_with_one,
        compound_cardinal_paradigm_with_options, compound_cardinal_with_one,
        compound_cardinal_with_options, compound_ordinal, compound_ordinal_paradigm, determiner,
        determiner_identity, determiner_paradigm, distributive_cardinal,
        distributive_cardinal_paradigm, distributive_cardinal_paradigm_with_one,
        distributive_cardinal_paradigm_with_options, distributive_cardinal_with_one,
        distributive_cardinal_with_options, finite, finite_paradigm, fractional_numeral,
        fractional_numeral_identity, fractional_numeral_paradigm,
        fractional_numeral_paradigm_identity, gendered_numeral, gendered_numeral_paradigm,
        gendered_pronoun, gendered_pronoun_paradigm, imperative, imperative_paradigm, imperfect,
        indefinite_numeral, indefinite_numeral_identity, indefinite_numeral_paradigm,
        indefinite_numeral_paradigm_identity, infinitive, interrogative_pronoun,
        irregular_agreeing, l_participle, l_participle_paradigm, long_adjective,
        long_only_adjective, lookup, noun, noun_paradigm, numeral, numeral_paradigm,
        ordinal_numeral, ordinal_numeral_identity, ordinal_numeral_paradigm,
        ordinal_numeral_paradigm_identity, participle_paradigm, past_active_participle,
        past_passive_participle, personal_pronoun, personal_pronoun_paradigm,
        personal_pronoun_with, present, present_active_participle, present_paradigm,
        present_passive_participle, pronoun, pronoun_paradigm, reflexive_pronoun,
        regular_pronominal, relative_pronoun, short_adjective, supine, verbal_noun,
        verbal_noun_form, verbal_noun_paradigm,
    };
}

/// The structured result returned by ordinary inflection calls.
pub type InflectionResult = Result<FormSet, InflectionError>;

/// Apply an explicit, evidence-carrying reconstruction of OCS stress.
///
/// This function composes after morphology. It never infers stress from a bare
/// spelling and always returns [`ReconstructedAccent`], keeping the output
/// distinct from source-attested diacritics.
///
/// ```
/// use old_church_slavonic::{
///     reconstruct_accent, AccentEvidence, AccentParadigm, AccentPlacement,
///     AccentReconstructionStatus, Case, Number, RequestedCell,
/// };
/// use old_church_slavonic::advanced::cells::NounCell;
/// let cell = RequestedCell::Noun(NounCell {
///     case: Case::Genitive,
///     number: Number::Singular,
/// });
/// let paradigm = AccentParadigm::fixed(
///     "comparative-fixed-root",
///     AccentPlacement::VowelFromStart(0),
///     AccentEvidence {
///         source_id: "caller-reviewed-accentology".into(),
///         citation: "caller-supplied comparative reconstruction".into(),
///         status: AccentReconstructionStatus::Comparative,
///     },
/// );
/// assert_eq!(
///     reconstruct_accent("града", &cell, &paradigm)?.text(),
///     "гра́да",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn reconstruct_accent(
    form: &str,
    cell: &RequestedCell,
    paradigm: &AccentParadigm,
) -> Result<ReconstructedAccent, InflectionError> {
    paradigm.apply(cell, form)
}

/// Realize one complete generated or exact OCS word in normalized Glagolitic.
///
/// Already-Glagolitic input is retained unchanged without manufacturing a
/// source-attestation claim. Cyrillic-only distinctions are rejected or returned
/// with ordered loss metadata according to `loss_policy`.
///
/// ```
/// use old_church_slavonic::{
///     realize_glagolitic, GlagoliticProfile, TransliterationFidelity,
///     TransliterationLossPolicy,
/// };
/// let realized = realize_glagolitic(
///     "слово",
///     GlagoliticProfile::Jagic1879NormalizedOcs,
///     TransliterationLossPolicy::Reject,
/// )?;
/// assert_eq!(realized.text(), "ⱄⰾⱁⰲⱁ");
/// assert_eq!(realized.fidelity(), TransliterationFidelity::Reversible);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn realize_glagolitic(
    form: &str,
    profile: GlagoliticProfile,
    loss_policy: TransliterationLossPolicy,
) -> Result<TransliteratedForm, InflectionError> {
    old_church_slavonic_core::realize_glagolitic(form, profile, loss_policy)
}

/// Realize every ordered variant of one morphology result in normalized
/// Glagolitic without selecting or discarding a variant.
///
/// The returned vector is one-to-one with [`FormSet::variants`]. Keep the
/// original `FormSet` to inspect dictionary or productive provenance; each
/// returned item adds only its orthographic fidelity, losses, and trace.
///
/// ```
/// use old_church_slavonic::{
///     aorist, realize_glagolitic_variants, GlagoliticProfile, Number, Person,
///     TransliterationLossPolicy,
/// };
/// let forms = aorist("бꙑти", Person::First, Number::Singular)?;
/// let realized = realize_glagolitic_variants(
///     &forms,
///     GlagoliticProfile::Jagic1879NormalizedOcs,
///     TransliterationLossPolicy::Reject,
/// )?;
/// assert_eq!(
///     realized.iter().map(|form| form.text()).collect::<Vec<_>>(),
///     ["ⰱⱑⱈⱏ", "ⰱⱏⰹⱈⱏ"],
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn realize_glagolitic_variants(
    forms: &FormSet,
    profile: GlagoliticProfile,
    loss_policy: TransliterationLossPolicy,
) -> Result<Vec<TransliteratedForm>, InflectionError> {
    forms
        .texts()
        .map(|form| old_church_slavonic_core::realize_glagolitic(form, profile, loss_policy))
        .collect()
}

/// Transliterate normalized OCS Glagolitic to the canonical Cyrillic choices
/// of the selected profile.
///
/// ```
/// use old_church_slavonic::{
///     transliterate_glagolitic_to_cyrillic, GlagoliticProfile,
///     TransliterationLossPolicy,
/// };
/// let cyrillic = transliterate_glagolitic_to_cyrillic(
///     "ⱄⰾⱁⰲⱁ",
///     GlagoliticProfile::Jagic1879NormalizedOcs,
///     TransliterationLossPolicy::Reject,
/// )?;
/// assert_eq!(cyrillic.text(), "слово");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn transliterate_glagolitic_to_cyrillic(
    form: &str,
    profile: GlagoliticProfile,
    loss_policy: TransliterationLossPolicy,
) -> Result<TransliteratedForm, InflectionError> {
    old_church_slavonic_core::transliterate_glagolitic_to_cyrillic(form, profile, loss_policy)
}

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

/// Decline one simple ordinal from first through tenth by lemma.
///
/// Ordinals expose the complete short/long adjective agreement space, including
/// animacy-sensitive masculine accusatives. `третии` is generated from its
/// source-listed `трет.ьj` workstem rather than forced through an ordinary soft
/// consonant profile.
///
/// ```
/// use old_church_slavonic::{
///     ordinal_numeral, AdjectiveForm, Animacy, Case, Gender, Number,
/// };
/// assert_eq!(
///     ordinal_numeral(
///         "четврьтъ",
///         AdjectiveForm::Long,
///         Case::Genitive,
///         Number::Singular,
///         Gender::Masculine,
///         Animacy::Inanimate,
///     )?
///     .primary_text(),
///     "четврьтаѥго",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn ordinal_numeral(
    lemma: &str,
    form: AdjectiveForm,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    resolver::ordinal_numeral(
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

/// Decline one simple ordinal by its stable reviewed grammatical identity.
///
/// ```
/// use old_church_slavonic::{
///     ordinal_numeral_identity, AdjectiveForm, Animacy, Case, Gender, Number,
///     OrdinalNumeralIdentity,
/// };
/// assert_eq!(
///     ordinal_numeral_identity(
///         OrdinalNumeralIdentity::Second,
///         AdjectiveForm::Short,
///         Case::Nominative,
///         Number::Singular,
///         Gender::Feminine,
///         Animacy::Inanimate,
///     )?
///     .primary_text(),
///     "вътора",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn ordinal_numeral_identity(
    identity: OrdinalNumeralIdentity,
    form: AdjectiveForm,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> InflectionResult {
    resolver::reviewed_ordinal_numeral(
        identity,
        old_church_slavonic_core::AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
        },
    )
}

/// Decline a collective numeral from the inherited two-through-ten series.
///
/// The cell type records the real inflectional split: `дъвои`, `обои`, and
/// `трои` take a pronominal cell, while four through ten take an adjectival
/// cell with explicit short/long form and animacy.
///
/// ```
/// use old_church_slavonic::{
///     collective_numeral, Case, CollectiveNumeralCell, Gender, Number,
/// };
/// assert_eq!(
///     collective_numeral(
///         "дъвои",
///         CollectiveNumeralCell::pronominal(
///             Case::Accusative,
///             Number::Singular,
///             Gender::Neuter,
///         ),
///     )?
///     .primary_text(),
///     "дъвоѥ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn collective_numeral(lemma: &str, cell: CollectiveNumeralCell) -> InflectionResult {
    resolver::collective_numeral(lemma, cell)
}

/// Decline a collective numeral by stable grammatical identity.
///
/// ```
/// use old_church_slavonic::{
///     collective_numeral_identity, AdjectiveForm, Animacy, Case,
///     CollectiveNumeralCell, CollectiveNumeralIdentity, Gender, Number,
/// };
/// let forms = collective_numeral_identity(
///     CollectiveNumeralIdentity::Four,
///     CollectiveNumeralCell::adjectival(
///         AdjectiveForm::Short,
///         Case::Nominative,
///         Number::Singular,
///         Gender::Masculine,
///         Animacy::Inanimate,
///     ),
/// )?;
/// assert_eq!(forms.texts().collect::<Vec<_>>(), ["четворъ", "четвѣръ"]);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn collective_numeral_identity(
    identity: CollectiveNumeralIdentity,
    cell: CollectiveNumeralCell,
) -> InflectionResult {
    resolver::reviewed_collective_numeral(identity, cell)
}

/// Decline one of the source-listed OCS fractional nouns.
///
/// The specialized inventory contains `полъ`, `половина`, `четврьть`, and
/// `десѧтина`. They use ordinary noun declension; later Church Slavonic
/// `третина` and `полътора` are intentionally outside this OCS API.
///
/// ```
/// use old_church_slavonic::{fractional_numeral, Case, Number};
/// assert_eq!(
///     fractional_numeral("четврьть", Case::Instrumental, Number::Singular)?
///         .primary_text(),
///     "четврьтьѭ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn fractional_numeral(lemma: &str, case: Case, number: Number) -> InflectionResult {
    resolver::fractional_numeral(lemma, old_church_slavonic_core::NounCell { case, number })
}

/// Decline an OCS fractional noun by stable grammatical identity.
///
/// ```
/// use old_church_slavonic::{
///     fractional_numeral_identity, Case, FractionalNumeralIdentity, Number,
/// };
/// assert_eq!(
///     fractional_numeral_identity(
///         FractionalNumeralIdentity::Tenth,
///         Case::Accusative,
///         Number::Singular,
///     )?
///     .primary_text(),
///     "десѧтинѫ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn fractional_numeral_identity(
    identity: FractionalNumeralIdentity,
    case: Case,
    number: Number,
) -> InflectionResult {
    resolver::reviewed_fractional_numeral(
        identity,
        old_church_slavonic_core::NounCell { case, number },
    )
}

/// Decline an OCS indefinite-quantity numeral noun.
///
/// The closed inventory currently contains `несъвѣда` “an incalculable
/// quantity”. It follows the hard feminine a-stem paradigm. It is not an exact
/// integer synonym for `тъма` and therefore never appears in composed cardinal
/// output.
///
/// ```
/// use old_church_slavonic::{indefinite_numeral, Case, Number};
/// assert_eq!(
///     indefinite_numeral("несъвѣда", Case::Instrumental, Number::Plural)?
///         .primary_text(),
///     "несъвѣдами",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn indefinite_numeral(lemma: &str, case: Case, number: Number) -> InflectionResult {
    resolver::indefinite_numeral(lemma, old_church_slavonic_core::NounCell { case, number })
}

/// Decline an indefinite-quantity numeral noun by stable identity.
///
/// ```
/// use old_church_slavonic::{
///     indefinite_numeral_identity, Case, IndefiniteNumeralIdentity, Number,
/// };
/// assert_eq!(
///     indefinite_numeral_identity(
///         IndefiniteNumeralIdentity::Nesveda,
///         Case::Accusative,
///         Number::Singular,
///     )?
///     .primary_text(),
///     "несъвѣдѫ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn indefinite_numeral_identity(
    identity: IndefiniteNumeralIdentity,
    case: Case,
    number: Number,
) -> InflectionResult {
    resolver::reviewed_indefinite_numeral(
        identity,
        old_church_slavonic_core::NounCell { case, number },
    )
}

/// Decline a cardinal magnitude head as a noun-like numeral.
///
/// All magnitude heads govern a genitive-plural complement and therefore have
/// no gender parameter. Both documented thousand spellings are stable lexical
/// identities; each retains the reviewed compact and expanded graphic forms.
///
/// ```
/// use old_church_slavonic::{cardinal_magnitude, CardinalMagnitudeIdentity, Case, Number};
/// assert_eq!(
///     cardinal_magnitude(
///         CardinalMagnitudeIdentity::ThousandBackYus,
///         Case::Nominative,
///         Number::Plural,
///     )?
///     .primary_text(),
///     "тꙑсѫщѩ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn cardinal_magnitude(
    identity: CardinalMagnitudeIdentity,
    case: Case,
    number: Number,
) -> InflectionResult {
    resolver::reviewed_cardinal_magnitude(
        identity,
        NumeralCell {
            case,
            number,
            gender: None,
        },
    )
}

/// Inflect a composed cardinal from 11 through 10,000.
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
    compound_cardinal_with_options(value, CardinalCompositionOptions::DEFAULT, case, gender)
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

/// Inflect a composed cardinal while explicitly selecting its lexical one and
/// thousand doublets.
///
/// ```
/// use old_church_slavonic::{
///     compound_cardinal_with_options, CardinalCompositionOptions,
///     CardinalMagnitudeIdentity, CardinalNumeralIdentity, Case,
/// };
/// let options = CardinalCompositionOptions {
///     one_identity: CardinalNumeralIdentity::OneYedyn,
///     thousand_identity: CardinalMagnitudeIdentity::ThousandLittleYus,
/// };
/// assert_eq!(
///     compound_cardinal_with_options(1000, options, Case::Nominative, None)?.primary_text(),
///     "тꙑсѧщи",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn compound_cardinal_with_options(
    value: u16,
    options: CardinalCompositionOptions,
    case: Case,
    gender: Option<Gender>,
) -> Result<RealizedCardinal, InflectionError> {
    resolver::compound_cardinal_with_options(value, CompoundCardinalCell { case, gender }, options)
}

/// Enumerate every case and optional-gender request for a composed cardinal
/// from 11 through 10,000, retaining invalid shapes as typed failures.
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
    compound_cardinal_paradigm_with_options(value, CardinalCompositionOptions::DEFAULT)
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
    compound_cardinal_paradigm_with_options(
        value,
        CardinalCompositionOptions {
            one_identity,
            ..CardinalCompositionOptions::DEFAULT
        },
    )
}

/// Enumerate every case and optional-gender request while explicitly selecting
/// the compound cardinal's lexical one and thousand doublets.
///
/// ```
/// use old_church_slavonic::{
///     compound_cardinal_paradigm_with_options, CardinalCompositionOptions,
///     CardinalMagnitudeIdentity, CardinalNumeralIdentity, Case,
/// };
/// let options = CardinalCompositionOptions {
///     one_identity: CardinalNumeralIdentity::OneYedyn,
///     thousand_identity: CardinalMagnitudeIdentity::ThousandLittleYus,
/// };
/// let paradigm = compound_cardinal_paradigm_with_options(2000, options)?;
/// assert_eq!(
///     paradigm.form(Case::Nominative, None)?.primary_text(),
///     "дъвѣ тꙑсѧщи",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compound_cardinal_paradigm_with_options(
    value: u16,
    options: CardinalCompositionOptions,
) -> Result<CompoundCardinalParadigm, InflectionError> {
    resolver::build_compound_cardinal_paradigm(value, options)
}

/// Realize distributive `по` with a dative cardinal from one through 10,000.
///
/// This is a structured syntactic construction rather than a synthetic
/// declension. Gender is required exactly when the cardinal's final unit is
/// one through four. The fixed dative prevents temporal `по + locative` from
/// being mistaken for a distributive.
///
/// ```
/// use old_church_slavonic::{distributive_cardinal, Gender};
/// assert_eq!(
///     distributive_cardinal(2, Some(Gender::Masculine))?.primary_text(),
///     "по дъвѣма",
/// );
/// assert_eq!(distributive_cardinal(50, None)?.primary_text(), "по пѧти десѧтъ");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn distributive_cardinal(
    value: u16,
    gender: Option<Gender>,
) -> Result<RealizedDistributiveCardinal, InflectionError> {
    distributive_cardinal_with_options(value, CardinalCompositionOptions::DEFAULT, gender)
}

/// Realize a distributive cardinal while selecting the lexical doublet used
/// for one.
///
/// ```
/// use old_church_slavonic::{
///     distributive_cardinal_with_one, CardinalNumeralIdentity, Gender,
/// };
/// assert_eq!(
///     distributive_cardinal_with_one(
///         1,
///         CardinalNumeralIdentity::OneYedyn,
///         Some(Gender::Feminine),
///     )?
///     .primary_text(),
///     "по ѥдьнои",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn distributive_cardinal_with_one(
    value: u16,
    one_identity: CardinalNumeralIdentity,
    gender: Option<Gender>,
) -> Result<RealizedDistributiveCardinal, InflectionError> {
    distributive_cardinal_with_options(
        value,
        CardinalCompositionOptions {
            one_identity,
            ..CardinalCompositionOptions::DEFAULT
        },
        gender,
    )
}

/// Realize a distributive cardinal while selecting both cardinal-composition
/// lexical doublets.
///
/// ```
/// use old_church_slavonic::{
///     distributive_cardinal_with_options, CardinalCompositionOptions,
///     CardinalMagnitudeIdentity, CardinalNumeralIdentity, Gender,
/// };
/// let options = CardinalCompositionOptions {
///     one_identity: CardinalNumeralIdentity::OneYedyn,
///     thousand_identity: CardinalMagnitudeIdentity::ThousandLittleYus,
/// };
/// assert_eq!(
///     distributive_cardinal_with_options(1_001, options, Some(Gender::Feminine))?
///         .primary_text(),
///     "по тꙑсѧщи и ѥдьнои",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn distributive_cardinal_with_options(
    value: u16,
    options: CardinalCompositionOptions,
    gender: Option<Gender>,
) -> Result<RealizedDistributiveCardinal, InflectionError> {
    resolver::distributive_cardinal_with_options(
        value,
        DistributiveCardinalCell { gender },
        options,
    )
}

/// Enumerate all four optional-gender requests for one distributive cardinal,
/// retaining invalid request shapes as typed historical failures.
///
/// ```
/// use old_church_slavonic::distributive_cardinal_paradigm;
/// let paradigm = distributive_cardinal_paradigm(50)?;
/// assert_eq!(paradigm.len(), 4);
/// assert_eq!(paradigm.successes().count(), 1);
/// assert_eq!(paradigm.form(None)?.primary_text(), "по пѧти десѧтъ");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn distributive_cardinal_paradigm(
    value: u16,
) -> Result<DistributiveCardinalParadigm, InflectionError> {
    distributive_cardinal_paradigm_with_options(value, CardinalCompositionOptions::DEFAULT)
}

/// Enumerate a distributive cardinal while selecting its lexical doublet of
/// one.
///
/// ```
/// use old_church_slavonic::{
///     distributive_cardinal_paradigm_with_one, CardinalNumeralIdentity, Gender,
/// };
/// let paradigm = distributive_cardinal_paradigm_with_one(
///     1,
///     CardinalNumeralIdentity::OneYedyn,
/// )?;
/// assert_eq!(
///     paradigm.form(Some(Gender::Feminine))?.primary_text(),
///     "по ѥдьнои",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn distributive_cardinal_paradigm_with_one(
    value: u16,
    one_identity: CardinalNumeralIdentity,
) -> Result<DistributiveCardinalParadigm, InflectionError> {
    distributive_cardinal_paradigm_with_options(
        value,
        CardinalCompositionOptions {
            one_identity,
            ..CardinalCompositionOptions::DEFAULT
        },
    )
}

/// Enumerate a distributive cardinal with explicit cardinal-composition
/// lexical choices.
///
/// ```
/// use old_church_slavonic::{
///     distributive_cardinal_paradigm_with_options, CardinalCompositionOptions,
///     CardinalMagnitudeIdentity, CardinalNumeralIdentity,
/// };
/// let options = CardinalCompositionOptions {
///     one_identity: CardinalNumeralIdentity::OneYedyn,
///     thousand_identity: CardinalMagnitudeIdentity::ThousandLittleYus,
/// };
/// let paradigm = distributive_cardinal_paradigm_with_options(2_000, options)?;
/// assert_eq!(paradigm.options(), options);
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn distributive_cardinal_paradigm_with_options(
    value: u16,
    options: CardinalCompositionOptions,
) -> Result<DistributiveCardinalParadigm, InflectionError> {
    resolver::build_distributive_cardinal_paradigm(value, options)
}

/// Inflect a compound ordinal from 11 through 1,000 as a structured sequence.
///
/// Analytic teens, fused historical heads, additive forms, competing asyndetic
/// declension accounts, and the alternative 21–29 turns remain separate
/// analyses; each token retains its own variants, evidence, warnings, and trace.
///
/// ```
/// use old_church_slavonic::{
///     compound_ordinal, AdjectiveForm, Animacy, Case, Gender, Number,
/// };
/// let ordinal = compound_ordinal(
///     18,
///     AdjectiveForm::Long,
///     Case::Nominative,
///     Number::Singular,
///     Gender::Neuter,
///     Animacy::Inanimate,
/// )?;
/// assert_eq!(ordinal.primary_text(), "осмоѥ на десѧте");
/// assert!(ordinal.analyses().iter().any(|analysis| {
///     analysis.primary_text().starts_with("осмонадесѧто")
/// }));
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn compound_ordinal(
    value: u16,
    form: AdjectiveForm,
    case: Case,
    number: Number,
    gender: Gender,
    animacy: Animacy,
) -> Result<RealizedOrdinal, InflectionError> {
    resolver::compound_ordinal(
        value,
        old_church_slavonic_core::AdjectiveCell {
            form,
            case,
            number,
            gender,
            animacy,
        },
    )
}

/// Enumerate all 252 adjective-agreement requests for one compound ordinal.
///
/// ```
/// use old_church_slavonic::{
///     compound_ordinal_paradigm, AdjectiveForm, Animacy, Case, Gender, Number,
/// };
/// let paradigm = compound_ordinal_paradigm(104)?;
/// assert_eq!(paradigm.len(), 252);
/// assert_eq!(
///     paradigm
///         .form(
///             AdjectiveForm::Long,
///             Case::Genitive,
///             Number::Singular,
///             Gender::Neuter,
///             Animacy::Inanimate,
///         )?
///         .primary_text(),
///     "сътьнаѥго четврьтаѥго",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn compound_ordinal_paradigm(value: u16) -> Result<CompoundOrdinalParadigm, InflectionError> {
    resolver::build_compound_ordinal_paradigm(value)
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

/// Return a source-listed or productively formed verbal-noun citation.
///
/// ```
/// let forms = old_church_slavonic::verbal_noun("благословити")?;
/// assert_eq!(forms.primary_text(), "благословлѥниѥ");
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn verbal_noun(lemma: &str) -> InflectionResult {
    resolver::verbal_noun(lemma)
}

/// Decline one case-number cell of the verbal noun derived from a verb.
///
/// ```
/// use old_church_slavonic::{verbal_noun_form, Case, Number};
/// assert_eq!(
///     verbal_noun_form("благословити", Case::Genitive, Number::Singular)?
///         .primary_text(),
///     "благословлѥниꙗ",
/// );
/// # Ok::<(), old_church_slavonic::InflectionError>(())
/// ```
pub fn verbal_noun_form(lemma: &str, case: Case, number: Number) -> InflectionResult {
    resolver::verbal_noun_form(lemma, old_church_slavonic_core::NounCell { case, number })
}

/// Enumerate all seven cases in singular, dual, and plural for a verbal noun.
///
/// ```
/// use old_church_slavonic::{verbal_noun_paradigm, Case, Number};
/// let paradigm = verbal_noun_paradigm("благословити")?;
/// assert_eq!(paradigm.len(), 21);
/// assert_eq!(
///     paradigm.form(Case::Genitive, Number::Singular)?.primary_text(),
///     "благословлѥниꙗ",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn verbal_noun_paradigm(lemma: &str) -> Result<VerbalNounParadigm, InflectionError> {
    Verb::resolve(lemma).map(|verb| verb.verbal_noun_paradigm())
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

/// Enumerate all 252 short/long agreement cells for a simple ordinal lemma.
///
/// ```
/// use old_church_slavonic::{
///     ordinal_numeral_paradigm, AdjectiveForm, Animacy, Case, Gender, Number,
/// };
/// let paradigm = ordinal_numeral_paradigm("третии")?;
/// assert_eq!(paradigm.len(), 252);
/// assert_eq!(
///     paradigm.form(
///         AdjectiveForm::Short,
///         Case::Nominative,
///         Number::Singular,
///         Gender::Neuter,
///         Animacy::Inanimate,
///     )?
///     .primary_text(),
///     "третиѥ",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn ordinal_numeral_paradigm(lemma: &str) -> Result<OrdinalNumeralParadigm, InflectionError> {
    resolver::ordinal_numeral_paradigm(lemma)
}

/// Enumerate all ordinal agreement cells from a stable reviewed identity.
///
/// ```
/// use old_church_slavonic::{
///     ordinal_numeral_paradigm_identity, AdjectiveForm, Animacy, Case, Gender, Number,
///     OrdinalNumeralIdentity,
/// };
/// let paradigm = ordinal_numeral_paradigm_identity(OrdinalNumeralIdentity::First);
/// assert_eq!(
///     paradigm.form(
///         AdjectiveForm::Long,
///         Case::Nominative,
///         Number::Singular,
///         Gender::Masculine,
///         Animacy::Inanimate,
///     )?
///     .primary_text(),
///     "прьвꙑи",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn ordinal_numeral_paradigm_identity(
    identity: OrdinalNumeralIdentity,
) -> OrdinalNumeralParadigm {
    resolver::build_ordinal_numeral_paradigm(identity)
}

/// Enumerate every licensed cell for a collective-numeral lemma.
///
/// ```
/// use old_church_slavonic::{
///     collective_numeral_paradigm, Case, Gender, Number,
/// };
/// let paradigm = collective_numeral_paradigm("обои")?;
/// assert_eq!(paradigm.len(), 63);
/// assert_eq!(
///     paradigm
///         .pronominal_form(Case::Nominative, Number::Singular, Gender::Neuter)?
///         .primary_text(),
///     "обоѥ",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn collective_numeral_paradigm(
    lemma: &str,
) -> Result<CollectiveNumeralParadigm, InflectionError> {
    resolver::collective_numeral_paradigm(lemma)
}

/// Enumerate every licensed cell from a stable collective identity.
///
/// ```
/// use old_church_slavonic::{
///     collective_numeral_paradigm_identity, AdjectiveForm, Animacy, Case,
///     CollectiveNumeralIdentity, Gender, Number,
/// };
/// let paradigm =
///     collective_numeral_paradigm_identity(CollectiveNumeralIdentity::Seven);
/// assert_eq!(paradigm.len(), 252);
/// assert_eq!(
///     paradigm
///         .adjectival_form(
///             AdjectiveForm::Short,
///             Case::Nominative,
///             Number::Singular,
///             Gender::Masculine,
///             Animacy::Inanimate,
///         )?
///         .primary_text(),
///     "седморъ",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn collective_numeral_paradigm_identity(
    identity: CollectiveNumeralIdentity,
) -> CollectiveNumeralParadigm {
    resolver::build_collective_numeral_paradigm(identity)
}

/// Enumerate all 21 noun cells for a source-listed fractional lemma.
///
/// ```
/// use old_church_slavonic::{fractional_numeral_paradigm, Case, Number};
/// let paradigm = fractional_numeral_paradigm("полъ")?;
/// assert_eq!(paradigm.len(), 21);
/// assert_eq!(
///     paradigm.form(Case::Genitive, Number::Singular)?.primary_text(),
///     "полоу",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn fractional_numeral_paradigm(
    lemma: &str,
) -> Result<FractionalNumeralParadigm, InflectionError> {
    resolver::fractional_numeral_paradigm(lemma)
}

/// Enumerate all 21 noun cells from a stable fractional identity.
///
/// ```
/// use old_church_slavonic::{
///     fractional_numeral_paradigm_identity, Case, FractionalNumeralIdentity, Number,
/// };
/// let paradigm =
///     fractional_numeral_paradigm_identity(FractionalNumeralIdentity::HalfPolovina);
/// assert_eq!(
///     paradigm.form(Case::Accusative, Number::Singular)?.primary_text(),
///     "половинѫ",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn fractional_numeral_paradigm_identity(
    identity: FractionalNumeralIdentity,
) -> FractionalNumeralParadigm {
    resolver::build_fractional_numeral_paradigm(identity)
}

/// Enumerate all 21 noun cells for a source-listed indefinite quantity.
///
/// ```
/// use old_church_slavonic::{indefinite_numeral_paradigm, Case, Number};
/// let paradigm = indefinite_numeral_paradigm("несъвѣда")?;
/// assert_eq!(paradigm.len(), 21);
/// assert_eq!(
///     paradigm.form(Case::Instrumental, Number::Plural)?.primary_text(),
///     "несъвѣдами",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn indefinite_numeral_paradigm(
    lemma: &str,
) -> Result<IndefiniteNumeralParadigm, InflectionError> {
    resolver::indefinite_numeral_paradigm(lemma)
}

/// Enumerate all 21 noun cells from a stable indefinite-quantity identity.
///
/// ```
/// use old_church_slavonic::{
///     indefinite_numeral_paradigm_identity, Case, IndefiniteNumeralIdentity, Number,
/// };
/// let paradigm =
///     indefinite_numeral_paradigm_identity(IndefiniteNumeralIdentity::Nesveda);
/// assert_eq!(
///     paradigm.form(Case::Dative, Number::Dual)?.primary_text(),
///     "несъвѣдама",
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn indefinite_numeral_paradigm_identity(
    identity: IndefiniteNumeralIdentity,
) -> IndefiniteNumeralParadigm {
    resolver::build_indefinite_numeral_paradigm(identity)
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
