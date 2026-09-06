//! The README's example, verified: every string is the crate's real output.
#![allow(clippy::unwrap_used)] // a test asserts with unwrap

use church_slavonic::*;

#[test]
fn the_readme_example() {
    let syn = Lexicon::synodal();
    let rab = syn.get("рабъ.n").unwrap();
    assert_eq!(rab.inflect(Cell::noun(Case::Dative, Number::Plural)).unwrap().print(Recension::Synodal), "рабѡ́мъ");
    let gen_pl: Vec<String> = rab.forms(Cell::parse(Pos::Noun, "gen.pl").unwrap()).iter().map(|f| f.print(Recension::Synodal)).collect();
    assert_eq!(gen_pl, ["рабѡ́въ", "ра̑бъ"]);
    assert!(matches!(rab.inflect(Cell::infinitive()), Err(InflectError::NotThisPartOfSpeech { .. })));
    let verbs = syn.find("рещѝ", Pos::Verb);
    assert_eq!(verbs[0].inflect(Cell::finite(FiniteTense::Aorist, Person::Third, Number::Singular)).unwrap().print(Recension::Synodal), "речѐ");
    let readings = syn.analyze("рабѡ́мъ");
    assert_eq!(readings[0].lexeme.id, "рабъ.n");
    assert_eq!(readings[0].cell.name(), "dat.pl");
    let exact: Vec<_> = syn.readings("свѣ́тъ").into_iter().filter(|r| r.exact).collect();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0].cell_set().unwrap().name(), "nom|acc.sg");
    #[cfg(feature = "ocs")]
    {
        let rab = Lexicon::ocs().find("рабъ", Pos::Noun)[0];
        assert_eq!(rab.inflect(Cell::noun(Case::Locative, Number::Plural)).unwrap().print(Recension::OldChurchSlavonic), "рабѣхъ");
    }
    let guessed = syn.guess("кора́бль", Pos::Noun);
    assert_eq!(guessed.provenance, Provenance::Guessed);
}
