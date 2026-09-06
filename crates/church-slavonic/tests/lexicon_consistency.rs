//! Lexicon self-consistency: a unit test, not a metric. Every lexicon
//! line names a known class, every override and variant is reproduced
//! when asked for, and the whole paradigm of every lexeme prints without
//! a panic. A line that breaks this is a lexicon error, reported with its
//! id.

use church_slavonic::{Cell, Lexicon, Pos, Recension};

fn check(lexicon: &Lexicon, recension: Recension) -> Vec<String> {
    let mut problems = Vec::new();
    for lexeme in lexicon.iter() {
        if lexeme.pos != Pos::Closed && lexeme.class().is_none() {
            problems.push(format!("{}: unknown class {}", lexeme.id, lexeme.class));
            continue;
        }
        let paradigm = lexeme.paradigm();
        if paradigm.is_empty() {
            problems.push(format!("{}: empty paradigm", lexeme.id));
        }
        for (cell, printed) in &lexeme.overrides {
            let got = lexeme.inflect(*cell).ok().map(|f| f.print(recension));
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
        if lexeme.src.is_empty() {
            problems.push(format!("{}: no provenance", lexeme.id));
        }
        // the citation cell prints the lemma (or an override says otherwise)
        let citation = match lexeme.pos {
            Pos::Noun if !lexeme.note.contains("pl-tantum") => Some("nom.sg"),
            // a possessive's lemma is its short nominative (а҆арѡ́новъ); a
            // compound's enclitic (пе́рвыйнадесѧть) sits after the long one
            Pos::Adjective => {
                let encl = lexeme.stems.iter().find(|(k, _)| k == "encl").map(|(_, v)| v.as_str()).unwrap_or("");
                let letters = church_slavonic::Form::from_print(&lexeme.lemma).letters;
                let core = letters.strip_suffix(encl).unwrap_or(&letters);
                // the long nominative: -ый/-їй in the print, -ꙑи/-ии in OCS
                let long = core.ends_with('й') || core.ends_with("ыи") || core.ends_with("ии");
                Some(if long { "long.pos.m.sg.nom" } else { "short.pos.m.sg.nom" })
            }
            Pos::Verb => Some("inf"),
            Pos::Pronoun if lexeme.class().is_some_and(|c| c.name.starts_with("PA") || c.name.starts_with("PN")) => Some("m.sg.nom"),
            _ => None,
        };
        if let Some(name) = citation
            && !lexeme.overrides.iter().any(|(c, _)| c.name() == name)
            && let Ok(cell) = Cell::parse(lexeme.pos, name)
            && let Ok(form) = lexeme.inflect(cell)
            && church_slavonic::orthography::comparison_key(&form.print(recension))
                != church_slavonic::orthography::comparison_key(&lexeme.lemma)
        {
            // an adjective's lemma may be either series' nominative (OCS
            // соуи, прокꙑи): the other series is asked before it is a problem
            let other = match name {
                "long.pos.m.sg.nom" => Some("short.pos.m.sg.nom"),
                "short.pos.m.sg.nom" => Some("long.pos.m.sg.nom"),
                _ => None,
            };
            let other_matches = other
                .and_then(|n| Cell::parse(lexeme.pos, n).ok())
                .and_then(|c| lexeme.inflect(c).ok())
                .is_some_and(|f| church_slavonic::orthography::comparison_key(&f.print(recension)) == church_slavonic::orthography::comparison_key(&lexeme.lemma));
            if !other_matches {
                problems.push(format!("{}: {name} {} is not the lemma {}", lexeme.id, form.print(recension), lexeme.lemma));
            }
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
#[cfg(feature = "ocs")]
fn ocs_lexicon_is_consistent() {
    let problems = check(Lexicon::ocs(), Recension::OldChurchSlavonic);
    assert!(problems.is_empty(), "{} problems:\n{}", problems.len(), problems.join("\n"));
}

#[test]
fn ids_are_lemma_plus_pos() {
    let mut seen = std::collections::HashSet::new();
    #[cfg(feature = "ocs")]
    let lexemes: Vec<&church_slavonic::Lexeme> = Lexicon::synodal().iter().chain(Lexicon::ocs().iter()).collect();
    #[cfg(not(feature = "ocs"))]
    let lexemes: Vec<&church_slavonic::Lexeme> = Lexicon::synodal().iter().collect();
    for lexeme in lexemes {
        assert!(seen.insert((lexeme.id.clone(), Lexicon::synodal().iter().any(|l| std::ptr::eq(l, lexeme)))), "{}: duplicate id", lexeme.id);
        // the id's stem is the lemma's letters up to typography (оу/ꙋ, ꙑ/ы)
        let stem = church_slavonic::orthography::comparison_key(&lexeme.lemma);
        let id_stem = lexeme.id.split('.').next().map(church_slavonic::orthography::comparison_key).unwrap_or_default();
        // an id never moves (3.1): where the attested citation form
        // replaced the source's headword (4.1: богоме́рзскїй under
        // богомерзкій.a), the id follows the headword the note keeps
        let headword = lexeme.note.split("; ").find_map(|n| n.strip_prefix("headword ")).map(church_slavonic::orthography::comparison_key);
        assert!(
            id_stem == stem || headword.as_deref() == Some(id_stem.as_str()),
            "{}: id does not follow the lemma {}",
            lexeme.id,
            lexeme.lemma
        );
    }
}
