#![allow(clippy::unwrap_used)]
//! Constructed ambiguity controls, not attested grammaticality judgments.
//! Adjacency and nominative morphology do not supply a subject attachment.
use church_slavonic::{Lexicon, Pos, Recension, matching::MatchPolicy, sentence::Sentence};
use church_slavonic_tools::analysis_document as codec;

fn lexicon(profile: Recension) -> Lexicon {
    let mut entries = church_slavonic::lexicon::parse_in(
        "noun.n\tрабъ\tn\tm\tinan\t-\t-\t-\tnom.sg=рабъ\t-\t-\t-\n",
        Pos::Noun,
        profile,
    )
    .unwrap();
    entries.extend(
        church_slavonic::lexicon::parse_in(
            "verb.v\tвидѣти\tv\t-\t-\t-\t-\t-\taor.2.sg=видѣ;aor.3.sg=видѣ\t-\t-\t-\n",
            Pos::Verb,
            profile,
        )
        .unwrap(),
    );
    Lexicon::try_from_lexemes(profile, entries).unwrap()
}

#[test]
fn nominative_neighbor_does_not_choose_the_verbs_person() {
    for profile in [Recension::Synodal, Recension::OldChurchSlavonic] {
        let lex = lexicon(profile);
        for text in ["рабъ видѣ", "видѣ рабъ", "  рабъ\tвидѣ\n"] {
            let mut sentence = Sentence::parse(&lex, text);
            let before = sentence.tree().clone();
            let stats = sentence.disambiguate();
            assert_eq!(sentence.tree(), &before, "{profile:?}: {text}");
            assert!(!stats.by_rule.contains_key("subj-verb"));
            let verb = sentence
                .tokens()
                .into_iter()
                .find(|t| t.surface == "видѣ")
                .unwrap();
            assert_eq!(verb.reading.unwrap().1.unwrap().name(), "aor.2|3.sg");
            assert_eq!(sentence.reproduce(), text);
        }
    }
}

#[test]
fn ordinary_context_and_reloaded_source_keep_both_person_readings() {
    for profile in [Recension::Synodal, Recension::OldChurchSlavonic] {
        let lex = lexicon(profile);
        let mut sentence = Sentence::parse(&lex, "  видѣ\tрабъ\n");
        let trace = sentence.contextual_trace().unwrap();
        assert!(trace.events().is_empty());
        assert_eq!(trace.input(), trace.proposed());
        sentence.disambiguate();
        let doc = sentence.analysis_document(MatchPolicy::Exact);
        let json = codec::to_json(&doc).unwrap();
        let back = codec::from_json(json.as_bytes(), &lex).unwrap();
        assert_eq!(back.witness().reproduce(), sentence.reproduce());
        let record = codec::Record::from_document(&back);
        let cells: Vec<_> = record
            .segments
            .iter()
            .flat_map(|s| &s.candidates)
            .filter(|c| c.lexeme == "verb.v")
            .map(|c| c.cell.as_str())
            .collect();
        assert_eq!(cells.len(), 2);
        assert!(cells.contains(&"aor.2.sg"));
        assert!(cells.contains(&"aor.3.sg"));
    }
}

fn transitive_lexicon(profile: Recension) -> Lexicon {
    let mut entries = church_slavonic::lexicon::parse_in(
        "subject.n\tрабъ\tn\tm\tinan\t-\t-\t-\tnom.sg=рабъ\t-\t-\t-\n\
         other.n\tсвѣтъ\tn\tm\tinan\t-\t-\t-\tnom.sg=свѣтъ;acc.sg=свѣтъ\t-\t-\t-\n",
        Pos::Noun,
        profile,
    )
    .unwrap();
    entries.extend(church_slavonic::lexicon::parse_in(
        "verb.v\tвидѣти\tv\t-\t-\t-\t-\t-\taor.1.sg=видѣхъ;aor.2.sg=видѣ;aor.3.sg=видѣ\t-\t-\ttran\n",
        Pos::Verb,
        profile,
    ).unwrap());
    Lexicon::try_from_lexemes(profile, entries).unwrap()
}

#[test]
fn transitivity_and_local_nominatives_do_not_prove_subject_or_object_attachment() {
    for profile in [Recension::Synodal, Recension::OldChurchSlavonic] {
        let lex = transitive_lexicon(profile);
        for text in [
            "видѣ рабъ свѣтъ",
            "рабъ свѣтъ видѣ",
            "свѣтъ рабъ видѣ",
            "видѣхъ свѣтъ",
        ] {
            let mut sentence = Sentence::parse(&lex, text);
            let before = sentence.tree().clone();
            let stats = sentence.disambiguate();
            assert_eq!(sentence.tree(), &before, "{profile:?}: {text}");
            assert!(!stats.by_rule.contains_key("one-subject"));
            let noun = sentence
                .tokens()
                .into_iter()
                .find(|t| t.surface == "свѣтъ")
                .unwrap();
            assert_eq!(noun.reading.unwrap().1.unwrap().name(), "nom|acc.sg");
            assert_eq!(sentence.reproduce(), text);
        }
    }
}

#[test]
fn one_subject_remains_an_inspectable_proposal_with_full_source_candidates() {
    for profile in [Recension::Synodal, Recension::OldChurchSlavonic] {
        let lex = transitive_lexicon(profile);
        let mut sentence = Sentence::parse(&lex, "  видѣ\tрабъ свѣтъ\n");
        let before = sentence.tree().clone();
        let trace = sentence.contextual_trace().unwrap();
        assert!(trace.events().iter().any(|e| e.rule == "one-subject"));
        assert!(
            trace
                .events()
                .iter()
                .all(|e| e.rule != "one-subject" || !e.applied_by_default)
        );
        assert_ne!(trace.input(), trace.proposed());
        assert_eq!(sentence.tree(), &before);
        sentence.disambiguate();
        assert_eq!(sentence.tree(), &before);
        let record =
            church_slavonic_tools::context_trace::evaluate(&lex, sentence.reproduce()).unwrap();
        let json = church_slavonic_tools::context_trace::to_json(&record).unwrap();
        let back = church_slavonic_tools::context_trace::from_json(json.as_bytes(), &lex).unwrap();
        assert_eq!(record, back);
        let cells: Vec<_> = back
            .source_analysis
            .segments
            .iter()
            .flat_map(|s| &s.candidates)
            .filter(|c| c.lexeme == "other.n")
            .map(|c| c.cell.as_str())
            .collect();
        assert_eq!(cells.len(), 2);
        assert!(cells.contains(&"nom.sg"));
        assert!(cells.contains(&"acc.sg"));
        assert_eq!(
            sentence.contextual_trace().unwrap().events(),
            trace.events()
        );
    }
}
