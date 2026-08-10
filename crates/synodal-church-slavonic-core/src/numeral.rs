use crate::{Error, Result, SynodalWord};

const TITLO: char = '\u{0483}';
const THOUSANDS: char = '\u{0482}';
const MAX_REGULAR: u32 = 9_999;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CyrillicNumeral {
    value: u32,
    text: SynodalWord,
}

impl CyrillicNumeral {
    pub fn from_value(value: u32) -> Result<Self> {
        let text = format_cyrillic_numeral(value)?;
        Ok(Self {
            value,
            text: SynodalWord::parse(text)?,
        })
    }

    pub fn parse(value: &str) -> Result<Self> {
        let number = parse_cyrillic_numeral(value)?;
        let canonical = format_cyrillic_numeral(number)?;
        Ok(Self {
            value: number,
            text: SynodalWord::parse(canonical)?,
        })
    }

    #[must_use]
    pub const fn value(&self) -> u32 {
        self.value
    }

    #[must_use]
    pub fn text(&self) -> &str {
        self.text.canonical()
    }
}

pub fn format_cyrillic_numeral(value: u32) -> Result<String> {
    if value == 0 {
        return Err(Error::InvalidNumeral {
            reason: "the traditional letter system has no zero glyph".into(),
        });
    }
    if value == 100_000 {
        return Ok(with_titlo(format!("{THOUSANDS}р")));
    }
    if value == 1_000_000 {
        return Ok(with_titlo(format!("{THOUSANDS}{THOUSANDS}а")));
    }
    if value > MAX_REGULAR {
        return Err(Error::OutOfRange {
            value,
            maximum: MAX_REGULAR,
        });
    }

    let mut letters = String::new();
    let thousands = value / 1_000;
    if thousands > 0 {
        letters.push(THOUSANDS);
        letters.push(unit_letter(thousands).ok_or_else(|| Error::InvalidNumeral {
            reason: "unsupported thousands digit".into(),
        })?);
    }
    letters.push_str(&format_sub_thousand(value % 1_000));
    Ok(with_titlo(letters))
}

pub fn parse_cyrillic_numeral(text: &str) -> Result<u32> {
    let word = SynodalWord::parse(text)?;
    let canonical = word.canonical();
    let titlo_count = canonical
        .chars()
        .filter(|character| *character == TITLO)
        .count();
    if titlo_count != 1 {
        return Err(Error::InvalidNumeral {
            reason: "a canonical numeral requires exactly one titlo".into(),
        });
    }
    let bare: String = canonical
        .chars()
        .filter(|character| *character != TITLO)
        .collect();
    let value = if bare == format!("{THOUSANDS}р") {
        100_000
    } else if bare == format!("{THOUSANDS}{THOUSANDS}а") {
        1_000_000
    } else {
        parse_regular(&bare)?
    };
    if format_cyrillic_numeral(value)? != canonical {
        return Err(Error::InvalidNumeral {
            reason: "letters or titlo are not in canonical Synodal numeral order".into(),
        });
    }
    Ok(value)
}

fn parse_regular(text: &str) -> Result<u32> {
    let mut characters = text.chars().peekable();
    let mut value = 0;
    if characters.peek() == Some(&THOUSANDS) {
        characters.next();
        let digit =
            characters
                .next()
                .and_then(letter_value)
                .ok_or_else(|| Error::InvalidNumeral {
                    reason: "thousands sign must be followed by a numeral unit letter".into(),
                })?;
        if digit > 9 {
            return Err(Error::InvalidNumeral {
                reason: "this version supports one-digit thousands".into(),
            });
        }
        value += digit * 1_000;
    }
    for character in characters {
        value += letter_value(character).ok_or_else(|| Error::InvalidNumeral {
            reason: format!("{character:?} is not a Church Slavonic numeral letter"),
        })?;
    }
    if value == 0 || value > MAX_REGULAR {
        Err(Error::InvalidNumeral {
            reason: "numeral is outside the supported regular range".into(),
        })
    } else {
        Ok(value)
    }
}

