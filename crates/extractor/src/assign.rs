//! Deterministic sense-key assignment.
//!
//! There is NO registry, lockfile, or identity here — key numbers are a pure
//! function of the current sources. For each `(recension, pos, lemma)` the
//! extractor collects the attested paradigms, drops the one the regular rule
//! engine already produces (so the rule serves it at runtime), and this module
//! numbers the survivors:
//!
//! 1. sort them deterministically — standard senses before soft ones, primary
//!    readings before second choices, then by emitted form signature
//!    lexicographically (`Candidate::order_key`);
//! 2. hand out suffixes from `1` (or `2` when a regular pattern was dropped, so
//!    the bare key is reserved for the rule engine): the first survivor gets the
//!    bare lemma or `lemma_2`, the next `lemma_3`, and so on ([`make_key`]).
//!
//! The non-lexicographic tiebreaks are standard-before-soft (a sense the
//! source marks as dialectal must never take the bare key from a standard
//! sibling), primary-before-variant (the shared personal-pronoun row's
//! second-choice alternatives must never take the bare key from the row of
//! its print-arbitrated first choices — the v1.2 finding: the shorter
//! variant row sorted first) and clean-before-noisy (a row storing an
//! accentless Synodal spelling, a transliteration's dropped mark, sorts
//! after the rows without one). Everything else is the plain form sort.
//!
//! # Stability
//!
//! Keys are DETERMINISTIC but NOT immutable. The assignment is reproducible from
//! given sources — reordering their entries can never change it — but if a
//! source adds, removes, or edits a lemma's attested forms, the sort can
//! renumber its keys: a lexicographically earlier new variant deliberately shifts
//! the later ones up. That is the accepted tradeoff of a system with no
//! carry-forward state to maintain — "fairly stable", not frozen. There is no
//! human review, no override file, and no cross-version immutability gate.
//!
//! # Emitted key format (shared with the `church-slavonic` runtime)
//!
//! Suffix 1 emits the bare lemma; `>= 2` emits `lemma_<n>`. The underscore is
//! unambiguous because extraction rejects any lemma containing `_`
//! ([`crate::extract::word_is_proper`]), so a trailing `_<digits>` can only be a
//! sense suffix. The format itself lives in
//! [`church_slavonic_core::sense_key`], the single owner shared with the
//! runtime; [`make_key`]/[`split_key`] are this crate's thin re-exports. The
//! generated tables prefix every key with its recension tag (`ocs:`/`syn:`);
//! that prefix is added by the emitter, after numbering.

/// A distinct attested paradigm for one lemma, awaiting a key.
#[derive(Debug, Clone, Default)]
pub struct Candidate {
    /// The row's cells in schema order (see [`crate::cells`]); an empty string
    /// is a cell the rule serves (unattested, or attested equal to the rule).
    pub forms: Vec<String>,
    /// The same cells before the rule was subtracted (an empty string is an
    /// unattested cell). Not part of the sort: two candidates that emit the
    /// same forms are one candidate.
    pub raw: Vec<String>,
    /// True when EVERY contributing sense is soft (dialectal). Such a candidate
    /// sorts after standard siblings so it can never take the bare key from one.
    pub soft_sense: bool,
    /// True when some contributing observation attested these forms as its
    /// PRIMARY reading (the first alternative of each cell). A primary
    /// candidate sorts before the second-choice rows, so the bare key never
    /// goes to a row of variants merely because it is shorter. The
    /// extractor sets it for the shared personal-pronoun row only, whose
    /// primaries the print arbitrates; see `extract::finalize`.
    pub primary: bool,
    /// How many of the row's stored Synodal forms carry no stress mark at
    /// all — a transliteration's dropped accent («всякую» once beside
    /// «всѧ́кꙋю» 227 times), never the print's. Such a row sorts after
    /// the clean rows, so noise never takes the bare key (v1.2 part 4).
    pub noise: usize,
}

impl Candidate {
    pub fn new(forms: Vec<String>) -> Self {
        Candidate {
            raw: forms.clone(),
            forms,
            soft_sense: false,
            primary: false,
            noise: 0,
        }
    }

    fn sig(&self) -> String {
        forms_sig(&self.forms)
    }

    /// The deterministic total order used to hand out suffixes: standard-before-
    /// soft, primary-before-variant, then the emitted form signature
    /// lexicographically. It is a pure function of the candidate's forms
    /// (softness and primacy included), so the output is invariant under any
    /// permutation of the sources' entry order.
    fn order_key(&self) -> (bool, bool, usize, String) {
        (self.soft_sense, !self.primary, self.noise, self.sig())
    }
}

/// One emitted key plus the forms published under it (and the attested
/// forms before the rule was subtracted, for the variant-row trimming in
/// `extract::finalize`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment {
    pub key: String,
    pub forms: Vec<String>,
    pub raw: Vec<String>,
}

