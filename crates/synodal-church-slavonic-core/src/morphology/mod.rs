//! Productive Synodal rules admitted from Alypy (Gamanovich) §§33–44, 53,
//! 57, 79–80, 86–87, 93, and 97.

mod adjective;
mod noun;
mod participle;
mod shared;
mod verb;

#[cfg(test)]
mod adjective_tests;
#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod noun_extended_tests;
#[cfg(test)]
mod noun_kernel_pin_tests;
#[cfg(test)]
mod noun_tests;
#[cfg(test)]
mod participle_tests;
#[cfg(test)]
mod reflexive_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod verb_kernel_pin_tests;
#[cfg(test)]
mod verb_tests;

#[allow(unused_imports)]
pub(crate) use adjective::*;
#[allow(unused_imports)]
pub(crate) use noun::*;
#[allow(unused_imports)]
pub(crate) use participle::*;
#[allow(unused_imports)]
pub(crate) use shared::*;
#[allow(unused_imports)]
pub(crate) use verb::*;

pub use adjective::{
    AdjectiveClass, AdjectiveLexeme, ComparisonFormation, ShortMasculineStemFormation,
    decline_adjective, validate_adjective_lexeme,
};
pub use noun::{
    NounAnimacyInventory, NounDeclension, NounLexeme, NounNumberInventory, decline_noun,
    validate_noun_lexeme,
};
pub use participle::{ActiveParticipleShortFormation, ParticiplePrincipalPart, decline_participle};
pub use shared::policy_allows_normative_rule;
pub use verb::{
    AoristFormation, Aspect, ImperativeFormation, ImperfectFormation, PresentPrincipalParts,
    REFLEXIVE_RULE_ID, VerbConjugation, VerbLexeme, VerbalNounFormation, VerbalNounPrincipalPart,
    aorist, decline_verbal_noun, future, imperative, imperfect, infinitive, l_participle, present,
    reflexive_base_candidates, reflexive_surface,
};
