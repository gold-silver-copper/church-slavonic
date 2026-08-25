# Implement Synodal v0.7 Evidence-Backed 70% Top-k Coverage

Continue the Synodal Russian Church Slavonic implementation in the
`church-slavonic` workspace. Increase real-corpus `Strict` top-k coverage from
the completed v0.6 result to at least 70%, while preserving or strengthening
the existing recension, evidence, provenance, precision, abstention, Unicode,
offline-runtime, evaluation-leakage, determinism, and reproducibility
contracts.

Backwards compatibility and breaking semver are not concerns.

Do not commit, push, publish, open or modify a pull request, delete branches,
or otherwise mutate remote state unless the user explicitly requests it.
Preserve unrelated staged, unstaged, and untracked work. The local `.git`
metadata may be read-only and the completed v0.3-v0.6 implementation may appear
as uncommitted work even when the equivalent commit exists remotely. Treat the
current v0.6 runtime, reviewed data, generated artifacts, and audit as the
baseline. Do not reset, discard, or reconstruct that work from a stale local
`HEAD`.

## Locked v0.6 baseline and v0.7 target

The authoritative v0.6 result is defined by the identical pinned corpus,
tokenizer, resolver policy, and orthography profile:

- target recension: `synodal-russian`;
- policy: `Strict`;
- orthography profile: `SynodalLiturgical`;
- tokenizer: `synodal-dictionary-tokenize-v1`;
- 74,130 passages;
- 1,313,344 tokens;
- 57,476 token types;
- source IDs `ponomar-elizabeth-bible-2026-08-09` and
  `wikisource-church-slavonic-bible-2026-08-09` at their pinned revisions;
- 655 reviewed lexemes;
- 655 reviewed senses;
- 2,450 generated exact normative or target-attested forms;
- 1,187 passage-held-out evaluation cells;
- 569,630 top-1 analyzed tokens;
- 853,770 top-k analyzed tokens, or 65.007%;
- 15,394 ambiguous tokens;
- 458,189 unresolved tokens.

At least 70% on the unchanged 1,313,344-token denominator requires at least
919,341 top-k-analyzed tokens. The v0.7 implementation therefore needs a
realized canonical gain of at least 65,571 tokens over v0.6.

Useful intermediate milestones on the locked denominator are:

| Target | Minimum top-k tokens | Gain over v0.6 |
|---|---:|---:|
| 66% | 866,808 | 13,038 |
| 67% | 879,941 | 26,171 |
| 68% | 893,074 | 39,304 |
| 69% | 906,208 | 52,438 |
| 70% | 919,341 | 65,571 |

The current overlap-adjusted marginal artifact contains 1,743 diagnostic
batches and 184,296 unique potential tokens. If every current diagnostic batch
were validly admitted, the counterfactual projection would be 1,038,066
top-k-analyzed tokens, approximately 79.04%. That is a diagnostic ceiling, not
an attainable-coverage claim.

Current modeled marginal recovery is:

| Route | Batches | Overlap-adjusted diagnostic tokens |
|---|---:|---:|
| Ungrouped unknown or exact families | 1,632 | 174,587 |
| Typed abbreviation registry | 77 | 6,417 |
| Spelling or orthographic variants | 32 | 3,121 |
| Reviewed declension or class | 1 | 129 |
| Reviewed principal part | 1 | 42 |

Current evidence-readiness bands are:

| Readiness / effort | Batches | Overlap-adjusted diagnostic tokens |
|---|---:|---:|
| Ready / small | 24 | 1,661 |
| Partial / medium | 144 | 14,307 |
| Partial / large | 81 | 10,102 |
| Weak / large | 1,493 | 158,226 |
| Ready / medium | 1 | 0 |

Of the ready/small set, 14 candidate-unreviewed batches account for at most 843
tokens, nine already-deferred batches account for 772 tokens, and one admitted
diagnostic batch still accounts for 46 unresolved tokens. Do not describe all
1,661 tokens as immediately admissible.

In the current greedy ordering:

- rank 298 reaches 65,600 cumulative diagnostic tokens, leaving effectively no
  rejection or overlap-change buffer;
- rank 375 reaches 80,080;
- rank 433 reaches 90,041;
- rank 508 reaches 100,116.

