#!/usr/bin/env python3
"""Seed lexicon/classes/{adj,verb,pronoun}.tsv from Polyakov's legend.

Part 3 of V2-PROMPT.md. Like polyakov-legend-to-classes.py for nouns: the
legend's tables become class rows; the stem-derivation spec per class is
hand-written here; the marks and primaries are then measured by the
importer (`--fix-marks`, the alternative-preference census). Re-running
the script must be a deliberate act, followed by the measurement.

Conventions as for nouns: `у` -> `ꙋ`, `я` -> `ѧ`, the wide `ѡ`/`є` and `^`
become the narrow letter plus `^` (Form::number_mark).
"""
import html
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEGEND = ROOT / "references/downloads/polyakov/flexslav.htm"
CLASSES = ROOT / "crates/church-slavonic/lexicon/classes"

GENDERS = ["m", "f", "n"]
NUMBERS = ["sg", "du", "pl"]
CASES = ["nom", "gen", "dat", "acc", "ins", "loc", "voc"]


def clean(cell):
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", cell))).strip()


def tables():
    text = LEGEND.read_bytes().decode("utf-8")
    out = []
    for table in re.findall(r"<table.*?</table>", text, flags=re.S):
        rows = [re.findall(r"<t[dh][^>]*>(.*?)</t[dh]>", r, flags=re.S) for r in re.findall(r"<tr.*?</tr>", table, flags=re.S)]
        out.append([[clean(c) for c in r] for r in rows])
    return out


def ending(text):
    """`2ѡвъ` -> ('2', 'овъ', mark)."""
    m = re.match(r"^(\d?)(.*)$", text.strip())
    stem = m.group(1) or "1"
    end = m.group(2)
    mark = end.endswith("^")
    end = end.rstrip("^")
    if "ѡ" in end or "є" in end:
        mark = True
    end = end.replace("ѡ", "о").replace("є", "е").replace("у", "ꙋ").replace("я", "ѧ").replace("#", "")
    return stem, end, mark


def alternatives(text):
    """One legend cell -> list of `N-ending[^]` specs (first exemplar only)."""
    text = text.strip()
    if not text or text in ("-", "?"):
        return []
    first = text.split(",")[0].strip()
    if "-" not in first:
        return []
    _, ends = first.split("-", 1)
    out = []
    for alt in re.split(r"\s*/\s*", ends):
        alt = alt.strip()
        # the dual's (-ѣ) variant: `ива(-ѣ)` -> ива | ивѣ
        m = re.match(r"^(.*?)\((-?)(.*)\)$", alt)
        if m:
            head = m.group(1)
            tail = m.group(3).replace("у", "ꙋ").replace("я", "ѧ")
            s, e, k = ending(head)
            out.append(f"{s}-{e}{'^' if k else ''}")
            out.append(f"{s}-{e[:-1]}{tail}{'^' if k else ''}")
            continue
        s, e, k = ending(alt)
        out.append(f"{s}-{e}{'^' if k else ''}")
    # dedupe, keep order
    seen = []
    for o in out:
        if o not in seen:
            seen.append(o)
    return seen


def write(path, header_cells, rows, comment):
    # every cell any class names is a column: a cell set on a row but
    # absent from the fixed header would silently vanish (it did, twice)
    header_cells = list(header_cells)
    for _, _, _, _, cells in rows:
        for c in cells:
            if c not in header_cells:
                header_cells.append(c)
    lines = [f"# {c}" for c in comment.strip().split("\n")]
    lines.append("\t".join(["class", "exemplar", "strip", "stems"] + header_cells))
    for code, exemplar, strip, stems, cells in rows:
        lines.append("\t".join([code, exemplar, str(strip), stems] + [cells.get(c, "-") for c in header_cells]))
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(rows)} classes to {path}")


# ---------------------------------------------------------------------------
# Adjectives (table 4)
# ---------------------------------------------------------------------------

# (class, strip, stems). Stem 1 is the base; 2 the short masculine
# nominative's stem where it differs (the fleeting vowel inserted, the
# geminate cut); 3 = base + ѣ for the short comparative nominatives; 4 = the
# comparative stem base + ѣйш (айш after a husher).
ADJ_STEMS = {
    "A1t": (2, "1=base;2=base;3=ext:ѣ;4=ext:ѣйш"),
    "A1t*": (2, "1=base;2=insert;3=ext:ѣ;4=ext:ѣйш"),
    "A1n*": (2, "1=base;2=cut;3=ext:ѣ;4=ext:ѣйш"),
    "A2t": (1, "1=base;2=base;3=ext:ѣ;4=ext:ѣйш"),
    "A1j": (2, "1=base;2=base;3=ext:ѣ;4=ext:ѣйш"),
    "A1j*": (2, "1=base;2=insert;3=ext:ѣ;4=ext:ѣйш"),
    "A2j": (1, "1=base;2=base;3=ext:ѣ;4=ext:ѣйш"),
    "A1k": (2, "1=base;2=base;3=ext:ѣ;4=ext:айш:pal1;5=pal2"),
    "A1g": (2, "1=base;2=base;3=ext:ѣ;4=ext:айш:pal1;5=pal2"),
    "A1x": (2, "1=base;2=base;3=ext:ѣ;4=ext:айш:pal1;5=pal2"),
    "A1k*": (2, "1=base;2=insert;3=ext:ѣ;4=ext:айш:pal1;5=pal2"),
    "A1sk": (2, "1=base;2=base;3=ext:ѣ;4=ext:айш:pal1;5=pal2"),
    "A1s": (2, "1=base;2=base;3=ext:ѣ;4=ext:айш"),
    "A1sx": (2, "1=base;2=base;3=ext:ѣ;4=ext:айш"),
    "A1i": (1, "1=base;2=base;3=ext:ѣ;4=ext:ѣйш"),
    "A2i": (1, "1=base;2=base;3=ext:ѣ;4=ext:ѣйш"),
}
# legend column -> the classes that use it
ADJ_COLUMNS = {
    "A1t, A2t": ["A1t", "A1t*", "A1n*", "A2t"],
    "A1j, A2j": ["A1j", "A1j*", "A2j"],
    "A1k, A1g": ["A1k", "A1g", "A1x", "A1k*", "A1sk"],
    "A1s": ["A1s", "A1sx"],
    "A1i": ["A1i", "A2i"],
}
# the second palatalisation column exemplars (велиц-2ѣ): stem 2 in the
# legend is our stem 5 for the velar classes
VELAR = {"A1k", "A1g", "A1x", "A1k*", "A1sk"}

ADJ_ROWS = {
    "ед.м.им./вин.": [("m", "sg", "nom")],
    "ед.ср.им./вин.": [("n", "sg", "nom"), ("n", "sg", "acc")],
    "ед.м./ср.род.": [("m", "sg", "gen"), ("n", "sg", "gen")],
    "ед.м.вин.": [("m", "sg", "acc")],
    "ед.м./ср.дат.": [("m", "sg", "dat"), ("n", "sg", "dat")],
    "ед.м./ср.пр.": [("m", "sg", "loc"), ("n", "sg", "loc")],
    "ед.м./ср.тв.": [("m", "sg", "ins"), ("n", "sg", "ins")],
    "ед.ж.им.": [("f", "sg", "nom")],
    "ед.ж.вин.": [("f", "sg", "acc")],
    "ед.ж.род.": [("f", "sg", "gen")],
    "ед.ж.дат./пр.": [("f", "sg", "dat"), ("f", "sg", "loc")],
    "ед.ж.тв.": [("f", "sg", "ins")],
    "мн.м.им.": [("m", "pl", "nom")],
    "мн.м.вин.|мн.ж.им./вин.": [("m", "pl", "acc"), ("f", "pl", "nom"), ("f", "pl", "acc")],
    "мн.ср.им./вин.": [("n", "pl", "nom"), ("n", "pl", "acc")],
    "мн.род./пр.": [(g, "pl", c) for g in GENDERS for c in ("gen", "loc")],
    "мн.дат.": [(g, "pl", "dat") for g in GENDERS],
    "мн.тв.": [(g, "pl", "ins") for g in GENDERS],
    "дв.м.им./вин.": [("m", "du", "nom"), ("m", "du", "acc")],
    "дв.ср./ж.им./вин.": [(g, "du", c) for g in ("n", "f") for c in ("nom", "acc")],
    "дв.род./пр.": [(g, "du", c) for g in GENDERS for c in ("gen", "loc")],
    "дв.дат./тв.": [(g, "du", c) for g in GENDERS for c in ("dat", "ins")],
}


