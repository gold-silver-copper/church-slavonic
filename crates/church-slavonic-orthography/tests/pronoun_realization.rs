//! Realization-coherence gate for the merged pronoun kernel.
//!
//! Every cell of `church_slavonic_core::pronoun` where both recensions are
//! populated must be one of:
//!
//! - **realization**: every Synodal surface is reachable from some OCS
//!   surface of the same cell under the declared projection rules
//!   (`projection::project` candidates, accent-blind via `comparison_key`);
//! - **a named divergence**: the cell appears in `DIVERGENT_CELLS` below,
//!   each row citing its `church_slavonic_core::divergence::NAMED` id.
//!
//! The test asserts the residue set EXACTLY equals the declared list, in
//! both directions — a new silent fork or a stale registry row fails.
//! The OCS palatal mark U+0484 (н҄-) is notation the Synodal recension does
//! not write; it is stripped before projection.

use church_slavonic_core::pronoun::{
    AgreeingClass, InterrogativeParadigm, PersonalParadigm, agreeing_ending, anaphoric_cell,
    interrogative_cell, personal_cell, proximal_cell, reflexive_cell, relative_nominative_base,
};
use church_slavonic_core::{Animacy, Case, Gender, Number, Recension};
use church_slavonic_orthography::projection::{comparison_key, project};

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::SynodalRussian;

/// Cells whose Synodal surfaces projection cannot reach. Each row cites
/// its `church_slavonic_core::divergence::NAMED` id, or names the Synodal
/// spelling norm (outside the declared projection rule set) that explains
/// it as realization. Kept sorted.
const DIVERGENT_CELLS: &[&str] = &[
    // pron:instr-loc-sg-jer
    "agreeing:hard:Instrumental:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "agreeing:hard:Instrumental:Singular:Neuter",
    // pron:instr-loc-sg-jer
    "agreeing:hard:Locative:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "agreeing:hard:Locative:Singular:Neuter",
    // realization: post-husher jer hardening (нашь ~ нашъ)
    "agreeing:soft:Accusative:Singular:Masculine",
    // realization: и spelled ы after the husher stem (нашымъ)
    "agreeing:soft:Dative:Plural:Feminine",
    // realization: и spelled ы after the husher stem
    "agreeing:soft:Dative:Plural:Masculine",
    // realization: и spelled ы after the husher stem
    "agreeing:soft:Dative:Plural:Neuter",
    // pron:instr-loc-sg-jer
    "agreeing:soft:Instrumental:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "agreeing:soft:Instrumental:Singular:Neuter",
    // pron:instr-loc-sg-jer
    "agreeing:soft:Locative:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "agreeing:soft:Locative:Singular:Neuter",
    // realization: post-husher jer hardening (нашь ~ нашъ)
    "agreeing:soft:Nominative:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "agreeing:softj:Instrumental:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "agreeing:softj:Instrumental:Singular:Neuter",
    // pron:instr-loc-sg-jer
    "agreeing:softj:Locative:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "agreeing:softj:Locative:Singular:Neuter",
    // pron:dual-accusative-gender-leveling
    "anaphoric:Accusative:Dual:Feminine:after=false",
    // pron:dual-accusative-gender-leveling
    "anaphoric:Accusative:Dual:Feminine:after=true",
    // pron:dual-accusative-gender-leveling
    "anaphoric:Accusative:Dual:Neuter:after=false",
    // pron:dual-accusative-gender-leveling
    "anaphoric:Accusative:Dual:Neuter:after=true",
    // pron:instr-loc-sg-jer
    "anaphoric:Instrumental:Singular:Masculine:after=false",
    // pron:instr-loc-sg-jer
    "anaphoric:Instrumental:Singular:Masculine:after=true",
    // pron:instr-loc-sg-jer
    "anaphoric:Instrumental:Singular:Neuter:after=false",
    // pron:instr-loc-sg-jer
    "anaphoric:Instrumental:Singular:Neuter:after=true",
    // pron:instr-loc-sg-jer
    "anaphoric:Locative:Singular:Masculine:after=true",
    // pron:instr-loc-sg-jer
    "anaphoric:Locative:Singular:Neuter:after=true",
    // pron:chto-oblique-inventory
    "interrogative:chto:Accusative",
    // pron:chto-oblique-inventory
    "interrogative:chto:Genitive",
    // pron:instr-loc-sg-jer
    "interrogative:chto:Instrumental",
    // pron:instr-loc-sg-jer + pron:chto-oblique-inventory
    "interrogative:chto:Locative",
    // pron:genitive-accusative
    "interrogative:kto:Accusative",
    // pron:kto-instrumental-stem
    "interrogative:kto:Instrumental",
    // pron:instr-loc-sg-jer
    "interrogative:kto:Locative",
    // pron:genitive-accusative
    "personal:first:Accusative:Plural",
    // pron:genitive-accusative + pron:accusative-clitic-status
    "personal:first:Accusative:Singular",
    // pron:dual-nominative-leveling
    "personal:first:Nominative:Dual",
    // pron:genitive-accusative
    "personal:second:Accusative:Plural",
    // pron:genitive-accusative + pron:accusative-clitic-status
    "personal:second:Accusative:Singular",
    // pron:proximal-nominative-reshape
    "proximal:Accusative:Dual:Feminine",
    // pron:proximal-nominative-reshape
    "proximal:Accusative:Dual:Neuter",
    // pron:proximal-nominative-reshape
    "proximal:Accusative:Plural:Neuter",
    // pron:proximal-nominative-reshape
    "proximal:Accusative:Singular:Masculine",
    // pron:proximal-nominative-reshape
    "proximal:Accusative:Singular:Neuter",
    // pron:instr-loc-sg-jer
    "proximal:Instrumental:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "proximal:Instrumental:Singular:Neuter",
    // pron:instr-loc-sg-jer
    "proximal:Locative:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "proximal:Locative:Singular:Neuter",
    // pron:proximal-nominative-reshape
    "proximal:Nominative:Dual:Feminine",
    // pron:proximal-nominative-reshape
    "proximal:Nominative:Dual:Neuter",
    // pron:proximal-nominative-reshape
    "proximal:Nominative:Plural:Neuter",
    // pron:proximal-nominative-reshape
    "proximal:Nominative:Singular:Feminine",
    // pron:proximal-nominative-reshape
    "proximal:Nominative:Singular:Masculine",
    // pron:proximal-nominative-reshape
    "proximal:Nominative:Singular:Neuter",
    // pron:genitive-accusative + pron:accusative-clitic-status
    "reflexive:Accusative",
];

