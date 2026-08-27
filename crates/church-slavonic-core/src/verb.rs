//! Regular verb conjugation.
//!
//! The infinitive's ending picks the conjugation and the two stems the
//! paradigm is built on (the present stem and the infinitive stem):
//! `-ати` first conjugation on a vowel stem (`дѣла-`), `-овати` on `-оу-`/
//! `-ꙋ-`, `-ноути`/`-нꙋти` on `-н-`, consonant `-ти` and velar `-щи` on the
//! bare consonant, `-ити`/`-ѣти`/husher + `-ати` second conjugation. This is
//! an approximation: iotating `-ати` stems (`писати` : `пишеши`), dental
//! infinitives (`вести` : `ведеши`), the `-мѣти` first-conjugation
//! `-ѣти` verbs and the mutating second-conjugation first singular
//! (`любити` : `люблю`) are tabled. The copula `бꙑти`/`быти` is fully
//! suppletive and handled by [`ChurchSlavonicCore::to_be`].

use crate::ChurchSlavonicCore;
use crate::grammar::*;

/// The three productive present-stem series.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Conj {
    /// First conjugation, consonant stem (`нес-е-ши`).
    Hard,
    /// First conjugation, vowel stem (OCS `дѣла-ѥ-ши`, Synodal `дѣла-е-ши`).
    Vowel,
    /// Second conjugation (`хвал-и-ши`).
    Second,
}

struct Stems {
    conj: Conj,
    /// The stem the present endings attach to.
    present: String,
    /// The stem the aorist, past participle and (in OCS) imperfect attach to.
    infinitive: String,
}

impl ChurchSlavonicCore {
    /// Conjugate a verb by rule.
    pub fn verb(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
    ) -> String {
        let synodal = *recension == Recension::Synodal;
        if word == "бꙑти" || word == "быти" {
            if let Some(form) = Self::to_be(person, number, tense, form, recension) {
                return form.to_string();
            }
        }
        let s = Self::stems(word, recension);
        let cell = Self::person_cell(person, number);
        match (tense, form) {
            (_, Form::Infinitive) => word.to_string(),
            (Tense::Present, Form::Finite) => {
                let row = match (s.conj, synodal) {
                    (Conj::Hard, false) => &PRESENT_HARD.0,
                    (Conj::Hard, true) => &PRESENT_HARD.1,
                    (Conj::Vowel, false) => &PRESENT_VOWEL.0,
                    (Conj::Vowel, true) => &PRESENT_VOWEL.1,
                    (Conj::Second, false) => &PRESENT_SECOND.0,
                    (Conj::Second, true) => &PRESENT_SECOND.1,
                };
                Self::attach(&s.present, row[cell], recension)
            }
            (Tense::Imperfect, Form::Finite) => {
                let (stem, marker) = Self::imperfect_stem(&s, recension);
                let row = if synodal { &IMPERFECT.1 } else { &IMPERFECT.0 };
                Self::attach(&stem, &format!("{marker}{}", row[cell]), recension)
            }
            (Tense::Aorist, Form::Finite) => {
                let row = match (s.conj == Conj::Hard, synodal) {
                    (true, false) => &AORIST_OX.0,
                    (true, true) => &AORIST_OX.1,
                    (false, false) => &AORIST_SIGMATIC.0,
                    (false, true) => &AORIST_SIGMATIC.1,
                };
                Self::attach(&s.infinitive, row[cell], recension)
            }
            (Tense::Present, Form::Participle) => {
                // verb:present-active-nominative-contraction: OCS -ꙑ/-ѩ
                // against Synodal -ый/-ѧ.
                let ending = match (s.conj, synodal) {
                    (Conj::Hard, false) => "ꙑ",
                    (Conj::Hard, true) => "ый",
                    (Conj::Vowel, false) => "ѩ",
                    (Conj::Vowel, true) | (Conj::Second, _) => "ѧ",
                };
                Self::attach(&s.present, ending, recension)
            }
            (Tense::Aorist | Tense::Imperfect, Form::Participle) => {
                // The imperfect has no participle of its own; both pasts
                // resolve to the past active participle.
                if s.conj == Conj::Hard {
                    format!("{}ъ", s.infinitive)
                } else if s.conj == Conj::Second && !synodal && s.infinitive.ends_with('и') {
                    format!("{}ь", s.present)
                } else {
                    format!("{}въ", s.infinitive)
                }
            }
            (_, Form::Imperative) => {
                // No first singular or third dual/plural exist: the first
                // singular answers with the hortative first plural, the third
                // person with the second.
                let cell = match (person, number) {
                    (Person::First, Number::Singular) => 6,
                    (Person::Third, n) => Self::person_cell(&Person::Second, n),
                    _ => cell,
                };
                let row = match (s.conj, synodal) {
                    (Conj::Hard, false) => &IMPERATIVE_YAT,
                    (Conj::Hard, true) => &IMPERATIVE_E,
                    (Conj::Vowel, true) => &IMPERATIVE_J,
                    (Conj::Vowel, false) | (Conj::Second, _) => {
                        if synodal {
                            &IMPERATIVE_I.1
                        } else {
                            &IMPERATIVE_I.0
                        }
                    }
                };
                Self::attach(&s.present, row[cell], recension)
            }
        }
    }

