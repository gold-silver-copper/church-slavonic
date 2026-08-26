//! Lossless display normalization, conservative lookup keys, and explicit
//! source-bounded script realization.
//!
//! The recension-agnostic primitives and the Glagolitic transliteration
//! engine live in the `church-slavonic-orthography` crate
//! (docs/REWRITE_PLAN.md, target layout). This module re-exports the shared
//! vocabulary unchanged and keeps thin adapters so the family API — its
//! `InflectionError` values and `RuleStep` provenance — is exactly what it
//! was before the extraction.

use crate::{InflectionError, RuleId, RuleStep};
use church_slavonic_orthography::glagolitic::{self, GlagoliticError};
use church_slavonic_orthography::text;
use core::{fmt, ops::Deref};
use unicode_normalization::char::is_combining_mark;

pub use church_slavonic_orthography::glagolitic::{
    GlagoliticProfile, TransliterationDirection, TransliterationFidelity, TransliterationLoss,
    TransliterationLossKind, TransliterationLossPolicy,
};
pub use church_slavonic_orthography::text::{MAX_INPUT_CHARS, Script, detect_script};

fn from_invalid_word(error: text::InvalidWord) -> InflectionError {
    InflectionError::InvalidInput {
        reason: error.reason,
    }
}

fn from_glagolitic_error(error: GlagoliticError) -> InflectionError {
    match error {
        GlagoliticError::InvalidInput { reason } => InflectionError::InvalidInput { reason },
        GlagoliticError::InvalidLemma { input, reason } => {
            InflectionError::InvalidLemma { input, reason }
        }
        GlagoliticError::Unrepresentable {
            input,
            profile,
            character,
            scalar_index,
            reason,
        } => InflectionError::UnrepresentableOrthography {
            input,
            profile,
            character,
            scalar_index,
            reason,
        },
    }
}

/// A source-bounded script realization with enough information to keep a
/// normalization from being mistaken for an exact spelling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransliteratedForm {
    text: String,
    profile: GlagoliticProfile,
    direction: TransliterationDirection,
    fidelity: TransliterationFidelity,
    losses: Vec<TransliterationLoss>,
    trace: Vec<RuleStep>,
}

impl TransliteratedForm {
    fn from_engine(realized: glagolitic::Transliteration) -> Self {
        Self {
            text: realized.text().to_string(),
            profile: realized.profile(),
            direction: realized.direction(),
            fidelity: realized.fidelity(),
            losses: realized.losses().to_vec(),
            trace: realized
                .steps()
                .iter()
                .map(|step| RuleStep {
                    rule_id: RuleId::OrthographyGlagoliticJagic,
                    before: step.before.clone(),
                    after: step.after.clone(),
                    reason: step.reason,
                })
                .collect(),
        }
    }

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

    pub fn trace(&self) -> &[RuleStep] {
        &self.trace
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
    pub fn parse(input: &str) -> Result<Self, InflectionError> {
        let text = canonical_display(input).map_err(|error| match error {
            InflectionError::InvalidInput { reason } => {
                InflectionError::invalid_lemma(input, reason)
            }
            other => other,
        })?;
        let mut has_base = false;
        for ch in text.chars() {
            if ch.is_alphabetic() {
                has_base = true;
            } else if is_combining_mark(ch) {
                if !has_base {
                    return Err(InflectionError::invalid_lemma(
                        input,
                        "a combining mark must follow a lemma letter",
                    ));
                }
            } else {
                return Err(InflectionError::invalid_lemma(
                    input,
                    format!("the lemma contains a non-letter character {ch:?}"),
                ));
            }
        }
        let script = detect_script(&text);
        match script {
            Script::Cyrillic | Script::Glagolitic => Ok(Self { text, script }),
            Script::Mixed => Err(InflectionError::invalid_lemma(
                input,
                "the lemma mixes Cyrillic, Glagolitic, Latin, or another script",
            )),
            Script::Latin => Err(InflectionError::invalid_lemma(
                input,
                "the lemma is Latin; expected Old Church Slavonic Cyrillic or Glagolitic",
            )),
            Script::Unknown => Err(InflectionError::invalid_lemma(
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

pub fn canonical_display(input: &str) -> Result<String, InflectionError> {
    text::canonical_display(input).map_err(from_invalid_word)
}

pub fn lookup_key(input: &str) -> Result<String, InflectionError> {
    text::lookup_key(input).map_err(from_invalid_word)
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
) -> Result<TransliteratedForm, InflectionError> {
    glagolitic::realize_glagolitic(input, profile, loss_policy)
        .map(TransliteratedForm::from_engine)
        .map_err(from_glagolitic_error)
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
) -> Result<TransliteratedForm, InflectionError> {
    glagolitic::transliterate_glagolitic_to_cyrillic(input, profile, loss_policy)
        .map(TransliteratedForm::from_engine)
        .map_err(from_glagolitic_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_is_nfc_and_lowercase_without_stripping_marks() {
        let decomposed = "А\u{301}ЗЪ";
        assert_eq!(lookup_key(decomposed).expect("valid OCS"), "а\u{301}зъ");
        assert_eq!(lookup_key("цар҄ь").expect("valid OCS"), "цар҄ь");
        assert_eq!(lookup_key("И\u{306}").expect("decomposed breve"), "й");
        assert_eq!(
            lookup_key("а\u{315}\u{301}").expect("valid combining marks"),
            lookup_key("а\u{301}\u{315}").expect("canonical mark order")
        );
    }

    #[test]
    fn scripts_are_distinguished_without_transliteration() {
        assert_eq!(detect_script("слово"), Script::Cyrillic);
        assert_eq!(detect_script("ⱄⰾⱁⰲⱁ"), Script::Glagolitic);
        assert_eq!(detect_script("slovo"), Script::Latin);
    }

    #[test]
    fn hostile_inputs_are_typed_errors() {
        assert!(lookup_key("").is_err());
        assert!(lookup_key("два слова").is_err());
        assert!(lookup_key("слово\0").is_err());
        assert!(lookup_key(&"x".repeat(MAX_INPUT_CHARS + 1)).is_err());
        assert_eq!(lookup_key(".").expect("punctuation is lossless"), ".");
        assert_eq!(
            lookup_key("LATIN").expect("non-OCS is not guessed"),
            "latin"
        );
    }

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
        assert_eq!(
            glagolitic.trace()[0].rule_id,
            RuleId::OrthographyGlagoliticJagic
        );

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
            Err(InflectionError::UnrepresentableOrthography {
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
        assert!(realized.trace().is_empty());
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
            Err(InflectionError::UnrepresentableOrthography {
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
