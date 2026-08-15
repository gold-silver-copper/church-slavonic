//! Closed class-0 substantive inventory from Polivanova chapter 12.
//!
//! The source lists 37 fixed-gender substantives in seven mixed profile
//! families. `десѧть` and the three class-`0/s` pronouns are deliberately not
//! duplicated here: their complete numeral and pronoun owners use the same
//! source chapter. Unprinted cells below are explicit historical
//! reconstructions from the profile's productive stem alternation.

use crate::noun::NounLexeme;
use crate::{
    Animacy, Case, Gender, InflectionError, NounCell, NounClass, Number, NumberRestriction,
    PredictedForm, RequestedCell, RuleId, RuleStep,
};

const EN_NEUTERS: [&str; 7] = ["брѣмѧ", "врѣмѧ", "имѧ", "писмѧ", "племѧ", "сѣмѧ", "чисмѧ"];
const YOUNG_NEUTERS: [&str; 7] = [
    "агнѧ",
    "жрѣбѧ",
    "кл҄юсѧ",
    "козьлѧ",
    "овьчѧ",
    "осьлѧ",
    "отрочѧ",
];
const ES_NEUTERS: [&str; 6] = ["исто", "коло", "небо", "слово", "тѣло", "чудо"];
const EYE_EAR_NEUTERS: [&str; 2] = ["око", "ухо"];
const YV_FEMININES: [&str; 12] = [
    "брады",
    "букъви",
    "жрьны",
    "локы",
    "л҄юбы",
    "неплоды",
    "прѣл҄юбы",
    "свекры",
    "смокы",
    "хорѫгы",
    "црькы",
    "цѣлы",
];
const ER_FEMININES: [&str; 2] = ["дъщи", "мати"];
const LORD_MASCULINES: [&str; 1] = ["господь"];

/// One of the seven source-defined fixed-gender mixed profile families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UniqueNounProfile {
    EnNeuter,
    YoungNeuter,
    EsNeuter,
    EyeEarNeuter,
    YvFeminine,
    ErFeminine,
    LordMasculine,
}

impl UniqueNounProfile {
    pub const fn code(self) -> &'static str {
        match self {
            Self::EnNeuter => "0-n-en",
            Self::YoungNeuter => "0-n-yat-t",
            Self::EsNeuter => "0-n-es",
            Self::EyeEarNeuter => "0-n-eye-ear",
            Self::YvFeminine => "0-f-yv",
            Self::ErFeminine => "0-f-er",
            Self::LordMasculine => "0-m-lord",
        }
    }

    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::EnNeuter => RuleId::NounUniqueEnNeuter,
            Self::YoungNeuter => RuleId::NounUniqueYoungNeuter,
            Self::EsNeuter => RuleId::NounUniqueEsNeuter,
            Self::EyeEarNeuter => RuleId::NounUniqueEyeEarNeuter,
            Self::YvFeminine => RuleId::NounUniqueYvFeminine,
            Self::ErFeminine => RuleId::NounUniqueErFeminine,
            Self::LordMasculine => RuleId::NounUniqueLordMasculine,
        }
    }

    pub const fn source_section(self) -> &'static str {
        match self {
            Self::EnNeuter => "§§358–359",
            Self::YoungNeuter => "§§360–361",
            Self::EsNeuter => "§§362–363",
            Self::EyeEarNeuter => "§§364–366",
            Self::YvFeminine => "§§367–368",
            Self::ErFeminine => "§§369–370",
            Self::LordMasculine => "§§371–372",
        }
    }
}

/// Evidential status of one ordered class-0 noun realization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UniqueNounVariantStatus {
    /// The group profile prints this cell or variant directly.
    ReviewedTable,
    /// The source defines the stem alternation, but prints no member form in this cell.
    ReconstructedRule,
}

impl UniqueNounVariantStatus {
    pub const fn code(self) -> &'static str {
        match self {
            Self::ReviewedTable => "reviewed-table",
            Self::ReconstructedRule => "reconstructed-rule",
        }
    }
}

