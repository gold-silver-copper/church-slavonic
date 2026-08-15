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
    pub const ALL: [Self; 2] = [Self::Animate, Self::Inanimate];

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

/// The two source-described Old Church Slavonic comparative strategies.
///
/// This is independent of [`AdjectiveForm`]: both an old and a new comparative
/// have short/simple and long/compound declensional forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComparativeFormation {
    /// Productive `-ѣj-` (surface `-ѣи-`, or `-аи-` after a palatalized velar).
    New,
    /// Lexically restricted `-ьj-` with a substitutively softened base.
    Old,
}

impl ComparativeFormation {
    pub const fn code(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Old => "old",
        }
    }
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
    pub const ALL: [Self; 4] = [
        Self::PresentActive,
        Self::PresentPassive,
        Self::PastActive,
        Self::PastPassive,
    ];

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerbAspect {
    Perfective,
    Imperfective,
    Biaspectual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImperfectFormation {
    /// The infinitive-aorist stem takes the `-ах-` series.
    A,
    /// The infinitive-aorist stem takes the `-ѣах-` series.
    YatA,
    /// A final velar is first-palatalized before the `-аах-` series.
    PalatalizedA,
    /// A present-system stem takes the short `-ах-` terminal series.
    PresentA,
    /// A truncated present-system stem takes the `-ѣах-` terminal series.
    PresentYatA,
}

/// Lexically/source-audited surface-variant policy for the imperfect system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImperfectVariantPolicy {
    /// Emit only the independently specified uncontracted series.
    UncontractedOnly,
    /// Emit only the source-licensed contracted terminal series.
    ContractedOnly,
    /// Emit only the rare source-licensed iotated `-(ѣ)ꙗх-` series.
    IotatedOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AoristFormation {
    Asigmatic,
    /// Old sigmatic 1 (`нѣсъ`) formation with an `-с-` main subbundle.
    SigmaticPrimary,
    /// Old sigmatic 2 (`рѣхъ`) formation with an `-х-` main subbundle.
    SigmaticSecondary,
    /// Standard sigmatic aorist of a vowel-final workstem (`знахъ`).
    SigmaticVowel,
    New,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImperativeFormation {
    /// `-и-` throughout the non-singular imperative, after a palatal or `j`-stem.
    ISeries,
    /// Singular `-и`, but non-singular `-ѣ-`, after a non-palatal consonant.
    YatSeries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PresentActiveParticipleFormation {
    YushtHard,
    YushtSoft,
    YeshtSoft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PresentPassiveParticipleFormation {
    Im,
    Em,
    Om,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PastActiveParticipleFormation {
    Ush,
    /// The primary transformed i-stem series with `-ьш-` obliques.
    Ish,
    /// The underlying final `j` is declared by the formation and absent from
    /// the supplied Cyrillic orthographic base before `-въш-` is attached.
    VushAfterJDeletion,
    /// Transform final `-ов` to `-оу` before attaching `-въш-`.
    VushAfterOvToU,
    /// Attach `-въш-` to a base on which no additional seam is required.
    Vush,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PastPassiveParticipleFormation {
    T,
    N,
    En,
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
    /// Canonical noun inventory, ordered by number and then case.
    pub fn all() -> impl Iterator<Item = Self> {
        Number::ALL
            .into_iter()
            .flat_map(|number| Case::ALL.into_iter().map(move |case| Self { case, number }))
    }

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
    /// Canonical agreement inventory, ordered by form, number, case, gender,
    /// and animacy.
    pub fn all() -> impl Iterator<Item = Self> {
        AdjectiveForm::ALL.into_iter().flat_map(|form| {
            Number::ALL.into_iter().flat_map(move |number| {
                Case::ALL.into_iter().flat_map(move |case| {
                    Gender::ALL.into_iter().flat_map(move |gender| {
                        Animacy::ALL.into_iter().map(move |animacy| Self {
                            case,
                            number,
                            gender,
                            animacy,
                            form,
                        })
                    })
                })
            })
        })
    }

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

/// A case-number-gender-animacy cell for an agreeing determiner.
///
/// Animacy is explicit because adjectival determiners can distinguish the
/// masculine accusative. Regular pronominal determiners accept the dimension
/// but are syncretic across it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeterminerCell {
    pub case: Case,
    pub number: Number,
    pub gender: Gender,
    pub animacy: Animacy,
}

impl DeterminerCell {
    /// Canonical determiner inventory, ordered by number, case, gender, and
    /// animacy.
    pub fn all() -> impl Iterator<Item = Self> {
        Number::ALL.into_iter().flat_map(|number| {
            Case::ALL.into_iter().flat_map(move |case| {
                Gender::ALL.into_iter().flat_map(move |gender| {
                    Animacy::ALL.into_iter().map(move |animacy| Self {
                        case,
                        number,
                        gender,
                        animacy,
                    })
                })
            })
        })
    }

    pub fn key(self) -> String {
        format!(
            "det:{}:{}:{}:{}",
            self.case.code(),
            self.number.code(),
            self.gender.code(),
            self.animacy.code()
        )
    }

    pub const fn closed_class(self) -> ClosedClassCell {
        ClosedClassCell {
            case: self.case,
            number: self.number,
            gender: Some(self.gender),
            person: None,
        }
    }
}

/// A case-number cell for a cardinal numeral, with gender only when the
/// numeral agrees with the counted noun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NumeralCell {
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
}

impl NumeralCell {
    /// Canonical numeral inventory, ordered by number, case, and then the
    /// ungendered cell followed by masculine, feminine, and neuter cells.
    pub fn all() -> impl Iterator<Item = Self> {
        Number::ALL.into_iter().flat_map(|number| {
            Case::ALL.into_iter().flat_map(move |case| {
                core::iter::once(Self {
                    case,
                    number,
                    gender: None,
                })
                .chain(Gender::ALL.into_iter().map(move |gender| Self {
                    case,
                    number,
                    gender: Some(gender),
                }))
            })
        })
    }

    pub fn key(self) -> String {
        let mut key = format!("num:{}:{}", self.case.code(), self.number.code());
        if let Some(gender) = self.gender {
            key.push(':');
            key.push_str(gender.code());
        }
        key
    }

    pub const fn closed_class(self) -> ClosedClassCell {
        ClosedClassCell {
            case: self.case,
            number: self.number,
            gender: self.gender,
            person: None,
        }
    }
}

/// A case cell for a composed cardinal, with gender present only when its
/// final governing unit agrees with the enumerated noun.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompoundCardinalCell {
    pub case: Case,
    pub gender: Option<Gender>,
}

impl CompoundCardinalCell {
    /// All case and optional-gender requests in stable order.
    pub fn all() -> impl Iterator<Item = Self> {
        Case::ALL.into_iter().flat_map(|case| {
            core::iter::once(Self { case, gender: None }).chain(Gender::ALL.into_iter().map(
                move |gender| Self {
                    case,
                    gender: Some(gender),
                },
            ))
        })
    }

    pub fn key(self) -> String {
        let mut key = format!("num:compound:{}", self.case.code());
        if let Some(gender) = self.gender {
            key.push(':');
            key.push_str(gender.code());
        }
        key
    }
}

/// The only lexical agreement dimension of an OCS distributive-cardinal
/// construction. Its case is not caller-selectable: distributive `по` selects
/// the dative, while `по` with a locative cardinal has a different, commonly
/// temporal meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DistributiveCardinalCell {
    pub gender: Option<Gender>,
}

impl DistributiveCardinalCell {
    /// Every optional-gender request in stable order.
    pub fn all() -> impl Iterator<Item = Self> {
        core::iter::once(Self { gender: None }).chain(Gender::ALL.into_iter().map(|gender| Self {
            gender: Some(gender),
        }))
    }

    pub fn key(self) -> String {
        let mut key = "num:distributive:dat".to_string();
        if let Some(gender) = self.gender {
            key.push(':');
            key.push_str(gender.code());
        }
        key
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FiniteVerbCell {
    pub tense: FiniteTense,
    pub person: Person,
    pub number: Number,
}

impl FiniteVerbCell {
    /// Canonical finite inventory, ordered by tense, number, and person.
    pub fn all() -> impl Iterator<Item = Self> {
        FiniteTense::ALL.into_iter().flat_map(Self::for_tense)
    }

    /// Nine person-number cells for one finite tense.
    pub fn for_tense(tense: FiniteTense) -> impl Iterator<Item = Self> {
        Number::ALL.into_iter().flat_map(move |number| {
            Person::ALL.into_iter().map(move |person| Self {
                tense,
                person,
                number,
            })
        })
    }

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
    pub const SUPPORTED: [Self; 6] = [
        Self {
            person: Person::Second,
            number: Number::Singular,
        },
        Self {
            person: Person::Third,
            number: Number::Singular,
        },
        Self {
            person: Person::First,
            number: Number::Dual,
        },
        Self {
            person: Person::Second,
            number: Number::Dual,
        },
        Self {
            person: Person::First,
            number: Number::Plural,
        },
        Self {
            person: Person::Second,
            number: Number::Plural,
        },
    ];

    pub const fn is_supported(self) -> bool {
        matches!(
            (self.person, self.number),
            (Person::Second | Person::Third, Number::Singular)
                | (
                    Person::First | Person::Second,
                    Number::Dual | Number::Plural
                )
        )
    }

    pub fn key(self) -> String {
        format!(
            "verb:imperative:{}:{}",
            self.person.code(),
            self.number.code()
        )
    }
}

/// A complete Old Church Slavonic verb subsystem that can be independently
/// supplied, generated, or declared defective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerbMorphologySystem {
    Finite(FiniteTense),
    Imperative,
    Infinitive,
    Supine,
    LParticiple,
    Participle(ParticipleKind),
}

/// A typed cell in the rule-only verb engine.
///
/// Irregular lexemes use this inventory for source-reviewed exact forms and
/// explicit lexical defects.  Keeping the keys grammatical prevents arbitrary
/// feature strings from bypassing the public cell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerbMorphologyCell {
    Finite(FiniteVerbCell),
    Imperative(ImperativeCell),
    Infinitive,
    Supine,
    LParticiple(LParticipleCell),
    Participle(ParticipleCell),
}

impl VerbMorphologyCell {
    pub const fn system(self) -> VerbMorphologySystem {
        match self {
            Self::Finite(cell) => VerbMorphologySystem::Finite(cell.tense),
            Self::Imperative(_) => VerbMorphologySystem::Imperative,
            Self::Infinitive => VerbMorphologySystem::Infinitive,
            Self::Supine => VerbMorphologySystem::Supine,
            Self::LParticiple(_) => VerbMorphologySystem::LParticiple,
            Self::Participle(cell) => VerbMorphologySystem::Participle(cell.kind),
        }
    }

    pub fn requested(self) -> RequestedCell {
        match self {
            Self::Finite(cell) => RequestedCell::FiniteVerb(cell),
            Self::Imperative(cell) => RequestedCell::Imperative(cell),
            Self::Infinitive => RequestedCell::Infinitive,
            Self::Supine => RequestedCell::Supine,
            Self::LParticiple(cell) => RequestedCell::LParticiple(cell),
            Self::Participle(cell) => RequestedCell::Participle(cell),
        }
    }
}

/// Why a source-reviewed irregular verb cell cannot be generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerbDefectKind {
    /// The form is excluded by the lexeme's grammar (for example, a genuinely
    /// defective imperative or a passive participle of an incompatible verb).
    HistoricallyInvalid,
    /// The corpus does not license a reconstruction and no productive analysis
    /// has been established. This is unsupported knowledge, not a claim that
    /// the grammatical cell itself is impossible.
    UnattestedUnreconstructable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LParticipleCell {
    pub gender: Gender,
    pub number: Number,
}

impl LParticipleCell {
    /// Canonical gender-number inventory, ordered by number and gender.
    pub fn all() -> impl Iterator<Item = Self> {
        Number::ALL.into_iter().flat_map(|number| {
            Gender::ALL
                .into_iter()
                .map(move |gender| Self { gender, number })
        })
    }

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

/// A case-number cell for an unpositioned closed-class word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UngenderedCell {
    pub case: Case,
    pub number: Number,
}

impl UngenderedCell {
    /// Canonical case-number inventory for an unpositioned closed class.
    pub fn all() -> impl Iterator<Item = Self> {
        NounCell::all().map(|cell| Self {
            case: cell.case,
            number: cell.number,
        })
    }

    pub fn closed_class(self) -> ClosedClassCell {
        ClosedClassCell {
            case: self.case,
            number: self.number,
            gender: None,
            person: None,
        }
    }
}

/// A case-number-gender cell for an agreeing closed-class word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GenderedCell {
    pub case: Case,
    pub number: Number,
    pub gender: Gender,
}

impl GenderedCell {
    /// Canonical case-number-gender inventory for an agreeing closed class.
    pub fn all() -> impl Iterator<Item = Self> {
        Number::ALL.into_iter().flat_map(|number| {
            Case::ALL.into_iter().flat_map(move |case| {
                Gender::ALL.into_iter().map(move |gender| Self {
                    case,
                    number,
                    gender,
                })
            })
        })
    }

    pub fn closed_class(self) -> ClosedClassCell {
        ClosedClassCell {
            case: self.case,
            number: self.number,
            gender: Some(self.gender),
            person: None,
        }
    }
}

/// A grammatically typed cell for a collective numeral.
///
/// `дъвои`, `обои`, and `трои` use the pronominal agreement system, which has
/// no short/long or animacy dimension. Collectives from four through ten are
/// adjectives and therefore use the complete adjective cell instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectiveNumeralCell {
    Pronominal(GenderedCell),
    Adjectival(AdjectiveCell),
}

impl CollectiveNumeralCell {
    pub const fn pronominal(case: Case, number: Number, gender: Gender) -> Self {
        Self::Pronominal(GenderedCell {
            case,
            number,
            gender,
        })
    }

    pub const fn adjectival(
        form: AdjectiveForm,
        case: Case,
        number: Number,
        gender: Gender,
        animacy: Animacy,
    ) -> Self {
        Self::Adjectival(AdjectiveCell {
            case,
            number,
            gender,
            animacy,
            form,
        })
    }

    pub fn key(self) -> String {
        match self {
            Self::Pronominal(cell) => format!(
                "collective:pronominal:{}:{}:{}",
                cell.case.code(),
                cell.number.code(),
                cell.gender.code()
            ),
            Self::Adjectival(cell) => format!(
                "collective:adjectival:{}:{}:{}:{}:{}",
                cell.form.code(),
                cell.case.code(),
                cell.number.code(),
                cell.gender.code(),
                cell.animacy.code()
            ),
        }
    }
}

/// A case-number-person cell for a personal pronoun table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PersonalPronounCell {
    pub case: Case,
    pub number: Number,
    pub person: Person,
}

