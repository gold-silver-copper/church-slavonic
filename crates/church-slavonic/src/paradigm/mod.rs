//! Letter classes (stage 2): per class and cell, an ending on a numbered
//! stem, with the number mark, alternatives in order, and references to
//! other cells. The tables are data — `lexicon/classes/<pos>.tsv`, one
//! class per line — read here; the stem derivations are the small closed
//! set below.
//!
//! Class line: `class  exemplar  strip  stems  cell=spec …` where
//!
//! - `strip` is how many letters of the lemma are its ending;
//! - `stems` is `n=derivation;…` with derivations `base` (the lemma minus
//!   `strip` letters), `drop` (base minus its last vowel — the fleeting
//!   vowel dropped), `insert` (base with a vowel inserted before its last
//!   consonant: the lexeme's `stems=ins=…` when given, else the rule of
//!   [`insert_fleeting`]), `pal1[:x]` / `pal2[:x]` (the first / second
//!   palatalisation of derivation `x`, `base` by default), `ext:suffix`
//!   (base plus a suffix), `cut` (base minus its last letter); a lexeme's
//!   `stems=base=…` replaces the strip rule's base and `stems=<n>=…`
//!   spells stem n outright;
//! - a cell spec is `|`-separated alternatives, primary first: `N-ending`
//!   (stem N plus the ending; a trailing `^` is the number mark), `@cell`
//!   (the same as that cell), `@lemma` (the lemma's own letters), each
//!   optionally prefixed `anim:` or `inan:` to apply to that animacy only.

use crate::cell::{Cell, Pos};
use crate::orthography::is_vowel_letter;
use std::collections::HashMap;
use std::sync::OnceLock;

pub mod noun;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Derivation {
    Base,
    Drop,
    Insert,
    /// Base minus its last letter (`знамені` -> `знамен` before `-ьми`).
    Cut,
    Pal1(Box<Derivation>),
    Pal2(Box<Derivation>),
    Ext(String),
}

impl Derivation {
    fn parse(s: &str) -> Result<Derivation, String> {
        Ok(match s {
            "base" => Derivation::Base,
            "drop" => Derivation::Drop,
            "insert" => Derivation::Insert,
            "cut" => Derivation::Cut,
            _ => {
                if let Some(rest) = s.strip_prefix("pal1") {
                    Derivation::Pal1(Box::new(sub(rest)?))
                } else if let Some(rest) = s.strip_prefix("pal2") {
                    Derivation::Pal2(Box::new(sub(rest)?))
                } else if let Some(suffix) = s.strip_prefix("ext:") {
                    Derivation::Ext(suffix.to_string())
                } else {
                    return Err(format!("unknown stem derivation {s}"));
                }
            }
        })
    }
}

fn sub(rest: &str) -> Result<Derivation, String> {
    match rest.strip_prefix(':') {
        None if rest.is_empty() => Ok(Derivation::Base),
        Some(inner) => Derivation::parse(inner),
        None => Err(format!("bad derivation suffix {rest}")),
    }
}

/// One alternative of a cell's spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alt {
    /// `Some(true)` animate only, `Some(false)` inanimate only.
    pub animacy: Option<bool>,
    pub shape: Shape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shape {
    Ending { stem: u8, ending: String, mark: bool },
    Ref(Cell),
    Lemma,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub exemplar: String,
    pub strip: usize,
    pub stems: Vec<(u8, Derivation)>,
    pub cells: HashMap<Cell, Vec<Alt>>,
    /// The cells in table order (the paradigm's iteration order).
    pub order: Vec<Cell>,
}

/// The letters of one alternative, with its number mark.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Letters {
    pub letters: String,
    pub mark: bool,
    /// How many vowels the stem contributed (the stress layer's boundary).
    pub stem_vowels: usize,
}

/// What a class needs to know about the lexeme it declines.
pub struct Subject<'a> {
    /// The lemma's letters (marks stripped).
    pub lemma: &'a str,
    pub animate: Option<bool>,
    /// The lexeme's `stems` column.
    pub stems: &'a [(String, String)],
}

impl Class {
    /// The numbered stems of a lexeme.
    pub fn stems_of(&self, subject: &Subject<'_>) -> HashMap<u8, String> {
        // the lexeme may name its own base stem (`stems=base=…`: a plurale
        // tantum, an irregular stem); the class's strip rule otherwise
        let base: String = match subject.stems.iter().find(|(k, _)| k == "base") {
            Some((_, b)) => b.clone(),
            None => {
                let n = subject.lemma.chars().count().saturating_sub(self.strip);
                subject.lemma.chars().take(n).collect()
            }
        };
        let mut out = HashMap::new();
        for (n, derivation) in &self.stems {
            out.insert(*n, derive(derivation, &base, subject));
        }
        // a numbered stem the lexeme spells itself (`stems=1=льв`: the
        // fleeting vowel that leaves ь behind, a suppletive stem)
        for (k, v) in subject.stems {
            if let Ok(n) = k.parse::<u8>() {
                out.insert(n, v.clone());
            }
        }
        out
    }

