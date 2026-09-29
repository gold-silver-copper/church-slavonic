//! Constructed adversarial fixtures. These test API semantics, not attestation.
#![allow(clippy::unwrap_used)]

use church_slavonic::{*, lexicon::parse, matching::{compare, MatchPolicy, Transformation}, sentence::{Sentence, lift::{Lifter, TokenFate}}, witness::{Witness, SegmentKind}};
use unicode_normalization::UnicodeNormalization;

fn noun(id: &str, class: &str, stress: &str, overrides: &str, variants: &str) -> Lexeme {
    parse(&format!("{id}\tра́бъ\tn\tm\tinan\t{class}\t{stress}\t-\t{overrides}\t{variants}\t-\t-\n"), Pos::Noun).unwrap().remove(0)
}
fn custom() -> Lexicon {
    Lexicon::from_lexemes(Recension::Synodal, vec![noun("audit.n", "N1t", "a", "-", "-")])
}

#[test]
fn matching_policies_do_not_call_case_or_accent_changes_exact() {
    let lex = custom();
    let query = "РА́БЪ";
    assert!(lex.analyze_with(query, MatchPolicy::Exact).is_empty());
    assert!(lex.analyze_with(query, MatchPolicy::UnicodeEquivalent).is_empty());
    let case = lex.analyze_with(query, MatchPolicy::CaseInsensitive);
    assert!(!case.is_empty());
    assert!(case.iter().all(|r| !r.analysis.exact));
    assert!(case[0].trace.query_steps.iter().any(|s| s.transformation == Transformation::Lowercase));
    assert!(lex.analyze_with("рабъ", MatchPolicy::CaseInsensitive).is_empty());
    assert!(!lex.analyze_with("рабъ", MatchPolicy::AccentInsensitive).is_empty());
    // Accent tolerance cannot manufacture or remove a breathing mark.
    assert!(compare("а", "а҆", MatchPolicy::AccentInsensitive).is_none());
    // The old print-key relation is available by its explicit legacy name.
    assert!(compare("рабѡ́мъ", "рабо́мъ", MatchPolicy::AccentInsensitive).is_none());
    assert!(compare("рабѡ́мъ", "рабо́мъ", MatchPolicy::LegacyOrthographic).is_some());
}

#[test]
fn unicode_equivalence_is_separate_from_byte_identity() {
    let nfc = "ѝ";
    let nfd: String = nfc.nfd().collect();
    assert_ne!(nfc, nfd);
    assert!(compare(nfc, &nfd, MatchPolicy::Exact).is_none());
    let trace = compare(nfc, &nfd, MatchPolicy::UnicodeEquivalent).unwrap();
    assert_eq!(trace.generated_steps.len(), 1);
    assert_eq!(trace.generated_steps[0].transformation, Transformation::UnicodeNormalization);
    assert!(compare("й", "и", MatchPolicy::AccentInsensitive).is_none());
}

#[test]
fn custom_lexicon_is_used_by_lifting_and_rendering() {
    let lex = custom();
    let sentence = Sentence::parse(&lex, "ра́бъ");
    assert_eq!(sentence.tokens()[0].reading.as_ref().unwrap().0, "audit.n");
    assert_eq!(sentence.print(Recension::Synodal).unwrap(), "ра́бъ");
    assert!(sentence.print(Recension::OldChurchSlavonic).is_err());
}

#[test]
fn observed_source_survives_analysis_edits_with_all_separators() {
    let lex = custom();
    let source = "  ра́бъ\t\nра́бъ\u{a0}";
    let mut sentence = Sentence::parse(&lex, source);
    *sentence.tree_mut() = sentence::Node::W { surface: "changed".to_string(), notes: Vec::new() };
    assert_eq!(sentence.reproduce().as_bytes(), source.as_bytes());
    assert_eq!(sentence.print(Recension::Synodal).unwrap(), "changed");
    let segments = sentence.witness().segments();
    assert_eq!(segments.iter().map(|(_, s)| s.text()).collect::<String>(), source);
    assert_eq!(segments[0].0, SegmentKind::Whitespace);
    assert_eq!(segments[1].1.range(), 2..12);
    let witness = Witness::new("ѣ", None);
    assert!(witness.span(1..2).is_none());
    assert!(witness.span(0..3).is_none());
    assert_eq!(witness.span(0..2).unwrap().text(), "ѣ");
    assert!(Witness::new("", None).segments().is_empty());
}

#[test]
fn explicit_only_cells_are_indexed_and_enumerated() {
    let nom = Cell::noun(Case::Nominative, Number::Singular);
    let genitive = Cell::noun(Case::Genitive, Number::Singular);
    let n = noun("explicit.n", "-", "a", "nom.sg=ра́бъ", "gen.sg=раба̀");
    assert!(n.inflect(nom).is_ok());
    assert_eq!(n.cells(), vec![nom, genitive]);
    assert_eq!(n.all_forms().len(), 2);
    let lex = Lexicon::from_lexemes(Recension::Synodal, vec![n]);
    for (surface, cell) in [("ра́бъ", nom), ("раба̀", genitive)] {
        assert!(lex.analyze_with(surface, MatchPolicy::Exact).iter().any(|r| r.analysis.cell == cell));
    }
}

