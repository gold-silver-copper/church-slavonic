//! Regular noun declension: one 21-cell ending row per declension class and
//! recension, selected by inspecting the lemma's ending.
//!
//! The class guess is an approximation in the `english` style: it picks the
//! most common class for each ending and the tables hold what it gets wrong
//! (the `-ь` masculine i-stems `пѫть`/`гость`, the neuter `-ѧ` Synodal
//! lemmas `отроча`, the OCS mobile-vowel stems `отьць`, every consonant
//! mutation the seam rule does not cover). The OCS accusative is answered in
//! its nominative shape and the genitive-shaped animate accusative is a
//! table cell; the Synodal masculine answers the genitive shape (the corpus
//! dictionary's masculine lexemes are three-quarters animate — persons'
//! names above all) and the inanimate nominative shape is the table cell.
//! The Synodal rows carry the print's plural marks (`^`, see
//! [`crate::accent`]): the wide `ѡ`/`є` or the kamora on the genitive,
//! dative and instrumental plural, the dual, and the direct plural of the
//! feminines and neuters that would otherwise read as a singular.

use crate::ChurchSlavonicCore;
use crate::accent::with_accent;
use crate::grammar::*;

impl ChurchSlavonicCore {
    /// Decline a noun by rule. `word` is the nominative-singular lemma in
    /// `recension`'s spelling (accented in Synodal — see [`crate::accent`]);
    /// the answer is in the same spelling.
    pub fn noun(word: &str, case: &Case, number: &Number, recension: &Recension) -> String {
        with_accent(word, recension, |w| {
            Self::noun_skeleton(w, case, number, recension)
        })
    }

