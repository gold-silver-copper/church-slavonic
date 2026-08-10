use synodal_church_slavonic_core::{
    FormSet, GenerationPolicy, GrammarCell, LexemeId, OrthographyProfile, Result, SynodalWord,
};

use crate::{LexemeSummary, registry, resolver};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Inflector {
    generation_policy: GenerationPolicy,
    orthography: OrthographyProfile,
    productive_mapping_threshold_basis_points: u16,
}

impl Default for Inflector {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Inflector {
    #[must_use]
    pub const fn builder() -> InflectorBuilder {
        InflectorBuilder {
            generation_policy: GenerationPolicy::Strict,
            orthography: OrthographyProfile::Expanded,
            productive_mapping_threshold_basis_points: 9_500,
        }
    }

    #[must_use]
    pub const fn generation_policy(self) -> GenerationPolicy {
        self.generation_policy
    }

    #[must_use]
    pub const fn orthography(self) -> OrthographyProfile {
        self.orthography
    }

    #[must_use]
    pub const fn productive_mapping_threshold_basis_points(self) -> u16 {
        self.productive_mapping_threshold_basis_points
    }

    pub fn resolve(self, lemma: &str) -> Result<LexemeSummary> {
        registry::resolve(&SynodalWord::parse(lemma)?)
    }

    pub fn from_id(self, id: &LexemeId) -> Result<LexemeSummary> {
        registry::from_id(id)
    }

    pub fn form(self, lemma: &str, cell: GrammarCell) -> Result<FormSet> {
        let lexeme = self.resolve(lemma)?;
        self.form_by_id(lexeme.id(), cell)
    }

    pub fn form_by_id(self, id: &LexemeId, cell: GrammarCell) -> Result<FormSet> {
        resolver::resolve_cell(self, id, cell)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InflectorBuilder {
    generation_policy: GenerationPolicy,
    orthography: OrthographyProfile,
    productive_mapping_threshold_basis_points: u16,
}

impl InflectorBuilder {
    #[must_use]
    pub const fn generation_policy(mut self, value: GenerationPolicy) -> Self {
        self.generation_policy = value;
        self
    }

    #[must_use]
    pub const fn orthography(mut self, value: OrthographyProfile) -> Self {
        self.orthography = value;
        self
    }

    #[must_use]
    pub const fn productive_mapping_threshold_basis_points(mut self, value: u16) -> Self {
        self.productive_mapping_threshold_basis_points =
            if value > 10_000 { 10_000 } else { value };
        self
    }

    #[must_use]
    pub const fn build(self) -> Inflector {
        Inflector {
            generation_policy: self.generation_policy,
            orthography: self.orthography,
            productive_mapping_threshold_basis_points: self
                .productive_mapping_threshold_basis_points,
        }
    }
}
