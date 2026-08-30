//! Regular verb conjugation.
//!
//! The infinitive's ending picks the conjugation and the two stems the
//! paradigm is built on (the present stem and the infinitive stem):
//! `-ати` first conjugation on a vowel stem (`дѣла-`), `-овати` on `-оу-`/
//! `-ꙋ-`, `-ноути`/`-нꙋти` on `-н-`, consonant `-ти` and velar `-щи` on the
//! bare consonant, `-ити`/`-ѣти`/husher + `-ати` second conjugation. This is
//! an approximation. Where the corpus made a class the majority for its
//! infinitive shape the Synodal rule follows it: `-сати`/`-мати` iotate
//! (`писати` : `пишеши`, `є҆́млетъ`), `-сти` is a dental stem (`вести` :
//! `ведеши`; the `нести` type is tabled), a second-conjugation stem iotates
//! in the first singular and the imperfect (`любити` : `люблю`, `люблѧ́хъ`;
//! `носити` : `ношꙋ`, `ноша́хъ`), the monosyllabic `-ити` (`бити` : `бію`)
//! is a vowel stem, and the reflexive `-сѧ` is carried on the outside of
//! every form. The rest — the other iotating `-ати` stems, the `-мѣти`
//! first-conjugation `-ѣти` verbs — is tabled, as is everything above in
//! Old Church Slavonic, where the rule keeps Polivanova's plain classes. The
//! copula `бꙑти`/`быти` is fully suppletive and handled by
//! [`ChurchSlavonicCore::to_be`].

use crate::ChurchSlavonicCore;
use crate::grammar::*;
use crate::orthography::strip_marks;

/// The three productive present-stem series.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Conj {
    /// First conjugation, consonant stem (`нес-е-ши`).
    Hard,
    /// First conjugation, vowel stem (OCS `дѣла-ѥ-ши`, Synodal `дѣла-е-ши`).
    Vowel,
    /// First conjugation on an iotated present stem with a vowel infinitive
    /// stem (`пиш-е-ши`, `писа-хъ`).
    Iotated,
    /// Second conjugation (`хвал-и-ши`).
    Second,
}

impl Conj {
    /// The canonical class token the tables store in the class cell.
    pub fn token(self) -> &'static str {
        match self {
            Conj::Hard => "hard",
            Conj::Vowel => "vowel",
            Conj::Iotated => "iotated",
            Conj::Second => "second",
        }
    }

    pub fn from_token(token: &str) -> Option<Conj> {
        Some(match token {
            "hard" => Conj::Hard,
            "vowel" => Conj::Vowel,
            "iotated" => Conj::Iotated,
            "second" => Conj::Second,
            _ => return None,
        })
    }

    pub const ALL: [Conj; 4] = [Conj::Hard, Conj::Vowel, Conj::Iotated, Conj::Second];
}

pub(crate) struct Stems {
    pub(crate) conj: Conj,
    /// The stem the present endings attach to.
    pub(crate) present: String,
    /// The stem the aorist, past participle and (in OCS) imperfect attach to.
    pub(crate) infinitive: String,
}

impl ChurchSlavonicCore {
    /// Conjugate a verb by rule. The lemma is accented in Synodal (see
    /// [`crate::accent`]).
    pub fn verb(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
    ) -> String {
        Self::verb_pattern(word, person, number, tense, form, recension, None)
    }

