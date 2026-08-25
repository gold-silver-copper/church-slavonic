//! Embeds a fingerprint of the generated registry so runtime consumers (the
//! xtask staleness tripwire) can detect a binary compiled against an older
//! registry than the one on disk. FNV-1a over the raw bytes plus the byte
//! length is ample for staleness detection and needs no build dependencies.

use std::{env, fs, path::Path};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR");
    let registry = Path::new(&manifest).join("generated/registry.rs");
    println!("cargo:rerun-if-changed={}", registry.display());
    let bytes = fs::read(&registry).expect("generated registry is part of the crate");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in &bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    println!(
        "cargo:rustc-env=SYNODAL_REGISTRY_FINGERPRINT={hash:016x}-{}",
        bytes.len()
    );
}
