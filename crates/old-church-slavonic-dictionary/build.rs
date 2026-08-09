#![forbid(unsafe_code)]

fn main() {
    let _attribution = include_str!("ATTRIBUTION.md");
    let _source = include_str!("SOURCE.toml");
    println!("cargo:rerun-if-changed=generated/dictionary.rs");
    println!("cargo:rerun-if-changed=ATTRIBUTION.md");
    println!("cargo:rerun-if-changed=SOURCE.toml");
}
