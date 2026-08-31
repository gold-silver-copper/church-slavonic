# Prompt: a roguelike entirely in Church Slavonic, in Bevy, on the `church-slavonic` crate

Copy everything below the line into a fresh Claude Code session started in an **empty
directory** (not inside the church-slavonic repo — this is a new project consuming the
published crates). One asset is prepared for you: copy
`~/Desktop/code/church-slavonic/roguelike-assets/PonomarUnicode.otf` and its `OFL.txt`
into the new project's `assets/fonts/` as your first action (details under **Fonts**).

Every grammar claim, lemma, and expected string in this prompt was verified against
`church-slavonic` v1.0.0 on 2026-08-30 — trust them over intuition.

---

Build **Вертогра́дъ Сме́ртный** (working title) — a classic turn-based roguelike whose
every visible word is Church Slavonic, in Rust with **Bevy 0.19.x** (pin the 0.19 minor;
read that version's migration guide rather than trusting memorized Bevy API — Bevy
breaks API every minor release), using the **`church-slavonic`** crate (v1.x,
`Recension::Synodal` everywhere) for all runtime grammar. The point is to showcase the
library: grammatically correct, fully accented sentences composed at runtime — no string
table can decline every monster × weapon × verb combination; the library can.

## Day-one setup (do these before any game code)

1. `cargo new`, add `bevy = "0.19"` and `church-slavonic = "1"`.
2. Copy `PonomarUnicode.otf` (213,536 bytes, sha256
   `e1e824ba1d797bbe87770eb5a3cb9de65d9e5e9b419c76d87a49617975f78584`) and `OFL.txt`
   from `~/Desktop/code/church-slavonic/roguelike-assets/` into `assets/fonts/`. If the
   files are missing, fetch the same font from
   `https://raw.githubusercontent.com/typiconman/fonts-cu/master/Ponomar/PonomarUnicode.otf`
   and verify the sha256. It is SIL OFL; keep OFL.txt shipped next to it. Its coverage
   of the needed codepoints (Ꙋꙋ ѡ ѻ ѕ ѣ ѧ і ї є ѱ ѳ ѵ ѿ Ᲊ U+1C82, combining U+0300,
   U+0301, U+0311, U+0483 titlo, U+0486 psili, U+2DE0 block) is already verified — do
   not swap fonts.
3. Font smoke test: a Bevy scene rendering «оу҆да́рилъ є҆сѝ бѣ́са мече́мъ» and the HUD
   sample «Живо́тъ: і҃» in Ponomar Unicode. Bevy ≥0.15 shapes text through cosmic-text,
   so combining marks should position correctly — but LOOK at the window (screenshot it)
   before proceeding. If marks are detached or boxed, debug the text pipeline now, not
   in step 5. Keep the library's output exactly as returned — do **not** run any Unicode
   normalization over it; the tables' spelling (combining marks and all) is canonical
   and is what the font is designed for.

## The grammar layer (`src/slavonic.rs`) — build and test FIRST, without Bevy

Wrap the crate behind a phrase builder. Every entity carries `lemma: &'static str`,
`Gender`, and `Animacy`; `phrase(verb, subject, object, instrument, adjective)`
assembles log lines. Player-performed events use the **perfect** (l-participle +
enclitic «є҆сѝ»); monster/world events use the **aorist** (3rd person).

Facts about the library you must build around (all verified):

- **Lemma spelling is the tables' spelling.** The initial у-digraph is «оу» + psili
  («оу҆да́рити», «оу҆мре́ти»); the crate also accepts the Ᲊ form on input, but write the
  «оу҆» form in your lexicon. Lemmas are the accented Synodal citation forms.
- **Personal pronouns work; non-personal pronouns do NOT in Synodal.**
  `ChurchSlavonic::pronoun(...)` is fine («є҆гѡ̀»). `ChurchSlavonic::npron(...)` in
  `Recension::Synodal` returns empty or unaccented output (its Synodal sources were
  never ingested — a documented 1.0 limitation). The game needs only a handful of
  demonstratives; put them in a small hand-written const table in `slavonic.rs`
  (e.g. «се́й»/«сїѐ»/«сїѧ̀») with a comment citing this limitation, and do not call
  `npron` at all.
