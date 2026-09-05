//! CLI contract for the extractor binary (normally invoked via
//! `cargo xtask refresh-data`, which forwards these flags verbatim).
//!
//! The generated tables always go to the committed layout
//! (`crates/church-slavonic/generated`) — the only location `check-registry`,
//! the runtime, and the accuracy harness read. The pinned sources default to
//! `references/downloads` (`--sources`); the filtered caches default to
//! `data/intermediate` (`--artifacts-dir`).

use std::env;
use std::error::Error;
use std::path::PathBuf;

const DEFAULT_GENERATED_DIR: &str = "legacy/church-slavonic/generated";
const DEFAULT_ARTIFACTS_DIR: &str = "data/intermediate";
const DEFAULT_SOURCES_DIR: &str = "references/downloads";

#[derive(Debug, Clone)]
pub struct Config {
    /// The directory holding the pinned source downloads.
    pub sources_dir: PathBuf,
    /// Where the PHF tables are written — always the committed layout.
    pub generated_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    /// Measure accuracy against the currently-compiled committed tables, then
    /// stop without regenerating (`cargo xtask accuracy`).
    pub checks_only: bool,
}

pub fn parse_args() -> Result<Config, Box<dyn Error>> {
    parse_args_from(env::args().skip(1))
}

pub fn parse_args_from<I>(args: I) -> Result<Config, Box<dyn Error>>
where
    I: IntoIterator<Item = String>,
{
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let mut sources_dir = repo_root.join(DEFAULT_SOURCES_DIR);
    let generated_dir = repo_root.join(DEFAULT_GENERATED_DIR);
    let mut artifacts_dir = repo_root.join(DEFAULT_ARTIFACTS_DIR);
    let mut checks_only = false;

    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--sources" => {
                sources_dir =
                    PathBuf::from(args.next().ok_or("expected a path after `--sources`")?);
            }
            "--artifacts-dir" => {
                artifacts_dir = PathBuf::from(
                    args.next()
                        .ok_or("expected a path after `--artifacts-dir`")?,
                );
            }
            "--checks-only" => checks_only = true,
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}").into()),
        }
    }

    Ok(Config {
        sources_dir,
        generated_dir,
        artifacts_dir,
        checks_only,
    })
}

pub fn print_usage() {
    eprintln!(
        "Usage: cargo run -p extractor-legacy --release -- [--sources DIR] [--artifacts-dir DIR] [--checks-only]"
    );
}
