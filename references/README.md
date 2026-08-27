# Pinned sources

The extractor reads exactly two sources, both pinned here and stored under
`references/downloads/` (gitignored):

| Directory | Source |
|---|---|
| `downloads/english-wiktionary-ocs/` | English Wiktionary Old Church Slavonic via Kaikki/Wiktextract (one JSONL file) |
| `downloads/alypy-grammar/` | Archbishop Alypy (Gamanovich), *Grammar of the Church Slavonic Language*, web edition (198 `.htm` pages) |

`SOURCES.toml` records each source's revision, licence and role;
`SOURCE_LOCK.tsv` locks every artifact's URL, path, size and SHA-256;
`SHA256SUMS` is the same checksum list in `shasum -c` form. Run

```sh
./scripts/fetch-sources.sh
```

to download whatever is missing and verify the cache, then
`cargo xtask refresh-data` to regenerate the tables. The README's source table
is the human-readable summary of this directory; see it for licences.
