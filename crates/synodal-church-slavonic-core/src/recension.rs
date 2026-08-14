/// A source or target language variety.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[non_exhaustive]
pub enum Recension {
    OldChurchSlavonic,
    SynodalRussian,
    OtherChurchSlavonic,
    OldRussian,
    ModernRussian,
    Mixed,
    Unknown,
}

impl Recension {
    pub const ALL: [Self; 7] = [
        Self::OldChurchSlavonic,
        Self::SynodalRussian,
        Self::OtherChurchSlavonic,
        Self::OldRussian,
        Self::ModernRussian,
        Self::Mixed,
        Self::Unknown,
    ];

    #[must_use]
    pub const fn is_synodal_target(self) -> bool {
        matches!(self, Self::SynodalRussian)
    }

    #[must_use]
    pub const fn is_forbidden_authority(self) -> bool {
        matches!(self, Self::ModernRussian)
    }
}
