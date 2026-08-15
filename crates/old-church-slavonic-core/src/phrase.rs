//! Structured Old Church Slavonic analytic constructions.

use crate::{FormSet, InflectionError, RuleId};

/// Source-described structured grammatical constructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AnalyticConstruction {
    RelativeSuperlative,
    AbsoluteSuperlativeAdverb,
    DaImperative,
    Perfect,
    Pluperfect,
    FutureInfinitive,
    FutureParticiple,
    FuturePerfect,
    ConditionalOptative,
    DaConditionalOptative,
    EllipticalConditionalOptative,
    ConditionalOptativePassive,
    AnalyticPassive,
    ImpersonalPredicate,
    /// A word or structured sequence in the `ни-/нѣ-` and postpositive
    /// pronominal families described by Polivanova §316.
    PronominalFamily,
}

impl AnalyticConstruction {
    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::RelativeSuperlative => RuleId::PhraseRelativeSuperlative,
            Self::AbsoluteSuperlativeAdverb => RuleId::PhraseAbsoluteSuperlativeAdverb,
            Self::DaImperative => RuleId::PhraseDaImperative,
            Self::Perfect => RuleId::PhrasePerfect,
            Self::Pluperfect => RuleId::PhrasePluperfect,
            Self::FutureInfinitive => RuleId::PhraseFutureInfinitive,
            Self::FutureParticiple => RuleId::PhraseFutureParticiple,
            Self::FuturePerfect => RuleId::PhraseFuturePerfect,
            Self::ConditionalOptative => RuleId::PhraseConditionalOptative,
            Self::DaConditionalOptative => RuleId::PhraseConditionalOptativeDa,
            Self::EllipticalConditionalOptative => RuleId::PhraseConditionalOptativeElliptical,
            Self::ConditionalOptativePassive => RuleId::PhraseConditionalOptativePassive,
            Self::AnalyticPassive => RuleId::PhraseAnalyticPassive,
            Self::ImpersonalPredicate => RuleId::PhraseImpersonalPredicate,
            Self::PronominalFamily => RuleId::PronounDerivedFamily,
        }
    }
}

/// The grammatical contribution of one independently realized phrase token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhraseRole {
    Adverb,
    PositiveAdjective,
    ComparativeAdjective,
    ComparisonReference,
    Particle,
    Conjunction,
    Auxiliary,
    AuxiliaryParticiple,
    FiniteVerb,
    Infinitive,
    LParticiple,
    ActiveParticiple,
    PassiveParticiple,
    Complement,
    PrefixalFormative,
    Preposition,
    Pronoun,
    Numeral,
    Postpositive,
}

/// One phrase component with all of its variants, evidence, warnings, and trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseToken {
    pub role: PhraseRole,
    pub forms: FormSet,
}

/// Whether a modifier or reference precedes or follows its head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PhraseOrder {
    DependentFirst,
    HeadFirst,
}

/// The three source-described auxiliary formations of the pluperfect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PluperfectAuxiliary {
    Imperfect,
    Aorist,
    Perfect,
}

/// Lexical auxiliaries licensed with an infinitive to express future time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FutureInfinitiveAuxiliary {
    Vochati,
    Nachati,
    Imeti,
    Khoteti,
}

impl FutureInfinitiveAuxiliary {
    pub const ALL: [Self; 4] = [Self::Vochati, Self::Nachati, Self::Imeti, Self::Khoteti];

    pub const fn lemma(self) -> &'static str {
        match self {
            Self::Vochati => "въчѧти",
            Self::Nachati => "начѧти",
            Self::Imeti => "имѣти",
            Self::Khoteti => "хотѣти",
        }
    }
}

/// Whether an infinitival future is located from the speech time or from a
/// past reference point. The past options are source-licensed only for
/// `имѣти` and `хотѣти`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FutureReferenceTense {
    Present,
    Imperfect,
    Aorist,
}

/// The dedicated conditional series or the source-described aorist replacement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConditionalAuxiliary {
    Conditional,
    AoristReplacement,
}

/// Copular series licensed with a passive participle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PassiveAuxiliary {
    Present,
    Imperfect,
    Aorist,
    Future,
    Conditional,
    ConditionalAoristReplacement,
}

/// A typed phrase whose word-level components retain independent provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealizedPhrase {
    construction: AnalyticConstruction,
    tokens: Vec<PhraseToken>,
}

impl RealizedPhrase {
    pub fn new(
        construction: AnalyticConstruction,
        tokens: Vec<PhraseToken>,
    ) -> Result<Self, InflectionError> {
        let roles = tokens.iter().map(|token| token.role).collect::<Vec<_>>();
        if !valid_roles(construction, &roles) {
            return Err(InflectionError::InvalidInput {
                reason: format!(
                    "the token roles {roles:?} do not form a valid {construction:?} construction"
                ),
            });
        }
        Ok(Self {
            construction,
            tokens,
        })
    }

