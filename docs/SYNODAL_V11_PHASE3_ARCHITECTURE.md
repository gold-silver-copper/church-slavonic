# Synodal v0.11 phase 3: architectural ceilings

This records what the phase found, changed, and deliberately did not change.
Two of the four ceilings the goal named turned out to be smaller than assumed
once measured, and this file states that plainly rather than performing the
work anyway.

## 1. Registry lookups were linear scans — fixed

Every lookup keyed on a lexeme scanned a `&'static` array in full:
`raw_by_id`, `exact_forms`, `has_exact_forms`, `has_exact_system`,
`has_principal_part`, `has_accent_data`, `pronoun_profiles`,
`accent_paradigm_for`, and `lexical_metadata`, which alone rescanned five
tables per lexeme. `Analyzer::new` calls `lexical_metadata` once per lexeme and
`index_cell` once per (lexeme, cell, profile), so construction grew as
Θ(L·E).

The generator already emits these tables sorted by their first column, so rows
sharing a lexeme are adjacent. `rows_for` binary-searches that run with
`partition_point`: O(log n), no allocation, and the rows come back in exactly
the order a scan produced, so resolver precedence and variant ordering are
untouched.

Sortedness was verified rather than assumed — of sixteen generated tables only
`IRREGULAR_VERB_INVENTORY` is unsorted, and it is not searched this way — and
three tests pin the contract, because an unsorted table would make lookups
silently return the wrong rows instead of merely slow ones.

| Measure | Before | After |
|---|---:|---:|
| slowest workspace test | 36.20 s | 7.63 s |
| analyzer construction test | 5.77 s | 3.24 s |
| dictionary lib suite | 36.33 s | 7.63 s |
| canonical coverage run | 21.0 s | 21.8 s |
| `synodal-check` | 4.74 s | 4.78 s |

The corpus pass is unchanged on purpose: it is dominated by per-token analysis,
not registry setup. What this removes is the quadratic term in construction,
which is what a fifteen-thousand-lexeme registry would have hit first.

## 2. Packaging — measured, and not rewritten

The goal assumed the generated registry was near the crates.io ceiling. It is
not. `cargo package -p synodal-church-slavonic` produces 3.4 MiB of files as a
**379 KiB** `.crate`, against a 10 MiB compressed limit — roughly 26× headroom,
at a compression ratio near 9:1.

Moving the payload out of generated Rust into a compact embedded format is
therefore premature: it is a large, risky change to buy capacity that is not
yet scarce. Extrapolating the measured ratio, publishing fails somewhere near
90 MiB of generated source, or on the order of a hundred thousand exact rows.

What was added instead is a tripwire. `check_package_metadata` now enforces a
40 MiB budget on each generated registry, currently at 6.7%, so the ceiling
arrives as a failing check naming its own remedy rather than as a surprise at
publish time. **Tripping that budget is the signal to change the storage
format, not to raise the number.**

## 3. Positional-letter realization — path implemented, narrower than assumed

The registry path genuinely could not carry a reviewed positional spelling
contract: `resolve_cell` applied accent metadata only, `apply_positional_paradigm`
ran solely for caller-supplied specs, and `positional_rules.tsv` was read-only
introspection. That structural gap is now closed. `positional_paradigms.tsv` is
a validated reviewed table, `registry::positional_paradigm_for` compiles it with
the same one-paradigm-per-cell and uniform-evidence rules accent paradigms use,
and `apply_generated_presentation` applies it after accent, on the printed form,
exactly as the caller-supplied path does. The operation vocabulary is closed so
a row cannot rewrite an unrelated character, and seven tests cover the parser,
its rejections, and the absent-contract case.

The table ships **empty**, and that is a finding rather than an omission:

- The endings the goal named as blocked are already handled. `градъ` generates
  `градѡ́мъ` and `градѡ́въ` today, because the §36 wide plural endings are built
  into the ending tables for that class. The `WidePluralEnding` operation is
  for classes where that general rule is *not* controlling.
