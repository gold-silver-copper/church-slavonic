//! Realization-coherence gate for the merged adjective kernel (and the
//! velar universal determiner that rides on it).
//!
//! Every cell of `church_slavonic_core::adjective` (and the velar universal
//! columns of `church_slavonic_core::determiner`) where both recensions are
//! populated must be one of:
//!
//! - **realization**: every Synodal surface is reachable from some OCS
//!   surface of the same cell under the declared projection rules
//!   (`projection::project` candidates, accent-blind via `comparison_key`);
//! - **a named divergence**: the cell appears in `DIVERGENT_CELLS` below,
//!   each row citing its `church_slavonic_core::divergence::NAMED` id, or
//!   naming the Synodal spelling norm (outside the declared projection rule
//!   set) that explains it as realization.
//!
//! The test asserts the residue set EXACTLY equals the declared list, in
//! both directions — a new silent fork or a stale registry row fails.

use church_slavonic_core::adjective::{self, AdjectiveClass};
use church_slavonic_core::{Animacy, Case, Gender, Number, Recension, determiner};
use church_slavonic_orthography::projection::{comparison_key, project};

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::SynodalRussian;

/// Cells whose Synodal surfaces projection cannot reach. Each row cites its
/// `church_slavonic_core::divergence::NAMED` id or names the Synodal
/// spelling norm that explains it as realization. Kept sorted.
const DIVERGENT_CELLS: &[&str] = &[
    // adj:long-contraction
    "adj-long-hard:Dative:Dual:Feminine",
    // adj:long-contraction
    "adj-long-hard:Dative:Dual:Masculine",
    // adj:long-contraction
    "adj-long-hard:Dative:Dual:Neuter",
    // adj:long-contraction
    "adj-long-hard:Dative:Plural:Feminine",
    // adj:long-contraction
    "adj-long-hard:Dative:Plural:Masculine",
    // adj:long-contraction
    "adj-long-hard:Dative:Plural:Neuter",
    // adj:long-contraction
    "adj-long-hard:Dative:Singular:Masculine",
    // adj:long-contraction
    "adj-long-hard:Dative:Singular:Neuter",
    // adj:long-contraction
    "adj-long-hard:Genitive:Plural:Feminine",
    // adj:long-contraction
    "adj-long-hard:Genitive:Plural:Masculine",
    // adj:long-contraction
    "adj-long-hard:Genitive:Plural:Neuter",
    // adj:long-contraction
    "adj-long-hard:Genitive:Singular:Masculine",
    // adj:long-contraction
    "adj-long-hard:Genitive:Singular:Neuter",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Dual:Feminine",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Dual:Masculine",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Dual:Neuter",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Plural:Feminine",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Plural:Masculine",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Plural:Neuter",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Singular:Feminine",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Singular:Masculine",
    // adj:long-contraction
    "adj-long-hard:Instrumental:Singular:Neuter",
    // adj:long-contraction
    "adj-long-hard:Locative:Plural:Feminine",
    // adj:long-contraction
    "adj-long-hard:Locative:Plural:Masculine",
    // adj:long-contraction
    "adj-long-hard:Locative:Plural:Neuter",
    // adj:long-contraction
    "adj-long-hard:Locative:Singular:Masculine",
    // adj:long-contraction
    "adj-long-hard:Locative:Singular:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Accusative:Dual:Masculine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Accusative:Plural:Feminine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Accusative:Plural:Masculine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Accusative:Plural:Neuter",
    // adj:long-contraction
    "adj-long-soft:Dative:Dual:Feminine",
    // adj:long-contraction
    "adj-long-soft:Dative:Dual:Masculine",
    // adj:long-contraction
    "adj-long-soft:Dative:Dual:Neuter",
    // adj:long-contraction
    "adj-long-soft:Dative:Plural:Feminine",
    // adj:long-contraction
    "adj-long-soft:Dative:Plural:Masculine",
    // adj:long-contraction
    "adj-long-soft:Dative:Plural:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Dative:Singular:Feminine",
    // adj:long-contraction
    "adj-long-soft:Dative:Singular:Masculine",
    // adj:long-contraction
    "adj-long-soft:Dative:Singular:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Genitive:Dual:Feminine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Genitive:Dual:Masculine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Genitive:Dual:Neuter",
    // adj:long-contraction
    "adj-long-soft:Genitive:Plural:Feminine",
    // adj:long-contraction
    "adj-long-soft:Genitive:Plural:Masculine",
    // adj:long-contraction
    "adj-long-soft:Genitive:Plural:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Genitive:Singular:Feminine",
    // adj:long-contraction + adj:soft-long-vowel-grade (аѥго vs ѧгѡ)
    "adj-long-soft:Genitive:Singular:Masculine",
    // adj:long-contraction + adj:soft-long-vowel-grade (аѥго vs ѧгѡ)
    "adj-long-soft:Genitive:Singular:Neuter",
    // adj:long-contraction
    "adj-long-soft:Instrumental:Dual:Feminine",
    // adj:long-contraction
    "adj-long-soft:Instrumental:Dual:Masculine",
    // adj:long-contraction
    "adj-long-soft:Instrumental:Dual:Neuter",
    // adj:long-contraction
    "adj-long-soft:Instrumental:Plural:Feminine",
    // adj:long-contraction
    "adj-long-soft:Instrumental:Plural:Masculine",
    // adj:long-contraction
    "adj-long-soft:Instrumental:Plural:Neuter",
    // adj:long-contraction
    "adj-long-soft:Instrumental:Singular:Masculine",
    // adj:long-contraction
    "adj-long-soft:Instrumental:Singular:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Locative:Dual:Feminine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Locative:Dual:Masculine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Locative:Dual:Neuter",
    // adj:long-contraction
    "adj-long-soft:Locative:Plural:Feminine",
    // adj:long-contraction
    "adj-long-soft:Locative:Plural:Masculine",
    // adj:long-contraction
    "adj-long-soft:Locative:Plural:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Locative:Singular:Feminine",
    // adj:long-contraction
    "adj-long-soft:Locative:Singular:Masculine",
    // adj:long-contraction
    "adj-long-soft:Locative:Singular:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Nominative:Dual:Masculine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Nominative:Plural:Feminine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Nominative:Plural:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Nominative:Singular:Feminine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Vocative:Dual:Masculine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Vocative:Plural:Feminine",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Vocative:Plural:Neuter",
    // adj:soft-long-vowel-grade
    "adj-long-soft:Vocative:Singular:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Dative:Dual:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Dative:Dual:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Dative:Dual:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Dative:Plural:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Dative:Plural:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Dative:Plural:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Genitive:Plural:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Genitive:Plural:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Genitive:Plural:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Instrumental:Dual:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Instrumental:Dual:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Instrumental:Dual:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Instrumental:Plural:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Instrumental:Plural:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Instrumental:Plural:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Instrumental:Singular:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Instrumental:Singular:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Locative:Plural:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Locative:Plural:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-hard:Locative:Plural:Neuter",
    // adj:short-vocative-leveling (о vs а)
    "adj-short-hard:Vocative:Singular:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Accusative:Dual:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Accusative:Plural:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Accusative:Plural:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Accusative:Plural:Neuter",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Dative:Dual:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Dative:Dual:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Dative:Dual:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Dative:Plural:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Dative:Plural:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Dative:Plural:Neuter",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Dative:Singular:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Dative:Singular:Neuter",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Genitive:Dual:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Genitive:Dual:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Genitive:Dual:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Genitive:Plural:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Genitive:Plural:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Genitive:Plural:Neuter",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Genitive:Singular:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Genitive:Singular:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Genitive:Singular:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Instrumental:Dual:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Instrumental:Dual:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Instrumental:Dual:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Instrumental:Plural:Feminine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Instrumental:Plural:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Instrumental:Plural:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Instrumental:Singular:Masculine",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Instrumental:Singular:Neuter",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Locative:Dual:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Locative:Dual:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Locative:Dual:Neuter",
    // adj:short-oblique-pronominalization
    "adj-short-soft:Locative:Plural:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Nominative:Dual:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Nominative:Plural:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Nominative:Plural:Neuter",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Nominative:Singular:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Vocative:Dual:Masculine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Vocative:Plural:Feminine",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Vocative:Plural:Neuter",
    // adj:soft-short-palatal-vowel-series
    "adj-short-soft:Vocative:Singular:Feminine",
    // adj:short-vocative-leveling (е vs ь)
    "adj-short-soft:Vocative:Singular:Masculine",
    // det:velar-universal-reshape
    "det-velar:Accusative:Plural:Feminine",
    // det:velar-universal-reshape
    "det-velar:Accusative:Plural:Masculine",
    // det:velar-universal-reshape
    "det-velar:Dative:Singular:Feminine",
    // det:velar-universal-reshape
    "det-velar:Genitive:Singular:Masculine",
    // det:velar-universal-reshape
    "det-velar:Genitive:Singular:Neuter",
    // pron:instr-loc-sg-jer + det:velar-universal-reshape
    "det-velar:Instrumental:Singular:Masculine",
    // pron:instr-loc-sg-jer + det:velar-universal-reshape
    "det-velar:Instrumental:Singular:Neuter",
    // det:velar-universal-reshape
    "det-velar:Locative:Singular:Feminine",
    // det:velar-universal-reshape
    "det-velar:Locative:Singular:Masculine",
    // det:velar-universal-reshape
    "det-velar:Locative:Singular:Neuter",
    // det:velar-universal-reshape
    "det-velar:Nominative:Plural:Feminine",
    // det:velar-universal-reshape
    "det-velar:Nominative:Plural:Masculine",
];

