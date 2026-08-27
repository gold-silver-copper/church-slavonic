# Bundled table attribution

The generated tables in this package (`generated/*_phf.rs`) are derived from
four labelled full-form sources, each tagged with the recension it attests.

## Old Church Slavonic (`ocs:` rows)

English Wiktionary Old Church Slavonic entries and inflection tables, obtained
through Wiktextract/Kaikki
(<https://kaikki.org/dictionary/Old%20Church%20Slavonic/index.html>).
English Wiktionary contributor content is available under CC BY-SA 4.0 and
GFDL 1.1 or later. This package redistributes the derived form data under
CC BY-SA 4.0; see <https://creativecommons.org/licenses/by-sa/4.0/legalcode>
and <https://en.wiktionary.org/wiki/Wiktionary:Copyrights>.

## Synodal (`syn:` rows)

The printed paradigm tables of Archbishop Alypy (Gamanovich), *Grammar of the
Church Slavonic Language* (Holy Trinity Monastery, Jordanville; web edition).
The rows reproduce the grammar's inflected word forms — linguistic data of a
liturgical language, not the grammar's expository text — with their printed
accents, breathings and titla.

A. E. Polyakov, *Grammatical dictionary of Church Slavonic (corpus-based)*,
tagged web edition (<http://dic.feb-web.ru/slavonic/dicgram/>; created 2013,
RGNF project 12-04-12045, revised 2015–2017, RFBR project 17-04-12064), built
on the Church Slavonic corpus of the Russian National Corpus. The rows
reproduce its corpus-attested inflected word forms with their printed
accents, in the print's canonical typography (the dictionary's `у` and `я`
as `ꙋ` and `ѧ`, the breathing on an initial vowel, oxia and varia by
position). Derived data ships under the crate licence by the institutional
grant recorded in the repository's `references/TERMS.md`.

The Russian Wiktionary's Церковнославянский section, obtained through
Wiktextract/Kaikki (<https://kaikki.org/ruwiktionary/Церковнославянский/>):
the 39 entries with structured inflection tables. Russian Wiktionary
contributor content is available under CC BY-SA 4.0; this package
redistributes the derived form data under CC BY-SA 4.0.

The two Old Church Slavonic treebanks (UD_Old_Church_Slavonic-PROIEL and
the Syntacticus PROIEL/TOROT texts, CC BY-NC-SA 4.0) are evaluation sources
only: no table cell derives from them.

## Code

The library code is dual-licensed under MIT and Apache-2.0.
