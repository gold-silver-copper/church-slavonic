//! Paradigm enumeration: every cell one lexeme actually supports.
//!
//! Each `*_paradigm` function enumerates the candidate cell space of one
//! part of speech in the historical order (the `Case::ALL` / `Number::ALL` /
//! `Gender::ALL` declaration orders; verbs follow the residue table's cell
//! ordering: present, imperfect, aorist, imperative, l-participle, then the
//! citations) and resolves every candidate through the **same single-cell
//! path the per-cell public functions use** — there is no second resolution
//! logic. A cell that path cannot serve ([`Error::Underdetermined`]: a
//! number-restricted or historically absent cell, an animacy-conditioned
//! accusative without an animacy fact, a kernel defect) is simply absent
//! from the listing, not an error. A lemma the facade knows nothing about
//! produces [`Error::UnknownLemma`].
//!
//! Variant lists are exactly what the corresponding single-cell function
//! returns for that cell (primary variant first); the accuracy gate
//! (`cargo xtask rewrite-pilot-accuracy`) proves this cell by cell over the
//! full attested inventory.

use crate::{
    AdjectiveForm, Case, Error, Gender, Number, Person, adjective_form_variants, aorist_variants,
    closed_meta, closed_pos_code, closed_variants, imperative_variants, imperfect_variants,
    infinitive_variants, l_participle_variants, noun_variants, past_active_participle_variants,
    past_passive_participle_variants, present_active_participle_variants,
    present_passive_participle_variants, present_variants, supine_variants, verbal_noun_variants,
};
use crate::ParticipleKind;
use old_church_slavonic_core::PartOfSpeech;

/// One lexeme's enumerated noun paradigm: every servable (case, number)
/// cell with its variant list (primary first), in `Case::ALL` x
/// `Number::ALL` order.
pub type NounParadigm = Vec<(Case, Number, Vec<String>)>;

/// One lexeme's enumerated adjective paradigm (one form): every servable
/// (case, number, gender) cell with its variant list.
pub type AdjectiveParadigm = Vec<(Case, Number, Gender, Vec<String>)>;

/// One lexeme's enumerated verb paradigm: every servable [`VerbCellKind`]
/// with its variant list.
pub type VerbParadigm = Vec<(VerbCellKind, Vec<String>)>;

/// One closed-class lexeme's enumerated paradigm: every servable
/// (case, number, optional gender) cell with its variant list. The gender
/// field is `Some` for gender-indexed lexemes and `None` for bare-shaped
/// ones (whose data draws no gender distinction).
pub type ClosedParadigm = Vec<(Case, Number, Option<Gender>, Vec<String>)>;

/// One verb-paradigm cell kind, spanning every attested verb cell the facade
/// serves. This is the public counterpart of the internal residue-table cell
/// key: the finite tenses, the imperative, and the l-participle carry their
/// index dimensions; the infinitive, supine, verbal noun, and the four
/// participle kinds are single citation cells (the participle citations are
/// the masculine nominative singular short forms the oracle stores —
/// declining them across case/number/gender/form is
/// [`participle_paradigm`], not this enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerbCellKind {
    /// Present-tense finite cell.
    Present { person: Person, number: Number },
    /// Imperfect-tense finite cell.
    Imperfect { person: Person, number: Number },
    /// Aorist-tense finite cell.
    Aorist { person: Person, number: Number },
    /// Imperative cell (the attested inventory includes third-person cells).
    Imperative { person: Person, number: Number },
    /// Resultative l-participle cell.
    LParticiple { gender: Gender, number: Number },
    /// Infinitive citation.
    Infinitive,
    /// Supine citation.
    Supine,
    /// Verbal-noun citation (nominative singular, the only attested cell).
    VerbalNoun,
    /// Present active participle citation.
    PresentActiveParticiple,
    /// Present passive participle citation.
    PresentPassiveParticiple,
    /// Past active participle citation.
    PastActiveParticiple,
    /// Past passive participle citation.
    PastPassiveParticiple,
}

