#![allow(clippy::unwrap_used)]
use church_slavonic::{Case, Cell, CellSet, Lexicon, Number, Pos, Recension};
use church_slavonic_tools::treebank::{node::Node, overlay_score::Score};
fn lexicon() -> Lexicon {
    let rows = ["one.n", "two.n"]
        .into_iter()
        .map(|id| {
            format!(
                "{id}\tра́бъ\tn\tm\tinan\t-\t-\t-\tnom.sg=ра́бъ;acc.sg=ра́бъ;gen.sg=ра́бъ\t-\t-\t-\n"
            )
        })
        .collect::<String>();
    Lexicon::try_from_lexemes(
        Recension::Synodal,
        church_slavonic::lexicon::parse(&rows, Pos::Noun).unwrap(),
    )
    .unwrap()
}
fn leaf(id: &str, cases: &[Case]) -> Node {
    Node::Lex {
        id: id.into(),
        cells: CellSet::new(
            cases
                .iter()
                .map(|&c| Cell::noun(c, Number::Singular))
                .collect(),
        )
        .unwrap(),
        alt: 0,
        notes: vec![],
    }
}
#[test]
fn ambiguous_gold_does_not_hide_wrong_lexemes_or_disjoint_cells() {
    let lex = lexicon();
    let hand = leaf("one.n", &[Case::Nominative, Case::Accusative]);
    let mut score = Score::default();
    score.observe(
        &hand,
        Some((&leaf("two.n", &[Case::Nominative]), "ра́бъ")),
        &lex,
    );
    score.observe(
        &hand,
        Some((&leaf("one.n", &[Case::Genitive]), "ра́бъ")),
        &lex,
    );
    score.validate().unwrap();
    assert_eq!(
        (
            score.lexical_gold,
            score.ambiguous_gold,
            score.wrong_lexeme,
            score.disjoint_cells,
            score.compatible
        ),
        (2, 2, 1, 1, 0)
    );
}
#[test]
fn retention_and_singleton_selection_are_distinct_with_uncertain_gold() {
    let lex = lexicon();
    let hand = leaf("one.n", &[Case::Nominative, Case::Accusative]);
    let mut score = Score::default();
    score.observe(
        &hand,
        Some((&leaf("one.n", &[Case::Accusative]), "ра́бъ")),
        &lex,
    );
    score.observe(
        &hand,
        Some((&leaf("one.n", &[Case::Nominative, Case::Genitive]), "ра́бъ")),
        &lex,
    );
    score.validate().unwrap();
    assert_eq!(
        (
            score.compatible,
            score.compatible_singleton,
            score.extra_cell_candidates
        ),
        (2, 1, 1)
    );
}
#[test]
fn missing_source_alignment_failure_and_no_prediction_remain_in_population() {
    let lex = lexicon();
    let hand = leaf("one.n", &[Case::Nominative]);
    let mut score = Score::default();
    score.observe(&hand, None, &lex);
    let other = Node::W {
        surface: "другъ".into(),
        notes: vec![],
    };
    score.observe(&hand, Some((&other, "ра́бъ")), &lex);
    let unknown = Node::W {
        surface: "ра́бъ".into(),
        notes: vec![],
    };
    score.observe(&hand, Some((&unknown, "ра́бъ")), &lex);
    score.observe(&unknown, Some((&unknown, "ра́бъ")), &lex);
    score.validate().unwrap();
    assert_eq!(
        (
            score.projected_words,
            score.lexical_gold,
            score.nonlexical_gold
        ),
        (4, 3, 1)
    );
    assert_eq!(
        (
            score.missing_source_lexical,
            score.alignment_failed_lexical,
            score.no_lexical_prediction
        ),
        (1, 1, 1)
    );
    assert_eq!(score.compatible, 0);
}
#[test]
fn empty_population_is_unavailable_and_candidate_order_does_not_change_credit() {
    assert!(Score::default().validate().is_err());
    let lex = lexicon();
    let mut first = Score::default();
    let mut second = Score::default();
    let predicted = leaf("one.n", &[Case::Accusative]);
    first.observe(
        &leaf("one.n", &[Case::Nominative, Case::Accusative]),
        Some((&predicted, "ра́бъ")),
        &lex,
    );
    second.observe(
        &leaf("one.n", &[Case::Accusative, Case::Nominative]),
        Some((&predicted, "ра́бъ")),
        &lex,
    );
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
}

#[test]
fn explicit_closed_class_ids_are_lexical_gold_even_without_inflection() {
    let rows = church_slavonic::lexicon::parse(
        "one.x\tже\tx\t-\t-\tpart\t-\t-\t-\t-\t-\t-\ntwo.x\tже\tx\t-\t-\tpart\t-\t-\t-\t-\t-\t-\n",
        Pos::Closed,
    )
    .unwrap();
    let lex = Lexicon::try_from_lexemes(Recension::Synodal, rows).unwrap();
    let mut score = Score::default();
    let hand = Node::Fn("one.x".into());
    score.observe(&hand, Some((&Node::Fn("two.x".into()), "же")), &lex);
    score.observe(
        &hand,
        Some((
            &Node::W {
                surface: "же".into(),
                notes: vec![],
            },
            "же",
        )),
        &lex,
    );
    score.validate().unwrap();
    assert_eq!(
        (
            score.lexical_gold,
            score.wrong_lexeme,
            score.no_lexical_prediction
        ),
        (2, 1, 1)
    );
}
