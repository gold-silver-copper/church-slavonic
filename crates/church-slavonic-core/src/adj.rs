//! Regular adjective declension and comparison.
//!
//! Two agreement classes (hard, soft) in two declensions (short/nominal,
//! long/compound), each a 21-cell row per gender and recension. The lemma is
//! the masculine nominative singular; its ending picks the class: `-ъ`/`-ь`
//! short, OCS `-ꙑи`/`-ии` and Synodal `-ый`/`-їй`/`-ій` long (a velar before
//! the Synodal `-ій` is the hard class: `благій`). Comparison is suffixal
//! `-ѣиш-`/`-ѣйш-` declined as a soft stem; the superlative is the `пре-`
//! prefix on the positive. Consonant-mutating comparatives (`болии`,
//! `лоучии`) and the suppletive pairs are tabled. The Synodal rows carry the
//! print's plural marks (`^`, see [`crate::accent`]) on the dual and on the
//! direct plural cells that would otherwise read as a singular; the short
//! `-енъ` adjectives with an unstressed fleeting vowel (`а҆́лченъ` :
//! `а҆́лчна`) drop it before a vowel ending.

use crate::ChurchSlavonicCore;
use crate::accent::with_accent;
use crate::grammar::*;

impl ChurchSlavonicCore {
    /// Decline (and grade) an adjective by rule. The lemma is accented in
    /// Synodal (see [`crate::accent`]).
    pub fn adj(
        word: &str,
        case: &Case,
        number: &Number,
        gender: &Gender,
        degree: &Degree,
        recension: &Recension,
    ) -> String {
        let fleeting = *recension == Recension::Synodal && has_fleeting_en(word);
        with_accent(word, recension, |w| {
            Self::adj_skeleton(w, case, number, gender, degree, recension, fleeting)
        })
    }

    fn adj_skeleton(
        word: &str,
        case: &Case,
        number: &Number,
        gender: &Gender,
        degree: &Degree,
        recension: &Recension,
        fleeting: bool,
    ) -> String {
        let synodal = *recension == Recension::Synodal;
        let (stem, long, hard) = Self::adj_class(word, recension);
        let stem = if fleeting && *degree == Degree::Positive {
            if *gender == Gender::Masculine
                && *number == Number::Singular
                && matches!(case, Case::Nominative | Case::Accusative)
            {
                return word.to_string();
            }
            format!("{}н", &stem[..stem.len() - "ен".len()])
        } else {
            stem
        };
        let (stem, long, hard) = match degree {
            Degree::Positive => (stem, long, hard),
            Degree::Comparative => {
                // Short masculine/neuter nominative: `новѣи`/`новѣѥ` (OCS),
                // `новѣй`/`новѣе` (Synodal); every other cell is the soft
                // `-ѣиш-`/`-ѣйш-` stem.
                let direct = matches!(case, Case::Nominative | Case::Vocative)
                    && *number == Number::Singular
                    && *gender != Gender::Feminine;
                if !long && direct {
                    let ending = match (gender, synodal) {
                        (Gender::Masculine, false) => "ѣи",
                        (Gender::Masculine, true) => "ѣй",
                        (_, false) => "ѣѥ",
                        (_, true) => "ѣе",
                    };
                    return format!("{stem}{ending}");
                }
                (
                    format!("{stem}{}", if synodal { "ѣйш" } else { "ѣиш" }),
                    long,
                    false,
                )
            }
            Degree::Superlative => (format!("пре{stem}"), long, hard),
        };
        let row = match (long, hard) {
            (false, true) => &SHORT_HARD,
            (false, false) => &SHORT_SOFT,
            (true, true) => &LONG_HARD,
            (true, false) => &LONG_SOFT,
        };
        let cell = Self::cell(case, number);
        let table = match (recension, gender) {
            (Recension::OldChurchSlavonic, Gender::Masculine) => &row.ocs[0],
            (Recension::OldChurchSlavonic, Gender::Feminine) => &row.ocs[1],
            (Recension::OldChurchSlavonic, Gender::Neuter) => &row.ocs[2],
            (Recension::Synodal, Gender::Masculine) => &row.syn[0],
            (Recension::Synodal, Gender::Feminine) => &row.syn[1],
            (Recension::Synodal, Gender::Neuter) => &row.syn[2],
        };
        Self::attach(&stem, table[cell], recension)
    }

