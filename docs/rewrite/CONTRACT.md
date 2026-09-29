# Rewrite contract and evidence record

Status: implementation in progress. This record was started during execution of
V5-PROMPT.md on 2026-09-06. It is an evidence-backed design proposal, not a new
source of instructions. The complete objective remains the five phases and
completion conditions of that prompt. `data/rewrite/requirements.json` retains
every requirement block; pending entries have not been satisfied by this record.

## Starting state and preservation

Audited revision: `09d14f3efc41d20503dd46d9391ca17a15a791c3`. The only initial
working-tree change was untracked V5-PROMPT.md. No commit, push, publication, or
external repository edit is authorized by the implementation task.

`data/rewrite/baseline.json` identifies all tracked input bytes. A local archive
of code, lexicon, data, scripts, and references exists under ignored
`target/rewrite/baseline-<revision>.tar`; the Git revision also preserves those
tracked inputs. Downloaded inputs are not all tracked: inventory and independent
source reacquisition are recorded separately. Neither a source label nor a hash
establishes authority, annotation quality, or redistribution permission.

Baseline verification is from the current source in a separate target directory,
not a previously built executable. Commands and results are recorded in
`data/rewrite/current.json`. The historical audit probes are stored as input
and output records; they are not acceptance tests for the rewritten behavior.

## Target systems and boundaries

The first grammar profiles are **OCS pedagogical morphology** and **Russian
Synodal morphology**. These are explicit implementation scopes, not a claim
that all witnesses within a historical period agree. Each profile must eventually
carry an inventory revision and bibliographic support. Orthographic realization
has its own profile and version, and witnesses retain their own identity and
unchanged text. Unsupported grammar/orthography combinations must be rejected
rather than silently projected. A display spelling is not a lexeme identifier.

The existing class and stress engine can supply migration candidates. Its output
is marked inherited/unverified until checked; its internal consistency supplies
engineering evidence only. Full migration, edition-specific defaults, and
independently reviewed paradigm coverage remain required later work.

### Feature audit from executable schema and loaders

| Area | Executable starting state | Required interpretation and disposition |
|---|---|---|
| Nominals | Seven Case values, sg/du/pl, three genders; animacy optional boolean on lexeme | Represent restrictions per lexeme/cell; unknown animacy is not inanimate. Inventory represented, licensing not yet audited throughout. |
| Adjectives | Short/long and degrees; adverb is an adjective Cell | Keep the productive mechanism but add lexical derivation identity; lexicalized meanings/preferences are not inflectional facts. |
| Finite verbs | Present, imperfect, aorist, future; separate imperative enum variant | Mood/aspect mapping must be explicit. Existing `Tense` and `FiniteTense` disagree about future; consolidate during migration. |
| Nonfinite verbs | Infinitive, l-participle, present/past active/passive short/long participles | The supine and conditional auxiliary forms are representation gaps. Periphrastic constructions belong to a composition layer, not a fabricated single-word cell. |
| Corpus verbal labels | UD loader excludes Sup and Sub; maps Past+Aspect=Imp to imperfect | Report exclusions and raw tags; tense/aspect convention is an adapter decision, not universally correct linguistic equivalence. |
| Pronouns | Typed optional person/gender/number and clitic bit; Word for closed classes | Separate lexical category from contextual determiner/pronoun use. Preserve closed-class lexical ambiguity. |
| Numerals | Represented through noun/adjective/pronoun/closed classes | Category and construction support incomplete; productive compounds require explicit licensing and independent examples. |
| Stems | Class derivations plus explicit numbered/base stems | Principal parts can be lexically required without being suppletive. Do not infer them from a citation ending with certainty. |
| Accent | A vowel index, number-mark flags, and None | Separate observed mark, predicted position, unknown stress, and an explicitly unaccented editorial policy. None does not describe historical accentlessness. |
| Abbreviation | Global generated prefix-pattern table; titlo index | Expansion candidates need witness/profile evidence and must not silently erase lexical alternatives. |
| Text units | Whitespace split, regenerated separators, verses/paragraphs | Preserve raw source and spans. Units do not establish clauses. Classification must not delete tokens from denominators. |

Missing categories are not licensed by returning empty output without a reason.
The first slice may return explicit unsupported/unknown results. The completed
migration must state its full supported inventory and account for exclusions.

## Independent linguistic evidence

Fresh retrieval records (including failures) are in `data/rewrite/sources.json`.
Raw downloads stay in ignored `target/rewrite/upstream`; this record does not
authorize redistributing an edition. Bibliographic propositions below were
checked against externally retrieved pages, not repository transcriptions.

