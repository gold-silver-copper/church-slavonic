# Open designs after 2.0.0

Recorded 2026-09-05 at the close of the 2.0 rewrite (tag `v2.0.0`), from
the analysis of the five open questions in `HANDOFF-PROMPT.md`. For each:
what the question is, why it is open, what the 2.0 measurements say, and
the linguistically proper answer — the design a Slavicist who builds
morphological analyzers would choose, as distinct from the quick fix. The
order at the end is the recommended order of execution.
`V2.1-PROMPT.md` executed the first two (2 and 1a) on 2026-09-05, tag
`v2.1.0`; `V2.2-PROMPT.md` executed 1b (Parts 4–5, tag `v2.3.0`), 3 and 4 and the non-present verb
cells of 2 the same day, tag `v2.2.0`. The outcomes are recorded in place
below; the open list is 5 and what 1b left (the order's item 4).

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

**3.4 (the transfer, measured).** Five folds of the 3,757-leaf overlay
by chapter: a tagger trained on the OCS material plus four folds scores
78.93% on the fifth (1,307 of 1,656 examples), the OCS-only training
75.12%, the bundled model 74.58% — 3.8 points for some 1,300 Synodal
examples. The shipped model stays OCS-only. The 90% bar is not a few
more chapters of gold away: it wants a Synodal source the size of the
OCS material, in the print's conventions, that is neither the Bible
(the test) nor the overlay (its gold) — an edition with morphology no
one has published, or a treebank of the Menaion or the Psalter someone
would have to make. The errors that remain are syntax (nominative
against accusative 114 of 387, gender 52, genitive against accusative
46) and one convention (по with the dative in UD, the locative in the
print, 12); none is a fold matter.

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

**Executed (2.2, 2026-09-05).** The census: 1,435 of 2,313 closed adverbs
are printed as an adjective's neuter short nominative (857 with the wide
ѡ) or short locative. The adverb is the adjective's cell (`adv=1-о^|1-ѣ`,
`comp.adv`; `Cell::Adv`); 1,123 closed lines went, their counts on the
adjectives' notes, 61 stayed with `adv-of=` (another accent), 807 have no
adjective. The closed lexeme's class is its subcategory; every
preposition carries `gov=` (the grammar's frames ordered by the treebank
census, extras as `gov?` notes) and `pros=procl`, the enclitics
`pros=encl`; the linter reads government from the lexicon.

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

**Executed (2.2, 2026-09-05).** `Form::with_enclitic`/`print_unit` (the
solid unit), `Form::print_hosting` (the host before an enclitic written
apart), `prosody::words`; treebank `(pw …)` and `(pwa …)`. The print
writes the unit apart far more often than solid: 1,854 `(pwa …)` (Рече́ же
289, Є҆гда́ же 138) against 441 `(pw …)` (и҆̀хже 270, ѻ҆́ньже 147); Genesis
1:2's Землѧ́ же lifts; verbatim 8.2% → 7.8%. Not done, and deferred in
3.2 with the record corrected: no second-position placement function
exists (`prosody::words` groups units, `Form::print_unit` accents one);
no consumer generates an enclitic; a generator that does will write the
placement as its own call.

## 5. The Part 1 noun gate — struck (3.0)

**What.** The plan asked for 99% of Polyakov's noun cells reproduced by
the primary form with true exceptions on at most 5% of lexemes; the
lexicon sits at 94.7% primary, 96.8% reachable.

**Struck from the record in 3.0.** `V2.2-PROMPT.md` Part 6 executed the
two designs below (the accent inventory in `lexicon/stress.tsv`, the
weighted variants and the print as arbiter); under them the 99% gate is
not a measurement of the language — it counted a corpus dictionary's
tags, its wrong ones included — and the gates that stand are held-out
recall and the Bible's coverage with the print as arbiter.

**Why it was open.** Recorded as unmet after inspection class by class;
not one missing mechanism.

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