    /// [`Self::verb`] with the row's accent-pattern token steering the
    /// stress (see [`crate::accent::with_accent_pattern`]) — the resolution
    /// engine's fallback path.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn verb_pattern(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
        pattern: Option<&str>,
    ) -> String {
        use crate::accent::with_accent_pattern;
        let skeleton = strip_marks(word);
        if (skeleton == "бꙑти" || skeleton == "быти")
            && let Some(form) = Self::to_be(person, number, tense, form, recension)
        {
            return form.to_string();
        }
        let reflexive = *recension == Recension::Synodal && skeleton.ends_with("сѧ");
        if reflexive {
            let bare: String = word.chars().take(word.chars().count() - 2).collect();
            return with_accent_pattern(&bare, recension, pattern, |w| {
                // The jer drops before the enclitic (`моли́тсѧ`, `моли́хсѧ`).
                let answer = Self::verb_skeleton(w, person, number, tense, form, recension);
                format!("{}сѧ", answer.strip_suffix('ъ').unwrap_or(&answer))
            });
        }
        with_accent_pattern(word, recension, pattern, |w| {
            Self::verb_skeleton(w, person, number, tense, form, recension)
        })
    }

    /// Conjugate with an explicit class/present-stem override — the runtime
    /// path for the tables' class cells, and the extractor's validation
    /// path. The suppletive [`Self::irregular`] layer and the copula answer
    /// BEFORE the override; the infinitive stem always comes from the
    /// lemma. The word and the answer are skeleton-level (no accent pass):
    /// the stored stem carries its own accent.
    #[allow(clippy::too_many_arguments)]
    pub fn verb_from_stems(
        word: &str,
        class: Option<&str>,
        present: Option<&str>,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
    ) -> String {
        let skeleton = strip_marks(word);
        if (skeleton == "бꙑти" || skeleton == "быти")
            && let Some(answer) = Self::to_be(person, number, tense, form, recension)
        {
            return answer.to_string();
        }
        if let Some(answer) = Self::irregular(&skeleton, person, number, tense, form, recension) {
            return answer;
        }
        if *form == Form::Infinitive {
            return word.to_string();
        }
        let s = Self::override_stems(word, class, present, recension);
        let answer = Self::conjugate(&skeleton, &s, person, number, tense, form, recension);
        if *recension == Recension::Synodal {
            crate::accent::final_varia(&answer)
        } else {
            answer
        }
    }

    /// The lemma's rule stems with the class and/or present stem replaced.
    pub(crate) fn override_stems(
        word: &str,
        class: Option<&str>,
        present: Option<&str>,
        recension: &Recension,
    ) -> Stems {
        // Class detection reads the infinitive's ending letters, so it runs
        // on the unaccented skeleton; the override stem keeps its accent.
        let mut s = Self::stems(&strip_marks(word), recension);
        if let Some(conj) = class.and_then(Conj::from_token) {
            s.conj = conj;
        }
        if let Some(present) = present.filter(|p| !p.is_empty()) {
            s.present = present.to_string();
        }
        s
    }

    fn verb_skeleton(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
    ) -> String {
        if let Some(answer) = Self::irregular(word, person, number, tense, form, recension) {
            return answer;
        }
        let s = Self::stems(word, recension);
        Self::conjugate(word, &s, person, number, tense, form, recension)
    }

    /// The regular route on explicit stems — the tail of [`verb_skeleton`]
    /// shared with the class/present-stem override path.
    pub(crate) fn conjugate(
        word: &str,
        s: &Stems,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
    ) -> String {
        let synodal = *recension == Recension::Synodal;
        let cell = Self::person_cell(person, number);
        match (tense, form) {
            (_, Form::Infinitive) => word.to_string(),
            (Tense::Present, Form::Finite) => {
                let row = match (s.conj, synodal) {
                    (Conj::Hard | Conj::Iotated, false) => &PRESENT_HARD.0,
                    (Conj::Hard | Conj::Iotated, true) => &PRESENT_HARD.1,
                    (Conj::Vowel, false) => &PRESENT_VOWEL.0,
                    (Conj::Vowel, true) => &PRESENT_VOWEL.1,
                    (Conj::Second, false) => &PRESENT_SECOND.0,
                    (Conj::Second, true) => &PRESENT_SECOND.1,
                };
                // The second conjugation iotates its first singular in
                // both recensions (`виждѫ`/`вижду`, `люблѭ`/`люблю`).
                if s.conj == Conj::Second && cell == 0 {
                    return Self::attach(&iotate(&s.present), row[cell], recension);
                }
                Self::attach(&s.present, row[cell], recension)
            }
            (Tense::Imperfect, Form::Finite) => {
                let (stem, marker) = Self::imperfect_stem(s, recension);
                let row = if synodal { &IMPERFECT.1 } else { &IMPERFECT.0 };
                Self::attach(&stem, &format!("{marker}{}", row[cell]), recension)
            }
            (Tense::Aorist, Form::Finite) => {
                // The ox grades belong to consonant stems; a vowel-final
                // infinitive stem (`забꙑ-`, `обꙑкнѫ-`) takes the sigmatic
                // row whatever its present class.
                let hard = s.conj == Conj::Hard
                    && !s
                        .infinitive
                        .chars()
                        .last()
                        .is_some_and(crate::orthography::is_vowel);
                let row = match (hard, synodal) {
                    (true, false) => &AORIST_OX.0,
                    (true, true) => &AORIST_OX.1,
                    (false, false) => &AORIST_SIGMATIC.0,
                    (false, true) => &AORIST_SIGMATIC.1,
                };
                // The `-ѧти`/`-ѩти` nasal stems close their 2/3 singular
                // with `-тъ` in OCS: `начѧтъ`, `приѩтъ`.
                if !synodal
                    && row[cell].is_empty()
                    && matches!(s.infinitive.chars().last(), Some('ѧ' | 'ѩ'))
                {
                    return format!("{}тъ", s.infinitive);
                }
                Self::attach(&s.infinitive, row[cell], recension)
            }
            (Tense::Present, Form::Participle) => {
                // verb:present-active-nominative-contraction: OCS -ꙑ/-ѩ
                // against Synodal -ый/-ѧ.
                let ending = match (s.conj, synodal) {
                    (Conj::Hard, false) => "ꙑ",
                    (Conj::Hard, true) => "ый",
                    (Conj::Vowel, false) => "ѩ",
                    (Conj::Vowel, true) | (Conj::Second | Conj::Iotated, _) => "ѧ",
                };
                Self::attach(&s.present, ending, recension)
            }
            (Tense::Aorist | Tense::Imperfect, Form::Participle) => {
                // The imperfect has no participle of its own; both pasts
                // resolve to the past active participle.
                if s.conj == Conj::Hard {
                    format!("{}ъ", s.infinitive)
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
                    (Conj::Vowel, false) | (Conj::Second | Conj::Iotated, _) => {
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
    /// (verb:copula-tense-reassignment): the aorist is the `бꙑхъ`/`быхъ`
    /// series in both recensions (the OCS `бѣ` series is the imperfective
    /// aorist, which the treebanks file under the imperfect); the OCS 3sg
    /// keeps the hard `бꙑстъ` against Synodal `бы́сть`.
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
            (Tense::Present, Form::Participle, true) => "сы́й",
            (_, Form::Participle, false) => "бꙑвъ",
            (_, Form::Participle, true) => "бы́въ",
            _ => return None,
        })
    }

    /// The athematic presents (`дати` : `дамь`, `вѣдѣти` : `вѣси`, `имѣти` :
    /// `имаши`) and the suppletive pieces of the `ити` family (`шьдъ`) —
    /// the cells the stem machinery cannot build. Everything else falls
    /// through to the regular route.
    fn irregular(
        word: &str,
        person: &Person,
        number: &Number,
        tense: &Tense,
        form: &Form,
        recension: &Recension,
    ) -> Option<String> {
        let synodal = *recension == Recension::Synodal;
        let cell = Self::person_cell(person, number);
        let prefix = |n: usize| -> String {
            let len = word.chars().count().saturating_sub(n);
            word.chars().take(len).collect()
        };
        if word.ends_with("дати") && !word.ends_with("ждати") && !word.ends_with("гадати")
        {
            // The athematic `дам-`/`даст-` present and the `даждь`
            // imperative; the sigmatic aorist keeps `-ст-` in 2/3 singular.
            let p = prefix(4);
            match (tense, form) {
                (Tense::Present, Form::Finite) => {
                    let row = ATHEMATIC_DA[cell];
                    return Some(format!("{p}{row}"));
                }
                (_, Form::Imperative) => {
                    if *number == Number::Singular {
                        return Some(format!("{p}даждь"));
                    }
                    let cell = match (person, number) {
                        (Person::First, Number::Singular) => 6,
                        (Person::Third, n) => Self::person_cell(&Person::Second, n),
                        _ => cell,
                    };
                    let row = if synodal {
                        &IMPERATIVE_I.1
                    } else {
                        &IMPERATIVE_I.0
                    };
                    return Some(format!("{p}дад{}", row[cell]));
                }
                (Tense::Aorist, Form::Finite)
                    if *number == Number::Singular && *person != Person::First =>
                {
                    return Some(format!("{p}дастъ"));
                }
                _ => return None,
            }
        }
        if (word.ends_with("рещи") || word.ends_with("решти"))
            && (*tense, *form) == (Tense::Aorist, Form::Finite)
            && !synodal
        {
            // The root aorist of `рещи`: `рѣхъ`, `рече`, `рѣшѧ`.
            let p = prefix(if word.ends_with("решти") { 5 } else { 4 });
            return Some(format!("{p}{}", RESTI_AORIST[cell]));
        }
        if word.ends_with("вѣдѣти") {
            let p = prefix(6);
            match (tense, form) {
                (Tense::Present, Form::Finite) => {
                    return Some(format!("{p}{}", ATHEMATIC_VE[cell]));
                }
                (Tense::Present, Form::Participle) if !synodal => {
                    return Some(format!("{p}вѣдꙑ"));
                }
                _ => return None,
            }
        }
        if word.ends_with("имѣти") && (*tense, *form) == (Tense::Present, Form::Finite) {
            // `имамь`, `имаши` — the athematic-shaped present of `имѣти`.
            return Some(format!("{}{}", prefix(5), ATHEMATIC_IMA[cell]));
        }
        if let Some(p) = go_prefix(word)
            && matches!(form, Form::Participle)
            && matches!(tense, Tense::Aorist | Tense::Imperfect)
        {
            // The suppletive past active participle of `ити`: `шьдъ`,
            // `пришьдъ` (`при` + `ити` contracts to `прити`, so the
            // participle's prefix is not always the present stem's).
            let p = match p.as_str() {
                "пр" | "прии" | "при" => "при",
                "вън" | "вьн" | "въни" => "въ",
                "из" | "изи" => "и",
                other => other,
            };
            return Some(format!("{p}ш{}дъ", if synodal { "е" } else { "ь" }));
        }
        None
    }

    pub(crate) fn stems(word: &str, recension: &Recension) -> Stems {
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
        } else if let Some(prefix) = go_prefix(word) {
            // The `ити` family: the present stem `ид-` carries the present,
            // the ox aorist (`идохъ`, `иде`) and the imperfect; the past
            // participle `шьдъ` is suppletive (see `irregular`).
            let id = format!("{prefix}ид");
            mk(Conj::Hard, id.clone(), id)
        } else if word.ends_with("хотѣти") {
            // The mixed conjugation: first-conjugation iotated present
            // (`хощеши`) on the second-conjugation infinitive stem.
            mk(Conj::Iotated, iotate(&stem(3)), stem(2))
        } else if synodal
            && word.ends_with("ити")
            && !stem(3).chars().any(crate::orthography::is_vowel)
        {
            // The monosyllabic `бити`, `пити`, `лити`: `бію`, `біеши`.
            mk(Conj::Vowel, format!("{}і", stem(3)), stem(2))
        } else if synodal && (word.ends_with("сати") || word.ends_with("мати")) && !husher {
            mk(Conj::Iotated, iotate(&stem(3)), stem(2))
        } else if word.ends_with("ити")
            || word.ends_with("ѣти")
            || (word.ends_with("ати") && husher)
        {
            mk(Conj::Second, stem(3), stem(2))
        } else if word.ends_with("ати")
            || word.ends_with("ꙗти")
            || word.ends_with("ѧти")
            || word.ends_with("ѩти")
        {
            mk(Conj::Vowel, stem(2), stem(2))
        } else if synodal && word.ends_with("сти") {
            // The dental stems are the majority of the `-сти` infinitives
            // (`вести` : `ведꙋ`; `нести` : `несꙋ` is tabled).
            mk(Conj::Hard, format!("{}д", stem(3)), format!("{}д", stem(3)))
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
            Conj::Vowel if synodal && s.infinitive.ends_with('и') => (s.present.clone(), "ѧ"),
            Conj::Vowel | Conj::Iotated => (s.infinitive.clone(), if synodal { "" } else { "а" }),
            Conj::Second if s.infinitive.ends_with('ѣ') => {
                if synodal {
                    (s.present.clone(), "ѧ")
                } else {
                    (s.infinitive.clone(), "а")
                }
            }
            Conj::Second if synodal => (iotate(&s.present), "ѧ"),
            Conj::Second => (s.present.clone(), "ꙗа"),
        }
    }
}