Use the first 508 batches as the initial evidence-acquisition horizon, not as a
guaranteed recovery set or a stopping boundary. Recovering 65,571 tokens from
that 100,116-token horizon would require approximately 65.5% of its current
diagnostic value to survive evidence review and canonical resolution. Continue
beyond rank 508 whenever rejections, deferrals, splits, zero-gain admissions,
or recomputed overlaps make that necessary.

The canonical coverage report currently classifies 450,945 gap tokens as
unknown lexemes, 16,582 as ambiguity or spelling variants, 7,055 as missing
accent or orthographic metadata, 129 as missing declension or class metadata,
and 60 as missing verb principal parts. These gap totals may overlap statuses
or route proposals and must not be added to marginal-batch totals.

The report's larger route pools currently contain 436,826 ungrouped-unknown
tokens, 13,429 abbreviation-registry tokens, 8,243 spelling-variant tokens,
690 exact-evidence tokens, 129 reviewed-class tokens, and 60
reviewed-principal-part tokens. Explain all differences between route-pool,
gap, raw candidate, family, and overlap-adjusted marginal totals. Never sum
them as if they were disjoint.

These values prioritize evidence work. They do not authorize admitting a
proposal, guessing an identity or cell, weakening `Strict`, stripping
meaningful marks, importing generic OCS morphology as Synodal morphology,
counting a generated form as attested, or using evaluation data to license a
runtime fact.

## 1. Read and reproduce the completed v0.6 state

Before editing, read completely:

- `SYNODAL_V03_CORPUS_DRIVEN_COVERAGE_PROMPT.md`;
- `SYNODAL_V04_MORPHOLOGICAL_FAMILY_COVERAGE_PROMPT.md`;
- `SYNODAL_V05_TOP_K_COVERAGE_PROMPT.md`;
- `SYNODAL_V06_65_PERCENT_TOP_K_COVERAGE_PROMPT.md`;
- `docs/SYNODAL_V03_IMPLEMENTATION_AUDIT.md`;
- `docs/SYNODAL_V04_MORPHOLOGICAL_FAMILY_AUDIT.md`;
- `docs/SYNODAL_V05_TOP_K_COVERAGE_AUDIT.md`;
- `docs/SYNODAL_V06_65_PERCENT_TOP_K_COVERAGE_AUDIT.md`;
- `docs/SYNODAL_CLI_AND_COVERAGE.md`;
- `docs/SYNODAL_MORPHOLOGY.md`;
- `docs/SYNODAL_ORTHOGRAPHY.md`;
- `docs/SYNODAL_DATA_PIPELINE.md`;
- `docs/SYNODAL_REQUIREMENTS.md`;
- `data/synodal/README.md`;
- `reports/synodal-coverage.json` and Markdown companion;
- `reports/synodal-evaluation.json` and Markdown companion;
- `reports/synodal-marginal-recovery.json`, TSV, and Markdown;
- `reports/synodal-v06-review-packets.json`, TSV, and Markdown;
- `reports/synodal-v05-baseline.json`;
- all current Synodal registries, review ledgers, verification ledgers, and
  queues;
- all relevant core, facade, dictionary, extractor, CLI, and `xtask` code;
- applicable repository instructions, including `AGENTS.md`.

Inspect the complete worktree before editing. Identify the intended base and
merge base when available, but do not treat a stale local `HEAD` as the v0.6
data baseline. Preserve unrelated work.

Run the deterministic v0.6 checks needed to prove that the current generated
artifacts match runtime behavior. Confirm the exact source revisions,
partitions, passage count, token count, token-type count, tokenizer,
normalization contract, policy, profile, candidate cutoff, resolver status
definitions, and denominator. Fail rather than compare against drifted input
or a changed meaning of “analyzed.”

Create an immutable machine-readable v0.6 baseline, for example
`reports/synodal-v06-baseline.json`, containing every identity, registry,
coverage, status, gap, route, evaluation, and partition field required for the
v0.7 comparison. Add deterministic generation and `--check` support. The
baseline must be captured before any v0.7 admission changes runtime behavior.

## 2. Preserve the evidence and evaluation boundary

Every new runtime analysis must be licensed by explicit reviewed evidence.
Maintain or strengthen these invariants:

- direct corpus evidence uses an exact pinned source ID and stable citation;
- a target occurrence contains the NFC whole token, not a substring;
- target evidence identifies `synodal-russian` explicitly;
- evaluation passages and passage-overlapping held-out passages never license
  lexical, semantic, accent, spelling, abbreviation, typed-cell,
  principal-part, class, or productive-rule facts;
