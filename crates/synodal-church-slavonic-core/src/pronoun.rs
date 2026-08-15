//! Source-reviewed Synodal Church Slavonic pronoun morphology.

use crate::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, Comparison, Error, FormSet,
    Gender, Number, OrthographyProfile, Person, PronounCell, Result, SynodalWord,
    morphology::{long_adjective_ending, normative_variants},
};

/// Productive and closed suppletive paradigms described by Alypy §§45–48.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PronounDeclension {
    PersonalFirst,
    PersonalSecond,
    Reflexive,
    ThirdPerson,
    /// The single lexeme `онъ`, whose person-bearing cells supply the
    /// third-person paradigm and whose personless cells decline as the short
    /// demonstrative in Alypy §§45–48.
    ThirdPersonAndDemonstrative,
    ProximalSei,
    Soft,
    SoftIAlternating,
    Hard,
    MixedPossessive,
    ShortHard,
    ShortOvMixed,
    ShortVelar,
    QuantityVelar,
    FullHard,
    FullSoft,
    FullVelar,
    InterrogativeKii,
    InterrogativeWho,
    InterrogativeWhat,
}

impl PronounDeclension {
    pub const ALL: [Self; 20] = [
        Self::PersonalFirst,
        Self::PersonalSecond,
        Self::Reflexive,
        Self::ThirdPerson,
        Self::ThirdPersonAndDemonstrative,
        Self::ProximalSei,
        Self::Soft,
        Self::SoftIAlternating,
        Self::Hard,
        Self::MixedPossessive,
        Self::ShortHard,
        Self::ShortOvMixed,
        Self::ShortVelar,
        Self::QuantityVelar,
        Self::FullHard,
        Self::FullSoft,
        Self::FullVelar,
        Self::InterrogativeKii,
        Self::InterrogativeWho,
        Self::InterrogativeWhat,
    ];

    #[must_use]
    pub const fn requires_stem(self) -> bool {
        matches!(
            self,
            Self::ThirdPersonAndDemonstrative
                | Self::Soft
                | Self::SoftIAlternating
                | Self::Hard
                | Self::MixedPossessive
                | Self::ShortHard
                | Self::ShortOvMixed
                | Self::ShortVelar
                | Self::QuantityVelar
                | Self::FullHard
                | Self::FullSoft
                | Self::FullVelar
        )
    }
}

/// Lexically licensed grammatical numbers for an agreeing pronoun.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PronounNumberInventory {
    #[default]
    All,
    SingularOnly,
    DualOnly,
    PluralOnly,
    SingularAndDual,
    SingularAndPlural,
    DualAndPlural,
}

impl PronounNumberInventory {
    #[must_use]
    pub const fn contains(self, number: Number) -> bool {
        matches!(
            (self, number),
            (Self::All, _)
                | (Self::SingularOnly, Number::Singular)
                | (Self::DualOnly, Number::Dual)
                | (Self::PluralOnly, Number::Plural)
                | (Self::SingularAndDual, Number::Singular | Number::Dual)
                | (Self::SingularAndPlural, Number::Singular | Number::Plural)
                | (Self::DualAndPlural, Number::Dual | Number::Plural)
        )
    }
}

/// Selects the source-table primary forms, forms explicitly identified by
/// Alypy §47 as enclitic, or both in table order.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PronounFormSelection {
    #[default]
    All,
    TablePrimary,
    Enclitic,
}

/// Conditions the `н-` allomorph of the third-person and relative pronouns.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PronounEnvironment {
    #[default]
    Independent,
    AfterPreposition,
    /// Retains both explicitly conditioned series for APIs that do not carry
    /// the governing preposition. Independent forms precede `н-` forms;
    /// nominatives remain independent and locatives remain prepositional.
    ContextualVariants,
}

/// Prefixes which Alypy §§46 and 48 use to derive indefinite and negative
/// pronouns from the interrogative paradigms.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PronounPrefix {
    IndefiniteNe,
    NegativeNi,
}

impl PronounPrefix {
    const fn text(self) -> &'static str {
        match self {
            Self::IndefiniteNe => "нѣ",
            Self::NegativeNi => "ни",
        }
    }
}

/// Invariant bound particles whose host is the inflected first component.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PronounPostpositive {
    Zhe,
    Zhdo,
}

impl PronounPostpositive {
    const fn text(self) -> &'static str {
        match self {
            Self::Zhe => "же",
            Self::Zhdo => "ждо",
        }
    }
}

/// Complete typed metadata for one productive or closed suppletive pronoun.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PronounLexeme {
    pub lemma: SynodalWord,
    /// Base before the endings of a regular agreeing paradigm (`мо-`, `т-`,
    /// `наш-`). Closed suppletive paradigms do not use a stem.
    pub stem: Option<SynodalWord>,
    pub declension: PronounDeclension,
    pub number_inventory: PronounNumberInventory,
    pub selection: PronounFormSelection,
    pub environment: PronounEnvironment,
    pub prefix: Option<PronounPrefix>,
    pub postpositive: Option<PronounPostpositive>,
}

impl PronounLexeme {
    #[must_use]
    pub const fn closed(lemma: SynodalWord, declension: PronounDeclension) -> Self {
        Self {
            lemma,
            stem: None,
            declension,
            number_inventory: PronounNumberInventory::All,
            selection: PronounFormSelection::All,
            environment: PronounEnvironment::Independent,
            prefix: None,
            postpositive: None,
        }
    }

    #[must_use]
    pub const fn regular(
        lemma: SynodalWord,
        stem: SynodalWord,
        declension: PronounDeclension,
    ) -> Self {
        Self {
            lemma,
            stem: Some(stem),
            declension,
            number_inventory: PronounNumberInventory::All,
            selection: PronounFormSelection::All,
            environment: PronounEnvironment::Independent,
            prefix: None,
            postpositive: None,
        }
    }

    #[must_use]
    pub const fn with_selection(mut self, selection: PronounFormSelection) -> Self {
        self.selection = selection;
        self
    }

    #[must_use]
    pub const fn with_number_inventory(mut self, inventory: PronounNumberInventory) -> Self {
        self.number_inventory = inventory;
        self
    }

    #[must_use]
    pub const fn with_environment(mut self, environment: PronounEnvironment) -> Self {
        self.environment = environment;
        self
    }

    #[must_use]
    pub const fn with_prefix(mut self, prefix: PronounPrefix) -> Self {
        self.prefix = Some(prefix);
        self
    }

    #[must_use]
    pub const fn with_postpositive(mut self, postpositive: PronounPostpositive) -> Self {
        self.postpositive = Some(postpositive);
        self
    }
}

