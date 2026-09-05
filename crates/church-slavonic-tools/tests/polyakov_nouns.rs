//! The Polyakov noun reproduction, pinned: the importer's fit over the
//! filtered intermediate must reproduce at least as many attested
//! primaries as the committed number — the count of unaccounted forms may
//! not grow. Skips soft when the intermediate is absent (a checkout
//! without the sources).

use church_slavonic_tools::import::polyakov::import_nouns;

/// Measured 2026-09-04 (Part 1): 46,013 attested cells.
const REPRODUCED_FLOOR: u64 = 43_500;
const REACHABLE_FLOOR: u64 = 44_400;

#[test]
fn polyakov_noun_reproduction_does_not_regress() {
    let path = church_slavonic_tools::import::intermediate_dir().join("polyakov.jsonl");
    if !path.exists() {
        eprintln!("polyakov intermediate absent — skipped");
        return;
    }
    let outcome = import_nouns().expect("import runs");
    let reproduced = outcome.counts.get("cells reproduced").copied().unwrap_or(0);
    let reachable = outcome.counts.get("cells reachable (any alternative/variant)").copied().unwrap_or(0);
    assert!(reproduced >= REPRODUCED_FLOOR, "reproduced {reproduced} < floor {REPRODUCED_FLOOR}");
    assert!(reachable >= REACHABLE_FLOOR, "reachable {reachable} < floor {REACHABLE_FLOOR}");
}