- Alypy, [§153, address](https://www.ponomar.net/files/gama2/p153.htm): address
  occurs without an adjacent imperative/interjection, and some forms used in
  address coincide with nominative forms. This contradicts a universal rule
  removing vocatives merely because no such neighbor was found.
- Alypy, [§197, dative absolute](https://www.ponomar.net/files/gama2/p197.htm):
  the construction has predication expressed through a dative nominal and
  participial form, including documented variations. Agreement alone cannot
  identify an attributive relation.
- Krause and Slocum, University of Texas LRC,
  [OCS lesson 1, §3.1](https://lrc.la.utexas.edu/eieol/ocsol/10): a hard-stem
  noun paradigm with distinct singular/dual/plural cells supplies an independent
  test of the nominal engine. Use the source's spelling and mark explicit
  representation conversions, rather than broad matching to pass the fixture.
- LRC [lesson 8, §38](https://lrc.la.utexas.edu/eieol/ocsol/80): locatives
  without prepositions include fossilized temporal expressions and verbal
  complements. This contradicts applying bare-loc as a general OCS exclusion.
- LRC [lesson 10, §46.1](https://lrc.la.utexas.edu/eieol/ocsol/100): infinitive
  and supine are distinct forms; morphological changes affect their realization.
  Add schema support before counting their absence as merely a lexical gap.
- [UD OCS PROIEL](https://universaldependencies.org/treebanks/cu_proiel/index.html)
  documents multiple witnesses and automatic conversion from PROIEL/TOROT.
  Its feature inventory includes a supine. It is an annotation source whose
  conversion conventions must be retained and audited, not a Synodal gold set.

These checks do not authenticate every existing paradigm, stress class, source
license assertion, or annotation. They do not constitute independent expert
review of newly annotated sentences. That validation remains explicitly open.

## Audit of contextual rules

All seven rules currently mutate candidate sets. None of their local search
procedures proves a unique attachment under an incomplete lexicon. They must
be reassessed as hypotheses in a separate contextual layer.

| Rule | Executable premise | Disposition for rewritten defaults |
|---|---|---|
| prep-gov | Nominal immediately after recognized preposition, optionally one more after adjective | Government requires an established attachment and reviewed frame. Adjacency is a preference only. |
| np-agree | Adjacent adjective-like/noun cells with intersecting agreement | Agreement is compatibility; another unsearched head or predicative use remains possible. Do not assert attachment from this alone. |
| subj-verb | Adjacent nominative noun and finite verb | Nominative can serve functions other than subject. Require attachment evidence before excluding person/number alternatives. |
| voc-drop | No immediately adjacent imperative/interjection | Unsupported as a hard exclusion; contrary to independent address evidence. |
| bare-voc | Same adjacency premise applied across lexemes | Same problem; a competing lexeme does not license discarding a possible address. |
| bare-loc | No locally found prepositional governor, with start-of-unit guard | Invalid as a general OCS constraint; profile and construction evidence required. |
| one-subject | Unique locally definite nominative in a transitive clause, heuristic apposition guards | Depends on unproved segmentation, attachment, argument structure, and apposition decisions. Report proposed constraints, not established exclusions. |

A rule that would remove every candidate currently does nothing. This guard
prevents empty output but does not validate partial elimination. Phrase and
clause identity must not depend on treebank verse boundaries or literal lemma
IDs. Rule order and arbitrary first-cell propagation require explicit tests.

## Evaluation specification

1. **Text preservation:** compare original UTF-8 bytes, including separators,
   case, normalization, and markup according to the declared artifact boundary.
2. **Generation:** compare requested licensed cell and chosen profile, with
   separate letters/accent and exact-surface results. Independently sourced
   examples establish correctness; generated round trips establish consistency.
3. **Recognition:** count all eligible tokens under each explicit matching
   policy. Report known and unknown lexemes separately; never give the unknown
   analyzer the gold lemma as an input.
4. **Identity:** distinguish POS/cell containment, lexical identity, and joint
   identity+cell. Ambiguous annotations retain sets; order must not pick a gold
   answer. Every excluded/unmatched/misaligned token remains in accounting.
5. **Context:** deploy with available/predicted context only. Gold neighbor
   lemmas and previous gold labels are forbidden inputs to deployment scoring.
   Oracle-context results are diagnostics with a different name and denominator.
6. **Uncertainty:** report candidate recall, excess candidates assessed by an
   appropriate independent sample, abstention, and chosen-result accuracy.
   A larger correct candidate set is not a regression in correctness.
7. **Controls:** fabricated lemmas, deleted gold candidates, wrong cells, empty
   sources, and mismatched tokenization must affect the appropriate measures.
   A conditional score may be reported in addition to an end-to-end score.
8. **Data lineage:** split by witness/document, record source overlap and reused
   passages, and preserve unconverted labels. Existing repeatedly consulted
   splits are regression material. New blind scores require a genuine held-out
   population and independent annotations; no such score is claimed yet.

The current recall function measures gold-lemma-conditioned generation, can
invoke a guesser with that lemma, and uses broad folds and accepted-cell
remappings. The explicitly named oracle_examples omit tokens without a gold-compatible
feature candidate and use oracle neighbor information; deployment evaluation
now uses the separate path documented below. The overlay scorer has
separate omissions and mixed-stage attribution. Their historical numbers will
not be used as acceptance floors for a different metric definition.

## First implementation slice

Use immutable snapshots of validated lexemes, separate matching policy and
match explanation, and immutable witness bytes with spans. Keep original
analysis candidates separate from contextual conclusions. Introduce form-level
evidence records and explicit cell availability without claiming all inherited
lexemes are attested. Integrate these APIs into consumers during migration;
a parallel unused API alone does not complete the rewrite.

Required first-slice tests cover OCS hard-stem noun cells independently checked above, a
Synodal stress/orthography exemplar with independently verified cells, an
irregular verb with explicit principal parts, a derived adverb relation,
abbreviation, competing closed lexemes, unknown cells, and malformed inputs.
The full vertical-slice gate remains open until all these fixtures work through
source evidence, generation, rendering, and analysis.

## Verified engineering increment — 2026-09-06

Explicit comparison policies now have lazy indices derived from each policy's
own key function. A direct-comparison fixture checks retrieval completeness as
well as match soundness. These indices still expand the inherited paradigms;
startup/memory benchmarking and a validated consumer artifact remain open.
The original `analyze` remains a legacy-key retrieval API during migration.

Lexicon construction now has a structured-error entry point and the snapshot's
recension cannot be mutated. Existing embedded construction uses the same
checks. The legacy convenience constructor still panics on invalid input;
standalone mutable Lexeme values and their inflection methods are not yet the
completed validated domain model. Empty/0 class values remain explicit legacy
unknowns and do not license a paradigm.

A separate diff inspection covered changed rendering paths, index identity,
construction checks, witness boundaries, and binary validation. It confirmed
that several goal requirements remain unsatisfied: closed alternatives are
still serialized as a surface/count, token rendering can still swallow errors,
abbreviation and closed-word tables remain global, and linguistic profiles
still share the legacy Recension enum. No full-rewrite completion is claimed.

The first bundled-data check exposed an overstrict new validation rule:
repeated variant groups are additive in the actual generator, unlike singleton
overrides. The validator was corrected rather than rewriting source data to
satisfy that mistaken rule. The rejected suspicion is recorded in the ledger.

`data/rewrite/ocs-hard-noun.json` preserves 21 expected cells from the separately
retrieved LRC pedagogical paradigm with exact source spellings. It is connected
to the runtime model and generation/analysis checker. This establishes table
reproduction, not held-out generalization. Check results and remaining work are
recorded in `data/rewrite/current.json`.

## Source-candidate records and the first new-model consumer

`AnalysisDocument` retains a partition of the source bytes and independent,
fully specified morphological candidates under an explicit matching policy.
`Sentence::analysis_document` uses the original witness even after legacy tree
edits. `xtask analyze` now exports these records, including closed alternatives,
Unicode transformations and source addresses when supplied. Reload recomputes
against the caller's lexicon and rejects changes to candidates or source spans.
The modeled backend binds source, model data, and selected implementation source
hashes and rejects changed snapshots even when the resulting forms are unchanged.
The legacy backend lacks this model binding. The implementation fingerprint
is not yet a complete build/dependency identity; full artifact provenance remains
part of migration work.

The segmentation is named `whitespace-punctuation-v1`. Segmentation itself
does not interpret clitics, abbreviations, syntactic words or apparatus. The legacy tree and
its corpus writers have not all migrated to this representation. JSON records
are limited to 16 MiB and their source text to 128 KiB; the modeled backend bounds document work and retained candidate strings. These
logical limits are not measured RSS bounds, and the legacy index needs broader
resource controls.

The new `morphology::Model` loads declarative stems, cell rules, lexical relations,
source claims and separate grammar/spelling profiles through a validated runtime
artifact. Rule IDs survive input reordering. Missing stems remain visible,
including when another alternative can be generated. Its initial expression
language is explicit-stem plus suffix concatenation, followed by declared
spelling replacements; stress prediction and historical alternations are not
implicitly inferred.

The first artifact is `data/rewrite/ocs-hard-noun-model.json`. Its independent
expected surfaces remain in `data/rewrite/ocs-hard-noun.json`. Run:

```sh
RUSTC_WRAPPER= cargo run --offline --release -p church-slavonic-tools --bin xtask -- check-linguistic-slice data/rewrite/ocs-hard-noun-model.json data/rewrite/ocs-hard-noun.json l-000001 o-lrc-cyrillic
```

This checks conditional generation and joint candidate retention for 21 cells
of one externally sourced pedagogical paradigm. It is not unknown-word accuracy,
contextual accuracy, or attestation of each cell in a manuscript. The model
fixture is newly specified, not a completed migration of either old градъ entry.
Synthetic restrictions and spelling changes used in API tests are explicitly
constructed test claims, not assertions attributed to the source grammar.


The separate review of the new model found that unverified restrictions could
suppress generated candidates. The corrected result is `UnresolvedRestriction`,
which preserves the otherwise generated forms and exposes the unresolved claim
to analysis. A dedicated regression checks this behavior. Model validation checks
references and structural consistency; it does not authenticate a caller's
review labels or prove a source claim true. External source checking remains a
separate obligation, and the fixture records do not claim expert review.


## Generic fixtures and supplied irregular forms

Fixture schema 2 uses a POS tag and cell name for each expected surface.
`check-linguistic-slice` reports concrete generation and joint candidate retention
with the complete expected-row denominator. Missing alternative stems and
unverified restrictions are deduplicated into `inventory_issues`; a known form
still receives credit while the warning prevents claiming a complete inventory.
A missing requested form remains an error. These are conditional fixture metrics,
not measures of excess candidates, unknown-word accuracy, or contextual accuracy.

`data/rewrite/synodal-byti.json` and `synodal-byti-model.json` supply ten selected
infinitive/present forms from [Alypy §81](https://www.ponomar.net/files/gama2/p081.htm).
The source was freshly retrieved and matched the previously downloaded bytes.
The model deliberately stores these whole forms as named principal parts, with
no claim to derive the irregular present from the infinitive. Accent/breathing
are supplied observations under a declared NFC rendering policy. Parenthesized
dual alternatives and other tenses remain outside this fixture. Tests pass this
verb through the same model-backed source document and JSON reload used by nouns.
These tests establish transcription and consumer integration, not independent
held-out evidence for productive rules or a complete verb paradigm.

The Synodal noun fixture separately exercises explicit stress positioning and
acute/grave/kamora rendering. Its table-derived settings likewise do not establish
general stress prediction. The remaining vertical-slice requirements include
broader cell modeling, abbreviation candidates, and observed evidence bound to
witness locations. The classification and adjective/adverb increment below does
not complete these requirements. No phase is declared complete.

Reported evidence review labels never authorize restrictions by themselves.
Caller acceptance must match both the complete evidence record and the specific
lexeme/restriction assertion. A trusted citation does not validate a new link.
Positive rules and relations still need broader claim-level acceptance/status
handling; structural validity is not linguistic authentication.

## Lexical classification and adjective/adverb integration

Runtime artifact version 4 requires source-linked `CategoryAssertion` records
on each modeled lexeme. `LexicalCategory` is separate from the legacy `Pos` used
by the current cell/paradigm implementation. Adverbs, particles, prepositions,
conjunctions, numerals, determiners and interjections therefore need not lose
their classification merely because they share an inflection implementation.
This does not implement every category's morphology: redesign of the inherited
cell schema and profile-specific feature inventories remains open.

Multiple assertions retain competing lexical classifications; they do not
assert a token's contextual use or license a Cartesian product of syntactic
readings. A lexeme with unresolved classification must say `Unclassified` with
supporting evidence. An empty or duplicate classification list, or a dangling
claim reference, fails construction. Source/review labels remain reported claims.
No category is inferred automatically from the paradigm's legacy POS.

`AnalysisDocument::Candidate::categories()` exposes modeled assertions; legacy
candidates return `None` because their lexical category was not independently
represented. Document record version 5 serializes those assertions, preserves
both lexical readings and rejects category tampering during model-bound reload.
Category/evidence string payloads are included in the document result budget.

`data/rewrite/synodal-mudr-model.json` contains three selected short adjective
cells from [Alypy §53](https://www.ponomar.net/files/gama2/p053.htm) and two adverb
spellings from [§106](https://www.ponomar.net/files/gama2/p106.htm). The two fixture
files retain exact expected spellings. The shared spelling мꙋ́дрѣ yields an
adjective locative reading and an adverb reading; мꙋ́дро and мꙋ́дрѡ remain distinct
under exact matching. Neither profile inventory nor excess-candidate accuracy
is established by these five rows. The source is the revised online edition.

The proposed `DerivedAdverb` link goes from the adjective lexeme to the adverb
lexeme and has a separate, explicitly unverified annotation record. Consumers
resolve its target and evidence through `Model::lexeme` and `Model::evidence`.
It does not cause the generator to infer the adverb from the adjective: both
entries supply their evidenced stem data. A productive derivation mechanism,
reviewed claim acceptance, witness-token attestations and broader abbreviation
handling remain separate work. The test establishes this relation's preservation and
source access, not expert adjudication of lexical identity.


## Registered abbreviation spellings

Runtime artifact version 3 adds optional `AbbreviationRule` records within an
orthographic profile. A rule binds its own ID, lexeme, cell, exact expanded
intermediate, abbreviated output, evidence, and explicit policy for retaining
the full spelling. These are finite registered spellings, not prefix or
subsequence guesses and not a claim to exhaust possible expansions.

The expanded intermediate must actually be realized by the supplied paradigm
under that profile. Rendering either adds the abbreviation as a variant or
replaces the full intermediate, according to `retain_expanded`. The latter is
an editorial rendering choice, not grammatical unavailability or source-text
editing. Grammar restrictions still govern the cell. `AbbreviationTrace` retains
the expanded and printed forms, rule ID and evidence separately from query
normalization. Identity within a lexeme/cell/profile snapshot includes both the
morphological rule and any abbreviation rule; the morphological rule alone does
not identify an abbreviated variant.

Generation, enumeration, exact/tolerant analysis, document candidates, and
record version 4 all use the same rules. The expanded spelling does not become
a retrieval equivalence or an attestation of an uncontracted printing. Several
registered expansions remain several candidates, including when the surface is
identical. Reload validates the complete derivation against the supplied model.
Source bytes and spans remain independent of editorial output.

The small fixture follows [Alypy §3, titla](https://www.ponomar.net/files/gama2/p003.htm):
the source contrasts бг҃ъ with бо́гъ for an idol. The model keeps two lexical IDs
and applies the contraction rule only to the former. This is a named online
edition's prescription; it is not a universal convention imposed on all witnesses.
The nominative-cell assignment and analytical expansion are explicitly unverified
annotations. No corpus-token attestation or independent expert gold is claimed.
The fixture model and two expected-form files are `data/rewrite/synodal-titlo-*`.

Construction rejects dangling claims, unrealizable intermediates, conflicting
print policies and oversized rules. Under an NFC output policy the registered
abbreviated spelling must already be NFC. This increment supports an abbreviated
output that fits one segment of the declared document tokenizer; a multiword or
punctuation-bearing abbreviation fails explicitly instead of being silently lost
by segmentation. More general segmentation alternatives remain required work.
Abbreviation work, candidate count and retained trace bytes participate in the
existing request limits; these logical budgets do not claim exact RSS bounds.

The tests use synthetic competing expansions and synthetic unavailable cells as
software controls, clearly separate from source claims. The new path does not
use the legacy global titlo table. Migration of that table and corpus consumers,
review of positive licensing claims, numeral-under-titlo interpretation, broader
abbreviation patterns, and witness-bound observations remain open.


## Verified source bytes, observations, and proposed analyses

Runtime artifact version 4 can declare sources by URI and SHA-256, exact UTF-8
witness slices by source-relative byte offsets, observed surfaces by offsets
within those witnesses, and separate `SourceAnnotation` records. Source files
remain external; `Model::build_with_sources` requires caller-supplied bytes
whose computed digest matches the declared source. Missing or changed material,
invalid UTF-8 boundaries, and a surface inconsistent with its span fail construction.
A serialized review label cannot substitute for source bytes. Matching bytes
establish identity of the supplied material, not authenticity of its attributed
URI, edition, or linguistic claims.

This first extraction operation copies an exact byte slice. It does not decode
HTML entities, flatten markup, normalize spelling, or extract PDF text. Such
operations still require explicit mappings. The original source hash and both
levels of offsets remain available through `Model::observations()`. Raw source
files are not embedded in the redistributed model artifact; only metadata,
selected short observations and annotations are stored there.

`ObservationKind` distinguishes running text, pedagogical examples, headings,
apparatus and unclassified material. An annotation preserves its original labels,
its evidence, and an optional mapped lexeme/cell/profile target. Unmapped labels
and annotations that disagree with generation remain in the archive. Valid named
references are checked, but agreement with the engine is not a construction gate.
An observation of a surface does not establish every syncretic analysis of it.

Generated derivations expose `source_annotations` only when the mapped lexeme,
cell, profile and observed bytes all match. These are links to asserted analyses,
not a Boolean attested status and not a gold label. No morphological form or
contextual preference is created from those links. The same spelling under a
different lexeme, cell or profile receives no inferred link. Source annotation
review remains inspectable separately from the evidence licensing generation.

`AnalysisDocument::analyze_registered_witness` uses the verified archive witness
and records its identity. Ordinary `analyze_model` does not acquire that identity
merely because the caller supplies a matching address. Document record version 5
preserves the witness's source metadata, spans, observations and all annotations,
including unmapped and contradictory ones. Reload recomputes against the supplied
model and rejects altered sources, observation spans or annotation content.

For the independently retrieved Alypy page already stored locally, run:

```sh
RUSTC_WRAPPER= cargo run --offline --release -p church-slavonic-tools --bin xtask -- analyze-witness data/rewrite/synodal-titlo-observed-model.json o-alypy-titlo-examples w-titlo-divine target/rewrite/upstream/alypy-p003.html
```

The observed model references the two short titlo examples in the online §3 note.
Their source bytes and offsets are verified; their nominated lexical/cell mapping
is an explicitly unverified assistant annotation. These are pedagogical examples,
not independent manuscript-token gold. Synthetic tests separately demonstrate
that a genitive annotation for града does not spread to the other three generated
readings, and that contradictory/unmapped annotations survive serialization.

Archive bytes and record counts have separate caller-owned limits. Raw sources,
extracted text, metadata, annotation labels and the support index are charged to
the archive budget; repeated annotation links also participate in request work,
result-byte and document-record limits. These are logical bounds, not measured
RSS or latency guarantees. Full corpus migration, general extraction mappings,
qualified annotation review and fixed-denominator corpus evaluation remain open.


The fixture checker also accepts explicit source-file arguments after its four
ordinary arguments, so observed models use the same generation/analysis gate.
The two observed titlo examples are the same selected rows already counted by
the spelling fixture; source-binding checks are reported separately, not added
to the linguistic denominator. Evaluation and ranking must keep held-out source
annotations and their support links out of prediction features; merely retaining
this metadata does not repair the legacy tagger's oracle-context evaluation.


## Deployment feature evaluation increment

`tag-sequence --ocs [--match policy] <token>…` and `tagger-curve` use the
same surface-only sequential predictor. Caller tokenization defines the sequence;
context resets between sequences. Feature choices remain distinct from retained
lexical readings. Unique dictionary lemmas may enter context, but source gold
lemmas, gold choices and dependency labels cannot enter prediction. Ties abstain.
The reported softmax share is not a calibrated probability.

| Consumer | Current path | Status |
|---|---|---|
| `tag-sequence` | Explicit embedded lexicon/policy; raw lookup and sequential feature tagger | Implemented baseline consumer |
| `tagger-curve` | Same predictor, followed by separate source-label scoring | Replaced oracle-only curve |
| `train-tagger` | Oracle training; conditional diagnostic plus separate deployment report | Training retained; artifact writes/provenance still need migration |
| `tagger-transfer` | Overlay hand context and conditional examples | Explicit oracle diagnostic; deployment transfer unresolved |
| `score-disambiguation` | Separate raw, rules-only, tagger-only and composed legacy trees | Fixed overlay-projection accounting; source-span migration pending |
| `eval` recall | Gold-lemma-conditioned generation and broad matching | Diagnostic requiring metric/matcher repair |
| Model-backed analysis documents | Explicit model/profile; morphological candidates | Contextual selection and corpus migration pending |

The evaluation counts all input tokens. Tokens without mapped labels are a
separate population, not successful predictions. Missing gold candidates and
abstentions remain in the mapped-gold denominator. Each mapped annotation slot
must belong to exactly one token; orphaned/reused slots, bad indices, surface
misalignment and declared token-count mismatches fail the evaluation. Original
source distinctions lost by the legacy adapter are not reconstructed here.

Feature retention, Unicode-equivalent displayed-lemma/cell retention and chosen
feature correctness are distinct counts. There is no adjudicated source-to-opaque
lexeme mapping and no joint lexical selection score. A controlled oracle report
changes only context on the same candidates, tokens and exact source feature
targets; its predictions never replace deployment output. No candidate-dependent
gold-cell broadening is used by this scorer.

Seven evaluator controls cover gold mutation, fabricated lemmas, missing
candidates, ties, unmapped tokens, malformed accounting, candidate/accepted-gold
order and the controlled oracle comparison. The real UD regression report is
recorded in current.json with source-file hashes. Direct integer-row counts in
local dev/test CoNLL-U files agree with its total. Local archive/cache byte
identity does not establish upstream authenticity, correct labels or an
independent split. Lexicon and model training lineage remain unverified for
contamination; these are regression diagnostics, not blind accuracy claims.


## Legacy overlay stage evaluation

`score-disambiguation` now reports four explicit stages built from the same
raw lifted tree. Rules run on one clone; the legacy tagger runs independently
on raw and rule-processed clones. Gold annotations enter only the scoring pass.
The tagger path here is the existing tree consumer, not the new surface-only
`tag-sequence` predictor; heuristic context and ambiguity loss in that tree
consumer remain subjects of the contextual rewrite.

The denominator includes every projected word in every parsed overlay entry
for the loaded source books, with nonlexical annotations counted separately.
Missing source verses and alignment failures retain their lexical annotation
counts. Missing overlay book files are listed; other I/O errors fail. Duplicate
annotation addresses fail. Empty lexical populations cannot report success.

A lexical target includes both a Lex cell set and an explicit Fn lexeme ID.
Wrong IDs are errors even when gold cells are ambiguous. Overlapping accepted
cells establish compatibility; a compatible singleton is reported separately.
Within compatible predictions, extra cell candidates outside the accepted gold
set are counted, not inferred
linguistically impossible. Unanalyzed/ambiguous surface nodes are reported as
no lexical prediction; the legacy tree does not expose full candidate sets,
so this is not complete analyzer candidate-retention measurement.

Alignment requires both complete tree renderings to equal the legacy source
print and each projected word rendering to match its counterpart. Invalid
renderings count as alignment failures. This conservative check can reject
legitimate differently structured abbreviation/clitic representations. It does
not repair lost spans: word_nodes omits punctuation and attached clitics, and
source print trims outer whitespace. Thus this report is explicitly an overlay
projection diagnostic, not a complete witness-token census or proof of exact
raw-source preservation. Model-backed source-span migration remains required.

Five constructed controls exercise ambiguous-gold lexical/cell errors, partial
compatibility versus singleton selection, omitted source/alignment/no-analysis
populations, empty evaluation/order controls, and closed-class identity. Corpus
results and byte identities are recorded in current.json; existing annotations
and pipeline inputs are not claimed to be independent linguistic gold.


## Verified source extraction

The UD and Syntacticus loaders now require exactly one `.tar.gz` per present
source directory. A missing source directory remains explicit absence; a
present empty/ambiguous directory, unreadable archive or invalid cache is an
error. Legacy extraction directories are preserved but no longer consumed.

The tools snapshot at most 64 MiB of compressed input privately, hash it, and
stream its tar entries through flate2/tar. Only normal relative paths, directories
and regular files are admitted. Links, special files, duplicate explicit paths,
more than 10,000 logical/expanded filesystem paths, paths over 4,096 bytes or
64 components, files over 64 MiB, and expanded streams over 512 MiB fail.
Global PAX metadata is limited to 16 KiB and comment keys, accommodating Git's
commit comment without honoring global path changes. These are logical input,
work and disk bounds, not a measured peak-RSS contract.

New files are written below a private staging directory and renamed together
into `target/sources/treebanks-verified-v1/<source>/<archive-sha256>`. A changed
archive gets another snapshot; it cannot silently merge into an old extraction.
A failed extraction does not publish its payload. A process killed before the
rename can leave an unused staging directory; power-loss durability is not
claimed. Concurrent publishers must produce the same file inventory.

Reuse reparses the actual archive and compares every regular file's path,
size and digest with the cache, rejecting missing, altered or extra files and
all links/special files. It does not trust an editable manifest. Empty extra
cache directories are harmless and permitted within the directory-count bound.
Revalidation reads the compressed archive and cached file bytes but does not
rewrite existing extracted files. This establishes identity at validation time;
callers must prevent concurrent mutation while consuming a returned directory.
The source hashes do not establish upstream authenticity or linguistic truth.

Six controls cover cache mutation and file presence, changed sources, failed
publication, unsafe entry kinds/paths, duplicate entries, corrupt/empty archives,
oversized file declarations, PAX metadata and actual adapter selection. An
explicit local-source integration test loads both downloaded corpora; ordinary
CI skips only that test. Independent Python tarfile comparison verifies all
186 extracted files against the two local archives in archive-source-audit.json.
Model/artifact publication elsewhere and original annotation preservation remain
separate migration work.

Implementation references: [tar archive iteration](https://docs.rs/tar/0.4.46/tar/struct.Archive.html)
and [flate2 gzip decoding](https://docs.rs/flate2/1.1.10/flate2/read/index.html).


## Original UD records and mapping export

The UD train and dev/test loaders now retain immutable source documents and an
ordered partition of every original line, including its newline bytes. Each
document has a relative archive path and SHA-256; records carry source-relative
UTF-8 byte ranges. Comments, separators, ordinary word rows, multiword rows,
empty nodes and malformed rows have distinct kinds. These are source-format
records, not a claim that every line is an independent linguistic token.

Each ordinary word token links to its source record. The record separately
lists mapped slot indices and any mapping failure. Raw forms, original lemma
case, UPOS/XPOS, features, dependencies and MISC survive in the source line even
when the legacy mapper lowercases lemmas, cleans surfaces or folds categories.
Duplicate feature keys and duplicate integer IDs cannot silently select a gold
label; those word rows remain in the token population with mapping failures.
Malformed rows, multiword rows and empty nodes remain explicitly visible records
outside the legacy ordinary-word denominator. Sentence comments are preserved
verbatim; semantic document/sentence lineage and alternative segmentation are
not yet adjudicated.

`cargo xtask corpus-observations <ud-train|ud-heldout>` streams JSONL: mapping
policy, source documents, original records, then mapped slots. Reconstruct each
source by concatenating its raw record fields in order. Mapping rows are legacy
assertions, not independently accepted evidence. This export does not feed gold
or original-annotation metadata into the deployment predictor.

Source documents are limited to 64 MiB, paths to 4,096 bytes, individual lines
to 64 KiB, and the observation archive to 10,000 documents, one million records
and 128 MiB of logical retained source/record data. Record ranges must partition
the complete UTF-8 source. Mapping is staged: rejected appends leave the existing
Corpus unchanged. Streaming export avoids materializing the entire JSON output;
it remains fallible on writer errors. These are logical limits, not measured RSS.

Four observation controls plus the existing deployment/archive controls cover
exact CRLF preservation, original labels, unsupported categories, malformed and
non-word records, duplicate IDs/features, source/slot addressing, empty export
refusal and rejected-append rollback. Independent consumer reconstruction of all
three real CoNLL-U files matched their source digests and bytes, accounting for
39,133 dev/test and 159,710 train word rows. All mapped slots had exactly one
original-record reference. Results are recorded in ud-observation-audit.json.

Syntacticus originally used a string-based XML adapter without source-record
links; that extraction path is replaced by the event-based intake below. Its
linguistic cell mapper still requires migration. Generalizing this archive to XML is separate from treating a
raw XML string search as validated extraction. Neither UD export nor the new
archive establishes independent gold or resolves the inherited cell ontology.

The record classifier is an intake adapter, not a full [CoNLL-U conformance
validator](https://universaldependencies.org/format.html). It preserves even
nonconforming line endings and does not yet validate every dependency graph,
ID ordering, field syntax or normalization constraint. Source-file reproduction
is distinct from reconstructing the pre-tokenization textual witness.


## Original PROIEL XML records

Syntacticus now uses quick-xml events instead of searches for token tags and
quoted substrings. The intake accepts one PROIEL root in UTF-8 XML 1.0, checks
balanced structure, rejects duplicate attributes, nested token/sentence/source
structures and unsupported DTDs, and decodes attribute entities with XML 1.0
normalization. It never fetches external entities. It is not a complete validator
for the PROIEL schema or every XML conformance rule.

A token opening tag has an exact source byte span; intervening markup, token
children, closing tags and other text remain in contiguous XmlMarkup spans.
Together these partition the full immutable XML document, not merely the visible
forms. Selected OCS documents preserve all their markup. Tokens in another
language within a mixed document remain OtherLanguageToken records. Entire
files with no source language chu have explicit path/hash/byte-count exclusion
records rather than silently disappearing from the input inventory.

A token with a nonempty form enters the legacy input sequence even if it lacks
lemma/POS/morphology annotations; its mapping failure remains explicit. Tokens
without a source surface remain XmlToken observations with no-source-surface
status and never acquire an invented input string. Thus Corpus.tokens means
surface-bearing XML input tokens; the observation population includes the
additional nonsurface nodes. Source sentence boundaries govern sequence resets.
Original token IDs, morphology strings, dependencies and attributes remain
available in the raw source independently from the legacy mapped cells.

`cargo xtask corpus-observations syntacticus` exports the same streaming record
and mapped-slot contract as UD, plus excluded-source rows. The unchanged legacy
morphology mapping is named legacy-corpus-cell-mapping-v1. End-to-end source,
record and slot offsets work across documents; failed XML appends publish no
partial Corpus changes. Source/record archive limits still apply, with XML tags
limited to 64 KiB and element depth to 128. These are bounded ingestion policies,
not claims of complete PROIEL schema validation or historical grammatical truth.

Three XML controls cover both quote styles, numeric and predefined entities,
single-pass decoding, missing annotations and surfaces, language/sentence
boundaries, malformed/nested/multiple roots, duplicate attributes, DTD refusal
and atomic rejection. Existing UD and deployment controls also pass. Independent
ElementTree and byte-reconstruction checks account for 224,990 XML token records
in nine OCS documents, including 11,332 without source surfaces, and validate
77 excluded XML files. All mapped slots have unique original-record links.
Counts and hashes are in xml-observation-audit.json.

Remaining work includes semantic source-address/annotation migration into the
new model, dependency graph validation, unknown morphology/category distinctions,
qualified annotation review, and separating shared source ancestry in evaluation.
Source preservation makes these questions inspectable; it does not settle them.


## Invariable verb cells and corpus mapping v2

The core schema now distinguishes `Cell::infinitive()` (`inf`) from
`Cell::supine()` (`sup`). Supine has no agreement number/person/gender/case in
this cell model. The string parser, legacy tree codec, modeled generation,
analysis, document codec and fixture checker all carry the distinction.
Adding a representable cell does not license it in every paradigm or profile.
An unrelated Synodal fixture returns UnsupportedCell, not a claim of historical
unavailability, for an unimplemented supine request.

Both corpus adapters previously checked number before reaching invariable verb
forms. Mapping v2 handles UD VerbForm=Inf/Sup and PROIEL mood n/u first. It does
not infer number or relabel supines as infinitives. Other source features (such
as PROIEL infinitive tense/voice and supine case) remain on original records;
the invariable target cell does not encode them. Finite forms still require
number. This is a stated source-to-schema mapping, not a claim that all source
features are redundant or erroneous.

The independently consulted [LRC §46.1](https://lrc.la.utexas.edu/eieol/ocsol/100)
supplies three selected infinitive/supine pairs, represented by six new fixture
rows. Two share a supplied stem plus hard-jer supine ending; the third uses a
supplied stem plus soft-jer ending. The implementation reproduces these examples
through reusable suffix rules and explicit lexical inputs. It does not derive
the supplied stems from finite forms or establish productive historical sound
changes. Written accents are not invented. These are pedagogical examples,
not independently adjudicated manuscript tokens or a heldout generalization set.

[Syntacticus's development guide](https://dev.syntacticus.org/development-guide/)
and the downloaded XML tag definitions identify n as infinitive and u as supine.
The UD cu VerbForm documentation currently includes Old East Slavic examples and
terminology, so it was not used as the OCS grammatical authority for this slice.
Only the explicit source tags are mapped; syntactic licensing after motion verbs
is not introduced as a hard filter.

Four controls cover cell/codec distinction, numberless mappings in both adapters,
continued finite-number requirements, exact generation and analysis of six forms,
document reload, and lack of automatic licensing in another profile. The ordinary
CLI fixture checker also passes the soft-jer pair. Independent export checks
match every original invariable annotation to exactly one corresponding target
cell: UD dev/test 693 infinitives and 25 supines; UD train 2,673 and 110;
Syntacticus 3,632 and 136. Corpus overlap prevents treating their sum as independent
linguistic evidence. Counts and scope are in invariable-mapping-audit.json.

These new mappings change annotated evaluation populations. Earlier deployment
and recall reports using mapping v1 are historical and must not be compared as
if they used the same denominators. The bundled tagger and legacy lexicon have
not been retrained or broadly migrated to supine paradigms by this increment.
# Given-lemma generation evaluation

The default `eval` corpus section and `eval-generation <ud-heldout|syntacticus>`
now use `eval::generation`. This is explicitly conditional generation from a
mapped annotated lemma and cell, not surface analysis accuracy. Candidates are
existing lexemes with the same inflection POS and NFC-equivalent displayed
lemma. No guessed fallback, extra clitic cell, imperfect/aorist substitution,
accent removal, or broad spelling tolerance is used. Exact surface counts mean
byte equality with the adapter-cleaned surface; a second count permits NFC.
Neither claims original-witness fidelity or adjudicated lexical identity.

Each mapped token contributes once, accepting any of its mapped alternatives;
slot counts separately show the alternative population. Missing lemmas and
missing generated forms remain failures. Unmapped tokens are explicit. Invalid
slot ownership, mismatched surfaces/counts and missing sources fail. An empty
mapped population is unavailable. `legacy_relaxed_generation` remains a named
diagnostic API and is no longer the ordinary `eval` corpus implementation.

Five controls exercise absent lemmas, unsupported cells, case/accent/letter
loss, positive Unicode equivalence without byte-exact credit, ambiguous targets,
unmapped inputs, reordered alternatives, and invalid/empty accounting. Real
UD and Syntacticus consumer results and hashes are in `data/rewrite/current.json`.
Source annotation mapping remains legacy v2 and corpus/lexicon independence is
unestablished. The separate surface tagger evaluator does not become a joint
lexical selector through this change. Full V5 migration remains incomplete.


## Retired vocative adjacency exclusions

`Sentence::disambiguate` and the tools' shared treebank rule layer no longer
apply `voc-drop` or `bare-voc`. Missing adjacent imperative/interjection is not
an adequate premise for removing a vocative or selecting another lexeme.
Krause and Slocum's [LRC OCS lesson 10](https://lrc.la.utexas.edu/eieol/ocsol/100)
identifies the address `филосѡѳє` followed by indicative `вѣмь`. This upstream
teaching analysis challenges the former premise; it is not independent expert
adjudication of the project's corpus or constructed test paradigms.

Two controls reproduced the previous destructive behavior. Final controls use
explicitly constructed neutral spellings under both legacy profiles to isolate
cell and lexical ambiguity retention from differing orthographic renderers.
They exercise the public Sentence path, punctuation and original whitespace,
tree reload, and actual candidate preservation through the source-document
codec. A legacy W tree node still retains only a surface and ambiguity marker;
its serialization is not being claimed to store full lexical alternatives.

Current overlay-stage results and the historical comparison are recorded in
`data/rewrite/current.json`. Increased ambiguity is accepted rather than removed
to preserve an old score. The composed tagger output also changes; no claim of
better contextual accuracy or adjudication of each changed reading is made.
Other adjacency rules, attachment modeling, and immutable contextual derivation
records remain unfinished V5 work. This removal is not a guarantee that other
rules or the tagger can never reject a vocative.


## Inspectable legacy contextual proposals

`Sentence::contextual_trace()` evaluates a fresh lift of the original witness
without editing the current tree. `context-trace [--ocs] <text>` exports its
input, proposed tree, and each successful rule update with a pre-update group
snapshot and proposed replacement child. This follows the existing sequential
schedule. It does not certify grammatical premises, add syntactic attachments,
or record every rejected attempt. Some predicates capture earlier information;
this operational history is not a complete premise dependency graph.

Tree-child indices are deliberately not described as source-token indices or
byte spans. The JSON also contains the source-analysis record and actual raw
morphological alternatives under its explicit Exact policy. Legacy lifter
matching, abbreviation and prosodic segmentation can differ. W tree nodes still
contain ambiguity markers rather than full candidates; the two representations
must not be conflated. The statistical tagger is not part of this trace.

`context_trace::from_json` recomputes from the original source under the supplied
lexicon and compares the entire record. Serialized trees are not executed.
Changed decisions, snapshots, or source candidates are rejected. This is
behavioral replay, not a complete build fingerprint or linguistic validation.

Tracing limits source size, node count, depth, logical tree storage and events;
serialization limits bytes while writing. Exhaustion returns an error and does
not mutate the input or yield a purported complete empty result. Nested groups
are explicitly unsupported. These bounds do not establish legacy index startup
memory or a complete end-to-end work bound. Current limits, commands, tests and
consumer artifact hashes are recorded in `data/rewrite/current.json`.

The ordinary mutating rule path remains available as the baseline. Three controls
exercise explicit successive cell states, replay against ordinary output,
source preservation, tampering and exhaustion. The new consumer makes legacy
proposals inspectable; it does not complete contextual-model or corpus migration.


## Selected inflected numerals

`data/rewrite/ocs-numeral-model.json` implements the listed forms of two, three,
and four from Krause and Slocum's [LRC lesson 9, §44](https://lrc.la.utexas.edu/eieol/ocsol/90).
The runtime profile uses the displayed Cyrillic spellings without additional
normalization or accent prediction. Explicit supplied stems and suffix rules
reproduce 45 case/gender/number readings over 15 distinct surfaces. Common-case
forms are expanded over the agreeing genders; unlabelled head forms are treated
as nominative. These are documented interpretations of a teaching list, not
45 independent manuscript attestations or productive stem predictions.

Lexical classifications remain Numeral and Adjective. The existing `pron` cell
shape supplies agreement features without imposing person or adjective degree;
it does not classify the lexemes as pronouns. Two uses the listed dual cells;
three and four use the listed plural cells. Unlisted cells, including invariant
Word and singular requests, remain unsupported. They are not declared impossible
on the strength of an incomplete list. The source's differing locative spelling
in lesson 8 was not silently added through a general orthographic fold.

The ordinary consumer can inspect the model with:

```sh
cargo xtask check-linguistic-slice data/rewrite/ocs-numeral-model.json data/rewrite/ocs-numeral-l-000012.json l-000012 o-lrc-numerals
cargo xtask analyze --model data/rewrite/ocs-numeral-model.json --orthography o-lrc-numerals 'три дъвоѭ'
```

Five readings of три and six of дъвоѭ survive source-document reload. Current
commands, hashes and checks are in `data/rewrite/current.json`. Numeral phrase
syntax, numeric-value interpretation, complete feature ontology, broader data
migration and legacy corpus NUM mapping remain incomplete. No rule has been
transferred to the Synodal profile by this increment.


## Lexical gender assertions and document version 6

Modeled lexical entries now accept `genders`, a list of typed m/f/n assertions
with evidence references. Missing input means no assertion supplied. Conflicting
values can coexist; neither construction nor analysis selects the first as a
resolved truth. Repeated values are rejected in favor of a combined evidence
list. The engine validates references, not the truth of the source's claim.
Lexical assertions remain separate from inflected agreement gender on a cell.

`Candidate::lexical_genders()` exposes modeled assertions. Source-document
version 6 exports and validates them on reload. Version5 records are historical;
the new codec rejects them, and contextual records nesting those source records
must be regenerated. Legacy candidates expose no evidence-backed lexical gender
through this API. Existing modeled entries without assertions are not silently
assigned gender. Model data and engine identities reflect the representation
change; this is not a claim of a complete build dependency fingerprint.

`ocs-noun-numeral-model.json` uses the feminine noun description and forms in
[Krause and Slocum, LRC lesson9 §44](https://lrc.la.utexas.edu/eieol/ocsol/90).
The listed nominative/genitive forms of five through ten share supplied-stem
rules; ten's separately listed locative is also represented. Thirteen form rows
pass generation and analysis. For example, пѧти has a genitive singular cell and
a separate feminine lexical assertion. Numeric quantity does not become plural
morphology by default. The teaching list, assistant segmentation, and unfinished
expert validation are recorded as such. Unlisted paradigmatic cells, numeral
phrase syntax and general lexical-gender/animacy migration remain incomplete.


## Retired bare-loc exclusion

The shared Sentence/treebank rule layer no longer removes locative readings
because a bounded backward search found no preposition. [LRC OCS lesson8,
§38](https://lrc.la.utexas.edu/eieol/ocsol/80) describes non-prepositional temporal
uses and locative complements of verbs. The old allowance for an already unique
locative did not justify removing one from an ambiguous candidate set.

Two constructed controls reproduced cell deletion and competing-lexeme selection
before the change and now retain their original trees in both legacy profiles.
The Sentence example preserves lexical/cell ambiguity. Operational trace tests
now use a constructed preposition/agreement sequence for their two successive
updates; they do not retain a disproven rule to keep a fixture passing.

The real overlay comparison retains all prior denominators. Raw and tagger-only
stages are unchanged; rules-only has more unresolved alternatives, and the
composed tagger choices also shift. Current deltas and limitations are recorded
in `data/rewrite/current.json`. Neither fewer decisions nor a changed score
establishes that all remaining analyses are correct. Other agreement, government
and subject rules still need attachment/premise validation. Historical contextual
records containing bare-loc decisions must be regenerated for current replay.

### Explicit stem-final replacement

`CellRule.stem_replacement` optionally replaces one exact nonempty ending of a
supplied stem before suffixation, accent computation, and spelling rendering.
It is a rule-local morphological instruction, not automatic application of a
historical sound law. A mismatching supplied stem is rejected during model
construction; a missing stem remains missing lexical information. No-op and
empty-from operations and missing evidence references are rejected. Input/output
stem bytes are bounded before transformation allocation, and retained traces
count towards generation, analysis, abbreviation, and document budgets.

`Derivation.stem_change` retains before/after text separately from editorial
spelling steps. Stress vowel indices address the transformed stem or resulting
word, not the original supplied stem. Operations that change vowel count must
therefore supply appropriate stress instructions; no historical accent or
phonological reconstruction is inferred. This initial operation is a single
explicit final replacement, not a general stem-expression language.

The public model consumer now exercises two teaching pairs from Krause and
Slocum, https://lrc.la.utexas.edu/eieol/ocsol/20 §6.1: отрокъ / отроци and
могѫ / можєши. The source is independently acquired; supplied stems отрок and
мог, endings, cell assignment, and rule-local replacements к→ц and г→ж are
explicit modeling decisions. Both pairs were consulted during implementation;
they are development examples, not independent held-out productivity evidence.
Only the four listed cells are represented. No full noun or verbal paradigm,
general sound-change inference, or transfer to Synodal is claimed.

Run `xtask check-linguistic-slice` with
`data/rewrite/ocs-stem-replacement-model.json`, either
`data/rewrite/ocs-stem-replacement.json` (l-000020) or
`data/rewrite/ocs-stem-replacement-verb.json` (l-000021), and profile
`o-lrc-otrok`. `xtask analyze --model` uses the same model through source document
export and replay. Source-document codec is now version 7; older exports,
including context records nesting version 6, must be regenerated. Model input
version 4 accepts the new optional operation; model and engine hashes change.
Evidence IDs and reported source-check status still do not establish scholarly
acceptance of the precise positive rule attachment; that broader V5 work remains
open, as do productive evaluation, full morphology, and consumer migration.

### Validated publication of individual import outputs

The existing refit, crosscheck, and lexicon-import writers now prepare a sibling
file, validate its written bytes, synchronize the staged file, and rename it
into place through `import::publication`. Lexicon output must pass the checked
snapshot constructor and parsing/formatting checks. Mixed profiles, duplicate
identities, invalid stress/class information, and mixed POS are rejected rather
than published. Import outcomes are validated before an ID-keyed merge could
hide duplicate input identities. Quarantine fields cannot inject row separators,
and errors reading existing quarantine data stop publication rather than being
interpreted as absence. Lexicon and quarantine stages are both validated before
either rename.

This is **per-file atomic replacement**, not a multi-file transaction. A process
exit between the lexicon and quarantine renames can still expose a mixed pair.
Parent directories are trusted and concurrent writers are not coordinated.
Existing permission bits are preserved, but ownership, ACLs, extended attributes,
and hard-link relationships are not promised. Staged-file `sync_all` does not
establish directory/power-loss durability. Abrupt exit before commit may leave an
unpublished temporary sibling; it does not truncate or replace the live file.

The staged payload has a 64 MiB admission limit. This is not a whole-import memory
or CPU limit: fitting, existing-file reads, formatting, and input snapshots have
separate unresolved resource accounting. Class-table `--fix-marks` and other
construction writers still require their own migration. Full V5 atomic import
and cached-generation requirements therefore remain incomplete.

Seven controls exercise validation failure, dropped staging, rename failure,
a subprocess exit that bypasses destructors, permission/symlink behaviour,
actual refit and import writers on temporary directories, and staging validation
of all nine existing TSV assets (Synodal five POS files, OCS four; no separate
OCS closed.tsv). Existing lexicon assets were read and staged in temporary
directories only. These checks establish engineering behaviour, not grammatical
correctness or source independence of the legacy lexical data.

### Caller acceptance of exact generated claims

Every ordinary modeled generation result now includes `claim_sha256` and
`GenerationReview`. `Availability::Licensed` means that the executable model
licenses the result under its assumptions; the default review is `Unreviewed`.
Neither an input `SourceChecked`/`ExpertReviewed` label nor caller verification
of the cited evidence record accepts its attachment to a positive form.

A caller that has reviewed a specific generated claim may supply a
`GenerationAcceptance` to `Model::with_generation_acceptance`. The decision
names the lexeme, cell, orthography, exact returned claim key, decision identity,
and review method. This replaces the policy while consuming the unshared model;
it does not mutate an already borrowed analysis snapshot. Decisions are not
loaded from `ModelInput` and are not automatically deserialized by the artifact
loader. Merely copying a generated hash into this caller API is not scholarly
review: `CallerAccepted` records the caller's decision, not authenticated expert
credentials or historical truth.

The key binds the input model identity, recorded engine identity, lexeme/cell,
and complete pre-review derivation after source-annotation linking, including
rendering, rule, spelling, accent, abbreviation, and evidence references. Thus
altered source claims or rule attachments invalidate old decisions even if an
unchanged citation/review label is copied. Binding uses the entire input model,
so unrelated input changes can conservatively invalidate decisions as well.
This is not a full dependency-closure/build identity: the existing engine hash
still covers selected source files. Caller policy identity is separate from
input-data identity and participates in document replay checks, even for an
empty document. Policy order does not affect that identity.

Accepting a form does not accept other cells sharing its surface or evidence,
does not delete competing forms, and does not resolve missing stems. A partial
result remains partial. Unsupported, unavailable, and pending-restriction
queries cannot be promoted through this positive policy. The policy currently
accepts generated claims only; lexical relations, categories, gender assertions,
and other positive claims still need their own acceptance contract.

Admission limits include at most 1024 decisions (also bounded by the configured
candidate count), nonempty identity/method strings of at most 4096 bytes each,
valid claim keys, total stored decision strings, and counted rule/abbreviation
checks. Generation, analysis, abbreviation derivation accounting, and document
accounting include the returned claim key and review strings. Exhaustion is an
error, not grammatical absence. These logical limits do not establish total
process memory or a complete construction-cost bound.

Source-document codec is now version 8, with caller-policy identity and reviewed
derivations. Earlier v7 and older exports/context records require regeneration.
Model input remains version 4; ordinary CLI loading leaves every claim
unreviewed. Seven constructed controls cover copied review labels, mismatched
cells, altered rules/evidence/attachments, forged serialized decisions, policy
order, preserved alternatives/missing information, and review-output budgets.
The actual CLI export `target/rewrite/acceptance-consumer.json` reproduces the
four existing noun/verb teaching examples with four distinct unreviewed keys.

### Caller acceptance of lexical assertions

`Model::lexical_claims` exposes categories, lexical gender assertions, and
explicit directed lexical relations as separate `ReviewedLexicalClaim` values.
Each has its own key and `ClaimReview`; all default to `Unreviewed`, including
when a cited evidence record was caller-verified. The raw `LexicalEntry` fields
remain source assertions. `GenerationReview` is an alias of the shared review
type; its serialized variants are unchanged.

`with_lexical_acceptance` consumes an unshared model and replaces only the lexical
caller policy. Decisions bind a lexeme and exact claim, including relation kind,
target, evidence attachment and the model's grammar context through input-model
identity. They cannot be transferred to a different lexeme with identical
assertion text. Altered claims or source locations invalidate old decisions.
The whole-model/selected-engine identity is conservative, not full dependency
closure or scholarly authentication. There is no automatic acceptance based on
an input review label, and the artifact loader does not load caller decisions.

Accepting a relation does not accept a target's category, a gender assertion,
or any generated form. Conflicting assertions remain available; acceptance is
not a selection or exclusion rule. Generation and lexical policies have distinct
identities and neither setter clears the other policy. Decision ordering does
not affect policy identity. Duplicate decisions fail; reset returns all claims
to unreviewed. Empty documents also check policy identity on replay.

Ordinary modeled analysis and document candidates now retain these reviewed
claims. Counts and string budgets are checked before copying claim data and
review strings; candidate construction uses the request's remaining output
budget. Counts accumulate into document analysis work. Limits do not establish
whole-process memory bounds or the authenticity of caller review methods.

Source-document codec is version 9. Earlier v8 and older exports/context records
must be regenerated; model input remains version 4. The actual default CLI
export `target/rewrite/lexical-acceptance-consumer.json` retains two readings of
мꙋ́дрѣ and three unreviewed claims, including the explicit adjective-to-adverb
relation. Seven constructed controls exercise policy isolation, changed claim
attachments, conflicting assertions, forged records, canonical policy ordering,
and bounded work/output. No independent linguistic endorsement was created.

### Additional hard o-stem teaching paradigms

`data/rewrite/ocs-o-stem-model.json` adds three explicitly supplied OCS paradigms
from Krause and Slocum, https://lrc.la.utexas.edu/eieol/ocsol/10 §3.1:
чловѣкъ (22 listed form rows across 21 cells, including two dative singular
alternatives), мѣсто (21), and вѣко (21). Their opaque identities are l-000022,
l-000023, and l-000024; lexical gender is represented separately from noun cells.
These are new modeled examples, not a completed migration of legacy entries.

Cell-local stem replacement distinguishes masculine чловѣци from neuter вѣка,
while preserving neuter dual вѣцѣ and the different vocative чловѣчє. There is no
global к→ц spelling substitution or universal front-vowel rule. The regular
neuter мѣсто provides a contrast. Analysis of вѣцѣ retains locative singular and
nominative/accusative/vocative dual; both dative singular variants of чловѣкъ
remain generated. All generation and lexical claims remain unreviewed by default.

A fresh download has the same SHA256 as the previously acquired lesson 1 source.
The source was already consulted, and all new table outputs were read during
implementation. Consequently these fixtures are development/source-transcription
checks, not blind evaluation, independent scholarship, or individual manuscript
attestation. The additional transfer control leaves the earlier отрокъ rules
byte-identical and supplies чловѣк as a new stem; it demonstrates two additional
rule applications, not stem induction or unrestricted productivity.

All 64 fixture rows are compared independently with the downloaded HTML table
cells by `target/rewrite/check-o-stem-transcription.py`; its report is
`target/rewrite/o-stem-transcription.json`. Three actual
`check-linguistic-slice` CLI runs pass 64/64 generation and joint retention.
The ordinary `analyze --model` consumer and document replay preserve nine
readings across the four-word probe. Four new tests also verify the differing
cell-local changes and a deliberately corrupted rule's failure against the
unchanged source fixture. Runtime code and source-document version 9 are unchanged.

### Present and imperfect teaching paradigms

`data/rewrite/ocs-finite-model.json` adds глаголати (l-000025) and молити
(l-000026) under the explicit profile `o-lrc-finite-tables`, following Krause and
Slocum, https://lrc.la.utexas.edu/eieol/ocsol/10 §§4.1–4.2. Each has nine present
and nine imperfect table cells plus its cited infinitive: 38 listed rows total.
The separately supplied present, imperfect, and infinitive stems are visible in
the lexical data. They were not inferred by the engine. The молити present uses
explicit rule-local stem-final replacements in first singular and third plural;
its imperfect allomorph is supplied directly.

Written U+0484 palatalization is preserved separately from stress. An unknown
stress position remains unknown, and accent-insensitive matching does not erase
this mark. Analysis retains present third dual/second plural syncretism and
imperfect second/third singular syncretism. Prose-described later or unspecified
textual variants are outside this literal table profile, not deemed impossible.
Aorist, future and supine queries remain unsupported for these two paradigms;
these are coverage limits, not universal linguistic exclusions.

The source was previously consulted, and its reacquired bytes match the earlier
lesson 1 acquisition. A separate HTML table extraction verifies all 36 finite
fixture cells; the two infinitives were checked against the source headings.
Both actual CLI fixture runs pass all 38 rows. The ordinary analysis/reload
probe retains five readings over three tokens, including the palatalization
mark and whitespace. Removing the present stem leaves nine unresolved cells
and reduces the unchanged 19-row fixture to ten generation/retention hits.

This is source-based development validation and supplied-stem rule execution,
not blind accuracy, principal-part induction, full verbal morphology or completed
legacy migration. Runtime code and document version 9 are unchanged. Four new
controls plus the existing five stem-operation tests and clippy pass.

### Nominative adjacency and finite-person uncertainty

The ordinary `Sentence::disambiguate` path no longer applies `subj-verb`.
That rule narrowed an adjacent finite verb to third person and the nominative
noun's number without establishing a subject relation. It is retired as an
unsupported hard exclusion, not as an independently demonstrated error on every
matching historical occurrence. No replacement attachment or preference model
is claimed. `prep-gov`, `np-agree`, and `one-subject` remain provisional legacy
exclusions requiring their own premise audits.

Constructed controls use supplied `видѣ` second/third singular aorist readings
beside a nominative-only noun in either order and both legacy profiles. They
exercise ordinary sentence decisions, unchanged source bytes, contextual replay,
and reloaded full source candidates. Passing these tests establishes retention
under this operational policy, not attestation or grammatical correctness of
every candidate. The source-document codec remains version 9.

### Indexed modeled analysis

`Model::analyze`, modeled source documents, and the existing `analyze --model`
CLI now use an in-memory retrieval index for each requested orthography/matching
policy pair. The first request constructs the index; later requests retrieve
matching derivations without regenerating unrelated paradigms. Equal retrieval
keys retain all lexical/cell/derivation alternatives. The existing comparison
function still produces each successful match trace. Missing stems and pending
restrictions remain explicit even when the queried surface has no candidates.

Construction publishes only complete indexes. Caller `ModelLimits` separately
bounds aggregate successful cache construction: `max_index_bytes` (64 MiB
logical accounting by default), `max_index_forms` (100,000), and
`max_index_checks` (1,000,000). These totals cover all cached profile/policy pairs
in the model; there is no eviction or silent fallback after exhaustion. Logical
accounting includes retained strings and specified record sizes; it is not an
allocator-capacity or process-RSS bound. Cold builds are serialized under a
write lock; warm queries share read locks. Query/output budgets remain active
and query work accumulates across source-document tokens. Construction and
query work are different reported units; warm queries are no longer charged for
regeneration they do not perform.

Changing generation acceptance discards the cache before applying the new
policy. Lexical review claims are obtained from the current policy when returning
candidates, not frozen in cached generation. Index state is not linguistic data
and does not change the model data identity. The new caller limits participate
in that identity, and the selected engine-source hash includes `index.rs`.
Neither fact establishes the outstanding full build/dependency fingerprint.
Source-document codec version 9 and model-input artifact version 4 are unchanged.

The reproducible `model_runtime` tools example duplicates the two supplied OCS
finite paradigms to measure 2/50/200-lexeme workloads. Duplicates are neither
migrated lexemes nor new linguistic coverage. The local 200-lexeme run reduced
40 warm queries from about 631 ms to 5.8 ms, with about 18 ms cold-query time and
3.1 MB of reported logical index storage. These are environment-specific samples,
not production latency guarantees. A 40-token CLI control preserves 4,000 full
candidate records and original whitespace; the prior executable exhausted its
regeneration-work budget on the same constructed population.

The initial in-memory increment did not provide a persisted compiled index.
The compiled transport below addresses that subsequent step; default legacy
consumer migration and production-scale validation remain outstanding.

### Compiled index transport and startup

The core now provides `Model::compile_analysis_index` and
`Model::install_compiled_analysis_index`. Installing restores the existing
modeled-analysis cache without running paradigm generation. Ordinary modeled
queries and source documents then use those entries. `restored_indexes` in
`analysis_index_stats()` distinguishes restored entries from newly built ones.
The model's validated construction input is still loaded separately.

The compiled transport is version 1, separate from model-input version 4 and
source-document version 9. It uses newline-delimited JSON: one compatibility
header followed by typed form, pending-restriction, and missing-stem records.
Retrieval keys are reconstructed under the selected matching policy. Input is
bounded to 64 MiB, each record to 1 MiB, and each record's JSON structure to depth
64 and 4,096 container/separator units before deserialization. Counts, duplicate
identities, references, and aggregate model index budgets are checked before a
complete index is installed. Failed installation leaves existing indexes intact.
These are logical/work bounds, not a complete allocator or process-RSS budget.

Fast loading requires an expected SHA-256 obtained through a caller-trusted
channel. The payload cannot authorize itself with a checksum or a review label.
Its declared model, selected engine, generation-policy, orthography, matching
policy, and caller runtime identity must match the live consumer. Compiler-output
completeness and the correctness of unreviewed derivations rely on that trusted
compiler output; structural validation does not independently rederive every
form. Operational trust in the cache is separate from linguistic endorsement.
Caller-accepted derivations additionally have their exact claim identity and
review decision recomputed from their serialized content on restoration. An
accepted label copied onto changed content is rejected even with a newly
supplied transport digest. Lexical claims continue to use the current model's
lexical acceptance policy when returning candidates.

The tools expose:

```text
RUSTC_WRAPPER= cargo xtask compile-index data/rewrite/ocs-finite-model.json o-lrc-finite-tables exact target/rewrite/finite.idx
RUSTC_WRAPPER= cargo xtask analyze --model data/rewrite/ocs-finite-model.json --orthography o-lrc-finite-tables --index target/rewrite/finite.idx --index-sha256 TRUSTED_SHA256 "мол҄ꙗашє"
```

Replace `TRUSTED_SHA256` with the digest returned by your trusted local compilation.
The compiler stages the file, validates a fresh-model reload of staged bytes,
and only then replaces the destination. It rejects an output that would replace
an input source. `analyze --index` requires the digest explicitly; it does not
read one from an adjacent untrusted manifest or silently fall back after failure.
The library API supports models loaded with registered sources; the `analyze-witness` CLI now accepts the same explicit index/digest options
as described below.

Tools use their executable-file SHA-256 as a conservative runtime identity.
The inspected executable embeds the Rust implementation and links dynamically
to libiconv and libSystem. Its identity does not cover changing system libraries,
OS state, or concurrent replacement of the executable on disk. This improves
compatibility checking without claiming the outstanding full dependency closure.
A different tools executable conservatively requires recompilation. Library
callers supply their own runtime/build identity under the same explicit contract.

The `compiled_runtime` example records cold generation, export, restore, and
post-restore query costs for 2/50/200 constructed lexemes. Restoration includes
transport hashing and record validation but excludes model-input loading and
executable hashing. Local 200-lexeme measurements show only a modest startup
benefit; do not generalize them into a production latency guarantee. The actual
40-token CLI control produces byte-identical cold/restored source documents,
with 4,000 candidates and all 522 source bytes preserved. This is transport and
consumer equivalence, not additional linguistic coverage.


### Registered witness consumers with compiled indexes

`analyze-witness` accepts `--index FILE --index-sha256 TRUSTED_SHA256` and
`--match POLICY` (default `exact`). The requested policy must match the compiled
index. Registered raw sources are loaded and validated before cache installation;
compiled forms cannot substitute for a missing or changed original source.
Both ordinary modeled analysis and witness analysis reject duplicate index
options and require the index path/digest pair together.

```text
RUSTC_WRAPPER= cargo xtask compile-index data/rewrite/synodal-titlo-observed-model.json o-alypy-titlo-examples exact target/rewrite/witness.idx target/rewrite/upstream/alypy-p003.html
RUSTC_WRAPPER= cargo xtask analyze-witness data/rewrite/synodal-titlo-observed-model.json o-alypy-titlo-examples w-titlo-divine target/rewrite/upstream/alypy-p003.html --index target/rewrite/witness.idx --index-sha256 TRUSTED_SHA256
```

The source pathname is a locally available pinned fixture in this workspace,
not a promise that a clean checkout contains the download. The source bytes
must match the model's declaration. Use the digest from trusted local compilation;
tools executable changes conservatively require recompiling the index.

Constructed executable tests preserve registered source offsets, original
whitespace, unmapped annotation labels and four competing readings. Changing
bytes outside the selected span still fails source validation; updating the
source declaration then makes the old cache incompatible. Case-insensitive
retrieval does not rewrite an uppercase witness or make every retained reading
attested. Actual existing Synodal teaching-witness controls preserve the two
source spans, one annotation link per witness, and byte-identical cold/restored
source documents. These checks establish consumer equivalence and provenance
preservation, not additional linguistic coverage or source authentication.

### Selected legacy noun observations

`migrate-noun-records SOURCE.tsv SEED.json MAPPING.json OUTPUT.json` preserves
selected legacy noun claims in a source-backed model. The seed must have no
registered sources. Mapping records bind the source SHA and explicitly propose
one or more target identities with a rationale. The original class, stems,
source labels, and notes survive as annotation metadata; they do not become
grammatical rules. Lemma observations carry no inferred nominative cell.
Explicit variant weights are preserved separately from the exact surface span.

The first mapping is `data/rewrite/ocs-noun-correspondences.json`; its output is
`data/rewrite/ocs-noun-observed-model.json`. It selects three of 3,493 OCS noun
records, retaining 12 observations including nine cell claims. Two cell claims
are exactly compatible with their proposed model target and three are compatible
under legacy-orthographic matching. These are claim-level counts, not target
counts, precision, or judgments of historical grammaticality. The other 3,490
records are explicitly unselected; there are zero newly validated lexemes.

```text
RUSTC_WRAPPER= cargo xtask migrate-noun-records crates/church-slavonic/lexicon/ocs/nouns.tsv data/rewrite/ocs-o-stem-model.json data/rewrite/ocs-noun-correspondences.json data/rewrite/ocs-noun-observed-model.json
RUSTC_WRAPPER= cargo xtask analyze-witness data/rewrite/ocs-noun-observed-model.json o-lrc-o-stems legacy:чловѣкъ.n:1 crates/church-slavonic/lexicon/ocs/nouns.tsv
```

All 12 witnesses were exercised through cold and restored-index CLI analysis;
the documents were byte-identical, including preserved unmatched claims. All 64
original teaching form/cell rows still generate and retain their joint analysis.
This checks migration and consumer behavior, not new independent grammar evidence.
The adapter bounds input/output bytes, selected records, extracted forms, and
estimated repeated metadata. It publishes one staged model file; the stdout
report is not a second transactional artifact. Whole-process RSS, concurrent
writers, complete lexical migration, and upstream source authentication are not
established by this increment.