/// Validates the class-specific lexical metadata without generating a form.
pub fn validate_pronoun_lexeme(lexeme: &PronounLexeme) -> Result<()> {
    if lexeme.declension.requires_stem() != lexeme.stem.is_some() {
        return Err(Error::ContradictoryMetadata {
            reason: "regular agreeing pronouns require exactly one explicit declensional stem"
                .into(),
        });
    }
    if matches!(
        lexeme.declension,
        PronounDeclension::ShortVelar
            | PronounDeclension::QuantityVelar
            | PronounDeclension::FullVelar
    ) && !required_stem(lexeme)?
        .chars()
        .last()
        .is_some_and(|last| matches!(last, 'к' | 'г' | 'х'))
    {
        return Err(Error::ContradictoryMetadata {
            reason: "a velar pronoun class requires a stem ending in к, г, or х".into(),
        });
    }
    if lexeme.declension == PronounDeclension::SoftIAlternating
        && !required_stem(lexeme)?.ends_with('і')
    {
        return Err(Error::ContradictoryMetadata {
            reason: "the і/ї-alternating soft pronoun class requires a stem ending in і".into(),
        });
    }
    if lexeme.declension == PronounDeclension::ShortOvMixed
        && !required_stem(lexeme)?.ends_with("ов")
    {
        return Err(Error::ContradictoryMetadata {
            reason: "the mixed -ов- pronoun class requires a stem ending in -ов".into(),
        });
    }
    if lexeme.selection != PronounFormSelection::All
        && !matches!(
            lexeme.declension,
            PronounDeclension::PersonalFirst
                | PronounDeclension::PersonalSecond
                | PronounDeclension::Reflexive
        )
    {
        return Err(Error::ContradictoryMetadata {
            reason: "enclitic selection is licensed only for first-person, second-person, and reflexive pronouns"
                .into(),
        });
    }
    if matches!(
        lexeme.declension,
        PronounDeclension::Reflexive
            | PronounDeclension::InterrogativeWho
            | PronounDeclension::InterrogativeWhat
    ) && !lexeme.number_inventory.contains(Number::Singular)
    {
        return Err(Error::ContradictoryMetadata {
            reason: "a formally singular-only pronoun requires a number inventory containing the singular"
                .into(),
        });
    }
    if lexeme.environment != PronounEnvironment::Independent
        && !matches!(
            lexeme.declension,
            PronounDeclension::ThirdPerson | PronounDeclension::ThirdPersonAndDemonstrative
        )
    {
        return Err(Error::ContradictoryMetadata {
            reason: "post-prepositional n-allomorphy is licensed only for the third-person base"
                .into(),
        });
    }
    if lexeme.prefix.is_some()
        && !matches!(
            lexeme.declension,
            PronounDeclension::InterrogativeKii
                | PronounDeclension::InterrogativeWho
                | PronounDeclension::InterrogativeWhat
                | PronounDeclension::FullHard
                | PronounDeclension::FullSoft
                | PronounDeclension::FullVelar
        )
    {
        return Err(Error::ContradictoryMetadata {
            reason: "нѣ-/ни- composition requires a typed interrogative base".into(),
        });
    }
    if lexeme.prefix.is_some()
        && lexeme.postpositive.is_some()
        && !matches!(
            (lexeme.prefix, lexeme.postpositive, lexeme.declension),
            (
                Some(PronounPrefix::NegativeNi),
                Some(PronounPostpositive::Zhe),
                PronounDeclension::InterrogativeWho | PronounDeclension::InterrogativeWhat
            )
        )
    {
        return Err(Error::ContradictoryMetadata {
            reason:
                "only ни- plus -же is licensed as a simultaneous pronoun prefix and postpositive"
                    .into(),
        });
    }
    if let Some(postpositive) = lexeme.postpositive {
        let valid = match postpositive {
            PronounPostpositive::Zhe => matches!(
                lexeme.declension,
                PronounDeclension::ThirdPerson
                    | PronounDeclension::InterrogativeWho
                    | PronounDeclension::InterrogativeWhat
            ),
            PronounPostpositive::Zhdo => lexeme.declension == PronounDeclension::InterrogativeKii,
        };
        if !valid {
            return Err(Error::ContradictoryMetadata {
                reason: "the selected postpositive is not licensed for this pronoun base".into(),
            });
        }
    }
    Ok(())
}

/// Generates one source-licensed Synodal pronoun cell.
pub fn decline_pronoun(
    lexeme: &PronounLexeme,
    cell: PronounCell,
    profile: OrthographyProfile,
) -> Result<FormSet> {
    validate_pronoun_lexeme(lexeme)?;
    validate_cell(lexeme, cell)?;

    let forms = match lexeme.declension {
        PronounDeclension::PersonalFirst => personal_forms(true, cell, lexeme.selection),
        PronounDeclension::PersonalSecond => personal_forms(false, cell, lexeme.selection),
        PronounDeclension::Reflexive => reflexive_forms(cell, lexeme.selection),
        PronounDeclension::ThirdPerson => {
            third_person_forms(cell, lexeme.environment, lexeme.postpositive)
        }
        PronounDeclension::ThirdPersonAndDemonstrative => {
            if cell.person == Some(Person::Third) {
                third_person_forms(cell, lexeme.environment, None)
            } else {
                short_hard_forms(required_stem(lexeme)?, cell)
            }
        }
        PronounDeclension::ProximalSei => proximal_sei_forms(cell),
        PronounDeclension::Soft => soft_forms(required_stem(lexeme)?, cell),
        PronounDeclension::SoftIAlternating => {
            soft_i_alternating_forms(required_stem(lexeme)?, cell)
        }
        PronounDeclension::Hard => hard_forms(required_stem(lexeme)?, cell),
        PronounDeclension::MixedPossessive => mixed_forms(required_stem(lexeme)?, cell),
        PronounDeclension::ShortHard => short_hard_forms(required_stem(lexeme)?, cell),
        PronounDeclension::ShortOvMixed => short_ov_mixed_forms(required_stem(lexeme)?, cell),
        PronounDeclension::ShortVelar => short_velar_forms(required_stem(lexeme)?, cell, false),
        PronounDeclension::QuantityVelar => short_velar_forms(required_stem(lexeme)?, cell, true),
        PronounDeclension::FullHard => full_forms(
            required_stem(lexeme)?,
            cell,
            AdjectiveClass::Hard,
            false,
            false,
        ),
        PronounDeclension::FullSoft => full_forms(
            required_stem(lexeme)?,
            cell,
            AdjectiveClass::Soft,
            false,
            false,
        ),
        PronounDeclension::FullVelar => full_forms(
            required_stem(lexeme)?,
            cell,
            AdjectiveClass::Hard,
            true,
            lexeme.lemma.canonical().ends_with("їй"),
        ),
        PronounDeclension::InterrogativeKii => interrogative_kii(cell),
        PronounDeclension::InterrogativeWho => interrogative_who(cell),
        PronounDeclension::InterrogativeWhat => interrogative_what(cell),
    }?;
    let forms = compose(
        forms,
        lexeme.prefix,
        lexeme.postpositive,
        lexeme.declension,
        cell,
    );
    let rule = rule_id(lexeme, cell);
    normative_variants(
        forms,
        rule,
        profile,
        "pronoun-declension",
        lexeme.lemma.canonical(),
    )
}

fn required_stem(lexeme: &PronounLexeme) -> Result<&str> {
    lexeme
        .stem
        .as_ref()
        .map(SynodalWord::canonical)
        .ok_or_else(|| Error::ContradictoryMetadata {
            reason: "regular pronoun stem is missing".into(),
        })
}

fn validate_cell(lexeme: &PronounLexeme, cell: PronounCell) -> Result<()> {
    if !lexeme.number_inventory.contains(cell.number) {
        return Err(Error::HistoricallyInvalidCell {
            reason: "the requested number is outside this pronoun's lexical inventory".into(),
        });
    }
    let valid_dimensions = match lexeme.declension {
        PronounDeclension::PersonalFirst => {
            cell.gender.is_none() && cell.person == Some(Person::First)
        }
        PronounDeclension::PersonalSecond => {
            cell.gender.is_none() && cell.person == Some(Person::Second)
        }
        PronounDeclension::Reflexive
        | PronounDeclension::InterrogativeWho
        | PronounDeclension::InterrogativeWhat => cell.gender.is_none() && cell.person.is_none(),
        PronounDeclension::ThirdPerson => {
            let expected_person = if lexeme.postpositive == Some(PronounPostpositive::Zhe) {
                None
            } else {
                Some(Person::Third)
            };
            cell.gender.is_some() && cell.person == expected_person
        }
        PronounDeclension::ThirdPersonAndDemonstrative => {
            cell.gender.is_some() && matches!(cell.person, None | Some(Person::Third))
        }
        PronounDeclension::Soft
        | PronounDeclension::SoftIAlternating
        | PronounDeclension::Hard
        | PronounDeclension::MixedPossessive
        | PronounDeclension::ShortHard
        | PronounDeclension::ShortOvMixed
        | PronounDeclension::ShortVelar
        | PronounDeclension::QuantityVelar
        | PronounDeclension::FullHard
        | PronounDeclension::FullSoft
        | PronounDeclension::FullVelar
        | PronounDeclension::ProximalSei
        | PronounDeclension::InterrogativeKii => cell.gender.is_some() && cell.person.is_none(),
    };
    if !valid_dimensions {
        return Err(Error::HistoricallyInvalidCell {
            reason: "pronoun cell gender/person dimensions contradict the lexical profile".into(),
        });
    }
    if cell.case == Case::Vocative {
        return Err(Error::HistoricallyInvalidCell {
            reason: "Alypy §§47–48 give no vocative in the pronoun paradigms".into(),
        });
    }
    if matches!(
        lexeme.declension,
        PronounDeclension::Reflexive
            | PronounDeclension::InterrogativeWho
            | PronounDeclension::InterrogativeWhat
    ) && cell.number != Number::Singular
    {
        return Err(Error::HistoricallyInvalidCell {
            reason: "this pronoun is licensed only in the singular table".into(),
        });
    }
    if lexeme.declension == PronounDeclension::Reflexive && cell.case == Case::Nominative {
        return Err(Error::HistoricallyInvalidCell {
            reason: "the reflexive pronoun has no nominative".into(),
        });
    }
    if uses_third_person_series(lexeme, cell)
        && lexeme.environment == PronounEnvironment::AfterPreposition
        && cell.case == Case::Nominative
    {
        return Err(Error::HistoricallyInvalidCell {
            reason: "a preposition cannot govern a nominative third-person pronoun".into(),
        });
    }
    if uses_third_person_series(lexeme, cell)
        && lexeme.environment == PronounEnvironment::Independent
        && cell.case == Case::Locative
    {
        return Err(Error::HistoricallyInvalidCell {
            reason: "the locative third-person pronoun is necessarily post-prepositional".into(),
        });
    }
    Ok(())
}

