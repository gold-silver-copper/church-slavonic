//! Pins the Synodal family participle citation machinery — which supplies
//! whole stems and strips hardcoded suffixes rather than reading the merged
//! kernel directly — to the kernel's Synodal columns, so the two copies of
//! each formation cannot drift (docs/UNIFIED_LANGUAGE_PROMPT.md, phase-4
//! verb slice; `unmerged:verb:participle-stem-supply`).

use super::participle::{ActiveParticipleShortFormation, active_participle_citation_variants};
use crate::Gender;
use church_slavonic_core::Recension;
use church_slavonic_core::verb_participle::{
    PastActiveFormation, PresentActiveFormation, past_active_nominative_edge,
    past_active_oblique_suffix, present_active_nominative_edge, present_active_oblique_suffix,
};

const SYN: Recension = Recension::SynodalRussian;

/// Each family formation, its kernel counterpart, and a sample base.
fn present_cases() -> Vec<(
    ActiveParticipleShortFormation,
    PresentActiveFormation,
    &'static str,
)> {
    vec![
        (
            ActiveParticipleShortFormation::PresentFirstUnpalatalized,
            PresentActiveFormation::HardUsht,
            "нес",
        ),
        (
            ActiveParticipleShortFormation::PresentFirstPalatalized,
            PresentActiveFormation::IotatedUsht,
            "зна",
        ),
        (
            ActiveParticipleShortFormation::PresentSecond,
            PresentActiveFormation::SoftAsht,
            "хвал",
        ),
        (
            ActiveParticipleShortFormation::PresentAfterSibilant,
            PresentActiveFormation::SibilantAsht,
            "слыш",
        ),
    ]
}

fn past_cases() -> Vec<(
    ActiveParticipleShortFormation,
    PastActiveFormation,
    &'static str,
)> {
    vec![
        (
            ActiveParticipleShortFormation::PastConsonant,
            PastActiveFormation::ConsonantHard,
            "нес",
        ),
        (
            ActiveParticipleShortFormation::PastVowel,
            PastActiveFormation::Vowel,
            "бы",
        ),
        (
            ActiveParticipleShortFormation::PastIotated,
            PastActiveFormation::SynodalIotated,
            "вожд",
        ),
    ]
}

#[test]
fn present_active_citation_edges_match_the_kernel_columns() {
    for (family, kernel, base) in present_cases() {
        let suffix = present_active_oblique_suffix(kernel, SYN)[0];
        let stem = format!("{base}{suffix}");
        let variants = active_participle_citation_variants(&stem, family, Gender::Masculine)
            .expect("citation variants")
            .expect("masculine citation variants");
        let expected: Vec<String> = present_active_nominative_edge(kernel, SYN)
            .iter()
            .map(|edge| format!("{base}{edge}"))
            .collect();
        assert_eq!(variants, expected, "{family:?}");
    }
}

#[test]
fn past_active_citation_edges_match_the_kernel_columns() {
    for (family, kernel, base) in past_cases() {
        let suffix = past_active_oblique_suffix(kernel, SYN)[0];
        let stem = format!("{base}{suffix}");
        let variants = active_participle_citation_variants(&stem, family, Gender::Masculine)
            .expect("citation variants")
            .expect("masculine citation variants");
        let expected: Vec<String> = past_active_nominative_edge(kernel, SYN)
            .iter()
            .map(|edge| format!("{base}{edge}"))
            .collect();
        assert_eq!(variants, expected, "{family:?}");
    }
}
