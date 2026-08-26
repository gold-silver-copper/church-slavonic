//! Realization-coherence gate for the merged verb kernel.
//!
//! Every cell of `church_slavonic_core::verb`, `verb_past`, and
//! `verb_participle` where both recensions are populated must be one of:
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
//! A second pin holds the kernel's Synodal copula columns equal to the
//! Alypy §81 tables committed in `data/synodal/exact_forms.tsv`, so the
//! merged closed system and the family's data-side paradigm cannot drift.

use church_slavonic_core::verb::{
    self, ImperativeSeries, PresentSeries, imperative_ending, l_participle_ending, present_ending,
};
use church_slavonic_core::verb_participle::{
    PastActiveFormation, PastPassiveFormation, PresentActiveFormation, PresentPassiveFormation,
    past_active_nominative_edge, past_active_oblique_suffix, past_passive_suffix,
    present_active_nominative_edge, present_active_oblique_suffix, present_passive_suffix,
};
use church_slavonic_core::verb_past::{
    AoristSeries, CopulaSeries, ImperfectMarker, aorist_ending, copula_form, imperfect_marker,
    imperfect_personal_ending,
};
use church_slavonic_core::{Gender, Number, Person, Recension};
use church_slavonic_orthography::projection::{comparison_key, project};

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::SynodalRussian;

