//! Stress paradigms (stage 3): a function from cell to a stress PLACE —
//! the stem (the lemma's own stressed vowel, by index from the start of
//! the word; the last stem vowel when the stem has lost it), the ending
//! (its first vowel; the last stem vowel when the ending has none), or an
//! explicit vowel index. Named paradigms live in `lexicon/stress.tsv`
//! (`a` and `b` are built in); a lexeme's `stress` column names one and
//! may add per-cell exceptions.
//!
//! Column grammar: `a` | `a<N>` (fixed on vowel N) | `b` | `<name>` |
//! `<name>{cell=S|E|<N>;…}` | `{…}` — with `sg`/`du`/`pl` accepted as
//! keys for a whole number and a block name (`part`, `part.pres.act`,
//! `short.comp`) for a whole block. `-` is no stress (Old Church Slavonic,
//! a titlo lemma).

use crate::cell::{Cell, Pos, parse_number};
use crate::grammar::Number;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Stem,
    End,
    /// The last vowel of the stem (a comparative's suffix: велича́йшїй).
    StemLast,
    Index(u8),
}

impl Place {
    fn parse(s: &str) -> Option<Place> {
        match s {
            "S" => Some(Place::Stem),
            "E" => Some(Place::End),
            "L" => Some(Place::StemLast),
            n => n.parse().ok().map(Place::Index),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Key {
    Cell(Cell),
    Number(Number),
    /// A block prefix (`part`, `part.pres.act`): every cell whose block
    /// name starts with it.
    Block(String),
}

/// A named or inline paradigm: a default place and exceptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paradigm {
    pub default: Place,
    rules: Vec<(Key, Place)>,
}

impl Paradigm {
    /// `S;pl=E;gen.pl=S` — a bare first token is the default; without one
    /// the default is `fallback`.
    fn parse(spec: &str, pos: Pos, fallback: Option<Place>) -> Result<Paradigm, String> {
        let mut default = fallback;
        let mut rules = Vec::new();
        for (i, item) in spec.split(';').map(str::trim).filter(|s| !s.is_empty()).enumerate() {
            if i == 0 && !item.contains('=') {
                default = Some(Place::parse(item).ok_or_else(|| format!("stress place {item}"))?);
                continue;
            }
            let (k, v) = item.split_once('=').ok_or_else(|| format!("stress item {item}"))?;
            let place = Place::parse(v).ok_or_else(|| format!("stress place {v}"))?;
            let key = if let Some(n) = parse_number(k) {
                Key::Number(n)
            } else if let Some(cell) = Cell::parse(pos, k) {
                Key::Cell(cell)
            } else if k == "part" || k.starts_with("part.") || k.ends_with(".comp") || k.ends_with(".pos") {
                Key::Block(k.to_string())
            } else {
                return Err(format!("stress cell {k}"));
            };
            rules.push((key, place));
        }
        Ok(Paradigm { default: default.ok_or("empty stress spec")?, rules })
    }

    fn place(&self, cell: Cell) -> Option<Place> {
        let exact = self.rules.iter().find(|(k, _)| *k == Key::Cell(cell)).map(|(_, p)| *p);
        exact
            .or_else(|| {
                let block = cell.block()?;
                self.rules
                    .iter()
                    .find(|(k, _)| matches!(k, Key::Block(b) if block == *b || block.starts_with(&format!("{b}."))))
                    .map(|(_, p)| *p)
            })
            .or_else(|| {
                let number = cell.number()?;
                self.rules.iter().find(|(k, _)| *k == Key::Number(number)).map(|(_, p)| *p)
            })
    }
}

/// The named paradigms: the built-in `a`/`b` and `lexicon/stress.tsv`.
fn named() -> &'static HashMap<String, String> {
    static NAMED: OnceLock<HashMap<String, String>> = OnceLock::new();
    NAMED.get_or_init(|| {
        let mut out = HashMap::new();
        out.insert("a".to_string(), "S".to_string());
        // the ending everywhere — except a participle, whose stem carries
        // the thematic vowel the finite endings supply (творю̀, творѧ́щій)
        out.insert("b".to_string(), "E;part=S".to_string());
        for line in include_str!("../lexicon/stress.tsv").lines() {
            if line.starts_with('#') || line.trim().is_empty() || line.starts_with("name\t") {
                continue;
            }
            if let Some((name, spec)) = line.split_once('\t') {
                out.insert(name.trim().to_string(), spec.trim().to_string());
            }
        }
        out
    })
}

/// A lexeme's stress column, parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StressSpec {
    base: Paradigm,
    exceptions: Paradigm,
}

