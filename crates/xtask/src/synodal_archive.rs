//! Verifies the immutable historical audit artifacts by checksum (v0.12
//! phase 5).
//!
//! The v0.4–v0.7 audits are immutable historical checkpoints: their
//! re-derivation can never legitimately produce anything new, so CI verifies
//! one manifest of sha256 digests instead of re-running seven audit commands
//! on every push. The audit commands themselves remain available for
//! on-demand re-derivation (`synodal-v04-audit` … `synodal-v07-audit`); a
//! manifest mismatch means an immutable artifact was edited, and the remedy
//! is to restore it, never to reseal the manifest casually.

use std::{error::Error, fs, path::Path};

use sha2::{Digest, Sha256};

use crate::report_io::write_if_changed_atomic;

const MANIFEST_PATH: &str = "reports/synodal-archive-manifest.tsv";
const HEADER: &str = "artifact\tsha256";

/// The frozen artifact set. Everything here is an immutable historical
/// checkpoint whose verifying command still exists for on-demand runs.
const ARTIFACTS: [&str; 10] = [
    "reports/synodal-v04-baseline.json",
    "reports/synodal-v04-marginal-recovery.json",
    "reports/synodal-v05-baseline.json",
    "reports/synodal-v06-baseline.json",
    "reports/synodal-v06-review-packets.json",
    "reports/synodal-v06-review-packets.md",
    "reports/synodal-v06-review-packets.tsv",
    "reports/synodal-v07-review-packets.json",
    "reports/synodal-v07-review-packets.md",
    "reports/synodal-v07-review-packets.tsv",
];

fn digest(path: &Path) -> Result<String, Box<dyn Error>> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn render(root: &Path) -> Result<String, Box<dyn Error>> {
    let mut output = String::from(HEADER);
    output.push('\n');
    for artifact in ARTIFACTS {
        output.push_str(&format!("{artifact}\t{}\n", digest(&root.join(artifact))?));
    }
    Ok(output)
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args.by_ref() {
        match argument.as_str() {
            "--check" => check = true,
            value => return Err(format!("unknown synodal-archive argument {value:?}").into()),
        }
    }
    let manifest_path = root.join(MANIFEST_PATH);
    let rendered = render(root)?;
    if check {
        let committed = fs::read_to_string(&manifest_path)
            .map_err(|error| format!("cannot read {}: {error}", manifest_path.display()))?;
        if committed != rendered {
            return Err(format!(
                "an immutable historical audit artifact disagrees with {}; restore the artifact (or, for a deliberate archival change, rerun cargo xtask synodal-archive and justify it in the commit)",
                manifest_path.display()
            )
            .into());
        }
        println!(
            "synodal archive: {} immutable artifacts verified",
            ARTIFACTS.len()
        );
        return Ok(());
    }
    write_if_changed_atomic(&manifest_path, &rendered)?;
    println!("synodal archive: {} artifacts sealed", ARTIFACTS.len());
    Ok(())
}
