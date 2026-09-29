#![allow(clippy::unwrap_used)]
use church_slavonic::{Case, Cell, Lexicon, Number, Pos, Recension, matching::MatchPolicy};
use church_slavonic_tagger::Tagger;
use church_slavonic_tools::{
    sources::ud::{Corpus, CorpusSlot, SequenceToken},
    tagger::deployment::{self, Evaluation},
};
fn lexicon(overrides: &str) -> Lexicon {
    let rows = church_slavonic::lexicon::parse(
        &format!("audit.n\tра́бъ\tn\tm\tinan\t-\t-\t-\t{overrides}\t-\t-\t-\n"),
        Pos::Noun,
    )
    .unwrap();
    Lexicon::try_from_lexemes(Recension::Synodal, rows).unwrap()
}
fn corpus() -> Corpus {
    let slots = [Case::Nominative, Case::Accusative]
        .iter()
        .map(|&case| CorpusSlot {
            lemma: "ра́бъ".into(),
            pos: Pos::Noun,
            cell: Cell::noun(case, Number::Singular),
            surface: "ра́бъ".into(),
        })
        .collect();
    Corpus {
        label: "constructed evaluator control",
        tokens: 2,
        slots,
        sentences: vec![
            (0..2)
                .map(|i| SequenceToken {
                    surface: "ра́бъ".into(),
                    lemma: "ра́бъ".into(),
                    object: false,
                    slots: vec![i],
                    source_record: None,
                })
                .collect(),
        ],
        ..Default::default()
    }
}
fn model() -> Tagger {
    let mut model = Tagger::default();
    model.set("c=acc.sg", 5.0);
    model
}
fn predictions(e: &Evaluation) -> serde_json::Value {
    serde_json::json!(
        e.predictions
            .iter()
            .map(|s| s
                .iter()
                .map(deployment::prediction_json)
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    )
}

#[test]
fn changing_gold_lemmas_cells_and_dependency_flags_cannot_change_inference() {
    let lex = lexicon("nom.sg=ра́бъ;acc.sg=ра́бъ");
    let mut source = corpus();
    let before = deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact).unwrap();
    assert_eq!(
        before.predictions[0][1].context().prev_choice.unwrap().cell,
        Cell::noun(Case::Accusative, Number::Singular)
    );
    // The first gold cell is nominative; previous context follows prediction.
    for t in &mut source.sentences[0] {
        t.lemma = "fabricated-gold".into();
        t.object = true;
    }
    for s in &mut source.slots {
        s.lemma = "fabricated-gold".into();
        s.cell = Cell::noun(Case::Dative, Number::Singular);
    }
    let after = deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact).unwrap();
    assert_eq!(predictions(&before), predictions(&after));
    assert_eq!(after.report.feature_gold_retained, 0);
    assert_eq!(after.report.lemma_cell_retained, 0);
    assert_eq!(after.report.mapped_gold_tokens, 2);
}

#[test]
fn fabricated_lemmas_receive_no_lemma_cell_credit_even_when_features_match() {
    let lex = lexicon("nom.sg=ра́бъ;acc.sg=ра́бъ");
    let mut source = corpus();
    for s in &mut source.slots {
        s.lemma = "fabricated".into();
    }
    let report = deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact)
        .unwrap()
        .report;
    assert_eq!(report.feature_gold_retained, 2);
    assert_eq!(report.lemma_cell_retained, 0);
    assert_eq!(report.correct_selected_features, 1);
}

#[test]
fn deleted_candidates_and_abstentions_stay_in_the_denominator() {
    let source = corpus();
    let correct = deployment::evaluate(
        &lexicon("nom.sg=ра́бъ;acc.sg=ра́бъ"),
        &model(),
        &source,
        MatchPolicy::Exact,
    )
    .unwrap()
    .report;
    let wrong = deployment::evaluate(
        &lexicon("gen.sg=ра́бъ"),
        &model(),
        &source,
        MatchPolicy::Exact,
    )
    .unwrap()
    .report;
    assert_eq!(
        (correct.mapped_gold_tokens, correct.feature_gold_retained),
        (2, 2)
    );
    assert_eq!(
        (
            wrong.mapped_gold_tokens,
            wrong.feature_gold_retained,
            wrong.correct_selected_features
        ),
        (2, 0, 0)
    );
    let lex = Lexicon::try_from_lexemes(Recension::Synodal, Vec::new()).unwrap();
    let empty = deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact)
        .unwrap()
        .report;
    assert_eq!(
        (
            empty.mapped_gold_tokens,
            empty.no_candidates,
            empty.abstained
        ),
        (2, 2, 2)
    );
    let tied = deployment::evaluate(
        &lexicon("nom.sg=ра́бъ;acc.sg=ра́бъ"),
        &Tagger::default(),
        &source,
        MatchPolicy::Exact,
    )
    .unwrap()
    .report;
    assert_eq!((tied.selected_features, tied.abstained), (0, 2));
}

