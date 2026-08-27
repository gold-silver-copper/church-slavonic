# analyzer (experiment)

The Synodal Church Slavonic text analyzer (`synodal-dict analyze-text`, lookup,
coverage and the analyzer benchmark) that used to ship inside
`synodal-church-slavonic-dictionary`.

**Status: unmaintained until adopted.** It is excluded from the workspace and
is not built by CI. It targets the *published* pre-parity crates
(`synodal-church-slavonic-dictionary = "=0.6.0"` and friends; the exact pins matter because the later `0.6.1` releases are empty deprecation stubs) by crates.io
version, not the current `church-slavonic` / `church-slavonic-core` crates in
this repository, and it has not been ported to them.

Build it on its own:

```sh
cd experiments/analyzer
cargo run --release -- analyze-text "во и҆́мѧ ѻ҆ц҃а̀"
cargo run --release --example analyzer_benchmark
```

If the analyzer is adopted, the port is: replace the 0.6 dependencies with the
`church-slavonic` facade and rebuild the reverse index by enumerating its
paradigms.
