//! Local compiled-index publication and loading with an explicit caller digest.
use crate::{import::publication::StagedReplacement, model_artifact};
use church_slavonic::{
    matching::MatchPolicy,
    morphology::{MAX_COMPILED_INDEX_BYTES, Model, OrthographyId},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    error::Error,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

/// Conservative identity of the executable file used by these tools. This
/// covers its compiled Rust code/dependencies/configuration, not dynamic system
/// libraries or concurrent replacement of the executable on disk.
pub fn executable_sha256() -> Result<String, Box<dyn Error>> {
    let mut file = File::open(std::env::current_exe()?)?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut total = 0usize;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        total += n;
        if total > 256 * 1024 * 1024 {
            return Err("executable exceeds identity read limit".into());
        }
        digest.update(&buffer[..n]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
pub fn load_into(
    model: &mut Model,
    path: &Path,
    trusted_digest: &str,
    orthography: &OrthographyId,
    policy: MatchPolicy,
) -> Result<(), Box<dyn Error>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(MAX_COMPILED_INDEX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_COMPILED_INDEX_BYTES {
        return Err("compiled index exceeds byte limit".into());
    }
    model.install_compiled_analysis_index(
        &bytes,
        trusted_digest,
        &executable_sha256()?,
        orthography,
        policy,
    )?;
    Ok(())
}
#[derive(Serialize)]
pub struct Receipt {
    pub path: PathBuf,
    pub sha256: String,
    pub executable_sha256: String,
    pub model_sha256: String,
    pub profile: String,
    pub policy: String,
    pub bytes: usize,
    pub forms: usize,
    pub trust: &'static str,
}
pub fn compile(
    model_path: &Path,
    orthography: OrthographyId,
    policy: MatchPolicy,
    output: &Path,
    sources: &[PathBuf],
) -> Result<Receipt, Box<dyn Error>> {
    if let Ok(destination) = output.canonicalize() {
        for input in std::iter::once(model_path).chain(sources.iter().map(PathBuf::as_path)) {
            if input.canonicalize()? == destination {
                return Err("compiled index output cannot replace an input source".into());
            }
        }
    }
    let model = model_artifact::load_with_sources(model_path, sources)?;
    let runtime = executable_sha256()?;
    let bytes = model.compile_analysis_index(&orthography, policy, &runtime)?;
    let sha256 = format!("{:x}", Sha256::digest(&bytes));
    let stage = StagedReplacement::prepare(output, &bytes, |staged| {
        let mut restored = model_artifact::load_with_sources(model_path, sources)?;
        restored.install_compiled_analysis_index(
            staged,
            &sha256,
            &runtime,
            &orthography,
            policy,
        )?;
        Ok(())
    })?;
    stage.commit()?;
    Ok(Receipt {
        path: output.into(),
        sha256,
        executable_sha256: runtime,
        model_sha256: model.data_sha256().into(),
        profile: orthography.0,
        policy: crate::analysis_document::policy_name(policy).into(),
        bytes: bytes.len(),
        forms: model.analysis_index_stats()?.forms,
        trust: "Digest of this locally compiled output; retain through a trusted channel. Neither an adjacent manifest nor this receipt authenticates downloaded data or historical grammar.",
    })
}
