//! Realization-coherence gate for the merged noun kernel.
//!
//! Every cell of `church_slavonic_core::noun` and
//! `church_slavonic_core::noun_consonant` where both recensions are
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
//!
//! Consonant-stem endings compare after the extended oblique stem; the OCS
//! jer-grade stem extension ъв against the Synodal ов is a family stem
//! fact outside these columns (see the kernel module docs).

use church_slavonic_core::noun::{self, VocalicNounClass};
use church_slavonic_core::noun_consonant::{self, ConsonantNounClass};
use church_slavonic_core::{Animacy, Case, Number, Recension};
use church_slavonic_orthography::projection::{comparison_key, project};

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::SynodalRussian;

/// Cells whose Synodal surfaces projection cannot reach. Each row cites its
/// `church_slavonic_core::divergence::NAMED` id or names the Synodal
/// spelling norm that explains it as realization. Kept sorted.
const DIVERGENT_CELLS: &[&str] = &[
    // noun:animate-accusative-coverage
    "noun-AHard:Accusative:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-IFeminine:Genitive:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-IFeminine:Genitive:Plural:Inanimate",
    // noun:i-stem-instrumental-i-grade
    "noun-IFeminine:Instrumental:Singular:Animate",
    // noun:i-stem-instrumental-i-grade
    "noun-IFeminine:Instrumental:Singular:Inanimate",
    // noun:i-stem-vocative-leveling
    "noun-IFeminine:Vocative:Singular:Animate",
    // noun:i-stem-vocative-leveling
    "noun-IFeminine:Vocative:Singular:Inanimate",
    // noun:soft-genitive-plural-reinventory
    "noun-IMasculine:Accusative:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-IMasculine:Genitive:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-IMasculine:Genitive:Plural:Inanimate",
    // noun:instrumental-singular-jer
    "noun-IMasculine:Instrumental:Singular:Animate",
    // noun:instrumental-singular-jer
    "noun-IMasculine:Instrumental:Singular:Inanimate",
    // noun:i-stem-vocative-leveling
    "noun-IMasculine:Vocative:Singular:Animate",
    // noun:i-stem-vocative-leveling
    "noun-IMasculine:Vocative:Singular:Inanimate",
    // noun:animate-accusative-coverage
    "noun-JaSoft:Accusative:Plural:Animate",
    // noun:soft-direct-plural-leveling
    "noun-JaSoft:Accusative:Plural:Inanimate",
    // noun:soft-feminine-genitive-leveling
    "noun-JaSoft:Genitive:Singular:Animate",
    // noun:soft-feminine-genitive-leveling
    "noun-JaSoft:Genitive:Singular:Inanimate",
    // noun:soft-direct-plural-leveling
    "noun-JaSoft:Nominative:Plural:Animate",
    // noun:soft-direct-plural-leveling
    "noun-JaSoft:Nominative:Plural:Inanimate",
    // noun:soft-direct-plural-leveling
    "noun-JaSoft:Vocative:Plural:Animate",
    // noun:soft-direct-plural-leveling
    "noun-JaSoft:Vocative:Plural:Inanimate",
    // noun:soft-genitive-plural-reinventory
    "noun-JoSoftMasculine:Accusative:Plural:Animate",
    // noun:soft-direct-plural-leveling
    "noun-JoSoftMasculine:Accusative:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Dative:Singular:Animate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Dative:Singular:Inanimate",
    // noun:soft-genitive-plural-reinventory
    "noun-JoSoftMasculine:Genitive:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-JoSoftMasculine:Genitive:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Instrumental:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Instrumental:Plural:Inanimate",
    // noun:instrumental-singular-jer
    "noun-JoSoftMasculine:Instrumental:Singular:Animate",
    // noun:instrumental-singular-jer
    "noun-JoSoftMasculine:Instrumental:Singular:Inanimate",
    // noun:locative-plural-reinventory
    "noun-JoSoftMasculine:Locative:Plural:Animate",
    // noun:locative-plural-reinventory
    "noun-JoSoftMasculine:Locative:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Locative:Singular:Animate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Locative:Singular:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Nominative:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Nominative:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Vocative:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftMasculine:Vocative:Plural:Inanimate",
    // noun:soft-genitive-plural-reinventory
    "noun-JoSoftNeuter:Genitive:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-JoSoftNeuter:Genitive:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftNeuter:Instrumental:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-JoSoftNeuter:Instrumental:Plural:Inanimate",
    // noun:instrumental-singular-jer
    "noun-JoSoftNeuter:Instrumental:Singular:Animate",
    // noun:instrumental-singular-jer
    "noun-JoSoftNeuter:Instrumental:Singular:Inanimate",
    // noun:locative-plural-reinventory
    "noun-JoSoftNeuter:Locative:Plural:Animate",
    // noun:locative-plural-reinventory
    "noun-JoSoftNeuter:Locative:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Accusative:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Dative:Singular:Animate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Dative:Singular:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Genitive:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Genitive:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Instrumental:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Instrumental:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Locative:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-OHardMasculine:Locative:Plural:Inanimate",
    // noun:dual-direct-reshape
    "noun-OHardNeuter:Accusative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-OHardNeuter:Accusative:Dual:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-OHardNeuter:Instrumental:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-OHardNeuter:Instrumental:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-OHardNeuter:Locative:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-OHardNeuter:Locative:Plural:Inanimate",
    // noun:dual-direct-reshape
    "noun-OHardNeuter:Nominative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-OHardNeuter:Nominative:Dual:Inanimate",
    // noun:dual-direct-reshape
    "noun-OHardNeuter:Vocative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-OHardNeuter:Vocative:Dual:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Accusative:Dual:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Accusative:Dual:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Accusative:Plural:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Accusative:Singular:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Dative:Plural:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Dative:Plural:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Dative:Singular:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Dative:Singular:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Genitive:Dual:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Genitive:Dual:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Genitive:Singular:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Genitive:Singular:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Instrumental:Plural:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Instrumental:Plural:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Instrumental:Singular:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Instrumental:Singular:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Locative:Dual:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Locative:Dual:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Locative:Plural:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Locative:Plural:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Locative:Singular:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Locative:Singular:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Nominative:Dual:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Nominative:Dual:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Nominative:Plural:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Nominative:Plural:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Vocative:Dual:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Vocative:Dual:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Vocative:Plural:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Vocative:Plural:Inanimate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Vocative:Singular:Animate",
    // noun:u-stem-dissolution
    "noun-UStemMasculine:Vocative:Singular:Inanimate",
    // noun:soft-direct-plural-leveling
    "noun-agent-plural:Accusative:Inanimate",
    // noun:agent-plural-reinventory
    "noun-agent-plural:Nominative:Animate",
    // noun:agent-plural-reinventory
    "noun-agent-plural:Nominative:Inanimate",
    // noun:agent-plural-reinventory
    "noun-agent-plural:Vocative:Animate",
    // noun:agent-plural-reinventory
    "noun-agent-plural:Vocative:Inanimate",
    // noun:soft-genitive-plural-reinventory
    "noun-consonant-NMasculine:Accusative:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-consonant-NMasculine:Genitive:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-consonant-NMasculine:Genitive:Plural:Inanimate",
    // noun:instrumental-singular-jer
    "noun-consonant-NMasculine:Instrumental:Singular:Animate",
    // noun:instrumental-singular-jer
    "noun-consonant-NMasculine:Instrumental:Singular:Inanimate",
    // noun:consonant-locative-singular-i
    "noun-consonant-NMasculine:Locative:Singular:Animate",
    // noun:consonant-locative-singular-i
    "noun-consonant-NMasculine:Locative:Singular:Inanimate",
    // noun:consonant-direct-reshape
    "noun-consonant-NMasculine:Nominative:Plural:Animate",
    // noun:consonant-direct-reshape
    "noun-consonant-NMasculine:Nominative:Plural:Inanimate",
    // noun:consonant-direct-reshape
    "noun-consonant-NMasculine:Vocative:Plural:Animate",
    // noun:consonant-direct-reshape
    "noun-consonant-NMasculine:Vocative:Plural:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-NNeuter:Accusative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-NNeuter:Accusative:Dual:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NNeuter:Dative:Dual:Animate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NNeuter:Dative:Dual:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NNeuter:Dative:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NNeuter:Dative:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NNeuter:Instrumental:Dual:Animate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NNeuter:Instrumental:Dual:Inanimate",
    // noun:instrumental-singular-jer
    "noun-consonant-NNeuter:Instrumental:Singular:Animate",
    // noun:instrumental-singular-jer
    "noun-consonant-NNeuter:Instrumental:Singular:Inanimate",
    // noun:locative-plural-reinventory
    "noun-consonant-NNeuter:Locative:Plural:Animate",
    // noun:locative-plural-reinventory
    "noun-consonant-NNeuter:Locative:Plural:Inanimate",
    // noun:consonant-locative-singular-i
    "noun-consonant-NNeuter:Locative:Singular:Animate",
    // noun:consonant-locative-singular-i
    "noun-consonant-NNeuter:Locative:Singular:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-NNeuter:Nominative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-NNeuter:Nominative:Dual:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-NNeuter:Vocative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-NNeuter:Vocative:Dual:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-NtNeuter:Accusative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-NtNeuter:Accusative:Dual:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NtNeuter:Dative:Dual:Animate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NtNeuter:Dative:Dual:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NtNeuter:Dative:Plural:Animate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NtNeuter:Dative:Plural:Inanimate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NtNeuter:Instrumental:Dual:Animate",
    // noun:hard-declension-variant-imports
    "noun-consonant-NtNeuter:Instrumental:Dual:Inanimate",
    // noun:instrumental-singular-jer
    "noun-consonant-NtNeuter:Instrumental:Singular:Animate",
    // noun:instrumental-singular-jer
    "noun-consonant-NtNeuter:Instrumental:Singular:Inanimate",
    // noun:locative-plural-reinventory
    "noun-consonant-NtNeuter:Locative:Plural:Animate",
    // noun:locative-plural-reinventory
    "noun-consonant-NtNeuter:Locative:Plural:Inanimate",
    // noun:consonant-locative-singular-i
    "noun-consonant-NtNeuter:Locative:Singular:Animate",
    // noun:consonant-locative-singular-i
    "noun-consonant-NtNeuter:Locative:Singular:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-NtNeuter:Nominative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-NtNeuter:Nominative:Dual:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-NtNeuter:Vocative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-NtNeuter:Vocative:Dual:Inanimate",
    // noun:animate-accusative-coverage
    "noun-consonant-RFeminine:Accusative:Plural:Animate",
    // noun:dual-oblique-reinventory
    "noun-consonant-RFeminine:Genitive:Dual:Animate",
    // noun:dual-oblique-reinventory
    "noun-consonant-RFeminine:Genitive:Dual:Inanimate",
    // noun:soft-genitive-plural-reinventory
    "noun-consonant-RFeminine:Genitive:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-consonant-RFeminine:Genitive:Plural:Inanimate",
    // noun:i-stem-instrumental-i-grade
    "noun-consonant-RFeminine:Instrumental:Singular:Animate",
    // noun:i-stem-instrumental-i-grade
    "noun-consonant-RFeminine:Instrumental:Singular:Inanimate",
    // noun:dual-oblique-reinventory
    "noun-consonant-RFeminine:Locative:Dual:Animate",
    // noun:dual-oblique-reinventory
    "noun-consonant-RFeminine:Locative:Dual:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-SNeuter:Accusative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-SNeuter:Accusative:Dual:Inanimate",
    // noun:instrumental-singular-jer
    "noun-consonant-SNeuter:Instrumental:Singular:Animate",
    // noun:instrumental-singular-jer
    "noun-consonant-SNeuter:Instrumental:Singular:Inanimate",
    // noun:locative-plural-reinventory
    "noun-consonant-SNeuter:Locative:Plural:Animate",
    // noun:locative-plural-reinventory
    "noun-consonant-SNeuter:Locative:Plural:Inanimate",
    // noun:consonant-locative-singular-i
    "noun-consonant-SNeuter:Locative:Singular:Animate",
    // noun:consonant-locative-singular-i
    "noun-consonant-SNeuter:Locative:Singular:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-SNeuter:Nominative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-SNeuter:Nominative:Dual:Inanimate",
    // noun:dual-direct-reshape
    "noun-consonant-SNeuter:Vocative:Dual:Animate",
    // noun:dual-direct-reshape
    "noun-consonant-SNeuter:Vocative:Dual:Inanimate",
    // noun:animate-accusative-coverage
    "noun-consonant-VFeminine:Accusative:Plural:Animate",
    // noun:dual-oblique-reinventory
    "noun-consonant-VFeminine:Genitive:Dual:Animate",
    // noun:dual-oblique-reinventory
    "noun-consonant-VFeminine:Genitive:Dual:Inanimate",
    // noun:soft-genitive-plural-reinventory
    "noun-consonant-VFeminine:Genitive:Plural:Animate",
    // noun:soft-genitive-plural-reinventory
    "noun-consonant-VFeminine:Genitive:Plural:Inanimate",
    // noun:i-stem-instrumental-i-grade
    "noun-consonant-VFeminine:Instrumental:Singular:Animate",
    // noun:i-stem-instrumental-i-grade
    "noun-consonant-VFeminine:Instrumental:Singular:Inanimate",
    // noun:dual-oblique-reinventory
    "noun-consonant-VFeminine:Locative:Dual:Animate",
    // noun:dual-oblique-reinventory
    "noun-consonant-VFeminine:Locative:Dual:Inanimate",
    // noun:consonant-locative-singular-i
    "noun-consonant-VFeminine:Locative:Singular:Animate",
    // noun:consonant-locative-singular-i
    "noun-consonant-VFeminine:Locative:Singular:Inanimate",
    // noun:in-singulative-inanimate-accusative
    "noun-in-singulative:Accusative:Inanimate",
];

