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

use old_church_slavonic_core::noun::NounLexeme;
use old_church_slavonic_core::unique_noun::UniqueNounFamilyMember;
use old_church_slavonic_core::{
    Animacy, Gender, NounCell, NounClass, NumberRestriction, TwofoldNounFamilyMember, orthography,
};

pub use church_slavonic_core::grammar::{Case, Number};

mod generated {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/noun_residue.rs"));
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
    let gender = meta
        .gender
        .or_else(|| class.intrinsic_gender())?;
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
    if let Ok(index) = generated::NOUN_RESIDUE
        .binary_search_by(|row| (row.0, row.1).cmp(&(lemma, code)))
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
        let variants = noun_variants("ногъть", Case::Dative, Number::Singular)
            .expect("attested cell");
        assert!(variants.iter().any(|form| form == "ногътю"), "{variants:?}");
    }

    #[test]
    fn unknown_lemma_is_typed_error() {
        assert_eq!(
            noun("nonexistent", Case::Nominative, Number::Singular),
            Err(Error::UnknownLemma("nonexistent".to_string()))
        );
    }
}
