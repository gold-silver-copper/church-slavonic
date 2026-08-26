//! Pins the OCS family routes that do NOT read the merged adjective kernel
//! directly to the kernel's OCS columns, so the two copies of each closed
//! table cannot drift (docs/UNIFIED_LANGUAGE_PROMPT.md, phase-4 adjective
//! slice): the velar universal determiners вьсакъ/вьсѣкъ, whose family
//! route runs through the pronominal `2/p` hard class with positional
//! second palatalization, against the merged
//! `church_slavonic_core::determiner::velar_universal_short_ending` column.

use church_slavonic_core::{Animacy, Recension, determiner as kernel, pronoun as kernel_pronoun};
use old_church_slavonic_core::pronoun::{StandardPronominalIdentity, decline_standard_pronominal};
use old_church_slavonic_core::{Case, Gender, Number};

const OCS: Recension = Recension::OldChurchSlavonic;

#[test]
fn velar_universal_pronominal_route_matches_the_kernel_column() {
    for (identity, stem) in [
        (StandardPronominalIdentity::UniversalVsak, "вьсак"),
        (StandardPronominalIdentity::UniversalVsek, "вьсѣк"),
    ] {
        for case in Case::ALL {
            for number in Number::ALL {
                for gender in Gender::ALL {
                    let expected: Vec<String> = kernel::velar_universal_short_ending(
                        case,
                        number,
                        gender,
                        Animacy::Inanimate,
                        OCS,
                    )
                    .iter()
                    .map(|ending| {
                        let base = if ending.palatalized {
                            kernel_pronoun::palatalize_final_velar(stem, OCS)
                                .expect("a velar universal stem ends in a velar")
                        } else {
                            stem.to_string()
                        };
                        format!("{base}{}", ending.text)
                    })
                    .collect();
                    match decline_standard_pronominal(identity, case, number, gender) {
                        Ok(form) => {
                            assert_eq!(
                                vec![form.text],
                                expected,
                                "{identity:?} {case:?} {number:?} {gender:?}"
                            );
                        }
                        Err(_) => assert!(
                            expected.is_empty(),
                            "family rejects a kernel-populated cell {identity:?} {case:?} {number:?} {gender:?}"
                        ),
                    }
                }
            }
        }
    }
}
