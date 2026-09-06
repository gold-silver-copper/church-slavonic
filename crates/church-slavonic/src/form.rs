//! A form: letters composed with a stress position. The only string a
//! consumer sees is [`Form::print`], the recension's typography applied
//! to the two layers — and the analyzer reads that string back through
//! [`Form::key`].
//!
//! `letters` are the recension's canonical alphabet with no combining
//! marks: for Synodal `а б в г д е ж ѕ з и і й к л м н о п р с т ꙋ ф х ц
//! ч ш щ ъ ы ь ѣ ю ѧ ꙗ ѯ ѱ ѳ ѵ`, plus a LEXICAL wide `ѡ`/`є` where the
//! word has one (`ѡ҆́блакъ`, `і҆ѡа́ннъ`); the positional wide letter of a
//! number-marked cell is never in the letters. `stress` is the 0-based
//! index of the stressed vowel (`None` for Old Church Slavonic, an
//! abbreviation under a titlo, or an unaccented query). `number_mark`
//! says the cell is one the print tells apart from a look-alike singular
//! (Alypy §6): the last narrow `о`/`е` at or after the stress widens
//! (`рабѡ́въ`, `а҆́ггєлъ`), a form stressed on its final vowel widens the
//! last narrow `о`/`е` anywhere (`вѡнѝ`), and a word with no candidate
//! takes the kamora at the stress instead (`рабы̑`, `сы̑ны`).
//!
//! `print` applies, in order: the number mark, the stress mark (oxia
//! inside the word, varia on a final vowel — [`crate::orthography::stress`]),
//! the print conventions of [`crate::orthography::realise`] (the psili on an
//! initial vowel, the initial `ѻ`/`є`, the monosyllable's varia), and the
//! `ї` rule: an UNSTRESSED non-initial `і` before a vowel or `й` is `ї`
//! (`лю́дїе`, `сїѧ̑`; the stressed `люді́й`, `сі́и` keep `і`). The rule reads
//! the stress, which is why stress is placed before typography. A `ї`
//! before a consonant is lexical (`кївѡ́тъ`) and is whatever the letters
//! say.