/// The Synodal kernel spellings embed the family's positional i-variant
/// typography (й, ї, і over shared и). `comparison_key` folds these only in
/// precomposed position; fold them up front so display typography never
/// masquerades as morphology.
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
        let plain: String = ocs_text.chars().filter(|c| *c != '\u{0484}').collect();
        if comparison_key(&plain) == target {
            return true;
        }
        match project(&plain, OCS, SYN) {
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

    for (paradigm, label) in [
        (PersonalParadigm::First, "first"),
        (PersonalParadigm::Second, "second"),
    ] {
        for case in Case::ALL {
            for number in Number::ALL {
                let ocs: Vec<&str> = personal_cell(paradigm, case, number, OCS)
                    .iter()
                    .map(|s| s.text)
                    .collect();
                let syn: Vec<&str> = personal_cell(paradigm, case, number, SYN)
                    .iter()
                    .map(|s| s.text)
                    .collect();
                check(
                    &mut residue,
                    &format!("personal:{label}:{case:?}:{number:?}"),
                    &ocs,
                    &syn,
                );
            }
        }
    }

    for case in Case::ALL {
        let ocs: Vec<&str> = reflexive_cell(case, OCS).iter().map(|s| s.text).collect();
        let syn: Vec<&str> = reflexive_cell(case, SYN).iter().map(|s| s.text).collect();
        check(&mut residue, &format!("reflexive:{case:?}"), &ocs, &syn);
    }

    for (paradigm, label) in [
        (InterrogativeParadigm::Kto, "kto"),
        (InterrogativeParadigm::Chto, "chto"),
    ] {
        for case in Case::ALL {
            let ocs: Vec<&str> = interrogative_cell(paradigm, case, OCS)
                .iter()
                .map(|s| s.text)
                .collect();
            let syn: Vec<&str> = interrogative_cell(paradigm, case, SYN)
                .iter()
                .map(|s| s.text)
                .collect();
            check(
                &mut residue,
                &format!("interrogative:{label}:{case:?}"),
                &ocs,
                &syn,
            );
        }
    }

    for (class, label) in [
        (AgreeingClass::Hard, "hard"),
        (AgreeingClass::Soft, "soft"),
        (AgreeingClass::SoftJ, "softj"),
    ] {
        for case in Case::ALL {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    let ocs = agreeing_ending(class, case, number, gender, Animacy::Inanimate, OCS);
                    let syn = agreeing_ending(class, case, number, gender, Animacy::Inanimate, SYN);
                    check(
                        &mut residue,
                        &format!("agreeing:{label}:{case:?}:{number:?}:{gender:?}"),
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
                let ocs = proximal_cell(case, number, gender, Animacy::Inanimate, OCS);
                let syn = proximal_cell(case, number, gender, Animacy::Inanimate, SYN);
                check(
                    &mut residue,
                    &format!("proximal:{case:?}:{number:?}:{gender:?}"),
                    ocs,
                    syn,
                );
                for after in [false, true] {
                    let ocs = anaphoric_cell(case, number, gender, Animacy::Inanimate, after, OCS);
                    let syn = anaphoric_cell(case, number, gender, Animacy::Inanimate, after, SYN);
                    check(
                        &mut residue,
                        &format!("anaphoric:{case:?}:{number:?}:{gender:?}:after={after}"),
                        ocs,
                        syn,
                    );
                }
            }
        }
    }

    for number in Number::ALL {
        for gender in Gender::ALL {
            let ocs = relative_nominative_base(number, gender, OCS);
            let syn = relative_nominative_base(number, gender, SYN);
            check(
                &mut residue,
                &format!("relative-nominative:{number:?}:{gender:?}"),
                ocs,
                syn,
            );
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
