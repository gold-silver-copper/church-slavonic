//! The Synodal non-personal pronouns: the print's closed lexicon, spelled
//! cell by cell in its typography (v1.2 part 2). The tables follow the
//! Alypy grammar's paradigms — §47 (то́й, мо́й), §48 (кто̀/что̀, кі́й,
//! на́шъ, ѻ҆́въ) — with the Bible's and Polyakov's primaries deciding where
//! the grammar prints alternatives (та̀ over та́ѧ, чесѡ̀ over чегѡ̀).
//! Every form carries its own stress: the monosyllabic stems (то́й, се́й,
//! ве́сь, мо́й/тво́й/сво́й, кто̀/что̀) stress their endings, the rest keep
//! the citation form's stress; the print's plural marks — the kamora
//! (всѧ̑, мои̑мъ, на̑ша), the wide letters (тѡ́ю, моє́ю) and, on the
//! relative's monosyllables, the varia (и҆̀же, ꙗ҆̀же, и҆̀хже, и҆̀мже) — are
//! written where the print writes them. The relative и҆́же is the
//! third-person row plus же; the `ни-`/`нѣ-` compounds and the же/жде/ждо
//! enclitics strip, decline and re-wrap (никто́же, нѣ́кій, то́йже, кі́йждо).
//! Anything outside the lexicon answers the empty string: the tables own
//! it (кі́йждо's stress shifts, и҆́нъ's mobile obliques, all the
//! `таковы́й`-type adjectival pronouns).

use crate::ChurchSlavonicCore;
use crate::grammar::*;
use crate::orthography::{realise, strip_marks};
use unicode_normalization::UnicodeNormalization;

const ACUTE: char = '\u{301}';
const GRAVE: char = '\u{300}';
const KAMORA: char = '\u{311}';
const PSILI: char = '\u{486}';

