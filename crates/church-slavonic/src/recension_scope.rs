//! The recension dimension of the facade (merge phase 5,
//! docs/UNIFIED_FACADE.md): a [`Recension`]-scoped handle whose methods
//! mirror the facade's function families, resolved through the shared
//! identity layer (docs/UNIFIED_IDENTITY.md).
//!
//! Design decision (recorded in docs/UNIFIED_FACADE.md §1): recension
//! selects *realization* — it is a profile, not a per-cell linguistic
//! dimension — so it rides on a scoped handle, never as an extra parameter
//! on every free function. The existing free functions are the OCS
//! compatibility surface, untouched;
//! `recension(Recension::OldChurchSlavonic)` delegates to them.
//!
//! Lemma resolution inside a scope is two steps (docs/UNIFIED_FACADE.md §2):
//! an abstract identity key (`<pos>:<form>[_<ordinal>]`, always containing
//! `:`) resolves through the identity table to the scope recension's native
//! handle — the table is the authority where an entry exists; anything else
//! is resolved as a native key of the scope's recension, so native keys
//! keep working for lexemes not yet identified.

use std::sync::OnceLock;

use church_slavonic_core::identity::IdentityRegistry;
pub use church_slavonic_core::recension::Recension;
use synodal_church_slavonic::Inflector;
use synodal_church_slavonic::core::{
    AdjectiveCell, Animacy, Comparison, FiniteTense, FiniteVerbCell, FormSet, GrammarCell,
    ImperativeCell, LParticipleCell, LexemeId, NounCell,
};

use crate::paradigm::{AdjectiveParadigm, NounParadigm, VerbParadigm};
use crate::{AdjectiveForm, Case, Error, Gender, Number, Person, VerbCellKind};

/// The committed shared identity table (`data/unified/identity.tsv`),
/// parsed once on first use. The `unified-identity --check` gate holds the
/// underlying file byte-stable; the deprecation release that publishes
/// 0.3.0 vendors it into the crate package (docs/UNIFIED_FACADE.md §3).
const IDENTITY_TSV: &str = include_str!("../../../data/unified/identity.tsv");

/// The shared lexeme-identity registry the facade ships: abstract identity
/// keys with both recensions' citation surfaces and native handles.
pub fn identity_registry() -> &'static IdentityRegistry {
    static REGISTRY: OnceLock<IdentityRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        IdentityRegistry::parse(IDENTITY_TSV).expect("committed identity table parses")
    })
}

/// Opens a scope over one recension. Total and infallible: an unservable
/// recension surfaces as [`Error::UnsupportedRecension`] from every method
/// of the returned scope, so a recension can be threaded through
/// configuration without a guard at every construction site.
#[must_use]
pub const fn recension(recension: Recension) -> RecensionScope {
    RecensionScope { recension }
}

/// A [`Recension`]-scoped view of the facade: the same function families,
/// realized in the scope's recension. See the module docs and
/// docs/UNIFIED_FACADE.md for the resolution and error contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecensionScope {
    recension: Recension,
}

/// A scope-resolved lemma: the native handle of the scope's recension.
enum Resolved<'a> {
    /// An OCS facade lemma key (homograph suffix included).
    Ocs(&'a str),
    /// A Synodal registry lexeme id, usable with the `Inflector`.
    Synodal(LexemeId),
}

impl RecensionScope {
    /// The recension this scope realizes.
    #[must_use]
    pub const fn recension(self) -> Recension {
        self.recension
    }