    fn noun_skeleton(word: &str, case: &Case, number: &Number, recension: &Recension) -> String {
        let (stem, row) = Self::noun_class(word, case, number, recension);
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
    fn noun_class(
        word: &str,
        case: &Case,
        number: &Number,
        recension: &Recension,
    ) -> (String, &'static Row) {
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
        } else if synodal
            && let Some(stem) = word.strip_suffix("ецъ")
            && stem.chars().last().is_some_and(|c| !is_vowel(c))
        {
            // The fleeting-vowel `-ецъ` masculines (`ѻтецъ` : `ѻтца`).
            return (stem.to_string(), &EC_MASCULINE);
        } else if synodal && (word.ends_with("іа") || word.ends_with("еа")) {
            // The Greek `-іа`/`-еа` feminines (`марі́а` : `марі́и`, `марі́ю`).
            return (strip(word, 1), &IA_FEMININE);
        } else if synodal && word.ends_with("ца") {
            return (strip(word, 1), &CA_FEMININE);
        } else if synodal && word.ends_with("ій") {
            return (strip(word, 1), &JI_MASCULINE);
        } else if synodal && word.ends_with('й') {
            return (strip(word, 1), &J_MASCULINE);
        } else if synodal && word.ends_with("іе") {
            let stem = strip(word, 1);
            // The instrumental plural drops the `і` (`беззако́ньми`).
            if Self::cell(case, number) == 18 {
                return (strip(&stem, 1), &IE_NEUTER);
            }
            return (stem, &IE_NEUTER);
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
        } else if word.ends_with('ꙗ') {
            &JA_SOFT
        } else if word.ends_with('а') && husher(word) {
            &JA_HUSHER
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

fn is_vowel(c: char) -> bool {
    crate::orthography::is_vowel(c)
}

/// A feminine i-stem is guessed from a dental/labial + `ь` ending (`кость`,
/// `заповѣдь`, `любовь`-type aside); `-тель` and the sonorant/husher `-ь`
/// lemmas are taken masculine. `пѫть`, `гость`, `звѣрь`, `голѫбь` are tabled.
/// The `-а` feminines on a husher: a soft stem whose Synodal plural is
/// spelled with the hard letters (`дꙋшѝ` : `дꙋ́шы`, `дꙋ́шъ`).
const JA_HUSHER: Row = Row {
    ocs: JA_SOFT.ocs,
    syn: [
        "а", "и", "и", "ꙋ", "ею", "и", "е", "^и", "ꙋ", "ама", "^и", "ама", "ꙋ", "^и", "^ы", "ъ",
        "амъ", "^ы", "ами", "ахъ", "^ы",
    ],
};
/// The `-ца` feminines (`ѻ҆вца̀`, `пти́ца`): hard endings after the `ц`
/// except the instrumental and vocative.
const CA_FEMININE: Row = Row {
    ocs: JA_SOFT.ocs,
    syn: [
        "а", "ы", "ѣ", "ꙋ", "ею", "ѣ", "е", "^ѣ", "^ꙋ", "ама", "^ѣ", "ама", "^ꙋ", "^ѣ", "^ы", "ъ",
        "амъ", "^ы", "ами", "ахъ", "^ы",
    ],
};
/// The Greek `-іа` feminines (`марі́а`, `а҆ллилꙋ́іа`): the nominative is the
/// lemma, the obliques the soft series.
const IA_FEMININE: Row = Row {
    ocs: JA_SOFT.ocs,
    syn: [
        "=", "и", "и", "ю", "ею", "и", "=", "^и", "ю", "ѧма", "^и", "ѧма", "ю", "^и", "^и", "й",
        "ѧмъ", "^и", "ѧми", "ѧхъ", "^и",
    ],
};
/// The `-й` masculines (`і҆ере́й`, `а҆́рій`, `край`): a soft stem whose
/// nominative is the lemma and whose direct plural is `-є`.
const J_MASCULINE: Row = Row {
    ocs: JO_SOFT_MASCULINE.ocs,
    syn: [
        "=", "ѧ", "ю", "ѧ", "емъ", "и", "е", "^ѧ", "ю", "ема", "^ѧ", "ема", "ю", "^ѧ", "є", "^овъ",
        "^омъ", "^овъ", "^и", "ехъ", "є",
    ],
};
/// The fleeting-vowel `-ецъ` masculines (`ѻ҆те́цъ` : `ѻ҆тца̀`, `ѻ҆тцы̀`,
/// `ѻ҆тє́цъ`): the endings carry the `ц` so the vocative can palatalise it,
/// and the stem is the letters before the fleeting vowel.
const EC_MASCULINE: Row = Row {
    ocs: JO_SOFT_MASCULINE.ocs,
    syn: [
        "=",
        "ца",
        "цꙋ",
        "ца",
        "цемъ",
        "цѣ",
        "че",
        "^ца",
        "^цꙋ",
        "цема",
        "^ца",
        "цема",
        "^цꙋ",
        "^ца",
        "цы",
        "^цевъ",
        "^цемъ",
        "^цевъ",
        "^цы",
        "цѣхъ",
        "цы",
    ],
};
/// The `-ій` masculines of the Greek names (`а҆́рій`, `а҆нало́гій`): the
/// `а`-grade after the `і` (`а҆́ріа`, `а҆нало́гіахъ`).
const JI_MASCULINE: Row = Row {
    ocs: JO_SOFT_MASCULINE.ocs,
    syn: [
        "=", "а", "ю", "а", "емъ", "и", "е", "^а", "ю", "ема", "^а", "ема", "ю", "^а", "и", "^овъ",
        "^омъ", "^овъ", "^ами", "ахъ", "и",
    ],
};
/// The `-іе` neuters (`бдѣ́ніе`, `беззако́ніе`): the genitive plural `-ій`,
/// the instrumental plural on the bare stem (`беззако́ньми`).
const IE_NEUTER: Row = Row {
    ocs: JO_SOFT_NEUTER.ocs,
    syn: [
        "е", "ѧ", "ю", "е", "емъ", "и", "е", "^и", "ю", "ема", "^и", "ема", "ю", "^и", "^ѧ", "й",
        "^емъ", "^ѧ", "ьми", "ихъ", "^ѧ",
    ],
};
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
        "ъ", "а", "ꙋ", "а", "омъ", "ѣ", "е", "^а", "^ꙋ", "ома", "^а", "ома", "^ꙋ", "^а", "и",
        "^овъ", "^омъ", "^овъ", "^ы", "ѣхъ", "и",
    ],
};
const O_HARD_NEUTER: Row = Row {
    ocs: [
        "о", "а", "оу", "о", "омъ", "ѣ", "о", "ѣ", "оу", "ома", "ѣ", "ома", "оу", "ѣ", "а", "ъ",
        "омъ", "а", "ꙑ", "ѣхъ", "а",
    ],
    syn: [
        "о", "а", "ꙋ", "о", "омъ", "ѣ", "о", "^а", "^ꙋ", "ома", "^а", "ома", "^ꙋ", "^а", "^а",
        "^ъ", "^омъ", "^а", "^ы", "ѣхъ", "^а",
    ],
};
const JO_SOFT_MASCULINE: Row = Row {
    ocs: [
        "ь", "ꙗ", "ю", "ь", "ѥмь", "и", "ю", "ꙗ", "ю", "ѥма", "ꙗ", "ѥма", "ю", "ꙗ", "и", "ь",
        "ѥмъ", "ѩ", "и", "ихъ", "и",
    ],
    syn: [
        "ь", "ѧ", "ю", "ѧ", "емъ", "и", "ю", "^ѧ", "ю", "ема", "^ѧ", "ема", "ю", "^ѧ", "и", "ей",
        "^емъ", "ей", "^и", "ехъ", "и",
    ],
};
const JO_SOFT_NEUTER: Row = Row {
    ocs: [
        "ѥ", "ꙗ", "ю", "ѥ", "ѥмь", "и", "ѥ", "и", "ю", "ѥма", "и", "ѥма", "ю", "и", "ꙗ", "ь",
        "ѥмъ", "ꙗ", "и", "ихъ", "ꙗ",
    ],
    syn: [
        "е", "ѧ", "ю", "е", "емъ", "и", "е", "^и", "ю", "ема", "^и", "ема", "ю", "^и", "^ѧ", "ей",
        "^емъ", "^ѧ", "^и", "ѧхъ", "^ѧ",
    ],
};
const A_HARD: Row = Row {
    ocs: [
        "а", "ꙑ", "ѣ", "ѫ", "оѭ", "ѣ", "о", "ѣ", "оу", "ама", "ѣ", "ама", "оу", "ѣ", "ꙑ", "ъ",
        "амъ", "ꙑ", "ами", "ахъ", "ꙑ",
    ],
    syn: [
        "а", "ы", "ѣ", "ꙋ", "ою", "ѣ", "о", "^ѣ", "^ꙋ", "ама", "^ѣ", "ама", "^ꙋ", "^ѣ", "^ы", "ъ",
        "амъ", "^ы", "ами", "ахъ", "^ы",
    ],
};
const JA_SOFT: Row = Row {
    ocs: [
        "ꙗ", "ѩ", "и", "ѭ", "еѭ", "и", "е", "и", "ю", "ꙗма", "и", "ꙗма", "ю", "и", "ѩ", "ь", "ꙗмъ",
        "ѩ", "ꙗми", "ꙗхъ", "ѩ",
    ],
    syn: [
        "ѧ", "и", "и", "ю", "ею", "и", "е", "^и", "ю", "ѧма", "^и", "ѧма", "ю", "^и", "^и", "ь",
        "ѧмъ", "^и", "ѧми", "ѧхъ", "^и",
    ],
};
const I_FEMININE: Row = Row {
    ocs: [
        "ь", "и", "и", "ь", "ьѭ", "и", "и", "и", "ию", "ьма", "и", "ьма", "ию", "и", "и", "ии",
        "ьмъ", "и", "ьми", "ьхъ", "и",
    ],
    syn: [
        "ь", "и", "и", "ь", "ію", "и", "е", "^и", "ію", "ема", "^и", "ема", "ію", "^и", "^и", "ей",
        "емъ", "^и", "ьми", "ехъ", "^и",
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
        "ина",
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
        "^омъ",
        "ъ",
        "^ы",
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
        "=", "е", "и", "ь", "ію", "и", "=", "и", "ію", "ема", "и", "ема", "ію", "и", "и", "ій",
        "емъ", "и", "ьми", "ехъ", "и",
    ],
};
const V_FEMININE: Row = Row {
    ocs: [
        "=", "е", "и", "ь", "ьѭ", "е", "=", "и", "оу", "ама", "и", "ама", "оу", "и", "и", "ъ",
        "амъ", "и", "ами", "ахъ", "и",
    ],
    syn: [
        "=", "е", "и", "ь", "ію", "и", "=", "и", "ію", "ама", "и", "ама", "ію", "и", "и", "ей",
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
            "коней"
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
            "церковію"
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
            "сынѡвъ"
        );
        assert_eq!(decline("сы́нъ", Case::Dative, Number::Plural, SYN), "сы́нѡмъ");
        assert_eq!(
            decline("сы́нъ", Case::Instrumental, Number::Plural, SYN),
            "сы̑ны"
        );
        assert_eq!(decline("ра́бъ", Case::Nominative, Number::Dual, SYN), "ра̑ба");
    }

