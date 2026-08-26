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
/// `crate::determiner`, `crate::numeral`, `crate::adjective`, `crate::noun`,
/// `crate::noun_consonant`, `crate::verb`, `crate::verb_past`,
/// `crate::verb_participle`).
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
    NamedDivergence {
        id: "adj:long-contraction",
        summary: "The Synodal long (compound) adjective declension contracts \
                  the OCS uncontracted vowel + ѥ/и sequences throughout the \
                  obliques: аѥго → агѡ/ѧгѡ, оуѥмоу → омꙋ/емꙋ, ꙑимь/иимь → \
                  ымъ/имъ, ѣѥмь/иѥмь → ѣмъ/емъ, instrumental feminine ѫѭ → \
                  ою, ꙑима/иима → ыма/има, and ꙑихъ/ꙑимъ/ꙑими (soft \
                  иихъ/иимъ/иими) → ыхъ/ымъ/ыми (ихъ/имъ/ими); the direct \
                  cells (благꙑимъ-type против the contracted column) stay \
                  fold-projectable. Predicted by the projection study as the \
                  merge's largest adjective family.",
        evidence: "projection-study §3 …аѥго/…оуѥмоу/…ꙑимь patterns; \
                   Polivanova's compound declension vs Alypy §57.",
    },
    NamedDivergence {
        id: "adj:short-oblique-pronominalization",
        summary: "The Synodal short adjective declension levels the OCS \
                  nominal (twofold o/a-stem) obliques to the pronominal/\
                  long-shaped series: омь/емь → ымъ/имъ, ома/ема/ама → \
                  ыма/има, genitive plural ъ/ь → ыхъ/ихъ, омъ/емъ/амъ → \
                  ымъ/имъ, instrumental ꙑ/и/ами → ыми/ими, and locative \
                  ѣхъ/ихъ/ахъ → ыхъ/ихъ, erasing the gendered feminine \
                  а-stem obliques.",
        evidence: "Polivanova's twofold nominal declension (§§303–305) vs \
                   Alypy §53 short tables.",
    },
    NamedDivergence {
        id: "adj:soft-short-palatal-vowel-series",
        summary: "The Synodal soft short column generalizes the palatal \
                  vowel series where OCS prints the plain letters after the \
                  soft stem: genitive/direct а → ѧ, dative оу → ю, the \
                  feminine genitive ѧ → и, and the feminine/neuter plural \
                  direct cells ѧ/а → и/ѧ.",
        evidence: "Polivanova's soft 2/a subtype vs Alypy §53 soft table.",
    },
    NamedDivergence {
        id: "adj:soft-long-vowel-grade",
        summary: "Beyond contraction, the Synodal soft long column levels \
                  stem-vowel grades: аꙗ → ѧѧ in the feminine singular and \
                  dual/neuter-plural direct cells, ѧѩ → їѧ in the feminine \
                  genitives, the feminine dative/locative ии → ей, and the \
                  dual genitive/locative оую → юю.",
        evidence: "Polivanova's compound soft declension vs Alypy §57 soft \
                   table.",
    },
    NamedDivergence {
        id: "adj:short-vocative-leveling",
        summary: "Synodal levels the short feminine vocative (OCS hard -о) \
                  and the soft masculine vocative (OCS -е) to the \
                  nominative shape; only the hard masculine vocative -е is \
                  shared.",
        evidence: "Polivanova's vocative rows vs Alypy §53, which prints a \
                   distinct short vocative only for the hard masculine.",
    },
    NamedDivergence {
        id: "det:velar-universal-reshape",
        summary: "The velar universal determiner (OCS вьсакъ/вьсѣкъ, Synodal \
                  всѧкъ) reshapes around the shared hard pronominal core: \
                  Synodal mixes long-adjective а-grade genitives (агѡ/аго \
                  against pronominal ого), co-lists palatalized/plain \
                  oblique doublets (ѣй/ой, омъ/ѣмъ), takes -и in the \
                  feminine nominative and inanimate accusative plural where \
                  the OCS class has -ы, drops the dual, adds the long \
                  всѧкїй paradigm, and carries the ѧ-grade stem всѧк- \
                  against OCS вьсак-/вьсѣк-.",
        evidence: "Alypy §§45, 48, and 57 всѧкъ tables vs Polivanova's 2/p \
                   velar-palatalizing terminals; the OCS column is pinned to \
                   the merged pronoun hard class by a kernel test.",
    },
    NamedDivergence {
        id: "noun:instrumental-singular-jer",
        summary: "The OCS soft and jer-grade instrumental singulars end in a \
                  soft jer (-ѥмь, -ьмь, -ъмь: мѫжемь, пѫтьмь, сꙑнъмь); \
                  Synodal hardens and vocalizes to -емъ/-омъ (мꙋжемъ, \
                  пꙋтемъ, сыномъ). Jer projection explains the medial vowel \
                  but cannot turn the final ь into ъ — the same condition as \
                  pron:instr-loc-sg-jer, extended over the noun classes.",
        evidence: "projection-study §3 top patterns …емь/…омь/…имь; Alypy \
                   §§34–44 instrumental rows vs Polivanova §§326–351.",
    },
    NamedDivergence {
        id: "noun:i-stem-instrumental-i-grade",
        summary: "The feminine i-grade instrumental singular re-vocalizes: \
                  OCS -ьѭ (костьѭ, матерьѭ, свекръвьѭ) against Synodal -їю \
                  (заповѣдїю, матерїю, церковїю); no projection rule takes \
                  the jer to и.",
        evidence: "Alypy §§41–44 vs Polivanova §§349–351 and the athematic \
                   tables.",
    },
    NamedDivergence {
        id: "noun:i-stem-vocative-leveling",
        summary: "The i-stem vocative singular is re-inventoried: OCS -и in \
                  both genders against Synodal feminine -е (заповѣде) and \
                  masculine -ь/-ю doublets.",
        evidence: "Alypy §41 vocative rows vs Polivanova §§333–335, \
                   349–351.",
    },
    NamedDivergence {
        id: "noun:soft-genitive-plural-reinventory",
        summary: "The soft, i-stem, and athematic genitive plurals are \
                  re-inventoried: OCS -ь (jo-stems), -ии (i-stems), and -ъ \
                  (consonant stems) against Synodal -ей/-ій/-їй (мꙋжей, \
                  заповѣдей, гостій, каменїй, матерїй‖ей), also carried by \
                  the genitive-shaped animate accusative plurals.",
        evidence: "Alypy §§34–44 genitive-plural rows vs Polivanova's -ь/-ии \
                   terminals.",
    },
    NamedDivergence {
        id: "noun:soft-direct-plural-leveling",
        summary: "The soft direct plurals level -ѩ/-ѧ to the \
                  nominative-shaped -и: OCS jo-masculine accusative мѫжѧ and \
                  ja-stem nominative/accusative доушѧ against Synodal мꙋжы/ \
                  дꙋши-type -и (the postvocalic ancient plural that keeps -ѧ \
                  survives as a Synodal family subclass).",
        evidence: "Alypy §§34–40 plural rows vs Polivanova tables 327/343.",
    },
    NamedDivergence {
        id: "noun:soft-feminine-genitive-leveling",
        summary: "The ja-stem genitive singular levels to the i-shape: OCS \
                  -ѩ (доушѧ, землѩ) against Synodal -и (дꙋши, земли) — the \
                  same grade shift as adj:soft-short-palatal-vowel-series's \
                  feminine genitive.",
        evidence: "Alypy §§39–40 vs Polivanova table 343.",
    },
    NamedDivergence {
        id: "noun:hard-declension-variant-imports",
        summary: "The Synodal first declension imports ordered variant sets \
                  the OCS twofold classes lack: the u-stem dative -ови/-еви \
                  and genitive plural -овъ (now primary over inherited -ъ), \
                  the i-stem instrumental -(ь)ми and nominative -їе, the \
                  a-stem instrumental/locative -ами/-ахъ (also on neuters \
                  and the -ама/-ѡмъ consonant-stem duals and datives), and \
                  the hard locative doublet -ѣ on soft masculines.",
        evidence: "Alypy §§33–44 variant rows vs Polivanova's variant-free \
                   tables 327/339; gold paradigm oracle variant orders.",
    },
    NamedDivergence {
        id: "noun:locative-plural-reinventory",
        summary: "The soft and neuter-athematic locative plurals are \
                  re-inventoried: OCS jo-stem -ихъ against Synodal \
                  -ехъ/-ѧхъ (мꙋжехъ, морѧхъ), and OCS neuter consonant-stem \
                  -ьхъ against Synodal -ѣхъ (именѣхъ, словесѣхъ); the \
                  jer-grade -ьхъ → -ехъ cells (i-stems, масс./fem. \
                  athematics) proved projectable and are realization.",
        evidence: "projection-study §3 …ѣхъ/…ихъ patterns; Alypy §§34–44 vs \
                   Polivanova's locative rows.",
    },
    NamedDivergence {
        id: "noun:animate-accusative-coverage",
        summary: "Synodal extends the genitive-shaped animate accusative \
                  over the a-stem, ja-stem, and feminine athematic plurals \
                  (жєнъ, дꙋшь, матерей‖и) where OCS keeps the \
                  nominative-shaped accusative; the o/jo-stem animate arms \
                  are shared (pron:genitive-accusative).",
        evidence: "Alypy §§39–44 animate rows vs Polivanova §§267, 289–290 \
                   (canonical nominative-like accusative).",
    },
    NamedDivergence {
        id: "noun:consonant-direct-reshape",
        summary: "The masculine n-stem direct cells reshape: OCS keeps the \
                  athematic nominative/accusative камꙑ and the plural \
                  камене; Synodal reshapes the citation to камень, generates \
                  accusative -ь/-е from the extended stem, and levels the \
                  direct plural to -и (камени).",
        evidence: "Alypy §§42–44 камень/день tables vs Polivanova's камꙑ \
                   paradigm.",
    },
    NamedDivergence {
        id: "noun:consonant-locative-singular-i",
        summary: "The athematic locative singular levels to the \
                  dative-shaped -и: OCS -е (камене, имене, словесе, \
                  свекръве) against Synodal -и (камени, имени, словеси, \
                  свекрови); only the r-stem locative -и is shared.",
        evidence: "Alypy §§42–44 locative rows vs Polivanova's athematic \
                   tables.",
    },
    NamedDivergence {
        id: "noun:dual-direct-reshape",
        summary: "The neuter dual direct cells reshape: OCS hard o-stem -ѣ \
                  (селѣ) against Synodal -а (села), and OCS consonant-stem \
                  -ѣ (именѣ, словесѣ) against Synodal -и (имени, словеси). \
                  Synodal RETAINS the dual as a category — the endings \
                  re-inventory, they do not vanish.",
        evidence: "Alypy §§34, 42–44 dual rows vs Polivanova tables \
                   339 and the athematic tables.",
    },
    NamedDivergence {
        id: "noun:dual-oblique-reinventory",
        summary: "The feminine athematic dual genitive/locative \
                  re-inventories: OCS -оу (матероу, свекръвоу) against \
                  Synodal -їю (матерїю, церковїю), the i-stem-shaped dual \
                  oblique.",
        evidence: "Alypy §§42–44 dual rows vs Polivanova's r/v-stem \
                   tables.",
    },
    NamedDivergence {
        id: "noun:u-stem-dissolution",
        summary: "The OCS u-stem paradigm (сꙑноу, сꙑнови, сꙑнъмь, dual \
                  сꙑнꙑ/сꙑновоу/сꙑнъма, vocative сꙑноу, plural \
                  сꙑнове/сꙑновъ/сꙑнъмъ/сꙑнъми/сꙑнъхъ) dissolves into the \
                  Synodal first declension carrying the u-stem endings as \
                  ordered variants (-ꙋ, -ови, -ове, -овомъ, -овѣхъ) beside \
                  the o-stem primaries; the distinct dual, vocative, and \
                  jer-grade obliques are not preserved.",
        evidence: "Alypy §§37–38 сынъ/домъ rows vs Polivanova §333 u-stem \
                   profile.",
    },
    NamedDivergence {
        id: "noun:in-singulative-inanimate-accusative",
        summary: "The -инъ singulative plural is otherwise shared \
                  (syncopated stem, -е/-ъ/-омъ/-ѣхъ), but the inanimate \
                  accusative differs: OCS -ꙑ against the Synodal \
                  nominative-shaped -е.",
        evidence: "Alypy §37 ethnonym table vs Polivanova's 2/m** class.",
    },
    NamedDivergence {
        id: "noun:agent-plural-reinventory",
        summary: "The agent -тель direct plural re-inventories: OCS \
                  nominative/vocative -ѥ (оучителѥ) against the Synodal \
                  ordered -и/-е/-їе variants; the inanimate accusative \
                  follows noun:soft-direct-plural-leveling.",
        evidence: "Alypy §37 agent rows vs Polivanova's 2/m* class.",
    },
    NamedDivergence {
        id: "verb:dual-first-person-va",
        summary: "The first-person dual ending re-vocalizes across every \
                  finite system: OCS -вѣ (несевѣ, ивѣ, ховѣ, оховѣ, ѥсвѣ) \
                  against Synodal -ва (несева, ива, хова, охова, єсва); no \
                  declared projection rule relates ѣ and а. In the copula \
                  the -вѣ archaism survives as the ordered Synodal \
                  normative variant.",
        evidence: "Alypy §§80, 86–87, 93 and the §81 copula dual rows \
                   (with their -вѣ normative variants in \
                   data/synodal/exact_forms.tsv) vs Polivanova §§412–424 \
                   and UT OCS Online §24.",
    },
    NamedDivergence {
        id: "verb:dual-third-person-leveling",
        summary: "OCS keeps a distinct third dual (-те: несете, ите, \
                  шете, осте, бѫдете) against the second dual (-та); \
                  Synodal levels the third dual to the second-dual shape \
                  (-та/-ста/-оста) in every finite system, with the \
                  ѣ-grade doublets co-listed in the copula.",
        evidence: "Alypy §§80, 86–87 person tables (one shared dual row \
                   for 2nd/3rd) vs Polivanova's distinct 3du terminals; \
                   the §81 copula tables.",
    },
    NamedDivergence {
        id: "verb:imperfect-contraction",
        summary: "The OCS uncontracted imperfect markers -ѣа- and the \
                  iotated -ѣꙗ-/-аꙗ- contract into the Synodal -ѧ- (Alypy \
                  §87 -ѧхъ series): бѣаше-type ~ бѧше-type. Contraction \
                  is not a character-level fold, so the marker is a \
                  morphological recension condition; the plain -аа- ~ -а- \
                  and -а- ~ -∅- pairs are realization because the OCS \
                  contracted variants match the Synodal grade exactly.",
        evidence: "Polivanova §§425–432 uncontracted/contracted variant \
                   pairs vs Alypy §87, which prints only the contracted \
                   grades.",
    },
    NamedDivergence {
        id: "verb:imperfect-hardening",
        summary: "The imperfect dual/plural personal endings re-inventory: \
                  OCS ш-series -шета/-шете (2du/3du and 2pl) against the \
                  Synodal aorist-shaped -ста/-сте.",
        evidence: "Alypy §87 imperfect table vs Polivanova §§425–432 and \
                   UT OCS Online §24.2.",
    },
    NamedDivergence {
        id: "verb:aorist-inventory",
        summary: "The aorist formation inventories differ: the OCS root \
                  (asigmatic) aorist and the first sigmatic -с- aorist \
                  have no Synodal counterpart (Alypy keeps only the \
                  х-/ох- series), while the Synodal closed ꙗти/начати/ \
                  вити/пити/клѧти list adds the ordered -тъ / bare-stem \
                  second/third-singular doublet absent from OCS.",
        evidence: "Polivanova §§433–445 formation inventory vs Alypy §86, \
                   including its -тъ list.",
    },
    NamedDivergence {
        id: "verb:aorist-third-plural-a-grade",
        summary: "The aorist third plural levels the small yus to а after \
                  the sibilant: OCS -шѧ/-ошѧ (and copula бѣшѧ, бꙑшѧ) \
                  against Synodal -ша/-оша (бѣша, быша); no declared \
                  projection rule takes ѧ to а.",
        evidence: "Alypy §86 and the §81 copula tables vs Polivanova's \
                   -шѧ terminals.",
    },
    NamedDivergence {
        id: "verb:imperative-vowel-grade",
        summary: "The first-conjugation imperative re-grades outside the \
                  shared -и singular: OCS yat-series -ѣвѣ/-ѣта/-ѣмъ/-ѣте \
                  against Synodal -ева/-ита/-емъ/-ите (е-grade in the \
                  first person, generalized и in the second), and Synodal \
                  adds the contracted j-series (-й, -йте) on vowel-final \
                  stems with no OCS counterpart.",
        evidence: "Alypy §93 imperative tables vs Polivanova §§446–452 \
                   yat-series terminals.",
    },
    NamedDivergence {
        id: "verb:l-participle-leveling",
        summary: "The l-participle agreement endings level: OCS gendered \
                  plurals -ли/-лꙑ/-ла and feminine/neuter dual -лѣ \
                  against the Synodal uniform plural -ли and dual \
                  -ла/-ли.",
        evidence: "Alypy §§97, 104 l-participle rows vs Polivanova's \
                   resultative table.",
    },
    NamedDivergence {
        id: "verb:present-active-nominative-contraction",
        summary: "The present-active masculine nominative-singular edge \
                  reshapes: OCS -ꙑ/-ѩ (несꙑ, знаѩ) against the Synodal \
                  contracted -ый/-ѧ with the retained uncontracted \
                  -ꙋщь/-ющь/-ѧщь prints as ordered variants (несый ‖ \
                  несꙋщь); the past-active edge likewise co-lists the \
                  retained -шъ/-вшъ prints beside the shared -ъ/-въ.",
        evidence: "Alypy §§95–96, 98 citation rows vs Polivanova \
                   §§453–470 participle nominatives.",
    },
    NamedDivergence {
        id: "verb:copula-third-person-soft-t",
        summary: "The copula's third-person endings soften: OCS ѥстъ and \
                  сѫтъ against Synodal єсть and сꙋть; the jer rules keep \
                  a final jer but cannot turn ъ into ь.",
        evidence: "Alypy §81 vs UT OCS Online §24.1 and Polivanova \
                   §§538–542.",
    },
    NamedDivergence {
        id: "verb:copula-first-plural-my",
        summary: "The copula's first plural reshapes: OCS ѥсмъ against \
                  Synodal єсмы.",
        evidence: "Alypy §81 vs UT OCS Online §24.1.",
    },
    NamedDivergence {
        id: "verb:copula-imperfect-restemming",
        summary: "The copula imperfect restems: the OCS uncontracted \
                  бѣах- series (бѣаше) against the Synodal бѧх- series \
                  (бѧше) — the copula-specific instance of \
                  verb:imperfect-contraction, with the vowel re-graded \
                  ѣа → ѧ throughout.",
        evidence: "Alypy §81 imperfect-бѧ table vs UT OCS Online §24.2 \
                   and Polivanova §§544–545.",
    },
    NamedDivergence {
        id: "verb:copula-tense-reassignment",
        summary: "The copula's бѣ- and бꙑ- series carry different tense \
                  labels per recension: the OCS aorist бѣхъ series is the \
                  Synodal imperfect-be table, and the OCS \
                  conditional-aorist бꙑхъ series is the Synodal plain \
                  aorist. The merged kernel keys the tables by form \
                  series, leaving the tense assignment family-side.",
        evidence: "Alypy §81 (бѣхъ printed under the imperfect, быхъ \
                   under the aorist) vs UT OCS Online §§24.2, 27 and \
                   Polivanova §§544–549.",
    },
    NamedDivergence {
        id: "verb:copula-aorist-sti",
        summary: "The Synodal copula aorist second/third singular adds \
                  бысть (with the -сть extension) as the primary print \
                  beside бы; OCS has only бꙑ.",
        evidence: "Alypy §81 aorist row (бы́сть ‖ бы̀) vs Polivanova \
                   §§546–549.",
    },
];

