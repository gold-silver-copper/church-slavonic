use church_slavonic::*;
fn main() {
    let r = Recension::Synodal;
    println!("npron lemmas: {}", ChurchSlavonic::lemmas(PartOfSpeech::NonPersonalPronoun, &r).count());
    for l in ChurchSlavonic::lemmas(PartOfSpeech::NonPersonalPronoun, &r).take(12) { print!("{l}  "); }
    println!();
    // personal pronouns: does gender matter for p1/2?
    for p in [Person::First, Person::Second, Person::Third] {
        for g in [Gender::Masculine, Gender::Feminine] {
            print!("{:?}/{:?}: {}  ", p, g, ChurchSlavonic::pronoun(&p, &Number::Singular, &g, &Case::Genitive, &r));
        }
        println!();
    }
    // vocative answers nominative?
    println!("pers voc: {}", ChurchSlavonic::pronoun(&Person::First, &Number::Singular, &Gender::Masculine, &Case::Vocative, &r));
    // participle samples
    println!("part pres act short nom m: {}", ChurchSlavonic::participle("нестѝ", &Tense::Present, &Voice::Active, &Series::Short, &Case::Nominative, &Number::Singular, &Gender::Masculine, &r));
    println!("part aor act short nom m: {}", ChurchSlavonic::participle("нестѝ", &Tense::Aorist, &Voice::Active, &Series::Short, &Case::Nominative, &Number::Singular, &Gender::Masculine, &r));
    println!("part pres pass long nom f: {}", ChurchSlavonic::participle("нестѝ", &Tense::Present, &Voice::Passive, &Series::Long, &Case::Nominative, &Number::Singular, &Gender::Feminine, &r));
    // adjective superlative distinct?
    for d in [Degree::Positive, Degree::Comparative, Degree::Superlative] {
        println!("вели́кїй {:?}: {}", d, ChurchSlavonic::adj("вели́кїй", &Case::Nominative, &Number::Singular, &Gender::Masculine, &d, &r));
    }
    println!("npron сво́й gen f sg: {}", ChurchSlavonic::npron("сво́й", &Gender::Feminine, &Number::Singular, &Case::Genitive, &r));
}