/// The Synodal kernel spellings embed the family's positional i-variant and
/// wide-letter typography (й, ї, і over shared и; є over е; ѡ over о); fold
/// it up front so display typography never masquerades as morphology.
fn fold_display(text: &str) -> String {
    text.chars()
        .map(|letter| match letter {
            'й' | 'ї' | 'і' => 'и',
            'є' => 'е',
            'ѡ' => 'о',
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
    for class in VocalicNounClass::ALL {
        for case in Case::ALL {
            for number in Number::ALL {
                for animacy in Animacy::ALL {
                    let ocs = noun::vocalic_ending(class, case, number, animacy, OCS);
                    let syn = noun::vocalic_ending(class, case, number, animacy, SYN);
                    check(
                        &mut residue,
                        &format!("noun-{class:?}:{case:?}:{number:?}:{animacy:?}"),
                        ocs,
                        syn,
                    );
                }
            }
        }
    }
    for class in ConsonantNounClass::ALL {
        for case in Case::ALL {
            for number in Number::ALL {
                for animacy in Animacy::ALL {
                    let ocs = noun_consonant::consonant_ending(class, case, number, animacy, OCS);
                    let syn = noun_consonant::consonant_ending(class, case, number, animacy, SYN);
                    check(
                        &mut residue,
                        &format!("noun-consonant-{class:?}:{case:?}:{number:?}:{animacy:?}"),
                        ocs,
                        syn,
                    );
                }
            }
        }
    }
    for case in Case::ALL {
        for animacy in Animacy::ALL {
            check(
                &mut residue,
                &format!("noun-in-singulative:{case:?}:{animacy:?}"),
                noun_consonant::in_singulative_plural_ending(case, animacy, OCS),
                noun_consonant::in_singulative_plural_ending(case, animacy, SYN),
            );
            check(
                &mut residue,
                &format!("noun-agent-plural:{case:?}:{animacy:?}"),
                noun_consonant::agent_direct_plural_ending(case, animacy, OCS),
                noun_consonant::agent_direct_plural_ending(case, animacy, SYN),
            );
        }
    }
    residue.sort_unstable();
    residue.dedup();
    let expected: Vec<&str> = DIVERGENT_CELLS.to_vec();
    assert_eq!(
        residue,
        expected,
        "named-divergence residue changed; update church_slavonic_core::divergence and this list together\ncomputed:\n{}",
        residue.join("\n")
    );
}