    /// Every alternative of `cell` for the subject, primary first; empty
    /// when the class has no such cell.
    pub fn letters(&self, cell: Cell, subject: &Subject<'_>) -> Vec<Letters> {
        let stems = self.stems_of(subject);
        let mut out = Vec::new();
        self.collect(cell, subject, &stems, &mut out, 0);
        out
    }

    fn collect(
        &self,
        cell: Cell,
        subject: &Subject<'_>,
        stems: &HashMap<u8, String>,
        out: &mut Vec<Letters>,
        depth: usize,
    ) {
        if depth > 4 {
            return;
        }
        let Some(alts) = self.cells.get(&cell) else { return };
        for alt in alts {
            match (alt.animacy, subject.animate) {
                (Some(want), Some(have)) if want != have => continue,
                // an unmarked lexeme reads the inanimate alternative for
                // neuters and the animate one otherwise — the guesser's
                // default; the lexicon nearly always says
                (Some(want), None) if want != default_animacy(subject) => continue,
                _ => {}
            }
            match &alt.shape {
                Shape::Ending { stem, ending, mark } => {
                    if let Some(stem) = stems.get(stem) {
                        out.push(Letters {
                            letters: format!("{stem}{ending}"),
                            mark: *mark,
                            stem_vowels: stem.chars().filter(|c| is_vowel_letter(*c)).count(),
                        });
                    }
                }
                Shape::Ref(other) => self.collect(*other, subject, stems, out, depth + 1),
                Shape::Lemma => out.push(Letters {
                    letters: subject.lemma.to_string(),
                    mark: false,
                    stem_vowels: subject.lemma.chars().filter(|c| is_vowel_letter(*c)).count(),
                }),
            }
        }
    }

    pub fn has(&self, cell: Cell) -> bool {
        self.cells.contains_key(&cell)
    }
}

fn default_animacy(subject: &Subject<'_>) -> bool {
    // neuters are inanimate; the rest animate (the measured 1.x default)
    !matches!(subject.lemma.chars().last(), Some('о' | 'е' | 'ѧ'))
}

fn derive(d: &Derivation, base: &str, subject: &Subject<'_>) -> String {
    match d {
        Derivation::Base => base.to_string(),
        Derivation::Drop => drop_fleeting(base),
        Derivation::Cut => {
            let n = base.chars().count().saturating_sub(1);
            base.chars().take(n).collect()
        }
        Derivation::Insert => subject
            .stems
            .iter()
            .find(|(k, _)| k == "ins")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| insert_fleeting(base)),
        Derivation::Pal1(inner) => palatalise(&derive(inner, base, subject), true),
        Derivation::Pal2(inner) => palatalise(&derive(inner, base, subject), false),
        Derivation::Ext(suffix) => {
            // a husher takes а, not ѧ (ѻ҆троча̀ : ѻ҆троча́та)
            let husher = matches!(base.chars().last(), Some('ж' | 'ч' | 'ш' | 'щ'));
            let suffix = match suffix.strip_prefix('ѧ') {
                Some(rest) if husher => format!("а{rest}"),
                _ => suffix.clone(),
            };
            format!("{base}{suffix}")
        }
    }
}

/// Drop the fleeting vowel: the last vowel of the stem (`осел` -> `осл`,
/// `отец` -> `отц`, `свиток` -> `свитк`). A stem with one vowel keeps it.
pub fn drop_fleeting(stem: &str) -> String {
    let chars: Vec<char> = stem.chars().collect();
    // a monosyllable drops its only vowel too (де́нь : днѝ, со́нъ : сна̀)
    let Some(last) = chars.iter().rposition(|c| is_vowel_letter(*c)) else {
        return stem.to_string();
    };
    let mut out: Vec<char> = chars[..last].to_vec();
    // a fleeting vowel after a vowel leaves `й` behind (`боец` -> `бойц`,
    // `заѧц` -> `заѧйц`… the print: бойцы̀, за́йца)
    if last > 0 && is_vowel_letter(chars[last - 1]) {
        out.push('й');
    }
    out.extend_from_slice(&chars[last + 1..]);
    out.into_iter().collect()
}

