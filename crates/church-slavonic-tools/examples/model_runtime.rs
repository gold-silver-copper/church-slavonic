//! Constructed scaling benchmark; duplicated lexemes are not linguistic coverage.
use church_slavonic::{
    matching::MatchPolicy,
    morphology::{LexemeId, Model, OrthographyId},
};
use church_slavonic_tools::model_artifact::Artifact;
use std::{error::Error, time::Instant};
fn main() -> Result<(), Box<dyn Error>> {
    for copies in [1, 25, 100] {
        let mut artifact: Artifact = serde_json::from_slice(include_bytes!(
            "../../../data/rewrite/ocs-finite-model.json"
        ))?;
        let originals = std::mem::take(&mut artifact.input.lexemes);
        for copy in 0..copies {
            for entry in &originals {
                let mut entry = entry.clone();
                entry.id = LexemeId(format!("{}-{copy}", entry.id.0));
                artifact.input.lexemes.push(entry);
            }
        }
        let start = Instant::now();
        let model = Model::build(artifact.input)?;
        let build = start.elapsed();
        let profile = OrthographyId("o-lrc-finite-tables".into());
        let start = Instant::now();
        let first = model.analyze("мол҄ꙗашє", &profile, MatchPolicy::Exact)?;
        let first_time = start.elapsed();
        let count = first.candidates.len();
        let start = Instant::now();
        for _ in 0..20 {
            assert_eq!(
                model
                    .analyze("мол҄ꙗашє", &profile, MatchPolicy::Exact)?
                    .candidates
                    .len(),
                count
            );
            assert!(
                model
                    .analyze("unknown", &profile, MatchPolicy::Exact)?
                    .candidates
                    .is_empty()
            );
        }
        let warm_time = start.elapsed();
        let index = model.analysis_index_stats()?;
        println!(
            "{}",
            serde_json::json!({"lexemes":copies*2,"expected_cells":copies*38,"candidates":count,"build_us":build.as_micros(),"first_query_us":first_time.as_micros(),"warm_40_queries_us":warm_time.as_micros(),"index":index,"population":"Constructed duplicates of two supplied teaching paradigms; no coverage or generalization claim"})
        );
    }
    Ok(())
}
