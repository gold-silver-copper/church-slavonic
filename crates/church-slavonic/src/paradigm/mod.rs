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
//!   (a suffix on a derivation: `ext:ен`, `ext:т:drop`), `cut` (base minus
//!   its last letter), `iot[:x]` (iotation: люб -> любл, род -> рожд),
//!   `ov` (-ова -> -ꙋ), `nasal` (a final vowel -> н), `iota` (a final и
//!   -> і); a lexeme's `stems=base=…` replaces the strip rule's base and
//!   `stems=<n>=…` spells stem n outright;
//! - a block column (`short.comp`, `part.pres.act.long`) gives the spec of
//!   every cell of the block without a column of its own; `N~C` declines
//!   stem N as adjective class C in the cell's adjective counterpart (a
//!   participle, a comparative);
//! - a cell spec is `|`-separated alternatives, primary first: `N-ending`
//!   (stem N plus the ending; a trailing `^` is the number mark), `@cell`
//!   (the same as that cell), `@lemma` (the lemma's own letters), each
//!   optionally prefixed `anim:` or `inan:` to apply to that animacy only.

use crate::cell::{Cell, Pos};
use crate::orthography::is_vowel_letter;
use std::collections::HashMap;
use std::sync::OnceLock;

pub mod adj;
pub mod closed;
pub mod noun;
pub mod ocs;
pub mod pronoun;
pub mod verb;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Derivation {
    Base,
    Drop,
    Insert(Box<Derivation>),
    /// Base minus its last letter (`знамені` -> `знамен` before `-ьми`).
    Cut,
    /// The tense jer before j: a stem-final и becomes ь, ы becomes ъ
    /// (пи → пь: пьѭ, пьѥши; ры → ръ: ръѭ).
    Jer,
    Pal1(Box<Derivation>),
    Pal2(Box<Derivation>),
    /// Iotation of the final consonant (`люб` -> `любл`, `род` -> `рожд`).
    Iot(Box<Derivation>),
    /// A suffix on a derivation (`ext:ен`, `ext:т:drop`).
    Ext(String, Box<Derivation>),
    /// `-ова`/`-ева` -> `-ꙋ` (`требова` -> `требꙋ`).
    Ov,
    /// The final vowel -> `н` (`мѧ` -> `мн`, `жа` -> `жн`).
    Nasal,
    /// A final `и` -> `і` before a vowel (`би` -> `бі`).
    Iota,
}

