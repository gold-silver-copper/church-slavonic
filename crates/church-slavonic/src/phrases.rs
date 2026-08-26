//! Consumer-facing Old Church Slavonic analytic (multi-token) constructions,
//! ported from the fat facade's phrase layer onto the pilot's single-cell
//! functions.
//!
//! Design rules (the same ones the single-cell facade follows):
//!
//! - **Typed parameters, `String`/`Vec<String>` out, typed [`Error`].** A
//!   phrase is returned as its space-joined surface text; every construction
//!   has a `*_variants` companion enumerating the token-level variant
//!   combinations (odometer order, primary first, duplicates dropped).
//! - **A paradigm-selecting distinction becomes a function.** The old
//!   `copula(series, …)` enum dispatch is six functions ([`copula_present`]
//!   … [`copula_conditional_aorist`]); the old `PluperfectAuxiliary` enum is
//!   [`pluperfect`] / [`pluperfect_aorist`] / [`pluperfect_perfect`]; the old
//!   `ConditionalAuxiliary` enum is the `*_aorist` function pairs; the old
//!   `FutureReferenceTense` enum is [`infinitival_future`] /
//!   [`infinitival_future_imperfect`] / [`infinitival_future_aorist`]; the
//!   old `impersonal_predicate(identity, tense)` is the lemma-keyed
//!   [`impersonal_present`] / [`impersonal_imperfect`] / [`impersonal_aorist`].
//! - **Composition lives in `old-church-slavonic-core`.** The §316
//!   pronominal-family validation and token assembly moved down into
//!   `old_church_slavonic_core::pronoun` (the fat facade delegates to the
//!   same functions); the copular series tables are the core's reviewed
//!   `CopulaSeries` cells; the inflected members come from this crate's own
//!   residue-table → rule-kernel functions.
//!
//! The constructions predicated on a *declined* participle ride on
//! [`crate::participle`]: the old `PassiveAuxiliary` enum became the
//! function family [`analytic_passive`] / [`analytic_passive_imperfect`] /
//! [`analytic_passive_aorist`] / [`analytic_passive_future`] /
//! [`conditional_passive`] / [`conditional_passive_aorist`], and
//! [`participial_future`] takes the active kinds. The old
//! `elliptical_conditional_optative` collapsed to [`crate::l_participle`];
//! `relative_superlative_with` fell with the comparative exclusion.
//!
//! `cargo xtask rewrite-pilot-accuracy` runs a differential gate: over a
//! deterministic parameter sweep every construction here must agree with the
//! old facade's phrase layer (or with the old facade's inflected member
//! joined per the same order convention), including agreement on rejected
//! cells, at 100%.

use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_core::{
    CopulaSeries, FiniteTense, ImpersonalVerbIdentity, InterrogativePronounIdentity,
    PronominalFamilySpec, orthography, pronoun as core_pronoun,
};

pub use old_church_slavonic_core::{
    DirectToTreatment, FutureInfinitiveAuxiliary, PhraseOrder, PronominalPostpositive,
    PronominalPrefix,
};

use crate::{Case, Error, Gender, Number, Person};

fn unsupported(reason: impl Into<String>) -> Error {
    Error::UnsupportedPhrase {
        reason: reason.into(),
    }
}

/// Odometer over each token's ordered variant list, primary first, so the
/// first rendered phrase is every token's primary form. Mirrors the
/// enumeration order of [`crate::numeral_variants`].
fn phrase_variants(tokens: &[Vec<String>]) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    let mut indices = vec![0usize; tokens.len()];
    'odometer: loop {
        let text = tokens
            .iter()
            .zip(&indices)
            .map(|(variants, index)| variants[*index].as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if !texts.contains(&text) {
            texts.push(text);
        }
        let mut position = tokens.len();
        loop {
            if position == 0 {
                break 'odometer;
            }
            position -= 1;
            indices[position] += 1;
            if indices[position] < tokens[position].len() {
                break;
            }
            indices[position] = 0;
        }
    }
    texts
}

fn first_phrase(mut variants: Vec<String>, lemma: &str) -> Result<String, Error> {
    if variants.is_empty() {
        return Err(Error::Underdetermined {
            lemma: lemma.to_string(),
        });
    }
    Ok(variants.remove(0))
}

fn ordered(dependent: Vec<String>, head: Vec<String>, order: PhraseOrder) -> Vec<Vec<String>> {
    match order {
        PhraseOrder::DependentFirst => vec![dependent, head],
        PhraseOrder::HeadFirst => vec![head, dependent],
    }
}

// ---------------------------------------------------------------------------
// §316 derived pronominal families (никъто-composition)
// ---------------------------------------------------------------------------

