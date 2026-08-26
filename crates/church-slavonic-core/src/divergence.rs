//! The named-divergence registry (docs/UNIFIED_LANGUAGE_PROMPT.md, layer 2).
//!
//! Every point where the merged inflection kernel carries an explicit
//! recension condition that orthographic projection cannot explain is named
//! here, with its evidence. An unexplained recension difference is a gap
//! row, never a silent fork; a difference that is realization (a projection
//! rule of `church-slavonic-orthography`) is cited in kernel comments by its
//! rule id (e.g. `gen:yery`, `fold:ja`) and does NOT appear here.
//!
//! `unmerged:`-prefixed entries record pieces of the per-family pronoun
//! kernels deliberately left in the family cores because the difference is
//! neither realization nor cleanly nameable at this slice (execution rule 4
//! of the pronoun merge).

/// One named morphological divergence between the recensions, carried by the
/// merged kernel as an explicit recension condition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NamedDivergence {
    /// Stable registry id, referenced from kernel code comments.
    pub id: &'static str,
    /// What differs, stated as OCS fact vs Synodal fact.
    pub summary: &'static str,
    /// Source evidence: projection-study divergence patterns
    /// (`reports/projection-study.md` §3), Alypy sections, Polivanova's
    /// grammar tables, or registry/oracle data.
    pub evidence: &'static str,
}

/// Named divergences encoded by the merged pronoun kernel
/// (`crate::pronoun`). Later POS merges append their own entries.
pub const NAMED: &[NamedDivergence] = &[
    NamedDivergence {
        id: "pron:instr-loc-sg-jer",
        summary: "OCS masculine/neuter instrumental and locative singular end \
                  in soft -мь (тѣмь, имь, ѥмь, комь); Synodal hardens to -мъ \
                  (тѣмъ, имъ, немъ, комъ). Projection's jer rules keep a \
                  final jer but cannot turn ь into ъ, so the ending is a \
                  morphological recension condition.",
        evidence: "projection-study §3 top patterns …емь (259 cells), …омь \
                   (194), …имь (203); Alypy §§46–48 pronoun tables vs \
                   Polivanova's 2/p terminals.",
    },
    NamedDivergence {
        id: "pron:genitive-accusative",
        summary: "Synodal marks animate accusatives with the genitive form \
                  (мене, тебе, себе, єго/него, ихъ/нихъ, сихъ, -ого/-его, \
                  -ѣхъ, кого) and carries an animacy dimension in the cell; \
                  OCS keeps the nominal accusative (мѧ, тѧ, сѧ, и, ѩ) and \
                  has no animacy dimension in the pronoun paradigm.",
        evidence: "Alypy §§46–48 animate/inanimate rows; Polivanova's tables \
                   have no animacy split; gold token oracle animate readings.",
    },
    NamedDivergence {
        id: "pron:accusative-clitic-status",
        summary: "The OCS accusative singulars мѧ/тѧ/сѧ are the table-primary \
                  forms; in Synodal they survive only as enclitics beside the \
                  genitive-shaped primaries мене/тебе/себе.",
        evidence: "Polivanova's personal-pronoun table vs Alypy §47's \
                   enclitic annotations.",
    },
    NamedDivergence {
        id: "pron:dual-nominative-leveling",
        summary: "OCS keeps distinct dual nominatives вѣ (1st) and ва (2nd); \
                  Synodal levels the dual nominative to the plural forms мы \
                  and вы while retaining the oblique duals наю/нама, \
                  ваю/вама.",
        evidence: "Alypy §47 dual rows vs Polivanova's dual column.",
    },
    NamedDivergence {
        id: "pron:dual-clitic-inventory",
        summary: "OCS has dual/plural dative clitics (на [disputed], ва, нꙑ, \
                  вꙑ) and a primary dual accusative на/ва; Synodal keeps only \
                  the enclitic dual accusatives ны/вы and no dative clitics \
                  outside the singular.",
        evidence: "Polivanova (with UT's disputed dual dative clitic) vs \
                   Alypy §47, which lists no dual clitic datives.",
    },
    NamedDivergence {
        id: "pron:third-person-nominative-on",
        summary: "The OCS third-person anaphoric *и has no nominative (the \
                  demonstratives тъ/онъ fill nominative syntax); Synodal \
                  lexicalizes онъ/она/оно (and dual/plural они, онѣ) as the \
                  suppletive third-person nominative of one paradigm.",
        evidence: "Polivanova's defective *и paradigm vs Alypy §46's full \
                   онъ paradigm; the ThirdPersonAndDemonstrative identity in \
                   the Synodal registry.",
    },
    NamedDivergence {
        id: "pron:third-person-locative-postprepositional",
        summary: "OCS attests a free anaphoric locative (ѥмь, ѥи); Synodal's \
                  third-person locative exists only after a preposition \
                  (немъ, ней) — the independent locative cell is invalid.",
        evidence: "Alypy §46 (locative given only in the н- series) vs \
                   Polivanova's free/adprepositional pairing.",
    },
    NamedDivergence {
        id: "pron:dual-accusative-gender-leveling",
        summary: "The OCS anaphoric accusative dual distinguishes masculine ꙗ \
                  from feminine/neuter и; Synodal levels all genders to ѧ \
                  (нѧ after a preposition).",
        evidence: "Alypy §46 dual row vs Polivanova's gendered dual cells; \
                   projection-study dual patterns (…ама/…има/…ома residue).",
    },
    NamedDivergence {
        id: "pron:kto-instrumental-stem",
        summary: "OCS instrumental цѣмь keeps the second-palatalization stem \
                  ц-; Synodal levels to the plain velar stem in кимъ.",
        evidence: "Alypy §48 кто table vs Polivanova's къто paradigm.",
    },
    NamedDivergence {
        id: "pron:chto-oblique-inventory",
        summary: "The чьто oblique variant sets are re-inventoried: OCS \
                  genitive чесо/чьсо/чесого against Synodal чегѡ/чесѡ/чесогѡ \
                  with innovated primary чегѡ; the Synodal accusative adds \
                  чесо beside что; the dative drops чьсому.",
        evidence: "Alypy §48 что table vs Polivanova's чьто variants.",
    },
    NamedDivergence {
        id: "pron:proximal-nominative-reshape",
        summary: "The proximal demonstrative reshapes its direct cells: OCS \
                  сь/си/се against Synodal сей‖сій/сїѧ/сїе, with the \
                  feminine/neuter direct plurals and duals following the \
                  full-form shape (сїѧ, сіи) instead of OCS short си/сиѩ.",
        evidence: "Alypy §45 сей table vs Polivanova's сь paradigm.",
    },
];

