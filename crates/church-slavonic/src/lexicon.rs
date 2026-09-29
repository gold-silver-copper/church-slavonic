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
//! `src` is `token;token` (`P:` with the class, `A:§n`, `R:`, `K:`, `U:`, `W:` with a reference, `H:`).

use crate::error::LexiconError;
use crate::cell::{Cell, Pos};
use crate::grammar::{Case, Gender, Prosody, Recension};
use crate::orthography::{comparison_key, strip_marks};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Where a lexeme (or a form) came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provenance {
    /// Imported or hand-maintained lexical data; this does not attest any form.
    LexiconEntry,
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
    /// Additional supplied forms per cell; source claims need separate validation.
    pub variants: Vec<(Cell, Vec<String>)>,
    /// A source's count on a variant (`form×12` in the column): the
    /// analyzer's weight; a form without one weighs 0.
    pub variant_weights: Vec<(Cell, String, u32)>,
    pub src: Vec<String>,
    pub note: String,
    pub provenance: Provenance,
    /// The recension whose class tables and print the lexeme lives in.
    pub recension: Recension,
}

impl Lexeme {
    /// The lemma with its marks stripped: the id's stem.
    pub fn bare_lemma(&self) -> String {
        strip_marks(&self.lemma)
    }

    pub fn is_hand_edited(&self) -> bool {
        self.src.iter().any(|s| s.starts_with("H:"))
    }

    fn stem_value(&self, key: &str) -> Option<&str> {
        self.stems.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }

    /// The cases a preposition governs (`stems=gov=acc|loc`), commonest
    /// first; empty for a word that governs nothing.
    pub fn government(&self) -> Vec<Case> {
        self.stem_value("gov").map(|v| v.split('|').filter_map(crate::cell::parse_case).collect()).unwrap_or_default()
    }

    /// The word's place in the accentual unit (`stems=pros=encl|procl`);
    /// a word without the mark is tonic.
    pub fn prosody(&self) -> Prosody {
        match self.stem_value("pros") {
            Some("encl") => Prosody::Enclitic,
            Some("procl") => Prosody::Proclitic,
            _ => Prosody::Tonic,
        }
    }

    /// A closed-class word's subcategory (`prep`, `conj`, `part`, `adv`,
    /// `advpro`, `intj`, `pred`): its class column.
    /// The weight a source's count gave a variant form (`×n`), 0 when
    /// none is recorded (the primary and the class's alternatives).
    pub fn variant_weight(&self, cell: Cell, print: &str) -> u32 {
        self.variant_weights.iter().find(|(c, f, _)| *c == cell && f == print).map(|(_, _, n)| *n).unwrap_or(0)
    }

    pub fn subcategory(&self) -> Option<&str> {
        (self.pos == Pos::Closed && !self.class.is_empty() && self.class != "0").then_some(self.class.as_str())
    }
}

/// The guesser's map: (part of speech, lemma ending) → class → votes.
pub(crate) type EndingVotes = HashMap<(Pos, String), HashMap<&'static str, usize>>;

pub struct Lexicon {
    recension: Recension,
    lexemes: Vec<Lexeme>,
    by_id: HashMap<String, usize>,
    by_key: HashMap<(String, Pos), Vec<usize>>,
    index: crate::analyze::IndexSlot,
    /// The guesser's ending → class map, built on first use.
    endings: std::sync::OnceLock<EndingVotes>,
}

/// The embedded files, by recension: (pos, text).
const SYN_FILES: [(Pos, &str); 5] = [
    (Pos::Noun, include_str!("../lexicon/syn/nouns.tsv")),
    (Pos::Adjective, include_str!("../lexicon/syn/adjectives.tsv")),
    (Pos::Verb, include_str!("../lexicon/syn/verbs.tsv")),
    (Pos::Pronoun, include_str!("../lexicon/syn/pronouns.tsv")),
    (Pos::Closed, include_str!("../lexicon/syn/closed.tsv")),
];
#[cfg(feature = "ocs")]
const OCS_FILES: [(Pos, &str); 4] = [
    (Pos::Noun, include_str!("../lexicon/ocs/nouns.tsv")),
    (Pos::Adjective, include_str!("../lexicon/ocs/adjectives.tsv")),
    (Pos::Verb, include_str!("../lexicon/ocs/verbs.tsv")),
    (Pos::Pronoun, include_str!("../lexicon/ocs/pronouns.tsv")),
];

impl Lexicon {
    /// The immutable profile used by this lexicon and its indices.
    pub fn recension(&self) -> Recension {
        self.recension
    }