/// One ordered form and its source status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniqueNounVariant {
    pub prediction: PredictedForm,
    pub status: UniqueNounVariantStatus,
}

/// One member of the exhaustive fixed-gender class-0 substantive inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniqueNounFamilyMember {
    profile: UniqueNounProfile,
    lemma: &'static str,
}

impl UniqueNounFamilyMember {
    pub const COUNT: usize = 37;

    pub fn all() -> impl Iterator<Item = Self> {
        EN_NEUTERS
            .into_iter()
            .map(|lemma| Self::new(UniqueNounProfile::EnNeuter, lemma))
            .chain(
                YOUNG_NEUTERS
                    .into_iter()
                    .map(|lemma| Self::new(UniqueNounProfile::YoungNeuter, lemma)),
            )
            .chain(
                ES_NEUTERS
                    .into_iter()
                    .map(|lemma| Self::new(UniqueNounProfile::EsNeuter, lemma)),
            )
            .chain(
                EYE_EAR_NEUTERS
                    .into_iter()
                    .map(|lemma| Self::new(UniqueNounProfile::EyeEarNeuter, lemma)),
            )
            .chain(
                YV_FEMININES
                    .into_iter()
                    .map(|lemma| Self::new(UniqueNounProfile::YvFeminine, lemma)),
            )
            .chain(
                ER_FEMININES
                    .into_iter()
                    .map(|lemma| Self::new(UniqueNounProfile::ErFeminine, lemma)),
            )
            .chain(
                LORD_MASCULINES
                    .into_iter()
                    .map(|lemma| Self::new(UniqueNounProfile::LordMasculine, lemma)),
            )
    }

    const fn new(profile: UniqueNounProfile, lemma: &'static str) -> Self {
        Self { profile, lemma }
    }

    pub fn classify_source_lemma(lemma: &str) -> Option<Self> {
        Self::all().find(|member| member.lemma == lemma)
    }

    pub const fn canonical_lemma(self) -> &'static str {
        self.lemma
    }

    pub const fn profile(self) -> UniqueNounProfile {
        self.profile
    }

    pub const fn source_class(self) -> &'static str {
        match self.profile {
            UniqueNounProfile::EnNeuter
            | UniqueNounProfile::YoungNeuter
            | UniqueNounProfile::EsNeuter
            | UniqueNounProfile::EyeEarNeuter => "0/n",
            UniqueNounProfile::YvFeminine | UniqueNounProfile::ErFeminine => "0/f",
            UniqueNounProfile::LordMasculine => "0/m",
        }
    }

    pub const fn source_section(self) -> &'static str {
        self.profile.source_section()
    }

    pub const fn gender(self) -> Gender {
        match self.profile {
            UniqueNounProfile::EnNeuter
            | UniqueNounProfile::YoungNeuter
            | UniqueNounProfile::EsNeuter
            | UniqueNounProfile::EyeEarNeuter => Gender::Neuter,
            UniqueNounProfile::YvFeminine | UniqueNounProfile::ErFeminine => Gender::Feminine,
            UniqueNounProfile::LordMasculine => Gender::Masculine,
        }
    }

    pub fn number_restriction(self) -> NumberRestriction {
        if self.lemma == "букъви" {
            NumberRestriction::PluralOnly
        } else {
            NumberRestriction::All
        }
    }

    pub fn lexeme(self) -> NounLexeme {
        NounLexeme {
            lemma: self.lemma.to_string(),
            class: NounClass::UniqueMixed,
            gender: self.gender(),
            animacy: Animacy::Inanimate,
            number_restriction: self.number_restriction(),
        }
    }

    /// Return every source-ordered variant for a licensed cell.
    pub fn decline(self, cell: NounCell) -> Result<Vec<UniqueNounVariant>, InflectionError> {
        if !restriction_supports(self.number_restriction(), cell.number) {
            return Err(InflectionError::unsupported(
                self.lemma,
                RequestedCell::Noun(cell),
            ));
        }
        Ok(match self.profile {
            UniqueNounProfile::EnNeuter => decline_en(self, cell),
            UniqueNounProfile::YoungNeuter => decline_young(self, cell),
            UniqueNounProfile::EsNeuter => decline_es(self, cell),
            UniqueNounProfile::EyeEarNeuter => decline_eye_ear(self, cell),
            UniqueNounProfile::YvFeminine => decline_yv(self, cell),
            UniqueNounProfile::ErFeminine => decline_er(self, cell),
            UniqueNounProfile::LordMasculine => decline_lord(self, cell),
        })
    }

    pub fn decline_primary(self, cell: NounCell) -> Result<PredictedForm, InflectionError> {
        self.decline(cell)?
            .into_iter()
            .next()
            .map(|variant| variant.prediction)
            .ok_or_else(|| InflectionError::InvalidInput {
                reason: format!("unique noun {} produced no form", self.lemma),
            })
    }

    fn variants(
        self,
        status: UniqueNounVariantStatus,
        texts: impl IntoIterator<Item = String>,
    ) -> Vec<UniqueNounVariant> {
        texts
            .into_iter()
            .map(|text| self.variant(status, text))
            .collect()
    }

    fn variant(self, status: UniqueNounVariantStatus, text: String) -> UniqueNounVariant {
        let rule_id = self.profile.rule_id();
        let reason = match status {
            UniqueNounVariantStatus::ReviewedTable => {
                "select the source-listed class-0 profile realization"
            }
            UniqueNounVariantStatus::ReconstructedRule => {
                "reconstruct an unprinted class-0 cell from the source-defined stem alternation"
            }
        };
        UniqueNounVariant {
            prediction: PredictedForm {
                text: text.clone(),
                rule_id,
                trace: vec![RuleStep {
                    rule_id,
                    before: self.lemma.to_string(),
                    after: text,
                    reason,
                }],
            },
            status,
        }
    }
}

