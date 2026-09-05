//! Fitting an attested paradigm to a class and a stress paradigm.
//!
//! Given the attested forms per cell (primary first) and a candidate
//! class, the fit finds the stress paradigm that reproduces the most
//! attested primaries, then lists what the class + paradigm still miss:
//! overrides (a primary they do not produce) and variants (other attested
//! forms they do not produce either).

use church_slavonic::cell::{Cell, Pos};
use church_slavonic::form::Form;
use church_slavonic::grammar::{Number, Recension};
use church_slavonic::lexicon::{Lexeme, Provenance};
use church_slavonic::paradigm::{Class, Subject};
use church_slavonic::stress::{Place, resolve};
use std::collections::BTreeMap;

/// The attested forms of one cell, primary first, as print strings with
/// the source's count (0 where the source has none).
pub type Attested = BTreeMap<Cell, Vec<(String, u64)>>;

/// What one attested primary says about where its stress falls: the
/// stressed vowel's index and the resolution context of the class
/// letters it matched (the enclitic's vowels excluded).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StressSample {
    pub index: u8,
    pub stem_vowels: usize,
    pub total: usize,
}

/// Cells whose every attestation came from a bundled tag set (`gen/acc`):
/// the source could not tell the cell's own form from the other's, so any
/// alternative the class offers satisfies it.
pub type Bundled = std::collections::BTreeSet<Cell>;

/// What the stress of one attested primary says about the paradigm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Evidence {
    Stem,
    End,
    /// The last stem vowel, where that is neither the lemma's stem vowel
    /// nor the ending (a comparative's suffix).
    StemLast,
    /// Stem and ending coincide here (no ending vowel): says nothing.
    Either,
    /// Neither: an explicit index.
    Index(u8),
    /// The letters differ; stress unreadable.
    Letters,
    /// The attested form carries no stress.
    None,
}

/// Are two print forms one form up to what a civil transliteration cannot
/// encode? Polyakov and ru.wiktionary write `і` for the print's positional
/// `ї` and `я` for both `ꙗ` and `ѧ`; a source form differing from the
/// prediction only there is the prediction's (lookup invariant 5 of 1.x,
/// now an import rule). Equal spellings are equal.
pub fn translit_equal(a: &str, b: &str) -> bool {
    use unicode_normalization::UnicodeNormalization;
    let fold = |s: &str| -> String {
        let folded: String = s
            .nfc()
            .map(|c| match c {
                'ї' => 'і',
                'ꙗ' => 'ѧ',
                other => other,
            })
            .collect();
        // the ligature ѿ against a spelled-out ѡ҆т/ѡт
        folded.replace("ѡ\u{486}т", "ѿ").replace("ѡт", "ѿ")
    };
    fold(a) == fold(b)
}

/// Read the stress evidence of an attested primary against the class's
/// letters for the cell (any alternative), and which alternative matched.
pub fn evidence(class: &Class, subject: &Subject<'_>, lemma_stress: Option<u8>, cell: Cell, printed: &str) -> Evidence {
    evidence_with_alt(class, subject, lemma_stress, cell, printed).0
}

pub fn evidence_with_alt(
    class: &Class,
    subject: &Subject<'_>,
    lemma_stress: Option<u8>,
    cell: Cell,
    printed: &str,
) -> (Evidence, Option<usize>) {
    let (e, m) = evidence_full(class, subject, lemma_stress, cell, printed);
    (e, m.map(|m| m.0))
}

/// Does the attested print carry the number mark — a kamora, or a wide
/// `ѡ`/`є` where the class's letters have the narrow one?
fn observed_mark(attested: &Form, class_letters: &str) -> bool {
    if attested.number_mark {
        return true;
    }
    attested
        .letters
        .chars()
        .zip(class_letters.chars())
        .any(|(a, c)| (a == 'ѡ' && c == 'о') || (a == 'є' && c == 'е'))
}