def adjectives(table):
    header = table[0][1:]
    kinds = table[2][1:]  # крат./полн. per column
    rows = table[3:]
    out = []
    # columns come in (short, long) pairs per header entry
    for hi, name in enumerate(header):
        short_col, long_col = 2 * hi + 1, 2 * hi + 2
        codes = ADJ_COLUMNS.get(name)
        if not codes:
            continue
        for code in codes:
            strip, stems = ADJ_STEMS[code]
            cells = {}
            for row in rows:
                label = row[0]
                targets = ADJ_ROWS.get(label)
                if not targets:
                    continue
                for series, col in (("short", short_col), ("long", long_col)):
                    text = row[col] if col < len(row) else ""
                    spec = None
                    if text.startswith("=полн."):
                        spec = ["@long.pos.{}.{}.{}".format(*t) for t in targets[:1]]
                        spec = None  # filled per target below
                    for g, n, c in targets:
                        key = f"{series}.pos.{g}.{n}.{c}"
                        if text.startswith("=полн."):
                            cells[key] = f"@long.pos.{g}.{n}.{c}"
                            continue
                        alts = alternatives(text)
                        if not alts:
                            continue
                        if code in VELAR:
                            alts = [a.replace("2-", "5-", 1) if a.startswith("2-") else a for a in alts]
                        # the short masculine nominative on stem 2 (fleeting)
                        if series == "short" and (g, n, c) == ("m", "sg", "nom"):
                            alts = [a.replace("1-", "2-", 1) if a.startswith("1-") else a for a in alts]
                        cells[key] = "|".join(alts)
            # the masculine accusative: the animate (genitive-shaped) reading
            # first where the legend lists it, the nominative-shaped second
            for series in ("short", "long"):
                acc = cells.get(f"{series}.pos.m.sg.acc")
                nom = f"@{series}.pos.m.sg.nom"
                cells[f"{series}.pos.m.sg.acc"] = f"{acc}|{nom}" if acc else nom
                for n in ("du", "pl"):
                    if f"{series}.pos.m.{n}.acc" not in cells:
                        cells[f"{series}.pos.m.{n}.acc"] = f"@{series}.pos.m.{n}.nom"
                # vocatives: the short masculine singular takes -е, the rest
                # answer with the nominative
                for g in GENDERS:
                    for n in NUMBERS:
                        key = f"{series}.pos.{g}.{n}.voc"
                        if series == "short" and g == "m" and n == "sg":
                            cells[key] = "1-е"
                        else:
                            cells[key] = f"@{series}.pos.{g}.{n}.nom"
            # the comparative: stem 4 declined as A1s, the short nominatives
            # on stem 3 (base + ѣ); the short plural nominative/accusative
            # take the participle-like -ше/-шѧ (велегла́снѣйше, мно́жайшѧ)
            cells["short.comp"] = "4~A1s"
            cells["long.comp"] = "4~A1s"
            cells["short.comp.m.sg.nom"] = "3-й"
            cells["short.comp.m.sg.acc"] = "3-й|4-а"
            cells["short.comp.n.sg.nom"] = "3-е|4-е"
            cells["short.comp.n.sg.acc"] = "3-е|4-е"
            cells["short.comp.f.sg.nom"] = "4-и"
            cells["short.comp.m.pl.nom"] = "4-е|4-и"
            cells["short.comp.m.pl.acc"] = "4-ѧ|4-и"
            cells["short.comp.f.pl.nom"] = "4-ѧ|4-и"
            cells["short.comp.f.pl.acc"] = "4-ѧ|4-и"
            cells["long.comp.m.sg.nom"] = "4-ій"
            # measured (Part 3): the short plural instrumental is the long
            # -ыми/-ими; the locative takes -омъ/-емъ beside -ѣмъ; the
            # feminine dative/locative -ой beside the palatalised -ѣй; the
            # nominative-shaped masculine accusative is the majority
            for g in GENDERS:
                ins = cells.get(f"short.pos.{g}.pl.ins", "")
                alts = [a for a in ins.split("|") if a]
                alts.sort(key=lambda a: 0 if a.endswith("ми") else 1)
                if alts:
                    cells[f"short.pos.{g}.pl.ins"] = "|".join(alts)
            for g in ("m", "n"):
                loc = cells.get(f"long.pos.{g}.sg.loc", "")
                extra = "1-емъ" if code in ("A1j", "A1j*", "A2j", "A1s", "A1sx", "A1i", "A2i") else "1-омъ"
                if extra not in loc.split("|"):
                    cells[f"long.pos.{g}.sg.loc"] = f"{loc}|{extra}" if loc else extra
            for c in ("dat", "loc"):
                fl = cells.get(f"long.pos.f.sg.{c}", "")
                if code in VELAR and "1-ой" not in fl.split("|"):
                    cells[f"long.pos.f.sg.{c}"] = f"{fl}|1-ой"
            for series in ("short", "long"):
                acc = cells.get(f"{series}.pos.m.sg.acc", "")
                alts = [a for a in acc.split("|") if a]
                alts.sort(key=lambda a: 0 if a.startswith("@") else 1)
                cells[f"{series}.pos.m.sg.acc"] = "|".join(alts)
            soft = code in ("A1j", "A1j*", "A2j", "A1s", "A1sx", "A1i", "A2i")
            if code in VELAR:
                # the -ск- stems (A1sk) palatalise to -ст- and take -іи
                # (ага́рѧнстїи, 147/152); the rest -цыи beside -цыи
                cells["long.pos.m.pl.nom"] = "5-іи|5-ыи" if code == "A1sk" else "5-ыи|5-іи|1-іе"
                cells["short.pos.m.pl.nom"] = "5-и|5-ы" if code == "A1sk" else "5-ы|5-и"
            elif not soft:
                # Polyakov's Russian-shaped long plural (во́льные), attested
                cells["long.pos.m.pl.nom"] = cells["long.pos.m.pl.nom"] + "|1-ые"
            # the short feminine nominative: Polyakov also tags the plural
            # -ы/-и as the feminine singular (безнавѣ́тны, вчера́шни: 615
            # overrides)
            cells["short.pos.f.sg.nom"] = cells["short.pos.f.sg.nom"] + ("|1-и" if soft else "|1-ы")
            # the animate accusative first on the soft and velar classes
            # (A1k 142/154, A1j* 15/16, A1s 34/52)
            if code in ("A1k", "A1k*", "A1sk", "A1g", "A1x", "A1j*", "A1s", "A1sx"):
                alts = cells["short.pos.m.sg.acc"].split("|")
                alts.sort(key=lambda a: 1 if a.startswith("@") else 0)
                cells["short.pos.m.sg.acc"] = "|".join(alts)
            if code in ("A2t", "A2j"):
                # the possessives: the marked plural nominative (а҆арѡ́нѡвы)
                # and the short plural instrumental (191/216, 108/109)
                cells["short.pos.m.pl.nom"] = ("1-ы^|1-и" if code == "A2t" else "1-и^")
                for g in GENDERS:
                    alts = cells[f"short.pos.{g}.pl.ins"].split("|")
                    alts.sort(key=lambda a: 1 if a.endswith("ми") else 0)
                    cells[f"short.pos.{g}.pl.ins"] = "|".join(alts)
            if code in ("A1t", "A1t*"):
                cells["short.pos.m.sg.nom"] = cells["short.pos.m.sg.nom"] + "|1-ь"
            if code == "A1n*":
                # the short series of the -нный adjectives keeps a single н
                # (свѧще́ни, сокращє́ны) beside the double one; the masculine
                # nominative also inserts е (тлѣ́ненъ, stem 13)
                for k, v in list(cells.items()):
                    if k.startswith("short.pos") and "1-" in v:
                        cells[k] = "|".join([a.replace("1-", "2-", 1) for a in v.split("|") if a.startswith("1-")] + [a for a in v.split("|")])
                cells["short.pos.m.sg.nom"] = "2-ъ|13-ъ"
                stems = stems + ";13=insert"
            # the number mark where the plural's letters equal a singular's
            # (measured, --fix-marks): the soft long plural -їѧ against the
            # feminine genitive, ни́щи against the locative, бѡ́жїѧ's dual
            def mark(key):
                if key in cells and not cells[key].split("|")[0].endswith("^") and not cells[key].startswith("@"):
                    alts = cells[key].split("|")
                    alts[0] += "^"
                    cells[key] = "|".join(alts)
            if code in ("A1j", "A1j*", "A2j"):
                for k in ("long.pos.f.pl.nom", "long.pos.f.pl.acc", "long.pos.n.pl.nom", "long.pos.n.pl.acc", "long.pos.m.pl.acc"):
                    mark(k)
            if code in ("A1s", "A1sx"):
                mark("short.pos.m.pl.nom")
            if code in ("A1i", "A2i"):
                mark("short.pos.m.du.nom")
                mark("short.pos.m.du.acc")
            # the adverb (2.2 Part 2): the neuter short nominative's ending
            # with the mark that prints the wide ѡ (мꙋ́дрѡ, до́брѡ) beside the
            # short locative (до́брѣ); the comparative adverb is the short
            # comparative neuter nominative (мꙋдрѣ́е)
            nom = cells.get("short.pos.n.sg.nom", "").split("|")[0]
            loc = cells.get("short.pos.m.sg.loc", "").split("|")[0]
            if nom and not nom.startswith("@"):
                adv = [nom.rstrip("^") + "^"]
                if loc and not loc.startswith("@") and loc != nom:
                    adv.append(loc)
                cells["adv"] = "|".join(adv)
                cells["comp.adv"] = "@short.comp.n.sg.nom"
            exemplar = table[1][1:][hi].split(",")[0].strip().replace("-", "")
            if code.startswith("A2") and code != "A2i":
                exemplar = table[1][1:][hi].split(",")[-1].strip().replace("-", "") if "," in table[1][1:][hi] else exemplar
            out.append((code, exemplar, strip, stems, cells))
    header_cells = [f"{s}.pos.{g}.{n}.{c}" for s in ("short", "long") for g in GENDERS for n in NUMBERS for c in CASES]
    header_cells += ["short.comp", "long.comp", "short.comp.m.sg.nom", "short.comp.n.sg.nom", "short.comp.n.sg.acc", "short.comp.f.sg.nom", "long.comp.m.sg.nom", "adv", "comp.adv"]
    return header_cells, out


# ---------------------------------------------------------------------------
# Pronominal adjectives (table 5) and the personal pronouns (table 6)
# ---------------------------------------------------------------------------

PRON_STEMS = {
    "PA1": (1, "1=base"),
    "PA1n": (1, "1=base"),
    "PA1t": (2, "1=base"),
    "PA1j": (2, "1=base"),
    "PA1j*": (1, "1=drop;2=base"),
    "PA1k": (2, "1=base"),
    "PA1s": (1, "1=base"),
    "PA1a": (1, "1=base"),
}
PRON_ROWS = {k: [(g, n, c) for (g, n, c) in v] for k, v in ADJ_ROWS.items()}


