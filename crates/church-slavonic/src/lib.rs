//! Church Slavonic inflection facade (rewrite pilot slice): a pure rule
//! kernel plus a compact irregular residue table, replacing the 24 MB
//! generated registry for nouns.
//!
//! Resolution precedence for a requested cell is exactly one channel deep:
//!
//! 1. the generated residue table (`generated/noun_residue.rs`), which holds
//!    only the attested cells the rule kernel does not reproduce verbatim;
//! 2. the rule kernel (`old-church-slavonic-core`), driven by a compact
//!    per-lemma metadata table (noun class, gender, animacy, number
//!    restriction) generated from the extracted lexeme inventory.
//!
//! Unknown lemmas produce [`Error::UnknownLemma`].
//!
//! # Oracle conventions (documented design choices)
//!
//! The generated tables are validated against every attested noun cell in
//! `data/extracted` (`cargo xtask rewrite-pilot-accuracy`). Two conventions
//! were required to reproduce the oracle at 100%:
//!
//! - **Homographs (deterministic numeric suffixes).** The extracted
//!   inventory contains lemmas with more than one lexeme entry (ten noun
//!   lemmas, e.g. `градъ`, `ногъть`, `сꙑнъ`; four verb pairs). Mirroring the
//!   `gold-silver-copper/english` crate's scheme, each such lexeme gets its
//!   own key: the bare lemma serves the default sense, and the others are
//!   reachable as `lemma_2`, `lemma_3`, … Sense numbering is decided by a
//!   pure deterministic sort of each lexeme's emitted form inventory — the
//!   sorted sequence of `(cell code, variant list)` pairs, compared
//!   lexicographically, with the encoded lexeme metadata as tie-break — so
//!   the assignment needs no external lockfile and is reproducible from the
//!   data alone across refreshes (two lexemes that tie under this sort have
//!   identical inventories and metadata, so their relative order cannot
//!   change any emitted table). A lookup for the bare lemma answers with
//!   only that lexeme's own variants (not a union across senses);
//!   [`base_lemma`] strips the suffix before any rule-kernel derivation.
//! - **Animacy.** Masculine accusative cells are animacy-conditioned. Where
//!   the extracted metadata carries no animacy fact for a class with an
//!   animacy contrast, the rules cannot commit, so those attested
//!   accusative cells ship in the residue table verbatim (the stored tables
//!   keep the plain accusative). Elsewhere animacy defaults to inanimate,
//!   matching the stored declension tables.
//!
//! # Adjectives
//!
//! [`adjective`] serves the long (definite) declension and
//! [`short_adjective`] the short (indefinite) one — a paradigm-selecting
//! distinction becomes a function, not an enum parameter. Two dimensions of
//! the extracted adjective cells are deliberately not parameters:
//!
//! - **Animacy is degenerate in the attested oracle.** The extracted
//!   adjective cells are keyed with an animacy dimension, but for every
//!   attested (lemma, form, case, number, gender) cell the animate and
//!   inanimate stored variant lists are byte-identical (the stored tables
//!   keep the plain accusative; the residue generator fails if this ever
//!   stops holding). Taking an `Animacy` parameter would therefore promise a
//!   distinction the data does not make, so these functions do not take one;
//!   the kernel is driven with the inanimate convention, and any cell where
//!   the kernel's animate-accusative convention would diverge from the
//!   stored tables ships in the residue table verbatim (the oracle decides).
//! - **Positive degree only.** The extracted inventory stores comparative
//!   *citations* only (`adj:comparative:citation`), and those carry
//!   unpredictable lexical facts (suppletion `велии` → `болии`, old vs new
//!   suffix grade `дражии` / `дражаи`), not a productive stem the kernel's
//!   `productive_new_comparative` can commit to. Comparatives are excluded
//!   from these functions and from the pilot accuracy denominator; the
//!   accuracy gate prints the excluded count.
//!
//! # Verbs
//!
//! Paradigm-selecting distinctions become functions in the ruthenian style:
//! [`present`], [`imperfect`], and [`aorist`] each serve one finite tense
//! ([`Person`], [`Number`] index within it); [`imperative`] serves the
//! imperative ([`Person`], [`Number`]); [`l_participle`] the resultative
//! l-participle ([`Gender`], [`Number`]). [`infinitive`], [`supine`], and
//! [`verbal_noun`] take only the lemma; the oracle stores the verbal noun as
//! its nominative-singular citation only, so that is what the function
//! returns. The four attested participle kinds are derivation-style citation
//! functions ([`present_active_participle`], [`present_passive_participle`],
//! [`past_active_participle`], [`past_passive_participle`]), each returning
//! the citation form (masculine nominative singular, short/indefinite) the
//! oracle stores under `verb:participle:<kind>:citation`. Every function has
//! a `*_variants` companion; no attested verb cell kind is excluded.
//!
//! Resolution precedence mirrors the nouns: the generated residue table
//! (`generated/verb_residue.rs`) first, then the rule kernel. The kernel path
//! is (a) the reviewed unique/irregular verb identity kernels, keyed by
//! lemma, then (b) the per-lemma principal-part metadata table (stems and
//! formation codes distilled from `verb_metadata.tsv`), replayed through the
//! core conjugation rules with the same multi-analysis merge the resolver's
//! dictionary-metadata generators use. `verb_metadata.tsv` covers only a
//! minority of the 711 attested verbs, so the verb residue table is
//! proportionally larger than the noun one; the oracle referee keeps the
//! facade at 100% either way. The homograph convention is the nouns'
//! deterministic numeric-suffix scheme: the four verb homograph pairs
//! (`вести`, `пасти`, `привести`, `съпасти` — transitive/intransitive or
//! lexical doublets sharing a citation) are served per lexeme, the default
//! sense under the bare lemma and the other under `lemma_2`, each with its
//! own principal-part metadata.
//!
//! # Closed classes (pronouns, numerals, determiners)
//!
//! The remaining attested parts of speech are closed inventories (29 pronoun
//! lexemes, 8 numeral lexemes, 1 determiner). Resolution precedence mirrors
//! the open classes: the generated residue table
//! (`generated/closed_residue.rs`) first, then the shared closed-class
//! identity-kernel dispatch ([`kernel_closed_variants`], the same
//! `#[doc(hidden)]` helper the derivability harness replays), which routes
//! each lemma to the reviewed Rust-encoded paradigms (personal/reflexive/
//! anaphoric pronouns, standard pronominal declension, `иже`, `сь`, `кꙑи`,
//! interrogatives, cardinal and ordinal numeral identities).
//!
//! The attested cells come in exactly three key shapes, and the API follows
//! them honestly:
//!
//! - **Person-indexed cells** (`decl:pron:<case>:<number>:<1|2>`). The
//!   extracted inventory duplicates one and the same personal-pronoun table
//!   (both persons, all numbers) under thirteen lemmas: the personal lemmas
//!   themselves (`азъ`, `тꙑ`, `мꙑ`, `вꙑ`, `вѣ`, `ва`, `наю`, `ваю`) and the
//!   five possessives (`мои`, `твои`, `свои`, `нашь`, `вашь`). Every
//!   duplicate is byte-identical per (person, number, case) key (the
//!   accuracy gate proves this cell by cell), so the attested personal cells
//!   are exactly person x number x case, and OCS has no gendered 1st/2nd
//!   person: [`pronoun`]`(person, number, case)` is the whole surface. It is
//!   served under the canonical lemmas `азъ` (first person) and `тꙑ`
//!   (second); the duplicated tables under the other eleven lemmas are the
//!   cells the kernels cannot serve (the kernel's intrinsic-person guard
//!   refuses a second-person request against `азъ`, and refuses possessive
//!   lemmas outright), so they ship in the residue table verbatim, keyed by
//!   their own lemma, and stay reachable through the lemma-keyed gate path.
//!   [`Person::Third`] is not a personal-pronoun value in this inventory
//!   (the third person is the gendered anaphoric series) and returns
//!   [`Error::Underdetermined`].
//! - **Gender-indexed cells** (`decl:pron:<case>:<number>:<m|f|n>`). All
//!   lexically identified: the anaphoric family (`и`, `ѥ`, `ѭ`, `ими` — four
//!   spellings of one identical table), the demonstrative family (`онъ`,
//!   `она`, `оно` — likewise one table), `тъ`, `иже`, `сь`, `вьсѣкъ`, the
//!   gendered halves of the possessives, the ordinal `прьвъ`, and the
//!   determiner `кꙑи`. Served by [`pronoun_form`] / [`numeral_form`] /
//!   [`determiner_form`]`(lemma, case, number, gender)`, where gender is a
//!   key dimension (must-match). [`anaphoric`]`(case, number, gender)` is
//!   the person-free third-person entry point, canonicalized to lemma `и`.
//! - **Bare cells** (`decl:<pos>:<case>:<number>`). The reflexive `сѧ`
//!   (whose attested number dimension is fully degenerate — the singular,
//!   dual, and plural rows are identical, and the accuracy gate replays all
//!   fifteen cells through the numberless [`reflexive`]`(case)`), the
//!   genderless interrogatives `къто`/`чьто`/`никъто`, the indefinite
//!   `етеръ`, and the non-ordinal numerals (`пѧть` … `десѧть`, plus the
//!   mistagged proper noun `Єѵрѡпа` the source data files under `num`).
//!   These lexemes' cells lack a gender dimension, so the `gender` parameter
//!   of the `*_form` functions is **ignored** for them (documented rather
//!   than must-match: the request cannot fail on a distinction the data does
//!   not draw).
//!
//! A lemma whose only attested cells are the shared person-indexed table
//! (`азъ`, `ва`, `наю`, `вашь`, …) has no case x number (x gender) table of
//! its own, so [`pronoun_form`] returns [`Error::Underdetermined`] for it;
//! its cells are served by [`pronoun`]. `етеръ` (no reviewed kernel) and
//! `Єѵрѡпа` are served entirely from the residue table verbatim.

