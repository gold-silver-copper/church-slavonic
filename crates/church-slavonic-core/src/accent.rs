//! The Synodal accent rule: where the stress of an inflected form falls,
//! given the accented citation form.
//!
//! The Synodal print marks the stress on every word, and the rule engine
//! predicts it the way it predicts the ending — from the lemma. The lemma is
//! the accented citation form (`ра́бъ`, `рꙋка̀`, `свѧты́й`, `твори́ти`), and
//! its accent decides the paradigm's pattern:
//!
//! - a stem-stressed lemma keeps the stress on the same stem vowel in every
//!   form (`аарѡ́нъ` : `аарѡ́на`, `до́брый` : `до́брагѡ`, `глаго́лати` :
//!   `глаго́лю`);
//! - an ending-stressed lemma (the stress on the vowel of its own ending —
//!   `рꙋка̀`, `свѧты́й`, `твори́ти`, `нестѝ`) stresses the first vowel of
//!   every ending (`рꙋкѝ`, `свѧта́гѡ`, `творю̀`, `несе́ши`), and falls back to
//!   the last stem vowel when the ending has none (`рꙋ́къ`).
//!
//! The mark itself is the print's: oxia inside the word, varia on a final
//! vowel, the kamora where a cell asks for it ([`KAMORA_CELL`]). A lemma
//! without a stress mark (an abbreviation under a titlo, an unaccented
//! query) is inflected without one; the stem's other marks (the titlo) are
//! carried over. What the rule gets wrong — the mobile paradigms (`рꙋ́кꙋ`,
//! `сы́нъ` : `сынѡ́въ`), the fleeting-vowel stems — is what the tables hold.

use crate::grammar::Recension;
use crate::orthography::{Unit, is_accented, join, stress, strip_marks, units};

/// A cell whose ending begins with this marker is one the print tells apart
/// from a look-alike singular (Alypy §6): the last `о`/`е` at or after the
/// stress becomes the wide `ѡ`/`є` (`рабѡ́въ`, `а҆́ггєлъ`, `ѻ҆тцє́мъ`,
/// `бє́здны`), a wide letter already there is mark enough (`а҆вессалѡ́мли`),
/// and a word with neither takes the kamora instead of the oxia/varia
/// (`рабы̑`, `рꙋ̑ки`, `сы̑ны`, `безпꙋ̑тіѧ`).
pub(crate) const KAMORA_CELL: char = '^';

/// The print's word-final varia (Alypy §5): an acute that lands on the last
/// vowel letter of a word becomes grave. The skeleton-level override paths
/// concatenate an accented stem with plain endings, so a stem-final acute
/// must be re-graded when nothing follows it.
pub(crate) fn final_varia(word: &str) -> String {
    let mut units = crate::orthography::units(word);
    let last = units.len().saturating_sub(1);
    for (i, unit) in units.iter_mut().enumerate() {
        for mark in &mut unit.marks {
            if *mark == crate::orthography::ACUTE && i == last {
                *mark = crate::orthography::GRAVE;
            } else if *mark == crate::orthography::GRAVE && i != last {
                *mark = crate::orthography::ACUTE;
            }
        }
    }
    crate::orthography::join(&units)
}

