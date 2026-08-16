use crate::{Error, FormSet, Recension, Result};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AnalyticConstruction {
    CompoundFuture,
    Perfect,
    Pluperfect,
    FutureAnterior,
    Conditional,
    Optative,
    AnalyticPassive,
    PeriphrasticTense,
    CopulaEllipsis,
    CompositeAdverbialParticiple,
    EncliticPronoun,
    EncliticParticle,
    ThirdPersonPrepositionalContraction,
    NegativePronounPrepositional,
    CompoundCardinal,
    CompoundOrdinal,
    RepeatedDistributive,
    MultiplicativeKrat,
    FractionalPart,
}

/// The two source-attested orders of a predicate and its auxiliary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PhraseOrder {
    AuxiliaryFirst,
    PredicateFirst,
}

/// Source-attested orders of two nested auxiliary components and a predicate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum CompoundAuxiliaryOrder {
    PredicateParticipleFinite,
    PredicateFiniteParticiple,
    ParticipleFinitePredicate,
    FiniteParticiplePredicate,
}

/// The two printed subordinate-conditional orders with an added present
/// copula in Alypy §91.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ConditionalCopulaOrder {
    ConditionalPredicatePresent,
    ConditionalPresentPredicate,
}

/// Finite auxiliaries admitted by Alypy §85 and Pletneva–Kravetsky lesson 13.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum CompoundFutureAuxiliary {
    Byti,
    Imati,
    Khoteti,
    Nachati,
}

impl CompoundFutureAuxiliary {
    pub const ALL: [Self; 4] = [Self::Byti, Self::Imati, Self::Khoteti, Self::Nachati];
}

/// Source-distinct realizations of the active perfect (Alypy §88).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PerfectFormation {
    PresentCopula,
    OmittedThirdSingularCopula,
    SharedPresentCopula,
}

/// The three independently described pluperfect auxiliaries (Alypy §§89, 168).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PluperfectFormation {
    AoristBe,
    ImperfectBya,
    PerfectCopula,
}

impl PluperfectFormation {
    pub const ALL: [Self; 3] = [Self::AoristBe, Self::ImperfectBya, Self::PerfectCopula];
}

/// Lexical modal predicates whose third-singular imperfect can carry
/// conditional force (Alypy §91).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ModalConditionalAuxiliary {
    Podobati,
    Dostoyati,
    Moshchi,
}

/// Every conditional pattern explicitly distinguished in Alypy §91.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ConditionalFormation {
    PersonalAorist,
    PersonalAoristWithPresentCopula,
    InvariantBy,
    InvariantByWithPresentCopula,
    InfinitiveWithInvariantBy,
    ModalImperfect(ModalConditionalAuxiliary),
}

/// Finite system selected after the optative particle `да` (Alypy §92).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum OptativeFiniteSystem {
    Present,
    SimpleFuture,
}

/// Copular tense or mood in the `быти` + present-active-participle
/// periphrasis (Alypy §§90, 163).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PeriphrasticTenseFormation {
    Present,
    AoristBe,
    ImperfectBya,
    Future,
    Imperative,
}

impl PeriphrasticTenseFormation {
    pub const ALL: [Self; 5] = [
        Self::Present,
        Self::AoristBe,
        Self::ImperfectBya,
        Self::Future,
        Self::Imperative,
    ];
}

/// The closed semi-auxiliary inventory printed in Alypy §§90 and 163.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PeriphrasticSemiAuxiliary {
    NePrestavati,
    Prebyvati,
    Prilezhati,
    NeOskudevati,
    Prestati,
    Sovershiti,
    Yavitisya,
    Obrestisya,
}

impl PeriphrasticSemiAuxiliary {
    pub const ALL: [Self; 8] = [
        Self::NePrestavati,
        Self::Prebyvati,
        Self::Prilezhati,
        Self::NeOskudevati,
        Self::Prestati,
        Self::Sovershiti,
        Self::Yavitisya,
        Self::Obrestisya,
    ];

