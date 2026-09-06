//! The titlo layer: sacred abbreviations (гдⷭ҇ь, бг҃ъ, ст҃ы́й …) inflect —
//! the abbreviated STEM is constant while the ending follows the full
//! lemma's paradigm, and (per row) the print writes the result unaccented
//! or keeps the tail's accents. That makes the class GENERATABLE from the
//! committed table `lexicon/titlo.tsv` (abbreviated prefix, the full
//! prefix as a base-letter skeleton, the full lemma, part of speech,
//! accent mode, the family's token count in the pinned print), not
//! listable form by form: [`abbreviate`] cuts the row's full prefix off a
//! form by base-letter count and attaches the abbreviated one. The
//! treebank's `(abbr "гдⷭ҇" X)` wrapper is its consumer.

use crate::cell::Pos;
use std::sync::OnceLock;

const TABLE: &str = include_str!("../lexicon/titlo.tsv");

pub struct Row {
    /// the abbreviated prefix as printed («гдⷭ҇»)
    pub abbr: &'static str,
    /// the full prefix as a base-letter skeleton («господ»)
    pub full: &'static str,
    /// the full lemma the paradigm comes from
    pub lemma: &'static str,
    pub pos: Pos,
    /// strip = the abbreviated form is written unaccented
    pub strip: bool,
    /// standalone family tokens in the pinned print (attestation)
    pub count: u32,
}

/// The committed rows; a malformed table is a hard error at first use.
pub fn rows() -> &'static [Row] {
    static ROWS: OnceLock<Vec<Row>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let mut out = Vec::new();
        for (i, line) in TABLE.lines().enumerate() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            let [abbr, full, lemma, pos, mode, count, _note] = fields[..] else {
                panic!("lexicon/titlo.tsv line {}: expected 7 columns", i + 1);
            };
            out.push(Row {
                abbr: leak(abbr),
                full: leak(full),
                lemma: leak(lemma),
                pos: match pos {
                    "n" => Pos::Noun,
                    "a" => Pos::Adjective,
                    "v" => Pos::Verb,
                    "x" => Pos::Closed,
                    other => panic!("lexicon/titlo.tsv line {}: pos {other}", i + 1),
                },
                strip: match mode {
                    "strip" => true,
                    "keep" => false,
                    other => panic!("lexicon/titlo.tsv line {}: mode {other}", i + 1),
                },
                count: count.parse().unwrap_or_else(|_| {
                    panic!("lexicon/titlo.tsv line {}: bad count", i + 1)
                }),
            });
        }
        out
    })
}

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// Superscript combining letters → their base letters, for skeletons.
fn sup_base(c: char) -> Option<char> {
    Some(match c {
        'ⷠ' => 'б', 'ⷡ' => 'в', 'ⷢ' => 'г', 'ⷣ' => 'д', 'ⷷ' => 'е',
        'ⷤ' => 'ж', 'ⷥ' => 'з', 'ⷦ' => 'к', 'ⷧ' => 'л', 'ⷨ' => 'м',
        'ⷩ' => 'н', 'ⷪ' => 'о', 'ⷫ' => 'п', 'ⷬ' => 'р', 'ⷭ' => 'с',
        'ⷮ' => 'т', 'ⷯ' => 'х', 'ⷰ' => 'ц', 'ⷱ' => 'ч', 'ⷲ' => 'ш',
        'ⷳ' => 'щ',
        _ => return None,
    })
}

/// Accent marks the strip mode removes. Breathing (U+0486), titlo
/// (U+0483), pokrytie (U+0487) and the superscripts are NOT accents.
fn is_accent(c: char) -> bool {
    matches!(c, '\u{0300}' | '\u{0301}' | '\u{0311}')
}

fn is_combining(c: char) -> bool {
    matches!(c, '\u{0300}'..='\u{036F}' | '\u{0483}'..='\u{0489}' | '\u{2DE0}'..='\u{2DFF}' | '\u{A66F}')
}