fn restriction_supports(restriction: NumberRestriction, number: Number) -> bool {
    match restriction {
        NumberRestriction::All => true,
        NumberRestriction::SingularOnly => number == Number::Singular,
        NumberRestriction::DualOnly => number == Number::Dual,
        NumberRestriction::PluralOnly => number == Number::Plural,
    }
}

fn strip<'a>(lemma: &'a str, ending: &str) -> &'a str {
    lemma
        .strip_suffix(ending)
        .filter(|stem| !stem.is_empty())
        .unwrap_or(lemma)
}

fn forms(
    member: UniqueNounFamilyMember,
    status: UniqueNounVariantStatus,
    forms: &[String],
) -> Vec<UniqueNounVariant> {
    member.variants(status, forms.iter().cloned())
}

fn one(
    member: UniqueNounFamilyMember,
    status: UniqueNounVariantStatus,
    text: String,
) -> Vec<UniqueNounVariant> {
    member.variants(status, [text])
}

fn decline_en(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    use UniqueNounVariantStatus::{ReconstructedRule as R, ReviewedTable as S};
    let expanded = format!("{}ен", strip(member.lemma, "ѧ"));
    match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => {
            one(member, S, member.lemma.into())
        }
        (Case::Vocative, Number::Singular) => one(member, R, member.lemma.into()),
        (Case::Genitive, Number::Singular) => {
            forms(member, S, &[format!("{expanded}е"), format!("{expanded}и")])
        }
        (Case::Dative | Case::Locative, Number::Singular) => one(member, S, format!("{expanded}и")),
        (Case::Instrumental, Number::Singular) => forms(
            member,
            S,
            &[format!("{expanded}емь"), format!("{expanded}ьмь")],
        ),
        (Case::Nominative | Case::Accusative, Number::Dual) => {
            forms(member, S, &[format!("{expanded}ѣ"), format!("{expanded}и")])
        }
        (Case::Vocative, Number::Dual) => {
            forms(member, R, &[format!("{expanded}ѣ"), format!("{expanded}и")])
        }
        (Case::Genitive | Case::Locative, Number::Dual) => one(member, R, format!("{expanded}оу")),
        (Case::Dative | Case::Instrumental, Number::Dual) => {
            one(member, R, format!("{expanded}ьма"))
        }
        (Case::Nominative | Case::Accusative, Number::Plural) => {
            one(member, S, format!("{expanded}а"))
        }
        (Case::Vocative, Number::Plural) => one(member, R, format!("{expanded}а")),
        (Case::Genitive, Number::Plural) => one(member, S, format!("{expanded}ъ")),
        (Case::Dative, Number::Plural) => one(member, S, format!("{expanded}емъ")),
        (Case::Instrumental, Number::Plural) => one(member, S, format!("{expanded}ы")),
        (Case::Locative, Number::Plural) => one(member, S, format!("{expanded}ехъ")),
    }
}