def pronominal(table):
    header = table[0][1:]
    rows = table[3:]
    out = []
    # PA1 has short and long columns; the others short only
    col = 1
    for hi, code in enumerate(header):
        ncols = 2 if code == "PA1" else 1
        short_col = col
        col += ncols
        strip, stems = PRON_STEMS[code]
        cells = {}
        for row in rows:
            targets = PRON_ROWS.get(row[0])
            if not targets:
                continue
            text = row[short_col] if short_col < len(row) else ""
            for g, n, c in targets:
                key = f"{g}.{n}.{c}"
                if text.startswith("=полн."):
                    continue
                alts = alternatives(text)
                if alts:
                    if (g, n, c) == ("m", "sg", "nom") and code.endswith("*"):
                        alts = [a.replace("1-", "2-", 1) if a.startswith("1-") else a for a in alts]
                    cells[key] = "|".join(alts)
        acc = cells.get("m.sg.acc")
        # the nominative-shaped accusative first: the 1.x tables' primary
        cells["m.sg.acc"] = f"@m.sg.nom|{acc}" if acc else "@m.sg.nom"
        if code == "PA1n" and cells.get("f.sg.acc") == "1-ꙋю":
            cells["f.sg.acc"] = "1-ꙋ|1-ꙋю"
        if code == "PA1s":
            # the possessives' plural dative writes ы after the husher
            # (на́шымъ), the neuter plural takes the kamora (на̑ша)
            for g in GENDERS:
                cells[f"{g}.pl.dat"] = "1-ымъ"
            for c in ("nom", "acc"):
                cells[f"n.pl.{c}"] = "1-а^"
        if code in ("PA1t", "PA1j", "PA1j*"):
            # the dual genitive/locative is unmarked (то́ю, се́ю, всѣ́ю);
            # the plural dative takes the kamora (тѣ̑мъ, си̑мъ)
            for g in GENDERS:
                for c in ("gen", "loc"):
                    if f"{g}.du.{c}" in cells:
                        cells[f"{g}.du.{c}"] = cells[f"{g}.du.{c}"].replace("^", "")
                if f"{g}.pl.dat" in cells and "^" not in cells[f"{g}.pl.dat"]:
                    cells[f"{g}.pl.dat"] = cells[f"{g}.pl.dat"].split("|")[0] + "^"
        if code == "PA1t":
            cells["f.pl.nom"] = "1-ыѧ^|1-ы^"
            cells["f.pl.acc"] = "1-ыѧ^|1-ы^"
        for n in ("du", "pl"):
            cells.setdefault(f"m.{n}.acc", f"@m.{n}.nom")
        for g in GENDERS:
            for n in NUMBERS:
                cells[f"{g}.{n}.voc"] = f"@{g}.{n}.nom"
        exemplar = table[1][1:][hi].split(",")[0].strip().replace("-", "")
        out.append((code, exemplar, strip, stems, cells))
        if code == "PA1t":
            # the velar twin (такі́й, какі́й, ꙗ҆кі́й): то́й's endings with the
            # second palatalisation before ѣ and ы (та́цѣмъ, та́цы) and the
            # -ій nominative
            velar = {}
            for k, v in cells.items():
                velar[k] = "|".join("5-" + a[2:] if a.startswith(("1-ѣ", "1-ы")) else a for a in v.split("|"))
            velar["m.sg.nom"] = "1-ій"
            velar["m.sg.acc"] = "@m.sg.nom|1-ого"
            out.append(("PA1tk", "такій", 2, "1=base;5=pal2", velar))
        if code == "PA1":
            # the fleeting-vowel hard pronominal (ѻ҆ди́нъ: ѻ҆дногѡ̀)
            fleeting = dict(cells)
            fleeting["m.sg.nom"] = "2-ъ"
            fleeting["m.sg.acc"] = "@m.sg.nom|1-ого"
            out.append(("PA1*", "одинъ", 1, "1=drop;2=base", fleeting))
    header_cells = [f"{g}.{n}.{c}" for g in GENDERS for n in NUMBERS for c in CASES]
    return header_cells, out


PERSONAL_ROWS = {
    "им.": "nom", "род.": "gen", "вин.": "acc", "дат.": "dat", "пр.": "loc", "тв.": "ins",
    "вин.клит.": "clit.acc", "дат.клит.": "clit.dat",
    "дв.им.": "du.nom", "дв.вин.": "du.acc", "дв.род./пр.": "du.gen|du.loc", "дв.дат./тв.": "du.dat|du.ins",
}
# class -> (strip, stems, person, number) — the stems are on the lexeme line
PERSONAL = {
    "PPja": ("1", "sg"), "PPty": ("2", "sg"), "PPseb": (None, None), "PPmy": ("1", "pl"), "PPvy": ("2", "pl"),
    "PPkto": (None, "sg"), "PPcto": (None, "sg"),
}


def personal(table):
    """The personal pronouns: the legend spells a different stem per cell
    (мен-е, мн-ѣ, м-я); stems are numbered by first appearance and the
    lexeme line must spell them (`stems=1=мен;2=мн;3=м`), which the
    exemplar column records as `азъ:мен,мн,м`."""
    header = table[0][1:]
    rows = table[2:]
    out = []
    names = []
    for hi, code in enumerate(header):
        if code == "PPmy" and hi == 4:
            code = "PPvy"
        col = hi + 1
        person, number = PERSONAL[code]
        cells = {}
        stems = []
        for row in rows:
            case = PERSONAL_ROWS.get(row[0])
            if not case:
                continue
            text = row[col].strip() if col < len(row) else ""
            if not text or text == "-":
                continue
            specs = []
            for alt in re.split(r"\s*/\s*", text):
                if "-" not in alt:
                    specs.append("@lemma")
                    continue
                stem, end = alt.split("-", 1)
                stem = stem.replace("у", "ꙋ").replace("я", "ѧ")
                if stem not in stems:
                    stems.append(stem)
                _, e, k = ending(end)
                specs.append(f"{stems.index(stem) + 1}-{e}{'^' if k else ''}")
            for target in case.split("|"):
                parts = []
                clitic = target.startswith("clit.")
                c = target.split(".")[-1]
                n = "du" if target.startswith("du.") else number
                if clitic:
                    parts.append("clit")
                if person:
                    parts.append(person)
                if code in ("PPkto", "PPcto"):
                    parts.append("n" if code == "PPcto" else "m")
                if n:
                    parts.append(n)
                parts.append(c)
                key = ".".join(parts)
                cells[key] = "|".join(specs)
                if key not in names:
                    names.append(key)
        # the number mark sits on the genitive (менє̀, тебє̀, себє̀: the
        # 1.x tables, Alypy §47), not the accusative the legend marks
        for key, v in list(cells.items()):
            last = key.split(".")[-1]
            if last == "gen" and not key.startswith("clit."):
                cells[key] = "|".join(a if a.endswith("^") or a.startswith("@") else a + "^" for a in v.split("|"))
            elif last == "acc" and not key.startswith("clit."):
                cells[key] = v.replace("^", "")
        for key in [k for k in cells if k.endswith(".nom") and not k.startswith("clit.")]:
            voc = key[: -len("nom")] + "voc"
            cells[voc] = f"@{key}"
            if voc not in names:
                names.append(voc)
        exemplar = table[1][1:][hi].replace("-", "") + ":" + ",".join(stems)
        out.append((code, exemplar, 0, ";".join(f"{i + 1}=base" for i in range(len(stems))) or "1=base", cells))
    # the third person (1.x personal matrix, Alypy §47): every cell is a
    # literal on the empty stem; the н- forms after a preposition and the
    # sense rows' alternatives second; the enclitic accusatives и҆̀/ю҆̀/є҆̀/ѧ҆̀
    third = {}
    sg = {
        "m": {"nom": "1-онъ", "gen": "1-его^|1-него^", "dat": "1-емꙋ|1-немꙋ", "acc": "1-его|1-него", "ins": "1-имъ|1-нимъ", "loc": "1-немъ|1-емъ"},
        "f": {"nom": "1-она", "gen": "1-еѧ|1-ее|1-неѧ", "dat": "1-ей|1-ней", "acc": "1-ю|1-ню", "ins": "1-ею|1-нею", "loc": "1-ней"},
        "n": {"nom": "1-оно", "gen": "1-его^|1-него^", "dat": "1-емꙋ|1-немꙋ", "acc": "1-е", "ins": "1-имъ|1-нимъ", "loc": "1-немъ|1-емъ"},
    }
    du = {"gen": "1-нею|1-ею", "dat": "1-има|1-нима", "acc": "1-ѧ|1-нѧ", "ins": "1-има|1-нима", "loc": "1-нею|1-ею"}
    pl = {"gen": "1-ихъ|1-нихъ", "dat": "1-имъ|1-нимъ", "acc": "1-ихъ|1-нихъ|1-нѧ", "ins": "1-ими|1-ними", "loc": "1-нихъ|1-ихъ"}
    for g in ("m", "f", "n"):
        for c, v in sg[g].items():
            third[f"3.{g}.sg.{c}"] = v
        third[f"3.{g}.sg.voc"] = f"@3.{g}.sg.nom"
        third[f"3.{g}.du.nom"] = "1-она" if g == "m" else "1-онѣ"
        for c, v in du.items():
            third[f"3.{g}.du.{c}"] = v
        third[f"3.{g}.du.voc"] = f"@3.{g}.du.nom"
        third[f"3.{g}.pl.nom"] = "1-онѣ" if g == "f" else "1-они"
        for c, v in pl.items():
            third[f"3.{g}.pl.{c}"] = "1-ѧ|1-нѧ" if (g == "n" and c == "acc") else v
        third[f"3.{g}.pl.voc"] = f"@3.{g}.pl.nom"
        third[f"clit.3.{g}.sg.acc"] = {"m": "1-и", "f": "1-ю", "n": "1-е"}[g]
        third[f"clit.3.{g}.du.acc"] = "1-ѧ"
        third[f"clit.3.{g}.pl.acc"] = "1-ѧ"
    for key in third:
        if key not in names:
            names.append(key)
    out.append(("PP3", "онъ", 3, "1=base", third))
    relative = {}
    for key, v in third.items():
        if key.startswith("clit."):
            continue
        g, n, c = key.split(".")[1:]
        relative[f"{g}.{n}.{c}"] = v.replace("@3.", "@")
    relative.update({
        "m.sg.nom": "1-и", "f.sg.nom": "1-ꙗ", "n.sg.nom": "1-е",
        "m.du.nom": "1-ꙗ", "f.du.nom": "1-и", "n.du.nom": "1-и",
        "m.pl.nom": "1-и", "f.pl.nom": "1-ꙗ", "n.pl.nom": "1-ꙗ",
        "n.sg.acc": "1-е",
    })
    for key in relative:
        if key not in names:
            names.append(key)
    out.append(("PPize", "иже", 3, "1=base", relative))
    return names, out