impl StressSpec {
    /// `None` for `-`/empty (no stress).
    pub fn parse(column: &str, pos: Pos) -> Result<Option<StressSpec>, String> {
        let column = column.trim();
        if column.is_empty() || column == "-" {
            return Ok(None);
        }
        let (name, braces) = match column.split_once('{') {
            Some((n, rest)) => (n.trim(), Some(rest.strip_suffix('}').ok_or("unclosed { in stress")?)),
            None => (column, None),
        };
        let base = if name.is_empty() {
            Paradigm { default: Place::Stem, rules: Vec::new() }
        } else if let Some(digits) = name.strip_prefix('a').filter(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit())) {
            let n: u8 = digits.parse().map_err(|_| format!("stress index {digits}"))?;
            Paradigm { default: Place::Index(n), rules: Vec::new() }
        } else {
            let spec = named().get(name).ok_or_else(|| format!("unknown stress paradigm {name}"))?;
            Paradigm::parse(spec, pos, None)?
        };
        let exceptions = match braces {
            // an inline map defaults to the base paradigm's default
            Some(inner) => Paradigm::parse(inner, pos, Some(base.default))?,
            None => Paradigm { default: base.default, rules: Vec::new() },
        };
        Ok(Some(StressSpec { base, exceptions }))
    }

    /// Where `cell` is stressed.
    pub fn place(&self, cell: Cell) -> Place {
        self.exceptions
            .place(cell)
            .or_else(|| self.base.place(cell))
            .unwrap_or(self.base.default)
    }
}

/// The stressed vowel index of a form: `place` resolved against the
/// lemma's stress, the stem's vowel count and the whole form's.
pub fn resolve(place: Place, lemma_stress: Option<u8>, stem_vowels: usize, total_vowels: usize) -> Option<u8> {
    if total_vowels == 0 {
        return None;
    }
    let last = total_vowels - 1;
    let last_stem = stem_vowels.saturating_sub(1).min(last);
    let index = match place {
        Place::Stem => {
            let k = usize::from(lemma_stress?);
            if k < stem_vowels { k } else { last_stem }
        }
        Place::End => {
            if total_vowels > stem_vowels { stem_vowels } else { last_stem }
        }
        Place::StemLast => last_stem,
        Place::Index(n) => usize::from(n).min(last),
    };
    u8::try_from(index).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::NounCell;

    fn cell(s: &str) -> Cell {
        Cell::Noun(NounCell::parse(s).expect("cell"))
    }

    #[test]
    fn parsing_and_places() {
        let a = StressSpec::parse("a", Pos::Noun).expect("ok").expect("some");
        assert_eq!(a.place(cell("gen.pl")), Place::Stem);
        let b = StressSpec::parse("b{nom.pl=S}", Pos::Noun).expect("ok").expect("some");
        assert_eq!(b.place(cell("gen.sg")), Place::End);
        assert_eq!(b.place(cell("nom.pl")), Place::Stem);
        let inline = StressSpec::parse("{S;pl=E;gen.pl=S}", Pos::Noun).expect("ok").expect("some");
        assert_eq!(inline.place(cell("gen.sg")), Place::Stem);
        assert_eq!(inline.place(cell("dat.pl")), Place::End);
        assert_eq!(inline.place(cell("gen.pl")), Place::Stem);
        let a1 = StressSpec::parse("a1", Pos::Noun).expect("ok").expect("some");
        assert_eq!(a1.place(cell("nom.sg")), Place::Index(1));
        assert!(StressSpec::parse("-", Pos::Noun).expect("ok").is_none());
        assert!(StressSpec::parse("zz", Pos::Noun).is_err());
    }

    #[test]
    fn resolution() {
        // ра́бъ (lemma stress 0), stem раб (1 vowel)
        assert_eq!(resolve(Place::Stem, Some(0), 1, 2), Some(0)); // ра́ба (a)
        assert_eq!(resolve(Place::End, Some(0), 1, 2), Some(1)); // раба̀ (b)
        assert_eq!(resolve(Place::End, Some(0), 1, 1), Some(0)); // ра̑бъ: no ending vowel
        // ѻ҆те́цъ (lemma stress 1) with the fleeting vowel dropped: stem ѻтц (1 vowel)
        assert_eq!(resolve(Place::Stem, Some(1), 1, 2), Some(0));
        assert_eq!(resolve(Place::Index(3), Some(0), 1, 2), Some(1), "clamped");
        assert_eq!(resolve(Place::Stem, None, 1, 2), None, "unaccented lemma");
    }
}
