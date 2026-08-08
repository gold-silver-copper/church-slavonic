//! Stable rule identifiers and optional rule traces.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleId {
    NounOMasculineHard,
    NounONeuterHard,
    NounJoMasculineSoft,
    NounJoNeuterSoft,
    NounAHard,
    NounJaSoft,
    NounIFeminine,
    NounIMasculine,
    NounUMasculine,
    NounNMasculine,
    NounNNeuter,
    NounNtNeuter,
    NounRStem,
    NounSNeuter,
    NounVFeminine,
    NounIndeclinable,
    AdjectiveHardShort,
    AdjectiveHardLong,
    AdjectiveSoftShort,
    AdjectiveSoftLong,
    VerbIA1,
    VerbIA2,
    VerbII1,
    VerbII2,
    VerbII3,
    VerbInfinitive,
    VerbSupine,
    VerbLParticiple,
}

impl RuleId {
    pub const fn code(self) -> &'static str {
        match self {
            Self::NounOMasculineHard => "N-O-M-HARD-01",
            Self::NounONeuterHard => "N-O-N-HARD-01",
            Self::NounJoMasculineSoft => "N-JO-M-SOFT-01",
            Self::NounJoNeuterSoft => "N-JO-N-SOFT-01",
            Self::NounAHard => "N-A-HARD-01",
            Self::NounJaSoft => "N-JA-SOFT-01",
            Self::NounIFeminine => "N-I-F-01",
            Self::NounIMasculine => "N-I-M-01",
            Self::NounUMasculine => "N-U-M-01",
            Self::NounNMasculine => "N-N-M-01",
            Self::NounNNeuter => "N-N-N-01",
            Self::NounNtNeuter => "N-NT-N-01",
            Self::NounRStem => "N-R-01",
            Self::NounSNeuter => "N-S-N-01",
            Self::NounVFeminine => "N-V-F-01",
            Self::NounIndeclinable => "N-INDECL-01",
            Self::AdjectiveHardShort => "ADJ-HARD-SHORT-01",
            Self::AdjectiveHardLong => "ADJ-HARD-LONG-01",
            Self::AdjectiveSoftShort => "ADJ-SOFT-SHORT-01",
            Self::AdjectiveSoftLong => "ADJ-SOFT-LONG-01",
            Self::VerbIA1 => "V-IA1-01",
            Self::VerbIA2 => "V-IA2-01",
            Self::VerbII1 => "V-II1-01",
            Self::VerbII2 => "V-II2-01",
            Self::VerbII3 => "V-II3-01",
            Self::VerbInfinitive => "V-INF-01",
            Self::VerbSupine => "V-SUP-01",
            Self::VerbLParticiple => "V-LPART-01",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleStep {
    pub rule_id: RuleId,
    pub before: String,
    pub after: String,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredictedForm {
    pub text: String,
    pub rule_id: RuleId,
    pub trace: Vec<RuleStep>,
}