use old_church_slavonic_core::adjective::AdjectiveLexeme;
use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::unique_noun::UniqueNounFamilyMember;
use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveClass, AnaphoricEnvironment, Animacy, AoristFormation,
    CardinalNumeralIdentity, FiniteTense, FiniteVerbCell, ImperativeCell, ImperativeFormation,
    ImperfectFormation, ImperfectVariantPolicy, InterrogativePronounIdentity,
    IrregularAgreeingIdentity, IrregularVerbFamilyMember, LParticipleCell, NounCell, NounClass,
    NumberRestriction, NumeralCell, OrdinalNumeralIdentity, PartOfSpeech, ParticipleCell,
    ParticipleKind, PastActiveParticipleFormation, PastPassiveParticipleFormation,
    PersonalPronounIdentity, PresentActiveParticipleFormation, PresentFormation,
    PresentPassiveParticipleFormation, PronominalPrefix, PronounFormSelection,
    StandardPronominalIdentity, TwofoldNounFamilyMember, UniqueVerbFamilyMember, VerbAspect,
    VerbClass, numeral, orthography, pronoun,
};

pub use church_slavonic_core::grammar::{AdjectiveForm, Case, Gender, Number, Person};

mod paradigm;
pub use paradigm::{
    AdjectiveParadigm, ClosedParadigm, NounParadigm, VerbCellKind, VerbParadigm,
    adjective_paradigm, determiner_form_paradigm, noun_paradigm,
    numeral_form_paradigm, pronoun_form_paradigm, verb_paradigm,
};

mod generated {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/noun_residue.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/adjective_residue.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/verb_residue.rs"
    ));
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/generated/closed_residue.rs"
    ));
}

