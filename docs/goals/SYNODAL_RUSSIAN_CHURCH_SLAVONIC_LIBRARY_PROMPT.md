# Implement a Synodal Russian Church Slavonic Library

Create a new Rust library dedicated exclusively to Synodal Russian Church
Slavonic: the normalized liturgical language used in modern printed editions of
the Russian Orthodox tradition.

Treat Synodal Russian Church Slavonic as a distinct target standard, not as an
orthographic switch that relabels arbitrary Old Church Slavonic output. At the
same time, treat OCS as a major inherited linguistic evidence source and the
existing `old-church-slavonic` implementation as a candidate source of shared
recension-neutral machinery. Reuse is expected when it is linguistically
justified, explicitly mapped, provenance-preserving, and tested against Synodal
evidence.

Start by reading the complete APIs, architecture, reports, and tests in:

- `../church-slavonic`, especially the existing `old-church-slavonic` crates;
- `../ruthenian`, as an API-ergonomics reference;
- Unicode Technical Note #41:
  <https://www.unicode.org/notes/tn41/>; and
- the Ponomar Church Slavonic documentation:
  <https://sci.ponomar.net/>.

Backwards compatibility is not required. Preserve unrelated uncommitted work.

## Exact language boundary

Generate and analyze only Synodal Russian Church Slavonic as the target
recension. Every returned form must identify that target recension.

Do not silently substitute forms from:

- Serbian, Ukrainian, Bulgarian, Croatian, or other Church Slavonic recensions;
- Old Russian;
- modern Russian; or
- modern constructed Slavic or Slovowiki data.

Modern constructed Slavic standards and Slovowiki must never be used as
linguistic authorities for this project. Modern Russian may be used only as
explicitly labeled comparative or translation evidence, never as proof of a
Church Slavonic form.

Every source record must declare its source recension and every generated record
must declare its target recension. Reject or quarantine data whose recension
cannot be established.

Old Church Slavonic is a special case because Synodal Russian Church Slavonic
stands in historical continuity with the older Church Slavonic tradition and
shares substantial inherited vocabulary, morphology, and grammatical structure
with OCS. Use OCS lexical records, paradigms, principal parts, and rules as
inherited evidence when a documented and testable OCS-to-Synodal mapping exists.
Never present an OCS surface form as a directly attested Synodal form, and never
assume that a shared-looking lemma has unchanged spelling, accentuation,
morphology, meaning, or liturgical usage.

The governing rule is therefore:

> Ban unmarked cross-recension substitution, not inherited OCS evidence.

Direct Synodal evidence always outranks an inherited prediction when they
conflict. Preserve the conflict in provenance and evaluation reports.

## Initial research and specification

Before implementing morphology, produce a source inventory that identifies
separate authorities for:

1. normative morphology;
2. normative accentuation;
3. printed Synodal orthography;
4. abbreviations and nomina sacra;
5. lexical meanings and principal parts;
6. traditional Cyrillic numerals; and
7. corpus evaluation.

Prefer public-domain normative grammars, dictionaries, and complete Synodal
liturgical editions. Pin every digital source by URL, edition, date, checksum,
license, and transformation procedure.

Classify every source by both recension and one or more authority roles:

- lexical authority;
- grammatical authority;
- orthographic authority;
- accentual authority;
- exact-form evidence; or
- evaluation-only corpus.

Also classify the epistemic role of each use:

- Synodal normative authority;
- exact Synodal attestation;
- inherited OCS evidence;
- other-recension comparative evidence; or
- evaluation-only evidence.

Conflicting sources must remain visible. Do not erase disagreement by arbitrary
source priority.

Write a morphology and orthography specification before adding productive rules.
Start from the OCS grammatical inventory where useful, then document which
categories and formation rules are retained, transformed, restricted, expanded,
or absent in the Synodal standard. Do not discard inherited structure merely
because every Synodal cell is not directly attested, and do not copy it without
testing its Synodal behavior.

## Seed machine-readable source inventory

Use the following as the initial source backlog. This is a seed list, not a claim
that every source is authoritative, independent, clean, or legally suitable for
redistribution. Before ingesting a byte, recheck the current source, recension,
edition, license, file format, upstream lineage, and download mechanism. Pin an
immutable revision or release and calculate checksums locally.

Create `docs/SOURCES.md` and a machine-readable `data/SOURCES.toml`. For every
source record store at least:

```toml
id = "stable-project-local-id"
name = "human-readable source name"
url = "https://..."
retrieved_at = "YYYY-MM-DD"
revision = "commit, release, dump date, or page revision"
sha256 = "..."
source_recension = "old-church-slavonic | synodal-russian | mixed | unknown"
content_kind = "dictionary | paradigms | corpus | treebank | liturgical-text"
format = "jsonl | xml | conllu | text | mediawiki | sword | html"
license = "exact upstream statement"
redistribution = "allowed | noncommercial-only | evaluation-only | unresolved"
authority_roles = ["lexical", "morphological", "orthographic", "accentual"]
upstream_lineage = ["ids-of-sources-or-transcriptions-this-derives-from"]
normalization = "path to a reproducible transformation manifest"
```

Do not collapse source lineage. Two downloads derived from the same transcription
count as one witness, not two independent confirmations.

### OCS dictionaries and paradigm data

1. **The existing pinned English Wiktionary OCS extraction**

   - Machine-readable OCS dictionary:
     <https://kaikki.org/dictionary/Old%20Church%20Slavonic/index.html>
   - Raw Wiktextract downloads:
     <https://kaikki.org/dictionary/rawdata.html>
   - Extractor and format documentation:
     <https://github.com/tatuylonen/wiktextract>
   - Official English Wiktionary dumps:
     <https://dumps.wikimedia.org/enwiktionary/>
   - OCS inflection-template inventory:
     <https://en.wiktionary.org/wiki/Category:Old_Church_Slavonic_inflection-table_templates>

   Reuse the already pinned snapshot, checksums, extraction reports, and
   attribution in `../church-slavonic/data/SOURCES.toml`,
   `../church-slavonic/data/evaluation-sources.json`, and
   `../church-slavonic/ATTRIBUTION.md` before considering a refresh. Treat
   Wiktionary as structured dictionary and paradigm evidence, not manuscript
   attestation. Preserve page/template revision information where available.
   Kaikki is a Wiktextract view of Wiktionary and is not independent evidence from
   it. The postprocessed per-language Kaikki export may merge or disambiguate
   additional data; prefer the raw pinned Wiktextract stream when exact lineage
   matters.

2. **Anna Polivanova, _Old Church Slavic: Grammar and Dictionaries_**

   - Open-access publication and downloadable XML, CC BY 4.0:
     <https://books.fupress.com/catalogue/old-church-slavic/8465>
   - Searchable grammatical/root dictionary and downloadable source spreadsheet:
     <https://integral.github.io/osd/>

   This is a particularly valuable machine-readable grammar-and-lexicon source:
   use it to validate OCS classes, morphophonology, roots, and productive rules
   independently of Wiktionary. Pin both the scholarly XML and the dictionary
   spreadsheet. Document the relationship between the spreadsheet, web database,
   and published book so they are not counted as three independent sources.

3. **GORAZD Old Church Slavonic Digital Hub**

   - Czech Academy of Sciences resource directory:
     <https://www.slu.cas.cz/cs/on-line-slovniky-a-databaze>
   - Project description:
     <https://digitalhumanities.cz/en/db/gorazd-digital-portal-of-old-church-slavonic/>

   GORAZD exposes major scholarly OCS lexical databases and a digitized card
   index. Use it initially for manual alignment review, lemma validation,
   citations, Greek correspondences, and discovery of counterexamples. Do not
   scrape, bulk-copy, or redistribute it until its current export mechanism and
   database terms have been confirmed in writing and recorded in the manifest.

### OCS attested-text corpora

4. **Syntacticus native PROIEL/TOROT data**

   - Repository: <https://github.com/syntacticus/syntacticus-treebank-data>
   - TOROT project and license description:
     <https://torottreebank.github.io/>

   Prefer the native PROIEL XML when its richer annotation is needed. The current
   OCS project already pins a reproducible selection including Codex Marianus,
   Codex Suprasliensis, Euchologium Sinaiticum, the Kiev Missal, Psalterium
   Sinaiticum, and Codex Zographensis. Reuse those exact file pins first. The XML
   contains surface tokens, lemmas, morphology, syntax, document divisions, and
   source metadata. Treat it as attested OCS evidence and alignment material, not
   Synodal output.

   The data is CC BY-NC-SA 4.0. Keep it optional and local for evaluation unless
   the intended distribution is compatible with that license. Do not compile its
   text or reconstructive token listings into crates.io packages.

5. **Universal Dependencies Old Church Slavonic PROIEL**

   - Repository:
     <https://github.com/UniversalDependencies/UD_Old_Church_Slavonic-PROIEL>
   - Treebank documentation:
     <https://universaldependencies.org/treebanks/cu_proiel/index.html>

   The CoNLL-U files provide convenient token, lemma, part-of-speech,
   morphological-feature, and dependency data. They are a converted selection of
   PROIEL/TOROT, so never count UD and Syntacticus agreement as independent
   evidence. Use UD for interoperable evaluation and Syntacticus XML when original
   metadata or finer distinctions are needed. This treebank is also CC BY-NC-SA
   4.0 and must remain optional/evaluation-only for permissively published crates.

