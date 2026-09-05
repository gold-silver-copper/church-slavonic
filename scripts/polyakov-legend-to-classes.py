#!/usr/bin/env python3
"""Seed lexicon/classes/noun.tsv from Polyakov's paradigm legend (flexslav.htm).

One-time bootstrap (Part 1 of V2-PROMPT.md): the legend's noun tables become
class rows; the stem-derivation spec per class is written here by hand (the
legend numbers the stems by exemplar, this script names how each is derived
from the lemma). The generated file is committed and hand-maintained after
this; re-running the script must be a deliberate act, followed by
`cargo xtask import polyakov --pos noun --fix-marks` (the number marks are
measured from the source, not copied from the legend, which omits many).

Ending conventions: the legend is a civil transliteration — `у` is `ꙋ`,
`я` is `ѧ`, the wide `ѡ`/`є` and the `^` mark are the print's number mark
(Form::number_mark) and are written as the narrow letter plus `^`.
"""
import html
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEGEND = ROOT / "references/downloads/polyakov/flexslav.htm"
OUT = ROOT / "crates/church-slavonic/lexicon/classes/noun.tsv"

# class -> (strip count, stems spec). Stem derivations:
#   base        the lemma minus `strip` letters
#   drop        base minus its last vowel (the fleeting vowel dropped)
#   insert      base with a vowel inserted before its last consonant — LEXICAL
#               (the lexeme's `stems=ins=…` column), rule fallback in the library
#   pal1[:x]    first palatalisation of x (к→ч г→ж х→ш ц→ч), x defaults to base
#   pal2[:x]    second palatalisation of x (к→ц г→з х→с)
#   ext:suffix  base + suffix
STEMS = {
    "N1t": (1, "1=base"),
    "N1t*": (1, "1=drop;2=base"),
    "N1j": (1, "1=base"),
    "N1j*": (1, "1=drop;2=base"),
    "N1k": (1, "1=base;2=pal2;3=pal1"),
    "N1g": (1, "1=base;2=pal2;3=pal1"),
    "N1x": (1, "1=base;2=pal2;3=pal1"),
    "N1k*": (1, "1=drop;2=base;3=pal2:drop;4=pal1:drop"),
    "N1s": (1, "1=base"),
    "N1sj": (1, "1=base"),
    "N1c": (1, "1=base;2=base;3=pal1"),
    "N1c*": (1, "1=drop;2=base;3=pal1:drop"),
    "N1a": (1, "1=base"),
    "N1i": (1, "1=base"),
    "N1e": (1, "1=base"),
    "N1in": (3, "1=base"),
    "N2t": (1, "1=base"),
    "N2t*": (1, "1=base;2=insert"),
    "N2j": (1, "1=base"),
    "N2k": (1, "1=base;2=pal2"),
    "N2g": (1, "1=base;2=pal2"),
    "N2s": (1, "1=base"),
    "N2c": (1, "1=base"),
    "N2c*": (1, "1=base;2=insert"),
    "N2i": (1, "1=base;2=cut"),
    "N2e": (1, "1=base;2=cut"),
    "N3t": (1, "1=base"),
    "N3t*": (1, "1=base;2=insert"),
    "N3j": (1, "1=base"),
    "N3j*": (1, "1=base;2=insert"),
    "N3k": (1, "1=base;2=pal2"),
    "N3k*": (1, "1=base;2=insert;3=pal2"),
    "N3s": (1, "1=base"),
    "N3c": (1, "1=base"),
    "N3c*": (1, "1=base;2=insert"),
    "N3a": (1, "1=base;2=cut"),
    "N3i": (1, "1=base;2=cut"),
    "N3e": (1, "1=base;2=cut"),
    "N41": (1, "1=base"),
    "N42": (1, "1=base"),
    "N43": (1, "1=base"),
    "N43*": (1, "1=drop;2=base"),
    "N5en": (1, "1=base;2=ext:ен"),
    "N5et": (1, "1=base;2=ext:ѧт"),
    "N5es": (1, "1=base;2=ext:ес"),
    "N5er": (1, "1=base;2=ext:ер"),
    "N5ov": (1, "1=base;2=ext:ов"),
    "N5*ov": (1, "1=drop;2=base"),
}
# Classes the data uses that the legend lacks, copied from a twin.
COPIES = {"N1g": "N1k", "N1x": "N1k", "N2g": "N2k", "N1c": "N1c*"}
# Hand corrections after measuring the import (Part 1): the print's plural
# accusative is the nominative-shaped one for animates too (а҆́ггелы,
# а҆рхіепі́скопы), the г/х stems take -и in the plural, the -іе neuters
# form -ьми on the cut stem, and the -ій/-ей names take -іѧ, -е, -ѣ.
PATCHES = {
    ("N1g", "nom.pl"): ["2-и"], ("N1x", "nom.pl"): ["2-и"],
    ("N1i", "gen.sg"): ["1-а", "1-ѧ"], ("N1i", "voc.sg"): ["1-е", "1-ю"],
    # measured alternative preferences (Part 1 import census)
    ("N1j", "acc.pl"): ["@gen.pl", "1-и"], ("N1j*", "acc.pl"): ["@gen.pl", "1-и"],
    ("N1c*", "acc.pl"): ["@gen.pl", "1-ы"], ("N1c", "acc.pl"): ["@gen.pl", "1-ы"],
    ("N1in", "acc.pl"): ["@gen.pl", "1-ы"],
    ("N2i", "ins.pl"): ["2-ьми", "1-и^", "1-ми", "1-ѧми"], ("N2e", "ins.pl"): ["2-ьми", "1-и^", "1-ми", "1-ѧми"],
    ("N1t", "ins.pl"): ["1-ы^", "1-ами", "1-ми"], ("N1t*", "ins.pl"): ["1-ы^", "1-ами", "1-ми"],
    ("N1k", "ins.pl"): ["1-и^", "1-ами"], ("N1g", "ins.pl"): ["1-и^", "1-ами"], ("N1x", "ins.pl"): ["1-и^", "1-ами"],
    ("N1s", "ins.pl"): ["1-ы^", "1-ами"], ("N1sj", "ins.pl"): ["1-и^", "1-ы^", "1-ами"],
    ("N1e", "dat.pl"): ["1-омъ^", "1-емъ^"], ("N5en", "dat.pl"): ["2-омъ^", "2-емъ^"],
    # the print's second series of plural endings (-ахъ/-амъ/-ами, -ове, -ови,
    # -ъ^), attested across the classes; primaries by the census
    ("N2i", "dat.pl"): ["1-емъ^", "1-ѧмъ"], ("N2i", "loc.pl"): ["1-ихъ", "1-ѧхъ"],
    ("N2e", "dat.pl"): ["1-емъ^", "1-ѧмъ"], ("N2e", "loc.pl"): ["1-ихъ", "1-ѧхъ"],
    ("N1t", "nom.pl"): ["1-и", "1-ове"], ("N1t", "dat.sg"): ["1-ꙋ", "1-ови"],
    ("N1t", "loc.pl"): ["1-ѣхъ", "1-ахъ"], ("N1t", "dat.pl"): ["1-омъ^", "1-амъ"],
    ("N1t*", "nom.pl"): ["1-и", "1-ове"], ("N1t*", "dat.sg"): ["1-ꙋ", "1-ови"],
    ("N1t*", "loc.pl"): ["1-ѣхъ", "1-ахъ"], ("N1t*", "dat.pl"): ["1-омъ^", "1-амъ"],
    ("N1k", "gen.pl"): ["1-овъ^", "1-ъ^"], ("N1g", "gen.pl"): ["1-овъ^", "1-ъ^"], ("N1x", "gen.pl"): ["1-овъ^", "1-ъ^"],
    ("N1k", "loc.pl"): ["2-ѣхъ", "1-ахъ"], ("N1g", "loc.pl"): ["2-ѣхъ", "1-ахъ"], ("N1x", "loc.pl"): ["2-ѣхъ", "1-ахъ"],
    ("N1k", "dat.pl"): ["1-омъ^", "1-амъ"], ("N1g", "dat.pl"): ["1-омъ^", "1-амъ"], ("N1x", "dat.pl"): ["1-омъ^", "1-амъ"],
    ("N1c*", "gen.pl"): ["1-евъ^", "1-ъ^", "1-овъ^"], ("N1c", "gen.pl"): ["1-евъ^", "1-ъ^", "1-овъ^"],
    ("N1e", "gen.pl"): ["1-овъ^", "1-евъ^"],
    ("N1j", "dat.sg"): ["1-ю", "1-еви"], ("N1j*", "dat.sg"): ["1-ю", "1-еви"],
    ("N1sj", "acc.pl"): ["1-и", "1-ы", "@gen.pl"], ("N1s", "acc.pl"): ["1-ы", "1-и", "@gen.pl"],
    ("N3k", "dat.sg"): ["1-ѣ", "2-ѣ"], ("N3k", "loc.sg"): ["2-ѣ", "1-ѣ"],
    ("N3k*", "dat.sg"): ["3-ѣ", "1-ѣ"], ("N3k*", "loc.sg"): ["3-ѣ", "1-ѣ"],
    ("N2k", "loc.sg"): ["2-ѣ", "1-ѣ"], ("N2g", "loc.sg"): ["2-ѣ", "1-ѣ"],
    # the fourth measured round: the zero genitive plural on the full stem
    # of the fleeting classes (ѻ҆тє́цъ), the -ей genitives, the gen-shaped
    # accusative plural of the -й masculines, the -е vocative of N3t
    ("N1c*", "gen.pl"): ["1-евъ^", "2-ъ^", "1-овъ^"], ("N1c", "gen.pl"): ["1-евъ^", "1-ъ^", "1-овъ^"],
    ("N1t*", "gen.pl"): ["1-овъ^", "2-ъ^"], ("N1j*", "gen.pl"): ["1-ей", "2-ь^"], ("N1k*", "gen.pl"): ["1-овъ^", "2-ъ^"],
    ("N1j", "gen.pl"): ["1-ей", "1-ь^"], ("N1j", "gen.sg"): ["1-ѧ", "1-а"],
    ("N1e", "acc.pl"): ["1-и^", "@gen.pl"], ("N1i", "acc.pl"): ["1-и^", "@gen.pl"], ("N1a", "acc.pl"): ["1-и^", "@gen.pl"],
    ("N1e", "dat.sg"): ["1-ю", "1-еви"],
    ("N43*", "dat.sg"): ["1-и", "1-еви", "1-ю"], ("N43", "dat.sg"): ["1-и", "1-ю"],
    ("N3t", "voc.sg"): ["1-о", "1-е"], ("N3t", "loc.pl"): ["1-ахъ", "1-ѣхъ"],
    ("N3j", "gen.pl"): ["1-ь", "1-ей"], ("N3s", "gen.pl"): ["1-ъ", "1-ей"],
    ("N3i", "gen.pl"): ["1-й", "2-ей"], ("N3a", "gen.pl"): ["1-й", "2-ей"], ("N3e", "gen.pl"): ["1-й", "2-ей"],
    ("N3i", "nom.sg"): ["@lemma"], ("N3e", "nom.sg"): ["@lemma"],
    ("N2s", "acc.pl"): ["@nom.pl", "1-ы^", "1-ѧ^"], ("N2s", "nom.du"): ["1-и^", "1-ѧ^"],
    ("N1s", "nom.du"): ["1-а^", "1-ѧ^"], ("N1sj", "nom.du"): ["1-а^", "1-ѧ^"],
    ("N3s", "dat.du"): ["1-ама", "1-ема"], ("N3s", "nom.pl"): ["1-ы^", "1-и^"],
    ("N1sj", "nom.pl"): ["1-и", "1-іе", "1-еве"], ("N1s", "nom.pl"): ["1-и", "1-іе", "1-еве"],
    # the third measured round
    ("N41", "ins.pl"): ["1-ьми", "1-ами", "1-ми"], ("N41", "dat.pl"): ["1-емъ", "1-ѧмъ"], ("N41", "loc.pl"): ["1-ехъ", "1-ѣхъ", "1-ѧхъ"],
    ("N2s", "ins.pl"): ["1-и^", "1-ами"], ("N2s", "dat.pl"): ["1-емъ^", "1-амъ"], ("N2s", "loc.pl"): ["1-ахъ", "1-ихъ"],
    ("N1j", "ins.pl"): ["1-и^", "1-ьми", "1-ми"], ("N1j*", "ins.pl"): ["1-и^", "1-ьми", "1-ми"],
    ("N3j", "dat.sg"): ["1-и", "1-ѣ"], ("N3j", "loc.pl"): ["1-ѧхъ", "1-ехъ"], ("N3j*", "dat.sg"): ["1-и", "1-ѣ"],
    ("N2j", "dat.pl"): ["1-емъ^", "1-ѧмъ"], ("N2j", "loc.pl"): ["1-ѧхъ", "1-ихъ"],
    ("N1e", "ins.sg"): ["1-емъ", "1-омъ"],
    ("N1c*", "loc.pl"): ["1-ѣхъ", "1-ахъ"], ("N1c", "loc.pl"): ["1-ѣхъ", "1-ахъ"], ("N1k*", "loc.pl"): ["3-ѣхъ", "1-ахъ"],
    ("N1c*", "dat.pl"): ["1-емъ^", "1-амъ"], ("N1c", "dat.pl"): ["1-емъ^", "1-амъ"],
    ("N1a", "dat.sg"): ["1-ю", "1-еви"],
    ("N3s", "nom.pl"): ["1-и^", "1-ы^"], ("N3s", "acc.pl"): ["1-ы^", "1-и^", "1-ъ"], ("N3s", "dat.pl"): ["1-амъ", "1-омъ^"],
    ("N1t", "ins.pl"): ["1-ы^", "1-ами", "1-ми", "1-и"], ("N1t*", "ins.pl"): ["1-ы^", "1-ами", "1-ми", "1-и"],
    ("N2t", "dat.pl"): ["1-омъ^", "1-амъ"], ("N2t", "loc.pl"): ["1-ѣхъ", "1-ахъ"], ("N2t", "ins.pl"): ["1-ы^", "1-ами"],
    ("N2t*", "dat.pl"): ["1-амъ", "1-омъ^"], ("N2t*", "loc.pl"): ["1-ахъ", "1-ѣхъ"], ("N2t*", "ins.pl"): ["1-ами", "1-ы^"],
    ("N2k", "dat.pl"): ["1-омъ^", "1-амъ"], ("N2k", "loc.pl"): ["2-ѣхъ", "1-ахъ"], ("N2k", "ins.pl"): ["1-и^", "1-ами"],
    ("N2g", "dat.pl"): ["1-омъ^", "1-амъ"], ("N2g", "loc.pl"): ["2-ѣхъ", "1-ахъ"], ("N2g", "ins.pl"): ["1-и^", "1-ами"],
    ("N1e", "gen.sg"): ["1-а", "1-ѧ"],
    ("N1j", "loc.sg"): ["1-и", "1-ѣ"], ("N1j", "nom.pl"): ["1-и", "1-іе", "1-е^"],
    ("N1j*", "loc.sg"): ["1-и", "1-ѣ"], ("N1sj", "loc.sg"): ["1-и", "1-ѣ"],
    ("N1a", "loc.sg"): ["1-и", "1-ѣ"], ("N1e", "loc.sg"): ["1-и", "1-ѣ"], ("N1i", "loc.sg"): ["1-и", "1-ѣ"],
    ("N1k", "loc.sg"): ["2-ѣ", "1-ѣ"], ("N1g", "loc.sg"): ["2-ѣ", "1-ѣ"], ("N1x", "loc.sg"): ["2-ѣ", "1-ѣ"],
    ("N1k", "voc.sg"): ["3-е", "1-е"], ("N1g", "voc.sg"): ["3-е", "1-е"], ("N1x", "voc.sg"): ["3-е", "1-е"],
}

