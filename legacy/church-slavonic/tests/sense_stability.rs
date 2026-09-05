//! Regression tests pinning the DETERMINISTIC output of sense-numbered keys.
//!
//! Key numbers are assigned by a pure sort of each lemma's emitted forms (see
//! `extractor_legacy::assign`), so for given sources the mapping is fixed and these
//! tests pin it. They are NOT an immutability contract: regenerating from newer
//! sources can renumber a lemma's keys if its attested forms change. What stays
//! true is that every attested variant is reachable through SOME key and that
//! the bare key goes to the rule whenever a regular paradigm is attested.

use church_slavonic_legacy::*;

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::Synodal;

fn noun(word: &str, case: Case, number: Number, r: Recension) -> String {
    ChurchSlavonic::noun(word, &case, &number, &r)
}

#[test]
fn attested_variants_are_numbered_by_their_form_signature() {
    // аблань lists two instrumentals; the signature sort puts `-иѭ` (и < ь)
    // on the bare key and `-ьѭ` at `_2`.
    assert_eq!(
        noun("аблань", Case::Instrumental, Number::Singular, OCS),
        "абланиѭ"
    );
    assert_eq!(
        noun("аблань_2", Case::Instrumental, Number::Singular, OCS),
        "абланьѭ"
    );
    // ра́бъ (a Synodal lemma is its accented citation form): the grammar
    // prints two genitive plurals and the corpus attests more spellings; the
    // sort spreads them over the bare key and `_n`.
    assert_eq!(noun("ра́бъ", Case::Genitive, Number::Plural, SYN), "ра̑бъ");
    assert_eq!(noun("ра́бъ_2", Case::Genitive, Number::Plural, SYN), "рабъ");
    assert_eq!(
        noun("ра́бъ_3", Case::Genitive, Number::Plural, SYN),
        "рабѡ́въ"
    );
}

#[test]
fn the_rule_keeps_the_bare_key_when_a_regular_paradigm_is_attested() {
    assert_eq!(noun("сꙑнъ", Case::Dative, Number::Singular, OCS), "сꙑнови");
    assert_eq!(noun("сꙑнъ_2", Case::Dative, Number::Singular, OCS), "сꙑноу");
}

#[test]
fn a_suffix_strips_only_for_real_table_keys() {
    // A known suffixed key strips and resolves; its bare cell is the base.
    assert_eq!(
        noun("сꙑнъ_2", Case::Nominative, Number::Singular, OCS),
        "сꙑнъ"
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "бы́ти_2",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Infinitive,
            &SYN
        ),
        "бы́ти"
    );
    // A suffix that resolves to no key is opaque: the whole string inflects.
    assert_eq!(
        ChurchSlavonic::verb(
            "котъ_2",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Infinitive,
            &OCS
        ),
        "котъ_2"
    );
    assert_eq!(
        noun("градъ_9", Case::Nominative, Number::Singular, OCS),
        "градъ_9ъ"
    );
}

#[test]
fn recensions_never_share_a_row() {
    // The Synodal rows are keyed `syn:` by the accented lemma; the same
    // letters in OCS are the rule.
    assert_eq!(noun("ра́бъ", Case::Dative, Number::Singular, SYN), "рабꙋ̀");
    assert_eq!(noun("ра́бъ", Case::Dative, Number::Singular, OCS), "рабоу");
    assert_eq!(noun("рабъ", Case::Dative, Number::Singular, OCS), "рабоу");
}