    /// The Synodal lexicon, parsed once on first use (about 0.1 s; the
    /// analyzer's index is built on the first `analyze`, see there).
    ///
    /// ```
    /// use church_slavonic::{Lexicon, Recension};
    /// let syn = Lexicon::synodal();
    /// assert_eq!(syn.recension(), Recension::Synodal);
    /// assert!(syn.len() > 30_000);
    /// ```
    pub fn synodal() -> &'static Lexicon {
        static L: OnceLock<Lexicon> = OnceLock::new();
        L.get_or_init(|| Lexicon::from_files(Recension::Synodal, &SYN_FILES))
    }

    /// The Old Church Slavonic lexicon, parsed once (the `ocs` feature).
    ///
    /// ```
    /// use church_slavonic::{Lexicon, Pos, Cell, Case, Number, Recension};
    /// let rab = Lexicon::ocs().find("рабъ", Pos::Noun)[0];
    /// let loc = rab.inflect(Cell::noun(Case::Locative, Number::Plural)).unwrap();
    /// assert_eq!(loc.print(Recension::OldChurchSlavonic), "рабѣхъ");
    /// ```
    #[cfg(feature = "ocs")]
    pub fn ocs() -> &'static Lexicon {
        static L: OnceLock<Lexicon> = OnceLock::new();
        L.get_or_init(|| Lexicon::from_files(Recension::OldChurchSlavonic, &OCS_FILES))
    }

    /// The lexicon of a recension. Old Church Slavonic needs the `ocs`
    /// feature; without it this panics for that recension.
    pub fn of(recension: Recension) -> &'static Lexicon {
        match recension {
            Recension::Synodal => Lexicon::synodal(),
            #[cfg(feature = "ocs")]
            Recension::OldChurchSlavonic => Lexicon::ocs(),
            #[cfg(not(feature = "ocs"))]
            Recension::OldChurchSlavonic => panic!("the Old Church Slavonic lexicon needs the `ocs` feature of church-slavonic"),
        }
    }

    fn from_files(recension: Recension, files: &[(Pos, &str)]) -> Lexicon {
        let mut lexemes = Vec::new();
        for (pos, text) in files {
            match parse_in(text, *pos, recension) {
                Ok(mut parsed) => lexemes.append(&mut parsed),
                Err(e) => panic!("lexicon/{}: {e}", pos.tag()),
            }
        }
        Lexicon::from_lexemes(recension, lexemes)
    }

    /// Build a lexicon from parsed lexemes (the tools crate builds
    /// candidate lexicons this way before writing them).
    pub fn from_lexemes(recension: Recension, lexemes: Vec<Lexeme>) -> Lexicon {
        Self::try_from_lexemes(recension, lexemes).expect("invalid trusted lexicon; use try_from_lexemes for external data")
    }

    /// Validate all entries before exposing a snapshot. Lexemes are owned by
    /// the snapshot and can only be borrowed immutably after construction.
    pub fn try_from_lexemes(recension: Recension, lexemes: Vec<Lexeme>) -> Result<Lexicon, crate::error::LexiconBuildError> {
        use crate::error::{LexiconBuildError, LexiconBuildProblem as Problem};
        use std::collections::HashSet;
        let mut by_id = HashMap::with_capacity(lexemes.len());
        let mut by_key: HashMap<(String, Pos), Vec<usize>> = HashMap::new();
        for (i, l) in lexemes.iter().enumerate() {
            let error = |problem| LexiconBuildError { lexeme_id: l.id.clone(), problem };
            if l.id.is_empty() || l.id == "-" {
                return Err(error(Problem::EmptyIdentity));
            }
            if by_id.insert(l.id.clone(), i).is_some() {
                return Err(error(Problem::DuplicateIdentity));
            }
            if l.recension != recension {
                return Err(error(Problem::MixedProfile));
            }
            crate::stress::StressSpec::parse(&l.stress, l.pos)
                .map_err(|e| error(Problem::InvalidStress(e)))?;
            // Empty/0 explicitly represent inherited unknown class information.
            // They do not license a generated paradigm.
            if l.pos != Pos::Closed && !l.class.is_empty() && l.class != "0" && l.class().is_none() {
                return Err(error(Problem::UnknownClass(l.class.clone())));
            }
            let mut stems = HashSet::new();
            for (key, _) in &l.stems {
                if !stems.insert(key) { return Err(error(Problem::DuplicateField(format!("stem:{key}")))); }
            }
            let mut overrides = HashSet::new();
            for (cell, _) in &l.overrides {
                if cell.pos() != l.pos { return Err(error(Problem::WrongCell(*cell))); }
                if !overrides.insert(cell) { return Err(error(Problem::DuplicateField(format!("override:{}", cell.name())))); }
            }
            // Variant groups are additive, unlike singleton overrides. Existing
            // consumers intentionally concatenate repeated groups for a cell.
            for (cell, _) in &l.variants {
                if cell.pos() != l.pos { return Err(error(Problem::WrongCell(*cell))); }
            }
            let mut weights = HashSet::new();
            for (cell, form, _) in &l.variant_weights {
                if cell.pos() != l.pos { return Err(error(Problem::WrongCell(*cell))); }
                if !weights.insert((cell, form)) {
                    return Err(error(Problem::DuplicateField(format!("weight:{}:{form}", cell.name()))));
                }
            }
            by_key.entry((comparison_key(&l.lemma), l.pos)).or_default().push(i);
        }
        Ok(Lexicon { recension, lexemes, by_id, by_key, index: crate::analyze::IndexSlot::new(), endings: std::sync::OnceLock::new() })
    }

    pub(crate) fn index_cell(&self) -> &crate::analyze::IndexSlot {
        &self.index
    }

    pub(crate) fn ending_slot(&self) -> &std::sync::OnceLock<EndingVotes> {
        &self.endings
    }

    pub(crate) fn lexeme_at(&self, i: usize) -> &Lexeme {
        &self.lexemes[i]
    }

    /// A lexeme by its id (`рабъ.n`, `рещи.v`, `той.pron`, `и.x.2`). Ids
    /// are stable across releases; a consumer may persist them.
    ///
    /// ```
    /// use church_slavonic::Lexicon;
    /// let rab = Lexicon::synodal().get("рабъ.n").unwrap();
    /// assert_eq!(rab.lemma, "ра́бъ");
    /// assert!(Lexicon::synodal().get("no.such").is_none());
    /// ```
    pub fn get(&self, id: &str) -> Option<&Lexeme> {
        self.by_id.get(id).map(|&i| &self.lexemes[i])
    }

    /// Every lexeme whose lemma matches, accent-tolerant; homographs come
    /// back together.
    /// The lexemes of a lemma and part of speech, accents ignored
    /// (homographs come back together: во́лна wool, волна̀ wave).
    ///
    /// ```
    /// use church_slavonic::{Lexicon, Pos};
    /// let found = Lexicon::synodal().find("рещѝ", Pos::Verb);
    /// assert_eq!(found[0].id, "рещи.v");
    /// ```
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
/// accepted; `#` lines and blank lines are skipped. A malformed line is
/// a [`LexiconError`] with its line number.
pub fn parse(text: &str, pos: Pos) -> Result<Vec<Lexeme>, LexiconError> {
    parse_in(text, pos, Recension::Synodal)
}