impl Derivation {
    fn parse(s: &str) -> Result<Derivation, String> {
        Ok(match s {
            "base" => Derivation::Base,
            "drop" => Derivation::Drop,
            "cut" => Derivation::Cut,
            "jer" => Derivation::Jer,
            "ov" => Derivation::Ov,
            "nasal" => Derivation::Nasal,
            "iota" => Derivation::Iota,
            _ => {
                if let Some(rest) = s.strip_prefix("insert") {
                    Derivation::Insert(Box::new(sub(rest)?))
                } else if let Some(rest) = s.strip_prefix("pal1") {
                    Derivation::Pal1(Box::new(sub(rest)?))
                } else if let Some(rest) = s.strip_prefix("pal2") {
                    Derivation::Pal2(Box::new(sub(rest)?))
                } else if let Some(rest) = s.strip_prefix("iot") {
                    Derivation::Iot(Box::new(sub(rest)?))
                } else if let Some(rest) = s.strip_prefix("ext:") {
                    let (suffix, inner) = match rest.split_once(':') {
                        Some((suffix, inner)) => (suffix, Derivation::parse(inner)?),
                        None => (rest, Derivation::Base),
                    };
                    Derivation::Ext(suffix.to_string(), Box::new(inner))
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
    /// Stem N declined as adjective class C in the cell's adjective
    /// counterpart (`4~A1s`: a participle or a comparative).
    Delegate { stem: u8, class: String },
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    /// The recension whose letters the derivations produce (the second
    /// palatalisation of г is з in the Synodal print, ѕ in OCS).
    pub recension: crate::grammar::Recension,
    pub exemplar: String,
    pub strip: usize,
    pub stems: Vec<(u8, Derivation)>,
    pub cells: HashMap<Cell, Vec<Alt>>,
    /// Block columns (`short.comp`, `part.pres.act.long`): the spec of
    /// every cell of the block that has no column of its own.
    pub blocks: HashMap<String, Vec<Alt>>,
    /// The cells in table order (the paradigm's iteration order),
    /// block-covered cells included.
    pub order: Vec<Cell>,
}

/// The letters of one alternative, with its number mark.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Letters {
    pub letters: String,
    pub mark: bool,
    /// How many vowels the stem contributed (the stress layer's boundary).
    pub stem_vowels: usize,
    /// How many vowels the stem had before the class's extension
    /// (`ext:им`, `ext:ѧщ`): the place `P` of the stress layer. Equal to
    /// `stem_vowels` on a stem the class did not extend.
    pub pre_vowels: usize,
    /// How many vowels the class's base stem has (the lemma minus its
    /// ending, before any derivation): the stress layer reads whether the
    /// lemma's stressed vowel was in the stem a derivation shortened.
    pub base_vowels: usize,
    /// Vowels of a solid enclitic or stressed tail at the end
    /// (`Form::mark_skip`).
    pub tail_vowels: u8,
    /// A stressed solid tail's stressed vowel (`stems=tail=на́десѧть`: the
    /// compound's one stress sits in the tail and the paradigm does not
    /// decide it), as an index over the whole word.
    pub tail_stress: Option<u8>,
}

/// What a class needs to know about the lexeme it declines.
pub struct Subject<'a> {
    /// The lemma's letters (marks stripped).
    pub lemma: &'a str,
    pub animate: Option<bool>,
    /// The lexeme's `stems` column.
    pub stems: &'a [(String, String)],
}

impl<'a> Subject<'a> {
    /// The enclitic (`stems=encl=сѧ`, `encl=же`, `encl=либо`) the print
    /// writes solid after every ending: a verb's reflexive particle, a
    /// pronoun's же/жде/ждо/либо.
    pub fn enclitic(&self) -> Option<&'a str> {
        self.stems.iter().find(|(k, _)| k == "encl").map(|(_, v)| v.as_str())
    }

    /// A stressed solid tail (`stems=tail=на́десѧть`): the second element
    /// of a compound numeral, written after every ending of the first and
    /// carrying the compound's one stress (первыйна́десѧть,
    /// первагѡна́десѧть) — a proclitic first element before a stressed
    /// host, not an enclitic. Returns the tail's letters and its stressed
    /// vowel's index within the tail.
    pub fn tail(&self) -> Option<(String, Option<u8>)> {
        self.stems.iter().find(|(k, _)| k == "tail").map(|(_, v)| {
            let f = crate::form::Form::from_print(v);
            (f.letters, f.stress)
        })
    }

    /// The subject with the enclitic or the tail stripped off the lemma:
    /// what the class table works on.
    pub fn core(&self) -> Subject<'a> {
        let lemma = match self.enclitic() {
            Some(r) => self.lemma.strip_suffix(r).unwrap_or(self.lemma),
            None => match self.tail() {
                Some((t, _)) => self.lemma.strip_suffix(t.as_str()).unwrap_or(self.lemma),
                None => self.lemma,
            },
        };
        Subject { lemma, animate: self.animate, stems: self.stems }
    }
}

impl Class {
    /// The numbered stems of a lexeme.
    pub fn stems_of(&self, subject: &Subject<'_>) -> HashMap<u8, String> {
        let subject = &subject.core();
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
            out.insert(*n, derive(derivation, &base, subject, self.recension));
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

    /// The vowel count of each numbered stem before the class's
    /// extension: for `n=ext:suffix:inner` the vowels of `inner`, for any
    /// other stem its own — the place `P` of the stress layer. A stem the
    /// lexeme spells itself (`stems=6=…`) has no extension the class knows.
    pub fn pre_vowels_of(&self, subject: &Subject<'_>, stems: &HashMap<u8, String>) -> HashMap<u8, usize> {
        self.vowels_of(subject, stems).1
    }

    /// The base stem's vowel count and [`Class::pre_vowels_of`].
    pub fn vowels_of(&self, subject: &Subject<'_>, stems: &HashMap<u8, String>) -> (usize, HashMap<u8, usize>) {
        let subject = &subject.core();
        let count = |s: &str| s.chars().filter(|c| is_vowel_letter(*c)).count();
        let mut out: HashMap<u8, usize> = stems.iter().map(|(n, s)| (*n, count(s))).collect();
        let spelled = |n: u8| subject.stems.iter().any(|(k, _)| k.parse::<u8>().ok() == Some(n));
        let base: String = match subject.stems.iter().find(|(k, _)| k == "base") {
            Some((_, b)) => b.clone(),
            None => {
                let k = subject.lemma.chars().count().saturating_sub(self.strip);
                subject.lemma.chars().take(k).collect()
            }
        };
        for (n, derivation) in &self.stems {
            let Derivation::Ext(_, inner) = derivation else { continue };
            if spelled(*n) {
                continue;
            }
            out.insert(*n, count(&derive(inner, &base, subject, self.recension)));
        }
        (count(&base), out)
    }

    /// Every alternative of `cell` for the subject, primary first; empty
    /// when the class has no such cell.
    pub fn letters(&self, cell: Cell, subject: &Subject<'_>) -> Vec<Letters> {
        let stems = self.stems_of(subject);
        self.letters_with(cell, subject, &stems)
    }

    /// [`Class::letters`] with the stems already derived (the index walks
    /// every cell of a lexeme: derive once).
    pub fn letters_with(&self, cell: Cell, subject: &Subject<'_>, stems: &HashMap<u8, String>) -> Vec<Letters> {
        // an enclitic (`stems=encl=сѧ`, `encl=же`): the class works on the
        // lemma without it, and the print writes it solid after every
        // ending, dropping the jer before it (бои́тсѧ, боѧ́хсѧ, боѧ́щихсѧ;
        // тогѡ́же, коегѡ́ждо)
        let refl = subject.enclitic();
        let tail = subject.tail();
        let (base, pre) = self.vowels_of(subject, stems);
        let subject = subject.core();
        let mut out = Vec::new();
        self.collect(cell, &subject, stems, &pre, None, base, &mut out, 0);
        // the print drops the jer before the enclitic (тѣ́мже, бои́тсѧ) and
        // before the tail (шестомна́десѧть); OCS keeps it (имъже, ихъже)
        let drop_jer = self.recension == crate::grammar::Recension::Synodal;
        if let Some(r) = refl {
            for l in &mut out {
                if drop_jer && l.letters.ends_with('ъ') {
                    l.letters.pop();
                }
                l.letters.push_str(r);
                l.tail_vowels = r.chars().filter(|c| is_vowel_letter(*c)).count().min(255) as u8;
            }
        } else if let Some((t, stress)) = tail {
            for l in &mut out {
                if drop_jer && l.letters.ends_with('ъ') {
                    l.letters.pop();
                }
                let host_vowels = l.letters.chars().filter(|c| is_vowel_letter(*c)).count();
                l.letters.push_str(&t);
                l.tail_vowels = t.chars().filter(|c| is_vowel_letter(*c)).count().min(255) as u8;
                l.tail_stress = stress.and_then(|k| u8::try_from(host_vowels + usize::from(k)).ok());
            }
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn collect(
        &self,
        cell: Cell,
        subject: &Subject<'_>,
        stems: &HashMap<u8, String>,
        pre: &HashMap<u8, usize>,
        // a delegating class's pre-extension count (the participle's stem
        // before -им-), which the adjective class declining it inherits
        inherited_pre: Option<usize>,
        // the base stem's vowels (the delegating class's, when delegated)
        base: usize,
        out: &mut Vec<Letters>,
        depth: usize,
    ) {
        if depth > 4 {
            return;
        }
        let alts = match self.cells.get(&cell) {
            Some(a) => a,
            None => match cell.block().and_then(|b| self.blocks.get(&b)) {
                Some(a) => a,
                None => return,
            },
        };
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
                Shape::Ending { stem: n, ending, mark } => {
                    if let Some(stem) = stems.get(n) {
                        // OCS spells the iotated vowel plain after a husher
                        // at the ending too (пиш + ѭ = пишѫ); the Synodal
                        // rule is a derivation's (ext), never an ending's
                        let ending = match self.recension {
                            crate::grammar::Recension::OldChurchSlavonic => after_husher(stem, ending, self.recension),
                            crate::grammar::Recension::Synodal => std::borrow::Cow::Borrowed(ending.as_str()),
                        };
                        let stem_vowels = stem.chars().filter(|c| is_vowel_letter(*c)).count();
                        out.push(Letters {
                            letters: format!("{stem}{ending}"),
                            mark: *mark,
                            stem_vowels,
                            pre_vowels: inherited_pre.or_else(|| pre.get(n).copied()).unwrap_or(stem_vowels),
                            base_vowels: base,
                            tail_vowels: 0,
                            tail_stress: None,
                        });
                    }
                }
                Shape::Ref(other) => self.collect(*other, subject, stems, pre, inherited_pre, base, out, depth + 1),
                Shape::Delegate { stem: n, class } => {
                    let (Some(stem), Some(adj_cell)) = (stems.get(n), cell.as_adjective()) else { continue };
                    let Some(adjective) = table_of(Pos::Adjective, self.recension).get(class) else { continue };
                    // the delegate's lemma: the stem plus the class's own
                    // ending letters, so its base is exactly the stem
                    let tail: String = {
                        let ex: Vec<char> = adjective.exemplar.chars().collect();
                        ex[ex.len().saturating_sub(adjective.strip)..].iter().collect()
                    };
                    let lemma = format!("{stem}{tail}");
                    let inner = Subject { lemma: &lemma, animate: subject.animate, stems: &[] };
                    let inner_stems = adjective.stems_of(&inner);
                    let (_, inner_pre) = adjective.vowels_of(&inner, &inner_stems);
                    let handed = inherited_pre.or_else(|| pre.get(n).copied());
                    adjective.collect(Cell::Adj(adj_cell), &inner, &inner_stems, &inner_pre, handed, base, out, depth + 1);
                }
                Shape::Lemma => {
                    let stem_vowels = subject.lemma.chars().filter(|c| is_vowel_letter(*c)).count();
                    out.push(Letters { letters: subject.lemma.to_string(), mark: false, stem_vowels, pre_vowels: stem_vowels, base_vowels: stem_vowels, tail_vowels: 0, tail_stress: None })
                }
            }
        }
    }

    pub fn has(&self, cell: Cell) -> bool {
        self.cells.contains_key(&cell) || cell.block().is_some_and(|b| self.blocks.contains_key(&b))
    }
}

fn default_animacy(subject: &Subject<'_>) -> bool {
    // neuters are inanimate; the rest animate (the measured 1.x default)
    !matches!(subject.lemma.chars().last(), Some('о' | 'е' | 'ѧ'))
}

fn derive(d: &Derivation, base: &str, subject: &Subject<'_>, recension: crate::grammar::Recension) -> String {
    match d {
        Derivation::Base => base.to_string(),
        Derivation::Drop => drop_fleeting(base),
        Derivation::Cut => {
            let n = base.chars().count().saturating_sub(1);
            base.chars().take(n).collect()
        }
        Derivation::Jer => {
            let mut chars: Vec<char> = base.chars().collect();
            match chars.last_mut() {
                Some(c @ 'и') => *c = 'ь',
                Some(c @ 'ы') => *c = 'ъ',
                _ => {}
            }
            chars.into_iter().collect()
        }
        Derivation::Insert(inner) => subject
            .stems
            .iter()
            .find(|(k, _)| k == "ins")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| insert_fleeting(&derive(inner, base, subject, recension))),
        Derivation::Pal1(inner) => palatalise_in(&derive(inner, base, subject, recension), true, recension),
        Derivation::Pal2(inner) => palatalise_in(&derive(inner, base, subject, recension), false, recension),
        Derivation::Iot(inner) => iotate(&derive(inner, base, subject, recension)),
        Derivation::Ext(suffix, inner) => {
            let stem = derive(inner, base, subject, recension);
            let suffix = after_husher(&stem, suffix, recension);
            format!("{stem}{suffix}")
        }
        Derivation::Ov => {
            // -ова- → -ꙋ- (вѣрꙋю), -ева- → -ю- (воюю, оу҆треню́ю; -ꙋ- after
            // a husher: ночꙋ́ю)
            let (s, soft) = match base.strip_suffix("ова") {
                Some(s) => (s, false),
                None => match base.strip_suffix("ева") {
                    Some(s) => (s, true),
                    None => (base, false),
                },
            };
            let last = s.chars().last();
            let husher = matches!(last, Some('ж' | 'ч' | 'ш' | 'щ' | 'ц'));
            let ju = (soft && !husher) || last.is_some_and(is_vowel_letter);
            format!("{s}{}", if ju { 'ю' } else { 'ꙋ' })
        }
        Derivation::Nasal => {
            let n = base.chars().count().saturating_sub(1);
            let head: String = base.chars().take(n).collect();
            format!("{head}н")
        }
        Derivation::Iota => match base.strip_suffix('и') {
            Some(head) => format!("{head}і"),
            None => base.to_string(),
        },
    }
}

/// The spelling of a front or iotated vowel after a husher, at the
/// boundary of a stem and what follows it. Synodal: a husher takes а, not
/// ѧ/ѣ (ѻ҆троча̀ : ѻ҆троча́та; вели́чайшій). OCS: after ж ч ш щ ц (and жд,
/// the iotation of д) the iotated vowels are written plain — ѭ as ѫ, ѥ as
/// е, ѩ as ѧ, ꙗ as а (пишѫ, пишетъ, рождѫ, хождаахъ; beside люблѭ,
/// глаголѥтъ, гонꙗахъ), which is what lets one class name the ending
/// once (`2-ѭ`) for every present stem the class derives.
fn after_husher<'s>(stem: &str, suffix: &'s str, recension: crate::grammar::Recension) -> std::borrow::Cow<'s, str> {
    use std::borrow::Cow;
    match recension {
        crate::grammar::Recension::Synodal => {
            let husher = matches!(stem.chars().last(), Some('ж' | 'ч' | 'ш' | 'щ'));
            match suffix.strip_prefix(['ѧ', 'ѣ']) {
                Some(rest) if husher => Cow::Owned(format!("а{rest}")),
                _ => Cow::Borrowed(suffix),
            }
        }
        crate::grammar::Recension::OldChurchSlavonic => {
            let husher = matches!(stem.chars().last(), Some('ж' | 'ч' | 'ш' | 'щ' | 'ц')) || stem.ends_with("жд");
            if !husher {
                return Cow::Borrowed(suffix);
            }
            let mut chars = suffix.chars();
            let plain = match chars.next() {
                Some('ѭ') => 'ѫ',
                Some('ѥ') => 'е',
                Some('ѩ') => 'ѧ',
                Some('ꙗ') => 'а',
                _ => return Cow::Borrowed(suffix),
            };
            Cow::Owned(format!("{plain}{}", chars.as_str()))
        }
    }
}