- rejected, deferred, diagnostic, or candidate evidence never licenses runtime
  facts;
- lexical identity and semantic alignment are reviewed independently whenever
  both are required;
- generic OCS dictionaries and treebanks provide candidates and cross-checks,
  not automatic Synodal identities, cells, accents, paradigms, or rules;
- corpus frequency, shared stems, edit distance, accentless equality,
  transliteration, endings, prefixes, or proximity may rank a proposal but do
  not prove it;
- target occurrence evidence proves only that a printed token occurs unless a
  governed source also supplies the claimed identity or morphology;
- generated output never becomes evidence for itself.

For every admitted analysis, keep separate evidence roles for:

1. lexical identity;
2. semantic identity or sense alignment;
3. target-recension surface occurrence;
4. exact typed morphology or a complete normative table;
5. accent, breathing, combining-mark order, positional letters,
   capitalization, and printed orthography;
6. abbreviation expansion and restrictions where applicable;
7. principal parts, productive class, or transformation rules where
   applicable;
8. passage-disjoint evaluation.

One citation may fill multiple roles only when its governed content explicitly
supports each role. Record that fact rather than silently reusing an occurrence
as morphology evidence.

When existing governed sources cannot support the needed facts, acquire new
sources only through the source-governance pipeline. Record stable source IDs,
versions or revisions, checksums, licenses, recension scope, extraction
boundaries, citations, and permitted evidence roles. Pin and cache all bytes
for deterministic offline reconstruction. Do not add a runtime network or
filesystem dependency.

If evidence supports only one exact surface or cell, admit only that surface or
cell. Keep the rest of the proposed family deferred with explicit blockers.

## 3. Build a v0.7 evidence-acquisition queue

Do not simply replay the v0.6 top-200 review. Many high-frequency packets were
already deferred because their evidence was incomplete. Re-reviewing unchanged
evidence is not progress.

Extend the deterministic packet tooling to distinguish:

- new candidate requiring first review;
- previously deferred with no new evidence;
- previously deferred with newly acquired evidence;
- previously rejected and still blocked;
- admitted but still leaving uncovered sibling surfaces;
- split family requiring member-level decisions;
- zero-marginal or already-covered candidate;
- evidence conflict requiring explicit resolution.

For each packet include:

- stable packet, family, member-candidate, lexeme, sense, and evidence IDs;
- prior v0.5/v0.6 decision and exact blocker;
- all current top-k-uncovered surfaces with exact token and document
  frequencies;
- raw, unique, and overlap-adjusted diagnostic gain;
- cumulative gain in the current ordering;
- representative target passages and all passages excluded as evaluation;
- candidate identities, senses, parts of speech, and typed cells;
- existing runtime analyses and traces for every surface;
- evidence grouped by role and source partition;
- missing roles, contradictions, false-grouping hazards, and assumptions;
- predicted top-1, top-k, ambiguity, and abstention deltas;
- expected review effort and a concrete next evidence-acquisition action;
- final admitted, partially admitted, split, deferred, rejected, false-grouping,
  or superseded decision with reviewer rationale;
- canonical realized delta after regeneration.

Rank first by realizable overlap-adjusted `Strict` top-k gain per missing
evidence cost, then by stable ID. Frequency alone is not readiness. Maintain a
separate exact-surface acquisition queue so high-frequency target spellings do
not disappear inside an incorrect family grouping.

Generate deterministic JSON, TSV, and Markdown artifacts, for example
`reports/synodal-v07-review-packets.*`, with `--check` support. Add tests for
stable IDs and ordering, prior-decision joins, new-evidence detection, family
splits, overlapping surfaces, duplicate source passages, evaluation exclusion,
zero-marginal admissions, and stale artifacts.

Recompute the canonical coverage report, marginal recovery, and packet order
after every material admission wave. Never continue using stale predicted
gains.

## 4. Wave A: complete the genuine ready/small work

Begin with the 14 candidate-unreviewed ready/small spelling batches, currently
worth at most 843 overlap-adjusted tokens:

| Diagnostic family or surface | Current tokens |
|---|---:|
| `нею` | 148 |
| `волꙗ` | 114 |
| `посла` | 95 |
| `даде` | 58 |
| `давати / данъ / дань / даꙗти` | 48 |
| `небесныхъ` | 45 |
| `домъ` | 44 |
| `брашна` | 44 |
| `мꙋжи` | 43 |
| `мꙋдръ` | 42 |
| `вѣчнꙋю` | 41 |
| `рабы` | 41 |
| `человѣки` | 40 |
| `дѣломъ` | 40 |

Validate these names and counts against the regenerated queue before acting.
They commonly lack exact printed-orthography or accent-class evidence. Obtain
that evidence and record exact target cells; do not convert a diagnostic
accentless relation into blanket normalization.

For the nine ready/small batches already deferred, do not reverse the decision
without new evidence that resolves the exact recorded blocker. For the
admitted diagnostic batch that still leaves 46 unresolved tokens, identify the
uncovered sibling surfaces and treat them as new exact decisions rather than
crediting the prior admission again.

For every spelling admission:

- preserve the exact NFC spelling and meaningful marks;
- specify exact identity and every returned typed cell;
- record capitalization and positional-letter restrictions;
- preserve all independently supported alternatives;
- reject accent stripping, fuzzy matching, transliteration matching, and
  cross-lexeme spelling equivalence in `Strict`;
- add negative controls for visually similar unrelated forms and unsafe mark
  removal.

Report predicted and realized gains separately. This wave is a correctness and
queue-hygiene pass; it cannot deliver 70% by itself.

## 5. Wave B: acquire evidence for the highest-frequency exact surfaces

The current 30 highest top-k-uncovered exact surfaces account for 13,685
tokens. Begin with the regenerated equivalents of:

`Заⷱ҇`, `в̾слѣ́дъ`, `сꙋ́дъ`, `заповѣ́да`, `дꙋ́шꙋ`, `ᲂу҆`, `ѹ҆`,
`дꙋша̀`, `вни́де`, `Жре́цъ`, `кнѧ̑зи`, `свидѣ́нїѧ`, `дꙋшѝ`, `кро́вь`,
`горы̀`, `Ѿвѣща́`, `воста̀`, `саꙋ́лъ`, `ча́сть`, `ѕла̑ѧ`, `на́нь`,
`жєны̀`, `сребро̀`, `сотворитѐ`, `а҆дѡнаі̀`, `ꙗ҆зы́цы`, `ча̑да`,
`і҆ꙋ́да`, `сосꙋ́ды`, and `ꙗ҆зы̑ки`.

The surfaces and counts are diagnostic. A surface is not automatically one
lexeme, one sense, one cell, or one family. Before admission establish:

- exact target-recension token occurrence;
- lexical and semantic identity;
- exact typed cell or independently supported alternatives;
- printed accent, breathing, case, positional letters, and capitalization;
- a source-partition witness independent of evaluation;
- a held-out evaluation occurrence selected without using it as runtime
  evidence.

Prefer exact-cell admissions when that is all the evidence supports. Do not
wait for a complete paradigm to recover a well-supported exact cell, and do not
manufacture a complete paradigm to avoid repetitive exact data.

The first 30 surfaces can at most supply roughly the 13,038-token gain needed
for 66%, and only if nearly all survive review. Continue immediately into the
remaining overlap-adjusted packet horizon rather than treating 66% as the
v0.7 stopping point.

## 6. Wave C: review consolidated high-return families

After exact-surface acquisition, review consolidated packets whose identities
are actually supported. Current high-return diagnostics include:

- the `кровъ / кровь` batch, currently 828 tokens;
- `оу / Ꙋ / ꙋ`, currently 544 tokens;
- `вина / вино`, currently 467 tokens;
- `доуша / душа`, currently 530 tokens;
- forms around `вниде`, currently 495 tokens;
- `камꙑ / камень`, currently 459 tokens;
- `злато`, currently 452 tokens;
- the `принес- / принесет-` batch, currently 325 tokens;
- forms around `неже`, currently 340 tokens;
- forms around `ꙗзыкъ`, currently 342 tokens;
- forms around `написати`, currently 225 tokens;
- every higher-return family produced by the regenerated queue.

Do not accept these labels as proved groupings. Split packets whenever member
surfaces differ in identity, sense, part of speech, inflectional system, or
evidence scope.

For nouns and nominal forms, require only the facts claimed by the runtime:

- lemma and sense identity;
- gender, animacy, and number restrictions;
- exact case, number, gender, and form variant;
- declension class only when independently supported;
- stem and suppletive alternants;
- accent and printed variants;
- target and normative citations.

For verbs and participles, require only the facts claimed by the runtime:

- lemma and sense identity;
- exact tense, mood, voice, person, number, gender, case, and form variant;
- aspect only where represented and supported;
- exact principal parts and alternants rather than guessed stems;
- accent and printed variants;
- target and normative citations.

Repeated surfaces prove recurrence, not a paradigm. OCS treebank cells remain
candidate evidence until independently supported for the Synodal target.

## 7. Wave D: expand the typed abbreviation registry

The current abbreviation marginal pool is 6,417 tokens across 77 batches.
Review it as a dedicated evidence lane, beginning with the regenerated
highest-return families, including:

- `заⷱ҇`, currently 585 tokens;
- `мцⷭ҇-`, currently 431 tokens;
- `спⷭ҇ни-`, currently 245 tokens;
- recurring Israel, Jerusalem, Savior, Lord, God, holy, righteous, apostle,
  disciple, prophet, father, mother, church, kingdom, and blessing
  contractions that rank highly after recomputation.

Do not infer an expansion from familiarity or from the target surface alone.
Each admitted abbreviation row must specify and independently support:

- exact marked NFC surface and canonical combining-mark order;
- exact expanded surface;
- stable lexeme and sense identity;
- exact typed cell or all independently supported alternative cells;
- superscript letters, titlo, pokrytie, payerok, breathings, accents, and mark
  ordering;
- capitalization, token position, and contextual restrictions;
- forward expansion and reverse-registry behavior;
- target-recension occurrence and normative expansion or morphology evidence;
- deterministic ranking and ambiguity behavior.

Do not implement blind string replacement. Do not normalize malformed
contractions into valid ones. Add negative controls for missing titla, wrong or
reordered superscripts, deceptive substrings, mixed scripts, wrong position,
unsupported expansions, and untyped cells.

Admit exact contraction-cell pairs when a full abbreviation family is not
supported. Track predicted and realized gain for every row and family.

## 8. Preserve and use genuine ambiguity

The objective is top-k coverage, not forced top-1 uniqueness. When two or more
identities or cells are independently supported for the same surface, return
all supported analyses with stable IDs, complete typed cells, evidence,
provenance, and deterministic ordering.

This is especially important for diagnostics such as `оу / Ꙋ / ꙋ`,
`вина / вино`, `доуша / душа`, and nominal or verbal homographs. A valid
alternative can increase top-k while increasing ambiguity; that is correct and
must not be reported as a top-1 gain.

Do not force one candidate merely because it is frequent. Conversely, do not
add unsupported alternatives to turn an unresolved token into an analyzed
ambiguous token. Every returned alternative must independently satisfy the
evidence boundary.

Report separately:

- top-1 gains and losses;
- top-k gains;
- newly ambiguous tokens;
- resolved ambiguity;
- changed top-1 ordering with before-and-after traces;
- abstention reduction;
- zero-gain correctness admissions.

Already top-k-covered ambiguity is not marginal recovery.

## 9. Add productive behavior only when it is higher-confidence work

The current overlap-adjusted remaining value attributed to reviewed classes and
principal parts is only 171 tokens. Do not prioritize speculative productive
morphology over exact evidence merely because a broad rule appears capable of
covering many unknown tokens.

Add or broaden a class, principal-part mechanism, spelling transformation, or
productive rule only when all of the following hold:

1. a governed Synodal normative source defines the rule and restrictions;
2. input lexemes have reviewed identities and every required metadata field;
3. stems, principal parts, accent behavior, alternants, and exceptions are
   explicit;
4. multiple independent target lexemes exercise the claimed scope;
5. passage-disjoint evaluation covers representative lexemes and cells;
6. negative tests prove unsupported inputs and cells still abstain;
7. exact irregular rows override productive results deterministically;
8. reverse analysis returns the same evidence-qualified interpretations;
9. predicted and realized unique recovery exceed the next practical exact
   packet work;
10. provenance distinguishes prediction from attestation.

Do not derive past stems from present classes, infer aspect or identity from a
surface, choose an aorist automatically, generate participles from an
undifferentiated stem, infer accent from frequency, or copy generic OCS endings
into the Synodal runtime.