/// Number a lemma's surviving candidates into deterministic keys.
///
/// `had_regular` records whether a regular-prediction-equal pattern was dropped
/// upstream (the rule engine serves it at runtime): when it was, the bare key is
/// reserved for the rule and numbering starts at `_2`; otherwise the first
/// candidate takes the bare lemma. Candidates are sorted by
/// `Candidate::order_key`, so the result depends only on the set of surviving
/// forms — never on source order.
pub fn assign(lemma: &str, mut candidates: Vec<Candidate>, had_regular: bool) -> Vec<Assignment> {
    candidates.sort_by_key(|c| c.order_key());
    let base = if had_regular { 2 } else { 1 };
    candidates
        .into_iter()
        .enumerate()
        .map(|(i, c)| Assignment {
            key: make_key(lemma, base + i as u32),
            forms: c.forms,
            raw: c.raw,
        })
        .collect()
}

/// The emitted key format lives in [`church_slavonic_core::sense_key`] — the
/// single owner shared with the runtime.
pub use church_slavonic_core::sense_key::make_key;

/// Decode an emitted key into `(base_lemma, suffix)`, or `None` for a bare key.
/// An overflowing digit run is a hard error (panic), never a silent coercion —
/// such a key can only come from a corrupt generated table.
pub fn split_key(key: &str) -> Option<(&str, u32)> {
    let (base, digits) = church_slavonic_core::sense_key::split(key)?;
    let suffix = digits
        .parse()
        .unwrap_or_else(|_| panic!("sense suffix overflows u32 in key {key:?}"));
    Some((base, suffix))
}

/// The canonical form-signature encoding (`forms.join("|")`). `Candidate::sig`,
/// the extract-layer dedup, and the regular drop all funnel through this, so two
/// patterns merge exactly when they emit identical forms.
pub fn forms_sig(forms: &[String]) -> String {
    forms.join("|")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(forms: &[&str]) -> Candidate {
        Candidate::new(forms.iter().map(|s| s.to_string()).collect())
    }

    fn soft(forms: &[&str]) -> Candidate {
        let mut c = cand(forms);
        c.soft_sense = true;
        c
    }

    fn keys(a: &[Assignment]) -> Vec<String> {
        a.iter().map(|x| x.key.clone()).collect()
    }

    #[test]
    fn reordering_candidates_yields_identical_keys() {
        let forward = assign("градъ", vec![cand(&["града"]), cand(&["градоу"])], false);
        let reversed = assign("градъ", vec![cand(&["градоу"]), cand(&["града"])], false);
        assert_eq!(forward, reversed);
        assert_eq!(keys(&forward), ["градъ", "градъ_2"]);
        assert_eq!(forward[0].forms, ["града"]);
    }

    #[test]
    fn dropped_regular_reserves_the_bare_key() {
        let a = assign("сꙑнъ", vec![cand(&["сꙑнови"])], true);
        assert_eq!(keys(&a), ["сꙑнъ_2"]);
    }

    #[test]
    fn a_lexicographically_earlier_candidate_renumbers_the_later_ones() {
        let before = assign("x", vec![cand(&["м"]), cand(&["я"])], false);
        assert_eq!(before[0].forms, ["м"]);
        let after = assign("x", vec![cand(&["м"]), cand(&["я"]), cand(&["а"])], false);
        assert_eq!(keys(&after), ["x", "x_2", "x_3"]);
        assert_eq!(after[0].forms, ["а"]);
    }

    #[test]
    fn a_variant_row_never_takes_the_bare_key_from_the_primary() {
        let mut primary = cand(&["я", "б"]);
        primary.primary = true;
        let a = assign("x", vec![cand(&["а", ""]), primary], false);
        assert_eq!(a[0].key, "x");
        assert_eq!(a[0].forms, ["я", "б"]);
        assert_eq!(a[1].forms, ["а", ""]);
    }

    #[test]
    fn a_noisy_row_never_takes_the_bare_key() {
        let mut noisy = cand(&["а", "б"]);
        noisy.noise = 1;
        let a = assign("x", vec![noisy, cand(&["я", "б"])], false);
        assert_eq!(a[0].key, "x");
        assert_eq!(a[0].forms, ["я", "б"]);
    }

    #[test]
    fn a_soft_sense_never_takes_the_bare_key() {
        let a = assign("x", vec![soft(&["а"]), cand(&["я"])], false);
        assert_eq!(a[0].key, "x");
        assert_eq!(a[0].forms, ["я"]);
        assert_eq!(a[1].key, "x_2");
    }

    #[test]
    fn split_key_round_trips_make_key() {
        assert_eq!(split_key("градъ_2"), Some(("градъ", 2)));
        assert_eq!(split_key("градъ"), None);
        assert_eq!(split_key("_2"), None);
        assert_eq!(split_key("x_"), None);
        assert_eq!(split_key("x_2a"), None);
        for (lemma, suffix) in [("бꙑти", 2u32), ("и", 3), ("x", 17)] {
            assert_eq!(split_key(&make_key(lemma, suffix)), Some((lemma, suffix)));
        }
    }

    #[test]
    #[should_panic(expected = "overflows u32")]
    fn split_key_overflow_is_a_hard_error() {
        let _ = split_key("foo_4294967296");
    }
}