fn decline_young(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    use UniqueNounVariantStatus::{ReconstructedRule as R, ReviewedTable as S};
    let expanded = format!("{}ѧт", strip(member.lemma, "ѧ"));
    match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => {
            one(member, S, member.lemma.into())
        }
        (Case::Vocative, Number::Singular) => one(member, R, member.lemma.into()),
        (Case::Genitive | Case::Locative, Number::Singular) => {
            forms(member, S, &[format!("{expanded}е"), format!("{expanded}и")])
        }
        (Case::Dative, Number::Singular) => one(member, S, format!("{expanded}и")),
        (Case::Instrumental, Number::Singular) => one(member, R, format!("{expanded}ьмь")),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            one(member, R, format!("{expanded}ѣ"))
        }
        (Case::Genitive | Case::Locative, Number::Dual) => one(member, R, format!("{expanded}оу")),
        (Case::Dative | Case::Instrumental, Number::Dual) => {
            one(member, R, format!("{expanded}ьма"))
        }
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Plural) => {
            one(member, R, format!("{expanded}а"))
        }
        (Case::Genitive, Number::Plural) => one(member, S, format!("{expanded}ъ")),
        (Case::Dative, Number::Plural) => one(member, R, format!("{expanded}ьмъ")),
        (Case::Instrumental, Number::Plural) => one(member, R, format!("{expanded}ы")),
        (Case::Locative, Number::Plural) => one(member, R, format!("{expanded}ьхъ")),
    }
}

fn decline_es(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    use UniqueNounVariantStatus::{ReconstructedRule as R, ReviewedTable as S};
    let expanded = format!("{}ес", strip(member.lemma, "о"));
    match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => {
            one(member, S, member.lemma.into())
        }
        (Case::Vocative, Number::Singular) => one(member, R, member.lemma.into()),
        (Case::Genitive, Number::Singular) => {
            forms(member, S, &[format!("{expanded}е"), format!("{expanded}и")])
        }
        (Case::Dative | Case::Locative, Number::Singular) => one(member, S, format!("{expanded}и")),
        (Case::Instrumental, Number::Singular) => forms(
            member,
            S,
            &[format!("{expanded}емь"), format!("{expanded}ьмь")],
        ),
        (Case::Nominative | Case::Accusative, Number::Dual) => {
            forms(member, S, &[format!("{expanded}ѣ"), format!("{expanded}и")])
        }
        (Case::Vocative, Number::Dual) => {
            forms(member, R, &[format!("{expanded}ѣ"), format!("{expanded}и")])
        }
        (Case::Genitive | Case::Locative, Number::Dual) => one(member, S, format!("{expanded}у")),
        (Case::Dative | Case::Instrumental, Number::Dual) => {
            one(member, R, format!("{expanded}ьма"))
        }
        (Case::Nominative | Case::Accusative, Number::Plural) => {
            one(member, S, format!("{expanded}а"))
        }
        (Case::Vocative, Number::Plural) => one(member, R, format!("{expanded}а")),
        (Case::Genitive, Number::Plural) => one(member, S, format!("{expanded}ъ")),
        (Case::Dative, Number::Plural) => one(member, S, format!("{expanded}емъ")),
        (Case::Instrumental, Number::Plural) => one(member, S, format!("{expanded}ы")),
        (Case::Locative, Number::Plural) => one(member, S, format!("{expanded}ехъ")),
    }
}

