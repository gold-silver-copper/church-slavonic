# Synodal corpus coverage

## Corpus-wide coverage

- Passages: 10
- Tokens: 155
- Types: 113
- Top-1 analyzed: 73 (4709 bp)
- Top-k analyzed: 120 (7741 bp)
- Ambiguous: 2
- Unresolved: 35

## Gap categories

| Category | Tokens |
|---|---:|
| `unknown-lexeme` | 33 |
| `missing-declension-or-class` | 0 |
| `missing-verb-principal-part` | 0 |
| `unsupported-formation` | 0 |
| `missing-accent-or-orthographic-metadata` | 2 |
| `ambiguity-or-spelling-variant` | 2 |

## Coverage composition

Strict top-k counts tokens that have *any* analysis. These measures describe what
that coverage is made of, so recall cannot be bought with rows that commit to no
morphology, and so a fall in unique-reading counts can be attributed rather than
assumed. `morphology-free` tokens carry only `lexical-form` readings.
`lemma-unique` is not capped by syncretism the way top-1 is.

| Measure | Tokens | Share of top-k |
|---|---:|---:|
| morphologically typed | 114 | 9500 bp |
| morphology-free | 6 | 500 bp |
| lemma-unique | 118 | 9833 bp |
| within-lexeme ambiguous (syncretism) | 44 | 3666 bp |
| cross-lexeme ambiguous (homonymy) | 2 | 166 bp |

## Estimated recovery routes

These are diagnostic estimates, not admitted lexical identities or guaranteed recoveries.

| Route | Tokens |
|---|---:|
| `exact-evidence` | 0 |
| `reviewed-class` | 0 |
| `reviewed-principal-part` | 0 |
| `abbreviation-registry` | 7 |
| `spelling-variant` | 2 |
| `unsupported-formation` | 0 |
| `ungrouped-unknown` | 26 |

## Unresolved tokens by probable family

| Family diagnostic | Tokens | Documents | Route | Surfaces |
|---|---:|---:|---|---|
| `ungrouped:нимиже` | 2 | 2 | `ungrouped-unknown` | ни́миже |
| `ungrouped:слышасте` | 2 | 2 | `ungrouped-unknown` | слы́шасте |
| `family:synodal:noun:v06-673b2df93b4f89a8` | 1 | 1 | `spelling-variant` | зна́менїихъ |
| `family:synodal:verb:wikt-6ceeefbe4e9e` | 1 | 1 | `spelling-variant` | ꙗ҆ды́й |
| `ungrouped:апⷭ҇лъ` | 1 | 1 | `abbreviation-registry` | а҆пⷭ҇лъ |
| `ungrouped:апⷭ҇лѡмъ` | 1 | 1 | `abbreviation-registry` | а҆пⷭ҇лѡмъ |
| `ungrouped:бж҃їею` | 1 | 1 | `abbreviation-registry` | бж҃їею |
| `ungrouped:благодаримъ` | 1 | 1 | `ungrouped-unknown` | Благодари́мъ |
| `ungrouped:бл҃говѣствованїѧ` | 1 | 1 | `abbreviation-registry` | бл҃говѣствова́нїѧ |
| `ungrouped:вознесесѧ` | 1 | 1 | `ungrouped-unknown` | вознесе́сѧ |
| `ungrouped:воньже` | 1 | 1 | `ungrouped-unknown` | во́ньже |
| `ungrouped:вѣрнымъ` | 1 | 1 | `ungrouped-unknown` | вѣ̑рнымъ |
| `ungrouped:ждати` | 1 | 1 | `ungrouped-unknown` | жда́ти |
| `ungrouped:заповѣдавъ` | 1 | 1 | `ungrouped-unknown` | заповѣ́давъ |
| `ungrouped:истинныхъ` | 1 | 1 | `ungrouped-unknown` | и҆́стинныхъ |
| `ungrouped:колоссаехъ` | 1 | 1 | `ungrouped-unknown` | колосса́ехъ |
| `ungrouped:крестилъ` | 1 | 1 | `ungrouped-unknown` | крести́лъ |
| `ungrouped:креститисѧ` | 1 | 1 | `ungrouped-unknown` | крести́тисѧ |
| `ungrouped:молѧщесѧ` | 1 | 1 | `ungrouped-unknown` | молѧ́щесѧ |
| `ungrouped:паѵелъ` | 1 | 1 | `ungrouped-unknown` | Па́ѵелъ |
| `ungrouped:слышавше` | 1 | 1 | `ungrouped-unknown` | слы́шавше |
| `ungrouped:страданїи` | 1 | 1 | `ungrouped-unknown` | страда́нїи |
| `ungrouped:ст҃ымъ` | 1 | 1 | `abbreviation-registry` | ст҃ы̑мъ |
| `ungrouped:тїмоѳей` | 1 | 1 | `ungrouped-unknown` | тїмоѳе́й |
| `ungrouped:хрⷭ҇товъ` | 1 | 1 | `abbreviation-registry` | хрⷭ҇то́въ |
| `ungrouped:ѡбѣтованїѧ` | 1 | 1 | `ungrouped-unknown` | ѡ҆бѣтова́нїѧ |
| `ungrouped:ѳеофїле` | 1 | 1 | `ungrouped-unknown` | ѳео́фїле |
| `ungrouped:ѹпованїе` | 1 | 1 | `ungrouped-unknown` | ѹ҆пова́нїе |
| `ungrouped:ѻч҃а` | 1 | 1 | `abbreviation-registry` | ѻ҆́ч҃а |
| `ungrouped:ѿложенное` | 1 | 1 | `ungrouped-unknown` | ѿложе́нное |
| `ungrouped:ѿлꙋчатисѧ` | 1 | 1 | `ungrouped-unknown` | ѿлꙋча́тисѧ |
| `ungrouped:ᲂучити` | 1 | 1 | `ungrouped-unknown` | ᲂу҆чи́ти |
| `ungrouped:ꙗвлѧѧсѧ` | 1 | 1 | `ungrouped-unknown` | ꙗ҆влѧ́ѧсѧ |

