//! The lifter lives in the library (`church_slavonic::sentence::lift`);
//! its tests that read the s-expression form stay here.

pub use church_slavonic::sentence::lift::*;

#[cfg(test)]
mod tests {
    use super::*;
    use church_slavonic::sentence::node::Node;
    use church_slavonic::Lexicon;
    use crate::treebank::node::render;
    use std::sync::OnceLock;

    fn lifter() -> &'static Lifter<'static> {
        static L: OnceLock<Lifter<'static>> = OnceLock::new();
        L.get_or_init(|| Lifter::new(Lexicon::synodal()))
    }

    #[test]
    fn lifting_preserves_the_round_trip() {
        let verse = "Въ нача́лѣ сотворѝ бг҃ъ не́бо и҆ зе́млю.";
        let (tree, coverage) = lifter().lift_verse(verse);
        assert_eq!(render(&tree, &RECENSION).unwrap(), verse);
        assert!(coverage.analyzed >= 2, "{coverage:?}");
        let text = crate::treebank::sexpr::print(&crate::treebank::node::to_sexpr(&tree));
        assert!(text.contains("(abbr \"бг҃\" (n богъ.n :case nom :num sg))") || text.contains(":amb"), "{text}");
    }

    #[test]
    fn the_pitfall_verse_lifts_without_touching_the_apparatus() {
        let verse = "и҆ речѐ ю҆нѣ́йшїй ꙾є҆ю̀꙾[26] ѻ҆тцꙋ̀: ѻ҆́тче, да́ждь мѝ досто́йнꙋю ча́сть и҆мѣ́нїѧ.";
        let (tree, coverage) = lifter().lift_verse(verse);
        assert_eq!(render(&tree, &RECENSION).unwrap(), verse);
        assert_eq!(coverage.apparatus, 1);
    }

    #[test]
    fn syncretism_is_an_underspecified_leaf() {
        // свѣ́тъ: one lexeme, the cells its paradigm does not tell apart
        let (node, fate) = lifter().lift_core("свѣ́тъ");
        assert_eq!(fate, TokenFate::Underspecified);
        let Node::Lex { id, cells, alt, .. } = node else { panic!("{node:?}") };
        assert_eq!(id, "свѣтъ.n");
        assert_eq!(cells.name(), "nom|acc.sg");
        assert_eq!(alt, 0);
        // the leaf writes the set as a disjunctive feature
        let text = crate::treebank::sexpr::print(&crate::treebank::node::to_sexpr(&Node::Lex { id, cells, alt, notes: Vec::new() }));
        assert_eq!(text, "(n свѣтъ.n :case nom|acc :num sg)");
    }

    #[test]
    fn homonymy_is_recorded_never_guessed() {
        // дꙋ́хъ: the noun's nominative and дꙋти's aorist — two lexemes
        let (node, fate) = lifter().lift_core("дꙋ́хъ");
        assert_eq!(fate, TokenFate::Ambiguous);
        assert!(matches!(node, Node::W { ref notes, .. } if notes.iter().any(|(k, v)| k == "amb" && v == "2")), "{node:?}");
    }

    #[test]
    fn a_titlo_token_names_every_cell_the_abbreviation_hides() {
        // дх҃ъ: the accent that tells дꙋ́хъ (nom.sg) from дꙋ̑хъ (gen.pl,
        // acc.pl) is gone under the titlo
        let (node, fate) = lifter().lift_core("дх҃ъ");
        assert_eq!(fate, TokenFate::Underspecified);
        let Node::Abbr { prefix, child, .. } = node else { panic!("{node:?}") };
        assert_eq!(prefix, "дх҃");
        let Node::Lex { cells, .. } = *child else { panic!("{child:?}") };
        assert_eq!(cells.name(), "nom.sg|gen.pl|acc.pl");
        assert_eq!(lifter().titlo.cells("дх҃ъ", "дх҃", "дꙋхъ.n").map(|c| c.name()), Some("nom.sg|gen.pl|acc.pl".to_string()));
    }
}
