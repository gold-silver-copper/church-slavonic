#![allow(clippy::unwrap_used)]
//! Constructed ambiguity controls. LRC OCS §38 independently establishes that
//! missing prepositions are not sufficient evidence against a locative reading.
use church_slavonic::{Lexicon, Pos, Recension, sentence::Sentence};

fn lexicon(profile: Recension, multiple: bool) -> Lexicon {
    let text = if multiple {
        "loc.n\tрабъ\tn\tm\tinan\t-\t-\t-\tloc.sg=рабъ\t-\t-\t-\nother.n\tconstructed\tn\tm\tinan\t-\t-\t-\tnom.sg=рабъ\t-\t-\t-\n"
    } else {
        "loc.n\tрабъ\tn\tm\tinan\t-\t-\t-\tnom.sg=рабъ;loc.sg=рабъ\t-\t-\t-\n"
    };
    Lexicon::try_from_lexemes(
        profile,
        church_slavonic::lexicon::parse_in(text, Pos::Noun, profile).unwrap(),
    )
    .unwrap()
}

#[test]
fn absence_of_local_governor_does_not_remove_a_locative_cell() {
    for profile in [Recension::Synodal, Recension::OldChurchSlavonic] {
        let lex = lexicon(profile, false);
        for text in ["xyz рабъ", "xyz, рабъ", "  xyz\tрабъ\n"] {
            let mut s = Sentence::parse(&lex, text);
            let before = s.tree().clone();
            let stats = s.disambiguate();
            assert_eq!(s.tree(), &before);
            assert!(!stats.by_rule.contains_key("bare-loc"));
            assert_eq!(s.reproduce(), text);
            assert_eq!(
                s.tokens()
                    .last()
                    .unwrap()
                    .reading
                    .as_ref()
                    .unwrap()
                    .1
                    .as_ref()
                    .unwrap()
                    .name(),
                "nom|loc.sg"
            );
        }
    }
}

#[test]
fn absence_of_local_governor_does_not_select_a_competing_lexeme() {
    for profile in [Recension::Synodal, Recension::OldChurchSlavonic] {
        let lex = lexicon(profile, true);
        let mut s = Sentence::parse(&lex, "xyz рабъ");
        let before = s.tree().clone();
        s.disambiguate();
        assert_eq!(s.tree(), &before);
        assert!(s.tokens()[1].ambiguous);
    }
}