    pub const fn construction(&self) -> AnalyticConstruction {
        self.construction
    }

    pub const fn rule_id(&self) -> RuleId {
        self.construction.rule_id()
    }

    pub fn tokens(&self) -> &[PhraseToken] {
        &self.tokens
    }

    /// Render deterministic source-first token choices without losing the
    /// structured token results available through [`Self::tokens`].
    pub fn primary_text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.forms.primary_text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn valid_roles(construction: AnalyticConstruction, roles: &[PhraseRole]) -> bool {
    use AnalyticConstruction::*;
    use PhraseRole::*;
    match construction {
        RelativeSuperlative => unordered_pair(roles, ComparativeAdjective, ComparisonReference),
        AbsoluteSuperlativeAdverb => unordered_pair(roles, PositiveAdjective, Adverb),
        DaImperative => roles == [Particle, FiniteVerb],
        Perfect | FuturePerfect | ConditionalOptative => {
            unordered_pair(roles, LParticiple, Auxiliary)
        }
        Pluperfect => {
            unordered_pair(roles, LParticiple, Auxiliary)
                || roles == [LParticiple, AuxiliaryParticiple, Auxiliary]
                || roles == [AuxiliaryParticiple, Auxiliary, LParticiple]
        }
        FutureInfinitive => unordered_pair(roles, Infinitive, Auxiliary),
        FutureParticiple => unordered_pair(roles, ActiveParticiple, Auxiliary),
        DaConditionalOptative => {
            roles == [Particle, LParticiple, Auxiliary]
                || roles == [Particle, Auxiliary, LParticiple]
        }
        EllipticalConditionalOptative => roles == [LParticiple],
        ConditionalOptativePassive => unordered_pair(roles, PassiveParticiple, Auxiliary),
        AnalyticPassive => unordered_pair(roles, PassiveParticiple, Auxiliary),
        ImpersonalPredicate => roles == [FiniteVerb] || roles == [FiniteVerb, Particle],
        PronominalFamily => matches!(
            roles,
            [Pronoun]
                | [Pronoun, Postpositive]
                | [PrefixalFormative, Pronoun]
                | [PrefixalFormative, Pronoun, Postpositive]
                | [PrefixalFormative, Preposition, Pronoun]
                | [PrefixalFormative, Preposition, Pronoun, Postpositive]
        ),
    }
}

fn unordered_pair(roles: &[PhraseRole], first: PhraseRole, second: PhraseRole) -> bool {
    roles == [first, second] || roles == [second, first]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FormAnalysis, FormSource, FormVariant, InflectionWarning, MetadataEvidence,
        MetadataProvenance,
    };

    fn form(text: &str) -> FormSet {
        let variant = FormVariant {
            text: text.to_string(),
            romanization: None,
        };
        FormSet::new(
            text,
            variant.clone(),
            Vec::new(),
            FormSource::ExplicitMetadataRule {
                rule_id: RuleId::PhrasePerfect,
            },
            vec![InflectionWarning::PredictedNotDictionaryBacked],
            Vec::new(),
            vec![FormAnalysis {
                variants: vec![variant],
                source: FormSource::ExplicitMetadataRule {
                    rule_id: RuleId::PhrasePerfect,
                },
                evidence: vec![MetadataEvidence {
                    field: None,
                    provenance: MetadataProvenance::ExplicitCallerMetadata,
                    source_feature: None,
                    source_form: None,
                    crosscheck_features: Vec::new(),
                    authority: None,
                }],
                trace: Vec::new(),
            }],
        )
    }

    #[test]
    fn phrases_keep_tokens_and_reject_wrong_role_shapes() {
        let phrase = RealizedPhrase::new(
            AnalyticConstruction::Perfect,
            vec![
                PhraseToken {
                    role: PhraseRole::LParticiple,
                    forms: form("неслъ"),
                },
                PhraseToken {
                    role: PhraseRole::Auxiliary,
                    forms: form("ѥсмь"),
                },
            ],
        )
        .expect("well-shaped phrase");
        assert_eq!(phrase.primary_text(), "неслъ ѥсмь");
        assert_eq!(phrase.tokens().len(), 2);
        assert_eq!(phrase.rule_id(), RuleId::PhrasePerfect);

        assert!(matches!(
            RealizedPhrase::new(
                AnalyticConstruction::Perfect,
                vec![PhraseToken {
                    role: PhraseRole::Infinitive,
                    forms: form("нести"),
                }],
            ),
            Err(InflectionError::InvalidInput { .. })
        ));

        assert!(matches!(
            RealizedPhrase::new(
                AnalyticConstruction::DaImperative,
                vec![PhraseToken {
                    role: PhraseRole::FiniteVerb,
                    forms: form("благословлю"),
                }],
            ),
            Err(InflectionError::InvalidInput { .. })
        ));
    }
}
