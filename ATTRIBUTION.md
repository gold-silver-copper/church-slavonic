# Attribution and licensing

This repository contains original source code and generated data under different
licenses.

| Material | Location | License |
|---|---|---|
| Original Rust code and documentation | `crates/*/src`, `docs`, manifests | MIT OR Apache-2.0 |
| Normalized dictionary registry | `data/extracted` | English Wiktionary CC BY-SA 4.0 |
| Generated Rust dictionary tables | `crates/old-church-slavonic/generated` | derived from English Wiktionary; CC BY-SA 4.0 |
| Extraction/accuracy reports containing forms | `reports` | derived from English Wiktionary; CC BY-SA 4.0 |
| UD OCS PROIEL | not bundled | CC BY-NC-SA 4.0; optional local evaluation only |

## English Wiktionary and Wiktextract

Dictionary entries and inflection tables originate with **English Wiktionary and its
contributors**. English Wiktionary makes the source available under CC BY-SA 4.0 and
GFDL 1.1 or later; this distribution elects CC BY-SA 4.0. License text and upstream
terms are available at <https://creativecommons.org/licenses/by-sa/4.0/legalcode> and
<https://en.wiktionary.org/wiki/Wiktionary:Copyrights>.

The machine-readable source was extracted by **Wiktextract**, by Tatu Ylonen, and
distributed through Kaikki:

- <https://github.com/tatuylonen/wiktextract>
- <https://kaikki.org/dictionary/Old%20Church%20Slavonic/index.html>

The exact source hash and extraction dates are recorded in `data/SOURCES.toml` and
`data/extracted/source.json`. This project modifies the source by selecting OCS
entries, rejecting unsafe cells, normalizing them into a structured registry, and
generating static Rust tables. The raw JSONL dump is not redistributed.

Wiktionary table cells are template-generated dictionary forms. The project does not
describe them as manuscript-attested unless separate corpus evidence supports that
claim.

## UD Old Church Slavonic PROIEL

The optional evaluation command accepts a user-supplied checkout of UD Old Church
Slavonic PROIEL. It is not included in this repository, generated data, runtime
crate, or package because its license is CC BY-NC-SA 4.0. See
<https://universaldependencies.org/treebanks/cu_proiel/index.html>.

## Reuse

Keep the MIT/Apache notices with code. When redistributing the registry, generated
tables, or reports, credit English Wiktionary and its contributors, retain this
attribution and modification notice, and comply with CC BY-SA 4.0.
