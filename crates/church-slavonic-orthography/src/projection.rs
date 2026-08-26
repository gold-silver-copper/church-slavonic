//! Cross-recension orthographic projection: the declared OCS ↔ Synodal
//! correspondence rules, promoted from the phase-1 projection study
//! (docs/UNIFIED_LANGUAGE_PROMPT.md, execution plan steps 1 and 3) into the
//! realization layer as a first-class module. `cargo xtask projection-study`
//! and `cargo xtask unified-identity` (including the projection-coherence
//! gate) are consumers of exactly these rules — the rule set, candidate
//! enumeration order, and comparison keys here define the study's published
//! numbers and the committed `data/unified` artifacts byte-for-byte.
//!
//! Two kinds of rule, mirroring the study's honesty contract:
//!
//! - **Symmetric folds** apply to BOTH recensions' surfaces and define the
//!   accent-blind comparison space ([`comparison_key`]). They never fire as
//!   counted generative events.
//! - **Generative rules** apply in the one implemented direction — Old
//!   Church Slavonic → Synodal ([`project`]) — and may branch (jer
//!   treatment, the big yus, zelo). A projection with several candidates is
//!   [`Projection::Ambiguous`], never silently a match; a character no
//!   declared rule handles makes the whole surface
//!   [`Projection::Unprojectable`].
//!
//! The reverse direction (Synodal → OCS) is deliberately NOT enumerated:
//! the study never generated OCS candidates from Synodal surfaces — the
//! Synodal side participates only through the symmetric folds of
//! [`comparison_key`]. [`project`] therefore returns
//! [`ProjectionError::UnsupportedDirection`] for every direction other than
//! OCS → Synodal, preserving the study's semantics exactly rather than
//! inventing an untested (and much more ambiguous) inverse enumeration.
//!
//! Accent asymmetry (the hard constraint from the merge contract): OCS
//! sources are unaccented, so OCS → Synodal projection yields accentless
//! surface skeletons. Accent facts come only from Synodal-side evidence;
//! [`accented_comparison_key`] exists for exactly that full-match tier.

use crate::synodal::normalize_lookup_accentless;
use church_slavonic_core::Recension;
use core::fmt;
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

/// A word projected into more candidates than this is counted as
/// over-ambiguous rather than enumerated (jer branching is exponential).
pub const CANDIDATE_CAP: usize = 32;

/// One declared correspondence rule, documented with one example each.
/// `Fold*` rules are symmetric (both sides, via [`comparison_key`]);
/// the rest are generative (OCS side only, via [`project`]).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Rule {
    /// acute/grave/kamora/breathing removed on both sides.
    FoldAccentStrip,
    /// оу / ѹ / ꙋ / ᲂу all fold to у on both sides.
    FoldUk,
    /// The OCS two-letter uk digraph оу collapses to у in every position.
    UkDigraph,
    /// ѡ folds to о (and ѿ to от) on both sides.
    FoldOmega,
    /// і, ї, й fold to и on both sides.
    FoldIVariants,
    /// ꙗ folds to ѧ on both sides (word-initial ja spelling).
    FoldJa,
    /// ѷ folds to ѵ on both sides.
    FoldIzhitsaKendema,
    /// ꙑ -> ы.
    Yery,
    /// ѫ -> у or ю (ambiguous).
    BigYus,
    /// ѭ -> ю.
    IotatedBigYus,
    /// ѩ -> ѧ.
    IotatedSmallYus,
    /// ѧ -> ѧ (retained).
    SmallYus,
    /// ѥ -> е.
    IotatedE,
    /// Word-final ъ/ь kept (Synodal retains them).
    JerFinal,
    /// Medial ъ -> dropped, о, or kept; medial ь -> dropped, е, or kept.
    JerMedial,
    /// ѕ -> ѕ or з (ambiguous).
    Zelo,
    /// ꙁ -> з, ꙃ -> ѕ/з (archaic letterforms).
    ZemljaVariant,
}