fn decline_eye_ear(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    if member.lemma == "око" {
        decline_eye(member, cell)
    } else {
        decline_ear(member, cell)
    }
}

fn decline_eye(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    use UniqueNounVariantStatus::{ReconstructedRule as R, ReviewedTable as S};
    match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => one(member, S, "око".into()),
        (Case::Vocative, Number::Singular) => one(member, R, "око".into()),
        (Case::Genitive, Number::Singular) => forms(member, S, &["очесе".into(), "ока".into()]),
        (Case::Dative, Number::Singular) => one(member, S, "очеси".into()),
        (Case::Instrumental, Number::Singular) => one(member, S, "окомь".into()),
        (Case::Locative, Number::Singular) => {
            forms(member, S, &["очесе".into(), "очеси".into(), "оцѣ".into()])
        }
        (Case::Nominative | Case::Accusative, Number::Dual) => one(member, S, "очи".into()),
        (Case::Vocative, Number::Dual) => one(member, R, "очи".into()),
        (Case::Genitive | Case::Locative, Number::Dual) => one(member, S, "очию".into()),
        (Case::Dative | Case::Instrumental, Number::Dual) => one(member, S, "очима".into()),
        (Case::Nominative | Case::Accusative, Number::Plural) => one(member, S, "очеса".into()),
        (Case::Vocative, Number::Plural) => one(member, R, "очеса".into()),
        (Case::Genitive, Number::Plural) => one(member, S, "очесъ".into()),
        (Case::Dative, Number::Plural) => one(member, R, "очесьмъ".into()),
        (Case::Instrumental, Number::Plural) => one(member, S, "очесы".into()),
        (Case::Locative, Number::Plural) => one(member, R, "очесьхъ".into()),
    }
}

fn decline_ear(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    use UniqueNounVariantStatus::{ReconstructedRule as R, ReviewedTable as S};
    match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => one(member, S, "ухо".into()),
        (Case::Vocative, Number::Singular) => one(member, R, "ухо".into()),
        (Case::Genitive | Case::Locative, Number::Singular) => one(member, R, "ушесе".into()),
        (Case::Dative, Number::Singular) => one(member, S, "уху".into()),
        (Case::Instrumental, Number::Singular) => one(member, R, "ухомь".into()),
        (Case::Nominative | Case::Accusative, Number::Dual) => one(member, S, "уши".into()),
        (Case::Vocative, Number::Dual) => one(member, R, "уши".into()),
        (Case::Genitive | Case::Locative, Number::Dual) => one(member, S, "ушию".into()),
        (Case::Dative | Case::Instrumental, Number::Dual) => one(member, S, "ушима".into()),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Plural) => {
            one(member, R, "ушеса".into())
        }
        (Case::Genitive, Number::Plural) => one(member, R, "ушесъ".into()),
        (Case::Dative, Number::Plural) => one(member, R, "ушесьмъ".into()),
        (Case::Instrumental, Number::Plural) => one(member, S, "ушесы".into()),
        (Case::Locative, Number::Plural) => one(member, R, "ушесьхъ".into()),
    }
}

