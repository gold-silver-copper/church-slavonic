//! The guesser: a provisional lexeme from a lemma alone, for words the
//! lexicon lacks. It reads the lemma's ending the way the 1.x rule engine
//! did and names a class of the inventory; gender follows the class,
//! animacy is left to the class default, and the stress paradigm is `a`
//! unless the accented lemma stresses its own ending vowel (`рꙋка̀`,
//! `землѧ̀`: `b`). Every form it produces carries `Provenance::Guessed`.
//! Its accuracy is the third eval number (leave-one-out over the lexicon).

use crate::cell::Pos;
use crate::form::Form;
use crate::grammar::{Gender, Recension};
use crate::lexicon::{Lexeme, Lexicon, Provenance};
use crate::orthography::is_vowel_letter;

/// Guess the noun class and gender of a Synodal lemma from its letters.
pub fn noun_class(letters: &str) -> (&'static str, Gender) {
    let chars: Vec<char> = letters.chars().collect();
    let n = chars.len();
    let last = chars.last().copied().unwrap_or(' ');
    let before = |k: usize| chars.get(n.wrapping_sub(k)).copied().unwrap_or(' ');
    let ends = |s: &str| letters.ends_with(s);
    use Gender::*;
    match last {
        'й' => {
            if ends("ій") { ("N1i", Masculine) } else if ends("ей") { ("N1e", Masculine) } else { ("N1a", Masculine) }
        }
        'ъ' => {
            if ends("анинъ") || ends("ѧнинъ") || ends("ѣнинъ") {
                return ("N1in", Masculine);
            }
            match before(2) {
                'к' => ("N1k", Masculine),
                'г' => ("N1g", Masculine),
                'х' => ("N1x", Masculine),
                'ж' | 'ш' | 'щ' | 'ч' => ("N1s", Masculine),
                'ц' => {
                    if before(3) == 'е' && !is_vowel_letter(before(4)) { ("N1c*", Masculine) } else { ("N1c", Masculine) }
                }
                _ => ("N1t", Masculine),
            }
        }
        'ь' => match before(2) {
            'ч' | 'ж' | 'ш' | 'щ' => ("N1sj", Masculine),
            'д' | 'т' | 'з' | 'с' | 'в' | 'б' | 'п' | 'м' if !ends("тель") => ("N41", Feminine),
            _ => ("N1j", Masculine),
        },
        'о' => match before(2) {
            'к' => ("N2k", Neuter),
            'г' => ("N2g", Neuter),
            _ => ("N2t", Neuter),
        },
        'е' => {
            if ends("іе") {
                ("N2i", Neuter)
            } else if matches!(before(2), 'ж' | 'ш' | 'щ' | 'ч') {
                ("N2s", Neuter)
            } else if before(2) == 'ц' {
                ("N2c", Neuter)
            } else {
                ("N2j", Neuter)
            }
        }
        'а' => {
            if ends("іа") {
                ("N3e", Feminine)
            } else {
                match before(2) {
                    'к' | 'г' | 'х' => ("N3k", Feminine),
                    'ц' => ("N3c", Feminine),
                    'ж' | 'ш' | 'щ' | 'ч' => ("N3s", Feminine),
                    _ => ("N3t", Feminine),
                }
            }
        }
        'ѧ' | 'ꙗ' => {
            if ends("мѧ") {
                ("N5en", Neuter)
            } else if ends("іѧ") {
                ("N3i", Feminine)
            } else if is_vowel_letter(before(2)) {
                ("N3a", Feminine)
            } else {
                ("N3j", Feminine)
            }
        }
        'и' => {
            if letters == "мати" || ends("дщи") { ("N5er", Feminine) } else { ("0", Feminine) }
        }
        _ => ("0", Masculine),
    }
}

