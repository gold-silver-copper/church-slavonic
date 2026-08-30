//! Declined participles: a verb stem in the adjective's clothes.
//!
//! Four stems per verb — present/past × active/passive — each declined
//! through [`ChurchSlavonicCore::decline_stem`], the same machinery the
//! adjectives use: the active stems (`-ѫщ-`/`-ꙋщ-`, `-въш-`/`-вш-`) are
//! soft, the passive stems (`-ом-`/`-ем-`/`-им-`, `-н-`/`-ен-`/`-т-`)
//! hard. The short active masculine (and neuter) nominative singular is the
//! suppletion-aware citation form [`ChurchSlavonicCore::verb`] already
//! produces (`несꙑ`, `шьдъ`, `сꙑ`); the short active feminine nominative is
//! the stem's `-и`. Where the grammars disagree or a class resists the rule
//! (the `-нѫти` past passives, the OCS long-active nominative contraction),
//! the prediction stays close to the majority pattern and the tables carry
//! the exceptions, as everywhere else.

use crate::ChurchSlavonicCore;
use crate::accent::with_accent;
use crate::grammar::*;
use crate::orthography::strip_marks;
use crate::verb::{Conj, iotate};

impl ChurchSlavonicCore {
    /// Decline a participle by rule. The lemma is the infinitive, accented
    /// in Synodal (see [`crate::accent`]). `Tense::Imperfect` and
    /// `Tense::Aorist` both select the past participle, as in
    /// [`ChurchSlavonicCore::verb`].
    #[allow(clippy::too_many_arguments)]
    pub fn participle(
        word: &str,
        tense: &Tense,
        voice: &Voice,
        series: &Series,
        case: &Case,
        number: &Number,
        gender: &Gender,
        recension: &Recension,
    ) -> String {
        // The reflexive `-сѧ` rides outside every form, as in
        // [`ChurchSlavonicCore::verb`]; the jer drops before the enclitic.
        if *recension == Recension::Synodal && strip_marks(word).ends_with("сѧ") {
            let bare: String = word.chars().take(word.chars().count() - 2).collect();
            return with_accent(&bare, recension, |w| {
                let answer =
                    Self::participle_skeleton(w, tense, voice, series, case, number, gender, recension);
                format!("{}сѧ", answer.strip_suffix('ъ').unwrap_or(&answer))
            });
        }
        with_accent(word, recension, |w| {
            Self::participle_skeleton(w, tense, voice, series, case, number, gender, recension)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn participle_skeleton(
        word: &str,
        tense: &Tense,
        voice: &Voice,
        series: &Series,
        case: &Case,
        number: &Number,
        gender: &Gender,
        recension: &Recension,
    ) -> String {
        let past = !matches!(tense, Tense::Present);
        let (stem, _) = Self::participle_stem(word, past, voice, recension);
        match Self::participle_from_stem(
            &stem, past, voice, series, case, number, gender, recension,
        ) {
            Some(form) => form,
            // The cells a stem cannot derive: the present active
            // citation-shaped nominatives (`несꙑ`, `несꙑи`).
            None => {
                let citation = Self::citation(word, past, recension);
                if *series == Series::Long && *recension == Recension::OldChurchSlavonic {
                    format!("{citation}и")
                } else {
                    citation
                }
            }
        }
    }

    /// Expand one participle cell from a stem — the shared path of the rule,
    /// the runtime's stem-cell lookup, and the extractor's stem inference.
    /// `None` marks the cells a stem alone cannot produce: the PRESENT
    /// active masculine/neuter nominative-singular shapes and the long
    /// masculine nominative (the `-ꙑ`/`-ꙑи` contractions); the PAST active
    /// citation shapes derive by stripping the stem's `-ш-`.
    #[allow(clippy::too_many_arguments)]
    pub fn participle_from_stem(
        stem: &str,
        past: bool,
        voice: &Voice,
        series: &Series,
        case: &Case,
        number: &Number,
        gender: &Gender,
        recension: &Recension,
    ) -> Option<String> {
        let synodal = *recension == Recension::Synodal;
        let long = *series == Series::Long;
        // The passive stems are hard everywhere; the Synodal print declines
        // its active `-щ-`/`-ш-` stems on the hard rows too (`-щагѡ`,
        // `-щымъ`), where OCS keeps them soft (`-щаѥго`, `-щиимъ`).
        let hard = *voice == Voice::Passive || synodal;
        if *voice == Voice::Active {
            let citation = || {
                if past {
                    let base = stem.strip_suffix('ш')?.to_string();
                    return Some(if synodal {
                        format!("{base}ъ")
                    } else {
                        base
                    });
                }
                // The present-active nominative contracts off the stem's
                // suffix: OCS `-ѫщ` -> `-ꙑ`, `-ѭщ` -> `-ѩ`, `-ѧщ` -> `-ѧ`;
                // Synodal `-ꙋщ` -> `-ый`, `-ющ`/`-ѧщ`/`-ащ` -> `-ѧй`/`-ай`.
                for (suffix, nom) in if synodal {
                    &[("ꙋщ", "ый"), ("ющ", "ѧй"), ("ѧщ", "ѧй"), ("ащ", "ай")][..]
                } else {
                    &[("ѫщ", "ꙑ"), ("ѭщ", "ѩ"), ("ѧщ", "ѧ"), ("ꙙщ", "ꙙ")][..]
                } {
                    if let Some(base) = stem.strip_suffix(suffix) {
                        return Some(format!("{base}{nom}"));
                    }
                }
                None
            };
            // The citation-shaped cells: the masculine nominative (and, in
            // the print, the masculine accusative), plus the OCS neuter
            // nominative; the Synodal neuter and the OCS neuter accusative
            // take the declined `-е` instead.
            let citation_shaped = *number == Number::Singular
                && ((*gender == Gender::Masculine
                    && (matches!(case, Case::Nominative | Case::Vocative)
                        || (synodal && *case == Case::Accusative)))
                    || (!synodal
                        && *gender == Gender::Neuter
                        && matches!(case, Case::Nominative | Case::Vocative)));
            if !long && citation_shaped {
                return citation();
            }
            if !long
                && *gender == Gender::Feminine
                && *number == Number::Singular
                && matches!(case, Case::Nominative | Case::Vocative)
            {
                return Some(format!("{stem}и"));
            }
            if long
                && *gender == Gender::Masculine
                && *number == Number::Singular
                && matches!(case, Case::Nominative | Case::Vocative)
            {
                let c = citation()?;
                return Some(if synodal {
                    let c = c.trim_end_matches('ъ');
                    if past { format!("{c}ый") } else { c.to_string() }
                } else if past {
                    format!("{c}и")
                } else {
                    format!("{c}и")
                });
            }
            if !long
                && *gender == Gender::Masculine
                && *number == Number::Plural
                && matches!(case, Case::Nominative | Case::Vocative)
            {
                // The consonant-stem `-е`: `несѫще`, `дѣлавъше`.
                return Some(Self::attach(stem, "е", recension));
            }
            if !synodal {
                // The OCS long series keeps participle-specific shapes the
                // long adjective does not: the `-щиимь` locative, the
                // `-щиꙗ` feminine nominative, the `-щѫѭ` feminine
                // instrumental, the `-щеи` masculine nominative plural, and
                // the short neuter accusative on the `-е` of the plural.
                if long {
                    let over = match (gender, number, case) {
                        (Gender::Masculine | Gender::Neuter, Number::Singular, Case::Locative) => {
                            Some("иимь")
                        }
                        (Gender::Feminine, Number::Singular, Case::Nominative | Case::Vocative) => {
                            Some("иꙗ")
                        }
                        (Gender::Feminine, Number::Singular, Case::Instrumental) => Some("ѫѭ"),
                        (
                            Gender::Masculine,
                            Number::Plural,
                            Case::Nominative | Case::Vocative,
                        ) => Some("еи"),
                        _ => None,
                    };
                    if let Some(over) = over {
                        return Some(format!("{stem}{over}"));
                    }
                }
            }
            if synodal {
                // The print's own mixed declension for the `-щ-`/`-ш-`
                // stems (Alypy's ending tables, pp. 95–96): hard in the
                // genitive (`-щагѡ`) and the dative plural (`-щымъ`), soft
                // in the genitive/locative plural (`-щихъ`); the endings
                // ride on the husher untouched, so they attach plainly.
                let row = if long {
                    &SYN_ACTIVE_LONG[*gender as usize]
                } else {
                    &SYN_ACTIVE_SHORT[*gender as usize]
                };
                let cell = *number as usize * 7 + *case as usize;
                return Some(format!("{stem}{}", row[cell]));
            }
        }
        // The stem path never re-enters the accent pass, so the kamora
        // marker the adjective rows carry is dropped here: the stem already
        // holds its accent.
        Some(Self::decline_stem(stem, long, hard, case, number, gender, recension).replace('^', ""))
    }

    /// The suppletion-aware masculine nominative-singular citation.
    fn citation(word: &str, past: bool, recension: &Recension) -> String {
        let tense = if past { Tense::Aorist } else { Tense::Present };
        Self::verb(
            word,
            &Person::Third,
            &Number::Singular,
            &tense,
            &Form::Participle,
            recension,
        )
    }

    /// [`ChurchSlavonicCore::participle`] with a class/present-stem
    /// override: the present-tense stems are rebuilt on the overridden
    /// class, so one override serves the finite block and the present
    /// participles alike. Skeleton-level, like `verb_from_stems`.
    #[allow(clippy::too_many_arguments)]
    pub fn participle_with_override(
        word: &str,
        class: Option<&str>,
        present: Option<&str>,
        tense: &Tense,
        voice: &Voice,
        series: &Series,
        case: &Case,
        number: &Number,
        gender: &Gender,
        recension: &Recension,
    ) -> String {
        let past = !matches!(tense, Tense::Present);
        let s = Self::override_stems(word, class, present, recension);
        let (stem, _) = Self::participle_stem_from(word, &s, past, voice, recension);
        let answer = match Self::participle_from_stem(
            &stem, past, voice, series, case, number, gender, recension,
        ) {
            Some(form) => form,
            None => {
                let citation = Self::citation(word, past, recension);
                if *series == Series::Long && *recension == Recension::OldChurchSlavonic {
                    format!("{citation}и")
                } else {
                    citation
                }
            }
        };
        if *recension == Recension::Synodal {
            crate::accent::final_varia(&answer)
        } else {
            answer
        }
    }

    /// `(stem, hard)` for one tense/voice pair.
    fn participle_stem(
        word: &str,
        past: bool,
        voice: &Voice,
        recension: &Recension,
    ) -> (String, bool) {
        let s = Self::stems(word, recension);
        Self::participle_stem_from(word, &s, past, voice, recension)
    }

    fn participle_stem_from(
        word: &str,
        s: &crate::verb::Stems,
        past: bool,
        voice: &Voice,
        recension: &Recension,
    ) -> (String, bool) {
        let synodal = *recension == Recension::Synodal;
        match (voice, past) {
            (Voice::Active, false) => {
                let suffix = match (s.conj, synodal) {
                    (Conj::Hard | Conj::Iotated, false) => "ѫщ",
                    (Conj::Hard | Conj::Iotated, true) => "ꙋщ",
                    (Conj::Vowel, false) => "ѭщ",
                    (Conj::Vowel, true) => "ющ",
                    (Conj::Second, _) => "ѧщ",
                };
                (Self::attach(&s.present, suffix, recension), false)
            }
            (Voice::Active, true) => {
                // The citation (`несъ`, `дѣлавъ`, `пришьдъ`) plus `-ш-`;
                // Synodal drops the jer before it (`дѣлавш-`).
                let citation = Self::citation(word, true, recension);
                let base = if synodal {
                    citation.trim_end_matches(['ъ', 'ь']).to_string()
                } else {
                    citation
                };
                (format!("{base}ш"), false)
            }
            (Voice::Passive, false) => {
                let suffix = match (s.conj, synodal) {
                    (Conj::Hard | Conj::Iotated, _) => "ом",
                    (Conj::Vowel, false) => "ѥм",
                    (Conj::Vowel, true) => "ем",
                    (Conj::Second, _) => "им",
                };
                (Self::attach(&s.present, suffix, recension), true)
            }
            (Voice::Passive, true) => {
                let stem = &s.infinitive;
                let last = stem.chars().last().unwrap_or(' ');
                let stem = if matches!(last, 'ѧ' | 'ѩ' | 'ѫ' | 'ꙑ' | 'ы' | 'ꙋ' | 'у') {
                    // The t-participle roots: `взѧтъ`, `покрꙑтъ`.
                    format!("{stem}т")
                } else if s.conj == Conj::Second {
                    // The second conjugation iotates: `хвалѥнъ`, `ношенъ`.
                    let iotated = iotate(&s.present);
                    let e = if !synodal && !ends_husher(&iotated) {
                        "ѥн"
                    } else {
                        "ен"
                    };
                    Self::attach(&iotated, e, recension)
                } else if matches!(last, 'а' | 'ѣ') {
                    format!("{stem}н")
                } else {
                    Self::attach(stem, "ен", recension)
                };
                (stem, true)
            }
        }
    }
}

// The Synodal print's active-participle endings on the `-щ-`/`-ш-` stem
// (Alypy pp. 95–96 and the corpus majorities), per gender, 21 cells in the
// noun cell order. The masculine (and, short, neuter) nominative-singular
// shapes never reach these rows — the citation specials answer first.
type PRow = [&'static str; 21];
const SYN_ACTIVE_SHORT: [PRow; 3] = [
    [
        "ь", "а", "ꙋ", "а", "имъ", "и", "ь", "а", "ꙋ", "ема", "а", "ема", "ꙋ", "а", "е", "ихъ",
        "ымъ", "ѧ", "ими", "ихъ", "е",
    ],
    [
        "и", "іѧ", "и", "ꙋ", "ею", "и", "и", "ѣ", "ꙋ", "ема", "ѣ", "ема", "ꙋ", "ѣ", "ѧ", "ихъ",
        "ымъ", "ѧ", "ими", "ихъ", "ѧ",
    ],
    [
        "е", "а", "ꙋ", "е", "имъ", "и", "е", "ѣ", "ꙋ", "ема", "ѣ", "ема", "ꙋ", "ѣ", "а", "ихъ",
        "ымъ", "а", "ими", "ихъ", "а",
    ],
];
const SYN_ACTIVE_LONG: [PRow; 3] = [
    [
        "ій", "агѡ", "емꙋ", "аго", "имъ", "емъ", "ій", "аѧ", "ꙋю", "има", "аѧ", "има", "ꙋю",
        "аѧ", "іи", "ихъ", "ымъ", "ыѧ", "ими", "ихъ", "іи",
    ],
    [
        "аѧ", "іѧ", "еи", "ꙋю", "ею", "еи", "аѧ", "іи", "ꙋю", "има", "іи", "има", "ꙋю", "іи",
        "ыѧ", "ихъ", "ымъ", "ыѧ", "ими", "ихъ", "ыѧ",
    ],
    [
        "ее", "агѡ", "емꙋ", "ее", "имъ", "емъ", "ее", "іи", "ꙋю", "има", "іи", "има", "ꙋю", "іи",
        "аѧ", "ихъ", "ымъ", "аѧ", "ими", "ихъ", "аѧ",
    ],
];

fn ends_husher(stem: &str) -> bool {

    matches!(stem.chars().last(), Some('ж' | 'ч' | 'ш' | 'щ')) || stem.ends_with("жд")
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::Synodal;

    fn p(
        w: &str,
        t: Tense,
        v: Voice,
        s: Series,
        c: Case,
        n: Number,
        g: Gender,
        r: Recension,
    ) -> String {
        ChurchSlavonicCore::participle(w, &t, &v, &s, &c, &n, &g, &r)
    }

    #[test]
    fn active_series_across_the_classes() {
        use {Case::*, Gender::*, Number::*, Series::*, Tense::*, Voice::*};
        // Present active: citation, feminine -и, oblique -ѫщ-.
        assert_eq!(
            p("нести", Present, Active, Short, Nominative, Singular, Masculine, OCS),
            "несꙑ"
        );
        assert_eq!(
            p("нести", Present, Active, Short, Nominative, Singular, Feminine, OCS),
            "несѫщи"
        );
        assert_eq!(
            p("нести", Present, Active, Short, Genitive, Singular, Masculine, OCS),
            "несѫща"
        );
        assert_eq!(
            p("хвалити", Present, Active, Short, Genitive, Singular, Masculine, OCS),
            "хвалѧща"
        );
        assert_eq!(
            p("дѣлати", Present, Active, Short, Genitive, Singular, Masculine, OCS),
            "дѣлаѭща"
        );
        assert_eq!(
            p("нести", Present, Active, Long, Nominative, Singular, Masculine, OCS),
            "несꙑи"
        );
        assert_eq!(
            p("нести", Present, Active, Long, Genitive, Singular, Masculine, OCS),
            "несѫщаѥго"
        );
        // Past active: citation + -ш-.
        assert_eq!(
            p("нести", Aorist, Active, Short, Genitive, Singular, Masculine, OCS),
            "несъша"
        );
        assert_eq!(
            p("дѣлати", Aorist, Active, Short, Nominative, Plural, Masculine, OCS),
            "дѣлавъше"
        );
        assert_eq!(
            p("прити", Aorist, Active, Short, Nominative, Singular, Feminine, OCS),
            "пришьдъши"
        );
    }

    #[test]
    fn passive_series_across_the_classes() {
        use {Case::*, Gender::*, Number::*, Series::*, Tense::*, Voice::*};
        assert_eq!(
            p("нести", Present, Passive, Short, Nominative, Singular, Masculine, OCS),
            "несомъ"
        );
        assert_eq!(
            p("хвалити", Present, Passive, Short, Nominative, Singular, Masculine, OCS),
            "хвалимъ"
        );
        assert_eq!(
            p("дѣлати", Aorist, Passive, Short, Nominative, Singular, Masculine, OCS),
            "дѣланъ"
        );
        assert_eq!(
            p("рещи", Aorist, Passive, Short, Nominative, Singular, Masculine, OCS),
            "реченъ"
        );
        assert_eq!(
            p("хвалити", Aorist, Passive, Short, Nominative, Singular, Masculine, OCS),
            "хвалѥнъ"
        );
        assert_eq!(
            p("носити", Aorist, Passive, Short, Nominative, Singular, Masculine, OCS),
            "ношенъ"
        );
        assert_eq!(
            p("възѧти", Aorist, Passive, Short, Nominative, Singular, Masculine, OCS),
            "възѧтъ"
        );
        assert_eq!(
            p("дѣлати", Aorist, Passive, Long, Genitive, Singular, Masculine, OCS),
            "дѣланаѥго"
        );
    }

    #[test]
    fn synodal_series_spell_the_print() {
        use {Case::*, Gender::*, Number::*, Series::*, Tense::*, Voice::*};
        assert_eq!(
            p("вести́", Present, Active, Short, Genitive, Singular, Masculine, SYN),
            "ведꙋ́ща"
        );
        assert_eq!(
            p("дѣ́лати", Aorist, Active, Short, Genitive, Singular, Masculine, SYN),
            "дѣ́лавша"
        );
        assert_eq!(
            p("дѣ́лати", Aorist, Passive, Short, Nominative, Singular, Feminine, SYN),
            "дѣ́лана"
        );
    }
}
