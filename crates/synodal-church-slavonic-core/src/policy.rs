#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum GenerationPolicy {
    #[default]
    Strict,
    Productive,
    Exploratory,
}

impl GenerationPolicy {
    pub const ALL: [Self; 3] = [Self::Strict, Self::Productive, Self::Exploratory];
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum VariantPolicy {
    #[default]
    NormativeFirst,
    AttestedFirst,
    First,
    Unique,
}

impl VariantPolicy {
    pub const ALL: [Self; 4] = [
        Self::NormativeFirst,
        Self::AttestedFirst,
        Self::First,
        Self::Unique,
    ];
}