/// Every rendered phrase for one §316 derived pronominal family, primary
/// first. See [`pronominal_family`].
pub fn pronominal_family_variants(
    lemma: &str,
    case: Case,
    prefix: Option<PronominalPrefix>,
    postpositive: Option<PronominalPostpositive>,
    direct_to: Option<DirectToTreatment>,
    preposition: Option<&str>,
) -> Result<Vec<String>, Error> {
    let identity = match lemma {
        "къто" => InterrogativePronounIdentity::Kto,
        "чьто" => InterrogativePronounIdentity::Chto,
        _ => return Err(Error::UnknownLemma(lemma.to_string())),
    };
    // The composition base is the reviewed kernel interrogative table (the
    // same base the fat facade's phrase layer resolves), canonicalized —
    // not the residue-first single-cell route, whose attested primaries may
    // prefer an orthographic doublet of the same form.
    let mut base: Vec<String> = Vec::new();
    for variant in core_pronoun::interrogative_forms(identity, case) {
        let text =
            orthography::canonical_display(variant.text).map_err(|_| Error::Underdetermined {
                lemma: lemma.to_string(),
            })?;
        if !base.contains(&text) {
            base.push(text);
        }
    }
    if base.is_empty() {
        return Err(Error::Underdetermined {
            lemma: lemma.to_string(),
        });
    }
    let base_texts: Vec<&str> = base.iter().map(String::as_str).collect();
    let spec = PronominalFamilySpec {
        prefix,
        postpositive,
        direct_to,
        preposition: preposition.map(str::to_string),
    };
    let tokens = core_pronoun::compose_pronominal_family_tokens(lemma, &base_texts, case, &spec)
        .map_err(|error| unsupported(error.to_string()))?;
    Ok(phrase_variants(&tokens))
}

/// The primary surface phrase of a §316 derived pronominal family built on
/// numberless `къто` or `чьто`: the negative/indefinite prefixes `ни-`/`нѣ-`,
/// the bound postpositives `-же`/`-жде`/`-жьдо`, the independently written
/// `любо`, an explicit retain/drop treatment of the direct-case `-то`, and
/// an optionally interposed preposition (`ни о комьже`).
///
/// Bound formatives stay in the pronominal word; `любо` and an interposed
/// prefix + preposition are independent tokens. Ill-formed requests (no
/// formative at all, a preposition without a prefix or with a nominative, a
/// missing or unlicensed `-то` treatment) return
/// [`Error::UnsupportedPhrase`].
///
/// ```
/// use church_slavonic::phrases::{pronominal_family, PronominalPrefix, PronominalPostpositive, DirectToTreatment};
/// use church_slavonic::Case;
/// assert_eq!(
///     pronominal_family(
///         "чьто", Case::Nominative,
///         Some(PronominalPrefix::Ni), Some(PronominalPostpositive::Ze),
///         Some(DirectToTreatment::Retain), None,
///     ).as_deref(),
///     Ok("ничьтоже")
/// );
/// ```
pub fn pronominal_family(
    lemma: &str,
    case: Case,
    prefix: Option<PronominalPrefix>,
    postpositive: Option<PronominalPostpositive>,
    direct_to: Option<DirectToTreatment>,
    preposition: Option<&str>,
) -> Result<String, Error> {
    first_phrase(
        pronominal_family_variants(lemma, case, prefix, postpositive, direct_to, preposition)?,
        lemma,
    )
}

// ---------------------------------------------------------------------------
// Absolute superlative (invariant ѕѣло + declined positive adjective)
// ---------------------------------------------------------------------------

const ABSOLUTE_SUPERLATIVE_ADVERB: &str = "ѕѣло";

fn superlative_tokens(adjective: Vec<String>, order: PhraseOrder) -> Vec<Vec<String>> {
    ordered(
        vec![ABSOLUTE_SUPERLATIVE_ADVERB.to_string()],
        adjective,
        order,
    )
}

/// Every rendered phrase for one long-form absolute superlative, primary
/// first. See [`absolute_superlative`].
pub fn absolute_superlative_variants(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    let adjective = crate::adjective_variants(lemma, case, number, gender)?;
    Ok(phrase_variants(&superlative_tokens(adjective, order)))
}

