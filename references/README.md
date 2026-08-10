# Machine-readable references

This directory is the local, reproducible source cache for Old Church Slavonic
and Synodal Russian Church Slavonic work. Run:

```sh
./references/fetch-sources.sh
```

Raw downloads are stored under `references/downloads/` and are intentionally
gitignored. `SOURCES.toml` records their immutable revisions, licensing status,
and intended evidential role. `SOURCE_LOCK.tsv` locks the URL, path, size, format,
and SHA-256 of every artifact; ordinary fetch and verification commands never
rewrite it. The complete locked cache is currently 321 files and approximately
4.6 GB. This includes all 198 linked sections of the
Alypy/Gamanovich grammar, not only its table of contents, plus the stable PDF and
metadata page for Unicode Technical Note #41 revision 1 and both publisher
editions (TEI XML and PDF) of Polivanova's grammar. Wikisource is stored as 78
exact revision-pinned wikitext artifacts listed in `WIKISOURCE_REVISIONS.tsv`.

The complete command, refresh, offline, recovery, prerequisite, and licensing
procedure is documented in `docs/SYNODAL_DATA_PIPELINE.md`. The default CI uses
`cargo xtask synodal-fixture-bootstrap`; the 4.6 GB full bootstrap is a separate
manually triggered workflow.

Not every item in the research backlog may lawfully or technically be bulk
downloaded. GORAZD and the Russian National Corpus are metadata-only until bulk
access terms are confirmed. The Ponomar web library is represented by its
catalog and legal page, rather than mirroring modern editions whose individual
rights have not yet been audited. Firenze University Press requires a
browser-compatible request for its public CC BY downloads; the fetch script
handles that source explicitly and caches both the scholarly TEI XML and PDF.

Evaluation-only and unresolved-license material remains local. Do not package
the contents of `references/downloads/` into crates or redistribute the cache as
a unit without reviewing each source's license.