/// The Synodal kernel spellings embed the family's positional i-variant
/// typography (й, ї, і over shared и); fold it up front so display
/// typography never masquerades as morphology.
fn fold_display(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            'й' | 'ї' | 'і' => 'и',
            other => other,
        })
        .collect()
}

fn reachable(ocs_texts: &[&str], syn_text: &str) -> bool {
    let target = comparison_key(&fold_display(syn_text));
    ocs_texts.iter().any(|ocs_text| {
        if comparison_key(ocs_text) == target {
            return true;
        }
        match project(ocs_text, OCS, SYN) {
            Ok(projection) => projection
                .into_candidates()
                .is_some_and(|candidates| candidates.contains(&target)),
            Err(_) => false,
        }
    })
}

fn check(residue: &mut Vec<String>, cell_id: &str, ocs_texts: &[&str], syn_texts: &[&str]) {
    if ocs_texts.is_empty() || syn_texts.is_empty() {
        return;
    }
    let coherent = syn_texts
        .iter()
        .all(|syn_text| reachable(ocs_texts, syn_text));
    if !coherent {
        residue.push(cell_id.to_string());
    }
}

#[test]
fn kernel_realization_pairs_project_and_divergences_are_registered() {
    let mut residue: Vec<String> = Vec::new();

    for class in AdjectiveClass::ALL {
        let class_label = match class {
            AdjectiveClass::Hard => "hard",
            AdjectiveClass::Soft => "soft",
        };
        for case in Case::ALL {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    let ocs = adjective::short_ending(
                        class,
                        case,
                        number,
                        gender,
                        Animacy::Inanimate,
                        OCS,
                    );
                    let syn = adjective::short_ending(
                        class,
                        case,
                        number,
                        gender,
                        Animacy::Inanimate,
                        SYN,
                    );
                    check(
                        &mut residue,
                        &format!("adj-short-{class_label}:{case:?}:{number:?}:{gender:?}"),
                        ocs,
                        syn,
                    );

                    let ocs = adjective::long_ending(
                        class,
                        case,
                        number,
                        gender,
                        Animacy::Inanimate,
                        OCS,
                    );
                    let syn = adjective::long_ending(
                        class,
                        case,
                        number,
                        gender,
                        Animacy::Inanimate,
                        SYN,
                    );
                    check(
                        &mut residue,
                        &format!("adj-long-{class_label}:{case:?}:{number:?}:{gender:?}"),
                        ocs,
                        syn,
                    );
                }
            }
        }
    }

    for case in Case::ALL {
        for number in Number::ALL {
            for gender in Gender::ALL {
                let ocs: Vec<&str> = determiner::velar_universal_short_ending(
                    case,
                    number,
                    gender,
                    Animacy::Inanimate,
                    OCS,
                )
                .iter()
                .map(|ending| ending.text)
                .collect();
                let syn: Vec<&str> = determiner::velar_universal_short_ending(
                    case,
                    number,
                    gender,
                    Animacy::Inanimate,
                    SYN,
                )
                .iter()
                .map(|ending| ending.text)
                .collect();
                check(
                    &mut residue,
                    &format!("det-velar:{case:?}:{number:?}:{gender:?}"),
                    &ocs,
                    &syn,
                );
            }
        }
    }

    residue.sort_unstable();
    let expected: Vec<&str> = DIVERGENT_CELLS.to_vec();
    assert_eq!(
        residue,
        expected,
        "named-divergence residue changed; update church_slavonic_core::divergence and this list together\ncomputed:\n{}",
        residue.join("\n")
    );
}