    /// The fully suppletive paradigm of `бꙑти`/`быти` — present, imperfect
    /// and aorist finites plus the two participles. Imperative and
    /// infinitive fall through to the regular route on the `бѫд-`/`бꙋд-`
    /// stem. The tense assignment differs per recension
    /// (verb:copula-tense-reassignment): the OCS aorist is the `бѣхъ`
    /// series, the Synodal aorist the `быхъ`/`бысть` series.
    pub fn to_be(
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
    ) -> Option<&'static str> {
        let synodal = *recension == Recension::Synodal;
        let cell = Self::person_cell(person, number);
        Some(match (tense, form, synodal) {
            (Tense::Present, Form::Finite, false) => BE_PRESENT.0[cell],
            (Tense::Present, Form::Finite, true) => BE_PRESENT.1[cell],
            (Tense::Imperfect, Form::Finite, false) => BE_IMPERFECT.0[cell],
            (Tense::Imperfect, Form::Finite, true) => BE_IMPERFECT.1[cell],
            (Tense::Aorist, Form::Finite, false) => BE_AORIST.0[cell],
            (Tense::Aorist, Form::Finite, true) => BE_AORIST.1[cell],
            (Tense::Present, Form::Participle, false) => "сꙑ",
            (Tense::Present, Form::Participle, true) => "сый",
            (_, Form::Participle, false) => "бꙑвъ",
            (_, Form::Participle, true) => "бывъ",
            _ => return None,
        })
    }

    fn stems(word: &str, recension: &Recension) -> Stems {
        let synodal = *recension == Recension::Synodal;
        let stem = |n: usize| -> String {
            let len = word.chars().count().saturating_sub(n);
            word.chars().take(len).collect()
        };
        let husher = matches!(stem(3).chars().last(), Some('ж' | 'ч' | 'ш' | 'щ'));
        let mk = |conj, present: String, infinitive: String| Stems {
            conj,
            present,
            infinitive,
        };
        if word == "бꙑти" || word == "быти" {
            let bud = if synodal { "бꙋд" } else { "бѫд" };
            mk(Conj::Hard, bud.into(), stem(2))
        } else if word.ends_with("овати") || word.ends_with("евати") {
            let present = format!("{}{}", stem(5), if synodal { "ꙋ" } else { "оу" });
            mk(Conj::Vowel, present, stem(2))
        } else if word.ends_with("ноути") || word.ends_with("нꙋти") {
            mk(Conj::Hard, stem(3), stem(2))
        } else if word.ends_with("мѣти") {
            mk(Conj::Vowel, stem(2), stem(2))
        } else if word.ends_with("ити")
            || word.ends_with("ѣти")
            || (word.ends_with("ати") && husher)
        {
            mk(Conj::Second, stem(3), stem(2))
        } else if word.ends_with("ати") || word.ends_with("ꙗти") || word.ends_with("ѧти")
        {
            mk(Conj::Vowel, stem(2), stem(2))
        } else if word.ends_with("шти") || word.ends_with("щи") {
            // Velar infinitives: the stem ends in `к` (the `г` stems `мошти`
            // are tabled); the seam rule palatalizes before the endings.
            let cut = if word.ends_with("шти") { 3 } else { 2 };
            mk(
                Conj::Hard,
                format!("{}к", stem(cut)),
                format!("{}к", stem(cut)),
            )
        } else {
            mk(Conj::Hard, stem(2), stem(2))
        }
    }

    /// The imperfect's stem and tense marker: OCS keeps the uncontracted
    /// primaries (`дѣлаахъ`, `несѣахъ`, `хвалꙗахъ`, `видѣахъ`), Synodal the
    /// contracted `-ѧ-`/`-а-` grades (`дѣлахъ`, `несѧхъ`, `хвалѧхъ`,
    /// `видѧхъ`) — verb:imperfect-contraction.
    fn imperfect_stem(s: &Stems, recension: &Recension) -> (String, &'static str) {
        let synodal = *recension == Recension::Synodal;
        let velar = matches!(s.present.chars().last(), Some('к' | 'г' | 'х'));
        match s.conj {
            Conj::Hard if velar => (
                Self::attach(&s.present, "е", recension)
                    .trim_end_matches('е')
                    .to_string(),
                if synodal { "а" } else { "аа" },
            ),
            Conj::Hard => (s.present.clone(), if synodal { "ѧ" } else { "ѣа" }),
            Conj::Vowel => (s.infinitive.clone(), if synodal { "" } else { "а" }),
            Conj::Second if s.infinitive.ends_with('ѣ') => {
                if synodal {
                    (s.present.clone(), "ѧ")
                } else {
                    (s.infinitive.clone(), "а")
                }
            }
            Conj::Second => (s.present.clone(), if synodal { "ѧ" } else { "ꙗа" }),
        }
    }
}