fn uses_third_person_series(lexeme: &PronounLexeme, cell: PronounCell) -> bool {
    lexeme.declension == PronounDeclension::ThirdPerson
        || (lexeme.declension == PronounDeclension::ThirdPersonAndDemonstrative
            && cell.person == Some(Person::Third))
}

fn rule_id(lexeme: &PronounLexeme, cell: PronounCell) -> &'static str {
    if lexeme.prefix.is_some() || lexeme.postpositive.is_some() {
        return "SYN-PRONOUN-DERIVED-ALYPY-46-48";
    }
    match lexeme.declension {
        PronounDeclension::PersonalFirst => "SYN-PRONOUN-PERSONAL-FIRST-ALYPY-47",
        PronounDeclension::PersonalSecond => "SYN-PRONOUN-PERSONAL-SECOND-ALYPY-47",
        PronounDeclension::Reflexive => "SYN-PRONOUN-REFLEXIVE-ALYPY-47",
        PronounDeclension::ThirdPerson => "SYN-PRONOUN-THIRD-PERSON-ALYPY-46-47",
        PronounDeclension::ThirdPersonAndDemonstrative => {
            if cell.person == Some(Person::Third) {
                "SYN-PRONOUN-THIRD-PERSON-ALYPY-46-47"
            } else {
                "SYN-PRONOUN-SHORT-HARD-ALYPY-48"
            }
        }
        PronounDeclension::ProximalSei => "SYN-PRONOUN-SEI-ALYPY-45-48",
        PronounDeclension::Soft => "SYN-PRONOUN-SOFT-ALYPY-47-48",
        PronounDeclension::SoftIAlternating => "SYN-PRONOUN-SOFT-I-ALTERNATING-ALYPY-45-48",
        PronounDeclension::Hard => "SYN-PRONOUN-HARD-ALYPY-47-48",
        PronounDeclension::MixedPossessive => "SYN-PRONOUN-MIXED-POSSESSIVE-ALYPY-48",
        PronounDeclension::ShortHard => "SYN-PRONOUN-SHORT-HARD-ALYPY-48",
        PronounDeclension::ShortOvMixed => "SYN-PRONOUN-SHORT-OV-MIXED-ALYPY-48",
        PronounDeclension::ShortVelar => "SYN-PRONOUN-SHORT-VELAR-ALYPY-48",
        PronounDeclension::QuantityVelar => "SYN-PRONOUN-QUANTITY-VELAR-ALYPY-48",
        PronounDeclension::FullHard => "SYN-PRONOUN-FULL-HARD-ALYPY-48-57",
        PronounDeclension::FullSoft => "SYN-PRONOUN-FULL-SOFT-ALYPY-48-57",
        PronounDeclension::FullVelar => "SYN-PRONOUN-FULL-VELAR-ALYPY-48-57",
        PronounDeclension::InterrogativeKii => "SYN-PRONOUN-KII-ALYPY-48",
        PronounDeclension::InterrogativeWho => "SYN-PRONOUN-KTO-ALYPY-48",
        PronounDeclension::InterrogativeWhat => "SYN-PRONOUN-CHTO-ALYPY-48",
    }
}

#[derive(Clone, Copy)]
struct PersonalVariant {
    text: &'static str,
    enclitic: bool,
}

const fn primary(text: &'static str) -> PersonalVariant {
    PersonalVariant {
        text,
        enclitic: false,
    }
}

const fn enclitic(text: &'static str) -> PersonalVariant {
    PersonalVariant {
        text,
        enclitic: true,
    }
}

fn personal_forms(
    first: bool,
    cell: PronounCell,
    selection: PronounFormSelection,
) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};

    let forms: &[PersonalVariant] = match (first, cell.number, cell.case) {
        (true, Sg, Nom) => &[primary("азъ")],
        (true, Sg, Gen) => &[primary("мене")],
        (true, Sg, Dat) => &[primary("мнѣ"), enclitic("ми")],
        (true, Sg, Acc) => &[primary("мене"), enclitic("мѧ")],
        (true, Sg, Ins) => &[primary("мною")],
        (true, Sg, Loc) => &[primary("мнѣ")],
        (true, Du, Nom) => &[primary("мы")],
        (true, Du, Acc) => &[enclitic("ны")],
        (true, Du, Gen | Loc) => &[primary("наю")],
        (true, Du, Dat | Ins) => &[primary("нама")],
        (true, Pl, Nom) => &[primary("мы")],
        (true, Pl, Gen | Loc) => &[primary("насъ")],
        (true, Pl, Dat) => &[primary("намъ")],
        (true, Pl, Acc) => &[enclitic("ны"), primary("насъ")],
        (true, Pl, Ins) => &[primary("нами")],

        (false, Sg, Nom) => &[primary("ты")],
        (false, Sg, Gen) => &[primary("тебе")],
        (false, Sg, Dat) => &[primary("тебѣ"), enclitic("ти")],
        (false, Sg, Acc) => &[primary("тебе"), enclitic("тѧ")],
        (false, Sg, Ins) => &[primary("тобою")],
        (false, Sg, Loc) => &[primary("тебѣ")],
        (false, Du, Nom) => &[primary("вы")],
        (false, Du, Acc) => &[enclitic("вы")],
        (false, Du, Gen | Loc) => &[primary("ваю")],
        (false, Du, Dat | Ins) => &[primary("вама")],
        (false, Pl, Nom) => &[primary("вы")],
        (false, Pl, Gen | Loc) => &[primary("васъ")],
        (false, Pl, Dat) => &[primary("вамъ")],
        (false, Pl, Acc) => &[enclitic("вы"), primary("васъ")],
        (false, Pl, Ins) => &[primary("вами")],
        (_, _, Case::Vocative) => unreachable!(),
    };
    select_personal(forms, selection)
}

fn reflexive_forms(cell: PronounCell, selection: PronounFormSelection) -> Result<Vec<String>> {
    let forms: &[PersonalVariant] = match cell.case {
        Case::Genitive => &[primary("себе")],
        Case::Dative => &[primary("себѣ"), enclitic("си")],
        Case::Accusative => &[primary("себе"), enclitic("сѧ")],
        Case::Instrumental => &[primary("собою")],
        Case::Locative => &[primary("себѣ")],
        Case::Nominative | Case::Vocative => unreachable!(),
    };
    select_personal(forms, selection)
}

fn select_personal(
    forms: &[PersonalVariant],
    selection: PronounFormSelection,
) -> Result<Vec<String>> {
    let selected = forms
        .iter()
        .filter(|variant| match selection {
            PronounFormSelection::All => true,
            PronounFormSelection::TablePrimary => !variant.enclitic,
            PronounFormSelection::Enclitic => variant.enclitic,
        })
        .map(|variant| variant.text.to_owned())
        .collect::<Vec<_>>();
    if selected.is_empty() {
        Err(Error::HistoricallyInvalidCell {
            reason: "the requested pronoun cell has no form in the selected clitic series".into(),
        })
    } else {
        Ok(selected)
    }
}

