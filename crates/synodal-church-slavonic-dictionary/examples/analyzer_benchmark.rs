use std::{hint::black_box, sync::Arc, time::Instant};

use synodal_church_slavonic_dictionary::{analyze, coverage::AnalyzerCache, morphology::Inflector};

const LOOKUP_REPETITIONS: usize = 10_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (construction, cache_hit, analyzer_lookup_batch, indexed_cells, constructions) = {
        let cache = AnalyzerCache::new();

        let started = Instant::now();
        let analyzer = cache.get(Inflector::default())?;
        let construction = started.elapsed();

        let started = Instant::now();
        let cached = cache.get(Inflector::default())?;
        let cache_hit = started.elapsed();
        assert!(Arc::ptr_eq(&analyzer, &cached));

        let started = Instant::now();
        for _ in 0..LOOKUP_REPETITIONS {
            black_box(analyzer.analyze(black_box("бытїе"))?);
        }
        let analyzer_lookup_batch = started.elapsed();
        (
            construction,
            cache_hit,
            analyzer_lookup_batch,
            analyzer.indexed_cell_count(),
            cache.construction_count(),
        )
    };

    black_box(analyze(black_box("бытїе"))?);
    let started = Instant::now();
    for _ in 0..LOOKUP_REPETITIONS {
        black_box(analyze(black_box("бытїе"))?);
    }
    let cached_dictionary_lookup_batch = started.elapsed();

    println!("construction_ns\t{}", construction.as_nanos());
    println!("cache_hit_ns\t{}", cache_hit.as_nanos());
    println!("lookup_repetitions\t{LOOKUP_REPETITIONS}");
    println!(
        "analyzer_lookup_batch_ns\t{}",
        analyzer_lookup_batch.as_nanos()
    );
    println!(
        "analyzer_lookup_mean_ns\t{}",
        analyzer_lookup_batch.as_nanos() / LOOKUP_REPETITIONS as u128
    );
    println!(
        "cached_dictionary_lookup_batch_ns\t{}",
        cached_dictionary_lookup_batch.as_nanos()
    );
    println!(
        "cached_dictionary_lookup_mean_ns\t{}",
        cached_dictionary_lookup_batch.as_nanos() / LOOKUP_REPETITIONS as u128
    );
    println!("indexed_cells\t{indexed_cells}");
    println!("analyzer_constructions\t{constructions}");

    Ok(())
}
