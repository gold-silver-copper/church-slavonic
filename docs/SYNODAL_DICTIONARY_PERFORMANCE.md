# Synodal dictionary test-performance report

This report records the optimization of the Synodal dictionary reverse analyzer
and its test architecture. Measurements were taken on 2026-08-13 on the same
Apple Silicon development machine, with no other CPU-consuming Cargo or Rust
compiler work active. Suite figures are wall-clock seconds from three warm
runs; compilation is reported separately.

## Results

| Measurement | Before runs | Before median | After runs | After median | Speedup |
| --- | --- | ---: | --- | ---: | ---: |
| Dictionary library tests | 322.78, 360.99, 351.42 | 351.42 s | 23.59, 23.80, 23.52 | 23.59 s | 14.90x |
| Dictionary CLI tests | 246.43, 250.41, 262.42 | 250.41 s | 0.85, 0.87, 0.87 | 0.87 s | 287.83x |
| Combined suite medians | - | 601.83 s | - | 24.46 s | 24.60x |
| Debug analyzer construction | 41.735, 41.713, 41.724 | 41.724 s | 1.847, 1.856, 1.857 | 1.856 s | 22.48x |
| Reused `Analyzer::analyze` lookup | 27.788, 27.930, 27.682 us | 27.788 us | 26.891, 26.983, 26.956 us | 26.956 us | 1.03x |
| Cached default dictionary lookup | not cached | - | 14.683, 14.633, 14.655 us | 14.655 us | - |
| Standalone debug CLI analysis | 45.426, 47.115, 46.138 | 46.138 s | 1.87, 1.86, 1.86 | 1.86 s | 24.81x |

The optimized analyzer admits 9,461 per-lexeme typed cells, or 18,922 form
resolution attempts across expanded and printed profiles. The prior universal
inventory performed approximately 550,994 attempts. The new inventory removes
about 96.6% of those probes (a 29.1x reduction) while retaining exact registered
cells and every supported productive system.

A process-local analyzer-cache retrieval had a 0.958 microsecond median. The
unchanged, two-profile public `Analyzer::analyze` path completed a 10,000-word
batch in about 270 milliseconds, while the warmed default dictionary path
completed the same batch in about 147 milliseconds. The latter reuses the
process-wide analyzer and preserves the dictionary's global mark-quality and
abbreviation-ranking semantics. A standalone debug `synodal-dict analyze`
command now takes a 1.86-second median, down from the 46.14-second fresh-main
baseline measured before implementation.

Peak process memory remained effectively flat: the median macOS peak-footprint
measurement moved from 41.14 MB to 41.26 MB (0.28%, within run-to-run noise).

## Test-profile tradeoff

The final workspace uses:

```toml
[profile.test]
opt-level = 1
```

Controlled clean target directories produced these results on the final source:

| Test optimization | Cold library test compilation | Warm library run |
| --- | ---: | ---: |
| 0 | 7.66 s | 129.62 s |
| 1 | 10.77 s | 24.08 s |
| 2 | 13.40 s | 25.06 s |

Level 1 made this CPU-bound suite 5.38x faster than level 0 while adding 3.11
seconds (40.6%) to the isolated cold compile. Level 2 compiled 2.63 seconds
slower than level 1 and did not improve this run, so level 1 is the lowest and
best measured tradeoff. Test assertions, overflow checks, and debug information
remain enabled.

## Work eliminated

- The reverse analyzer now derives a deterministic per-lexeme cell inventory
  from exact registry keys, part of speech, subtype, number restrictions,
  available principal parts, productive class, and capability metadata.
- Default library analysis and vocabulary linting share a fallible,
  thread-safe analyzer cache. Cache keys include generation policy,
  orthography profile, and productive-mapping threshold, so incompatible
  configurations cannot alias.
- CLI dispatch now accepts injected streams and a process-local analyzer cache.
  Seven tests run in-process against shared state; one subprocess test retains
  coverage of executable wiring, stdin/stdout/stderr, and exit status.
- Analyzer construction creates the two reusable inflectors outside the inner
  cell loop.

Before the change, the library suite performed approximately 26 exhaustive or
equivalent registry scans. Compatible default analysis now constructs one
cached analyzer. The CLI suite moved from roughly nine fresh-process scans to
two shared configurations plus one deliberately retained subprocess build.
The library still performs three exhaustive reference constructions on purpose:
they independently prove that optimized and exhaustive indexes contain equal
full `Analysis` values under strict, productive, and exploratory policies.

## Correctness evidence

The retained exhaustive oracle compares all four marked/accentless,
expanded/printed index maps and spelling candidates. It compares complete
analysis values, including stable lexeme identity, typed grammar cells,
evidence, recension provenance, source classification, confidence, mapping,
assumptions, contradictions, warnings, and rule trace. Dedicated tests also
cover concurrent cache initialization, one construction per compatible
configuration, mark-sensitive matching, accentless fallback, abbreviations,
generation-policy boundaries, CLI golden output, and subprocess contracts.

## Generated strict reverse index decision

No generated reverse-index artifact was added. The extractor-generated exact
registry remains authoritative, while the compact runtime inventory now builds
in 1.86 seconds in an unoptimized debug binary and ordinary queries avoid the
old theoretical cross product. Adding another generated artifact would increase
generator, staleness-check, and provenance-validation surface without being
needed to meet the local/CI targets. If startup latency later becomes a product
requirement, a deterministic generated exact-surface index remains the next
concrete optimization; productive forms would still be generated at runtime.

The remaining deliberate hot path is the exhaustive equivalence oracle. It is
kept at one correctness boundary instead of being repeated throughout the
suite, and currently accounts for nearly all of the 24-second library runtime.

## Reproducing focused measurements

The lightweight harness reports analyzer construction and cache retrieval,
then measures the reused public `Analyzer::analyze` path and warmed default
dictionary lookup separately. It also reports indexed cells and construction
count:

```bash
cargo run -p synodal-church-slavonic-dictionary \
  --example analyzer_benchmark --all-features
```

Run performance commands sequentially. Parallel Cargo processes contend for
CPU and previously inflated the library and CLI components to 8m19.62s and
6m57.31s respectively.
