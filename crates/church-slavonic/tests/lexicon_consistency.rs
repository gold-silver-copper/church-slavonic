//! Lexicon self-consistency: a unit test, not a metric. Every lexicon
//! line names a known class, every override and variant is reproduced
//! when asked for, and the whole paradigm of every lexeme prints without
//! a panic. A line that breaks this is a lexicon error, reported with its
//! id.

use church_slavonic::{Cell, Lexicon, Pos, Recension};

fn check(lexicon: &Lexicon, recension: Recension) -> Vec<String> {
    let mut problems = Vec::new();
    for lexeme in lexicon.iter() {
        if lexeme.pos == Pos::Closed {
            continue;
        }
        if lexeme.class().is_none() {
            problems.push(format!("{}: unknown class {}", lexeme.id, lexeme.class));
            continue;
        }
        let paradigm = lexeme.paradigm();
        if paradigm.is_empty() {
            problems.push(format!("{}: empty paradigm", lexeme.id));
        }
        for (cell, printed) in &lexeme.overrides {
            let got = lexeme.inflect(*cell).map(|f| f.print(recension));
            if got.as_deref() != Some(printed.as_str()) {
                problems.push(format!("{}: override {}={printed} prints as {got:?}", lexeme.id, cell.name()));
            }
        }
        for (cell, variants) in &lexeme.variants {
            let forms: Vec<String> = lexeme.forms(*cell).iter().map(|f| f.print(recension)).collect();
            for v in variants {
                if !forms.contains(v) {
                    problems.push(format!("{}: variant {}={v} is not among the cell's forms {forms:?}", lexeme.id, cell.name()));
                }
            }
        }
        // the nominative singular of a noun is its lemma (or an override)
        if lexeme.pos == Pos::Noun
            && !lexeme.note.contains("pl-tantum")
            && !lexeme.overrides.iter().any(|(c, _)| c.name() == "nom.sg")
            && let Some(nom) = Cell::parse(Pos::Noun, "nom.sg")
            && let Some(form) = lexeme.inflect(nom)
            && church_slavonic::orthography::comparison_key(&form.print(recension))
                != church_slavonic::orthography::comparison_key(&lexeme.lemma)
        {
            problems.push(format!("{}: nom.sg {} is not the lemma {}", lexeme.id, form.print(recension), lexeme.lemma));
        }
    }
    problems
}

#[test]
fn synodal_lexicon_is_consistent() {
    let problems = check(Lexicon::synodal(), Recension::Synodal);
    assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
}

#[test]
fn ocs_lexicon_is_consistent() {
    let problems = check(Lexicon::ocs(), Recension::OldChurchSlavonic);
    assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
}

#[test]
fn ids_are_lemma_plus_pos() {
    for lexeme in Lexicon::synodal().iter().chain(Lexicon::ocs().iter()) {
        let stem = church_slavonic::Form::from_print(&lexeme.lemma).letters;
        let expected = format!("{stem}.{}", lexeme.pos.tag());
        assert!(
            lexeme.id == expected || lexeme.id.starts_with(&format!("{expected}.")),
            "{}: id does not follow the lemma {}",
            lexeme.id,
            lexeme.lemma
        );
    }
}