/// Insert the fleeting vowel before the stem's last consonant: `о` when
/// either of the two final consonants is a velar (`окн` -> `окон`,
/// `егѵптѧнк` -> `егѵптѧнок`), else `е` (`гривн` -> `гривен`, `овц` ->
/// `овец`). The lexeme's `stems=ins=…` overrides the rule.
pub fn insert_fleeting(stem: &str) -> String {
    let chars: Vec<char> = stem.chars().collect();
    let n = chars.len();
    if n < 2 || is_vowel_letter(chars[n - 1]) {
        return stem.to_string();
    }
    let velar = |c: char| matches!(c, 'к' | 'г' | 'х');
    let vowel = if velar(chars[n - 1]) || velar(chars[n - 2]) { 'о' } else { 'е' };
    let mut out: String = chars[..n - 1].iter().collect();
    out.push(vowel);
    out.push(chars[n - 1]);
    out
}

/// The palatalisation of a stem's final consonant: first (`к`→`ч`, `г`→`ж`,
/// `х`→`ш`, `ц`→`ч`) or second (`к`→`ц`, `г`→`з`, `х`→`с`).
pub fn palatalise(stem: &str, first: bool) -> String {
    let mut chars: Vec<char> = stem.chars().collect();
    if let Some(last) = chars.last_mut() {
        *last = match (*last, first) {
            ('к', true) => 'ч',
            ('г', true) => 'ж',
            ('х', true) => 'ш',
            ('ц', true) => 'ч',
            ('к', false) => 'ц',
            ('г', false) => 'з',
            ('х', false) => 'с',
            (c, _) => c,
        };
    }
    chars.into_iter().collect()
}

/// Parse one class table.
pub fn parse_table(text: &str, pos: Pos) -> Result<Vec<Class>, String> {
    let mut out = Vec::new();
    let mut header: Option<Vec<String>> = None;
    for (n, line) in text.lines().enumerate() {
        let line_no = n + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols[0] == "class" {
            header = Some(cols.iter().map(|s| s.to_string()).collect());
            continue;
        }
        let Some(header) = &header else {
            return Err(format!("line {line_no}: the header line must come first"));
        };
        if cols.len() != header.len() {
            return Err(format!("line {line_no}: {} columns, header has {}", cols.len(), header.len()));
        }
        let mut class = Class {
            name: cols[0].to_string(),
            exemplar: cols[1].to_string(),
            strip: cols[2].parse().map_err(|_| format!("line {line_no}: strip {}", cols[2]))?,
            stems: Vec::new(),
            cells: HashMap::new(),
            order: Vec::new(),
        };
        for item in cols[3].split(';') {
            let (n, d) = item.split_once('=').ok_or_else(|| format!("line {line_no}: stems item {item}"))?;
            let n: u8 = n.parse().map_err(|_| format!("line {line_no}: stem number {n}"))?;
            class.stems.push((n, Derivation::parse(d).map_err(|e| format!("line {line_no}: {e}"))?));
        }
        for (name, spec) in header.iter().zip(cols.iter()).skip(4) {
            if *spec == "-" {
                continue;
            }
            let cell = Cell::parse(pos, name).ok_or_else(|| format!("line {line_no}: cell {name}"))?;
            let mut alts = Vec::new();
            for alt in spec.split('|') {
                let (animacy, rest) = if let Some(r) = alt.strip_prefix("anim:") {
                    (Some(true), r)
                } else if let Some(r) = alt.strip_prefix("inan:") {
                    (Some(false), r)
                } else {
                    (None, alt)
                };
                let shape = if rest == "@lemma" {
                    Shape::Lemma
                } else if let Some(target) = rest.strip_prefix('@') {
                    Shape::Ref(Cell::parse(pos, target).ok_or_else(|| format!("line {line_no}: ref {rest}"))?)
                } else {
                    let (stem, ending) =
                        rest.split_once('-').ok_or_else(|| format!("line {line_no}: alternative {rest}"))?;
                    let stem: u8 = stem.parse().map_err(|_| format!("line {line_no}: stem {stem}"))?;
                    let mark = ending.ends_with('^');
                    Shape::Ending { stem, ending: ending.trim_end_matches('^').to_string(), mark }
                };
                alts.push(Alt { animacy, shape });
            }
            class.cells.insert(cell, alts);
            class.order.push(cell);
        }
        out.push(class);
    }
    Ok(out)
}

/// A parsed class table with lookup by name.
pub struct Table {
    classes: Vec<Class>,
    by_name: HashMap<String, usize>,
}

impl Table {
    pub fn parse(text: &str, pos: Pos) -> Result<Table, String> {
        let classes = parse_table(text, pos)?;
        let by_name = classes.iter().enumerate().map(|(i, c)| (c.name.clone(), i)).collect();
        Ok(Table { classes, by_name })
    }
    pub fn get(&self, name: &str) -> Option<&Class> {
        self.by_name.get(name).map(|&i| &self.classes[i])
    }
    pub fn iter(&self) -> impl Iterator<Item = &Class> {
        self.classes.iter()
    }
}

