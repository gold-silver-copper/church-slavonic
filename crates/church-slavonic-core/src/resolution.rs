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
//! accent-pattern token picks the STRESS. On the plain-rule paths the
//! token rides inside the accent pass itself (`crate::accent`), so the
//! print's stress-coupled conventions — the wide `ѡ`/`є`, the kamora, the
//! final varia — follow the token's position; on the skeleton-level
//! stem/override paths, whose endings carry no convention marker, it is a
//! bare re-stress applied last. This module
//! is steps 2–3; the runtime facade, the extractor's subtraction and
//! reachability passes, and both dead-weight audits all call it instead of
//! keeping their own copies.

use crate::ChurchSlavonicCore;
use crate::grammar::*;
use crate::schema::{
    ADJ_ACCENT_CELL, NOUN_ACCENT_CELL, NOUN_SHAPE_SOURCE_CELLS, PRESENT_STEM_CELL,
    VERB_ACCENT_CELL, VERB_CLASS_CELL, adj_features, finite_features, noun_features,
    participle_features, participle_stem_cell,
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
    let token = fact(VERB_ACCENT_CELL);
    verb_letters(lemma, recension, cell, fact, token.as_deref())
}

/// Apply an accent-pattern token to a produced form as a bare re-stress:
/// `s<N>` stresses the N-th vowel, `e` the last; anything else leaves the
/// form alone. Only the skeleton-level stem/override paths use this — the
/// rule paths thread the token through the accent pass instead, where the
/// print conventions can follow it.
pub fn apply_accent_pattern(form: &str, token: &str) -> String {
    if form.is_empty() {
        return form.to_string();
    }
    let n = if token == "e" {
        crate::orthography::vowel_count(form).saturating_sub(1)
    } else if let Some(n) = token
        .strip_prefix('s')
        .and_then(|d| d.parse::<usize>().ok())
    {
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
    let token = fact(NOUN_ACCENT_CELL);
    // The accusative-shape fact: a lower stored accusative that is
    // nominative-shaped where the rule answers the genitive shape (an
    // inanimate: `а҆́ггелы`, not `а҆́ггелѡвъ`) teaches this accusative the
    // nominative shape too. Sources derive upward only, so the anchor
    // cell itself always resolves by the plain ladder.
    if *recension == Recension::Synodal && case == Case::Accusative {
        use crate::orthography::comparison_key;
        for src in NOUN_SHAPE_SOURCE_CELLS {
            if src >= cell {
                break;
            }
            let Some(stored) = fact(src) else { continue };
            let (_, src_number) = noun_features(src);
            let nom = ChurchSlavonicCore::noun(lemma, &Case::Nominative, &src_number, recension);
            let acc = ChurchSlavonicCore::noun(lemma, &Case::Accusative, &src_number, recension);
            let key = comparison_key(&stored);
            if key == comparison_key(&nom) && key != comparison_key(&acc) {
                return ChurchSlavonicCore::noun_pattern(
                    lemma,
                    &Case::Nominative,
                    &number,
                    recension,
                    token.as_deref(),
                );
            }
        }
    }
    ChurchSlavonicCore::noun_pattern(lemma, &case, &number, recension, token.as_deref())
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
    let token = fact(ADJ_ACCENT_CELL);
    ChurchSlavonicCore::adj_pattern(
        lemma,
        &case,
        &number,
        &gender,
        &degree,
        recension,
        token.as_deref(),
    )
}

/// The letter-level resolution with the accent-pattern token in its right
/// seat: the plain-rule paths thread it through the accent pass (the one
/// copy of the stress-coupled print conventions in `crate::accent`), while
/// the skeleton-level stem/override paths — whose stored stems carry their
/// own accents and whose endings carry no convention marker — post-apply
/// it as a bare re-stress, as before.
fn verb_letters(
    lemma: &str,
    recension: &Recension,
    cell: usize,
    fact: &dyn Fn(usize) -> Option<String>,
    pattern: Option<&str>,
) -> String {
    if cell >= 542 {
        return String::new();
    }
    let reaccent = |form: String| match pattern {
        Some(token) => apply_accent_pattern(&form, token),
        None => form,
    };
    if (38..542).contains(&cell) {
        let (voice, series, past, gender, number, case) = participle_features(cell);
        let tense = if past { Tense::Aorist } else { Tense::Present };
        if let Some(stem) = fact(participle_stem_cell(&voice, &tense))
            && let Some(form) = ChurchSlavonicCore::participle_from_stem(
                &stem, past, &voice, &series, &case, &number, &gender, recension,
            )
        {
            return reaccent(form);
        }
    }
    let class = fact(VERB_CLASS_CELL);
    let present = fact(PRESENT_STEM_CELL);
    if class.is_some() || present.is_some() {
        if cell < 38 {
            let (person, number, tense, form) = finite_features(cell);
            return reaccent(ChurchSlavonicCore::verb_from_stems(
                lemma,
                class.as_deref(),
                present.as_deref(),
                &person,
                &number,
                &tense,
                &form,
                recension,
            ));
        }
        let (voice, series, past, gender, number, case) = participle_features(cell);
        let tense = if past { Tense::Aorist } else { Tense::Present };
        return reaccent(ChurchSlavonicCore::participle_with_override(
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
        ));
    }
    if cell < 38 {
        let (person, number, tense, form) = finite_features(cell);
        ChurchSlavonicCore::verb_pattern(lemma, &person, &number, &tense, &form, recension, pattern)
    } else {
        let (voice, series, past, gender, number, case) = participle_features(cell);
        let tense = if past { Tense::Aorist } else { Tense::Present };
        ChurchSlavonicCore::participle_pattern(
            lemma, &tense, &voice, &series, &case, &number, &gender, recension, pattern,
        )
    }
}
