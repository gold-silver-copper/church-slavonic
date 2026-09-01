//! The closed-class table: prepositions, conjunctions and particles —
//! hand-written, uninflected, so the linter can tell a function word from
//! an unanalyzed one. Every entry is attested VERBATIM as a standalone
//! token of the pinned Bible (counted 2026-09-01 over all 34,470 verses;
//! the count follows each entry). Growing the table means attesting the
//! new word the same way — nothing enters on memory.

/// (word, role, standalone occurrences in the pinned print).
/// Roles: `prep` (and the case it governs, checked by the linter where
/// reliable), `conj`, `part` (particle, enclitics included).
pub const TABLE: &[(&str, &str, u32)] = &[
    // conjunctions
    ("и҆", "conj", 55459),
    ("же", "conj", 8607),
    ("а҆", "conj", 554),
    ("но", "conj", 1219),
    ("бо", "conj", 2621),
    ("да", "conj", 5639),
    ("ꙗ҆́кѡ", "conj", 5774),
    ("ꙗ҆́коже", "conj", 1660),
    ("а҆́ще", "conj", 2261),
    ("є҆гда̀", "conj", 1110),
    ("тогда̀", "conj", 206),
    ("та́кѡ", "conj", 829),
    // particles
    ("не", "part", 9477),
    ("ни", "part", 841),
    ("ли", "part", 937),
    ("сѐ", "part", 1270),
    ("ра́ди", "part", 1199),
    // prepositions
    ("въ", "prep", 12898),
    ("на", "prep", 8680),
    ("къ", "prep", 3177),
    ("ко", "prep", 2134),
    ("съ", "prep", 2562),
    ("со", "prep", 1015),
    ("ѿ", "prep", 9078),
    ("по", "prep", 2562),
    ("до", "prep", 1293),
    ("за", "prep", 597),
    ("при", "prep", 298),
    ("ѡ҆", "prep", 2198),
    ("и҆з̾", "prep", 806),
    ("без̾", "prep", 164),
    ("чрез̾", "prep", 51),
    ("пред̾", "prep", 1925),
    ("над̾", "prep", 667),
    ("под̾", "prep", 347),
    ("междꙋ̀", "prep", 318),
    ("ѡ҆́крестъ", "prep", 356),
];

/// Is this exact spelling a listed function word?
pub fn is_closed(word: &str) -> bool {
    TABLE.iter().any(|&(w, _, _)| w == word)
}

/// The listed role, if any.
pub fn role(word: &str) -> Option<&'static str> {
    TABLE.iter().find(|&&(w, _, _)| w == word).map(|&(_, r, _)| r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_well_formed() {
        for (i, &(w, r, count)) in TABLE.iter().enumerate() {
            assert!(!w.is_empty());
            assert!(matches!(r, "prep" | "conj" | "part"), "{w}");
            assert!(count > 0, "{w}: an unattested word may not enter");
            assert!(
                !TABLE[..i].iter().any(|&(w2, _, _)| w2 == w),
                "{w}: duplicate entry"
            );
        }
        assert!(is_closed("и҆") && role("въ") == Some("prep"));
        assert!(!is_closed("гдⷭ҇ь"));
    }
}
