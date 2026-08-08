#![forbid(unsafe_code)]

fn main() {
    if let Err(error) = old_church_slavonic_extractor::extract::run_cli() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
