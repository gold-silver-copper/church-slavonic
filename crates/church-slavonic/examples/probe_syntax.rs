use church_slavonic::*;
fn main() {
    let r = Recension::Synodal;
    for (w, c) in [("нача́ло", Case::Locative), ("не́бо", Case::Accusative), ("землѧ̀", Case::Accusative), ("вода̀", Case::Genitive)] {
        println!("{w} {:?}: {}", c, ChurchSlavonic::noun(w, &c, &Number::Singular, &r));
    }
    println!("сотвори́ти aor3sg: {}", ChurchSlavonic::verb("сотвори́ти", &Person::Third, &Number::Singular, &Tense::Aorist, &Form::Finite, &r));
    println!("рещѝ aor3sg: {}", ChurchSlavonic::verb("рещѝ", &Person::Third, &Number::Singular, &Tense::Aorist, &Form::Finite, &r));
    println!("бы́ти aor3sg: {}", ChurchSlavonic::verb("бы́ти", &Person::Third, &Number::Singular, &Tense::Aorist, &Form::Finite, &r));
    println!("бы́ти pres3sg: {}", ChurchSlavonic::verb("бы́ти", &Person::Third, &Number::Singular, &Tense::Present, &Form::Finite, &r));
    println!("вели́кїй acc sg n: {}", ChurchSlavonic::adj("вели́кїй", &Case::Accusative, &Number::Singular, &Gender::Neuter, &Degree::Positive, &r));
}