impl Rule {
    /// Every declared rule, in the study's declared (report) order.
    pub const ALL: [Self; 17] = [
        Self::FoldAccentStrip,
        Self::FoldUk,
        Self::UkDigraph,
        Self::FoldOmega,
        Self::FoldIVariants,
        Self::FoldJa,
        Self::FoldIzhitsaKendema,
        Self::Yery,
        Self::BigYus,
        Self::IotatedBigYus,
        Self::IotatedSmallYus,
        Self::SmallYus,
        Self::IotatedE,
        Self::JerFinal,
        Self::JerMedial,
        Self::Zelo,
        Self::ZemljaVariant,
    ];

    /// The study's stable rule identifier (`fold:*` for symmetric folds,
    /// `gen:*` for generative rules).
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::FoldAccentStrip => "fold:accent-strip",
            Self::FoldUk => "fold:uk",
            Self::UkDigraph => "gen:uk-digraph",
            Self::FoldOmega => "fold:omega",
            Self::FoldIVariants => "fold:i-variants",
            Self::FoldJa => "fold:ja",
            Self::FoldIzhitsaKendema => "fold:izhitsa-kendema",
            Self::Yery => "gen:yery",
            Self::BigYus => "gen:big-yus",
            Self::IotatedBigYus => "gen:iotated-big-yus",
            Self::IotatedSmallYus => "gen:iotated-small-yus",
            Self::SmallYus => "gen:small-yus",
            Self::IotatedE => "gen:iotated-e",
            Self::JerFinal => "gen:jer-final",
            Self::JerMedial => "gen:jer-medial",
            Self::Zelo => "gen:zelo",
            Self::ZemljaVariant => "gen:zemlja-variant",
        }
    }

    /// The study's documented description, one example each.
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::FoldAccentStrip => {
                "acute/grave/kamora/breathing removed on both sides (Синъ ~ сѵ́нъ level); \
                 the orthography crate's normalize_lookup_accentless"
            }
            Self::FoldUk => {
                "оу / ѹ / ꙋ / ᲂу all fold to у on both sides: ѹчитель ~ ᲂучи́тель -> учитель"
            }
            Self::UkDigraph => {
                "the OCS two-letter uk digraph оу collapses to у in every position \
                 (OCS spells /u/ as оу throughout): рабоу -> рабу, благоую -> благую"
            }
            Self::FoldOmega => {
                "ѡ folds to о (and ѿ to от) on both sides: рабѡ́мъ ~ рабомъ -> рабом(ъ)"
            }
            Self::FoldIVariants => "і, ї, й fold to и on both sides: і҆ере́й -> иереи",
            Self::FoldJa => "ꙗ folds to ѧ on both sides (word-initial ja spelling): ꙗ҆зы́къ ~ ѧзыкъ",
            Self::FoldIzhitsaKendema => "ѷ folds to ѵ on both sides: мѷ́ро ~ мѵ́ро",
            Self::Yery => "ꙑ -> ы: рꙑба -> рыба",
            Self::BigYus => "ѫ -> у or ю (ambiguous): рѫка -> рука",
            Self::IotatedBigYus => "ѭ -> ю: землѭ -> землю",
            Self::IotatedSmallYus => "ѩ -> ѧ: ѩзꙑкъ -> ѧзыкъ",
            Self::SmallYus => "ѧ -> ѧ (retained): пѧть -> пѧть",
            Self::IotatedE => "ѥ -> е: моѥ -> мое",
            Self::JerFinal => "word-final ъ/ь kept (Synodal retains them): градъ -> градъ",
            Self::JerMedial => {
                "medial ъ -> dropped, о, or kept; medial ь -> dropped, е, or kept \
                 (ambiguous): дьнь -> день / днь; сънъ -> сонъ / снъ"
            }
            Self::Zelo => "ѕ -> ѕ or з (ambiguous): ѕвѣзда -> ѕвѣзда / звѣзда",
            Self::ZemljaVariant => "ꙁ -> з, ꙃ -> ѕ/з (archaic letterforms)",
        }
    }
}

/// Per-rule fire counts across a batch of [`project`] calls. Symmetric folds
/// never fire (they act inside the comparison keys, not as counted events).
#[derive(Debug, Default)]
pub struct RuleCounts(BTreeMap<Rule, usize>);

