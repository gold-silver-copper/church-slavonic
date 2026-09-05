//! `cargo xtask eval`: the three numbers, each of which can go down —
//! held-out recall (UD PROIEL dev+test, Syntacticus), Bible coverage
//! through the analyzer, and guesser accuracy (leave-one-out). Part 2
//! fills them in; Part 0 prints `n/a` and, with `--legacy`, the 1.2
//! baselines by running the legacy harness.

use std::error::Error;
use std::process::Command;

pub fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    if args.iter().any(|a| a == "--legacy") {
        return legacy_baselines();
    }
    println!("held-out recall (UD PROIEL dev+test): n/a (Part 2)");
    println!("Bible coverage (analyzer):            n/a (Part 2)");
    println!("guesser accuracy (leave-one-out):     n/a (Part 1)");
    Ok(())
}

/// The 1.2 numbers, from the legacy instruments themselves.
fn legacy_baselines() -> Result<(), Box<dyn Error>> {
    let root = crate::workspace_root();
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    for command in ["accuracy", "check-treebank"] {
        println!("== legacy {command}");
        let status = Command::new(&cargo)
            .current_dir(&root)
            .args(["run", "-p", "xtask-legacy", "--release", "--", command])
            .status()?;
        if !status.success() {
            return Err(format!("legacy {command} failed").into());
        }
    }
    Ok(())
}
