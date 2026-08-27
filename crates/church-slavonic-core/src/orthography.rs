//! Realisation: the orthographic projection between the two recensions'
//! canonical spellings, and the Synodal typography (accent, breathing,
//! titlo). Pure string functions; the `church-slavonic` crate applies
//! [`realise`] on input and output the way `english` restores case, and the
//! extractor uses [`comparison_key`] to match a source form against a rule
//! prediction regardless of which recension's letters it was typed in.
//!
//! The projection folds are the declared OCS ↔ Synodal correspondence rules
//! of the projection study, in the one deterministic direction each:
//! `ꙑ ~ ы`, `оу ~ ꙋ` (the uk digraph, kept word-initially), `ѫ -> ꙋ`,
//! `ѭ -> ю`, `ѩ -> ѧ`, `ѥ -> е`, non-initial `ꙗ -> ѧ`, `ꙁ -> з`. Ambiguous
//! folds (a medial jer, `ѕ`) are left alone: the jers are kept in both
//! recensions and `ѕ` is a live Synodal letter.
//!
//! # The canonical Synodal typography
//!
//! An Old Church Slavonic word is canonical when unaccented: the source
//! dump prints no accents. A Synodal word is canonical WITH its accent — the
//! accent is a lexical fact of the print (`ра́бъ` : `рабы̀`), the rule engine
//! reads it off the citation form, and the tables hold the forms as printed
//! — so [`realise`] into Synodal keeps the combining marks and normalises
//! them to the print's conventions (Alypy §5): the modern `я` and `у` are
//! the print's `ѧ`/`ꙗ` and `ꙋ`/`оу`, a word-initial `о`/`е` is the wide
//! `ѻ`/`є`, the decimal i is the plain `і` (the corpus edition never dots
//! it; the grammar's `ї` before a vowel is the same letter), every
//! word-initial vowel carries the psili, the stress is the oxia inside the
//! word and the varia on a word-final vowel, the kamora (either encoding)
//! stays where the print put it.

use crate::grammar::Recension;
use unicode_normalization::UnicodeNormalization;

pub(crate) const ACUTE: char = '\u{0301}';
pub(crate) const GRAVE: char = '\u{0300}';
pub(crate) const KAMORA: char = '\u{0311}';
const CIRCUMFLEX: char = '\u{0302}';
const PSILI: char = '\u{0486}';
const DASIA: char = '\u{0485}';
const TITLO: char = '\u{0483}';

/// Rewrite `word` into `recension`'s canonical spelling (lowercase; OCS
/// unaccented, Synodal in the print's typography — see the module docs).
/// Letters the target recension does not use are folded to the ones it
/// does; everything else passes through unchanged, so a word already in the
/// target spelling is a fixed point.
pub fn realise(word: &str, recension: &Recension) -> String {
    match recension {
        Recension::OldChurchSlavonic => realise_ocs(&strip_marks(word).to_lowercase()),
        Recension::Synodal => realise_synodal(word),
    }
}

fn realise_ocs(base: &str) -> String {
    let mut out = String::with_capacity(base.len());
    let chars: Vec<char> = base.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            'о' if next == Some('у') => {
                out.push_str("оу");
                i += 1;
            }
            'ы' => out.push('ꙑ'),
            'ꙋ' | 'у' | 'ѹ' => out.push_str("оу"),
            'є' => out.push('е'),
            'ѻ' | 'ѡ' => out.push('о'),
            'ѿ' => out.push_str("от"),
            'ї' | 'і' | 'й' => out.push('и'),
            'ꙁ' => out.push('з'),
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

/// A letter with the combining marks printed on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Unit {
    pub base: char,
    pub marks: Vec<char>,
}

impl Unit {
    fn bare(base: char) -> Unit {
        Unit {
            base,
            marks: Vec::new(),
        }
    }

    pub fn is_vowel(&self) -> bool {
        is_vowel(self.base)
    }