## 10. Maintain exact canonical coverage accounting

Coverage is authoritative only when computed by the canonical resolver under
`Strict` and `SynodalLiturgical`. Preserve and test:

```text
top-k analyzed + Cyrillic numerals + top-k-uncovered = total corpus tokens
```

Resolver statuses must remain mutually exclusive. Distinguish at least:

- exact Synodal attestation;
- Synodal normative exact table;
- typed abbreviation expansion;
- evidence-qualified spelling variant;
- Synodal productive rule;
- ambiguous analysis;
- Cyrillic numeral;
- unresolved.

Do not count a probable family, candidate, heuristic, review decision,
generated form, evaluation expectation, or source search hit unless the
canonical runtime returns at least one policy-valid analysis for that exact
token. Do not double-count overlaps, duplicated sources, capitalization
variants, multiple analyses, already-covered ambiguity, or route transitions.

For each wave retain:

- raw, unique, and overlap-adjusted predicted gains;
- canonical realized token-level delta;
- surface-, family-, route-, and status-level attribution;
- top-1, top-k, ambiguity, numeral, top-k-uncovered, and unresolved deltas;
- discrepancies from splits, overlaps, route precedence, invalid groupings,
  Unicode normalization, or unsupported cells;
- remaining exact token deficit to every 66%-70% milestone.

After each wave regenerate, do not hand-edit, all canonical reports and queues.
Add regression tests for every accounting discrepancy discovered.

`check-text --strict` must continue to reject every non-numeral token that
remains top-k-uncovered, including abbreviation and spelling diagnostics.

## 11. Expand held-out evaluation without leakage

Add passage-disjoint evaluation for every new lexeme, sense, exact cell,
abbreviation, table, spelling row, principal part, and productive rule. Select
and register evaluation passages before using source passages for runtime
evidence. Enforce zero overlap mechanically.

Test at minimum:

- expanded and printed exact lookup;
- `Strict`, `Productive`, and `Exploratory` policy behavior;
- deterministic top-1 and complete evidence-qualified top-k;
- genuine lexical and cell ambiguity;
- reverse-analysis and registry round trips;
- exact abbreviation expansion and malformed-mark rejection;
- accent, breathing, positional-letter, capitalization, and mark-order
  behavior;
- unsupported cells, absent metadata, and false family matches;
- malformed combining sequences, missing titla, private-use characters, mixed
  scripts, substring collisions, and hostile Unicode;
- masked-leakage reconstruction for expanded and printed forms;
- overlap-adjusted attribution and status partitions.

No existing v0.6 reviewed exact or normative expectation may disappear from
top-k. Explain all changed top-1 orderings with traces and evidence. Do not
require top-1 equality when genuine supported ambiguity exists.

The evaluation dataset is a regression suite, not runtime evidence and not a
claim of language-wide accuracy.

## 12. Runtime, CLI, reports, and packaging constraints

Keep source discovery, corpus search, extraction, evidence acquisition, review
simulation, report generation, and audits in `xtask`, the extractor, or the
development CLI. Runtime crates must remain deterministic, filesystem-free,
network-free, `no_std`-compatible where documented, and WASM-compatible.

Expose only the additional CLI or API output needed to inspect:

- v0.7 packets and prior decisions;
- exact current and proposed analyses;
- evidence roles, source partitions, blockers, and contradictions;
- resolver traces and route transitions;
- predicted and canonical realized recovery;
- progress and remaining tokens for 66%-70%;
- packets with new evidence versus unchanged deferrals.

Keep registries, queues, reports, and audits byte-deterministic. Add `--check`
support for every generated artifact. Do not package raw corpora, reports,
review queues, references, caches, source locks, or other development artifacts
inside runtime crates.

Update documentation and bounded CI stale-output checks. Keep multi-gigabyte
pinned-source verification in the existing manual or scheduled full-source
workflow.

## 13. Work program and stopping rules

Execute in measured waves:

1. lock and verify the v0.6 baseline;
2. regenerate the v0.7 packet and exact-surface acquisition queues;
3. adjudicate the 14 genuinely unreviewed ready/small packets;
4. acquire independent evidence for the leading exact surfaces;
5. process supported consolidated families;
6. process the typed abbreviation lane;
7. continue through at least the current 100,116-token initial horizon,
   extending beyond it whenever the realized target remains unmet;