## Coverage by corpus

| Corpus | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| Church Slavonic Bible corpus | 70 | 32 | 55 | 1 | 15 |
| Elizabeth Bible corpus | 85 | 41 | 65 | 1 | 20 |

## Coverage by source

| Source | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `ponomar-elizabeth-bible-2026-08-09` | 85 | 41 | 65 | 1 | 20 |
| `wikisource-church-slavonic-bible-2026-08-09` | 70 | 32 | 55 | 1 | 15 |

## Coverage by partition

| Partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `evaluation` | 60 | 30 | 46 | 0 | 14 |
| `source` | 95 | 43 | 74 | 2 | 21 |

## Coverage by source and partition

| Source/partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `ponomar-elizabeth-bible-2026-08-09:evaluation` | 50 | 24 | 39 | 0 | 11 |
| `ponomar-elizabeth-bible-2026-08-09:source` | 35 | 17 | 26 | 1 | 9 |
| `wikisource-church-slavonic-bible-2026-08-09:evaluation` | 10 | 6 | 7 | 0 | 3 |
| `wikisource-church-slavonic-bible-2026-08-09:source` | 60 | 26 | 48 | 1 | 12 |

## Gap categories by source

| Source | Category | Tokens |
|---|---|---:|
| `ponomar-elizabeth-bible-2026-08-09` | `unknown-lexeme` | 18 |
| `ponomar-elizabeth-bible-2026-08-09` | `missing-accent-or-orthographic-metadata` | 2 |
| `ponomar-elizabeth-bible-2026-08-09` | `ambiguity-or-spelling-variant` | 1 |
| `wikisource-church-slavonic-bible-2026-08-09` | `unknown-lexeme` | 15 |
| `wikisource-church-slavonic-bible-2026-08-09` | `ambiguity-or-spelling-variant` | 1 |

## Gap categories by partition

| Partition | Category | Tokens |
|---|---|---:|
| `evaluation` | `unknown-lexeme` | 13 |
| `evaluation` | `missing-accent-or-orthographic-metadata` | 1 |
| `source` | `unknown-lexeme` | 20 |
| `source` | `missing-accent-or-orthographic-metadata` | 1 |
| `source` | `ambiguity-or-spelling-variant` | 2 |

## Review queue

| Rank | Gap | Token | Frequency | Documents | Action |
|---:|---|---|---:|---:|---|
| 1 | `unknown-lexeme` | `ни́миже` | 2 | 2 | review the token against target-recension evidence and create or reject a lexical candidate |
| 2 | `unknown-lexeme` | `слы́шасте` | 2 | 2 | review the token against target-recension evidence and create or reject a lexical candidate |
| 3 | `ambiguity-or-spelling-variant` | `и҆́мате` | 2 | 2 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 4 | `unknown-lexeme` | `а҆пⷭ҇лъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 5 | `unknown-lexeme` | `а҆пⷭ҇лѡмъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 6 | `unknown-lexeme` | `бж҃їею` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 7 | `unknown-lexeme` | `Благодари́мъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 8 | `unknown-lexeme` | `бл҃говѣствова́нїѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 9 | `unknown-lexeme` | `вознесе́сѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 10 | `unknown-lexeme` | `во́ньже` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 11 | `unknown-lexeme` | `вѣ̑рнымъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 12 | `unknown-lexeme` | `жда́ти` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 13 | `unknown-lexeme` | `заповѣ́давъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 14 | `unknown-lexeme` | `и҆́стинныхъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 15 | `unknown-lexeme` | `колосса́ехъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 16 | `unknown-lexeme` | `крести́лъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 17 | `unknown-lexeme` | `крести́тисѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 18 | `unknown-lexeme` | `молѧ́щесѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 19 | `unknown-lexeme` | `Па́ѵелъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 20 | `unknown-lexeme` | `слы́шавше` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 21 | `unknown-lexeme` | `страда́нїи` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 22 | `unknown-lexeme` | `ст҃ы̑мъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 23 | `unknown-lexeme` | `тїмоѳе́й` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 24 | `unknown-lexeme` | `хрⷭ҇то́въ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 25 | `unknown-lexeme` | `ѡ҆бѣтова́нїѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 26 | `unknown-lexeme` | `ѳео́фїле` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 27 | `unknown-lexeme` | `ѹ҆пова́нїе` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 28 | `unknown-lexeme` | `ѻ҆́ч҃а` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 29 | `unknown-lexeme` | `ѿложе́нное` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 30 | `unknown-lexeme` | `ѿлꙋча́тисѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 31 | `unknown-lexeme` | `ᲂу҆чи́ти` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 32 | `unknown-lexeme` | `ꙗ҆влѧ́ѧсѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 33 | `missing-accent-or-orthographic-metadata` | `зна́менїихъ` | 1 | 1 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 34 | `missing-accent-or-orthographic-metadata` | `ꙗ҆ды́й` | 1 | 1 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
