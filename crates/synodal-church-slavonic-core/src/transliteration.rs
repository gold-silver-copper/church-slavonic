use crate::{Loss, Romanization, SynodalWord};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum TransliterationScheme {
    Scientific,
    AsciiSearch,
}

impl TransliterationScheme {
    pub const ALL: [Self; 2] = [Self::Scientific, Self::AsciiSearch];
}

#[must_use]
pub fn transliterate(word: &SynodalWord, scheme: TransliterationScheme) -> Romanization {
    let mut text = String::new();
    let mut losses = Vec::new();
    for character in word.canonical().chars() {
        let (scientific, ascii, distinction_lost) = mapping(character);
        let output = match scheme {
            TransliterationScheme::Scientific => scientific,
            TransliterationScheme::AsciiSearch => ascii,
        };
        match output {
            Some(output) => {
                text.push_str(output);
                if scheme == TransliterationScheme::AsciiSearch && distinction_lost {
                    losses.push(Loss {
                        kind: "orthographic-distinction".into(),
                        original: character.to_string(),
                        replacement: output.into(),
                    });
                }
            }
            None => {
                losses.push(Loss {
                    kind: "untransliterated-mark".into(),
                    original: character.to_string(),
                    replacement: String::new(),
                });
            }
        }
    }
    Romanization {
        scheme: match scheme {
            TransliterationScheme::Scientific => "scientific-cs-v1",
            TransliterationScheme::AsciiSearch => "ascii-search-v1",
        }
        .into(),
        text,
        losses,
    }
}

fn mapping(character: char) -> (Option<&'static str>, Option<&'static str>, bool) {
    match character {
        'а' => (Some("a"), Some("a"), false),
        'б' => (Some("b"), Some("b"), false),
        'в' => (Some("v"), Some("v"), false),
        'г' => (Some("g"), Some("g"), false),
        'д' => (Some("d"), Some("d"), false),
        'е' => (Some("e"), Some("e"), false),
        'є' => (Some("e"), Some("e"), true),
        'ж' => (Some("ž"), Some("zh"), true),
        'ѕ' => (Some("dz"), Some("dz"), false),
        'з' => (Some("z"), Some("z"), false),
        'и' => (Some("i"), Some("i"), false),
        'і' => (Some("i"), Some("i"), true),
        'ї' => (Some("ï"), Some("i"), true),
        'й' => (Some("j"), Some("j"), false),
        'к' => (Some("k"), Some("k"), false),
        'л' => (Some("l"), Some("l"), false),
        'м' => (Some("m"), Some("m"), false),
        'н' => (Some("n"), Some("n"), false),
        'о' => (Some("o"), Some("o"), false),
        'ѡ' => (Some("ō"), Some("o"), true),
        'ѻ' => (Some("o"), Some("o"), true),
        'п' => (Some("p"), Some("p"), false),
        'р' => (Some("r"), Some("r"), false),
        'с' => (Some("s"), Some("s"), false),
        'т' => (Some("t"), Some("t"), false),
        'у' | 'ꙋ' | 'ᲂ' => (Some("u"), Some("u"), character != 'у'),
        'ф' => (Some("f"), Some("f"), false),
        'х' => (Some("x"), Some("kh"), true),
        'ѿ' => (Some("ot"), Some("ot"), true),
        'ц' => (Some("c"), Some("ts"), true),
        'ч' => (Some("č"), Some("ch"), true),
        'ш' => (Some("š"), Some("sh"), true),
        'щ' => (Some("št"), Some("sht"), true),
        'ъ' => (Some("ŭ"), Some(""), true),
        'ы' => (Some("y"), Some("y"), false),
        'ь' => (Some("ĭ"), Some(""), true),
        'ѣ' => (Some("ě"), Some("e"), true),
        'ю' => (Some("ju"), Some("yu"), true),
        'я' | 'ꙗ' => (Some("ja"), Some("ya"), true),
        'ѧ' => (Some("ę"), Some("ya"), true),
        'ѫ' => (Some("ǫ"), Some("u"), true),
        'ѯ' => (Some("ks"), Some("ks"), false),
        'ѱ' => (Some("ps"), Some("ps"), false),
        'ѳ' => (Some("th"), Some("th"), false),
        'ѵ' => (Some("y"), Some("y"), true),
        '\u{0300}' => (Some("\u{0300}"), Some(""), true),
        '\u{0301}' => (Some("\u{0301}"), Some(""), true),
        '\u{0311}' => (Some("\u{0311}"), Some(""), true),
        '\u{0486}' => (Some("ʾ"), Some(""), true),
        '\u{0483}'..='\u{0489}' => (None, None, true),
        _ => (None, None, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_reports_historical_letter_loss() {
        let word = SynodalWord::parse("вѣра").expect("word");
        let result = transliterate(&word, TransliterationScheme::AsciiSearch);
        assert_eq!(result.text, "vera");
        assert_eq!(result.losses.len(), 1);
    }
}
