use church_slavonic_legacy::*;

const OCS: Recension = Recension::OldChurchSlavonic;
const SYN: Recension = Recension::Synodal;

#[test]
fn base_surface_helpers_work() {
    assert_eq!(
        ChurchSlavonic::verb(
            "нести",
            &Person::Second,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
            &OCS
        ),
        "несеши"
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "нестѝ",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
            &SYN
        ),
        // The corpus primary in the print's typography (the Synodal `-сти`
        // rule is the dental class, so `нестѝ` is a table row).
        "несꙋ̀"
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "вестѝ",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
            &SYN
        ),
        "ведꙋ̀"
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "глаголати",
            &Person::Second,
            &Number::Singular,
            &Tense::Present,
            &Form::Imperative,
            &OCS
        ),
        "глаголи"
    );
    assert_eq!(
        ChurchSlavonic::noun("градъ", &Case::Genitive, &Number::Singular, &OCS),
        "града"
    );
    assert_eq!(
        ChurchSlavonic::noun("ра́бъ", &Case::Dative, &Number::Singular, &SYN),
        "рабꙋ̀"
    );
    assert_eq!(
        ChurchSlavonic::adj(
            "мꙋ́дръ",
            &Case::Nominative,
            &Number::Singular,
            &Gender::Feminine,
            &Degree::Positive,
            &SYN
        ),
        "мꙋ́дра"
    );
    assert_eq!(
        ChurchSlavonic::adj(
            "мꙋ́дръ",
            &Case::Nominative,
            &Number::Singular,
            &Gender::Masculine,
            &Degree::Comparative,
            &SYN
        ),
        "мꙋ́дрѣй"
    );
    assert_eq!(
        ChurchSlavonic::adj(
            "мꙋ́дръ",
            &Case::Nominative,
            &Number::Singular,
            &Gender::Masculine,
            &Degree::Superlative,
            &SYN
        ),
        "премꙋ́дръ"
    );
    assert_eq!(
        ChurchSlavonic::adj(
            "новъ",
            &Case::Genitive,
            &Number::Singular,
            &Gender::Neuter,
            &Degree::Positive,
            &OCS
        ),
        "нова"
    );
    assert_eq!(ChurchSlavonic::capitalize_first(""), "");
    assert_eq!(ChurchSlavonic::capitalize_first("градъ"), "Градъ");
}

#[test]
fn casing_is_folded_for_lookup_and_restored_on_output() {
    // Title-case and ALL-CAPS input reach the lowercase tables.
    assert_eq!(
        ChurchSlavonic::noun("Ра́бъ", &Case::Dative, &Number::Singular, &SYN),
        "Рабꙋ̀"
    );
    assert_eq!(
        ChurchSlavonic::noun("РА́БЪ", &Case::Dative, &Number::Singular, &SYN),
        "РАБꙊ̀"
    );
    // ...and so does the rule fallback.
    assert_eq!(
        ChurchSlavonic::noun("Градъ", &Case::Genitive, &Number::Singular, &OCS),
        "Града"
    );
    // A Synodal lemma is its accented citation form: the accent is the
    // rule's input (`рꙋка̀` : `рꙋкѝ`), and the print's typography is
    // restored on the letters the caller typed (`у` -> `ꙋ`, the breathing).
    assert_eq!(
        ChurchSlavonic::noun("рука́", &Case::Genitive, &Number::Singular, &SYN),
        "рꙋкѝ"
    );
    assert_eq!(
        ChurchSlavonic::noun("аарѡ́нъ", &Case::Genitive, &Number::Singular, &SYN),
        "а҆арѡ́на"
    );
    // An unaccented Synodal lemma is answered by the rule, unaccented.
    assert_eq!(
        ChurchSlavonic::noun("рабъ", &Case::Dative, &Number::Singular, &SYN),
        "рабꙋ"
    );
}

#[test]
fn rule_output_is_realised_in_the_requested_recension() {
    // An OCS-spelled lemma queried in Synodal answers in Synodal letters, and
    // vice versa; table cells are returned as attested.
    assert_eq!(
        ChurchSlavonic::noun("рꙑба", &Case::Genitive, &Number::Singular, &SYN),
        "рыбы"
    );
    assert_eq!(
        ChurchSlavonic::noun("рыба", &Case::Genitive, &Number::Singular, &OCS),
        "рꙑбꙑ"
    );
    assert_eq!(
        ChurchSlavonic::noun("рѫка", &Case::Dative, &Number::Singular, &SYN),
        "рꙋцѣ"
    );
}