    /// `(stem, long, hard)` from the masculine nominative-singular lemma.
    fn adj_class(word: &str, recension: &Recension) -> (String, bool, bool) {
        let chars: Vec<char> = word.chars().collect();
        let cut = |n: usize| {
            chars[..chars.len().saturating_sub(n)]
                .iter()
                .collect::<String>()
        };
        let velar = |n: usize| {
            matches!(
                chars.get(chars.len().wrapping_sub(n)),
                Some('к' | 'г' | 'х')
            )
        };
        match recension {
            Recension::OldChurchSlavonic => {
                if word.ends_with("ꙑи") {
                    (cut(2), true, true)
                } else if word.ends_with("ии") {
                    (cut(2), true, false)
                } else if word.ends_with('ь') {
                    (cut(1), false, false)
                } else {
                    (cut(1), false, true)
                }
            }
            Recension::Synodal => {
                if word.ends_with("ый") {
                    (cut(2), true, true)
                } else if word.ends_with("ій") {
                    (cut(2), true, velar(3))
                } else if word.ends_with('ь') {
                    (cut(1), false, false)
                } else {
                    (cut(1), false, true)
                }
            }
        }
    }
}

/// A short adjective in unstressed `-енъ` after a consonant has a fleeting
/// vowel (`а҆́лченъ` : `а҆́лчна`, `вѣ́ренъ` : `вѣ́рна`); the stressed `-е́нъ`
/// (`блаже́нъ` : `блаже́на`) keeps it. The lemma is the accented citation.
fn has_fleeting_en(word: &str) -> bool {
    let skeleton = crate::orthography::strip_marks(word);
    let Some(before) = skeleton.strip_suffix("енъ") else {
        return false;
    };
    let consonant_before = before
        .chars()
        .last()
        .is_some_and(|c| !crate::orthography::is_vowel(c));
    let vowels = skeleton
        .chars()
        .filter(|c| crate::orthography::is_vowel(*c))
        .count();
    // The stressed vowel is the fleeting one when the stress is the last vowel.
    let stressed_last = crate::orthography::units(word)
        .iter()
        .filter(|u| u.is_vowel())
        .enumerate()
        .any(|(i, u)| u.has_stress() && i + 1 == vowels);
    consonant_before && vowels >= 2 && !stressed_last
}