impl RuleCounts {
    fn fire(&mut self, rule: Rule) {
        *self.0.entry(rule).or_default() += 1;
    }

    /// How often `rule` fired, or `None` if it never did (folds never do).
    #[must_use]
    pub fn fired(&self, rule: Rule) -> Option<usize> {
        self.0.get(&rule).copied()
    }
}

/// The outcome of projecting one surface into the target recension's
/// accent-blind comparison space.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Projection {
    /// Exactly one candidate spelling the declared rules admit.
    Unambiguous(String),
    /// Several candidate spellings (deterministic enumeration order);
    /// never silently counted as a match.
    Ambiguous(Vec<String>),
    /// The rules branch past [`CANDIDATE_CAP`].
    OverAmbiguous,
    /// The source contains a character no declared rule handles
    /// (Glagolitic rows, djerv, hyphenated notations).
    Unprojectable,
}

impl Projection {
    /// Every enumerated candidate, in enumeration order; `None` for
    /// [`Projection::OverAmbiguous`] and [`Projection::Unprojectable`].
    #[must_use]
    pub fn into_candidates(self) -> Option<Vec<String>> {
        match self {
            Self::Unambiguous(candidate) => Some(vec![candidate]),
            Self::Ambiguous(candidates) => Some(candidates),
            Self::OverAmbiguous | Self::Unprojectable => None,
        }
    }
}

/// A direction the declared rules do not implement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectionError {
    /// The requested source recension.
    pub from: Recension,
    /// The requested target recension.
    pub to: Recension,
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported projection direction {:?} -> {:?}: only \
             OldChurchSlavonic -> SynodalRussian is implemented; the reverse \
             was never enumerated by the study (the Synodal side participates \
             through the symmetric comparison_key folds only)",
            self.from, self.to
        )
    }
}

impl std::error::Error for ProjectionError {}

/// Applies the symmetric folds after the crate's accent-insensitive lookup
/// projection; the result is the accent-blind comparison key shared by both
/// recensions (the study's `study_key`).
#[must_use]
pub fn comparison_key(value: &str) -> String {
    let mut output = String::new();
    for character in normalize_lookup_accentless(value).nfd() {
        match character {
            // presentation marks the lookup projection leaves in place
            '\u{0300}' | '\u{0301}' | '\u{0311}' | '\u{033e}' => {}
            'ѡ' => output.push('о'),
            'ѿ' => output.push_str("от"),
            'ѽ' | 'ѻ' => output.push('о'),
            'і' | 'ї' | 'й' => output.push('и'),
            'ꙗ' => output.push('ѧ'),
            'ѷ' => output.push('ѵ'),
            'ꙋ' => output.push('у'),
            other => output.push(other),
        }
    }
    let output: String = output.nfc().collect();
    // the word-initial uk digraph (оу / ᲂу / ѹ) folds to plain у
    output
        .strip_prefix("оу")
        .map_or(output.clone(), |rest| format!("у{rest}"))
}

/// Comparison key that keeps one accent mark (grave, kamora, and the
/// precomposed ѐ/ѝ all fold to the acute) for the full-match tier of the
/// accent asymmetry: OCS surfaces that DO carry a printed accent can be
/// checked against Synodal accented evidence. `collapse_uk_digraph` is set
/// on the OCS side (OCS spells /u/ as оу in every position; the mark, if
/// any, sits on the у and survives the collapse).
#[must_use]
pub fn accented_comparison_key(value: &str, collapse_uk_digraph: bool) -> String {
    let mut result = String::new();
    for character in value.nfd() {
        match character {
            '\u{0300}' | '\u{0311}' => result.push('\u{0301}'),
            '\u{0484}' | '\u{0486}' | '\u{033e}' => {}
            'ѡ' | 'Ѡ' => result.push('о'),
            'ѿ' | 'Ѿ' => result.push_str("от"),
            'ѽ' | 'ѻ' | 'Ѻ' | 'Ѽ' => result.push('о'),
            'є' | 'Є' => result.push('е'),
            '\u{1c82}' => result.push('о'),
            'ѹ' | 'Ѹ' => result.push_str("оу"),
            'ꙋ' | 'Ꙋ' => result.push('у'),
            'і' | 'І' | 'ї' | 'Ї' | 'й' | 'Й' => result.push('и'),
            'ꙗ' | 'Ꙗ' => result.push('ѧ'),
            'ѷ' | 'Ѷ' => result.push('ѵ'),
            other => result.extend(other.to_lowercase()),
        }
    }
    let mut result: String = result.nfc().collect();
    if collapse_uk_digraph {
        result = result.replace("оу", "у");
    }
    result
        .strip_prefix("оу")
        .map_or_else(|| result.clone(), |rest| format!("у{rest}"))
}