- **`noun()` takes no gender** — declension class is inferred from the lemma, so a
  handful of feminine i-stems misdecline (verified: «мы́шь» answers a masculine
  accusative «мы́ша»). Do not use «мы́шь». The verified-good feminine monsters are
  «ѕмїѧ̀» (acc «ѕмію̀») and «льви́ца» (acc «льви́цꙋ»).
- Animacy is handled by the tables/rule: animates take the genitive-shaped accusative
  («бѣ́са», «во́лка», «дра́кона»), inanimates keep the nominative shape («щи́тъ»,
  «сви́токъ»). An out-of-vocabulary lemma like «дра́конъ» still declines correctly by
  rule.

### The verified lexicon (use these; expected outputs were produced by the crate)

Monsters (glyph suggestion in parens):
«бѣ́съ» m (б), «ѕмі́й» m (ѕ), «во́лкъ» m (в), «мертве́цъ» m (м — acc «мертвца̀», a
fleeting-vowel showcase), «а҆́спідъ» m (а), «ѕмїѧ̀» f (ж), «льви́ца» f (л), boss
«дра́конъ» m (Д). Items: «ме́чь» (ins «мече́мъ»), «щи́тъ», «шле́мъ», «бронѧ̀» (acc
«броню̀»), «сви́токъ» (ins «сви́ткомъ»), potion «цѣле́бное питіѐ» (n). Adjectives:
«ѻ҆́стрый» (ins m «ѻ҆́стрымъ»), «лю́тый», «цѣле́бный».

### Required unit tests (exact strings, all pre-verified against v1.0.0)

```rust
use church_slavonic::*;
const SYN: Recension = Recension::Synodal;
// perfect: the player struck the demon with the sword
assert_eq!(phrase_strike("бѣ́съ", "ме́чь"), "оу҆да́рилъ є҆сѝ бѣ́са мече́мъ");
// building blocks the phrase fns must reproduce:
assert_eq!(ChurchSlavonic::l_participle("оу҆да́рити", &Gender::Masculine, &Number::Singular, &SYN), "оу҆да́рилъ");
assert_eq!(ChurchSlavonic::verb("бы́ти", &Person::Second, &Number::Singular, &Tense::Present, &Form::Finite, &SYN), "є҆сѝ");
assert_eq!(ChurchSlavonic::noun("бѣ́съ", &Case::Accusative, &Number::Singular, &SYN), "бѣ́са");        // animate
assert_eq!(ChurchSlavonic::noun("щи́тъ", &Case::Accusative, &Number::Singular, &SYN), "щи́тъ");        // inanimate
assert_eq!(ChurchSlavonic::noun("мертве́цъ", &Case::Accusative, &Number::Singular, &SYN), "мертвца̀"); // fleeting vowel
assert_eq!(ChurchSlavonic::noun("ѕмїѧ̀", &Case::Accusative, &Number::Singular, &SYN), "ѕмію̀");        // feminine
assert_eq!(ChurchSlavonic::noun("ме́чь", &Case::Instrumental, &Number::Singular, &SYN), "мече́мъ");
assert_eq!(ChurchSlavonic::adj("ѻ҆́стрый", &Case::Instrumental, &Number::Singular, &Gender::Masculine, &Degree::Positive, &SYN), "ѻ҆́стрымъ");
// aorist: the serpent died / the demon struck
assert_eq!(ChurchSlavonic::verb("оу҆мре́ти", &Person::Third, &Number::Singular, &Tense::Aorist, &Form::Finite, &SYN), "оу҆́мре");
assert_eq!(ChurchSlavonic::verb("оу҆да́рити", &Person::Third, &Number::Singular, &Tense::Aorist, &Form::Finite, &SYN), "оу҆да́ри");
// feminine perfect: the lioness has died
assert_eq!(ChurchSlavonic::l_participle("оу҆мре́ти", &Gender::Feminine, &Number::Singular, &SYN), "оу҆мерла̀");
// reflexive perfect for the death screen: "сконча́лсѧ є҆сѝ"
assert_eq!(ChurchSlavonic::l_participle("сконча́тисѧ", &Gender::Masculine, &Number::Singular, &SYN), "сконча́лсѧ");
// pickup: "взѧ́лъ є҆сѝ ѻ҆́стрый ме́чь"
assert_eq!(ChurchSlavonic::l_participle("взѧ́ти", &Gender::Masculine, &Number::Singular, &SYN), "взѧ́лъ");
// drink prompt imperative
assert_eq!(ChurchSlavonic::verb("пи́ти", &Person::Second, &Number::Singular, &Tense::Present, &Form::Imperative, &SYN), "пі́й");
```