/// Pieces of the per-family pronoun kernels deliberately NOT merged at this
/// slice, with the reason. Each stays behind its family's public API and its
/// family test suite; a later slice (or the adjective merge) revisits them.
pub const UNMERGED: &[NamedDivergence] = &[
    NamedDivergence {
        id: "unmerged:pron:adjective-coupled-classes",
        summary: "Synodal FullHard/FullSoft/FullVelar delegate to the Synodal \
                  long-adjective kernel, and Hard (contracted/uncontracted \
                  doublets), ShortVelar/QuantityVelar (palatalized variant \
                  doubling), ShortOvMixed (noun-like sub-paradigm) and \
                  InterrogativeKii interleave with it; the OCS counterpart \
                  кꙑи declines with suppletive stems кꙑ-/ко-/цѣ-/ци- that do \
                  not align cell-by-cell under any nameable condition. These \
                  classes merge together with the adjective POS slice.",
        evidence: "synodal pronoun.rs full_forms → long_adjective_ending; \
                   OCS kyi_forms suppletion; Alypy §§48, 57.",
    },
    NamedDivergence {
        id: "unmerged:pron:derived-family-composition",
        summary: "The OCS §316 derived-family composer (interposed \
                  preposition tokens, direct-case -то retain/drop, unbound \
                  любо) and the Synodal Alypy §§46–48 prefix/postpositive \
                  licensing differ in token model and licensing semantics; \
                  only the ни-/нѣ- prefix inventory is shared. Each family \
                  keeps its composer.",
        evidence: "OCS compose_pronominal_family_tokens vs Synodal \
                   compose/validate_pronoun_lexeme.",
    },
    NamedDivergence {
        id: "unmerged:pron:ocs-only-irregulars",
        summary: "The OCS irregular agreeing paradigms вьсь and сиць have no \
                  Synodal kernel counterpart to merge against (Synodal весь \
                  is served through the velar classes); they remain OCS \
                  lexical data.",
        evidence: "Polivanova's mixed-terminal paradigms; absent from Alypy \
                   §§45–48 as closed tables.",
    },
];

#[cfg(test)]
mod tests {
    use super::{NAMED, UNMERGED};

    #[test]
    fn registry_ids_are_unique_and_well_prefixed() {
        let mut ids: Vec<&str> = NAMED.iter().chain(UNMERGED).map(|entry| entry.id).collect();
        let total = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), total, "duplicate divergence id");
        assert!(NAMED.iter().all(|entry| entry.id.starts_with("pron:")));
        assert!(
            UNMERGED
                .iter()
                .all(|entry| entry.id.starts_with("unmerged:pron:"))
        );
    }
}