6. **Corpus Cyrillo-Methodianum Helsingiense (CCMH)**

   - Corpus description and canonical text inventory:
     <https://www.kielipankki.fi/download/ccmh-src/www/>
   - FIN-CLARIN/Kielipankki catalog record:
     <https://www.kielipankki.fi/aineistot/ccmh/>

   CCMH contains machine-readable canonical OCS texts including the Assemanianus,
   Marianus, Suprasliensis, Zographensis, and Savvina Kniga witnesses, plus later
   copies of OCS-period works. Its maintainers explicitly warn that encoding is
   simple and not every text has been fully checked. Use it for additional witness
   coverage and disagreement discovery. Record the access and license of each
   exact version separately; do not infer one license for the entire catalog.

7. **DIACU diachronic Church Slavonic dataset**

   - Dataset and code: <https://github.com/MariaCassese/DIACU>
   - Dataset paper: <https://aclanthology.org/2025.bsnlp-1.12/>

   DIACU contains machine-readable documents labeled across Old Church Slavonic,
   Church Slavonic, New Church Slavonic, and Ruthenian periods. Use it for
   recension/period classification experiments, contamination tests, and source
   discovery—not as a homogeneous OCS or Synodal corpus. The repository does not
   currently make a sufficiently explicit top-level data license obvious; keep it
   quarantined from redistribution and generated registries until every component
   source and its license have been audited.

### Synodal Russian Church Slavonic text

8. **Ponomar Elizabeth Bible Unicode text**

   - Repository: <https://github.com/typiconman/ponomar>
   - Elizabeth Bible text directory:
     <https://github.com/typiconman/ponomar/tree/master/Ponomar/languages/cu/bible/elis>
   - Example raw file:
     <https://raw.githubusercontent.com/typiconman/ponomar/master/Ponomar/languages/cu/bible/elis/Mt.text>

   This should be the first large Synodal target corpus. The repository contains
   Unicode plain-text books of the Elizabeth Bible with chapter/verse structure,
   accents, breathings, positional letters, abbreviations, pericope markers, and
   editorial material. Parse its lightweight markup explicitly: chapter markers,
   verse separators, bold markers, braces, notes, and lectionary labels must not
   become word tokens.

   The repository declares GPL-3.0-or-later for the project. Audit whether that
   declaration covers each text file and whether generated lexical facts can be
   distributed under the planned data license. Until resolved, keep raw text out
   of permissively licensed runtime crates; derived non-reconstructive metadata or
   optional local evaluation may have a different legal analysis that must be
   documented rather than assumed.

9. **CrossWire `CSlElizabeth` SWORD module**

   - Module metadata and download:
     <https://www.crosswire.org/sword/modules/ModInfo.jsp?modName=CSlElizabeth>
   - SWORD module-format tooling:
     <https://wiki.crosswire.org/DevTools:Modules>

   CrossWire describes this as the 1757 Church Slavonic Elizabeth Bible, language
   `cu`, distributed as public domain, and warns that the electronic edition uses
   modernized spelling. Extract it reproducibly through SWORD tooling rather than
   reverse-engineering opaque module files. Use it for expanded token coverage,
   verse alignment, and an orthographic contrast corpus; do not use its modernized
   spelling as exact Synodal liturgical-print evidence.

10. **Wikisource Church Slavonic Bible**

    - Root work: <https://wikisource.org/wiki/Бі́блїа>
    - Church Slavonic Wikisource landing page:
      <https://wikisource.org/wiki/Main_Page/Слове́нскїй>
    - MediaWiki API endpoint: <https://wikisource.org/w/api.php>

    This provides Unicode MediaWiki source with traditional letters, combining
    marks, accents, abbreviations, and book/chapter structure under CC BY-SA. Pin
    every page revision or a Wikimedia XML dump; do not scrape rendered HTML as
    the canonical input. Preserve templates and transclusion lineage in the raw
    layer, then produce a deterministic plain-text layer.

    Treat it as a community transcription requiring comparison against named
    printed editions. Do not assume that Ponomar, Wikisource, and CrossWire are
    independent simply because they are hosted separately; run verse-level
    lineage/fingerprint checks for common upstream transcriptions.