    #[must_use]
    pub const fn lemma(self) -> &'static str {
        match self {
            Self::NePrestavati => "преставати",
            Self::Prebyvati => "пребывати",
            Self::Prilezhati => "прилѣжати",
            Self::NeOskudevati => "ѡскудѣвати",
            Self::Prestati => "престати",
            Self::Sovershiti => "совершити",
            Self::Yavitisya => "ꙗвитися",
            Self::Obrestisya => "ѡбрѣстися",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PeriphrasticFormation {
    Copular(PeriphrasticTenseFormation),
    SemiAuxiliary(PeriphrasticSemiAuxiliary),
}

/// The exhaustive participle/auxiliary pairings in Alypy §102's passive
/// tense-and-mood table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PassiveFormation {
    PresentParticipleInfinitive,
    PastParticipleInfinitive,
    Present,
    PresentParticipleFuture,
    PastParticipleFuture,
    PresentParticipleAorist,
    PastParticipleAorist,
    PresentParticipleAoristBe,
    Imperfect,
    Perfect,
    PluperfectAoristBe,
    PluperfectImperfectBya,
    PluperfectPerfectCopula,
    PresentParticipleConditional,
    PastParticipleConditional,
    PresentParticipleImperative,
    PastParticipleImperative,
}

impl PassiveFormation {
    pub const ALL: [Self; 17] = [
        Self::PresentParticipleInfinitive,
        Self::PastParticipleInfinitive,
        Self::Present,
        Self::PresentParticipleFuture,
        Self::PastParticipleFuture,
        Self::PresentParticipleAorist,
        Self::PastParticipleAorist,
        Self::PresentParticipleAoristBe,
        Self::Imperfect,
        Self::Perfect,
        Self::PluperfectAoristBe,
        Self::PluperfectImperfectBya,
        Self::PluperfectPerfectCopula,
        Self::PresentParticipleConditional,
        Self::PastParticipleConditional,
        Self::PresentParticipleImperative,
        Self::PastParticipleImperative,
    ];
}

/// Government of an explicitly realized passive agent (Alypy §101).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PassiveAgentGovernment {
    Instrumental,
    OtGenitive,
}

/// Closed contexts in which Alypy §§88 and 123–124 explicitly license an
/// unpronounced copula.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum CopulaOmissionContext {
    PresentNominalPredicate,
    SePresent,
    SePastAorist,
    SePastImperfect,
    Imperative,
    NarrativePast,
    ImpersonalPredicate,
}

/// Composite circumstantial participles explicitly described in Alypy §146.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AdverbialParticipleFormation {
    PastPassiveWithPastActiveByti,
    PresentCopularNominal,
    PastCopularNominal,
}

/// A closed, inspectable subtype of a realized analytic construction.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PhraseFormation {
    CompoundFuture(CompoundFutureAuxiliary),
    Perfect(PerfectFormation),
    Pluperfect(PluperfectFormation),
    FutureAnterior,
    Conditional(ConditionalFormation),
    Optative(OptativeFiniteSystem),
    AnalyticPassive(PassiveFormation),
    PeriphrasticTense(PeriphrasticFormation),
    CopulaEllipsis(CopulaOmissionContext),
    CompositeAdverbialParticiple(AdverbialParticipleFormation),
}

impl PhraseFormation {
    #[must_use]
    pub const fn construction(self) -> AnalyticConstruction {
        match self {
            Self::CompoundFuture(_) => AnalyticConstruction::CompoundFuture,
            Self::Perfect(_) => AnalyticConstruction::Perfect,
            Self::Pluperfect(_) => AnalyticConstruction::Pluperfect,
            Self::FutureAnterior => AnalyticConstruction::FutureAnterior,
            Self::Conditional(_) => AnalyticConstruction::Conditional,
            Self::Optative(_) => AnalyticConstruction::Optative,
            Self::AnalyticPassive(_) => AnalyticConstruction::AnalyticPassive,
            Self::PeriphrasticTense(_) => AnalyticConstruction::PeriphrasticTense,
            Self::CopulaEllipsis(_) => AnalyticConstruction::CopulaEllipsis,
            Self::CompositeAdverbialParticiple(_) => {
                AnalyticConstruction::CompositeAdverbialParticiple
            }
        }
    }
}

