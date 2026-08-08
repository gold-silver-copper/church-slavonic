#![forbid(unsafe_code)]

fn main() {
    // Keep attribution in the published artifact; packaging without it is a build error.
    let _attribution = include_str!("ATTRIBUTION.md");
    println!("cargo:rerun-if-changed=generated/registry.rs");
    println!("cargo:rerun-if-changed=ATTRIBUTION.md");
}
