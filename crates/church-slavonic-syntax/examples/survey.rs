use church_slavonic::Recension;
use church_slavonic_syntax::{bible, lift, node, sexpr};
fn main() {
    let bible = bible::load().expect("parse").expect("present");
    let index = lift::Index::build(&Recension::Synodal);
    let gen1 = &bible.books[0].chapters[0];
    for v in &gen1.verses {
        if v.verse < 9 { continue; }
        let (tree, _) = lift::lift_verse(v.print(), &index);
        println!("(verse 1 {} {})", v.verse, sexpr::print(&node::to_sexpr(&tree)));
    }
}
