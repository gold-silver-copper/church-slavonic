use crate::{
    Assumption, Confidence, Contradiction, Error, Evidence, FormSource, Loss, Recension,
    RecensionMappingId, Result, RuleTrace, VariantPolicy,
};

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Romanization {
    pub scheme: String,
    pub text: String,
    pub losses: Vec<Loss>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FormVariant {
    pub expanded: String,
    pub accented: Option<String>,
    pub printed: String,
    pub romanization: Option<Romanization>,
    pub source_recension: Option<Recension>,
    pub target_recension: Recension,
    pub recension_mapping: Option<RecensionMappingId>,
    pub confidence: Confidence,
    pub source: FormSource,
    pub assumptions: Vec<Assumption>,
    pub evidence: Vec<Evidence>,
    pub contradictions: Vec<Contradiction>,
    pub warnings: Vec<String>,
    pub rule_trace: RuleTrace,
}

impl FormVariant {
    pub fn validate(self) -> Result<Self> {
        if self.expanded.is_empty() || self.printed.is_empty() {
            return Err(Error::EmptyFormSet);
        }
        if self.target_recension != Recension::SynodalRussian {
            return Err(Error::ContradictoryMetadata {
                reason: "every Synodal form variant must identify SynodalRussian as target".into(),
            });
        }
        match &self.source {
            FormSource::SynodalAttestation { .. }
            | FormSource::SynodalNormativeGeneration { .. }
            | FormSource::SynodalIrregularOverride { .. }
            | FormSource::CallerSpecifiedPrediction { .. } => {
                if self.source_recension != Some(Recension::SynodalRussian)
                    || self.recension_mapping.is_some()
                {
                    return Err(Error::ContradictoryMetadata {
                        reason: "direct Synodal forms require a Synodal source recension and no cross-recension mapping".into(),
                    });
                }
            }
            FormSource::InheritedPrediction {
                source_recension,
                mapping,
                ..
            } => {
                if *source_recension != Recension::OldChurchSlavonic
                    || self.source_recension != Some(*source_recension)
                    || self.recension_mapping.as_ref() != Some(mapping)
                {
                    return Err(Error::ContradictoryMetadata {
                        reason: "inherited OCS forms require matching OCS source-recension and mapping fields".into(),
                    });
                }
            }
            FormSource::AnalogicalPrediction { .. } => {
                if self.recension_mapping.is_some() {
                    return Err(Error::ContradictoryMetadata {
                        reason: "an analogical prediction cannot masquerade as a reviewed recension mapping".into(),
                    });
                }
            }
        }
        Ok(self)
    }

    #[must_use]
    pub const fn is_attested(&self) -> bool {
        self.source.is_attested()
    }

    #[must_use]
    pub const fn is_predicted(&self) -> bool {
        self.source.is_prediction()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(try_from = "Vec<FormVariant>", into = "Vec<FormVariant>")
)]
pub struct FormSet {
    variants: Vec<FormVariant>,
}

impl FormSet {
    pub fn new(variant: FormVariant) -> Result<Self> {
        Self::try_from_variants(vec![variant])
    }

    pub fn try_from_variants(variants: Vec<FormVariant>) -> Result<Self> {
        if variants.is_empty() {
            return Err(Error::EmptyFormSet);
        }
        let variants = variants
            .into_iter()
            .map(FormVariant::validate)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { variants })
    }

    #[must_use]
    pub fn primary_text(&self) -> &str {
        &self.variants[0].printed
    }

    #[must_use]
    pub fn primary(&self) -> &FormVariant {
        &self.variants[0]
    }

    pub fn texts(&self) -> impl Iterator<Item = &str> {
        self.variants.iter().map(|variant| variant.printed.as_str())
    }

    pub fn unique_text(&self) -> Result<&str> {
        let primary = self.primary_text();
        if self
            .variants
            .iter()
            .all(|variant| variant.printed == primary)
        {
            Ok(primary)
        } else {
            Err(Error::AmbiguousVariant {
                count: self.variants.len(),
            })
        }
    }

    #[must_use]
    pub fn variants(&self) -> &[FormVariant] {
        &self.variants
    }

    pub fn select(&self, policy: VariantPolicy) -> Result<VariantSelection<'_>> {
        let variant = match policy {
            VariantPolicy::First => &self.variants[0],
            VariantPolicy::Unique => {
                self.unique_text()?;
                &self.variants[0]
            }
            VariantPolicy::AttestedFirst => self
                .variants
                .iter()
                .find(|variant| variant.source.is_attested())
                .unwrap_or(&self.variants[0]),
            VariantPolicy::NormativeFirst => self
                .variants
                .iter()
                .min_by_key(|variant| variant.source.precedence())
                .unwrap_or(&self.variants[0]),
        };
        Ok(VariantSelection { variant })
    }

    #[must_use]
    pub fn common_source_recension(&self) -> Option<Recension> {
        let first = self.variants[0].source_recension?;
        self.variants
            .iter()
            .all(|variant| variant.source_recension == Some(first))
            .then_some(first)
    }

    /// Returns the shared source recension, if every variant has the same
    /// declared source recension. Prefer per-variant fields when this is `None`.
    #[must_use]
    pub fn source_recension(&self) -> Option<Recension> {
        self.common_source_recension()
    }

    #[must_use]
    pub const fn target_recension(&self) -> Recension {
        Recension::SynodalRussian
    }

    pub fn recension_mappings(&self) -> impl Iterator<Item = &RecensionMappingId> {
        self.variants
            .iter()
            .filter_map(|variant| variant.recension_mapping.as_ref())
    }

    /// Returns one common mapping without concealing per-variant differences.
    pub fn recension_mapping(&self) -> Result<Option<&RecensionMappingId>> {
        let first = self.variants[0].recension_mapping.as_ref();
        if self
            .variants
            .iter()
            .all(|variant| variant.recension_mapping.as_ref() == first)
        {
            Ok(first)
        } else {
            Err(Error::AmbiguousVariant {
                count: self.variants.len(),
            })
        }
    }

    pub fn provenance(&self) -> impl Iterator<Item = &FormSource> {
        self.variants.iter().map(|variant| &variant.source)
    }

    pub fn rule_traces(&self) -> impl Iterator<Item = &RuleTrace> {
        self.variants.iter().map(|variant| &variant.rule_trace)
    }

    /// Returns one common trace only when no variant-specific path would be
    /// hidden. Use `rule_traces()` for an ambiguous set.
    pub fn rule_trace(&self) -> Result<&RuleTrace> {
        let first = &self.variants[0].rule_trace;
        if self
            .variants
            .iter()
            .all(|variant| &variant.rule_trace == first)
        {
            Ok(first)
        } else {
            Err(Error::AmbiguousVariant {
                count: self.variants.len(),
            })
        }
    }

    pub fn attested(&self) -> impl Iterator<Item = &FormVariant> {
        self.variants.iter().filter(|variant| variant.is_attested())
    }

    pub fn predicted(&self) -> impl Iterator<Item = &FormVariant> {
        self.variants
            .iter()
            .filter(|variant| variant.is_predicted())
    }
}

