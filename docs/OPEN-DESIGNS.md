# Open designs after 2.0.0

Recorded 2026-09-05 at the close of the 2.0 rewrite (tag `v2.0.0`), from
the analysis of the five open questions in `HANDOFF-PROMPT.md`. For each:
what the question is, why it is open, what the 2.0 measurements say, and
the linguistically proper answer — the design a Slavicist who builds
morphological analyzers would choose, as distinct from the quick fix. The
order at the end is the recommended order of execution.
`V2.1-PROMPT.md` executed the first two (2 and 1a) on 2026-09-05, tag
`v2.1.0`; their outcomes are recorded in place below, and the open list
is 1b, 3, 4, 5.

## 1. The ambiguous band

**What.** 40.2% of the Bible's 631,946 tokens have several exact readings
in the Synodal analyzer and stay verbatim in the treebank with `:amb n`.
The band grew from 31.0% (1.2) to 40.2% because the lexicon now holds
every part of speech: a token that used to fall through as verbatim now
has readings, usually more than one.

**Why open.** The lifter enters a leaf only when exactly one exact reading
exists. It never chooses, and the round-trip invariant cannot see a wrong
choice between readings that print identically (the є҆́жъ vocative of
v1.1 was rendered correctly and analysed wrongly).

**What the readings are.** Three shapes, measured by sampling:

- one lexeme, several cells — every masculine inanimate prints nom = acc
  = voc in the singular; long adjective forms are shared by the masculine
  and the neuter in the oblique cases; dual nom = acc; plural cells
  coincide across genders in the oblique cases;
- several lexemes, one cell — homographs distinct by accent only (во́лна
  wool, волна̀ wave), or by nothing (Polyakov's и҆ as conjunction and as
  particle, folded into one function word in Part 3);
- a closed-class word that is also an inflected form (да, the
  conjunction, beside the forms of да́ти).

**The proper answer.** Separate *syncretism* from *homonymy* and treat
them differently.

Syncretism is a property of the paradigm, not doubt about the word. The
standard representation is **underspecification**: one reading with a
disjunctive feature bundle (case ∈ {nom, acc, voc}) rather than three
readings. The UD annotation of PROIEL already does this (`Case=Dat,Gen`;
the 2.0 loader skips those tokens). Under that representation a
syncretic token is *analyzed*: the lexeme is known, the cell is a set, the
leaf carries the set (`:case nom|acc`), and every member prints the same
form, so the invariant still holds. This changes what the treebank
reports without any guessing. `V2.1-PROMPT.md` Part 2.

**Executed (2.1, 2026-09-05).** `Lexicon::readings`, `cell::CellSet`
(`nom|acc|voc.sg`), leaves with disjunctive features or `:cell`, the
linter satisfied by any member, `check-treebank` asserting every
auto-lifted leaf names every cell that prints its token (364,073, none
incomplete), `narrow-hand` over Genesis 1 (283 leaves, 179 narrow a
larger set, 0 outside the lexicon's set). The treebank: analysed (one
cell) 23.6%, one lexeme in several cells 34.0%, closed 28.1%, several
lexemes 6.0%, verbatim 8.2%. The 40.2% "ambiguous" of 2.0 was five parts
syncretism to one part homonymy; the honest size of the disambiguation
problem (1b) is 6.0% of the Bible's tokens, and the 179 hand choices of
Genesis 1 are its first gold.

