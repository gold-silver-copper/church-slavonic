//! The analyzer: a printed word back to (lexeme, cell). The index is
//! every lexeme × every cell, every alternative and variant included,
//! keyed by [`Form::key`] (accent-blind, typographic letter pairs folded)
//! and built lazily on first use; a query folds the input by the same key
//! and ranks exact-print matches first, then primaries before other
//! alternatives, then lexicon order. Ambiguity is returned, never
//! resolved. An unknown surface returns nothing.

use crate::cell::Cell;
use crate::form::Form;
use crate::lexicon::{Lexeme, Lexicon};
use crate::orthography::comparison_key;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

/// One reading of a surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis<'a> {
    pub lexeme: &'a Lexeme,
    pub cell: Cell,
    /// Which of the cell's forms matched: 0 the primary (`inflect`), else
    /// the index into `forms(cell)`.
    pub alt: usize,
    /// The surface equals the form's print byte-for-byte (after NFC).
    pub exact: bool,
    /// The form as the lexicon prints it.
    pub print: String,
}

#[derive(Debug)]
struct Entry {
    key: String,
    lexeme: u32,
    cell: Cell,
    alt: u8,
    print: String,
}

/// The sorted index over a lexicon.
#[derive(Debug, Default)]
pub struct Index {
    entries: Vec<Entry>,
}

impl Index {
    fn build(lexicon: &Lexicon) -> Index {
        let timing = std::env::var_os("CS_INDEX_TIMING").is_some();
        let started = std::time::Instant::now();
        // every lexeme's paradigm, generated in parallel chunks (the
        // Synodal lexicon is ~7.8 million forms)
        let lexemes: Vec<&Lexeme> = lexicon.iter().collect();
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).clamp(1, 16);
        let chunk = lexemes.len().div_ceil(threads).max(1);
        let mut entries: Vec<Entry> = std::thread::scope(|scope| {
            let handles: Vec<_> = lexemes
                .chunks(chunk)
                .enumerate()
                .map(|(c, slice)| {
                    scope.spawn(move || {
                        let mut out = Vec::new();
                        for (j, lexeme) in slice.iter().enumerate() {
                            let i = c * chunk + j;
                            for (cell, forms) in lexeme.all_forms() {
                                for (alt, (_, print)) in forms.into_iter().enumerate() {
                                    out.push(Entry { key: comparison_key(&print), lexeme: i as u32, cell, alt: alt.min(255) as u8, print });
                                }
                            }
                        }
                        out
                    })
                })
                .collect();
            handles.into_iter().flat_map(|h| h.join().expect("index chunk")).collect()
        });
        if timing {
            eprintln!("index: {} entries generated in {:.2?}", entries.len(), started.elapsed());
        }
        entries.sort_by(|a, b| a.key.cmp(&b.key).then(a.lexeme.cmp(&b.lexeme)).then(a.alt.cmp(&b.alt)));
        if timing {
            eprintln!("index: sorted in {:.2?}", started.elapsed());
        }
        Index { entries }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Lexicon {
    /// The index, built on first use.
    pub fn index(&self) -> &Index {
        self.index_cell().get_or_init(|| Index::build(self))
    }

    /// Every reading of a printed word, ranked: exact print first, then
    /// the primary form before other alternatives, then lexicon order.
    pub fn analyze(&self, surface: &str) -> Vec<Analysis<'_>> {
        let key = comparison_key(surface);
        let surface: String = surface.nfc().collect::<String>().to_lowercase();
        let entries = &self.index().entries;
        let start = entries.partition_point(|e| e.key < key);
        let mut out: Vec<Analysis<'_>> = entries[start..]
            .iter()
            .take_while(|e| e.key == key)
            .map(|e| Analysis {
                lexeme: self.lexeme_at(e.lexeme as usize),
                cell: e.cell,
                alt: usize::from(e.alt),
                exact: e.print.nfc().collect::<String>() == surface,
                print: e.print.clone(),
            })
            .collect();
        out.sort_by(|a, b| b.exact.cmp(&a.exact).then(a.alt.cmp(&b.alt)));
        out
    }
}

