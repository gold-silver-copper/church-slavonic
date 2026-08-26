//! The Old Church Slavonic script engine: explicit source-bounded
//! Cyrillic/Glagolitic realization with reported losses, moved wholesale from
//! `old-church-slavonic-core::orthography` (docs/REWRITE_PLAN.md, target
//! layout). The family core re-exports the profile and loss vocabulary and
//! adapts [`Transliteration`] into its own trace and error types.

use crate::text::{Script, canonical_display, detect_script};
use core::{fmt, ops::Deref};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

/// A typed failure of lemma validation or script realization. The family core
/// maps these onto its own inflection-error variants one-for-one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlagoliticError {
    InvalidInput {
        reason: String,
    },
    InvalidLemma {
        input: String,
        reason: String,
    },
    /// A validated spelling cannot be represented by the requested explicit
    /// orthographic profile, or its representation would be lossy under a
    /// reject-loss policy.
    Unrepresentable {
        input: String,
        profile: &'static str,
        character: char,
        scalar_index: usize,
        reason: String,
    },
}

impl GlagoliticError {
    fn invalid_lemma(input: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidLemma {
            input: input.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for GlagoliticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { reason } => write!(f, "invalid input: {reason}"),
            Self::InvalidLemma { input, reason } => {
                write!(f, "invalid lemma {input:?}: {reason}")
            }
            Self::Unrepresentable {
                input,
                profile,
                character,
                scalar_index,
                reason,
            } => write!(
                f,
                "{input:?} cannot be realized by profile {profile}: character \
                 {character:?} at scalar index {scalar_index}: {reason}"
            ),
        }
    }
}

impl std::error::Error for GlagoliticError {}

/// One engine-level normalization step, adapted by the family core into its
/// provenance trace under its Jagić orthography rule identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransliterationStep {
    pub before: String,
    pub after: String,
    pub reason: &'static str,
}

/// Explicit Cyrillic/Glagolitic transliteration profile.
///
/// This is a normalized scholarly realization, not diplomatic manuscript
/// transcription. Its shared-alphabet mappings follow Polivanova §§131–133 and
/// the Jagić table reproduced in Unicode TN41 revision 1, Appendix A.
/// Polivanova's natural-Cyrillic allographs and iotated-vowel boundary control
/// the non-reversible extensions. Every such distinction is reported as a loss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlagoliticProfile {
    Jagic1879NormalizedOcs,
}

impl GlagoliticProfile {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Jagic1879NormalizedOcs => "jagic-1879-normalized-ocs",
        }
    }

    pub const fn source_id(self) -> &'static str {
        match self {
            Self::Jagic1879NormalizedOcs => "unicode-tn41-revision-1",
        }
    }
}

/// Policy for a Cyrillic or Glagolitic distinction that the target script
/// cannot preserve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransliterationLossPolicy {
    /// Return a typed error at the first lossy mapping.
    Reject,
    /// Return the normalized spelling and an ordered loss report.
    Report,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransliterationDirection {
    CyrillicToGlagolitic,
    GlagoliticToCyrillic,
    PreserveGlagoliticInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransliterationFidelity {
    /// Existing Glagolitic input was validated and retained unchanged. This is
    /// not by itself a claim that the caller's spelling is source-attested.
    InputUnchanged,
    /// The profile can recover the input spelling from the output.
    Reversible,
    /// One or more source-script distinctions were normalized away.
    LossReported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransliterationLossKind {
    /// Several Cyrillic code points share one historical Glagolitic letter.
    CyrillicVariantFold,
    /// A Cyrillic letter without a Glagolitic counterpart becomes a sequence.
    CyrillicLetterExpansion,
    /// A literal Cyrillic yer-plus-i sequence collides with a single yeri
    /// spelling in the reverse profile.
    CyrillicSequenceCollision,
    /// A non-classical or colliding Glagolitic letter becomes canonical Cyrillic.
    GlagoliticVariantFold,
}

/// One ordered, explicit loss in a normalized transliteration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransliterationLoss {
    pub scalar_index: usize,
    pub source: String,
    pub replacement: String,
    pub kind: TransliterationLossKind,
    pub reason: &'static str,
}

/// A source-bounded script realization with enough information to keep a
/// normalization from being mistaken for an exact spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transliteration {
    text: String,
    profile: GlagoliticProfile,
    direction: TransliterationDirection,
    fidelity: TransliterationFidelity,
    losses: Vec<TransliterationLoss>,
    steps: Vec<TransliterationStep>,
}

impl Transliteration {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn into_text(self) -> String {
        self.text
    }

    pub fn profile(&self) -> GlagoliticProfile {
        self.profile
    }

    pub fn direction(&self) -> TransliterationDirection {
        self.direction
    }

    pub fn fidelity(&self) -> TransliterationFidelity {
        self.fidelity
    }

    pub fn losses(&self) -> &[TransliterationLoss] {
        &self.losses
    }

    pub fn steps(&self) -> &[TransliterationStep] {
        &self.steps
    }
}

/// A normalized, single-script Old Church Slavonic dictionary lemma.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lemma {
    text: String,
    script: Script,
}

impl Lemma {
    /// Parse, NFC-normalize, and validate one Cyrillic or Glagolitic lemma.
    pub fn parse(input: &str) -> Result<Self, GlagoliticError> {
        let text = canonical_display(input)
            .map_err(|error| GlagoliticError::invalid_lemma(input, error.reason))?;
        let mut has_base = false;
        for ch in text.chars() {
            if ch.is_alphabetic() {
                has_base = true;
            } else if is_combining_mark(ch) {
                if !has_base {
                    return Err(GlagoliticError::invalid_lemma(
                        input,
                        "a combining mark must follow a lemma letter",
                    ));
                }
            } else {
                return Err(GlagoliticError::invalid_lemma(
                    input,
                    format!("the lemma contains a non-letter character {ch:?}"),
                ));
            }
        }
        let script = detect_script(&text);
        match script {
            Script::Cyrillic | Script::Glagolitic => Ok(Self { text, script }),
            Script::Mixed => Err(GlagoliticError::invalid_lemma(
                input,
                "the lemma mixes Cyrillic, Glagolitic, Latin, or another script",
            )),
            Script::Latin => Err(GlagoliticError::invalid_lemma(
                input,
                "the lemma is Latin; expected Old Church Slavonic Cyrillic or Glagolitic",
            )),
            Script::Unknown => Err(GlagoliticError::invalid_lemma(
                input,
                "the lemma has no Cyrillic or Glagolitic letters",
            )),
        }
    }