/// Iotation of a stem's final consonant(s): the labials take л, the
/// dentals and velars their hushers (`люб` -> `любл`, `род` -> `рожд`,
/// `свѣт` -> `свѣщ`, `пис` -> `пиш`, `маз` -> `маж`, `алк` -> `алч`,
/// `мысл` -> `мышл`, `пꙋст` -> `пꙋщ`).
pub fn iotate(stem: &str) -> String {
    let chars: Vec<char> = stem.chars().collect();
    let n = chars.len();
    if n == 0 {
        return String::new();
    }
    // iotation is vacuous on a stem already palatal (дъжд-, blaž-)
    if matches!(chars[n - 1], 'ж' | 'ш' | 'щ' | 'ч') || stem.ends_with("жд") {
        return stem.to_string();
    }
    let head: String = chars[..n - 1].iter().collect();
    let last = chars[n - 1];
    // two-letter clusters first
    if n >= 2 {
        let pair: String = chars[n - 2..].iter().collect();
        let replaced = match pair.as_str() {
            "ст" => Some("щ"),
            "ск" => Some("щ"),
            "сл" => Some("шл"),
            "зд" => Some("жд"),
            _ => None,
        };
        if let Some(r) = replaced {
            let head2: String = chars[..n - 2].iter().collect();
            return format!("{head2}{r}");
        }
    }
    let replaced = match last {
        'б' => "бл",
        'п' => "пл",
        'в' => "вл",
        'м' => "мл",
        'ф' => "фл",
        'д' => "жд",
        'т' => "щ",
        'з' => "ж",
        'с' => "ш",
        'к' => "ч",
        'г' => "ж",
        'х' => "ш",
        'ц' => "ч",
        _ => return stem.to_string(),
    };
    format!("{head}{replaced}")
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
    palatalise_in(stem, first, crate::grammar::Recension::Synodal)
}