fn decline_yv(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    use UniqueNounVariantStatus::{ReconstructedRule as R, ReviewedTable as S};
    let expanded = format!("{}ъв", strip(member.lemma, "ы"));
    match (cell.case, cell.number) {
        (Case::Nominative, Number::Singular) => one(member, S, member.lemma.into()),
        (Case::Vocative, Number::Singular) => one(member, R, member.lemma.into()),
        (Case::Accusative, Number::Singular) => one(member, S, format!("{expanded}ь")),
        (Case::Genitive | Case::Locative, Number::Singular) => {
            forms(member, S, &[format!("{expanded}е"), format!("{expanded}и")])
        }
        (Case::Dative, Number::Singular) => one(member, S, format!("{expanded}и")),
        (Case::Instrumental, Number::Singular) => one(member, S, format!("{expanded}иѭ")),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            one(member, R, format!("{expanded}и"))
        }
        (Case::Genitive | Case::Locative, Number::Dual) => one(member, R, format!("{expanded}оу")),
        (Case::Dative | Case::Instrumental, Number::Dual) => {
            one(member, R, format!("{expanded}ама"))
        }
        (Case::Nominative, Number::Plural) => one(member, S, format!("{expanded}и")),
        (Case::Accusative | Case::Vocative, Number::Plural) => {
            one(member, R, format!("{expanded}и"))
        }
        (Case::Genitive, Number::Plural) => one(member, S, format!("{expanded}ъ")),
        (Case::Dative, Number::Plural) => one(member, S, format!("{expanded}амъ")),
        (Case::Instrumental, Number::Plural) => one(member, R, format!("{expanded}ами")),
        (Case::Locative, Number::Plural) => one(member, S, format!("{expanded}ахъ")),
    }
}

fn decline_er(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    use UniqueNounVariantStatus::{ReconstructedRule as R, ReviewedTable as S};
    let expanded = format!("{}ер", strip(member.lemma, "и"));
    match (cell.case, cell.number) {
        (Case::Nominative, Number::Singular) => one(member, S, member.lemma.into()),
        (Case::Vocative, Number::Singular) => one(member, R, member.lemma.into()),
        (Case::Accusative, Number::Singular) => one(member, S, format!("{expanded}ь")),
        (Case::Genitive | Case::Locative, Number::Singular) => {
            forms(member, S, &[format!("{expanded}е"), format!("{expanded}и")])
        }
        (Case::Dative, Number::Singular) => one(member, S, format!("{expanded}и")),
        (Case::Instrumental, Number::Singular) => one(member, S, format!("{expanded}иѭ")),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            one(member, R, format!("{expanded}и"))
        }
        (Case::Genitive | Case::Locative, Number::Dual) => one(member, R, format!("{expanded}оу")),
        (Case::Dative | Case::Instrumental, Number::Dual) => {
            one(member, R, format!("{expanded}ьма"))
        }
        (Case::Nominative, Number::Plural) => one(member, S, format!("{expanded}и")),
        (Case::Accusative | Case::Vocative, Number::Plural) => {
            one(member, R, format!("{expanded}и"))
        }
        (Case::Genitive, Number::Plural) => forms(
            member,
            S,
            &[format!("{expanded}ъ"), format!("{expanded}ии")],
        ),
        (Case::Dative, Number::Plural) => one(member, S, format!("{expanded}емъ")),
        (Case::Instrumental, Number::Plural) => one(member, S, format!("{expanded}ьми")),
        (Case::Locative, Number::Plural) => one(member, S, format!("{expanded}ехъ")),
    }
}

