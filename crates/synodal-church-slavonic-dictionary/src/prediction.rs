//! The exploratory predictive tier (v0.12 phase 3).
//!
//! A surface with no registry reading gets a *typed, lower-confidence*
//! analysis by segmentation against the ending inventories the reviewed
//! grammar already licenses (Alypy §§82–97), instead of no analysis at all.
//! Predictions are corpus-free and deterministic: the runtime scores a split
//! only by its morphological shape; corpus sibling-cell support is added by
//! `cargo xtask synodal-predict`, which also measures the tier's precision by
//! masking reviewed lexemes and re-deriving their surfaces.
//!
//! Walls (the coverage contract cannot mistake this for evidence):
//!
//! - a prediction is its own type, never an [`crate::Analysis`];
//! - the strict and productive resolvers never consult this module;
//! - `is_top_k_analyzed` never sees a prediction, and no sealed floor reads
//!   the predicted slice;
//! - a prediction promoted to a reviewed row passes every admission rule; its
//!   origin shortcuts nothing.

use serde::{Deserialize, Serialize};
use synodal_church_slavonic::{
    FiniteTense, FiniteVerbCell, GrammarCell, ImperativeCell, LParticipleCell,
};
use synodal_church_slavonic_core::{
    Gender, Number, Person, normalize_lookup_accentless, reflexive_base_candidates,
};

/// The model identifier every segmentation prediction carries.
pub const SEGMENTATION_MODEL: &str = "SYN-PREDICT-VERB-SEGMENTATION-V1";

/// One typed, unreviewed reading of an unknown surface.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    /// The surface the prediction explains, in the accentless lookup key.
    pub surface: String,
    /// The hypothesised stem the ending was removed from.
    pub stem: String,
    /// The licensed ending that was matched.
    pub ending: &'static str,
    /// The grammatical cell the ending realises.
    pub cell: GrammarCell,
    /// The conjugation-class requirement the ending carries.
    pub class: &'static str,
    /// A conservative confidence in basis points. Corpus-free: shape only.
    pub confidence_bp: u16,
    /// The §73 enclitic was stripped before segmentation.
    pub reflexive: bool,
    /// The model that produced this reading.
    pub model: &'static str,
}

struct EndingRow {
    ending: &'static str,
    cell: GrammarCell,
    class: &'static str,
    /// Base confidence: longer, more distinctive endings score higher.
    confidence_bp: u16,
}

const fn finite(tense: FiniteTense, person: Person, number: Number) -> GrammarCell {
    GrammarCell::FiniteVerb(FiniteVerbCell {
        tense,
        person,
        number,
    })
}

