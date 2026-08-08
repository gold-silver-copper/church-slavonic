use old_church_slavonic::{Case, NounCell, Number, noun};

fn main() {
    let form = noun(
        "обѣдъ",
        NounCell {
            case: Case::Dative,
            number: Number::Dual,
        },
    );
    println!("{form:?}");
}