fn decline_lord(member: UniqueNounFamilyMember, cell: NounCell) -> Vec<UniqueNounVariant> {
    use UniqueNounVariantStatus::{ReconstructedRule as R, ReviewedTable as S};
    let stem = strip(member.lemma, "ь");
    match (cell.case, cell.number) {
        (Case::Nominative | Case::Accusative, Number::Singular) => {
            one(member, S, member.lemma.into())
        }
        (Case::Genitive, Number::Singular) => forms(
            member,
            S,
            &[format!("{stem}и"), format!("{stem}а"), format!("{stem}ѣ")],
        ),
        (Case::Dative, Number::Singular) => forms(
            member,
            S,
            &[
                format!("{stem}и"),
                format!("{stem}у"),
                format!("{stem}ю"),
                format!("{stem}еви"),
            ],
        ),
        (Case::Instrumental, Number::Singular) => {
            forms(member, S, &[format!("{stem}ьмь"), format!("{stem}емь")])
        }
        (Case::Locative | Case::Vocative, Number::Singular) => one(member, S, format!("{stem}и")),
        (Case::Nominative | Case::Accusative | Case::Vocative, Number::Dual) => {
            one(member, R, format!("{stem}и"))
        }
        (Case::Genitive | Case::Locative, Number::Dual) => one(member, R, format!("{stem}ью")),
        (Case::Dative | Case::Instrumental, Number::Dual) => one(member, S, format!("{stem}ьма")),
        (Case::Nominative, Number::Plural) => one(member, S, format!("{stem}иѥ")),
        (Case::Vocative, Number::Plural) => one(member, R, format!("{stem}иѥ")),
        (Case::Genitive, Number::Plural) => one(member, S, format!("{stem}ии")),
        (Case::Dative, Number::Plural) => one(member, R, format!("{stem}ьмъ")),
        (Case::Accusative, Number::Plural) => one(member, R, format!("{stem}и")),
        (Case::Instrumental, Number::Plural) => one(member, R, format!("{stem}ьми")),
        (Case::Locative, Number::Plural) => one(member, R, format!("{stem}ьхъ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn fixed_gender_inventory_is_exhaustive_unique_and_profiled() {
        let members = UniqueNounFamilyMember::all().collect::<Vec<_>>();
        assert_eq!(members.len(), UniqueNounFamilyMember::COUNT);
        assert_eq!(
            members
                .iter()
                .map(|member| member.canonical_lemma())
                .collect::<BTreeSet<_>>()
                .len(),
            UniqueNounFamilyMember::COUNT
        );
        for (profile, expected) in [
            (UniqueNounProfile::EnNeuter, 7),
            (UniqueNounProfile::YoungNeuter, 7),
            (UniqueNounProfile::EsNeuter, 6),
            (UniqueNounProfile::EyeEarNeuter, 2),
            (UniqueNounProfile::YvFeminine, 12),
            (UniqueNounProfile::ErFeminine, 2),
            (UniqueNounProfile::LordMasculine, 1),
        ] {
            assert_eq!(
                members
                    .iter()
                    .filter(|member| member.profile == profile)
                    .count(),
                expected,
                "{}",
                profile.code()
            );
        }
        for excluded in ["десѧть", "азъ", "ты", "сѧ"] {
            assert_eq!(
                UniqueNounFamilyMember::classify_source_lemma(excluded),
                None
            );
        }
    }

    #[test]
    fn every_member_realizes_every_lexically_valid_cell() {
        for member in UniqueNounFamilyMember::all() {
            for cell in NounCell::all() {
                let result = member.decline(cell);
                if member.canonical_lemma() == "букъви" && cell.number != Number::Plural {
                    assert!(matches!(
                        result,
                        Err(InflectionError::UnsupportedCell { .. })
                    ));
                    continue;
                }
                let variants =
                    result.unwrap_or_else(|error| panic!("{member:?} {cell:?}: {error}"));
                assert!(!variants.is_empty(), "{member:?} {cell:?}");
                assert!(variants.iter().all(|variant| {
                    !variant.prediction.text.is_empty()
                        && variant.prediction.rule_id == member.profile.rule_id()
                }));
            }
        }
    }

    #[test]
    fn model_profiles_keep_source_variants_and_reconstruction_boundaries() {
        let variants = |lemma, case, number| {
            UniqueNounFamilyMember::classify_source_lemma(lemma)
                .unwrap_or_else(|| panic!("missing {lemma}"))
                .decline(NounCell { case, number })
                .unwrap_or_else(|error| panic!("{lemma}: {error}"))
        };
        assert_eq!(
            variants("имѧ", Case::Instrumental, Number::Singular)
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["именемь", "именьмь"]
        );
        assert_eq!(
            variants("око", Case::Locative, Number::Singular)
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["очесе", "очеси", "оцѣ"]
        );
        assert_eq!(
            variants("господь", Case::Dative, Number::Singular)
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect::<Vec<_>>(),
            ["господи", "господу", "господю", "господеви"]
        );
        let unattested = variants("ухо", Case::Genitive, Number::Plural);
        assert_eq!(unattested[0].prediction.text, "ушесъ");
        assert_eq!(
            unattested[0].status,
            UniqueNounVariantStatus::ReconstructedRule
        );
    }
}