/// One base letter, folded for skeleton comparison (і/ї, ѻ/о, є/е merge; ѡ is a letter the print keeps — бѡ́гъ is never бг҃ъ).
fn fold(c: char) -> char {
    match c {
        'ї' => 'і',
        'ѻ' => 'о',
        'є' => 'е',
        c => c.to_lowercase().next().unwrap_or(c),
    }
}

/// The base-letter skeleton of a word (superscripts read as letters).
pub fn skeleton(word: &str) -> String {
    let mut out = String::new();
    for c in word.chars() {
        if let Some(base) = sup_base(c) {
            out.push(base);
        } else if !is_combining(c) {
            out.push(fold(c));
        }
    }
    out
}

/// Abbreviate one full form under a row: the row's full prefix (counted in
/// base letters) is cut off — its combining marks with it — and the
/// abbreviated prefix takes its place; strip-mode rows lose the tail's
/// accents. `None` when the form does not begin with the row's prefix.
pub fn abbreviate(full_form: &str, row: &Row) -> Option<String> {
    let want: Vec<char> = row.full.chars().collect();
    let mut matched = 0;
    let mut cut = 0; // byte offset where the tail begins
    for (i, c) in full_form.char_indices() {
        if is_combining(c) {
            // a mark belongs to the letter before it: the prefix's last
            // letter takes its accent with it (спа́са → сп҃са, 3.3)
            if matched <= want.len() {
                cut = i + c.len_utf8();
            }
            continue;
        }
        if matched == want.len() {
            break;
        }
        if fold(c) != want[matched] {
            return None;
        }
        matched += 1;
        cut = i + c.len_utf8();
    }
    if matched < want.len() {
        return None;
    }
    let tail = &full_form[cut..];
    if tail.is_empty() {
        return None; // the whole word elided — not this class
    }
    let tail: String = if row.strip {
        tail.chars().filter(|c| !is_accent(*c)).collect()
    } else {
        tail.to_string()
    };
    Some(format!("{}{}", row.abbr, tail))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_loads_and_is_attested() {
        let rows = rows();
        assert!(rows.len() >= 20);
        for row in rows {
            assert!(row.count > 0, "{}: unattested row", row.abbr);
            assert!(!row.full.is_empty() && !row.abbr.is_empty());
        }
    }

    #[test]
    fn abbreviation_by_prefix_cut() {
        let gd = &rows()[0]; // гдⷭ҇ / господ / strip
        assert_eq!(abbreviate("го́спода", gd).as_deref(), Some("гдⷭ҇а"));
        assert_eq!(abbreviate("госпо́день", gd).as_deref(), Some("гдⷭ҇ень"));
        assert_eq!(abbreviate("рабо́мъ", gd), None, "wrong stem refuses");
        let st = rows().iter().find(|r| r.abbr == "ст҃").expect("свѧты́й row");
        // keep-mode: the tail accent survives
        assert_eq!(abbreviate("свѧта́гѡ", st).as_deref(), Some("ст҃а́гѡ"));
        let otec = rows().iter().find(|r| r.abbr == "ѻ҆ц҃").expect("ѻ҆те́цъ row");
        assert_eq!(abbreviate("ѻ҆тца̀", otec).as_deref(), Some("ѻ҆ц҃а̀"));
        assert_eq!(abbreviate("ѻ҆те́цъ", otec), None, "nominative stem differs");
        // the accent on the prefix's last letter goes with the prefix
        let sp = rows().iter().find(|r| r.abbr == "сп҃" && r.full == "спа").expect("сп҃ row");
        assert_eq!(abbreviate("спа́са", sp).as_deref(), Some("сп҃са"));
    }

    #[test]
    fn skeletons_read_superscripts_as_letters() {
        assert_eq!(skeleton("гдⷭ҇ь"), "гдсь");
        assert_eq!(skeleton("прⷪ҇ро́къ"), "пророкъ");
        assert_eq!(skeleton("бг҃ъ"), "бгъ");
    }
}