/// [`palatalise`] in a recension: OCS writes ѕ for the second
/// palatalisation of г (дроуѕѣ), the Synodal print з (дрꙋзѣ).
pub fn palatalise_in(stem: &str, first: bool, recension: crate::grammar::Recension) -> String {
    // the -ск- stems: ск -> ст before ѣ/и (ага́рѧнстїи)
    if !first && let Some(head) = stem.strip_suffix("ск") {
        return format!("{head}ст");
    }
    let ocs = recension == crate::grammar::Recension::OldChurchSlavonic;
    let mut chars: Vec<char> = stem.chars().collect();
    if let Some(last) = chars.last_mut() {
        *last = match (*last, first) {
            ('к', true) => 'ч',
            ('г', true) => 'ж',
            ('х', true) => 'ш',
            ('ц', true) => 'ч',
            ('к', false) => 'ц',
            ('г', false) if ocs => 'ѕ',
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
            recension: crate::grammar::Recension::Synodal,
            name: cols[0].to_string(),
            exemplar: cols[1].to_string(),
            strip: cols[2].parse().map_err(|_| format!("line {line_no}: strip {}", cols[2]))?,
            stems: Vec::new(),
            cells: HashMap::new(),
            blocks: HashMap::new(),
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
            let cell = Cell::parse(pos, name);
            if cell.is_none() && !is_block_name(pos, name) {
                return Err(format!("line {line_no}: cell {name}"));
            }
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
                } else if let Some((stem, class)) = rest.split_once('~') {
                    let stem: u8 = stem.parse().map_err(|_| format!("line {line_no}: delegate stem {stem}"))?;
                    Shape::Delegate { stem, class: class.to_string() }
                } else {
                    let (stem, ending) =
                        rest.split_once('-').ok_or_else(|| format!("line {line_no}: alternative {rest}"))?;
                    let stem: u8 = stem.parse().map_err(|_| format!("line {line_no}: stem {stem}"))?;
                    let mark = ending.ends_with('^');
                    Shape::Ending { stem, ending: ending.trim_end_matches('^').to_string(), mark }
                };
                alts.push(Alt { animacy, shape });
            }
            match cell {
                Some(cell) => {
                    class.cells.insert(cell, alts);
                    class.order.push(cell);
                }
                None => {
                    class.blocks.insert(name.clone(), alts);
                }
            }
        }
        // block-covered cells join the order after the explicit ones
        for cell in all_cells(pos) {
            if !class.cells.contains_key(&cell) && cell.block().is_some_and(|b| class.blocks.contains_key(&b)) {
                class.order.push(cell);
            }
        }
        out.push(class);
    }
    Ok(out)
}

