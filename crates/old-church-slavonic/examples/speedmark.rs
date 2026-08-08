use old_church_slavonic::{Case, Number, noun};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    const CALLS: usize = 1_000_000;
    let started = Instant::now();
    for _ in 0..CALLS {
        black_box(
            noun(
                black_box("обѣдъ"),
                black_box(Case::Dative),
                black_box(Number::Dual),
            )
            .expect("bundled golden lexeme"),
        );
    }
    let elapsed = started.elapsed();
    println!(
        "{CALLS} dictionary noun calls in {elapsed:?} ({:.0} calls/s, {:.1} ns/call)",
        CALLS as f64 / elapsed.as_secs_f64(),
        elapsed.as_nanos() as f64 / CALLS as f64
    );
}
