//! Fixed-population diagnostics for legacy overlay projections. These are
//! annotation compatibility counts, not independent linguistic gold accuracy.
use super::{
    node::{Node, is_lexeme_id, render_with},
    runner::word_nodes,
};
use church_slavonic::{Cell, CellSet, Lexicon};
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct Score {
    pub units: usize,
    pub projected_words: usize,
    pub lexical_gold: usize,
    pub nonlexical_gold: usize,
    pub ambiguous_gold: usize,
    pub missing_source_units: usize,
    pub missing_source_lexical: usize,
    pub alignment_failed_units: usize,
    pub alignment_failed_lexical: usize,
    pub no_lexical_prediction: usize,
    pub wrong_lexeme: usize,
    pub disjoint_cells: usize,
    pub compatible: usize,
    pub compatible_singleton: usize,
    pub extra_cell_candidates: usize,
}

fn reading(node: &Node) -> Option<(&str, CellSet)> {
    match node {
        Node::Lex { id, cells, .. } => Some((id, cells.clone())),
        Node::Fn(id) if is_lexeme_id(id) => Some((id, CellSet::one(Cell::Word))),
        _ => None,
    }
}

impl Score {
    /// `None` represents an overlay address absent from the source. All its
    /// projected words remain counted. A malformed rendering fails alignment.
    pub fn observe(&mut self, hand: &Node, prediction: Option<(&Node, &str)>, lexicon: &Lexicon) {
        self.units += 1;
        let gold = word_nodes(hand);
        let lexical = gold.iter().filter(|n| reading(n).is_some()).count();
        self.projected_words += gold.len();
        self.lexical_gold += lexical;
        self.nonlexical_gold += gold.len() - lexical;
        self.ambiguous_gold += gold
            .iter()
            .filter(|n| matches!(n, Node::Lex { cells, .. } if cells.len() > 1))
            .count();
        let Some((auto, source)) = prediction else {
            self.missing_source_units += 1;
            self.missing_source_lexical += lexical;
            return;
        };
        let predicted = word_nodes(auto);
        // Equal leaf counts alone can align different words. Require both
        // complete renderings to match source and projected word renderings
        // to agree. Legacy projection drops punctuation and attached clitics;
        // the caller must name that limitation rather than call it token gold.
        let aligned = gold.len() == predicted.len()
            && render_with(hand, lexicon).is_ok_and(|s| s == source)
            && render_with(auto, lexicon).is_ok_and(|s| s == source)
            && gold.iter().zip(&predicted).all(|(h, a)| {
                match (render_with(h, lexicon), render_with(a, lexicon)) {
                    (Ok(h), Ok(a)) => h == a,
                    _ => false,
                }
            });
        if !aligned {
            self.alignment_failed_units += 1;
            self.alignment_failed_lexical += lexical;
            return;
        }
        for (h, a) in gold.into_iter().zip(predicted) {
            let Some((hid, hc)) = reading(h) else {
                continue;
            };
            let Some((aid, ac)) = reading(a) else {
                self.no_lexical_prediction += 1;
                continue;
            };
            if hid != aid {
                self.wrong_lexeme += 1;
            } else if !hc.iter().any(|c| ac.contains(c)) {
                self.disjoint_cells += 1;
            } else {
                self.compatible += 1;
                self.compatible_singleton += usize::from(ac.len() == 1);
                self.extra_cell_candidates += ac.iter().filter(|c| !hc.contains(*c)).count();
            }
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.units == 0 || self.lexical_gold == 0 {
            return Err("overlay evaluation unavailable: no lexical annotations");
        }
        if self.lexical_gold
            != self.missing_source_lexical
                + self.alignment_failed_lexical
                + self.no_lexical_prediction
                + self.wrong_lexeme
                + self.disjoint_cells
                + self.compatible
            || self.projected_words != self.lexical_gold + self.nonlexical_gold
        {
            return Err("overlay evaluation accounting mismatch");
        }
        Ok(())
    }
}
