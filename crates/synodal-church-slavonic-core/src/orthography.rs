use unicode_normalization::{UnicodeNormalization, char::canonical_combining_class};

use crate::{
    AccentScope, AuthorityRole, Case, Error, Evidence, EvidenceKind, GrammarCell, MetadataField,
    Number, Recension, Result,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum OrthographyProfile {
    #[default]
    Expanded,
    ExpandedAccentless,
    SynodalLiturgical,
}

impl OrthographyProfile {
    pub const ALL: [Self; 3] = [
        Self::Expanded,
        Self::ExpandedAccentless,
        Self::SynodalLiturgical,
    ];
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct SynodalWord {
    source: String,
    canonical: String,
}

impl SynodalWord {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let source = value.into();
        validate_word(&source)?;
        let canonical = source.nfc().collect();
        Ok(Self { source, canonical })
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    #[must_use]
    pub fn lookup_key(&self) -> String {
        normalize_lookup(&self.canonical)
    }
}

impl TryFrom<String> for SynodalWord {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(value)
    }
}

impl From<SynodalWord> for String {
    fn from(value: SynodalWord) -> Self {
        value.canonical
    }
}

impl AsRef<str> for SynodalWord {
    fn as_ref(&self) -> &str {
        self.canonical()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct RenderedText(String);

impl RenderedText {
    pub fn parse(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(Error::EmptyInput);
        }
        for (byte_index, character) in value.char_indices() {
            if is_private_use(character) || character.is_control() {
                return Err(Error::InvalidUnicode {
                    byte_index,
                    character,
                    reason: "controls and private-use characters are forbidden".into(),
                });
            }
            if character.is_alphabetic() && !is_cyrillic(character) {
                return Err(Error::InvalidUnicode {
                    byte_index,
                    character,
                    reason: "rendered Church Slavonic text cannot contain another script".into(),
                });
            }
        }
        Ok(Self(value.nfc().collect()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RenderedText {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(value)
    }
}

impl From<RenderedText> for String {
    fn from(value: RenderedText) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Loss {
    pub kind: String,
    pub original: String,
    pub replacement: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NormalizationReport {
    pub original: String,
    pub normalized: String,
    pub losses: Vec<Loss>,
}

/// Explicit caller-supplied positional-letter decision. These choices are
/// never inferred from spelling alone because Alypy §2 lists lexical and
/// grammatical exceptions.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum InitialPresentation {
    Preserve,
    WideE,
    BroadOn,
    IotatedYa,
    DigraphUk,
}

/// One closed positional-letter substitution licensed by source-specific
/// Synodal positional and number-antistich rules.
/// The expected input letter is part of the variant, so a rule cannot silently
/// rewrite an unrelated character.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PositionalReplacement {
    WideE,
    BroadOn,
    Omega,
    DecimalI,
    IotatedYa,
    Yeri,
    LittleYus,
}

impl PositionalReplacement {
    pub const ALL: [Self; 7] = [
        Self::WideE,
        Self::BroadOn,
        Self::Omega,
        Self::DecimalI,
        Self::IotatedYa,
        Self::Yeri,
        Self::LittleYus,
    ];

    const fn letters(self) -> (char, char) {
        match self {
            Self::WideE => ('е', 'є'),
            Self::BroadOn => ('о', 'ѻ'),
            Self::Omega => ('о', 'ѡ'),
            Self::DecimalI => ('и', 'ї'),
            Self::IotatedYa => ('ѧ', 'ꙗ'),
            Self::Yeri => ('и', 'ы'),
            Self::LittleYus => ('а', 'ѧ'),
        }
    }
}

/// Zero-based occurrence of the input letter selected by a positional rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum LetterOccurrence {
    FromStart(u8),
    FromEnd(u8),
}

/// Explicit operations for one lexical/cell-conditioned printed spelling.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum PositionalOperation {
    Initial(InitialPresentation),
    DecimalIBeforeVowel,
    /// Applies Alypy §36's `-ѡвъ/-євъ` and `-ѡмъ/-ємъ` spelling when the
    /// selected source/class treats that general rule as controlling. Some
    /// reviewed noun tables retain ordinary vowels, so this is explicit.
    WidePluralEnding,
    Replace {
        replacement: PositionalReplacement,
        occurrence: LetterOccurrence,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PositionalRule {
    pub scope: AccentScope,
    pub operations: Vec<PositionalOperation>,
}

/// Complete caller-supplied positional spelling decisions for a lexeme.
/// An empty operation list is an explicit preserve decision, not missing
/// metadata. Exactly one rule must cover every requested liturgical cell.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PositionalParadigm {
    pub id: String,
    pub rules: Vec<PositionalRule>,
    pub evidence: Evidence,
}

impl PositionalParadigm {
    #[must_use]
    pub fn preserve(id: impl Into<String>, scope: AccentScope, evidence: Evidence) -> Self {
        Self {
            id: id.into(),
            rules: vec![PositionalRule {
                scope,
                operations: vec![],
            }],
            evidence,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() || self.rules.is_empty() {
            return Err(Error::ContradictoryMetadata {
                reason: "a positional paradigm requires a stable ID and at least one scoped rule"
                    .into(),
            });
        }
        if self.evidence.id.as_str().trim().is_empty()
            || self.evidence.source.as_str().trim().is_empty()
            || self.evidence.citation.trim().is_empty()
            || self.evidence.source_recension != Recension::SynodalRussian
            || self.evidence.kind != EvidenceKind::OrthographicParadigm
            || !self
                .evidence
                .authority_roles
                .contains(&AuthorityRole::Orthographic)
        {
            return Err(Error::ContradictoryMetadata {
                reason: "a positional paradigm requires nonempty Synodal orthographic evidence"
                    .into(),
            });
        }
        if self.rules.iter().any(|rule| rule.scope.is_empty()) {
            return Err(Error::ContradictoryMetadata {
                reason: "a positional rule cannot have an empty cell scope".into(),
            });
        }
        Ok(())
    }

    pub fn apply(&self, cell: GrammarCell, expanded: &str) -> Result<String> {
        self.validate()?;
        if contains_prosodic_mark(expanded) {
            return Err(Error::ContradictoryMetadata {
                reason: "a positional paradigm requires an unaccented, unbreathed expanded form"
                    .into(),
            });
        }
        let mut applicable = self.rules.iter().filter(|rule| rule.scope.applies_to(cell));
        let rule = applicable
            .next()
            .ok_or(Error::OrthographicMetadataRequired {
                field: MetadataField::PositionalParadigm,
            })?;
        if applicable.next().is_some() {
            return Err(Error::ContradictoryMetadata {
                reason: "more than one positional rule applies to the requested cell".into(),
            });
        }
        let mut output = expanded.to_owned();
        for operation in &rule.operations {
            output = apply_positional_operation(cell, &output, operation)?;
        }
        Ok(SynodalWord::parse(output)?.canonical().to_owned())
    }
}

/// Applies the selected §36 wide-ending rule. Selection remains explicit
/// because reviewed source tables do not realize the generalization uniformly.
fn apply_wide_plural_ending(cell: GrammarCell, expanded: &str) -> Result<String> {
    if contains_prosodic_mark(expanded) {
        return Err(Error::ContradictoryMetadata {
            reason: "grammatical positional presentation requires an unaccented expanded form"
                .into(),
        });
    }
    let Some((number, case)) = nominal_number_and_case(cell) else {
        return Ok(expanded.to_owned());
    };
    if number != Number::Plural {
        return Ok(expanded.to_owned());
    }
    let suffixes = match case {
        Case::Genitive => [("овъ", "ѡвъ"), ("евъ", "євъ")],
        Case::Dative => [("омъ", "ѡмъ"), ("емъ", "ємъ")],
        _ => return Ok(expanded.to_owned()),
    };
    for (ordinary, wide) in suffixes {
        if let Some(stem) = expanded.strip_suffix(ordinary) {
            let output = format!("{stem}{wide}");
            return Ok(SynodalWord::parse(output)?.canonical().to_owned());
        }
    }
    Ok(expanded.to_owned())
}

fn nominal_number_and_case(cell: GrammarCell) -> Option<(Number, Case)> {
    match cell {
        GrammarCell::Noun(cell) | GrammarCell::VerbalNoun(cell) => Some((cell.number, cell.case)),
        GrammarCell::Adjective(cell) | GrammarCell::Determiner(cell) => {
            Some((cell.number, cell.case))
        }
        GrammarCell::Numeral(cell) => Some((cell.number, cell.case)),
        GrammarCell::Participle(cell) => Some((cell.agreement.number, cell.agreement.case)),
        GrammarCell::Pronoun(cell) => Some((cell.number, cell.case)),
        _ => None,
    }
}

fn apply_positional_operation(
    cell: GrammarCell,
    value: &str,
    operation: &PositionalOperation,
) -> Result<String> {
    match operation {
        PositionalOperation::Initial(presentation) => {
            Ok(apply_initial_presentation(&SynodalWord::parse(value)?, *presentation)?.normalized)
        }
        PositionalOperation::DecimalIBeforeVowel => decimal_i_before_vowel(value),
        PositionalOperation::WidePluralEnding => apply_wide_plural_ending(cell, value),
        PositionalOperation::Replace {
            replacement,
            occurrence,
        } => replace_occurrence(value, *replacement, *occurrence),
    }
}

fn decimal_i_before_vowel(value: &str) -> Result<String> {
    let characters = value.char_indices().collect::<Vec<_>>();
    let mut output = String::with_capacity(value.len());
    for (index, (_, character)) in characters.iter().copied().enumerate() {
        if character == 'и'
            && characters
                .iter()
                .skip(index + 1)
                .find(|(_, next)| canonical_combining_class(*next) == 0 && *next != '\u{034f}')
                .is_some_and(|(_, next)| is_synodal_vowel(*next))
        {
            output.push('ї');
        } else {
            output.push(character);
        }
    }
    Ok(SynodalWord::parse(output)?.canonical().to_owned())
}

fn replace_occurrence(
    value: &str,
    replacement: PositionalReplacement,
    occurrence: LetterOccurrence,
) -> Result<String> {
    let (input, output) = replacement.letters();
    let matches = value
        .char_indices()
        .filter_map(|(index, character)| (character == input).then_some(index))
        .collect::<Vec<_>>();
    let selected = match occurrence {
        LetterOccurrence::FromStart(offset) => matches.get(usize::from(offset)).copied(),
        LetterOccurrence::FromEnd(offset) => matches
            .len()
            .checked_sub(usize::from(offset) + 1)
            .and_then(|index| matches.get(index).copied()),
    }
    .ok_or_else(|| Error::ContradictoryMetadata {
        reason: format!(
            "positional replacement {replacement:?} at {occurrence:?} is outside form {value:?}"
        ),
    })?;
    let mut replaced = String::with_capacity(value.len());
    for (index, character) in value.char_indices() {
        replaced.push(if index == selected { output } else { character });
    }
    Ok(SynodalWord::parse(replaced)?.canonical().to_owned())
}

fn contains_prosodic_mark(value: &str) -> bool {
    value.nfd().any(|character| {
        matches!(
            character,
            '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0485}' | '\u{0486}'
        )
    })
}

fn is_synodal_vowel(character: char) -> bool {
    matches!(
        character.to_lowercase().next().unwrap_or(character),
        'а' | 'е'
            | 'є'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѻ'
            | 'ѡ'
            | 'ꙍ'
            | 'у'
            | 'ꙋ'
            | 'ы'
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

impl InitialPresentation {
    pub const ALL: [Self; 5] = [
        Self::Preserve,
        Self::WideE,
        Self::BroadOn,
        Self::IotatedYa,
        Self::DigraphUk,
    ];
}

/// Applies one reviewed positional-letter decision and reports the change.
/// This is deliberately explicit: lexical semantics decide exceptions such as
/// the two spellings of `ꙗзыкъ`/`ѧзыкъ`, not a blind string rewrite.
pub fn apply_initial_presentation(
    word: &SynodalWord,
    presentation: InitialPresentation,
) -> Result<NormalizationReport> {
    if presentation == InitialPresentation::Preserve {
        return Ok(NormalizationReport {
            original: word.canonical().into(),
            normalized: word.canonical().into(),
            losses: Vec::new(),
        });
    }
    let mut characters = word.canonical().chars();
    let first = characters.next().ok_or(Error::EmptyInput)?;
    let replacement = match (presentation, first) {
        (InitialPresentation::WideE, 'е') => "є",
        (InitialPresentation::BroadOn, 'о') => "ѻ",
        (InitialPresentation::IotatedYa, 'ѧ') => "ꙗ",
        (InitialPresentation::DigraphUk, 'ꙋ' | 'у') => "ᲂу",
        _ => {
            return Err(Error::InvalidOrthography {
                reason: format!("{presentation:?} is incompatible with initial letter {first:?}"),
            });
        }
    };
    let mut normalized = String::from(replacement);
    normalized.extend(characters);
    let normalized = SynodalWord::parse(normalized)?.canonical().to_owned();
    Ok(NormalizationReport {
        original: word.canonical().into(),
        normalized,
        losses: vec![Loss {
            kind: "explicit-positional-presentation".into(),
            original: first.to_string(),
            replacement: replacement.into(),
        }],
    })
}

#[must_use]
pub fn normalize_lookup(value: &str) -> String {
    value.chars().flat_map(char::to_lowercase).nfc().collect()
}

/// Produces the explicit accent-insensitive lookup projection. Historical
/// letters remain distinct; only presentation accents and breathing are removed.
#[must_use]
pub fn normalize_lookup_accentless(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .nfd()
        .filter(|character| {
            !matches!(
                character,
                '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}' | '\u{0486}'
            )
        })
        .nfc()
        .collect()
}

fn validate_word(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::EmptyInput);
    }

    let mut cluster_has_base = false;
    let mut saw_accent = false;
    let mut previous_ccc = 0;
    for (byte_index, character) in value.char_indices() {
        if is_private_use(character) || character.is_control() {
            return Err(Error::InvalidUnicode {
                byte_index,
                character,
                reason: "controls and private-use characters are forbidden".into(),
            });
        }

        let ccc = canonical_combining_class(character);
        if ccc == 0 && character != '\u{034f}' {
            if !is_cyrillic(character) {
                return Err(Error::InvalidUnicode {
                    byte_index,
                    character,
                    reason: "a Synodal word admits only standard Cyrillic letters and marks".into(),
                });
            }
            cluster_has_base = true;
            saw_accent = false;
            previous_ccc = 0;
            continue;
        }

        if !cluster_has_base {
            return Err(Error::InvalidOrthography {
                reason: "a combining mark cannot precede its base letter".into(),
            });
        }
        if !is_permitted_mark(character) {
            return Err(Error::InvalidUnicode {
                byte_index,
                character,
                reason: "combining mark is outside the Church Slavonic repertoire".into(),
            });
        }
        if character == '\u{0486}' && saw_accent {
            return Err(Error::InvalidOrthography {
                reason: "Church Slavonic breathing U+0486 must precede the accent".into(),
            });
        }
        if is_accent(character) && saw_accent {
            return Err(Error::InvalidOrthography {
                reason: "a letter cluster cannot carry more than one accent mark".into(),
            });
        }
        if is_accent(character) {
            saw_accent = true;
        }
        if ccc != 0 && previous_ccc > ccc {
            return Err(Error::InvalidOrthography {
                reason: "combining marks are not in canonical order".into(),
            });
        }
        if ccc != 0 {
            previous_ccc = ccc;
        }
    }
    Ok(())
}

fn is_accent(character: char) -> bool {
    matches!(character, '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{0484}')
}

fn is_permitted_mark(character: char) -> bool {
    matches!(
        character as u32,
        0x0300..=0x036f | 0x0483..=0x0489 | 0x2de0..=0x2dff | 0xfe20..=0xfe2f
    )
}

fn is_cyrillic(character: char) -> bool {
    matches!(
        character as u32,
        0x0400..=0x052f
            | 0x1c80..=0x1c8f
            | 0x2de0..=0x2dff
            | 0xa640..=0xa69f
            | 0x1e030..=0x1e08f
    )
}

fn is_private_use(character: char) -> bool {
    matches!(
        character as u32,
        0xe000..=0xf8ff | 0xf0000..=0xffffd | 0x100000..=0x10fffd
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Animacy, EpistemicRole, EvidenceId, NounCell, SourceId};

    fn evidence() -> Evidence {
        Evidence {
            id: EvidenceId::from("alypy-2-positional-test"),
            source: SourceId::from("alypy-gamanovich-grammar-web-2023"),
            source_recension: Recension::SynodalRussian,
            kind: EvidenceKind::OrthographicParadigm,
            authority_roles: vec![AuthorityRole::Orthographic],
            epistemic_role: EpistemicRole::CallerSuppliedMetadata,
            citation: "Alypy §§2, 36".into(),
            note: None,
        }
    }

    fn noun_cell(case: Case, number: Number) -> GrammarCell {
        GrammarCell::Noun(NounCell {
            case,
            number,
            animacy: Animacy::Animate,
        })
    }

    #[test]
    fn validates_historical_letters_and_marks() {
        let word = SynodalWord::parse("сло\u{0486}\u{0301}во").expect("valid word");
        assert_eq!(word.lookup_key(), "сло\u{0486}\u{0301}во");
    }

    #[test]
    fn rejects_accent_before_breathing() {
        let error = SynodalWord::parse("о\u{0301}\u{0486}").expect_err("invalid order");
        assert!(matches!(error, Error::InvalidOrthography { .. }));
    }

    #[test]
    fn rejects_multiple_accents_on_one_letter_cluster() {
        let error = SynodalWord::parse("а\u{0301}\u{0301}").expect_err("duplicate accents");
        assert!(matches!(error, Error::InvalidOrthography { .. }));
    }

    #[test]
    fn rejects_private_use_and_other_scripts() {
        assert!(SynodalWord::parse("сло\u{e000}во").is_err());
        assert!(SynodalWord::parse("slovo").is_err());
        assert!(SynodalWord::parse("слovo").is_err());
    }

    #[test]
    fn supports_standard_titlo_superscripts_payerok_and_kavyka() {
        for value in ["бг҃ъ", "б\u{2de1}\u{0487}", "слоꙿво", "сло꙾во"] {
            SynodalWord::parse(value).expect("standard encoded Church Slavonic spelling");
        }
    }

    #[test]
    fn positional_rendering_requires_an_explicit_compatible_choice() {
        let word = SynodalWord::parse("его").expect("expanded word");
        let report = apply_initial_presentation(&word, InitialPresentation::WideE)
            .expect("compatible presentation");
        assert_eq!(report.normalized, "єго");
        assert_eq!(report.losses.len(), 1);
        assert!(apply_initial_presentation(&word, InitialPresentation::BroadOn).is_err());
    }

    #[test]
    fn every_initial_presentation_is_explicit_and_shape_checked() {
        for (input, presentation, expected) in [
            ("его", InitialPresentation::Preserve, "его"),
            ("его", InitialPresentation::WideE, "єго"),
            ("отецъ", InitialPresentation::BroadOn, "ѻтецъ"),
            ("ѧзыкъ", InitialPresentation::IotatedYa, "ꙗзыкъ"),
            ("ꙋченикъ", InitialPresentation::DigraphUk, "ᲂученикъ"),
        ] {
            assert_eq!(
                apply_initial_presentation(
                    &SynodalWord::parse(input).expect("valid initial-presentation input"),
                    presentation,
                )
                .expect("compatible initial presentation")
                .normalized,
                expected
            );
        }
    }

    #[test]
    fn scoped_positional_paradigm_preserves_semantics_and_case_distinctions() {
        let singular = noun_cell(Case::Nominative, Number::Singular);
        let plural = noun_cell(Case::Genitive, Number::Plural);
        let paradigm = PositionalParadigm {
            id: "farisei-case-spelling".into(),
            rules: vec![
                PositionalRule {
                    scope: AccentScope::OtherCells(vec![singular]),
                    operations: vec![],
                },
                PositionalRule {
                    scope: AccentScope::OtherCells(vec![plural]),
                    operations: vec![PositionalOperation::Replace {
                        replacement: PositionalReplacement::WideE,
                        occurrence: LetterOccurrence::FromEnd(0),
                    }],
                },
            ],
            evidence: evidence(),
        };
        assert_eq!(
            paradigm
                .apply(singular, "фарисей")
                .expect("singular preserve rule"),
            "фарисей"
        );
        assert_eq!(
            paradigm
                .apply(plural, "фарисей")
                .expect("plural wide-e rule"),
            "фарисєй"
        );

        let people = PositionalParadigm {
            id: "yazyk-people".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::Initial(InitialPresentation::IotatedYa)],
            }],
            evidence: evidence(),
        };
        let organ = PositionalParadigm::preserve("yazyk-organ", AccentScope::All, evidence());
        assert_eq!(
            people
                .apply(singular, "ѧзыкъ")
                .expect("people semantic spelling"),
            "ꙗзыкъ"
        );
        assert_eq!(
            organ
                .apply(singular, "ѧзыкъ")
                .expect("organ semantic spelling"),
            "ѧзыкъ"
        );
    }

    #[test]
    fn decimal_i_rule_is_opt_in_and_preserves_the_sion_exception() {
        let cell = noun_cell(Case::Genitive, Number::Singular);
        let regular = PositionalParadigm {
            id: "i-before-vowel".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::DecimalIBeforeVowel],
            }],
            evidence: evidence(),
        };
        let exception =
            PositionalParadigm::preserve("sion-king-exception", AccentScope::All, evidence());
        assert_eq!(
            regular
                .apply(cell, "сиона")
                .expect("regular decimal-i rule"),
            "сїона"
        );
        assert_eq!(
            exception.apply(cell, "сиѡна").expect("Sihon preserve rule"),
            "сиѡна"
        );
    }

    #[test]
    fn number_antistich_letter_pairs_are_all_source_selectable() {
        let cell = noun_cell(Case::Instrumental, Number::Plural);
        for (input, replacement, occurrence, expected) in [
            (
                "жене",
                PositionalReplacement::WideE,
                LetterOccurrence::FromEnd(0),
                "женє",
            ),
            (
                "мироносицы",
                PositionalReplacement::Omega,
                LetterOccurrence::FromStart(0),
                "мирѡносицы",
            ),
            (
                "мужи",
                PositionalReplacement::Yeri,
                LetterOccurrence::FromEnd(0),
                "мужы",
            ),
            (
                "пришедша",
                PositionalReplacement::LittleYus,
                LetterOccurrence::FromEnd(0),
                "пришедшѧ",
            ),
        ] {
            let paradigm = PositionalParadigm {
                id: "source-selected-number-antistich".into(),
                rules: vec![PositionalRule {
                    scope: AccentScope::All,
                    operations: vec![PositionalOperation::Replace {
                        replacement,
                        occurrence,
                    }],
                }],
                evidence: evidence(),
            };
            assert_eq!(
                paradigm
                    .apply(cell, input)
                    .expect("compatible number-antistich replacement"),
                expected
            );
        }
        assert_eq!(PositionalReplacement::ALL.len(), 7);
    }

    #[test]
    fn selected_plural_ending_rule_is_wide_but_singular_instrumentals_are_not() {
        let paradigm = PositionalParadigm {
            id: "alypy-36-wide-endings".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::WidePluralEnding],
            }],
            evidence: evidence(),
        };
        assert_eq!(
            paradigm
                .apply(noun_cell(Case::Dative, Number::Plural), "рабомъ")
                .expect("plural dative wide ending"),
            "рабѡмъ"
        );
        assert_eq!(
            paradigm
                .apply(noun_cell(Case::Genitive, Number::Plural), "сыновъ")
                .expect("plural genitive wide ending"),
            "сынѡвъ"
        );
        assert_eq!(
            paradigm
                .apply(noun_cell(Case::Instrumental, Number::Singular), "рабомъ",)
                .expect("singular instrumental preserve"),
            "рабомъ"
        );
    }

    #[test]
    fn positional_paradigms_fail_typed_on_missing_overlap_and_bad_occurrence() {
        let cell = noun_cell(Case::Nominative, Number::Singular);
        let missing = PositionalParadigm::preserve(
            "missing",
            AccentScope::OtherCells(vec![noun_cell(Case::Genitive, Number::Plural)]),
            evidence(),
        );
        assert!(matches!(
            missing.apply(cell, "его"),
            Err(Error::OrthographicMetadataRequired {
                field: MetadataField::PositionalParadigm
            })
        ));

        let mut overlap = PositionalParadigm::preserve("overlap", AccentScope::All, evidence());
        overlap.rules.push(PositionalRule {
            scope: AccentScope::All,
            operations: vec![],
        });
        assert!(matches!(
            overlap.apply(cell, "его"),
            Err(Error::ContradictoryMetadata { .. })
        ));

        let bad = PositionalParadigm {
            id: "bad-occurrence".into(),
            rules: vec![PositionalRule {
                scope: AccentScope::All,
                operations: vec![PositionalOperation::Replace {
                    replacement: PositionalReplacement::Omega,
                    occurrence: LetterOccurrence::FromStart(2),
                }],
            }],
            evidence: evidence(),
        };
        assert!(matches!(
            bad.apply(cell, "слово"),
            Err(Error::ContradictoryMetadata { .. })
        ));
    }
}