/// Evidence, the matched alternative, and whether the print marked it.
pub fn evidence_full(
    class: &Class,
    subject: &Subject<'_>,
    lemma_stress: Option<u8>,
    cell: Cell,
    printed: &str,
) -> (Evidence, Option<(usize, bool)>) {
    let attested = Form::from_print(printed);
    let key = attested.key();
    let alts = class.letters(cell, subject);
    let Some((index, letters)) = alts
        .iter()
        .enumerate()
        .find(|(_, l)| Form::new(l.letters.clone(), None, false).key() == key)
    else {
        return (Evidence::Letters, None);
    };
    let index = (index, observed_mark(&attested, &letters.letters));
    let Some(k) = attested.stress else { return (Evidence::None, Some(index)) };
    let letters = &alts[index.0];
    // a solid enclitic's vowels are not the ending's (возда́стсѧ)
    let total = attested.letters.chars().filter(|c| church_slavonic::orthography::is_vowel_letter(*c)).count().saturating_sub(usize::from(letters.tail_vowels));
    let stem = resolve(Place::Stem, lemma_stress, letters.stem_vowels, total);
    let end = resolve(Place::End, lemma_stress, letters.stem_vowels, total);
    let last = resolve(Place::StemLast, lemma_stress, letters.stem_vowels, total);
    let e = match (stem == Some(k), end == Some(k)) {
        (true, true) => Evidence::Either,
        (true, false) => Evidence::Stem,
        (false, true) => Evidence::End,
        (false, false) if last == Some(k) => Evidence::StemLast,
        (false, false) => Evidence::Index(k),
    };
    (e, Some(index))
}

/// The stress sample of an attested primary: its stressed vowel and the
/// resolution context of the class letters it matched; `None` when the
/// letters differ or the form carries no stress.
pub fn stress_sample(class: &Class, subject: &Subject<'_>, cell: Cell, printed: &str) -> Option<StressSample> {
    let attested = Form::from_print(printed);
    let key = attested.key();
    let alts = class.letters(cell, subject);
    let letters = alts.iter().find(|l| Form::new(l.letters.clone(), None, false).key() == key)?;
    let index = attested.stress?;
    let total = attested.letters.chars().filter(|c| church_slavonic::orthography::is_vowel_letter(*c)).count().saturating_sub(usize::from(letters.tail_vowels));
    Some(StressSample { index, stem_vowels: letters.stem_vowels, total })
}

/// The stress column that explains the samples with the fewest
/// exceptions: every paradigm of the inventory (`a`, `b`, then
/// `lexicon/stress.tsv` in order) is tried bare, then with one number
/// moved (`{pl=E}`); a cell the paradigm's resolved index misses becomes
/// an exception (`S`, `E`, `L` or the index, whichever says it simplest);
/// ties go to the simpler column, then to the inventory's order. A
/// lexeme without a readable sample falls back to the evidence.
pub fn stress_column(pos: Pos, evidence: &BTreeMap<Cell, Evidence>, samples: &BTreeMap<Cell, StressSample>, lemma_stress: Option<u8>) -> String {
    if samples.is_empty() {
        return stress_column_by_evidence(pos, evidence);
    }
    use church_slavonic::stress::StressSpec;
    let place_name = |s: &StressSample| -> String {
        let k = Some(s.index);
        if resolve(Place::Stem, lemma_stress, s.stem_vowels, s.total) == k {
            "S".to_string()
        } else if resolve(Place::End, lemma_stress, s.stem_vowels, s.total) == k {
            "E".to_string()
        } else if resolve(Place::StemLast, lemma_stress, s.stem_vowels, s.total) == k {
            "L".to_string()
        } else if resolve(Place::Final, lemma_stress, s.stem_vowels, s.total) == k {
            "F".to_string()
        } else {
            s.index.to_string()
        }
    };
    let names = church_slavonic::stress::paradigm_names();
    let mut candidates: Vec<String> = names.clone();
    for name in &names {
        for number in ["sg", "du", "pl"] {
            for place in ["S", "E"] {
                candidates.push(format!("{name}{{{number}={place}}}"));
            }
        }
    }
    let mut best: Option<((usize, usize, usize), String)> = None;
    for (order, cand) in candidates.iter().enumerate() {
        let Ok(Some(spec)) = StressSpec::parse(cand, pos) else { continue };
        let mut items: Vec<String> = Vec::new();
        for (cell, s) in samples {
            if resolve(spec.place(*cell), lemma_stress, s.stem_vowels, s.total) == Some(s.index) {
                continue;
            }
            items.push(format!("{}={}", cell.name(), place_name(s)));
        }
        let key = (items.len(), spec.complexity(), order);
        if best.as_ref().is_some_and(|(k, _)| *k <= key) {
            continue;
        }
        let column = if items.is_empty() {
            cand.clone()
        } else if let Some(inner) = cand.strip_suffix('}') {
            format!("{inner};{}}}", items.join(";"))
        } else {
            format!("{cand}{{{}}}", items.join(";"))
        };
        best = Some((key, column));
    }
    best.map(|(_, c)| c).unwrap_or_else(|| "a".to_string())
}