NOTE ON SOURCE ENCODING: the expected strings above use combining marks exactly as the
crate emits them. Copy them from this file byte-for-byte (or, when in doubt, print the
crate's actual output first and paste it into the assertion — never retype accents by
hand). Add tests for every new lemma you adopt BEFORE using it in game content; if a
lemma misdeclines, pick a synonym rather than patching output.

Also in `slavonic.rs`: the **Cyrillic numeral** converter (а҃ в҃ г҃ … і҃ к҃, titlo
placement per convention: 11–19 are unit-before-ten, e.g. в҃і for 12) with its own
tests for 1, 11, 12, 20, 21, 100, 123.

## Game scope (deliberately classic and small)

- Procedural rooms-and-corridors dungeon, 10 floors, stairs down; floor 10 holds
  «дра́конъ» guarding «сокро́вище».
- Turn-based grid movement (arrows/hjkl), bump-to-attack, FOV, fog of war.
- The 8 monsters above with simple chase AI; each is an ECS entity carrying its lemma,
  gender, and animacy so the grammar layer declines it.
- Items: the weapons/armor/potion/scroll above; inventory screen; pickup line
  «взѧ́лъ є҆сѝ …» with the item declined in the accusative and its adjective agreeing.
- HP/attack/defense, XP with 2–3 level-ups, permadeath (death screen headed
  «сконча́лсѧ є҆сѝ» + run summary), victory screen.
- Message log, last ~6 lines, every line from the grammar layer.
- HUD numbers in Cyrillic numerals («Живо́тъ: і҃», «Степе́нь: г҃»).

OUT of scope (resist all of it): audio, animation beyond a movement tween, saves,
classes, ranged combat, town levels, localization toggles.

Map tiles render as colored glyph text on a grid (Slavonic letters as monster glyphs,
per the lexicon); UI panels and the log use ordinary Bevy text nodes. All identifiers
and comments in English; all player-visible text Church Slavonic. Maintain
`GLOSSARY.md` mapping every visible string to an English gloss so a non-reader can
review the game — update it in the same commit as any new string.

## Order of work (each step ends runnable; commit at each step)

1. `slavonic.rs` + all tests above passing (`cargo test`, no Bevy yet).
2. Day-one setup above: font vendored, smoke-test scene, screenshot saved to
   `docs/smoke.png`.
3. Map gen + player movement + FOV, rendered.
4. Monsters, bump combat, live message log through the grammar layer.
5. Items, inventory, potions/scrolls.
6. Floors, stairs, XP, boss, death/victory screens.
7. Polish: color, HUD numerals, README (screenshot, glossary link, a short section on
   how the grammar layer uses `church-slavonic`, font attribution + OFL).

## Definition of done

- `cargo run --release` gives a playable, winnable, losable 10-floor run on a clean
  checkout with no manual steps.
- `cargo test` passes, including every exact-string grammar test above.
- Zero player-visible English (audited screen-by-screen against GLOSSARY.md).
- No log sentence with a variable slot is hardcoded; `npron` is never called.
- `docs/smoke.png` shows correctly positioned combining marks; font + OFL.txt shipped.