    pub fn has_stress(&self) -> bool {
        self.marks
            .iter()
            .any(|m| matches!(*m, ACUTE | GRAVE | KAMORA | CIRCUMFLEX))
    }

    fn has_breathing(&self) -> bool {
        self.marks.iter().any(|m| matches!(*m, PSILI | DASIA))
    }

    /// The marks that are not a stress: breathing, titlo, and the rest.
    pub fn marks_but_stress(&self) -> Vec<char> {
        self.marks
            .iter()
            .copied()
            .filter(|m| !matches!(*m, ACUTE | GRAVE | KAMORA | CIRCUMFLEX))
            .collect()
    }
}

/// Split a word into letters with their marks (lowercase, NFC letters; the
/// precomposed `ѐ`/`ѝ` are a letter plus the varia; `ᲂ` is `о`).
pub(crate) fn units(word: &str) -> Vec<Unit> {
    let mut out: Vec<Unit> = Vec::new();
    for c in word.nfc().collect::<String>().to_lowercase().chars() {
        match c {
            '\u{1c82}' => out.push(Unit::bare('о')),
            'ѐ' => out.push(Unit {
                base: 'е',
                marks: vec![GRAVE],
            }),
            'ѝ' => out.push(Unit {
                base: 'и',
                marks: vec![GRAVE],
            }),
            m if is_mark(m) => {
                if let Some(unit) = out.last_mut() {
                    unit.marks.push(m);
                }
            }
            _ => out.push(Unit::bare(c)),
        }
    }
    out
}

/// Join units back into an NFC string; the marks of a letter are written in
/// the print's order — breathing, stress, titlo, the rest.
pub(crate) fn join(units: &[Unit]) -> String {
    let mut out = String::with_capacity(units.len() * 3);
    for unit in units {
        out.push(unit.base);
        let mut marks = unit.marks.clone();
        marks.sort_by_key(|m| match *m {
            PSILI | DASIA => 0,
            ACUTE | GRAVE | KAMORA => 1,
            TITLO => 2,
            _ => 3,
        });
        marks.dedup();
        out.extend(marks);
    }
    out.nfc().collect()
}

/// Canonical Synodal typography (see the module docs).
fn realise_synodal(word: &str) -> String {
    let source = units(word);
    let mut out: Vec<Unit> = Vec::with_capacity(source.len() + 1);
    let mut i = 0;
    while i < source.len() {
        let mut unit = source[i].clone();
        let initial = out.is_empty();
        let next = source.get(i + 1);
        match unit.base {
            'о' if next.is_some_and(|n| n.base == 'у') => {
                // The uk digraph: kept word-initially, the monograph inside.
                let second = source[i + 1].clone();
                if initial {
                    out.push(Unit::bare('о'));
                    out.push(second);
                } else {
                    let mut merged = second;
                    merged.base = 'ꙋ';
                    merged.marks.extend(unit.marks);
                    out.push(merged);
                }
                i += 2;
                continue;
            }
            'у' | 'ꙋ' | 'ѹ' if initial => {
                out.push(Unit::bare('о'));
                unit.base = 'у';
            }
            'у' | 'ѹ' | 'ѫ' => unit.base = 'ꙋ',
            'ꙑ' => unit.base = 'ы',
            'ѭ' => unit.base = 'ю',
            'ѥ' => unit.base = 'е',
            'ꙗ' | 'я' | 'ѩ' => unit.base = if initial { 'ꙗ' } else { 'ѧ' },
            'ꙁ' => unit.base = 'з',
            'ꙃ' => unit.base = 'ѕ',
            'ї' => unit.base = 'і',
            'о' if initial => unit.base = 'ѻ',
            'е' if initial => unit.base = 'є',
            _ => {}
        }
        out.push(unit);
        i += 1;
    }
    normalise_marks(&mut out);
    join(&out)
}