/// Facade error type.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The lemma is not in the per-lemma metadata table and has no residue
    /// rows: the facade knows nothing about it.
    UnknownLemma(String),
    /// The lemma is known, but its metadata does not determine this cell
    /// (missing class metadata, an animacy-conditioned masculine accusative
    /// without an animacy fact, a number-restricted paradigm, or a kernel
    /// defect). Attested cells never reach this arm; it marks unattested
    /// requests the rules cannot commit to.
    Underdetermined { lemma: String },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnknownLemma(lemma) => write!(f, "unknown lemma `{lemma}`"),
            Error::Underdetermined { lemma } => {
                write!(f, "cell underdetermined for lemma `{lemma}`")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Compact per-lemma noun facts decoded from the generated metadata table.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NounMeta {
    pub class: Option<NounClass>,
    pub gender: Option<Gender>,
    pub animacy: Option<Animacy>,
    pub restriction: NumberRestriction,
}

/// Stable numeric key for one (case, number) cell, used by the generated
/// residue table. `case_index * 3 + number_index`.
#[doc(hidden)]
pub fn cell_code(case: Case, number: Number) -> u8 {
    let case = match case {
        Case::Nominative => 0u8,
        Case::Genitive => 1,
        Case::Dative => 2,
        Case::Accusative => 3,
        Case::Instrumental => 4,
        Case::Locative => 5,
        Case::Vocative => 6,
    };
    let number = match number {
        Number::Singular => 0u8,
        Number::Dual => 1,
        Number::Plural => 2,
    };
    case * 3 + number
}

fn decode_class(code: u8) -> Option<NounClass> {
    Some(match code {
        1 => NounClass::OMasculineHard,
        2 => NounClass::ONeuterHard,
        3 => NounClass::AHard,
        4 => NounClass::JoMasculineSoft,
        5 => NounClass::JoNeuterSoft,
        6 => NounClass::JaSoft,
        7 => NounClass::IFeminine,
        8 => NounClass::IMasculine,
        9 => NounClass::UMasculine,
        10 => NounClass::NMasculine,
        11 => NounClass::NNeuter,
        12 => NounClass::NtNeuter,
        13 => NounClass::RStem,
        14 => NounClass::SNeuter,
        15 => NounClass::VFeminine,
        _ => return None,
    })
}

fn decode_meta(row: &(&str, u8, u8, u8, u8)) -> NounMeta {
    NounMeta {
        class: decode_class(row.1),
        gender: match row.2 {
            1 => Some(Gender::Masculine),
            2 => Some(Gender::Feminine),
            3 => Some(Gender::Neuter),
            _ => None,
        },
        animacy: match row.3 {
            1 => Some(Animacy::Animate),
            2 => Some(Animacy::Inanimate),
            _ => None,
        },
        restriction: match row.4 {
            1 => NumberRestriction::SingularOnly,
            2 => NumberRestriction::DualOnly,
            3 => NumberRestriction::PluralOnly,
            _ => NumberRestriction::All,
        },
    }
}

/// Strip a numeric homograph suffix (`_2`, `_3`, …) from a lemma key,
/// returning the surface lemma the rule kernel inflects. Keys without a
/// suffix are returned unchanged. See the module docs ("Homographs") for the
/// deterministic numbering scheme.
pub fn base_lemma(lemma: &str) -> &str {
    if let Some((base, suffix)) = lemma.rsplit_once('_') {
        if !base.is_empty() && !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
        {
            return base;
        }
    }
    lemma
}

/// Look up the compact metadata record for a lemma.
#[doc(hidden)]
pub fn noun_meta(lemma: &str) -> Option<NounMeta> {
    generated::NOUN_META
        .binary_search_by(|row| row.0.cmp(lemma))
        .ok()
        .map(|index| decode_meta(&generated::NOUN_META[index]))
}

/// Rule-kernel prediction for one noun cell, given per-lemma metadata.
/// Reviewed unique/twofold identities take precedence over class-driven
/// declension, mirroring the resolver's reviewed-profile dispatch. Returns
/// `None` when the metadata does not determine the cell.
#[doc(hidden)]
pub fn kernel_noun_variants(
    lemma: &str,
    meta: &NounMeta,
    case: Case,
    number: Number,
) -> Option<Vec<String>> {
    let cell = NounCell { case, number };
    if let Some(member) = UniqueNounFamilyMember::classify_source_lemma(lemma) {
        if let Some(texts) = member.decline(cell).ok().and_then(|variants| {
            let mut texts: Vec<String> = Vec::new();
            for variant in variants {
                let text = orthography::canonical_display(&variant.prediction.text).ok()?;
                if !texts.contains(&text) {
                    texts.push(text);
                }
            }
            (!texts.is_empty()).then_some(texts)
        }) {
            return Some(texts);
        }
    }
    if let Some(member) = TwofoldNounFamilyMember::classify_source_lemma(lemma) {
        if let Some(texts) = single(old_church_slavonic_core::noun::decline(
            &member.lexeme(),
            cell,
        )) {
            return Some(texts);
        }
    }
    let class = meta.class?;
    let gender = meta.gender.or_else(|| class.intrinsic_gender())?;
    if meta.animacy.is_none()
        && class.has_animacy_contrast()
        && gender == Gender::Masculine
        && case == Case::Accusative
    {
        // Animacy-conditioned cell with no animacy fact: the rules cannot
        // commit; attested cells of this shape live in the residue table.
        return None;
    }
    let noun = NounLexeme {
        lemma: lemma.to_string(),
        class,
        gender,
        animacy: meta.animacy.unwrap_or(Animacy::Inanimate),
        number_restriction: meta.restriction,
    };
    single(old_church_slavonic_core::noun::decline(&noun, cell))
}

fn single(
    predicted: Result<
        old_church_slavonic_core::PredictedForm,
        old_church_slavonic_core::InflectionError,
    >,
) -> Option<Vec<String>> {
    let predicted = predicted.ok()?;
    Some(vec![orthography::canonical_display(&predicted.text).ok()?])
}

/// All attested-or-predicted surface variants for one noun cell, primary
/// variant first. Residue table first, rule kernel second.
pub fn noun_variants(lemma: &str, case: Case, number: Number) -> Result<Vec<String>, Error> {
    let code = cell_code(case, number);
    if let Ok(index) =
        generated::NOUN_RESIDUE.binary_search_by(|row| (row.0, row.1).cmp(&(lemma, code)))
    {
        return Ok(generated::NOUN_RESIDUE[index]
            .2
            .iter()
            .map(|text| (*text).to_string())
            .collect());
    }
    let meta = noun_meta(lemma).ok_or_else(|| Error::UnknownLemma(lemma.to_string()))?;
    kernel_noun_variants(base_lemma(lemma), &meta, case, number).ok_or_else(|| {
        Error::Underdetermined {
            lemma: lemma.to_string(),
        }
    })
}

/// The primary surface form for one noun cell.
pub fn noun(lemma: &str, case: Case, number: Number) -> Result<String, Error> {
    let mut variants = noun_variants(lemma, case, number)?;
    if variants.is_empty() {
        return Err(Error::Underdetermined {
            lemma: lemma.to_string(),
        });
    }
    Ok(variants.remove(0))
}

/// Stable numeric key for one adjective (form, case, number, gender) cell,
/// used by the generated residue table. Animacy is not part of the key: the
/// attested oracle is animacy-degenerate (see the module docs).
/// `form_index * 63 + cell_code(case, number) * 3 + gender_index`.
#[doc(hidden)]
pub fn adjective_cell_code(form: AdjectiveForm, case: Case, number: Number, gender: Gender) -> u8 {
    let form = match form {
        AdjectiveForm::Short => 0u8,
        AdjectiveForm::Long => 1,
    };
    let gender = match gender {
        Gender::Masculine => 0u8,
        Gender::Feminine => 1,
        Gender::Neuter => 2,
    };
    form * 63 + cell_code(case, number) * 3 + gender
}

fn decode_adjective_class(code: u8) -> Option<AdjectiveClass> {
    Some(match code {
        1 => AdjectiveClass::Hard,
        2 => AdjectiveClass::Soft,
        _ => return None,
    })
}

/// Look up the compact class code for an adjective lemma.
#[doc(hidden)]
pub fn adjective_class(lemma: &str) -> Option<Option<AdjectiveClass>> {
    generated::ADJ_META
        .binary_search_by(|row| row.0.cmp(lemma))
        .ok()
        .map(|index| decode_adjective_class(generated::ADJ_META[index].1))
}

/// Rule-kernel prediction for one adjective cell. The kernel is driven with
/// the inanimate convention (the oracle's animacy dimension is degenerate;
/// see the module docs). Returns `None` when the class is unknown or the
/// kernel reports a defect (e.g. a short cell of a long-only adjective).
#[doc(hidden)]
pub fn kernel_adjective_variants(
    lemma: &str,
    class: Option<AdjectiveClass>,
    form: AdjectiveForm,
    case: Case,
    number: Number,
    gender: Gender,
) -> Option<Vec<String>> {
    let class = class?;
    let adjective = AdjectiveLexeme {
        lemma: lemma.to_string(),
        class,
    };
    let cell = AdjectiveCell {
        case,
        number,
        gender,
        animacy: Animacy::Inanimate,
        form,
    };
    single(old_church_slavonic_core::adjective::decline(
        &adjective, cell,
    ))
}

pub(crate) fn adjective_form_variants(
    lemma: &str,
    form: AdjectiveForm,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<Vec<String>, Error> {
    let code = adjective_cell_code(form, case, number, gender);
    if let Ok(index) =
        generated::ADJ_RESIDUE.binary_search_by(|row| (row.0, row.1).cmp(&(lemma, code)))
    {
        return Ok(generated::ADJ_RESIDUE[index]
            .2
            .iter()
            .map(|text| (*text).to_string())
            .collect());
    }
    let class = adjective_class(lemma).ok_or_else(|| Error::UnknownLemma(lemma.to_string()))?;
    kernel_adjective_variants(base_lemma(lemma), class, form, case, number, gender).ok_or_else(|| {
        Error::Underdetermined {
            lemma: lemma.to_string(),
        }
    })
}

fn primary(mut variants: Vec<String>, lemma: &str) -> Result<String, Error> {
    if variants.is_empty() {
        return Err(Error::Underdetermined {
            lemma: lemma.to_string(),
        });
    }
    Ok(variants.remove(0))
}

/// All variants for one long (definite) adjective cell, primary first.
pub fn adjective_variants(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<Vec<String>, Error> {
    adjective_form_variants(lemma, AdjectiveForm::Long, case, number, gender)
}

/// The primary surface form for one long (definite) adjective cell.
pub fn adjective(lemma: &str, case: Case, number: Number, gender: Gender) -> Result<String, Error> {
    primary(adjective_variants(lemma, case, number, gender)?, lemma)
}

/// All variants for one short (indefinite) adjective cell, primary first.
pub fn short_adjective_variants(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<Vec<String>, Error> {
    adjective_form_variants(lemma, AdjectiveForm::Short, case, number, gender)
}

/// The primary surface form for one short (indefinite) adjective cell.
pub fn short_adjective(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<String, Error> {
    primary(
        short_adjective_variants(lemma, case, number, gender)?,
        lemma,
    )
}

/// Compact per-lemma verb principal-part metadata decoded from the generated
/// table. Field encodings (0 always means "absent" where the field is
/// optional):
///
/// - `aspect`: 1 perfective, 2 imperfective, 3 biaspectual.
/// - `present` analyses: (stem, class, first-singular stem, third-plural
///   stem, formation) with class 1 IA1 … 7 irregular and formation 1
///   iotated-e, 2 hard-i; empty strings mean "no independent allomorph".
/// - `imperfect` analyses: (stem, formation 1 a / 2 yat-a / 3 palatalized-a /
///   4 present-a / 5 present-yat-a, variant policy 1 uncontracted-only /
///   2 contracted-only / 3 iotated-only).
/// - `aorist` analyses: (stem, 2/3sg principal part or empty, formation 1
///   asigmatic … 5 new).
/// - `imperative` / participle analyses: (stem, formation code in the order
///   the core enum declares its variants, 1-based).
/// - `l_participle`: bare stems.
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct VerbMeta<'a> {
    pub aspect: u8,
    pub present: &'a [(&'a str, u8, &'a str, &'a str, u8)],
    pub imperfect: &'a [(&'a str, u8, u8)],
    pub aorist: &'a [(&'a str, &'a str, u8)],
    pub imperative: &'a [(&'a str, u8)],
    pub l_participle: &'a [&'a str],
    pub present_active_participle: &'a [(&'a str, u8)],
    pub present_passive_participle: &'a [(&'a str, u8)],
    pub past_active_participle: &'a [(&'a str, u8)],
    pub past_passive_participle: &'a [(&'a str, u8)],
}