/// Accentual behavior of a short personal/reflexive pronoun after its host
/// (Alypy §47).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PronounCliticProsody {
    /// The host's final vowel receives an acute and the enclitic is unaccented.
    AfterFinalVowelStress,
    /// Logical emphasis retains the short pronoun's lexical accent.
    LogicallyStressed,
}

/// Interrogative base retained after a negative `ни-` prefix is separated by
/// a governing preposition (Alypy §48).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum NegativePronounBase {
    Who,
    What,
    Kii,
    Kotoryi,
}

impl NegativePronounBase {
    pub const ALL: [Self; 4] = [Self::Who, Self::What, Self::Kii, Self::Kotoryi];
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PhraseRole {
    Host,
    FusedPrepositionPronoun,
    Auxiliary,
    Infinitive,
    LParticiple,
    ActiveParticiple,
    PassiveParticiple,
    AuxiliaryParticiple,
    FiniteVerb,
    Particle,
    Preposition,
    Pronoun,
    Numeral,
    Conjunction,
    MultiplicativeUnit,
    FractionNoun,
    Complement,
    Agent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PhraseToken {
    pub role: PhraseRole,
    pub forms: FormSet,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct RealizedPhrase {
    construction: AnalyticConstruction,
    formation: Option<PhraseFormation>,
    agent_government: Option<PassiveAgentGovernment>,
    tokens: Vec<PhraseToken>,
    warnings: Vec<String>,
}

impl RealizedPhrase {
    pub fn new(construction: AnalyticConstruction, tokens: Vec<PhraseToken>) -> Result<Self> {
        if tokens.is_empty() {
            return Err(Error::EmptyFormSet);
        }
        if tokens
            .iter()
            .any(|token| token.forms.target_recension() != Recension::SynodalRussian)
        {
            return Err(Error::ContradictoryMetadata {
                reason: "every analytic phrase token must target Synodal Russian".into(),
            });
        }
        Ok(Self {
            construction,
            formation: None,
            agent_government: None,
            tokens,
            warnings: Vec::new(),
        })
    }

    /// Constructs one source-defined analytic subtype and rejects token-role
    /// shapes that do not belong to that subtype.
    pub fn new_typed(formation: PhraseFormation, tokens: Vec<PhraseToken>) -> Result<Self> {
        Self::new_typed_with_government(formation, tokens, None)
    }

    pub fn new_typed_with_government(
        formation: PhraseFormation,
        tokens: Vec<PhraseToken>,
        agent_government: Option<PassiveAgentGovernment>,
    ) -> Result<Self> {
        let construction = formation.construction();
        let phrase = Self::new(construction, tokens)?;
        let roles = phrase
            .tokens
            .iter()
            .map(|token| token.role)
            .collect::<Vec<_>>();
        if !valid_typed_roles(formation, &roles, agent_government) {
            return Err(Error::ContradictoryMetadata {
                reason: format!(
                    "the token roles {roles:?} do not realize the typed formation {formation:?}"
                ),
            });
        }
        Ok(Self {
            formation: Some(formation),
            agent_government,
            ..phrase
        })
    }

    #[must_use]
    pub const fn construction(&self) -> AnalyticConstruction {
        self.construction
    }

    #[must_use]
    pub const fn formation(&self) -> Option<PhraseFormation> {
        self.formation
    }

    #[must_use]
    pub const fn agent_government(&self) -> Option<PassiveAgentGovernment> {
        self.agent_government
    }

    #[must_use]
    pub fn tokens(&self) -> &[PhraseToken] {
        &self.tokens
    }

    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    #[must_use]
    pub fn primary_text(&self) -> String {
        self.tokens
            .iter()
            .map(|token| token.forms.primary_text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn valid_typed_roles(
    formation: PhraseFormation,
    roles: &[PhraseRole],
    government: Option<PassiveAgentGovernment>,
) -> bool {
    use PhraseRole::{
        ActiveParticiple, Agent, Auxiliary, AuxiliaryParticiple, Complement, Conjunction,
        FiniteVerb, Infinitive, LParticiple, Particle, PassiveParticiple, Preposition,
    };

    let core_roles = match government {
        None => roles,
        Some(PassiveAgentGovernment::Instrumental) if roles.last() == Some(&Agent) => {
            &roles[..roles.len() - 1]
        }
        Some(PassiveAgentGovernment::OtGenitive) if roles.ends_with(&[Preposition, Agent]) => {
            &roles[..roles.len() - 2]
        }
        Some(_) => return false,
    };
    if government.is_some() && !matches!(formation, PhraseFormation::AnalyticPassive(_)) {
        return false;
    }

    match formation {
        PhraseFormation::CompoundFuture(_) => unordered_pair(core_roles, Auxiliary, Infinitive),
        PhraseFormation::Perfect(PerfectFormation::PresentCopula) => {
            unordered_pair(core_roles, Auxiliary, LParticiple)
        }
        PhraseFormation::Perfect(PerfectFormation::OmittedThirdSingularCopula) => {
            core_roles == [LParticiple]
        }
        PhraseFormation::Perfect(PerfectFormation::SharedPresentCopula) => {
            core_roles == [LParticiple, LParticiple, Auxiliary]
        }
        PhraseFormation::Pluperfect(PluperfectFormation::PerfectCopula) => {
            compound_roles(core_roles, LParticiple, AuxiliaryParticiple, Auxiliary)
        }
        PhraseFormation::Pluperfect(_) => unordered_pair(core_roles, Auxiliary, LParticiple),
        PhraseFormation::FutureAnterior => {
            core_roles.first() == Some(&Conjunction)
                && unordered_pair(&core_roles[1..], Auxiliary, LParticiple)
        }
        PhraseFormation::Conditional(ConditionalFormation::PersonalAorist) => {
            unordered_pair(core_roles, Auxiliary, LParticiple)
        }
        PhraseFormation::Conditional(ConditionalFormation::InvariantBy) => {
            unordered_pair(core_roles, Particle, LParticiple)
        }
        PhraseFormation::Conditional(ConditionalFormation::PersonalAoristWithPresentCopula) => {
            core_roles == [Auxiliary, LParticiple, Auxiliary]
                || core_roles == [Auxiliary, Auxiliary, LParticiple]
        }
        PhraseFormation::Conditional(ConditionalFormation::InvariantByWithPresentCopula) => {
            core_roles == [Particle, LParticiple, Auxiliary]
                || core_roles == [Particle, Auxiliary, LParticiple]
        }
        PhraseFormation::Conditional(ConditionalFormation::InfinitiveWithInvariantBy) => {
            unordered_pair(core_roles, Particle, Infinitive)
        }
        PhraseFormation::Conditional(ConditionalFormation::ModalImperfect(auxiliary)) => {
            matches!(
                core_roles,
                [Auxiliary, Infinitive] | [Auxiliary, PassiveParticiple, Auxiliary]
            ) || auxiliary == ModalConditionalAuxiliary::Podobati
                && matches!(
                    core_roles,
                    [Auxiliary, Particle, Infinitive]
                        | [Auxiliary, Particle, PassiveParticiple, Auxiliary]
                )
        }
        PhraseFormation::Optative(_) => core_roles == [Particle, FiniteVerb],
        PhraseFormation::AnalyticPassive(formation) => {
            if matches!(
                formation,
                PassiveFormation::PluperfectPerfectCopula
                    | PassiveFormation::PresentParticipleConditional
                    | PassiveFormation::PastParticipleConditional
            ) {
                compound_roles(
                    core_roles,
                    PassiveParticiple,
                    AuxiliaryParticiple,
                    Auxiliary,
                )
            } else {
                unordered_pair(core_roles, PassiveParticiple, Auxiliary)
            }
        }
        PhraseFormation::PeriphrasticTense(PeriphrasticFormation::Copular(_)) => {
            unordered_pair(core_roles, ActiveParticiple, Auxiliary)
        }
        PhraseFormation::PeriphrasticTense(PeriphrasticFormation::SemiAuxiliary(
            semi_auxiliary,
        )) => {
            let expected_negation = matches!(
                semi_auxiliary,
                PeriphrasticSemiAuxiliary::NePrestavati | PeriphrasticSemiAuxiliary::NeOskudevati
            );
            if expected_negation {
                core_roles == [Particle, Auxiliary, ActiveParticiple]
                    || core_roles == [Particle, ActiveParticiple, Auxiliary]
            } else {
                unordered_pair(core_roles, Auxiliary, ActiveParticiple)
            }
        }
        PhraseFormation::CopulaEllipsis(context) => {
            if matches!(
                context,
                CopulaOmissionContext::SePresent
                    | CopulaOmissionContext::SePastAorist
                    | CopulaOmissionContext::SePastImperfect
            ) {
                core_roles == [Particle, Complement]
            } else {
                core_roles == [Complement]
            }
        }
        PhraseFormation::CompositeAdverbialParticiple(
            AdverbialParticipleFormation::PastPassiveWithPastActiveByti,
        ) => unordered_pair(core_roles, AuxiliaryParticiple, PassiveParticiple),
        PhraseFormation::CompositeAdverbialParticiple(_) => {
            unordered_pair(core_roles, ActiveParticiple, Complement)
        }
    }
}

fn unordered_pair(roles: &[PhraseRole], first: PhraseRole, second: PhraseRole) -> bool {
    roles == [first, second] || roles == [second, first]
}

fn compound_roles(
    roles: &[PhraseRole],
    predicate: PhraseRole,
    participle: PhraseRole,
    finite: PhraseRole,
) -> bool {
    roles == [predicate, participle, finite]
        || roles == [predicate, finite, participle]
        || roles == [participle, finite, predicate]
        || roles == [finite, participle, predicate]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Confidence, FormSource, FormVariant, RuleId, RuleTrace};

    fn forms(text: &str) -> FormSet {
        FormSet::new(FormVariant {
            expanded: text.into(),
            accented: None,
            printed: text.into(),
            romanization: None,
            source_recension: Some(Recension::SynodalRussian),
            target_recension: Recension::SynodalRussian,
            recension_mapping: None,
            confidence: Confidence::CERTAIN,
            source: FormSource::SynodalNormativeGeneration {
                rule: RuleId::from("test-analytic-shape"),
            },
            assumptions: vec![],
            evidence: vec![],
            contradictions: vec![],
            warnings: vec![],
            rule_trace: RuleTrace::default(),
        })
        .expect("valid test form")
    }

    fn tokens(roles: &[PhraseRole]) -> Vec<PhraseToken> {
        roles
            .iter()
            .enumerate()
            .map(|(index, &role)| PhraseToken {
                role,
                forms: forms(&format!("token-{index}")),
            })
            .collect()
    }

    fn assert_shape(formation: PhraseFormation, roles: &[PhraseRole]) {
        let phrase = RealizedPhrase::new_typed(formation, tokens(roles))
            .unwrap_or_else(|error| panic!("{formation:?} rejected {roles:?}: {error}"));
        assert_eq!(phrase.formation(), Some(formation));
        assert_eq!(phrase.construction(), formation.construction());
    }

    #[test]
    fn every_closed_analytic_formation_accepts_its_role_shape() {
        use PhraseRole::{
            ActiveParticiple, Auxiliary, AuxiliaryParticiple, Complement, Conjunction, FiniteVerb,
            Infinitive, LParticiple, Particle, PassiveParticiple,
        };

        for auxiliary in CompoundFutureAuxiliary::ALL {
            assert_shape(
                PhraseFormation::CompoundFuture(auxiliary),
                &[Auxiliary, Infinitive],
            );
        }
        assert_shape(
            PhraseFormation::Perfect(PerfectFormation::PresentCopula),
            &[LParticiple, Auxiliary],
        );
        assert_shape(
            PhraseFormation::Perfect(PerfectFormation::OmittedThirdSingularCopula),
            &[LParticiple],
        );
        assert_shape(
            PhraseFormation::Perfect(PerfectFormation::SharedPresentCopula),
            &[LParticiple, LParticiple, Auxiliary],
        );
        for formation in PluperfectFormation::ALL {
            let roles = if formation == PluperfectFormation::PerfectCopula {
                vec![LParticiple, AuxiliaryParticiple, Auxiliary]
            } else {
                vec![LParticiple, Auxiliary]
            };
            assert_shape(PhraseFormation::Pluperfect(formation), &roles);
        }
        assert_shape(
            PhraseFormation::FutureAnterior,
            &[Conjunction, LParticiple, Auxiliary],
        );

        let conditional_shapes = [
            (
                ConditionalFormation::PersonalAorist,
                vec![LParticiple, Auxiliary],
            ),
            (
                ConditionalFormation::PersonalAoristWithPresentCopula,
                vec![Auxiliary, LParticiple, Auxiliary],
            ),
            (
                ConditionalFormation::InvariantBy,
                vec![LParticiple, Particle],
            ),
            (
                ConditionalFormation::InvariantByWithPresentCopula,
                vec![Particle, LParticiple, Auxiliary],
            ),
            (
                ConditionalFormation::InfinitiveWithInvariantBy,
                vec![Infinitive, Particle],
            ),
            (
                ConditionalFormation::ModalImperfect(ModalConditionalAuxiliary::Moshchi),
                vec![Auxiliary, PassiveParticiple, Auxiliary],
            ),
        ];
        for (formation, roles) in conditional_shapes {
            assert_shape(PhraseFormation::Conditional(formation), &roles);
        }
        for system in [
            OptativeFiniteSystem::Present,
            OptativeFiniteSystem::SimpleFuture,
        ] {
            assert_shape(PhraseFormation::Optative(system), &[Particle, FiniteVerb]);
        }
        for formation in PassiveFormation::ALL {
            let roles = if matches!(
                formation,
                PassiveFormation::PluperfectPerfectCopula
                    | PassiveFormation::PresentParticipleConditional
                    | PassiveFormation::PastParticipleConditional
            ) {
                vec![PassiveParticiple, AuxiliaryParticiple, Auxiliary]
            } else {
                vec![PassiveParticiple, Auxiliary]
            };
            assert_shape(PhraseFormation::AnalyticPassive(formation), &roles);
        }
        for formation in PeriphrasticTenseFormation::ALL {
            assert_shape(
                PhraseFormation::PeriphrasticTense(PeriphrasticFormation::Copular(formation)),
                &[ActiveParticiple, Auxiliary],
            );
        }
        for auxiliary in PeriphrasticSemiAuxiliary::ALL {
            let roles = if matches!(
                auxiliary,
                PeriphrasticSemiAuxiliary::NePrestavati | PeriphrasticSemiAuxiliary::NeOskudevati
            ) {
                vec![Particle, Auxiliary, ActiveParticiple]
            } else {
                vec![Auxiliary, ActiveParticiple]
            };
            assert_shape(
                PhraseFormation::PeriphrasticTense(PeriphrasticFormation::SemiAuxiliary(auxiliary)),
                &roles,
            );
        }
        for context in [
            CopulaOmissionContext::PresentNominalPredicate,
            CopulaOmissionContext::Imperative,
            CopulaOmissionContext::NarrativePast,
            CopulaOmissionContext::ImpersonalPredicate,
        ] {
            assert_shape(PhraseFormation::CopulaEllipsis(context), &[Complement]);
        }
        for context in [
            CopulaOmissionContext::SePresent,
            CopulaOmissionContext::SePastAorist,
            CopulaOmissionContext::SePastImperfect,
        ] {
            assert_shape(
                PhraseFormation::CopulaEllipsis(context),
                &[Particle, Complement],
            );
        }
        assert_shape(
            PhraseFormation::CompositeAdverbialParticiple(
                AdverbialParticipleFormation::PastPassiveWithPastActiveByti,
            ),
            &[PassiveParticiple, AuxiliaryParticiple],
        );
        for formation in [
            AdverbialParticipleFormation::PresentCopularNominal,
            AdverbialParticipleFormation::PastCopularNominal,
        ] {
            assert_shape(
                PhraseFormation::CompositeAdverbialParticiple(formation),
                &[Complement, ActiveParticiple],
            );
        }
    }

    #[test]
    fn typed_shapes_reject_malformed_roles_and_government() {
        use PhraseRole::{
            ActiveParticiple, Agent, Auxiliary, AuxiliaryParticiple, Infinitive, LParticiple,
            Particle, PassiveParticiple, Preposition,
        };

        assert!(matches!(
            RealizedPhrase::new_typed(
                PhraseFormation::CompoundFuture(CompoundFutureAuxiliary::Imati),
                tokens(&[Auxiliary, Auxiliary]),
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
        assert!(matches!(
            RealizedPhrase::new_typed_with_government(
                PhraseFormation::CompoundFuture(CompoundFutureAuxiliary::Imati),
                tokens(&[Auxiliary, Infinitive, Agent]),
                Some(PassiveAgentGovernment::Instrumental),
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
        assert!(matches!(
            RealizedPhrase::new_typed(
                PhraseFormation::Pluperfect(PluperfectFormation::PerfectCopula),
                tokens(&[AuxiliaryParticiple, LParticiple, Auxiliary]),
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
        assert!(matches!(
            RealizedPhrase::new_typed(
                PhraseFormation::Conditional(ConditionalFormation::InvariantByWithPresentCopula,),
                tokens(&[LParticiple, Particle, Auxiliary]),
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
        assert!(matches!(
            RealizedPhrase::new_typed(
                PhraseFormation::PeriphrasticTense(PeriphrasticFormation::SemiAuxiliary(
                    PeriphrasticSemiAuxiliary::NePrestavati,
                )),
                tokens(&[Auxiliary, Particle, ActiveParticiple]),
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let instrumental = RealizedPhrase::new_typed_with_government(
            PhraseFormation::AnalyticPassive(PassiveFormation::Perfect),
            tokens(&[PassiveParticiple, Auxiliary, Agent]),
            Some(PassiveAgentGovernment::Instrumental),
        )
        .expect("instrumental agent shape");
        assert_eq!(
            instrumental.agent_government(),
            Some(PassiveAgentGovernment::Instrumental)
        );
        RealizedPhrase::new_typed_with_government(
            PhraseFormation::AnalyticPassive(PassiveFormation::Perfect),
            tokens(&[PassiveParticiple, Auxiliary, Preposition, Agent]),
            Some(PassiveAgentGovernment::OtGenitive),
        )
        .expect("ot-genitive agent shape");
        assert!(matches!(
            RealizedPhrase::new_typed_with_government(
                PhraseFormation::AnalyticPassive(PassiveFormation::Perfect),
                tokens(&[PassiveParticiple, Auxiliary, Agent]),
                Some(PassiveAgentGovernment::OtGenitive),
            ),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}
