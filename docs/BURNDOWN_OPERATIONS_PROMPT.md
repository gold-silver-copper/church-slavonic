# Prompt: run the gold-gap burn-down at machine speed

You are working in the church-slavonic workspace. This prompt changes HOW
`docs/GOLD_GAP_BURNDOWN_PROMPT.md` is executed, not what it demands: the
contract, the class order, and every gate stay exactly as written there.
Read that prompt and `docs/SYNODAL_DATA_PIPELINE.md` ("The gold-gap inner
loop") first.

## The diagnosis this fixes

Slice 0 made the compute loop fast (full replay 2–3 s, 20 admissions in
21 s) and then the program stayed slow anyway, for five reasons that are
all orchestration, not computation:

1. One agent per class, serially, an hour each — because every class was
   made to wait on the previous one over a shared gap file.
2. The battery run twice per slice (agent, then orchestrator): ~4 minutes
   of duplicated verification per commit.
3. Every fresh agent spends 10–15 minutes re-reading before its first
   edit, then writes a long report.
4. Agents going idle mid-battery and being noticed 30 minutes later.
5. The mechanical bulk of the gap (6,652 already-fitting clusters, 17,847
   cells) never started, because it sat behind the judgment classes.

The artificial serialisation is the big one. Admissions are append-only
data rows; the gap file is *regenerated*, never merged; the registry
artifact is rebuilt from the data. Nothing about the work requires
streams to wait for each other.

## The operating model

### Streams, not slices

Work runs as parallel **streams**, each in its own git worktree with its
own target dir and registry artifact, each owning a disjoint slice of
inputs:

| stream | input it owns | shape |
|---|---|---|
| `bulk` | token `unregistered-lemma` clusters `propose` already marks fit | unattended loop, no LLM |
| `alypy` | paradigm oracle rows (class 2) | judgment; long-lived agent |
| `abbrev` | `abbreviation-unexpanded` rows (class 3) | judgment; long-lived agent |
| `engine` | remaining `engine-*` rows + rejected-hypothesis triage | judgment; long-lived agent |
| `identity` | projection-seeded admissions (class 4) — starts when `bulk` drains | loop + agent for rejects |

Worktrees: `git worktree add ../cs-<stream> -b burndown/<stream>` off
`main`; each uses `CARGO_TARGET_DIR=../cs-<stream>/target` so builds do
not contend. A stream never edits files another stream owns; the only
shared files are the append-only admission TSVs under `data/synodal/`,
the identity table, and the regenerated artifacts — see the merge
protocol.

### The unattended bulk stream — start it first

```
cargo gold loop --only unregistered-lemma --min-cells 2 --take 200 \
  --until-dry --log reports/burndown-bulk.log
```

Add `--until-dry` if it does not exist: keep proposing and admitting
until a full pass admits nothing. Every batch that lands is one commit on
the stream branch with the one-line delta as its message (`cargo gold
commit` or a shell loop — no human, no LLM). Expected: the 6,652 fitting
clusters land in ~30 minutes of machine time. Rejections accumulate in
`reports/synodal-gold-rejected-hypotheses.tsv` for the `engine` stream.

### Judgment streams — one long-lived agent each

Spawn each agent once with its stream's scope and keep it alive: resume
it by message for the next batch instead of spawning a new one (the
ramp cost is the single largest per-slice overhead). Rules for these
agents:

- Batch size: dozens to hundreds; commit each landed batch on the stream
  branch with the delta line only.
- Report: the delta line per batch; a paragraph at stream end. No
  per-batch prose.
- Verification per batch: **scoped replay only** (`cargo gold --check
  --only <class>` / `--lemma`) plus the unit tests of any crate they
  edited. Nothing else.
- If an agent has emitted nothing for 10 minutes, ping it; if it is
  waiting on its own background command, tell it to poll that command,
  not to stop.

### The merge protocol (every ~30–60 minutes, or when a stream ends)

On `main`, in order, one command (`cargo gold merge <branches...>` — add
it if it does not exist):

1. `git merge` each stream branch. The only legitimate conflicts are
   append-only TSVs (resolve by union, stable-sorted by the file's key
   column) and regenerated artifacts (discard both sides).
2. Regenerate: `synodal-regenerate` (artifact), `unified-identity`,
   `cargo gold --fix`. The result must be a strict subset of the last
   merged gap; if two streams' admissions interact so that a row got
   worse, the merge command reverts the later stream's rows for that
   lexeme into its rejected report and re-fixes. No human in this loop.
3. Lightweight gate: `cargo gold --check` + `cargo test -p` for the
   crates the merge touched. Commit. Push.

### What CI does now

Per push: `cargo test --workspace`, `synodal-gold --check`,
`rewrite-pilot-accuracy`, `unified-identity --check`, clippy, fmt.
Nightly (a scheduled workflow, not per push): the full `check-structure`
(87 s), publish dry-runs, wasm/no-default builds, guard witnesses,
archive check. A per-push failure is fixed on `main` before the next
merge; a nightly failure opens the day's first task. Nobody waits on CI
inside any loop.

### Rules that do not change

Every clause of `docs/GOLD_GAP_BURNDOWN_PROMPT.md`'s contract: gap file
as the only metric, rules-first admissions with the split reported,
review-by-oracle with provenance, the accent asymmetry, one override
precedence, the comparison contract off-limits, the OCS oracle at 100%.
The `--until-dry` loop and the merge command must refuse to land
anything the scoped replay did not prove.

## First hour, concretely

1. Add `--until-dry`, `cargo gold commit`, and `cargo gold merge` (small
   xtask additions; test each on a throwaway worktree).
2. Create the five worktrees. Start `bulk` immediately.
3. Spawn the `alypy`, `abbrev`, and `engine` agents with their scopes
   (the in-flight class-2 agent, if any, becomes the `alypy` stream:
   move its worktree, do not restart it).
4. First merge at the 45-minute mark; push; report one table.

## Report back

The gap table before/after each merge (exact counts per class), rows
landed per stream and their rules-vs-residue split, rejected-hypothesis
count and dominant rejection reasons, and — the number this prompt
exists to move — **rows cleared per wall-clock hour**, per stream and
in total. If that number is not at least an order of magnitude above
class 1's (~660 rows in 80 minutes), say what is still serialising.