/// One requested verb cell, spanning every attested verb cell kind.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbCell {
    Finite(FiniteVerbCell),
    Imperative(ImperativeCell),
    LParticiple(LParticipleCell),
    Infinitive,
    Supine,
    VerbalNoun,
    ParticipleCitation(ParticipleKind),
}

fn person_index(person: Person) -> u8 {
    match person {
        Person::First => 0,
        Person::Second => 1,
        Person::Third => 2,
    }
}

fn number_index(number: Number) -> u8 {
    match number {
        Number::Singular => 0,
        Number::Dual => 1,
        Number::Plural => 2,
    }
}

fn gender_index(gender: Gender) -> u8 {
    match gender {
        Gender::Masculine => 0,
        Gender::Feminine => 1,
        Gender::Neuter => 2,
    }
}

/// Stable numeric key for one verb cell, used by the generated residue
/// table. Finite cells occupy 0..27 (`tense * 9 + person * 3 + number`),
/// imperatives 27..36, l-participles 36..45 (`gender * 3 + number`),
/// infinitive/supine/verbal-noun 45/46/47, participle citations 48..52.
#[doc(hidden)]
pub fn verb_cell_code(cell: VerbCell) -> u8 {
    match cell {
        VerbCell::Finite(cell) => {
            let tense = match cell.tense {
                FiniteTense::Present => 0u8,
                FiniteTense::Imperfect => 1,
                FiniteTense::Aorist => 2,
            };
            tense * 9 + person_index(cell.person) * 3 + number_index(cell.number)
        }
        VerbCell::Imperative(cell) => {
            27 + person_index(cell.person) * 3 + number_index(cell.number)
        }
        VerbCell::LParticiple(cell) => {
            36 + gender_index(cell.gender) * 3 + number_index(cell.number)
        }
        VerbCell::Infinitive => 45,
        VerbCell::Supine => 46,
        VerbCell::VerbalNoun => 47,
        VerbCell::ParticipleCitation(kind) => {
            48 + match kind {
                ParticipleKind::PresentActive => 0u8,
                ParticipleKind::PresentPassive => 1,
                ParticipleKind::PastActive => 2,
                ParticipleKind::PastPassive => 3,
            }
        }
    }
}

/// Look up the compact metadata record for a verb lemma.
#[doc(hidden)]
pub fn verb_meta(lemma: &str) -> Option<VerbMeta<'static>> {
    generated::VERB_META
        .binary_search_by(|row| row.0.cmp(lemma))
        .ok()
        .map(|index| generated::VERB_META[index].1)
}

fn decode_verb_class(code: u8) -> VerbClass {
    match code {
        1 => VerbClass::IA1,
        2 => VerbClass::IA2,
        3 => VerbClass::II1,
        4 => VerbClass::II2,
        5 => VerbClass::II3,
        6 => VerbClass::Root,
        _ => VerbClass::Irregular,
    }
}

fn decode_aspect(code: u8) -> Option<VerbAspect> {
    Some(match code {
        1 => VerbAspect::Perfective,
        2 => VerbAspect::Imperfective,
        3 => VerbAspect::Biaspectual,
        _ => return None,
    })
}

fn optional_stem(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_string())
}

fn participle_citation_cell(kind: ParticipleKind) -> ParticipleCell {
    ParticipleCell {
        kind,
        adjective: AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Short,
        },
    }
}

fn verbal_noun_citation_cell() -> NounCell {
    NounCell {
        case: Case::Nominative,
        number: Number::Singular,
    }
}

/// One rule-kernel prediction for a fully assembled verb lexeme profile.
/// Surface text is used as produced (the resolver's `*_with` and
/// dictionary-metadata generators do not re-canonicalize predicted text).
fn verb_lexeme_prediction(verb: &VerbLexeme, cell: VerbCell) -> Option<String> {
    let predicted = match cell {
        VerbCell::Finite(cell) => old_church_slavonic_core::verb::finite(verb, cell),
        VerbCell::Imperative(cell) => old_church_slavonic_core::verb::imperative(verb, cell),
        VerbCell::LParticiple(cell) => old_church_slavonic_core::verb::l_participle(verb, cell),
        VerbCell::Infinitive => old_church_slavonic_core::verb::infinitive(verb),
        VerbCell::Supine => old_church_slavonic_core::verb::supine(verb),
        VerbCell::VerbalNoun => {
            old_church_slavonic_core::verb::verbal_noun(verb, verbal_noun_citation_cell())
        }
        VerbCell::ParticipleCitation(kind) => {
            old_church_slavonic_core::verb::participle(verb, participle_citation_cell(kind))
        }
    };
    Some(predicted.ok()?.text)
}

/// Reviewed unique/irregular verb identity kernels, keyed by lemma.
fn identity_verb_variants(lemma: &str, cell: VerbCell) -> Option<Vec<String>> {
    let verb = UniqueVerbFamilyMember::classify_source_union_lemma(lemma)
        .map(|member| member.lexeme())
        .or_else(|| {
            IrregularVerbFamilyMember::classify_source_lemma(lemma).map(|member| member.lexeme())
        })?;
    verb_lexeme_prediction(&verb, cell).map(|text| vec![text])
}

/// The resolver's `metadata_verb` base profile: class from the first present
/// analysis (irregular otherwise), plus the aspect fact.
fn base_verb_lexeme(lemma: &str, meta: &VerbMeta<'_>) -> VerbLexeme {
    let class = meta
        .present
        .first()
        .map_or(VerbClass::Irregular, |analysis| {
            decode_verb_class(analysis.1)
        });
    let mut lexeme = VerbLexeme::new(lemma, class);
    lexeme.aspect = decode_aspect(meta.aspect);
    lexeme
}

/// Replay one analysis list through the core rules and merge the predicted
/// texts in analysis order (stable, duplicates dropped), mirroring the
/// resolver's `metadata_form_set`. Any failing analysis fails the cell.
fn merge_analyses<T>(
    analyses: &[T],
    mut predict: impl FnMut(&T) -> Option<String>,
) -> Option<Vec<String>> {
    if analyses.is_empty() {
        return None;
    }
    let mut texts: Vec<String> = Vec::new();
    for analysis in analyses {
        let text = predict(analysis)?;
        if !texts.contains(&text) {
            texts.push(text);
        }
    }
    Some(texts)
}

