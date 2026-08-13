use std::collections::BTreeSet;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Prepare every artifact before replacing any target, and roll the batch
/// back if installation fails partway through.
pub(crate) fn atomic_write_batch(artifacts: &[(PathBuf, &[u8])]) -> Result<(), Box<dyn Error>> {
    let process = std::process::id();
    let unique_targets = artifacts
        .iter()
        .map(|(target, _)| target)
        .collect::<BTreeSet<_>>();
    if unique_targets.len() != artifacts.len() {
        return Err("atomic batch contains duplicate target paths".into());
    }
    for target in &unique_targets {
        if target.exists() && !fs::metadata(target)?.is_file() {
            return Err(format!("atomic batch target is not a file: {}", target.display()).into());
        }
    }
    let prepared = artifacts
        .iter()
        .map(|(target, _)| artifact_paths(target, process))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

    if let Err(error) = prepare_artifacts(artifacts, &prepared) {
        remove_temporaries(&prepared);
        return Err(error);
    }

    if let Err(error) = install_artifacts(&prepared) {
        restore_artifacts(&prepared);
        return Err(error);
    }
    for (_, _, backup) in &prepared {
        if backup.exists() {
            fs::remove_file(backup)?;
        }
    }
    Ok(())
}

fn artifact_paths(
    target: &Path,
    process: u32,
) -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn Error>> {
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("artifact has no UTF-8 filename: {}", target.display()))?;
    Ok((
        target.to_path_buf(),
        target.with_file_name(format!("{name}.refresh-{process}.tmp")),
        target.with_file_name(format!("{name}.refresh-{process}.bak")),
    ))
}

fn prepare_artifacts(
    artifacts: &[(PathBuf, &[u8])],
    prepared: &[(PathBuf, PathBuf, PathBuf)],
) -> Result<(), Box<dyn Error>> {
    for ((_, bytes), (_, temp, backup)) in artifacts.iter().zip(prepared) {
        if temp.exists() {
            fs::remove_file(temp)?;
        }
        if backup.exists() {
            return Err(format!(
                "refusing to overwrite an existing refresh backup: {}",
                backup.display()
            )
            .into());
        }
        let mut file = File::create(temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    Ok(())
}

fn install_artifacts(prepared: &[(PathBuf, PathBuf, PathBuf)]) -> Result<(), Box<dyn Error>> {
    for (target, _, backup) in prepared {
        if target.exists() {
            fs::rename(target, backup)?;
        }
    }
    for (target, temp, _) in prepared {
        fs::rename(temp, target)?;
    }
    Ok(())
}

fn restore_artifacts(prepared: &[(PathBuf, PathBuf, PathBuf)]) {
    for (target, temp, backup) in prepared {
        if backup.exists() {
            if target.exists() {
                let _ = fs::remove_file(target);
            }
            let _ = fs::rename(backup, target);
        } else if !temp.exists() && target.exists() {
            // This target did not exist before the batch and its temporary
            // file was installed. Remove only the newly created artifact.
            let _ = fs::remove_file(target);
        }
        if temp.exists() {
            let _ = fs::remove_file(temp);
        }
    }
}

fn remove_temporaries(prepared: &[(PathBuf, PathBuf, PathBuf)]) {
    for (_, temp, _) in prepared {
        if temp.exists() {
            let _ = fs::remove_file(temp);
        }
    }
}