/// Projects one surface from `from` into `to`'s candidate accent-blind
/// comparison keys under the declared rules.
///
/// Only OCS → Synodal is implemented; every other direction is an
/// [`ProjectionError`] (see the module docs for why the reverse is an
/// honest error rather than an enumeration).
///
/// # Errors
///
/// [`ProjectionError`] when `(from, to)` is not
/// `(Recension::OldChurchSlavonic, Recension::SynodalRussian)`.
pub fn project(
    surface: &str,
    from: Recension,
    to: Recension,
) -> Result<Projection, ProjectionError> {
    let mut counts = RuleCounts::default();
    project_with_counts(surface, from, to, &mut counts)
}

/// [`project`], accumulating per-rule fire counts into `counts` (the study's
/// rule-activity table).
///
/// # Errors
///
/// [`ProjectionError`] when `(from, to)` is not
/// `(Recension::OldChurchSlavonic, Recension::SynodalRussian)`.
pub fn project_with_counts(
    surface: &str,
    from: Recension,
    to: Recension,
    counts: &mut RuleCounts,
) -> Result<Projection, ProjectionError> {
    if !matches!(
        (from, to),
        (Recension::OldChurchSlavonic, Recension::SynodalRussian)
    ) {
        return Err(ProjectionError { from, to });
    }
    Ok(project_ocs_to_synodal(surface, counts))
}

/// The implemented direction: OCS surface → candidate Synodal comparison keys.
fn project_ocs_to_synodal(surface: &str, counts: &mut RuleCounts) -> Projection {
    let folded = comparison_key(surface);
    let mut candidates = vec![String::new()];
    let characters: Vec<char> = folded.chars().collect();
    let mut skip_next = false;
    for (index, &character) in characters.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        let is_final = index + 1 == characters.len();
        if character == 'о' && characters.get(index + 1) == Some(&'у') {
            counts.fire(Rule::UkDigraph);
            skip_next = true;
            candidates.iter_mut().for_each(|c| c.push('у'));
            continue;
        }
        let options: Vec<&str> = match character {
            'ꙑ' => {
                counts.fire(Rule::Yery);
                vec!["ы"]
            }
            'ѫ' => {
                counts.fire(Rule::BigYus);
                vec!["у", "ю"]
            }
            'ѭ' => {
                counts.fire(Rule::IotatedBigYus);
                vec!["ю"]
            }
            'ѩ' => {
                counts.fire(Rule::IotatedSmallYus);
                vec!["ѧ"]
            }
            'ѧ' => {
                counts.fire(Rule::SmallYus);
                vec!["ѧ"]
            }
            'ѥ' => {
                counts.fire(Rule::IotatedE);
                vec!["е"]
            }
            'ъ' if is_final => {
                counts.fire(Rule::JerFinal);
                vec!["ъ"]
            }
            'ь' if is_final => {
                counts.fire(Rule::JerFinal);
                vec!["ь"]
            }
            'ъ' => {
                counts.fire(Rule::JerMedial);
                vec!["", "о", "ъ"]
            }
            'ь' => {
                counts.fire(Rule::JerMedial);
                vec!["", "е", "ь"]
            }
            'ѕ' => {
                counts.fire(Rule::Zelo);
                vec!["ѕ", "з"]
            }
            'ꙁ' => {
                counts.fire(Rule::ZemljaVariant);
                vec!["з"]
            }
            'ꙃ' => {
                counts.fire(Rule::ZemljaVariant);
                vec!["ѕ", "з"]
            }
            other if is_synodal_candidate_letter(other) => {
                candidates.iter_mut().for_each(|c| c.push(other));
                continue;
            }
            _ => return Projection::Unprojectable,
        };
        if candidates.len() * options.len() > CANDIDATE_CAP {
            return Projection::OverAmbiguous;
        }
        candidates = candidates
            .iter()
            .flat_map(|prefix| {
                options.iter().map(move |option| {
                    let mut next = prefix.clone();
                    next.push_str(option);
                    next
                })
            })
            .collect();
    }
    if candidates.len() == 1 {
        Projection::Unambiguous(candidates.pop().unwrap_or_default())
    } else {
        Projection::Ambiguous(candidates)
    }
}