/// The pre-3.0 column from evidence alone (kept for lexemes without a
/// readable sample: an unaccented source, letters the class misses): the
/// base (`a` or `b`) is the majority over unambiguous cells; a number
/// whose majority differs from the base adds `sg=`/`du=`/`pl=`; a cell
/// that differs from its number's place adds its own entry. `-` when no
/// cell carries stress.
fn stress_column_by_evidence(pos: Pos, evidence: &BTreeMap<Cell, Evidence>) -> String {
    let place_of = |e: Evidence| match e {
        Evidence::Stem => Some(Place::Stem),
        Evidence::End => Some(Place::End),
        Evidence::StemLast => Some(Place::StemLast),
        Evidence::Index(n) => Some(Place::Index(n)),
        _ => None,
    };
    // the base is the majority outside the participle blocks: `b` already
    // says the participles are stem-stressed (E;part=S), so they must not
    // outvote a verb's finite cells (дои́ти: дои́ши, дои́ша, доѧ́щꙋю)
    let participle = |c: &Cell| c.block().is_some_and(|b| b.starts_with("part."));
    let (mut stem, mut end) = (0usize, 0usize);
    for (cell, e) in evidence {
        if participle(cell) {
            continue;
        }
        match e {
            Evidence::Stem => stem += 1,
            Evidence::End => end += 1,
            _ => {}
        }
    }
    if stem + end == 0 {
        for e in evidence.values() {
            match e {
                Evidence::Stem => stem += 1,
                Evidence::End => end += 1,
                _ => {}
            }
        }
    }
    if stem + end == 0 && !evidence.values().any(|e| matches!(e, Evidence::Index(_) | Evidence::Either | Evidence::StemLast)) {
        // no readable stress at all: an accented lemma is stem-stressed by
        // default (`a`), an unaccented one has none
        return if evidence.values().all(|e| *e == Evidence::None) && !evidence.is_empty() { "-" } else { "a" }.to_string();
    }
    let base = if end > stem { Place::End } else { Place::Stem };
    let mut number_place: BTreeMap<u8, Place> = BTreeMap::new();
    for number in [Number::Singular, Number::Dual, Number::Plural] {
        let (mut s, mut e) = (0, 0);
        for (cell, ev) in evidence {
            if cell.number() != Some(number) || participle(cell) {
                continue;
            }
            match ev {
                Evidence::Stem => s += 1,
                Evidence::End => e += 1,
                _ => {}
            }
        }
        let place = if s + e == 0 {
            base
        } else if e > s {
            Place::End
        } else if s > e {
            Place::Stem
        } else {
            base
        };
        number_place.insert(number as u8, place);
    }
    let name = |p: Place| match p {
        Place::Stem => "S".to_string(),
        Place::End => "E".to_string(),
        Place::StemLast => "L".to_string(),
        Place::Final => "F".to_string(),
        Place::Index(n) => n.to_string(),
    };
    let mut items: Vec<String> = Vec::new();
    for number in [Number::Singular, Number::Dual, Number::Plural] {
        let place = number_place[&(number as u8)];
        if place != base {
            items.push(format!("{}={}", church_slavonic::cell::number_name(number), name(place)));
        }
    }
    let base_name = if base == Place::End { "b" } else { "a" };
    // the exceptions are read against the paradigm's own answer (the
    // named `b` places the participles on the stem)
    let draft = if items.is_empty() { base_name.to_string() } else { format!("{base_name}{{{}}}", items.join(";")) };
    let spec = church_slavonic::stress::StressSpec::parse(&draft, pos).ok().flatten();
    for (cell, ev) in evidence {
        let Some(place) = place_of(*ev) else { continue };
        let expected = spec.as_ref().map(|s| s.place(*cell)).unwrap_or_else(|| cell.number().map(|n| number_place[&(n as u8)]).unwrap_or(base));
        if place != expected {
            items.push(format!("{}={}", cell.name(), name(place)));
        }
    }
    if items.is_empty() {
        base_name.to_string()
    } else {
        format!("{base_name}{{{}}}", items.join(";"))
    }
}

/// A source form in the print's own typography.
pub fn canonical(printed: &str) -> String {
    canonical_in(printed, Recension::Synodal)
}

/// [`canonical`] in a recension's typography.
pub fn canonical_in(printed: &str, recension: Recension) -> String {
    Form::from_print(printed).print(recension)
}

