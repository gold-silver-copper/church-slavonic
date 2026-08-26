use super::{reflexive_base_candidates, reflexive_surface};

#[test]
fn reflexive_surface_follows_alypy_73() {
    assert_eq!(reflexive_surface("собраша"), "собрашасѧ");
    assert_eq!(reflexive_surface("возврати́"), "возврати́сѧ");
    assert_eq!(reflexive_surface("клѧ́тъ"), "клѧ́тсѧ");
    assert_eq!(reflexive_surface("да́стъ"), "да́стсѧ");
    assert_eq!(reflexive_surface("бо́йте"), "бо́йтесѧ");
    assert_eq!(reflexive_surface("ѡ҆блече́"), "ѡ҆блече́сѧ");
    assert_eq!(reflexive_surface("бои́мъ"), "бои́мсѧ");
}

#[test]
fn j_series_imperative_follows_alypy_93_vowel_stems() {
    use super::{ImperativeFormation, imperative_ending};
    use crate::{ImperativeCell, Number, Person};
    let cell = |person, number| ImperativeCell { person, number };
    let f = ImperativeFormation::JSeries;
    assert_eq!(
        imperative_ending(f, cell(Person::Second, Number::Singular)),
        "й"
    );
    assert_eq!(
        imperative_ending(f, cell(Person::Third, Number::Singular)),
        "й"
    );
    assert_eq!(
        imperative_ending(f, cell(Person::Second, Number::Plural)),
        "йте"
    );
    assert_eq!(
        imperative_ending(f, cell(Person::First, Number::Plural)),
        "ймъ"
    );
    assert_eq!(
        imperative_ending(f, cell(Person::First, Number::Dual)),
        "йва"
    );
    assert_eq!(
        imperative_ending(f, cell(Person::Second, Number::Dual)),
        "йта"
    );
}

#[test]
fn reflexive_base_candidates_restore_the_deleted_jer_only_after_a_consonant() {
    assert_eq!(reflexive_base_candidates("собрашасѧ"), vec!["собраша"]);
    assert_eq!(reflexive_base_candidates("клѧтсѧ"), vec!["клѧт", "клѧтъ"]);
    assert_eq!(
        reflexive_base_candidates("возврати\u{301}сѧ"),
        vec!["возврати\u{301}", "возврати\u{300}"]
    );
    assert_eq!(reflexive_base_candidates("бойсѧ"), vec!["бой"]);
    assert!(reflexive_base_candidates("сѧ").is_empty());
    assert!(reflexive_base_candidates("рабъ").is_empty());
}