    #[test]
    fn synodal_plural_marks_and_the_print_classes() {
        use Case::*;
        use Number::*;
        // The wide letter tells the plural from the singular it looks like.
        assert_eq!(decline("а҆́ггелъ", Genitive, Plural, SYN), "а҆́ггелѡвъ");
        assert_eq!(decline("а҆́ггелъ", Dative, Plural, SYN), "а҆́ггелѡмъ");
        // The Synodal masculine accusative is the genitive's shape.
        assert_eq!(decline("а҆́ггелъ", Accusative, Singular, SYN), "а҆́ггела");
        assert_eq!(decline("а҆́ггелъ", Accusative, Plural, SYN), "а҆́ггелѡвъ");
        assert_eq!(decline("рабъ", Accusative, Singular, OCS), "рабъ");
        assert_eq!(decline("бдѣ́ніе", Genitive, Plural, SYN), "бдѣ́ній");
        assert_eq!(decline("бдѣ́ніе", Instrumental, Plural, SYN), "бдѣ́ньми");
        assert_eq!(decline("а҆́рій", Genitive, Singular, SYN), "а҆́ріа");
        assert_eq!(decline("а҆нало́гій", Locative, Plural, SYN), "а҆нало́гіахъ");
        assert_eq!(decline("а҆віге́а", Genitive, Singular, SYN), "а҆віге́и");
        assert_eq!(decline("бе́здна", Nominative, Plural, SYN), "бє́здны");
        assert_eq!(decline("бдѣ́ніе", Nominative, Plural, SYN), "бдѣ̑ніѧ");
        assert_eq!(decline("ко́сть", Accusative, Plural, SYN), "кѡ́сти");
        assert_eq!(decline("рꙋка̀", Nominative, Plural, SYN), "рꙋки̑");
        assert_eq!(decline("гражда́нинъ", Dative, Plural, SYN), "гражда́нѡмъ");
        // -ца, -іа, -й and the fleeting -ецъ.
        assert_eq!(decline("пти́ца", Genitive, Singular, SYN), "пти́цы");
        assert_eq!(decline("пти́ца", Instrumental, Singular, SYN), "пти́цею");
        assert_eq!(decline("пти́ца", Genitive, Plural, SYN), "пти́цъ");
        assert_eq!(decline("дꙋша̀", Nominative, Plural, SYN), "дꙋшы̑");
        assert_eq!(decline("марі́а", Genitive, Singular, SYN), "марі́и");
        assert_eq!(decline("марі́а", Accusative, Singular, SYN), "марі́ю");
        assert_eq!(decline("марі́а", Nominative, Singular, SYN), "марі́а");
        assert_eq!(decline("і҆ере́й", Genitive, Singular, SYN), "і҆ере́ѧ");
        assert_eq!(decline("і҆ере́й", Nominative, Singular, SYN), "і҆ере́й");
        assert_eq!(decline("і҆ере́й", Nominative, Plural, SYN), "і҆ере́є");
        assert_eq!(decline("ѻ҆те́цъ", Genitive, Singular, SYN), "ѻ҆тца̀");
        assert_eq!(decline("ѻ҆те́цъ", Vocative, Singular, SYN), "ѻ҆тчѐ");
        assert_eq!(decline("ѻ҆те́цъ", Genitive, Plural, SYN), "ѻ҆тцє́въ");
        assert_eq!(decline("ѻ҆те́цъ", Dative, Plural, SYN), "ѻ҆тцє́мъ");
        assert_eq!(decline("а҆́гнецъ", Genitive, Singular, SYN), "а҆́гнца");
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
            "гражданъ"
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
