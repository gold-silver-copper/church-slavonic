use church_slavonic::*;
fn main() {
    let r = Recension::Synodal;
    for pos in [PartOfSpeech::Noun, PartOfSpeech::Adjective, PartOfSpeech::Verb, PartOfSpeech::NonPersonalPronoun] {
        println!("{:?}: {}", pos, ChurchSlavonic::lemmas(pos, &r).count());
    }
    println!("has неꙋстро́енъ: {}", ChurchSlavonic::lemmas(PartOfSpeech::Adjective, &r).any(|l| l=="неꙋстро́енъ"));
}
