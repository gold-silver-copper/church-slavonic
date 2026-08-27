//! Regular noun declension: one 21-cell ending row per declension class and
//! recension, selected by inspecting the lemma's ending.
//!
//! The class guess is an approximation in the `english` style: it picks the
//! most common class for each ending and the tables hold what it gets wrong
//! (the `-ь` masculine i-stems `пѫть`/`гость`, the neuter `-ѧ` Synodal
//! lemmas `отроча`, the mobile-vowel stems `отьць`/`ѻтецъ`, every consonant
//! mutation the seam rule does not cover). The accusative is answered in its
//! nominative shape; the genitive-shaped animate accusative is a table cell.

use crate::ChurchSlavonicCore;
use crate::grammar::*;

impl ChurchSlavonicCore {
    /// Decline a noun by rule. `word` is the nominative-singular lemma in
    /// `recension`'s spelling; the answer is in the same spelling.
    pub fn noun(word: &str, case: &Case, number: &Number, recension: &Recension) -> String {
        let (stem, row) = Self::noun_class(word, recension);
        let cell = Self::cell(case, number);
        let ending = match recension {
            Recension::OldChurchSlavonic => row.ocs[cell],
            Recension::Synodal => row.syn[cell],
        };
        // The citation cell of the athematic classes is the lemma itself
        // (`мати`, `имѧ`, `свекрꙑ`): the extended stem never surfaces there.
        if ending == "=" {
            return word.to_string();
        }
        Self::attach(&stem, ending, recension)
    }

    /// Pick the declension row and the stem it attaches to. Order is
    /// load-bearing: whole-word lists first, then the longest suffixes.
    fn noun_class(word: &str, recension: &Recension) -> (String, &'static Row) {
        let synodal = *recension == Recension::Synodal;
        if let Some(stem) = Self::pair_match(word, R_STEMS) {
            return (stem, &R_FEMININE);
        }
        if let Some(stem) = Self::pair_match(word, S_STEMS) {
            return (stem, &S_NEUTER);
        }
        if !synodal && U_STEMS.contains(&word) {
            return (strip(word, 1), &U_MASCULINE);
        }
        let husher = |w: &str| {
            matches!(
                strip(w, 1).chars().last(),
                Some('ж' | 'ч' | 'ш' | 'щ' | 'ц')
            )
        };
        let row: &Row = if let Some(stem) = word.strip_suffix("мѧ") {
            return (format!("{stem}мен"), &N_NEUTER);
        } else if let Some(stem) = word.strip_suffix("анинъ") {
            return (format!("{stem}ан"), &IN_SINGULATIVE);
        } else if let Some(stem) = word.strip_suffix("ѣнинъ") {
            return (format!("{stem}ѣн"), &IN_SINGULATIVE);
        } else if let Some(stem) = word.strip_suffix("ѧнинъ") {
            return (format!("{stem}ѧн"), &IN_SINGULATIVE);
        } else if !synodal && word.ends_with('ꙑ') {
            return (format!("{}ъв", strip(word, 1)), &V_FEMININE);
        } else if synodal && word.ends_with("овь") {
            return (strip(word, 1), &V_FEMININE);
        } else if word.ends_with("тель") {
            &AGENT
        } else if word.ends_with('ѧ') {
            // OCS nt-stem neuters (`отрочѧ`); Synodal soft feminines
            // (`землѧ`) — the Synodal nt-stems spell `-а` after a husher.
            if synodal {
                &JA_SOFT
            } else {
                return (format!("{}ѧт", strip(word, 1)), &NT_NEUTER);
            }
        } else if word.ends_with('ꙗ') || (word.ends_with('а') && husher(word)) {
            &JA_SOFT
        } else if word.ends_with('а') {
            &A_HARD
        } else if word.ends_with('о') {
            &O_HARD_NEUTER
        } else if word.ends_with('е') || word.ends_with('ѥ') {
            &JO_SOFT_NEUTER
        } else if word.ends_with('ъ') {
            &O_HARD_MASCULINE
        } else if Self::iter_replace_last(word, I_FEMININE_SUFFIXES).is_some() {
            &I_FEMININE
        } else if word.ends_with('ь') || word.ends_with('и') || word.ends_with('й') {
            &JO_SOFT_MASCULINE
        } else {
            return (word.to_string(), &O_HARD_MASCULINE);
        };
        (strip(word, 1), row)
    }
}

