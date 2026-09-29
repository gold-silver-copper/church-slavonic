#![allow(clippy::unwrap_used)]
//! Operational controls for existing heuristics, not linguistic gold.
use church_slavonic::{
    Lexicon, Pos, Recension,
    sentence::{Node, Sentence, rules, trace},
};
use church_slavonic_tools::context_trace;

fn lexicon() -> Lexicon {
    let mut rows = church_slavonic::lexicon::parse(
        "audit.n\tрабъ\tn\tm\tinan\t-\t-\t-\tnom.sg=рабъ;gen.sg=рабъ;loc.sg=рабъ\t-\t-\t-\n",
        Pos::Noun,
    )
    .unwrap();
    rows.extend(
        church_slavonic::lexicon::parse(
            "audit.v\tвидѣти\tv\t-\t-\t-\t-\t-\taor.1.sg=ви́дѣхъ\t-\t-\ttran\n",
            Pos::Verb,
        )
        .unwrap(),
    );
    rows.extend(
        church_slavonic::lexicon::parse(
            "preposition.x\tpreptest\tx\t-\t-\tprep\t-\tgov=gen|loc\t-\t-\t-\t-\n",
            Pos::Closed,
        )
        .unwrap(),
    );
    rows.extend(
        church_slavonic::lexicon::parse(
            "adjective.a\tзлый\ta\t-\t-\t-\t-\t-\tpos.m.sg.gen=зла\t-\t-\t-\n",
            Pos::Adjective,
        )
        .unwrap(),
    );
    Lexicon::try_from_lexemes(Recension::Synodal, rows).unwrap()
}

#[test]
fn successive_decisions_replay_exactly_and_inspection_does_not_edit() {
    let lex = lexicon();
    let mut sentence = Sentence::parse(&lex, "  preptest\tрабъ зла\n");
    let original = sentence.tree().clone();
    let t = sentence.contextual_trace().unwrap();
    assert_eq!(sentence.tree(), &original);
    assert_eq!(
        t.events().iter().map(|e| e.rule).collect::<Vec<_>>(),
        vec!["prep-gov", "np-agree"]
    );
    let cells = |node: &Node| match rules::leaf(node) {
        Some(Node::Lex { cells, .. }) => cells.name(),
        _ => panic!("lexical child"),
    };
    assert_eq!(cells(&t.events()[0].context[1]), "nom|gen|loc.sg");
    assert_eq!(cells(&t.events()[0].proposed), "gen|loc.sg");
    assert_eq!(cells(&t.events()[1].proposed), "gen.sg");
    let Node::Group { children, .. } = t.input() else {
        panic!("group")
    };
    let mut replay = children.clone();
    for event in t.events() {
        assert_eq!(event.context, replay);
        assert_ne!(replay[event.child], event.proposed);
        replay[event.child] = event.proposed.clone();
    }
    let Node::Group { children, .. } = t.proposed() else {
        panic!("group")
    };
    assert_eq!(&replay, children);
    let mut ordinary = original.clone();
    rules::disambiguate(&mut ordinary, &lex);
    assert_eq!(&ordinary, t.proposed());
    *sentence.tree_mut() = Node::W {
        surface: "editorial replacement".into(),
        notes: vec![],
    };
    let again = sentence.contextual_trace().unwrap();
    assert_eq!(again.events(), t.events());
    assert_eq!(sentence.reproduce(), "  preptest\tрабъ зла\n");
}

#[test]
fn serialized_proposals_and_source_readings_are_recomputed_on_reload() {
    let lex = lexicon();
    let original = context_trace::evaluate(&lex, "preptest рабъ зла").unwrap();
    let json = context_trace::to_json(&original).unwrap();
    assert_eq!(
        context_trace::from_json(json.as_bytes(), &lex).unwrap(),
        original
    );
    assert_eq!(original.events.len(), 2);
    for change in 0..4 {
        let mut changed = original.clone();
        match change {
            0 => {
                changed.events.remove(0);
            }
            1 => changed.events[0].rule = "fabricated authority".into(),
            2 => changed.events[1].context[1] = "(w changed)".into(),
            _ => changed
                .source_analysis
                .segments
                .iter_mut()
                .find(|s| !s.candidates.is_empty())
                .unwrap()
                .candidates
                .clear(),
        }
        let encoded = context_trace::to_json(&changed).unwrap();
        assert!(context_trace::from_json(encoded.as_bytes(), &lex).is_err());
    }
}

#[test]
fn exhausted_trace_returns_error_and_preserves_input() {
    let lex = lexicon();
    let sentence = Sentence::parse(&lex, "ви́дѣхъ рабъ");
    let Node::Group { children, .. } = sentence.tree() else {
        panic!("group")
    };
    let mut many = vec![children[0].clone()];
    many.extend(std::iter::repeat_n(
        children[1].clone(),
        trace::MAX_EVENTS + 1,
    ));
    let input = Node::Group {
        head: "s".into(),
        children: many,
    };
    let before = input.clone();
    assert!(trace::evaluate(&input, &lex).is_err());
    assert_eq!(input, before);
    let notes = Node::Group {
        head: "s".into(),
        children: vec![Node::W {
            surface: String::new(),
            notes: vec![(String::new(), String::new()); 1025],
        }],
    };
    assert!(trace::evaluate(&notes, &lex).is_err());
    let mut deep = Node::W {
        surface: "x".into(),
        notes: vec![],
    };
    for _ in 0..34 {
        deep = Node::Cap(Box::new(deep));
    }
    assert!(
        trace::evaluate(
            &Node::Group {
                head: "s".into(),
                children: vec![deep]
            },
            &lex
        )
        .is_err()
    );
    let nested = Node::Group {
        head: "s".into(),
        children: vec![Node::Group {
            head: "np".into(),
            children: vec![],
        }],
    };
    assert!(trace::evaluate(&nested, &lex).is_err());
    assert!(context_trace::evaluate(&lex, &"x".repeat(4097)).is_err());
}
