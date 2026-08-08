use old_church_slavonic::adjective::AdjectiveLexeme;
use old_church_slavonic::noun::NounLexeme;
use old_church_slavonic::verb::VerbLexeme;
use old_church_slavonic::{
    AdjectiveCell, AdjectiveClass, AdjectiveForm, Animacy, Case, FiniteTense, FiniteVerbCell,
    Gender, InflectionError, NounCell, NounClass, Number, NumberRestriction, PartOfSpeech, Person,
    VerbClass,
};

fn main() -> Result<(), InflectionError> {
    let dual = old_church_slavonic::noun(
        "обѣдъ",
        NounCell {
            case: Case::Dative,
            number: Number::Dual,
        },
    )?;
    println!("dictionary dual: {dual:?}");
    let noun_id = old_church_slavonic::lookup("обѣдъ", PartOfSpeech::Noun)?
        .into_iter()
        .next()
        .ok_or(InflectionError::UnknownLemma)?
        .id;
    let noun_paradigm = old_church_slavonic::noun_paradigm(&noun_id)?;
    println!(
        "dictionary noun paradigm: {} cells",
        noun_paradigm.cells.len()
    );

    let adjective_cell = AdjectiveCell {
        case: Case::Nominative,
        number: Number::Singular,
        gender: Gender::Masculine,
        animacy: Animacy::Inanimate,
        form: AdjectiveForm::Short,
    };
    let short = old_church_slavonic::adjective("добръ", adjective_cell)?;
    let long = old_church_slavonic::adjective(
        "добръ",
        AdjectiveCell {
            form: AdjectiveForm::Long,
            ..adjective_cell
        },
    )?;
    println!("short/long: {short:?} / {long:?}");

    let verb = old_church_slavonic::finite_verb(
        "бꙑти",
        FiniteVerbCell {
            tense: FiniteTense::Aorist,
            person: Person::First,
            number: Number::Singular,
        },
    )?;
    println!("dictionary verb variants: {verb:?}");
    let verb_id = old_church_slavonic::lookup("бꙑти", PartOfSpeech::Verb)?
        .into_iter()
        .next()
        .ok_or(InflectionError::UnknownLemma)?
        .id;
    let verb_paradigm = old_church_slavonic::dictionary_paradigm_by_id(&verb_id)?;
    println!(
        "dictionary verb paradigm: {} extracted cells",
        verb_paradigm.cells.len()
    );

    let oov = old_church_slavonic::noun_with(
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

    let predicted_long = old_church_slavonic::adjective_with(
        &AdjectiveLexeme {
            lemma: "новъ".to_string(),
            class: AdjectiveClass::Hard,
        },
        AdjectiveCell {
            form: AdjectiveForm::Long,
            ..adjective_cell
        },
    )?;
    println!("predicted adjective: {predicted_long:?}");

    let explicit_verb = old_church_slavonic::finite_verb_with(
        &VerbLexeme {
            lemma: "правити".to_string(),
            class: VerbClass::II1,
            present_stem: Some("прав".to_string()),
            aorist_stem: Some("прави".to_string()),
        },
        FiniteVerbCell {
            tense: FiniteTense::Present,
            person: Person::First,
            number: Number::Singular,
        },
    )?;
    println!("explicit verb stem: {explicit_verb:?}");

    let ambiguous = old_church_slavonic::noun(
        "блѧдь",
        NounCell {
            case: Case::Nominative,
            number: Number::Singular,
        },
    );
    println!("ambiguity: {ambiguous:?}");

    let multi = old_church_slavonic::noun(
        "аблань",
        NounCell {
            case: Case::Genitive,
            number: Number::Dual,
        },
    )?;
    println!("source-ordered variants: {multi:?}");
    println!(
        "lookup candidates: {:?}",
        old_church_slavonic::lookup("блѧдь", PartOfSpeech::Noun)?
    );
    Ok(())
}