/// The class table of a part of speech (parsed once).
pub fn table(pos: Pos) -> &'static Table {
    static NOUN: OnceLock<Table> = OnceLock::new();
    match pos {
        Pos::Noun => NOUN.get_or_init(|| {
            Table::parse(noun::TABLE, Pos::Noun).unwrap_or_else(|e| panic!("classes/noun.tsv: {e}"))
        }),
        _ => unimplemented!("class tables for {pos:?} arrive in Part 3"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::NounCell;
    use crate::grammar::{Case, Number};

    fn letters(class: &str, lemma: &str, animate: Option<bool>, cell: &str) -> Vec<String> {
        let t = table(Pos::Noun);
        let c = t.get(class).expect("class");
        let subject = Subject { lemma, animate, stems: &[] };
        c.letters(Cell::Noun(NounCell::parse(cell).expect("cell")), &subject)
            .into_iter()
            .map(|l| format!("{}{}", l.letters, if l.mark { "^" } else { "" }))
            .collect()
    }

    #[test]
    fn the_legend_exemplars() {
        assert_eq!(letters("N1t", "рабъ", Some(true), "gen.sg"), ["раба"]);
        assert_eq!(letters("N1t", "рабъ", Some(true), "acc.sg"), ["раба"]);
        assert_eq!(letters("N1t", "градъ", Some(false), "acc.sg"), ["градъ"]);
        assert_eq!(letters("N1t", "рабъ", Some(true), "gen.pl"), ["рабовъ^", "рабъ^"]);
        assert_eq!(letters("N1t", "рабъ", Some(true), "acc.pl"), ["рабы", "рабовъ^", "рабъ^"]);
        assert_eq!(letters("N1t", "градъ", Some(false), "acc.pl"), ["грады", "градовъ^", "градъ^"]);
        assert_eq!(letters("N1t", "рабъ", Some(true), "voc.du"), ["раба^"]);
        assert_eq!(letters("N1k", "отрокъ", Some(true), "loc.sg"), ["отроцѣ", "отрокѣ"]);
        assert_eq!(letters("N1k", "отрокъ", Some(true), "voc.sg"), ["отроче", "отроке"]);
        assert_eq!(letters("N1c*", "отецъ", Some(true), "gen.sg"), ["отца"]);
        assert_eq!(letters("N1c*", "отецъ", Some(true), "voc.sg"), ["отче"]);
        assert_eq!(letters("N1c*", "отецъ", Some(true), "nom.sg"), ["отецъ"]);
        assert_eq!(letters("N1k*", "свитокъ", Some(false), "loc.sg"), ["свитцѣ"]);
        assert_eq!(letters("N1k*", "свитокъ", Some(false), "voc.sg"), ["свитче"]);
        assert_eq!(letters("N3t*", "гривна", Some(false), "gen.pl"), ["гривенъ"]);
        assert_eq!(letters("N3k*", "егѵптѧнка", Some(true), "gen.pl"), ["егѵптѧнокъ"]);
        assert_eq!(letters("N3k*", "егѵптѧнка", Some(true), "dat.sg"), ["егѵптѧнцѣ", "егѵптѧнкѣ"]);
        assert_eq!(letters("N5en", "имѧ", Some(false), "nom.sg"), ["имѧ"]);
        assert_eq!(letters("N5en", "имѧ", Some(false), "gen.sg"), ["имене"]);
        assert_eq!(letters("N5er", "мати", Some(true), "acc.sg"), ["матерь"]);
        assert_eq!(letters("N5*ov", "церковь", Some(false), "gen.sg"), ["церкве"]);
        assert_eq!(letters("N1in", "галілеанинъ", Some(true), "nom.pl"), ["галілеане"]);
        assert_eq!(letters("N1e", "іерей", Some(true), "nom.pl"), ["іерее^"]);
        assert_eq!(letters("0", "аллилꙋіа", None, "dat.pl"), ["аллилꙋіа"]);
        // the lexeme's own inserted stem beats the rule
        let t = table(Pos::Noun);
        let c = t.get("N3t*").expect("class");
        let stems = vec![("ins".to_string(), "сотон".to_string())];
        let s = Subject { lemma: "сотна", animate: Some(false), stems: &stems };
        let l = c.letters(Cell::Noun(NounCell::new(Case::Genitive, Number::Plural)), &s);
        assert_eq!(l[0].letters, "сотонъ");
    }

    #[test]
    fn derivations() {
        assert_eq!(drop_fleeting("осел"), "осл");
        assert_eq!(drop_fleeting("боец"), "бойц");
        assert_eq!(drop_fleeting("день"), "днь");
        assert_eq!(drop_fleeting("ден"), "дн");
        assert_eq!(insert_fleeting("окн"), "окон");
        assert_eq!(insert_fleeting("овц"), "овец");
        assert_eq!(palatalise("отрок", false), "отроц");
        assert_eq!(palatalise("дꙋх", true), "дꙋш");
    }
}