/// The analyses of a surface grouped by (lexeme, print): one lexeme and
/// every cell whose form prints this surface, with the alternative index
/// of each. A surface with one exact reading whose `cells` has several
/// members is syncretism (the paradigm does not tell the cells apart);
/// several readings are homonymy (several lexemes, or several prints of
/// one lexeme).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reading<'a> {
    pub lexeme: &'a Lexeme,
    /// The cells printing this surface, in cell order, each with which
    /// of its forms matched (`0` the primary).
    pub cells: Vec<(Cell, usize)>,
    pub exact: bool,
    pub print: String,
}

impl Reading<'_> {
    /// The cells as an underspecified cell (`None` only for a closed-class
    /// word, whose one cell is [`Cell::Word`]).
    pub fn cell_set(&self) -> Option<crate::cell::CellSet> {
        crate::cell::CellSet::new(self.cells.iter().map(|(c, _)| *c).collect())
    }
}

impl Lexicon {
    /// [`Lexicon::analyze`] grouped by (lexeme, print), exact readings
    /// first; within a reading the cells are sorted and each carries the
    /// lowest alternative index that printed the surface.
    pub fn readings(&self, surface: &str) -> Vec<Reading<'_>> {
        let mut out: Vec<Reading<'_>> = Vec::new();
        for a in self.analyze(surface) {
            match out.iter_mut().find(|r| std::ptr::eq(r.lexeme, a.lexeme) && r.print == a.print) {
                Some(r) => match r.cells.iter_mut().find(|(c, _)| *c == a.cell) {
                    Some((_, alt)) => *alt = (*alt).min(a.alt),
                    None => r.cells.push((a.cell, a.alt)),
                },
                None => out.push(Reading { lexeme: a.lexeme, cells: vec![(a.cell, a.alt)], exact: a.exact, print: a.print }),
            }
        }
        for r in &mut out {
            r.cells.sort();
        }
        out
    }
}

/// The per-lexicon index slot (kept out of `Lexicon`'s public fields).
pub(crate) type IndexSlot = OnceLock<Index>;

/// Recover the Form of an analysis (for callers that want the layers).
impl Analysis<'_> {
    pub fn form(&self) -> Form {
        Form::from_print(&self.print)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Pos;
    use crate::grammar::Recension;
    use crate::lexicon::parse;

    fn lexicon() -> Lexicon {
        let text = "id\tlemma\tpos\tgender\tanim\tclass\tstress\tstems\toverrides\tvariants\tsrc\tnote\n\
                    рабъ.n\tра́бъ\tn\tm\tanim\tN1t\tb{voc.sg=S}\t-\t-\tgen.pl=ра̑бъ\tP:N1t\t-\n\
                    градъ.n\tгра́дъ\tn\tm\tinan\tN1t\ta\t-\t-\t-\tP:N1t\t-\n\
                    свѣтъ.n\tсвѣ́тъ\tn\tm\tinan\tN1t\ta\t-\t-\t-\tP:N1t\t-\n";
        Lexicon::from_lexemes(Recension::Synodal, parse(text, Pos::Noun).expect("parses"))
    }

    #[test]
    fn readings_are_ranked_and_ambiguity_is_kept() {
        let lex = lexicon();
        // рабѡ́мъ (dat.pl) and рабо́мъ (ins.sg) share the accent-blind key:
        // both come back, the exact print first
        let a = lex.analyze("рабѡ́мъ");
        assert_eq!(a.len(), 2, "{a:?}");
        assert_eq!(a[0].lexeme.id, "рабъ.n");
        assert_eq!(a[0].cell.name(), "dat.pl");
        assert!(a[0].exact && !a[1].exact);
        // accent-blind input still resolves, inexactly
        let b = lex.analyze("рабомъ");
        assert_eq!(b.len(), 2, "{b:?}"); // ins.sg and dat.pl
        assert!(b.iter().all(|x| !x.exact));
        // a variant is reachable and ranked after the primary
        let c = lex.analyze("ра̑бъ");
        assert!(c.iter().any(|x| x.cell.name() == "gen.pl" && x.alt > 0), "{c:?}");
        // nominative = accusative of an inanimate: two readings, never one
        let d = lex.analyze("свѣ́тъ");
        let cells: Vec<String> = d.iter().map(|x| x.cell.name()).collect();
        assert!(cells.contains(&"nom.sg".to_string()) && cells.contains(&"acc.sg".to_string()), "{cells:?}");
        assert!(lex.analyze("нѣ́что").is_empty());
        assert!(lex.index().len() > 60);
    }
}
