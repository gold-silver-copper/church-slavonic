//! The lexicon: the committed tsv files under `lexicon/`, embedded and
//! parsed on first use. One line per lexeme; the columns are documented
//! in `docs/DESIGN.md` and checked here — a malformed line is a hard
//! error at first use, never a silently skipped entry.
//!
//! ```text
//! id  lemma  pos  gender  anim  class  stress  stems  overrides  variants  src  note
//! ```
//!
//! `-` is the empty value in every column. `stems` is `name=letters;…`,
//! `overrides` is `cell=printform;…`, `variants` is `cell=form|form;…`,
//! `src` is `token;token` (P:<class>, A:§n, R:, K:, U:, W:<ref>, H:).

use crate::cell::{Cell, Pos};
use crate::grammar::{Gender, Recension};
use crate::orthography::{comparison_key, strip_marks};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Where a lexeme (or a form) came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// A lexicon line.
    Attested,
    /// Built by the guesser from a lemma alone.
    Guessed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lexeme {
    pub id: String,
    /// The accented citation form (Synodal) or the plain lemma (OCS).
    pub lemma: String,
    pub pos: Pos,
    pub gender: Option<Gender>,
    /// `Some(true)` animate, `Some(false)` inanimate, `None` unmarked.
    pub animate: Option<bool>,
    /// The letter class (a row of `lexicon/classes/*.toml`).
    pub class: String,
    /// The stress paradigm, unparsed (`-` for none); see `crate::stress`.
    pub stress: String,
    pub stems: Vec<(String, String)>,
    /// Cells whose printed form is not what class + stress produce.
    pub overrides: Vec<(Cell, String)>,
    /// Additional attested forms per cell, for the analyzer.
    pub variants: Vec<(Cell, Vec<String>)>,
    pub src: Vec<String>,
    pub note: String,
    pub provenance: Provenance,
}

impl Lexeme {
    /// The lemma with its marks stripped: the id's stem.
    pub fn bare_lemma(&self) -> String {
        strip_marks(&self.lemma)
    }

    pub fn is_hand_edited(&self) -> bool {
        self.src.iter().any(|s| s.starts_with("H:"))
    }
}

pub struct Lexicon {
    pub recension: Recension,
    lexemes: Vec<Lexeme>,
    by_id: HashMap<String, usize>,
    by_key: HashMap<(String, Pos), Vec<usize>>,
    index: crate::analyze::IndexSlot,
}

/// The embedded files, by recension: (pos, text).
const SYN_FILES: [(Pos, &str); 5] = [
    (Pos::Noun, include_str!("../lexicon/syn/nouns.tsv")),
    (Pos::Adjective, include_str!("../lexicon/syn/adjectives.tsv")),
    (Pos::Verb, include_str!("../lexicon/syn/verbs.tsv")),
    (Pos::Pronoun, include_str!("../lexicon/syn/pronouns.tsv")),
    (Pos::Closed, include_str!("../lexicon/syn/closed.tsv")),
];
const OCS_FILES: [(Pos, &str); 4] = [
    (Pos::Noun, include_str!("../lexicon/ocs/nouns.tsv")),
    (Pos::Adjective, include_str!("../lexicon/ocs/adjectives.tsv")),
    (Pos::Verb, include_str!("../lexicon/ocs/verbs.tsv")),
    (Pos::Pronoun, include_str!("../lexicon/ocs/pronouns.tsv")),
];

