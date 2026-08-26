# church-slavonic

**church-slavonic** is a rule-first Old Church Slavonic inflection library:
a pure rule kernel plus compact generated residue tables holding only the
attested cells the rules do not reproduce verbatim. Total generated data is
**964 KB** (four sorted-slice tables) versus the **24 MB** compiled registry
of the crates it replaces. It reproduces **100% of the attested oracle for
every part of speech** in `data/extracted`.

> **Pilot status.** This crate is the pilot slice of the workspace rewrite
> plan (`docs/REWRITE_PLAN.md`, phases 3–4). It serves every attested POS at
> 100%, but the dictionary/analyze layer, phrase-level constructions, and the
> synodal recension are not yet ported, and the API may still move before the
> deprecation release. See `docs/DEPRECATION_MAP.md` for how the old crates'
> surface maps onto this one.

## Accuracy

`cargo xtask rewrite-pilot-accuracy` replays every attested cell in
`data/extracted` against the facade and requires 100% per POS:

| Part of speech | Attested cells | Accuracy |
|----------------|---------------:|----------|
| Nouns          | 41,566         | 100%     |
| Adjectives     | 39,204         | 100%     |
| Verbs          | 13,260         | 100%     |
| Pronouns       | 1,341          | 100%     |
| Numerals       | 126            | 100%     |
| Determiners    | 36             | 100%     |

Resolution precedence for a requested cell is exactly one channel deep:
residue table first, rule kernel second. The residue tables hold only what
the rules cannot derive — for nouns the kernel derives the great majority of
attested cells (~85% of the whole registry derives exactly); for verbs,
principal-part metadata synthesis keeps 473 of 707 verbs fully rules-backed,
leaving a residue of about 2,100 cells. Because unseen lemmas run through the
same rules, the library inflects words it has never stored.

Unknown lemmas return `Error::UnknownLemma`; cells the metadata cannot commit
to return `Error::Underdetermined` (attested cells never do). No empty-string
holes, ever.

## API

All functions return `Result<String, Error>` (the primary form) and each has
a `*_variants` companion returning `Result<Vec<String>, Error>` with every
attested spelling, primary first. Grammar enums (`Case`, `Number`, `Gender`,
`Person`) are re-exported from `church-slavonic-core`.

### Nouns

```rust
use church_slavonic::{noun, noun_variants, Case, Number};

assert!(noun("градъ", Case::Genitive, Number::Singular).is_ok()); // града
let all = noun_variants("градъ", Case::Locative, Number::Singular)?; // all attested spellings
```

### Adjectives

Long (definite) and short (indefinite) declensions are separate functions —
a paradigm-selecting distinction becomes a function, not an enum parameter:

```rust
use church_slavonic::{adjective, short_adjective, Case, Number, Gender};

adjective("новъ", Case::Nominative, Number::Singular, Gender::Masculine)?;      // новꙑи
short_adjective("новъ", Case::Nominative, Number::Singular, Gender::Masculine)?; // новъ
```

### Verbs

One function per finite tense (`Person` and `Number` index within it), plus
citation-only forms and four participle derivations:

```rust
use church_slavonic::*;

present("нести", Person::Third, Number::Singular)?;   // несетъ
aorist("нести", Person::Third, Number::Plural)?;
imperfect("нести", Person::First, Number::Singular)?;
imperative("нести", Person::Second, Number::Singular)?;
l_participle("нести", Gender::Feminine, Number::Singular)?;
infinitive("нести")?;
supine("нести")?;
verbal_noun("нести")?;                                 // nominative-singular citation
present_active_participle("нести")?;                   // citation (masc nom sg, short)
present_passive_participle("нести")?;
past_active_participle("нести")?;
past_passive_participle("нести")?;
```

### Closed classes (pronouns, numerals, determiners)

The attested closed inventories (29 pronoun lexemes, 8 numerals, 1
determiner) come in three key shapes, and the API follows them honestly:

```rust
use church_slavonic::*;

// Person-indexed personal pronouns (Person::Third returns Underdetermined —
// the third person is the gendered anaphoric series):
pronoun(Person::First, Number::Singular, Case::Dative)?;   // мьнѣ
anaphoric(Case::Accusative, Number::Singular, Gender::Feminine)?;
reflexive(Case::Accusative)?;                              // сѧ (numberless)

// Gender-indexed lexical cells (gender is a key dimension where the lemma
// draws the distinction, and ignored where it does not):
pronoun_form("иже", Case::Nominative, Number::Singular, Gender::Masculine)?;
numeral_form("пѧть", Case::Genitive, Number::Singular, Gender::Feminine)?;
determiner_form("кꙑи", Case::Nominative, Number::Singular, Gender::Neuter)?;
```

### Value-driven numerals

One function per construction, range 1–10,000 (the evidential myriad
boundary), replacing the old crate's twelve compound/distributive variants:

```rust
numeral(123, Case::Nominative, Gender::Masculine, Animacy::Inanimate)?;
distributive_numeral(5, Gender::Feminine, Animacy::Inanimate)?; // по + dative
numeral_variants(33, Case::Nominative, Gender::Masculine, Animacy::Inanimate)?;
```

Both are gated differentially against the old machinery (cardinals
2,919/2,919 sweep cells, distributives 417/417).

### Paradigm enumeration

Every lexeme's full table through the same single-cell resolution path,
self-consistency-gated (a paradigm contains exactly the cells the
single-cell API serves):

```rust
noun_paradigm("градъ")?;                       // Vec<(Case, Number, Vec<String>)>
adjective_paradigm("новъ", AdjectiveForm::Long)?;
verb_paradigm("нести")?;                       // Vec<(VerbCellKind, Vec<String>)>
```

Defective cells are honestly absent: pluralia tantum list seven plural
cells, singular-only proper names seven singular ones, and masculines with
no animacy fact omit the animacy-contrastive accusatives.

### Errors

```rust
pub enum Error {
    UnknownLemma(String),             // the facade knows nothing about this lemma
    Underdetermined { lemma: String }, // known lemma, but the metadata cannot
                                       // determine this cell
    ValueOutOfRange { value: u64 },    // numeral outside 1..=10_000
}
```

## Homographs: deterministic numeric suffixes

Lemmas with more than one lexeme entry (ten noun lemmas such as `градъ`,
`ногъть`, `сꙑнъ`; four verb pairs such as `вести`, `пасти`) follow the
`gold-silver-copper/english` scheme: the bare lemma serves the default sense
and the others are reachable as `lemma_2`, `lemma_3`, … Sense numbering is a
pure deterministic sort of each lexeme's emitted form inventory (tie-broken
by encoded metadata), so it needs no external lockfile and is reproducible
from the data alone across refreshes. A bare-lemma lookup answers with only
that lexeme's own variants, never a union across senses.

## Documented design decisions

- **Animacy is not a parameter.** For every attested adjective cell the
  animate and inanimate stored variant lists are byte-identical (the tables
  keep the plain accusative), so an `Animacy` parameter would promise a
  distinction the data does not make; the residue generator fails if this
  ever stops holding. Noun masculine accusatives where the metadata carries
  no animacy fact ship in the residue table verbatim.
- **Positive degree only.** The extracted inventory stores comparative
  *citations* only, and those carry unpredictable lexical facts (suppletive
  `велии` → `болии`, old vs new suffix grade `дражии` / `дражаи`), not a
  productive stem. Comparatives are excluded from these functions and from
  the accuracy denominator; the gate prints the excluded count.
- **The oracle decides.** Any cell where a kernel convention would diverge
  from the stored tables ships in the residue verbatim; the accuracy gate
  keeps the facade at 100% either way.

## Rules vs residue

| Table                    | Size    |
|--------------------------|--------:|
| `noun_residue.rs`        | 556 KB  |
| `verb_residue.rs`        | 322 KB  |
| `adjective_residue.rs`   | 74 KB   |
| `closed_residue.rs`      | 23 KB   |
| **Total**                | **964 KB** |

`cargo xtask check-structure` gates on the six per-POS oracles and the ≤2 MB
facade data budget.
