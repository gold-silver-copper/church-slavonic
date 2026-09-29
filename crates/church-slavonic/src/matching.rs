//! Explicit comparison policies. A tolerant match never establishes identity.

use unicode_normalization::UnicodeNormalization;

/// Each policy names all the transformations it permits. The legacy policy is
/// retained for migration diagnostics, not as a statement of linguistic equality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchPolicy {
    Exact,
    UnicodeEquivalent,
    CaseInsensitive,
    /// NFC, lowercasing, and removal of acute/grave/kamora/circumflex accents.
    /// Breathing and abbreviation marks remain significant.
    AccentInsensitive,
    /// The old shared OCS/Synodal print projection, including letter folds.
    LegacyOrthographic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transformation {
    UnicodeNormalization,
    Lowercase,
    RemoveAccent,
    LegacyPrintProjection,
}

/// An auditable change to one side of a comparison.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchStep {
    pub transformation: Transformation,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchTrace {
    pub policy: MatchPolicy,
    pub query_steps: Vec<MatchStep>,
    pub generated_steps: Vec<MatchStep>,
}

fn normalize(text: &str, policy: MatchPolicy, trace: bool) -> (String, Vec<MatchStep>) {
    let mut value = text.to_string();
    let mut steps = Vec::new();
    fn step(value: &mut String, steps: &mut Vec<MatchStep>, kind: Transformation, next: String, trace: bool) {
        if *value != next {
            if trace {
                steps.push(MatchStep { transformation: kind, before: value.clone(), after: next.clone() });
            }
            *value = next;
        }
    }
    if policy == MatchPolicy::Exact {
        return (value, steps);
    }
    step(&mut value, &mut steps, Transformation::UnicodeNormalization, text.nfc().collect(), trace);
    if matches!(policy, MatchPolicy::CaseInsensitive | MatchPolicy::AccentInsensitive | MatchPolicy::LegacyOrthographic) {
        let next = value.to_lowercase().nfc().collect();
        step(&mut value, &mut steps, Transformation::Lowercase, next, trace);
    }
    if matches!(policy, MatchPolicy::AccentInsensitive | MatchPolicy::LegacyOrthographic) {
        let next = value.nfd().filter(|c| !matches!(*c, '\u{300}' | '\u{301}' | '\u{311}' | '\u{302}')).nfc().collect();
        step(&mut value, &mut steps, Transformation::RemoveAccent, next, trace);
    }
    if policy == MatchPolicy::LegacyOrthographic {
        let next = crate::orthography::comparison_key(&value);
        step(&mut value, &mut steps, Transformation::LegacyPrintProjection, next, trace);
    }
    (value, steps)
}

/// Compare without altering either input. Only successful comparisons return
/// a trace; empty traces mean the bytes already agreed under the requested policy.
pub fn compare(query: &str, generated: &str, policy: MatchPolicy) -> Option<MatchTrace> {
    let (q, query_steps) = normalize(query, policy, true);
    let (g, generated_steps) = normalize(generated, policy, true);
    (q == g).then_some(MatchTrace { policy, query_steps, generated_steps })
}

/// The retrieval key uses exactly the same operations as comparison.
pub(crate) fn key(text: &str, policy: MatchPolicy) -> String {
    normalize(text, policy, false).0
}

impl MatchPolicy {
    pub(crate) fn slot(self) -> usize {
        match self {
            Self::Exact => 0,
            Self::UnicodeEquivalent => 1,
            Self::CaseInsensitive => 2,
            Self::AccentInsensitive => 3,
            Self::LegacyOrthographic => 4,
        }
    }
}