/// Principal-part-metadata-driven verb prediction, mirroring the resolver's
/// dictionary-metadata generators over the compact per-lemma table.
fn metadata_verb_variants(lemma: &str, meta: &VerbMeta<'_>, cell: VerbCell) -> Option<Vec<String>> {
    match cell {
        VerbCell::Infinitive | VerbCell::Supine => {
            let verb = base_verb_lexeme(lemma, meta);
            verb_lexeme_prediction(&verb, cell).map(|text| vec![text])
        }
        VerbCell::Finite(finite) => match finite.tense {
            FiniteTense::Present => merge_analyses(meta.present, |analysis| {
                let (stem, class, first_singular, third_plural, formation) = *analysis;
                let mut verb = VerbLexeme::new(lemma, decode_verb_class(class));
                verb.aspect = decode_aspect(meta.aspect);
                verb.stems.present = Some(stem.to_string());
                verb.stems.present_first_singular = optional_stem(first_singular);
                verb.stems.present_third_plural = optional_stem(third_plural);
                verb.formations.present = match formation {
                    1 => Some(PresentFormation::IotatedE),
                    2 => Some(PresentFormation::HardI),
                    _ => None,
                };
                verb_lexeme_prediction(&verb, cell)
            }),
            FiniteTense::Imperfect => merge_analyses(meta.imperfect, |analysis| {
                let (stem, formation, policy) = *analysis;
                let mut verb = base_verb_lexeme(lemma, meta);
                verb.stems.imperfect = Some(stem.to_string());
                verb.formations.imperfect = Some(match formation {
                    1 => ImperfectFormation::A,
                    2 => ImperfectFormation::YatA,
                    3 => ImperfectFormation::PalatalizedA,
                    4 => ImperfectFormation::PresentA,
                    _ => ImperfectFormation::PresentYatA,
                });
                verb.formations.imperfect_variant_policy = Some(match policy {
                    1 => ImperfectVariantPolicy::UncontractedOnly,
                    2 => ImperfectVariantPolicy::ContractedOnly,
                    _ => ImperfectVariantPolicy::IotatedOnly,
                });
                verb_lexeme_prediction(&verb, cell)
            }),
            FiniteTense::Aorist => merge_analyses(meta.aorist, |analysis| {
                let (stem, second_third_singular, formation) = *analysis;
                let mut verb = base_verb_lexeme(lemma, meta);
                verb.stems.aorist = Some(stem.to_string());
                verb.stems.aorist_second_third_singular = optional_stem(second_third_singular);
                verb.formations.aorist = Some(match formation {
                    1 => AoristFormation::Asigmatic,
                    2 => AoristFormation::SigmaticPrimary,
                    3 => AoristFormation::SigmaticSecondary,
                    4 => AoristFormation::SigmaticVowel,
                    _ => AoristFormation::New,
                });
                verb_lexeme_prediction(&verb, cell)
            }),
        },
        VerbCell::Imperative(imperative) => {
            if !imperative.is_supported() {
                return None;
            }
            merge_analyses(meta.imperative, |analysis| {
                let (stem, formation) = *analysis;
                let mut verb = base_verb_lexeme(lemma, meta);
                verb.stems.imperative = Some(stem.to_string());
                verb.formations.imperative = Some(match formation {
                    1 => ImperativeFormation::ISeries,
                    _ => ImperativeFormation::YatSeries,
                });
                verb_lexeme_prediction(&verb, cell)
            })
        }
        VerbCell::LParticiple(_) => merge_analyses(meta.l_participle, |stem| {
            let mut verb = base_verb_lexeme(lemma, meta);
            verb.stems.l_participle = Some((*stem).to_string());
            verb_lexeme_prediction(&verb, cell)
        }),
        VerbCell::ParticipleCitation(kind) => {
            let analyses = match kind {
                ParticipleKind::PresentActive => meta.present_active_participle,
                ParticipleKind::PresentPassive => meta.present_passive_participle,
                ParticipleKind::PastActive => meta.past_active_participle,
                ParticipleKind::PastPassive => meta.past_passive_participle,
            };
            merge_analyses(analyses, |analysis| {
                let (stem, formation) = *analysis;
                let mut verb = base_verb_lexeme(lemma, meta);
                match kind {
                    ParticipleKind::PresentActive => {
                        verb.stems.present_active_participle = Some(stem.to_string());
                        verb.formations.present_active_participle = Some(match formation {
                            1 => PresentActiveParticipleFormation::YushtHard,
                            2 => PresentActiveParticipleFormation::YushtSoft,
                            3 => PresentActiveParticipleFormation::YeshtSoft,
                            4 => PresentActiveParticipleFormation::MixedYushtSoft,
                            _ => PresentActiveParticipleFormation::IotatedYushtSoft,
                        });
                    }
                    ParticipleKind::PresentPassive => {
                        verb.stems.present_passive_participle = Some(stem.to_string());
                        verb.formations.present_passive_participle = Some(match formation {
                            1 => PresentPassiveParticipleFormation::Im,
                            2 => PresentPassiveParticipleFormation::Em,
                            3 => PresentPassiveParticipleFormation::IotatedEm,
                            _ => PresentPassiveParticipleFormation::Om,
                        });
                    }
                    ParticipleKind::PastActive => {
                        verb.stems.past_active_participle = Some(stem.to_string());
                        verb.formations.past_active_participle = Some(match formation {
                            1 => PastActiveParticipleFormation::Ush,
                            2 => PastActiveParticipleFormation::Ish,
                            3 => PastActiveParticipleFormation::IshAfterGlide,
                            4 => PastActiveParticipleFormation::VushAfterJDeletion,
                            5 => PastActiveParticipleFormation::VushAfterOvToU,
                            _ => PastActiveParticipleFormation::Vush,
                        });
                    }
                    ParticipleKind::PastPassive => {
                        verb.stems.past_passive_participle = Some(stem.to_string());
                        verb.formations.past_passive_participle =
                            Some(decode_past_passive_formation(formation));
                    }
                }
                verb_lexeme_prediction(&verb, cell)
            })
        }
        VerbCell::VerbalNoun => merge_analyses(meta.past_passive_participle, |analysis| {
            let (stem, formation) = *analysis;
            let mut verb = base_verb_lexeme(lemma, meta);
            verb.stems.past_passive_participle = Some(stem.to_string());
            verb.formations.past_passive_participle =
                Some(decode_past_passive_formation(formation));
            verb_lexeme_prediction(&verb, cell)
        }),
    }
}

fn decode_past_passive_formation(code: u8) -> PastPassiveParticipleFormation {
    match code {
        1 => PastPassiveParticipleFormation::T,
        2 => PastPassiveParticipleFormation::N,
        _ => PastPassiveParticipleFormation::En,
    }
}

/// Rule-kernel prediction for one verb cell: reviewed unique/irregular
/// identity kernels first, per-lemma principal-part metadata second. Returns
/// `None` when neither path determines the cell.
#[doc(hidden)]
pub fn kernel_verb_variants(
    lemma: &str,
    meta: &VerbMeta<'_>,
    cell: VerbCell,
) -> Option<Vec<String>> {
    if let Some(texts) = identity_verb_variants(lemma, cell) {
        return Some(texts);
    }
    metadata_verb_variants(lemma, meta, cell)
}

fn verb_form_variants(lemma: &str, cell: VerbCell) -> Result<Vec<String>, Error> {
    let code = verb_cell_code(cell);
    if let Ok(index) =
        generated::VERB_RESIDUE.binary_search_by(|row| (row.0, row.1).cmp(&(lemma, code)))
    {
        return Ok(generated::VERB_RESIDUE[index]
            .2
            .iter()
            .map(|text| (*text).to_string())
            .collect());
    }
    let meta = verb_meta(lemma).ok_or_else(|| Error::UnknownLemma(lemma.to_string()))?;
    kernel_verb_variants(base_lemma(lemma), &meta, cell).ok_or_else(|| Error::Underdetermined {
        lemma: lemma.to_string(),
    })
}

/// All variants for one present-tense finite cell, primary first.
pub fn present_variants(lemma: &str, person: Person, number: Number) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::Finite(FiniteVerbCell {
            tense: FiniteTense::Present,
            person,
            number,
        }),
    )
}

/// The primary surface form for one present-tense finite cell.
pub fn present(lemma: &str, person: Person, number: Number) -> Result<String, Error> {
    primary(present_variants(lemma, person, number)?, lemma)
}

/// All variants for one imperfect-tense finite cell, primary first.
pub fn imperfect_variants(
    lemma: &str,
    person: Person,
    number: Number,
) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::Finite(FiniteVerbCell {
            tense: FiniteTense::Imperfect,
            person,
            number,
        }),
    )
}

/// The primary surface form for one imperfect-tense finite cell.
pub fn imperfect(lemma: &str, person: Person, number: Number) -> Result<String, Error> {
    primary(imperfect_variants(lemma, person, number)?, lemma)
}

/// All variants for one aorist-tense finite cell, primary first.
pub fn aorist_variants(lemma: &str, person: Person, number: Number) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::Finite(FiniteVerbCell {
            tense: FiniteTense::Aorist,
            person,
            number,
        }),
    )
}

/// The primary surface form for one aorist-tense finite cell.
pub fn aorist(lemma: &str, person: Person, number: Number) -> Result<String, Error> {
    primary(aorist_variants(lemma, person, number)?, lemma)
}

