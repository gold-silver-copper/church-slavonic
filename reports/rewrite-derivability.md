# Rewrite derivability (phase 3 groundwork)

How much of the extracted registry the pure rule kernel reproduces from
compact per-lexeme metadata alone (class codes, genders, verb
principal-part metadata), bypassing every stored surface form. A cell is
*derivable* when the rule prediction matches the stored variant list
exactly (Cyrillic text, variant order included); *divergent* when the
rules predict something else; *unsupported* when no rule prediction is
possible. Divergent and unsupported cells are the residue that must ship
as lookup tables.

## Per part of speech

| POS | cells | derivable | divergent | unsupported | derivable % | lexemes | fully derivable lexemes |
|---|---:|---:|---:|---:|---:|---:|---:|
| adj | 78432 | 74633 | 3775 | 24 | 95.16% | 312 | 0 |
| det | 36 | 0 | 0 | 36 | 0.00% | 1 | 0 |
| noun | 41566 | 33132 | 4058 | 4376 | 79.71% | 2020 | 563 |
| num | 126 | 0 | 0 | 126 | 0.00% | 8 | 0 |
| pron | 1341 | 0 | 0 | 1341 | 0.00% | 29 | 0 |
| verb | 13260 | 6425 | 311 | 6524 | 48.45% | 711 | 0 |
| **total** | 134761 | 114190 | 8144 | 12427 | 84.74% | 3081 | 563 |

## Residual table estimate

- Residual cells (divergent + unsupported): 20571
- Residual surface variants: 23172
- Divergent cells whose primary variant still matches the rules (the table only adds or reorders variants): 790 of 8144
- Residual form text: 346602 UTF-8 bytes (romanization adds 207767 bytes)
- Lexemes needing zero table rows: 563 of 3081 (18.27%)

## Largest divergent categories

| category | cells |
|---|---:|
| adj/adj-hard:short | 1976 |
| noun/i-f | 916 |
| adj/adj-hard:long | 828 |
| noun/jo-m-soft | 805 |
| adj/adj-soft:short | 765 |
| noun/a-hard | 644 |
| noun/ja-soft | 523 |
| noun/jo-n-soft | 401 |
| noun/o-m-hard | 375 |
| adj/adj-soft:long | 206 |
| noun/i-m | 204 |
| verb/finite:imperfect | 150 |
| verb/finite:present | 121 |
| noun/u-m | 91 |
| noun/s-n | 36 |
| (other) | 103 |

## Largest unsupported categories

| category | cells |
|---|---:|
| verb/l-participle | 4734 |
| noun/o-m-hard | 1833 |
| closed-class/pron | 1341 |
| noun/raw:a-stem | 1260 |
| noun/jo-m-soft | 772 |
| verb/imperative | 709 |
| verb/finite:present | 630 |
| verb/finite:imperfect | 339 |
| noun/raw:o-stem | 245 |
| closed-class/num | 126 |
| noun/jo-n-soft | 105 |
| noun/(no class) | 70 |
| verb/verbal-noun | 57 |
| verb/finite:aorist | 45 |
| noun/s-n | 42 |
| (other) | 119 |

## Example divergences (first per top category)

- adj/adj-hard:short (1976): авьнъ `adj:short:acc:pl:m:an`: stored авьнꙑ vs rules авьнъ
- noun/i-f (916): аблань `noun:gen:du`: stored абланью / абланию vs rules абланию
- adj/adj-hard:long (828): авьнъ `adj:long:acc:pl:m:an`: stored авьнꙑѩ vs rules авьнꙑихъ
- noun/jo-m-soft (805): агньць `noun:dat:sg`: stored агньцоу / агньцеви vs rules агньцоу
- adj/adj-soft:short (765): боуи `adj:short:acc:du:m:an`: stored боуꙗ vs rules боуа
- noun/a-hard (644): багърѣница `noun:acc:du`: stored багърѣници vs rules багърѣницѣ
- noun/ja-soft (523): алъдии `noun:ins:sg`: stored алъдиѥѭ vs rules алъдиеѭ
- noun/jo-n-soft (401): алъкаиѥ `noun:dat:du`: stored алъкаиема vs rules алъкаиѥма
- noun/o-m-hard (375): агарѣнинъ `noun:dat:pl`: stored агарѣньмъ vs rules агарѣниномъ
- adj/adj-soft:long (206): боуи `adj:long:acc:pl:m:an`: stored боуѧѩ vs rules боуиихъ

## Notes

- The dominant adjective divergence is the animate accusative:
the extracted tables keep the plain accusative in `acc:pl/du:m:an`
cells while the core rules apply genitive syncretism. That is one
systematic convention difference, not thousands of independent
irregularities.
- Noun divergences are mostly extra stored variants (e.g. dative
`-оу / -еви`), `ѥ/е`-style orthographic variant spellings, and the
`-инъ` singulative subclass whose plural drops the suffix
(not modelled as a class).
- The verb residue is dominated by missing metadata, not failing
rules: `verb_metadata.tsv` covers only a minority of verbs per
system (about 121 present analyses, 185 l-participle stems for 711
verbs), so most verb cells have no rule input at all.
- Verb predictions consume `verb_metadata.tsv` (stems and formation
codes with provenance); that metadata is itself compact per-lexeme
data the rewrite would keep, not a surface table.
- Closed classes (pron/num/det `decl:*` cells) are counted as
unsupported here because this harness does not wire the reviewed
closed-class kernels; the core crate already models the major
paradigms as reviewed tables keyed by identity.
- Comparisons are on canonical Cyrillic text only; romanization is
assumed to be regenerable by transliteration.