/// The licensed verbal ending inventory (Alypy §§82, 86, 87, 93, 97).
/// Present/future endings carry their conjugation; past endings attach to
/// aorist or imperfect bases whose class the split does not fix.
fn ending_rows() -> &'static [EndingRow] {
    use FiniteTense::{Aorist, Imperfect, Present};
    use Number::{Plural, Singular};
    use Person::{First, Second, Third};
    static ROWS: std::sync::OnceLock<Vec<EndingRow>> = std::sync::OnceLock::new();
    ROWS.get_or_init(|| {
        let mut rows = vec![
            // §86 vowel aorist.
            EndingRow {
                ending: "хъ",
                cell: finite(Aorist, First, Singular),
                class: "vowel-aorist",
                confidence_bp: 3200,
            },
            EndingRow {
                ending: "ша",
                cell: finite(Aorist, Third, Plural),
                class: "vowel-aorist",
                confidence_bp: 3600,
            },
            EndingRow {
                ending: "сте",
                cell: finite(Aorist, Second, Plural),
                class: "vowel-aorist",
                confidence_bp: 3400,
            },
            EndingRow {
                ending: "хомъ",
                cell: finite(Aorist, First, Plural),
                class: "vowel-aorist",
                confidence_bp: 3800,
            },
            // §86 consonant aorist.
            EndingRow {
                ending: "охъ",
                cell: finite(Aorist, First, Singular),
                class: "consonant-aorist",
                confidence_bp: 3600,
            },
            EndingRow {
                ending: "оша",
                cell: finite(Aorist, Third, Plural),
                class: "consonant-aorist",
                confidence_bp: 3800,
            },
            EndingRow {
                ending: "осте",
                cell: finite(Aorist, Second, Plural),
                class: "consonant-aorist",
                confidence_bp: 3800,
            },
            EndingRow {
                ending: "охомъ",
                cell: finite(Aorist, First, Plural),
                class: "consonant-aorist",
                confidence_bp: 4000,
            },
            EndingRow {
                ending: "е",
                cell: finite(Aorist, Third, Singular),
                class: "consonant-aorist",
                confidence_bp: 2200,
            },
            // §87 imperfect (the а/ѧ of the base stays in the stem).
            EndingRow {
                ending: "ше",
                cell: finite(Imperfect, Third, Singular),
                class: "imperfect",
                confidence_bp: 3000,
            },
            EndingRow {
                ending: "хꙋ",
                cell: finite(Imperfect, Third, Plural),
                class: "imperfect",
                confidence_bp: 3400,
            },
            // §§80–82 present / simple future.
            EndingRow {
                ending: "етъ",
                cell: finite(Present, Third, Singular),
                class: "first-conjugation",
                confidence_bp: 3200,
            },
            EndingRow {
                ending: "еши",
                cell: finite(Present, Second, Singular),
                class: "first-conjugation",
                confidence_bp: 3200,
            },
            EndingRow {
                ending: "емъ",
                cell: finite(Present, First, Plural),
                class: "first-conjugation",
                confidence_bp: 3000,
            },
            EndingRow {
                ending: "ете",
                cell: finite(Present, Second, Plural),
                class: "first-conjugation",
                confidence_bp: 3000,
            },
            EndingRow {
                ending: "ꙋтъ",
                cell: finite(Present, Third, Plural),
                class: "first-unpalatalized",
                confidence_bp: 3400,
            },
            EndingRow {
                ending: "ютъ",
                cell: finite(Present, Third, Plural),
                class: "first-palatalized",
                confidence_bp: 3400,
            },
            EndingRow {
                ending: "итъ",
                cell: finite(Present, Third, Singular),
                class: "second",
                confidence_bp: 3200,
            },
            EndingRow {
                ending: "иши",
                cell: finite(Present, Second, Singular),
                class: "second",
                confidence_bp: 3200,
            },
            EndingRow {
                ending: "имъ",
                cell: finite(Present, First, Plural),
                class: "second",
                confidence_bp: 2800,
            },
            EndingRow {
                ending: "ите",
                cell: finite(Present, Second, Plural),
                class: "second",
                confidence_bp: 2800,
            },
            EndingRow {
                ending: "ѧтъ",
                cell: finite(Present, Third, Plural),
                class: "second",
                confidence_bp: 3400,
            },
            EndingRow {
                ending: "атъ",
                cell: finite(Present, Third, Plural),
                class: "second-after-sibilant",
                confidence_bp: 3000,
            },
            // §93 imperative (the i-suffix; plural shares -ите with the
            // second-conjugation present and stays ambiguous by design).
            EndingRow {
                ending: "и",
                cell: GrammarCell::Imperative(ImperativeCell {
                    person: Second,
                    number: Singular,
                }),
                class: "imperative",
                confidence_bp: 2000,
            },
            // §97 l-participle.
            EndingRow {
                ending: "лъ",
                cell: GrammarCell::LParticiple(LParticipleCell {
                    gender: Gender::Masculine,
                    number: Singular,
                }),
                class: "l-participle",
                confidence_bp: 3200,
            },
            EndingRow {
                ending: "ла",
                cell: GrammarCell::LParticiple(LParticipleCell {
                    gender: Gender::Feminine,
                    number: Singular,
                }),
                class: "l-participle",
                confidence_bp: 2400,
            },
            EndingRow {
                ending: "ли",
                cell: GrammarCell::LParticiple(LParticipleCell {
                    gender: Gender::Masculine,
                    number: Plural,
                }),
                class: "l-participle",
                confidence_bp: 2400,
            },
            // §79 infinitive.
            EndingRow {
                ending: "ти",
                cell: GrammarCell::Infinitive,
                class: "infinitive",
                confidence_bp: 2600,
            },
        ];
        rows.sort_by_key(|row| std::cmp::Reverse(row.ending.chars().count()));
        rows
    })
}

