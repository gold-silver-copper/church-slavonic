# Data pipeline

## Stages and ownership

| Stage | Reads | Owns |
|---|---|---|
| `refresh-data` | an explicit local Kaikki/Wiktextract JSONL path | `data/extracted/*.tsv`, `data/extracted/source.json`, generated Rust, extraction reports |
| `check-registry` | committed normalized TSV | nothing; compares deterministic generated Rust and validates semantics |
| `accuracy` | committed registry and public facade | `reports/accuracy.{json,md}` when refreshed |
| `accuracy-corpus` | explicit local paths to hash-pinned UD and/or Syntacticus data | aggregate `reports/corpus-accuracy.{json,md}` only with `--write`; optional token details stay local |
| runtime facade | generated Rust only | nothing |

The raw dump is gitignored. No normal build, test, example, or package operation uses
the network.

## Accepted extraction surface

Entries must have `lang_code == "cu"`. Noun and adjective cells require complete
case/number (and adjective gender) tags. Closed-class cells use the dimensions
actually present, including person on combined personal-pronoun tables. Pages whose
senses are all marked `form-of` are excluded as duplicate lexemes. The source's
repeated personal/reflexive table is split: personal rows remain on personal entries,
while `сѧ` retains only reflexive rows; its numberless source cells are exposed for all
three numbers because reflexive agreement is number-invariant. Latin-only,
mixed-script, empty, and table-sentinel forms are never public variants. Literal
MediaWiki rendering artifacts such as `{{{2}}}` are likewise rejected from
page, lemma, alias, comparative, and paradigm spellings and reported under a
`template-markup-*` drop reason. A comma-separated list is split into ordered
variants only when every component is independently a valid NFC OCS word and no
unsplittable romanization is attached. Negated forms, prepositional strings, and
other multiword/contextual output are excluded from this word-level API. Verb cells
are accepted only when their identity is safe:
infinitive, supine, verbal noun, unambiguous l-participle cells, citation participles,
or finite/imperative cells carrying complete person and number without
`error-unrecognized-form`.

Unsafe declined-participle blocks and malformed finite rows are counted by exact drop
reason. This is the prompt's permitted constrained-coverage strategy; it is not a
claim that the missing cells do not exist.

The sampled snapshot confirmed two flattening hazards:

- finite verb rows can have spurious `l-participle` tags, and many aorist rows have
  `error-unrecognized-form`; the former is ignored only if tense/person/number are
  complete, while the latter always fails closed;
- adjective short and long rows lack a per-form marker. Two `table-tags` sentinels
  delimit the complete short and long blocks. The mapping is fixture-tested; an
  unknown block is rejected.

## Normalized registry

- `lexemes.tsv`: deterministic content-derived lexeme IDs, raw page word, normalized
  and raw class markers, and serialized source head templates;
- `aliases.tsv`: explicit NFC/lowercase aliases plus their raw page/canonical/
  alternative spelling relationship;
- `forms.tsv`: one ordered variant per row with the exact raw source spelling,
  source tags, and romanization;
- `source.json`: input filename, size, SHA-256, and registry schema.

TSV fields are rejected if they contain a tab or newline. Generated Rust is a pure
function of these files plus `data/overrides.tsv`. An override row contains
`lemma`, `pos`, a complete feature key, source-ordered variants separated by ` || `
(optional romanization follows ` :: `), a reason, a source citation, and the literal
review status `approved`. Missing citations, unapproved rows, ambiguous lemmas, and
duplicate overridden cells fail validation. Refresh writes through temporary files
and renames only after successful extraction and validation.

## Schema drift

Every parse failure is counted; a rate above 0.1% aborts. Unknown or unsafe feature
shapes are reported. The initial snapshot establishes coverage denominators; future
refreshes must explain drops instead of changing the denominator silently.
`check-registry` pins an offline floor of 3,000 accepted lexemes and 130,000 variants,
checks report denominators, validates contiguous variant ranks and sentinels, then
regenerates the runtime source byte-for-byte.

`accuracy --dump PATH` accepts either a normalized-registry directory or the pinned
raw dump. For a raw dump it verifies the committed byte length and SHA-256 before
evaluating the committed normalization; changed raw data must go through
`refresh-data` first. Normal builds and checks never download a source.

`accuracy-corpus --ud PATH --syntacticus PATH` verifies every file against
`data/evaluation-sources.json` before parsing. The UD mapper admits only complete,
lossless verb bundles. In particular, UD finite `Tense=Past` is counted as
`incompatible-past-subtype`; lexical `Aspect` is never reinterpreted as aorist or
imperfect. Native PROIEL/TOROT ten-character morphology retains `i` (imperfect) and
`a` (aorist), so it owns past-subtype evaluation.

The report separates public facade recall, productive core generalization using an
explicit oracle principal part, and a lemma-disjoint OOV view. When a native token
supplies a diagnostic stem/formation, every token in that person-number cell is
excluded. Diplomatic and shared NFC/lowercase lookup scores, top-1 and any-variant,
coverage, conditional accuracy, documents, cells, and exact skip reasons remain
separate. `--details PATH` writes local token-level evidence; these CC BY-NC-SA files
and derived excerpts must never be committed. See `docs/CORPUS_EVALUATION.md`.

The original `accuracy-ud` command remains a compatibility alias for the UD-only
diagnostic. It does not evaluate native past subtypes.
