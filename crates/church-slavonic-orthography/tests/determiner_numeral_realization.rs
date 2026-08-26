//! Realization-coherence gate for the merged determiner and numeral kernels.
//!
//! Every cell of `church_slavonic_core::determiner` and
//! `church_slavonic_core::numeral` where both recensions are populated must
//! be one of:
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

use church_slavonic_core::{Animacy, Case, Gender, Number, Recension, determiner, numeral};
use church_slavonic_orthography::projection::{comparison_key, project};

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::SynodalRussian;

/// Cells whose Synodal surfaces projection cannot reach. Each row cites its
/// `church_slavonic_core::divergence::NAMED` id or names the Synodal
/// spelling norm that explains it as realization. Kept sorted.
const DIVERGENT_CELLS: &[&str] = &[
    // det:hard-feminine-plural-nominative (inanimate accusative -и vs -ы)
    "det-hard:Accusative:Plural:Feminine",
    // det:hard-feminine-plural-nominative (inanimate accusative -и vs -ы)
    "det-hard:Accusative:Plural:Masculine",
    // det:hard-oblique-jat-doublets (ѣй beside ой)
    "det-hard:Dative:Singular:Feminine",
    // pron:instr-loc-sg-jer
    "det-hard:Instrumental:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "det-hard:Instrumental:Singular:Neuter",
    // det:hard-oblique-jat-doublets (ѣй beside ой)
    "det-hard:Locative:Singular:Feminine",
    // pron:instr-loc-sg-jer + det:hard-oblique-jat-doublets
    "det-hard:Locative:Singular:Masculine",
    // pron:instr-loc-sg-jer + det:hard-oblique-jat-doublets
    "det-hard:Locative:Singular:Neuter",
    // det:hard-feminine-plural-nominative
    "det-hard:Nominative:Plural:Feminine",
    // det:ves-direct-reshape (neuter plural вьса/вьсѣ vs всѧ)
    "det-ves:Accusative:Plural:Neuter",
    // det:ves-plural-jat-leveling
    "det-ves:Genitive:Plural:Feminine",
    // det:ves-plural-jat-leveling
    "det-ves:Genitive:Plural:Masculine",
    // det:ves-plural-jat-leveling
    "det-ves:Genitive:Plural:Neuter",
    // pron:instr-loc-sg-jer
    "det-ves:Instrumental:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "det-ves:Instrumental:Singular:Neuter",
    // det:ves-plural-jat-leveling
    "det-ves:Locative:Plural:Feminine",
    // det:ves-plural-jat-leveling
    "det-ves:Locative:Plural:Masculine",
    // det:ves-plural-jat-leveling
    "det-ves:Locative:Plural:Neuter",
    // pron:instr-loc-sg-jer
    "det-ves:Locative:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "det-ves:Locative:Singular:Neuter",
    // det:ves-direct-reshape (neuter plural вьса/вьсѣ vs всѧ)
    "det-ves:Nominative:Plural:Neuter",
    // det:ves-direct-reshape (вьса/вьсѣ vs всѧ)
    "det-ves:Nominative:Singular:Feminine",
    // num:collective-agreeing-reshape (inanimate accusative -и vs -ѩ)
    "num-collective:Accusative:Feminine",
    // num:collective-agreeing-reshape (inanimate accusative -и vs -ѩ)
    "num-collective:Accusative:Masculine",
    // num:collective-agreeing-reshape (-и vs -ѩ)
    "num-collective:Nominative:Feminine",
    // num:four-oblique-reinventory (четыре doublet beside четыри)
    "num-four:Accusative:Feminine",
    // num:four-oblique-reinventory (четыре doublet beside четыри)
    "num-four:Accusative:Masculine",
    // num:four-oblique-reinventory (четыре doublet beside четыри)
    "num-four:Accusative:Neuter",
    // num:four-oblique-reinventory (четыръ vs четырехъ)
    "num-four:Genitive:Feminine",
    // num:four-oblique-reinventory (четыръ vs четырехъ)
    "num-four:Genitive:Masculine",
    // num:four-oblique-reinventory (четыръ vs четырехъ)
    "num-four:Genitive:Neuter",
    // num:four-oblique-reinventory (четыре doublet beside четыри)
    "num-four:Nominative:Feminine",
    // num:four-oblique-reinventory (четыри doublet beside четыре)
    "num-four:Nominative:Masculine",
    // num:four-oblique-reinventory (четыре doublet beside четыри)
    "num-four:Nominative:Neuter",
    // num:one-long-genitive-shapes (єдинꙋю vs ѥдинѫ)
    "num-one:Accusative:Singular:Feminine",
    // num:one-long-genitive-shapes (єдинагѡ/аго vs ѥдиного)
    "num-one:Genitive:Singular:Masculine",
    // num:one-long-genitive-shapes (єдинагѡ/аго vs ѥдиного)
    "num-one:Genitive:Singular:Neuter",
    // pron:instr-loc-sg-jer
    "num-one:Instrumental:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "num-one:Instrumental:Singular:Neuter",
    // pron:instr-loc-sg-jer
    "num-one:Locative:Singular:Masculine",
    // pron:instr-loc-sg-jer
    "num-one:Locative:Singular:Neuter",
    // num:ten-oblique-reinventory (десѧте doublets)
    "num-ten:Accusative:Plural",
    // num:ten-oblique-reinventory (reviewed accusative десѧте doublet)
    "num-ten:Accusative:Singular",
    // num:ten-oblique-reinventory (десѧтимъ doublet)
    "num-ten:Dative:Plural",
    // num:ten-oblique-reinventory (десѧтихъ doublet)
    "num-ten:Genitive:Plural",
    // num:ten-oblique-reinventory (десѧты vs десѧтьми)
    "num-ten:Instrumental:Plural",
    // num:ten-oblique-reinventory (десѧтихъ doublet)
    "num-ten:Locative:Plural",
    // num:ten-oblique-reinventory (десѧти doublet)
    "num-ten:Nominative:Plural",
    // num:three-oblique-reinventory (трїемъ doublet)
    "num-three:Dative:Masculine",
    // num:three-oblique-reinventory (трии vs трехъ)
    "num-three:Genitive:Feminine",
    // num:three-oblique-reinventory (трии vs трїехъ/трехъ)
    "num-three:Genitive:Masculine",
    // num:three-oblique-reinventory (трии vs трехъ)
    "num-three:Genitive:Neuter",
    // num:three-oblique-reinventory (трїеми doublet)
    "num-three:Instrumental:Masculine",
    // num:three-oblique-reinventory (трїехъ doublet)
    "num-three:Locative:Masculine",
    // num:three-oblique-reinventory (трїе doublet beside три)
    "num-three:Nominative:Masculine",
    // num:two-genitive-u-doublet
    "num-two:Genitive:Feminine",
    // num:two-genitive-u-doublet
    "num-two:Genitive:Masculine",
    // num:two-genitive-u-doublet
    "num-two:Genitive:Neuter",
    // num:two-genitive-u-doublet
    "num-two:Locative:Feminine",
    // num:two-genitive-u-doublet
    "num-two:Locative:Masculine",
    // num:two-genitive-u-doublet
    "num-two:Locative:Neuter",
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

    for case in Case::ALL {
        for number in Number::ALL {
            for gender in Gender::ALL {
                let ocs =
                    determiner::hard_short_ending(case, number, gender, Animacy::Inanimate, OCS);
                let syn =
                    determiner::hard_short_ending(case, number, gender, Animacy::Inanimate, SYN);
                check(
                    &mut residue,
                    &format!("det-hard:{case:?}:{number:?}:{gender:?}"),
                    ocs,
                    syn,
                );

                let ocs = determiner::total_ves_cell(case, number, gender, Animacy::Inanimate, OCS);
                let syn = determiner::total_ves_cell(case, number, gender, Animacy::Inanimate, SYN);
                check(
                    &mut residue,
                    &format!("det-ves:{case:?}:{number:?}:{gender:?}"),
                    ocs,
                    syn,
                );

                let ocs = numeral::cardinal_one_cell(case, number, gender, Animacy::Inanimate, OCS);
                let syn = numeral::cardinal_one_cell(case, number, gender, Animacy::Inanimate, SYN);
                check(
                    &mut residue,
                    &format!("num-one:{case:?}:{number:?}:{gender:?}"),
                    ocs,
                    syn,
                );
            }
        }
    }

    for (paradigm, label) in [
        (numeral::PairedCardinal::Two, "two"),
        (numeral::PairedCardinal::Both, "both"),
    ] {
        for case in Case::ALL {
            for gender in Gender::ALL {
                let ocs = numeral::paired_cardinal_cell(paradigm, case, gender, OCS);
                let syn = numeral::paired_cardinal_cell(paradigm, case, gender, SYN);
                check(
                    &mut residue,
                    &format!("num-{label}:{case:?}:{gender:?}"),
                    ocs,
                    syn,
                );
            }
        }
    }

    for case in Case::ALL {
        for gender in Gender::ALL {
            let ocs = numeral::cardinal_three_cell(case, gender, Animacy::Inanimate, OCS);
            let syn = numeral::cardinal_three_cell(case, gender, Animacy::Inanimate, SYN);
            check(
                &mut residue,
                &format!("num-three:{case:?}:{gender:?}"),
                ocs,
                syn,
            );

            let ocs = numeral::cardinal_four_cell(case, gender, OCS);
            let syn = numeral::cardinal_four_cell(case, gender, SYN);
            check(
                &mut residue,
                &format!("num-four:{case:?}:{gender:?}"),
                ocs,
                syn,
            );

            let ocs =
                numeral::collective_agreeing_plural_ending(case, gender, Animacy::Inanimate, OCS);
            let syn =
                numeral::collective_agreeing_plural_ending(case, gender, Animacy::Inanimate, SYN);
            check(
                &mut residue,
                &format!("num-collective:{case:?}:{gender:?}"),
                ocs,
                syn,
            );
        }

        let ocs = numeral::i_stem_cardinal_plural_oblique_ending(case, OCS);
        let syn = numeral::i_stem_cardinal_plural_oblique_ending(case, SYN);
        check(&mut residue, &format!("num-five-nine:{case:?}"), ocs, syn);

        for number in Number::ALL {
            let ocs = numeral::cardinal_ten_cell(case, number, OCS);
            let syn = numeral::cardinal_ten_cell(case, number, SYN);
            check(
                &mut residue,
                &format!("num-ten:{case:?}:{number:?}"),
                ocs,
                syn,
            );

            let ocs = numeral::cardinal_hundred_cell(case, number, OCS);
            let syn = numeral::cardinal_hundred_cell(case, number, SYN);
            check(
                &mut residue,
                &format!("num-hundred:{case:?}:{number:?}"),
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