use crate::grammar::Recension;
use crate::orthography::{
    self, Unit, comparison_key, is_vowel_letter, join, realise, stressed_vowel_index, strip_marks,
    units,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Form {
    pub letters: String,
    pub stress: Option<u8>,
    pub number_mark: bool,
    /// Vowels at the end of `letters` the number mark must skip: an
    /// enclitic written solid (`-же`, `-сѧ`) is not where the plural's
    /// wide letter goes (є҆гѡ́же, тѣ̑мже). Zero for a plain word.
    pub mark_skip: u8,
    /// The stress is written as a varia where the rule would write an
    /// oxia: the print's convention for a few homographs (и҆̀хъ the
    /// accusative against и҆́хъ the genitive, ꙗ҆̀же). Set by
    /// [`Form::from_print`] from an attested print; a class-built form
    /// leaves it false and takes the rule.
    pub varia: bool,
    /// The number mark is written as a kamora although a wide letter was
    /// available (своѧ̑ beside свѡѧ̀): the print's choice, kept from an
    /// attested form; a class-built form leaves it false and takes the rule.
    pub kamora: bool,
}

impl Form {
    pub fn new(letters: impl Into<String>, stress: Option<u8>, number_mark: bool) -> Form {
        Form { letters: letters.into(), stress, number_mark, mark_skip: 0, varia: false, kamora: false }
    }

    /// An unaccented form (Old Church Slavonic, or a titlo lemma).
    pub fn unaccented(letters: impl Into<String>) -> Form {
        Form { letters: letters.into(), stress: None, number_mark: false, mark_skip: 0, varia: false, kamora: false }
    }

    /// Read a printed form back into its layers: the letters with every
    /// mark stripped (wide letters kept as printed — the importer decides
    /// against the class's prediction which are lexical), the stressed
    /// vowel's index, and `number_mark` when the print wrote a kamora.
    pub fn from_print(printed: &str) -> Form {
        // a print that still spells the initial uk «оу» (a source's
        // typography, the 2.x lexicon files) is the same word as one with
        // ѹ: fold before counting vowels, so the stress index is the
        // letters' (3.1)
        let folded = fold_initial_uk(printed);
        let printed = folded.as_str();
        let stress = stressed_vowel_index(printed).and_then(|i| u8::try_from(i).ok());
        let kamora = units(printed)
            .iter()
            .any(|u| u.marks.iter().any(|m| matches!(*m, '\u{0311}' | '\u{0302}')));
        // The positional ї is the rule's, not the letters': un-apply it.
        let mut letters = units(&strip_marks(printed));
        let stressed: Vec<bool> = units(printed).iter().map(Unit::has_stress).collect();
        for i in 1..letters.len() {
            if letters[i].base == 'ї'
                && !stressed.get(i).copied().unwrap_or(false)
                && letters.get(i + 1).is_some_and(|n| is_vowel_letter(n.base) || n.base == 'й')
            {
                letters[i].base = 'і';
            }
        }
        // the initial ѻ/є are the print's (realise writes them): fold them
        // so the letters layer, and the ids built from it, are bare
        if let Some(first) = letters.first_mut() {
            match first.base {
                'ѻ' => first.base = 'о',
                'є' => first.base = 'е',
                _ => {}
            }
        }
        // a varia on a stressed vowel that is not the word's last letter
        // is the print's own choice (и҆̀хъ, ꙗ҆̀же), not the positional rule
        let varia = {
            let us = units(printed);
            let last = us.len().saturating_sub(1);
            us.iter().enumerate().any(|(i, u)| u.is_vowel() && u.has_stress() && u.marks.contains(&'\u{300}') && i != last)
        };
        Form { letters: join(&letters), stress, number_mark: kamora, mark_skip: 0, varia, kamora }
    }

    /// The accent-blind comparison key (typographic letter pairs folded)
    /// — the analyzer's index key and the one equality of the library.
    pub fn key(&self) -> String {
        comparison_key(&self.letters)
    }

    /// The form with an enclitic written solid after it: the accentual
    /// unit (the phonological word) the print accents as one — землѧ̀ +
    /// же = землѧ́же, the host's final varia an oxia because the unit's last
    /// vowel is the enclitic's; the number mark skips the enclitic's
    /// vowels; the Synodal print drops the host's jer before the enclitic
    /// (ихъ + же = и҆́хже, тѣ́мже), OCS keeps it (имъже). The `encl=` lexemes
    /// (иже, кождо, the reflexive verbs) are this rule applied by the
    /// class at the letters stage; this is the same rule for any host.
    pub fn with_enclitic(&self, enclitic: &str, recension: Recension) -> Form {
        let mut letters = self.letters.clone();
        if recension == Recension::Synodal && letters.ends_with('ъ') {
            letters.pop();
        }
        let enclitic_letters = strip_marks(enclitic);
        let tail = enclitic_letters.chars().filter(|c| is_vowel_letter(*c)).count();
        letters.push_str(&enclitic_letters);
        Form {
            letters,
            stress: self.stress,
            number_mark: self.number_mark,
            mark_skip: self.mark_skip.saturating_add(tail.min(255) as u8),
            varia: self.varia,
            kamora: self.kamora,
        }
    }

    /// The print of the form with its enclitics written solid after it
    /// (the phonological word): `print` of [`Form::with_enclitic`]
    /// applied in order.
    pub fn print_unit(&self, recension: Recension, enclitics: &[&str]) -> String {
        enclitics.iter().fold(self.clone(), |f, e| f.with_enclitic(e, recension)).print(recension)
    }

    /// The print of a host whose enclitic is written apart (Землѧ́ же): the
    /// unit continues, so the host's final stressed vowel takes the oxia
    /// the positional rule would make a varia; everything else as
    /// [`Form::print`].
    pub fn print_hosting(&self, recension: Recension) -> String {
        let printed = self.print(recension);
        if recension != Recension::Synodal || self.varia {
            return printed;
        }
        let mut units = units(&printed);
        let last = units.len().saturating_sub(1);
        if let Some(u) = units.get_mut(last)
            && u.is_vowel()
            && u.marks.contains(&'\u{300}')
        {
            for m in &mut u.marks {
                if *m == '\u{300}' {
                    *m = '\u{301}';
                }
            }
        }
        join(&units)
    }

    /// The printed word in `recension`'s typography.
    pub fn print(&self, recension: Recension) -> String {
        match recension {
            Recension::OldChurchSlavonic => realise(&self.letters, &recension),
            Recension::Synodal => self.print_synodal(),
        }
    }

    fn print_synodal(&self) -> String {
        let mut out = units(&self.letters);
        let total = out.iter().filter(|u| u.is_vowel()).count();
        let target = self.stress.map(usize::from).filter(|t| *t < total);
        let mut kamora = false;
        if self.number_mark && self.kamora {
            kamora = target.is_some();
        } else if self.number_mark {
            // a stress inside the skipped tail (первыйна́десѧть) leaves the
            // first element to be widened as a word of its own
            let skip = usize::from(self.mark_skip);
            let from = match target {
                Some(t) if t + 1 < total && t + skip < total => t,
                _ => 0,
            };
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
                if seen + skip >= total {
                    continue;
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
                    _ => {}
                }
            }
            kamora = !widened && target.is_some();
        }
        let word = join(&out);
        let word = match target {
            Some(t) => {
                let stressed = orthography::stress(&word, t, kamora);
                if self.varia && !kamora { stressed.replace('\u{301}', "\u{300}") } else { stressed }
            }
            None => word,
        };
        let realised = realise(&word, &Recension::Synodal);
        apply_izhitsa_rule(&realised)
    }
}

/// «оу» at the head of a word (with the у's marks) as the one letter ѹ.
fn fold_initial_uk(printed: &str) -> String {
    let mut us = units(printed);
    if us.len() >= 2 && us[0].base == 'о' && us[0].marks.is_empty() && us[1].base == 'у' {
        us[1].base = 'ѹ';
        us.remove(0);
    }
    join(&us)
}

