//! The sentence (4.1): what a word is *here*. `Sentence::parse` tokenizes
//! a verse and lifts every token to its readings — a word with one
//! lexeme (one cell, or the set its paradigm does not tell apart), a
//! function word, a phonological word (a host with its enclitics), a
//! titlo-written word, a capitalised word, punctuation, the apparatus, a
//! verbatim token, a token that is several lexemes (the readings kept,
//! none chosen). `Sentence::disambiguate` applies the constraint layer:
//! seven eliminations that name themselves on the leaf they narrowed
//! (`prep-gov`, `np-agree`, `subj-verb`, `voc-drop`, `one-subject`,
//! `bare-loc`, `bare-voc`), each 100% precise on the hand gold, none a
//! selection. `Sentence::print` is the round trip. A statistical choice
//! among what the rules leave is the tagger crate's, not the library's.

pub mod closed;
pub mod lift;
pub mod node;
pub mod rules;

use crate::Lexicon;
use crate::cell::CellSet;
use crate::grammar::Recension;
pub use lift::{Coverage, TokenFate};
pub use node::{Node, TreeError};
pub use rules::Stats;

/// A verse or a sentence of the print, lifted: the tree of its tokens.
#[derive(Clone)]
pub struct Sentence<'a> {
    lexicon: &'a Lexicon,
    tree: Node,
    coverage: Coverage,
}

/// One word of a sentence as the consumer reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The token as printed.
    pub surface: String,
    /// The lexeme's id and the cells that print the token, when the token
    /// is one lexeme (a function word's id with no cells).
    pub reading: Option<(String, Option<CellSet>)>,
    /// The rules that narrowed the reading (`prep-gov+tagger` …), the
    /// set they narrowed from.
    pub narrowed_by: Option<String>,
    pub narrowed_from: Option<String>,
    /// The token is several lexemes and none was chosen.
    pub ambiguous: bool,
}

impl<'a> Sentence<'a> {
    /// Tokenize and lift a verse: every token to what the lexicon says of
    /// it, nothing chosen.
    ///
    /// ```
    /// use church_slavonic::{Lexicon, sentence::Sentence, Recension};
    /// let mut s = Sentence::parse(Lexicon::synodal(), "И҆ ви́дѣ бг҃ъ свѣ́тъ, ꙗ҆́кѡ добро̀.");
    /// assert_eq!(s.print(Recension::Synodal).unwrap(), "И҆ ви́дѣ бг҃ъ свѣ́тъ, ꙗ҆́кѡ добро̀.");
    /// let stats = s.disambiguate();
    /// let words: Vec<_> = s.tokens();
    /// assert_eq!(words[1].reading.as_ref().unwrap().0, "видѣти.v"); // ви́дѣ: the aorist, not ви́дъ's locative
    /// assert_eq!(words[1].narrowed_by.as_deref(), Some("bare-loc"));
    /// assert_eq!(words[2].reading.as_ref().unwrap().1.as_ref().unwrap().name(), "nom.sg"); // бг҃ъ under its titlo row
    /// assert_eq!(words[3].narrowed_by.as_deref(), Some("one-subject")); // свѣ́тъ: nom|acc.sg → acc.sg, the subject being бг҃ъ
    /// assert_eq!(words[3].narrowed_from.as_deref(), Some("nom|acc.sg"));
    /// assert!(stats.by_rule.contains_key("one-subject"));
    /// ```
    pub fn parse(lexicon: &'a Lexicon, text: &str) -> Sentence<'a> {
        let lifter = lift::Lifter::new(lexicon);
        let (tree, coverage) = lifter.lift_verse(text);
        Sentence { lexicon, tree, coverage }
    }

    /// Apply the constraint layer; what each rule narrowed.
    pub fn disambiguate(&mut self) -> Stats {
        rules::disambiguate(&mut self.tree, self.lexicon)
    }

    /// The text back from the tree, byte for byte.
    pub fn print(&self, recension: Recension) -> Result<String, TreeError> {
        node::render(&self.tree, &recension)
    }

    /// The lift's coverage: how many tokens were analyzed, underspecified,
    /// closed, ambiguous, verbatim, apparatus.
    pub fn coverage(&self) -> &Coverage {
        &self.coverage
    }

    /// The tree itself (the tools' treebank writes it as an s-expression).
    pub fn tree(&self) -> &Node {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut Node {
        &mut self.tree
    }

    /// The words in order (punctuation left out), as the consumer reads
    /// them.
    pub fn tokens(&self) -> Vec<Token> {
        let mut out = Vec::new();
        collect(&self.tree, self.lexicon, &mut out);
        out
    }
}

fn collect(node: &Node, lexicon: &Lexicon, out: &mut Vec<Token>) {
    match node {
        Node::Group { children, .. } => {
            for c in children {
                collect(c, lexicon, out);
            }
        }
        Node::Punct(_) => {}
        Node::Pw { host, enclitics, .. } => {
            collect(host, lexicon, out);
            for e in enclitics {
                collect(e, lexicon, out);
            }
        }
        other => {
            let surface = node::render(other, &lexicon.recension).unwrap_or_default();
            let leaf = rules::leaf(other);
            let (reading, narrowed_by, narrowed_from) = match leaf {
                Some(Node::Lex { id, cells, notes, .. }) => (
                    Some((id.clone(), Some(cells.clone()))),
                    notes.iter().find(|(k, _)| k == "by").map(|(_, v)| v.clone()),
                    notes.iter().find(|(k, _)| k == "from").map(|(_, v)| v.clone()),
                ),
                _ => (rules::fn_word(other).map(|w| (w.to_string(), None)), None, None),
            };
            let ambiguous = rules::amb_surface(other).is_some();
            out.push(Token { surface, reading, narrowed_by, narrowed_from, ambiguous });
        }
    }
}