11. **Ponomar Library of liturgical texts**

    - Catalog: <https://www.ponomar.net/maktabah/index.html>
    - Site licensing terms: <https://www.ponomar.net/legal.html>

    The catalog exposes edition-labeled HTML for the Apostle, Gospel, Irmologion,
    General and monthly Menaia, Octoechos, Psalter, 1906 Service Book and Book of
    Needs (Trebnik), Typicon, Lenten and Paschal Triodia, Horologion, and other
    material. This is the best seed for moving beyond biblical vocabulary into
    actual liturgical genres. Capture the exact named edition and passage identity
    with every token.

    The site offers its own textual information under GFDL 1.2-or-later or CC BY-SA
    3.0, but several catalog entries identify modern printed editions. Audit rights
    and source lineage per work; the catalog-wide statement must not be used to
    erase a third-party edition's status. Keep material with unresolved rights as
    local evaluation input only.

12. **Russian National Corpus, historical Church Slavonic corpus**

    - Corpus portal and publications:
      <https://ruscorpora.ru/en/corpus/orthlib/publications>
    - Corpus tooling and frequency-dictionary description:
      <https://ruscorpora.ru/en/page/tool-corpus/>

    The RNC Church Slavonic corpus offers millions of words with searchable
    historical, morphological, syntactic, and metadata views plus a frequency
    dictionary. Use it for frequency-informed lexeme prioritization, targeted
    concordance checks, held-out queries, and discovery of genres absent from the
    Bible. It is not Synodal-only: require date, recension, edition, and genre
    filters, and manually validate any result promoted to target evidence. Do not
    automate bulk extraction or redistribute results unless the current RNC terms
    explicitly permit the intended use.

13. **D'yachenko's public-domain _Complete Church Slavonic Dictionary_**

    - Wikisource scan record:
      <https://ru.wikisource.org/wiki/Файл:Полный_церковнославянский_словарь_(Протоиерей_Г.Дьяченко).djvu>
    - Wikimedia Commons file category:
      <https://commons.wikimedia.org/wiki/Category:Full_Church_Slavic_dictionary>

    The 1900 dictionary is useful for Russian Church Slavonic vocabulary,
    definitions, citations, and game-oriented semantic discovery. Obtain the
    public-domain scan plus OCR/DjVuXML or proofread Wikisource pages where
    available, preserving page coordinates so entries can be reviewed against the
    image. Record the public-domain scan and any CC BY-SA Wikisource transcription
    as separate source layers with separate provenance.

    It intentionally includes older Church Slavonic and Old Russian material and
    is not a Synodal-only orthographic or morphological authority. Every extracted
    lemma must be linked to Synodal corpus evidence or explicitly classified as
    historical/comparative. Treat OCR as a candidate generator, never as an exact
    form table.

### Mandatory source-lineage and naming cautions

- ISO language code `cu` does not prove that a record is canonical OCS or Synodal
  Russian Church Slavonic. Inspect the work, witness, date, edition, and recension.
- The **Church Slavonic Elizabeth Bible** is not the **Russian Synodal Translation
  of 1876**, which is Russian. Reject modern-Russian Bible datasets even when their
  English title contains “Synodal Bible.”
- UD OCS PROIEL is derived from PROIEL/TOROT; Kaikki is derived from Wiktionary;
  web and spreadsheet views of one scholarly dictionary are one lineage.
- Ponomar, CrossWire, Wikisource, and other Elizabeth Bible editions may share
  upstream electronic transcriptions. Detect this before claiming independent
  agreement.
- A machine-readable source can supply candidates without being normative. Record
  separately whether it is allowed to support lexical identity, inflection class,
  exact surface attestation, accent, printed orthography, meaning, or only
  evaluation.
- Never combine source and evaluation partitions. If a passage supplies a lexeme,
  principal part, mapping, exception, or transformation rule, that passage cannot
  also count as held-out accuracy evidence for the same decision.

## OCS reuse and recension mapping

Audit the existing OCS code and data before implementing duplicate machinery.
For every reusable component, classify it as:

- recension-neutral and safe to share;
- parameterized by a documented recension profile;
- OCS-specific and requiring a Synodal counterpart; or
- unsuitable because it encodes unsupported assumptions.

Prefer a shared internal crate such as `historical-slavonic-core` for genuinely
common types, paradigm-cell definitions, Unicode primitives, rule tracing,
provenance, and algorithms. Keep recension-specific ending tables,
accentuation, orthographic realization, lexical registries, and exceptions in
their own modules or crates. Do not create a generic abstraction that conceals
real grammatical differences.

Create an explicit, reviewable lexeme-alignment registry rather than joining
records by normalized lemma text. At minimum model:

```rust
enum Recension {
    OldChurchSlavonic,
    SynodalRussian,
}

enum LexemeRelation {
    InheritedFrom,
    SameEtymon,
    BorrowedFrom,
    Uncertain,
}

enum MappingStatus {
    Reviewed,
    AutomaticallyValidated,
    Exploratory,
}

struct RecensionMapping {
    id: RecensionMappingId,
    source: LexemeId,
    target: LexemeId,
    relation: LexemeRelation,
    status: MappingStatus,
    confidence: Confidence,
    semantic_alignment: SemanticAlignment,
    morphology: MorphologyMapping,
    orthography: OrthographyMapping,
    evidence: Vec<EvidenceId>,
}
```

An identity transformation must still be explicit; equality of spelling is not
proof of identity. Track semantic drift independently from morphological
inheritance so a morphologically related lexeme cannot silently supply the wrong
game or dictionary meaning.

The productive inherited path should be observable as:

```text
OCS lexical analysis
  -> verified inherited morphological relation
  -> Synodal recension transformation
  -> Synodal accentuation
  -> Synodal printed orthography
  -> predicted Synodal form
```

Each stage must be independently testable and visible in the rule trace. A
failure or ambiguity at any stage must produce a typed result rather than a
guessed form.

## Workspace architecture

Create a workspace with clear boundaries such as:

- `historical-slavonic-core` or an equivalent internal shared crate
  - only functionality proven to be recension-neutral;
  - shared grammar cells, validated-text primitives, provenance, and rule traces;
  - no Synodal or OCS lexical registry;

- `synodal-church-slavonic-core`
  - pure Synodal morphology, accentuation, and orthography rules;
  - no bundled dictionary and no runtime I/O;
- `synodal-church-slavonic`
  - dictionary-backed ergonomic facade;
  - generated static registry;
  - typed paradigms and resolved handles;
- `synodal-church-slavonic-dictionary`
  - semantic lookup, reverse analysis, source examples, and vocabulary tools;
- `synodal-church-slavonic-extractor`
  - offline source ingestion, recension alignment, and validation; and
- `xtask`
  - deterministic regeneration, accuracy evaluation, guards, and packaging checks.

The shared crate is optional. Extract it only when the audit identifies a stable
shared boundary. Direct reuse of a small well-factored OCS module is preferable
to a premature universal Church Slavonic engine.

Alternative crate names are acceptable after checking crates.io availability, but
the public package name must make the Synodal recension unambiguous.

Runtime morphology must not read files, parse JSON/XML/TSV, or access the network.
Generate deterministic Rust data at build or maintenance time.

## Internal word representation

Separate the lexical word from its printed presentation.

Maintain at least:

1. raw source spelling;
2. expanded canonical lexical spelling;
3. canonical inflectional stem representation;
4. normalized lookup key;
5. accented expanded form; and
6. fully printed Synodal form with positional letters and abbreviations.

Each lexical or generated record must additionally preserve:

- source recension and target recension;
- stable source and target lexeme IDs;
- an optional inherited-from OCS lexeme ID;
- recension-mapping ID and transformation steps;
- semantic-alignment status;
- evidence IDs and authority roles; and
- whether the surface form is attested, normatively generated, inherited,
  analogical, or exploratory.

Do not use a printed abbreviation as the only lexical identity.

For example, abbreviation behavior for divine names must depend on lexical
identity and meaning, not a blind string replacement. Preserve the distinction
between a nomen sacrum and an identical sequence used with another meaning.

## Unicode and orthography

Implement Synodal orthography according to UTN #41 and pinned normative sources.

Support and test:

- standard Unicode Church Slavonic characters;
- canonical combining-mark ordering;
- acute, grave, kamora, breathing marks, and combined marks;
- titlo and pokrytie;
- superscript titlo letters;
- payerok, yerok, kavyka, and other required signs;
- initial, medial, and final positional letter variants;
- omega, broad on, uk, yat, decimal/dotted i, and other normative distinctions;
- traditional Cyrillic numeral notation;
- abbreviation expansion and contraction;
- capitalization and case behavior; and
- collation and normalized lookup.

Do not use font-specific private-use characters in stored text.

Do not assume ordinary NFC normalization is the complete contract. Define and
test the exact canonical sequence required for Church Slavonic combining marks.
Preserve original source text while exposing a separately normalized form.

Provide explicit presentation profiles:

```rust
OrthographyProfile::Expanded
OrthographyProfile::ExpandedAccentless
OrthographyProfile::SynodalLiturgical
```

`Expanded` should avoid abbreviations while retaining normative morphology.
`SynodalLiturgical` should apply accents, breathings, positional letter rules,
and approved abbreviations.

Any lossy transformation must report what was lost or changed.

## Typed morphology

Audit and model the full Synodal grammatical system with closed enums and typed
cells.

At minimum investigate and implement, where supported by normative sources:

- noun declension by case and number;
- adjective short/long forms and comparison;
- determiner and pronoun systems;
- cardinal and ordinal numerals;
- present, imperfect, and aorist verb systems;
- imperative;
- infinitive and supine if normatively represented;
- l-participles;
- active and passive participles;
- verbal nouns;
- irregular and suppletive paradigms;
- productive and lexical accentuation; and
- multiword future, perfect, pluperfect, conditional, and passive constructions.

Keep single-word morphology separate from phrase realization. Multiword tenses
must be represented as structured phrases, not strings pretending to be one word.

Verb metadata must keep independent principal parts for each system. Never infer
aorist formation from aspect or present class alone.

## Strict and predictive generation

Support explicit generation policies:

```rust
GenerationPolicy::Strict
GenerationPolicy::Productive
GenerationPolicy::Exploratory
```

Their behavior must be:

- `Strict`
  - exact Synodal source cells;
  - Synodal normative rules with independently sourced Synodal principal parts;
  - reviewed irregular overrides;
  - explicitly validated identity mappings where Synodal sources establish the
    lexeme and OCS merely supplies additional historical provenance;
  - otherwise fail explicitly;
- `Productive`
  - allow an inferred lexical analysis, including one seeded by aligned OCS
    metadata, only when an independently specified Synodal formation rule and
    recension mapping are uniquely compatible with all available evidence and
    pass a calibrated confidence threshold;
- `Exploratory`
  - return every compatible ranked analysis, including uncertain comparative
    OCS mappings, rather than forcing one class.

`Strict` must never treat an OCS surface form alone as Synodal attestation.
`Productive` may use inherited OCS evidence to recover a class, stem, or
principal-part candidate, but the output must be realized by Synodal rules and
labeled as a prediction. `Exploratory` must expose ambiguity instead of
collapsing several possible recension mappings.

`Productive` may accept only `Reviewed` or calibrated
`AutomaticallyValidated` mappings. `Exploratory` mappings must remain confined
to exploratory results. Confidence in a lexeme alignment and confidence in a
generated form are separate quantities and must not be conflated.

Never label a generated form as attested.

Every candidate must preserve:

```rust
Candidate {
    forms: FormSet,
    analysis: LexicalAnalysis,
    source_recension: Option<Recension>,
    target_recension: Recension,
    recension_mapping: Option<RecensionMappingId>,
    confidence: Confidence,
    assumptions: Vec<Assumption>,
    evidence: Vec<Evidence>,
    contradictions: Vec<Evidence>,
}
```

Distinguish at least:

- exact Synodal table;
- normative grammar rule with sourced principal parts;
- reviewed irregular override;
- inherited OCS lexical metadata;
- verified OCS-to-Synodal recension transformation;
- explicit identity transformation;
- explicit caller metadata;
- inferred lexical metadata;
- analogical prediction; and
- corpus observation.

Use a source classification such as:

```rust
enum FormSource {
    SynodalAttestation { evidence: EvidenceId },
    SynodalNormativeGeneration { rule: RuleId },
    InheritedPrediction {
        source_recension: Recension,
        mapping: RecensionMappingId,
        rule: RuleId,
    },
    AnalogicalPrediction { model: ModelId },
}
```

## Public API

Use a discoverable lemma-plus-grammar API inspired by the current
`old-church-slavonic` facade:

```rust
noun(...)
long_adjective(...)
short_adjective(...)
present(...)
imperfect(...)
aorist(...)
imperative(...)
infinitive(...)
l_participle(...)
```

Also provide:

```rust
let inflector = Inflector::builder()
    .generation_policy(GenerationPolicy::Productive)
    .orthography(OrthographyProfile::SynodalLiturgical)
    .build();

let verb = Verb::resolve("благословити")?;
let forms = verb.present(Person::First, Number::Singular)?;
```

Successful word generation must return a nonempty structured `FormSet`, not a bare
string. Preserve:

- all variants;
- accentuation;
- expanded and printed spellings;
- romanization if available;
- provenance;
- warnings;
- assumptions;
- rule traces; and
- competing analyses.

Make inherited provenance inspectable without requiring consumers to parse a
debug string:

```rust
forms.source_recension()
forms.target_recension()
forms.recension_mapping()
forms.provenance()
forms.rule_trace()
```

When several variants have different provenance, expose these properties per
variant rather than reporting a misleading aggregate.

Provide explicit variant selection:

```rust
forms.primary_text()
forms.unique_text()?
forms.variants()
forms.select(VariantPolicy::NormativeFirst)
```

Typed errors must distinguish:

- invalid Unicode or orthography;
- unknown lemma;
- ambiguous lexeme;
- missing principal part;
- contradictory metadata;
- unsupported formation;
- missing or ambiguous recension mapping;
- semantic alignment not established;
- inherited evidence contradicted by Synodal evidence;
- historically invalid cell;
- unsupported but conceptually possible cell; and
- orthographic transformation requiring unavailable semantic metadata.

## Resolved identities and paradigms

Provide resolved handles for nouns, adjectives, verbs, pronouns, determiners, and
numerals, with:

```rust
resolve(lemma)
from_id(id)
id()
lemma()
capabilities()
paradigm()
```

Paradigms must retain every requested row, including failures and predictions:

```rust
paradigm.form(...)
paradigm.iter()
paradigm.attested()
paradigm.predicted()
paradigm.failures()
paradigm.into_rows()
```

Every paradigm row should expose one of:

- `Attested`;
- `SourcedPrediction`;
- `InferredPrediction`;
- `AmbiguousPrediction`;
- `HistoricallyInvalid`; or
- `Unsupported`.

All direct calls, handles, stable-ID calls, and paradigms must delegate to one
canonical cell resolver.

## Higher-level utilities

Implement or prepare typed APIs for:

1. morphological analysis and lemmatization;
2. reverse lookup from printed and expanded forms;
3. abbreviation expansion and contraction;
4. traditional Cyrillic numeral parsing and formatting;
5. explicit transliteration with loss reporting;
6. per-lexeme capability and missing-metadata inspection;
7. source attestation and concordance lookup;
8. phrase realization for agreement, government, and analytic verb forms;
9. rendered-text validation;
10. vocabulary-manifest linting for games and applications; and
11. serde/JSON and WASM-friendly feature configurations.

The analyzer must return every compatible typed grammatical analysis rather than
a single guessed lemma.

## Extraction and generated data

Never edit generated Rust registries manually.

The extractor must:

- stream large inputs;
- preserve source spelling and ordering;
- reject malformed or recension-unknown rows from target registries while
  retaining them in a quarantine report;
- retain every parse failure with counted reasons;
- have a strict parse-failure ceiling;
- derive stable content-based lexical identities;
- preserve homographs and competing analyses;
- atomically replace all generated outputs; and
- produce reviewable normalized TSV/JSON reports before Rust generation.

Keep OCS source records and Synodal target records in separate registries, plus
an explicit alignment registry connecting them. Never copy an OCS surface row
into the Synodal exact-form table. Store inherited or transformed rows separately
from attested rows, including the mapping, transformations, confidence, and all
supporting and contradicting evidence.

The alignment pipeline must:

- propose candidates from stable IDs, spelling, morphology, etymology, and
  meaning without accepting string similarity as proof;
- require reviewable evidence for accepted mappings;
- record one-to-many, many-to-one, homographic, and uncertain mappings;
- validate morphology and semantic compatibility separately;
- make identity mappings explicit;
- preserve rejected candidates and reasons; and
- regenerate deterministically.

Maintain independent registries for:

- lexical entries;
- exact forms;
- principal parts;
- accent metadata;
- abbreviation rules;
- orthographic positional rules;
- irregular overrides; and
- semantic senses and examples.

Also maintain registries for:

- OCS-to-Synodal lexeme alignments;
- recension transformation rules;
- semantic-alignment decisions; and
- inherited-evidence conflicts and reviewed resolutions.

## Real-world evaluation

Do not evaluate only by reproducing generated dictionary rows.

Add:

1. exact registry round-trip tests;
2. masked-cell completion tests;
3. lemma-disjoint class-inference tests;
4. independent liturgical-corpus evaluation;
5. accentuation evaluation;
6. expanded-to-printed orthography tests;
7. abbreviation round-trip tests where the transformation is reversible;
8. traditional numeral tests;
9. hostile Unicode and malformed combining-mark tests; and
10. cross-recension contamination tests.

Also add dedicated OCS-inheritance evaluation:

11. a reviewed gold set of aligned OCS and Synodal lexemes;
12. identity-versus-transformed mapping tests;
13. morphology-mapping accuracy by declension and conjugation system;
14. semantic-drift and false-friend tests;
15. leave-one-Synodal-lexeme-out tests that infer through OCS evidence without
    leaking the target's exact-form rows; and
16. conflict tests proving that exact Synodal evidence defeats an OCS-derived
    prediction.

Report separately:

- returned-form coverage;
- exact printed agreement;
- expanded spelling agreement;
- accent agreement;
- top-1 accuracy;
- top-k accuracy;
- abstention;
- calibrated confidence;
- regular versus irregular lexemes; and
- attested versus predicted paths.

For inherited generation, report separately:

- number and percentage of Synodal lexemes aligned to OCS;
- mapping precision on the reviewed gold set;
- extra returned-form coverage attributable to inherited OCS evidence;
- exact-form precision of that added coverage;
- results by identity mapping versus transformed mapping;
- results by morphological system and confidence band; and
- abstention caused by ambiguous, unsupported, or semantically unsafe mappings.

When evaluating an OCS-derived prediction for a Synodal lexeme, exclude that
target lexeme's exact Synodal form and any derivative row from the inference
inputs. Evaluation data may score the result but must not leak into it.

A higher-coverage policy must not be allowed to hide falling precision.

Pin real passages from normative Synodal editions as golden integration fixtures.
Include nouns, adjectives, pronouns, numerals, irregular verbs, participles,
analytic tenses, abbreviations, nomina sacra, accents, and Cyrillic numerals.

## Required guards

Add structural guards proving:

- every source and target record declares its recension;
- every inherited OCS dependency retains a stable source ID and provenance;
- every OCS-derived Synodal output passes through an explicit mapping and a
  reviewed Synodal realization rule;
- mapping status prevents exploratory alignments from entering Productive or
  Strict output;
- no OCS surface form is labeled exact Synodal attestation;
- exact Synodal evidence outranks inherited or analogical predictions;
- other-recension surface records cannot enter Synodal exact-form registries;
- modern constructed Slavic standards and Slovowiki cannot enter linguistic
  source manifests or generated registries;
- generated forms are never labeled attested;
- runtime crates perform no file or network access;
- every public raw string passes through validated orthography;
- all grammar `ALL` inventories are exhaustive;
- paradigms call the canonical cell resolver;
- source variants and competing analyses remain lossless;
- private-use Unicode characters cannot enter generated data;
- abbreviation rules require the necessary lexical/semantic identity;
- reports and generated artifacts are current; and
- source attribution and licenses are included in packages.

## Suggested implementation order

1. Audit the OCS crates, rules, data, and tests; classify each component as
   shared, parameterizable, OCS-specific, or unsuitable.
2. Pin Synodal and OCS sources and write the recension, morphology, accent,
   orthography, and evidence-authority specifications.
3. Establish stable source/target lexeme identities and build a small reviewed
   OCS-to-Synodal alignment gold set.
4. Implement shared validated Unicode, grammar-cell, provenance, and rule-trace
   primitives without erasing recension distinctions.
5. Implement Synodal expanded lexical spelling, printed orthography, accents,
   positional letters, and explicit OCS-to-Synodal transformations.
6. Implement nouns and adjectives with exact Synodal golden paradigms plus
   aligned OCS inheritance tests.
7. Implement pronouns, determiners, and numerals.
8. Implement verb principal-part structures, productive systems, participles,
   and irregular records.
9. Add abbreviations, nomina sacra, and Cyrillic numerals.
10. Add dictionary-backed stable identities and the reviewed alignment registry.
11. Add Strict/Productive/Exploratory policies, including inherited OCS
    predictions with calibrated confidence.
12. Add typed reverse analysis, text checking, and semantic vocabulary tools.
13. Add structured phrase realization for analytic constructions.
14. Run leakage-resistant Synodal corpus and OCS-inheritance evaluation, then
    close the largest measured failures without lowering precision silently.

## Verification

Run at minimum:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask check-all
cargo xtask guard-witnesses
cargo publish --dry-run -p synodal-church-slavonic-core --allow-dirty
cargo publish --dry-run -p synodal-church-slavonic --allow-dirty
```

Also run the independent corpus, accentuation, abbreviation, and orthography
evaluation commands established by the project.

Do not commit, push, tag, publish, or deploy unless separately requested.

## Deliverables

Report:

- the exact Synodal recension boundary;
- which OCS implementation components were reused, shared, parameterized,
  rewritten, or rejected, with reasons;
- every source, edition, checksum, license, and authority role;
- aligned OCS/Synodal lexeme counts, mapping kinds, unresolved mappings, and
  semantic conflicts;
- the implemented grammatical inventory;
- the canonical lexical and printed orthography contracts;
- supported abbreviation and accent behavior;
- the public API;
- attested and predictive coverage separately;
- coverage and precision added specifically by inherited OCS evidence;
- masked-cell and real-text evaluation results;
- all remaining unsupported formations and irregular systems;
- package contents and runtime boundaries; and
- verification commands and results.

The resulting library must favor explicit uncertainty over false certainty while
making productive, normatively possible Synodal forms available under an opt-in
prediction policy. It should exploit the historical and structural value of OCS
aggressively where evidence supports reuse, while making it impossible for
consumers to confuse inherited predictions with Synodal attestations.