Homonymy needs context. The linguistically serious tool is a
constraint-based disambiguator (Constraint Grammar tradition), layered:
agreement inside the noun phrase and between a finite verb and its
nominative subject (the linter's rules, generalised); government
(prepositions with their case frames — the linter has ten, the lexicon
should hold all of them — and verb valency: the accusative object, the
dative addressee, the genitive under negation, which Church Slavonic
keeps productively); and a statistical tagger for what constraints leave
open, trained on the OCS treebanks (PROIEL and Syntacticus, gold
morphology, roughly 250,000 tokens) and transferred to the Synodal
Bible. The Genesis 1 hand overlay is the Synodal gold and must grow before
a tagger is trusted; evaluation stays outside the round-trip invariant.

## 2. Guessed lemmas and the present stem

**What.** `Lexicon::guess` builds a provisional lexeme for a lemma the
lexicon lacks; for OCS it reads the class off the lexicon's own lemma
endings, which works for nouns and adjectives and fails for verbs.

**Why open.** The OCS verb classes were seeded from Kaikki with the
present stem read off the attested present forms and stored on the
lexeme line (`stems=2=пь` for пити; 1,442 of 2,456 OCS verb lines carry
one, 113 Synodal lines). A guessed verb has no attested forms, inherits
`2=base`, and produces любитъ-style presents where the language has
люблѭ. The held-out miss census (`CS_RECALL_BLOCKS=1`) shows present
cells of guessed verbs as the largest remaining OCS verb miss.

**What the stored stems are.** A first classification of the 1,442 (by
the relation of the stored stem to the infinitive stem): theme-vowel
drop (авити → ав, 170 lines), theme drop plus iotation or palatalisation
of the stem-final consonant (любити → любл, свѣтити → свѣщ, ходити →
хожд, писати → пиш, алъкати → алъч), -ова-/-ева- → -ꙋ (баловати →
балꙋ), and seeding artefacts where only one present form was attested and
the longest common prefix is that whole form (багърити → багъримъ,
безаконьновати → безаконьноуѭштеи). The true residue is suppletion and
the irregular athematics (ити → ид/шьд, быти → ѥс/бѫд, дати → дад).

**The proper answer.** The present stem of a Church Slavonic verb is
predictable from its class, and the classes are Leskien's. Encode the
two-stem system (infinitive stem, present stem) with the morphonological
rules as *class-level derivations*, as the Synodal tables already do
(V21p любити: `1=base;2=iot`, `pres.1.sg=2-ю`, `pres.2.sg=1-иши`):

- class IV (-ити, -ѣти with -i- present): first person singular and the
  present passive participle iotate the stem-final consonant — labials
  take epenthetic л (люблѭ), dentals become щ/жд (свѣщѫ, хождѫ),
  sibilants palatalise (прошѫ); the other present cells take the plain
  stem plus -и-;
- class III with -j- (писати, глаголати): the whole present is iotated
  (пишѫ, пишеши); class III with -aje- (дѣлати) keeps the vowel (дѣлаѭ);
  the -ова-/-ева- verbs take -ꙋ- (`ov`);
- class I consonant stems (нести, рещи): the first palatalisation before
  the front vowels of the present and the aorist (речеши, рѣхъ), the
  second in the imperative (рьци);
- class II (-нѫти) and class V (athematic) as already modelled.

The primitives exist (`iot`, `pal1`, `pal2`, `ov`, `cut`). The lexeme's
`stems=2=` is then reserved for suppletion; a guessed verb inflects from
its infinitive alone; the lexicon shrinks, which is the design's own test
of correctness. `V2.1-PROMPT.md` Part 1.

**Executed (2.1, 2026-09-05).** The OCS verb classes are Leskien types
with derived stems (27 classes, 12 of them residue), the spelling rule
after a husher (ѭ/ѥ/ѩ/ꙗ → ѫ/е/ѧ/а) lives in the crate, `jer` joined the
derivations, `ov` gives -ю- after -ева-. Stored OCS present stems 1,442 →
56 (all named suppletion: възьмати 2=въземл, пѣти 2=по, трьти 2=тьр);
Kaikki verb cells reproduced 94.5% → 95.7%; UD dev+test verb recall
85.79% → 90.59%, Syntacticus 93.68% → 94.91%; a guessed OCS verb's
present cells 22.7% → 79.0%. Synodal stored stems 636 → 358 with three
class-level derivations (the soft-stem past active participle
и҆зба́вльшїй, the -нꙋ-less воздви́гшїй, бити's ї-stems); the rest is
Polyakov's нн/н variation and genuine suppletion. Two 2.0 defects fell
out: OCS past participles had never been produced (the delegation read
the Synodal adjective table), and the UD fit's tie-break was the table's
first row. Open within this design: Kaikki's aorist and l-participle
junk (косехъ, кослъ) is still reproduced by the data-driven cells; the
next step is to declare those blocks by type as the present now is.

## 3. Adverbs and the closed classes

**What.** Polyakov's ADV, ADVPRO, CONJ, PR, PART, INTJ, PRED entries
became 2,503 one-cell lexemes (`lexicon/syn/closed.tsv`). Right as a
first step: the analyzer resolves them and closed-class coverage returned
to 28.1%.

**Why open.** Adverbs from adjectives (мꙋ́дрѡ, до́брѣ) are morphologically
cells of adjectives — the neuter short form in -о (historically the
accusative) or the locative in -ѣ. Stored as separate lexemes, a new
adjective gets no adverb and an adverb's provenance is cut off from its
adjective.

**The proper answer.** An adverb cell on the adjective classes, and the
typographic fact the 1.x tables never captured: the Synodal print writes
the adverb with the wide ѡ (до́брѡ, мꙋ́дрѡ) precisely to tell it from the
neuter adjective до́бро — the same device as the plural number mark, and
the class tables already have it (`^` on a cell): `adv=1-о^|1-ѣ`, the
comparative adverb beside it (мꙋдрѣ́е). The census to run first: how many
of the 2,503 closed lines an adjective already in the lexicon produces.
The remaining closed classes deserve structure rather than a flat list:
prepositions with the cases they govern (for the linter and the
disambiguator), conjunctions, and the clitics as a category with prosodic
properties (see 4).

## 4. Sentence-level accent

**What.** `Form::print` decides oxia against varia by position inside one
word; 2.0 models the enclitic written solid (`encl=же`, `encl=сѧ`) by
skipping its vowels for the number mark. Genesis 1 still carries
«Землѧ́же» verbatim: the host's final varia is written as an oxia before
же.

**Why open.** This is a property of the sentence, not of the lexeme; it
does not belong in the lexicon, and no stage of the four handles a unit
larger than a word.

**The proper answer.** Church Slavonic accentuation is a word-level
system whose *orthographic* realisation is computed over the
**phonological word**: a host plus its enclitics (же, бо, ли, ми, ти, сѧ,
мѧ, тѧ, ны, вы) and proclitics (не, the prepositions) is accented as one
unit, so a stressed syllable that is final in the lexical word but not in
the unit takes the oxia. The model: a prosodic feature on the closed-class
lexicon (enclitic, proclitic, accented word); a pass in the treebank
renderer, and in any text generator, that forms phonological words from
the token sequence and applies the oxia/varia rule to the unit;
Wackernagel (second-position) placement of enclitics when generating.
The kamora, the wide letters and the monosyllabic varia (и҆̀хъ against
и҆́хъ) are orthographic disambiguation of homographic cells and stay at
the word level, where 2.0 has them. An optional fifth stage, needed by
the renderer and the game, not by `inflect`.

## 5. The Part 1 noun gate

**What.** The plan asked for 99% of Polyakov's noun cells reproduced by
the primary form with true exceptions on at most 5% of lexemes; the
lexicon sits at 94.7% primary, 96.8% reachable.

**Why open.** Recorded as unmet after inspection class by class; not one
missing mechanism.

**What the residue is.** Two things. Source noise: forms tagged for the
wrong cell, unaccented forms in an accented source, abbreviation marks
read as accents, bundled tags (`gen/acc`). And genuine per-lexeme
preference between two alternatives the class offers (-ѡвъ against the
zero genitive plural; -ами against -ы), which the lexicon records as
overrides.

**The proper answer.** Two designs. (a) **Accent paradigms as a closed
inventory.** Slavic accentology (Zaliznyak's inventory for Russian, Dybo's
for the reconstructed system) describes the mobile paradigms as a small
set with rules — stem throughout, ending throughout, stem in the
singular and ending in the plural, and the finer types where one cell
retracts (рꙋка̀ → рꙋ́кꙋ, рꙋ́ки). Synodal accent is close to the Russian
system with archaisms; the Part 1 census found the two commonest mobile
types (`c`, `d`), and the exception lists in the stress column are the
raw material for the others. (b) **Evidence with weights.** Polyakov is a
corpus dictionary: its forms carry counts and its tags are sometimes
wrong. Treat each source as evidence with a reliability weight, choose a
cell's primary by attestation frequency in the print (the pinned Bible
decided ѻ҆́вцꙋ over ѻ҆вцꙋ̀ fourteen to one), keep the rest as weighted
variants, and rank the analyzer's readings by that frequency. Under that
regime "does the class reproduce 99% of the dictionary's tags" dissolves:
the target is held-out recall with the print as arbiter, which Parts 2–4
measured. The 99% was a gate written before the data was seen; it
measured the dictionary, not the language.

## Order

1. ~~Present-stem derivation (2)~~ — executed in 2.1.
2. ~~Underspecified syncretism (1a)~~ — executed in 2.1.
3. Constraint-based disambiguation (1b), adverb derivation (3) and the
   phonological word (4): each needs a corpus census first. For 1b the
   census exists now: 37,621 Bible tokens are several lexemes, and the
   Genesis 1 overlay records 179 hand narrowings of a syncretic set —
   the seed of the gold a disambiguator is scored against.
4. The accent-paradigm inventory and weighted evidence (5): the deepest
   linguistics on the list and the one that would most change how the
   lexicon reads.