# the fifth measured round: primaries by the alternative-preference census
PATCHES.update({
    ("N1a", "acc.pl"): ["@gen.pl", "1-и^"], ("N1e", "acc.pl"): ["@gen.pl", "1-и^"], ("N1i", "acc.pl"): ["@gen.pl", "1-и^"],
    ("N1j*", "acc.pl"): ["@gen.pl", "1-и"], ("N43*", "dat.sg"): ["1-ю", "1-и", "1-еви"],
    ("N3s", "nom.pl"): ["1-ы^", "1-и^"], ("N3s", "gen.pl"): ["1-ей", "1-ъ"],
    ("N2s", "ins.pl"): ["1-ами", "1-и^"], ("N1k*", "loc.pl"): ["1-ахъ", "3-ѣхъ"],
})

ROW_CELLS = {
    "ед.им.": ["nom.sg"],
    "ед.вин.": ["acc.sg"],
    "ед.род.": ["gen.sg"],
    "ед.дат.": ["dat.sg"],
    "ед.пр.": ["loc.sg"],
    "ед.тв.": ["ins.sg"],
    "ед.зв.": ["voc.sg"],
    "мн.им./зв.": ["nom.pl", "voc.pl"],
    "мн.вин.": ["acc.pl"],
    "мн.род.": ["gen.pl"],
    "мн.дат.": ["dat.pl"],
    "мн.пр.": ["loc.pl"],
    "мн.тв.": ["ins.pl"],
    "дв.им./вин.": ["nom.du", "acc.du"],
    "дв.род./пр.": ["gen.du", "loc.du"],
    "дв.дат./тв.": ["dat.du", "ins.du"],
}
CELL_ORDER = [
    "nom.sg", "gen.sg", "dat.sg", "acc.sg", "ins.sg", "loc.sg", "voc.sg",
    "nom.du", "gen.du", "dat.du", "acc.du", "ins.du", "loc.du", "voc.du",
    "nom.pl", "gen.pl", "dat.pl", "acc.pl", "ins.pl", "loc.pl", "voc.pl",
]


