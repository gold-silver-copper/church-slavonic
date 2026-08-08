//! Grammatical dimensions and lexical metadata used by the inflector.

use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartOfSpeech {
    Noun,
    Adjective,
    Verb,
    Pronoun,
    Numeral,
    Determiner,
}

impl PartOfSpeech {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Noun => "noun",
            Self::Adjective => "adj",
            Self::Verb => "verb",
            Self::Pronoun => "pron",
            Self::Numeral => "num",
            Self::Determiner => "det",
        }
    }
}

impl fmt::Display for PartOfSpeech {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Locative,
    Vocative,
}

impl Case {
    pub const ALL: [Self; 7] = [
        Self::Nominative,
        Self::Genitive,
        Self::Dative,
        Self::Accusative,
        Self::Instrumental,
        Self::Locative,
        Self::Vocative,
    ];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Nominative => "nom",
            Self::Genitive => "gen",
            Self::Dative => "dat",
            Self::Accusative => "acc",
            Self::Instrumental => "ins",
            Self::Locative => "loc",
            Self::Vocative => "voc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Number {
    Singular,
    Dual,
    Plural,
}

impl Number {
    pub const ALL: [Self; 3] = [Self::Singular, Self::Dual, Self::Plural];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Singular => "sg",
            Self::Dual => "du",
            Self::Plural => "pl",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
}

impl Gender {
    pub const ALL: [Self; 3] = [Self::Masculine, Self::Feminine, Self::Neuter];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Masculine => "m",
            Self::Feminine => "f",
            Self::Neuter => "n",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Animacy {
    Animate,
    Inanimate,
}

impl Animacy {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Animate => "an",
            Self::Inanimate => "in",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Person {
    First,
    Second,
    Third,
}

impl Person {
    pub const ALL: [Self; 3] = [Self::First, Self::Second, Self::Third];

    pub const fn code(self) -> &'static str {
        match self {
            Self::First => "1",
            Self::Second => "2",
            Self::Third => "3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdjectiveForm {
    Short,
    Long,
}

impl AdjectiveForm {
    pub const ALL: [Self; 2] = [Self::Short, Self::Long];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Short => "short",
            Self::Long => "long",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FiniteTense {
    Present,
    Imperfect,
    Aorist,
}

impl FiniteTense {
    pub const ALL: [Self; 3] = [Self::Present, Self::Imperfect, Self::Aorist];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Imperfect => "imperfect",
            Self::Aorist => "aorist",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ParticipleKind {
    PresentActive,
    PresentPassive,
    PastActive,
    PastPassive,
}

impl ParticipleKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::PresentActive => "present-active",
            Self::PresentPassive => "present-passive",
            Self::PastActive => "past-active",
            Self::PastPassive => "past-passive",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumberRestriction {
    All,
    SingularOnly,
    DualOnly,
    PluralOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NounClass {
    OMasculineHard,
    ONeuterHard,
    JoMasculineSoft,
    JoNeuterSoft,
    AHard,
    JaSoft,
    IFeminine,
    IMasculine,
    UMasculine,
    NMasculine,
    NNeuter,
    NtNeuter,
    RStem,
    SNeuter,
    VFeminine,
    Indeclinable,
}

impl NounClass {
    pub const fn code(self) -> &'static str {
        match self {
            Self::OMasculineHard => "o-m-hard",
            Self::ONeuterHard => "o-n-hard",
            Self::JoMasculineSoft => "jo-m-soft",
            Self::JoNeuterSoft => "jo-n-soft",
            Self::AHard => "a-hard",
            Self::JaSoft => "ja-soft",
            Self::IFeminine => "i-f",
            Self::IMasculine => "i-m",
            Self::UMasculine => "u-m",
            Self::NMasculine => "n-m",
            Self::NNeuter => "n-n",
            Self::NtNeuter => "nt-n",
            Self::RStem => "r-n",
            Self::SNeuter => "s-n",
            Self::VFeminine => "v-f",
            Self::Indeclinable => "indeclinable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AdjectiveClass {
    Hard,
    Soft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerbClass {
    IA1,
    IA2,
    II1,
    II2,
    II3,
    Root,
    Irregular,
}

impl VerbClass {
    pub const fn code(self) -> &'static str {
        match self {
            Self::IA1 => "IA1",
            Self::IA2 => "IA2",
            Self::II1 => "II1",
            Self::II2 => "II2",
            Self::II3 => "II3",
            Self::Root => "root",
            Self::Irregular => "irregular",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NounCell {
    pub case: Case,
    pub number: Number,
}

impl NounCell {
    pub fn key(self) -> String {
        format!("noun:{}:{}", self.case.code(), self.number.code())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdjectiveCell {
    pub case: Case,
    pub number: Number,
    pub gender: Gender,
    pub animacy: Animacy,
    pub form: AdjectiveForm,
}

impl AdjectiveCell {
    pub fn key(self) -> String {
        format!(
            "adj:{}:{}:{}:{}:{}",
            self.form.code(),
            self.case.code(),
            self.number.code(),
            self.gender.code(),
            self.animacy.code()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FiniteVerbCell {
    pub tense: FiniteTense,
    pub person: Person,
    pub number: Number,
}

impl FiniteVerbCell {
    pub fn key(self) -> String {
        format!(
            "verb:finite:{}:{}:{}",
            self.tense.code(),
            self.person.code(),
            self.number.code()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImperativeCell {
    pub person: Person,
    pub number: Number,
}

impl ImperativeCell {
    pub fn key(self) -> String {
        format!(
            "verb:imperative:{}:{}",
            self.person.code(),
            self.number.code()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LParticipleCell {
    pub gender: Gender,
    pub number: Number,
}

impl LParticipleCell {
    pub fn key(self) -> String {
        format!(
            "verb:l-participle:{}:{}",
            self.gender.code(),
            self.number.code()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParticipleCell {
    pub kind: ParticipleKind,
    pub adjective: AdjectiveCell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClosedClassCell {
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
    pub person: Option<Person>,
}

impl ClosedClassCell {
    pub fn key(self, part_of_speech: PartOfSpeech) -> String {
        let mut key = format!(
            "decl:{}:{}:{}",
            part_of_speech.code(),
            self.case.code(),
            self.number.code()
        );
        if let Some(gender) = self.gender {
            key.push(':');
            key.push_str(gender.code());
        }
        if let Some(person) = self.person {
            key.push(':');
            key.push_str(person.code());
        }
        key
    }
}

impl ParticipleCell {
    pub fn key(self) -> String {
        format!(
            "verb:participle:{}:{}",
            self.kind.code(),
            self.adjective.key()
        )
    }
}
