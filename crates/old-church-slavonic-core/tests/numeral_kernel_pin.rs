//! Pins the OCS family routes that do NOT read the merged numeral kernel
//! directly (the pronominal `2/p` route for one/two/both/collectives and
//! the noun route for hundred) to the kernel's OCS columns, so the two
//! copies of each closed table cannot drift (docs/UNIFIED_LANGUAGE_PROMPT.md,
//! phase-4 determiner/numeral slice).

use church_slavonic_core::numeral as kernel;
use church_slavonic_core::{Animacy, Recension};
use old_church_slavonic_core::numeral::{CardinalMagnitudeIdentity, decline_magnitude};
use old_church_slavonic_core::pronoun::{StandardPronominalIdentity, decline_standard_pronominal};
use old_church_slavonic_core::{Case, Gender, Number, NumeralCell};

const OCS: Recension = Recension::OldChurchSlavonic;

#[test]
fn cardinal_one_pronominal_route_matches_the_kernel_column() {
    for case in Case::ALL {
        for number in Number::ALL {
            for gender in Gender::ALL {
                let kernel_cell =
                    kernel::cardinal_one_cell(case, number, gender, Animacy::Inanimate, OCS);
                let family = decline_standard_pronominal(
                    StandardPronominalIdentity::IndefiniteYedin,
                    case,
                    number,
                    gender,
                );
                match family {
                    Ok(form) => assert_eq!(
                        [form.text.as_str()],
                        kernel_cell,
                        "{case:?} {number:?} {gender:?}"
                    ),
                    Err(_) => assert!(
                        kernel_cell.is_empty(),
                        "family rejects a kernel-populated cell {case:?} {number:?} {gender:?}"
                    ),
                }
            }
        }
    }
}

#[test]
fn paired_cardinals_match_the_kernel_columns() {
    for (identity, paradigm) in [
        (
            StandardPronominalIdentity::NumeralDva,
            kernel::PairedCardinal::Two,
        ),
        (
            StandardPronominalIdentity::NumeralOba,
            kernel::PairedCardinal::Both,
        ),
    ] {
        for case in Case::ALL {
            for gender in Gender::ALL {
                let kernel_cell = kernel::paired_cardinal_cell(paradigm, case, gender, OCS);
                match decline_standard_pronominal(identity, case, Number::Dual, gender) {
                    Ok(form) => assert_eq!(
                        [form.text.as_str()],
                        kernel_cell,
                        "{identity:?} {case:?} {gender:?}"
                    ),
                    Err(_) => assert!(kernel_cell.is_empty(), "{identity:?} {case:?} {gender:?}"),
                }
            }
        }
    }
}

#[test]
fn collective_pronominal_plural_route_matches_the_kernel_endings() {
    for (identity, stem) in [
        (StandardPronominalIdentity::NumeralDvoi, "дъво"),
        (StandardPronominalIdentity::NumeralOboi, "обо"),
        (StandardPronominalIdentity::NumeralTroi, "тро"),
    ] {
        for case in Case::ALL {
            for gender in Gender::ALL {
                let expected: Vec<String> = kernel::collective_agreeing_plural_ending(
                    case,
                    gender,
                    Animacy::Inanimate,
                    OCS,
                )
                .iter()
                .map(|ending| format!("{stem}{ending}"))
                .collect();
                match decline_standard_pronominal(identity, case, Number::Plural, gender) {
                    Ok(form) => {
                        assert_eq!(
                            vec![form.text],
                            expected,
                            "{identity:?} {case:?} {gender:?}"
                        );
                    }
                    Err(_) => assert!(expected.is_empty(), "{identity:?} {case:?} {gender:?}"),
                }
            }
        }
    }
}

#[test]
fn hundred_noun_route_matches_the_kernel_column() {
    for case in Case::ALL {
        if case == Case::Vocative {
            // The kernel leaves the construction-serving vocative to the
            // family noun route.
            continue;
        }
        for number in Number::ALL {
            let kernel_cell = kernel::cardinal_hundred_cell(case, number, OCS);
            let family = decline_magnitude(
                CardinalMagnitudeIdentity::HundredSto,
                NumeralCell {
                    case,
                    number,
                    gender: None,
                },
            )
            .expect("hundred declines in every non-vocative noun cell");
            let texts: Vec<&str> = family
                .iter()
                .map(|variant| variant.prediction.text.as_str())
                .collect();
            assert_eq!(texts, kernel_cell, "{case:?} {number:?}");
        }
    }
}