/// The print's mark conventions on already-folded letters: the psili on the
/// word-initial vowel (the `у` of an initial `оу`), one stress per vowel as
/// oxia/varia/kamora by position.
fn normalise_marks(units: &mut [Unit]) {
    let last = units.len().saturating_sub(1);
    for (i, unit) in units.iter_mut().enumerate() {
        let final_vowel = i == last && unit.is_vowel();
        let kamora = unit.marks.iter().any(|m| matches!(*m, KAMORA | CIRCUMFLEX));
        let stressed = unit.has_stress();
        unit.marks
            .retain(|m| !matches!(*m, ACUTE | GRAVE | KAMORA | CIRCUMFLEX));
        if kamora {
            unit.marks.push(KAMORA);
        } else if stressed {
            unit.marks.push(if final_vowel { GRAVE } else { ACUTE });
        }
    }
    let breathing_at = match units {
        [o, u, ..] if o.base == 'о' && u.base == 'у' => Some(1),
        [first, ..] if first.is_vowel() => Some(0),
        _ => None,
    };
    if let Some(i) = breathing_at
        && !units[i].has_breathing()
    {
        units[i].marks.push(PSILI);
    }
}

/// The accent-blind comparison key shared by both recensions: [`realise`]
/// into Synodal spelling, drop the marks, then fold the remaining letter
/// pairs that are mere typography (`ѡ`/`ѻ ~ о`, `є ~ е`, `ї`/`і`/`й`/`ꙇ ~
/// и`, `ꙗ ~ ѧ`, `ꙙ ~ ѧ`, `ꙋ ~ у`, `ѕ ~ з`, `ѷ ~ ѵ`, the manuscripts' `ъі` ~
/// `ꙑ`). Two forms with equal keys are one form.
pub fn comparison_key(word: &str) -> String {
    strip_marks(&realise(word, &Recension::Synodal))
        .chars()
        .map(|c| match c {
            'ѡ' | 'ѻ' | 'ѽ' => 'о',
            'є' => 'е',
            'ї' | 'і' | 'й' | 'ꙇ' => 'и',
            'ꙗ' | 'ꙙ' => 'ѧ',
            'ꙋ' => 'у',
            'ѕ' => 'з',
            'ѷ' => 'ѵ',
            other => other,
        })
        .collect::<String>()
        .replace("оу", "у")
        .replace("ъи", "ы")
        .replace('ѿ', "от")
}

/// Remove every combining mark (accents, breathing, titlo, kamora) and the
/// digraph half `ᲂ`, returning the NFC skeleton. The precomposed letters
/// `й`, `ї` and `ѷ` are letters of the alphabet, not accented vowels, and are
/// kept.
pub fn strip_marks(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for c in word.nfc() {
        if matches!(c, 'й' | 'Й' | 'ї' | 'Ї' | 'ѷ' | 'Ѷ') {
            out.push(c);
            continue;
        }
        out.extend(
            c.nfd()
                .filter(|m| !is_mark(*m))
                .map(|m| if m == '\u{1c82}' { 'о' } else { m }),
        );
    }
    out.nfc().collect()
}

/// Does the word carry a stress mark (oxia, varia or kamora)?
pub fn is_accented(word: &str) -> bool {
    units(word).iter().any(Unit::has_stress)
}

/// Place the stress on the `vowel`-th vowel (0-based), replacing any stress
/// the word carries and keeping its other marks: the oxia inside the word,
/// the varia on a word-final vowel letter (Alypy §5), or the kamora when
/// `kamora` is set — the print's mark on a plural or dual form that would
/// otherwise be spelled like a singular (`рабы̑`, `сы̑ны`). A `vowel` past the
/// last vowel places nothing.
pub fn stress(word: &str, vowel: usize, kamora: bool) -> String {
    let mut units = units(word);
    let last = units.len().saturating_sub(1);
    let mut seen = 0;
    for (index, unit) in units.iter_mut().enumerate() {
        unit.marks
            .retain(|m| !matches!(*m, ACUTE | GRAVE | KAMORA | CIRCUMFLEX));
        if !unit.is_vowel() {
            continue;
        }
        if seen == vowel {
            unit.marks.push(if kamora {
                KAMORA
            } else if index == last {
                GRAVE
            } else {
                ACUTE
            });
        }
        seen += 1;
    }
    join(&units)
}