impl VerbCellKind {
    /// Every candidate verb cell in the paradigm enumeration order (the
    /// residue table's cell-code order: finite tenses person x number,
    /// imperatives, l-participles gender x number, then the citations).
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut cells = Vec::with_capacity(49);
        for person in Person::ALL {
            for number in Number::ALL {
                cells.push(Self::Present { person, number });
            }
        }
        for person in Person::ALL {
            for number in Number::ALL {
                cells.push(Self::Imperfect { person, number });
            }
        }
        for person in Person::ALL {
            for number in Number::ALL {
                cells.push(Self::Aorist { person, number });
            }
        }
        for person in Person::ALL {
            for number in Number::ALL {
                cells.push(Self::Imperative { person, number });
            }
        }
        for gender in Gender::ALL {
            for number in Number::ALL {
                cells.push(Self::LParticiple { gender, number });
            }
        }
        cells.extend([
            Self::Infinitive,
            Self::Supine,
            Self::VerbalNoun,
            Self::PresentActiveParticiple,
            Self::PresentPassiveParticiple,
            Self::PastActiveParticiple,
            Self::PastPassiveParticiple,
        ]);
        cells
    }
}

/// Resolve every candidate cell through one single-cell resolver, keeping
/// the servable cells and skipping the underdetermined ones. The lemma
/// counts as known if any candidate resolves or is refused as
/// `Underdetermined` (the arm the single-cell path reserves for known
/// lemmas); a lemma every candidate rejects as unknown propagates
/// [`Error::UnknownLemma`].
fn collect_cells<C: Copy>(
    lemma: &str,
    candidates: impl IntoIterator<Item = C>,
    mut resolve: impl FnMut(C) -> Result<Vec<String>, Error>,
) -> Result<Vec<(C, Vec<String>)>, Error> {
    let mut cells = Vec::new();
    let mut lemma_known = false;
    for cell in candidates {
        match resolve(cell) {
            Ok(variants) => {
                lemma_known = true;
                cells.push((cell, variants));
            }
            Err(Error::Underdetermined { .. }) => lemma_known = true,
            Err(_) => {}
        }
    }
    if !lemma_known {
        return Err(Error::UnknownLemma(lemma.to_string()));
    }
    Ok(cells)
}

fn case_number_grid() -> impl Iterator<Item = (Case, Number)> {
    Case::ALL
        .into_iter()
        .flat_map(|case| Number::ALL.into_iter().map(move |number| (case, number)))
}

/// Every noun cell the lexeme actually supports, in `Case::ALL` x
/// `Number::ALL` order, each with its variant list (primary first) — exactly
/// the cells [`noun_variants`](crate::noun_variants) serves for this lemma.
pub fn noun_paradigm(lemma: &str) -> Result<NounParadigm, Error> {
    let cells = collect_cells(lemma, case_number_grid(), |(case, number)| {
        noun_variants(lemma, case, number)
    })?;
    Ok(cells
        .into_iter()
        .map(|((case, number), variants)| (case, number, variants))
        .collect())
}

/// Every adjective cell of one form (long/short) the lexeme actually
/// supports, in `Case::ALL` x `Number::ALL` x `Gender::ALL` order — exactly
/// the cells [`adjective_variants`](crate::adjective_variants) (long) or
/// [`short_adjective_variants`](crate::short_adjective_variants) (short)
/// serve. A long-only lexeme simply has no short cells.
pub fn adjective_paradigm(
    lemma: &str,
    form: AdjectiveForm,
) -> Result<AdjectiveParadigm, Error> {
    let candidates = case_number_grid().flat_map(|(case, number)| {
        Gender::ALL
            .into_iter()
            .map(move |gender| (case, number, gender))
    });
    let cells = collect_cells(lemma, candidates, |(case, number, gender)| {
        adjective_form_variants(lemma, form, case, number, gender)
    })?;
    Ok(cells
        .into_iter()
        .map(|((case, number, gender), variants)| (case, number, gender, variants))
        .collect())
}