    /// Two-step lemma resolution: identity table first (authoritative where
    /// an entry of the requested part of speech exists), then the native
    /// key space of the scope's recension.
    fn resolve<'a>(self, lemma: &'a str, pos: &str) -> Result<Resolved<'a>, Error> {
        let identified = lemma
            .contains(':')
            .then(|| identity_registry().resolve(lemma))
            .flatten()
            .filter(|entry| entry.pos == pos);
        match self.recension {
            Recension::OldChurchSlavonic => match identified {
                Some(entry) => match entry.ocs_lemma_key.as_deref() {
                    Some(key) => Ok(Resolved::Ocs(key)),
                    None => Err(Error::NotInRecension {
                        lemma: lemma.to_owned(),
                        recension: self.recension,
                    }),
                },
                None if lemma.contains(':') => Err(Error::UnidentifiedLemma {
                    lemma: lemma.to_owned(),
                    recension: self.recension,
                }),
                None => Ok(Resolved::Ocs(lemma)),
            },
            Recension::SynodalRussian => match identified {
                Some(entry) => Ok(Resolved::Synodal(LexemeId::new(
                    entry.synodal_lexeme_id.clone(),
                ))),
                None => Inflector::default()
                    .resolve(lemma)
                    .map(|summary| Resolved::Synodal(summary.id().clone()))
                    .map_err(|_| Error::UnidentifiedLemma {
                        lemma: lemma.to_owned(),
                        recension: self.recension,
                    }),
            },
            other => Err(Error::UnsupportedRecension { recension: other }),
        }
    }

    /// Resolves one Synodal cell through the `Inflector`, keeping the
    /// facade's error semantics: a cell the engine cannot commit to is
    /// [`Error::Underdetermined`].
    fn synodal_cell(id: &LexemeId, lemma: &str, cell: GrammarCell) -> Result<Vec<String>, Error> {
        Inflector::default()
            .form_by_id(id, cell)
            .map(|forms| texts(&forms))
            .map_err(|_| Error::Underdetermined {
                lemma: lemma.to_owned(),
            })
    }

    /// A noun cell in the Synodal realization. The lexeme's own animacy
    /// inventory decides the accusative: the inanimate convention first
    /// (matching the OCS stored-table convention), animate where the
    /// lexeme licenses only that.
    fn synodal_noun_cell(
        id: &LexemeId,
        lemma: &str,
        case: Case,
        number: Number,
    ) -> Result<Vec<String>, Error> {
        let mut last = Error::Underdetermined {
            lemma: lemma.to_owned(),
        };
        for animacy in [Animacy::Inanimate, Animacy::Animate] {
            let cell = GrammarCell::Noun(NounCell {
                case,
                number,
                animacy,
            });
            match Self::synodal_cell(id, lemma, cell) {
                Ok(forms) => return Ok(forms),
                Err(error) => last = error,
            }
        }
        Err(last)
    }

    // ------------------------------------------------------------------
    // Nouns
    // ------------------------------------------------------------------

    /// Every variant of one noun cell in the scope's recension.
    pub fn noun_variants(
        self,
        lemma: &str,
        case: Case,
        number: Number,
    ) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "noun")? {
            Resolved::Ocs(key) => crate::noun_variants(key, case, number),
            Resolved::Synodal(id) => Self::synodal_noun_cell(&id, lemma, case, number),
        }
    }

    /// The primary variant of one noun cell in the scope's recension.
    pub fn noun(self, lemma: &str, case: Case, number: Number) -> Result<String, Error> {
        self.noun_variants(lemma, case, number)
            .map(|variants| variants[0].clone())
    }

    /// Every servable noun cell of one lexeme, in `Case::ALL` x
    /// `Number::ALL` order; underdetermined cells are absent, not errors.
    pub fn noun_paradigm(self, lemma: &str) -> Result<NounParadigm, Error> {
        match self.resolve(lemma, "noun")? {
            Resolved::Ocs(key) => crate::noun_paradigm(key),
            Resolved::Synodal(id) => {
                let mut cells = NounParadigm::new();
                for case in Case::ALL {
                    for number in Number::ALL {
                        if let Ok(variants) = Self::synodal_noun_cell(&id, lemma, case, number) {
                            cells.push((case, number, variants));
                        }
                    }
                }
                Ok(cells)
            }
        }
    }

    // ------------------------------------------------------------------
    // Adjectives
    // ------------------------------------------------------------------

    fn synodal_adjective_cell(
        id: &LexemeId,
        lemma: &str,
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<Vec<String>, Error> {
        let mut last = Error::Underdetermined {
            lemma: lemma.to_owned(),
        };
        for animacy in [Animacy::Inanimate, Animacy::Animate] {
            let cell = GrammarCell::Adjective(AdjectiveCell {
                case,
                number,
                gender,
                animacy,
                form,
                comparison: Comparison::Positive,
            });
            match Self::synodal_cell(id, lemma, cell) {
                Ok(forms) => return Ok(forms),
                Err(error) => last = error,
            }
        }
        Err(last)
    }

    /// Every variant of one long-declension adjective cell.
    pub fn adjective_variants(
        self,
        lemma: &str,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "adj")? {
            Resolved::Ocs(key) => crate::adjective_variants(key, case, number, gender),
            Resolved::Synodal(id) => {
                Self::synodal_adjective_cell(&id, lemma, AdjectiveForm::Long, case, number, gender)
            }
        }
    }

    /// The primary variant of one long-declension adjective cell.
    pub fn adjective(
        self,
        lemma: &str,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<String, Error> {
        self.adjective_variants(lemma, case, number, gender)
            .map(|variants| variants[0].clone())
    }

    /// Every variant of one short-declension adjective cell.
    pub fn short_adjective_variants(
        self,
        lemma: &str,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "adj")? {
            Resolved::Ocs(key) => crate::short_adjective_variants(key, case, number, gender),
            Resolved::Synodal(id) => {
                Self::synodal_adjective_cell(&id, lemma, AdjectiveForm::Short, case, number, gender)
            }
        }
    }

    /// The primary variant of one short-declension adjective cell.
    pub fn short_adjective(
        self,
        lemma: &str,
        case: Case,
        number: Number,
        gender: Gender,
    ) -> Result<String, Error> {
        self.short_adjective_variants(lemma, case, number, gender)
            .map(|variants| variants[0].clone())
    }

    /// Every servable adjective cell of one lexeme and declension form.
    pub fn adjective_paradigm(
        self,
        lemma: &str,
        form: AdjectiveForm,
    ) -> Result<AdjectiveParadigm, Error> {
        match self.resolve(lemma, "adj")? {
            Resolved::Ocs(key) => crate::adjective_paradigm(key, form),
            Resolved::Synodal(id) => {
                let mut cells = AdjectiveParadigm::new();
                for case in Case::ALL {
                    for number in Number::ALL {
                        for gender in Gender::ALL {
                            if let Ok(variants) =
                                Self::synodal_adjective_cell(&id, lemma, form, case, number, gender)
                            {
                                cells.push((case, number, gender, variants));
                            }
                        }
                    }
                }
                Ok(cells)
            }
        }
    }

    // ------------------------------------------------------------------
    // Verbs
    // ------------------------------------------------------------------

    fn synodal_verb_cell(
        id: &LexemeId,
        lemma: &str,
        kind: VerbCellKind,
    ) -> Result<Vec<String>, Error> {
        let cell = match kind {
            VerbCellKind::Present { person, number } => GrammarCell::FiniteVerb(FiniteVerbCell {
                tense: FiniteTense::Present,
                person,
                number,
            }),
            VerbCellKind::Imperfect { person, number } => GrammarCell::FiniteVerb(FiniteVerbCell {
                tense: FiniteTense::Imperfect,
                person,
                number,
            }),
            VerbCellKind::Aorist { person, number } => GrammarCell::FiniteVerb(FiniteVerbCell {
                tense: FiniteTense::Aorist,
                person,
                number,
            }),
            VerbCellKind::Imperative { person, number } => {
                GrammarCell::Imperative(ImperativeCell { person, number })
            }
            VerbCellKind::LParticiple { gender, number } => {
                GrammarCell::LParticiple(LParticipleCell { gender, number })
            }
            VerbCellKind::Infinitive => GrammarCell::Infinitive,
            _ => {
                return Err(Error::Underdetermined {
                    lemma: lemma.to_owned(),
                });
            }
        };
        Self::synodal_cell(id, lemma, cell)
    }

    /// Every variant of one present-tense cell.
    pub fn present_variants(
        self,
        lemma: &str,
        person: Person,
        number: Number,
    ) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "verb")? {
            Resolved::Ocs(key) => crate::present_variants(key, person, number),
            Resolved::Synodal(id) => {
                Self::synodal_verb_cell(&id, lemma, VerbCellKind::Present { person, number })
            }
        }
    }

    /// The primary variant of one present-tense cell.
    pub fn present(self, lemma: &str, person: Person, number: Number) -> Result<String, Error> {
        self.present_variants(lemma, person, number)
            .map(|variants| variants[0].clone())
    }

    /// Every variant of one imperfect-tense cell.
    pub fn imperfect_variants(
        self,
        lemma: &str,
        person: Person,
        number: Number,
    ) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "verb")? {
            Resolved::Ocs(key) => crate::imperfect_variants(key, person, number),
            Resolved::Synodal(id) => {
                Self::synodal_verb_cell(&id, lemma, VerbCellKind::Imperfect { person, number })
            }
        }
    }

    /// The primary variant of one imperfect-tense cell.
    pub fn imperfect(self, lemma: &str, person: Person, number: Number) -> Result<String, Error> {
        self.imperfect_variants(lemma, person, number)
            .map(|variants| variants[0].clone())
    }

    /// Every variant of one aorist-tense cell.
    pub fn aorist_variants(
        self,
        lemma: &str,
        person: Person,
        number: Number,
    ) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "verb")? {
            Resolved::Ocs(key) => crate::aorist_variants(key, person, number),
            Resolved::Synodal(id) => {
                Self::synodal_verb_cell(&id, lemma, VerbCellKind::Aorist { person, number })
            }
        }
    }

    /// The primary variant of one aorist-tense cell.
    pub fn aorist(self, lemma: &str, person: Person, number: Number) -> Result<String, Error> {
        self.aorist_variants(lemma, person, number)
            .map(|variants| variants[0].clone())
    }

    /// Every variant of one imperative cell.
    pub fn imperative_variants(
        self,
        lemma: &str,
        person: Person,
        number: Number,
    ) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "verb")? {
            Resolved::Ocs(key) => crate::imperative_variants(key, person, number),
            Resolved::Synodal(id) => {
                Self::synodal_verb_cell(&id, lemma, VerbCellKind::Imperative { person, number })
            }
        }
    }

    /// The primary variant of one imperative cell.
    pub fn imperative(self, lemma: &str, person: Person, number: Number) -> Result<String, Error> {
        self.imperative_variants(lemma, person, number)
            .map(|variants| variants[0].clone())
    }

    /// Every variant of one l-participle cell.
    pub fn l_participle_variants(
        self,
        lemma: &str,
        gender: Gender,
        number: Number,
    ) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "verb")? {
            Resolved::Ocs(key) => crate::l_participle_variants(key, gender, number),
            Resolved::Synodal(id) => {
                Self::synodal_verb_cell(&id, lemma, VerbCellKind::LParticiple { gender, number })
            }
        }
    }

    /// The primary variant of one l-participle cell.
    pub fn l_participle(
        self,
        lemma: &str,
        gender: Gender,
        number: Number,
    ) -> Result<String, Error> {
        self.l_participle_variants(lemma, gender, number)
            .map(|variants| variants[0].clone())
    }

    /// Every variant of the infinitive citation.
    pub fn infinitive_variants(self, lemma: &str) -> Result<Vec<String>, Error> {
        match self.resolve(lemma, "verb")? {
            Resolved::Ocs(key) => crate::infinitive_variants(key),
            Resolved::Synodal(id) => Self::synodal_verb_cell(&id, lemma, VerbCellKind::Infinitive),
        }
    }

    /// The primary variant of the infinitive citation.
    pub fn infinitive(self, lemma: &str) -> Result<String, Error> {
        self.infinitive_variants(lemma)
            .map(|variants| variants[0].clone())
    }

    /// Every servable verb cell of one lexeme. Under the Synodal recension
    /// the enumeration covers the mapped [`VerbCellKind`]s (finite tenses,
    /// imperative, l-participle, infinitive); the OCS enumeration is the
    /// facade's full one.
    pub fn verb_paradigm(self, lemma: &str) -> Result<VerbParadigm, Error> {
        match self.resolve(lemma, "verb")? {
            Resolved::Ocs(key) => crate::verb_paradigm(key),
            Resolved::Synodal(id) => {
                let mut cells = VerbParadigm::new();
                for kind in VerbCellKind::all() {
                    if let Ok(variants) = Self::synodal_verb_cell(&id, lemma, kind) {
                        cells.push((kind, variants));
                    }
                }
                Ok(cells)
            }
        }
    }
}