/// The result of fitting one class.
pub struct Fit {
    pub lexeme: Lexeme,
    /// Attested primaries the fit reproduces as the PRIMARY form.
    pub reproduced: usize,
    /// Attested primaries reachable through any alternative or variant
    /// (the analyzer's view).
    pub reachable: usize,
    /// Overrides whose form no class alternative produces: true exceptions.
    pub exceptions: usize,
    pub attested: usize,
    /// Cells whose letters the class gets wrong.
    pub letter_misses: Vec<Cell>,
    /// Cells whose letters are right and stress wrong (before the paradigm).
    pub stress_misses: Vec<Cell>,
    /// Which class alternative each attested primary's letters matched,
    /// and whether the print marked it.
    pub alt_matches: Vec<(Cell, Option<(usize, bool)>)>,
    /// The stress evidence per attested cell.
    pub evidence: BTreeMap<Cell, Evidence>,
}

/// Fit `attested` to `class`, producing a lexeme line.
#[allow(clippy::too_many_arguments)]
pub fn fit(
    id: &str,
    lemma: &str,
    pos: Pos,
    recension: Recension,
    class: &Class,
    gender: Option<church_slavonic::grammar::Gender>,
    animate: Option<bool>,
    stems: Vec<(String, String)>,
    attested: &Attested,
    bundled: &Bundled,
    src: Vec<String>,
    note: String,
) -> Fit {
    let lemma_form = Form::from_print(lemma);
    let subject = Subject { lemma: &lemma_form.letters, animate, stems: &stems };
    let mut ev = BTreeMap::new();
    let mut samples = BTreeMap::new();
    let mut letter_misses = Vec::new();
    let mut alt_matches = Vec::new();
    for (cell, forms) in attested {
        let Some((primary, _)) = forms.first() else { continue };
        let (e, alt) = evidence_full(class, &subject, lemma_form.stress, *cell, primary);
        if e == Evidence::Letters {
            letter_misses.push(*cell);
        }
        alt_matches.push((*cell, alt));
        ev.insert(*cell, e);
        if let Some(sample) = stress_sample(class, &subject, *cell, primary) {
            samples.insert(*cell, sample);
        }
    }
    let stress = stress_column(pos, &ev, &samples, lemma_form.stress);
    let mut lexeme = Lexeme {
        id: id.to_string(),
        lemma: lemma.to_string(),
        pos,
        gender,
        animate,
        class: class.name.clone(),
        stress,
        stems,
        overrides: Vec::new(),
        variants: Vec::new(),
        src,
        note,
        variant_weights: Vec::new(),
        provenance: Provenance::Attested,
        recension,
    };
    let mut reproduced = 0;
    let mut reachable = 0;
    let mut exceptions = 0;
    let mut stress_misses = Vec::new();
    for (cell, forms) in attested {
        let Some((primary, _)) = forms.first() else { continue };
        let predicted = lexeme.inflect(*cell).map(|f| f.print(recension));
        let any_alt = lexeme.forms(*cell).iter().any(|f| translit_equal(&f.print(recension), primary));
        let satisfied = predicted.as_deref().is_some_and(|p| translit_equal(p, primary))
            || (bundled.contains(cell) && any_alt);
        if satisfied {
            reproduced += 1;
            reachable += 1;
        } else {
            if any_alt {
                reachable += 1;
            } else {
                exceptions += 1;
            }
            if !letter_misses.contains(cell) {
                stress_misses.push(*cell);
            }
            // stored in the print's typography (the source's і where the
            // print writes ї, its я for ѧ/ꙗ), never in the source's
            lexeme.overrides.push((*cell, canonical_in(primary, recension)));
        }
    }
    // variants: other attested forms the class's alternatives do not
    // give, each with the source's count as its weight
    for (cell, forms) in attested {
        let produced: Vec<String> = lexeme.forms(*cell).iter().map(|f| f.print(recension)).collect();
        let mut extra: Vec<String> = Vec::new();
        for (f, count) in forms.iter().skip(1) {
            let c = canonical_in(f, recension);
            if !produced.iter().any(|p| translit_equal(p, &c)) && !extra.contains(&c) {
                if *count > 0 {
                    lexeme.variant_weights.push((*cell, c.clone(), u32::try_from(*count).unwrap_or(u32::MAX)));
                }
                extra.push(c);
            }
        }
        if !extra.is_empty() {
            lexeme.variants.push((*cell, extra));
        }
    }
    Fit { lexeme, reproduced, reachable, exceptions, attested: attested.len(), letter_misses, stress_misses, alt_matches, evidence: ev }
}