/// Every declined cell of one participle system (kind) in one form
/// (long/short) the lexeme actually supports, in `Case::ALL` x
/// `Number::ALL` x `Gender::ALL` order — exactly the cells
/// [`participle_variants`](crate::participle_variants) serves for this
/// lemma, kind, and form. The shape is [`AdjectiveParadigm`]: a participle
/// declines as an adjective. The [`VerbCellKind`] verb paradigm keeps only
/// the citation cell of each kind; this function is the declension of that
/// citation across the agreement grid.
pub fn participle_paradigm(
    lemma: &str,
    kind: ParticipleKind,
    form: AdjectiveForm,
) -> Result<AdjectiveParadigm, Error> {
    let candidates = case_number_grid().flat_map(|(case, number)| {
        Gender::ALL
            .into_iter()
            .map(move |gender| (case, number, gender))
    });
    let cells = collect_cells(lemma, candidates, |(case, number, gender)| {
        crate::participle_variants(lemma, kind, case, number, gender, form)
    })?;
    Ok(cells
        .into_iter()
        .map(|((case, number, gender), variants)| (case, number, gender, variants))
        .collect())
}

/// Every verb cell the lexeme actually supports, in [`VerbCellKind::all`]
/// order — exactly the cells the per-cell public functions
/// ([`present_variants`](crate::present_variants), …,
/// [`past_passive_participle_variants`](crate::past_passive_participle_variants))
/// serve for this lemma, participle citations included.
pub fn verb_paradigm(lemma: &str) -> Result<VerbParadigm, Error> {
    collect_cells(lemma, VerbCellKind::all(), |cell| match cell {
        VerbCellKind::Present { person, number } => present_variants(lemma, person, number),
        VerbCellKind::Imperfect { person, number } => imperfect_variants(lemma, person, number),
        VerbCellKind::Aorist { person, number } => aorist_variants(lemma, person, number),
        VerbCellKind::Imperative { person, number } => imperative_variants(lemma, person, number),
        VerbCellKind::LParticiple { gender, number } => l_participle_variants(lemma, gender, number),
        VerbCellKind::Infinitive => infinitive_variants(lemma),
        VerbCellKind::Supine => supine_variants(lemma),
        VerbCellKind::VerbalNoun => verbal_noun_variants(lemma),
        VerbCellKind::PresentActiveParticiple => present_active_participle_variants(lemma),
        VerbCellKind::PresentPassiveParticiple => present_passive_participle_variants(lemma),
        VerbCellKind::PastActiveParticiple => past_active_participle_variants(lemma),
        VerbCellKind::PastPassiveParticiple => past_passive_participle_variants(lemma),
    })
}

/// Shared closed-class enumeration: the lemma's attested cell shape decides
/// the candidate space exactly the way the single-cell `*_form` functions
/// decide their key dimensions — gendered lexemes enumerate case x number x
/// gender (`Some(gender)`), bare-shaped lexemes case x number (`None`; the
/// data draws no gender distinction there), and a lemma attesting only the
/// shared person-indexed personal table has no lemma-keyed paradigm of its
/// own (empty listing; its cells are served by [`pronoun`](crate::pronoun)).
fn closed_paradigm(
    lemma: &str,
    part_of_speech: PartOfSpeech,
) -> Result<ClosedParadigm, Error> {
    let (_, shape) = closed_meta(lemma)
        .filter(|(pos, _)| *pos == closed_pos_code(part_of_speech))
        .ok_or_else(|| Error::UnknownLemma(lemma.to_string()))?;
    let genders: &[Option<Gender>] = if shape & 2 != 0 {
        &[
            Some(Gender::Masculine),
            Some(Gender::Feminine),
            Some(Gender::Neuter),
        ]
    } else if shape & 1 != 0 {
        &[None]
    } else {
        return Ok(Vec::new());
    };
    let candidates = case_number_grid().flat_map(|(case, number)| {
        genders
            .iter()
            .copied()
            .map(move |gender| (case, number, gender))
    });
    let cells = collect_cells(lemma, candidates, |(case, number, gender)| {
        closed_variants(lemma, part_of_speech, case, number, gender, None)
    })?;
    Ok(cells
        .into_iter()
        .map(|((case, number, gender), variants)| (case, number, gender, variants))
        .collect())
}

