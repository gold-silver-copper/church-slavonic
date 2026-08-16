//! Typed, source-backed accent realization for productively generated forms.

use unicode_normalization::{UnicodeNormalization, char::canonical_combining_class};

use crate::{
    AdjectiveForm, AuthorityRole, Case, Comparison, Error, Evidence, EvidenceKind, FiniteTense,
    Gender, GrammarCell, MetadataField, Number, Recension, Result, SynodalWord,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AccentMark {
    Acute,
    Grave,
    Kamora,
}

impl AccentMark {
    const fn character(self) -> char {
        match self {
            Self::Acute => '\u{0301}',
            Self::Grave => '\u{0300}',
            Self::Kamora => '\u{0311}',
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BreathingMark {
    Psili,
}

impl BreathingMark {
    const fn character(self) -> char {
        match self {
            Self::Psili => '\u{0486}',
        }
    }
}

/// The three invariant postpositives whose presence conditions a preceding
/// word-final accent in Alypy §3.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum EncliticParticle {
    Zhe,
    Bo,
    Li,
}

impl EncliticParticle {
    pub const ALL: [Self; 3] = [Self::Zhe, Self::Bo, Self::Li];
}

/// The closed set of postpositive environments that change a word-final
/// varia to an acute in Alypy §3. Short personal/reflexive pronouns remain a
/// separate kind because their lexical identity and cell must be validated by
/// the phrase layer before this presentation context is selected.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AccentEnclitic {
    Particle(EncliticParticle),
    ShortPersonalOrReflexivePronoun,
}

impl AccentEnclitic {
    pub const ALL: [Self; 4] = [
        Self::Particle(EncliticParticle::Zhe),
        Self::Particle(EncliticParticle::Bo),
        Self::Particle(EncliticParticle::Li),
        Self::ShortPersonalOrReflexivePronoun,
    ];
}

/// Syntactic environment used only for surface realization of an already
/// selected lexical stress position.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AccentEnvironment {
    #[default]
    Standalone,
    BeforeEnclitic(AccentEnclitic),
}

/// A vowel position in the generated expanded form. Stem positions count from
/// the lexical left edge; ending positions count from the word's right edge.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AccentPlacement {
    StemVowelFromStart(u8),
    EndingVowelFromEnd(u8),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum AccentScope {
    All,
    Noun {
        numbers: Vec<Number>,
    },
    /// A reusable noun rule restricted by both number and case. This permits
    /// genuine cell-conditioned mobility without storing accented strings.
    NounCases {
        numbers: Vec<Number>,
        cases: Vec<Case>,
    },
    /// A reusable pronoun rule restricted by grammatical number and case.
    /// Person, gender, and animacy remain part of the validated cell but do
    /// not need to be duplicated when a source table gives one stress pattern.
    PronounCases {
        numbers: Vec<Number>,
        cases: Vec<Case>,
    },
    /// A pronoun rule whose stress also depends on agreement gender or the
    /// accusative animacy contrast.
    PronounAgreement {
        numbers: Vec<Number>,
        cases: Vec<Case>,
        genders: Vec<Gender>,
        animacies: Vec<crate::Animacy>,
    },
    Adjective {
        form: AdjectiveForm,
        comparison: Comparison,
        numbers: Vec<Number>,
    },
    FiniteVerb {
        tense: FiniteTense,
        numbers: Vec<Number>,
    },
    OtherCells(Vec<GrammarCell>),
}

impl AccentScope {
    #[must_use]
    pub fn applies_to(&self, cell: GrammarCell) -> bool {
        match (self, cell) {
            (Self::All, _) => true,
            (Self::Noun { numbers }, GrammarCell::Noun(cell)) => numbers.contains(&cell.number),
            (Self::NounCases { numbers, cases }, GrammarCell::Noun(cell)) => {
                numbers.contains(&cell.number) && cases.contains(&cell.case)
            }
            (Self::PronounCases { numbers, cases }, GrammarCell::Pronoun(cell)) => {
                numbers.contains(&cell.number) && cases.contains(&cell.case)
            }
            (
                Self::PronounAgreement {
                    numbers,
                    cases,
                    genders,
                    animacies,
                },
                GrammarCell::Pronoun(cell),
            ) => {
                numbers.contains(&cell.number)
                    && cases.contains(&cell.case)
                    && cell.gender.is_some_and(|gender| genders.contains(&gender))
                    && animacies.contains(&cell.animacy)
            }
            (
                Self::Adjective {
                    form,
                    comparison,
                    numbers,
                },
                GrammarCell::Adjective(cell) | GrammarCell::Determiner(cell),
            ) => {
                cell.form == *form
                    && cell.comparison == *comparison
                    && numbers.contains(&cell.number)
            }
            (Self::FiniteVerb { tense, numbers }, GrammarCell::FiniteVerb(cell)) => {
                cell.tense == *tense && numbers.contains(&cell.number)
            }
            (Self::OtherCells(cells), cell) => cells.contains(&cell),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AccentRule {
    pub scope: AccentScope,
    pub placement: AccentPlacement,
    pub mark: AccentMark,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BreathingRule {
    pub scope: AccentScope,
    pub placement: AccentPlacement,
    pub mark: BreathingMark,
}

/// A reviewed lexical accent contract. Several scoped rules can represent
/// number- or cell-conditioned mobility without storing one accented string
/// for every inflected cell.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct AccentParadigm {
    pub id: String,
    pub accent_rules: Vec<AccentRule>,
    pub breathing_rules: Vec<BreathingRule>,
    pub evidence: Evidence,
}

impl AccentParadigm {
    #[must_use]
    pub fn fixed_stem(
        id: impl Into<String>,
        scope: AccentScope,
        vowel_from_start: u8,
        mark: AccentMark,
        evidence: Evidence,
    ) -> Self {
        Self {
            id: id.into(),
            accent_rules: vec![AccentRule {
                scope,
                placement: AccentPlacement::StemVowelFromStart(vowel_from_start),
                mark,
            }],
            breathing_rules: vec![],
            evidence,
        }
    }

    #[must_use]
    pub fn fixed_ending(
        id: impl Into<String>,
        scope: AccentScope,
        vowel_from_end: u8,
        mark: AccentMark,
        evidence: Evidence,
    ) -> Self {
        Self {
            id: id.into(),
            accent_rules: vec![AccentRule {
                scope,
                placement: AccentPlacement::EndingVowelFromEnd(vowel_from_end),
                mark,
            }],
            breathing_rules: vec![],
            evidence,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() || self.accent_rules.is_empty() {
            return Err(Error::ContradictoryMetadata {
                reason: "an accent paradigm requires a stable ID and at least one accent rule"
                    .into(),
            });
        }
        if self.evidence.id.as_str().trim().is_empty()
            || self.evidence.source.as_str().trim().is_empty()
            || self.evidence.citation.trim().is_empty()
            || self.evidence.source_recension != Recension::SynodalRussian
            || self.evidence.kind != EvidenceKind::AccentParadigm
            || !self
                .evidence
                .authority_roles
                .contains(&AuthorityRole::Accentual)
        {
            return Err(Error::ContradictoryMetadata {
                reason: "an accent paradigm requires nonempty Synodal accentual evidence".into(),
            });
        }
        if self
            .accent_rules
            .iter()
            .any(|rule| scope_is_empty(&rule.scope))
            || self
                .breathing_rules
                .iter()
                .any(|rule| scope_is_empty(&rule.scope))
        {
            return Err(Error::ContradictoryMetadata {
                reason: "an accent or breathing rule cannot have an empty cell scope".into(),
            });
        }
        Ok(())
    }

    /// Applies the unique accent rule licensed for one isolated word. A
    /// missing or overlapping rule is a typed metadata failure. Alypy §3's
    /// language-wide initial-vowel psili and final-vowel varia are surface
    /// rules; callers do not have to repeat them in every lexical paradigm.
    pub fn apply(&self, cell: GrammarCell, expanded: &str) -> Result<String> {
        self.apply_in_environment(cell, expanded, AccentEnvironment::Standalone)
    }

    /// Applies lexical stress in an explicit syntactic environment. Before
    /// one of Alypy §3's closed enclitic classes, a final stressed vowel keeps
    /// an acute; in isolation the same position receives a grave. Nonfinal
    /// stress and exceptional lexical grave/kamora marks are preserved.
    pub fn apply_in_environment(
        &self,
        cell: GrammarCell,
        expanded: &str,
        environment: AccentEnvironment,
    ) -> Result<String> {
        self.validate()?;
        if contains_accent_or_breathing_mark(expanded) {
            return Err(Error::ContradictoryMetadata {
                reason: "an accent paradigm requires an unaccented, unbreathed expanded form"
                    .into(),
            });
        }
        let accent = unique_rule(
            self.accent_rules
                .iter()
                .filter(|rule| rule.scope.applies_to(cell)),
            "accent",
        )?
        .ok_or(Error::OrthographicMetadataRequired {
            field: MetadataField::AccentParadigm,
        })?;
        let breathing = unique_rule(
            self.breathing_rules
                .iter()
                .filter(|rule| rule.scope.applies_to(cell)),
            "breathing",
        )?;

        let accent_index = vowel_index(expanded, accent.placement)?;
        let explicit_breathing_index = breathing
            .map(|rule| vowel_index(expanded, rule.placement))
            .transpose()?;
        let automatic_breathing_index = initial_vowel_index(expanded);
        if explicit_breathing_index.is_some()
            && explicit_breathing_index != automatic_breathing_index
        {
            return Err(Error::ContradictoryMetadata {
                reason: "Synodal psili is licensed only over a word-initial vowel".into(),
            });
        }
        let breathing_index = explicit_breathing_index.or(automatic_breathing_index);
        let accent_mark = surface_accent_mark(expanded, accent_index, accent.mark, environment);
        let mut output = String::with_capacity(expanded.len() + 6);
        for (index, character) in expanded.char_indices() {
            output.push(character);
            if breathing_index == Some(index) {
                output.push(
                    breathing
                        .map_or(BreathingMark::Psili, |rule| rule.mark)
                        .character(),
                );
            }
            if accent_index == index {
                output.push(accent_mark.character());
            }
        }
        let output: String = output.nfc().collect();
        SynodalWord::parse(output.clone())?;
        Ok(output)
    }
}

fn contains_accent_or_breathing_mark(value: &str) -> bool {
    value.nfd().any(|character| {
        matches!(
            character,
            '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0485}' | '\u{0486}'
        )
    })
}

fn surface_accent_mark(
    expanded: &str,
    accent_index: usize,
    lexical_mark: AccentMark,
    environment: AccentEnvironment,
) -> AccentMark {
    if !accent_is_on_final_vowel(expanded, accent_index)
        || !matches!(lexical_mark, AccentMark::Acute | AccentMark::Grave)
    {
        return lexical_mark;
    }
    match environment {
        AccentEnvironment::Standalone => AccentMark::Grave,
        AccentEnvironment::BeforeEnclitic(_) => AccentMark::Acute,
    }
}

fn accent_is_on_final_vowel(value: &str, accent_index: usize) -> bool {
    value
        .char_indices()
        .rev()
        .find(|(_, character)| {
            canonical_combining_class(*character) == 0 && *character != '\u{034f}'
        })
        .is_some_and(|(index, character)| index == accent_index && is_vowel(character))
}

fn initial_vowel_index(value: &str) -> Option<usize> {
    if let Some((_, vowel_index)) = initial_digraph_uk_indices(value) {
        return Some(vowel_index);
    }
    let mut characters = value.char_indices().filter(|(_, character)| {
        canonical_combining_class(*character) == 0 && *character != '\u{034f}'
    });
    let (first_index, first) = characters.next()?;
    if is_vowel(first) {
        return Some(first_index);
    }
    None
}

fn initial_digraph_uk_indices(value: &str) -> Option<(usize, usize)> {
    let mut characters = value.char_indices().filter(|(_, character)| {
        canonical_combining_class(*character) == 0 && *character != '\u{034f}'
    });
    let (first_index, first) = characters.next()?;
    let (second_index, second) = characters.next()?;
    (matches!(first, 'ᲂ' | 'О') && matches!(second, 'у' | 'У'))
        .then_some((first_index, second_index))
}

fn scope_is_empty(scope: &AccentScope) -> bool {
    match scope {
        AccentScope::All => false,
        AccentScope::Noun { numbers }
        | AccentScope::Adjective { numbers, .. }
        | AccentScope::FiniteVerb { numbers, .. } => numbers.is_empty(),
        AccentScope::NounCases { numbers, cases }
        | AccentScope::PronounCases { numbers, cases } => numbers.is_empty() || cases.is_empty(),
        AccentScope::PronounAgreement {
            numbers,
            cases,
            genders,
            animacies,
        } => numbers.is_empty() || cases.is_empty() || genders.is_empty() || animacies.is_empty(),
        AccentScope::OtherCells(cells) => cells.is_empty(),
    }
}

fn unique_rule<'a, T>(
    mut rules: impl Iterator<Item = &'a T>,
    label: &str,
) -> Result<Option<&'a T>> {
    let first = rules.next();
    if rules.next().is_some() {
        return Err(Error::ContradictoryMetadata {
            reason: format!("more than one {label} rule applies to the requested cell"),
        });
    }
    Ok(first)
}

fn vowel_index(value: &str, placement: AccentPlacement) -> Result<usize> {
    let initial_uk_lead = initial_digraph_uk_indices(value).map(|(lead, _)| lead);
    let vowels: Vec<usize> = value
        .char_indices()
        .filter_map(|(index, character)| {
            (Some(index) != initial_uk_lead && is_vowel(character)).then_some(index)
        })
        .collect();
    let selected = match placement {
        AccentPlacement::StemVowelFromStart(offset) => vowels.get(usize::from(offset)).copied(),
        AccentPlacement::EndingVowelFromEnd(offset) => vowels
            .len()
            .checked_sub(usize::from(offset) + 1)
            .and_then(|index| vowels.get(index).copied()),
    };
    selected.ok_or_else(|| Error::ContradictoryMetadata {
        reason: format!("accent placement {placement:?} is outside generated form {value:?}"),
    })
}

fn is_vowel(character: char) -> bool {
    matches!(
        character.to_lowercase().next().unwrap_or(character),
        'а' | 'е'
            | 'є'
            | 'ё'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
            | 'э'
            | 'ю'
            | 'я'
            | 'ѧ'
            | 'ѩ'
            | 'ѣ'
            | 'ѥ'
            | 'ѫ'
            | 'ѭ'
            | 'ѵ'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdjectiveCell, Animacy, AuthorityRole, Case, EpistemicRole, EvidenceId, EvidenceKind,
        Gender, Person, PronounCell, Recension, SourceId,
    };

    fn evidence() -> Evidence {
        Evidence {
            id: EvidenceId::from("accent-test"),
            source: SourceId::from("accent-test-source"),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::AccentParadigm,
            authority_roles: vec![AuthorityRole::Accentual],
            epistemic_role: EpistemicRole::SynodalNormativeAuthority,
            citation: "test citation".into(),
            note: None,
        }
    }

    #[test]
    fn fixed_stem_rule_applies_to_multiple_cells() {
        let paradigm = AccentParadigm {
            id: "test-fixed-stem".into(),
            accent_rules: vec![AccentRule {
                scope: AccentScope::Adjective {
                    form: AdjectiveForm::Long,
                    comparison: Comparison::Positive,
                    numbers: vec![Number::Singular],
                },
                placement: AccentPlacement::StemVowelFromStart(0),
                mark: AccentMark::Acute,
            }],
            breathing_rules: vec![],
            evidence: evidence(),
        };
        for case in [Case::Nominative, Case::Genitive, Case::Dative] {
            let cell = GrammarCell::Adjective(AdjectiveCell {
                case,
                number: Number::Singular,
                gender: Gender::Masculine,
                animacy: Animacy::Inanimate,
                form: AdjectiveForm::Long,
                comparison: Comparison::Positive,
            });
            assert!(paradigm.apply(cell, "мꙋдрый").expect("accent").contains('́'));
        }
    }

    #[test]
    fn pronoun_case_scope_applies_across_gender_and_animacy_dimensions() {
        let paradigm = AccentParadigm::fixed_stem(
            "test-pronoun-fixed-stem",
            AccentScope::PronounCases {
                numbers: vec![Number::Singular],
                cases: vec![Case::Genitive, Case::Dative],
            },
            0,
            AccentMark::Acute,
            evidence(),
        );
        for (case, gender) in [
            (Case::Genitive, Gender::Masculine),
            (Case::Dative, Gender::Feminine),
        ] {
            let cell = GrammarCell::Pronoun(PronounCell {
                case,
                number: Number::Singular,
                gender: Some(gender),
                person: Some(Person::Third),
                animacy: Animacy::Animate,
            });
            assert_eq!(
                paradigm
                    .apply(cell, "моего")
                    .expect("scoped pronoun accent"),
                "мо́его"
            );
        }
        let plural = GrammarCell::Pronoun(PronounCell {
            case: Case::Genitive,
            number: Number::Plural,
            gender: Some(Gender::Masculine),
            person: Some(Person::Third),
            animacy: Animacy::Animate,
        });
        assert!(matches!(
            paradigm.apply(plural, "моихъ"),
            Err(Error::OrthographicMetadataRequired { .. })
        ));
    }

    #[test]
    fn number_conditioned_mobility_supports_fixed_stem_and_ending_stress() {
        let paradigm = AccentParadigm {
            id: "test-mobile".into(),
            accent_rules: vec![
                AccentRule {
                    scope: AccentScope::Noun {
                        numbers: vec![Number::Singular],
                    },
                    placement: AccentPlacement::StemVowelFromStart(0),
                    mark: AccentMark::Acute,
                },
                AccentRule {
                    scope: AccentScope::Noun {
                        numbers: vec![Number::Plural],
                    },
                    placement: AccentPlacement::EndingVowelFromEnd(0),
                    mark: AccentMark::Grave,
                },
            ],
            breathing_rules: vec![],
            evidence: evidence(),
        };
        let cell = |number| {
            GrammarCell::Noun(crate::NounCell {
                case: Case::Dative,
                number,
                animacy: Animacy::Inanimate,
            })
        };
        assert_eq!(
            paradigm
                .apply(cell(Number::Singular), "рабꙋ")
                .expect("stem stress"),
            "ра́бꙋ"
        );
        assert_eq!(
            paradigm
                .apply(cell(Number::Plural), "рабами")
                .expect("ending stress"),
            "рабамѝ"
        );
    }

    #[test]
    fn noun_case_scopes_are_disjoint_and_detect_overlap() {
        let cell = |case| {
            GrammarCell::Noun(crate::NounCell {
                case,
                number: Number::Plural,
                animacy: Animacy::Inanimate,
            })
        };
        let mut paradigm = AccentParadigm {
            id: "test-case-mobile".into(),
            accent_rules: vec![
                AccentRule {
                    scope: AccentScope::NounCases {
                        numbers: vec![Number::Plural],
                        cases: vec![Case::Nominative, Case::Accusative],
                    },
                    placement: AccentPlacement::EndingVowelFromEnd(0),
                    mark: AccentMark::Grave,
                },
                AccentRule {
                    scope: AccentScope::NounCases {
                        numbers: vec![Number::Plural],
                        cases: vec![Case::Genitive],
                    },
                    placement: AccentPlacement::StemVowelFromStart(1),
                    mark: AccentMark::Acute,
                },
            ],
            breathing_rules: vec![],
            evidence: evidence(),
        };
        assert_eq!(
            paradigm
                .apply(cell(Case::Nominative), "имена")
                .expect("ending rule"),
            "и\u{0486}мена̀"
        );
        assert_eq!(
            paradigm
                .apply(cell(Case::Genitive), "именъ")
                .expect("stem rule"),
            "и\u{0486}ме́нъ"
        );
        assert!(matches!(
            paradigm.apply(cell(Case::Dative), "именємъ"),
            Err(Error::OrthographicMetadataRequired {
                field: MetadataField::AccentParadigm
            })
        ));

        paradigm.accent_rules.push(AccentRule {
            scope: AccentScope::Noun {
                numbers: vec![Number::Plural],
            },
            placement: AccentPlacement::StemVowelFromStart(0),
            mark: AccentMark::Kamora,
        });
        assert!(matches!(
            paradigm.apply(cell(Case::Nominative), "имена"),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }

    #[test]
    fn breathing_is_inserted_before_accent() {
        let paradigm = AccentParadigm {
            id: "test-breathing".into(),
            accent_rules: vec![AccentRule {
                scope: AccentScope::All,
                placement: AccentPlacement::StemVowelFromStart(0),
                mark: AccentMark::Acute,
            }],
            breathing_rules: vec![BreathingRule {
                scope: AccentScope::All,
                placement: AccentPlacement::StemVowelFromStart(0),
                mark: BreathingMark::Psili,
            }],
            evidence: evidence(),
        };
        assert_eq!(
            paradigm
                .apply(GrammarCell::LexicalForm, "око")
                .expect("accent"),
            "о\u{0486}\u{0301}ко"
        );
    }

    #[test]
    fn language_wide_initial_psili_does_not_require_lexical_duplication() {
        let paradigm = AccentParadigm::fixed_stem(
            "test-automatic-psili",
            AccentScope::All,
            0,
            AccentMark::Acute,
            evidence(),
        );
        assert_eq!(
            paradigm
                .apply(GrammarCell::LexicalForm, "имѧ")
                .expect("initial-vowel psili"),
            "и\u{0486}\u{0301}мѧ"
        );
        assert_eq!(
            paradigm
                .apply(GrammarCell::LexicalForm, "мꙋдръ")
                .expect("consonant-initial word"),
            "мꙋ́дръ"
        );
    }

    #[test]
    fn productive_accent_rejects_preaccented_or_prebreathed_inputs() {
        let paradigm = AccentParadigm::fixed_stem(
            "test-unmarked-input",
            AccentScope::All,
            0,
            AccentMark::Acute,
            evidence(),
        );
        for marked in [
            "ѐже",
            "е\u{0301}же",
            "е\u{0311}же",
            "е\u{0485}же",
            "е\u{0486}же",
        ] {
            assert!(matches!(
                paradigm.apply(GrammarCell::LexicalForm, marked),
                Err(Error::ContradictoryMetadata { .. })
            ));
        }
    }

    #[test]
    fn digraph_uk_places_psili_over_its_vocalic_second_component() {
        let paradigm = AccentParadigm::fixed_stem(
            "test-digraph-uk-psili",
            AccentScope::All,
            0,
            AccentMark::Grave,
            evidence(),
        );
        for (expanded, expected) in [
            ("ᲂубо", "ᲂу\u{0486}\u{0300}бо"),
            ("Оубо", "Оу\u{0486}\u{0300}бо"),
        ] {
            assert_eq!(
                paradigm
                    .apply(GrammarCell::LexicalForm, expanded)
                    .expect("digraph-uk psili and accent"),
                expected
            );
        }
    }

    #[test]
    fn final_vowel_accent_is_conditioned_by_the_closed_enclitic_environment() {
        let paradigm = AccentParadigm::fixed_ending(
            "test-final-vowel-context",
            AccentScope::All,
            0,
            AccentMark::Acute,
            evidence(),
        );
        assert_eq!(
            paradigm
                .apply(GrammarCell::LexicalForm, "се")
                .expect("standalone varia"),
            "сѐ"
        );
        for enclitic in AccentEnclitic::ALL {
            assert_eq!(
                paradigm
                    .apply_in_environment(
                        GrammarCell::LexicalForm,
                        "се",
                        AccentEnvironment::BeforeEnclitic(enclitic),
                    )
                    .expect("pre-enclitic acute"),
                "се́"
            );
        }
    }

    #[test]
    fn exceptional_nonfinal_grave_and_kamora_are_not_rewritten() {
        for mark in [AccentMark::Grave, AccentMark::Kamora] {
            let paradigm = AccentParadigm::fixed_stem(
                "test-exceptional-mark",
                AccentScope::All,
                0,
                mark,
                evidence(),
            );
            let expected = match mark {
                AccentMark::Grave => "и\u{0486}\u{0300}же",
                AccentMark::Kamora => "и\u{0486}\u{0311}же",
                AccentMark::Acute => unreachable!(),
            };
            assert_eq!(
                paradigm
                    .apply_in_environment(
                        GrammarCell::LexicalForm,
                        "иже",
                        AccentEnvironment::BeforeEnclitic(AccentEnclitic::Particle(
                            EncliticParticle::Li,
                        )),
                    )
                    .expect("nonfinal lexical mark"),
                expected
            );
        }
    }

    #[test]
    fn explicit_breathing_cannot_be_moved_away_from_the_initial_vowel() {
        let paradigm = AccentParadigm {
            id: "test-medial-breathing".into(),
            accent_rules: vec![AccentRule {
                scope: AccentScope::All,
                placement: AccentPlacement::StemVowelFromStart(0),
                mark: AccentMark::Acute,
            }],
            breathing_rules: vec![BreathingRule {
                scope: AccentScope::All,
                placement: AccentPlacement::StemVowelFromStart(1),
                mark: BreathingMark::Psili,
            }],
            evidence: evidence(),
        };
        assert!(matches!(
            paradigm.apply(GrammarCell::LexicalForm, "око"),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }

    #[test]
    fn missing_scope_is_a_typed_failure() {
        let paradigm = AccentParadigm {
            id: "test-scope".into(),
            accent_rules: vec![AccentRule {
                scope: AccentScope::Noun {
                    numbers: vec![Number::Singular],
                },
                placement: AccentPlacement::StemVowelFromStart(0),
                mark: AccentMark::Acute,
            }],
            breathing_rules: vec![],
            evidence: evidence(),
        };
        assert!(matches!(
            paradigm.apply(GrammarCell::LexicalForm, "око"),
            Err(Error::OrthographicMetadataRequired {
                field: MetadataField::AccentParadigm
            })
        ));
    }

    #[test]
    fn rejects_unsourced_or_empty_accent_contracts() {
        let mut paradigm = AccentParadigm::fixed_stem(
            "test-invalid-evidence",
            AccentScope::All,
            0,
            AccentMark::Acute,
            evidence(),
        );
        paradigm.evidence.citation.clear();
        assert!(matches!(
            paradigm.validate(),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let empty_scope = AccentParadigm::fixed_ending(
            "test-empty-scope",
            AccentScope::Noun { numbers: vec![] },
            0,
            AccentMark::Grave,
            evidence(),
        );
        assert!(matches!(
            empty_scope.validate(),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}
