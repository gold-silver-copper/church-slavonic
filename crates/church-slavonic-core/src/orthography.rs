//! Realisation: the orthographic projection between the two recensions'
//! canonical spellings, and the printed-form helpers (accent, breathing,
//! titlo). Pure string functions; the `church-slavonic` crate applies
//! [`realise`] on output the way `english` restores case, and the extractor
//! uses [`comparison_key`] to match a source form against a rule prediction
//! regardless of which recension's letters it was typed in.
//!
//! The projection folds are the declared OCS ↔ Synodal correspondence rules
//! of the projection study, in the one deterministic direction each:
//! `ꙑ ~ ы`, `оу ~ ꙋ` (the uk digraph, kept word-initially), `ѫ -> ꙋ`,
//! `ѭ -> ю`, `ѩ -> ѧ`, `ѥ -> е`, non-initial `ꙗ -> ѧ`, `ꙁ -> з`. Ambiguous
//! folds (a medial jer, `ѕ`) are left alone: the jers are kept in both
//! recensions and `ѕ` is a live Synodal letter.

use crate::grammar::Recension;
use unicode_normalization::UnicodeNormalization;

const ACUTE: char = '\u{0301}';
const GRAVE: char = '\u{0300}';
const BREATHING: char = '\u{0486}';
const TITLO: char = '\u{0483}';

/// Rewrite `word` into `recension`'s canonical (unaccented, lowercase)
/// spelling. Letters the target recension does not use are folded to the
/// ones it does; everything else passes through unchanged, so a word already
/// in the target spelling is a fixed point.
pub fn realise(word: &str, recension: &Recension) -> String {
    let base = strip_marks(word).to_lowercase();
    let mut out = String::with_capacity(base.len());
    let chars: Vec<char> = base.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let initial = out.is_empty();
        let next = chars.get(i + 1).copied();
        match recension {
            Recension::Synodal => match c {
                'о' if next == Some('у') => {
                    out.push_str(if initial { "оу" } else { "ꙋ" });
                    i += 1;
                }
                'ꙑ' => out.push('ы'),
                'ѫ' => out.push('ꙋ'),
                'ѭ' => out.push('ю'),
                'ѩ' => out.push('ѧ'),
                'ѥ' => out.push('е'),
                'ꙗ' if !initial => out.push('ѧ'),
                'у' if !initial => out.push('ꙋ'),
                'ꙋ' | 'ѹ' if initial => out.push_str("оу"),
                'ꙁ' => out.push('з'),
                'ꙃ' => out.push('ѕ'),
                _ => out.push(c),
            },
            Recension::OldChurchSlavonic => match c {
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
            },
        }
        i += 1;
    }
    out
}

/// The accent-blind comparison key shared by both recensions: [`realise`]
/// into Synodal spelling, then fold the remaining letter pairs that are
/// mere typography (`ѡ`/`ѻ ~ о`, `є ~ е`, `ї`/`і`/`й ~ и`, `ꙗ ~ ѧ`,
/// `ꙋ ~ у`, `ѕ ~ з`, `ѷ ~ ѵ`). Two forms with equal keys are one form.
pub fn comparison_key(word: &str) -> String {
    realise(word, &Recension::Synodal)
        .chars()
        .map(|c| match c {
            'ѡ' | 'ѻ' | 'ѽ' => 'о',
            'є' => 'е',
            'ї' | 'і' | 'й' => 'и',
            'ꙗ' => 'ѧ',
            'ꙋ' => 'у',
            'ѕ' => 'з',
            'ѷ' => 'ѵ',
            other => other,
        })
        .collect::<String>()
        .replace("оу", "у")
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

/// Place the stress on the `syllable`-th vowel (0-based): the acute
/// (oxia) inside the word, the grave (varia) on a word-final vowel letter
/// (Alypy §5), and the breathing (psili) on a word-initial vowel. A `syllable`
/// past the last vowel leaves the word unaccented except for the breathing.
pub fn accent(word: &str, syllable: usize) -> String {
    let skeleton = strip_marks(word);
    let last = skeleton.chars().count().saturating_sub(1);
    let mut out = String::with_capacity(skeleton.len() + 4);
    let mut seen = 0;
    for (index, c) in skeleton.chars().enumerate() {
        out.push(c);
        if !is_vowel(c) {
            continue;
        }
        if index == 0 {
            out.push(BREATHING);
        }
        if seen == syllable {
            out.push(if index == last { GRAVE } else { ACUTE });
        }
        seen += 1;
    }
    out.nfc().collect()
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
    matches!(c as u32, 0x0300..=0x036f | 0x0483..=0x0489 | 0x2de0..=0x2dff | 0xfe20..=0xfe2f)
}

fn is_vowel(c: char) -> bool {
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
        assert_eq!(realise("оученикъ", &SYN), "оученикъ"); // initial digraph kept
        assert_eq!(realise("рѫка", &SYN), "рꙋка");
        assert_eq!(realise("землѭ", &SYN), "землю");
        assert_eq!(realise("ѩзꙑкъ", &SYN), "ѧзыкъ");
        assert_eq!(realise("моѥ", &SYN), "мое");
        assert_eq!(realise("землꙗ", &SYN), "землѧ");
        assert_eq!(realise("ꙗзꙑкъ", &SYN), "ꙗзыкъ"); // initial ꙗ kept
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
    fn comparison_key_is_one_space_for_both_recensions() {
        assert_eq!(comparison_key("рабоу"), comparison_key("рабꙋ́"));
        assert_eq!(comparison_key("рабѡ́мъ"), comparison_key("рабомъ"));
        assert_eq!(comparison_key("ꙗ҆зы́къ"), comparison_key("ѩзꙑкъ"));
        assert_eq!(comparison_key("ѹчитель"), "учитель");
        assert_eq!(comparison_key("ᲂучи́тель"), "учитель");
        assert_eq!(comparison_key("і҆ере́й"), "иереи");
        assert_eq!(comparison_key("ѕѣлѡ"), "зѣло");
    }

    #[test]
    fn accent_breathing_and_titlo_helpers() {
        assert_eq!(accent("рꙋка", 1), "рꙋка\u{300}");
        assert_eq!(accent("рꙋка", 0), "рꙋ\u{301}ка");
        assert_eq!(accent("отецъ", 1), "о\u{486}те\u{301}цъ");
        assert_eq!(accent("азъ", 0), "а\u{486}\u{301}зъ");
        assert_eq!(accent("ра́бъ", 5), "рабъ");
        assert_eq!(strip_marks("ѻ҆те́цъ"), "ѻтецъ");
        assert_eq!(strip_marks("ᲂу҆чени́къ"), "оученикъ");
        assert_eq!(strip_marks("бж҃їй кра́й"), "бжїй край");
        assert_eq!(titlo("бгъ"), "бг\u{483}ъ");
        assert_eq!(titlo("б"), "б");
    }
}
