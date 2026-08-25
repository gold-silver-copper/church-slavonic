# The Synodal consumer API (v0.12 phase 4)

The scenario this phase delivers: *given a Synodal passage the registry has
never seen, return for every token its readings (lemma, cell, provenance,
confidence), attested and normative before predicted, in one call, with
stable serialisation.*

## The call

`synodal_church_slavonic_dictionary::analyze_text(text, inflector) ->
TextAnalysis` — one documented public function, doc-tested against Acts 8:30
from the held-out evaluation partition (a passage that is never a source of
reviewed evidence). Each `TokenReading` carries:

- `token`: the print exactly as written, with its byte span;
- `readings`: reviewed `Analysis` values in provenance order (exact
  attestation → irregular override → normative table → productive rule →
  predictions), each naming its lexeme, cell, `AnalysisSource`, confidence,
  evidence ids, `reflexive` flag, and rule trace;
- `predictions`: exploratory segmentation hypotheses (`Prediction`), present
  only when the token has no reviewed reading **and** the inflector's policy
  is `Exploratory`. Never mixed into `readings`.

`TextAnalysis` derives `serde::Serialize`; the JSON shape is the CLI's
`--json` output and is the stable interchange format.

## The CLI

`synodal-dict analyze-text TEXT [--policy POLICY] [--profile PROFILE]
[--json]` (`-` reads stdin). Tokens with no reading say so explicitly;
predictions print with a leading `?`, their split, cell, confidence, and
model id.

## Ergonomics audit

What the scenario audit found and changed:

- There was no passage-level entry point at all: `analyze` handled one word,
  `check_text` returned coverage bookkeeping (gap kinds, summaries) rather
  than a consumer result. `analyze_text` wraps the same analyzer with a
  consumer-shaped, serialisable type.
- Provenance was already visible per reading (`AnalysisSource`, evidence
  ids, trace) and ranking was already provenance order — kept as-is; ranking
  within a token is provenance order *only*. No contextual disambiguator was
  built, deliberately: if a later goal wants context-sensitive top-1 it now
  has a stable analysis API to build on.
- `Prediction` values are `&'static`-labelled and therefore serialise but do
  not deserialise; `TextAnalysis` is `Serialize`-only. Consumers own their
  input types.

## Breaking changes

None. The new items are additive (`analyze_text`, `TokenReading`,
`TextAnalysis`, `prediction` module, `Analysis.reflexive` with a serde
default).
