# Downloads over GitHub's file limit

Three downloads are larger than GitHub's 100 MB per-file limit, so they are
not in the repository. They are assets of the release
[`reference-downloads-2026-08-09`](https://github.com/gold-silver-copper/church-slavonic/releases/tag/reference-downloads-2026-08-09),
as they were downloaded on 2026-08-09. Upstream does not keep them: Wikimedia
deletes old dumps, and kaikki.org replaces its file every week.

| Path under `references/downloads/` | Bytes | SHA-256 |
|---|---|---|
| `english-wiktionary-ocs-lineage/enwiktionary-20260801-pages-articles.xml.bz2` | 1622726291 | `a0db4eeaea5e6b9940e21ea143eaecf93195610888d345333ae560b9a15527d1` |
| `english-wiktionary-ocs-lineage/raw-wiktextract-data.jsonl.gz` | 2826631951 | `4bd65813f0f56c6a584ca78974c8e015a832039f9fca4ac474d5db7c28ea277f` |
| `ponomar-elizabeth-bible/ponomar-0af645f438856f45c22026912d2e4a9ce495e531.tar.gz` | 206389751 | `86c5e584dbe2135b87466ed8a38353794f20cac6fdea89eef9d17a5819c1681c` |

A release asset may be at most 2 GB, so `raw-wiktextract-data.jsonl.gz` is
split into `.part-aa` and `.part-ab`. To restore all three:

```sh
cd references/downloads
gh release download reference-downloads-2026-08-09 -R gold-silver-copper/church-slavonic -D /tmp/cs-large
mv /tmp/cs-large/enwiktionary-20260801-pages-articles.xml.bz2 english-wiktionary-ocs-lineage/
cat /tmp/cs-large/raw-wiktextract-data.jsonl.gz.part-* > english-wiktionary-ocs-lineage/raw-wiktextract-data.jsonl.gz
mv /tmp/cs-large/ponomar-0af645f438856f45c22026912d2e4a9ce495e531.tar.gz ponomar-elizabeth-bible/
shasum -a 256 english-wiktionary-ocs-lineage/*.bz2 english-wiktionary-ocs-lineage/raw-wiktextract-data.jsonl.gz ponomar-elizabeth-bible/*.tar.gz
```
