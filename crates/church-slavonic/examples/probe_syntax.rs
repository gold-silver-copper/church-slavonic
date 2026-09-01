use church_slavonic::*;
fn main() {
    let r = Recension::Synodal;
    let has = |pos, w: &str| ChurchSlavonic::lemmas(pos, &r).any(|l| l == w);
    for w in ["госпо́дь","госпо́день","бо́гъ","бо́жїй","глаго́лати","дꙋ́хъ","сы́нъ","ѻ҆те́цъ","свѧты́й","хрїсто́съ","мѣ́сѧцъ","проро́къ","благода́ть","ми́лость","спасе́нїе","і҆зра́иль","і҆ерꙋсали́мъ","і҆исꙋ́съ","а҆́ггелъ","не́бо"] {
        let pos = [PartOfSpeech::Noun, PartOfSpeech::Adjective, PartOfSpeech::Verb]
            .into_iter().filter(|p| has(*p, w)).map(|p| format!("{p:?}")).collect::<Vec<_>>();
        println!("{w}: {:?}", pos);
    }
    println!("не́бо gen: {}", ChurchSlavonic::noun("не́бо", &Case::Genitive, &Number::Singular, &r));
    println!("госпо́дь gen: {}", ChurchSlavonic::noun("госпо́дь", &Case::Genitive, &Number::Singular, &r));
    println!("ѻ҆те́цъ gen: {}", ChurchSlavonic::noun("ѻ҆те́цъ", &Case::Genitive, &Number::Singular, &r));
    println!("свѧты́й gen m sg: {}", ChurchSlavonic::adj("свѧты́й", &Case::Genitive, &Number::Singular, &Gender::Masculine, &Degree::Positive, &r));
    println!("глаго́лати pres3sg: {}", ChurchSlavonic::verb("глаго́лати", &Person::Third, &Number::Singular, &Tense::Present, &Form::Finite, &r));
    println!("хрїсто́съ gen: {}", ChurchSlavonic::noun("хрїсто́съ", &Case::Genitive, &Number::Singular, &r));
    println!("і҆исꙋ́съ nom: {}", ChurchSlavonic::noun("і҆исꙋ́съ", &Case::Nominative, &Number::Singular, &r));
}
