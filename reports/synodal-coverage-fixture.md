# Synodal corpus coverage

## Corpus-wide coverage

- Passages: 10
- Tokens: 155
- Types: 113
- Top-1 analyzed: 73 (4709 bp)
- Top-k analyzed: 123 (7935 bp)
- Ambiguous: 2
- Unresolved: 32

## Gap categories

| Category | Tokens |
|---|---:|
| `unknown-lexeme` | 29 |
| `missing-declension-or-class` | 0 |
| `missing-verb-principal-part` | 0 |
| `unsupported-formation` | 0 |
| `missing-accent-or-orthographic-metadata` | 3 |
| `ambiguity-or-spelling-variant` | 2 |

## Coverage composition

Strict top-k counts tokens that have *any* analysis. These measures describe what
that coverage is made of, so recall cannot be bought with rows that commit to no
morphology, and so a fall in unique-reading counts can be attributed rather than
assumed. `morphology-free` tokens carry only `lexical-form` readings.
`lemma-unique` is not capped by syncretism the way top-1 is.

| Measure | Tokens | Share of top-k |
|---|---:|---:|
| morphologically typed | 117 | 9512 bp |
| morphology-free | 6 | 487 bp |
| lemma-unique | 121 | 9837 bp |
| within-lexeme ambiguous (syncretism) | 47 | 3821 bp |
| cross-lexeme ambiguous (homonymy) | 2 | 162 bp |

## Estimated recovery routes

These are diagnostic estimates, not admitted lexical identities or guaranteed recoveries.

| Route | Tokens |
|---|---:|
| `exact-evidence` | 0 |
| `reviewed-class` | 0 |
| `reviewed-principal-part` | 0 |
| `abbreviation-registry` | 7 |
| `spelling-variant` | 3 |
| `unsupported-formation` | 0 |
| `ungrouped-unknown` | 22 |

## Exploratory predictions over the unresolved remainder

Diagnostic only. These tokens have no reviewed reading; the corpus-free
segmentation tier (`SYN-PREDICT-VERB-SEGMENTATION-V1`, reachable only under
`GenerationPolicy::Exploratory`) can offer a typed hypothesis for them. They
never count toward strict top-k and no sealed floor reads this table; the
masked precision gate lives in `reports/synodal-prediction-precision.md`.

| Top prediction's system | Tokens |
|---|---:|
| `aorist` | 9 |
| `imperative` | 1 |
| `imperfect` | 1 |
| `infinitive` | 4 |
| `l-participle` | 2 |
| `present` | 1 |

| Confidence bucket (bp) | Tokens |
|---|---:|
| 0-2399 | 7 |
| 2400-2999 | 5 |
| 3000-3399 | 6 |

## Unresolved tokens by probable family

| Family diagnostic | Tokens | Documents | Route | Surfaces |
|---|---:|---:|---|---|
| `ungrouped:нимиже` | 2 | 2 | `ungrouped-unknown` | ни́миже |
| `family:synodal:noun:v06-673b2df93b4f89a8` | 1 | 1 | `spelling-variant` | зна́менїихъ |
| `family:synodal:verb:v12-slyshati` | 1 | 1 | `spelling-variant` | слы́шавше |
| `family:synodal:verb:wikt-6ceeefbe4e9e` | 1 | 1 | `spelling-variant` | ꙗ҆ды́й |
| `ungrouped:апⷭ҇лъ` | 1 | 1 | `abbreviation-registry` | а҆пⷭ҇лъ |
| `ungrouped:апⷭ҇лѡмъ` | 1 | 1 | `abbreviation-registry` | а҆пⷭ҇лѡмъ |
| `ungrouped:бж҃їею` | 1 | 1 | `abbreviation-registry` | бж҃їею |
| `ungrouped:благодаримъ` | 1 | 1 | `ungrouped-unknown` | Благодари́мъ |
| `ungrouped:бл҃говѣствованїѧ` | 1 | 1 | `abbreviation-registry` | бл҃говѣствова́нїѧ |
| `ungrouped:воньже` | 1 | 1 | `ungrouped-unknown` | во́ньже |
| `ungrouped:вѣрнымъ` | 1 | 1 | `ungrouped-unknown` | вѣ̑рнымъ |
| `ungrouped:ждати` | 1 | 1 | `ungrouped-unknown` | жда́ти |
| `ungrouped:заповѣдавъ` | 1 | 1 | `ungrouped-unknown` | заповѣ́давъ |
| `ungrouped:истинныхъ` | 1 | 1 | `ungrouped-unknown` | и҆́стинныхъ |
| `ungrouped:колоссаехъ` | 1 | 1 | `ungrouped-unknown` | колосса́ехъ |
| `ungrouped:крестилъ` | 1 | 1 | `ungrouped-unknown` | крести́лъ |
| `ungrouped:креститисѧ` | 1 | 1 | `ungrouped-unknown` | крести́тисѧ |
| `ungrouped:молѧщесѧ` | 1 | 1 | `ungrouped-unknown` | молѧ́щесѧ |
| `ungrouped:оучити` | 1 | 1 | `ungrouped-unknown` | ᲂу҆чи́ти |
| `ungrouped:паѵелъ` | 1 | 1 | `ungrouped-unknown` | Па́ѵелъ |
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
| `ungrouped:ꙗвлѧѧсѧ` | 1 | 1 | `ungrouped-unknown` | ꙗ҆влѧ́ѧсѧ |