**The residue (3.0 Part 1, 2026-09-05), so that no later session
re-opens it without a new source.** After the place `P`, the stem place
through a derivation, the stressed tail and the adverb cell, the
exception lists are nouns 210, adjectives 213, verbs 567, pronouns 20 =
1,010 (from 1,871), in 148 / 188 / 459 / 20 shapes; 71 more columns move
one number (`b{sg=S}`), which the notation spells inline and the census
counts apart — they are paradigms. What the 1,010 are: (a) the fitter's
own remainders on lines with few attested cells, where one cell's accent
contradicts every paradigm by a hair (`b{pres.3.sg=S}` 6, `a{pres.3.pl=E}`
4, `b{inf=S}`-like number moves on `b.inf`), Polyakov's own variation;
(b) the proper names' citation cell (`a{nom.sg=<n>}` 20 nouns: а҆раві́а,
а҆гнѧ̀ — the headword is stressed elsewhere than the oblique cells, a
source fact); (c) the past passive participle's retractions the shapes
show under three lines (`a{part.past.pass.short.m.sg.nom=<n>}` 11 with
an index no place names, since the past participle's stem is the
infinitive's and the retraction lands two vowels back); (d) the
adjectives' long lists of a single letter-and-accent pattern
(коме́льскій, кѵзі́ческій: the plural genitive and locative on the first
vowel), source tags read against a class whose ending vowel the print
widens; (e) four adverbs whose accent fits no paradigm (ве́селѡ ~
весе́лый). None of these is a mechanism the format lacks; each is a
source's fact or a line with too few cells to say otherwise. The 135
primaries the Bible outnumbers are 115 letter variants (the arbiter never
crosses a letter), 10 citation cells (ids stable), one stress twin the
Bible also prints inside a set, and 7 stress twins of names and small
counts (наѳана́илъ 8:5, высоты̑ 2:1) — the two pronoun twins (ѻ҆нꙋ̀ 67:16,
ѡ҆́ны 19:0) were decided by hand.

## Order

1. ~~Present-stem derivation (2)~~ — executed in 2.1; the non-present
   cells by type in 2.2.
2. ~~Underspecified syncretism (1a)~~ — executed in 2.1.
3. ~~Adverb derivation and the closed lexicon (3), the phonological word
   (4)~~ — executed in 2.2.
4. ~~Constraint-based disambiguation (1b)~~ — executed in 2.3
   (`V2.2-PROMPT.md` Parts 4–5): four eliminating rules at 100%
   precision on a 2,095-leaf overlay (resolution 45.0%), an averaged
   perceptron for the rest (86.9% on OCS dev+test's several-reading
   tokens, 74.7% precision on the overlay, its choices in their own
   column). What remains of 1b: the syntax a one-token window does not
   see (subject against object of an inanimate, a pronoun's gender from
   its antecedent), a calibrated confidence, and the lexicon duplicates
   the homonymy census named (гдⷭ҇ь, а҆́зъ). 3.1 merged the twins (the
   rule of identity; гдⷭ҇ь's two lines stay, their plural nominative
   differs) and grew the gold to 3,377 leaves; 3.2 added the clause rule
   `one-subject` (100% precision, 2,545 Bible leaves) and measured the
   tagger's calibration on OCS dev (`tagger-curve`: the share is
   informative, no threshold meets the overlay's 90% bar). What remains:
   a clause whose verb is itself several lexemes (ви́дѣ) gives the rules
   nothing; the tagger's transfer gap (89% on OCS, 76% on the overlay).
5. ~~The accent-paradigm inventory and weighted evidence (5)~~ — executed
   in 3.0 (`V2.2-PROMPT.md` Part 6): the inventory of `lexicon/stress.tsv`
   (31 named paradigms), the enclitic's vowels out of the stress count,
   index-based fitting, `×n` weights, the print as arbiter; the lists
   fell from 1,871 to what no paradigm names (recorded in CHANGELOG
   3.0.0). `V3.0-PROMPT.md` Part 1 then added the place `P` (и҆зго́нимъ),
   the stem place through a derivation as a crate rule (цѣлꙋ́ющїй), the
   stressed tail of the -надесѧть numerals and the adverb cell's accent:
   1,344 → 1,010, the residue described above.
7. ~~The lexicon intake the gold exposed~~ — executed in 3.3
   (`V3.3-PROMPT.md` Parts 1–2): the print's last letters as rules or
   as letters of the lexeme written by the importer with the Bible as
   arbiter, 113 titlo rows, the numerals, the pronominal adjectives'
   adjective endings, the composite verb class, любы̀, the pronoun
   clitics as enclitics; the Bible's verbatim share 5.7% → 0.9%. What
   remains is in `HANDOFF-PROMPT.md`'s open list 2. 4.1 (`V4-PROMPT.md`
   Part 2) added the second print — the Ponomar library, 3.19 million
   tokens, its verbatim share 5.6% → 2.2% by the titlo-written lexemes
   the importer had skipped (rows 135 → 559) — and left the residue
   ranked by both censuses in the HANDOFF's open list.