// Nine cells per row: singular 1 2 3, dual 1 2 3, plural 1 2 3 — as `(OCS,
// Synodal)` pairs. Every dual differs by verb:dual-first-person-va (-вѣ/-ва)
// and verb:dual-third-person-leveling (OCS distinct 3du, Synodal = 2du).
type Row = [&'static str; 9];
const PRESENT_HARD: (Row, Row) = (
    ["ѫ", "еши", "етъ", "евѣ", "ета", "ете", "емъ", "ете", "ѫтъ"],
    ["ꙋ", "еши", "етъ", "ева", "ета", "ета", "емъ", "ете", "ꙋтъ"],
);
const PRESENT_VOWEL: (Row, Row) = (
    ["ѭ", "ѥши", "ѥтъ", "ѥвѣ", "ѥта", "ѥте", "ѥмъ", "ѥте", "ѭтъ"],
    ["ю", "еши", "етъ", "ева", "ета", "ета", "емъ", "ете", "ютъ"],
);
const PRESENT_SECOND: (Row, Row) = (
    ["ѭ", "иши", "итъ", "ивѣ", "ита", "ите", "имъ", "ите", "ѧтъ"],
    ["ю", "иши", "итъ", "ива", "ита", "ита", "имъ", "ите", "ѧтъ"],
);
// verb:imperfect-hardening: OCS -шета/-шете against Synodal -ста/-сте.
const IMPERFECT: (Row, Row) = (
    [
        "хъ", "ше", "ше", "ховѣ", "шета", "шете", "хомъ", "шете", "хѫ",
    ],
    ["хъ", "ше", "ше", "хова", "ста", "ста", "хомъ", "сте", "хꙋ"],
);
// verb:aorist-third-plural-a-grade: -шѧ against -ша.
const AORIST_SIGMATIC: (Row, Row) = (
    ["хъ", "", "", "ховѣ", "ста", "сте", "хомъ", "сте", "шѧ"],
    ["хъ", "", "", "хова", "ста", "ста", "хомъ", "сте", "ша"],
);
const AORIST_OX: (Row, Row) = (
    [
        "охъ",
        "е",
        "е",
        "оховѣ",
        "оста",
        "осте",
        "охомъ",
        "осте",
        "ошѧ",
    ],
    [
        "охъ",
        "е",
        "е",
        "охова",
        "оста",
        "оста",
        "охомъ",
        "осте",
        "оша",
    ],
);
// verb:imperative-vowel-grade: the OCS yat series against the Synodal e/i
// series, and the Synodal-only contracted j series on vowel stems. Slot 0
// (first singular) is unused.
const IMPERATIVE_YAT: Row = ["", "и", "и", "ѣвѣ", "ѣта", "ѣта", "ѣмъ", "ѣте", "ѣте"];
const IMPERATIVE_E: Row = ["", "и", "и", "ева", "ита", "ита", "емъ", "ите", "ите"];
const IMPERATIVE_J: Row = ["", "й", "й", "йва", "йта", "йта", "ймъ", "йте", "йте"];
const IMPERATIVE_I: (Row, Row) = (
    ["", "и", "и", "ивѣ", "ита", "ита", "имъ", "ите", "ите"],
    ["", "и", "и", "ива", "ита", "ита", "имъ", "ите", "ите"],
);
// verb:copula-third-person-soft-t (ѥстъ/сѫтъ ~ єсть/сꙋть),
// verb:copula-first-plural-my, verb:copula-imperfect-restemming (бѣах- ~
// бѧх-), verb:copula-aorist-sti (бысть).
const BE_PRESENT: (Row, Row) = (
    [
        "ѥсмь", "ѥси", "ѥстъ", "ѥсвѣ", "ѥста", "ѥсте", "ѥсмъ", "ѥсте", "сѫтъ",
    ],
    [
        "єсмь",
        "єси",
        "єсть",
        "єсва",
        "єста",
        "єста",
        "єсмы",
        "єсте",
        "сꙋть",
    ],
);
const BE_IMPERFECT: (Row, Row) = (
    [
        "бѣахъ",
        "бѣаше",
        "бѣаше",
        "бѣаховѣ",
        "бѣашета",
        "бѣашете",
        "бѣахомъ",
        "бѣашете",
        "бѣахѫ",
    ],
    [
        "бѧхъ",
        "бѧше",
        "бѧше",
        "бѧхова",
        "бѧста",
        "бѧста",
        "бѧхомъ",
        "бѧсте",
        "бѧхꙋ",
    ],
);
const BE_AORIST: (Row, Row) = (
    [
        "бѣхъ",
        "бѣ",
        "бѣ",
        "бѣховѣ",
        "бѣста",
        "бѣсте",
        "бѣхомъ",
        "бѣсте",
        "бѣшѧ",
    ],
    [
        "быхъ",
        "бысть",
        "бысть",
        "быхова",
        "быста",
        "быста",
        "быхомъ",
        "бысте",
        "быша",
    ],
);

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;
    const SYN: Recension = Recension::Synodal;

    fn v(w: &str, p: Person, n: Number, t: Tense, f: Form, r: Recension) -> String {
        ChurchSlavonicCore::verb(w, &p, &n, &t, &f, &r)
    }

    #[test]
    fn present_series_and_the_dual_conditions() {
        use Form::Finite;
        use Number::*;
        use Person::*;
        use Tense::Present;
        assert_eq!(v("нести", First, Singular, Present, Finite, OCS), "несѫ");
        assert_eq!(v("нести", First, Singular, Present, Finite, SYN), "несꙋ");
        assert_eq!(
            v("дѣлати", Second, Singular, Present, Finite, OCS),
            "дѣлаѥши"
        );
        assert_eq!(
            v("дѣлати", Second, Singular, Present, Finite, SYN),
            "дѣлаеши"
        );
        assert_eq!(v("дѣлати", Third, Plural, Present, Finite, SYN), "дѣлаютъ");
        assert_eq!(v("хвалити", First, Singular, Present, Finite, OCS), "хвалѭ");
        assert_eq!(v("хвалити", Third, Plural, Present, Finite, SYN), "хвалѧтъ");
        assert_eq!(v("слꙑшати", First, Singular, Present, Finite, OCS), "слꙑшѫ");
        assert_eq!(v("слышати", Third, Plural, Present, Finite, SYN), "слышатъ");
        assert_eq!(v("пешти", Second, Singular, Present, Finite, OCS), "печеши");
        assert_eq!(
            v("цѣловати", Second, Singular, Present, Finite, SYN),
            "цѣлꙋеши"
        );
        assert_eq!(
            v("двигнꙋти", Third, Singular, Present, Finite, SYN),
            "двигнетъ"
        );
        // verb:dual-first-person-va, verb:dual-third-person-leveling
        assert_eq!(v("нести", First, Dual, Present, Finite, OCS), "несевѣ");
        assert_eq!(v("нести", First, Dual, Present, Finite, SYN), "несева");
        assert_eq!(v("нести", Third, Dual, Present, Finite, OCS), "несете");
        assert_eq!(v("нести", Third, Dual, Present, Finite, SYN), "несета");
    }

    #[test]
    fn imperfect_contracts_and_hardens_in_synodal() {
        use Form::Finite;
        use Number::*;
        use Person::*;
        use Tense::Imperfect;
        assert_eq!(
            v("нести", First, Singular, Imperfect, Finite, OCS),
            "несѣахъ"
        );
        assert_eq!(
            v("нести", First, Singular, Imperfect, Finite, SYN),
            "несѧхъ"
        );
        assert_eq!(
            v("дѣлати", First, Singular, Imperfect, Finite, OCS),
            "дѣлаахъ"
        );
        assert_eq!(
            v("дѣлати", First, Singular, Imperfect, Finite, SYN),
            "дѣлахъ"
        );
        assert_eq!(
            v("хвалити", Third, Singular, Imperfect, Finite, OCS),
            "хвалꙗаше"
        );
        assert_eq!(
            v("хвалити", Third, Singular, Imperfect, Finite, SYN),
            "хвалѧше"
        );
        assert_eq!(
            v("видѣти", First, Singular, Imperfect, Finite, SYN),
            "видѧхъ"
        );
        assert_eq!(v("пещи", Third, Singular, Imperfect, Finite, SYN), "печаше");
        assert_eq!(
            v("нести", Second, Plural, Imperfect, Finite, OCS),
            "несѣашете"
        );
        assert_eq!(
            v("нести", Second, Plural, Imperfect, Finite, SYN),
            "несѧсте"
        );
    }

    #[test]
    fn aorists_participles_and_imperatives() {
        use Number::*;
        use Person::*;
        assert_eq!(
            v("нести", First, Singular, Tense::Aorist, Form::Finite, SYN),
            "несохъ"
        );
        assert_eq!(
            v("пещи", Third, Singular, Tense::Aorist, Form::Finite, SYN),
            "пече"
        );
        assert_eq!(
            v("дѣлати", Third, Plural, Tense::Aorist, Form::Finite, OCS),
            "дѣлашѧ"
        );
        assert_eq!(
            v("дѣлати", Third, Plural, Tense::Aorist, Form::Finite, SYN),
            "дѣлаша"
        );
        assert_eq!(
            v(
                "нести",
                Third,
                Singular,
                Tense::Present,
                Form::Participle,
                OCS
            ),
            "несꙑ"
        );
        assert_eq!(
            v(
                "нести",
                Third,
                Singular,
                Tense::Present,
                Form::Participle,
                SYN
            ),
            "несый"
        );
        assert_eq!(
            v(
                "хвалити",
                Third,
                Singular,
                Tense::Aorist,
                Form::Participle,
                OCS
            ),
            "хваль"
        );
        assert_eq!(
            v(
                "хвалити",
                Third,
                Singular,
                Tense::Aorist,
                Form::Participle,
                SYN
            ),
            "хваливъ"
        );
        assert_eq!(
            v(
                "дѣлати",
                Second,
                Singular,
                Tense::Present,
                Form::Imperative,
                OCS
            ),
            "дѣлаи"
        );
        assert_eq!(
            v(
                "дѣлати",
                Second,
                Singular,
                Tense::Present,
                Form::Imperative,
                SYN
            ),
            "дѣлай"
        );
        assert_eq!(
            v(
                "нести",
                Second,
                Plural,
                Tense::Present,
                Form::Imperative,
                OCS
            ),
            "несѣте"
        );
        assert_eq!(
            v(
                "нести",
                Second,
                Plural,
                Tense::Present,
                Form::Imperative,
                SYN
            ),
            "несите"
        );
        assert_eq!(
            v(
                "пещи",
                Second,
                Singular,
                Tense::Present,
                Form::Imperative,
                SYN
            ),
            "пецы"
        );
    }

    #[test]
    fn the_copula_is_a_table() {
        use Number::*;
        use Person::*;
        assert_eq!(
            v("бꙑти", Third, Singular, Tense::Present, Form::Finite, OCS),
            "ѥстъ"
        );
        assert_eq!(
            v("быти", Third, Singular, Tense::Present, Form::Finite, SYN),
            "єсть"
        );
        assert_eq!(
            v("быти", First, Plural, Tense::Present, Form::Finite, SYN),
            "єсмы"
        );
        assert_eq!(
            v("бꙑти", Third, Singular, Tense::Imperfect, Form::Finite, OCS),
            "бѣаше"
        );
        assert_eq!(
            v("быти", Third, Singular, Tense::Imperfect, Form::Finite, SYN),
            "бѧше"
        );
        assert_eq!(
            v("бꙑти", Third, Singular, Tense::Aorist, Form::Finite, OCS),
            "бѣ"
        );
        assert_eq!(
            v("быти", Third, Singular, Tense::Aorist, Form::Finite, SYN),
            "бысть"
        );
        assert_eq!(
            v(
                "быти",
                Second,
                Singular,
                Tense::Present,
                Form::Imperative,
                SYN
            ),
            "бꙋди"
        );
        assert_eq!(
            v(
                "бꙑти",
                Second,
                Plural,
                Tense::Present,
                Form::Imperative,
                OCS
            ),
            "бѫдѣте"
        );
        assert_eq!(
            v(
                "быти",
                Third,
                Singular,
                Tense::Present,
                Form::Infinitive,
                SYN
            ),
            "быти"
        );
    }
}