impl TryFrom<Vec<FormVariant>> for FormSet {
    type Error = Error;

    fn try_from(value: Vec<FormVariant>) -> Result<Self> {
        Self::try_from_variants(value)
    }
}

impl From<FormSet> for Vec<FormVariant> {
    fn from(value: FormSet) -> Self {
        value.variants
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VariantSelection<'a> {
    variant: &'a FormVariant,
}

impl<'a> VariantSelection<'a> {
    #[must_use]
    pub const fn variant(self) -> &'a FormVariant {
        self.variant
    }

    #[must_use]
    pub fn text(self) -> &'a str {
        &self.variant.printed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EvidenceId, RuleId};

    fn variant(text: &str, source: FormSource) -> FormVariant {
        FormVariant {
            expanded: text.into(),
            accented: None,
            printed: text.into(),
            romanization: None,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            recension_mapping: None,
            confidence: Confidence::CERTAIN,
            source,
            assumptions: vec![],
            evidence: vec![],
            contradictions: vec![],
            warnings: vec![],
            rule_trace: RuleTrace::default(),
        }
    }

    #[test]
    fn attestation_outranks_generation() {
        let forms = FormSet::try_from_variants(vec![
            variant(
                "generated",
                FormSource::SynodalNormativeGeneration {
                    rule: RuleId::from("test-rule"),
                },
            ),
            variant(
                "attested",
                FormSource::SynodalAttestation {
                    evidence: EvidenceId::from("test-evidence"),
                },
            ),
        ])
        .expect("nonempty forms");
        assert_eq!(
            forms
                .select(VariantPolicy::NormativeFirst)
                .expect("selection")
                .text(),
            "attested"
        );
    }

    #[test]
    fn refuses_non_synodal_target() {
        let mut candidate = variant(
            "слово",
            FormSource::SynodalAttestation {
                evidence: EvidenceId::from("test-evidence"),
            },
        );
        candidate.target_recension = Recension::OldChurchSlavonic;
        assert!(FormSet::new(candidate).is_err());
    }

    #[test]
    fn refuses_inconsistent_cross_recension_provenance() {
        let mapping = crate::RecensionMappingId::from("map:reviewed");
        let mut candidate = variant(
            "градъ",
            FormSource::InheritedPrediction {
                source_recension: Recension::OldChurchSlavonic,
                mapping: mapping.clone(),
                rule: RuleId::from("target-rule"),
            },
        );
        candidate.source_recension = Some(Recension::OldChurchSlavonic);
        candidate.recension_mapping = Some(crate::RecensionMappingId::from("map:different"));
        assert!(matches!(
            FormSet::new(candidate),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let mut mislabeled = variant(
            "градъ",
            FormSource::SynodalAttestation {
                evidence: EvidenceId::from("target:attestation"),
            },
        );
        mislabeled.source_recension = Some(Recension::OldChurchSlavonic);
        assert!(matches!(
            FormSet::new(mislabeled),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }

    #[test]
    fn aggregate_accessors_do_not_hide_variant_specific_provenance() {
        let first_mapping = crate::RecensionMappingId::from("map:first");
        let mut first = variant(
            "градъ",
            FormSource::InheritedPrediction {
                source_recension: Recension::OldChurchSlavonic,
                mapping: first_mapping.clone(),
                rule: RuleId::from("first"),
            },
        );
        first.source_recension = Some(Recension::OldChurchSlavonic);
        first.recension_mapping = Some(first_mapping);
        let second_mapping = crate::RecensionMappingId::from("map:second");
        let mut second = variant(
            "градъ",
            FormSource::InheritedPrediction {
                source_recension: Recension::OldChurchSlavonic,
                mapping: second_mapping.clone(),
                rule: RuleId::from("second"),
            },
        );
        second.source_recension = Some(Recension::OldChurchSlavonic);
        second.recension_mapping = Some(second_mapping);
        second.rule_trace = RuleTrace::new(vec![crate::TraceStep {
            rule: RuleId::from("second"),
            stage: "test".into(),
            input: "градъ".into(),
            output: "градъ".into(),
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            mapping: second.recension_mapping.clone(),
            evidence: vec![],
        }]);
        let forms = FormSet::try_from_variants(vec![first, second]).expect("valid variants");
        assert!(matches!(
            forms.recension_mapping(),
            Err(Error::AmbiguousVariant { count: 2 })
        ));
        assert!(matches!(
            forms.rule_trace(),
            Err(Error::AmbiguousVariant { count: 2 })
        ));
    }
}
