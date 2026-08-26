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
synodal-dict show-family family:synodal:determiner:ves --json
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

`families` returns every matching reviewed lexical family, preserving
homographs. `show-family` accepts a reviewed `family:LEXEME_ID`. With
`--proposals PATH`, both can additionally read an explicitly supplied offline
proposal report (the wave-era queue generators are retired, so no committed
default exists). Candidate reports are never loaded by the library and never
become reviewed runtime identities.

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
paths. JSON and Markdown also report every partition and every
source/partition slice so aggregate gains cannot hide regressions. Repo-level
coverage gating is the synodal-gold full-enumeration gate
(`cargo xtask synodal-gold --check`, `docs/SYNODAL_GOLD_ORACLE.md`); this
consumer command measures arbitrary user-supplied passages.

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

## Retired wave-era reproduction workflow

The wave-era report generators (`synodal-coverage`, the lexical/evaluation/
family review queues, `synodal-marginal-recovery`, `synodal-accent-fit`,
`synodal-type-holdout`, and the v0.4-v0.7 audit re-derivations) were retired
on 2026-08-26 with the wave program itself; the synodal-gold gate
(`cargo xtask synodal-gold --check`) is the single full-enumeration coverage
measure, and `reports/synodal-gold-gap.tsv` is the worklist. The wave-era
methodology is preserved in `docs/history/` and the immutable archive
(`cargo xtask synodal-archive --check`).

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
