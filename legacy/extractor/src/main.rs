fn main() {
    if let Err(e) = extractor_legacy::run_from_env() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