/// The prefix of an `ити`-family lemma (`ити`, `прити`, `вънити`,
/// `изити`, `отити`...), or `None` when the word is not one — the head
/// before `-ити` must be a known preverb, so `ходити` or `бити` never
/// match.
fn go_prefix(word: &str) -> Option<String> {
    let head = word.strip_suffix("ити")?;
    const PREVERBS: [&str; 18] = [
        "", "по", "пр", "прии", "при", "вън", "вьн", "въни", "из", "изи", "от", "до", "на", "за",
        "прѣ", "мимо", "съ", "об",
    ];
    PREVERBS.contains(&head).then(|| head.to_string())
}

// The athematic present rows, full forms after the preverb: singular 1 2 3,
// dual 1 2 3, plural 1 2 3.
const ATHEMATIC_DA: Row = [
    "дамь",
    "даси",
    "дастъ",
    "давѣ",
    "даста",
    "дасте",
    "дамъ",
    "дасте",
    "дадѧтъ",
];
const RESTI_AORIST: Row = [
    "рѣхъ",
    "рече",
    "рече",
    "рѣховѣ",
    "рѣста",
    "рѣсте",
    "рѣхомъ",
    "рѣсте",
    "рѣшѧ",
];
const ATHEMATIC_VE: Row = [
    "вѣмь",
    "вѣси",
    "вѣстъ",
    "вѣвѣ",
    "вѣста",
    "вѣсте",
    "вѣмъ",
    "вѣсте",
    "вѣдѧтъ",
];
const ATHEMATIC_IMA: Row = [
    "имамь",
    "имаши",
    "иматъ",
    "имавѣ",
    "имата",
    "имате",
    "имамъ",
    "имате",
    "имѫтъ",
];