/// All variants for one imperative cell, primary first. The attested
/// inventory includes third-person cells (`3:sg` broadly, `3:pl` for five
/// verbs), so [`Person`] spans all three values here.
pub fn imperative_variants(
    lemma: &str,
    person: Person,
    number: Number,
) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::Imperative(ImperativeCell { person, number }),
    )
}

/// The primary surface form for one imperative cell.
pub fn imperative(lemma: &str, person: Person, number: Number) -> Result<String, Error> {
    primary(imperative_variants(lemma, person, number)?, lemma)
}

/// All variants for one l-participle (resultative) cell, primary first.
pub fn l_participle_variants(
    lemma: &str,
    gender: Gender,
    number: Number,
) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::LParticiple(LParticipleCell { gender, number }),
    )
}

/// The primary surface form for one l-participle (resultative) cell.
pub fn l_participle(lemma: &str, gender: Gender, number: Number) -> Result<String, Error> {
    primary(l_participle_variants(lemma, gender, number)?, lemma)
}

/// All variants of the infinitive citation, primary first.
pub fn infinitive_variants(lemma: &str) -> Result<Vec<String>, Error> {
    verb_form_variants(lemma, VerbCell::Infinitive)
}

/// The primary infinitive citation form.
pub fn infinitive(lemma: &str) -> Result<String, Error> {
    primary(infinitive_variants(lemma)?, lemma)
}

/// All variants of the supine, primary first.
pub fn supine_variants(lemma: &str) -> Result<Vec<String>, Error> {
    verb_form_variants(lemma, VerbCell::Supine)
}

/// The primary supine form.
pub fn supine(lemma: &str) -> Result<String, Error> {
    primary(supine_variants(lemma)?, lemma)
}

/// All variants of the verbal-noun citation (nominative singular — the only
/// cell the oracle stores), primary first.
pub fn verbal_noun_variants(lemma: &str) -> Result<Vec<String>, Error> {
    verb_form_variants(lemma, VerbCell::VerbalNoun)
}

/// The primary verbal-noun citation form.
pub fn verbal_noun(lemma: &str) -> Result<String, Error> {
    primary(verbal_noun_variants(lemma)?, lemma)
}

/// All variants of the present active participle citation (masculine
/// nominative singular, indefinite), primary first.
pub fn present_active_participle_variants(lemma: &str) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::ParticipleCitation(ParticipleKind::PresentActive),
    )
}

/// The primary present active participle citation form.
pub fn present_active_participle(lemma: &str) -> Result<String, Error> {
    primary(present_active_participle_variants(lemma)?, lemma)
}

/// All variants of the present passive participle citation, primary first.
pub fn present_passive_participle_variants(lemma: &str) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::ParticipleCitation(ParticipleKind::PresentPassive),
    )
}

/// The primary present passive participle citation form.
pub fn present_passive_participle(lemma: &str) -> Result<String, Error> {
    primary(present_passive_participle_variants(lemma)?, lemma)
}

/// All variants of the past active participle citation, primary first.
pub fn past_active_participle_variants(lemma: &str) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::ParticipleCitation(ParticipleKind::PastActive),
    )
}

/// The primary past active participle citation form.
pub fn past_active_participle(lemma: &str) -> Result<String, Error> {
    primary(past_active_participle_variants(lemma)?, lemma)
}

/// All variants of the past passive participle citation, primary first.
pub fn past_passive_participle_variants(lemma: &str) -> Result<Vec<String>, Error> {
    verb_form_variants(
        lemma,
        VerbCell::ParticipleCitation(ParticipleKind::PastPassive),
    )
}

/// The primary past passive participle citation form.
pub fn past_passive_participle(lemma: &str) -> Result<String, Error> {
    primary(past_passive_participle_variants(lemma)?, lemma)
}

/// Stable numeric key for one closed-class cell, used by the generated
/// residue table. `cell_code(case, number) * 6 + dim`, where `dim` encodes
/// the third attested key dimension: 0 none (bare case x number cell),
/// 1/2/3 gender m/f/n, 4/5 person 1/2.
#[doc(hidden)]
pub fn closed_cell_code(
    case: Case,
    number: Number,
    gender: Option<Gender>,
    person: Option<Person>,
) -> Option<u8> {
    let dim = match (gender, person) {
        (None, None) => 0u8,
        (Some(gender), None) => 1 + gender_index(gender),
        (None, Some(Person::First)) => 4,
        (None, Some(Person::Second)) => 5,
        _ => return None,
    };
    Some(cell_code(case, number) * 6 + dim)
}

pub(crate) fn closed_pos_code(pos: PartOfSpeech) -> u8 {
    match pos {
        PartOfSpeech::Pronoun => 1,
        PartOfSpeech::Numeral => 2,
        _ => 3,
    }
}

/// Shape flags for one closed-class lemma's attested cells: bit 1 bare
/// (case x number) cells, bit 2 gender-indexed cells, bit 4 person-indexed
/// cells. A lemma may attest several shapes (the possessives are gendered
/// and person-indexed).
#[doc(hidden)]
pub fn closed_meta(lemma: &str) -> Option<(u8, u8)> {
    generated::CLOSED_META
        .binary_search_by(|row| row.0.cmp(lemma))
        .ok()
        .map(|index| {
            let row = &generated::CLOSED_META[index];
            (row.1, row.2)
        })
}

fn closed_pronoun_texts(variants: Vec<pronoun::PronounVariant>) -> Option<Vec<String>> {
    if variants.is_empty() {
        return None;
    }
    variants
        .into_iter()
        .map(|variant| orthography::canonical_display(variant.text).ok())
        .collect()
}