impl Lexicon {
    /// The class the lexicon's own lexemes give a lemma ending: the
    /// commonest class among the lexemes of the part of speech sharing the
    /// lemma's last three letters, then two, then one — the OCS guesser,
    /// which reads the lexicon instead of a hand rule. `0` when nothing
    /// shares an ending.
    pub fn class_by_ending(&self, letters: &str, pos: Pos) -> &'static str {
        let index = self.ending_index();
        let chars: Vec<char> = letters.chars().collect();
        for n in (1..=3).rev() {
            if chars.len() < n {
                continue;
            }
            let ending: String = chars[chars.len() - n..].iter().collect();
            if let Some(class) = index.get(&(pos, ending)) {
                return class;
            }
        }
        "0"
    }

    fn ending_index(&self) -> &std::collections::HashMap<(Pos, String), &'static str> {
        self.ending_slot().get_or_init(|| {
            let mut votes: std::collections::HashMap<(Pos, String), std::collections::HashMap<String, usize>> = std::collections::HashMap::new();
            for l in self.iter() {
                if l.class == "0" || l.class.is_empty() {
                    continue;
                }
                let chars: Vec<char> = Form::from_print(&l.lemma).letters.chars().collect();
                for n in 1..=3 {
                    if chars.len() < n {
                        continue;
                    }
                    let ending: String = chars[chars.len() - n..].iter().collect();
                    *votes.entry((l.pos, ending)).or_default().entry(l.class.clone()).or_default() += 1;
                }
            }
            votes
                .into_iter()
                .map(|(k, classes)| {
                    let best = classes.into_iter().max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0))).map(|(c, _)| c).unwrap_or_default();
                    (k, &*Box::leak(best.into_boxed_str()))
                })
                .collect()
        })
    }

    /// A provisional lexeme for a lemma the lexicon lacks: the class and
    /// gender guessed from the lemma's letters, the stress paradigm from
    /// its accent, `Provenance::Guessed` on the line.
    pub fn guess(&self, lemma: &str, pos: Pos) -> Lexeme {
        let form = Form::from_print(lemma);
        let (class, gender) = match (self.recension, pos) {
            (Recension::Synodal, Pos::Noun) => noun_class(&form.letters),
            (Recension::OldChurchSlavonic, _) => (self.class_by_ending(&form.letters, pos), Gender::Masculine),
            _ => ("0", Gender::Masculine),
        };
        let stems = Vec::new();
        let stress = match self.recension {
            Recension::OldChurchSlavonic => String::new(),
            Recension::Synodal => match form.stress {
                None => String::new(),
                Some(k) => {
                    let vowels = form.letters.chars().filter(|c| is_vowel_letter(*c)).count();
                    let ends_in_vowel = form.letters.chars().last().is_some_and(is_vowel_letter);
                    if ends_in_vowel && usize::from(k) + 1 == vowels { "b" } else { "a" }.to_string()
                }
            },
        };
        Lexeme {
            id: format!("{}.{}", form.letters, pos.tag()),
            lemma: lemma.to_string(),
            pos,
            gender: Some(gender),
            animate: None,
            class: class.to_string(),
            stress,
            stems,
            overrides: Vec::new(),
            variants: Vec::new(),
            src: Vec::new(),
            note: String::new(),
            provenance: Provenance::Guessed,
            recension: self.recension,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::NounCell;
    use unicode_normalization::UnicodeNormalization;

    #[test]
    fn guesses_follow_the_ending() {
        assert_eq!(noun_class("рабъ").0, "N1t");
        assert_eq!(noun_class("отрокъ").0, "N1k");
        assert_eq!(noun_class("отецъ").0, "N1c*");
        assert_eq!(noun_class("мѣсѧцъ").0, "N1c");
        assert_eq!(noun_class("галілеанинъ").0, "N1in");
        assert_eq!(noun_class("царь").0, "N1j");
        assert_eq!(noun_class("заповѣдь"), ("N41", Gender::Feminine));
        assert_eq!(noun_class("знаменіе").0, "N2i");
        assert_eq!(noun_class("село").0, "N2t");
        assert_eq!(noun_class("землѧ").0, "N3j");
        assert_eq!(noun_class("рꙋка").0, "N3k");
        assert_eq!(noun_class("имѧ").0, "N5en");
        let syn = Lexicon::synodal();
        let g = syn.guess("а҆дама́нтъ", Pos::Noun);
        assert_eq!(g.provenance, Provenance::Guessed);
        assert_eq!(g.class, "N1t");
        assert_eq!(g.stress, "a");
        let f = g.inflect(NounCell::parse("dat.pl").expect("cell")).expect("form");
        assert_eq!(f.print(Recension::Synodal), "а҆дама́нтѡмъ".nfc().collect::<String>());
        let r = syn.guess("рꙋка̀", Pos::Noun);
        assert_eq!(r.stress, "b");
        assert_eq!(r.inflect(NounCell::parse("gen.sg").expect("cell")).expect("form").print(Recension::Synodal), "рꙋкѝ".nfc().collect::<String>());
    }
}
