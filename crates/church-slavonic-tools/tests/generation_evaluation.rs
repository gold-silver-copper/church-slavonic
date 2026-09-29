#![allow(clippy::unwrap_used)]
use church_slavonic::{Cell, Lexicon, Pos, Recension};
use church_slavonic_tools::{
    eval::generation,
    sources::ud::{Corpus, CorpusSlot, SequenceToken},
};

fn lexicon() -> Lexicon {
    let rows = church_slavonic::lexicon::parse(
        "audit.n\tра́бъ\tn\tm\tinan\t-\t-\t-\tnom.sg=ра́бъ\t-\t-\t-\n",
        Pos::Noun,
    )
    .unwrap();
    Lexicon::try_from_lexemes(Recension::Synodal, rows).unwrap()
}
fn corpus() -> Corpus {
    Corpus {
        tokens: 1,
        slots: vec![CorpusSlot {
            lemma: "ра́бъ".into(),
            pos: Pos::Noun,
            cell: Cell::parse(Pos::Noun, "nom.sg").unwrap(),
            surface: "ра́бъ".into(),
        }],
        sentences: vec![vec![SequenceToken {
            surface: "ра́бъ".into(),
            lemma: "unused annotation".into(),
            object: false,
            slots: vec![0],
            source_record: None,
        }]],
        ..Default::default()
    }
}

#[test]
fn no_guessed_lemma_or_extra_cell_can_rescue_a_miss() {
    let lex = lexicon();
    let mut c = corpus();
    assert_eq!(
        generation::evaluate(&lex, &c).unwrap().tokens.exact_surface,
        1
    );
    c.slots[0].lemma = "fabricated".into();
    let r = generation::evaluate(&lex, &c).unwrap();
    assert_eq!(
        (
            r.tokens.total,
            r.tokens.lemma_present,
            r.tokens.exact_surface
        ),
        (1, 0, 0)
    );
    c.slots[0].lemma = "ра́бъ".into();
    c.slots[0].cell = Cell::parse(Pos::Noun, "acc.sg").unwrap();
    let r = generation::evaluate(&lex, &c).unwrap();
    assert_eq!(
        (
            r.tokens.total,
            r.tokens.lemma_present,
            r.tokens.cell_has_forms,
            r.tokens.exact_surface
        ),
        (1, 1, 0, 0)
    );
}

#[test]
fn surface_case_accent_and_letter_loss_are_not_silently_accepted() {
    let lex = lexicon();
    for surface in ["РА́БЪ", "рабъ", "рбъ"] {
        let mut c = corpus();
        c.slots[0].surface = surface.into();
        c.sentences[0][0].surface = surface.into();
        let r = generation::evaluate(&lex, &c).unwrap();
        assert_eq!(
            (
                r.tokens.cell_has_forms,
                r.tokens.exact_surface,
                r.tokens.unicode_equivalent_surface
            ),
            (1, 0, 0)
        );
    }
}

#[test]
fn alternatives_are_one_token_and_unmapped_inputs_remain_visible() {
    let lex = lexicon();
    let mut c = corpus();
    let mut alternative = c.slots[0].clone();
    alternative.lemma = "wrong lemma".into();
    c.slots.push(alternative);
    c.sentences[0][0].slots.push(1);
    let mut unmapped = c.sentences[0][0].clone();
    unmapped.slots.clear();
    c.sentences[0].push(unmapped);
    c.tokens += 1;
    let r = generation::evaluate(&lex, &c).unwrap();
    assert_eq!(
        (
            r.input_tokens,
            r.unmapped_tokens,
            r.tokens.total,
            r.tokens.exact_surface,
            r.slots.total,
            r.slots.exact_surface
        ),
        (2, 1, 1, 1, 2, 1)
    );
    c.sentences[0][0].slots.reverse();
    assert_eq!(
        serde_json::to_value(r).unwrap(),
        serde_json::to_value(generation::evaluate(&lex, &c).unwrap()).unwrap()
    );
}

#[test]
fn unicode_equivalence_does_not_receive_byte_exact_credit() {
    let rows = church_slavonic::lexicon::parse(
        "audit.n\tкрай\tn\tm\tinan\t-\t-\t-\tnom.sg=край\t-\t-\t-\n",
        Pos::Noun,
    )
    .unwrap();
    let lex = Lexicon::try_from_lexemes(Recension::Synodal, rows).unwrap();
    let mut c = corpus();
    c.slots[0].lemma = "краи\u{306}".into();
    c.slots[0].surface = "краи\u{306}".into();
    c.sentences[0][0].surface = c.slots[0].surface.clone();
    let r = generation::evaluate(&lex, &c).unwrap();
    assert_eq!(
        (
            r.tokens.lemma_present,
            r.tokens.exact_surface,
            r.tokens.unicode_equivalent_surface
        ),
        (1, 0, 1)
    );
}

#[test]
fn empty_is_unavailable_and_invalid_accounting_is_an_error() {
    let lex = lexicon();
    assert_eq!(
        generation::evaluate(&lex, &Corpus::default())
            .unwrap()
            .status,
        "unavailable"
    );
    let mut c = corpus();
    c.sentences[0][0].slots.push(0);
    assert!(generation::evaluate(&lex, &c).is_err());
    c.sentences[0][0].slots = vec![3];
    assert!(generation::evaluate(&lex, &c).is_err());
    c.sentences[0][0].slots.clear();
    assert!(generation::evaluate(&lex, &c).is_err());
    c.sentences[0][0].slots = vec![0];
    c.sentences[0][0].surface = "misaligned".into();
    assert!(generation::evaluate(&lex, &c).is_err());
    c = corpus();
    c.tokens += 1;
    assert!(generation::evaluate(&lex, &c).is_err());
}