# ---------------------------------------------------------------------------
# Verbs (tables 7–11)
# ---------------------------------------------------------------------------

# (strip, stems). Stem 1 the infinitive stem; 2 the present stem where it
# differs; 3 the imperative's second palatalisation; the participle stems
# 5..8 built on the right base: 5 present active (-ѧщ/-ꙋщ), 6 present
# passive (-им/-ем), 7 past active (-вш/-ш), 8 past passive (-ен/-ан/-т).
VERB_STEMS = {
    "V21n": (3, "1=base;2=base;5=ext:ѧщ;6=ext:им;7=ext:ивш;8=ext:ен;9=ext:ѧ;11=ext:ив;12=ext:н:ext:ен;14=ext:ьш"),
    "V21a": (3, "1=base;2=base;5=ext:ѧщ;6=ext:им;7=ext:ивш;8=ext:ен;9=ext:ѧ;11=ext:ив;12=ext:н:ext:ен"),
    "V21s": (3, "1=base;2=base;5=ext:ащ;6=ext:им;7=ext:ивш;8=ext:ен;9=ext:а;11=ext:ив;12=ext:н:ext:ен;14=ext:ш"),
    "V21p": (3, "1=base;2=iot;5=ext:ѧщ;6=ext:им;7=ext:ивш;8=ext:ен:iot;9=ext:ѧ;11=ext:ив;12=ext:н:ext:ен:iot;14=ext:ьш:iot"),
    "V21t": (3, "1=base;2=iot;5=ext:ѧщ;6=ext:им;7=ext:ивш;8=ext:ен:iot;9=ext:ѧ;11=ext:ив;12=ext:н:ext:ен:iot;14=ext:ш:iot"),
    "V22n": (3, "1=base;2=base;5=ext:ѧщ;6=ext:им;7=ext:ѣвш;8=ext:ен;9=ext:ѧ;11=ext:ѣв;12=ext:н:ext:ен"),
    "V22p": (3, "1=base;2=iot;5=ext:ѧщ;6=ext:им;7=ext:ѣвш;8=ext:ен:iot;9=ext:ѧ;11=ext:ѣв;12=ext:н:ext:ен:iot"),
    "V22t": (3, "1=base;2=iot;5=ext:ѧщ;6=ext:им;7=ext:ѣвш;8=ext:ен:iot;9=ext:ѧ;11=ext:ѣв;12=ext:н:ext:ен:iot"),
    "V22s": (3, "1=base;2=base;5=ext:ащ;6=ext:им;7=ext:авш;8=ext:ан;9=ext:а;11=ext:ав;12=ext:н:ext:ан"),
    "V22a": (3, "1=base;2=base;5=ext:ѧщ;6=ext:им;7=ext:ѧвш;8=ext:ѧн;9=ext:ѧ;11=ext:ѧв;12=ext:ѧн"),
    "V11a": (2, "1=base;2=base;5=ext:ющ;6=ext:ем;7=ext:вш;8=ext:н;9=ext:ю;11=ext:в;12=ext:н"),
    "V11e": (2, "1=base;2=cut;5=ext:ющ;6=ext:ем;7=ext:вш;8=ext:н;9=ext:ю;11=ext:в;12=ext:н"),
    "V12ov": (2, "1=base;2=ov;5=ext:ющ:ov;6=ext:ем:ov;7=ext:вш;8=ext:н;9=ext:ю:ov;11=ext:в;12=ext:н"),
    "V12n": (3, "1=base;2=base;5=ext:ющ;6=ext:ем;7=ext:авш;8=ext:ан;9=ext:ю;11=ext:ав;12=ext:н:ext:ан"),
    "V12p": (3, "1=base;2=iot;5=ext:ющ:iot;6=ext:ем:iot;7=ext:авш;8=ext:ан;9=ext:ю:iot;11=ext:ав;12=ext:н:ext:ан"),
    "V12t": (3, "1=base;2=iot;5=ext:ꙋщ:iot;6=ext:ем:iot;7=ext:авш;8=ext:ан;9=ext:ꙋ:iot;11=ext:ав;12=ext:н:ext:ан"),
    "V12k": (3, "1=base;2=iot;5=ext:ꙋщ:iot;6=ext:ем:iot;7=ext:авш;8=ext:ан;9=ext:ꙋ:iot;11=ext:ав;12=ext:н:ext:ан"),
    "V12a": (3, "1=base;2=base;5=ext:ющ;6=ext:ем;7=ext:ѧвш;8=ext:ѧн;9=ext:ю;11=ext:ѧв;12=ext:ѧн"),
    "V12x": (3, "1=base;2=base;5=ext:ꙋщ;6=ext:ем;7=ext:авш;8=ext:ан;9=ext:ꙋ;11=ext:ав;12=ext:н:ext:ан"),
    "V12x*": (3, "1=base;2=insert;5=ext:ꙋщ:insert;6=ext:ем:insert;7=ext:авш;8=ext:ан;9=ext:ꙋ:insert;11=ext:ав;12=ext:н:ext:ан"),
    "V13a": (3, "1=base;2=base;5=ext:ꙋщ;6=ext:ем;7=ext:ꙋвш;8=ext:ꙋт;9=ext:ꙋ;11=ext:ꙋв;12=ext:ꙋт"),
    "V13t": (4, "1=base;2=base;5=ext:нꙋщ;6=ext:нем;7=ext:нꙋвш;8=ext:нꙋт;9=ext:нꙋ;11=ext:нꙋв;12=ext:нꙋт;14=ext:ш"),
    "V13k": (4, "1=base;2=iot;5=ext:нꙋщ;6=ext:нем;7=ext:нꙋвш;8=ext:нꙋт;9=ext:нꙋ;11=ext:нꙋв;12=ext:нꙋт;14=ext:ш"),
    "V14p": (3, "1=ext:б;2=ext:б;5=ext:ꙋщ:ext:б;6=ext:ом:ext:б;7=ext:ш:ext:б;8=ext:ен:ext:б;9=ext:ꙋ:ext:б;11=ext:б;12=ext:н:ext:ен:ext:б"),
    "V14z": (2, "1=base;2=base;5=ext:ꙋщ;6=ext:ом;7=ext:ш;8=ext:ен;9=ext:ꙋ;11=base;12=ext:н:ext:ен"),
    "V14t": (3, "1=ext:т;2=ext:т;5=ext:ꙋщ:ext:т;6=ext:ом:ext:т;7=ext:ш:ext:т;8=ext:ен:ext:т;9=ext:ꙋ:ext:т;11=ext:т;12=ext:н:ext:ен:ext:т"),
    "V14d": (3, "1=ext:д;2=ext:д;5=ext:ꙋщ:ext:д;6=ext:ом:ext:д;7=ext:ш:ext:д;8=ext:ен:ext:д;9=ext:ꙋ:ext:д;11=ext:д;12=ext:н:ext:ен:ext:д"),
    "V14st": (2, "1=ext:т;2=ext:т;5=ext:ꙋщ:ext:т;6=ext:ом:ext:т;7=ext:ш:ext:т;8=ext:ен:ext:т;9=ext:ꙋ:ext:т;11=ext:т;12=ext:н:ext:ен:ext:т"),
    "V14t*": (3, "1=ext:т:drop;2=ext:т:drop;5=ext:ꙋщ:ext:т:drop;6=ext:ом:ext:т:drop;7=ext:ш:ext:т:drop;8=ext:ен:ext:т:drop;9=ext:ꙋ:ext:т:drop;11=ext:т:drop;12=ext:н:ext:ен:ext:т:drop"),
    "V14ed": (3, "1=ext:д;2=ext:д;5=ext:ꙋщ:ext:д;6=ext:ом:ext:д;7=ext:ш:ext:д;8=ext:ен:ext:д;9=ext:ꙋ:ext:д;11=ext:д;12=ext:н:ext:ен:ext:д"),
    "V14k": (2, "1=ext:к;2=pal1:ext:к;3=pal2:ext:к;5=ext:ꙋщ:ext:к;6=ext:ом:ext:к;7=ext:ш:ext:к;8=ext:ен:pal1:ext:к;9=ext:ꙋ:ext:к;11=ext:к;12=ext:н:ext:ен:pal1:ext:к"),
    "V14g": (2, "1=ext:г;2=pal1:ext:г;3=pal2:ext:г;5=ext:ꙋщ:ext:г;6=ext:ом:ext:г;7=ext:ш:ext:г;8=ext:ен:pal1:ext:г;9=ext:ꙋ:ext:г;11=ext:г;12=ext:н:ext:ен:pal1:ext:г"),
    "V14g*": (2, "1=ext:г:drop;2=pal1:ext:г:drop;3=pal2:ext:г:drop;5=ext:ꙋщ:ext:г:drop;6=ext:ом:ext:г:drop;7=ext:ш:ext:г:drop;8=ext:ен:pal1:ext:г:drop;9=ext:ꙋ:ext:г:drop;11=ext:г:drop;12=ext:н:ext:ен:pal1:ext:г:drop"),
    "V14eg": (2, "1=ext:г;2=pal1:ext:г;3=pal2:ext:г;5=ext:ꙋщ:ext:г;6=ext:ом:ext:г;7=ext:ш:ext:г;8=ext:ен:pal1:ext:г;9=ext:ꙋ:ext:г;11=ext:г;12=ext:н:ext:ен:pal1:ext:г"),
    "V15er": (2, "1=drop;2=insert:drop;3=base;5=ext:ꙋщ:drop;6=ext:ем:drop;7=ext:ш:insert:drop;8=ext:т:insert:drop;9=ext:ꙋ:drop;11=insert:drop;12=ext:т:insert:drop"),
    "V15ol": (2, "1=base;2=base;5=ext:ющ;6=ext:ем;7=ext:вш;8=ext:ен;9=ext:ю;11=ext:в;12=ext:н:ext:ен"),
    "V15el": (2, "1=base;2=base;5=ext:ющ;6=ext:ем;7=ext:вш;8=ext:ен;9=ext:ю;11=ext:в;12=ext:н:ext:ен"),
    "V15i": (2, "1=base;2=iota;5=ext:ющ:iota;6=ext:ем:iota;7=ext:вш;8=ext:т;9=ext:ю:iota;11=ext:в;12=ext:т;14=ext:ен:iota;15=ext:н:ext:ен:iota"),
    "V15y": (2, "1=base;2=base;5=ext:ющ;6=ext:ем;7=ext:вш;8=ext:т;9=ext:ю;11=ext:в;12=ext:т"),
    "V15e": (2, "1=base;2=base;5=ext:ющ;6=ext:ем;7=ext:вш;8=ext:т;9=ext:ю;11=ext:в;12=ext:т"),
    "V15n": (2, "1=base;2=nasal;5=ext:ꙋщ:nasal;6=ext:ем:nasal;7=ext:вш;8=ext:т;9=ext:ꙋ:nasal;11=ext:в;12=ext:т"),
    "V15a": (2, "1=base;2=ext:н;5=ext:ꙋщ:ext:н;6=ext:ем:ext:н;7=ext:вш;8=ext:т;9=ext:ꙋ:ext:н;11=ext:в;12=ext:т"),
    "V15v": (2, "1=base;2=ext:в;5=ext:ꙋщ:ext:в;6=ext:ем:ext:в;7=ext:вш;8=ext:т;9=ext:ꙋ:ext:в;11=ext:в;12=ext:т"),
}
# the legend's stem numbers refer to its own exemplar; ours: legend 1 -> 1,
# legend 2 -> 2 (the present/iotated stem), legend 3 -> 3 (palatalised
# imperative), 4/5 (V14eg) -> lexical
VERB_ROWS = {
    "из.наст.ед.1": ["pres.1.sg"], "из.наст.ед.2": ["pres.2.sg"], "из.наст.ед.3": ["pres.3.sg"],
    "из.наст.мн.1": ["pres.1.pl"], "из.наст.мн.2": ["pres.2.pl"], "из.наст.мн.3": ["pres.3.pl"],
    "из.наст.дв.1": ["pres.1.du"], "из.наст.дв.2/3": ["pres.2.du", "pres.3.du"],
    "пов.ед.2/3": ["impv.2.sg", "impv.3.sg"], "пов.мн.2": ["impv.2.pl"], "пов.мн.1": ["impv.1.pl"],
    "пов.дв.1": ["impv.1.du"], "пов.дв.2": ["impv.2.du"],
    "из.имперф.ед.1": ["impf.1.sg"], "из.имперф.ед.2/3": ["impf.2.sg", "impf.3.sg"],
    "из.имперф.мн.1": ["impf.1.pl"], "из.имперф.мн.2": ["impf.2.pl"], "из.имперф.мн.3": ["impf.3.pl"],
    "из.имперф.дв.1": ["impf.1.du"], "из.имперф.дв.2/3": ["impf.2.du", "impf.3.du"],
    "из.аор.ед.1": ["aor.1.sg"], "из.аор.ед.2/3": ["aor.2.sg", "aor.3.sg"],
    "из.аор.мн.1": ["aor.1.pl"], "из.аор.мн.2": ["aor.2.pl"], "из.аор.мн.3": ["aor.3.pl"],
    "из.аор.дв.1": ["aor.1.du"], "из.аор.дв.2/3": ["aor.2.du", "aor.3.du"],
    "инф.": ["inf"],
}
LPART_ENDINGS = {"m.sg": "ъ", "f.sg": "а", "n.sg": "о", "m.pl": "и", "f.pl": "ы", "n.pl": "а", "m.du": "а", "f.du": "ѣ", "n.du": "ѣ"}


