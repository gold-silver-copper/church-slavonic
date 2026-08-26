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
| det | 36 | 34 | 2 | 0 | 94.44% | 1 | 0 |
| noun | 41566 | 33360 | 4160 | 4046 | 80.26% | 2020 | 563 |
| num | 126 | 100 | 19 | 7 | 79.37% | 8 | 0 |
| pron | 1341 | 824 | 127 | 390 | 61.45% | 29 | 1 |
| verb | 13260 | 7349 | 815 | 5096 | 55.42% | 711 | 38 |
| **total** | 134761 | 116300 | 8898 | 9563 | 86.30% | 3081 | 602 |

## Residual table estimate

- Residual cells (divergent + unsupported): 18461
- Residual surface variants: 20972
- Divergent cells whose primary variant still matches the rules (the table only adds or reorders variants): 862 of 8898
- Residual form text: 321444 UTF-8 bytes (romanization adds 192217 bytes)
- Lexemes needing zero table rows: 602 of 3081 (19.54%)

## Largest divergent categories

| category | cells |
|---|---:|
| adj/adj-hard:short | 1976 |
| noun/i-f | 916 |
| adj/adj-hard:long | 828 |
| adj/adj-soft:short | 765 |
| noun/a-hard | 644 |
| noun/reviewed-twofold | 612 |
| noun/ja-soft | 514 |
| noun/jo-n-soft | 401 |
| noun/o-m-hard | 354 |
| noun/jo-m-soft | 338 |
| verb/l-participle | 252 |
| adj/adj-soft:long | 206 |
| verb/finite:present | 190 |
| noun/i-m | 185 |
| verb/finite:imperfect | 173 |
| (other) | 544 |

## Largest unsupported categories

| category | cells |
|---|---:|
| verb/l-participle | 4086 |
| noun/o-m-hard | 1821 |
| noun/raw:a-stem | 1071 |
| noun/jo-m-soft | 685 |
| verb/finite:present | 423 |
| closed-class/pron | 390 |
| verb/imperative | 337 |
| verb/finite:imperfect | 210 |
| noun/raw:o-stem | 203 |
| noun/jo-n-soft | 105 |
| noun/(no class) | 70 |
| noun/s-n | 42 |
| verb/verbal-noun | 40 |
| adj/comparative-citation | 24 |
| noun/raw:i-stem | 21 |
| (other) | 35 |

## Example divergences (first per top category)

- adj/adj-hard:short (1976): авьнъ `adj:short:acc:pl:m:an`: stored авьнꙑ vs rules авьнъ
- noun/i-f (916): аблань `noun:gen:du`: stored абланью / абланию vs rules абланию
- adj/adj-hard:long (828): авьнъ `adj:long:acc:pl:m:an`: stored авьнꙑѩ vs rules авьнꙑихъ
- adj/adj-soft:short (765): боуи `adj:short:acc:du:m:an`: stored боуꙗ vs rules боуа
- noun/a-hard (644): багърѣница `noun:acc:du`: stored багърѣници vs rules багърѣницѣ
- noun/reviewed-twofold (612): балии `noun:ins:sg`: stored балиѥѭ vs rules балиеѭ
- noun/ja-soft (514): алъдии `noun:ins:sg`: stored алъдиѥѭ vs rules алъдиеѭ
- noun/jo-n-soft (401): алъкаиѥ `noun:dat:du`: stored алъкаиема vs rules алъкаиѥма
- noun/o-m-hard (354): агарѣнинъ `noun:dat:pl`: stored агарѣньмъ vs rules агарѣниномъ
- noun/jo-m-soft (338): агньць `noun:dat:sg`: stored агньцоу / агньцеви vs rules агньцоу

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
- Closed classes (pron/num/det `decl:*` cells) and the reviewed
unique/twofold noun and unique/irregular verb identities are
predicted through the Rust-encoded identity kernels in the rules
crate, mirroring the facade resolver's reviewed-profile dispatch.
The remaining unsupported closed-class cells are the ones the
facade itself serves from duplicated generated source tables (for
example the personal-pronoun table replicated under possessive
lemmas) with no kernel behind them.
- Comparisons are on canonical Cyrillic text only; romanization is
assumed to be regenerable by transliteration.
