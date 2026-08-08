use old_church_slavonic::{Case, Number, noun};

fn main() {
    let form = noun("обѣдъ", Case::Dative, Number::Dual);
    println!("{form:?}");
}