## Coverage by corpus

| Corpus | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| Church Slavonic Bible corpus | 70 | 32 | 56 | 1 | 14 |
| Elizabeth Bible corpus | 85 | 41 | 67 | 1 | 18 |

## Coverage by source

| Source | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `ponomar-elizabeth-bible-2026-08-09` | 85 | 41 | 67 | 1 | 18 |
| `wikisource-church-slavonic-bible-2026-08-09` | 70 | 32 | 56 | 1 | 14 |

## Coverage by partition

| Partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `evaluation` | 60 | 30 | 47 | 0 | 13 |
| `source` | 95 | 43 | 76 | 2 | 19 |

## Coverage by source and partition

| Source/partition | Tokens | Top-1 | Top-k | Ambiguous | Unresolved |
|---|---:|---:|---:|---:|---:|
| `ponomar-elizabeth-bible-2026-08-09:evaluation` | 50 | 24 | 40 | 0 | 10 |
| `ponomar-elizabeth-bible-2026-08-09:source` | 35 | 17 | 27 | 1 | 8 |
| `wikisource-church-slavonic-bible-2026-08-09:evaluation` | 10 | 6 | 7 | 0 | 3 |
| `wikisource-church-slavonic-bible-2026-08-09:source` | 60 | 26 | 49 | 1 | 11 |

## Gap categories by source

| Source | Category | Tokens |
|---|---|---:|
| `ponomar-elizabeth-bible-2026-08-09` | `unknown-lexeme` | 16 |
| `ponomar-elizabeth-bible-2026-08-09` | `missing-accent-or-orthographic-metadata` | 2 |
| `ponomar-elizabeth-bible-2026-08-09` | `ambiguity-or-spelling-variant` | 1 |
| `wikisource-church-slavonic-bible-2026-08-09` | `unknown-lexeme` | 13 |
| `wikisource-church-slavonic-bible-2026-08-09` | `missing-accent-or-orthographic-metadata` | 1 |
| `wikisource-church-slavonic-bible-2026-08-09` | `ambiguity-or-spelling-variant` | 1 |

## Gap categories by partition

| Partition | Category | Tokens |
|---|---|---:|
| `evaluation` | `unknown-lexeme` | 12 |
| `evaluation` | `missing-accent-or-orthographic-metadata` | 1 |
| `source` | `unknown-lexeme` | 17 |
| `source` | `missing-accent-or-orthographic-metadata` | 2 |
| `source` | `ambiguity-or-spelling-variant` | 2 |

## Review queue

| Rank | Gap | Token | Frequency | Documents | Action |
|---:|---|---|---:|---:|---|
| 1 | `unknown-lexeme` | `ни́миже` | 2 | 2 | review the token against target-recension evidence and create or reject a lexical candidate |
| 2 | `ambiguity-or-spelling-variant` | `и҆́мате` | 2 | 2 | review semantic identity and variant ordering; require a stable lexeme ID when ambiguity remains |
| 3 | `unknown-lexeme` | `а҆пⷭ҇лъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 4 | `unknown-lexeme` | `а҆пⷭ҇лѡмъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 5 | `unknown-lexeme` | `бж҃їею` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 6 | `unknown-lexeme` | `Благодари́мъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 7 | `unknown-lexeme` | `бл҃говѣствова́нїѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 8 | `unknown-lexeme` | `во́ньже` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 9 | `unknown-lexeme` | `вѣ̑рнымъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 10 | `unknown-lexeme` | `жда́ти` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 11 | `unknown-lexeme` | `заповѣ́давъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 12 | `unknown-lexeme` | `и҆́стинныхъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 13 | `unknown-lexeme` | `колосса́ехъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 14 | `unknown-lexeme` | `крести́лъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 15 | `unknown-lexeme` | `крести́тисѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 16 | `unknown-lexeme` | `молѧ́щесѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 17 | `unknown-lexeme` | `ᲂу҆чи́ти` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 18 | `unknown-lexeme` | `Па́ѵелъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 19 | `unknown-lexeme` | `страда́нїи` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 20 | `unknown-lexeme` | `ст҃ы̑мъ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 21 | `unknown-lexeme` | `тїмоѳе́й` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 22 | `unknown-lexeme` | `хрⷭ҇то́въ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 23 | `unknown-lexeme` | `ѡ҆бѣтова́нїѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 24 | `unknown-lexeme` | `ѳео́фїле` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 25 | `unknown-lexeme` | `ѹ҆пова́нїе` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 26 | `unknown-lexeme` | `ѻ҆́ч҃а` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 27 | `unknown-lexeme` | `ѿложе́нное` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 28 | `unknown-lexeme` | `ѿлꙋча́тисѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 29 | `unknown-lexeme` | `ꙗ҆влѧ́ѧсѧ` | 1 | 1 | review the token against target-recension evidence and create or reject a lexical candidate |
| 30 | `missing-accent-or-orthographic-metadata` | `зна́менїихъ` | 1 | 1 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 31 | `missing-accent-or-orthographic-metadata` | `слы́шавше` | 1 | 1 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
| 32 | `missing-accent-or-orthographic-metadata` | `ꙗ҆ды́й` | 1 | 1 | review the accent, breathing, titlo, and positional-letter evidence instead of accepting an accentless fallback |