/// The 54 cells of a lemma in schema order: `(gender * 3 + number) * 6 +
/// case` over M/F/N × sg/du/pl × nom/gen/dat/acc/ins/loc.
type Cells = [&'static str; 54];

const TOJ: Cells = [
    "то́й", "тогѡ̀", "томꙋ̀", "то́й", "тѣ́мъ", "то́мъ",
    "та̑", "тѡ́ю", "тѣ́ма", "та̑", "тѣ́ма", "тѡ́ю",
    "ті́и", "тѣ́хъ", "тѣ̑мъ", "ты̑ѧ", "тѣ́ми", "тѣ́хъ",
    "та̀", "тоѧ̀", "то́й", "тꙋ̀", "то́ю", "то́й",
    "тѣ̀", "тѡ́ю", "тѣ́ма", "тѣ̀", "тѣ́ма", "тѡ́ю",
    "ты̑ѧ", "тѣ́хъ", "тѣ̑мъ", "ты̑ѧ", "тѣ́ми", "тѣ́хъ",
    "то̀", "тогѡ̀", "томꙋ̀", "то̀", "тѣ́мъ", "то́мъ",
    "тѣ̀", "тѡ́ю", "тѣ́ма", "тѣ̀", "тѣ́ма", "тѡ́ю",
    "та̑", "тѣ́хъ", "тѣ̑мъ", "та̑", "тѣ́ми", "тѣ́хъ",
];

const SEJ: Cells = [
    "се́й", "сегѡ̀", "семꙋ̀", "се́й", "си́мъ", "се́мъ",
    "сіѧ̑", "се́ю", "си́ма", "сіѧ̑", "си́ма", "се́ю",
    "сі́и", "си́хъ", "си̑мъ", "сіѧ̑", "си́ми", "си́хъ",
    "сіѧ̀", "сеѧ̀", "се́й", "сію̀", "се́ю", "се́й",
    "сі́и", "се́ю", "си́ма", "сі́и", "си́ма", "се́ю",
    "сіѧ̑", "си́хъ", "си̑мъ", "сіѧ̑", "си́ми", "си́хъ",
    "сіѐ", "сегѡ̀", "семꙋ̀", "сіѐ", "си́мъ", "се́мъ",
    "сі́и", "се́ю", "си́ма", "сі́и", "си́ма", "се́ю",
    "сіѧ̑", "си́хъ", "си̑мъ", "сіѧ̑", "си́ми", "си́хъ",
];

const VES: Cells = [
    "ве́сь", "всегѡ̀", "всемꙋ̀", "ве́сь", "всѣ́мъ", "все́мъ",
    "всѧ̑", "всѣ́ю", "всѣ́ма", "всѧ̑", "всѣ́ма", "всѣ́ю",
    "всѝ", "всѣ́хъ", "всѣ̑мъ", "всѧ̑", "всѣ́ми", "всѣ́хъ",
    "всѧ̀", "всеѧ̀", "все́й", "всю̀", "все́ю", "все́й",
    "всѝ", "всѣ́ю", "всѣ́ма", "всѝ", "всѣ́ма", "всѣ́ю",
    "всѧ̑", "всѣ́хъ", "всѣ̑мъ", "всѧ̑", "всѣ́ми", "всѣ́хъ",
    "всѐ", "всегѡ̀", "всемꙋ̀", "всѐ", "всѣ́мъ", "все́мъ",
    "всѝ", "всѣ́ю", "всѣ́ма", "всѝ", "всѣ́ма", "всѣ́ю",
    "всѧ̑", "всѣ́хъ", "всѣ̑мъ", "всѧ̑", "всѣ́ми", "всѣ́хъ",
];

const KIJ: Cells = [
    "кі́й", "ко́егѡ", "ко́емꙋ", "кі́й", "кі́имъ", "ко́емъ",
    "ка̑ѧ", "ко́єю", "кі́има", "ка̑ѧ", "кі́има", "ко́єю",
    "кі́и", "кі́ихъ", "кі̑имъ", "кі́ѧ", "кі́ими", "кі́ихъ",
    "ка́ѧ", "коеѧ̀", "ко́ей", "кꙋ́ю", "ко́ею", "ко́ей",
    "кі́и", "ко́єю", "кі́има", "кі́и", "кі́има", "ко́єю",
    "кі́ѧ", "кі́ихъ", "кі̑имъ", "кі́ѧ", "кі́ими", "кі́ихъ",
    "ко́е", "ко́егѡ", "ко́емꙋ", "ко́е", "кі́имъ", "ко́емъ",
    "кі́и", "ко́єю", "кі́има", "кі́и", "кі́има", "ко́єю",
    "ка̑ѧ", "кі́ихъ", "кі̑имъ", "ка̑ѧ", "кі́ими", "кі́ихъ",
];

/// The singular-only interrogatives: every gender and number answers the
/// same six forms (the genitive/dative of что̀ are the print's frequent
/// чесѡ̀/чесомꙋ̀; the grammar's чегѡ̀/чемꙋ̀ are stored variants).
const KTO: [&str; 6] = ["кто̀", "когѡ̀", "комꙋ̀", "кого̀", "ки́мъ", "ко́мъ"];
const CHTO: [&str; 6] = ["что̀", "чесѡ̀", "чесомꙋ̀", "что̀", "чи́мъ", "че́мъ"];

/// The ending-stressed soft possessives (мо́й, тво́й, сво́й, чі́й): the
/// stem is the lemma without its `й`, unaccented; the endings carry the
/// stress and the plural marks.
const SOFT_ENDINGS: Cells = [
    "й", "егѡ̀", "емꙋ̀", "й", "и́мъ", "е́мъ",
    "ѧ̑", "є́ю", "и́ма", "ѧ̑", "и́ма", "є́ю",
    "ѝ", "и́хъ", "и̑мъ", "ѧ̑", "и́ми", "и́хъ",
    "ѧ̀", "еѧ̀", "е́й", "ю̀", "е́ю", "е́й",
    "ѝ", "є́ю", "и́ма", "ѝ", "и́ма", "є́ю",
    "ѧ̑", "и́хъ", "и̑мъ", "ѧ̑", "и́ми", "и́хъ",
    "ѐ", "егѡ̀", "емꙋ̀", "ѐ", "и́мъ", "е́мъ",
    "ѝ", "є́ю", "и́ма", "ѝ", "и́ма", "є́ю",
    "ѧ̑", "и́хъ", "и̑мъ", "ѧ̑", "и́ми", "и́хъ",
];

/// The stem-stressed soft possessives (на́шъ, ва́шъ): the accented stem is
/// the lemma without its `ъ`; a `^` ending asks for the kamora on the
/// stem (на̑ша, на̑шею), the dual/plural mark of a form spelled like a
/// singular.
const SOFT_STEM_ENDINGS: Cells = [
    "ъ", "егѡ", "емꙋ", "ъ", "имъ", "емъ",
    "^а", "^ею", "има", "^а", "има", "^ею",
    "и", "ихъ", "ымъ", "ѧ", "ими", "ихъ",
    "а", "еѧ", "ей", "ꙋ", "ею", "ей",
    "и", "^ею", "има", "и", "има", "^ею",
    "ѧ", "ихъ", "ымъ", "ѧ", "ими", "ихъ",
    "е", "егѡ", "емꙋ", "е", "имъ", "емъ",
    "и", "^ею", "има", "и", "има", "^ею",
    "^а", "ихъ", "ымъ", "^а", "ими", "ихъ",
];

/// The stem-stressed hard pronominals (ѻ҆́въ, ѻ҆́нъ, є҆ди́нъ, всѧ́къ,
/// толи́къ, є҆ли́къ, коли́къ, и҆́нъ, са́мъ): Alypy §48.4's ѻ҆́въ column. A
/// velar stem softens before `ѣ` and in the masculine plural nominative
/// (всѧ́цѣмъ, є҆ли́цы). The mobile obliques of и҆́нъ and са́мъ (ино́гѡ,
/// самомꙋ̀) are the tables'.
const HARD_ENDINGS: Cells = [
    "ъ", "огѡ", "омꙋ", "ъ", "ѣмъ", "омъ",
    "^а", "ѡю", "ѣма", "^а", "ѣма", "ѡю",
    "и", "ѣхъ", "^ѣмъ", "^ы", "ѣми", "ѣхъ",
    "а", "оѧ", "ой", "ꙋ", "ою", "ой",
    "ѣ", "ѡю", "ѣма", "ѣ", "ѣма", "ѡю",
    "^ы", "ѣхъ", "^ѣмъ", "^ы", "ѣми", "ѣхъ",
    "о", "огѡ", "омꙋ", "о", "ѣмъ", "омъ",
    "ѣ", "ѡю", "ѣма", "ѣ", "ѣма", "ѡю",
    "^а", "ѣхъ", "^ѣмъ", "^а", "ѣми", "ѣхъ",
];

const HARD_LEMMAS: [&str; 9] = [
    "ѻ҆́въ", "ѻ҆́нъ", "є҆ди́нъ", "всѧ́къ", "толи́къ", "є҆ли́къ", "коли́къ", "и҆́нъ", "са́мъ",
];

fn cell(gender: &Gender, number: &Number, case: &Case) -> usize {
    (*gender as usize * 3 + *number as usize) * 6 + *case as usize
}

/// Move a stressed vowel's oxia/varia to the kamora (the print's plural
/// mark on a stem-stressed word: на̑ша, єди̑на).
fn kamora(word: &str) -> String {
    word.nfd()
        .map(|c| if c == ACUTE || c == GRAVE { KAMORA } else { c })
        .collect::<String>()
        .nfc()
        .collect()
}

/// A word-final varia becomes the oxia when an enclitic follows (то̀же →
/// то́же); the relative's plural forms keep their varia by their own table.
fn before_enclitic(host: &str) -> String {
    let mut chars: Vec<char> = host.nfd().collect();
    if let Some(last) = chars.iter().rposition(|c| *c == GRAVE) {
        chars[last] = ACUTE;
    }
    chars.into_iter().collect::<String>().nfc().collect()
}

/// Attach a solid enclitic: the host's final jer is dropped (тѣ́мъ + же =
/// тѣ́мже, кі́й + ждо = кі́йждо).
fn attach(host: &str, enclitic: &str) -> String {
    let host = host.strip_suffix('ъ').unwrap_or(host);
    format!("{host}{enclitic}")
}

/// The relative и҆́же: the third-person row (Alypy §47) plus же, with the
/// nominatives the anaphor keeps (и҆́же, ꙗ҆́же, є҆́же; the plural и҆̀же /
/// ꙗ҆̀же) and the print's plural varia on the monosyllables.
fn relative(gender: &Gender, number: &Number, case: &Case) -> String {
    use Case::*;
    use Gender::*;
    use Number::*;
    let fixed = match (number, gender, case) {
        (Singular, Masculine, Nominative) => Some("и҆́же"),
        (Singular, Feminine, Nominative) => Some("ꙗ҆́же"),
        (Singular, Neuter, Nominative) => Some("є҆́же"),
        (Dual, Masculine, Nominative | Accusative) => Some("ꙗ҆̀же"),
        (Dual, _, Nominative | Accusative) => Some("и҆̀же"),
        (Plural, Masculine, Nominative) => Some("и҆̀же"),
        (Plural, _, Nominative) => Some("ꙗ҆̀же"),
        // the plural accusative is the short «ꙗ҆̀же» (the Bible: 1,214
        // against 268 «и҆̀хже», which the tables keep as the variant)
        (Plural, _, Accusative) => Some("ꙗ҆̀же"),
        (Plural, _, Dative) => Some("и҆̀мже"),
        _ => None,
    };
    if let Some(form) = fixed {
        return form.to_string();
    }
    let host = ChurchSlavonicCore::pronoun(&Person::Third, number, gender, case, &Recension::Synodal);
    attach(&before_enclitic(host), "же")
}

/// Decline a Synodal non-personal pronoun; the empty string outside the
/// lexicon.
pub(crate) fn npron_synodal(lemma: &str, gender: &Gender, number: &Number, case: &Case) -> String {
    let lemma: String = lemma.nfc().collect();
    if lemma == "и҆́же" {
        return relative(gender, number, case);
    }
    // The negative and indefinite prefixes: ни- keeps the base's stress
    // (никто́же), the accented нѣ́- takes it (нѣ́кто, нѣ́кій).
    for (prefix, accented) in [("нѣ́", true), ("ни", false), ("нѣ", false)] {
        if let Some(rest) = lemma.strip_prefix(prefix)
            && !rest.is_empty()
        {
            // The base is spelled as a word of its own inside the lexicon
            // (є҆ди́нъ with its psili and wide letter); inside the compound
            // it loses both (ниеди́нъ).
            let rest = realise(rest, &Recension::Synodal);
            let base = if accented { find_unaccented(&rest) } else { Some(rest) };
            let Some(base) = base else { return String::new() };
            let inner = npron_synodal(&base, gender, number, case);
            if inner.is_empty() {
                return String::new();
            }
            let inner: String = inner.nfd().filter(|c| *c != PSILI).collect::<String>().nfc().collect();
            let inner = match inner.strip_prefix('є') {
                Some(rest) => format!("е{rest}"),
                None => match inner.strip_prefix('ѻ') {
                    Some(rest) => format!("о{rest}"),
                    None => inner,
                },
            };
            let inner = if accented { strip_marks(&inner) } else { inner };
            return realise(&format!("{prefix}{inner}"), &Recension::Synodal);
        }
    }
    // The solid enclitics: strip, decline, re-attach.
    for enclitic in ["ждо", "жде", "же"] {
        if let Some(host) = lemma.strip_suffix(enclitic)
            && !host.is_empty()
        {
            // The citation form stresses the host as the enclitic's
            // neighbour (кто́же); the lexicon spells it alone (кто̀).
            let host = realise(host, &Recension::Synodal);
            let inner = npron_synodal(&host, gender, number, case);
            if inner.is_empty() {
                return String::new();
            }
            return realise(&attach(&before_enclitic(&inner), enclitic), &Recension::Synodal);
        }
    }
    let i = cell(gender, number, case);
    match lemma.as_str() {
        "то́й" => return TOJ[i].to_string(),
        "се́й" => return SEJ[i].to_string(),
        "ве́сь" => return VES[i].to_string(),
        "кі́й" => return KIJ[i].to_string(),
        "кто̀" => return KTO[*case as usize].to_string(),
        "что̀" => return CHTO[*case as usize].to_string(),
        _ => {}
    }
    if let Some(stem) = lemma.strip_suffix('й')
        && matches!(lemma.as_str(), "мо́й" | "тво́й" | "сво́й" | "чі́й")
    {
        let ending = SOFT_ENDINGS[i];
        if ending == "й" {
            return lemma.to_string();
        }
        return format!("{}{ending}", strip_marks(stem));
    }
    if let Some(stem) = lemma.strip_suffix('ъ')
        && matches!(lemma.as_str(), "на́шъ" | "ва́шъ")
    {
        return with_stem(stem, SOFT_STEM_ENDINGS[i]);
    }
    if HARD_LEMMAS.contains(&lemma.as_str())
        && let Some(stem) = lemma.strip_suffix('ъ')
    {
        let ending = HARD_ENDINGS[i];
        let velar = stem.ends_with('к');
        let (stem, ending): (String, &str) = if velar && ending.trim_start_matches('^').starts_with('ѣ') {
            (format!("{}ц", stem.strip_suffix('к').unwrap_or(stem)), ending)
        } else if velar && ending == "и" {
            (format!("{}ц", stem.strip_suffix('к').unwrap_or(stem)), "ы")
        } else {
            (stem.to_string(), ending)
        };
        return with_stem(&stem, ending);
    }
    String::new()
}

/// A stem-stressed form: the accented stem plus a plain ending, the `^`
/// marker moving the stress mark to the kamora.
fn with_stem(stem: &str, ending: &str) -> String {
    match ending.strip_prefix('^') {
        Some(ending) => format!("{}{ending}", kamora(stem)),
        None => format!("{stem}{ending}"),
    }
}

/// The lexicon entry whose accent-stripped citation form is `rest` (the
/// base of an accented нѣ́- compound: нѣ́кто → кто̀, нѣ́кій → кі́й).
fn find_unaccented(rest: &str) -> Option<String> {
    ["кто̀", "что̀", "кі́й", "то́й", "се́й"]
        .into_iter()
        .chain(HARD_LEMMAS)
        .find(|l| strip_marks(l) == strip_marks(rest))
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orthography::realise;

    fn p(lemma: &str, g: Gender, n: Number, c: Case) -> String {
        npron_synodal(lemma, &g, &n, &c)
    }

    #[test]
    fn the_print_paradigms() {
        use Case::*;
        use Gender::*;
        use Number::*;
        assert_eq!(p("то́й", Masculine, Singular, Genitive), "тогѡ̀");
        assert_eq!(p("то́й", Masculine, Plural, Nominative), "ті́и");
        assert_eq!(p("то́й", Feminine, Singular, Nominative), "та̀");
        assert_eq!(p("то́й", Neuter, Plural, Dative), "тѣ̑мъ");
        assert_eq!(p("се́й", Neuter, Singular, Nominative), "сіѐ");
        assert_eq!(p("се́й", Masculine, Plural, Accusative), "сіѧ̑");
        assert_eq!(p("ве́сь", Masculine, Plural, Nominative), "всѝ");
        assert_eq!(p("ве́сь", Feminine, Plural, Accusative), "всѧ̑");
        assert_eq!(p("ве́сь", Masculine, Singular, Genitive), "всегѡ̀");
        assert_eq!(p("мо́й", Masculine, Singular, Genitive), "моегѡ̀");
        assert_eq!(p("тво́й", Feminine, Plural, Nominative), "твоѧ̑");
        assert_eq!(p("сво́й", Masculine, Plural, Dative), "свои̑мъ");
        assert_eq!(p("сво́й", Masculine, Dual, Genitive), "своє́ю");
        assert_eq!(p("на́шъ", Masculine, Singular, Genitive), "на́шегѡ");
        assert_eq!(p("на́шъ", Masculine, Dual, Nominative), "на̑ша");
        assert_eq!(p("на́шъ", Masculine, Plural, Dative), "на́шымъ");
        assert_eq!(p("кі́й", Feminine, Singular, Nominative), "ка́ѧ");
        assert_eq!(p("кто̀", Feminine, Plural, Accusative), "кого̀");
        assert_eq!(p("что̀", Masculine, Singular, Genitive), "чесѡ̀");
        assert_eq!(p("ѻ҆́въ", Masculine, Singular, Genitive), "ѻ҆́вогѡ");
        assert_eq!(p("ѻ҆́въ", Masculine, Dual, Genitive), "ѻ҆́вѡю");
        assert_eq!(p("є҆ди́нъ", Masculine, Dual, Nominative), "є҆ди̑на");
        assert_eq!(p("всѧ́къ", Masculine, Singular, Instrumental), "всѧ́цѣмъ");
        assert_eq!(p("є҆ли́къ", Masculine, Plural, Nominative), "є҆ли́цы");
        assert_eq!(p("всѧ́къ", Feminine, Singular, Accusative), "всѧ́кꙋ");
        // the relative: the third-person row + же, the plural varia kept
        assert_eq!(p("и҆́же", Masculine, Singular, Genitive), "є҆гѡ́же");
        assert_eq!(p("и҆́же", Masculine, Singular, Accusative), "є҆го́же");
        assert_eq!(p("и҆́же", Feminine, Singular, Accusative), "ю҆́же");
        assert_eq!(p("и҆́же", Neuter, Singular, Nominative), "є҆́же");
        assert_eq!(p("и҆́же", Feminine, Singular, Nominative), "ꙗ҆́же");
        assert_eq!(p("и҆́же", Feminine, Plural, Nominative), "ꙗ҆̀же");
        assert_eq!(p("и҆́же", Masculine, Plural, Nominative), "и҆̀же");
        assert_eq!(p("и҆́же", Masculine, Plural, Accusative), "ꙗ҆̀же");
        assert_eq!(p("и҆́же", Masculine, Plural, Genitive), "и҆́хже");
        assert_eq!(p("и҆́же", Masculine, Plural, Dative), "и҆̀мже");
        assert_eq!(p("и҆́же", Masculine, Singular, Instrumental), "и҆́мже");
        assert_eq!(p("и҆́же", Masculine, Singular, Locative), "не́мже");
        assert_eq!(p("и҆́же", Masculine, Plural, Locative), "ни́хже");
        // the compounds
        assert_eq!(p("никто́же", Masculine, Singular, Genitive), "никогѡ́же");
        assert_eq!(p("ничто́же", Masculine, Singular, Genitive), "ничесѡ́же");
        assert_eq!(p("нѣ́кто", Masculine, Singular, Nominative), "нѣ́кто");
        assert_eq!(p("нѣ́кій", Masculine, Singular, Genitive), "нѣ́коегѡ");
        assert_eq!(p("нѣ́кій", Feminine, Singular, Nominative), "нѣ́каѧ");
        assert_eq!(p("то́йже", Masculine, Singular, Instrumental), "тѣ́мже");
        assert_eq!(p("то́йже", Feminine, Singular, Nominative), "та́же");
        assert_eq!(p("то́йже", Neuter, Singular, Nominative), "то́же");
        assert_eq!(p("кі́йждо", Masculine, Singular, Nominative), "кі́йждо");
        assert_eq!(p("кі́йждо", Masculine, Singular, Genitive), "ко́егѡждо");
        assert_eq!(p("никі́йже", Neuter, Singular, Nominative), "нико́еже");
        assert_eq!(p("ниєди́нъ", Masculine, Singular, Nominative), "ниеди́нъ");
        // outside the lexicon
        assert_eq!(p("таковы́й", Masculine, Singular, Genitive), "");
        assert_eq!(p("са́мый", Masculine, Singular, Genitive), "");
    }

    /// The registry conditions of the non-personal pronoun, visible cell by
    /// cell against the OCS rule: npron:demonstrative-citation (сь/се́й,
    /// тъ/то́й), npron:jer-loss (вьсь/ве́сь, къто/кто̀, чьто/что̀),
    /// npron:genitive-letter (-ѥго/-егѡ̀), npron:relative-plural-varia.
    #[test]
    fn the_registry_conditions_are_visible_cell_by_cell() {
        use Case::*;
        use Gender::*;
        use Number::*;
        let ocs = |l: &str, g, n, c| {
            ChurchSlavonicCore::npron(l, &g, &n, &c, &Recension::OldChurchSlavonic)
        };
        assert_eq!(ocs("сь", Masculine, Singular, Genitive), "сего");
        assert_eq!(p("се́й", Masculine, Singular, Genitive), "сегѡ̀");
        assert_eq!(ocs("тъ", Masculine, Singular, Genitive), "того");
        assert_eq!(p("то́й", Masculine, Singular, Genitive), "тогѡ̀");
        assert_eq!(ocs("вьсь", Masculine, Plural, Genitive), "вьсѣхъ");
        assert_eq!(p("ве́сь", Masculine, Plural, Genitive), "всѣ́хъ");
        assert_eq!(ocs("къто", Masculine, Singular, Dative), "комоу");
        assert_eq!(p("кто̀", Masculine, Singular, Dative), "комꙋ̀");
        assert_eq!(ocs("мои", Masculine, Singular, Genitive), "моѥго");
        assert_eq!(p("мо́й", Masculine, Singular, Genitive), "моегѡ̀");
        assert_eq!(ocs("иже", Masculine, Plural, Nominative), "иже");
        assert_eq!(p("и҆́же", Masculine, Plural, Nominative), "и҆̀же");
    }

    #[test]
    fn every_cell_is_the_print_typography() {
        let syn = Recension::Synodal;
        for lemma in ["то́й", "се́й", "ве́сь", "мо́й", "на́шъ", "кі́й", "кто̀", "что̀", "ѻ҆́въ", "всѧ́къ", "и҆́же", "то́йже", "никто́же", "нѣ́кій"] {
            for g in [Gender::Masculine, Gender::Feminine, Gender::Neuter] {
                for n in [Number::Singular, Number::Dual, Number::Plural] {
                    for c in [Case::Nominative, Case::Genitive, Case::Dative, Case::Accusative, Case::Instrumental, Case::Locative] {
                        let form = p(lemma, g, n, c);
                        assert!(!form.is_empty(), "{lemma} {g:?} {n:?} {c:?}");
                        assert_eq!(realise(&form, &syn), form, "{lemma} {g:?} {n:?} {c:?}");
                    }
                }
            }
        }
    }
}
