# Deprecation map (rewrite phase 4)

Status: draft, 2026-08-26. Maps the public surface of the old crate families
(`reports/rewrite-api-snapshot.txt`, taken at `v0.14-pre-rewrite`, plus the
free-function export list of `old-church-slavonic/src/lib.rs`) onto the new
`church-slavonic` facade (`crates/church-slavonic`). Dispositions:

- **Replaced** — a named `church-slavonic` function serves the same requests
  today, at 100% of the attested oracle.
- **Planned** — an equivalent is designed but not yet built; the covering
  phase is named (paradigm enumeration, phrases & numeral composition,
  dictionary/analyze, synodal family, orthography).
- **Dropped** — intentionally not carried forward; the rationale and, where
  one exists, the collapsed target signature are named.

Methods on structs (`lemma`, `forms`, `iter`, `len`, `into_rows`, builder
accessors, …) follow the disposition of the struct that owns them and are not
listed one by one.

---

## Crate: `old-church-slavonic` (the fat facade)

### Nouns

| Old item | Disposition |
|---|---|
| `noun(lemma, case, number)` | **Replaced**: `church_slavonic::noun(lemma, case, number) -> Result<String, Error>` + `noun_variants` |
| `noun_by_id` | **Dropped** — registry ids die with the 24 MB registry; homographs are addressed by the deterministic `lemma_2`, `lemma_3` numeric-suffix keys instead |
| `noun_paradigm`, `noun_paradigm_by_id`, `NounParadigm`, `Noun` (handle) | **Planned** — paradigm enumeration (a `*_paradigm` layer enumerating all cells of one lexeme is designed but not built) |
| `noun_with` | **Dropped** — the caller-override channel is one of the five override channels the rewrite collapses into residue-table → rule-kernel |

### Adjectives

| Old item | Disposition |
|---|---|
| `long_adjective(lemma, case, number, gender, animacy)` | **Replaced**: `adjective(lemma, case, number, gender)`. The `Animacy` parameter is intentionally gone: for every attested cell the animate and inanimate variant lists are byte-identical, so the parameter promised a distinction the data does not make |
| `short_adjective(...)` | **Replaced**: `short_adjective(lemma, case, number, gender)` |
| `long_only_adjective` | **Replaced**: served through `adjective` (long-only lexemes simply have no short cells; `short_adjective` returns `Underdetermined`) |
| `adjective`, `adjective_by_id`, `adjective_with`, `adjectival_form` | **Dropped** — id-addressed, override-channel, and enum-dispatch entry points collapse into the two lemma-keyed functions above |
| `adjective_paradigm`, `adjective_paradigm_by_id`, `AdjectiveParadigm`, `Adjective` (handle) | **Planned** — paradigm enumeration |
| `adjective_comparatives`, `comparative_citation`, `comparative_citation_by_id`, `comparative_with`, `comparative_paradigm_with`, `ComparativeParadigm` | **Dropped** — the comparative-exclusion decision: the oracle stores comparative *citations* only, carrying unpredictable lexical facts (suppletion, suffix grade), not a productive stem; excluded from the pilot API and accuracy denominator |
| `absolute_superlative_adverb_with` | **Replaced**: `phrases::absolute_superlative` / `phrases::short_absolute_superlative(lemma, case, number, gender, order)` — the invariant `ѕѣло` phrase over the pilot's lemma-keyed adjective (the old function took a bare `AdjectiveLexeme`); the long/short form dimension became a function pair, per the ruthenian rule. Differential-gated against the old facade's lemma-keyed adjective joined per the same order convention |
| `pre_superlative_with`, `relative_superlative_with` | **Dropped** — with the comparatives (`relative_superlative_with` heads a `ComparativeLexeme`); may return as data-backed derivations if the comparative citations are ever tabled, but no phase currently covers them |

### Verbs (finite and non-finite)