def verbs(tables_):
    out = []
    seen = set()
    for table in tables_:
        header = table[0][1:]
        rows = table[2:]
        for hi, name in enumerate(header):
            col = hi + 1
            codes = [c.strip() for c in name.split(",")]
            for code in codes:
                if code not in VERB_STEMS or code in seen:
                    continue
                seen.add(code)
                strip, stems = VERB_STEMS[code]
                cells = {}
                for row in rows:
                    label = row[0]
                    text = row[col] if col < len(row) else ""
                    targets = VERB_ROWS.get(label)
                    if targets:
                        alts = alternatives(text)
                        # the legend drops the final jer on a few finite
                        # endings (V12ov -ют): every finite ending in -т
                        # ends in -тъ
                        alts = [a + "ъ" if a.endswith("т") else a for a in alts]
                        if alts:
                            for t in targets:
                                cells[t] = "|".join(alts)
                        continue
                    if label == "прич.перф.ед.м.":
                        # `твор-ил-ъ`: the l-suffix on stem 1
                        m = re.match(r"^\S+?-(\d?)([^-\s]*)-ъ", text.split(",")[0].strip())
                        if m:
                            s = m.group(1) or "1"
                            suf = m.group(2).replace("у", "ꙋ").replace("я", "ѧ")
                            for gn, e in LPART_ENDINGS.items():
                                cells[f"lpart.{gn}"] = f"{s}-{suf}{e}"
                        continue
                    if label == "прич.наст.действ.ед.м./ср.":
                        # the bare participle (stem 9 = the row's stem plus
                        # its ending: дѣла-я -> ext:ѧ) is the short
                        # nominative; the long one adds й; a legend `-ый`
                        # nominative (рв-ый) is that ending on the stem
                        alts = alternatives(text)
                        if alts:
                            shorts, longs = [], []
                            for a in alts:
                                st, e = a.split("-", 1)
                                if e.endswith("ый") or e.endswith("ій"):
                                    shorts.append(a)
                                    longs.append(a)
                                else:
                                    parts = dict(x.split("=", 1) for x in stems.split(";"))
                                    d = parts.get(st, "base")
                                    parts["9"] = f"ext:{e}" if d == "base" else f"ext:{e}:{d}"
                                    stems = ";".join(f"{k}={v}" for k, v in parts.items())
                                    shorts.append("9-")
                                    longs.append("9-й")
                            cells["part.pres.act.short.m.sg.nom"] = "|".join(dict.fromkeys(shorts))
                            cells["part.pres.act.long.m.sg.nom"] = "|".join(dict.fromkeys(longs))
                            if "9-" not in shorts:
                                cells["part.pres.act.short.n.sg.nom"] = "5-е"
                                cells["part.pres.act.short.n.sg.acc"] = "5-е"
                        continue
                    if label == "прич.прош.действ." and "part.past.act.short.m.sg.nom" not in cells:
                        # `твор-ив-ъ`: the short masculine nominative and its long twin
                        cells["part.past.act.short.m.sg.nom"] = "11-ъ"
                        cells["part.past.act.long.m.sg.nom"] = "11-ый"
                        continue
                    if label == "прич.наст.страд.":
                        m = re.match(r"^\S+?-(\d?)([^-\s]*)-ь", text.split(",")[0].strip())
                        if m:
                            cells["part.pres.pass.short.m.sg.nom"] = "6-ь|6-ъ"
                        continue
                    if label == "прич.прош.страд.":
                        cells["part.past.pass.short.m.sg.nom"] = "8-ъ"
                        continue
                # the legend's V14 table stops at the aorist's first plural:
                # the rest follows the class pattern
                if code.startswith("V14"):
                    cells.setdefault("aor.2.pl", "1-осте")
                    cells.setdefault("aor.3.pl", "1-оша")
                    cells.setdefault("aor.1.du", "1-охова|1-оховѣ")
                    cells.setdefault("aor.2.du", "1-оста|1-остѣ")
                    cells.setdefault("aor.3.du", "1-оста|1-остѣ")
                    cells.setdefault("inf", "@lemma")
                    lstem = "4" if code in ("V14t", "V14d", "V14st", "V14t*", "V14ed") else "1"
                    if lstem == "4":
                        stems = stems + ";4=base"
                    for gn, e in LPART_ENDINGS.items():
                        cells.setdefault(f"lpart.{gn}", f"{lstem}-л{e}")
                    cells.setdefault("part.pres.act.short.m.sg.nom", "1-ый")
                    cells.setdefault("part.pres.act.long.m.sg.nom", "1-ый")
                    cells.setdefault("part.past.act.short.m.sg.nom", "11-ъ")
                    cells.setdefault("part.past.act.long.m.sg.nom", "11-ый")
                    cells.setdefault("part.pres.pass.short.m.sg.nom", "6-ь|6-ъ")
                    cells.setdefault("part.past.pass.short.m.sg.nom", "8-ъ")
                # the imperative's first plural: -емъ before -имъ (298/309)
                if "impv.1.pl" in cells:
                    alts = cells["impv.1.pl"].split("|")
                    alts.sort(key=lambda a: 0 if a.endswith("емъ") else 1)
                    cells["impv.1.pl"] = "|".join(alts)
                # the l-participle's dual and feminine/neuter plural: Polyakov
                # tags the -ли form for them too
                for gn in ("f.du", "n.du", "f.pl", "n.pl"):
                    if gn in [k.split("lpart.")[1] for k in cells if k.startswith("lpart.")]:
                        v = cells[f"lpart.{gn}"]
                        st = v.split("-", 1)[0]
                        suf = v.split("-", 1)[1][:-1]
                        # the dual's -ли is what Polyakov attests (55/55);
                        # the plural keeps -ы/-а first
                        cells[f"lpart.{gn}"] = f"{st}-{suf}и|{v}" if gn.endswith("du") else f"{v}|{st}-{suf}и"
                # participle blocks on the stems 5..8, declined as adjectives;
                # the short masculine accusative names the participle's own
                # nominative (the delegate's would be the adjective's), the
                # short plural nominative/accusative take -ще/-щѧ, the long
                # neuter nominative may be the bare -ѧй (Polyakov m/n)
                cells["part.pres.act.short"] = "5~A1s"
                cells["part.pres.act.long"] = "5~A1s"
                cells["part.pres.act.short.f.sg.nom"] = "5-и"
                # the neuter short nominative: the bare stem (m/n, the
                # legend) and -ще (Polyakov: взе́млюще brev,sg,n)
                cells.setdefault("part.pres.act.short.n.sg.nom", "9-|5-е")
                cells.setdefault("part.pres.act.short.n.sg.acc", "9-|5-е")
                cells["part.pres.pass.short"] = "6~A1t"
                cells["part.pres.pass.long"] = "6~A1t"
                cells["part.past.act.short"] = "7~A1s"
                cells["part.past.act.long"] = "7~A1s"
                cells["part.past.act.short.f.sg.nom"] = "7-и"
                cells["part.past.act.short.n.sg.nom"] = "7-е"
                cells["part.past.pass.short"] = "8~A1t"
                cells["part.past.pass.long"] = "12~A1t"
                velar = code.startswith(("V14k", "V14g", "V14x", "V14eg"))
                if velar:
                    # the velar's long ending: -кі́й, -гі́й (влекі́й, могі́й)
                    for k in ("part.pres.act.short.m.sg.nom", "part.pres.act.long.m.sg.nom"):
                        if k in cells:
                            cells[k] = cells[k].replace("ый", "їй")
                # a participle's subject is animate: the accusative is the
                # genitive first, the nominative second (census 330/374,
                # 1412 overrides)
                pres_nom = cells.get("part.pres.act.short.m.sg.nom", "9-")
                for series, st in (("pres.act", "5"), ("past.act", "7")):
                    cells[f"part.{series}.short.m.sg.acc"] = f"{st}-а|@part.{series}.short.m.sg.nom"
                    cells[f"part.{series}.long.m.sg.acc"] = f"{st}-аго|@part.{series}.long.m.sg.nom"
                    # the short plural nominative -ще/-вше before the
                    # adjective's -щи (census 1084/874 overrides)
                    cells[f"part.{series}.short.m.pl.nom"] = f"{st}-е|{st}-и"
                cells["part.pres.pass.long.m.sg.acc"] = "6-аго|@part.pres.pass.long.m.sg.nom"
                cells["part.past.pass.long.m.sg.acc"] = "12-аго|@part.past.pass.long.m.sg.nom"
                # the present active short masculine: the bare form, then the
                # -ющъ/-ѧщь Polyakov also tags as the nominative
                cells["part.pres.act.short.m.sg.nom"] = "|".join(dict.fromkeys(pres_nom.split("|") + ["5-ъ", "5-ь"]))
                # the neuter short accusative is -ще first (608/612)
                if cells.get("part.pres.act.short.n.sg.acc", "").startswith("9-|"):
                    cells["part.pres.act.short.n.sg.acc"] = "5-е|9-"
                if code.startswith("V14"):
                    # the legend's -ый/-їй present participle is tagged m/n
                    cells["part.pres.act.short.n.sg.nom"] = pres_nom.split("|")[0] + "|9-|5-е"
                # the long neuter nominative: Polyakov's -ѧй is m/n (918 overrides)
                cells["part.pres.act.long.n.sg.nom"] = cells.get("part.pres.act.long.m.sg.nom", "9-й") + "|5-ее"
                # the past active long nominative: -ивый and -ившїй both attested
                cells["part.past.act.long.m.sg.nom"] = cells.get("part.past.act.long.m.sg.nom", "11-ый") + "|7-їй"
                # the V21 old short participle in -ь on the iotated stem
                # (возстꙋ́пль, и҆спо́лнь, вѣ́рь)
                if code.startswith("V21"):
                    cells["part.past.act.short.m.sg.nom"] = "11-ъ|2-ь"
                # the present passive short masculine: -ъ before -ь (census)
                if cells.get("part.pres.pass.short.m.sg.nom") == "6-ь|6-ъ":
                    cells["part.pres.pass.short.m.sg.nom"] = "6-ъ|6-ь"
                # the past passive short plural obliques and the singular
                # instrumental keep the long stem's double н (-нныхъ, -ннымъ)
                for g in ("m", "f", "n"):
                    cells[f"part.past.pass.short.{g}.pl.gen"] = "12-ыхъ^"
                    cells[f"part.past.pass.short.{g}.pl.loc"] = "12-ыхъ^"
                    cells[f"part.past.pass.short.{g}.pl.dat"] = "12-ымъ^"
                for g in ("m", "n"):
                    cells[f"part.past.pass.short.{g}.sg.ins"] = "12-ымъ"
                cells["part.past.pass.short.f.sg.nom"] = "8-а|12-а"
                # the archaic past active participle on the soft stem
                # (и҆зба́вльшїй, вмѣ́щшїй, вложшїй; воздви́гшїй, воскре́сшїй
                # without -нꙋ-): stem 14, beside the -вш- forms (V2.1 Part
                # 1.4: 76 + 25 stored stems named by the class)
                parts = dict(x.split("=", 1) for x in stems.split(";"))
                if "14" in parts and code.startswith(("V21", "V13")):
                    for series in ("short", "long"):
                        cells[f"part.past.act.{series}"] = "7~A1s|14~A1s"
                    cells["part.past.act.short.f.sg.nom"] = "7-и|14-и"
                    cells["part.past.act.short.n.sg.nom"] = "7-е|14-е"
                    cells["part.past.act.short.m.sg.acc"] = "7-а|14-а|@part.past.act.short.m.sg.nom"
                    cells["part.past.act.long.m.sg.acc"] = "7-аго|14-аго|@part.past.act.long.m.sg.nom"
                    cells["part.past.act.short.m.pl.nom"] = "7-е|7-и|14-е|14-и"
                    cells["part.past.act.long.m.sg.nom"] = "11-ый|7-їй|14-їй"
                    if code.startswith("V13"):
                        cells["part.past.act.short.m.sg.nom"] = "11-ъ|1-ъ"
                # бити's past passive participle: бїе́нъ, бїе́нный beside би́тъ
                if code == "V15i":
                    cells["part.past.pass.short"] = "8~A1t|14~A1t"
                    cells["part.past.pass.long"] = "12~A1t|15~A1t"
                    cells["part.past.pass.short.m.sg.nom"] = "8-ъ|14-ъ"
                    cells["part.past.pass.short.f.sg.nom"] = "8-а|12-а|14-а"
                    cells["part.past.pass.long.m.sg.acc"] = "12-аго|15-аго|@part.past.pass.long.m.sg.nom"
                    parts["9"] = "ext:ѧ:iota"
                    stems = ";".join(f"{k}={v}" for k, v in parts.items())
                    # an ending that opens with a vowel or й is on the
                    # ї-stem (бїѧ́хъ, бі́й), like the present; the
                    # consonant-initial ones stay on и (би́хъ, би́лъ, би́ти)
                    for k in list(cells):
                        alts = []
                        for a in cells[k].split("|"):
                            if a.startswith("1-") and a[2:3] and a[2:3] in "аеиоѧюѣыꙋй":
                                a = "2-" + a[2:]
                            alts.append(a)
                        cells[k] = "|".join(alts)
                # the long locative: -омъ before -ѣмъ on participles (census)
                for g in ("m", "n"):
                    cells[f"part.past.pass.long.{g}.sg.loc"] = "12-омъ|12-ѣмъ"
                # V22t's aorist 2/3 sg is the bare -ѣ, not the first person
                if code.startswith("V22") and cells.get("aor.2.sg", "").endswith("хъ"):
                    cells["aor.2.sg"] = cells["aor.2.sg"][:-2]
                    cells["aor.3.sg"] = cells["aor.2.sg"]
                # V13k: the root aorist (дости́же, воздви́же) on the
                # palatalised stem, and its plural first (census 34/46, 8/8)
                if code == "V13k":
                    parts = dict(x.split("=", 1) for x in stems.split(";"))
                    parts["13"] = "pal1"
                    stems = ";".join(f"{k}={v}" for k, v in parts.items())
                    cells["aor.2.sg"] = "1-нꙋ|13-е"
                    cells["aor.3.sg"] = "1-нꙋ|13-е"
                if code in ("V13k", "V13t"):
                    for k in ("aor.1.sg", "aor.1.pl", "aor.2.pl", "aor.3.pl", "aor.1.du", "aor.2.du", "aor.3.du"):
                        if k in cells:
                            alts = cells[k].split("|")
                            alts.sort(key=lambda a: 0 if "-о" in a else 1)
                            cells[k] = "|".join(alts)
                # the vocatives and the third-person imperative
                exemplar = table[1][1:][hi].split(",")[0].strip().replace("-", "")
                out.append((code, exemplar, strip, stems, cells))
    out.extend(athematic())
    header_cells = []
    for t in ("pres", "fut", "impf", "aor"):
        for p in ("1", "2", "3"):
            for n in NUMBERS:
                header_cells.append(f"{t}.{p}.{n}")
    for p, n in (("2", "sg"), ("3", "sg"), ("1", "pl"), ("2", "pl"), ("1", "du"), ("2", "du")):
        header_cells.append(f"impv.{p}.{n}")
    header_cells.append("inf")
    header_cells += [f"lpart.{gn}" for gn in ("m.sg", "f.sg", "n.sg", "m.du", "f.du", "n.du", "m.pl", "f.pl", "n.pl")]
    for t in ("pres", "past"):
        for v in ("act", "pass"):
            for s in ("short", "long"):
                header_cells.append(f"part.{t}.{v}.{s}")
                header_cells.append(f"part.{t}.{v}.{s}.m.sg.nom")
    header_cells += ["part.pres.act.short.n.sg.nom", "part.pres.act.short.n.sg.acc", "part.pres.act.short.f.sg.nom", "part.past.act.short.f.sg.nom", "part.past.act.short.n.sg.nom", "part.past.act.short.n.sg.acc"]
    header_cells += [f"part.{t}.act.short.{g}.{n}.{c}" for t in ("pres", "past") for (g, n, c) in (("m", "sg", "acc"), ("m", "pl", "nom"), ("m", "pl", "acc"), ("f", "pl", "nom"), ("f", "pl", "acc"), ("m", "sg", "loc"), ("n", "sg", "loc"))]
    header_cells += ["part.pres.act.long.n.sg.nom", "part.pres.pass.short.m.sg.acc", "part.past.pass.short.m.sg.acc"]
    header_cells += ["part.pres.act.long.m.sg.acc", "part.past.act.long.m.sg.acc", "part.pres.pass.long.m.sg.acc", "part.past.pass.long.m.sg.acc"]
    header_cells += [f"part.past.pass.short.{g}.pl.{c}" for g in ("m", "f", "n") for c in ("gen", "dat", "loc")]
    header_cells += ["part.past.pass.short.m.sg.ins", "part.past.pass.short.n.sg.ins", "part.past.pass.short.f.sg.nom", "part.past.pass.long.m.sg.loc", "part.past.pass.long.n.sg.loc"]
    return header_cells, out