/// Rule-kernel prediction for one closed-class cell: the reviewed identity
/// kernels (personal/reflexive/anaphoric pronouns, standard pronominal
/// declension, `иже`/`сь`/`кꙑи`, interrogatives, cardinal and ordinal
/// numeral identities), keyed by lemma. This is the exact dispatch the
/// derivability harness replays; cells it cannot serve ship in the residue
/// table. Returns `None` when no kernel covers the request.
#[doc(hidden)]
pub fn kernel_closed_variants(
    lemma: &str,
    part_of_speech: PartOfSpeech,
    case: Case,
    number: Number,
    gender: Option<Gender>,
    person: Option<Person>,
) -> Option<Vec<String>> {
    // Regular class `2/p` identities span all three closed parts of speech.
    if let Some(identity) = StandardPronominalIdentity::classify_source_union_lemma(lemma)
        .filter(|identity| identity.part_of_speech() == part_of_speech)
    {
        let (None, Some(gender)) = (person, gender) else {
            return None;
        };
        let form = pronoun::decline_standard_pronominal(identity, case, number, gender).ok()?;
        return Some(vec![orthography::canonical_display(&form.text).ok()?]);
    }
    match part_of_speech {
        PartOfSpeech::Pronoun => {
            if let Some(identity) = PersonalPronounIdentity::classify_source_union_lemma(lemma) {
                let variants = match identity {
                    PersonalPronounIdentity::First | PersonalPronounIdentity::Second => {
                        match (person, gender, identity.person()) {
                            (Some(requested), None, Some(intrinsic)) if requested == intrinsic => {
                                pronoun::personal_forms(
                                    identity,
                                    case,
                                    number,
                                    PronounFormSelection::All,
                                )
                            }
                            _ => return None,
                        }
                    }
                    PersonalPronounIdentity::Reflexive => {
                        if person.is_none() && gender.is_none() {
                            pronoun::reflexive_forms(case, PronounFormSelection::All)
                        } else {
                            return None;
                        }
                    }
                    PersonalPronounIdentity::AnaphoricThird => match (person, gender) {
                        (None, Some(gender)) => pronoun::anaphoric_form(
                            case,
                            number,
                            gender,
                            AnaphoricEnvironment::Free,
                        )
                        .into_iter()
                        .collect(),
                        _ => return None,
                    },
                };
                return closed_pronoun_texts(variants);
            }
            match lemma {
                "иже" => {
                    let (None, Some(gender)) = (person, gender) else {
                        return None;
                    };
                    let text = pronoun::relative_izhe_form(
                        case,
                        number,
                        gender,
                        AnaphoricEnvironment::Free,
                    )?;
                    Some(vec![orthography::canonical_display(&text).ok()?])
                }
                "сь" => {
                    let (None, Some(gender)) = (person, gender) else {
                        return None;
                    };
                    closed_pronoun_texts(pronoun::irregular_agreeing_forms(
                        IrregularAgreeingIdentity::ProximalSi,
                        case,
                        number,
                        gender,
                    ))
                }
                "къто" | "чьто" | "никъто" => {
                    if person.is_some() || gender.is_some() {
                        return None;
                    }
                    let identity = if lemma == "чьто" {
                        InterrogativePronounIdentity::Chto
                    } else {
                        InterrogativePronounIdentity::Kto
                    };
                    let base = closed_pronoun_texts(pronoun::interrogative_forms(identity, case))?;
                    if lemma == "никъто" {
                        base.into_iter()
                            .map(|text| {
                                pronoun::compose_pronominal_family_text(
                                    &text,
                                    Some(PronominalPrefix::Ni),
                                    None,
                                    None,
                                )
                                .ok()
                            })
                            .collect()
                    } else {
                        Some(base)
                    }
                }
                _ => None,
            }
        }
        PartOfSpeech::Determiner => {
            if lemma == "кꙑи" {
                let (None, Some(gender)) = (person, gender) else {
                    return None;
                };
                closed_pronoun_texts(pronoun::irregular_agreeing_forms(
                    IrregularAgreeingIdentity::InterrogativeKyi,
                    case,
                    number,
                    gender,
                ))
            } else {
                None
            }
        }
        PartOfSpeech::Numeral => {
            if person.is_some() {
                return None;
            }
            if let Some(identity) = CardinalNumeralIdentity::classify_source_union_lemma(lemma) {
                let variants = numeral::decline_cardinal(
                    identity,
                    NumeralCell {
                        case,
                        number,
                        gender,
                    },
                )
                .ok()?;
                let mut texts: Vec<String> = Vec::new();
                for variant in variants {
                    let text = orthography::canonical_display(&variant.prediction.text).ok()?;
                    if !texts.contains(&text) {
                        texts.push(text);
                    }
                }
                return (!texts.is_empty()).then_some(texts);
            }
            // Ordinal registry cells merge the short and long adjectival
            // series into one gendered cell; both come from the reviewed
            // ordinal kernel.
            let identity = OrdinalNumeralIdentity::classify_source_union_lemma(lemma)?;
            let gender = gender?;
            let mut texts: Vec<String> = Vec::new();
            for form in [AdjectiveForm::Short, AdjectiveForm::Long] {
                let variants = numeral::decline_ordinal(
                    identity,
                    AdjectiveCell {
                        case,
                        number,
                        gender,
                        animacy: Animacy::Inanimate,
                        form,
                    },
                )
                .ok()?;
                if variants.is_empty() {
                    return None;
                }
                for variant in variants {
                    let text = orthography::canonical_display(&variant.prediction.text).ok()?;
                    if !texts.contains(&text) {
                        texts.push(text);
                    }
                }
            }
            (!texts.is_empty()).then_some(texts)
        }
        _ => None,
    }
}

/// Lemma-keyed resolution for one closed-class cell: residue table first,
/// identity kernels second. The public wrappers and the accuracy gate both
/// go through this path.
#[doc(hidden)]
pub fn closed_variants(
    lemma: &str,
    part_of_speech: PartOfSpeech,
    case: Case,
    number: Number,
    gender: Option<Gender>,
    person: Option<Person>,
) -> Result<Vec<String>, Error> {
    let code =
        closed_cell_code(case, number, gender, person).ok_or_else(|| Error::Underdetermined {
            lemma: lemma.to_string(),
        })?;
    if let Ok(index) =
        generated::CLOSED_RESIDUE.binary_search_by(|row| (row.0, row.1).cmp(&(lemma, code)))
    {
        return Ok(generated::CLOSED_RESIDUE[index]
            .2
            .iter()
            .map(|text| (*text).to_string())
            .collect());
    }
    closed_meta(lemma)
        .filter(|(pos, _)| *pos == closed_pos_code(part_of_speech))
        .ok_or_else(|| Error::UnknownLemma(lemma.to_string()))?;
    kernel_closed_variants(lemma, part_of_speech, case, number, gender, person).ok_or_else(|| {
        Error::Underdetermined {
            lemma: lemma.to_string(),
        }
    })
}

/// All variants for one personal-pronoun cell, primary first. The attested
/// personal cells are exactly person x number x case (see the module docs);
/// they are served under the canonical lemmas `азъ` (first person) and `тꙑ`
/// (second). [`Person::Third`] is not a personal-pronoun value in this
/// inventory — the third person is the gendered anaphoric series, served by
/// [`anaphoric`] — and returns [`Error::Underdetermined`].
pub fn pronoun_variants(person: Person, number: Number, case: Case) -> Result<Vec<String>, Error> {
    let lemma = match person {
        Person::First => "азъ",
        Person::Second => "тꙑ",
        Person::Third => {
            return Err(Error::Underdetermined {
                lemma: "азъ".to_string(),
            });
        }
    };
    closed_variants(
        lemma,
        PartOfSpeech::Pronoun,
        case,
        number,
        None,
        Some(person),
    )
}

/// The primary surface form for one personal-pronoun cell.
pub fn pronoun(person: Person, number: Number, case: Case) -> Result<String, Error> {
    let lemma = if person == Person::First {
        "азъ"
    } else {
        "тꙑ"
    };
    primary(pronoun_variants(person, number, case)?, lemma)
}

/// All variants for one reflexive-pronoun cell, primary first. The reflexive
/// `сѧ` is numberless: its attested number dimension is fully degenerate
/// (singular, dual, and plural rows are identical; the accuracy gate replays
/// all of them through this function), so [`Case`] is the whole key.
/// Nominative and vocative are historically absent and return
/// [`Error::Underdetermined`].
pub fn reflexive_variants(case: Case) -> Result<Vec<String>, Error> {
    closed_variants(
        "сѧ",
        PartOfSpeech::Pronoun,
        case,
        Number::Singular,
        None,
        None,
    )
}

/// The primary surface form for one reflexive-pronoun cell.
pub fn reflexive(case: Case) -> Result<String, Error> {
    primary(reflexive_variants(case)?, "сѧ")
}