/// The source-described absolute superlative: invariant `ѕѣло` with the
/// declined long (definite) positive adjective, in either attested modifier
/// order (`PhraseOrder::DependentFirst` puts the adverb first).
pub fn absolute_superlative(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<String, Error> {
    first_phrase(
        absolute_superlative_variants(lemma, case, number, gender, order)?,
        lemma,
    )
}

/// Every rendered phrase for one short-form absolute superlative, primary
/// first. See [`short_absolute_superlative`].
pub fn short_absolute_superlative_variants(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    let adjective = crate::short_adjective_variants(lemma, case, number, gender)?;
    Ok(phrase_variants(&superlative_tokens(adjective, order)))
}

/// The absolute superlative over the short (indefinite) adjective — a
/// paradigm-selecting distinction, so a separate function, exactly like
/// [`crate::short_adjective`] beside [`crate::adjective`].
pub fn short_absolute_superlative(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<String, Error> {
    first_phrase(
        short_absolute_superlative_variants(lemma, case, number, gender, order)?,
        lemma,
    )
}

// ---------------------------------------------------------------------------
// Copular series
// ---------------------------------------------------------------------------

fn copula_series_variants(series: CopulaSeries, person: Person, number: Number) -> Vec<String> {
    series
        .forms(person, number)
        .into_iter()
        .map(|variant| variant.text.to_string())
        .collect()
}

macro_rules! copula_functions {
    ($($(#[$doc:meta])* $name:ident, $variants_name:ident => $series:expr;)*) => {
        $(
            $(#[$doc])*
            ///
            /// The reviewed series is complete over person x number, so this
            /// function is total. The `_variants` companion returns every
            /// source-ordered variant, primary first.
            pub fn $name(person: Person, number: Number) -> String {
                copula_series_variants($series, person, number).remove(0)
            }

            /// All source-ordered variants for the same cell, primary first.
            pub fn $variants_name(person: Person, number: Number) -> Vec<String> {
                copula_series_variants($series, person, number)
            }
        )*
    };
}

copula_functions! {
    /// The present copular series `ѥс-` (`ѥсмь`, `ѥси`, …).
    copula_present, copula_present_variants => CopulaSeries::PresentEs;
    /// The future copular series `бѫд-`.
    copula_future, copula_future_variants => CopulaSeries::FutureBud;
    /// The imperfect series of `бꙑти` (`бѣахъ`, …).
    copula_imperfect, copula_imperfect_variants => CopulaSeries::ImperfectBe;
    /// The aorist series of `бꙑти` (`бѣхъ`, …).
    copula_aorist, copula_aorist_variants => CopulaSeries::AoristBe;
    /// The dedicated conditional series `би-`.
    copula_conditional, copula_conditional_variants => CopulaSeries::ConditionalBi;
    /// The source-described aorist replacement of the conditional (`бꙑ-`).
    copula_conditional_aorist, copula_conditional_aorist_variants => CopulaSeries::ConditionalAoristBy;
}

// ---------------------------------------------------------------------------
// да + present (analytic imperative/optative)
// ---------------------------------------------------------------------------

/// Every rendered phrase for one `да`-imperative cell, primary first.
pub fn da_imperative_variants(
    lemma: &str,
    person: Person,
    number: Number,
) -> Result<Vec<String>, Error> {
    let present = crate::present_variants(lemma, person, number)?;
    Ok(phrase_variants(&[vec!["да".to_string()], present]))
}

/// The `да` + present imperative/optative for any person-number cell.
///
/// Deliberately distinct from the synthetic imperative: the sources use the
/// periphrasis for the missing first/third-person commands and also beside
/// an existing synthetic imperative when its modal force is appropriate.
pub fn da_imperative(lemma: &str, person: Person, number: Number) -> Result<String, Error> {
    first_phrase(da_imperative_variants(lemma, person, number)?, lemma)
}

// ---------------------------------------------------------------------------
// l-participle periphrases (perfect, pluperfect, future perfect, conditionals)
// ---------------------------------------------------------------------------

fn l_participle_periphrasis(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    series: CopulaSeries,
    order: PhraseOrder,
) -> Result<Vec<Vec<String>>, Error> {
    let head = crate::l_participle_variants(lemma, gender, number)?;
    let auxiliary = copula_series_variants(series, person, number);
    Ok(ordered(auxiliary, head, order))
}

macro_rules! l_participle_constructions {
    ($($(#[$doc:meta])* $name:ident, $variants_name:ident => $series:expr;)*) => {
        $(
            /// Every rendered phrase for the same cell, primary first.
            pub fn $variants_name(
                lemma: &str,
                person: Person,
                number: Number,
                gender: Gender,
                order: PhraseOrder,
            ) -> Result<Vec<String>, Error> {
                let tokens =
                    l_participle_periphrasis(lemma, person, number, gender, $series, order)?;
                Ok(phrase_variants(&tokens))
            }

            $(#[$doc])*
            pub fn $name(
                lemma: &str,
                person: Person,
                number: Number,
                gender: Gender,
                order: PhraseOrder,
            ) -> Result<String, Error> {
                first_phrase($variants_name(lemma, person, number, gender, order)?, lemma)
            }
        )*
    };
}

l_participle_constructions! {
    /// The OCS perfect: agreeing l-participle + present `ѥс-` copula
    /// (`благословилъ ѥсмь`). `PhraseOrder::HeadFirst` puts the participle
    /// first.
    perfect, perfect_variants => CopulaSeries::PresentEs;
    /// The pluperfect with the imperfect series of `бꙑти` (`бѣаше`); the
    /// aorist-auxiliary and perfect-auxiliary formations are the separate
    /// functions [`pluperfect_aorist`] and [`pluperfect_perfect`] — a
    /// paradigm-selecting distinction becomes a function.
    pluperfect, pluperfect_variants => CopulaSeries::ImperfectBe;
    /// The pluperfect with the aorist series of `бꙑти` (`бѣ`).
    pluperfect_aorist, pluperfect_aorist_variants => CopulaSeries::AoristBe;
    /// The future perfect: agreeing l-participle + future `бѫд-`.
    future_perfect, future_perfect_variants => CopulaSeries::FutureBud;
    /// The conditional-optative with the dedicated conditional series `би-`.
    conditional_optative, conditional_optative_variants => CopulaSeries::ConditionalBi;
    /// The conditional-optative with the source-described aorist replacement
    /// series `бꙑ-`.
    conditional_optative_aorist, conditional_optative_aorist_variants => CopulaSeries::ConditionalAoristBy;
}

/// Every rendered phrase for one perfect-auxiliary pluperfect cell, primary
/// first. See [`pluperfect_perfect`].
pub fn pluperfect_perfect_variants(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    let head = crate::l_participle_variants(lemma, gender, number)?;
    let auxiliary_participle = crate::l_participle_variants("бꙑти", gender, number)?;
    let auxiliary = copula_series_variants(CopulaSeries::PresentEs, person, number);
    let tokens = match order {
        PhraseOrder::HeadFirst => vec![head, auxiliary_participle, auxiliary],
        PhraseOrder::DependentFirst => vec![auxiliary_participle, auxiliary, head],
    };
    Ok(phrase_variants(&tokens))
}

/// The three-token pluperfect built on the perfect of `бꙑти`: the lexical
/// l-participle with `бꙑлъ` + present `ѥс-` (`благословилъ бꙑлъ ѥсмь`).
pub fn pluperfect_perfect(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<String, Error> {
    first_phrase(
        pluperfect_perfect_variants(lemma, person, number, gender, order)?,
        lemma,
    )
}

fn da_prefixed(inner: Result<Vec<String>, Error>) -> Result<Vec<String>, Error> {
    Ok(inner?
        .into_iter()
        .map(|text| format!("да {text}"))
        .collect())
}

/// Every rendered phrase for one `да`-marked conditional-optative cell,
/// primary first.
pub fn da_conditional_optative_variants(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    da_prefixed(conditional_optative_variants(
        lemma, person, number, gender, order,
    ))
}

/// The independently described `да`-marked optative: the particle `да`
/// before the conditional-optative (`би-` series).
pub fn da_conditional_optative(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<String, Error> {
    first_phrase(
        da_conditional_optative_variants(lemma, person, number, gender, order)?,
        lemma,
    )
}

/// Every rendered phrase for one `да`-marked aorist-replacement
/// conditional-optative cell, primary first.
pub fn da_conditional_optative_aorist_variants(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    da_prefixed(conditional_optative_aorist_variants(
        lemma, person, number, gender, order,
    ))
}

/// The `да`-marked optative over the aorist replacement series `бꙑ-`.
pub fn da_conditional_optative_aorist(
    lemma: &str,
    person: Person,
    number: Number,
    gender: Gender,
    order: PhraseOrder,
) -> Result<String, Error> {
    first_phrase(
        da_conditional_optative_aorist_variants(lemma, person, number, gender, order)?,
        lemma,
    )
}

// ---------------------------------------------------------------------------
// Infinitival future
// ---------------------------------------------------------------------------

fn infinitival_future_tokens(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    tense: FiniteTense,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    if tense != FiniteTense::Present && !auxiliary.licensed_for_past_reference() {
        return Err(unsupported(format!(
            "{auxiliary:?} is not source-licensed as a past-reference future auxiliary"
        )));
    }
    let auxiliary_forms = match tense {
        FiniteTense::Present => crate::present_variants(auxiliary.lemma(), person, number)?,
        FiniteTense::Imperfect => crate::imperfect_variants(auxiliary.lemma(), person, number)?,
        FiniteTense::Aorist => crate::aorist_variants(auxiliary.lemma(), person, number)?,
    };
    let infinitive = crate::infinitive_variants(lemma)?;
    Ok(phrase_variants(&ordered(
        auxiliary_forms,
        infinitive,
        order,
    )))
}

/// Every rendered phrase for one present-reference infinitival-future cell,
/// primary first.
pub fn infinitival_future_variants(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    infinitival_future_tokens(
        lemma,
        auxiliary,
        FiniteTense::Present,
        person,
        number,
        order,
    )
}

/// The infinitival future: a present-tense lexical auxiliary
/// (`имѣти`/`хотѣти`/`начѧти`/`въчѧти`) with the infinitive
/// (`имѫтъ благословити`). The auxiliary is a lexical index within one
/// construction, so it stays a parameter; the past-reference formations are
/// the separate functions [`infinitival_future_imperfect`] and
/// [`infinitival_future_aorist`].
pub fn infinitival_future(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<String, Error> {
    first_phrase(
        infinitival_future_variants(lemma, auxiliary, person, number, order)?,
        lemma,
    )
}

/// Every rendered phrase for one imperfect-reference infinitival-future
/// cell, primary first.
pub fn infinitival_future_imperfect_variants(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    infinitival_future_tokens(
        lemma,
        auxiliary,
        FiniteTense::Imperfect,
        person,
        number,
        order,
    )
}

/// The future-in-the-past with an imperfect auxiliary; source-licensed for
/// `имѣти` and `хотѣти` only (anything else is
/// [`Error::UnsupportedPhrase`]).
pub fn infinitival_future_imperfect(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<String, Error> {
    first_phrase(
        infinitival_future_imperfect_variants(lemma, auxiliary, person, number, order)?,
        lemma,
    )
}

/// Every rendered phrase for one aorist-reference infinitival-future cell,
/// primary first.
pub fn infinitival_future_aorist_variants(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<Vec<String>, Error> {
    infinitival_future_tokens(lemma, auxiliary, FiniteTense::Aorist, person, number, order)
}

/// The future-in-the-past with an aorist auxiliary; source-licensed for
/// `имѣти` and `хотѣти` only.
pub fn infinitival_future_aorist(
    lemma: &str,
    auxiliary: FutureInfinitiveAuxiliary,
    person: Person,
    number: Number,
    order: PhraseOrder,
) -> Result<String, Error> {
    first_phrase(
        infinitival_future_aorist_variants(lemma, auxiliary, person, number, order)?,
        lemma,
    )
}

// ---------------------------------------------------------------------------
// Declined-participle predicates (analytic passive, conditional passive,
// participial future)
// ---------------------------------------------------------------------------

/// The agreeing predicative participle: short nominative, agreeing with the
/// subject in number and gender ([`crate::participle_variants`]). The old
/// facade let the caller pass any `AdjectiveCell` and rejected everything
/// but the short nominative subject-agreeing one; the pilot signature keeps
/// only the free dimensions (kind, number, gender) and derives the rest, so
/// the case/form validation arm cannot arise. The active/passive licensing
/// of the participle kind remains a typed refusal.
fn predicative_participle_variants(
    lemma: &str,
    kind: crate::ParticipleKind,
    number: Number,
    gender: Gender,
    active: bool,
) -> Result<Vec<String>, Error> {
    use crate::ParticipleKind;
    let valid_kind = if active {
        matches!(
            kind,
            ParticipleKind::PresentActive | ParticipleKind::PastActive
        )
    } else {
        matches!(
            kind,
            ParticipleKind::PresentPassive | ParticipleKind::PastPassive
        )
    };
    if !valid_kind {
        return Err(unsupported(format!(
            "the requested analytic construction requires an {} participle",
            if active { "active" } else { "passive" }
        )));
    }
    crate::participle_variants(
        lemma,
        kind,
        Case::Nominative,
        number,
        gender,
        crate::AdjectiveForm::Short,
    )
}

macro_rules! participle_predicate_constructions {
    ($($(#[$doc:meta])* $name:ident, $variants_name:ident => $series:expr, $active:expr;)*) => {
        $(
            /// Every rendered phrase for the same cell, primary first.
            pub fn $variants_name(
                lemma: &str,
                kind: crate::ParticipleKind,
                person: Person,
                number: Number,
                gender: Gender,
                order: PhraseOrder,
            ) -> Result<Vec<String>, Error> {
                let participle =
                    predicative_participle_variants(lemma, kind, number, gender, $active)?;
                let auxiliary = copula_series_variants($series, person, number);
                Ok(phrase_variants(&ordered(auxiliary, participle, order)))
            }

            $(#[$doc])*
            ///
            /// The participle is the short nominative agreeing with the
            /// subject ([`crate::participle`]); `kind` is a parameter (a
            /// lexical-style index into the licensed participle systems),
            /// and an unlicensed kind is [`Error::UnsupportedPhrase`].
            /// `PhraseOrder::HeadFirst` puts the participle first.
            pub fn $name(
                lemma: &str,
                kind: crate::ParticipleKind,
                person: Person,
                number: Number,
                gender: Gender,
                order: PhraseOrder,
            ) -> Result<String, Error> {
                first_phrase(
                    $variants_name(lemma, kind, person, number, gender, order)?,
                    lemma,
                )
            }
        )*
    };
}

participle_predicate_constructions! {
    /// The present analytic passive: agreeing passive participle + present
    /// `ѥс-` copula (`благословленъ ѥсмь`). The old facade's
    /// `PassiveAuxiliary` enum selected the copular series — a
    /// paradigm-selecting distinction, so it became the function family
    /// [`analytic_passive`] / [`analytic_passive_imperfect`] /
    /// [`analytic_passive_aorist`] / [`analytic_passive_future`] /
    /// [`conditional_passive`] / [`conditional_passive_aorist`].
    analytic_passive, analytic_passive_variants => CopulaSeries::PresentEs, false;
    /// The analytic passive with the imperfect series of `бꙑти` (`бѣаше`).
    analytic_passive_imperfect, analytic_passive_imperfect_variants => CopulaSeries::ImperfectBe, false;
    /// The analytic passive with the aorist series of `бꙑти` (`бѣ`).
    analytic_passive_aorist, analytic_passive_aorist_variants => CopulaSeries::AoristBe, false;
    /// The future analytic passive with the future series `бѫд-`.
    analytic_passive_future, analytic_passive_future_variants => CopulaSeries::FutureBud, false;
    /// The conditional with a passive-participle predicate and the dedicated
    /// conditional series `би-` (the modal arm of the old facade's
    /// `analytic_passive`/`conditional_passive` pair).
    conditional_passive, conditional_passive_variants => CopulaSeries::ConditionalBi, false;
    /// The conditional passive with the source-described aorist replacement
    /// series `бꙑ-`.
    conditional_passive_aorist, conditional_passive_aorist_variants => CopulaSeries::ConditionalAoristBy, false;
    /// The occasional active-participle future: agreeing *active* participle
    /// + future `бѫд-` (a passive kind is [`Error::UnsupportedPhrase`]).
    participial_future, participial_future_variants => CopulaSeries::FutureBud, true;
}

// ---------------------------------------------------------------------------
// Impersonal predicates
// ---------------------------------------------------------------------------

fn impersonal_identity(lemma: &str) -> Result<ImpersonalVerbIdentity, Error> {
    ImpersonalVerbIdentity::ALL
        .into_iter()
        .find(|identity| identity.lemma() == lemma)
        .ok_or_else(|| Error::UnknownLemma(lemma.to_string()))
}

/// Resolution precedence mirroring the old facade's dictionary-first
/// dispatch: an attested residue row for the third-singular cell first (the
/// pilot's residue rows are exactly the attested cells the kernel does not
/// reproduce), then the reviewed impersonal lexeme replayed through the core
/// conjugation rules.
fn impersonal_tense_variants(lemma: &str, tense: FiniteTense) -> Result<Vec<String>, Error> {
    let identity = impersonal_identity(lemma)?;
    let finite_cell = identity.predicate_cell(tense);
    let code = crate::verb_cell_code(crate::VerbCell::Finite(finite_cell));
    let base: Vec<String> = if let Ok(index) =
        crate::generated::VERB_RESIDUE.binary_search_by(|row| (row.0, row.1).cmp(&(lemma, code)))
    {
        crate::generated::VERB_RESIDUE[index]
            .2
            .iter()
            .map(|text| (*text).to_string())
            .collect()
    } else {
        let lexeme: VerbLexeme = identity.lexeme();
        vec![
            old_church_slavonic_core::verb::finite(&lexeme, finite_cell)
                .map_err(|_| Error::Underdetermined {
                    lemma: lemma.to_string(),
                })?
                .text,
        ]
    };
    Ok(match identity.reflexive_particle() {
        Some(particle) => base
            .into_iter()
            .map(|text| format!("{text} {particle}"))
            .collect(),
        None => base,
    })
}

macro_rules! impersonal_functions {
    ($($(#[$doc:meta])* $name:ident, $variants_name:ident => $tense:expr;)*) => {
        $(
            /// Every rendered phrase for the same predicate, primary first.
            pub fn $variants_name(lemma: &str) -> Result<Vec<String>, Error> {
                impersonal_tense_variants(lemma, $tense)
            }

            $(#[$doc])*
            ///
            /// The construction always selects third-person singular.
            /// `достоꙗти` is a one-token lexically impersonal predicate;
            /// impersonal `мьнѣти` retains the independently written
            /// reflexive particle `сѧ`. Other lemmas return
            /// [`Error::UnknownLemma`].
            pub fn $name(lemma: &str) -> Result<String, Error> {
                first_phrase($variants_name(lemma)?, lemma)
            }
        )*
    };
}

impersonal_functions! {
    /// The present of a source-identified impersonal predicate
    /// (`достоитъ`, `мьнитъ сѧ`).
    impersonal_present, impersonal_present_variants => FiniteTense::Present;
    /// The imperfect of a source-identified impersonal predicate.
    impersonal_imperfect, impersonal_imperfect_variants => FiniteTense::Imperfect;
    /// The aorist of a source-identified impersonal predicate (a missing but
    /// regular aorist is reconstructed from the reviewed lexical profile).
    impersonal_aorist, impersonal_aorist_variants => FiniteTense::Aorist;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pronominal_families_cover_prefixes_postpositives_and_interposition() {
        assert_eq!(
            pronominal_family(
                "къто",
                Case::Dative,
                Some(PronominalPrefix::Ni),
                None,
                None,
                None
            )
            .as_deref(),
            Ok("никому")
        );
        assert_eq!(
            pronominal_family(
                "къто",
                Case::Nominative,
                Some(PronominalPrefix::Ne),
                None,
                None,
                None
            )
            .as_deref(),
            Ok("нѣкъто")
        );
        assert_eq!(
            pronominal_family(
                "чьто",
                Case::Accusative,
                Some(PronominalPrefix::Ni),
                Some(PronominalPostpositive::Ze),
                Some(DirectToTreatment::Retain),
                None
            )
            .as_deref(),
            Ok("ничьтоже")
        );
        assert_eq!(
            pronominal_family(
                "чьто",
                Case::Accusative,
                Some(PronominalPrefix::Ni),
                Some(PronominalPostpositive::Ze),
                Some(DirectToTreatment::Drop),
                None
            )
            .as_deref(),
            Ok("ничьже")
        );
        assert_eq!(
            pronominal_family(
                "къто",
                Case::Locative,
                Some(PronominalPrefix::Ni),
                Some(PronominalPostpositive::Ze),
                None,
                Some("о")
            )
            .as_deref(),
            Ok("ни о комьже")
        );
        assert_eq!(
            pronominal_family(
                "чьто",
                Case::Genitive,
                None,
                Some(PronominalPostpositive::Liubo),
                None,
                None
            )
            .as_deref(),
            Ok("чесо любо")
        );
        assert_eq!(
            pronominal_family(
                "къто",
                Case::Nominative,
                None,
                Some(PronominalPostpositive::Liubo),
                None,
                None
            )
            .as_deref(),
            Ok("къто любо")
        );
    }

    #[test]
    fn pronominal_family_rejects_malformed_requests() {
        // No formative at all.
        assert!(matches!(
            pronominal_family("къто", Case::Nominative, None, None, None, None),
            Err(Error::UnsupportedPhrase { .. })
        ));
        // A preposition cannot govern a nominative.
        assert!(matches!(
            pronominal_family(
                "къто",
                Case::Nominative,
                Some(PronominalPrefix::Ni),
                None,
                None,
                Some("о")
            ),
            Err(Error::UnsupportedPhrase { .. })
        ));
        // A direct -то-final base before a bound postpositive requires an
        // explicit treatment.
        assert!(matches!(
            pronominal_family(
                "къто",
                Case::Nominative,
                None,
                Some(PronominalPostpositive::Ze),
                None,
                None
            ),
            Err(Error::UnsupportedPhrase { .. })
        ));
        // The family is closed over къто/чьто.
        assert!(matches!(
            pronominal_family(
                "тъ",
                Case::Nominative,
                Some(PronominalPrefix::Ni),
                None,
                None,
                None
            ),
            Err(Error::UnknownLemma(_))
        ));
    }

    #[test]
    fn absolute_superlative_serves_both_orders_and_forms() {
        assert_eq!(
            short_absolute_superlative(
                "свѧтъ",
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::HeadFirst
            )
            .as_deref(),
            Ok("свѧтъ ѕѣло")
        );
        assert_eq!(
            short_absolute_superlative(
                "свѧтъ",
                Case::Nominative,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::DependentFirst
            )
            .as_deref(),
            Ok("ѕѣло свѧтъ")
        );
        assert_eq!(
            absolute_superlative(
                "новъ",
                Case::Genitive,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::HeadFirst
            )
            .as_deref(),
            Ok("новаѥго ѕѣло")
        );
    }

    #[test]
    fn da_imperative_covers_every_person_number_cell() {
        for number in [Number::Singular, Number::Dual, Number::Plural] {
            for person in [Person::First, Person::Second, Person::Third] {
                let phrase =
                    da_imperative("благословити", person, number).expect("da-imperative cell");
                assert!(phrase.starts_with("да "), "{phrase}");
            }
        }
        assert_eq!(
            da_imperative("благословити", Person::First, Number::Singular).as_deref(),
            Ok("да благословлѭ")
        );
        assert_eq!(
            da_imperative("благословити", Person::Third, Number::Plural).as_deref(),
            Ok("да благословѧтъ")
        );
    }

    #[test]
    fn copular_series_match_the_reviewed_goldens() {
        assert_eq!(copula_present(Person::First, Number::Singular), "ѥсмь");
        assert_eq!(copula_future(Person::Third, Number::Plural), "бѫдѫтъ");
        assert_eq!(copula_conditional(Person::First, Number::Dual), "бивѣ");
        assert_eq!(
            copula_conditional_variants(Person::Third, Number::Plural),
            vec!["бѫ".to_string(), "бишѧ".to_string()]
        );
        assert_eq!(
            copula_conditional_aorist(Person::Second, Number::Singular),
            "бꙑ"
        );
        assert_eq!(copula_imperfect(Person::Third, Number::Singular), "бѣаше");
        assert_eq!(copula_aorist(Person::Third, Number::Plural), "бѣшѧ");
    }

    #[test]
    fn l_participle_periphrases_agree_with_the_old_goldens() {
        assert_eq!(
            perfect(
                "благословити",
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::HeadFirst
            )
            .as_deref(),
            Ok("благословилъ ѥсмь")
        );
        assert_eq!(
            future_perfect(
                "благословити",
                Person::Third,
                Number::Singular,
                Gender::Feminine,
                PhraseOrder::DependentFirst
            )
            .as_deref(),
            Ok("бѫдетъ благословила")
        );
        assert_eq!(
            pluperfect(
                "благословити",
                Person::Third,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::HeadFirst
            )
            .as_deref(),
            Ok("благословилъ бѣаше")
        );
        assert_eq!(
            pluperfect_perfect(
                "благословити",
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::HeadFirst
            )
            .as_deref(),
            Ok("благословилъ бꙑлъ ѥсмь")
        );
        assert_eq!(
            da_conditional_optative(
                "благословити",
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::DependentFirst
            )
            .as_deref(),
            Ok("да бимь благословилъ")
        );
    }

    #[test]
    fn infinitival_future_enforces_the_past_reference_license() {
        assert_eq!(
            infinitival_future(
                "благословити",
                FutureInfinitiveAuxiliary::Imeti,
                Person::Third,
                Number::Plural,
                PhraseOrder::DependentFirst
            )
            .as_deref(),
            Ok("имѫтъ благословити")
        );
        assert!(matches!(
            infinitival_future_imperfect(
                "благословити",
                FutureInfinitiveAuxiliary::Vochati,
                Person::First,
                Number::Singular,
                PhraseOrder::DependentFirst
            ),
            Err(Error::UnsupportedPhrase { .. })
        ));
        assert!(
            infinitival_future_aorist(
                "благословити",
                FutureInfinitiveAuxiliary::Khoteti,
                Person::First,
                Number::Dual,
                PhraseOrder::DependentFirst
            )
            .is_ok()
        );
    }

    #[test]
    fn impersonal_predicates_keep_lexical_and_reflexive_structures_distinct() {
        assert_eq!(impersonal_present("достоꙗти").as_deref(), Ok("достоитъ"));
        assert_eq!(impersonal_present("мьнѣти").as_deref(), Ok("мьнитъ сѧ"));
        assert_eq!(impersonal_aorist("достоꙗти").as_deref(), Ok("достоꙗ"));
        assert_eq!(impersonal_imperfect("достоꙗти").as_deref(), Ok("достоꙗаше"));
        assert!(matches!(
            impersonal_present("благословити"),
            Err(Error::UnknownLemma(_))
        ));
    }

    #[test]
    fn declined_participle_predicates() {
        use crate::ParticipleKind;
        assert_eq!(
            analytic_passive(
                "благословити",
                ParticipleKind::PastPassive,
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::DependentFirst,
            )
            .as_deref(),
            Ok("ѥсмь благословлѥнъ")
        );
        assert_eq!(
            conditional_passive_aorist(
                "любити",
                ParticipleKind::PresentPassive,
                Person::Second,
                Number::Singular,
                Gender::Feminine,
                PhraseOrder::HeadFirst,
            )
            .as_deref(),
            Ok("любима бꙑ")
        );
        assert_eq!(
            participial_future(
                "творити",
                ParticipleKind::PresentActive,
                Person::Third,
                Number::Plural,
                Gender::Masculine,
                PhraseOrder::DependentFirst,
            )
            .as_deref(),
            Ok("бѫдѫтъ творѧште")
        );
    }

    #[test]
    fn declined_participle_predicates_enforce_kind_licensing() {
        use crate::ParticipleKind;
        // A passive construction refuses active kinds; the participial
        // future refuses passive kinds.
        assert!(matches!(
            analytic_passive(
                "благословити",
                ParticipleKind::PresentActive,
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::DependentFirst,
            ),
            Err(Error::UnsupportedPhrase { .. })
        ));
        assert!(matches!(
            conditional_passive(
                "благословити",
                ParticipleKind::PastActive,
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::DependentFirst,
            ),
            Err(Error::UnsupportedPhrase { .. })
        ));
        assert!(matches!(
            participial_future(
                "благословити",
                ParticipleKind::PastPassive,
                Person::First,
                Number::Singular,
                Gender::Masculine,
                PhraseOrder::DependentFirst,
            ),
            Err(Error::UnsupportedPhrase { .. })
        ));
        // A reviewed participle defect propagates as Underdetermined.
        assert!(matches!(
            analytic_passive(
                "ити",
                ParticipleKind::PastPassive,
                Person::First,
                Number::Plural,
                Gender::Masculine,
                PhraseOrder::DependentFirst,
            ),
            Err(Error::Underdetermined { .. })
        ));
    }
}