/// The letters a candidate Synodal comparison key may contain.
fn is_synodal_candidate_letter(character: char) -> bool {
    matches!(
        character,
        'а'..='я' | 'ѣ' | 'ѧ' | 'ѳ' | 'ѵ' | 'ѯ' | 'ѱ' | 'ѕ'
    )
}

#[cfg(test)]
mod tests {
    use super::{
        CANDIDATE_CAP, Projection, ProjectionError, Rule, RuleCounts, accented_comparison_key,
        comparison_key, project, project_with_counts,
    };
    use church_slavonic_core::Recension;

    fn ocs(surface: &str) -> Projection {
        project(
            surface,
            Recension::OldChurchSlavonic,
            Recension::SynodalRussian,
        )
        .expect("implemented direction")
    }

    fn candidates(surface: &str) -> Vec<String> {
        ocs(surface).into_candidates().expect("enumerable")
    }

    #[test]
    fn only_ocs_to_synodal_is_implemented() {
        let reverse = project(
            "градъ",
            Recension::SynodalRussian,
            Recension::OldChurchSlavonic,
        );
        assert_eq!(
            reverse.clone(),
            Err(ProjectionError {
                from: Recension::SynodalRussian,
                to: Recension::OldChurchSlavonic,
            })
        );
        let error = reverse.expect_err("reverse direction is unsupported");
        assert!(error.to_string().contains("unsupported"));
        assert!(
            project(
                "градъ",
                Recension::OldChurchSlavonic,
                Recension::OldChurchSlavonic
            )
            .is_err()
        );
    }

    #[test]
    fn uk_digraph_collapses_in_every_position() {
        // gen:uk-digraph documented examples
        assert_eq!(candidates("рабоу"), ["рабу"]);
        assert_eq!(candidates("благоую"), ["благую"]);
    }

    #[test]
    fn yery_projects_to_ы() {
        // gen:yery documented example
        assert_eq!(candidates("рꙑба"), ["рыба"]);
    }

    #[test]
    fn big_yus_is_ambiguous_between_у_and_ю() {
        // gen:big-yus documented example: рѫка -> рука (plus the ю branch)
        assert_eq!(
            ocs("рѫка"),
            Projection::Ambiguous(vec!["рука".to_owned(), "рюка".to_owned(),])
        );
    }

    #[test]
    fn iotated_yuses_and_e_are_unambiguous() {
        // gen:iotated-big-yus, gen:iotated-small-yus, gen:iotated-e,
        // gen:small-yus documented examples
        assert_eq!(candidates("землѭ"), ["землю"]);
        assert_eq!(candidates("ѩзꙑкъ"), ["ѧзыкъ"]);
        assert_eq!(candidates("моѥ"), ["мое"]);
        assert_eq!(candidates("пѧть"), ["пѧть"]);
    }

    #[test]
    fn final_jers_are_kept_and_medial_jers_branch() {
        // gen:jer-final and gen:jer-medial documented examples
        assert_eq!(ocs("градъ"), Projection::Unambiguous("градъ".to_owned()));
        assert_eq!(candidates("дьнь"), ["днь", "день", "дьнь"]);
        assert_eq!(candidates("сънъ"), ["снъ", "сонъ", "сънъ"]);
    }