/// All variants for one third-person (anaphoric) pronoun cell, primary
/// first. The oracle keys these cells by gender, not person, so the minimal
/// signature is case x number x gender; the identical tables the source
/// duplicates under the spellings `ѥ`, `ѭ`, and `ими` are canonicalized to
/// lemma `и` here (each spelling also answers through [`pronoun_form`]).
pub fn anaphoric_variants(
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<Vec<String>, Error> {
    closed_variants("и", PartOfSpeech::Pronoun, case, number, Some(gender), None)
}

/// The primary surface form for one third-person (anaphoric) pronoun cell.
pub fn anaphoric(case: Case, number: Number, gender: Gender) -> Result<String, Error> {
    primary(anaphoric_variants(case, number, gender)?, "и")
}

fn lexical_closed_variants(
    lemma: &str,
    part_of_speech: PartOfSpeech,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<Vec<String>, Error> {
    let (_, shape) = closed_meta(lemma)
        .filter(|(pos, _)| *pos == closed_pos_code(part_of_speech))
        .ok_or_else(|| Error::UnknownLemma(lemma.to_string()))?;
    // Gender is a key dimension exactly when the lexeme attests
    // gender-indexed cells; for bare-shaped lexemes it is ignored (the data
    // draws no such distinction). Person-indexed-only lemmas have no
    // case x number (x gender) table of their own (see the module docs).
    let gender = if shape & 2 != 0 {
        Some(gender)
    } else if shape & 1 != 0 {
        None
    } else {
        return Err(Error::Underdetermined {
            lemma: lemma.to_string(),
        });
    };
    closed_variants(lemma, part_of_speech, case, number, gender, None)
}

/// All variants for one lexically identified pronoun cell, primary first.
/// For gender-indexed lexemes (`тъ`, `иже`, `сь`, `вьсѣкъ`, the anaphoric
/// and demonstrative spellings, the gendered halves of the possessives)
/// `gender` is a key; for bare-shaped lexemes (`етеръ`, `къто`, `чьто`,
/// `никъто`, `сѧ`) it is ignored. Lemmas attesting only the shared
/// person-indexed table are served by [`pronoun`] and return
/// [`Error::Underdetermined`] here.
pub fn pronoun_form_variants(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<Vec<String>, Error> {
    lexical_closed_variants(lemma, PartOfSpeech::Pronoun, case, number, gender)
}

/// The primary surface form for one lexically identified pronoun cell.
pub fn pronoun_form(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<String, Error> {
    primary(pronoun_form_variants(lemma, case, number, gender)?, lemma)
}

/// All variants for one numeral cell, primary first. The ordinal `прьвъ` is
/// gender-indexed (`gender` is a key; the oracle merges its short and long
/// adjectival series into one cell); the cardinals `пѧть` … `десѧть` (and
/// the mistagged proper noun `Єѵрѡпа` the source files under `num`) attest
/// bare case x number cells, so `gender` is ignored for them.
pub fn numeral_form_variants(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<Vec<String>, Error> {
    lexical_closed_variants(lemma, PartOfSpeech::Numeral, case, number, gender)
}

/// The primary surface form for one numeral cell.
pub fn numeral_form(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<String, Error> {
    primary(numeral_form_variants(lemma, case, number, gender)?, lemma)
}

/// All variants for one determiner cell, primary first. The single attested
/// determiner `кꙑи` is gender-indexed, so `gender` is a key.
pub fn determiner_form_variants(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<Vec<String>, Error> {
    lexical_closed_variants(lemma, PartOfSpeech::Determiner, case, number, gender)
}

/// The primary surface form for one determiner cell.
pub fn determiner_form(
    lemma: &str,
    case: Case,
    number: Number,
    gender: Gender,
) -> Result<String, Error> {
    primary(
        determiner_form_variants(lemma, case, number, gender)?,
        lemma,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_noun_via_rules() {
        // аблъко is a plain o-neuter-hard noun with no residue rows for this
        // cell; the form comes from the rule kernel.
        assert_eq!(
            noun("аблъко", Case::Genitive, Number::Singular).as_deref(),
            Ok("аблъка")
        );
    }

    #[test]
    fn irregular_noun_via_residue() {
        // ногъть keeps attested variant sets the kernel does not derive.
        let variants =
            noun_variants("ногъть", Case::Dative, Number::Singular).expect("attested cell");
        assert!(variants.iter().any(|form| form == "ногътю"), "{variants:?}");
    }

    #[test]
    fn regular_adjective_via_rules() {
        // новъ is a plain hard adjective; the long genitive comes from the
        // rule kernel (no residue row for this cell).
        assert_eq!(
            adjective("новъ", Case::Genitive, Number::Singular, Gender::Masculine).as_deref(),
            Ok("новаѥго")
        );
    }

    #[test]
    fn irregular_adjective_via_residue() {
        // вьсемогъ keeps an attested two-variant long nominative plural the
        // kernel does not derive; the residue table serves it verbatim.
        assert_eq!(
            adjective_variants(
                "вьсемогъ",
                Case::Nominative,
                Number::Plural,
                Gender::Masculine
            ),
            Ok(vec!["вьсемогъшеи".to_string(), "вьсемогъшии".to_string()])
        );
    }

    #[test]
    fn short_adjective_uses_short_paradigm() {
        assert_eq!(
            short_adjective("новъ", Case::Genitive, Number::Singular, Gender::Masculine).as_deref(),
            Ok("нова")
        );
        assert_eq!(
            adjective(
                "nonexistent",
                Case::Nominative,
                Number::Singular,
                Gender::Feminine
            ),
            Err(Error::UnknownLemma("nonexistent".to_string()))
        );
    }

    #[test]
    fn verb_via_metadata_rules() {
        // блажити carries an l-participle principal-part stem (блажи) in the
        // generated metadata table; the form comes from the rule kernel (no
        // residue row for this cell).
        assert_eq!(
            l_participle("блажити", Gender::Masculine, Number::Singular).as_deref(),
            Ok("блажилъ")
        );
    }

    #[test]
    fn verb_via_irregular_identity_kernel() {
        // бꙑти is a reviewed unique-verb identity; its l-participle cells
        // carry no residue rows and come from the identity kernel.
        assert_eq!(
            l_participle("бꙑти", Gender::Masculine, Number::Singular).as_deref(),
            Ok("бꙑлъ")
        );
    }

    #[test]
    fn verb_via_residue() {
        // блажити has no present-system metadata, so its attested present
        // third-dual variant pair is served verbatim from the residue table.
        assert_eq!(
            present_variants("блажити", Person::Third, Number::Dual),
            Ok(vec!["блажите".to_string(), "блажита".to_string()])
        );
    }

    #[test]
    fn unknown_verb_lemma_is_typed_error() {
        assert_eq!(
            infinitive("nonexistent"),
            Err(Error::UnknownLemma("nonexistent".to_string()))
        );
    }

    #[test]
    fn personal_and_reflexive_pronouns_via_kernel() {
        // The dative-singular personal cells carry the marked clitics; the
        // reflexive is numberless and refuses the nominative.
        assert_eq!(
            pronoun_variants(Person::First, Number::Singular, Case::Dative),
            Ok(vec!["мьнѣ".to_string(), "ми".to_string()])
        );
        assert_eq!(
            pronoun(Person::Second, Number::Plural, Case::Nominative).as_deref(),
            Ok("вꙑ")
        );
        assert_eq!(reflexive(Case::Dative).as_deref(), Ok("себѣ"));
        assert!(matches!(
            reflexive(Case::Nominative),
            Err(Error::Underdetermined { .. })
        ));
    }

    #[test]
    fn gendered_closed_class_forms() {
        // The anaphoric third person is canonicalized to lemma `и`; the
        // demonstrative family and the determiner resolve lemma-keyed.
        assert_eq!(
            anaphoric(Case::Dative, Number::Singular, Gender::Feminine).as_deref(),
            Ok("ѥи")
        );
        assert_eq!(
            pronoun_form("онъ", Case::Dative, Number::Singular, Gender::Feminine).as_deref(),
            Ok("онои")
        );
        assert_eq!(
            determiner_form("кꙑи", Case::Nominative, Number::Singular, Gender::Masculine)
                .as_deref(),
            Ok("кꙑи")
        );
        // прьвъ is gender-indexed; the bare cardinals ignore the gender
        // parameter (десѧть's genitive singular is the same either way).
        assert_eq!(
            numeral_form("десѧть", Case::Genitive, Number::Singular, Gender::Feminine),
            numeral_form(
                "десѧть",
                Case::Genitive,
                Number::Singular,
                Gender::Masculine
            ),
        );
    }

    #[test]
    fn closed_class_residue_and_shape_errors() {
        // етеръ has no reviewed kernel: its attested cells come verbatim
        // from the residue table (gender ignored, bare shape).
        assert!(pronoun_form("етеръ", Case::Nominative, Number::Singular, Gender::Neuter).is_ok());
        // вашь attests only the duplicated person-indexed table, so the
        // lemma-keyed pronoun_form cannot address it.
        assert!(matches!(
            pronoun_form(
                "вашь",
                Case::Nominative,
                Number::Singular,
                Gender::Masculine
            ),
            Err(Error::Underdetermined { .. })
        ));
        assert_eq!(
            numeral_form(
                "nonexistent",
                Case::Nominative,
                Number::Singular,
                Gender::Masculine
            ),
            Err(Error::UnknownLemma("nonexistent".to_string()))
        );
    }

    #[test]
    fn unknown_lemma_is_typed_error() {
        assert_eq!(
            noun("nonexistent", Case::Nominative, Number::Singular),
            Err(Error::UnknownLemma("nonexistent".to_string()))
        );
    }
}