/// Every lemma-keyed pronoun cell the lexeme actually supports — exactly the
/// cells [`pronoun_form_variants`](crate::pronoun_form_variants) serves. The
/// third tuple field is `Some(gender)` for gender-indexed lexemes and `None`
/// for bare-shaped ones (where the single-cell function ignores its gender
/// parameter). See [`closed_paradigm`]'s shape rules in the source.
pub fn pronoun_form_paradigm(
    lemma: &str,
) -> Result<ClosedParadigm, Error> {
    closed_paradigm(lemma, PartOfSpeech::Pronoun)
}

/// Every numeral cell the lexeme actually supports — exactly the cells
/// [`numeral_form_variants`](crate::numeral_form_variants) serves. Gendered
/// for `прьвъ` (`Some(gender)`), bare (`None`) for the cardinals.
pub fn numeral_form_paradigm(
    lemma: &str,
) -> Result<ClosedParadigm, Error> {
    closed_paradigm(lemma, PartOfSpeech::Numeral)
}

/// Every determiner cell the lexeme actually supports — exactly the cells
/// [`determiner_form_variants`](crate::determiner_form_variants) serves.
pub fn determiner_form_paradigm(
    lemma: &str,
) -> Result<ClosedParadigm, Error> {
    closed_paradigm(lemma, PartOfSpeech::Determiner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_noun_paradigm_shape() {
        // аблъко is a plain o-neuter-hard noun: every case x number cell is
        // servable, in Case::ALL x Number::ALL order.
        let paradigm = noun_paradigm("аблъко").expect("known lemma");
        assert_eq!(paradigm.len(), 21, "{paradigm:?}");
        assert_eq!(
            (paradigm[0].0, paradigm[0].1),
            (Case::Nominative, Number::Singular)
        );
        assert_eq!(
            (paradigm[20].0, paradigm[20].1),
            (Case::Vocative, Number::Plural)
        );
        // Every listed cell agrees with the single-cell API.
        for (case, number, variants) in &paradigm {
            assert_eq!(
                crate::noun_variants("аблъко", *case, *number).as_ref(),
                Ok(variants)
            );
        }
    }

    #[test]
    fn defective_noun_paradigm_skips_unservable_cells() {
        // врата is a plurale tantum: its number restriction leaves exactly
        // the seven plural cells, and the absent singular/dual cells are
        // simply missing from the listing, not an error.
        let paradigm = noun_paradigm("врата").expect("known lemma");
        assert_eq!(paradigm.len(), 7, "{paradigm:?}");
        assert!(paradigm.iter().all(|(_, number, _)| *number == Number::Plural));
        // гладъ is a masculine of an animacy-contrasting class with no
        // animacy fact: the kernel cannot commit to the accusative, and only
        // the attested accusative singular ships in the residue, so the
        // unattested accusative dual/plural are absent from the paradigm.
        let paradigm = noun_paradigm("гладъ").expect("known lemma");
        assert_eq!(paradigm.len(), 19, "{paradigm:?}");
        for (case, number) in [(Case::Accusative, Number::Dual), (Case::Accusative, Number::Plural)] {
            assert!(!paradigm.iter().any(|(c, n, _)| (*c, *n) == (case, number)));
        }
        for (case, number, variants) in &paradigm {
            assert_eq!(
                crate::noun_variants("гладъ", *case, *number).as_ref(),
                Ok(variants)
            );
        }
    }

    #[test]
    fn verb_paradigm_includes_participle_citations() {
        let paradigm = verb_paradigm("блажити").expect("known lemma");
        let kinds: Vec<VerbCellKind> = paradigm.iter().map(|(kind, _)| *kind).collect();
        assert!(kinds.contains(&VerbCellKind::Infinitive));
        assert!(kinds.contains(&VerbCellKind::PresentActiveParticiple));
        assert!(kinds.contains(&VerbCellKind::LParticiple {
            gender: Gender::Masculine,
            number: Number::Singular
        }));
        // The residue-served present third dual appears with its full
        // variant list, exactly as the single-cell API returns it.
        let cell = VerbCellKind::Present {
            person: Person::Third,
            number: Number::Dual,
        };
        let listed = paradigm
            .iter()
            .find(|(kind, _)| *kind == cell)
            .map(|(_, variants)| variants.clone());
        assert_eq!(
            listed,
            Some(vec!["блажите".to_string(), "блажита".to_string()])
        );
    }

    #[test]
    fn adjective_paradigm_long_and_short() {
        let long = adjective_paradigm("новъ", AdjectiveForm::Long).expect("known lemma");
        assert_eq!(long.len(), 63, "{long:?}");
        let short = adjective_paradigm("новъ", AdjectiveForm::Short).expect("known lemma");
        assert!(!short.is_empty());
        assert!(matches!(
            adjective_paradigm("nonexistent", AdjectiveForm::Long),
            Err(Error::UnknownLemma(_))
        ));
    }

    #[test]
    fn participle_paradigm_declines_as_adjective() {
        let long = participle_paradigm(
            "благословити",
            ParticipleKind::PastPassive,
            AdjectiveForm::Long,
        )
        .expect("known lemma");
        assert_eq!(long.len(), 63, "{long:?}");
        for (case, number, gender, variants) in &long {
            assert_eq!(
                crate::participle_variants(
                    "благословити",
                    ParticipleKind::PastPassive,
                    *case,
                    *number,
                    *gender,
                    AdjectiveForm::Long
                )
                .as_ref(),
                Ok(variants)
            );
        }
        // The reviewed profile of `ити` refuses the past passive: no long
        // cell serves (empty listing, not an error), and the short paradigm
        // keeps only the metadata-backed citation cell.
        assert_eq!(
            participle_paradigm("ити", ParticipleKind::PastPassive, AdjectiveForm::Long),
            Ok(Vec::new())
        );
        assert_eq!(
            participle_paradigm("ити", ParticipleKind::PastPassive, AdjectiveForm::Short)
                .map(|cells| cells.len()),
            Ok(1)
        );
        assert!(matches!(
            participle_paradigm(
                "nonexistent",
                ParticipleKind::PresentActive,
                AdjectiveForm::Long
            ),
            Err(Error::UnknownLemma(_))
        ));
    }

    #[test]
    fn closed_paradigm_shapes() {
        // Gendered demonstrative: cells carry Some(gender).
        let demonstrative = pronoun_form_paradigm("онъ").expect("known lemma");
        assert!(!demonstrative.is_empty());
        assert!(demonstrative.iter().all(|(_, _, gender, _)| gender.is_some()));
        // Bare-shaped interrogative: gender dimension absent.
        let interrogative = pronoun_form_paradigm("къто").expect("known lemma");
        assert!(!interrogative.is_empty());
        assert!(interrogative.iter().all(|(_, _, gender, _)| gender.is_none()));
        // A person-indexed-only possessive has no lemma-keyed paradigm.
        assert_eq!(pronoun_form_paradigm("вашь"), Ok(Vec::new()));
        // Unknown lemma propagates.
        assert!(matches!(
            numeral_form_paradigm("nonexistent"),
            Err(Error::UnknownLemma(_))
        ));
    }

    #[test]
    fn unknown_lemma_propagates_from_every_family() {
        assert!(matches!(
            noun_paradigm("nonexistent"),
            Err(Error::UnknownLemma(_))
        ));
        assert!(matches!(
            verb_paradigm("nonexistent"),
            Err(Error::UnknownLemma(_))
        ));
    }
}