/// Run `rule` on the unaccented skeleton of `word` and put the accent back
/// by the pattern above. Outside the Synodal recension, or for an
/// unaccented lemma, the rule's answer is returned as is (minus the kamora
/// marker).
pub(crate) fn with_accent(
    word: &str,
    recension: &Recension,
    rule: impl FnOnce(&str) -> String,
) -> String {
    if *recension != Recension::Synodal {
        return rule(word).replace(KAMORA_CELL, "");
    }
    let lemma = units(word);
    let skeleton = strip_marks(word);
    let answer = rule(&skeleton);
    let distinguish = answer.contains(KAMORA_CELL);
    let answer = answer.replace(KAMORA_CELL, "");
    let unmarked = !is_accented(word) && lemma.iter().all(|u| u.marks.is_empty());
    // Carry the stem's marks over the letters the answer shares with the
    // lemma (past any prefix the rule added: the superlative's `пре-`),
    // then place the stress.
    let mut out = units(&answer);
    let skeleton_units = units(&skeleton);
    let offset = (0..out.len())
        .find(|&at| {
            out[at..]
                .iter()
                .zip(skeleton_units.iter())
                .take(2)
                .filter(|(o, l)| o.base == l.base)
                .count()
                == skeleton_units.len().min(2)
        })
        .unwrap_or(0);
    let shared = lemma
        .iter()
        .zip(out[offset..].iter())
        .take_while(|(l, o)| l.base == o.base)
        .count();
    for (i, unit) in out.iter_mut().skip(offset).enumerate().take(shared) {
        unit.marks = lemma[i].marks_but_stress();
    }
    let accent = lemma.iter().position(Unit::has_stress);
    let vowels_before =
        |units: &[Unit], n: usize| units[..n].iter().filter(|u| u.is_vowel()).count();
    let total = out.iter().filter(|u| u.is_vowel()).count();
    let prefix_vowels = vowels_before(&out, offset);
    let target = accent.map(|accent| {
        let k = vowels_before(&lemma, accent) + prefix_vowels;
        let stem_vowels = vowels_before(&out, offset + shared);
        if k < stem_vowels || stem_vowels >= total {
            k.min(total.saturating_sub(1))
        } else {
            stem_vowels
        }
    });
    // The plural mark: the last `о`/`е` from the stressed vowel on becomes
    // wide; a wide letter already there needs nothing; a word with neither
    // takes the kamora at the stress.
    let mut kamora = false;
    if distinguish && (target.is_some() || unmarked) {
        let from = target.unwrap_or(0);
        let mut seen = total;
        let mut widened = false;
        for unit in out.iter_mut().rev() {
            if !unit.is_vowel() {
                continue;
            }
            seen -= 1;
            if seen < from {
                break;
            }
            match unit.base {
                'о' => {
                    unit.base = 'ѡ';
                    widened = true;
                    break;
                }
                'е' => {
                    unit.base = 'є';
                    widened = true;
                    break;
                }
                'ѡ' | 'є' => {
                    widened = true;
                    break;
                }
                _ => {}
            }
        }
        kamora = !widened;
    }
    match target {
        Some(target) if !unmarked => stress(&join(&out), target, kamora),
        Some(target) => stress(&join(&out), target, kamora)
            .chars()
            .filter(|c| *c != crate::orthography::ACUTE && *c != crate::orthography::GRAVE)
            .collect(),
        // An unaccented lemma has no stress to hang the kamora on.
        None => join(&out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYN: Recension = Recension::Synodal;

    fn apply(lemma: &str, answer: &str) -> String {
        with_accent(lemma, &SYN, |skeleton| {
            assert!(!is_accented(skeleton));
            answer.to_string()
        })
    }

    #[test]
    fn stem_stress_stays_and_ending_stress_moves_to_the_ending() {
        assert_eq!(apply("аарѡ́нъ", "аарѡна"), "аарѡ́на");
        assert_eq!(apply("до́брый", "добрагѡ"), "до́брагѡ");
        assert_eq!(apply("глаго́лати", "глаголю"), "глаго́лю");
        assert_eq!(apply("рꙋка̀", "рꙋки"), "рꙋкѝ");
        assert_eq!(apply("рꙋка̀", "рꙋцѣ"), "рꙋцѣ̀");
        assert_eq!(apply("рꙋка̀", "рꙋкою"), "рꙋко́ю");
        assert_eq!(apply("рꙋка̀", "рꙋкахъ"), "рꙋка́хъ");
        assert_eq!(apply("рꙋка̀", "рꙋкъ"), "рꙋ́къ");
        assert_eq!(apply("свѧты́й", "свѧтагѡ"), "свѧта́гѡ");
        assert_eq!(apply("свѧты́й", "свѧтый"), "свѧты́й");
        assert_eq!(apply("твори́ти", "творю"), "творю̀");
        assert_eq!(apply("твори́ти", "твориши"), "твори́ши");
        assert_eq!(apply("твори́ти", "творѧше"), "творѧ́ше");
        assert_eq!(apply("нестѝ", "несꙋ"), "несꙋ̀");
        assert_eq!(apply("нестѝ", "несъ"), "не́съ");
        assert_eq!(apply("бы́ти", "бꙋди"), "бꙋ́ди");
        assert_eq!(apply("мꙋ́дръ", "премꙋдръ"), "премꙋ́дръ");
        assert_eq!(apply("свѧты́й", "пресвѧтый"), "пресвѧты́й");
    }

    #[test]
    fn marks_other_than_the_stress_are_carried_and_the_kamora_is_a_cell_property() {
        assert_eq!(apply("бг҃ъ", "бга"), "бг҃а");
        assert_eq!(apply("рабъ", "раба"), "раба");
        assert_eq!(apply("ра́бъ", "^рабы"), "ра\u{311}бы");
        assert_eq!(apply("ра́бъ", "^рабовъ"), "ра́бѡвъ");
        assert_eq!(apply("а҆́ггелъ", "^аггелъ"), "а҆́ггєлъ");
        assert_eq!(apply("бе́здна", "^бездны"), "бє́здны");
        assert_eq!(apply("безпꙋ́тіе", "^безпꙋтіѧ"), "безпꙋ̑тіѧ");
        assert_eq!(apply("а҆вессалѡ́мль", "^авессалѡмли"), "а҆вессалѡ́мли");
        assert_eq!(apply("бг҃ъ", "^бгы"), "бг҃ы");
        assert_eq!(apply("рꙋка̀", "^рꙋками"), "рꙋка\u{311}ми");
        assert_eq!(
            with_accent("ра́бъ", &Recension::OldChurchSlavonic, |w| format!(
                "{w}!"
            )),
            "ра́бъ!"
        );
    }
}