# the finite series in the order sg 1 2 3, du 1 2 3, pl 1 2 3
def finite(tense, endings):
    cells = {}
    i = 0
    for n in ("sg", "du", "pl"):
        for p in ("1", "2", "3"):
            cells[f"{tense}.{p}.{n}"] = endings[i]
            i += 1
    return cells


def athematic():
    """The athematic verbs (Polyakov Vbyt/Vdat/Vved/Vest/Vima): every cell
    a literal on the base, the participles on their own stems; the
    prefixed perfectives of бы́ти (добы́ти) take the twin Vbyt* whose
    present is the бꙋ́дꙋ series."""
    common_part = {
        "part.pres.act.short": "5~A1s", "part.pres.act.long": "5~A1s",
        "part.pres.act.short.f.sg.nom": "5-и", "part.pres.act.short.n.sg.nom": "9-|5-е", "part.pres.act.short.n.sg.acc": "5-е|9-",
        "part.pres.act.short.m.pl.nom": "5-е|5-и", "part.pres.act.short.m.sg.acc": "5-а|@part.pres.act.short.m.sg.nom",
        "part.pres.act.long.m.sg.acc": "5-аго|@part.pres.act.long.m.sg.nom", "part.pres.act.long.n.sg.nom": "@part.pres.act.long.m.sg.nom|5-ее",
        "part.pres.pass.short": "6~A1t", "part.pres.pass.long": "6~A1t", "part.pres.pass.short.m.sg.nom": "6-ъ|6-ь",
        "part.pres.pass.long.m.sg.acc": "6-аго|@part.pres.pass.long.m.sg.nom",
        "part.past.act.short": "7~A1s", "part.past.act.long": "7~A1s", "part.past.act.short.m.sg.nom": "11-ъ", "part.past.act.long.m.sg.nom": "11-ый|7-їй",
        "part.past.act.short.f.sg.nom": "7-и", "part.past.act.short.n.sg.nom": "7-е", "part.past.act.short.m.pl.nom": "7-е|7-и",
        "part.past.act.short.m.sg.acc": "7-а|@part.past.act.short.m.sg.nom", "part.past.act.long.m.sg.acc": "7-аго|@part.past.act.long.m.sg.nom",
        "part.past.pass.short": "8~A1t", "part.past.pass.long": "12~A1t", "part.past.pass.short.m.sg.nom": "8-ъ",
        "part.past.pass.long.m.sg.acc": "12-аго|@part.past.pass.long.m.sg.nom", "part.past.pass.short.f.sg.nom": "8-а|12-а",
    }
    for g in ("m", "f", "n"):
        common_part[f"part.past.pass.short.{g}.pl.gen"] = "12-ыхъ^"
        common_part[f"part.past.pass.short.{g}.pl.loc"] = "12-ыхъ^"
        common_part[f"part.past.pass.short.{g}.pl.dat"] = "12-ымъ^"
    for g in ("m", "n"):
        common_part[f"part.past.pass.short.{g}.sg.ins"] = "12-ымъ"
        common_part[f"part.past.pass.long.{g}.sg.loc"] = "12-омъ|12-ѣмъ"

    def lpart(stem, suffix):
        return {f"lpart.{gn}": f"{stem}-{suffix}л{e}" for gn, e in LPART_ENDINGS.items()} | {
            "lpart.f.du": f"{stem}-{suffix}ли|{stem}-{suffix}лѣ", "lpart.n.du": f"{stem}-{suffix}ли|{stem}-{suffix}лѣ"}

    def impv(two, three, one_pl, two_pl, one_du, two_du):
        return {"impv.2.sg": two, "impv.3.sg": three, "impv.1.pl": one_pl, "impv.2.pl": two_pl, "impv.1.du": one_du, "impv.2.du": two_du}

    rows = []
    # бы́ти: strip 4 leaves the empty base; the copula's present is
    # suppletive (є҆́смь … сꙋ́ть), the future бꙋ́дꙋ, the imperfect бѧ́хъ,
    # the aorist бы́хъ beside бѣ́хъ (бы́сть/бѣ̀)
    byti = {}
    byti |= finite("pres", ["1-есмь", "1-еси", "1-есть", "1-есма", "1-еста", "1-еста", "1-есмы", "1-есте", "1-сꙋть"])
    byti |= finite("fut", ["1-бꙋдꙋ", "1-бꙋдеши", "1-бꙋдетъ", "1-бꙋдева", "1-бꙋдета", "1-бꙋдета", "1-бꙋдемъ", "1-бꙋдете", "1-бꙋдꙋтъ"])
    byti |= finite("impf", ["1-бѧхъ", "1-бѧше", "1-бѧше", "1-бѧхова", "1-бѧста", "1-бѧста", "1-бѧхомъ", "1-бѧсте", "1-бѧхꙋ"])
    byti |= finite("aor", ["1-быхъ|1-бѣхъ", "1-бысть|1-бѣ", "1-бысть|1-бѣ", "1-быхова|1-бѣхова", "1-быста|1-бѣста", "1-быста|1-бѣста", "1-быхомъ|1-бѣхомъ", "1-бысте|1-бѣсте", "1-быша|1-бѣша"])
    byti |= impv("1-бꙋди", "1-бꙋди", "1-бꙋдимъ", "1-бꙋдите", "1-бꙋдива", "1-бꙋдита")
    byti["inf"] = "@lemma"
    byti |= lpart("1", "бы")
    byti |= common_part
    byti["part.pres.act.short.m.sg.nom"] = "1-сый|9-"
    byti["part.pres.act.long.m.sg.nom"] = "1-сый|5-ій"
    byti["part.pres.act.short.n.sg.nom"] = "1-сый|9-|5-е"
    byti["part.past.act.short.m.sg.nom"] = "11-ъ"
    stems_byti = "1=base;5=ext:сꙋщ;6=ext:бꙋдом;7=ext:бывш;8=ext:бывен;9=ext:сы;11=ext:быв;12=ext:н:ext:бывен"
    rows.append(("Vbyt", "быти", 4, stems_byti, byti))
    prefixed = dict(byti)
    prefixed |= finite("pres", ["1-бꙋдꙋ", "1-бꙋдеши", "1-бꙋдетъ", "1-бꙋдева", "1-бꙋдета", "1-бꙋдета", "1-бꙋдемъ", "1-бꙋдете", "1-бꙋдꙋтъ"])
    for k in [k for k in prefixed if k.startswith("fut.")]:
        del prefixed[k]
    prefixed["part.pres.act.short.m.sg.nom"] = "9-"
    prefixed["part.pres.act.long.m.sg.nom"] = "9-й"
    prefixed["part.pres.act.short.n.sg.nom"] = "9-|5-е"
    rows.append(("Vbyt*", "добыти", 4, "1=base;5=ext:бꙋдꙋщ;6=ext:бꙋдом;7=ext:бывш;8=ext:бывен;9=ext:бꙋдꙋ;11=ext:быв;12=ext:н:ext:бывен", prefixed))
    # да́ти (base да): the athematic present да́мъ … дадѧ́тъ, the aorist
    # да́хъ/дадѐ, the imperative да́ждь
    dati = {}
    dati |= finite("pres", ["1-мъ", "1-си", "1-стъ|1-сть", "1-ва", "1-ста", "1-ста", "1-мы|1-мъ", "1-сте", "1-дѧтъ|1-дꙋтъ"])
    dati |= finite("impf", ["1-дѧхъ", "1-дѧше", "1-дѧше", "1-дѧхова", "1-дѧста", "1-дѧста", "1-дѧхомъ", "1-дѧсте", "1-дѧхꙋ"])
    dati |= finite("aor", ["1-хъ", "1-де", "1-де", "1-хова", "1-ста", "1-ста", "1-хомъ", "1-сте", "1-ша|1-доша"])
    dati |= impv("1-ждь|1-й", "1-ждь|1-й", "1-димъ", "1-дите|1-йте", "1-дива", "1-дита")
    dati["inf"] = "1-ти"
    dati |= lpart("1", "")
    dati |= common_part
    dati["part.pres.act.short.m.sg.nom"] = "9-"
    dati["part.pres.act.long.m.sg.nom"] = "9-й"
    rows.append(("Vdat", "дати", 2, "1=base;5=ext:дꙋщ;6=ext:дом;7=ext:вш;8=ext:н;9=ext:ды;11=ext:в;12=ext:н:ext:н", dati))
    # ꙗ҆́сти (base ꙗ): ꙗ҆́мъ … ꙗ҆дѧ́тъ, ꙗ҆́хъ/ꙗ҆дѐ, ꙗ҆́ждь
    esti = {}
    esti |= finite("pres", ["1-мъ", "1-си", "1-стъ|1-сть", "1-ва", "1-ста", "1-ста", "1-мы", "1-сте", "1-дѧтъ"])
    esti |= finite("impf", ["1-дѧхъ", "1-дѧше", "1-дѧше", "1-дѧхова", "1-дѧста", "1-дѧста", "1-дѧхомъ", "1-дѧсте", "1-дѧхꙋ"])
    esti |= finite("aor", ["1-дохъ|1-хъ", "1-де", "1-де", "1-дохова", "1-доста|1-ста", "1-доста|1-ста", "1-дохомъ|1-хомъ", "1-досте|1-сте", "1-доша|1-ша"])
    esti |= impv("1-ждь", "1-ждь", "1-димъ", "1-дите", "1-дива", "1-дита")
    esti["inf"] = "1-сти"
    esti |= lpart("1", "")
    esti |= common_part
    esti["part.pres.act.short.m.sg.nom"] = "9-"
    esti["part.pres.act.long.m.sg.nom"] = "9-й"
    rows.append(("Vest", "ꙗсти", 3, "1=base;5=ext:дꙋщ;6=ext:дом;7=ext:дш;8=ext:ден;9=ext:ды;11=ext:дъ;12=ext:н:ext:ден", esti))
    # вѣ́дѣти (base вѣд): вѣ́мъ … вѣ́дѧтъ, вѣ́дѣхъ, вѣ́ждь
    vedeti = {}
    vedeti |= finite("pres", ["1-ѣмъ", "1-ѣси", "1-ѣсть", "1-ѣва", "1-ѣста", "1-ѣста", "1-ѣмы", "1-ѣсте", "1-ѧтъ"])
    vedeti |= finite("impf", ["1-ѧхъ", "1-ѧше", "1-ѧше", "1-ѧхова", "1-ѧста", "1-ѧста", "1-ѧхомъ", "1-ѧсте", "1-ѧхꙋ"])
    vedeti |= finite("aor", ["1-ѣхъ", "1-ѣ", "1-ѣ", "1-ѣхова", "1-ѣста", "1-ѣста", "1-ѣхомъ", "1-ѣсте", "1-ѣша"])
    vedeti |= impv("1-ждь", "1-ждь", "1-димъ", "1-дите|1-ждьте", "1-дива", "1-дита")
    vedeti["inf"] = "1-ѣти"
    vedeti |= lpart("1", "ѣ")
    vedeti |= common_part
    vedeti["part.pres.act.short.m.sg.nom"] = "9-й|9-"
    vedeti["part.pres.act.long.m.sg.nom"] = "9-й"
    rows.append(("Vved", "вѣдѣти", 3, "1=base;5=ext:ꙋщ;6=ext:ом;7=ext:ѣвш;8=ext:ѣн;9=ext:ы;11=ext:ѣв;12=ext:н:ext:ѣн", vedeti))
    # и҆мѣ́ти (base им): и҆́мамъ … и҆́мꙋтъ, и҆мѣ́ѧхъ, и҆мѣ́хъ, и҆мѣ́й
    imeti = {}
    imeti |= finite("pres", ["1-амъ", "1-аши", "1-ать", "1-ава", "1-ата", "1-ата", "1-амы", "1-ате", "1-ꙋтъ"])
    imeti |= finite("impf", ["1-ѣѧхъ", "1-ѣѧше", "1-ѣѧше", "1-ѣѧхова", "1-ѣѧста", "1-ѣѧста", "1-ѣѧхомъ", "1-ѣѧсте", "1-ѣѧхꙋ"])
    imeti |= finite("aor", ["1-ѣхъ", "1-ѣ", "1-ѣ", "1-ѣхова", "1-ѣста", "1-ѣста", "1-ѣхомъ", "1-ѣсте", "1-ѣша"])
    imeti |= impv("1-ѣй", "1-ѣй", "1-ѣимъ", "1-ѣйте", "1-ѣива", "1-ѣита")
    imeti["inf"] = "1-ѣти"
    imeti |= lpart("1", "ѣ")
    imeti |= common_part
    imeti["part.pres.act.short.m.sg.nom"] = "9-"
    imeti["part.pres.act.long.m.sg.nom"] = "9-й"
    rows.append(("Vima", "имѣти", 4, "1=base;5=ext:ꙋщ;6=ext:ѣем;7=ext:ѣвш;8=ext:ѣн;9=ext:ѣѧ;11=ext:ѣв;12=ext:н:ext:ѣн", imeti))
    # и҆тѝ and its compounds (Polyakov Viti, 41 lemmas): the present on the
    # base with -д- (и҆дꙋ̀), the aorist и҆до́хъ/и҆́де, the suppletive
    # l-participle ше́лъ on the base without its final и (stem 4)
    iti = {}
    iti |= finite("pres", ["1-дꙋ", "1-деши", "1-детъ", "1-дева", "1-дета", "1-дета", "1-демъ", "1-дете", "1-дꙋтъ"])
    iti |= finite("impf", ["1-дѧхъ", "1-дѧше", "1-дѧше", "1-дѧхова", "1-дѧста", "1-дѧста", "1-дѧхомъ", "1-дѧсте", "1-дѧхꙋ"])
    iti |= finite("aor", ["1-дохъ", "1-де", "1-де", "1-дохова", "1-доста", "1-доста", "1-дохомъ", "1-досте", "1-доша"])
    iti |= impv("1-ди", "1-ди", "1-демъ|1-димъ", "1-дите", "1-дива", "1-дита")
    iti["inf"] = "@lemma"
    iti |= lpart("4", "ше")
    iti["lpart.f.sg"] = "4-шла"
    iti["lpart.n.sg"] = "4-шло"
    iti["lpart.m.du"] = "4-шла"
    iti["lpart.f.du"] = "4-шли|4-шлѣ"
    iti["lpart.n.du"] = "4-шли|4-шлѣ"
    iti["lpart.m.pl"] = "4-шли"
    iti["lpart.f.pl"] = "4-шлы|4-шли"
    iti["lpart.n.pl"] = "4-шла"
    iti |= common_part
    iti["part.pres.act.short.m.sg.nom"] = "9-й|9-"
    iti["part.pres.act.long.m.sg.nom"] = "9-й"
    iti["part.past.act.short.m.sg.nom"] = "11-ъ"
    iti["part.past.act.long.m.sg.nom"] = "11-ый|7-їй"
    rows.append(("Viti", "ити", 2, "1=base;4=cut;5=ext:дꙋщ;6=ext:дом;7=ext:шедш:cut;8=ext:ден;9=ext:ды;11=ext:шед:cut;12=ext:н:ext:ден", iti))
    return rows