def clean(cell):
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", cell))).strip()


def canonical_ending(text):
    """`2ѡвъ` -> ('2', 'овъ', True)."""
    m = re.match(r"^(\d?)(.*)$", text)
    stem = m.group(1) or "1"
    end = m.group(2)
    mark = end.endswith("^")
    end = end.rstrip("^")
    if "ѡ" in end or "є" in end:
        mark = True
    end = end.replace("ѡ", "о").replace("є", "е").replace("у", "ꙋ").replace("я", "ѧ")
    return stem, end, mark


def parse_entry(text, cells, number):
    """One legend cell -> the class spec entries for that cell."""
    text = text.strip()
    if text.startswith("="):
        refs = text[1:].split("/")
        if refs == ["им."]:
            return [f"@nom.{number}"]
        if refs == ["им.", "род."]:
            return [f"inan:@nom.{number}", f"anim:@gen.{number}"]
        if refs == ["полн."]:
            return ["@long"]
        raise SystemExit(f"unknown reference {text}")
    # exemplar forms: `отроц-2ѣ, враз-2ѣ, дус-2ѣ` — take the first exemplar's
    first = text.split(",")[0].strip()
    if "-" not in first:
        # a bare form (the nominative of an athematic: имя, мати): the lemma
        return ["@lemma"]
    _, endings = first.rsplit("-", 1)
    out = []
    for alt in endings.split("/"):
        stem, end, mark = canonical_ending(alt)
        out.append(f"{stem}-{end}{'^' if mark else ''}")
    return out


