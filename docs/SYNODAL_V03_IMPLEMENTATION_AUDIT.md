# Synodal v0.3 corpus-driven coverage audit

This audit records the implementation of
`SYNODAL_V03_CORPUS_DRIVEN_COVERAGE_PROMPT.md`. Counts are raw deterministic
counts, not an estimate of language-wide accuracy.

## Delivered surface

- `synodal-dict` provides `search`, `show`, `analyze`, `lint`, `check-text`, and
  `coverage`, with human and JSON output, typed exit failures, policy/profile
  selection, stable IDs, and source-visible analyses.
- The reusable indexed analyzer preserves byte spans and Church Slavonic marks.
  Explicit accent/breathing input must match reviewed marks; only genuinely
  unmarked input uses the accentless index, where homographs remain ambiguous.
- Every unresolved lexical token receives one of six serializable gap kinds.
  Pairwise tests enforce the documented precedence, and no lexical token falls
  into a generic bucket.
- `cargo xtask synodal-coverage --offline` reproduces the committed real-corpus
  reports from locked normalized intermediates. Candidate-only lexical and
  evaluation queues are independently reproducible.
- Runtime libraries remain compiled-data-only and WASM-compatible. Raw sources,
  normalized corpora, evaluation passages, and review tools do not enter the
  packages.

## Registry growth and review accounting

| Measure | v0.2 baseline | v0.3 |
|---|---:|---:|
| Runtime lexemes | 61 | 505 |
| Reviewed senses | 61 | 505 |
| Generated exact rows | 260 | 754 |
| Base normative/attested exact rows | 260 | 311 |
| Lexical review rows | 0 | 515 |
| Reviewed lexical-overlay rows | 0 | 443 |
| Preserved rejected rows | 0 | 72 |

The 443 admitted overlay rows came from a frequency-ranked cross-source review:
target surface evidence is a Ponomar Synodal source-partition passage, while OCS
Wiktionary contributes only an independently reviewed semantic candidate. An
inflectable item without independently reviewed class or principal parts is an
exact `LexicalForm`; it does not acquire productive morphology. The regenerated
lexical queue excludes admitted entries and currently preserves 155 unreviewed
candidates plus 71 blocked ambiguous homographs.

Additional high-frequency reviews in this milestone include `ѿ`, `но`, `єгда`,
`предъ`, and the positional `во`/`ко` variants. Alypy §47 supplies the complete
gendered third-person pronoun paradigm and its case-distinguishing printed
accents. The previously rejected surface-only Wiktionary candidate for `и`
remains rejected; the admitted `онъ` paradigm instead rests on the independent
target normative table.

## Corpus coverage

The committed full report covers the locked Ponomar Elizabeth Bible and the
exact-revision Church Slavonic Wikisource adaptation only. Both are labeled
`synodal-russian`; OCS, modern Russian, and mixed dictionary text are excluded.

| Measure | Count | Share of 1,313,344 tokens |
|---|---:|---:|
| Passages | 74,130 | — |
| Token types | 57,476 | — |
| Top-1 analyzed | 404,452 | 30.80% |
| Top-k analyzed | 530,005 | 40.36% |
| Exact target attestation status | 358,593 | 27.30% |
| Ambiguous | 13,510 | 1.03% |
| Unresolved/abstained | 782,134 | 59.55% |

Primary gap frequencies are:

| Gap | Tokens |
|---|---:|
| `UnknownLexeme` | 733,626 |
| `MissingAccentOrOrthographicMetadata` | 47,416 |
| `AmbiguityOrSpellingVariant` | 14,518 |
| `MissingDeclensionOrClass` | 1,092 |
| `MissingVerbPrincipalPart` | 0 |
| `UnsupportedFormation` | 0 |

Zero in the last two rows does not mean those library gaps are absent. The
token-only classifier cannot infer a requested verb cell for most unknown
surface forms; direct API and manifest requests still return typed missing-part
or unsupported-formation failures. The most frequent remaining review items are
the legitimate adverb/conjunction ambiguity `ꙗ҆́кѡ` (13,077), `Рече́` (7,677),
the `господь` abbreviation family (7,565 for nominative alone), `всѧ̀` (4,077),
and missing accent coverage for forms of `землѧ` (3,810 for `землѝ`).
The JSON report also contains per-corpus and per-source gap matrices; each
frequency aggregate lists every contributing corpus, source, edition,
partition, and source recension while preserving one position-bearing sample.