fn is_cyrillic_letter(character: char) -> bool {
    matches!(
        character,
        'а'..='я'
            | 'ѣ'
            | 'ѡ'
            | 'ѧ'
            | 'ѫ'
            | 'ꙋ'
            | 'ї'
            | 'і'
            | 'є'
            | 'ѵ'
            | 'ѳ'
            | 'ѕ'
            | 'ѯ'
            | 'ѱ'
            | 'ѿ'
            | 'ꙗ'
            | 'ѩ'
            | 'ѭ'
    )
}

/// A stem the licensed endings could attach to: at least two letters, all
/// Cyrillic, and not itself ending in a jer.
fn admissible_stem(stem: &str) -> bool {
    let mut count = 0;
    for character in stem.chars() {
        if !is_cyrillic_letter(character) {
            return false;
        }
        count += 1;
    }
    count >= 2 && !stem.ends_with(['ъ', 'ь'])
}

/// Typed, unreviewed readings of an unknown surface by segmentation against
/// the licensed verbal endings. Deterministic and corpus-free; ordered by
/// confidence, then by ending length.
#[must_use]
pub fn predict(surface: &str) -> Vec<Prediction> {
    let key = normalize_lookup_accentless(surface);
    let mut output = Vec::new();
    let mut hosts = vec![(key.clone(), false)];
    for host in reflexive_base_candidates(&key) {
        hosts.push((host, true));
    }
    for (host, reflexive) in hosts {
        for row in ending_rows() {
            let Some(stem) = host.strip_suffix(row.ending) else {
                continue;
            };
            if !admissible_stem(stem) {
                continue;
            }
            output.push(Prediction {
                surface: key.clone(),
                stem: stem.to_owned(),
                ending: row.ending,
                cell: row.cell,
                class: row.class,
                confidence_bp: row.confidence_bp,
                reflexive,
                model: SEGMENTATION_MODEL,
            });
        }
    }
    output.sort_by(|left, right| {
        right
            .confidence_bp
            .cmp(&left.confidence_bp)
            .then_with(|| right.ending.len().cmp(&left.ending.len()))
            .then_with(|| left.stem.cmp(&right.stem))
    });
    output
}

/// The policy wall: predictions are reachable only under
/// [`GenerationPolicy::Exploratory`]. Under `Strict` and `Productive` the
/// surface stays unresolved and this returns nothing.
#[must_use]
pub fn predict_under(
    policy: synodal_church_slavonic::GenerationPolicy,
    surface: &str,
) -> Vec<Prediction> {
    if policy == synodal_church_slavonic::GenerationPolicy::Exploratory {
        predict(surface)
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segmentation_reads_a_vowel_aorist_and_its_reflexive() {
        let plain = predict("сотвориша");
        assert!(plain.iter().any(|prediction| {
            prediction.stem == "сотвори"
                && prediction.ending == "ша"
                && matches!(prediction.cell, GrammarCell::FiniteVerb(cell) if cell.tense == FiniteTense::Aorist)
                && !prediction.reflexive
        }));
        let reflexive = predict("возврати́шасѧ");
        assert!(reflexive.iter().any(|prediction| {
            prediction.stem == "возврати" && prediction.ending == "ша" && prediction.reflexive
        }));
    }

    #[test]
    fn predictions_are_walled_behind_the_exploratory_policy() {
        use synodal_church_slavonic::GenerationPolicy;
        assert!(predict_under(GenerationPolicy::Strict, "сотвориша").is_empty());
        assert!(predict_under(GenerationPolicy::Productive, "сотвориша").is_empty());
        assert!(!predict_under(GenerationPolicy::Exploratory, "сотвориша").is_empty());
    }

    #[test]
    fn segmentation_refuses_non_cyrillic_and_short_stems() {
        assert!(predict("abcша").is_empty());
        assert!(
            !predict("даша")
                .iter()
                .any(|prediction| prediction.stem.len() < 2)
        );
    }
}