fn strip(word: &str, chars: usize) -> String {
    let n = word.chars().count().saturating_sub(chars);
    word.chars().take(n).collect()
}

/// A feminine i-stem is guessed from a dental/labial + `ь` ending (`кость`,
/// `заповѣдь`, `любовь`-type aside); `-тель` and the sonorant/husher `-ь`
/// lemmas are taken masculine. `пѫть`, `гость`, `звѣрь`, `голѫбь` are tabled.
const I_FEMININE_SUFFIXES: &[(&str, &str)] = &[
    ("сть", ""),
    ("ть", ""),
    ("дь", ""),
    ("зь", ""),
    ("сь", ""),
    ("вь", ""),
    ("бь", ""),
    ("пь", ""),
    ("мь", ""),
];

/// The closed OCS u-stems (Polivanova §333); Synodal declines them as
/// first-declension hard masculines with the u-stem endings as variants.
const U_STEMS: &[&str] = &["сꙑнъ", "домъ", "врьхъ", "медъ", "полъ", "волъ"];
/// The r-stems, lemma -> extended stem.
const R_STEMS: &[(&str, &str)] = &[("мати", "матер"), ("дъщи", "дъщер"), ("дщи", "дщер")];
/// The s-stems, lemma -> extended stem.
const S_STEMS: &[(&str, &str)] = &[
    ("слово", "словес"),
    ("небо", "небес"),
    ("тѣло", "тѣлес"),
    ("чоудо", "чоудес"),
    ("чꙋдо", "чꙋдес"),
    ("дрѣво", "дрѣвес"),
    ("древо", "древес"),
    ("коло", "колес"),
];