fn format_sub_thousand(value: u32) -> String {
    let hundreds = value / 100;
    let remainder = value % 100;
    let mut output = String::new();
    if hundreds > 0 {
        if let Some(letter) = hundred_letter(hundreds) {
            output.push(letter);
        }
    }
    if (11..=19).contains(&remainder) {
        if let Some(letter) = unit_letter(remainder - 10) {
            output.push(letter);
        }
        output.push('і');
    } else {
        let tens = remainder / 10;
        let units = remainder % 10;
        if tens > 0 {
            if let Some(letter) = ten_letter(tens) {
                output.push(letter);
            }
        }
        if units > 0 {
            if let Some(letter) = unit_letter(units) {
                output.push(letter);
            }
        }
    }
    output
}

fn with_titlo(value: String) -> String {
    let mut characters: Vec<char> = value.chars().collect();
    let numeral_positions: Vec<usize> = characters
        .iter()
        .enumerate()
        .filter_map(|(index, character)| letter_value(*character).map(|_| index))
        .collect();
    let target = if numeral_positions.len() == 1 {
        numeral_positions[0]
    } else {
        numeral_positions[numeral_positions.len() - 2]
    };
    characters.insert(target + 1, TITLO);
    characters.into_iter().collect()
}

fn unit_letter(value: u32) -> Option<char> {
    match value {
        1 => Some('а'),
        2 => Some('в'),
        3 => Some('г'),
        4 => Some('д'),
        5 => Some('є'),
        6 => Some('ѕ'),
        7 => Some('з'),
        8 => Some('и'),
        9 => Some('ѳ'),
        _ => None,
    }
}

fn ten_letter(value: u32) -> Option<char> {
    match value {
        1 => Some('і'),
        2 => Some('к'),
        3 => Some('л'),
        4 => Some('м'),
        5 => Some('н'),
        6 => Some('ѯ'),
        7 => Some('ѻ'),
        8 => Some('п'),
        9 => Some('ч'),
        _ => None,
    }
}

fn hundred_letter(value: u32) -> Option<char> {
    match value {
        1 => Some('р'),
        2 => Some('с'),
        3 => Some('т'),
        4 => Some('у'),
        5 => Some('ф'),
        6 => Some('х'),
        7 => Some('ѱ'),
        8 => Some('ѿ'),
        9 => Some('ц'),
        _ => None,
    }
}

fn letter_value(character: char) -> Option<u32> {
    match character {
        'а' => Some(1),
        'в' => Some(2),
        'г' => Some(3),
        'д' => Some(4),
        'є' => Some(5),
        'ѕ' => Some(6),
        'з' => Some(7),
        'и' => Some(8),
        'ѳ' => Some(9),
        'і' => Some(10),
        'к' => Some(20),
        'л' => Some(30),
        'м' => Some(40),
        'н' => Some(50),
        'ѯ' => Some(60),
        'ѻ' => Some(70),
        'п' => Some(80),
        'ч' => Some(90),
        'р' => Some(100),
        'с' => Some(200),
        'т' => Some(300),
        'у' => Some(400),
        'ф' => Some(500),
        'х' => Some(600),
        'ѱ' => Some(700),
        'ѿ' => Some(800),
        'ц' => Some(900),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_alypy_5_examples() {
        for (value, expected) in [
            (15, "є҃і"),
            (21, "к҃а"),
            (1_000, "҂а҃"),
            (1_964, "҂ацѯ҃д"),
            (7_472, "҂зуѻ҃в"),
            (100_000, "҂р҃"),
            (1_000_000, "҂҂а҃"),
        ] {
            assert_eq!(format_cyrillic_numeral(value).expect("format"), expected);
            assert_eq!(parse_cyrillic_numeral(expected).expect("parse"), value);
        }
    }

    #[test]
    fn rejects_noncanonical_order_and_missing_titlo() {
        assert!(parse_cyrillic_numeral("іє҃").is_err());
        assert!(parse_cyrillic_numeral("ка").is_err());
    }
}
