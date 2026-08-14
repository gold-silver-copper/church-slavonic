# Synodal Russian Church Slavonic recension boundary

## Target

The Synodal crates target the normalized liturgical Church Slavonic used by the
Russian Orthodox tradition in Church-Slavonic-script editions descended from the
Elizabeth Bible and represented by modern Synodal typography. The target label is
always `SynodalRussian`; `cu` by itself is not a sufficient recension label.

The target includes:

- the Russian recension's grammatical system as described by Alypy
  (Gamanovich) and checked against named Synodal editions;
- expanded lexical spelling and traditional printed spelling as separate
  representations;
- Church Slavonic accents, breathings, positional letters, abbreviations, nomina
  sacra, and Cyrillic numerals when their lexical or semantic preconditions are
  known; and
- productive, normatively possible forms when the caller opts into predictive
  generation and the required Synodal principal parts or reviewed mappings exist.

The target does not include Serbian, Ukrainian, Bulgarian, Croatian, or other
Church Slavonic recensions, arbitrary Old Russian, modern Russian, modern
constructed Slavic standards, or Slovowiki. The Russian Synodal Translation of
the Bible is modern Russian and is not evidence for this target.

## Evidence boundary

Every source record declares a `source_recension`; every successful generated
variant declares `target_recension = SynodalRussian`. Exact-form evidence may be
labeled Synodal only when its work, edition, and transcription layer establish
that recension. Unknown or mixed-recension data is quarantined until reviewed.

Old Church Slavonic is inherited evidence, not forbidden evidence. OCS lexical
classes, stems, principal parts, and paradigms may propose a Synodal analysis only
through a stable, reviewable `RecensionMapping`. The surface output is then
created by a separately identified Synodal rule. An OCS spelling is never copied
into the Synodal exact-form registry and never relabeled as Synodal attestation.

The precedence rule is:

1. exact, edition-identified Synodal evidence;
2. reviewed Synodal irregular overrides;
3. Synodal normative generation from independently sourced principal parts;
4. reviewed or calibrated OCS-to-Synodal inheritance plus a Synodal realization
   rule;
5. analogical or exploratory candidates.

Conflicts remain attached to the candidate as evidence. Higher-ranked evidence
does not erase lower-ranked disagreement.

## Epistemic labels

The implementation distinguishes:

- `SynodalNormativeAuthority`;
- `ExactSynodalAttestation`;
- `InheritedOcsEvidence`;
- `OtherRecensionComparativeEvidence`; and
- `EvaluationOnlyEvidence`.

Likewise, a returned form is one of exact attestation, normative generation,
inherited prediction, or analogical prediction. "Generated" and "attested" are
never synonyms.

## Initial normative anchors

- Alypy (Gamanovich), *Grammar of the Church Slavonic Language*, corrected web
  transcription based on the 1991 edition: grammatical categories, paradigms,
  accentual and numeral rules.
- Unicode Technical Note #41: encoded Church Slavonic repertoire, combining-mark
  behavior, typography, collation, and numeral representation. UTN #41 is not a
  morphology authority.
- Edition-identified Elizabeth Bible and liturgical transcriptions: exact-form,
  printed-orthography, accent, and corpus evidence at the passage level.
- D'yachenko (1900): lexical and semantic candidate evidence, subject to corpus
  confirmation because the dictionary includes historical material beyond the
  target recension.

The complete pinned inventory and legal status are in [SOURCES.md](SOURCES.md).