impl Lexicon {
    /// The Synodal lexicon, parsed once.
    pub fn synodal() -> &'static Lexicon {
        static L: OnceLock<Lexicon> = OnceLock::new();
        L.get_or_init(|| Lexicon::from_files(Recension::Synodal, &SYN_FILES))
    }

    /// The Old Church Slavonic lexicon, parsed once.
    pub fn ocs() -> &'static Lexicon {
        static L: OnceLock<Lexicon> = OnceLock::new();
        L.get_or_init(|| Lexicon::from_files(Recension::OldChurchSlavonic, &OCS_FILES))
    }

    pub fn of(recension: Recension) -> &'static Lexicon {
        match recension {
            Recension::Synodal => Lexicon::synodal(),
            Recension::OldChurchSlavonic => Lexicon::ocs(),
        }
    }

    fn from_files(recension: Recension, files: &[(Pos, &str)]) -> Lexicon {
        let mut lexemes = Vec::new();
        for (pos, text) in files {
            match parse(text, *pos) {
                Ok(mut parsed) => lexemes.append(&mut parsed),
                Err(e) => panic!("lexicon/{}: {e}", pos.tag()),
            }
        }
        Lexicon::from_lexemes(recension, lexemes)
    }

    /// Build a lexicon from parsed lexemes (the tools crate builds
    /// candidate lexicons this way before writing them).
    pub fn from_lexemes(recension: Recension, lexemes: Vec<Lexeme>) -> Lexicon {
        let mut by_id = HashMap::with_capacity(lexemes.len());
        let mut by_key: HashMap<(String, Pos), Vec<usize>> = HashMap::new();
        for (i, l) in lexemes.iter().enumerate() {
            if by_id.insert(l.id.clone(), i).is_some() {
                panic!("lexicon: duplicate id {}", l.id);
            }
            by_key.entry((comparison_key(&l.lemma), l.pos)).or_default().push(i);
        }
        Lexicon { recension, lexemes, by_id, by_key, index: crate::analyze::IndexSlot::new() }
    }

    pub(crate) fn index_cell(&self) -> &crate::analyze::IndexSlot {
        &self.index
    }

    pub(crate) fn lexeme_at(&self, i: usize) -> &Lexeme {
        &self.lexemes[i]
    }

    pub fn get(&self, id: &str) -> Option<&Lexeme> {
        self.by_id.get(id).map(|&i| &self.lexemes[i])
    }

    /// Every lexeme whose lemma matches, accent-tolerant; homographs come
    /// back together.
    pub fn find(&self, lemma: &str, pos: Pos) -> Vec<&Lexeme> {
        self.by_key
            .get(&(comparison_key(lemma), pos))
            .map(|v| v.iter().map(|&i| &self.lexemes[i]).collect())
            .unwrap_or_default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Lexeme> {
        self.lexemes.iter()
    }

    pub fn len(&self) -> usize {
        self.lexemes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lexemes.is_empty()
    }
}

pub const COLUMNS: [&str; 12] = [
    "id", "lemma", "pos", "gender", "anim", "class", "stress", "stems", "overrides", "variants",
    "src", "note",
];

fn empty(s: &str) -> bool {
    s == "-" || s.is_empty()
}

/// Parse one tsv file of `pos`. A header line naming the columns is
/// accepted; `#` lines and blank lines are skipped.
pub fn parse(text: &str, pos: Pos) -> Result<Vec<Lexeme>, String> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line_no = n + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols[0] == "id" {
            if cols != COLUMNS {
                return Err(format!("line {line_no}: header columns must be {}", COLUMNS.join(" ")));
            }
            continue;
        }
        if cols.len() != COLUMNS.len() {
            return Err(format!(
                "line {line_no}: expected {} tab-separated columns, found {}",
                COLUMNS.len(),
                cols.len()
            ));
        }
        let [id, lemma, pos_tag, gender, anim, class, stress, stems, overrides, variants, src, note] =
            cols[..]
        else {
            unreachable!()
        };
        let line_pos = Pos::parse(pos_tag).ok_or_else(|| format!("line {line_no}: pos {pos_tag}"))?;
        if line_pos != pos {
            return Err(format!("line {line_no}: pos {pos_tag} in the {} file", pos.tag()));
        }
        let gender = match gender {
            "m" => Some(Gender::Masculine),
            "f" => Some(Gender::Feminine),
            "n" => Some(Gender::Neuter),
            g if empty(g) => None,
            other => return Err(format!("line {line_no}: gender {other}")),
        };
        let animate = match anim {
            "anim" => Some(true),
            "inan" => Some(false),
            a if empty(a) => None,
            other => return Err(format!("line {line_no}: anim {other}")),
        };
        let parse_cell = |s: &str| -> Result<Cell, String> {
            Cell::parse(pos, s).ok_or_else(|| format!("line {line_no}: cell {s}"))
        };
        let mut stems_v = Vec::new();
        if !empty(stems) {
            for item in stems.split(';') {
                let (k, v) = item
                    .split_once('=')
                    .ok_or_else(|| format!("line {line_no}: stems item {item}"))?;
                stems_v.push((k.to_string(), v.to_string()));
            }
        }
        let mut overrides_v = Vec::new();
        if !empty(overrides) {
            for item in overrides.split(';') {
                let (k, v) = item
                    .split_once('=')
                    .ok_or_else(|| format!("line {line_no}: overrides item {item}"))?;
                overrides_v.push((parse_cell(k)?, v.to_string()));
            }
        }
        let mut variants_v = Vec::new();
        if !empty(variants) {
            for item in variants.split(';') {
                let (k, v) = item
                    .split_once('=')
                    .ok_or_else(|| format!("line {line_no}: variants item {item}"))?;
                variants_v.push((parse_cell(k)?, v.split('|').map(str::to_string).collect()));
            }
        }
        let src_v: Vec<String> = if empty(src) {
            Vec::new()
        } else {
            src.split(';').map(str::to_string).collect()
        };
        out.push(Lexeme {
            id: id.to_string(),
            lemma: lemma.to_string(),
            pos,
            gender,
            animate,
            class: if empty(class) { String::new() } else { class.to_string() },
            stress: if empty(stress) { String::new() } else { stress.to_string() },
            stems: stems_v,
            overrides: overrides_v,
            variants: variants_v,
            src: src_v,
            note: if empty(note) { String::new() } else { note.to_string() },
            provenance: Provenance::Attested,
        });
    }
    Ok(out)
}