#[test]
fn alternative_identity_is_not_saturated_at_255() {
    let variants = (1..=260).map(|n| format!("р{}б", "а".repeat(n))).collect::<Vec<_>>().join("|");
    let n = noun("many.n", "N1t", "a", "-", &format!("nom.sg={variants}"));
    let nom = Cell::noun(Case::Nominative, Number::Singular);
    let prints: Vec<_> = n.forms(nom).iter().map(|f| f.print(Recension::Synodal)).collect();
    let lex = Lexicon::from_lexemes(Recension::Synodal, vec![n]);
    for index in [254, 255, 256, 260] {
        let analyses = lex.analyze_with(&prints[index], MatchPolicy::Exact);
        let result = analyses.iter().find(|r| r.analysis.cell == nom).unwrap();
        assert_eq!(result.analysis.alt, index);
        assert_eq!(prints[result.analysis.alt], result.analysis.print);
    }
}

#[test]
fn malformed_stress_fails_at_the_input_boundary() {
    let line = "bad.n\tра́бъ\tn\tm\tinan\tN1t\tnot_a_stress_paradigm\t-\t-\t-\t-\t-\n";
    let error = parse(line, Pos::Noun).unwrap_err();
    assert_eq!(error.line, 1);
    assert!(error.message.contains("unknown stress paradigm"));
}

#[test]
fn closed_class_alternatives_are_still_ambiguous() {
    let text = "one.x\tже\tx\t-\t-\tpart\t-\t-\t-\t-\t-\t-\ntwo.x\tже\tx\t-\t-\tconj\t-\t-\t-\t-\t-\t-\n";
    let lex = Lexicon::from_lexemes(Recension::Synodal, parse(text, Pos::Closed).unwrap());
    let (_, fate) = Lifter::new(&lex).lift_core("же");
    assert_eq!(fate, TokenFate::Ambiguous);
    let sentence = Sentence::parse(&lex, "же");
    assert!(sentence.tokens()[0].ambiguous);
    assert!(sentence.tokens()[0].reading.is_none());
    assert_eq!(lex.analyze_with("же", MatchPolicy::Exact).len(), 2);
}

#[test]
fn each_policy_index_agrees_with_direct_comparison() {
    let surfaces = ["и", "ѝ", "й", "а҆", "а́", "ѡ", "о", "ꙋ", "у", "é", "e", "İ", "i", "а\u{301}\u{300}"];
    let text = surfaces.iter().enumerate().map(|(i, surface)|
        format!("fixture.{i}\t{surface}\tx\t-\t-\tpart\t-\t-\t-\t-\t-\t-\n")
    ).collect::<String>();
    let lex = Lexicon::from_lexemes(Recension::Synodal, parse(&text, Pos::Closed).unwrap());
    let forms: Vec<_> = lex.iter().flat_map(|l| l.all_forms().into_iter().flat_map(move |(cell, forms)| {
        forms.into_iter().enumerate().map(move |(alt, (_, print))| (l.id.clone(), cell, alt, print))
    })).collect();
    let queries: Vec<_> = surfaces.iter().flat_map(|s| [s.to_string(), s.to_uppercase(), s.nfd().collect()]).collect();
    for policy in [MatchPolicy::Exact, MatchPolicy::UnicodeEquivalent, MatchPolicy::CaseInsensitive,
        MatchPolicy::AccentInsensitive, MatchPolicy::LegacyOrthographic] {
        for query in &queries {
            let expected: std::collections::BTreeSet<_> = forms.iter()
                .filter(|(_, _, _, print)| compare(query, print, policy).is_some())
                .map(|(id, cell, alt, print)| (id.clone(), cell.name(), *alt, print.clone())).collect();
            let actual: std::collections::BTreeSet<_> = lex.analyze_with(query, policy).into_iter().map(|m| {
                let a = m.analysis;
                (a.lexeme.id.clone(), a.cell.name(), a.alt, a.print)
            }).collect();
            assert_eq!(actual, expected, "{policy:?}, query {query:?}");
        }
    }
}

#[test]
fn snapshot_construction_validates_directly_modified_entries() {
    use church_slavonic::error::LexiconBuildProblem as Problem;
    let original = noun("validated.n", "N1t", "a", "-", "-");
    let problem = |entries| Lexicon::try_from_lexemes(Recension::Synodal, entries).err().unwrap().problem;
    assert_eq!(problem(vec![original.clone(), original.clone()]), Problem::DuplicateIdentity);
    let mut entry = original.clone();
    entry.recension = Recension::OldChurchSlavonic;
    assert_eq!(problem(vec![entry]), Problem::MixedProfile);
    let mut entry = original.clone();
    entry.stress = "corrupt".into();
    assert!(matches!(problem(vec![entry]), Problem::InvalidStress(_)));
    let mut entry = original.clone();
    entry.class = "missing".into();
    assert_eq!(problem(vec![entry]), Problem::UnknownClass("missing".into()));
    let mut entry = original.clone();
    entry.stems = vec![("obl".into(), "раб".into()), ("obl".into(), "друг".into())];
    assert_eq!(problem(vec![entry]), Problem::DuplicateField("stem:obl".into()));
    let mut entry = original.clone();
    entry.overrides = vec![(Cell::infinitive(), "нести".into())];
    assert_eq!(problem(vec![entry]), Problem::WrongCell(Cell::infinitive()));
    let nom = Cell::noun(Case::Nominative, Number::Singular);
    let mut entry = original.clone();
    entry.overrides = vec![(nom, "ра́бъ".into()), (nom, "ра̀бъ".into())];
    assert_eq!(problem(vec![entry]), Problem::DuplicateField("override:nom.sg".into()));
    let lex = Lexicon::try_from_lexemes(Recension::Synodal, vec![original]).unwrap();
    assert_eq!(lex.recension(), Recension::Synodal);
    assert!(!lex.analyze_with("ра́бъ", MatchPolicy::Exact).is_empty());
}