    #[test]
    fn zelo_and_zemlja_variants() {
        // gen:zelo and gen:zemlja-variant documented examples
        assert_eq!(candidates("ѕвѣзда"), ["ѕвѣзда", "звѣзда"]);
        assert_eq!(candidates("ꙁима"), ["зима"]);
        assert_eq!(candidates("ꙃѣло"), ["ѕѣло", "зѣло"]);
    }

    #[test]
    fn jer_branching_past_the_cap_is_over_ambiguous() {
        // four medial jers branch 3^4 = 81 > CANDIDATE_CAP
        assert_eq!(ocs("въсъвъсъвъ"), Projection::OverAmbiguous);
        const { assert!(CANDIDATE_CAP < 81) };
    }

    #[test]
    fn undeclared_characters_are_unprojectable() {
        // Glagolitic rows, djerv, hyphenated notations
        assert_eq!(ocs("ⰳⰾⰰⰳⱁⰾⱏ"), Projection::Unprojectable);
        assert_eq!(ocs("ꙉ"), Projection::Unprojectable);
        assert_eq!(ocs("да-ва"), Projection::Unprojectable);
    }

    #[test]
    fn symmetric_folds_define_one_comparison_space() {
        // fold:accent-strip + fold:uk: ѹчитель ~ ᲂучи́тель -> учитель
        assert_eq!(comparison_key("ѹчитель"), "учитель");
        assert_eq!(comparison_key("ᲂучи́тель"), "учитель");
        // fold:omega: рабѡ́мъ ~ рабомъ
        assert_eq!(comparison_key("рабѡ́мъ"), comparison_key("рабомъ"));
        // fold:i-variants: і folds to и (a precomposed й decomposes to
        // и + breve under NFD before the fold sees it, so the breve
        // survives into the key — study semantics preserved exactly)
        assert_eq!(comparison_key("і҆ере́й"), "иерей");
        assert_eq!(comparison_key("ікона"), "икона");
        // fold:ja: ꙗ҆зы́къ ~ ѧзыкъ
        assert_eq!(comparison_key("ꙗ҆зы́къ"), comparison_key("ѧзыкъ"));
        // fold:izhitsa-kendema: the accent strips from ѵ; ѷ itself
        // decomposes to ѵ + kendema (U+030F) under NFD, and the kendema —
        // like й's breve above — survives into the key (study semantics
        // preserved exactly; the composed-ѷ branch documents the intent).
        assert_eq!(comparison_key("мѵ́ро"), "мѵро");
        assert_eq!(comparison_key("мѷ́ро"), "мѷро");
    }

    #[test]
    fn accented_key_keeps_one_acute_and_folds_variant_marks() {
        assert_eq!(
            accented_comparison_key("сѵ́нъ", false),
            accented_comparison_key("сѵ\u{0300}нъ", false)
        );
        // the OCS-side digraph collapse keeps the mark on the у
        assert_eq!(
            accented_comparison_key("рабоу́", true),
            accented_comparison_key("рабꙋ́", false)
        );
    }

    #[test]
    fn ambiguity_counts_and_rule_counts_accumulate() {
        let mut counts = RuleCounts::default();
        let projection = project_with_counts(
            "дьнь",
            Recension::OldChurchSlavonic,
            Recension::SynodalRussian,
            &mut counts,
        )
        .expect("implemented direction");
        assert_eq!(projection.into_candidates().map(|c| c.len()), Some(3));
        assert_eq!(counts.fired(Rule::JerMedial), Some(1));
        assert_eq!(counts.fired(Rule::JerFinal), Some(1));
        assert_eq!(counts.fired(Rule::FoldUk), None, "folds never fire");
    }

    #[test]
    fn rule_ids_and_descriptions_are_declared_for_every_rule() {
        for rule in Rule::ALL {
            assert!(rule.id().starts_with("fold:") || rule.id().starts_with("gen:"));
            assert!(!rule.description().is_empty());
        }
    }
}