#[test]
fn orphaned_or_reused_annotations_cannot_establish_a_valid_denominator() {
    let lex = lexicon("nom.sg=ра́бъ;acc.sg=ра́бъ");
    let mut source = corpus();
    source.sentences[0][1].slots.clear();
    assert!(
        deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact)
            .err()
            .unwrap()
            .contains("unreferenced")
    );
    let mut source = corpus();
    source.sentences[0][1].slots.push(0);
    assert!(
        deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact)
            .err()
            .unwrap()
            .contains("more than once")
    );
    let mut source = corpus();
    source.sentences[0][0].slots.push(0);
    assert!(
        deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact)
            .err()
            .unwrap()
            .contains("more than once")
    );
}

#[test]
fn unmapped_tokens_are_visible_and_empty_or_misaligned_evaluations_fail() {
    let lex = lexicon("nom.sg=ра́бъ;acc.sg=ра́бъ");
    let mut source = corpus();
    source.sentences[0].push(SequenceToken {
        surface: "?".into(),
        lemma: "_".into(),
        object: false,
        slots: vec![],
        source_record: None,
    });
    source.tokens += 1;
    let report = deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact)
        .unwrap()
        .report;
    assert_eq!(
        (
            report.input_tokens,
            report.mapped_gold_tokens,
            report.tokens_without_mapped_gold
        ),
        (3, 2, 1)
    );
    assert!(deployment::evaluate(&lex, &model(), &Corpus::default(), MatchPolicy::Exact).is_err());
    source.sentences[0][0].slots = vec![999];
    assert!(deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact).is_err());
    let mut source = corpus();
    source.slots[0].surface = "wrong source".into();
    assert!(deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact).is_err());
    let mut source = corpus();
    source.tokens += 1;
    assert!(deployment::evaluate(&lex, &model(), &source, MatchPolicy::Exact).is_err());
}

#[test]
fn candidate_order_and_ambiguous_gold_order_do_not_choose_a_scoring_target() {
    let mut source = corpus();
    source.tokens = 1;
    source.sentences[0].truncate(1);
    source.sentences[0][0].slots = vec![0, 1];
    let first = deployment::evaluate(
        &lexicon("nom.sg=ра́бъ;acc.sg=ра́бъ"),
        &model(),
        &source,
        MatchPolicy::Exact,
    )
    .unwrap();
    source.sentences[0][0].slots.reverse();
    let second = deployment::evaluate(
        &lexicon("acc.sg=ра́бъ;nom.sg=ра́бъ"),
        &model(),
        &source,
        MatchPolicy::Exact,
    )
    .unwrap();
    assert_eq!(predictions(&first), predictions(&second));
    assert_eq!(first.report.correct_selected_features, 1);
    assert_eq!(second.report.correct_selected_features, 1);
}

#[test]
fn oracle_context_is_a_separate_same_candidate_diagnostic() {
    let lex = lexicon("nom.sg=ра́бъ;acc.sg=ра́бъ");
    let mut source = corpus();
    source.sentences[0][1].lemma = "fabricated".into();
    let mut tagger = model();
    tagger.set("nl=fabricated|c=nom.sg", 20.0);
    let evaluated = deployment::evaluate(&lex, &tagger, &source, MatchPolicy::Exact).unwrap();
    assert_eq!(evaluated.report.mapped_gold_tokens, 2);
    assert_eq!(evaluated.report.correct_selected_features, 1);
    assert_eq!(
        evaluated
            .report
            .oracle_context_diagnostic
            .correct_selected_features,
        2
    );
    assert_eq!(
        evaluated.predictions[0][0].context().next_lemma.as_deref(),
        Some("ра́бъ")
    );
}
