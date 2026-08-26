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

/// Named divergences encoded by the merged POS kernels (`crate::pronoun`,
/// `crate::determiner`, `crate::numeral`). Later POS merges append their
/// own entries.
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
        id: "det:hard-oblique-jat-doublets",
        summary: "The Synodal hard short determiner (самъ) co-lists ѣ-grade \
                  oblique doublets — ѣй beside ой in the feminine \
                  dative/locative singular, ѣмъ beside омъ in the \
                  masculine/neuter locative singular, and the огѡ/ого \
                  variant orders — that the OCS `2/p` hard class does not \
                  have.",
        evidence: "Alypy §§45 and 48 самъ table vs Polivanova's 2/p hard \
                   terminals (такъ, толикъ).",
    },
    NamedDivergence {
        id: "det:hard-feminine-plural-nominative",
        summary: "The Synodal hard short determiner takes -и in the feminine \
                  nominative plural and the inanimate accusative plural \
                  (сами) where the OCS hard class has -ы (такы); no declared \
                  projection rule relates ы and и.",
        evidence: "Alypy §48 самъ plural row vs Polivanova's 2/p hard \
                   plural terminals.",
    },
    NamedDivergence {
        id: "det:ves-direct-reshape",
        summary: "The totalizing вьсь reshapes its feminine and neuter \
                  direct cells: OCS вьса/вьсѣ (feminine singular and neuter \
                  plural) against Synodal soft-levelled всѧ; the accusative \
                  вьсѫ ~ всю pair, by contrast, is realization \
                  (gen:big-yus + gen:jer-medial).",
        evidence: "Polivanova's mixed-terminal вьсь paradigm vs Alypy §48 \
                   весь table.",
    },
    NamedDivergence {
        id: "det:ves-plural-jat-leveling",
        summary: "Synodal весь levels ѣ to е in the genitive/locative and \
                  animate accusative plural (всехъ against OCS вьсѣхъ) \
                  while keeping ѣ in the dative and instrumental (всѣмъ, \
                  всѣми) — a cell-conditioned split no projection rule \
                  explains.",
        evidence: "Alypy §48 весь plural row vs Polivanova's вьсь paradigm.",
    },
    NamedDivergence {
        id: "num:one-long-genitive-shapes",
        summary: "Synodal єдинъ mixes long-adjective shapes into the \
                  singular — єдинагѡ/єдинаго, єдинꙋю, єдиной — where OCS \
                  ѥдинъ keeps the pure pronominal hard endings ого, ѫ, ои.",
        evidence: "Alypy §62 єдинъ table vs Polivanova §§314–316 2/p \
                   ѥдинъ.",
    },
    NamedDivergence {
        id: "num:one-number-inventory",
        summary: "OCS ѥдинъ declines through all three numbers as a regular \
                  2/p lexeme; the Synodal cardinal-one paradigm is \
                  singular-only.",
        evidence: "Polivanova's unrestricted 2/p inventory vs Alypy §62, \
                   which gives єдинъ only singular cells.",
    },
    NamedDivergence {
        id: "num:two-genitive-u-doublet",
        summary: "Synodal два adds the genitive/locative doublet двꙋ beside \
                  двою; OCS дъва and both recensions' оба keep only the \
                  -ою form.",
        evidence: "Alypy §62 два/оба table vs Polivanova's dual 2/p \
                   terminals.",
    },
    NamedDivergence {
        id: "num:three-oblique-reinventory",
        summary: "The OCS cardinal three keeps a distinct genitive трии \
                  against the locative трьхъ; Synodal syncretizes genitive \
                  and locative in -хъ, co-lists the masculine трїе- doublet \
                  series through the obliques, and adds a masculine animate \
                  genitive-accusative arm.",
        evidence: "Polivanova §§321–322 and UT OCS Online §44.3 vs Alypy \
                   §62 три table.",
    },
    NamedDivergence {
        id: "num:four-oblique-reinventory",
        summary: "The OCS cardinal four keeps the genitive четыръ against \
                  the locative четырехъ; Synodal syncretizes both in \
                  четырехъ and co-lists четыре/четыри doublets in the \
                  direct cells where OCS has one gendered form.",
        evidence: "Polivanova §§383–384 and UT OCS Online §44.4 vs Alypy \
                   §62 четыре table.",
    },
    NamedDivergence {
        id: "num:five-nine-plural-obliques",
        summary: "The OCS cardinals five through nine are singular-only \
                  i-stem nouns; Synodal adds adjectival plural obliques \
                  -ихъ (genitive/locative) and -имъ (dative) to the same \
                  lexemes.",
        evidence: "Polivanova §§349–351 singular-only profile vs Alypy §62 \
                   пѧть obliques.",
    },
    NamedDivergence {
        id: "num:ten-oblique-reinventory",
        summary: "The reviewed десѧть tables re-inventory the obliques: the \
                  plural instrumental десѧты against десѧтьми, Synodal \
                  adjectival doublets десѧтихъ/десѧтимъ/десѧтихъ, the \
                  reviewed accusative десѧте, and reviewed Synodal singular \
                  obliques where OCS lists only productive i-stem forms.",
        evidence: "Polivanova §§373–374 and UT OCS Online §44.5–20 vs Alypy \
                   §62 десѧть table.",
    },
    NamedDivergence {
        id: "num:collective-agreeing-reshape",
        summary: "The Synodal agreeing collective (двои) is plural-only \
                  with feminine nominative -и, inanimate accusative -и, and \
                  a licensed vocative; the OCS collectives дъвои/обои/трои \
                  decline through the full-number pronominal J class with \
                  -ѩ/-ꙗ plural terminals and no vocative.",
        evidence: "Alypy §69 двои table vs Polivanova's 2/p J terminals \
                   (дъвои, трои).",
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

/// Pieces of the per-family POS kernels deliberately NOT merged at their
/// slice, with the reason. Each stays behind its family's public API and its
/// family test suite; a later slice (or the adjective/noun merge) revisits
/// them.
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
        summary: "The OCS irregular agreeing paradigm сиць has no Synodal \
                  kernel counterpart to merge against; it remains OCS \
                  lexical data. (вьсь, deferred here by the pronoun slice, \
                  merged in the determiner slice as \
                  `crate::determiner::total_ves_cell` against Synodal весь.)",
        evidence: "Polivanova's mixed-terminal paradigms; сиць is absent \
                   from Alypy §§45–48 as a closed table.",
    },
    NamedDivergence {
        id: "unmerged:det:velar-universal",
        summary: "The velar universal determiners (OCS вьсакъ/вьсѣкъ, \
                  Synodal всѧкъ/всѧкїй) couple to the velar-palatalized \
                  variant doubling already deferred with the Synodal velar \
                  pronoun classes (unmerged:pron:adjective-coupled-classes) \
                  and to the Synodal а-grade genitives; they merge with the \
                  adjective slice.",
        evidence: "Synodal determiner.rs vsyak_short/long_forms vs the OCS \
                   2/p velar-palatalizing generator; Alypy §§48, 57.",
    },
    NamedDivergence {
        id: "unmerged:det:adjective-backed-inventory",
        summary: "OCS которꙑи/ѥтеръ/кꙑи and the Synodal FullSk (-скїй with \
                  -ск-/-ст- alternation) and long hard determiner forms \
                  delegate to the family adjective kernels and merge with \
                  the adjective slice; the remaining identity inventories \
                  (OCS такъ-series vs Synodal самъ) do not overlap \
                  lexically and are per-recension lexical facts.",
        evidence: "OCS determiner.rs adjectival profiles; Synodal \
                   determiner.rs FullSk/full_hard_forms; Alypy §§45, 57 vs \
                   Polivanova §§285, 303–305, 375–376.",
    },
    NamedDivergence {
        id: "unmerged:num:ordinal-adjective-coupled",
        summary: "Ordinal declension delegates to each family's adjective \
                  kernel (OCS hard/j-stem short and long forms vs Synodal \
                  Hard/PossessiveIi long classes) and merges with the \
                  adjective slice; the OCS closed ordinal stem inventory \
                  and compound-ordinal stems, and the Synodal ordinal \
                  lexemes, remain family and lexicon facts.",
        evidence: "OCS numeral.rs decline_ordinal/decline_compound_ordinal_stem \
                   vs Synodal numeral_morphology.rs adjective_like_forms.",
    },
    NamedDivergence {
        id: "unmerged:num:noun-backed-magnitudes-and-fractionals",
        summary: "The thousand and myriad magnitudes, the five–nine \
                  singulars, and both fractional lexicons decline through \
                  the family noun kernels and merge with the noun slice; \
                  the fractional inventories (OCS substantival \
                  полъ/половина/четврьть/десѧтина vs Synodal fractional \
                  adjective and noun classes) also do not overlap \
                  lexically.",
        evidence: "OCS numeral.rs decline_thousand/decline_fractional vs \
                   Synodal numeral_morphology.rs noun_like_forms; Leuta and \
                   Havryliuk 2018 p. 162 vs Alypy §§61–70.",
    },
    NamedDivergence {
        id: "unmerged:num:collective-remainder",
        summary: "Only the agreeing collective plural terminals merge; the \
                  OCS -ер-/-ор- adjectival collectives (hard `2/a`), the \
                  Synodal governing-neuter singular двое/трое profile (a \
                  Synodal innovation with no OCS paradigm), and the Synodal \
                  hard-plural -ер- class (ы/ыхъ against the adjectival \
                  space) are adjective-coupled or one-recension paradigms.",
        evidence: "OCS numeral.rs COLLECTIVE_*_STEMS and decline_collective \
                   vs Synodal collective_governing/hard_plural_forms; Alypy \
                   §69.",
    },
    NamedDivergence {
        id: "unmerged:num:composition-machinery",
        summary: "Value-driven composition is facade-serving per family: \
                  the OCS compound/distributive phrase composers with their \
                  provenance-carrying tokens, and the Synodal letter-numeral \
                  (titlo) formatter, are constructions over the paradigm \
                  cells, not paradigm cells.",
        evidence: "OCS numeral.rs compose_cardinal_analyses and the \
                   compound/distributive APIs vs Synodal numeral.rs \
                   CyrillicNumeral.",
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
        let pos_prefixes = ["pron:", "det:", "num:"];
        assert!(NAMED.iter().all(|entry| {
            pos_prefixes
                .iter()
                .any(|prefix| entry.id.starts_with(prefix))
        }));
        assert!(UNMERGED.iter().all(|entry| {
            pos_prefixes.iter().any(|prefix| {
                entry
                    .id
                    .strip_prefix("unmerged:")
                    .is_some_and(|rest| rest.starts_with(prefix))
            })
        }));
    }
}
