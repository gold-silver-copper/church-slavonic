#![allow(clippy::unwrap_used)]
//! Constructed ambiguity controls, not claims that these supplied alternatives
//! are historical paradigms. The contextual premise is independently challenged
//! by LRC OCS lesson 10's address `филосѡѳє, вѣмь …`.
use church_slavonic::{Cell, Lexicon, Pos, Recension, sentence::Sentence};
use church_slavonic_tools::treebank::{node, sexpr};

fn lexicon(profile: Recension, multiple: bool) -> Lexicon {
    let rows = if multiple {
        "voc.n\tфилософъ\tn\tm\tinan\t-\t-\t-\tvoc.sg=рабъ\t-\t-\t-\nother.n\tconstructed\tn\tm\tinan\t-\t-\t-\tnom.sg=рабъ\t-\t-\t-\n"
    } else {
        "voc.n\tфилософъ\tn\tm\tinan\t-\t-\t-\tnom.sg=рабъ;voc.sg=рабъ\t-\t-\t-\n"
    };
    Lexicon::try_from_lexemes(
        profile,
        church_slavonic::lexicon::parse_in(rows, Pos::Noun, profile).unwrap(),
    )
    .unwrap()
}

#[test]
fn isolated_or_punctuated_address_retains_syncretic_cells() {
    for profile in [Recension::Synodal, Recension::OldChurchSlavonic] {
        let lex = lexicon(profile, false);
        for text in ["рабъ", "рабъ,", "  рабъ,\txyz\n"] {
            let mut sentence = Sentence::parse(&lex, text);
            let before = sentence.tree().clone();
            let stats = sentence.disambiguate();
            assert_eq!(sentence.tree(), &before);
            assert!(!stats.by_rule.contains_key("voc-drop"));
            assert_eq!(sentence.reproduce(), text);
            let tokens = sentence.tokens();
            let cells = tokens[0].reading.as_ref().unwrap().1.as_ref().unwrap();
            assert!(cells.contains(Cell::parse(Pos::Noun, "voc.sg").unwrap()));
            assert_eq!(cells.len(), 2);
        }
    }
}

#[test]
fn address_retains_competing_lexemes_through_tree_serialization() {
    for profile in [Recension::Synodal, Recension::OldChurchSlavonic] {
        let lex = lexicon(profile, true);
        let mut sentence = Sentence::parse(&lex, "рабъ,");
        let before = sentence.tree().clone();
        let stats = sentence.disambiguate();
        assert_eq!(sentence.tree(), &before);
        assert!(!stats.by_rule.contains_key("bare-voc"));
        assert!(sentence.tokens()[0].ambiguous);
        let encoded = sexpr::print(&node::to_sexpr(sentence.tree()));
        let decoded = node::from_sexpr(&sexpr::parse_many(&encoded).unwrap()[0]).unwrap();
        assert_eq!(decoded, before);
        // The legacy W codec preserves only the ambiguity marker. Use the
        // source document codec to demonstrate preservation of actual readings.
        let document = sentence.analysis_document(church_slavonic::matching::MatchPolicy::Exact);
        let encoded = church_slavonic_tools::analysis_document::to_json(&document).unwrap();
        let restored =
            church_slavonic_tools::analysis_document::from_json(encoded.as_bytes(), &lex).unwrap();
        let ids: std::collections::BTreeSet<_> = restored.segments()[0]
            .candidates()
            .iter()
            .map(|c| c.lexeme_id())
            .collect();
        assert_eq!(ids, ["voc.n", "other.n"].into_iter().collect());
        assert_eq!(restored.witness().reproduce(), "рабъ,");
    }
}