    /// The normalized spelling.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The lemma's single validated script.
    pub fn script(&self) -> Script {
        self.script
    }
}

impl AsRef<str> for Lemma {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for Lemma {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for Lemma {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Realize one complete OCS word in normalized Glagolitic.
///
/// Existing Glagolitic input is preserved exactly, which lets source-backed
/// dictionary spellings retain precedence. Cyrillic input is transliterated as
/// a complete word, so productive morphology never attaches Cyrillic endings
/// to a Glagolitic stem. The function does not claim to reconstruct manuscript
/// hands, abbreviations, superscripts, or recension-specific letter variants.
pub fn realize_glagolitic(
    input: &str,
    profile: GlagoliticProfile,
    loss_policy: TransliterationLossPolicy,
) -> Result<Transliteration, GlagoliticError> {
    let lemma = Lemma::parse(input)?;
    if lemma.script() == Script::Glagolitic {
        validate_existing_glagolitic(lemma.as_str(), profile)?;
        return Ok(Transliteration {
            text: lemma.as_str().to_string(),
            profile,
            direction: TransliterationDirection::PreserveGlagoliticInput,
            fidelity: TransliterationFidelity::InputUnchanged,
            losses: Vec::new(),
            steps: Vec::new(),
        });
    }

    let characters = lemma.as_str().chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(lemma.as_str().len());
    let mut losses = Vec::new();
    let mut scalar_index = 0;
    while scalar_index < characters.len() {
        let character = characters[scalar_index];
        if scalar_index + 1 < characters.len() {
            let sequence = match (character, characters[scalar_index + 1]) {
                ('Ъ', 'І') => Some(("ЪІ", "ⰟⰉ")),
                ('ъ', 'і') => Some(("ъі", "ⱏⰹ")),
                ('Ь', 'І') => Some(("ЬІ", "ⰠⰉ")),
                ('ь', 'і') => Some(("ьі", "ⱐⰹ")),
                _ => None,
            };
            if let Some((source, replacement)) = sequence {
                record_or_reject_sequence_loss(
                    &mut losses,
                    loss_policy,
                    lemma.as_str(),
                    profile,
                    scalar_index,
                    character,
                    source,
                    replacement,
                )?;
            }
        }
        if is_combining_mark(character) {
            if is_cyrillic_specific_mark(character) {
                return Err(unrepresentable(
                    lemma.as_str(),
                    profile,
                    character,
                    scalar_index,
                    "Cyrillic manuscript marks and abbreviations require an explicit diplomatic profile",
                ));
            }
            output.push(character);
            scalar_index += 1;
            continue;
        }
        let mapping = cyrillic_to_glagolitic(character).ok_or_else(|| {
            unrepresentable(
                lemma.as_str(),
                profile,
                character,
                scalar_index,
                "the normalized OCS profile has no source-backed Glagolitic counterpart",
            )
        })?;
        if let Some((kind, reason)) = mapping.loss {
            record_or_reject_loss(
                &mut losses,
                loss_policy,
                lemma.as_str(),
                profile,
                scalar_index,
                character,
                mapping.output,
                kind,
                reason,
            )?;
        }
        output.push_str(mapping.output);
        scalar_index += 1;
    }

    debug_assert_eq!(detect_script(&output), Script::Glagolitic);
    Ok(transliterated(
        lemma.as_str(),
        output,
        profile,
        TransliterationDirection::CyrillicToGlagolitic,
        losses,
        "realize the complete word with the normalized Jagić Glagolitic profile",
    ))
}

fn validate_existing_glagolitic(
    input: &str,
    profile: GlagoliticProfile,
) -> Result<(), GlagoliticError> {
    for (scalar_index, character) in input.chars().enumerate() {
        if is_combining_mark(character) {
            if is_cyrillic_specific_mark(character) || is_glagolitic_supplemental_mark(character) {
                return Err(unrepresentable(
                    input,
                    profile,
                    character,
                    scalar_index,
                    "Glagolitic superscripts and diplomatic breathing or abbreviation marks require an explicit diplomatic profile",
                ));
            }
            continue;
        }
        if glagolitic_to_cyrillic(character).is_none() {
            return Err(unrepresentable(
                input,
                profile,
                character,
                scalar_index,
                "the character is outside the normalized Old Church Slavonic Glagolitic profile",
            ));
        }
    }
    Ok(())
}

/// Transliterate normalized Glagolitic to the canonical Cyrillic choices of
/// the Jagić/TN41 profile.
///
/// The two standard yer-plus-i sequences are matched before their component
/// letters. Rare colliding Glagolitic variants are available only with an
/// explicit report-loss policy. Later Croatian Glagolitic letters and
/// manuscript superscripts remain outside this normalized OCS profile.
pub fn transliterate_glagolitic_to_cyrillic(
    input: &str,
    profile: GlagoliticProfile,
    loss_policy: TransliterationLossPolicy,
) -> Result<Transliteration, GlagoliticError> {
    let lemma = Lemma::parse(input)?;
    if lemma.script() != Script::Glagolitic {
        let character = lemma.as_str().chars().next().unwrap_or('\0');
        return Err(unrepresentable(
            lemma.as_str(),
            profile,
            character,
            0,
            "the reverse transliterator requires a Glagolitic word",
        ));
    }

    let characters = lemma.as_str().chars().collect::<Vec<_>>();
    let mut output = String::with_capacity(lemma.as_str().len());
    let mut losses = Vec::new();
    let mut scalar_index = 0;
    while scalar_index < characters.len() {
        if scalar_index + 1 < characters.len() {
            match (characters[scalar_index], characters[scalar_index + 1]) {
                ('Ⱏ', 'Ⰹ') => {
                    output.push('Ꙑ');
                    scalar_index += 2;
                    continue;
                }
                ('ⱏ', 'ⰹ') => {
                    output.push('ꙑ');
                    scalar_index += 2;
                    continue;
                }
                ('Ⱐ', 'Ⰹ') => {
                    output.push('Ы');
                    scalar_index += 2;
                    continue;
                }
                ('ⱐ', 'ⰹ') => {
                    output.push('ы');
                    scalar_index += 2;
                    continue;
                }
                _ => {}
            }
        }

        let character = characters[scalar_index];
        if is_combining_mark(character) {
            if is_cyrillic_specific_mark(character) || is_glagolitic_supplemental_mark(character) {
                return Err(unrepresentable(
                    lemma.as_str(),
                    profile,
                    character,
                    scalar_index,
                    "Glagolitic superscripts and diplomatic breathing or abbreviation marks require an explicit expansion profile",
                ));
            }
            output.push(character);
            scalar_index += 1;
            continue;
        }
        let mapping = glagolitic_to_cyrillic(character).ok_or_else(|| {
            unrepresentable(
                lemma.as_str(),
                profile,
                character,
                scalar_index,
                "the character is outside the normalized Old Church Slavonic Glagolitic profile",
            )
        })?;
        if let Some((kind, reason)) = mapping.loss {
            record_or_reject_loss(
                &mut losses,
                loss_policy,
                lemma.as_str(),
                profile,
                scalar_index,
                character,
                mapping.output,
                kind,
                reason,
            )?;
        }
        output.push_str(mapping.output);
        scalar_index += 1;
    }

    debug_assert_eq!(detect_script(&output), Script::Cyrillic);
    Ok(transliterated(
        lemma.as_str(),
        output,
        profile,
        TransliterationDirection::GlagoliticToCyrillic,
        losses,
        "transliterate normalized Glagolitic with the Jagić Cyrillic table",
    ))
}

#[derive(Clone, Copy)]
struct ScriptMapping {
    output: &'static str,
    loss: Option<(TransliterationLossKind, &'static str)>,
}

const fn direct(output: &'static str) -> ScriptMapping {
    ScriptMapping { output, loss: None }
}

const fn lossy(
    output: &'static str,
    kind: TransliterationLossKind,
    reason: &'static str,
) -> ScriptMapping {
    ScriptMapping {
        output,
        loss: Some((kind, reason)),
    }
}

fn cyrillic_to_glagolitic(character: char) -> Option<ScriptMapping> {
    let mapping = match character {
        'А' => direct("Ⰰ"),
        'а' => direct("ⰰ"),
        'Б' => direct("Ⰱ"),
        'б' => direct("ⰱ"),
        'В' => direct("Ⰲ"),
        'в' => direct("ⰲ"),
        'Г' => direct("Ⰳ"),
        'г' => direct("ⰳ"),
        'Д' => direct("Ⰴ"),
        'д' => direct("ⰴ"),
        'Е' => direct("Ⰵ"),
        'е' => direct("ⰵ"),
        'Ж' => direct("Ⰶ"),
        'ж' => direct("ⰶ"),
        'Ѕ' => direct("Ⰷ"),
        'ѕ' => direct("ⰷ"),
        'З' => direct("Ⰸ"),
        'з' => direct("ⰸ"),
        'И' => direct("Ⰻ"),
        'и' => direct("ⰻ"),
        'Й' => direct("Ⰻ\u{306}"),
        'й' => direct("ⰻ\u{306}"),
        'І' => direct("Ⰹ"),
        'і' => direct("ⰹ"),
        'Ꙇ' => direct("Ⰺ"),
        'ꙇ' => direct("ⰺ"),
        'Ꙉ' => direct("Ⰼ"),
        'ꙉ' => direct("ⰼ"),
        'К' => direct("Ⰽ"),
        'к' => direct("ⰽ"),
        'Л' => direct("Ⰾ"),
        'л' => direct("ⰾ"),
        'М' => direct("Ⰿ"),
        'м' => direct("ⰿ"),
        'Н' => direct("Ⱀ"),
        'н' => direct("ⱀ"),
        'О' => direct("Ⱁ"),
        'о' => direct("ⱁ"),
        'П' => direct("Ⱂ"),
        'п' => direct("ⱂ"),
        'Р' => direct("Ⱃ"),
        'р' => direct("ⱃ"),
        'С' => direct("Ⱄ"),
        'с' => direct("ⱄ"),
        'Т' => direct("Ⱅ"),
        'т' => direct("ⱅ"),
        'Ꙋ' => direct("Ⱆ"),
        'ꙋ' => direct("ⱆ"),
        'Ф' => direct("Ⱇ"),
        'ф' => direct("ⱇ"),
        'Х' => direct("Ⱈ"),
        'х' => direct("ⱈ"),
        'Ѡ' => direct("Ⱉ"),
        'ѡ' => direct("ⱉ"),
        'Ц' => direct("Ⱌ"),
        'ц' => direct("ⱌ"),
        'Ч' => direct("Ⱍ"),
        'ч' => direct("ⱍ"),
        'Ш' => direct("Ⱎ"),
        'ш' => direct("ⱎ"),
        'Щ' => direct("Ⱋ"),
        'щ' => direct("ⱋ"),
        'Ъ' => direct("Ⱏ"),
        'ъ' => direct("ⱏ"),
        'Ꙑ' => direct("ⰟⰉ"),
        'ꙑ' => direct("ⱏⰹ"),
        'Ь' => direct("Ⱐ"),
        'ь' => direct("ⱐ"),
        'Ы' => direct("ⰠⰉ"),
        'ы' => direct("ⱐⰹ"),
        'Ѣ' => direct("Ⱑ"),
        'ѣ' => direct("ⱑ"),
        'Ю' => direct("Ⱓ"),
        'ю' => direct("ⱓ"),
        'Ѧ' => direct("Ⱔ"),
        'ѧ' => direct("ⱔ"),
        'Ѫ' => direct("Ⱘ"),
        'ѫ' => direct("ⱘ"),
        'Ѩ' => direct("Ⱗ"),
        'ѩ' => direct("ⱗ"),
        'Ѭ' => direct("Ⱙ"),
        'ѭ' => direct("ⱙ"),
        'Ѳ' => direct("Ⱚ"),
        'ѳ' => direct("ⱚ"),
        'Ѵ' => direct("Ⱛ"),
        'ѵ' => direct("ⱛ"),
        'Ѷ' => direct("Ⱛ\u{30f}"),
        'ѷ' => direct("ⱛ\u{30f}"),

        'Є' => lossy(
            "Ⰵ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic ye and e have one normalized Glagolitic counterpart",
        ),
        'є' => lossy(
            "ⰵ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic ye and e have one normalized Glagolitic counterpart",
        ),
        'Ї' => lossy(
            "Ⰺ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic yi is a secondary allograph of initial izhe in Polivanova's Glagolitic transliteration table",
        ),
        'ї' => lossy(
            "ⰺ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic yi is a secondary allograph of initial izhe in Polivanova's Glagolitic transliteration table",
        ),
        'У' => lossy(
            "Ⱆ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic u and monograph uk have one normalized Glagolitic counterpart",
        ),
        'у' => lossy(
            "ⱆ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic u and monograph uk have one normalized Glagolitic counterpart",
        ),
        'Ꙗ' => lossy(
            "Ⰰ",
            TransliterationLossKind::CyrillicVariantFold,
            "natural Cyrillic iotated a normalizes to non-iotated a because early Glagolitic has no matching iotated letter",
        ),
        'ꙗ' => lossy(
            "ⰰ",
            TransliterationLossKind::CyrillicVariantFold,
            "natural Cyrillic iotated a normalizes to non-iotated a because early Glagolitic has no matching iotated letter",
        ),
        'Ѥ' => lossy(
            "Ⰵ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic iotated e has no distinct Glagolitic letter",
        ),
        'ѥ' => lossy(
            "ⰵ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic iotated e has no distinct Glagolitic letter",
        ),
        'Ѻ' => lossy(
            "Ⱁ",
            TransliterationLossKind::CyrillicVariantFold,
            "round o and ordinary o share normalized Glagolitic onu",
        ),
        'ѻ' => lossy(
            "ⱁ",
            TransliterationLossKind::CyrillicVariantFold,
            "round o and ordinary o share normalized Glagolitic onu",
        ),
        'Ꙍ' => lossy(
            "Ⱉ",
            TransliterationLossKind::CyrillicVariantFold,
            "broad omega and omega share normalized Glagolitic ot",
        ),
        'ꙍ' => lossy(
            "ⱉ",
            TransliterationLossKind::CyrillicVariantFold,
            "broad omega and omega share normalized Glagolitic ot",
        ),
        'Ѿ' => lossy(
            "ⰙⰕ",
            TransliterationLossKind::CyrillicLetterExpansion,
            "Cyrillic ot has no single Glagolitic counterpart",
        ),
        'ѿ' => lossy(
            "ⱉⱅ",
            TransliterationLossKind::CyrillicLetterExpansion,
            "Cyrillic ot has no single Glagolitic counterpart",
        ),
        'Ѯ' => lossy(
            "ⰍⰔ",
            TransliterationLossKind::CyrillicLetterExpansion,
            "Cyrillic xi expands to Glagolitic k-s",
        ),
        'ѯ' => lossy(
            "ⰽⱄ",
            TransliterationLossKind::CyrillicLetterExpansion,
            "Cyrillic xi expands to Glagolitic k-s",
        ),
        'Ѱ' => lossy(
            "ⰒⰔ",
            TransliterationLossKind::CyrillicLetterExpansion,
            "Cyrillic psi expands to Glagolitic p-s",
        ),
        'ѱ' => lossy(
            "ⱂⱄ",
            TransliterationLossKind::CyrillicLetterExpansion,
            "Cyrillic psi expands to Glagolitic p-s",
        ),
        'Ꙁ' => lossy(
            "Ⰸ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic zemlya variants share Glagolitic zemlja",
        ),
        'ꙁ' => lossy(
            "ⰸ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic zemlya variants share Glagolitic zemlja",
        ),
        'Ꙃ' => lossy(
            "Ⰷ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic dzelo variants share Glagolitic djelo",
        ),
        'ꙃ' => lossy(
            "ⰷ",
            TransliterationLossKind::CyrillicVariantFold,
            "Cyrillic dzelo variants share Glagolitic djelo",
        ),
        'Ꙙ' => lossy(
            "Ⱔ",
            TransliterationLossKind::CyrillicVariantFold,
            "closed little yus shares normalized Glagolitic small yus",
        ),
        'ꙙ' => lossy(
            "ⱔ",
            TransliterationLossKind::CyrillicVariantFold,
            "closed little yus shares normalized Glagolitic small yus",
        ),
        'Ꙝ' => lossy(
            "Ⱗ",
            TransliterationLossKind::CyrillicVariantFold,
            "iotated closed little yus shares normalized Glagolitic iotated small yus",
        ),
        'ꙝ' => lossy(
            "ⱗ",
            TransliterationLossKind::CyrillicVariantFold,
            "iotated closed little yus shares normalized Glagolitic iotated small yus",
        ),
        'Ꙩ' | 'Ꙫ' | 'Ꙭ' => lossy(
            "Ⱁ",
            TransliterationLossKind::CyrillicVariantFold,
            "ornamental o variants share normalized Glagolitic onu",
        ),
        'ꙩ' | 'ꙫ' | 'ꙭ' => lossy(
            "ⱁ",
            TransliterationLossKind::CyrillicVariantFold,
            "ornamental o variants share normalized Glagolitic onu",
        ),
        _ => return None,
    };
    Some(mapping)
}

fn glagolitic_to_cyrillic(character: char) -> Option<ScriptMapping> {
    let mapping = match character {
        'Ⰰ' => direct("А"),
        'ⰰ' => direct("а"),
        'Ⰱ' => direct("Б"),
        'ⰱ' => direct("б"),
        'Ⰲ' => direct("В"),
        'ⰲ' => direct("в"),
        'Ⰳ' => direct("Г"),
        'ⰳ' => direct("г"),
        'Ⰴ' => direct("Д"),
        'ⰴ' => direct("д"),
        'Ⰵ' => direct("Е"),
        'ⰵ' => direct("е"),
        'Ⰶ' => direct("Ж"),
        'ⰶ' => direct("ж"),
        'Ⰷ' => direct("Ѕ"),
        'ⰷ' => direct("ѕ"),
        'Ⰸ' => direct("З"),
        'ⰸ' => direct("з"),
        'Ⰻ' => direct("И"),
        'ⰻ' => direct("и"),
        'Ⰹ' => direct("І"),
        'ⰹ' => direct("і"),
        'Ⰺ' => direct("Ꙇ"),
        'ⰺ' => direct("ꙇ"),
        'Ⰼ' => direct("Ꙉ"),
        'ⰼ' => direct("ꙉ"),
        'Ⰽ' => direct("К"),
        'ⰽ' => direct("к"),
        'Ⰾ' => direct("Л"),
        'ⰾ' => direct("л"),
        'Ⰿ' => direct("М"),
        'ⰿ' => direct("м"),
        'Ⱀ' => direct("Н"),
        'ⱀ' => direct("н"),
        'Ⱁ' => direct("О"),
        'ⱁ' => direct("о"),
        'Ⱂ' => direct("П"),
        'ⱂ' => direct("п"),
        'Ⱃ' => direct("Р"),
        'ⱃ' => direct("р"),
        'Ⱄ' => direct("С"),
        'ⱄ' => direct("с"),
        'Ⱅ' => direct("Т"),
        'ⱅ' => direct("т"),
        'Ⱆ' => direct("Ꙋ"),
        'ⱆ' => direct("ꙋ"),
        'Ⱇ' => direct("Ф"),
        'ⱇ' => direct("ф"),
        'Ⱈ' => direct("Х"),
        'ⱈ' => direct("х"),
        'Ⱉ' => direct("Ѡ"),
        'ⱉ' => direct("ѡ"),
        'Ⱌ' => direct("Ц"),
        'ⱌ' => direct("ц"),
        'Ⱍ' => direct("Ч"),
        'ⱍ' => direct("ч"),
        'Ⱎ' => direct("Ш"),
        'ⱎ' => direct("ш"),
        'Ⱋ' => direct("Щ"),
        'ⱋ' => direct("щ"),
        'Ⱏ' => direct("Ъ"),
        'ⱏ' => direct("ъ"),
        'Ⱐ' => direct("Ь"),
        'ⱐ' => direct("ь"),
        'Ⱑ' => direct("Ѣ"),
        'ⱑ' => direct("ѣ"),
        'Ⱓ' => direct("Ю"),
        'ⱓ' => direct("ю"),
        'Ⱔ' => direct("Ѧ"),
        'ⱔ' => direct("ѧ"),
        'Ⱘ' => direct("Ѫ"),
        'ⱘ' => direct("ѫ"),
        'Ⱗ' => direct("Ѩ"),
        'ⱗ' => direct("ѩ"),
        'Ⱙ' => direct("Ѭ"),
        'ⱙ' => direct("ѭ"),
        'Ⱚ' => direct("Ѳ"),
        'ⱚ' => direct("ѳ"),
        'Ⱛ' => direct("Ѵ"),
        'ⱛ' => direct("ѵ"),
        'Ⱊ' => lossy(
            "П",
            TransliterationLossKind::GlagoliticVariantFold,
            "the character collides with ordinary Glagolitic pokoji and has disputed identity",
        ),
        'ⱊ' => lossy(
            "п",
            TransliterationLossKind::GlagoliticVariantFold,
            "the character collides with ordinary Glagolitic pokoji and has disputed identity",
        ),
        'Ⱒ' => lossy(
            "Х",
            TransliterationLossKind::GlagoliticVariantFold,
            "the rare character has no distinct Cyrillic analog",
        ),
        'ⱒ' => lossy(
            "х",
            TransliterationLossKind::GlagoliticVariantFold,
            "the rare character has no distinct Cyrillic analog",
        ),
        'Ⱕ' => lossy(
            "Ѧ",
            TransliterationLossKind::GlagoliticVariantFold,
            "small yus with tail is a graphical variant in this normalized profile",
        ),
        'ⱕ' => lossy(
            "ѧ",
            TransliterationLossKind::GlagoliticVariantFold,
            "small yus with tail is a graphical variant in this normalized profile",
        ),
        _ => return None,
    };
    Some(mapping)
}

fn is_cyrillic_specific_mark(character: char) -> bool {
    matches!(
        u32::from(character),
        0x0483 | 0x0485..=0x0489 | 0xa66f | 0xa67c..=0xa67d
    )
}

fn is_glagolitic_supplemental_mark(character: char) -> bool {
    (0x1e000..=0x1e02f).contains(&u32::from(character))
}

#[allow(clippy::too_many_arguments)]
fn record_or_reject_sequence_loss(
    losses: &mut Vec<TransliterationLoss>,
    policy: TransliterationLossPolicy,
    input: &str,
    profile: GlagoliticProfile,
    scalar_index: usize,
    character: char,
    source: &'static str,
    replacement: &'static str,
) -> Result<(), GlagoliticError> {
    let reason = "the literal yer-plus-i sequence collides with one canonical yeri sequence in reverse transliteration";
    if policy == TransliterationLossPolicy::Reject {
        return Err(unrepresentable(
            input,
            profile,
            character,
            scalar_index,
            reason,
        ));
    }
    losses.push(TransliterationLoss {
        scalar_index,
        source: source.to_string(),
        replacement: replacement.to_string(),
        kind: TransliterationLossKind::CyrillicSequenceCollision,
        reason,
    });
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn record_or_reject_loss(
    losses: &mut Vec<TransliterationLoss>,
    policy: TransliterationLossPolicy,
    input: &str,
    profile: GlagoliticProfile,
    scalar_index: usize,
    character: char,
    replacement: &'static str,
    kind: TransliterationLossKind,
    reason: &'static str,
) -> Result<(), GlagoliticError> {
    if policy == TransliterationLossPolicy::Reject {
        return Err(unrepresentable(
            input,
            profile,
            character,
            scalar_index,
            reason,
        ));
    }
    losses.push(TransliterationLoss {
        scalar_index,
        source: character.to_string(),
        replacement: replacement.to_string(),
        kind,
        reason,
    });
    Ok(())
}

fn unrepresentable(
    input: &str,
    profile: GlagoliticProfile,
    character: char,
    scalar_index: usize,
    reason: impl Into<String>,
) -> GlagoliticError {
    GlagoliticError::Unrepresentable {
        input: input.to_string(),
        profile: profile.code(),
        character,
        scalar_index,
        reason: reason.into(),
    }
}

fn transliterated(
    before: &str,
    text: String,
    profile: GlagoliticProfile,
    direction: TransliterationDirection,
    losses: Vec<TransliterationLoss>,
    reason: &'static str,
) -> Transliteration {
    let text = text.nfc().collect::<String>();
    let fidelity = if losses.is_empty() {
        TransliterationFidelity::Reversible
    } else {
        TransliterationFidelity::LossReported
    };
    Transliteration {
        steps: vec![TransliterationStep {
            before: before.to_string(),
            after: text.clone(),
            reason,
        }],
        text,
        profile,
        direction,
        fidelity,
        losses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;




    #[test]
    fn lemma_is_normalized_single_script_and_letters_only() {
        let lemma = Lemma::parse("И\u{306}").expect("decomposed Cyrillic lemma");
        assert_eq!(lemma.as_str(), "Й");
        assert_eq!(lemma.script(), Script::Cyrillic);
        assert_eq!(
            Lemma::parse("ⱄⰾⱁⰲⱁ").expect("Glagolitic").script(),
            Script::Glagolitic
        );
        for invalid in [
            "слоword",
            "слоα",
            "слово.",
            "<слово>",
            "\u{301}слово",
            "latin",
        ] {
            assert!(Lemma::parse(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn normalized_jagic_core_is_reversible_and_complete_word_scoped() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        let glagolitic = realize_glagolitic("слово", profile, TransliterationLossPolicy::Reject)
            .expect("shared alphabet");
        assert_eq!(glagolitic.text(), "ⱄⰾⱁⰲⱁ");
        assert_eq!(glagolitic.fidelity(), TransliterationFidelity::Reversible);
        assert_eq!(
            glagolitic.direction(),
            TransliterationDirection::CyrillicToGlagolitic
        );
        assert_eq!(glagolitic.steps()[0].before, "слово");
        assert_eq!(glagolitic.steps()[0].after, "ⱄⰾⱁⰲⱁ");

        let cyrillic = transliterate_glagolitic_to_cyrillic(
            glagolitic.text(),
            profile,
            TransliterationLossPolicy::Reject,
        )
        .expect("reverse shared alphabet");
        assert_eq!(cyrillic.text(), "слово");
        assert_eq!(cyrillic.fidelity(), TransliterationFidelity::Reversible);
    }

    #[test]
    fn every_shared_jagic_letter_round_trips_in_both_cases() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        let pairs = [
            ('А', "Ⰰ"),
            ('а', "ⰰ"),
            ('Б', "Ⰱ"),
            ('б', "ⰱ"),
            ('В', "Ⰲ"),
            ('в', "ⰲ"),
            ('Г', "Ⰳ"),
            ('г', "ⰳ"),
            ('Д', "Ⰴ"),
            ('д', "ⰴ"),
            ('Е', "Ⰵ"),
            ('е', "ⰵ"),
            ('Ж', "Ⰶ"),
            ('ж', "ⰶ"),
            ('Ѕ', "Ⰷ"),
            ('ѕ', "ⰷ"),
            ('З', "Ⰸ"),
            ('з', "ⰸ"),
            ('И', "Ⰻ"),
            ('и', "ⰻ"),
            ('І', "Ⰹ"),
            ('і', "ⰹ"),
            ('Ꙇ', "Ⰺ"),
            ('ꙇ', "ⰺ"),
            ('Ꙉ', "Ⰼ"),
            ('ꙉ', "ⰼ"),
            ('К', "Ⰽ"),
            ('к', "ⰽ"),
            ('Л', "Ⰾ"),
            ('л', "ⰾ"),
            ('М', "Ⰿ"),
            ('м', "ⰿ"),
            ('Н', "Ⱀ"),
            ('н', "ⱀ"),
            ('О', "Ⱁ"),
            ('о', "ⱁ"),
            ('П', "Ⱂ"),
            ('п', "ⱂ"),
            ('Р', "Ⱃ"),
            ('р', "ⱃ"),
            ('С', "Ⱄ"),
            ('с', "ⱄ"),
            ('Т', "Ⱅ"),
            ('т', "ⱅ"),
            ('Ꙋ', "Ⱆ"),
            ('ꙋ', "ⱆ"),
            ('Ф', "Ⱇ"),
            ('ф', "ⱇ"),
            ('Х', "Ⱈ"),
            ('х', "ⱈ"),
            ('Ѡ', "Ⱉ"),
            ('ѡ', "ⱉ"),
            ('Ц', "Ⱌ"),
            ('ц', "ⱌ"),
            ('Ч', "Ⱍ"),
            ('ч', "ⱍ"),
            ('Ш', "Ⱎ"),
            ('ш', "ⱎ"),
            ('Щ', "Ⱋ"),
            ('щ', "ⱋ"),
            ('Ъ', "Ⱏ"),
            ('ъ', "ⱏ"),
            ('Ь', "Ⱐ"),
            ('ь', "ⱐ"),
            ('Ѣ', "Ⱑ"),
            ('ѣ', "ⱑ"),
            ('Ю', "Ⱓ"),
            ('ю', "ⱓ"),
            ('Ѧ', "Ⱔ"),
            ('ѧ', "ⱔ"),
            ('Ѫ', "Ⱘ"),
            ('ѫ', "ⱘ"),
            ('Ѩ', "Ⱗ"),
            ('ѩ', "ⱗ"),
            ('Ѭ', "Ⱙ"),
            ('ѭ', "ⱙ"),
            ('Ѳ', "Ⱚ"),
            ('ѳ', "ⱚ"),
            ('Ѵ', "Ⱛ"),
            ('ѵ', "ⱛ"),
        ];
        for (cyrillic, glagolitic) in pairs {
            let input = cyrillic.to_string();
            let forward = realize_glagolitic(&input, profile, TransliterationLossPolicy::Reject)
                .expect("shared letter");
            assert_eq!(forward.text(), glagolitic, "{cyrillic}");
            assert_eq!(forward.fidelity(), TransliterationFidelity::Reversible);
            let reverse = transliterate_glagolitic_to_cyrillic(
                glagolitic,
                profile,
                TransliterationLossPolicy::Reject,
            )
            .expect("shared reverse letter");
            assert_eq!(reverse.text(), input, "{glagolitic}");
            assert_eq!(reverse.fidelity(), TransliterationFidelity::Reversible);
        }
    }

    #[test]
    fn yeri_sequences_preserve_the_two_cyrillic_spellings() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        for (cyrillic, glagolitic) in [("Ꙑ", "ⰟⰉ"), ("ꙑ", "ⱏⰹ"), ("Ы", "ⰠⰉ"), ("ы", "ⱐⰹ")]
        {
            let forward = realize_glagolitic(cyrillic, profile, TransliterationLossPolicy::Reject)
                .expect("standard yeri sequence");
            assert_eq!(forward.text(), glagolitic);
            let reverse = transliterate_glagolitic_to_cyrillic(
                glagolitic,
                profile,
                TransliterationLossPolicy::Reject,
            )
            .expect("longest sequence first");
            assert_eq!(reverse.text(), cyrillic);
        }
    }

    #[test]
    fn cyrillic_only_distinctions_are_reported_or_rejected() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        let realized = realize_glagolitic("землꙗ", profile, TransliterationLossPolicy::Report)
            .expect("reported historical fold");
        assert_eq!(realized.text(), "ⰸⰵⰿⰾⰰ");
        assert_eq!(realized.fidelity(), TransliterationFidelity::LossReported);
        assert_eq!(realized.losses().len(), 1);
        assert_eq!(realized.losses()[0].scalar_index, 4);
        assert_eq!(
            realized.losses()[0].kind,
            TransliterationLossKind::CyrillicVariantFold
        );

        assert!(matches!(
            realize_glagolitic("землꙗ", profile, TransliterationLossPolicy::Reject),
            Err(GlagoliticError::Unrepresentable {
                character: 'ꙗ',
                scalar_index: 4,
                ..
            })
        ));
    }

    #[test]
    fn every_reviewed_cyrillic_fold_or_expansion_is_loss_reported() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        let cases = [
            ("Є", "Ⰵ"),
            ("є", "ⰵ"),
            ("Ї", "Ⰺ"),
            ("ї", "ⰺ"),
            ("У", "Ⱆ"),
            ("у", "ⱆ"),
            ("Ꙗ", "Ⰰ"),
            ("ꙗ", "ⰰ"),
            ("Ѥ", "Ⰵ"),
            ("ѥ", "ⰵ"),
            ("Ѻ", "Ⱁ"),
            ("ѻ", "ⱁ"),
            ("Ꙍ", "Ⱉ"),
            ("ꙍ", "ⱉ"),
            ("Ѿ", "ⰙⰕ"),
            ("ѿ", "ⱉⱅ"),
            ("Ѯ", "ⰍⰔ"),
            ("ѯ", "ⰽⱄ"),
            ("Ѱ", "ⰒⰔ"),
            ("ѱ", "ⱂⱄ"),
            ("Ꙁ", "Ⰸ"),
            ("ꙁ", "ⰸ"),
            ("Ꙃ", "Ⰷ"),
            ("ꙃ", "ⰷ"),
            ("Ꙙ", "Ⱔ"),
            ("ꙙ", "ⱔ"),
            ("Ꙝ", "Ⱗ"),
            ("ꙝ", "ⱗ"),
            ("Ꙩ", "Ⱁ"),
            ("ꙩ", "ⱁ"),
            ("Ꙫ", "Ⱁ"),
            ("ꙫ", "ⱁ"),
            ("Ꙭ", "Ⱁ"),
            ("ꙭ", "ⱁ"),
        ];
        for (cyrillic, expected) in cases {
            let realized = realize_glagolitic(cyrillic, profile, TransliterationLossPolicy::Report)
                .expect("reviewed normalization");
            assert_eq!(realized.text(), expected, "{cyrillic}");
            assert_eq!(realized.fidelity(), TransliterationFidelity::LossReported);
            assert_eq!(realized.losses().len(), 1);
            assert!(
                realize_glagolitic(cyrillic, profile, TransliterationLossPolicy::Reject,).is_err()
            );
        }
    }

    #[test]
    fn rare_glagolitic_collisions_are_never_silent() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        for (glagolitic, expected) in [
            ("Ⱊ", "П"),
            ("ⱊ", "п"),
            ("Ⱒ", "Х"),
            ("ⱒ", "х"),
            ("Ⱕ", "Ѧ"),
            ("ⱕ", "ѧ"),
        ] {
            let realized = transliterate_glagolitic_to_cyrillic(
                glagolitic,
                profile,
                TransliterationLossPolicy::Report,
            )
            .expect("reviewed rare-letter fold");
            assert_eq!(realized.text(), expected, "{glagolitic}");
            assert_eq!(realized.fidelity(), TransliterationFidelity::LossReported);
            assert!(
                transliterate_glagolitic_to_cyrillic(
                    glagolitic,
                    profile,
                    TransliterationLossPolicy::Reject,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn exact_glagolitic_precedes_normalized_realization() {
        let exact = "ⱁⰽⱁ";
        let realized = realize_glagolitic(
            exact,
            GlagoliticProfile::Jagic1879NormalizedOcs,
            TransliterationLossPolicy::Reject,
        )
        .expect("exact source form");
        assert_eq!(realized.text(), exact);
        assert_eq!(realized.fidelity(), TransliterationFidelity::InputUnchanged);
        assert_eq!(
            realized.direction(),
            TransliterationDirection::PreserveGlagoliticInput
        );
        assert!(realized.losses().is_empty());
        assert!(realized.steps().is_empty());
    }

    #[test]
    fn neutral_and_palatalization_marks_are_preserved_but_diplomatic_marks_fail() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        assert_eq!(
            realize_glagolitic("гра\u{301}дъ", profile, TransliterationLossPolicy::Reject,)
                .expect("neutral scholarly acute")
                .text(),
            "ⰳⱃⰰ\u{301}ⰴⱏ"
        );
        assert_eq!(
            realize_glagolitic("цар\u{484}ь", profile, TransliterationLossPolicy::Reject,)
                .expect("Polivanova kamora is shared by the script profiles")
                .text(),
            "ⱌⰰⱃ\u{484}ⱐ"
        );
        assert!(matches!(
            realize_glagolitic("аз\u{486}", profile, TransliterationLossPolicy::Report),
            Err(GlagoliticError::Unrepresentable {
                character: '\u{486}',
                ..
            })
        ));
    }

    #[test]
    fn literal_yer_i_sequences_are_not_mislabeled_reversible() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        let result = realize_glagolitic("ъі", profile, TransliterationLossPolicy::Report)
            .expect("explicit collision report");
        assert_eq!(result.text(), "ⱏⰹ");
        assert_eq!(result.fidelity(), TransliterationFidelity::LossReported);
        assert_eq!(
            result.losses()[0].kind,
            TransliterationLossKind::CyrillicSequenceCollision
        );
        assert!(realize_glagolitic("ъі", profile, TransliterationLossPolicy::Reject).is_err());
    }

    #[test]
    fn unicode_composites_and_source_allographs_have_explicit_fidelity() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        for cyrillic in ["Й", "й", "Ѷ", "ѷ"] {
            let forward = realize_glagolitic(cyrillic, profile, TransliterationLossPolicy::Reject)
                .expect("canonical Unicode decomposition");
            assert_eq!(forward.fidelity(), TransliterationFidelity::Reversible);
            assert_eq!(
                transliterate_glagolitic_to_cyrillic(
                    forward.text(),
                    profile,
                    TransliterationLossPolicy::Reject,
                )
                .expect("canonical Unicode recomposition")
                .text(),
                cyrillic
            );
        }

        for (cyrillic, glagolitic) in [("Ї", "Ⰺ"), ("ї", "ⰺ")] {
            let forward = realize_glagolitic(cyrillic, profile, TransliterationLossPolicy::Report)
                .expect("Polivanova source allograph");
            assert_eq!(forward.text(), glagolitic);
            assert_eq!(forward.fidelity(), TransliterationFidelity::LossReported);
            assert!(
                realize_glagolitic(cyrillic, profile, TransliterationLossPolicy::Reject,).is_err()
            );
        }
    }

    #[test]
    fn productive_cyrillic_inventory_has_no_silent_glagolitic_holes() {
        // Union of letters emitted by the rule kernel. Test-only hostile input
        // examples and manuscript-only combining marks are intentionally absent.
        let productive = "АЗИЪабвгдежзиклмнопрстуфхцчшщъыьюєѕіѡѣѥѧѩѫѭѵꙃꙋꙍꙑꙗꙙ";
        let realized = realize_glagolitic(
            productive,
            GlagoliticProfile::Jagic1879NormalizedOcs,
            TransliterationLossPolicy::Report,
        )
        .expect("every productive letter is mapped or explicitly normalized");
        assert_eq!(detect_script(realized.text()), Script::Glagolitic);
        assert_eq!(realized.fidelity(), TransliterationFidelity::LossReported);
    }

    #[test]
    fn late_or_mixed_script_input_never_falls_through() {
        let profile = GlagoliticProfile::Jagic1879NormalizedOcs;
        for input in ["", "слоword"] {
            let result = std::panic::catch_unwind(|| {
                realize_glagolitic(input, profile, TransliterationLossPolicy::Report)
            });
            assert!(result.is_ok(), "panicked for {input:?}");
            assert!(result.expect("no panic").is_err(), "accepted {input:?}");
        }
        for input in ["ⱄⰾⱁⰬⱁ", "ⰰ\u{1e000}", "ⰰ\u{486}"] {
            assert!(
                realize_glagolitic(input, profile, TransliterationLossPolicy::Report).is_err(),
                "accepted out-of-profile Glagolitic input {input:?}"
            );
        }
        assert!(
            transliterate_glagolitic_to_cyrillic(
                "ⱄⰾⱁⰬⱁ",
                profile,
                TransliterationLossPolicy::Report,
            )
            .is_err()
        );
        assert!(
            transliterate_glagolitic_to_cyrillic(
                "ⰰ\u{1e000}",
                profile,
                TransliterationLossPolicy::Report,
            )
            .is_err()
        );
    }
}
