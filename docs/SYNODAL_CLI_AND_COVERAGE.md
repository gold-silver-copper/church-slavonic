# Synodal dictionary CLI and corpus coverage

`synodal-dict` is the offline command-line interface for the reviewed Synodal
Russian Church Slavonic registry. Install it from the workspace with

```sh
cargo install --path crates/synodal-church-slavonic-dictionary
```

The executable and all runtime crates use compiled registries. They perform no
network or filesystem data acquisition; source reconstruction remains an
`xtask` concern.

## Commands

```sh
synodal-dict search "king" --pos noun --limit 10
synodal-dict search "lord" --exact --json
synodal-dict show synodal:verb:byti
synodal-dict families 'ꙗкѡ' --reviewed-only --json
synodal-dict families 'ꙗкоже' --proposals reports/synodal-family-review-queue.json --json
synodal-dict show-family family:synodal:determiner:ves --json
synodal-dict show-family synodal:family-candidate:ID --proposals reports/synodal-family-review-queue.json --json
synodal-dict analyze 'бꙋ́детъ' --policy strict --profile printed
synodal-dict lint vocabulary.json --json
synodal-dict check-text passage.txt --strict
synodal-dict coverage passages.tsv \
  --by-family \
  --json-out coverage.json \
  --markdown-out coverage.md \
  --tsv-out review-queue.tsv
```

`search` checks normalized lemmas and reviewed English glosses. Fuzzy search is
the default; `--exact` disables it. `show` accepts a lemma or stable ID and
returns every lemma match rather than silently selecting an ambiguous entry.
`analyze` returns every best mark-sensitive lexical/cell analysis under the
selected `strict`, `productive`, or `exploratory` policy. Explicit accents and
breathings must match reviewed marks and never silently fall back to an
accentless result; genuinely unmarked input can remain ambiguous.

`families` returns every matching reviewed lexical family, preserving homographs.
`--reviewed-only` prevents proposal-file lookup. With `--proposals PATH`, the
output also includes matching diagnostic proposals from an explicitly supplied
offline report. `show-family` accepts a reviewed `family:LEXEME_ID`; a candidate
ID is displayable only when its proposal report is supplied. Candidate reports
are never loaded by the library and never become reviewed runtime identities.

`lint` accepts this JSON shape:

```json
{
  "entries": [
    {
      "text": "бꙋ́детъ",
      "expected_lexeme_id": "synodal:verb:byti",
      "expected_part_of_speech": "Verb",
      "required_sense_id": "sense:be",
      "requested_cell": {
        "FiniteVerb": {
          "tense": "Future",
          "person": "Third",
          "number": "Singular"
        }
      }
    }
  ]
}
```

`check-text` accepts a UTF-8 file or `-` for stdin. It preserves original byte
spans, line and column positions, capitalization, combining marks, titla, and
punctuation. `--max-unknown` limits only `unknown-lexeme` tokens,
`--max-ambiguous` limits ambiguous tokens, and `--strict` rejects every
unresolved or ambiguous token. These switches make the command suitable for CI
gates. `--summary` suppresses per-token diagnostics.

`coverage` accepts JSONL records matching `CoveragePassage`, or TSV with this
exact header:

```text
corpus\tsource_id\twork\tedition\tpassage\tpartition\tsource_recension\ttext
```

Newlines inside the final TSV field use the literal escape `\n`. The consumer
`coverage` command emits deterministic JSON or Markdown and can write those
formats plus its bounded frequency-ranked review queue to explicitly requested
paths. The canonical `cargo xtask synodal-coverage` workflow additionally
writes a complete `*-frontier.tsv`. That frontier has one row per
strict-top-k-uncovered surface/status combination, is never truncated, retains
true document frequency and bounded contexts, and is the authoritative
inventory for work toward 100% coverage. JSON and Markdown also report every
partition and every source/partition slice so aggregate gains cannot hide
held-out regressions.

## Gap model and precedence

Every unresolved lexical token has one primary category. When more than one
reason applies, the stable precedence is:

1. `AmbiguityOrSpellingVariant`;
2. `MissingAccentOrOrthographicMetadata`;
3. `MissingVerbPrincipalPart`;
4. `MissingDeclensionOrClass`;
5. `UnsupportedFormation`;
6. `UnknownLexeme`.

Secondary reasons are retained. Gap records also retain original and normalized
text, source/work/edition/passage/partition, byte position, candidate lexeme
IDs, inferred system, policy/profile, resolver detail, missing fields,
frequency, true document frequency, bounded source contexts, and the next review
action. Aggregated rows list all contributing corpora, sources, editions,
partitions, and recensions, and
the report includes per-corpus, per-source, per-partition, and
per-source/partition gap matrices. Non-lexical punctuation is excluded by the
tokenizer. A canonical Cyrillic numeral receives a typed numeric analysis with
its parsed value and canonical spelling; it counts only after that parse
succeeds and never fabricates a dictionary lexeme.

