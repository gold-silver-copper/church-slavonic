//! The ONE copy of the fact-resolution order. A published verb row can
//! carry derived facts — the four participle stems (542..546), the
//! present-stem override (546) and the conjugation-class override (547) —
//! and everything that answers a cell walks the same ladder:
//!
//! 1. the row's own exact cell, then the bare row's exact cell (the call
//!    sites own these lookups — `attested_cell` in the runtime, the
//!    subtraction/reachability loops in the extractor);
//! 2. the facts, each read own-else-bare per cell: for a declined
//!    participle cell, the block's participle stem expanded through
//!    [`ChurchSlavonicCore::participle_from_stem`]; then the
//!    class/present-stem override re-running the rule
//!    ([`ChurchSlavonicCore::verb_from_stems`] /
//!    [`ChurchSlavonicCore::participle_with_override`]);
//! 3. the plain rule.
//!
//! Facts compose as: stems and the override pick the LETTERS, the
//! accent-pattern token picks the STRESS — it re-accents whatever the
//! letter-level resolution produced, always last. This module
//! is steps 2–3; the runtime facade, the extractor's subtraction and
//! reachability passes, and both dead-weight audits all call it instead of
//! keeping their own copies.

use crate::ChurchSlavonicCore;
use crate::grammar::*;
use crate::schema::{
    ADJ_ACCENT_CELL, NOUN_ACCENT_CELL, PRESENT_STEM_CELL, VERB_ACCENT_CELL, VERB_CLASS_CELL,
    adj_features, finite_features, noun_features, participle_features, participle_stem_cell,
};

/// Compose two per-cell accessors into the own-else-bare read every fact
/// cell uses (mirroring the runtime's `attested_cell` fallback).
pub fn own_else_bare<'a>(
    own: impl Fn(usize) -> Option<String> + 'a,
    bare: impl Fn(usize) -> Option<String> + 'a,
) -> impl Fn(usize) -> Option<String> + 'a {
    move |i| own(i).or_else(|| bare(i))
}

/// What a verb key answers for `cell` once its exact cells (own and bare)
/// have missed: the facts, then the plain rule. `fact` is the own-else-bare
/// accessor over the row's fact cells; the fact cells themselves resolve to
/// the empty string (they are not forms).
pub fn verb_fact_fallback(
    lemma: &str,
    recension: &Recension,
    cell: usize,
    fact: &dyn Fn(usize) -> Option<String>,
) -> String {
    let letters = verb_letters(lemma, recension, cell, fact);
    match fact(VERB_ACCENT_CELL) {
        Some(token) => apply_accent_pattern(&letters, &token),
        None => letters,
    }
}

/// Apply an accent-pattern token to a produced form: `s<N>` stresses the
/// N-th vowel, `e` the last; anything else leaves the form alone.
pub fn apply_accent_pattern(form: &str, token: &str) -> String {
    if form.is_empty() {
        return form.to_string();
    }
    let n = if token == "e" {
        crate::orthography::vowel_count(form).saturating_sub(1)
    } else if let Some(n) = token.strip_prefix('s').and_then(|d| d.parse::<usize>().ok()) {
        n
    } else {
        return form.to_string();
    };
    crate::orthography::stress(form, n, false)
}

/// What a noun key answers once its exact cells miss: the plain rule,
/// re-accented by the row's accent-pattern token if it carries one.
pub fn noun_fact_fallback(
    lemma: &str,
    recension: &Recension,
    cell: usize,
    fact: &dyn Fn(usize) -> Option<String>,
) -> String {
    if cell >= 21 {
        return String::new();
    }
    let (case, number) = noun_features(cell);
    let letters = ChurchSlavonicCore::noun(lemma, &case, &number, recension);
    match fact(NOUN_ACCENT_CELL) {
        Some(token) => apply_accent_pattern(&letters, &token),
        None => letters,
    }
}

/// What an adjective key answers once its exact cells miss.
pub fn adj_fact_fallback(
    lemma: &str,
    recension: &Recension,
    cell: usize,
    fact: &dyn Fn(usize) -> Option<String>,
) -> String {
    if cell >= 126 {
        return String::new();
    }
    let (case, number, gender, degree) = adj_features(cell);
    let letters = ChurchSlavonicCore::adj(lemma, &case, &number, &gender, &degree, recension);
    match fact(ADJ_ACCENT_CELL) {
        Some(token) => apply_accent_pattern(&letters, &token),
        None => letters,
    }
}

fn verb_letters(
    lemma: &str,
    recension: &Recension,
    cell: usize,
    fact: &dyn Fn(usize) -> Option<String>,
) -> String {
    if cell >= 542 {
        return String::new();
    }
    if (38..542).contains(&cell) {
        let (voice, series, past, gender, number, case) = participle_features(cell);
        let tense = if past { Tense::Aorist } else { Tense::Present };
        if let Some(stem) = fact(participle_stem_cell(&voice, &tense))
            && let Some(form) = ChurchSlavonicCore::participle_from_stem(
                &stem, past, &voice, &series, &case, &number, &gender, recension,
            )
        {
            return form;
        }
    }
    let class = fact(VERB_CLASS_CELL);
    let present = fact(PRESENT_STEM_CELL);
    if class.is_some() || present.is_some() {
        if cell < 38 {
            let (person, number, tense, form) = finite_features(cell);
            return ChurchSlavonicCore::verb_from_stems(
                lemma,
                class.as_deref(),
                present.as_deref(),
                &person,
                &number,
                &tense,
                &form,
                recension,
            );
        }
        let (voice, series, past, gender, number, case) = participle_features(cell);
        let tense = if past { Tense::Aorist } else { Tense::Present };
        return ChurchSlavonicCore::participle_with_override(
            lemma,
            class.as_deref(),
            present.as_deref(),
            &tense,
            &voice,
            &series,
            &case,
            &number,
            &gender,
            recension,
        );
    }
    if cell < 38 {
        let (person, number, tense, form) = finite_features(cell);
        ChurchSlavonicCore::verb(lemma, &person, &number, &tense, &form, recension)
    } else {
        let (voice, series, past, gender, number, case) = participle_features(cell);
        let tense = if past { Tense::Aorist } else { Tense::Present };
        ChurchSlavonicCore::participle(
            lemma, &tense, &voice, &series, &case, &number, &gender, recension,
        )
    }
}
