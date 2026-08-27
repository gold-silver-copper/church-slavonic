use church_slavonic::*;

fn main() {
    let ocs = Recension::OldChurchSlavonic;
    let syn = Recension::Synodal;

    // --- A phrase in each recension ---
    // "the slave carries the new city" — nominative subject, present verb,
    // accusative object with an agreeing adjective.
    let phrase = format!(
        "{} {} {} {}",
        ChurchSlavonic::noun("рабъ", &Case::Nominative, &Number::Singular, &ocs),
        ChurchSlavonic::verb(
            "нести",
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
            &ocs
        ),
        ChurchSlavonic::adj(
            "новъ",
            &Case::Accusative,
            &Number::Singular,
            &Gender::Masculine,
            &Degree::Positive,
            &ocs
        ),
        ChurchSlavonic::noun("градъ", &Case::Accusative, &Number::Singular, &ocs),
    );
    assert_eq!(phrase, "рабъ несетъ новъ градъ");

    let phrase = format!(
        "{} {} {}",
        ChurchSlavonic::pronoun(
            &Person::First,
            &Number::Singular,
            &Gender::Masculine,
            &Case::Nominative,
            &syn
        ),
        ChurchSlavonic::verb(
            "быти",
            &Person::First,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
            &syn
        ),
        ChurchSlavonic::noun("рабъ", &Case::Nominative, &Number::Singular, &syn),
    );
    assert_eq!(phrase, "а҆́зъ є҆́смь ра́бъ");

    // --- Nouns: rule fallback and tabled exceptions ---
    assert_eq!(
        ChurchSlavonic::noun("градъ", &Case::Locative, &Number::Plural, &ocs),
        "градѣхъ"
    );
    assert_eq!(
        ChurchSlavonic::noun("отьць", &Case::Vocative, &Number::Singular, &ocs),
        "отьче"
    );
    // Sense-numbered keys expose attested variants.
    assert_eq!(
        ChurchSlavonic::noun("сꙑнъ", &Case::Dative, &Number::Singular, &ocs),
        "сꙑнови"
    );
    assert_eq!(
        ChurchSlavonic::noun("сꙑнъ_2", &Case::Dative, &Number::Singular, &ocs),
        "сꙑноу"
    );
    // Synodal cells keep the grammar's accents.
    assert_eq!(
        ChurchSlavonic::noun("жена", &Case::Nominative, &Number::Singular, &syn),
        "жена̀"
    );

    // --- Verbs ---
    assert_eq!(
        ChurchSlavonic::verb(
            "глаголати",
            &Person::Third,
            &Number::Plural,
            &Tense::Present,
            &Form::Finite,
            &ocs
        ),
        "глаголѭтъ"
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "нести",
            &Person::First,
            &Number::Singular,
            &Tense::Aorist,
            &Form::Finite,
            &syn
        ),
        "несо́хъ"
    );
    assert_eq!(
        ChurchSlavonic::verb(
            "бꙑти",
            &Person::Third,
            &Number::Plural,
            &Tense::Present,
            &Form::Finite,
            &ocs
        ),
        "сѫтъ"
    );

    // --- Adjectives and the pronoun ---
    assert_eq!(
        ChurchSlavonic::adj(
            "мꙋдръ",
            &Case::Genitive,
            &Number::Singular,
            &Gender::Feminine,
            &Degree::Positive,
            &syn
        ),
        "мꙋ́дры"
    );
    assert_eq!(
        ChurchSlavonic::pronoun(
            &Person::Third,
            &Number::Singular,
            &Gender::Feminine,
            &Case::Nominative,
            &ocs
        ),
        "ꙗ"
    );

    // --- Case restoration and realisation ---
    assert_eq!(
        ChurchSlavonic::noun("Рабъ", &Case::Dative, &Number::Singular, &syn),
        "Рабꙋ̀"
    );
    assert_eq!(
        ChurchSlavonic::noun("рꙑба", &Case::Genitive, &Number::Singular, &syn),
        "рыбы"
    );

    println!("church-slavonic example: all assertions passed");
}
