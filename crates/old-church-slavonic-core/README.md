# old-church-slavonic-core

Pure, rule-based Old Church Slavonic morphology. This crate bundles no dictionary,
performs no I/O, and returns typed failures when a lemma does not carry enough
lexical information. Use `old-church-slavonic` for dictionary-backed forms.

Grammar, result, and trace types have explicit root re-exports. Productive entry
points remain organized by linguistic owner in `noun`, `adjective`, and `verb`;
callers supply the corresponding typed lexeme and cell. A citation alone never
silently selects a lexical class or principal part.

Verb generation requires independent typed principal parts for the present,
imperfect, aorist, imperative, l-participle, and each non-l participle system.
Lexical aspect never chooses an aorist. The core supports explicit A/Yat-A/
palatalized imperfects under an explicit `UncontractedOnly` variant policy,
asigmatic and new *ox*-aorists, both historical imperative
series, and audited participial seams including transformed i-stem `-ьш-`, declared
final-j deletion, and `ov → u`. `SigmaticPrimary` and `SigmaticSecondary` are
represented separately but intentionally return `UnsupportedFormation` until
their root-grade, singular-allomorph, seam, and optional-`-тъ` dimensions are
modeled independently.