impl PersonalPronounCell {
    /// Canonical case-number-person inventory for personal pronouns.
    pub fn all() -> impl Iterator<Item = Self> {
        Number::ALL.into_iter().flat_map(|number| {
            Case::ALL.into_iter().flat_map(move |case| {
                Person::ALL.into_iter().map(move |person| Self {
                    case,
                    number,
                    person,
                })
            })
        })
    }

    pub fn closed_class(self) -> ClosedClassCell {
        ClosedClassCell {
            case: self.case,
            number: self.number,
            gender: None,
            person: Some(self.person),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClosedClassCell {
    pub case: Case,
    pub number: Number,
    pub gender: Option<Gender>,
    pub person: Option<Person>,
}

/// The grammatical request associated with a failed inflection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RequestedCell {
    Noun(NounCell),
    Adjective(AdjectiveCell),
    Determiner(DeterminerCell),
    Numeral(NumeralCell),
    CollectiveNumeral(CollectiveNumeralCell),
    CompoundCardinal {
        value: u16,
        cell: CompoundCardinalCell,
    },
    DistributiveCardinal {
        value: u16,
        cell: DistributiveCardinalCell,
    },
    CompoundOrdinal {
        value: u16,
        cell: AdjectiveCell,
    },
    Comparative(AdjectiveCell),
    FiniteVerb(FiniteVerbCell),
    Imperative(ImperativeCell),
    LParticiple(LParticipleCell),
    Participle(ParticipleCell),
    ClosedClass {
        part_of_speech: PartOfSpeech,
        cell: ClosedClassCell,
    },
    Infinitive,
    Supine,
    VerbalNoun,
    ComparativeCitation,
    RawFeature {
        feature: String,
    },
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
    /// Canonical adjective-agreement inventory for one participle system.
    pub fn for_kind(kind: ParticipleKind) -> impl Iterator<Item = Self> {
        AdjectiveCell::all().map(move |adjective| Self { kind, adjective })
    }

    pub fn key(self) -> String {
        format!(
            "verb:participle:{}:{}",
            self.kind.code(),
            self.adjective.key()
        )
    }
}