| Old item | Disposition |
|---|---|
| `present`, `imperfect`, `aorist`, `imperative` (lemma, person, number) | **Replaced**: same names, `-> Result<String, Error>` + `*_variants` |
| `l_participle(lemma, gender, number)` | **Replaced**: same name |
| `infinitive`, `supine`, `verbal_noun` (lemma) | **Replaced**: same names (citation forms) |
| `finite(lemma, tense, person, number)`, `finite_by_id`, `finite_verb`, `finite_verb_with`, `FiniteTense` dispatch | **Dropped** — per the ruthenian rule, an enum that selects a paradigm becomes a function: call `present`/`imperfect`/`aorist` directly |
| `verbal_noun_form(lemma, case, number)`, `verbal_noun_form_by_id`, `verbal_noun_paradigm`, `VerbalNounParadigm` | **Dropped** — the oracle attests the verbal noun as its nominative-singular citation only; `verbal_noun(lemma)` is the honest surface. A declined verbal noun would need new attestation, which no phase currently supplies |
| `present_active_participle`, `present_passive_participle`, `past_active_participle`, `past_passive_participle` (returning `Participle` handles) | **Replaced**: same names as citation-derivation functions returning `Result<String, Error>` (masculine nominative singular, short) + `*_variants` |
| `participle`, `participle_by_id`, `participle_with`, `participle_citation(_by_id)`, `Participle` declined cells (`.long`/`.short`) | **Replaced**: `participle(lemma, kind, case, number, gender, form)` + `participle_variants` — one declined-cell entry over the derivation: the per-kind stem feeds the core adjective declension machinery. `kind` and `form` stay parameters (they index within one adjective-declension machine; the four kind-named citation functions remain the derivation shorthand, `present_active_participle(lemma)` = the short masculine nominative singular). The `Animacy` parameter is gone per the pilot adjective convention (inanimate convention). No attested oracle exists (extraction excluded declined participles as not safely attributed), so resolution is: attested/metadata citation precedence on the citation cell, then the reviewed verb-family kernel (`old_church_slavonic_core::verb::reviewed_verb_lexemes`, moved to core from the resolver's `ReviewedVerbProfile` composition), then principal-part metadata — differential-gated at 100% against the old facade's declined-participle output in `cargo xtask rewrite-pilot-accuracy` (citation cells are gated as self-consistency with the attested-first citation functions, with old-facade agreement reported) |
| `participle_paradigm(_by_id)`, `ParticipleParadigm` | **Replaced**: `participle_paradigm(lemma, kind, form) -> AdjectiveParadigm` — the declension grid of one participle system, self-consistency-gated cell by cell against `participle_variants` |
| `*_from_dictionary_metadata` (`finite_verb_`, `imperative_`, `l_participle_`, `participle_`, `verbal_noun_`), `verb_metadata_by_id`, `DictionaryVerbMetadata`, `VerbStemMetadata`, `VerbSystemMetadata`, `SourcedMetadata`, `NormalizedVerbMetadataField`, `from_normalized_fields`, `*MetadataAnalysis` structs | **Dropped** — the dictionary-metadata channel is folded into the facade's generated principal-part metadata table; there is one resolution precedence (residue → kernel), not five |
| `present_paradigm(_by_id)`, `finite_paradigm(_by_id)`, `imperative_paradigm(_by_id)`, `l_participle_paradigm(_by_id)`, `VerbParadigm`, `FiniteVerbParadigm`, `ImperativeParadigm`, `LParticipleParadigm`, `Verb` (handle) | **Planned** — paradigm enumeration |

### Pronouns and determiners

| Old item | Disposition |
|---|---|
| `personal_pronoun`, `personal_pronoun_with`, `personal_pronoun_by_id` | **Replaced**: `pronoun(person, number, case)` (`Person::Third` returns `Underdetermined`; the third person is the anaphoric series) |
| `reflexive_pronoun(case, selection)` | **Replaced**: `reflexive(case)` — the attested number dimension is fully degenerate, and the `PronounFormSelection` policy enum is gone (variants come back ordered, primary first) |
| `anaphoric_pronoun` | **Replaced**: `anaphoric(case, number, gender)` |
| `relative_pronoun`, `interrogative_pronoun`, `gendered_pronoun(_by_id)`, `regular_pronominal`, `irregular_agreeing`, `pronominal_form`, `pronominal_with`, `pronoun(lemma, ...)` | **Replaced**: `pronoun_form(lemma, case, number, gender)` — one lemma-keyed entry point over the reviewed identity kernels; the per-family functions and identity enums are internal dispatch now |
| `phrases::interrogative_pronoun_family(identity, case, spec)` | **Replaced**: `phrases::pronominal_family(lemma, case, prefix, postpositive, direct_to, preposition)` (+ `_variants`) — lemma-keyed (`къто`/`чьто`) instead of identity-keyed; the §316 validation and token composition moved into `old_church_slavonic_core::pronoun` (`validate_pronominal_family_spec`, `canonical_cyrillic_preposition`, `compose_pronominal_family_tokens`), with the fat facade delegating. Ill-formed specs are the typed `Error::UnsupportedPhrase`; differential-gated over identities x cases x every formative combination |
| `phrases::pronominal_family_with(base: FormSet, ...)`, `single_token_pronominal_family_with` | **Dropped** — `FormSet`-generic plumbing; the pilot's lemma-keyed `pronominal_family` covers the consumer surface, and direct kernel callers use the core composer |
| `determiner`, `determiner_by_id`, `determiner_with`, `determiner_identity` | **Replaced**: `determiner_form(lemma, case, number, gender)` |
| `pronoun_paradigm(_by_id)`, `personal_pronoun_paradigm(_by_id)`, `gendered_pronoun_paradigm(_by_id)`, `determiner_paradigm(_by_id)`, `Pronoun`/`Determiner` handles, `PronounParadigm`/`PersonalPronounParadigm`/`GenderedPronounParadigm`/`DeterminerParadigm` | **Planned** — paradigm enumeration |

### Numerals

| Old item | Disposition |
|---|---|
| `numeral(lemma, case, number)`, `numeral_by_id`, `gendered_numeral(_by_id)`, `ordinal_numeral` | **Replaced**: `numeral_form(lemma, case, number, gender)` (gender is a key for `прьвъ`, ignored for the bare-cell cardinals) |
| `cardinal`, `cardinal_numeral_identity`, `ordinal_numeral_identity`, `ordinal` | **Dropped** — identity-enum entry points; the lemma-keyed `numeral_form` is the surface |
| `collective_numeral(_identity)`, `fractional_numeral(_identity)`, `indefinite_numeral(_identity)`, `cardinal_magnitude`, `distributive` | **Planned** — phrases & numeral composition (these classes have no attested cells in the pilot oracle; they ride on the numeral-system port) |
| `compound_cardinal`, `compound_cardinal_with_one`, `compound_cardinal_with_options`, `compound_cardinal_paradigm`, `compound_cardinal_paradigm_with_one`, `compound_cardinal_paradigm_with_options` | **Replaced**: `numeral(value: u64, case, gender, animacy) -> Result<String, Error>` and `numeral_variants` (values 1–10,000, the evidential boundary; out of range is `Error::ValueOutOfRange`). The `_with_one` / `_with_options` axes turned out to select only lexical/orthographic doublets of the same construction (`ѥдинъ`/`ѥдьнъ`, `тꙑсѫщи`/`тꙑсѧщи`), so per the ruthenian rule they are dropped outright — no options struct; the pilot serves the defaults, and correlated structural analyses plus token doublets surface through `numeral_variants`. The composition kernel moved into `old-church-slavonic-core::numeral` (`cardinal`, `compound_cardinal`, `distributive_cardinal`, `compose_cardinal_analyses`), with the fat facade delegating; `cargo xtask rewrite-pilot-accuracy` gates a 100% differential sweep against the old facade |
| `distributive_cardinal` and its five `_with_one`/`_with_options`/`_paradigm*` variants, `DistributiveCardinalParadigm`, `DistributiveCardinalOutcome` | **Replaced**: `distributive_numeral(value, gender, animacy)` and `distributive_numeral_variants` — the distributive (`по` + dative) is a different construction, so it is a separate function per the ruthenian rule, not an options field; the preposition fixes the case, so there is no case parameter. Its `_with_one` / `_with_options` axes are dropped like the cardinal's; the paradigm structs stay with the fat facade |
| `compound_ordinal`, `compound_ordinal_paradigm`, `CompoundOrdinalParadigm`, `CompoundOrdinalOutcome`, `CompoundCardinalParadigm`, `CompoundCardinalOutcome` | **Planned** — phrases & numeral composition (`ordinal(value, case, number, gender)` counterpart of the collapsed cardinal entry) |
| `numeral_paradigm(_by_id)`, `cardinal_numeral_paradigm`, `ordinal_numeral_paradigm(_identity)`, `collective_numeral_paradigm(_identity)`, `fractional_numeral_paradigm(_identity)`, `indefinite_numeral_paradigm(_identity)`, `gendered_numeral_paradigm(_by_id)`, the `*NumeralParadigm` structs, `Numeral` handle | **Planned** — paradigm enumeration |
| `reviewed_cardinal_magnitude`, `reviewed_cardinal_numeral`, `reviewed_collective_numeral`, `reviewed_determiner`, `reviewed_fractional_numeral`, `reviewed_indefinite_numeral`, `reviewed_ordinal_numeral` | **Dropped** — the reviewed-lexeme review process becomes research tooling, not public API |

### Analytic / periphrastic constructions

All kept constructions live in `church_slavonic::phrases`, return the
space-joined phrase as `Result<String, Error>` with a `*_variants`
companion (odometer over token-level variant lists, primary first), and are
differential-gated against this crate's phrase layer inside
`cargo xtask rewrite-pilot-accuracy` at 100% (agreement on rejected cells
included).

| Old item | Disposition |
|---|---|
| `copula(series, person, number)` | **Replaced**: the series enum selects a paradigm, so it became six total functions — `copula_present`, `copula_future`, `copula_imperfect`, `copula_aorist`, `copula_conditional`, `copula_conditional_aorist` `(person, number) -> String` — over the core's reviewed `CopulaSeries` tables |
| `da_imperative(lemma, person, number)` | **Replaced**: same name and key, `-> Result<String, Error>` |
| `perfect(lemma, person, number, gender, order)` | **Replaced**: same name and key |
| `pluperfect(lemma, …, auxiliary, order)` | **Replaced**: the `PluperfectAuxiliary` enum became the functions `pluperfect` (imperfect `бꙑти`), `pluperfect_aorist`, and the three-token `pluperfect_perfect` |
| `future_perfect(lemma, person, number, gender, order)` | **Replaced**: same name and key |
| `conditional_optative`, `da_conditional_optative` (…, `auxiliary`, `order`) | **Replaced**: the `ConditionalAuxiliary` enum became function pairs — `conditional_optative` / `conditional_optative_aorist` and `da_conditional_optative` / `da_conditional_optative_aorist` |
| `infinitival_future(lemma, auxiliary, reference_tense, person, number, order)` | **Replaced**: `infinitival_future` (present reference) plus `infinitival_future_imperfect` / `infinitival_future_aorist` (past reference; `имѣти`/`хотѣти` only, per `FutureInfinitiveAuxiliary::licensed_for_past_reference`, now a core fact — anything else is `Error::UnsupportedPhrase`). The auxiliary stays a parameter: it is a lexical index within one construction |
| `impersonal_predicate(identity, tense)` | **Replaced**: lemma-keyed `impersonal_present` / `impersonal_imperfect` / `impersonal_aorist(lemma)` for `достоꙗти` and reflexive `мьнѣти` (the particle `сѧ` stays an independent token in the joined text). Resolution is attested residue row first, reviewed impersonal lexeme second — the same precedence as the old dictionary-first dispatch |
| `elliptical_conditional_optative(lemma, number, gender)` | **Dropped** — the construction is exactly the bare l-participle; call `l_participle(lemma, gender, number)` |
| `analytic_passive(lemma, kind, cell, person, number, auxiliary, order)` | **Replaced**: the `PassiveAuxiliary` enum selected the copular series, so it became the functions `analytic_passive` (present `ѥс-`), `analytic_passive_imperfect`, `analytic_passive_aorist`, `analytic_passive_future`, `conditional_passive` (`би-`), `conditional_passive_aorist` (`бꙑ-`), each `(lemma, kind, person, number, gender, order)`. The caller-supplied `AdjectiveCell` is gone: the old function rejected everything but the short nominative subject-agreeing cell, so the pilot derives it from `number`/`gender` and only the free dimensions remain. A non-passive `kind` is `Error::UnsupportedPhrase` |
| `conditional_passive(…, auxiliary, order)` | **Replaced**: `conditional_passive` / `conditional_passive_aorist` (see above — the conditional arms of one function family over the copular series) |
| `participial_future(lemma, kind, cell, person, number, order)` | **Replaced**: `participial_future(lemma, kind, person, number, gender, order)` — agreeing *active* participle + future `бѫд-`; a passive `kind` is `Error::UnsupportedPhrase`. All three constructions are differential-gated at 100% against this crate's phrase layer over verbs x kinds x person x number x gender x auxiliary x order |

### Script, accent, lookup, plumbing

| Old item | Disposition |
|---|---|
| `reconstruct_accent`, `realize_glagolitic`, `realize_glagolitic_variants`, `transliterate_glagolitic_to_cyrillic` | **Planned** — orthography (`church-slavonic-orthography`, the deprioritised remainder of phase 2; `Recension` becomes a parameter there) |
| `lookup`, `from_id`, `form(_by_id)`, `dictionary_form_by_id`, `dictionary_paradigm_by_id`, `DictionaryParadigm`, `resolve`, `cell`, `citation`, `expanded_citation`, `syncopated_citation` | **Dropped** — registry-id and registry-handle access dies with the registry; lemma + numeric-suffix keys are the only addressing scheme. Dictionary-shaped lookup returns in the dictionary/analyze phase under the dictionary crate, not the inflection facade |
| `InflectionResult`, `ParadigmLookupError`, `CellOutcome`, `error`, `failures`, `successes`, `into_parts` | **Replaced** (in shape): the facade's typed `Error { UnknownLemma, Underdetermined }` with `Result<String, _>` / `Result<Vec<String>, _>`; the outcome-accumulator machinery is gone |

## Crate: `old-church-slavonic-core`

The pilot facade **depends on this crate directly** — its rule kernels, identity
enums, and lexeme types are the kernel half of the residue→kernel precedence.
Nothing here is dropped by the pilot itself; the dispositions describe the
phase-2 continuation.

| Area | Disposition |
|---|---|
| Grammar enums `Case`, `Number`, `Gender`, `Person` (and `Animacy`) | **Replaced** — moved to `church-slavonic-core::grammar` with dual `code()`/`abbrev()` spellings; the facade re-exports them |
| Rule kernels (`decline_*` family, `present`/`aorist`/`imperfect`/`imperative`/participle formations, `NounClass`, `AdjectiveClass`, `VerbClass`, formation enums, lexeme structs `NounLexeme`/`AdjectiveLexeme`/`VerbLexeme`/`PronominalLexeme`/`DeterminerLexeme`, identity enums `PersonalPronounIdentity`, `CardinalNumeralIdentity`, `OrdinalNumeralIdentity`, `StandardPronominalIdentity`, `InterrogativePronounIdentity`, `IrregularAgreeingIdentity`, unique/irregular family members, Polivanova classifications) | **Planned** — merge into `church-slavonic-core` per-POS modules (phase 2 continuation); until then they remain public in this crate and reachable, but new code should call the facade |
| `PredictedForm`, `RuleStep`, `RuleId`, `FormSet`, `FormVariant`, trace machinery | **Planned** — trace/result unification with the synodal `RuleTrace` model is deliberately deferred until the rule kernels merge (the two `RuleId` models are semantically different); the provenance idea is explicitly preserved |
| Accent (`AccentRule`, `AccentParadigm`, `ReconstructedAccent`, `AccentEvidence`, placement/scope enums), Glagolitic (`Script`, `GlagoliticProfile`, `Transliteration*` family, `detect_script`, `transliterate_glagolitic_to_cyrillic`) | **Planned** — orthography crate |
| Phrase machinery (`PhraseToken`, `RealizedPhrase`, `PhraseOrder`, `PhraseRole`, `CardinalPhraseAnalysis`, `OrdinalPhraseAnalysis`, `RealizedCardinal`, `RealizedOrdinal`, `RealizedDistributiveCardinal`, `CardinalCompositionOptions`, `NumeralGovernment`, compound-ordinal constants) | **Planned** — phrases & numeral composition (these types are the intended vocabulary of the collapsed `numeral(value, …)` entry). `PhraseOrder` is already re-exported by the pilot's `phrases` module, along with `PronominalPrefix`/`PronominalPostpositive`/`DirectToTreatment`/`FutureInfinitiveAuxiliary` |
| Copula/analytic types (`CopulaSeries`, `CopulaVariant`, `AnalyticConstruction`, auxiliary enums) | **Planned** — phrase-layer kernel vocabulary; the pilot's phrase functions consume `CopulaSeries::forms` and the moved §316 composer (`pronoun::validate_pronominal_family_spec` / `compose_pronominal_family_tokens` / `canonical_cyrillic_preposition`) directly, so the enums stay internal to the kernel rather than pilot API |
| `InflectionError`, `InflectionWarning`, `VariantPolicy`, `VariantSelectionError`, warning plumbing | **Replaced** (in shape) by the facade `Error`; the warning channel does not survive — defects are typed errors, not warnings |
| `MetadataField`, `MetadataProvenance`, `MetadataEvidence`, `FormSource`, `FormAnalysis`, `classify_source_*`, family/source-member structs | **Dropped** from the published surface — extractor-side vocabulary; the extractor leaves the published workspace in phase 5 |

## Crate: `old-church-slavonic-dictionary`

| Area | Disposition |
|---|---|
| `lookup`, `search`, `SearchOptions`, `SearchResult`, `Sense`, `Example`, glosses/tags/topics accessors, `SOURCE_*` constants | **Planned** — dictionary/analyze (`church-slavonic-dictionary`, loading a compact binary artifact instead of generated `.rs`) |
| `analyze_token`, `analyze_dictionary_form`, `analyze_generated_form`, `analyze_example_token`, `check_text`, `TokenAnalysis`, `TextReport`, `TextTokenAnalysis`, `MatchKind`, `TextTokenStatus` | **Planned** — dictionary/analyze; `analyze_text` with attested-before-predicted ordering is the explicitly preserved headline entry point |
| `validate_vocabulary_tsv`, `VocabularyIssue*`, `VocabularyReport` | **Dropped** — data-pipeline validation moves out of the published surface with the extractor |
| `inflection_lexeme_id` | **Dropped** — registry ids are gone; the dictionary links to lexemes by lemma + numeric-suffix key |

## Crates: `synodal-church-slavonic`, `synodal-church-slavonic-core`

The synodal recension is not in the pilot at all; its entire consumer surface
is **planned — synodal family** (re-serving synodal inflection through the
same residue→kernel silhouette once the OCS pilot's approach is generalised),
except where noted:

| Area | Disposition |
|---|---|
| `Inflector`, `InflectorBuilder`, `LexemeSpec` / `NounSpec` / `VerbSpec(Builder)` / `AdjectiveSpec` / `PronounSpec` / `DeterminerSpec` / `NumeralSpec`, `LexemeProvider`, `StaticLexemeProvider`, `InMemoryLexemeProvider` | **Planned** — synodal family; the plan explicitly keeps and promotes the `Inflector`/`*Spec` shape as the advanced/configurable layer, with the free functions as thin wrappers over a default `Inflector` |
| `SpecifiedForm`, `ProviderLexeme::with_exact_form` / `exact_forms()`, `*Spec::with_irregular_form`, `VerbSpecBuilder::irregular_form` | **Dropped (landed, v0.14 rewrite phase)** — the provider-exact and caller-irregular override channels are removed from the runtime and the public API; exact surface forms now live only in the generated merged irregular table (the extractor folds `irregular_overrides.tsv` provenance into the `exact_forms.tsv`-fed table), consulted before typed defects and the rule kernel. `homonymy_allowlist.tsv` was kept through the rewrite (it injected no forms) and later deleted with its sole consumer, the wave-era admission preflight (gold-gate retirement, 2026-08-26) |
| POS free functions (`noun`, `adjective`, `short_adjective`, `long_adjective`, `participle`, `pronoun`, `determiner`, `numeral`, `present`/`aorist`/`imperfect`/`imperative`/`infinitive`/`l_participle`/participles, `supine`, `verbal_noun*`) | **Planned** — synodal family, converging on the same names the pilot facade now fixes |
| Periphrastic/analytic surface (`analytic_passive*`, `perfect*`, `pluperfect*`, `conditional*`, `compound_future*`, `future*`, `optative`, `periphrastic_tense`, `semi_auxiliary_periphrasis*`, `modal_conditional_*`, `copula_ellipsis`, enclitic/particle functions, formation enums) | **Planned** — phrases (shared phrase layer, `Recension` as a parameter) |
| Numeral composition (`cardinal(_with)`, `ordinal(_with)`, `fraction*`, `multiplicative_krat*`, `repeated_distributive*`, `compose`, `CompoundNumeralCell`, `RealizedCardinal`/`RealizedOrdinal`, government types) | **Planned** — phrases & numeral composition (same collapsed `numeral(value, …)` target as the OCS side) |
| Core grammar enums (`Case`/`Number`/`Gender`/`Person` etc.) | **Replaced** — already re-exported from `church-slavonic-core` (phase 2 slice 1, landed) |
| Typed defects (`Error`, `ErrorCode`, `DefectiveCell`, `HistoricallyAbsent` vs `EvidenceIncomplete`), `RuleTrace`/`TraceStep`, evidence types | **Planned** — kernel unification carries these into `church-slavonic-core`; explicitly preserved ideas |
| Orthography/accent (`OrthographyProfile`, accent enums, `BreathingRule`, `PositionalRule`/`PositionalParadigm`, `apply_initial_presentation`, `normalize_lookup*`, collation, `format_cyrillic_numeral`/`parse_cyrillic_numeral`, `transliterate`, `Romanization`) | **Planned** — orthography crate, with `Recension { OldChurchSlavonic, Synodal }` as a parameter |
| `Recension`, `RecensionMapping`, `recension_mapping(s)`, `recension_alignments`, `recension_conflicts`, `recension_transformations`, `MorphologyAlignment`, `SemanticAlignment`, `common_source_recension`, alignment/transformation summaries | **Dropped** — the plan deletes the `RecensionMapping` bridge machinery rather than porting it; recension becomes an orthography-layer parameter |
| Registry plumbing (`REGISTRY_FINGERPRINT`, `registered_lexeme`, `irregular_overrides`, `exact_forms`, `irregular_verb_inventory`, `missing_metadata*`, `capabilities*`, `contract*`, `batch`/`BatchRequest`/`BatchResult`, `*_by_id`, summary structs, `all_system_paradigms`, `system_paradigm*`) | **Dropped** — the five override channels collapse into residue-table + lemma markers; registry ids and fingerprint-gated batch access die with the registry (the fingerprint *idea* survives as the xtask stale-binary guard) |
| Validation (`validate_*_lexeme`, `NormalizationReport`, `Assumption`/`Contradiction`/`Confidence`/`Evidence` review vocabulary) | **Dropped** — review process becomes research tooling |

## Crate: `synodal-church-slavonic-dictionary`

| Area | Disposition |
|---|---|
| `analyze`, `analyze_text`, `analyze_with`, `analyze_profile`, `lemmatize(_with)`, `tokenize`, `lookup(_all)`, `search`, `search_gloss`, `check_text`, `Entry`, `Sense`, `Analysis`, `Analyzer(Cache)`, `TextAnalysis`/`TextReport`/`TextToken*`, `TokenReading`, `Prediction`, `predict(_under)`, `spelling_candidates`, `concordance`, the `synodal-dict` CLI (`CliContext`, `run`) | **Planned** — dictionary/analyze: these are the keep-list by name (`analyze_text`/`lookup`/`search`/`lemmatize`), re-served over a binary artifact with the 4,137-line `coverage.rs` and 3,487-line `lib.rs` split into focused modules |
| `analyze_cardinal_word*`, `CardinalWordAnalysis` | **Planned** — dictionary/analyze (riding on the phrases numeral layer) |
| Coverage/research surface (`coverage`, `coverage_with_type_holdout`, `held_out_*`, `marginal_recovery_report`, `project_surface_counts`, `gaps_tsv`, `uncovered_frontier_tsv`, `Coverage*` structs, `GapKind`/`GapRecord`/`GapContext`/`GapOccurrence`, `RecoveryRoute`/`Recovery*` batches, `ReviewQueueItem`, `ReviewEffort`, `GENERALISING_STATUSES`, `MEMORISING_STATUS`, `SEGMENTATION_MODEL`, `families`/`FamilyId`/`FamilySummary`/`show_family_by_id`, `semantic_alignments`, `lint_vocabulary*`, `VocabularyManifest`) | **Dropped** — wave/holdout/coverage research tooling moves to `tools/research/` outside the published workspace (phases 1 and 5); it stays runnable but is not API |
| `REGISTRY_FINGERPRINT`, `analysis_cells_by_id`, `lookup_by_id` | **Dropped** — with the registry |

---

## Migration sequencing

1. **Now (pilot)** — `church-slavonic` ships alongside the old crates;
   nothing is deprecated yet. New code targets the facade for the six
   attested POS surfaces.
2. **Phase 4 completion** — paradigm enumeration, phrases & numeral
   composition (including the collapsed `numeral(value, case, gender,
   animacy)` + one options struct), the orthography crate, and the
   dictionary/analyze port land; the synodal family is re-served through the
   same silhouette.
3. **Final release of the old names** — each old crate gets one last release
   that depends on the new crates and re-exports:
   - `#[deprecated(note = "use church_slavonic::…")]` re-exports where the
     signature survives unchanged (the verb tense functions, `noun`,
     `short_adjective`, `infinitive`/`supine`/`verbal_noun`, the participle
     citations);
   - `#[deprecated]` thin wrappers where only the shape changed
     (`long_adjective` dropping `Animacy` → `adjective`; `personal_pronoun`
     → `pronoun`; `reflexive_pronoun` dropping the selection enum →
     `reflexive`; `InflectionResult` → `Result<Vec<String>, Error>`);
   - a README pointer, not a re-export, for everything Dropped above
     (registry ids, `*_by_id`, the `_with_*` override channels, the six
     `compound_cardinal*` and six `distributive_cardinal*` variants, the
     recension-mapping bridge, the review/coverage tooling) — these have no
     signature-compatible target by design.
4. **Phase 5** — once the new crates pass the full oracle replay and the CI
   gates (per-POS accuracy ≥ baseline, ≤2 MB facade data, max-file-size
   lint, `cargo public-api` diff review), the old crate families are deleted
   from the workspace and `v1.0.0-alpha` is tagged. The deprecated releases
   remain on crates.io as the migration bridge.

---

## Merge phase 5, slice 1 (2026-08-26): the recension dimension

`church-slavonic` 0.3.0 (unpublished) gains the recension axis per
`docs/UNIFIED_FACADE.md`: `recension(Recension) -> RecensionScope`, a
scoped handle resolving lemmas through the shared identity table
(abstract keys authoritative, native keys still working) and realizing
nouns, adjectives, the finite tenses, the imperative, the l-participle,
and the infinitive in either attested recension, each with `_variants`
and paradigm-enumeration companions. The OCS free functions are unchanged
and are the delegation target of the OCS scope. Typed errors:
`UnsupportedRecension`, `UnidentifiedLemma`, `NotInRecension`.

`church-slavonic-dictionary` 0.2.0 (unpublished) gains `lookup_in` /
`RecensionSense` (senses reachable across recensions via the identity
table, marked with their provenance recension) and `lemmatize_in` /
`RecensionReading` (recension-tagged readings; the Synodal index inverts
the facade's Synodal-scope paradigm enumeration, accent-blind).

Still with the old names (dispositions above unchanged):
`synodal-church-slavonic`'s phrase/numeral/orthography surfaces, the
closed classes, the supine/verbal-noun/participle recension routes, and —
last of all — `synodal-church-slavonic-dictionary`'s `Inflector`-backed
analyze layer, blocked by the accent asymmetry and the gap burn-down
state (`docs/UNIFIED_FACADE.md` §5 has the merge order).