/// Mark an abbreviation with the titlo over its second letter (`бгъ` ->
/// `бг҃ъ`, `дхъ` -> `дх҃ъ`); a one-letter word is returned unchanged.
pub fn titlo(abbreviation: &str) -> String {
    let mut out = String::with_capacity(abbreviation.len() + 2);
    for (index, c) in abbreviation.chars().enumerate() {
        out.push(c);
        if index == 1 {
            out.push(TITLO);
        }
    }
    out
}

fn is_mark(c: char) -> bool {
    matches!(
        c as u32,
        0x0300..=0x036f
            | 0x0483..=0x0489
            | 0x2de0..=0x2dff
            | 0xa66f..=0xa672
            | 0xa674..=0xa67d
            | 0xfe20..=0xfe2f
    )
}

pub(crate) fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'а' | 'е'
            | 'є'
            | 'и'
            | 'і'
            | 'ї'
            | 'о'
            | 'ѻ'
            | 'ѡ'
            | 'у'
            | 'ꙋ'
            | 'ѹ'
            | 'ы'
            | 'ꙑ'
            | 'ѣ'
            | 'ю'
            | 'ꙗ'
            | 'ѧ'
            | 'ѩ'
            | 'ѫ'
            | 'ѭ'
            | 'ѥ'
            | 'ѵ'
            | 'ѷ'
            | 'я'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::Synodal;

    #[test]
    fn projection_folds_the_declared_letter_pairs() {
        assert_eq!(realise("рꙑба", &SYN), "рыба");
        assert_eq!(realise("рабоу", &SYN), "рабꙋ");
        assert_eq!(realise("оученикъ", &SYN), "оу҆ченикъ"); // initial digraph kept
        assert_eq!(realise("рѫка", &SYN), "рꙋка");
        assert_eq!(realise("землѭ", &SYN), "землю");
        assert_eq!(realise("ѩзꙑкъ", &SYN), "ꙗ҆зыкъ");
        assert_eq!(realise("моѥ", &SYN), "мое");
        assert_eq!(realise("землꙗ", &SYN), "землѧ");
        assert_eq!(realise("ꙗзꙑкъ", &SYN), "ꙗ҆зыкъ"); // initial ꙗ kept
        assert_eq!(realise("градъ", &SYN), "градъ"); // jers kept
        assert_eq!(realise("дьнь", &SYN), "дьнь");
        assert_eq!(realise("ꙁима", &SYN), "зима");
        assert_eq!(realise("ѕвѣзда", &SYN), "ѕвѣзда"); // ambiguous: kept
        assert_eq!(realise("ры́ба", &OCS), "рꙑба");
        assert_eq!(realise("ѹ҆чени́къ", &OCS), "оученикъ");
        assert_eq!(realise("ᲂу҆чени́къ", &OCS), "оученикъ");
        assert_eq!(realise("є҆гѡ̀", &OCS), "его");
        assert_eq!(realise("ѻ҆те́цъ", &OCS), "отецъ");
        assert_eq!(realise("і҆ере́й", &OCS), "иереи");
        assert_eq!(realise("Рꙋка̀", &OCS), "роука");
        // fixed points
        assert_eq!(realise("рꙋка", &SYN), "рꙋка");
        assert_eq!(realise("рѫка", &OCS), "рѫка");
    }

    #[test]
    fn synodal_realisation_is_the_print_typography() {
        // Polyakov's modern letters and oxia-only accents become the print's.
        assert_eq!(realise("раба́", &SYN), "раба̀");
        assert_eq!(realise("рабу́", &SYN), "рабꙋ̀");
        assert_eq!(realise("творя́тъ", &SYN), "творѧ́тъ");
        assert_eq!(realise("я́", &SYN), "ꙗ҆̀");
        assert_eq!(realise("учени́къ", &SYN), "оу҆чени́къ");
        assert_eq!(realise("оте́цъ", &SYN), "ѻ҆те́цъ");
        assert_eq!(realise("егѡ́", &SYN), "є҆гѡ̀");
        assert_eq!(realise("а́зъ", &SYN), "а҆́зъ");
        assert_eq!(realise("рабы̂", &SYN), "рабы̑");
        // The grammar's print is a fixed point.
        for printed in ["ѻ҆́трокъ", "є҆гѡ̀", "ᲂу҆чени́къ", "рабы̑", "бг҃ъ", "сн҃а"]
        {
            let once = realise(printed, &SYN);
            assert_eq!(realise(&once, &SYN), once, "{printed}");
        }
        assert_eq!(realise("ᲂу҆чени́къ", &SYN), "оу҆чени́къ");
        assert_eq!(realise("бг҃ъ", &SYN), "бг҃ъ");
        assert_eq!(realise("Ра́бъ", &SYN), "ра́бъ");
        assert_eq!(realise("тѣ̀мже", &SYN), "тѣ́мже");
        assert_eq!(realise("менѐ", &SYN), "менѐ");
        assert_eq!(realise("бж҃їй", &SYN), "бж҃ій");
    }

    #[test]
    fn comparison_key_is_one_space_for_both_recensions() {
        assert_eq!(comparison_key("рабоу"), comparison_key("рабꙋ́"));
        assert_eq!(comparison_key("рабѡ́мъ"), comparison_key("рабомъ"));
        assert_eq!(comparison_key("ꙗ҆зы́къ"), comparison_key("ѩзꙑкъ"));
        assert_eq!(comparison_key("ѹчитель"), "учитель");
        assert_eq!(comparison_key("ᲂучи́тель"), "учитель");
        assert_eq!(comparison_key("і҆ере́й"), "иереи");
        assert_eq!(comparison_key("ѕѣлѡ"), "зѣло");
        assert_eq!(comparison_key("отецъ"), comparison_key("ѻ҆те́цъ"));
        assert_eq!(comparison_key("творя́тъ"), "творѧтъ");
        // manuscript spellings
        assert_eq!(comparison_key("ѩзъікъ"), comparison_key("ѩзꙑкъ"));
        assert_eq!(comparison_key("доушꙙ"), comparison_key("доушѧ"));
        assert_eq!(comparison_key("ст꙯ааго"), "стааго");
        assert_eq!(comparison_key("ꙇсоусъ"), "исусъ");
    }

    #[test]
    fn stress_breathing_and_titlo_helpers() {
        assert_eq!(stress("рꙋка", 1, false), "рꙋка\u{300}");
        assert_eq!(stress("рꙋка", 0, false), "рꙋ\u{301}ка");
        assert_eq!(stress("ра́бъ", 5, false), "рабъ");
        assert_eq!(stress("рабы", 1, true), "рабы\u{311}");
        assert_eq!(stress("сн҃ы", 0, true), "сн\u{483}ы\u{311}");
        assert_eq!(stress("є҆гѡ́", 1, false), "є҆гѡ̀");
        assert!(is_accented("ра́бъ"));
        assert!(!is_accented("бг҃ъ"));
        assert_eq!(strip_marks("ѻ҆те́цъ"), "ѻтецъ");
        assert_eq!(strip_marks("ᲂу҆чени́къ"), "оученикъ");
        assert_eq!(strip_marks("бж҃їй кра́й"), "бжїй край");
        assert_eq!(titlo("бгъ"), "бг\u{483}ъ");
        assert_eq!(titlo("б"), "б");
    }
}