/// One declension class: 21 endings per recension in [`Case`] order for the
/// singular, dual and plural. `=` marks the lemma's own citation cell.
struct Row {
    ocs: [&'static str; 21],
    syn: [&'static str; 21],
}

// The Synodal columns are the Alypy §§33–44 primaries; the OCS columns are
// Polivanova's tables 327/339/343/351 and the athematic tables. Cells that
// differ beyond spelling are the divergence registry's recension conditions:
// the `-ѥмь`/`-емъ` instrumental, the `-ь`/`-ей` soft genitive plural, the
// `-ѩ`/`-и` soft direct plural, the `-ѩ`/`-и` ja-stem genitive singular,
// the Synodal `-овъ` genitive plural import, the neuter dual `-ѣ`/`-а`,
// the athematic locative `-е`/`-и`, and the u-stem dissolution.
const O_HARD_MASCULINE: Row = Row {
    ocs: [
        "ъ", "а", "оу", "ъ", "омъ", "ѣ", "е", "а", "оу", "ома", "а", "ома", "оу", "а", "и", "ъ",
        "омъ", "ꙑ", "ꙑ", "ѣхъ", "и",
    ],
    syn: [
        "ъ", "а", "ꙋ", "ъ", "омъ", "ѣ", "е", "а", "ꙋ", "ома", "а", "ома", "ꙋ", "а", "и", "овъ",
        "омъ", "ы", "ы", "ѣхъ", "и",
    ],
};
const O_HARD_NEUTER: Row = Row {
    ocs: [
        "о", "а", "оу", "о", "омъ", "ѣ", "о", "ѣ", "оу", "ома", "ѣ", "ома", "оу", "ѣ", "а", "ъ",
        "омъ", "а", "ꙑ", "ѣхъ", "а",
    ],
    syn: [
        "о", "а", "ꙋ", "о", "омъ", "ѣ", "о", "а", "ꙋ", "ома", "а", "ома", "ꙋ", "а", "а", "ъ",
        "омъ", "а", "ы", "ѣхъ", "а",
    ],
};
const JO_SOFT_MASCULINE: Row = Row {
    ocs: [
        "ь", "ꙗ", "ю", "ь", "ѥмь", "и", "ю", "ꙗ", "ю", "ѥма", "ꙗ", "ѥма", "ю", "ꙗ", "и", "ь",
        "ѥмъ", "ѩ", "и", "ихъ", "и",
    ],
    syn: [
        "ь", "ѧ", "ю", "ь", "емъ", "и", "ю", "ѧ", "ю", "ема", "ѧ", "ема", "ю", "ѧ", "и", "ей",
        "емъ", "и", "и", "ехъ", "и",
    ],
};
const JO_SOFT_NEUTER: Row = Row {
    ocs: [
        "ѥ", "ꙗ", "ю", "ѥ", "ѥмь", "и", "ѥ", "и", "ю", "ѥма", "и", "ѥма", "ю", "и", "ꙗ", "ь",
        "ѥмъ", "ꙗ", "и", "ихъ", "ꙗ",
    ],
    syn: [
        "е", "ѧ", "ю", "е", "емъ", "и", "е", "и", "ю", "ема", "и", "ема", "ю", "и", "ѧ", "ей",
        "емъ", "ѧ", "и", "ѧхъ", "ѧ",
    ],
};
const A_HARD: Row = Row {
    ocs: [
        "а", "ꙑ", "ѣ", "ѫ", "оѭ", "ѣ", "о", "ѣ", "оу", "ама", "ѣ", "ама", "оу", "ѣ", "ꙑ", "ъ",
        "амъ", "ꙑ", "ами", "ахъ", "ꙑ",
    ],
    syn: [
        "а", "ы", "ѣ", "ꙋ", "ою", "ѣ", "о", "ѣ", "ꙋ", "ама", "ѣ", "ама", "ꙋ", "ѣ", "ы", "ъ", "амъ",
        "ы", "ами", "ахъ", "ы",
    ],
};
const JA_SOFT: Row = Row {
    ocs: [
        "ꙗ", "ѩ", "и", "ѭ", "еѭ", "и", "е", "и", "ю", "ꙗма", "и", "ꙗма", "ю", "и", "ѩ", "ь", "ꙗмъ",
        "ѩ", "ꙗми", "ꙗхъ", "ѩ",
    ],
    syn: [
        "ѧ", "и", "и", "ю", "ею", "и", "е", "и", "ю", "ѧма", "и", "ѧма", "ю", "и", "и", "ь", "ѧмъ",
        "и", "ѧми", "ѧхъ", "и",
    ],
};
const I_FEMININE: Row = Row {
    ocs: [
        "ь", "и", "и", "ь", "ьѭ", "и", "и", "и", "ию", "ьма", "и", "ьма", "ию", "и", "и", "ии",
        "ьмъ", "и", "ьми", "ьхъ", "и",
    ],
    syn: [
        "ь", "и", "и", "ь", "їю", "и", "е", "и", "їю", "ема", "и", "ема", "їю", "и", "и", "ей",
        "емъ", "и", "ьми", "ехъ", "и",
    ],
};
const U_MASCULINE: Row = Row {
    ocs: [
        "ъ", "оу", "ови", "ъ", "ъмь", "оу", "оу", "ꙑ", "овоу", "ъма", "ꙑ", "ъма", "овоу", "ꙑ",
        "ове", "овъ", "ъмъ", "ꙑ", "ъми", "ъхъ", "ове",
    ],
    syn: O_HARD_MASCULINE.syn,
};
/// The `-инъ` singulative: singular and dual on the full stem as a hard
/// masculine, plural on the syncopated stem (`гражданинъ` : `граждане`).
const IN_SINGULATIVE: Row = Row {
    ocs: [
        "инъ",
        "ина",
        "иноу",
        "инъ",
        "иномъ",
        "инѣ",
        "ине",
        "ина",
        "иноу",
        "инома",
        "ина",
        "инома",
        "иноу",
        "ина",
        "е",
        "ъ",
        "омъ",
        "ꙑ",
        "ꙑ",
        "ѣхъ",
        "е",
    ],
    syn: [
        "инъ",
        "ина",
        "инꙋ",
        "инъ",
        "иномъ",
        "инѣ",
        "ине",
        "ина",
        "инꙋ",
        "инома",
        "ина",
        "инома",
        "инꙋ",
        "ина",
        "е",
        "ъ",
        "омъ",
        "е",
        "ы",
        "ѣхъ",
        "е",
    ],
};
/// The `-тель` agent nouns: a soft masculine whose OCS direct plural is `-ѥ`.
const AGENT: Row = Row {
    ocs: [
        "ь", "ꙗ", "ю", "ь", "ѥмь", "и", "ю", "ꙗ", "ю", "ѥма", "ꙗ", "ѥма", "ю", "ꙗ", "ѥ", "ь",
        "ѥмъ", "ѩ", "и", "ихъ", "ѥ",
    ],
    syn: JO_SOFT_MASCULINE.syn,
};
// Athematic classes: endings after the extended stem (`имен-`, `отрочѧт-`,
// `матер-`, `словес-`, `свекръв-`/`церков-`).
const N_NEUTER: Row = Row {
    ocs: [
        "=", "е", "и", "=", "ьмь", "е", "=", "ѣ", "оу", "ьма", "ѣ", "ьма", "оу", "ѣ", "а", "ъ",
        "ьмъ", "а", "ꙑ", "ьхъ", "а",
    ],
    syn: [
        "=", "е", "и", "=", "емъ", "и", "=", "и", "ꙋ", "ема", "и", "ема", "ꙋ", "и", "а", "ъ",
        "ємъ", "а", "ы", "ѣхъ", "а",
    ],
};
const NT_NEUTER: Row = N_NEUTER;
const S_NEUTER: Row = Row {
    ocs: N_NEUTER.ocs,
    syn: [
        "=", "е", "и", "=", "емъ", "и", "=", "и", "ꙋ", "ема", "и", "ема", "ꙋ", "и", "а", "ъ",
        "ємъ", "а", "ы", "ѣхъ", "а",
    ],
};
const R_FEMININE: Row = Row {
    ocs: [
        "=", "е", "и", "ь", "ьѭ", "и", "=", "и", "оу", "ьма", "и", "ьма", "оу", "и", "и", "ъ",
        "ьмъ", "и", "ьми", "ьхъ", "и",
    ],
    syn: [
        "=", "е", "и", "ь", "їю", "и", "=", "и", "їю", "ема", "и", "ема", "їю", "и", "и", "їй",
        "емъ", "и", "ьми", "ехъ", "и",
    ],
};
const V_FEMININE: Row = Row {
    ocs: [
        "=", "е", "и", "ь", "ьѭ", "е", "=", "и", "оу", "ама", "и", "ама", "оу", "и", "и", "ъ",
        "амъ", "и", "ами", "ахъ", "и",
    ],
    syn: [
        "=", "е", "и", "ь", "їю", "и", "=", "и", "їю", "ама", "и", "ама", "їю", "и", "и", "ей",
        "амъ", "и", "ами", "ахъ", "и",
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::Synodal;

    fn decline(word: &str, case: Case, number: Number, recension: Recension) -> String {
        ChurchSlavonicCore::noun(word, &case, &number, &recension)
    }

    #[test]
    fn instrumental_singular_jer_condition_holds() {
        // noun:instrumental-singular-jer — OCS soft `-ѥмь` against Synodal `-емъ`.
        assert_eq!(
            decline("конь", Case::Instrumental, Number::Singular, OCS),
            "конѥмь"
        );
        assert_eq!(
            decline("конь", Case::Instrumental, Number::Singular, SYN),
            "конемъ"
        );
        assert_eq!(
            decline("мѫжь", Case::Instrumental, Number::Singular, OCS),
            "мѫжемь"
        );
    }

    #[test]
    fn soft_plurals_are_re_inventoried_in_synodal() {
        // noun:soft-genitive-plural-reinventory and noun:soft-direct-plural-leveling.
        assert_eq!(decline("конь", Case::Genitive, Number::Plural, OCS), "конь");
        assert_eq!(
            decline("конь", Case::Genitive, Number::Plural, SYN),
            "коней"
        );
        assert_eq!(
            decline("конь", Case::Accusative, Number::Plural, OCS),
            "конѩ"
        );
        assert_eq!(
            decline("конь", Case::Accusative, Number::Plural, SYN),
            "кони"
        );
        assert_eq!(
            decline("землꙗ", Case::Genitive, Number::Singular, OCS),
            "землѩ"
        );
        assert_eq!(
            decline("землѧ", Case::Genitive, Number::Singular, SYN),
            "земли"
        );
    }

    #[test]
    fn athematic_stems_extend_and_keep_their_citation_cell() {
        assert_eq!(
            decline("имѧ", Case::Nominative, Number::Singular, OCS),
            "имѧ"
        );
        assert_eq!(
            decline("имѧ", Case::Genitive, Number::Singular, OCS),
            "имене"
        );
        // noun:consonant-locative-singular-i
        assert_eq!(
            decline("имѧ", Case::Locative, Number::Singular, OCS),
            "имене"
        );
        assert_eq!(
            decline("имѧ", Case::Locative, Number::Singular, SYN),
            "имени"
        );
        assert_eq!(
            decline("мати", Case::Genitive, Number::Singular, SYN),
            "матере"
        );
        assert_eq!(
            decline("мати", Case::Instrumental, Number::Singular, OCS),
            "матерьѭ"
        );
        assert_eq!(
            decline("слово", Case::Genitive, Number::Plural, SYN),
            "словесъ"
        );
        assert_eq!(
            decline("свекрꙑ", Case::Genitive, Number::Singular, OCS),
            "свекръве"
        );
        assert_eq!(
            decline("церковь", Case::Instrumental, Number::Singular, SYN),
            "церковїю"
        );
        assert_eq!(
            decline("отрочѧ", Case::Genitive, Number::Singular, OCS),
            "отрочѧте"
        );
    }

    #[test]
    fn u_stem_dissolves_into_the_synodal_first_declension() {
        // noun:u-stem-dissolution
        assert_eq!(
            decline("сꙑнъ", Case::Dative, Number::Singular, OCS),
            "сꙑнови"
        );
        assert_eq!(
            decline("сꙑнъ", Case::Nominative, Number::Plural, OCS),
            "сꙑнове"
        );
        assert_eq!(decline("сынъ", Case::Dative, Number::Singular, SYN), "сынꙋ");
        assert_eq!(
            decline("сынъ", Case::Genitive, Number::Plural, SYN),
            "сыновъ"
        );
    }

    #[test]
    fn singulative_and_agent_plurals() {
        assert_eq!(
            decline("гражданинъ", Case::Nominative, Number::Plural, SYN),
            "граждане"
        );
        assert_eq!(
            decline("гражданинъ", Case::Genitive, Number::Singular, SYN),
            "гражданина"
        );
        // noun:in-singulative-inanimate-accusative
        assert_eq!(
            decline("гражданинъ", Case::Accusative, Number::Plural, OCS),
            "гражданꙑ"
        );
        assert_eq!(
            decline("гражданинъ", Case::Accusative, Number::Plural, SYN),
            "граждане"
        );
        // noun:agent-plural-reinventory
        assert_eq!(
            decline("оучитель", Case::Nominative, Number::Plural, OCS),
            "оучителѥ"
        );
        assert_eq!(
            decline("ѹчитель", Case::Nominative, Number::Plural, SYN),
            "ѹчители"
        );
    }
}
