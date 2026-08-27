//! Small string helpers shared by the rule modules.

use crate::ChurchSlavonicCore;
use crate::grammar::*;

impl ChurchSlavonicCore {
    /// Exact-word lookup in a `(word, replacement)` list. Unlike
    /// [`ChurchSlavonicCore::iter_replace_last`], this matches the WHOLE word
    /// — used where suffix generalization would be wrong.
    pub fn pair_match(word: &str, list: &[(&str, &str)]) -> Option<String> {
        list.iter()
            .find(|(from, _)| *from == word)
            .map(|(_, to)| to.to_string())
    }

    /// Replace the LAST occurrence of `pattern` with `replacement`, keeping
    /// everything before it. Combined with an `ends_with` check by the caller,
    /// this rewrites a word's suffix while preserving any compound prefix.
    pub fn replace_last_occurrence(input: &str, pattern: &str, replacement: &str) -> String {
        if let Some(last_index) = input.rfind(pattern) {
            let (before_last, _after_last) = input.split_at(last_index);
            format!("{}{}", before_last, replacement)
        } else {
            input.into()
        }
    }

    /// Apply the FIRST suffix-class rule whose suffix matches the end of the
    /// word. Order in `pairs` is load-bearing: more specific suffixes must be
    /// listed before their own suffixes' suffixes, because matching stops at
    /// the first hit. Reordering a list silently changes predictions — and
    /// therefore what the extractor tables — so any edit here requires a
    /// regeneration (see the crate docs).
    pub fn iter_replace_last(word: &str, pairs: &[(&str, &str)]) -> Option<String> {
        for (from, to) in pairs {
            if word.ends_with(from) {
                return Some(ChurchSlavonicCore::replace_last_occurrence(word, from, to));
            }
        }
        None
    }

    /// Attach an ending to a stem, applying the sandhi both recensions share
    /// at the seam — the one place the rules touch the stem:
    /// - a stem-final velar palatalizes before a front ending: `к/г/х` ->
    ///   `ц/ѕ(з)/с` before `и`/`ѣ` (`врагъ` -> `вразѣ`, `рѫка` -> `рѫцѣ`),
    ///   `ч/ж/ш` before `е` (`боже`, `печеши`); Synodal writes `ы` after the
    ///   new `ц` (`ѹченицы`) and `и` for `ы` after a velar (`рꙋки`);
    /// - a stem-final husher (`ж ч ш щ ц`) de-iotates the ending's first
    ///   vowel (OCS `мѫжь` -> `мѫжа`, `мѫжоу`; Synodal `мꙋжа`, `слышꙋ`).
    pub(crate) fn attach(stem: &str, ending: &str, recension: &Recension) -> String {
        let synodal = *recension == Recension::Synodal;
        let mut stem = stem.to_string();
        let mut ending = ending.to_string();
        let last = stem.chars().last().unwrap_or(' ');
        let first = ending.chars().next().unwrap_or(' ');
        if matches!(last, 'к' | 'г' | 'х') {
            let (second, first_pal) = match last {
                'к' => ("ц", "ч"),
                'г' => (if synodal { "з" } else { "ѕ" }, "ж"),
                _ => ("с", "ш"),
            };
            match first {
                'и' | 'ѣ' => {
                    stem.pop();
                    stem.push_str(second);
                    if synodal && first == 'и' && last == 'к' {
                        ending.replace_range(..'и'.len_utf8(), "ы");
                    }
                }
                'е' => {
                    stem.pop();
                    stem.push_str(first_pal);
                }
                'ы' if synodal => ending.replace_range(..'ы'.len_utf8(), "и"),
                _ => {}
            }
        } else if matches!(last, 'ж' | 'ч' | 'ш' | 'щ' | 'ц') {
            let plain = if synodal {
                match first {
                    'ѧ' => Some("а"),
                    'ю' => Some("ꙋ"),
                    _ => None,
                }
            } else {
                match first {
                    'ꙗ' => Some("а"),
                    'ѥ' => Some("е"),
                    'ѩ' => Some("ѧ"),
                    'ѭ' => Some("ѫ"),
                    'ю' => Some("оу"),
                    _ => None,
                }
            };
            if let Some(plain) = plain {
                ending.replace_range(..first.len_utf8(), plain);
            }
        }
        stem.push_str(&ending);
        stem
    }

    /// Index of a `(number, case)` cell in a 21-slot paradigm row: the seven
    /// cases in [`Case`] order, singular first, then dual, then plural.
    pub(crate) fn cell(case: &Case, number: &Number) -> usize {
        *number as usize * 7 + *case as usize
    }

    /// Index of a `(person, number)` cell in a 9-slot conjugation row.
    pub(crate) fn person_cell(person: &Person, number: &Number) -> usize {
        *number as usize * 3 + *person as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_palatalizes_velars_and_de_iotates_after_hushers() {
        let ocs = Recension::OldChurchSlavonic;
        let syn = Recension::Synodal;
        assert_eq!(ChurchSlavonicCore::attach("враг", "ѣ", &ocs), "враѕѣ");
        assert_eq!(ChurchSlavonicCore::attach("враг", "ѣ", &syn), "вразѣ");
        assert_eq!(ChurchSlavonicCore::attach("бог", "е", &syn), "боже");
        assert_eq!(ChurchSlavonicCore::attach("ѹченик", "и", &syn), "ѹченицы");
        assert_eq!(ChurchSlavonicCore::attach("оученик", "и", &ocs), "оученици");
        assert_eq!(ChurchSlavonicCore::attach("рꙋк", "ы", &syn), "рꙋки");
        assert_eq!(ChurchSlavonicCore::attach("рѫк", "ꙑ", &ocs), "рѫкꙑ");
        assert_eq!(ChurchSlavonicCore::attach("мѫж", "ꙗ", &ocs), "мѫжа");
        assert_eq!(ChurchSlavonicCore::attach("мѫж", "ю", &ocs), "мѫжоу");
        assert_eq!(ChurchSlavonicCore::attach("слꙑш", "ѭ", &ocs), "слꙑшѫ");
        assert_eq!(ChurchSlavonicCore::attach("мꙋж", "ѧ", &syn), "мꙋжа");
        assert_eq!(ChurchSlavonicCore::attach("слыш", "ю", &syn), "слышꙋ");
        assert_eq!(ChurchSlavonicCore::attach("кон", "ꙗ", &ocs), "конꙗ");
    }
}