def main():
    t = tables()
    header, rows = adjectives(t[4])
    write(CLASSES / "adj.tsv", header, rows, "Adjective letter classes, seeded from Polyakov's legend (flexslav.htm, table 4)\nby scripts/legend-adj-verb-pron.py; hand-maintained since. Cells: <series>.<degree>.<g>.<n>.<case>;\nthe block columns short.comp/long.comp decline stem 4 (base + ѣйш) as A1s.")
    ph, prows = pronominal(t[5])
    pph, pprows = personal(t[6])
    a2t = next(r for r in rows if r[0] == "A2t")
    nominal = {}
    for k, v in a2t[4].items():
        if k.startswith("short.pos."):
            nominal[k[len("short.pos."):]] = v.replace("@short.pos.", "@").replace("@long.pos.", "@")
    # the long-series references have no twin here: the pronominal ending
    for k, v in list(nominal.items()):
        if v.startswith("@") and (v[1:] not in nominal or v[1:] == k):
            nominal[k] = {"m.sg.ins": "1-ымъ", "n.sg.ins": "1-ымъ", "m.pl.gen": "1-ыхъ", "f.pl.gen": "1-ыхъ", "n.pl.gen": "1-ыхъ",
                          "m.pl.loc": "1-ыхъ", "f.pl.loc": "1-ыхъ", "n.pl.loc": "1-ыхъ", "m.pl.dat": "1-ымъ^", "f.pl.dat": "1-ымъ^", "n.pl.dat": "1-ымъ^"}.get(k, v)
    prows.append(("PN", "таковъ", 1, "1=base;2=base", nominal))
    # the velar nominal (всѧ́къ, толи́къ, є҆ли́къ): the second palatalisation
    # before ѣ and ы (всѧ́цѣмъ, толи́цы)
    velar = dict(nominal)
    for k, v in nominal.items():
        if v.startswith("1-ѣ") or v.startswith("1-ы"):
            velar[k] = "5-" + v[2:]
    prows.append(("PNk", "всякъ", 1, "1=base;2=base;5=pal2", velar))
    all_header = ph + [c for c in pph if c not in ph] + [c for c in nominal if c not in ph and c not in pph]
    write(CLASSES / "pronoun.tsv", all_header, prows + pprows, "Pronoun letter classes: the pronominal adjectives (legend table 5, short columns)\nand the personal pronouns (table 6; their stems are on the lexeme line: stems=1=мен;2=мн).")
    vh, vrows = verbs(t[7:12])
    write(CLASSES / "verb.tsv", vh, vrows, "Verb letter classes, seeded from Polyakov's legend (tables 7–11) by scripts/legend-adj-verb-pron.py;\nhand-maintained since. Stems: 1 infinitive, 2 present, 3 palatalised imperative, 5–8 the participle stems\n(present active, present passive, past active, past passive), declined as the named adjective class;\n9 the bare present participle (творѧ), 11 the past active short stem (творив), 12 the long past passive (творенн).")


if __name__ == "__main__":
    main()