/// The Slavonic iotation of a stem-final consonant (before the first
/// singular `-ю` and the Synodal imperfect `-ѧ-`/`-а-`): `т`/`ст`/`ск` ->
/// `щ`, `д`/`зд` -> `жд`, `с` -> `ш`, `з` -> `ж`, `к` -> `ч`, `г` -> `ж`, `х`
/// -> `ш`, a labial takes `л`; the sonorants and the hushers stay.
pub(crate) fn iotate(stem: &str) -> String {
    let mut chars: Vec<char> = stem.chars().collect();
    let Some(last) = chars.pop() else {
        return String::new();
    };
    let prev = chars.last().copied();
    let mutated: &str = match last {
        'т' | 'к' if prev == Some('с') => {
            chars.pop();
            "щ"
        }
        'д' if prev == Some('з') => {
            chars.pop();
            "жд"
        }
        'т' => "щ",
        'д' => "жд",
        'с' => "ш",
        'з' => "ж",
        'к' => "ч",
        'г' => "ж",
        'х' => "ш",
        'б' => "бл",
        'п' => "пл",
        'в' => "вл",
        'м' => "мл",
        'ф' => "фл",
        _ => {
            chars.push(last);
            ""
        }
    };
    let mut out: String = chars.into_iter().collect();
    out.push_str(mutated);
    out
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
const IMPERATIVE_J: Row = ["", "й", "й", "йва", "йта", "йта", "емъ", "йте", "йте"];
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
        "є҆́смь",
        "є҆сѝ",
        "є҆́сть",
        "є҆сва̀",
        "є҆ста̀",
        "є҆ста̀",
        "є҆смы̀",
        "є҆стѐ",
        "сꙋ́ть",
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
        "бѧ́хъ",
        "бѧ́ше",
        "бѧ́ше",
        "бѧ́хова",
        "бѧ́ста",
        "бѧ́ста",
        "бѧ́хомъ",
        "бѧ́сте",
        "бѧ́хꙋ",
    ],
);
const BE_AORIST: (Row, Row) = (
    [
        "бꙑхъ",
        "бꙑстъ",
        "бꙑстъ",
        "бꙑховѣ",
        "бꙑста",
        "бꙑсте",
        "бꙑхомъ",
        "бꙑсте",
        "бꙑшѧ",
    ],
    [
        "бы́хъ",
        "бы́сть",
        "бы́сть",
        "бы́хова",
        "бы́ста",
        "бы́ста",
        "бы́хомъ",
        "бы́сте",
        "бы́ша",
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
    fn irregulars_answer_before_any_override() {
        use Form::*;
        use Number::*;
        use Person::*;
        // A wrong override must not break the athematics, the ити family
        // or the copula.
        assert_eq!(
            ChurchSlavonicCore::verb_from_stems(
                "дати",
                Some("vowel"),
                Some("дава"),
                &First,
                &Singular,
                &Tense::Present,
                &Finite,
                &OCS
            ),
            "дамь"
        );
        assert_eq!(
            ChurchSlavonicCore::verb_from_stems(
                "прити",
                Some("second"),
                Some("прит"),
                &Third,
                &Singular,
                &Tense::Aorist,
                &Participle,
                &OCS
            ),
            "пришьдъ"
        );
        assert_eq!(
            ChurchSlavonicCore::verb_from_stems(
                "бꙑти",
                Some("hard"),
                Some("бꙑва"),
                &Third,
                &Singular,
                &Tense::Present,
                &Finite,
                &OCS
            ),
            "ѥстъ"
        );
        // And the override does steer a regular verb.
        assert_eq!(
            ChurchSlavonicCore::verb_from_stems(
                "глаголати",
                Some("iotated"),
                Some("глагол"),
                &Third,
                &Singular,
                &Tense::Present,
                &Finite,
                &OCS
            ),
            "глаголетъ"
        );
    }

    #[test]
    fn present_series_and_the_dual_conditions() {
        use Form::Finite;
        use Number::*;
        use Person::*;
        use Tense::Present;
        assert_eq!(v("нести", First, Singular, Present, Finite, OCS), "несѫ");
        assert_eq!(v("вести", First, Singular, Present, Finite, SYN), "ведꙋ");
        assert_eq!(v("нести", First, Singular, Present, Finite, OCS), "несѫ");
        assert_eq!(v("носи́ти", First, Singular, Present, Finite, SYN), "ношꙋ̀");
        assert_eq!(v("люби́ти", First, Singular, Present, Finite, SYN), "люблю̀");
        assert_eq!(v("проси́ти", First, Singular, Present, Finite, SYN), "прошꙋ̀");
        assert_eq!(
            v("проси́ти", Second, Singular, Present, Finite, SYN),
            "проси́ши"
        );
        assert_eq!(
            v("писа́ти", Second, Singular, Present, Finite, SYN),
            "пише́ши"
        );
        assert_eq!(
            v("писа́ти", Third, Singular, Tense::Imperfect, Finite, SYN),
            "писа́ше"
        );
        assert_eq!(v("би́ти", Third, Plural, Present, Finite, SYN), "бі́ютъ");
        assert_eq!(
            v("би́ти", Third, Singular, Tense::Imperfect, Finite, SYN),
            "бі́ѧше"
        );
        assert_eq!(
            v("моли́тисѧ", First, Singular, Present, Finite, SYN),
            "молю́сѧ"
        );
        assert_eq!(
            v("моли́тисѧ", Third, Singular, Present, Finite, SYN),
            "моли́тсѧ"
        );
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
        assert_eq!(v("вести", First, Dual, Present, Finite, SYN), "ведева");
        assert_eq!(v("нести", Third, Dual, Present, Finite, OCS), "несете");
        assert_eq!(v("вести", Third, Dual, Present, Finite, SYN), "ведета");
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
            v("вести", First, Singular, Imperfect, Finite, SYN),
            "ведѧхъ"
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
            v("носи́ти", Third, Singular, Imperfect, Finite, SYN),
            "ноша́ше"
        );
        assert_eq!(
            v("люби́ти", First, Singular, Imperfect, Finite, SYN),
            "люблѧ́хъ"
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
            v("вести", Second, Plural, Imperfect, Finite, SYN),
            "ведѧсте"
        );
    }

    #[test]
    fn aorists_participles_and_imperatives() {
        use Number::*;
        use Person::*;
        assert_eq!(
            v("вести", First, Singular, Tense::Aorist, Form::Finite, SYN),
            "ведохъ"
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
                "вести",
                Third,
                Singular,
                Tense::Present,
                Form::Participle,
                SYN
            ),
            "ведый"
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
            "хваливъ"
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
                "вести",
                Second,
                Plural,
                Tense::Present,
                Form::Imperative,
                SYN
            ),
            "ведите"
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
            "є҆́сть"
        );
        assert_eq!(
            v("быти", First, Plural, Tense::Present, Form::Finite, SYN),
            "є҆смы̀"
        );
        assert_eq!(
            v("бꙑти", Third, Singular, Tense::Imperfect, Form::Finite, OCS),
            "бѣаше"
        );
        assert_eq!(
            v("быти", Third, Singular, Tense::Imperfect, Form::Finite, SYN),
            "бѧ́ше"
        );
        assert_eq!(
            v("бꙑти", Third, Singular, Tense::Aorist, Form::Finite, OCS),
            "бꙑстъ"
        );
        assert_eq!(
            v("быти", Third, Singular, Tense::Aorist, Form::Finite, SYN),
            "бы́сть"
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
