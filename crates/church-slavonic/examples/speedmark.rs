use church_slavonic::*;
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let ocs = Recension::OldChurchSlavonic;
    let syn = Recension::Synodal;
    // Generally the worst case: a long word outside the tables, so every call
    // probes the map (twice, folded) and then runs the rule.
    let words = ["ꙁꙁꙁꙁꙁꙁꙁꙁꙁꙁꙁꙁꙁꙁꙁꙁꙁъ"];
    let tabled = ["ра́бъ"];

    run_benchmark("noun (rule)", &words, |w| {
        ChurchSlavonic::noun(w, &Case::Genitive, &Number::Plural, &ocs)
    });
    run_benchmark("noun (table)", &tabled, |w| {
        ChurchSlavonic::noun(w, &Case::Genitive, &Number::Plural, &syn)
    });
    run_benchmark("verb", &words, |w| {
        ChurchSlavonic::verb(
            w,
            &Person::Third,
            &Number::Singular,
            &Tense::Present,
            &Form::Finite,
            &ocs,
        )
    });
    run_benchmark("adjective", &words, |w| {
        ChurchSlavonic::adj(
            w,
            &Case::Genitive,
            &Number::Singular,
            &Gender::Feminine,
            &Degree::Comparative,
            &ocs,
        )
    });
    run_benchmark("pronoun", &[""], |_| {
        ChurchSlavonic::pronoun(
            &Person::Third,
            &Number::Plural,
            &Gender::Neuter,
            &Case::Instrumental,
            &syn,
        )
        .to_string()
    });
}

fn run_benchmark(label: &str, words: &[&str], f: impl Fn(&str) -> String) {
    let iterations = 1_000_000;
    let start = Instant::now();
    for _ in 0..iterations {
        for w in words {
            black_box(f(black_box(w)));
        }
    }
    let elapsed = start.elapsed();
    let calls = iterations * words.len();
    println!(
        "{label:>14}: {calls} calls in {elapsed:?} = {:.0} calls/sec",
        calls as f64 / elapsed.as_secs_f64()
    );
}