/// Write lexemes back in the file format (sorted by id by the caller).
pub fn format(lexemes: &[Lexeme]) -> String {
    let dash = |s: &str| if s.is_empty() { "-".to_string() } else { s.to_string() };
    let mut out = String::new();
    out.push_str(&COLUMNS.join("\t"));
    out.push('\n');
    for l in lexemes {
        let stems = l.stems.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(";");
        let overrides =
            l.overrides.iter().map(|(c, v)| format!("{}={v}", c.name())).collect::<Vec<_>>().join(";");
        let variants = l
            .variants
            .iter()
            .map(|(c, v)| format!("{}={}", c.name(), v.join("|")))
            .collect::<Vec<_>>()
            .join(";");
        let cols = [
            l.id.clone(),
            l.lemma.clone(),
            l.pos.tag().to_string(),
            l.gender.map(crate::cell::gender_name).unwrap_or("-").to_string(),
            match l.animate {
                Some(true) => "anim",
                Some(false) => "inan",
                None => "-",
            }
            .to_string(),
            dash(&l.class),
            dash(&l.stress),
            dash(&stems),
            dash(&overrides),
            dash(&variants),
            dash(&l.src.join(";")),
            dash(&l.note),
        ];
        out.push_str(&cols.join("\t"));
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_parses_and_formats_back() {
        let text = "id\tlemma\tpos\tgender\tanim\tclass\tstress\tstems\toverrides\tvariants\tsrc\tnote\n\
                    рабъ.n\tра́бъ\tn\tm\tanim\tN1t\tb\t-\t-\tgen.pl=рабѡ́въ\tP:N1t;A:§12\t-\n\
                    ѻтецъ.n\tѻ҆те́цъ\tn\tm\tanim\tN1c*\tb\tobl=ѻтц\tvoc.sg=ѻ҆́тче\t-\tP:N1c*\tfleeting\n";
        let parsed = parse(text, Pos::Noun).expect("parses");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].variants[0].0.name(), "gen.pl");
        assert_eq!(parsed[1].overrides[0].1, "ѻ҆́тче");
        assert_eq!(parsed[1].stems, vec![("obl".to_string(), "ѻтц".to_string())]);
        assert_eq!(format(&parsed), text);
        let lex = Lexicon::from_lexemes(Recension::Synodal, parsed);
        assert_eq!(lex.get("рабъ.n").map(|l| l.lemma.as_str()), Some("ра́бъ"));
        assert_eq!(lex.find("рабъ", Pos::Noun).len(), 1, "accent-tolerant");
        assert!(lex.find("рабъ", Pos::Verb).is_empty());
    }

    #[test]
    fn malformed_lines_are_errors() {
        assert!(parse("x.n\tx\tn\n", Pos::Noun).is_err());
        assert!(parse("x.n\tx\tv\t-\t-\t-\t-\t-\t-\t-\t-\t-\n", Pos::Noun).is_err());
        assert!(parse("x.n\tx\tn\t-\t-\t-\t-\t-\tgen=x\t-\t-\t-\n", Pos::Noun).is_err());
    }

    #[test]
    fn the_embedded_files_parse() {
        let _ = Lexicon::synodal();
        let _ = Lexicon::ocs();
    }
}
