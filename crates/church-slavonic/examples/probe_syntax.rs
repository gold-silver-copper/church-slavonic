use church_slavonic::*;
fn main() {
    let r = Recension::Synodal;
    for pos in [PartOfSpeech::Noun, PartOfSpeech::Adjective, PartOfSpeech::Verb] {
        println!("{:?}: {}", pos, ChurchSlavonic::lemmas(pos, &r).count());
    }
}