/// [`parse`] for a recension's lexicon file.
pub fn parse_in(text: &str, pos: Pos, recension: Recension) -> Result<Vec<Lexeme>, LexiconError> {
    parse_lines(text, pos, recension).map_err(|message| {
        // every message the parser writes begins «line N: »
        let (line, rest) = message
            .strip_prefix("line ")
            .and_then(|r| r.split_once(": "))
            .and_then(|(n, rest)| n.parse::<usize>().ok().map(|n| (n, rest.to_string())))
            .unwrap_or((0, message.clone()));
        LexiconError { line, message: rest }
    })
}

fn parse_lines(text: &str, pos: Pos, recension: Recension) -> Result<Vec<Lexeme>, String> {
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
            Cell::parse(pos, s).map_err(|_| format!("line {line_no}: cell {s}"))
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
        let mut weights_v = Vec::new();
        if !empty(variants) {
            for item in variants.split(';') {
                let (k, v) = item
                    .split_once('=')
                    .ok_or_else(|| format!("line {line_no}: variants item {item}"))?;
                let cell = parse_cell(k)?;
                let mut forms = Vec::new();
                for form in v.split('|') {
                    match form.split_once('×') {
                        Some((f, n)) => {
                            let n: u32 = n.parse().map_err(|_| format!("line {line_no}: variant weight {form}"))?;
                            forms.push(f.to_string());
                            weights_v.push((cell, f.to_string(), n));
                        }
                        None => forms.push(form.to_string()),
                    }
                }
                variants_v.push((cell, forms));
            }
        }
        let src_v: Vec<String> = if empty(src) {
            Vec::new()
        } else {
            src.split(';').map(str::to_string).collect()
        };
        crate::stress::StressSpec::parse(stress, pos)
            .map_err(|e| format!("line {line_no}: stress: {e}"))?;
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
            variant_weights: weights_v,
            src: src_v,
            note: if empty(note) { String::new() } else { note.to_string() },
            provenance: Provenance::LexiconEntry,
            recension,
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
            .map(|(c, v)| {
                let forms: Vec<String> = v
                    .iter()
                    .map(|f| match l.variant_weight(*c, f) {
                        0 => f.clone(),
                        n => format!("{f}×{n}"),
                    })
                    .collect();
                format!("{}={}", c.name(), forms.join("|"))
            })
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
        // a variant's weight (`×n`) parses and formats back
        let weighted = "id\tlemma\tpos\tgender\tanim\tclass\tstress\tstems\toverrides\tvariants\tsrc\tnote\n\
            ѻвца.n\tѻ҆вца̀\tn\tf\tanim\tN3c\tb\t-\t-\tacc.sg=ѻ҆́вцꙋ×14|ѻ҆вцꙋ́\tP:N3c\t-\n";
        let w = parse(weighted, Pos::Noun).expect("parses");
        let acc = Cell::parse(Pos::Noun, "acc.sg").expect("cell");
        assert_eq!(w[0].variant_weight(acc, "ѻ҆́вцꙋ"), 14);
        assert_eq!(w[0].variant_weight(acc, "ѻ҆вцꙋ́"), 0);
        assert!(format(&w).contains("acc.sg=ѻ҆́вцꙋ×14|ѻ҆вцꙋ́"), "{}", format(&w));
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
        #[cfg(feature = "ocs")]
        let _ = Lexicon::ocs();
    }
}
