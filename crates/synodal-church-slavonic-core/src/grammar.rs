macro_rules! closed_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const ALL: [Self; closed_enum!(@count $($variant),+)] = [$(Self::$variant),+];
        }
    };
    (@count $($variant:ident),+) => {
        <[()]>::len(&[$(closed_enum!(@unit $variant)),+])
    };
    (@unit $variant:ident) => { () };
}

closed_enum!(Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Locative,
    Vocative,
});
closed_enum!(Number {
    Singular,
    Dual,
    Plural,
});
closed_enum!(Gender {
    Masculine,
    Feminine,
    Neuter,
});
closed_enum!(Animacy { Inanimate, Animate });
closed_enum!(Person {
    First,
    Second,
    Third,
});
closed_enum!(AdjectiveForm { Short, Long });
closed_enum!(Comparison {
    Positive,
    Comparative,
    Superlative,
});
closed_enum!(Voice {
    Active,
    Middle,
    Passive,
});
// `Past` represents a source-typed finite past whose evidence does not
// distinguish aorist from imperfect. It is exact-only and is never
// productively generated.
closed_enum!(FiniteTense {
    Present,
    Future,
    Past,
    Imperfect,
    Aorist,
});
closed_enum!(ParticipleTense { Present, Past });
closed_enum!(ParticipleVoice { Active, Passive });
closed_enum!(NumeralKind {
    Cardinal,
    Ordinal,
    Collective,
});

/// A complete verb-system inventory understood by the public paradigm API.
/// Exact-only systems remain represented here so their unsupported or missing
/// cells stay visible instead of disappearing from a partial table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum VerbSystem {
    Finite(FiniteTense),
    Imperative,
    Infinitive,
    LParticiple,
    Participle {
        tense: ParticipleTense,
        voice: ParticipleVoice,
        form: AdjectiveForm,
    },
    Supine,
    VerbalNoun {
        animacy: Animacy,
    },
}

impl VerbSystem {
    pub const ALL: [Self; 19] = [
        Self::Finite(FiniteTense::Present),
        Self::Finite(FiniteTense::Future),
        Self::Finite(FiniteTense::Past),
        Self::Finite(FiniteTense::Imperfect),
        Self::Finite(FiniteTense::Aorist),
        Self::Imperative,
        Self::Infinitive,
        Self::LParticiple,
        Self::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Short,
        },
        Self::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Long,
        },
        Self::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Short,
        },
        Self::Participle {
            tense: ParticipleTense::Present,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Long,
        },
        Self::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Short,
        },
        Self::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Active,
            form: AdjectiveForm::Long,
        },
        Self::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Short,
        },
        Self::Participle {
            tense: ParticipleTense::Past,
            voice: ParticipleVoice::Passive,
            form: AdjectiveForm::Long,
        },
        Self::Supine,
        Self::VerbalNoun {
            animacy: Animacy::Inanimate,
        },
        Self::VerbalNoun {
            animacy: Animacy::Animate,
        },
    ];
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NounCell {
    pub case: Case,
    pub number: Number,
    pub animacy: Animacy,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AdjectiveCell {
    pub case: Case,
    pub number: Number,
    pub gender: Gender,
    pub animacy: Animacy,
    pub form: AdjectiveForm,
    pub comparison: Comparison,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FiniteVerbCell {
    pub tense: FiniteTense,
    pub person: Person,
    pub number: Number,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ImperativeCell {
    pub person: Person,
    pub number: Number,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LParticipleCell {
    pub gender: Gender,
    pub number: Number,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ParticipleCell {
    pub tense: ParticipleTense,
    pub voice: ParticipleVoice,
    pub agreement: AdjectiveCell,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PronounCell {
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
    pub person: Option<Person>,
    pub animacy: Animacy,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NumeralCell {
    pub kind: NumeralKind,
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
    pub animacy: Animacy,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum GrammarCell {
    /// A reviewed dictionary headword attested in target-recension text when
    /// its inflectional cell has not yet been independently established.
    /// This is lexical evidence only and never enables productive inflection.
    LexicalForm,
    /// Exact lexical form for adverbs, prepositions, conjunctions, particles,
    /// and interjections. This cell never enables productive inflection.
    Indeclinable,
    Noun(NounCell),
    Adjective(AdjectiveCell),
    FiniteVerb(FiniteVerbCell),
    Imperative(ImperativeCell),
    Infinitive,
    Supine,
    LParticiple(LParticipleCell),
    Participle(ParticipleCell),
    VerbalNoun(NounCell),
    Pronoun(PronounCell),
    Determiner(AdjectiveCell),
    Numeral(NumeralCell),
}
