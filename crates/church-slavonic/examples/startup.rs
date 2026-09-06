//! A start-up measurement (3.4 → 4.0 Part 0.4): the lexicon's first use,
//! the analyzer's first use (the index of every form), the split of the
//! index's cost between generating the forms and keying them.
fn main() {
    use church_slavonic::orthography::comparison_key;
    let t = std::time::Instant::now();
    let syn = church_slavonic::Lexicon::synodal();
    let parse = t.elapsed();
    let sample: Vec<_> = syn.iter().take(3000).collect();
    let t = std::time::Instant::now();
    let mut prints: Vec<String> = Vec::new();
    for l in &sample {
        for (_, forms) in l.all_forms() {
            for (_, p) in forms {
                prints.push(p);
            }
        }
    }
    let generated = t.elapsed();
    let t = std::time::Instant::now();
    let keys: usize = prints.iter().map(|p| comparison_key(p).len()).sum();
    let key = t.elapsed();
    println!("parse {parse:?} ({} lexemes); 3000 lexemes: {} forms generated in {generated:?} ({:.1} µs/form), keyed in {key:?} ({:.1} µs/form, {keys} bytes)", syn.len(), prints.len(), generated.as_secs_f64() * 1e6 / prints.len() as f64, key.as_secs_f64() * 1e6 / prints.len() as f64);
    let t = std::time::Instant::now();
    let r = syn.analyze("рабѡ́мъ");
    println!("first analyze {:?} ({} readings, index {} entries)", t.elapsed(), r.len(), syn.index().len());
}