- The remaining need is stem-internal and lexeme-specific, and often is not a
  rule at all. `знаменїе` prints both `зна́мєнїѧ` (65) and `зна́менїѧ` (18) in
  the source partition for the same cell: that is ordered variants, not a
  deterministic positional substitution, and forcing a rule would delete a real
  attested spelling.

Populating the table is therefore lexical review work that belongs with the
phase 4 lexicon, not a mechanical sweep.

## 4. A confirmed data defect this phase found but could not fix

Building the positional path surfaced a real error in the reviewed data.

`exact_forms.tsv` maps `synodal:noun:wikt-f7edd7b689ae`
(`грѣхъ`) `noun:dative:plural:inanimate` to expanded `грѣхѡмъ` with printed
`грѣхо́мъ`. The two disagree on the letter, and the print is the wrong cell:

- The row cites **Ezek.14.13**, `є҆́же па́стисѧ грѣхо́мъ` — "to fall **by** sin",
  instrumental singular.
- Its held-out evaluation row `eval:v07:6594200c7a319421` cites **Rom.5.12**,
  `є҆ди́нѣмъ человѣ́комъ … и҆ грѣхо́мъ сме́рть`. The parallel with instrumental
  singular `человѣ́комъ` settles it: instrumental singular again.
- Every sampled occurrence of `грѣхо́мъ` (6 in the source partition) is
  instrumental singular, governed by `под̾` or agreeing with `свои́мъ`.
- The genuine dative plural is `грѣхѡ́мъ` with the number-antistich broad omega,
  20 occurrences, agreeing with dative-plural `твои̑мъ`, `ва́шымъ`, `на́шымъ`
  (Apoc.18.4, Dan.4.24, Ezek.21.24, II_Paral.6.25, II_Macc.6.15, Dan.8.23).

### What landed

The genuinely attested dative plural `грѣхѡ́мъ` is now an exact row citing
Apoc.18.4, a source-partition witness. `synodal-v07-apply` preserves rows it
does not own, so the addition survives materialisation. It is worth **+40
tokens** (963,251 to 963,291) and leaves the held-out contract unchanged at
2,203/2,267 expanded and 2,119/2,267 printed.

### What did not land

The mis-celled row itself remains. Re-celling it to
`noun:instrumental:singular:inanimate` — which is what its Ezek.14.13 evidence
actually attests — was implemented and verified end to end, and then reverted.
The row is owned by the v0.7 wave: `v07_packet_ownership.tsv` line 323 declares
it and that ledger is pinned by `HISTORICAL_PACKET_OWNER_DIGEST`, so changing
the cell means either unfreezing a sealed digest or adding a v0.11
cell-correction ledger that `synodal-v07-apply` honours. The existing
`v10_exact_cell_corrections.tsv` only retracts rows and is read by
`synodal-check`, not by the wave tool. Bumping a frozen digest to absorb a data
change is exactly the failure mode the review contract warns about, so the
defect is recorded here with its evidence instead of being forced in.

The consequence is visible and bounded: `noun:dative:plural:inanimate` now
carries two ordered variants, the correct `грѣхѡ́мъ` and the mis-celled
`грѣхо́мъ`. Coverage and the held-out contract both improve or hold, but the
cell claims one spelling that belongs to the instrumental singular.

The same ledger also carries a second row (line 458) assigning the identical
print `грѣхо́мъ` to `noun:instrumental:singular:inanimate` with the same
mismatched expanded form, so a correction pass should treat both together.

**Next action:** add a v0.11 exact-cell correction ledger that
`synodal-v07-apply` consults when deriving rows, then re-land the correction
and the `eval:v11:grekh-dative-plural` held-out row.

## Deferred from this phase

Quadratic validators (`validate_candidate_links` reading every intermediate
JSONL in full, `validate_abbreviation_families`,
`validate_exact_form_attestation_evidence`) and the wave-frozen constants in
`crates/xtask/src/synodal_v07_*.rs` are untouched. `synodal-check` runs in 4.8 s
today, so these are not yet binding; the frozen constants are, however, the
direct cause of the unfixable defect above and should be addressed with the
correction ledger.
