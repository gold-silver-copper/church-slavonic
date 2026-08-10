# OCS reuse and recension-mapping audit

The existing OCS implementation was audited together with the Ruthenian API.
The safe boundary is design and algorithm reuse, not a shared lexical registry or
an orthographic switch. A shared `historical-slavonic-core` is deliberately
deferred until two independently specified recension implementations demonstrate
a genuinely stable common type boundary.

| Existing component | Classification | Synodal treatment | Reason |
|---|---|---|---|
| `Case`, `Number`, `Gender`, `Person`, `Animacy` shapes | independently specified counterpart | Recreate closed Synodal enums with exhaustive `ALL` arrays | Names overlap, but the valid category products and future extension points belong to the target grammar. |
| OCS noun/adjective ending tables | OCS-specific | Rewrite from Alypy and Synodal golden paradigms | Shared historical origins do not prove identical endings, spelling, accent, or distribution. |
| OCS verb classes and formation enums | OCS-specific | Rewrite with independent Synodal principal-part systems | Existing types intentionally encode OCS aorist and imperfect limitations. |
| `FormSet`, variant selection, cell outcomes | recension-neutral design | Reimplement with per-variant target/provenance fields | The API pattern is sound; the existing concrete evidence and rule types are OCS-bound. |
| Stable IDs | recension-neutral algorithm | Use content-based `SourceId`, `LexemeId`, `EvidenceId`, and `MappingId` with target namespaces | Prevents text-only joins and permits deterministic generation. |
| Trace and evidence model | parameterizable design | Reimplement with Synodal rule IDs and explicit cross-recension stages | A copied OCS trace could falsely imply target authority. |
| Unicode NFC lookup primitive | partially neutral | Reuse the dependency, not the complete contract | Equal-class Church Slavonic combining marks and semantic abbreviations require stricter validation than NFC. |
| OCS normalized lookup | OCS-specific | Separate expanded, lookup, accented, and printed representations | Stripping or conflating marks would lose Synodal lexical and presentation distinctions. |
| Dictionary table-first resolver | recension-neutral algorithm | Preserve precedence with explicit evidence kinds | Exact Synodal evidence must outrank all inherited predictions. |
| OCS generated registries | OCS-specific | Keep separate; access only in the extractor/alignment layer | No OCS surface row may enter Synodal exact-form data. |
| OCS extractor streaming and deterministic writers | recension-neutral engineering | Adapt for Synodal source adapters and quarantine reports | The parsing schemas and legal boundaries differ, but the operational pattern is proven. |
| OCS paradigm builders | parameterizable design | Recreate over the Synodal canonical cell resolver | Guarantees handles, direct calls, and paradigms cannot diverge. |
| OCS abbreviation behavior | unsuitable as authority | Specify from Synodal orthographic and semantic evidence | A blind string contraction cannot distinguish nomina sacra from homographs. |
| Ruthenian lemma-plus-dimensions API | ergonomics reference | Adopt direct calls and resolved handles | It is discoverable and compact. |
| Ruthenian bare-string/total generation | unsuitable | Reject in favor of `Result<FormSet, Error>` | Historical data has real ambiguity, gaps, and provenance obligations. |

## Explicit cross-recension path

No runtime call may transform an OCS row implicitly. The only allowed inherited
path is:

```text
OCS LexemeId + sourced lexical analysis
  -> reviewed RecensionMappingId
  -> independently specified Synodal transformation steps
  -> Synodal principal parts / morphology rule
  -> Synodal accent realization
  -> selected Synodal orthography profile
  -> predicted FormVariant
```

Each stage may return a typed failure. Mapping confidence and generated-form
confidence are separate. `Strict` accepts inherited data only where independent
Synodal evidence establishes the target lexeme and identity mapping; `Productive`
requires a reviewed or calibrated mapping; `Exploratory` may retain uncertain
alternatives.

## Initial alignment gold set

The generated registry contains five accepted mappings and one rejected semantic
false-friend control rather than automatic text joins. It covers explicit
identity (`рабъ`, `градъ`), orthographically transformed (`землѧ`), independently
re-sourced verb principal parts (`нести`), and a calibrated lexical-stem mapping
(`любити`). Each row stores source and target IDs, relation, status, morphology
compatibility, semantic compatibility, evidence IDs, transformations, confidence,
and reviewer note. Separate semantic decisions prevent morphological inheritance
from licensing an unreviewed application meaning.

The current runtime has no dependency on an OCS crate and no OCS surface table in
its target registry. `Productive` uses the reviewed `градъ` class mapping as a
held-out path; `Exploratory` is implemented to retain all non-rejected compatible
mappings as separate variants when the registry grows. Creating a shared
`historical-slavonic-core` remains deferred because the independently implemented
types have not yet demonstrated enough stable common surface to justify it.