/// Pieces of the per-family POS kernels deliberately NOT merged at their
/// slice, with the reason. Each stays behind its family's public API and its
/// family test suite; a later slice (or the adjective/noun merge) revisits
/// them.
pub const UNMERGED: &[NamedDivergence] = &[
    NamedDivergence {
        id: "unmerged:pron:kii-suppletive-interrogative",
        summary: "Since the adjective slice the Synodal Full*/velar/kii/ov \
                  pronoun classes ride the merged long-adjective kernel \
                  through the family ending shim; what remains unmerged is \
                  the OCS interrogative кꙑи itself, whose suppletive stems \
                  кꙑ-/ко-/цѣ-/ци- do not align cell-by-cell with the \
                  Synodal кїй long-velar paradigm under any nameable \
                  condition. It stays an OCS lexical paradigm.",
        evidence: "OCS kyi_forms suppletion vs the Synodal full_velar_forms \
                   route; Alypy §§48, 57.",
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
        id: "unmerged:det:determiner-identity-inventories",
        summary: "Since the adjective slice the adjective-backed determiner \
                  routes (OCS которꙑи/ѥтеръ/кꙑи long citations, the Synodal \
                  FullSk -скїй with its -ск-/-ст- alternation and the long \
                  hard forms) read the merged adjective kernel through their \
                  family shims; what remains per-recension is the identity \
                  inventories themselves — the OCS такъ-series against the \
                  Synodal самъ — which do not overlap lexically.",
        evidence: "OCS determiner.rs adjectival profiles and Synodal \
                   determiner.rs FullSk/full_hard_forms, both now shimmed \
                   onto crate::adjective; Alypy §§45, 57 vs Polivanova \
                   §§285, 303–305, 375–376.",
    },
    NamedDivergence {
        id: "unmerged:num:ordinal-stem-inventories",
        summary: "Since the adjective slice ordinal declension rides the \
                  merged adjective kernel through each family's adjective \
                  shim (OCS hard/j-stem short and long routes, Synodal \
                  Hard/PossessiveIi long classes); what remains per-recension \
                  is the closed ordinal stem inventories — the OCS simple \
                  and compound-ordinal stems and the Synodal ordinal \
                  lexemes — which are family lexicon facts, not paradigm \
                  rules.",
        evidence: "OCS numeral.rs decline_ordinal/decline_compound_ordinal_stem \
                   and Synodal numeral_morphology.rs adjective_like_forms, \
                   both now shimmed onto crate::adjective.",
    },
    NamedDivergence {
        id: "unmerged:num:noun-backed-magnitudes-and-fractionals",
        summary: "Since the noun slice the noun-backed numeral routes ride \
                  the merged noun kernel through their family noun shims: \
                  the OCS thousand's ja-stem obliques and the Synodal \
                  magnitude/fractional noun classes (SecondHard, \
                  SecondMixed, FirstHardMasculine(UStem), ThirdFeminine) \
                  all read the kernel columns. What remains per-recension \
                  is the magnitude and fractional lexeme inventories \
                  themselves (OCS тꙑсѫщи/тьма spellings and substantival \
                  полъ/половина/четврьть/десѧтина vs the Synodal fractional \
                  adjective and noun lexemes), which do not overlap \
                  lexically.",
        evidence: "OCS numeral.rs decline_thousand over crate::noun (now \
                   kernel-backed) vs Synodal numeral_morphology.rs \
                   noun_like_forms over decline_noun (now kernel-backed); \
                   Leuta and Havryliuk 2018 p. 162 vs Alypy §§61–70.",
    },
    NamedDivergence {
        id: "unmerged:noun:synodal-subclass-tables",
        summary: "The Synodal velar, sibilant-mixed, mobile-ц, glide-й, \
                  -ей, -їа, -їе, postvocalic-ancient-plural, and \
                  second-mixed subclasses are family reshapes of the merged \
                  columns conditioned on Synodal stem phonology (positional \
                  ы/и after velars, -є-/-ѡ- wide-letter duals, sibilant \
                  vocatives) with no OCS class to align cell-by-cell; they \
                  stay Synodal family tables over the kernel's shared \
                  classes.",
        evidence: "Synodal noun.rs FirstHardVelarMasculine, FirstMixed*, \
                   FirstSoftMasculineJ/Ey, SecondSoft*Ia, SecondMixed, \
                   SecondSoftPostvocalicAncientPlural arms; Alypy §§8, \
                   32–40.",
    },
    NamedDivergence {
        id: "unmerged:noun:lexeme-specific-contracts",
        summary: "The lexeme-specific paradigms stay family-side: Synodal \
                  господь, день, камень's alternative series, ꙋдъ's mixed \
                  -ес- background, дщерь, the syncopating -овь members, the \
                  paired-body ѻко/ꙋхо duals, and the -ище locative variants; \
                  OCS class-0 (UniqueMixed) reviewed substantives. Each is \
                  a closed per-recension lexical contract, not a paradigm \
                  rule.",
        evidence: "Synodal noun.rs lexeme-specific declensions (Alypy \
                   §§35–44); OCS UniqueNounFamilyMember data.",
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
    NamedDivergence {
        id: "unmerged:verb:irregular-identity-inventories",
        summary: "The reviewed irregular identity inventories are \
                  per-recension lexical facts: the OCS unique/irregular \
                  verb kernels (дати, вѣдѣти, ꙗсти, имѣти, хотѣти and \
                  their compounds), the impersonal identities, and the \
                  Synodal exact-form tables of \
                  data/synodal/exact_forms.tsv. Only the copula бꙑти ~ \
                  бы́ти, attested closed on both sides, merges (as \
                  crate::verb_past::copula_form); its family routes are \
                  pinned to the kernel columns.",
        evidence: "OCS unique_verb.rs/irregular_verb.rs vs the Synodal \
                   Alypy §§81, 104 exact tables; both copula tables ride \
                   the kernel via shim and pin.",
    },
    NamedDivergence {
        id: "unmerged:verb:synodal-suppletive-present-edges",
        summary: "The Synodal present first singular and third plural are \
                  supplied suppletive principal parts (the ꙋ/ю and \
                  ꙋтъ/ютъ/атъ/ѧтъ choices ride lexical stem alternation); \
                  OCS generates them productively (-ѫ/-ѭ, -ѫтъ/-ѧтъ). \
                  The kernel carries OCS-only columns for those cells; \
                  gen:big-yus would relate the shapes once a Synodal \
                  productive route exists.",
        evidence: "Synodal verb.rs PresentPrincipalParts vs OCS \
                   present_ending; Alypy §80 vs Polivanova §§412–424.",
    },
    NamedDivergence {
        id: "unmerged:verb:ocs-conditional-bi",
        summary: "The OCS conditional би- series (бимь, би, бишѧ) has no \
                  Synodal paradigm counterpart: the Synodal conditional \
                  is the analytic аще бы + l-participle construction, a \
                  phrase-level fact. The series stays in the OCS copula \
                  module; the shared бꙑ-series member merged as \
                  crate::verb_past::CopulaSeries::AoristBy.",
        evidence: "Polivanova §§546–549 and UT OCS Online §27 vs Alypy \
                   §91 conditional formations.",
    },
    NamedDivergence {
        id: "unmerged:verb:participle-stem-supply",
        summary: "Participle stem supply stays family-side: the Synodal \
                  wholesale short/long stems with lexical нн doubling and \
                  the sibilant subclass reshape, and the OCS ов→оу \
                  transformation and final-j deletion. The kernel merges \
                  the formation suffixes and citation edges \
                  (crate::verb_participle); the family assembly around \
                  them is lexical.",
        evidence: "Synodal participle.rs ParticiplePrincipalPart vs OCS \
                   past_active_participle formations.",
    },
    NamedDivergence {
        id: "unmerged:verb:constructions-and-reflexive",
        summary: "The construction layers stay family-side: the Synodal \
                  reflexive -сѧ enclitic surface rules, periphrastic \
                  tense/passive/optative phrase composers, and verbal \
                  noun licensing (Alypy §§27, 73, 88–92, 101–102, 163), \
                  and the OCS supine and phrase modules. Each is a \
                  construction over the paradigm cells, not a paradigm \
                  cell.",
        evidence: "Synodal phrase.rs/verb.rs reflexive machinery vs OCS \
                   supine and phrase.rs; the supine itself is an \
                   OCS-only system (the Synodal target has no productive \
                   supine).",
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
        let pos_prefixes = ["pron:", "det:", "num:", "adj:", "noun:", "verb:"];
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