/// `[masculine, feminine, neuter]` rows of 21 cells per recension.
struct Row {
    ocs: [[&'static str; 21]; 3],
    syn: [[&'static str; 21]; 3],
}

// Beyond spelling the columns carry the divergence registry's adjective
// conditions: adj:short-oblique-pronominalization (Synodal `-ымъ/-ыхъ/-ыми`
// short obliques for the OCS nominal `-омь/-ъ/-ꙑ`),
// adj:soft-short-palatal-vowel-series (Synodal `ѧ/ю/и` after the soft stem),
// adj:short-vocative-leveling, adj:long-contraction (`аѥго` -> `агѡ`,
// `ꙑимь` -> `ымъ`) and adj:soft-long-vowel-grade (`аꙗ` -> `ѧѧ`, `ии` -> `ей`).
const SHORT_HARD: Row = Row {
    ocs: [
        [
            "ъ", "а", "оу", "ъ", "омь", "ѣ", "е", "а", "оу", "ома", "а", "ома", "оу", "а", "и",
            "ъ", "омъ", "ꙑ", "ꙑ", "ѣхъ", "и",
        ],
        [
            "а", "ꙑ", "ѣ", "ѫ", "оѭ", "ѣ", "о", "ѣ", "оу", "ама", "ѣ", "ама", "оу", "ѣ", "ꙑ", "ъ",
            "амъ", "ꙑ", "ами", "ахъ", "ꙑ",
        ],
        [
            "о", "а", "оу", "о", "омь", "ѣ", "о", "ѣ", "оу", "ома", "ѣ", "ома", "оу", "ѣ", "а",
            "ъ", "омъ", "а", "ꙑ", "ѣхъ", "а",
        ],
    ],
    syn: [
        [
            "ъ", "а", "ꙋ", "ъ", "ымъ", "ѣ", "е", "^а", "^ꙋ", "ыма", "^а", "ыма", "^ꙋ", "^а", "и",
            "ыхъ", "^ымъ", "^ы", "^ы", "ыхъ", "и",
        ],
        [
            "а", "ы", "ѣ", "ꙋ", "ою", "ѣ", "а", "^ѣ", "^ꙋ", "ыма", "^ѣ", "ыма", "^ꙋ", "^ѣ", "^ы",
            "ыхъ", "^ымъ", "^ы", "^ы", "ыхъ", "^ы",
        ],
        [
            "о", "а", "ꙋ", "о", "ымъ", "ѣ", "о", "^ѣ", "^ꙋ", "ыма", "^ѣ", "ыма", "^ꙋ", "^ѣ", "^а",
            "ыхъ", "^ымъ", "^а", "^ы", "ыхъ", "^а",
        ],
    ],
};
const SHORT_SOFT: Row = Row {
    ocs: [
        [
            "ь", "а", "оу", "ь", "емь", "и", "е", "а", "оу", "ема", "а", "ема", "оу", "а", "и",
            "ь", "емъ", "ѧ", "и", "ихъ", "и",
        ],
        [
            "а", "ѧ", "и", "ѫ", "еѭ", "и", "а", "и", "оу", "ама", "и", "ама", "оу", "и", "ѧ", "ь",
            "амъ", "ѧ", "ами", "ахъ", "ѧ",
        ],
        [
            "е", "а", "оу", "е", "емь", "и", "е", "и", "оу", "ема", "и", "ема", "оу", "и", "а",
            "ь", "емъ", "а", "и", "ихъ", "а",
        ],
    ],
    syn: [
        [
            "ь", "ѧ", "ю", "ь", "имъ", "и", "ь", "^ѧ", "ю", "има", "^ѧ", "има", "ю", "^ѧ", "и",
            "ихъ", "^имъ", "и", "^и", "ихъ", "и",
        ],
        [
            "ѧ", "и", "и", "ю", "ею", "и", "ѧ", "^и", "ю", "има", "^и", "има", "ю", "^и", "и",
            "ихъ", "^имъ", "и", "^и", "ихъ", "и",
        ],
        [
            "е", "ѧ", "ю", "е", "имъ", "и", "е", "^и", "ю", "има", "^и", "има", "ю", "^и", "ѧ",
            "ихъ", "^имъ", "ѧ", "^и", "ихъ", "ѧ",
        ],
    ],
};
const LONG_HARD: Row = Row {
    ocs: [
        [
            "ꙑи",
            "аѥго",
            "оуѥмоу",
            "ꙑи",
            "ꙑимь",
            "ѣѥмь",
            "ꙑи",
            "аꙗ",
            "оую",
            "ꙑима",
            "аꙗ",
            "ꙑима",
            "оую",
            "аꙗ",
            "ии",
            "ꙑихъ",
            "ꙑимъ",
            "ꙑѩ",
            "ꙑими",
            "ꙑихъ",
            "ии",
        ],
        [
            "аꙗ",
            "ꙑѩ",
            "ѣи",
            "ѫѭ",
            "ѫѭ",
            "ѣи",
            "аꙗ",
            "ѣи",
            "оую",
            "ꙑима",
            "ѣи",
            "ꙑима",
            "оую",
            "ѣи",
            "ꙑѩ",
            "ꙑихъ",
            "ꙑимъ",
            "ꙑѩ",
            "ꙑими",
            "ꙑихъ",
            "ꙑѩ",
        ],
        [
            "оѥ",
            "аѥго",
            "оуѥмоу",
            "оѥ",
            "ꙑимь",
            "ѣѥмь",
            "оѥ",
            "ѣи",
            "оую",
            "ꙑима",
            "ѣи",
            "ꙑима",
            "оую",
            "ѣи",
            "аꙗ",
            "ꙑихъ",
            "ꙑимъ",
            "аꙗ",
            "ꙑими",
            "ꙑихъ",
            "аꙗ",
        ],
    ],
    syn: [
        [
            "ый", "агѡ", "омꙋ", "ый", "ымъ", "ѣмъ", "ый", "^аѧ", "^ꙋю", "ыма", "^аѧ", "ыма", "^ꙋю",
            "^аѧ", "іи", "ыхъ", "^ымъ", "^ыѧ", "ыми", "ыхъ", "іи",
        ],
        [
            "аѧ", "ыѧ", "ѣй", "ꙋю", "ою", "ѣй", "аѧ", "^ѣи", "^ꙋю", "ыма", "^ѣи", "ыма", "^ꙋю",
            "^ѣи", "^ыѧ", "ыхъ", "^ымъ", "^ыѧ", "ыми", "ыхъ", "^ыѧ",
        ],
        [
            "ое", "агѡ", "омꙋ", "ое", "ымъ", "ѣмъ", "ое", "^ѣи", "^ꙋю", "ыма", "^ѣи", "ыма", "^ꙋю",
            "^ѣи", "^аѧ", "ыхъ", "^ымъ", "^аѧ", "ыми", "ыхъ", "^аѧ",
        ],
    ],
};
const LONG_SOFT: Row = Row {
    ocs: [
        [
            "ии",
            "аѥго",
            "оуѥмоу",
            "ии",
            "иимь",
            "иѥмь",
            "ии",
            "аꙗ",
            "оую",
            "иима",
            "аꙗ",
            "иима",
            "оую",
            "аꙗ",
            "ии",
            "иихъ",
            "иимъ",
            "ѧѩ",
            "иими",
            "иихъ",
            "ии",
        ],
        [
            "аꙗ", "ѧѩ", "ии", "ѫѭ", "еѭ", "ии", "аꙗ", "ии", "оую", "иима", "ии", "иима", "оую",
            "ии", "ѧѩ", "иихъ", "иимъ", "ѧѩ", "иими", "иихъ", "ѧѩ",
        ],
        [
            "еѥ",
            "аѥго",
            "оуѥмоу",
            "еѥ",
            "иимь",
            "иѥмь",
            "еѥ",
            "ии",
            "оую",
            "иима",
            "ии",
            "иима",
            "оую",
            "ии",
            "аꙗ",
            "иихъ",
            "иимъ",
            "аꙗ",
            "иими",
            "иихъ",
            "аꙗ",
        ],
    ],
    syn: [
        [
            "ій", "ѧгѡ", "емꙋ", "ій", "имъ", "емъ", "ій", "^ѧѧ", "^юю", "има", "^ѧѧ", "има", "^юю",
            "^ѧѧ", "іи", "ихъ", "^имъ", "^іѧ", "ими", "ихъ", "іи",
        ],
        [
            "ѧѧ", "іѧ", "ей", "юю", "ею", "ей", "ѧѧ", "^іи", "^юю", "има", "^іи", "има", "^юю",
            "^іи", "^іѧ", "ихъ", "^имъ", "^іѧ", "ими", "ихъ", "^іѧ",
        ],
        [
            "ее", "ѧгѡ", "емꙋ", "ее", "имъ", "емъ", "ее", "^іи", "^юю", "има", "^іи", "има", "^юю",
            "^іи", "^ѧѧ", "ихъ", "^имъ", "^ѧѧ", "ими", "ихъ", "^ѧѧ",
        ],
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::Synodal;

    fn adj(w: &str, c: Case, n: Number, g: Gender, d: Degree, r: Recension) -> String {
        ChurchSlavonicCore::adj(w, &c, &n, &g, &d, &r)
    }

    #[test]
    fn long_contraction_and_short_pronominalization_hold() {
        use Case::*;
        use Gender::*;
        use Number::*;
        let pos = Degree::Positive;
        // adj:long-contraction
        assert_eq!(
            adj("новꙑи", Genitive, Singular, Masculine, pos, OCS),
            "новаѥго"
        );
        assert_eq!(
            adj("новый", Genitive, Singular, Masculine, pos, SYN),
            "новагѡ"
        );
        assert_eq!(
            adj("новꙑи", Instrumental, Singular, Neuter, pos, OCS),
            "новꙑимь"
        );
        assert_eq!(
            adj("новый", Instrumental, Singular, Neuter, pos, SYN),
            "новымъ"
        );
        // adj:short-oblique-pronominalization
        assert_eq!(
            adj("новъ", Instrumental, Singular, Masculine, pos, OCS),
            "новомь"
        );
        assert_eq!(
            adj("новъ", Instrumental, Singular, Masculine, pos, SYN),
            "новымъ"
        );
        assert_eq!(adj("новъ", Genitive, Plural, Feminine, pos, OCS), "новъ");
        assert_eq!(adj("новъ", Genitive, Plural, Feminine, pos, SYN), "новыхъ");
        // the print's plural marks and the fleeting -енъ
        assert_eq!(
            adj("до́брый", Nominative, Plural, Neuter, pos, SYN),
            "дѡ́браѧ"
        );
        assert_eq!(adj("до́брый", Dative, Plural, Neuter, pos, SYN), "дѡ́брымъ");
        assert_eq!(
            adj("а҆́динъ", Instrumental, Plural, Masculine, pos, SYN),
            "а҆̑дины"
        );
        assert_eq!(
            adj("а҆враа́мскій", Accusative, Plural, Masculine, pos, SYN),
            "а҆враа̑мскіѧ"
        );
        assert_eq!(
            adj("свѧты́й", Accusative, Plural, Masculine, pos, SYN),
            "свѧты̑ѧ"
        );
        assert_eq!(
            adj("а҆́лченъ", Genitive, Singular, Masculine, pos, SYN),
            "а҆́лчна"
        );
        assert_eq!(
            adj("а҆́лченъ", Nominative, Singular, Masculine, pos, SYN),
            "а҆́лченъ"
        );
        assert_eq!(
            adj("блаже́нъ", Genitive, Singular, Masculine, pos, SYN),
            "блаже́на"
        );
        assert_eq!(
            adj("а҆́ггельскій", Nominative, Plural, Masculine, pos, SYN),
            "а҆́ггельстіи"
        );
        // adj:short-vocative-leveling
        assert_eq!(adj("новъ", Vocative, Singular, Feminine, pos, OCS), "ново");
        assert_eq!(adj("новъ", Vocative, Singular, Feminine, pos, SYN), "нова");
    }

    #[test]
    fn soft_grades_and_velar_long_stems() {
        use Case::*;
        use Gender::*;
        use Number::*;
        let pos = Degree::Positive;
        // adj:soft-short-palatal-vowel-series / adj:soft-long-vowel-grade
        assert_eq!(adj("синь", Genitive, Singular, Masculine, pos, OCS), "сина");
        assert_eq!(adj("синь", Genitive, Singular, Masculine, pos, SYN), "синѧ");
        assert_eq!(
            adj("синии", Nominative, Singular, Feminine, pos, OCS),
            "синаꙗ"
        );
        assert_eq!(
            adj("синій", Nominative, Singular, Feminine, pos, SYN),
            "синѧѧ"
        );
        assert_eq!(
            adj("нищій", Genitive, Singular, Masculine, pos, SYN),
            "нищагѡ"
        );
        // a velar before -ій is the hard class; ы -> и after the velar
        assert_eq!(
            adj("благій", Genitive, Singular, Masculine, pos, SYN),
            "благагѡ"
        );
        assert_eq!(
            adj("благій", Instrumental, Singular, Masculine, pos, SYN),
            "благимъ"
        );
        assert_eq!(
            adj("благꙑи", Instrumental, Singular, Masculine, pos, OCS),
            "благꙑимь"
        );
    }

    #[test]
    fn comparison_is_suffixal_and_the_superlative_prefixal() {
        use Case::*;
        use Gender::*;
        use Number::*;
        let cmp = Degree::Comparative;
        assert_eq!(
            adj("новъ", Nominative, Singular, Masculine, cmp, OCS),
            "новѣи"
        );
        assert_eq!(
            adj("новъ", Nominative, Singular, Masculine, cmp, SYN),
            "новѣй"
        );
        assert_eq!(adj("новъ", Nominative, Singular, Neuter, cmp, SYN), "новѣе");
        assert_eq!(
            adj("новъ", Nominative, Singular, Feminine, cmp, SYN),
            "новѣйша"
        );
        assert_eq!(
            adj("новъ", Genitive, Singular, Masculine, cmp, OCS),
            "новѣиша"
        );
        assert_eq!(
            adj("новый", Nominative, Singular, Masculine, cmp, SYN),
            "новѣйшій"
        );
        assert_eq!(
            adj("новꙑи", Nominative, Singular, Masculine, cmp, OCS),
            "новѣишии"
        );
        assert_eq!(
            adj(
                "свѧтый",
                Nominative,
                Singular,
                Masculine,
                Degree::Superlative,
                SYN
            ),
            "пресвѧтый"
        );
    }
}
