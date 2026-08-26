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
//! - **Homograph merge.** The extracted inventory contains ten noun lemmas
//!   with two lexeme entries each (e.g. `градъ`, `ногъть`, `сꙑнъ`), whose
//!   stored tables differ only in variant coverage. This facade is keyed by
//!   lemma, so the oracle is defined at lemma granularity: for each
//!   (lemma, cell) the stored variant lists of all homograph lexemes are
//!   merged in rank order (stable, first occurrence wins on duplicates), and
//!   the facade reproduces that merged list.
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
//! facade at 100% either way. The homograph convention is the nouns' lemma
//! merge; the four verb homograph pairs (`вести`, `пасти`, `привести`,
//! `съпасти` — transitive/intransitive or lexical doublets sharing a
//! citation) are served with rank-merged variant lists, first lexeme's
//! metadata winning.

use old_church_slavonic_core::adjective::AdjectiveLexeme;
use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::unique_noun::UniqueNounFamilyMember;
use old_church_slavonic_core::verb::VerbLexeme;
use old_church_slavonic_core::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, AoristFormation, FiniteTense,
    FiniteVerbCell, ImperativeCell, ImperativeFormation, ImperfectFormation,
    ImperfectVariantPolicy, IrregularVerbFamilyMember, LParticipleCell, NounCell, NounClass,
    NumberRestriction, ParticipleCell, ParticipleKind, PastActiveParticipleFormation,
    PastPassiveParticipleFormation, PresentActiveParticipleFormation, PresentFormation,
    PresentPassiveParticipleFormation, TwofoldNounFamilyMember, UniqueVerbFamilyMember, VerbAspect,
    VerbClass, orthography,
};

pub use church_slavonic_core::grammar::{Case, Gender, Number, Person};

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
    kernel_noun_variants(lemma, &meta, case, number).ok_or_else(|| Error::Underdetermined {
        lemma: lemma.to_string(),
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

fn adjective_form_variants(
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
    kernel_adjective_variants(lemma, class, form, case, number, gender).ok_or_else(|| {
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
    kernel_verb_variants(lemma, &meta, cell).ok_or_else(|| Error::Underdetermined {
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
    fn unknown_lemma_is_typed_error() {
        assert_eq!(
            noun("nonexistent", Case::Nominative, Number::Singular),
            Err(Error::UnknownLemma("nonexistent".to_string()))
        );
    }
}