/// The print's `ї`: an unstressed non-initial `і` before a vowel or `й`.
fn apply_izhitsa_rule(word: &str) -> String {
    let mut out: Vec<Unit> = units(word);
    for i in 1..out.len() {
        if out[i].base != 'і' || out[i].has_stress() {
            continue;
        }
        let next = out.get(i + 1);
        if next.is_some_and(|n| is_vowel_letter(n.base) || n.base == 'й') {
            out[i].base = 'ї';
        }
    }
    join(&out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_normalization::UnicodeNormalization;

    const SYN: Recension = Recension::Synodal;

    fn nfc(s: &str) -> String {
        s.nfc().collect()
    }

    #[test]
    fn the_worked_example_rab() {
        // singular
        assert_eq!(Form::new("рабъ", Some(0), false).print(SYN), nfc("ра́бъ"));
        assert_eq!(Form::new("раба", Some(1), false).print(SYN), nfc("раба̀"));
        assert_eq!(Form::new("рабꙋ", Some(1), false).print(SYN), nfc("рабꙋ̀"));
        assert_eq!(Form::new("рабомъ", Some(1), false).print(SYN), nfc("рабо́мъ"));
        assert_eq!(Form::new("рабе", Some(0), false).print(SYN), nfc("ра́бе"));
        // plural: the number mark widens or takes the kamora
        assert_eq!(Form::new("раби", Some(1), false).print(SYN), nfc("рабѝ"));
        assert_eq!(Form::new("рабъ", Some(0), true).print(SYN), nfc("ра̑бъ"));
        assert_eq!(Form::new("рабомъ", Some(1), true).print(SYN), nfc("рабѡ́мъ"));
        assert_eq!(Form::new("рабы", Some(1), true).print(SYN), nfc("рабы̑"));
        assert_eq!(Form::new("рабѣхъ", Some(1), false).print(SYN), nfc("рабѣ́хъ"));
        assert_eq!(Form::new("рабовъ", Some(1), true).print(SYN), nfc("рабѡ́въ"));
    }

    #[test]
    fn widening_conventions() {
        // final-vowel stress widens anywhere
        assert_eq!(Form::new("вони", Some(1), true).print(SYN), nfc("вѡнѝ"));
        // an о before a non-final stress stays; the kamora lands instead
        assert_eq!(Form::new("безпꙋтіѧ", Some(1), true).print(SYN), nfc("безпꙋ̑тїѧ"));
        // a lexical wide letter is not mark enough
        assert_eq!(Form::new("аарѡнимъ", Some(2), true).print(SYN), nfc("а҆арѡ̑нимъ"));
        assert_eq!(Form::new("аггелъ", Some(0), true).print(SYN), nfc("а҆́ггєлъ"));
    }

    #[test]
    fn typography_and_the_izhitsa_rule() {
        assert_eq!(Form::new("отецъ", Some(1), false).print(SYN), nfc("ѻ҆те́цъ"));
        assert_eq!(Form::new("людіе", Some(0), false).print(SYN), nfc("лю́дїе"));
        assert_eq!(Form::new("людій", Some(1), false).print(SYN), nfc("люді́й"));
        assert_eq!(Form::new("сіѧ", Some(1), true).print(SYN), nfc("сїѧ̑"));
        assert_eq!(Form::new("сіи", Some(0), false).print(SYN), nfc("сі́и"));
        assert_eq!(Form::new("пріиде", Some(1), false).print(SYN), nfc("прїи́де"));
        // initial і stays; ї before a consonant is lexical
        assert_eq!(Form::new("іерей", Some(2), false).print(SYN), nfc("і҆ере́й"));
        assert_eq!(Form::new("кївотъ", Some(1), false).print(SYN), nfc("кївѡ́тъ").replace('ѡ', "о"));
        // unaccented: no marks beyond the psili
        assert_eq!(Form::unaccented("отецъ").print(SYN), nfc("ѻ҆тецъ"));
        // OCS drops the stress and maps the alphabet
        assert_eq!(Form::new("рабꙋ", Some(1), false).print(Recension::OldChurchSlavonic), "рабоу");
    }

    #[test]
    fn from_print_inverts_print() {
        for (letters, stress, mark) in [
            ("рабомъ", Some(1), false),
            ("рабы", Some(1), true),
            ("рабѣхъ", Some(1), false),
            ("людіе", Some(0), false),
            ("сіѧ", Some(1), true),
            ("кївотъ", Some(1), false),
        ] {
            let form = Form::new(letters, stress, mark);
            let back = Form::from_print(&form.print(SYN));
            assert_eq!(back.stress, stress);
            assert_eq!(back.key(), form.key());
            assert_eq!(back.letters, form.letters, "{letters}");
            assert_eq!(back.number_mark, mark && form.print(SYN).contains('\u{311}'));
        }
    }
}
