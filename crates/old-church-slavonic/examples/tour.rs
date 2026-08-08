use old_church_slavonic::advanced::cells::{
    AdjectiveCell, AdjectiveForm, FiniteVerbCell, NounCell,
};
use old_church_slavonic::advanced::raw_features::dictionary_paradigm_by_id;
use old_church_slavonic::advanced::rules::{
    AdjectiveClass, AdjectiveLexeme, NounClass, NounLexeme, NumberRestriction, VerbClass,
    VerbLexeme, adjective_with, finite_verb_with, noun_with,
};
use old_church_slavonic::{
    Adjective, Animacy, Case, FiniteTense, Gender, InflectionError, Noun, Number, Person, Verb,
    aorist, noun,
};

fn main() -> Result<(), InflectionError> {
    let dual = noun("обѣдъ", Case::Dative, Number::Dual)?;
    println!("dictionary dual: {dual:?}");

    let meal = Noun::new("обѣдъ")?;
    println!("dictionary noun paradigm: {} cells", meal.paradigm().len());

    let good = Adjective::new("добръ")?;
    let short = good.short(
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    )?;
    let long = good.long(
        Case::Nominative,
        Number::Singular,
        Gender::Masculine,
        Animacy::Inanimate,
    )?;
    println!("short/long: {short:?} / {long:?}");

    let aorist = aorist("бꙑти", Person::First, Number::Singular)?;
    println!("dictionary verb variants: {aorist:?}");
    let be = Verb::new("бꙑти")?;
    let raw = dictionary_paradigm_by_id(be.id())?;
    println!("dictionary verb paradigm: {} extracted cells", raw.len());

    let oov = noun_with(
        &NounLexeme {
            lemma: "роботъ".to_string(),
            class: NounClass::OMasculineHard,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            number_restriction: NumberRestriction::All,
        },
        NounCell {
            case: Case::Locative,
            number: Number::Plural,
        },
    )?;
    println!("explicit OOV with trace: {oov:?}");

    let predicted_long = adjective_with(
        &AdjectiveLexeme {
            lemma: "новъ".to_string(),
            class: AdjectiveClass::Hard,
        },
        AdjectiveCell {
            case: Case::Nominative,
            number: Number::Singular,
            gender: Gender::Masculine,
            animacy: Animacy::Inanimate,
            form: AdjectiveForm::Long,
        },
    )?;
    println!("predicted adjective: {predicted_long:?}");

    let mut verb_lexeme = VerbLexeme::new("правити", VerbClass::II1);
    verb_lexeme.stems.present = Some("прав".to_string());
    verb_lexeme.stems.present_first_singular = Some("правл".to_string());
    let explicit_verb = finite_verb_with(
        &verb_lexeme,
        FiniteVerbCell {
            tense: FiniteTense::Present,
            person: Person::First,
            number: Number::Singular,
        },
    )?;
    println!("explicit verb stem: {explicit_verb:?}");

    println!(
        "ambiguity: {:?}",
        noun("блѧдь", Case::Nominative, Number::Singular)
    );
    let variants = noun("аблань", Case::Genitive, Number::Dual)?;
    println!("source-ordered variants: {variants:?}");
    Ok(())
}