## Morphology and rule decisions

No new productive formation rule was admitted merely to improve a metric. The
report was dominated by lexical identity and exact orthography, so the
implementation expanded reviewed data and continued to abstain on unsupported
classes.

One new finite-tense category, `Future`, was added because the corpus repeats
`бꙋ́детъ` 4,426 times and Alypy §81 provides the complete nine-cell simple
future of `быти`. It is an exact normative table. Any other verb requested in a
simple-future cell fails explicitly; no general future rule is inferred. The
complete §47 third-person pronoun is likewise exact-only. These additions are
sourced predictions, not corpus attestations.

Deferred productive proposals include mixed/consonantal/heteroclitic noun
classes, automatic principal-part derivation, generalized accent classes,
general nomina-sacra contraction, automatic comparison stems, participles from
an undifferentiated verb stem, the supine, and verbal nouns. Their metadata and
exception contracts remain insufficient for safe implementation.

## Evaluation

The passage-disjoint table grew from 38 to 437 cells. It includes 384 held-out
exact lexical attestations and 53 predicted morphology cases, plus five analytic
phrase cases. Source-passage guards include lexical review passages as forbidden
generation evidence.

| Slice | Returned | Top-1 | Top-k |
|---|---:|---:|---:|
| Expanded | 437/437 | 436/437 | 437/437 |
| Printed/liturgical | 437/437 | 436/437 | 437/437 |
| Strict | 436/437 | 435/437 | 436/437 |
| Productive | 437/437 | 436/437 | 437/437 |
| Exploratory | 437/437 | 436/437 | 437/437 |
| Analytic phrases, expanded | 5/5 | 5/5 | 5/5 |
| Analytic phrases, printed | 5/5 | 5/5 | 5/5 |

The sole general top-1 disagreement is the reviewed `трїе`/corpus `три` variant
ordering. Systems represented include nouns, adjectives, determiners, numerals,
pronouns, present, simple future, imperfect, aorist, imperative, infinitive,
l-participle, active participle, exact indeclinables, and exact lexical forms.
The phrase suite covers analytic future, perfect, pluperfect, conditional, and
passive. The separate candidate queue now has 31 uniquely matchable cells and
383 cells blocked because a surface alone cannot select one grammatical cell.

Limitations remain material: exact lexical forms dominate the 437 rows; several
productive class families have only one or a few real-passage cells; most
passive-participle and analytic cases are normative fixtures rather than a
large independent corpus sample; and the single inherited held-out cell cannot
calibrate inheritance confidence. The evaluation is a regression suite over
registered behavior, not a claim about arbitrary Synodal text.

## Verification

All commands below passed on 2026-08-10. The three publish dry-runs packaged and
reached the simulated upload stage without uploading. The full bootstrap used
the local 4.6 GB cache, verified all 321 artifacts, regenerated the same
505/754/505 registries, and ended with `synodal checks: current`.

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-fixture-bootstrap
cargo xtask synodal-check
cargo xtask synodal-coverage --fixture --offline --check
cargo xtask synodal-coverage --offline --check
cargo xtask synodal-lexical-review-queue --check
cargo xtask synodal-evaluation-queue --check
cargo check -p synodal-church-slavonic-core --no-default-features
cargo check -p synodal-church-slavonic --no-default-features
cargo check -p synodal-church-slavonic-dictionary --no-default-features
cargo check --target wasm32-unknown-unknown -p synodal-church-slavonic-core
cargo check --target wasm32-unknown-unknown -p synodal-church-slavonic
cargo check --target wasm32-unknown-unknown -p synodal-church-slavonic-dictionary --no-default-features
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic-core
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic-dictionary
```

The final full-source command was:

```text
cargo xtask synodal-bootstrap --offline --cache references/downloads
```

It passed. A network-backed reconstruction into a newly downloaded disposable
cache was not repeated because every locked byte was already checksum-verified;
the fixture bootstrap separately exercised atomic fetching into two empty
disposable caches.