8. add productive behavior only when its evidence and marginal value beat the
   next exact work;
9. recompute and verify after every material wave;
10. finish with a complete audit and independent review.

Do not spend repeated review cycles on a packet whose evidence and blocker are
unchanged. Record the unchanged deferral, identify the precise missing source
role, and move to the next realizable packet. Return only when new governed
evidence exists.

The primary completion gate is all of the following:

- at least 919,341 canonical `Strict` top-k-analyzed tokens;
- the identical 1,313,344-token corpus and all locked identity fields;
- no evidence, evaluation-leakage, recension, precision, Unicode, abstention,
  runtime, determinism, packaging, or reproducibility regression;
- all generated artifacts current and mutually consistent;
- all verification and independent-review requirements satisfied.

Secondary objectives are:

- reduce unresolved tokens to at most 392,618, assuming the fixed numeral
  partition and recovery from currently unresolved tokens;
- clear all genuine ready/small unreviewed work;
- materially reduce the top 100 exact uncovered surfaces;
- adjudicate the highest-value abbreviation families with exact blockers for
  every remainder;
- preserve every genuine alternative in top-k;
- leave no admitted row without complete evidence-role and held-out linkage;
- leave no deferred or rejected row without a precise reason;
- make the next post-70% work mechanically rankable.

Continue until either:

1. the canonical full-corpus result reaches at least 919,341 top-k-analyzed
   tokens and every completion gate passes; or
2. every evidence-ready and partial packet in the recomputed horizon, all
   governed source-acquisition options, and enough lower-ranked work to test
   the target have been explicitly exhausted, and the audit proves with exact
   deficits and evidence blockers why 70% cannot be reached without violating
   the contracts.

Condition 2 is an evidence-limited failure report, not successful completion.
Never fabricate evidence, lower the target, change the corpus or denominator,
weaken `Strict`, change the tokenizer or profile, increase candidate cutoffs in
a way that changes the metric, or relabel unresolved tokens to claim success.

Do not stop at 66%, 67%, 68%, or 69%. Those are progress checkpoints only.

## 14. Deterministic verification

Run targeted checks after every implementation wave, then the complete
repository gate. Record commands, results, relevant counts, and expected
warnings in a deterministic v0.7 verification ledger.