/// Cells whose Synodal surfaces projection cannot reach. Each row cites its
/// `church_slavonic_core::divergence::NAMED` id or names the Synodal
/// spelling norm that explains it as realization. Kept sorted.
const DIVERGENT_CELLS: &[&str] = &[
    // verb:dual-first-person-va
    "verb-aorist-New:First:Dual",
    // verb:dual-third-person-leveling
    "verb-aorist-New:Third:Dual",
    // verb:aorist-third-plural-a-grade
    "verb-aorist-New:Third:Plural",
    // verb:dual-first-person-va
    "verb-aorist-SigmaticSecondary:First:Dual",
    // verb:dual-third-person-leveling
    "verb-aorist-SigmaticSecondary:Third:Dual",
    // verb:aorist-third-plural-a-grade
    "verb-aorist-SigmaticSecondary:Third:Plural",
    // verb:dual-first-person-va
    "verb-aorist-SigmaticVowel:First:Dual",
    // verb:dual-third-person-leveling
    "verb-aorist-SigmaticVowel:Third:Dual",
    // verb:aorist-third-plural-a-grade
    "verb-aorist-SigmaticVowel:Third:Plural",
    // verb:dual-first-person-va (the -вѣ archaism survives as the ordered variant)
    "verb-copula-AoristBe:First:Dual",
    // verb:dual-third-person-leveling (the co-listed ѣ-grade dual doublet)
    "verb-copula-AoristBe:Second:Dual",
    // verb:dual-third-person-leveling
    "verb-copula-AoristBe:Third:Dual",
    // verb:aorist-third-plural-a-grade
    "verb-copula-AoristBe:Third:Plural",
    // verb:dual-first-person-va (the -вѣ archaism survives as the ordered variant)
    "verb-copula-AoristBy:First:Dual",
    // verb:dual-third-person-leveling (the co-listed ѣ-grade dual doublet)
    "verb-copula-AoristBy:Second:Dual",
    // verb:copula-aorist-sti
    "verb-copula-AoristBy:Second:Singular",
    // verb:dual-third-person-leveling
    "verb-copula-AoristBy:Third:Dual",
    // verb:aorist-third-plural-a-grade
    "verb-copula-AoristBy:Third:Plural",
    // verb:copula-aorist-sti
    "verb-copula-AoristBy:Third:Singular",
    // verb:dual-first-person-va (the -вѣ archaism survives as the ordered variant)
    "verb-copula-FutureBud:First:Dual",
    // verb:dual-third-person-leveling (the co-listed ѣ-grade dual doublet)
    "verb-copula-FutureBud:Second:Dual",
    // verb:dual-third-person-leveling
    "verb-copula-FutureBud:Third:Dual",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:First:Dual",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:First:Plural",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:First:Singular",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:Second:Dual",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:Second:Plural",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:Second:Singular",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:Third:Dual",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:Third:Plural",
    // verb:copula-imperfect-restemming
    "verb-copula-ImperfectBea:Third:Singular",
    // verb:dual-first-person-va (the -вѣ archaism survives as the ordered variant)
    "verb-copula-PresentEs:First:Dual",
    // verb:copula-first-plural-my
    "verb-copula-PresentEs:First:Plural",
    // verb:dual-third-person-leveling (the co-listed ѣ-grade dual doublet)
    "verb-copula-PresentEs:Second:Dual",
    // verb:dual-third-person-leveling
    "verb-copula-PresentEs:Third:Dual",
    // verb:copula-third-person-soft-t
    "verb-copula-PresentEs:Third:Plural",
    // verb:copula-third-person-soft-t
    "verb-copula-PresentEs:Third:Singular",
    // verb:imperative-vowel-grade
    "verb-imperative-EGrade:First:Dual",
    // verb:imperative-vowel-grade
    "verb-imperative-EGrade:First:Plural",
    // verb:imperative-vowel-grade
    "verb-imperative-EGrade:Second:Dual",
    // verb:imperative-vowel-grade
    "verb-imperative-EGrade:Second:Plural",
    // verb:dual-first-person-va
    "verb-imperative-I:First:Dual",
    // verb:imperfect-contraction
    "verb-imperfect-marker:IotatedPalatalizedA",
    // verb:imperfect-contraction
    "verb-imperfect-marker:IotatedYatA",
    // verb:imperfect-contraction
    "verb-imperfect-marker:YatA",
    // verb:dual-first-person-va
    "verb-imperfect-personal:First:Dual",
    // verb:imperfect-hardening
    "verb-imperfect-personal:Second:Dual",
    // verb:imperfect-hardening
    "verb-imperfect-personal:Second:Plural",
    // verb:imperfect-hardening + verb:dual-third-person-leveling
    "verb-imperfect-personal:Third:Dual",
    // verb:l-participle-leveling
    "verb-l-participle:Feminine:Dual",
    // verb:l-participle-leveling
    "verb-l-participle:Feminine:Plural",
    // verb:l-participle-leveling
    "verb-l-participle:Neuter:Dual",
    // verb:l-participle-leveling
    "verb-l-participle:Neuter:Plural",
    // verb:present-active-nominative-contraction (the retained -шъ/-вшъ print)
    "verb-participle-past-active-nominative:ConsonantHard",
    // verb:present-active-nominative-contraction (the retained -шъ/-вшъ print)
    "verb-participle-past-active-nominative:Vowel",
    // verb:present-active-nominative-contraction
    "verb-participle-present-active-nominative:HardUsht",
    // verb:present-active-nominative-contraction
    "verb-participle-present-active-nominative:IotatedUsht",
    // verb:present-active-nominative-contraction
    "verb-participle-present-active-nominative:SoftAsht",
    // realization by the Synodal ligature norm щ = шт, outside the declared projection rule set
    "verb-participle-present-active-oblique:HardUsht",
    // realization by the Synodal ligature norm щ = шт, outside the declared projection rule set
    "verb-participle-present-active-oblique:IotatedUsht",
    // realization by the Synodal ligature norm щ = шт, outside the declared projection rule set
    "verb-participle-present-active-oblique:SoftAsht",
    // verb:dual-first-person-va
    "verb-present-FirstHard:First:Dual",
    // verb:dual-third-person-leveling
    "verb-present-FirstHard:Third:Dual",
    // verb:dual-first-person-va
    "verb-present-FirstIotated:First:Dual",
    // verb:dual-third-person-leveling
    "verb-present-FirstIotated:Third:Dual",
    // verb:dual-first-person-va
    "verb-present-SecondHardI:First:Dual",
    // verb:dual-third-person-leveling
    "verb-present-SecondHardI:Third:Dual",
    // verb:dual-first-person-va
    "verb-present-SecondSoft:First:Dual",
    // verb:dual-third-person-leveling
    "verb-present-SecondSoft:Third:Dual",
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
    for series in PresentSeries::ALL {
        for person in Person::ALL {
            for number in Number::ALL {
                check(
                    &mut residue,
                    &format!("verb-present-{series:?}:{person:?}:{number:?}"),
                    present_ending(series, person, number, OCS),
                    present_ending(series, person, number, SYN),
                );
            }
        }
    }
    for series in ImperativeSeries::ALL {
        for person in Person::ALL {
            for number in Number::ALL {
                check(
                    &mut residue,
                    &format!("verb-imperative-{series:?}:{person:?}:{number:?}"),
                    imperative_ending(series, person, number, OCS),
                    imperative_ending(series, person, number, SYN),
                );
            }
        }
    }
    for gender in Gender::ALL {
        for number in Number::ALL {
            check(
                &mut residue,
                &format!("verb-l-participle:{gender:?}:{number:?}"),
                verb::l_participle_ending(gender, number, OCS),
                l_participle_ending(gender, number, SYN),
            );
        }
    }
    for marker in ImperfectMarker::ALL {
        check(
            &mut residue,
            &format!("verb-imperfect-marker:{marker:?}"),
            imperfect_marker(marker, OCS),
            imperfect_marker(marker, SYN),
        );
    }
    for person in Person::ALL {
        for number in Number::ALL {
            check(
                &mut residue,
                &format!("verb-imperfect-personal:{person:?}:{number:?}"),
                imperfect_personal_ending(person, number, OCS),
                imperfect_personal_ending(person, number, SYN),
            );
        }
    }
    for series in AoristSeries::ALL {
        for person in Person::ALL {
            for number in Number::ALL {
                check(
                    &mut residue,
                    &format!("verb-aorist-{series:?}:{person:?}:{number:?}"),
                    aorist_ending(series, person, number, OCS),
                    aorist_ending(series, person, number, SYN),
                );
            }
        }
    }
    for series in CopulaSeries::ALL {
        for person in Person::ALL {
            for number in Number::ALL {
                check(
                    &mut residue,
                    &format!("verb-copula-{series:?}:{person:?}:{number:?}"),
                    copula_form(series, person, number, OCS),
                    copula_form(series, person, number, SYN),
                );
            }
        }
    }
    for formation in PresentActiveFormation::ALL {
        check(
            &mut residue,
            &format!("verb-participle-present-active-oblique:{formation:?}"),
            present_active_oblique_suffix(formation, OCS),
            present_active_oblique_suffix(formation, SYN),
        );
        check(
            &mut residue,
            &format!("verb-participle-present-active-nominative:{formation:?}"),
            present_active_nominative_edge(formation, OCS),
            present_active_nominative_edge(formation, SYN),
        );
    }
    for formation in PastActiveFormation::ALL {
        check(
            &mut residue,
            &format!("verb-participle-past-active-oblique:{formation:?}"),
            past_active_oblique_suffix(formation, OCS),
            past_active_oblique_suffix(formation, SYN),
        );
        check(
            &mut residue,
            &format!("verb-participle-past-active-nominative:{formation:?}"),
            past_active_nominative_edge(formation, OCS),
            past_active_nominative_edge(formation, SYN),
        );
    }
    for formation in PresentPassiveFormation::ALL {
        check(
            &mut residue,
            &format!("verb-participle-present-passive:{formation:?}"),
            present_passive_suffix(formation, OCS),
            present_passive_suffix(formation, SYN),
        );
    }
    for formation in PastPassiveFormation::ALL {
        check(
            &mut residue,
            &format!("verb-participle-past-passive:{formation:?}"),
            past_passive_suffix(formation, OCS),
            past_passive_suffix(formation, SYN),
        );
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

/// Pin the kernel's Synodal copula columns to the committed Alypy §81
/// exact-form rows (accent-blind), so the merged closed system and the
/// family's data-side paradigm cannot drift apart.
#[test]
fn synodal_copula_columns_match_the_committed_alypy_81_tables() {
    let data = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/synodal/exact_forms.tsv"
    ))
    .expect("read data/synodal/exact_forms.tsv");
    // cell key -> ordered unaccented forms as committed (normative-table
    // rows precede normative-variant rows in file order for each cell).
    let mut committed: Vec<(String, String)> = Vec::new();
    for line in data.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 7 || fields[0] != "synodal:verb:byti" {
            continue;
        }
        if !fields[4].starts_with("alypy-81-byti") {
            continue;
        }
        committed.push((fields[1].to_string(), fields[2].to_string()));
    }
    assert!(!committed.is_empty(), "no committed byti rows found");
    let tense_key = |series: CopulaSeries| match series {
        CopulaSeries::PresentEs => "present",
        CopulaSeries::FutureBud => "future",
        // Alypy files both the бѣ- and бѧ- series under the imperfect.
        CopulaSeries::ImperfectBea | CopulaSeries::AoristBe => "imperfect",
        CopulaSeries::AoristBy => "aorist",
    };
    for series in CopulaSeries::ALL {
        for person in Person::ALL {
            for number in Number::ALL {
                let key = format!("{}:{}:{}", tense_key(series), person.code(), number.code());
                let kernel = copula_form(series, person, number, SYN);
                let cell_rows: Vec<&str> = committed
                    .iter()
                    .filter(|(cell, _)| *cell == key)
                    .map(|(_, form)| form.as_str())
                    .collect();
                for form in kernel {
                    assert!(
                        cell_rows.contains(form),
                        "kernel Synodal copula form {form:?} for {series:?} {key} is not a \
                         committed Alypy §81 row (committed: {cell_rows:?})"
                    );
                }
                // Every committed row for the cell belongs to some kernel
                // series column.
                for row in &cell_rows {
                    let in_some_series = CopulaSeries::ALL.iter().any(|&other| {
                        tense_key(other) == tense_key(series)
                            && copula_form(other, person, number, SYN).contains(row)
                    });
                    assert!(
                        in_some_series,
                        "committed Alypy §81 row {row:?} for {key} is missing from every \
                         kernel Synodal copula column"
                    );
                }
            }
        }
    }
}