fn texts(forms: &FormSet) -> Vec<String> {
    forms.texts().map(str::to_owned).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: RecensionScope = RecensionScope {
        recension: Recension::OldChurchSlavonic,
    };
    const SYNODAL: RecensionScope = RecensionScope {
        recension: Recension::SynodalRussian,
    };

    /// First identified noun the Synodal engine can realize in the
    /// nominative singular (the Synodal side is evidence-gated, so not
    /// every identified lexeme serves every cell).
    fn servable_noun_key() -> &'static str {
        identity_registry()
            .entries()
            .iter()
            .find(|entry| {
                entry.pos == "noun"
                    && SYNODAL
                        .noun(&entry.abstract_key, Case::Nominative, Number::Singular)
                        .is_ok()
            })
            .expect("some identified noun inflects on the Synodal side")
            .abstract_key
            .as_str()
    }

    #[test]
    fn identity_table_parses_and_is_nonempty() {
        assert!(identity_registry().len() > 500);
    }

    #[test]
    fn ocs_scope_matches_the_free_functions() {
        let entry = identity_registry()
            .entries()
            .iter()
            .find(|entry| entry.pos == "noun" && entry.ocs_lemma_key.is_some())
            .expect("noun entry with a facade key");
        let key = entry.ocs_lemma_key.as_deref().expect("filtered to Some");
        assert_eq!(
            OCS.noun_variants(&entry.abstract_key, Case::Nominative, Number::Singular),
            crate::noun_variants(key, Case::Nominative, Number::Singular),
        );
        assert_eq!(
            OCS.noun_variants(key, Case::Nominative, Number::Singular),
            crate::noun_variants(key, Case::Nominative, Number::Singular),
        );
    }

    #[test]
    fn synodal_scope_serves_an_identified_noun() {
        let form = SYNODAL
            .noun(servable_noun_key(), Case::Nominative, Number::Singular)
            .expect("identified lemma inflects");
        assert!(!form.is_empty());
        // The OCS side of the same identity resolves through its own key.
        OCS.noun(servable_noun_key(), Case::Nominative, Number::Singular)
            .expect("OCS side inflects");
    }

    #[test]
    fn synodal_scope_paradigm_uses_the_single_cell_path() {
        let paradigm = SYNODAL
            .noun_paradigm(servable_noun_key())
            .expect("paradigm");
        assert!(!paradigm.is_empty());
        for (case, number, variants) in &paradigm {
            assert_eq!(
                SYNODAL
                    .noun_variants(servable_noun_key(), *case, *number)
                    .ok(),
                Some(variants.clone()),
            );
        }
    }

    #[test]
    fn unidentified_lemma_is_a_typed_error() {
        assert_eq!(
            SYNODAL.noun("noun:not-a-lexeme", Case::Nominative, Number::Singular),
            Err(Error::UnidentifiedLemma {
                lemma: "noun:not-a-lexeme".to_owned(),
                recension: Recension::SynodalRussian,
            }),
        );
        assert!(matches!(
            SYNODAL.noun("ⰽⰾⱏ", Case::Nominative, Number::Singular),
            Err(Error::UnidentifiedLemma { .. }),
        ));
    }

    #[test]
    fn unsupported_recension_is_a_typed_error() {
        assert_eq!(
            recension(Recension::ModernRussian).noun(
                "noun:агнецъ",
                Case::Nominative,
                Number::Singular
            ),
            Err(Error::UnsupportedRecension {
                recension: Recension::ModernRussian,
            }),
        );
    }

    #[test]
    fn native_synodal_lemmas_keep_working() {
        let entry = identity_registry()
            .resolve(servable_noun_key())
            .expect("агнецъ entry");
        let via_native = SYNODAL.noun(&entry.synodal_citation, Case::Nominative, Number::Singular);
        let via_key = SYNODAL.noun(servable_noun_key(), Case::Nominative, Number::Singular);
        assert_eq!(via_native, via_key);
    }

    #[test]
    fn synodal_verb_scope_serves_an_identified_verb() {
        let paradigm = identity_registry()
            .entries()
            .iter()
            .filter(|entry| entry.pos == "verb")
            .filter_map(|entry| SYNODAL.verb_paradigm(&entry.abstract_key).ok())
            .find(|paradigm| !paradigm.is_empty())
            .expect("some identified verb inflects on the Synodal side");
        for (kind, variants) in &paradigm {
            assert!(!variants.is_empty(), "{kind:?}");
        }
    }
}