fn third_person_forms(
    cell: PronounCell,
    environment: PronounEnvironment,
    postpositive: Option<PronounPostpositive>,
) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};

    let gender = cell.gender.expect("validated third-person gender");
    if environment == PronounEnvironment::ContextualVariants {
        if cell.case == Nom {
            return third_person_forms(cell, PronounEnvironment::Independent, postpositive);
        }
        if cell.case == Loc {
            return third_person_forms(cell, PronounEnvironment::AfterPreposition, postpositive);
        }
        let mut forms = third_person_forms(cell, PronounEnvironment::Independent, postpositive)?;
        forms.extend(third_person_forms(
            cell,
            PronounEnvironment::AfterPreposition,
            postpositive,
        )?);
        forms.dedup();
        return Ok(forms);
    }
    if cell.case == Nom && postpositive == Some(PronounPostpositive::Zhe) {
        return Ok(match (cell.number, gender) {
            (Sg, M) => vec!["и".into()],
            (Sg, F) => vec!["ꙗ".into()],
            (Sg, N) => vec!["є".into()],
            (Du, M) => vec!["ѧ".into()],
            (Du, F | N) => vec!["и".into()],
            (Pl, M) => vec!["и".into()],
            (Pl, F | N) => vec!["ꙗ".into()],
        });
    }
    let after = environment == PronounEnvironment::AfterPreposition;
    let form = match (cell.number, gender, cell.case, cell.animacy, after) {
        (Sg, M, Nom, _, false) => "онъ",
        (Sg, F, Nom, _, false) => "она",
        (Sg, N, Nom, _, false) => "оно",
        (Sg, M | N, Gen, _, false) => "єгѡ",
        (Sg, M | N, Gen, _, true) => "негѡ",
        (Sg, F, Gen, _, false) => "єѧ",
        (Sg, F, Gen, _, true) => "неѧ",
        (Sg, M | N, Dat, _, false) => "ємꙋ",
        (Sg, M | N, Dat, _, true) => "немꙋ",
        (Sg, F, Dat, _, false) => "єй",
        (Sg, F, Dat, _, true) => "ней",
        (Sg, M, Acc, Animacy::Animate, false) => "єго",
        (Sg, M, Acc, Animacy::Animate, true) => "него",
        (Sg, M, Acc, Animacy::Inanimate, false) => "и",
        (Sg, M, Acc, Animacy::Inanimate, true) => "нь",
        (Sg, F, Acc, _, false) => "ю",
        (Sg, F, Acc, _, true) => "ню",
        (Sg, N, Acc, _, false) => "є",
        (Sg, N, Acc, _, true) => "не",
        (Sg, M | N, Ins, _, false) => "имъ",
        (Sg, M | N, Ins, _, true) => "нимъ",
        (Sg, F, Ins, _, false) => "єю",
        (Sg, F, Ins, _, true) => "нею",
        (Sg, M | N, Loc, _, true) => "немъ",
        (Sg, F, Loc, _, true) => "ней",

        (Du, M, Nom, _, false) => "она",
        (Du, F, Nom, _, false) => "онѣ",
        (Du, N, Nom, _, false) => "онѣ",
        (Du, _, Acc, _, false) => "ѧ",
        (Du, _, Acc, _, true) => "нѧ",
        (Du, _, Gen | Loc, _, false) => "єю",
        (Du, _, Gen | Loc, _, true) => "нею",
        (Du, _, Dat | Ins, _, false) => "има",
        (Du, _, Dat | Ins, _, true) => "нима",

        (Pl, M | N, Nom, _, false) => "они",
        (Pl, F, Nom, _, false) => "онѣ",
        (Pl, _, Gen | Loc, _, false) => "ихъ",
        (Pl, _, Gen | Loc, _, true) => "нихъ",
        (Pl, _, Dat, _, false) => "имъ",
        (Pl, _, Dat, _, true) => "нимъ",
        (Pl, M | F, Acc, Animacy::Animate, false) => "ихъ",
        (Pl, M | F, Acc, Animacy::Animate, true) => "нихъ",
        (Pl, _, Acc, _, false) => "ѧ",
        (Pl, _, Acc, _, true) => "нѧ",
        (Pl, _, Ins, _, false) => "ими",
        (Pl, _, Ins, _, true) => "ними",
        (_, _, Case::Vocative, _, _) => unreachable!(),
        (_, _, Nom, _, true) | (_, _, Loc, _, false) => unreachable!(),
    };
    let mut forms = vec![form.to_owned()];
    if cell.number == Du && gender == N && cell.case == Nom && !after {
        forms.push("она".into());
    }
    Ok(forms)
}

fn proximal_sei_forms(cell: PronounCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let gender = cell.gender.expect("validated agreeing gender");
    if cell.number == Sg
        && gender == M
        && (cell.case == Nom || (cell.case == Acc && cell.animacy == Animacy::Inanimate))
    {
        return Ok(vec!["сей".into(), "сій".into()]);
    }
    let form = match (cell.number, gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) | (Sg, M, Acc, Animacy::Inanimate) => unreachable!(),
        (Sg, F, Nom, _) => "сїѧ",
        (Sg, N, Nom, _) => "сїе",
        (Sg, M | N, Gen, _) => "сегѡ",
        (Sg, F, Gen, _) => "сеѧ",
        (Sg, M | N, Dat, _) => "семꙋ",
        (Sg, F, Dat | Loc, _) => "сей",
        (Sg, M, Acc, Animacy::Animate) => "сего",
        (Sg, F, Acc, _) => "сїю",
        (Sg, N, Acc, _) => "сїе",
        (Sg, M | N, Ins, _) => "симъ",
        (Sg, F, Ins, _) => "сею",
        (Sg, M | N, Loc, _) => "семъ",
        (Du, M, Nom | Acc, _) => "сїѧ",
        (Du, F | N, Nom | Acc, _) => "сіи",
        (Du, _, Gen | Loc, _) => "сею",
        (Du, _, Dat | Ins, _) => "сима",
        (Pl, M, Nom, _) => "сіи",
        (Pl, F | N, Nom, _) => "сїѧ",
        (Pl, _, Gen | Loc, _) => "сихъ",
        (Pl, _, Dat, _) => "симъ",
        (Pl, M | F, Acc, Animacy::Animate) => "сихъ",
        (Pl, M | F, Acc, Animacy::Inanimate) | (Pl, N, Acc, _) => "сїѧ",
        (Pl, _, Ins, _) => "сими",
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    Ok(vec![form.into()])
}