Run at minimum:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc
cargo xtask synodal-fixture-bootstrap
cargo xtask synodal-check
cargo xtask synodal-coverage --fixture --offline --check
cargo xtask synodal-coverage --offline --check
cargo xtask synodal-lexical-review-queue --check
cargo xtask synodal-evaluation-queue --check
cargo xtask synodal-family-review-queue --check
cargo xtask synodal-marginal-recovery --check
cargo xtask synodal-v04-audit --check
cargo xtask synodal-v05-baseline --check
cargo xtask synodal-v05-audit --check
cargo xtask synodal-v06-review-packets --check
cargo xtask synodal-v06-audit --check
cargo xtask synodal-v07-baseline --check
cargo xtask synodal-v07-review-packets --check
cargo xtask synodal-v07-audit --check
cargo xtask check-all
cargo xtask guard-witnesses
cargo xtask synodal-guard-witnesses
cargo check -p synodal-church-slavonic-core --no-default-features
cargo check -p synodal-church-slavonic --no-default-features
cargo check -p synodal-church-slavonic-dictionary --no-default-features
cargo check -p synodal-church-slavonic-core --target wasm32-unknown-unknown --no-default-features
cargo check -p synodal-church-slavonic --target wasm32-unknown-unknown --no-default-features
cargo check -p synodal-church-slavonic-dictionary --target wasm32-unknown-unknown --no-default-features
cargo package --list --allow-dirty -p synodal-church-slavonic-core
cargo package --list --allow-dirty -p synodal-church-slavonic
cargo package --list --allow-dirty -p synodal-church-slavonic-dictionary
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic-core
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic
cargo publish --dry-run --no-verify --allow-dirty -p synodal-church-slavonic-dictionary
cargo xtask synodal-bootstrap --offline --cache references/downloads
git diff --check
```

If a v0.7 command does not yet exist, implement its deterministic equivalent
rather than silently omitting it. If an older command name differs, discover
and record the actual command.

Reconstruct the fixture twice from separate empty temporary caches and prove
byte-identical output. The full offline bootstrap must verify every locked
artifact and reproduce reviewed registries, evaluation data, reports, queues,
and audits byte for byte.

Inspect package contents. Confirm that no raw corpora, reports, queues,
references, caches, source artifacts, or unintended generated files ship in a
runtime crate. Expected Cargo dirty-worktree or unused-patch warnings are not
failures, but record them accurately.

Do not claim remote CI or review-thread status unless it was actually inspected.

## 15. Independent review and correction loop

After implementation and local verification, perform a separate full-diff
review against the intended base branch and merge base. Include every relevant
staged, unstaged, and untracked file. If a pull request is in scope, follow all
repository PR completion-gate requirements, including CI and unresolved review
threads.

Review for:

- false lexical identities, senses, typed cells, attestations, and citations;
- wrong-recension, substring-only, circular, or evaluation-backed evidence;
- passage overlap and partition leakage;
- unchanged deferrals incorrectly treated as new admissions;
- unsafe abbreviation expansion or mark normalization;
- over-broad spelling, accent, family, or productive rules;
- Unicode normalization, combining-order, positional-letter, capitalization,
  private-use, and mixed-script hazards;
- hidden ambiguity, forced top-1, duplicate analyses, and weakened abstention;
- coverage double-counting, route overlap, stale ranking, and zero-gain credit;
- nondeterministic or stale generated artifacts;
- runtime filesystem or network dependencies;
- `no_std`, WASM, packaging, and publication regressions;
- missing hostile tests and inaccurate documentation.

Validate every finding against current code and evidence. Fix every confirmed
P0/P1 issue and every safe in-scope lower-severity issue. Rerun affected checks
and perform a final independent review. Continue the review-and-fix loop when a
new confirmed P0/P1 appears. Document rejected findings with concrete
rationale.

## 16. Completion audit

Create `docs/SYNODAL_V07_70_PERCENT_TOP_K_COVERAGE_AUDIT.md` and a deterministic
`cargo xtask synodal-v07-audit --check` command.

The audit must contain:

- exact v0.6 baseline and final v0.7 registry counts;
- proof of corpus, source revision, partition, tokenizer, normalization,
  profile, policy, candidate cutoff, passage, token, type, and denominator
  identity;
- top-1, top-k, ambiguous, numeral, top-k-uncovered, and unresolved counts and
  percentages;
- exact realized gain and margins or deficits at 66%, 67%, 68%, 69%, and 70%;
- route-pool, gap, raw-candidate, and overlap-adjusted totals without
  conflation;
- initial versus final marginal ordering and the effect of recomputation;
- predicted and canonical realized gain for every admitted or partially
  admitted packet;
- prior deferred/rejected decisions revisited, the new evidence that justified
  each change, and unchanged deferrals skipped;
- every family split, overlap, route transition, and zero-gain admission;
- spelling and abbreviation adjudication tables with all exact blockers;
- every new lexeme, sense, exact cell, table, principal part, abbreviation,
  spelling row, transformation, and productive rule with precise evidence IDs
  and citations;
- leading uncovered surfaces and probable families before and after;
- evaluation by policy, morphological system, attestation, regularity, and
  provenance route;
- all expanded and printed top-1 disagreements and changes;
- passage-overlap, masked-leakage, hostile-input, Unicode, reverse-analysis,
  and accounting results;
- deterministic reconstruction and stale-output results;
- formatting, linting, tests, native, WASM, packaging, and publish-dry-run
  results;
- full-diff review scope, findings fixed, findings rejected with rationale, and
  remaining risks;
- CI and review-thread state only when actually inspected;
- the next highest-marginal evidence-ready work after 70%.

Use a machine-readable verification ledger, for example
`data/synodal/v07_verification.tsv`, as the source for the audit's verification
table. The audit generator must reject missing commands, duplicate ledger
entries, failed required checks, inconsistent counts, invalid partitions,
missing evidence linkage, or stale generated prose.

The milestone is complete only when reviewed data, generated registries,
canonical resolver behavior, CLI/API output, coverage and marginal reports,
evaluation, documentation, fixture reconstruction, full offline bootstrap,
verification ledger, independent review, and the v0.7 audit all agree.

Crossing 70% alone is not proof of correctness. Passing tests alone is not
proof of evidence integrity. The successful result is at least 919,341
canonical top-k tokens with every existing contract intact.
