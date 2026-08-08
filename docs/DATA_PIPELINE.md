# Data pipeline

## Stages and ownership

| Stage | Reads | Owns |
|---|---|---|
| `refresh-data` | an explicit local Kaikki/Wiktextract JSONL path | `data/extracted/*.tsv`, `data/extracted/source.json`, generated Rust, extraction reports |
| `check-registry` | committed normalized TSV | nothing; compares deterministic generated Rust and validates semantics |
| `accuracy` | committed registry and public facade | `reports/accuracy.{json,md}` when refreshed |
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

`accuracy-ud` accepts a user-supplied PROIEL checkout and compares only feature
bundles that map without guessing: nominal/closed-class cells, present finite verbs,
imperatives, infinitives, supines, and resultative/l-participles. PROIEL does not
encode the short/long adjective distinction and collapses multiple past finite
tenses, so those incompatible bundles are skipped. Raw and NFC/lowercase results are
printed separately; the noncommercial corpus is never copied into this project.