/// Is `name` a block column of the part of speech?
fn is_block_name(pos: Pos, name: &str) -> bool {
    match pos {
        Pos::Adjective => matches!(name, "short.pos" | "long.pos" | "short.comp" | "long.comp"),
        Pos::Verb => name.starts_with("part.") && name.matches('.').count() == 3,
        _ => false,
    }
}

/// Every cell a class of the part of speech might declare.
fn all_cells(pos: Pos) -> Vec<Cell> {
    match pos {
        Pos::Adjective => crate::cell::AdjCell::all().map(Cell::Adj).collect(),
        Pos::Verb => crate::cell::VerbCell::participles().map(Cell::Verb).collect(),
        _ => Vec::new(),
    }
}

/// A parsed class table with lookup by name.
pub struct Table {
    classes: Vec<Class>,
    by_name: HashMap<String, usize>,
}

impl Table {
    pub fn parse(text: &str, pos: Pos) -> Result<Table, String> {
        Table::parse_in(text, pos, crate::grammar::Recension::Synodal)
    }

    /// [`Table::parse`] for a recension's table.
    pub fn parse_in(text: &str, pos: Pos, recension: crate::grammar::Recension) -> Result<Table, String> {
        let mut classes = parse_table(text, pos)?;
        for c in &mut classes {
            c.recension = recension;
        }
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
    table_of(pos, crate::grammar::Recension::Synodal)
}

/// The class table of a part of speech in a recension: the Synodal
/// tables under `classes/`, the Old Church Slavonic ones under
/// `classes/ocs/` (seeded from Kaikki's own paradigms).
pub fn table_of(pos: Pos, recension: crate::grammar::Recension) -> &'static Table {
    use crate::grammar::Recension::{OldChurchSlavonic, Synodal};
    static SYN: [OnceLock<Table>; 4] = [OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new()];
    static OCS: [OnceLock<Table>; 4] = [OnceLock::new(), OnceLock::new(), OnceLock::new(), OnceLock::new()];
    static CLOSED: OnceLock<Table> = OnceLock::new();
    let (slot, text, name) = match (pos, recension) {
        (Pos::Noun, Synodal) => (&SYN[0], noun::TABLE, "classes/noun.tsv"),
        (Pos::Adjective, Synodal) => (&SYN[1], adj::TABLE, "classes/adj.tsv"),
        (Pos::Verb, Synodal) => (&SYN[2], verb::TABLE, "classes/verb.tsv"),
        (Pos::Pronoun, Synodal) => (&SYN[3], pronoun::TABLE, "classes/pronoun.tsv"),
        (Pos::Noun, OldChurchSlavonic) => (&OCS[0], ocs::NOUN, "classes/ocs/noun.tsv"),
        (Pos::Adjective, OldChurchSlavonic) => (&OCS[1], ocs::ADJ, "classes/ocs/adj.tsv"),
        (Pos::Verb, OldChurchSlavonic) => (&OCS[2], ocs::VERB, "classes/ocs/verb.tsv"),
        (Pos::Pronoun, OldChurchSlavonic) => (&OCS[3], ocs::PRONOUN, "classes/ocs/pronoun.tsv"),
        (Pos::Closed, _) => (&CLOSED, closed::TABLE, "classes/closed.tsv"),
    };
    slot.get_or_init(|| Table::parse_in(text, pos, recension).unwrap_or_else(|e| panic!("{name}: {e}")))
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
