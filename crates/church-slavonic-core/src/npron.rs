//! The non-personal pronouns: the pronominal declension over a closed
//! lexicon. A hard stem takes the `тъ` endings (`того`, `томоу`, `тѣмь`),
//! a soft stem the `сь`/`мои` endings (`сего`, `моѥго`); `вьсь` mixes the
//! two (soft singular, `вьсѣхъ` hard-vowel plural); `къто` and `чьто`
//! decline in the singular only (every gender/number cell answers the same
//! six forms); the relative `иже` is the anaphoric `и` series with the
//! enclitic `же` on the outside; the `ни-`/`нѣ-` compounds strip the
//! prefix (and a trailing `же`), decline the base, and re-wrap
//! (`никътоже` : `никогоже`). The accusative of the animate interrogative
//! is the genitive (`кого`); everything else keeps the nominative shape —
//! the treebank's non-personal accusatives are overwhelmingly inanimate.
//! Anything outside the lexicon answers the empty string and lives in the
//! tables. The rule is written in OCS letters; the Synodal print's
//! accented rows are stored.

use crate::ChurchSlavonicCore;
use crate::grammar::*;

/// The six case slots of one gender/number row (vocative = nominative).
type Row = [&'static str; 6];

/// A hard pronominal stem's endings, by gender and number.
fn hard(gender: &Gender, number: &Number) -> Row {
    match (number, gender) {
        (Number::Singular, Gender::Masculine) => ["ъ", "ого", "омоу", "ъ", "ѣмь", "омь"],
        (Number::Singular, Gender::Neuter) => ["о", "ого", "омоу", "о", "ѣмь", "омь"],
        (Number::Singular, Gender::Feminine) => ["а", "оѩ", "ои", "ѫ", "оѭ", "ои"],
        (Number::Dual, Gender::Masculine) => ["а", "ою", "ѣма", "а", "ѣма", "ою"],
        (Number::Dual, _) => ["ѣ", "ою", "ѣма", "ѣ", "ѣма", "ою"],
        (Number::Plural, Gender::Masculine) => ["и", "ѣхъ", "ѣмъ", "ꙑ", "ѣми", "ѣхъ"],
        (Number::Plural, Gender::Feminine) => ["ꙑ", "ѣхъ", "ѣмъ", "ꙑ", "ѣми", "ѣхъ"],
        (Number::Plural, Gender::Neuter) => ["а", "ѣхъ", "ѣмъ", "а", "ѣми", "ѣхъ"],
    }
}

/// A soft pronominal stem's endings; the nominative (and the accusative
/// that copies it) is supplied by the caller per lemma.
fn soft(gender: &Gender, number: &Number) -> Row {
    match (number, gender) {
        (Number::Singular, Gender::Masculine | Gender::Neuter) => {
            ["", "ѥго", "ѥмоу", "", "имь", "ѥмь"]
        }
        (Number::Singular, Gender::Feminine) => ["", "ѥѩ", "ѥи", "ѭ", "ѥѭ", "ѥи"],
        (Number::Dual, Gender::Masculine) => ["ꙗ", "ѥю", "има", "ꙗ", "има", "ѥю"],
        (Number::Dual, _) => ["и", "ѥю", "има", "и", "има", "ѥю"],
        (Number::Plural, Gender::Masculine) => ["и", "ихъ", "имъ", "ѩ", "ими", "ихъ"],
        (Number::Plural, Gender::Feminine) => ["ѩ", "ихъ", "имъ", "ѩ", "ими", "ихъ"],
        (Number::Plural, Gender::Neuter) => ["а", "ихъ", "имъ", "а", "ими", "ихъ"],
    }
}

/// The soft nominative-singular series of a lemma: (masculine, feminine,
/// neuter) full forms.
fn soft_nominative(lemma: &str) -> Option<(&'static str, String, String)> {
    let stem = soft_stem(lemma)?;
    Some(match lemma {
        "сь" => ("сь", "си".into(), "се".into()),
        _ if lemma.ends_with('и') => ("", format!("{stem}ꙗ"), format!("{stem}ѥ")),
        _ => ("", format!("{stem}а"), format!("{stem}е")),
    })
}

/// The stem of a soft-declension lemma, or `None` when it is not one.
fn soft_stem(lemma: &str) -> Option<String> {
    match lemma {
        "сь" => Some("с".into()),
        "мои" | "твои" | "свои" => Some(lemma.strip_suffix('и').unwrap().to_string()),
        "нашь" | "вашь" => Some(lemma.strip_suffix('ь').unwrap().to_string()),
        _ => None,
    }
}

/// The stem of a hard-declension lemma, or `None`.
fn hard_stem(lemma: &str) -> Option<&str> {
    match lemma {
        "тъ" | "овъ" | "онъ" | "инъ" | "ѥдинъ" | "единъ" | "самъ" | "толикъ" | "селикъ"
        | "коликъ" | "сиць" => lemma.strip_suffix(['ъ', 'ь']),
        _ => None,
    }
}

/// The anaphoric `и` series (the base of the relative `иже`).
fn anaphor(gender: &Gender, number: &Number) -> Row {
    match (number, gender) {
        (Number::Singular, Gender::Masculine) => ["и", "ѥго", "ѥмоу", "и", "имь", "ѥмь"],
        (Number::Singular, Gender::Neuter) => ["ѥ", "ѥго", "ѥмоу", "ѥ", "имь", "ѥмь"],
        (Number::Singular, Gender::Feminine) => ["ꙗ", "ѥѩ", "ѥи", "ѭ", "ѥѭ", "ѥи"],
        (Number::Dual, Gender::Masculine) => ["ꙗ", "ѥю", "има", "ꙗ", "има", "ѥю"],
        (Number::Dual, _) => ["и", "ѥю", "има", "и", "има", "ѥю"],
        (Number::Plural, Gender::Masculine) => ["и", "ихъ", "имъ", "ѩ", "ими", "ихъ"],
        (Number::Plural, Gender::Feminine) => ["ѩ", "ихъ", "имъ", "ѩ", "ими", "ихъ"],
        (Number::Plural, Gender::Neuter) => ["ꙗ", "ихъ", "имъ", "ꙗ", "ими", "ихъ"],
    }
}

/// The singular-only interrogative rows.
fn interrogative(lemma: &str) -> Option<Row> {
    match lemma {
        "къто" => Some(["къто", "кого", "комоу", "кого", "цѣмь", "комь"]),
        "чьто" => Some(["чьто", "чесо", "чемоу", "чьто", "чимь", "чемь"]),
        _ => None,
    }
}

/// Attach a soft ending: after a consonant-final stem (`наш-`, `вьс-`)
/// the iotated onsets flatten (`ѥ` : `е`, `ꙗ` : `а`, `ѩ` : `ѧ`) —
/// `нашего`, not `нашѥго`; a vowel-final stem (`мо-`) keeps them.
fn attach_soft(stem: &str, ending: &str) -> String {
    let vowel_final = matches!(stem.chars().last(), Some(c) if crate::orthography::is_vowel(c));
    if vowel_final {
        return format!("{stem}{ending}");
    }
    let mut chars = ending.chars();
    let mapped = match chars.next() {
        Some('ѥ') => Some('е'),
        Some('ꙗ') => Some('а'),
        Some('ѩ') => Some('ѧ'),
        other => {
            return match other {
                Some(c) => format!("{stem}{c}{}", chars.as_str()),
                None => stem.to_string(),
            };
        }
    };
    format!("{stem}{}{}", mapped.unwrap(), chars.as_str())
}

impl ChurchSlavonicCore {
    /// Decline a non-personal pronoun by rule; the vocative answers with
    /// the nominative. Returns the empty string for a lemma outside the
    /// rule's closed lexicon — the caller's tables own those.
    pub fn npron(lemma: &str, gender: &Gender, number: &Number, case: &Case, _: &Recension) -> String {
        let case = if *case == Case::Vocative {
            &Case::Nominative
        } else {
            case
        };
        let i = *case as usize;
        // The `ни-`/`нѣ-` compounds: strip the wrap, decline, re-wrap.
        for prefix in ["ни", "нѣ"] {
            if let Some(rest) = lemma.strip_prefix(prefix)
                && !rest.is_empty()
            {
                let base = rest.strip_suffix("же").unwrap_or(rest);
                let suffix = if rest.ends_with("же") { "же" } else { "" };
                let inner = Self::npron(base, gender, number, case, &Recension::OldChurchSlavonic);
                if !inner.is_empty() {
                    return format!("{prefix}{inner}{suffix}");
                }
                return String::new();
            }
        }
        if let Some(row) = interrogative(lemma) {
            return row[i].to_string();
        }
        if lemma == "иже" {
            return format!("{}же", anaphor(gender, number)[i]);
        }
        if lemma == "вьсь" {
            // Soft singular, hard-vowel plural and dual oblique on `вьс-`.
            let row: Row = match (number, gender) {
                (Number::Singular, Gender::Masculine) => {
                    ["вьсь", "вьсего", "вьсемоу", "вьсь", "вьсѣмь", "вьсемь"]
                }
                (Number::Singular, Gender::Neuter) => {
                    ["вьсе", "вьсего", "вьсемоу", "вьсе", "вьсѣмь", "вьсемь"]
                }
                (Number::Singular, Gender::Feminine) => {
                    ["вьсꙗ", "вьсеѩ", "вьсеи", "вьсѭ", "вьсеѭ", "вьсеи"]
                }
                (Number::Dual, Gender::Masculine) => {
                    ["вьсꙗ", "вьсею", "вьсѣма", "вьсꙗ", "вьсѣма", "вьсею"]
                }
                (Number::Dual, _) => ["вьси", "вьсею", "вьсѣма", "вьси", "вьсѣма", "вьсею"],
                (Number::Plural, Gender::Masculine) => {
                    ["вьси", "вьсѣхъ", "вьсѣмъ", "вьсѧ", "вьсѣми", "вьсѣхъ"]
                }
                (Number::Plural, Gender::Feminine) => {
                    ["вьсѧ", "вьсѣхъ", "вьсѣмъ", "вьсѧ", "вьсѣми", "вьсѣхъ"]
                }
                (Number::Plural, Gender::Neuter) => {
                    ["вьса", "вьсѣхъ", "вьсѣмъ", "вьса", "вьсѣми", "вьсѣхъ"]
                }
            };
            return row[i].to_string();
        }
        if let Some(stem) = hard_stem(lemma) {
            let ending = hard(gender, number)[i];
            if ending == "ъ" && *gender == Gender::Masculine && *number == Number::Singular {
                return lemma.to_string();
            }
            return format!("{stem}{ending}");
        }
        if let Some(stem) = soft_stem(lemma) {
            let ending = soft(gender, number)[i];
            if ending.is_empty() {
                // The lemma-shaped nominative/accusative cells.
                let (m, f, n) = soft_nominative(lemma).expect("soft stem has nominatives");
                return match gender {
                    Gender::Masculine => {
                        if m.is_empty() {
                            lemma.to_string()
                        } else {
                            m.to_string()
                        }
                    }
                    Gender::Feminine => f,
                    Gender::Neuter => n,
                };
            }
            return attach_soft(&stem, ending);
        }
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCS: Recension = Recension::OldChurchSlavonic;

    fn p(lemma: &str, g: Gender, n: Number, c: Case) -> String {
        ChurchSlavonicCore::npron(lemma, &g, &n, &c, &OCS)
    }

    #[test]
    fn the_pronominal_declension() {
        use Case::*;
        use Gender::*;
        use Number::*;
        assert_eq!(p("тъ", Masculine, Singular, Genitive), "того");
        assert_eq!(p("тъ", Masculine, Singular, Instrumental), "тѣмь");
        assert_eq!(p("тъ", Feminine, Singular, Accusative), "тѫ");
        assert_eq!(p("тъ", Masculine, Plural, Nominative), "ти");
        assert_eq!(p("сь", Neuter, Singular, Genitive), "сего");
        assert_eq!(p("сь", Masculine, Singular, Nominative), "сь");
        assert_eq!(p("вьсь", Masculine, Plural, Genitive), "вьсѣхъ");
        assert_eq!(p("вьсь", Masculine, Singular, Genitive), "вьсего");
        assert_eq!(p("иже", Masculine, Singular, Genitive), "ѥгоже");
        assert_eq!(p("иже", Masculine, Singular, Nominative), "иже");
        assert_eq!(p("иже", Feminine, Singular, Nominative), "ꙗже");
        assert_eq!(p("къто", Masculine, Singular, Dative), "комоу");
        assert_eq!(p("къто", Masculine, Singular, Accusative), "кого");
        assert_eq!(p("чьто", Masculine, Singular, Genitive), "чесо");
        assert_eq!(p("никътоже", Masculine, Singular, Genitive), "никогоже");
        assert_eq!(p("ничьтоже", Masculine, Singular, Dative), "ничемоуже");
        assert_eq!(p("мои", Masculine, Singular, Genitive), "моѥго");
        assert_eq!(p("мои", Feminine, Singular, Nominative), "моꙗ");
        assert_eq!(p("нашь", Masculine, Singular, Dative), "нашемоу");
        assert_eq!(p("самъ", Masculine, Singular, Dative), "самомоу");
        assert_eq!(p("кꙑи", Masculine, Singular, Genitive), "");
    }
}