fn short_hard_forms(stem: &str, cell: PronounCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let gender = cell.gender.expect("validated agreeing gender");
    let ending = match (cell.number, gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => "ъ",
        (Sg, F, Nom, _) => "а",
        (Sg, N, Nom, _) => "о",
        (Sg, M | N, Gen, _) => "огѡ",
        (Sg, F, Gen, _) => "оѧ",
        (Sg, M | N, Dat, _) => "омꙋ",
        (Sg, F, Dat | Loc, _) => "ой",
        (Sg, M, Acc, Animacy::Inanimate) => "ъ",
        (Sg, M, Acc, Animacy::Animate) => "ого",
        (Sg, F, Acc, _) => "ꙋ",
        (Sg, N, Acc, _) => "о",
        (Sg, M | N, Ins, _) => "ѣмъ",
        (Sg, F, Ins, _) => "ою",
        (Sg, M | N, Loc, _) => "омъ",
        (Du, M, Nom | Acc, _) => "а",
        (Du, F | N, Nom | Acc, _) => "ѣ",
        (Du, _, Gen | Loc, _) => "ѡю",
        (Du, _, Dat | Ins, _) => "ѣма",
        (Pl, M, Nom, _) => "и",
        (Pl, F, Nom, _) => "ы",
        (Pl, N, Nom, _) => "а",
        (Pl, _, Gen | Loc, _) => "ѣхъ",
        (Pl, _, Dat, _) => "ѣмъ",
        (Pl, M | F, Acc, Animacy::Inanimate) => "ы",
        (Pl, M | F, Acc, Animacy::Animate) => "ѣхъ",
        (Pl, N, Acc, _) => "а",
        (Pl, _, Ins, _) => "ѣми",
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    Ok(vec![join(stem, ending)])
}

fn short_ov_mixed_forms(stem: &str, cell: PronounCell) -> Result<Vec<String>> {
    if !stem.ends_with("ов") {
        return Err(Error::ContradictoryMetadata {
            reason: "the mixed -ов- pronoun class requires a stem ending in -ов".into(),
        });
    }
    let mut forms = short_hard_forms(stem, cell)?;
    if cell.number == Number::Singular
        && matches!(cell.gender, Some(Gender::Masculine | Gender::Neuter))
    {
        let short = match cell.case {
            Case::Genitive => Some("а"),
            Case::Dative => Some("ꙋ"),
            _ => None,
        };
        if let Some(ending) = short {
            forms.insert(0, join(stem, ending));
        }
    }
    Ok(forms)
}

fn short_velar_forms(stem: &str, cell: PronounCell, quantity: bool) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let gender = cell.gender.expect("validated agreeing gender");
    let palatalized = palatalize_final_velar(stem)?;
    let raw = |ending| join(stem, ending);
    let soft = |ending| join(&palatalized, ending);
    let mut forms = match (cell.number, gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => vec![raw("ъ")],
        (Sg, F, Nom, _) => vec![raw("а")],
        (Sg, N, Nom, _) => vec![raw("о")],
        (Sg, M | N, Gen, _) => vec![raw("огѡ")],
        (Sg, F, Gen, _) => vec![raw("оѧ")],
        (Sg, M | N, Dat, _) => vec![raw("омꙋ")],
        (Sg, F, Dat | Loc, _) => vec![raw("ой"), soft("ѣй")],
        (Sg, M, Acc, Animacy::Inanimate) => vec![raw("ъ")],
        (Sg, M, Acc, Animacy::Animate) => vec![raw("ого")],
        (Sg, F, Acc, _) => vec![raw("ꙋ")],
        (Sg, N, Acc, _) => vec![raw("о")],
        (Sg, M | N, Ins, _) => vec![soft("ѣмъ")],
        (Sg, F, Ins, _) => vec![raw("ою")],
        (Sg, M | N, Loc, _) => vec![soft("ѣмъ"), raw("омъ")],
        (Du, M, Nom | Acc, _) => vec![raw("а")],
        (Du, F | N, Nom | Acc, _) => vec![soft("ѣ")],
        (Du, _, Gen | Loc, _) => vec![raw("ѡю")],
        (Du, _, Dat | Ins, _) => vec![soft("ѣма")],
        (Pl, M, Nom, _) => vec![soft("ы")],
        (Pl, F, Nom, _) => vec![raw("и")],
        (Pl, N, Nom, _) => vec![raw("а")],
        (Pl, _, Gen | Loc, _) => vec![soft("ѣхъ")],
        (Pl, _, Dat, _) => vec![soft("ѣмъ")],
        (Pl, M | F, Acc, Animacy::Inanimate) => vec![raw("и")],
        (Pl, M | F, Acc, Animacy::Animate) => vec![soft("ѣхъ")],
        (Pl, N, Acc, _) => vec![raw("а")],
        (Pl, _, Ins, _) => vec![soft("ѣми")],
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    if quantity && cell.number == Sg && matches!(gender, M | N) {
        match cell.case {
            Gen => forms.insert(0, raw("а")),
            Dat => forms.insert(0, raw("ꙋ")),
            Loc => forms.insert(0, soft("ѣ")),
            _ => {}
        }
    }
    Ok(forms)
}

fn full_forms(
    stem: &str,
    cell: PronounCell,
    class: AdjectiveClass,
    velar: bool,
    diaeresis_direct: bool,
) -> Result<Vec<String>> {
    if velar {
        return full_velar_forms(stem, cell, diaeresis_direct);
    }
    let adjective_cell = AdjectiveCell {
        case: cell.case,
        number: cell.number,
        gender: cell.gender.expect("validated agreeing gender"),
        animacy: cell.animacy,
        form: AdjectiveForm::Long,
        comparison: Comparison::Positive,
    };
    let ending = long_adjective_ending(class, adjective_cell)?;
    let mut forms = vec![join(stem, ending)];
    if class == AdjectiveClass::Hard && cell.number == Number::Singular {
        match (cell.gender, cell.case) {
            (Some(Gender::Feminine), Case::Dative | Case::Locative) => {
                forms.insert(0, join(stem, "ой"));
            }
            (Some(Gender::Masculine | Gender::Neuter), Case::Locative) => {
                forms.insert(0, join(stem, "омъ"));
            }
            _ => {}
        }
    }
    forms.dedup();
    Ok(forms)
}

fn full_velar_forms(stem: &str, cell: PronounCell, diaeresis_direct: bool) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let gender = cell.gender.expect("validated agreeing gender");
    let palatalized = palatalize_final_velar(stem)?;
    let raw = |ending| join(stem, ending);
    let soft = |ending| join(&palatalized, ending);
    let form = match (cell.number, gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => raw(if diaeresis_direct { "їй" } else { "ій" }),
        (Sg, F, Nom, _) => raw("аѧ"),
        (Sg, N, Nom, _) => raw("ое"),
        (Sg, M | N, Gen, _) => raw("агѡ"),
        (Sg, F, Gen, _) => raw(if diaeresis_direct { "їѧ" } else { "іѧ" }),
        (Sg, M | N, Dat, _) => raw("омꙋ"),
        (Sg, F, Dat | Loc, _) => soft("ѣй"),
        (Sg, M, Acc, Animacy::Inanimate) => raw(if diaeresis_direct { "їй" } else { "ій" }),
        (Sg, M, Acc, Animacy::Animate) => raw("аго"),
        (Sg, F, Acc, _) => raw("ꙋю"),
        (Sg, N, Acc, _) => raw("ое"),
        (Sg, M | N, Ins, _) => raw("имъ"),
        (Sg, F, Ins, _) => raw("ою"),
        (Sg, M | N, Loc, _) => soft("ѣмъ"),
        (Du, M, Nom | Acc, _) => raw("аѧ"),
        (Du, F | N, Nom | Acc, _) => soft("ѣи"),
        (Du, _, Gen | Loc, _) => raw("ꙋю"),
        (Du, _, Dat | Ins, _) => raw("има"),
        (Pl, M, Nom, _) => soft("іи"),
        (Pl, F, Nom, _) => raw(if diaeresis_direct { "їѧ" } else { "іѧ" }),
        (Pl, N, Nom, _) => raw("аѧ"),
        (Pl, _, Gen | Loc, _) => raw("ихъ"),
        (Pl, _, Dat, _) => raw("имъ"),
        (Pl, M, Acc, Animacy::Animate) => raw("ихъ"),
        (Pl, M | F, Acc, _) => raw(if diaeresis_direct { "їѧ" } else { "іѧ" }),
        (Pl, N, Acc, _) => raw("аѧ"),
        (Pl, _, Ins, _) => raw("ими"),
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    Ok(vec![form])
}

fn palatalize_final_velar(stem: &str) -> Result<String> {
    let (base, replacement) = match stem.chars().last() {
        Some('к') => (&stem[..stem.len() - 'к'.len_utf8()], 'ц'),
        Some('г') => (&stem[..stem.len() - 'г'.len_utf8()], 'з'),
        Some('х') => (&stem[..stem.len() - 'х'.len_utf8()], 'с'),
        _ => {
            return Err(Error::ContradictoryMetadata {
                reason: "a velar pronoun class requires a stem ending in к, г, or х".into(),
            });
        }
    };
    let mut result = String::with_capacity(stem.len());
    result.push_str(base);
    result.push(replacement);
    Ok(result)
}

fn soft_i_alternating_forms(stem: &str, cell: PronounCell) -> Result<Vec<String>> {
    let masculine_direct = cell.number == Number::Singular
        && cell.gender == Some(Gender::Masculine)
        && (cell.case == Case::Nominative
            || (cell.case == Case::Accusative && cell.animacy == Animacy::Inanimate));
    if masculine_direct {
        return soft_forms(stem, cell);
    }
    let base = stem
        .strip_suffix('і')
        .ok_or_else(|| Error::ContradictoryMetadata {
            reason: "the і/ї-alternating soft pronoun class requires a stem ending in і".into(),
        })?;
    let mut alternate = String::with_capacity(stem.len());
    alternate.push_str(base);
    alternate.push('ї');
    soft_forms(&alternate, cell)
}

fn soft_forms(stem: &str, cell: PronounCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let gender = cell.gender.expect("validated agreeing gender");
    let endings: &[&str] = match (cell.number, gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => &["й"],
        (Sg, F, Nom, _) => &["ѧ"],
        (Sg, N, Nom, _) => &["е"],
        (Sg, M | N, Gen, _) => &["егѡ"],
        (Sg, F, Gen, _) => &["еѧ"],
        (Sg, M | N, Dat, _) => &["емꙋ"],
        (Sg, F, Dat | Loc, _) => &["ей"],
        (Sg, M, Acc, Animacy::Inanimate) => &["й"],
        (Sg, M, Acc, Animacy::Animate) => &["его"],
        (Sg, F, Acc, _) => &["ю"],
        (Sg, N, Acc, _) => &["е"],
        (Sg, M | N, Ins, _) => &["имъ"],
        (Sg, F, Ins, _) => &["ею"],
        (Sg, M | N, Loc, _) => &["емъ"],
        (Du, M, Nom | Acc, _) => &["ѧ"],
        (Du, F | N, Nom | Acc, _) => &["и"],
        (Du, _, Gen | Loc, _) => &["єю"],
        (Du, _, Dat | Ins, _) => &["има"],
        (Pl, M, Nom, _) => &["и"],
        (Pl, F | N, Nom, _) => &["ѧ"],
        (Pl, _, Gen | Loc, _) => &["ихъ"],
        (Pl, _, Dat, _) => &["имъ"],
        (Pl, M | F, Acc, Animacy::Animate) => &["ихъ"],
        (Pl, M | F, Acc, Animacy::Inanimate) | (Pl, N, Acc, _) => &["ѧ"],
        (Pl, _, Ins, _) => &["ими"],
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    Ok(endings.iter().map(|ending| join(stem, ending)).collect())
}

fn hard_forms(stem: &str, cell: PronounCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let gender = cell.gender.expect("validated agreeing gender");
    let endings: &[&str] = match (cell.number, gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => &["ой"],
        (Sg, F, Nom, _) => &["аѧ", "а"],
        (Sg, N, Nom, _) => &["ое", "о"],
        (Sg, M | N, Gen, _) => &["огѡ"],
        (Sg, F, Gen, _) => &["оѧ"],
        (Sg, M | N, Dat, _) => &["омꙋ"],
        (Sg, F, Dat | Loc, _) => &["ой"],
        (Sg, M, Acc, Animacy::Inanimate) => &["ой"],
        (Sg, M, Acc, Animacy::Animate) => &["ого"],
        (Sg, F, Acc, _) => &["ꙋ", "ꙋю"],
        (Sg, N, Acc, _) => &["ое", "о"],
        (Sg, M | N, Ins, _) => &["ѣмъ"],
        (Sg, F, Ins, _) => &["ою"],
        (Sg, M | N, Loc, _) => &["омъ"],
        (Du, M, Nom | Acc, _) => &["а"],
        (Du, F, Nom | Acc, _) => &["ѣ"],
        (Du, N, Nom | Acc, _) => &["ѣ", "а"],
        (Du, _, Gen | Loc, _) => &["ѡю"],
        (Du, _, Dat | Ins, _) => &["ѣма"],
        (Pl, M, Nom, _) => &["іи", "и"],
        (Pl, F, Nom, _) => &["ыѧ", "ы"],
        (Pl, N, Nom, _) => &["аѧ", "а"],
        (Pl, _, Gen | Loc, _) => &["ѣхъ"],
        (Pl, _, Dat, _) => &["ѣмъ"],
        (Pl, M | F, Acc, Animacy::Animate) => &["ѣхъ"],
        (Pl, M, Acc, Animacy::Inanimate) => &["ыѧ"],
        (Pl, F, Acc, Animacy::Inanimate) => &["ыѧ", "ы"],
        (Pl, N, Acc, _) => &["аѧ", "а"],
        (Pl, _, Ins, _) => &["ѣми"],
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    Ok(endings.iter().map(|ending| join(stem, ending)).collect())
}

fn mixed_forms(stem: &str, cell: PronounCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let gender = cell.gender.expect("validated agreeing gender");
    let ending = match (cell.number, gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => "ъ",
        (Sg, F, Nom, _) => "а",
        (Sg, N, Nom, _) => "е",
        (Sg, M | N, Gen, _) => "егѡ",
        (Sg, F, Gen, _) => "еѧ",
        (Sg, M | N, Dat, _) => "емꙋ",
        (Sg, F, Dat | Loc, _) => "ей",
        (Sg, M, Acc, Animacy::Inanimate) => "ъ",
        (Sg, M, Acc, Animacy::Animate) => "его",
        (Sg, F, Acc, _) => "ꙋ",
        (Sg, N, Acc, _) => "е",
        (Sg, M | N, Ins, _) => "имъ",
        (Sg, F, Ins, _) => "ею",
        (Sg, M | N, Loc, _) => "емъ",
        (Du, M, Nom | Acc, _) => "а",
        (Du, F | N, Nom | Acc, _) => "и",
        (Du, _, Gen | Loc, _) => "ею",
        (Du, _, Dat | Ins, _) => "има",
        (Pl, M, Nom, _) => "и",
        (Pl, F, Nom, _) => "ѧ",
        (Pl, N, Nom, _) => "а",
        (Pl, _, Gen | Loc, _) => "ихъ",
        (Pl, _, Dat, _) => "ымъ",
        (Pl, M | F, Acc, Animacy::Animate) => "ихъ",
        (Pl, M | F, Acc, Animacy::Inanimate) => "ѧ",
        (Pl, N, Acc, _) => "а",
        (Pl, _, Ins, _) => "ими",
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    Ok(vec![join(stem, ending)])
}

fn interrogative_kii(cell: PronounCell) -> Result<Vec<String>> {
    use Case::{Accusative as Acc, Dative as Dat, Genitive as Gen, Instrumental as Ins};
    use Case::{Locative as Loc, Nominative as Nom};
    use Gender::{Feminine as F, Masculine as M, Neuter as N};
    use Number::{Dual as Du, Plural as Pl, Singular as Sg};
    let gender = cell.gender.expect("validated agreeing gender");
    let form = match (cell.number, gender, cell.case, cell.animacy) {
        (Sg, M, Nom, _) => "кій",
        (Sg, F, Nom, _) => "каѧ",
        (Sg, N, Nom, _) => "кое",
        (Sg, M | N, Gen, _) => "коегѡ",
        (Sg, F, Gen, _) => "коеѧ",
        (Sg, M | N, Dat, _) => "коемꙋ",
        (Sg, F, Dat | Loc, _) => "коей",
        (Sg, M, Acc, Animacy::Inanimate) => "кій",
        (Sg, M, Acc, Animacy::Animate) => "коего",
        (Sg, F, Acc, _) => "кꙋю",
        (Sg, N, Acc, _) => "кое",
        (Sg, M | N, Ins, _) => "кіимъ",
        (Sg, F, Ins, _) => "коею",
        (Sg, M | N, Loc, _) => "коемъ",
        (Du, M, Nom | Acc, _) => "каѧ",
        (Du, F | N, Nom | Acc, _) => "кіи",
        (Du, _, Gen | Loc, _) => "коєю",
        (Du, _, Dat | Ins, _) => "кіима",
        (Pl, M, Nom, _) => "кіи",
        (Pl, F, Nom, _) => "кіѧ",
        (Pl, N, Nom, _) => "каѧ",
        (Pl, _, Gen | Loc, _) => "кіихъ",
        (Pl, _, Dat, _) => "кіимъ",
        (Pl, M | F, Acc, Animacy::Inanimate) => "кіѧ",
        (Pl, M | F, Acc, Animacy::Animate) => "кіихъ",
        (Pl, N, Acc, _) => "каѧ",
        (Pl, _, Ins, _) => "кіими",
        (_, _, Case::Vocative, _) => unreachable!(),
    };
    Ok(vec![form.into()])
}

fn interrogative_who(cell: PronounCell) -> Result<Vec<String>> {
    Ok(vec![
        match cell.case {
            Case::Nominative => "кто",
            Case::Genitive => "когѡ",
            Case::Dative => "комꙋ",
            Case::Accusative => "кого",
            Case::Instrumental => "кимъ",
            Case::Locative => "комъ",
            Case::Vocative => unreachable!(),
        }
        .into(),
    ])
}

fn interrogative_what(cell: PronounCell) -> Result<Vec<String>> {
    Ok(match cell.case {
        Case::Nominative => vec!["что".into()],
        Case::Genitive => vec!["чегѡ".into(), "чесѡ".into(), "чесогѡ".into()],
        Case::Dative => vec!["чемꙋ".into(), "чесомꙋ".into()],
        Case::Accusative => vec!["что".into(), "чесо".into()],
        Case::Instrumental => vec!["чимъ".into()],
        Case::Locative => vec!["чемъ".into(), "чесомъ".into()],
        Case::Vocative => unreachable!(),
    })
}

fn compose(
    forms: Vec<String>,
    prefix: Option<PronounPrefix>,
    postpositive: Option<PronounPostpositive>,
    declension: PronounDeclension,
    cell: PronounCell,
) -> Vec<String> {
    let mut composed = forms
        .into_iter()
        .map(|form| {
            let mut result = String::new();
            if let Some(prefix) = prefix {
                result.push_str(prefix.text());
            }
            result.push_str(&form);
            if declension != PronounDeclension::ThirdPerson {
                if let Some(postpositive) = postpositive {
                    result.push_str(postpositive.text());
                }
            } else if postpositive == Some(PronounPostpositive::Zhe) {
                result.push_str("же");
            }
            result
        })
        .collect::<Vec<_>>();
    if prefix == Some(PronounPrefix::IndefiniteNe)
        && declension == PronounDeclension::InterrogativeKii
        && cell.number == Number::Plural
    {
        let fused = match cell.case {
            Case::Genitive | Case::Locative => Some("нѣкихъ"),
            Case::Dative => Some("нѣкимъ"),
            _ => None,
        };
        if let Some(fused) = fused {
            composed.push(fused.into());
        }
    }
    composed
}

fn join(stem: &str, ending: &str) -> String {
    let mut form = String::with_capacity(stem.len() + ending.len());
    form.push_str(stem);
    form.push_str(ending);
    form
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(text: &str) -> SynodalWord {
        SynodalWord::parse(text).expect("valid Synodal test word")
    }

    fn cell(
        case: Case,
        number: Number,
        gender: Option<Gender>,
        person: Option<Person>,
        animacy: Animacy,
    ) -> PronounCell {
        PronounCell {
            case,
            number,
            gender,
            person,
            animacy,
        }
    }

    #[test]
    fn alpy_47_personal_and_reflexive_tables_keep_typed_clitics() {
        let first = PronounLexeme::closed(word("азъ"), PronounDeclension::PersonalFirst);
        let forms = decline_pronoun(
            &first,
            cell(
                Case::Accusative,
                Number::Plural,
                None,
                Some(Person::First),
                Animacy::Inanimate,
            ),
            OrthographyProfile::Expanded,
        )
        .expect("first plural accusative");
        assert_eq!(forms.texts().collect::<Vec<_>>(), ["ны", "насъ"]);

        let clitic = first.with_selection(PronounFormSelection::Enclitic);
        assert_eq!(
            decline_pronoun(
                &clitic,
                cell(
                    Case::Dative,
                    Number::Singular,
                    None,
                    Some(Person::First),
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("first singular dative clitic")
            .primary_text(),
            "ми"
        );

        let reflexive = PronounLexeme::closed(word("себе"), PronounDeclension::Reflexive);
        assert_eq!(
            decline_pronoun(
                &reflexive,
                cell(
                    Case::Accusative,
                    Number::Singular,
                    None,
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("reflexive accusative")
            .texts()
            .collect::<Vec<_>>(),
            ["себе", "сѧ"]
        );
    }

    #[test]
    fn alpy_47_third_person_and_relative_are_environment_typed() {
        let independent = PronounLexeme::closed(word("онъ"), PronounDeclension::ThirdPerson);
        let genitive = cell(
            Case::Genitive,
            Number::Singular,
            Some(Gender::Masculine),
            Some(Person::Third),
            Animacy::Animate,
        );
        assert_eq!(
            decline_pronoun(&independent, genitive, OrthographyProfile::Expanded)
                .expect("independent genitive")
                .primary_text(),
            "єгѡ"
        );
        let governed = independent
            .clone()
            .with_environment(PronounEnvironment::AfterPreposition);
        assert_eq!(
            decline_pronoun(&governed, genitive, OrthographyProfile::Expanded)
                .expect("governed genitive")
                .primary_text(),
            "негѡ"
        );
        let relative = independent.with_postpositive(PronounPostpositive::Zhe);
        assert_eq!(
            decline_pronoun(
                &relative,
                cell(
                    Case::Nominative,
                    Number::Dual,
                    Some(Gender::Feminine),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("relative nominative dual")
            .primary_text(),
            "иже"
        );
    }

    #[test]
    fn alpy_45_48_on_has_one_identity_and_two_typed_profiles() {
        let on = PronounLexeme::regular(
            word("онъ"),
            word("он"),
            PronounDeclension::ThirdPersonAndDemonstrative,
        )
        .with_environment(PronounEnvironment::ContextualVariants);

        let mut successes = 0;
        let mut invalid = 0;
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    for animacy in Animacy::ALL {
                        for person in [None, Some(Person::Third)] {
                            match decline_pronoun(
                                &on,
                                cell(case, number, Some(gender), person, animacy),
                                OrthographyProfile::Expanded,
                            ) {
                                Ok(forms) => {
                                    assert!(!forms.variants().is_empty());
                                    successes += 1;
                                }
                                Err(Error::HistoricallyInvalidCell { .. }) => invalid += 1,
                                outcome => panic!(
                                    "unexpected combined онъ outcome for {number:?}/{case:?}/{gender:?}/{animacy:?}/{person:?}: {outcome:?}"
                                ),
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(successes, 216);
        assert_eq!(invalid, 36);

        let third_person = cell(
            Case::Genitive,
            Number::Singular,
            Some(Gender::Masculine),
            Some(Person::Third),
            Animacy::Animate,
        );
        assert_eq!(
            decline_pronoun(&on, third_person, OrthographyProfile::Expanded)
                .expect("third-person reading")
                .texts()
                .collect::<Vec<_>>(),
            ["єгѡ", "негѡ"]
        );

        let demonstrative = PronounCell {
            person: None,
            ..third_person
        };
        let form = decline_pronoun(&on, demonstrative, OrthographyProfile::Expanded)
            .expect("demonstrative reading");
        assert_eq!(form.primary_text(), "оногѡ");
        assert_eq!(
            form.primary().rule_trace.steps()[0].rule.as_str(),
            "SYN-PRONOUN-SHORT-HARD-ALYPY-48"
        );
    }

    #[test]
    fn alpy_47_48_regular_pronominal_tables_are_complete() {
        let lexemes = [
            PronounLexeme::regular(word("мой"), word("мо"), PronounDeclension::Soft),
            PronounLexeme::regular(word("той"), word("т"), PronounDeclension::Hard),
            PronounLexeme::regular(
                word("нашъ"),
                word("наш"),
                PronounDeclension::MixedPossessive,
            ),
        ];
        for lexeme in &lexemes {
            let mut successes = 0;
            let mut invalid = 0;
            for number in Number::ALL {
                for case in Case::ALL {
                    for gender in Gender::ALL {
                        for animacy in Animacy::ALL {
                            match decline_pronoun(
                                lexeme,
                                cell(case, number, Some(gender), None, animacy),
                                OrthographyProfile::Expanded,
                            ) {
                                Ok(forms) => {
                                    assert!(!forms.variants().is_empty());
                                    successes += 1;
                                }
                                Err(Error::HistoricallyInvalidCell { .. }) => invalid += 1,
                                outcome => {
                                    panic!("unexpected regular pronoun outcome: {outcome:?}")
                                }
                            }
                        }
                    }
                }
            }
            assert_eq!(successes, 108, "{:?}", lexeme.declension);
            assert_eq!(invalid, 18, "{:?}", lexeme.declension);
        }
    }

    #[test]
    fn alpy_48_interrogative_derivation_is_singular_and_source_compositional() {
        let negative = PronounLexeme::closed(word("никтоже"), PronounDeclension::InterrogativeWho)
            .with_prefix(PronounPrefix::NegativeNi)
            .with_postpositive(PronounPostpositive::Zhe);
        assert_eq!(
            decline_pronoun(
                &negative,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    None,
                    None,
                    Animacy::Animate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("negative genitive")
            .primary_text(),
            "никогѡже"
        );
        assert!(matches!(
            decline_pronoun(
                &negative,
                cell(Case::Genitive, Number::Plural, None, None, Animacy::Animate,),
                OrthographyProfile::Expanded,
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let impossible_inventory =
            PronounLexeme::closed(word("кто"), PronounDeclension::InterrogativeWho)
                .with_number_inventory(PronounNumberInventory::PluralOnly);
        assert!(matches!(
            validate_pronoun_lexeme(&impossible_inventory),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let impossible_affixes =
            PronounLexeme::closed(word("нѣкійждо"), PronounDeclension::InterrogativeKii)
                .with_prefix(PronounPrefix::IndefiniteNe)
                .with_postpositive(PronounPostpositive::Zhdo);
        assert!(matches!(
            validate_pronoun_lexeme(&impossible_affixes),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let what = PronounLexeme::closed(word("что"), PronounDeclension::InterrogativeWhat);
        assert_eq!(
            decline_pronoun(
                &what,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    None,
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("what genitive variants")
            .texts()
            .collect::<Vec<_>>(),
            ["чегѡ", "чесѡ", "чесогѡ"]
        );
    }

    #[test]
    fn alpy_48_two_base_kii_table_and_derived_particles_are_complete() {
        let kii = PronounLexeme::closed(word("кій"), PronounDeclension::InterrogativeKii);
        let mut successes = 0;
        let mut invalid = 0;
        for number in Number::ALL {
            for case in Case::ALL {
                for gender in Gender::ALL {
                    for animacy in Animacy::ALL {
                        match decline_pronoun(
                            &kii,
                            cell(case, number, Some(gender), None, animacy),
                            OrthographyProfile::Expanded,
                        ) {
                            Ok(forms) => {
                                assert!(!forms.variants().is_empty());
                                successes += 1;
                            }
                            Err(Error::HistoricallyInvalidCell { .. }) => invalid += 1,
                            outcome => panic!("unexpected кій outcome: {outcome:?}"),
                        }
                    }
                }
            }
        }
        assert_eq!(successes, 108);
        assert_eq!(invalid, 18);

        assert_eq!(
            decline_pronoun(
                &kii,
                cell(
                    Case::Accusative,
                    Number::Singular,
                    Some(Gender::Masculine),
                    None,
                    Animacy::Animate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("animate masculine accusative")
            .primary_text(),
            "коего"
        );
        let distributive = kii.clone().with_postpositive(PronounPostpositive::Zhdo);
        assert_eq!(
            decline_pronoun(
                &distributive,
                cell(
                    Case::Dative,
                    Number::Dual,
                    Some(Gender::Neuter),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("derived dual dative")
            .primary_text(),
            "кіимаждо"
        );

        let indefinite = kii.with_prefix(PronounPrefix::IndefiniteNe);
        assert_eq!(
            decline_pronoun(
                &indefinite,
                cell(
                    Case::Genitive,
                    Number::Plural,
                    Some(Gender::Feminine),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("indefinite fused plural")
            .texts()
            .collect::<Vec<_>>(),
            ["нѣкіихъ", "нѣкихъ"]
        );
    }

    #[test]
    fn alpy_45_48_sei_short_and_full_pronouns_cover_every_agreement_cell() {
        let lexemes = [
            PronounLexeme::closed(word("сей"), PronounDeclension::ProximalSei),
            PronounLexeme::regular(word("чій"), word("чі"), PronounDeclension::SoftIAlternating),
            PronounLexeme::regular(word("овъ"), word("ов"), PronounDeclension::ShortHard),
            PronounLexeme::regular(word("овый"), word("ов"), PronounDeclension::FullHard),
            PronounLexeme::regular(word("синїй"), word("син"), PronounDeclension::FullSoft),
            PronounLexeme::regular(word("благій"), word("благ"), PronounDeclension::FullVelar),
            PronounLexeme::regular(word("ꙗковъ"), word("ꙗков"), PronounDeclension::ShortOvMixed),
        ];
        for lexeme in &lexemes {
            let mut successes = 0;
            let mut invalid = 0;
            for number in Number::ALL {
                for case in Case::ALL {
                    for gender in Gender::ALL {
                        for animacy in Animacy::ALL {
                            match decline_pronoun(
                                lexeme,
                                cell(case, number, Some(gender), None, animacy),
                                OrthographyProfile::Expanded,
                            ) {
                                Ok(forms) => {
                                    assert!(!forms.variants().is_empty());
                                    successes += 1;
                                }
                                Err(Error::HistoricallyInvalidCell { .. }) => invalid += 1,
                                outcome => panic!(
                                    "unexpected agreeing pronoun outcome for {:?}: {outcome:?}",
                                    lexeme.declension
                                ),
                            }
                        }
                    }
                }
            }
            assert_eq!(successes, 108, "{:?}", lexeme.declension);
            assert_eq!(invalid, 18, "{:?}", lexeme.declension);
        }

        let sei = &lexemes[0];
        assert_eq!(
            decline_pronoun(
                sei,
                cell(
                    Case::Accusative,
                    Number::Plural,
                    Some(Gender::Feminine),
                    None,
                    Animacy::Animate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("сей feminine animate plural accusative")
            .primary_text(),
            "сихъ"
        );

        assert_eq!(
            decline_pronoun(
                &lexemes[1],
                cell(
                    Case::Genitive,
                    Number::Singular,
                    Some(Gender::Neuter),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("чій vowel-edge alternation")
            .primary_text(),
            "чїегѡ"
        );

        let short = &lexemes[2];
        assert_eq!(
            decline_pronoun(
                short,
                cell(
                    Case::Instrumental,
                    Number::Singular,
                    Some(Gender::Neuter),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("short instrumental")
            .primary_text(),
            "овѣмъ"
        );

        let full = &lexemes[3];
        assert_eq!(
            decline_pronoun(
                full,
                cell(
                    Case::Locative,
                    Number::Singular,
                    Some(Gender::Masculine),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("full locative variants")
            .texts()
            .collect::<Vec<_>>(),
            ["овомъ", "овѣмъ"]
        );
    }

    #[test]
    fn alpy_48_velar_and_quantity_pronouns_preserve_palatalized_variants() {
        let vsyak =
            PronounLexeme::regular(word("всѧкъ"), word("всѧк"), PronounDeclension::ShortVelar)
                .with_number_inventory(PronounNumberInventory::SingularAndPlural);
        assert_eq!(
            decline_pronoun(
                &vsyak,
                cell(
                    Case::Locative,
                    Number::Singular,
                    Some(Gender::Masculine),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("velar locative variants")
            .texts()
            .collect::<Vec<_>>(),
            ["всѧцѣмъ", "всѧкомъ"]
        );
        assert_eq!(
            decline_pronoun(
                &vsyak,
                cell(
                    Case::Dative,
                    Number::Singular,
                    Some(Gender::Feminine),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("velar feminine variants")
            .texts()
            .collect::<Vec<_>>(),
            ["всѧкой", "всѧцѣй"]
        );
        assert!(matches!(
            decline_pronoun(
                &vsyak,
                cell(
                    Case::Nominative,
                    Number::Dual,
                    Some(Gender::Masculine),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            ),
            Err(Error::HistoricallyInvalidCell { .. })
        ));

        let tolik = PronounLexeme::regular(
            word("толикъ"),
            word("толик"),
            PronounDeclension::QuantityVelar,
        );
        assert_eq!(
            decline_pronoun(
                &tolik,
                cell(
                    Case::Genitive,
                    Number::Singular,
                    Some(Gender::Neuter),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("quantity nominal/pronominal genitives")
            .texts()
            .collect::<Vec<_>>(),
            ["толика", "толикогѡ"]
        );
        assert_eq!(
            decline_pronoun(
                &tolik,
                cell(
                    Case::Nominative,
                    Number::Plural,
                    Some(Gender::Masculine),
                    None,
                    Animacy::Inanimate,
                ),
                OrthographyProfile::Expanded,
            )
            .expect("quantity masculine plural")
            .primary_text(),
            "толицы"
        );
    }

    #[test]
    fn full_velar_diaeresis_is_preserved_in_feminine_iya_endings() {
        let lexeme =
            PronounLexeme::regular(word("єликїй"), word("єлик"), PronounDeclension::FullVelar);
        for (number, case, expected) in [
            (Number::Singular, Case::Genitive, "єликїѧ"),
            (Number::Plural, Case::Nominative, "єликїѧ"),
            (Number::Plural, Case::Accusative, "єликїѧ"),
        ] {
            let forms = decline_pronoun(
                &lexeme,
                PronounCell {
                    case,
                    number,
                    gender: Some(Gender::Feminine),
                    person: None,
                    animacy: Animacy::Inanimate,
                },
                OrthographyProfile::Expanded,
            )
            .expect("full velar cell");
            assert_eq!(forms.primary_text(), expected);
        }
    }

    #[test]
    fn compound_ov_pronouns_preserve_noun_like_genitive_and_dative_first() {
        let lexeme =
            PronounLexeme::regular(word("ꙗковъ"), word("ꙗков"), PronounDeclension::ShortOvMixed);
        let cell = |case| {
            cell(
                case,
                Number::Singular,
                Some(Gender::Masculine),
                None,
                Animacy::Inanimate,
            )
        };
        assert_eq!(
            decline_pronoun(&lexeme, cell(Case::Genitive), OrthographyProfile::Expanded)
                .expect("mixed genitive")
                .texts()
                .collect::<Vec<_>>(),
            ["ꙗкова", "ꙗковогѡ"]
        );
        assert_eq!(
            decline_pronoun(&lexeme, cell(Case::Dative), OrthographyProfile::Expanded)
                .expect("mixed dative")
                .texts()
                .collect::<Vec<_>>(),
            ["ꙗковꙋ", "ꙗковомꙋ"]
        );

        let invalid =
            PronounLexeme::regular(word("инъ"), word("ин"), PronounDeclension::ShortOvMixed);
        assert!(matches!(
            validate_pronoun_lexeme(&invalid),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}
