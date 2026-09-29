//! Local compiled transport benchmark; duplicated lexemes are not new coverage.
use church_slavonic::{
    matching::MatchPolicy,
    morphology::{LexemeId, Model, OrthographyId},
};
use church_slavonic_tools::model_artifact::Artifact;
use sha2::{Digest, Sha256};
use std::{error::Error, time::Instant};
fn main() -> Result<(), Box<dyn Error>> {
    for copies in [1, 25, 100] {
        let mut input: Artifact = serde_json::from_slice(include_bytes!(
            "../../../data/rewrite/ocs-finite-model.json"
        ))?;
        let original = std::mem::take(&mut input.input.lexemes);
        for i in 0..copies {
            for e in &original {
                let mut e = e.clone();
                e.id = LexemeId(format!("{}-{i}", e.id.0));
                input.input.lexemes.push(e);
            }
        }
        let source = serde_json::to_vec(&input.input)?;
        let producer = Model::build(input.input)?;
        let profile = OrthographyId("o-lrc-finite-tables".into());
        let runtime = "a".repeat(64); // Controlled identity; CLI hashes its executable.
        let start = Instant::now();
        let first = producer.analyze("мол҄ꙗашє", &profile, MatchPolicy::Exact)?;
        let cold = start.elapsed();
        let count = first.candidates.len();
        let start = Instant::now();
        let bytes = producer.compile_analysis_index(&profile, MatchPolicy::Exact, &runtime)?;
        let export = start.elapsed();
        let digest = format!("{:x}", Sha256::digest(&bytes));
        let mut consumer = Model::build(serde_json::from_slice(&source)?)?;
        let start = Instant::now();
        consumer.install_compiled_analysis_index(
            &bytes,
            &digest,
            &runtime,
            &profile,
            MatchPolicy::Exact,
        )?;
        let restored = start.elapsed();
        let start = Instant::now();
        assert_eq!(
            consumer
                .analyze("мол҄ꙗашє", &profile, MatchPolicy::Exact)?
                .candidates
                .len(),
            count
        );
        let query = start.elapsed();
        println!(
            "{}",
            serde_json::json!({"lexemes":copies*2,"forms":copies*38,"cold_query_us":cold.as_micros(),"export_us":export.as_micros(),"restore_us":restored.as_micros(),"restored_query_us":query.as_micros(),"compiled_bytes":bytes.len(),"index":consumer.analysis_index_stats()?,"scope":"Constructed duplicates; restore includes digest and record validation, excludes model-input loading and executable hashing; no production-scale claim"})
        );
    }
    Ok(())
}
