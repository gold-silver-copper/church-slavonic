//! Legacy sentence-tree editing and source-preserving analysis access.
//!
//! The legacy constraint layer applies local heuristics; its exclusions do not
//! constitute validated grammatical judgments. `analysis_document` returns the
//! immutable source's morphological alternatives independently of tree edits.
//! `print` regenerates the edited tree with normalized spacing; `reproduce`
//! returns the original input.

pub mod closed;
pub mod lift;
pub mod node;
pub mod rules;
pub mod trace;

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
    witness: crate::witness::Witness,
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
    /// assert!(words[1].ambiguous); // no lexical choice from a missing local governor
    /// assert!(words[1].reading.is_none());
    /// assert_eq!(words[2].reading.as_ref().unwrap().1.as_ref().unwrap().name(), "nom.sg");
    /// assert_eq!(words[3].reading.as_ref().unwrap().1.as_ref().unwrap().name(), "nom|acc.sg");
    /// assert!(!stats.by_rule.contains_key("bare-loc"));
    /// assert_eq!(s.reproduce(), "И҆ ви́дѣ бг҃ъ свѣ́тъ, ꙗ҆́кѡ добро̀.");
    /// ```
    pub fn parse(lexicon: &'a Lexicon, text: &str) -> Sentence<'a> {
        let lifter = lift::Lifter::new(lexicon);
        let (tree, coverage) = lifter.lift_verse(text);
        Sentence { lexicon, tree, coverage, witness: crate::witness::Witness::new(text, None) }
    }

    /// Apply the remaining legacy exclusion rules; what each rule narrowed.
    /// `one-subject` is available only as an unvalidated proposal through
    /// `contextual_trace`, and cannot narrow the live tree here.
    pub fn disambiguate(&mut self) -> Stats {
        rules::disambiguate(&mut self.tree, self.lexicon)
    }

    /// Inspect legacy rule proposals from a fresh lift of the original witness.
    /// Neither the source nor the current edited tree is mutated.
    pub fn contextual_trace(&self) -> Result<trace::Trace, TreeError> {
        if self.reproduce().len() > 4096 { return Err(TreeError("context trace source exceeds 4096 bytes".into())); }
        let (raw, _) = lift::Lifter::new(self.lexicon).lift_verse(self.reproduce());
        trace::evaluate(&raw, self.lexicon)
    }

    /// Regenerate the current analysis with normalized spacing. A different
    /// lexicon's grammar is not selected implicitly by the requested print.
    pub fn print(&self, recension: Recension) -> Result<String, TreeError> {
        if recension != self.lexicon.recension() {
            return Err(TreeError("sentence and rendering profile differ".to_string()));
        }
        node::render_with(&self.tree, self.lexicon)
    }

    /// The original input bytes, unaffected by analysis or tree edits.
    pub fn reproduce(&self) -> &str { self.witness.reproduce() }

    pub fn witness(&self) -> &crate::witness::Witness { &self.witness }

    /// Analyze the original witness under an explicit matching policy. Existing
    /// tree edits and heuristic exclusions never change this candidate layer.
    pub fn analysis_document(&self, policy: crate::matching::MatchPolicy) -> crate::document::AnalysisDocument<'a> {
        crate::document::AnalysisDocument::analyze(self.lexicon, self.witness.clone(), policy)
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
            let surface = node::render_with(other, lexicon).unwrap_or_default();
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