def main():
    text = LEGEND.read_bytes().decode("utf-8")
    tables = re.findall(r"<table.*?</table>", text, flags=re.S)
    classes = {}
    for table in tables[:4]:
        rows = [re.findall(r"<t[dh][^>]*>(.*?)</t[dh]>", r, flags=re.S) for r in re.findall(r"<tr.*?</tr>", table, flags=re.S)]
        rows = [[clean(c) for c in r] for r in rows]
        header = rows[0][1:]
        exemplars = rows[1][1:]
        for ci, name in enumerate(header):
            codes = [c.strip() for c in name.split(",")]
            spec = {"exemplar": exemplars[ci].split(",")[0].strip().replace("-", "")}
            for row in rows[2:]:
                label = row[0]
                if label not in ROW_CELLS:
                    raise SystemExit(f"unknown row label {label}")
                cell_names = ROW_CELLS[label]
                number = cell_names[0].split(".")[1]
                entries = parse_entry(row[ci + 1], cell_names, number)
                for cn in cell_names:
                    spec[cn] = entries if cn == cell_names[0] else [f"@{cell_names[0]}"]
            spec["voc.du"] = ["@nom.du"]
            for code in codes:
                classes[code] = dict(spec)
    for code, twin in COPIES.items():
        if code not in classes:
            classes[code] = dict(classes[twin])
    # the animacy rule: an accusative alternative that is also a genitive
    # ending of the same number is the animate reading
    for code, spec in classes.items():
        for number in ("sg", "pl"):
            acc, gen = spec.get(f"acc.{number}", []), spec.get(f"gen.{number}", [])
            if any(e.startswith("@") or ":" in e for e in acc):
                continue
            inan = [e for e in acc if e not in gen]
            anim = [e for e in acc if e in gen]
            if anim and inan:
                if number == "sg":
                    spec[f"acc.{number}"] = [f"inan:{e}" for e in inan] + [f"anim:@gen.{number}"]
                else:
                    spec[f"acc.{number}"] = inan + [f"@gen.{number}"]
        # the plural: the nominative-shaped accusative for animates too
        if spec.get("acc.pl") == ["inan:@nom.pl", "anim:@gen.pl"]:
            spec["acc.pl"] = ["@nom.pl", "@gen.pl"]
    for (code, cell), alts in PATCHES.items():
        if code in classes:
            classes[code][cell] = alts
    # the indeclinable class
    classes["0"] = {"exemplar": "-", **{c: ["@lemma"] for c in CELL_ORDER}}
    STEMS["0"] = (0, "1=base")
    lines = [
        "# Noun letter classes, seeded from Polyakov's legend (flexslav.htm) by",
        "# scripts/polyakov-legend-to-classes.py; hand-maintained since.",
        "# Columns: class, strip (lemma letters that are the ending), stems",
        "# (n=derivation; base|drop|insert|pal1[:x]|pal2[:x]|ext:suffix), then",
        "# cell=spec where spec is `|`-separated alternatives, primary first:",
        "#   N-ending[^]   stem N plus the ending, ^ = the number mark",
        "#   @cell         the same as that cell; @lemma the lemma itself",
        "#   anim:… inan:… an alternative that applies to that animacy only",
        "\t".join(["class", "exemplar", "strip", "stems"] + CELL_ORDER),
    ]
    for code in sorted(classes, key=lambda c: (c == "0", c)):
        if code not in STEMS:
            print(f"warning: no stems spec for {code}, skipped", file=sys.stderr)
            continue
        spec = classes[code]
        strip, stems = STEMS[code]
        cols = [code, spec["exemplar"], str(strip), stems]
        for cell in CELL_ORDER:
            cols.append("|".join(spec.get(cell, ["-"])))
        lines.append("\t".join(cols))
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(classes)} classes to {OUT}")


if __name__ == "__main__":
    main()
