use church_slavonic::{*, lexicon::parse, sentence::{Sentence, node}};
use church_slavonic_tools::{eval::corpus_matches, sources::ud::{Corpus, CorpusSlot, SequenceToken}, tagger::examples};
fn row(id: &str, class: &str, stress: &str, overrides: &str, variants: &str) -> Lexeme {
    parse(&format!("{id}\tра́бъ\tn\tm\tinan\t{class}\t{stress}\t-\t{overrides}\t{variants}\t-\t-\n"), Pos::Noun).unwrap().remove(0)
}
fn main() {
    let syn = Recension::Synodal;
    let cell = Cell::noun(Case::Nominative, Number::Singular);
    let lex = Lexicon::from_lexemes(syn, vec![row("audit.n", "N1t", "a", "-", "-")]);
    let printed = lex.get("audit.n").unwrap().inflect(cell).unwrap().print(syn);
    let upper = printed.to_uppercase();
    println!("uppercase_exact: query={upper:?} generated={printed:?} exact={}", lex.analyze(&upper).iter().any(|a| a.exact));
    println!("custom_candidates: exact={}", lex.analyze(&printed).iter().filter(|a| a.exact).count());
    let sentence = Sentence::parse(&lex, &printed);
    println!("custom_lexicon: tokens={:?} render={:?}", sentence.tokens(), sentence.print(syn));
    let raw = "  xyz\tqrs\n";
    println!("whitespace: input={raw:?} render={:?}", node::render(&node::verbatim_tree(raw), &syn));
    let orphan = row("orphan.n", "-", "a", "nom.sg=ра́бъ", "-");
    println!("override_only: inflect_ok={} cells={} all_forms={}", orphan.inflect(cell).is_ok(), orphan.cells().len(), orphan.all_forms().len());
    let variants = (1..=260).map(|i| format!("р{}б", "а".repeat(i))).collect::<Vec<_>>().join("|");
    let many = row("many.n", "N1t", "a", "-", &format!("nom.sg={variants}"));
    let prints = many.forms(cell).iter().map(|f| f.print(syn)).collect::<Vec<_>>();
    let query = prints[256].clone();
    let many_lex = Lexicon::from_lexemes(syn, vec![many]);
    let found = many_lex.analyze(&query).into_iter().find(|a| a.exact && a.cell == cell).unwrap();
    println!("alternative_overflow: expected_alt=256 actual_alt={} same_form={}", found.alt, prints[found.alt] == found.print);
    println!("broad_evaluation: noun_лесъ_лсъ={} verb_нести_ести={}", corpus_matches("лесъ", "лсъ", Pos::Noun), corpus_matches("нести", "ести", Pos::Verb));
    let bad = row("bad.n", "N1t", "not_a_stress_paradigm", "-", "-");
    std::panic::set_hook(Box::new(|_| {}));
    println!("invalid_stress: parse_accepted=true inflect_panics={}", std::panic::catch_unwind(|| bad.inflect(cell)).is_err());
    let acc = Cell::noun(Case::Accusative, Number::Singular);
    let mut corpus = Corpus::default();
    corpus.slots = vec![CorpusSlot{lemma:"gold-only-lemma-1".into(),pos:Pos::Noun,cell:acc,surface:printed.clone()}, CorpusSlot{lemma:"gold-only-lemma-2".into(),pos:Pos::Noun,cell,surface:printed.clone()}];
    corpus.sentences = vec![vec![SequenceToken{surface:printed.clone(),lemma:"gold-only-lemma-1".into(),object:false,slots:vec![0]},SequenceToken{surface:printed,lemma:"gold-only-lemma-2".into(),object:false,slots:vec![1]}]];
    let (ex, gold_count, _) = examples(&lex, &corpus);
    println!("gold_lemma_identity: fabricated_gold_lemmas_counted_as_present={gold_count}");
    println!("gold_context: examples={} next_lemma={:?} second_prev_choice={:?}", ex.len(), ex[0].ctx.next_lemma, ex[1].ctx.prev_choice);
    let text = "one.x\tже\tx\t-\t-\tpart\t-\t-\t-\t-\t-\t-\ntwo.x\tже\tx\t-\t-\tconj\t-\t-\t-\t-\t-\t-\n";
    let closed = Lexicon::from_lexemes(syn, parse(text, Pos::Closed).unwrap());
    let lifted = church_slavonic::sentence::lift::Lifter::new(&closed).lift_core("же");
    println!("closed_ambiguity: exact_readings={} lifted={lifted:?}", closed.readings("же").iter().filter(|r| r.exact).count());
    let mut bytes = b"CST1".to_vec();
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());
    bytes.extend_from_slice(&f32::NAN.to_le_bytes());
    bytes.push(99);
    println!("malformed_model: nan_and_trailing_byte_accepted={}", church_slavonic_tagger::Tagger::from_bytes(&bytes).is_some());

}