With `--by-family`, coverage also reports reviewed family slices, unresolved
probable-family diagnostics, and recovery-route estimates for exact evidence,
reviewed classes, reviewed principal parts, abbreviations, spelling variants,
unsupported formations, and ungrouped unknowns. These are diagnostics, not
automatic identity decisions. Conservative stem/endings and mark patterns may
group queue candidates, but every assumption and contradiction remains visible.

## Reproducing committed reports

```sh
cargo xtask synodal-coverage --fixture --offline
cargo xtask synodal-coverage --offline
cargo xtask synodal-lexical-review-queue
cargo xtask synodal-evaluation-queue
cargo xtask synodal-family-review-queue
cargo xtask synodal-marginal-recovery
cargo xtask synodal-v06-review-packets
cargo xtask synodal-v07-review-packets
cargo xtask synodal-v07-apply --check
cargo xtask synodal-v04-audit --check
cargo xtask synodal-v05-audit
cargo xtask synodal-v06-audit --check
cargo xtask synodal-v07-audit --check
```

Each coverage command writes `{stem}.json`, `{stem}.md`,
`{stem}-review-queue.tsv`, and `{stem}-frontier.tsv`; `--check` validates all
four byte-for-byte. The first command uses the committed ten-passage fixture. The full command uses
the locked Ponomar Elizabeth Bible and exact-revision Wikisource adaptations in
`data/intermediate/synodal/`; it does not treat OCS or modern Russian text as
target attestation. The lexical queue cross-matches source-partition target
frequency with independently sourced OCS semantics and exact-headword semantics
from the locked SCI Ponomar dictionary. Ponomar proposals remain
`unknown`/`untyped` until human part-of-speech and cell review. The queue
excludes admitted target lexemes, preserves ambiguous rejections, never treats
dictionary rows as target surface evidence, and never edits reviewed TSVs. The
evaluation queue uses only held-out passages, blocks generated surface forms
that correspond to more than one cell, and likewise remains candidate-only.

The family queue considers accentless positional spelling, conservative
prefix/ending boundaries, dictionary form membership, known runtime candidates,
abbreviation marks, and bounded corpus contexts. Candidate IDs hash the grouping
contract rather than volatile frequency or surface order. `--check` requires
decisions for all current top 200 rows and verifies the committed JSON/TSV bytes.
The marginal-recovery command writes deterministic JSON, Markdown, and TSV
diagnostics. It greedily removes overlap between unresolved candidate batches,
reports the additional tokens required by the strict milestone, and never
counts a proposal as analyzed. The v0.4 audit verifies the immutable committed
baseline and its audit digest; it cannot be relabeled by later registry growth.
The v0.5 audit is generated from the current registries, locked baseline,
coverage/evaluation reports, marginal report, and durable review decisions.
The v0.6 audit freezes the historical 65% result. The v0.7 packet command
builds an exact-surface acquisition queue with separate identity, semantic,
target-occurrence, source-morphology, and held-out roles. The apply gate
materializes only admitted rows and rejects missing evidence or partition
overlap; the v0.7 audit freezes the canonical 70% result and verification
ledger.

`cargo xtask synodal-coverage --offline --check --require-complete` is the
non-gameable final gate. It accepts only the default full sources, locked
intermediate hashes, strict policy, and liturgical profile; rejects fixture,
custom-source, alternate-intermediate, and truncated runs; locks the passage,
token, and type denominator; and requires zero uncovered tokens in every
corpus, source, partition, source/partition, and policy slice. It is expected to
fail until the 100% program is genuinely finished.

Reviewed admissions are made explicitly in `data/synodal/lexical_reviews.tsv`.
They require a stable target identity, a reviewed semantic decision, a locked
target-recension candidate, and a review note. Productive morphology requires
the stronger class/principal-part contract in the base registries. A lexical
attestation never implies a productive class.

Morphological-family decisions are made explicitly in
`data/synodal/family_reviews.tsv`. Admissions require a linked target lexeme,
class/table scope, target and semantic evidence, confidence, and review note.
Deferrals and rejections are durable; a proposal generator never writes this
file. Typed contractions live in `data/synodal/abbreviations.tsv`, while their
passage-disjoint expansion/reverse-lookup cases live in
`data/synodal/abbreviation_evaluation.tsv`.

Raw source bytes remain outside packages and are subject to the source-specific
licenses and redistribution flags in `references/SOURCES.toml`. See
`SYNODAL_DATA_PIPELINE.md` for acquisition and full reconstruction.
