#!/usr/bin/env python3
"""Seed the Old Church Slavonic class tables from Kaikki's own paradigm
tables (data/intermediate/kaikki.jsonl → lexicon/classes/ocs/*.tsv) and
write every entry's cells as data/intermediate/kaikki-cells.jsonl for the
Rust importer (`cargo xtask import kaikki --pos <pos>`).

A class is a group of entries with one paradigm shape: Kaikki's stem-class
tag (o-stem, a-stem, i-stem, …; IA1, II1, … for verbs) joined with the
nominative's ending and, for nouns, the gender. Within a group every
entry's stem is the longest common prefix of its forms; the class row is
the majority ending per cell, with a second ending listed as an
alternative when a quarter of the group uses it. Cell names are the
crate's (`gen.pl`, `short.pos.m.sg.nom`, `pres.1.sg`, `lpart.m.sg`,
`part.pres.act.short.m.sg.nom`, `m.sg.gen`)."""
import json
import re
import unicodedata
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "data/intermediate/kaikki.jsonl"
CLASSES = ROOT / "crates/church-slavonic/lexicon/classes/ocs"
CELLS = ROOT / "data/intermediate/kaikki-cells.jsonl"

CASES = {"nominative": "nom", "genitive": "gen", "dative": "dat", "accusative": "acc", "instrumental": "ins", "locative": "loc", "vocative": "voc"}
NUMBERS = {"singular": "sg", "dual": "du", "plural": "pl"}
GENDERS = {"masculine": "m", "feminine": "f", "neuter": "n"}
SKIP = {"table-tags", "class", "inflection-template", "error-unrecognized-form", "canonical", "romanization", "Glagolitic", "Cyrillic", "alternative"}


def letters(form):
    """The letters of a printed form: combining marks (the palatalisation
    hook of цар҄ь) stripped, the digraph оу and ꙑ kept as the print has them
    — the crate's OCS print maps ы→ꙑ and ꙋ→оу, so the letters layer stores
    ы/ꙋ; ѫ, ѧ, ѥ, ꙗ, ѣ, ь are letters of their own."""
    s = unicodedata.normalize("NFD", form.strip().lower())
    s = "".join(c for c in s if not unicodedata.combining(c))
    s = unicodedata.normalize("NFC", s)
    s = s.replace("ꙑ", "ы").replace("оу", "ꙋ").replace("ѹ", "ꙋ").replace("ꙁ", "з").replace("ї", "и").replace("і", "и").replace("й", "и").replace("є", "е").replace("ѡ", "о")
    return s


def alternatives(form):
    if form.strip() in ("-", "—", ""):
        return []
    if any(c in form for c in " /"):
        # «чьсо, чесого» is a list; «ни/о/при чесомьже» a phrase: keep the list
        if "/" in form:
            return []
        return [letters(p) for p in form.split(",") if p.strip() and " " not in p.strip()]
    return [letters(form)]


def noun_cells(entry):
    out = defaultdict(list)
    for f in entry["forms"]:
        if f.get("source") != "declension":
            continue
        tags = [t for t in f.get("tags", []) if t not in SKIP]
        case = next((CASES[t] for t in tags if t in CASES), None)
        number = next((NUMBERS[t] for t in tags if t in NUMBERS), None)
        if not case or not number or any(t in GENDERS for t in tags):
            continue
        for alt in alternatives(f["form"]):
            if alt and alt not in out[f"{case}.{number}"]:
                out[f"{case}.{number}"].append(alt)
    return out


def adj_cells(entry, series_required=True):
    out = defaultdict(list)
    # Kaikki prints an adjective's short series as its first declension
    # table and the long series as the second, without series tags
    table_index = -1
    for f in entry["forms"]:
        if f.get("source") != "declension":
            continue
        if "table-tags" in f.get("tags", []):
            table_index += 1
            continue
        tags = [t for t in f.get("tags", []) if t not in SKIP]
        case = next((CASES[t] for t in tags if t in CASES), None)
        number = next((NUMBERS[t] for t in tags if t in NUMBERS), None)
        genders = [GENDERS[t] for t in tags if t in GENDERS]
        series = "short" if "short-form" in tags else "long" if "long-form" in tags else None
        if series is None and series_required:
            series = "short" if table_index <= 0 else "long"
        degree = "comp" if "comparative" in tags else "pos"
        if not case or not number or not genders:
            continue
        for g in genders:
            key = f"{series}.{degree}.{g}.{number}.{case}" if series else f"{g}.{number}.{case}"
            for alt in alternatives(f["form"]):
                if alt and alt not in out[key]:
                    out[key].append(alt)
    return out


def verb_cells(entry):
    out = defaultdict(list)
    forms = [f for f in entry["forms"] if f.get("source") == "conjugation"]
    # the finite rows come in order 1, 2, 3 per (tense, number); the
    # singular of the aorist, imperfect and imperative prints two rows (the
    # second and third persons share one)
    seq = defaultdict(list)
    banner = ""
    for f in forms:
        tags = f.get("tags", [])
        if "table-tags" in tags:
            banner = f["form"]
            continue
        if "inflection-template" in tags or "class" in tags:
            continue
        t = [x for x in tags if x not in SKIP]
        tense = next((x for x in t if x in ("present", "imperfect", "aorist", "imperative")), None)
        number = next((NUMBERS[x] for x in t if x in NUMBERS), None)
        if tense and number and not any(x in GENDERS for x in t) and "l-participle" not in t:
            seq[(tense, number)].append(f["form"])
            continue
        if "infinitive" in t:
            out["inf"].extend(a for a in alternatives(f["form"]) if a)
        elif "l-participle" in t:
            g = next((GENDERS[x] for x in t if x in GENDERS), None)
            if g and number:
                out[f"lpart.{g}.{number}"].extend(a for a in alternatives(f["form"]) if a)
        elif banner in ("present", "past") and any(x in GENDERS for x in t):
            # the participle tables under a tense banner (present active,
            # past active)
            case = next((CASES[x] for x in t if x in CASES), None)
            series = "short" if "short-form" in t else "long" if "long-form" in t else None
            if case and number and series:
                tense_name = "pres" if banner == "present" else "past"
                for g in [GENDERS[x] for x in t if x in GENDERS]:
                    out[f"part.{tense_name}.act.{series}.{g}.{number}.{case}"].extend(a for a in alternatives(f["form"]) if a)
    for (tense, number), rows in seq.items():
        tname = {"present": "pres", "imperfect": "impf", "aorist": "aor", "imperative": "impv"}[tense]
        persons = ["1", "2", "3"] if (number != "sg" or tense == "present" or len(rows) == 3) else ["1", "23"]
        for p, form in zip(persons, rows):
            alts = [a for a in alternatives(form) if a and a != "-"]
            targets = ["2", "3"] if p == "23" else [p]
            for person in targets:
                key = f"{tname}.{person}.{number}"
                for a in alts:
                    if a not in out[key]:
                        out[key].append(a)
    return {k: list(dict.fromkeys(v)) for k, v in out.items() if v}


def class_tag(entry):
    for f in entry["forms"]:
        if "class" in f.get("tags", []):
            return f["form"]
    return "-"


def gender_tag(entry):
    for f in entry["forms"]:
        if "canonical" in f.get("tags", []):
            for t in f["tags"]:
                if t in GENDERS:
                    return GENDERS[t]
    return "-"


PAL2 = {"к": "ц", "г": "ѕ", "х": "с"}
PAL1 = {"к": "ч", "г": "ж", "х": "ш"}


def strip_of(pos, lemma):
    """The letters of the citation form that are its ending: one for a
    vowel or jer, none for a consonant (мати takes 1, крꙑ 1); verbs strip
    the infinitive's -ти."""
    if pos == "v":
        return 2 if lemma.endswith("ти") else 2 if lemma.endswith("щи") else 0
    if lemma[-1:] in "аꙗиыъьоеѥѧѫюѣꙋ":
        return 1
    return 0


def stems_of(stem):
    """Stem 1 the base; 5 its second palatalisation; 3 the first."""
    out = {"1": stem}
    if stem and stem[-1] in PAL2:
        out["5"] = stem[:-1] + PAL2[stem[-1]]
        out["3"] = stem[:-1] + PAL1[stem[-1]]
    return out


def spec_of(form, stems, prefer=("1", "5", "3")):
    """`<stem>-<ending>` for a form, the base stem preferred; None when no
    stem is a prefix (suppletion: the fit will override it)."""
    for n in prefer:
        st = stems.get(n)
        if st is not None and form.startswith(st):
            return f"{n}-{form[len(st):]}"
    return None


def lcp(strings):
    if not strings:
        return ""
    s1, s2 = min(strings), max(strings)
    i = 0
    while i < len(s1) and i < len(s2) and s1[i] == s2[i]:
        i += 1
    return s1[:i]


PRESENT_CELLS = ("pres.", "impv.", "part.pres.")


def verb_stems(lemma, strip, cells):
    """Stem 1 the infinitive's; stem 2 the present's (the longest prefix
    shared by the present, imperative and present-participle forms),
    named on the lexeme line when it is not stem 1."""
    stem1 = lemma[: len(lemma) - strip] if strip else lemma
    present = [f for k, v in cells.items() if k.startswith(PRESENT_CELLS) for f in v]
    stem2 = lcp(present) if present else stem1
    if not stem2 or len(stem2) < 2:
        stem2 = stem1
    return stem1, stem2


def seed(pos, entries, cells_of, name_of, out_file, header_cells, comment):
    """Group entries into classes and write the table and the cells."""
    groups = defaultdict(list)
    records = []
    for e in entries:
        cells = cells_of(e)
        if not cells:
            continue
        lemma = letters(e["word"])
        strip = strip_of(pos, lemma)
        stem = lemma[: len(lemma) - strip] if strip else lemma
        stems = stems_of(stem)
        own = {}
        if pos == "v":
            stem1, stem2 = verb_stems(lemma, strip, cells)
            stems = {"1": stem1, "2": stem2}
            if stem2 != stem1:
                own["2"] = stem2
        endings = {}
        for k, v in cells.items():
            prefer = ("2", "1") if (pos == "v" and k.startswith(PRESENT_CELLS)) else ("1", "5", "3", "2")
            endings[k] = [spec_of(f, stems, prefer) for f in v]
        group = name_of(e, lemma, strip)
        groups[group].append((lemma, strip, endings))
        records.append({"pos": pos, "lemma": e["word"], "letters": lemma, "class": group, "gender": gender_tag(e), "stems": own, "cells": cells})
    rows = []
    for group, members in sorted(groups.items()):
        strip = Counter(s for _, s, _ in members).most_common(1)[0][0]
        exemplar = members[0][0]
        cells = {}
        for cell in header_cells:
            votes = Counter()
            for _, s, endings in members:
                if s != strip:
                    continue
                for i, e in enumerate(endings.get(cell, [])):
                    if e is not None:
                        votes[e] += 1 if i == 0 else 0.5
            if not votes:
                continue
            total = sum(votes.values())
            ranked = [e for e, n in votes.most_common() if n >= 0.25 * total or e == votes.most_common(1)[0][0]]
            cells[cell] = "|".join(ranked[:3])
        if pos == "v":
            stems = "1=base;2=base"
        else:
            stems = "1=base;3=pal1;5=pal2" if any("5-" in v or "3-" in v for v in cells.values()) else "1=base"
        rows.append((group, exemplar, strip, stems, cells, len(members)))
    if pos == "a":
        # the treebanks' short masculine plural nominative in -ꙑ (свѧтꙑ)
        # beside the tables' -и; the contracted long nominative/accusative
        # (-ꙑ for -ꙑи, -и for -ии) as alternatives
        for group, exemplar, strip, stems, cells, n in rows:
            if group.endswith(":ъ") and "short.pos.m.pl.nom" in cells and "1-ы" not in cells["short.pos.m.pl.nom"]:
                cells["short.pos.m.pl.nom"] += "|1-ы"
            for k in ("long.pos.m.sg.nom", "long.pos.m.sg.acc"):
                v = cells.get(k, "")
                if v.startswith("1-ыи") and "1-ы" not in v.split("|"):
                    cells[k] = v + "|1-ы"
                if v.startswith("1-ии") and "1-и" not in v.split("|"):
                    cells[k] = v + "|1-и"
        # a class seeded from an incomplete table (Kaikki's soft вьсь) takes
        # its missing cells from the complete class of the same ending
        by_end = {}
        for group, exemplar, strip, stems, cells, n in rows:
            by_end.setdefault(group.rsplit(":", 1)[-1], []).append(cells)
        for group, exemplar, strip, stems, cells, n in rows:
            for other in by_end.get(group.rsplit(":", 1)[-1], []):
                if other is cells or len(other) <= len(cells):
                    continue
                for k, v in other.items():
                    cells.setdefault(k, v)
    if pos == "v":
        # the past participles the tables do not print: the active on stem 7
        # (base + вш after a vowel, + ъш after a consonant; nominative on
        # stem 11), the passive on stem 8 (-н- after а/ѣ/ꙗ, -т- after another
        # vowel, -ен- after a consonant or the и of the fourth class), both
        # declined as the seeded adjective classes
        fixed = []
        for group, exemplar, strip, stems, cells, n in rows:
            base = exemplar[: len(exemplar) - strip] if strip else exemplar
            vowel_end = base[-1:] in "аꙗѣиыꙋѫѧѥе"
            if vowel_end:
                stems += ";7=ext:вш;11=ext:в"
            else:
                stems += ";7=ext:ъш;11=ext:ъ"
            if base[-1:] in "аꙗѣ":
                stems += ";8=ext:н"
            elif base[-1:] == "и":
                stems += ";8=ext:ен"
            elif vowel_end:
                stems += ";8=ext:т"
            else:
                stems += ";8=ext:ен"
            cells.setdefault("part.past.act.short", "7~A:-:ь")
            cells.setdefault("part.past.act.long", "7~A:-:ь")
            cells.setdefault("part.past.act.short.m.sg.nom", "11-ъ")
            cells.setdefault("part.past.act.short.n.sg.nom", "11-ъ")
            cells.setdefault("part.past.act.short.f.sg.nom", "7-и")
            cells.setdefault("part.past.act.long.m.sg.nom", "7-ии|11-ыи")
            cells.setdefault("part.past.pass.short", "8~A:-:ъ")
            cells.setdefault("part.past.pass.long", "8~A:-:ъ")
            fixed.append((group, exemplar, strip, stems, cells, n))
        rows = fixed
        for extra in ("part.past.act.short", "part.past.act.long", "part.past.act.short.m.sg.nom", "part.past.act.short.n.sg.nom", "part.past.act.short.f.sg.nom", "part.past.act.long.m.sg.nom", "part.past.pass.short", "part.past.pass.long"):
            if extra not in header_cells:
                header_cells.append(extra)
    lines = [f"# {c}" for c in comment.strip().split("\n")]
    lines.append("\t".join(["class", "exemplar", "strip", "stems"] + header_cells))
    for group, exemplar, strip, stems, cells, n in rows:
        lines.append("\t".join([group, exemplar, str(strip), stems] + [cells.get(c, "-") for c in header_cells]))
    out_file.parent.mkdir(parents=True, exist_ok=True)
    out_file.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(rows)} classes ({len(records)} entries) to {out_file}")
    return records


def pronominal(hard):
    """The pronominal declension of тъ (hard) and сь (soft): cell → spec."""
    if hard:
        sg = {"m": {"nom": "1-ъ", "gen": "1-ого", "dat": "1-омꙋ", "acc": "1-ъ|1-ого", "ins": "1-ѣмь", "loc": "1-омь"},
              "f": {"nom": "1-а", "gen": "1-оѩ", "dat": "1-ои", "acc": "1-ѫ", "ins": "1-оѭ", "loc": "1-ои"},
              "n": {"nom": "1-о", "gen": "1-ого", "dat": "1-омꙋ", "acc": "1-о", "ins": "1-ѣмь", "loc": "1-омь"}}
        du = {"m": {"nom": "1-а", "acc": "1-а"}, "f": {"nom": "1-ѣ", "acc": "1-ѣ"}, "n": {"nom": "1-ѣ", "acc": "1-ѣ"}}
        du_common = {"gen": "1-ою", "loc": "1-ою", "dat": "1-ѣма", "ins": "1-ѣма"}
        pl = {"m": {"nom": "1-и", "acc": "1-ы"}, "f": {"nom": "1-ы", "acc": "1-ы"}, "n": {"nom": "1-а", "acc": "1-а"}}
        pl_common = {"gen": "1-ѣхъ", "loc": "1-ѣхъ", "dat": "1-ѣмъ", "ins": "1-ѣми"}
    else:
        sg = {"m": {"nom": "1-ь", "gen": "1-его", "dat": "1-емꙋ", "acc": "1-ь|1-его", "ins": "1-имь", "loc": "1-емь"},
              "f": {"nom": "1-и|1-ꙗ", "gen": "1-еѩ", "dat": "1-еи", "acc": "1-иѭ|1-ѭ", "ins": "1-еѭ", "loc": "1-еи"},
              "n": {"nom": "1-е", "gen": "1-его", "dat": "1-емꙋ", "acc": "1-е", "ins": "1-имь", "loc": "1-емь"}}
        du = {"m": {"nom": "1-иꙗ|1-ꙗ", "acc": "1-иꙗ|1-ꙗ"}, "f": {"nom": "1-и", "acc": "1-и"}, "n": {"nom": "1-и", "acc": "1-и"}}
        du_common = {"gen": "1-ею", "loc": "1-ею", "dat": "1-има", "ins": "1-има"}
        pl = {"m": {"nom": "1-ии|1-и", "acc": "1-иѩ|1-ѩ"}, "f": {"nom": "1-иѩ|1-ѩ", "acc": "1-иѩ|1-ѩ"}, "n": {"nom": "1-и|1-ꙗ", "acc": "1-и|1-ꙗ"}}
        pl_common = {"gen": "1-ихъ", "loc": "1-ихъ", "dat": "1-имъ", "ins": "1-ими"}
    cells = {}
    for g in "mfn":
        for c, v in sg[g].items():
            cells[f"{g}.sg.{c}"] = v
        cells[f"{g}.sg.voc"] = f"@{g}.sg.nom"
        for c, v in {**du[g], **du_common}.items():
            cells[f"{g}.du.{c}"] = v
        cells[f"{g}.du.voc"] = f"@{g}.du.nom"
        for c, v in {**pl[g], **pl_common}.items():
            cells[f"{g}.pl.{c}"] = v
        cells[f"{g}.pl.voc"] = f"@{g}.pl.nom"
    return cells


def write_pronoun_classes(path):
    rows = []
    hard = pronominal(True)
    soft = pronominal(False)
    rows.append(("PA1", "тъ", 1, "1=base", hard))
    velar = {k: v.replace("1-ѣ", "5-ѣ").replace("1-и", "5-и") if k.split(".")[1] != "sg" or k.endswith("ins") else v for k, v in hard.items()}
    velar["m.sg.nom"] = "1-ъ"
    rows.append(("PA1k", "вьсѣкъ", 1, "1=base;3=pal1;5=pal2", velar))
    rows.append(("PA1j", "сь", 1, "1=base", soft))
    fleeting = dict(soft)
    fleeting["m.sg.nom"] = "2-ь"
    fleeting["m.sg.acc"] = "2-ь|1-его"
    rows.append(("PA1j*", "вьсь", 1, "1=drop;2=base", fleeting))
    # the possessives мои/твои/свои/нашь: soft on the stem before -и/-ь
    poss = dict(soft)
    poss["m.sg.nom"] = "@lemma"
    poss["m.sg.acc"] = "@lemma|1-его"
    rows.append(("PA1a", "мои", 1, "1=base", poss))
    rows.append(("PA1s", "нашь", 1, "1=base", dict(soft)))
    # къто / чьто: the stem is on the lexeme line (stems=1=к)
    rows.append(("PPkto", "къто:к", 0, "1=base", {"m.sg.nom": "1-ъто", "m.sg.gen": "1-ого", "m.sg.dat": "1-омꙋ", "m.sg.acc": "1-ого", "m.sg.ins": "1-имь", "m.sg.loc": "1-омь", "m.sg.voc": "@m.sg.nom"}))
    rows.append(("PPcto", "чьто:ч", 0, "1=base", {"n.sg.nom": "1-ьто", "n.sg.gen": "1-есо|1-ьсо|1-есого", "n.sg.dat": "1-есомꙋ|1-емꙋ", "n.sg.acc": "1-ьто", "n.sg.ins": "1-имь", "n.sg.loc": "1-емь|1-есомь", "n.sg.voc": "@n.sg.nom"}))
    # the third person / relative: obliques on the empty stem
    third = {"m": {"nom": "1-и", "gen": "1-ѥго|1-него", "dat": "1-ѥмꙋ|1-немꙋ", "acc": "1-и|1-нь|1-ѥго", "ins": "1-имь|1-нимь", "loc": "1-немь|1-ѥмь"},
             "f": {"nom": "1-ꙗ", "gen": "1-ѥѩ|1-неѩ", "dat": "1-ѥи|1-неи", "acc": "1-ѭ|1-нѭ", "ins": "1-ѥѭ|1-неѭ", "loc": "1-неи|1-ѥи"},
             "n": {"nom": "1-ѥ", "gen": "1-ѥго|1-него", "dat": "1-ѥмꙋ|1-немꙋ", "acc": "1-ѥ|1-не", "ins": "1-имь|1-нимь", "loc": "1-немь|1-ѥмь"}}
    third_du = {"m": {"nom": "1-ꙗ", "acc": "1-ꙗ|1-нꙗ"}, "f": {"nom": "1-и", "acc": "1-и"}, "n": {"nom": "1-и", "acc": "1-и"}}
    third_du_common = {"gen": "1-ѥю|1-нею", "loc": "1-нею|1-ѥю", "dat": "1-има|1-нима", "ins": "1-има|1-нима"}
    third_pl = {"m": {"nom": "1-и", "acc": "1-ѩ|1-нѩ"}, "f": {"nom": "1-ѩ", "acc": "1-ѩ|1-нѩ"}, "n": {"nom": "1-ꙗ", "acc": "1-ꙗ|1-нꙗ"}}
    third_pl_common = {"gen": "1-ихъ|1-нихъ", "loc": "1-нихъ|1-ихъ", "dat": "1-имъ|1-нимъ", "ins": "1-ими|1-ними"}
    relative = {}
    personal3 = {}
    for g in "mfn":
        for n, table, common in (("sg", third, {}), ("du", third_du, third_du_common), ("pl", third_pl, third_pl_common)):
            for c, v in {**table[g], **common}.items():
                relative[f"{g}.{n}.{c}"] = v
                personal3[f"3.{g}.{n}.{c}"] = v
            relative[f"{g}.{n}.voc"] = f"@{g}.{n}.nom"
            personal3[f"3.{g}.{n}.voc"] = f"@3.{g}.{n}.nom"
        personal3[f"clit.3.{g}.sg.acc"] = {"m": "1-и", "f": "1-ѭ", "n": "1-ѥ"}[g]
        personal3[f"clit.3.{g}.pl.acc"] = "1-ѩ" if g != "n" else "1-ꙗ"
    rows.append(("PPize", "иже", 3, "1=base", relative))
    rows.append(("PP3", "и", 1, "1=base", personal3))
    # the first and second persons and the reflexive: stems on the lexeme
    # line (азъ: 1=мен;2=мън;3=м)
    rows.append(("PPja", "азъ:мен,мън,м", 0, "1=base;2=base;3=base", {
        "1.sg.nom": "@lemma", "1.sg.voc": "@lemma", "1.sg.gen": "1-е", "1.sg.dat": "2-ѣ", "1.sg.acc": "1-е|3-ѧ", "1.sg.ins": "2-оѭ", "1.sg.loc": "2-ѣ",
        "clit.1.sg.acc": "3-ѧ", "clit.1.sg.dat": "3-и"}))
    rows.append(("PPty", "ты:теб,тоб,т", 0, "1=base;2=base;3=base", {
        "2.sg.nom": "@lemma", "2.sg.voc": "@lemma", "2.sg.gen": "1-е", "2.sg.dat": "1-ѣ", "2.sg.acc": "1-е|3-ѧ", "2.sg.ins": "2-оѭ", "2.sg.loc": "1-ѣ",
        "clit.2.sg.acc": "3-ѧ", "clit.2.sg.dat": "3-и"}))
    rows.append(("PPseb", "себе:себ,соб,с", 0, "1=base;2=base;3=base", {
        "gen": "1-е", "dat": "1-ѣ", "acc": "1-е|3-ѧ", "ins": "2-оѭ", "loc": "1-ѣ", "clit.acc": "3-ѧ", "clit.dat": "3-и"}))
    rows.append(("PPmy", "мы:м,н", 0, "1=base;2=base", {
        "1.pl.nom": "1-ы", "1.pl.voc": "1-ы", "1.pl.gen": "2-асъ", "1.pl.dat": "2-амъ", "1.pl.acc": "2-ы|2-асъ", "1.pl.ins": "2-ами", "1.pl.loc": "2-асъ", "clit.1.pl.acc": "2-ы",
        "1.du.nom": "2-а|1-ы", "1.du.voc": "2-а", "1.du.gen": "2-аю", "1.du.dat": "2-ама", "1.du.acc": "2-а|2-ы", "1.du.ins": "2-ама", "1.du.loc": "2-аю"}))
    rows.append(("PPvy", "вы:в", 0, "1=base", {
        "2.pl.nom": "1-ы", "2.pl.voc": "1-ы", "2.pl.gen": "1-асъ", "2.pl.dat": "1-амъ", "2.pl.acc": "1-ы|1-асъ", "2.pl.ins": "1-ами", "2.pl.loc": "1-асъ", "clit.2.pl.acc": "1-ы",
        "2.du.nom": "1-а|1-ы", "2.du.voc": "1-а", "2.du.gen": "1-аю", "2.du.dat": "1-ама", "2.du.acc": "1-а|1-ы", "2.du.ins": "1-ама", "2.du.loc": "1-аю"}))
    # the nominal declension of етеръ, овъ, такъ: the hard adjective's
    # short series (the seeded A:-:ъ row) under the pronoun's cell names
    adj = CLASSES / "adj.tsv"
    if adj.exists():
        lines = [l.rstrip("\n").split("\t") for l in adj.open(encoding="utf-8") if not l.startswith("#")]
        head = lines[0]
        for r in lines[1:]:
            if r[0] == "A:-:ъ":
                d = dict(zip(head, r))
                nominal = {k[len("short.pos."):]: v.replace("@short.pos.", "@") for k, v in d.items() if k.startswith("short.pos.") and v != "-"}
                nominal["m.sg.acc"] = "@m.sg.nom|1-а"
                rows.append(("PN", "етеръ", 1, "1=base", nominal))
                velar = {k: v.replace("1-ѣ", "5-ѣ").replace("1-и", "5-и") for k, v in nominal.items()}
                rows.append(("PNk", "такъ", 1, "1=base;3=pal1;5=pal2", velar))
    header = []
    for _, _, _, _, cells in rows:
        for c in cells:
            if c not in header:
                header.append(c)
    lines = ["# Old Church Slavonic pronoun classes, hand-written from Kaikki's tables of тъ, сь, иже, къто, чьто and",
             "# the UD PROIEL train forms of the personal pronoun (scripts/kaikki-to-classes.py writes this file).",
             "\t".join(["class", "exemplar", "strip", "stems"] + header)]
    for code, exemplar, strip, stems, cells in rows:
        lines.append("\t".join([code, exemplar, str(strip), stems] + [cells.get(c, "-") for c in header]))
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {len(rows)} pronoun classes to {path}")


def main():
    entries = [json.loads(l) for l in SRC.open(encoding="utf-8")]
    # Glagolitic headwords (ⰴⱁⰱⱃⱏ) are the Cyrillic entry's twin: skipped
    entries = [e for e in entries if not any("\u2c00" <= c <= "\u2c5f" for c in e["word"])]
    by_pos = defaultdict(list)
    for e in entries:
        by_pos[e["pos"]].append(e)
    records = []
    noun_header = [f"{c}.{n}" for n in ("sg", "du", "pl") for c in ("nom", "gen", "dat", "acc", "ins", "loc", "voc")]

    def noun_name(e, lemma, strip):
        stem = lemma[: len(lemma) - strip] if strip else lemma
        velar = "k" if stem[-1:] in PAL2 else ""
        return f"{class_tag(e)}{velar}:{lemma[len(lemma) - strip:] or '0'}:{gender_tag(e)}".replace("-stem", "")

    records += seed("n", by_pos["noun"], noun_cells, noun_name, CLASSES / "noun.tsv", noun_header,
                    "Old Church Slavonic noun classes seeded from Kaikki's tables by scripts/kaikki-to-classes.py:\nclass = Kaikki stem class : nominative ending : gender; the majority ending per cell, a second ending\nas an alternative when a quarter of the group uses it. No stress. Hand-maintained since.")
    adj_header = [f"{s}.pos.{g}.{n}.{c}" for s in ("short", "long") for g in ("m", "f", "n") for n in ("sg", "du", "pl") for c in ("nom", "gen", "dat", "acc", "ins", "loc", "voc")]

    def adj_name(e, lemma, strip):
        stem = lemma[: len(lemma) - strip] if strip else lemma
        velar = "k" if stem[-1:] in PAL2 else ""
        return f"A:{class_tag(e)}{velar}:{lemma[len(lemma) - strip:] or '0'}".replace("-stem", "")

    records += seed("a", by_pos["adj"], adj_cells, adj_name, CLASSES / "adj.tsv", adj_header,
                    "Old Church Slavonic adjective classes seeded from Kaikki's tables (short and long series).")
    verb_header = [f"{t}.{p}.{n}" for t in ("pres", "impf", "aor", "impv") for n in ("sg", "du", "pl") for p in ("1", "2", "3")] + ["inf"] + [f"lpart.{g}.{n}" for g in ("m", "f", "n") for n in ("sg", "du", "pl")]
    verb_header += [f"part.{t}.act.{s}.{g}.{n}.{c}" for t in ("pres", "past") for s in ("short", "long") for g in ("m", "f", "n") for n in ("sg", "du", "pl") for c in ("nom", "gen", "dat", "acc", "ins", "loc", "voc")]

    def verb_name(e, lemma, strip):
        # the infinitive's ending (its theme included), the present's
        # first- and third-person endings on the present stem, and whether
        # the present stem is the infinitive's (дѣлати:аѭ; пити:ѭ,ѥтъ:2)
        cells = verb_cells(e)
        stem1, stem2 = verb_stems(lemma, strip, cells)
        p1 = cells.get("pres.1.sg", [""])[0]
        p3 = cells.get("pres.3.sg", [""])[0]
        e1 = p1[len(stem2):] if p1.startswith(stem2) else "?"
        e3 = p3[len(stem2):] if p3.startswith(stem2) else "?"
        inf_end = lemma[-3:] if strip == 2 and lemma[-3] in "аиѣꙗыꙋѫѧѥе" else lemma[-2:]
        return f"V:{inf_end}:{e1},{e3}" + (":2" if stem2 != stem1 else "")

    records += seed("v", by_pos["verb"], verb_cells, verb_name, CLASSES / "verb.tsv", verb_header,
                    "Old Church Slavonic verb classes seeded from Kaikki's conjugation tables: the finite rows read\nby position (1, 2, 3 per number; the singular of the aorist, imperfect and imperative shares 2 = 3),\nthe l-participle, the infinitive, and the participle tables under the present/past banners.")
    pron_header = [f"{g}.{n}.{c}" for g in ("m", "f", "n") for n in ("sg", "du", "pl") for c in ("nom", "gen", "dat", "acc", "ins", "loc", "voc")]

    def pron_cells(e):
        return adj_cells(e, series_required=False)

    # the pronoun cells go to the importer; the classes are hand-written
    # below (Kaikki's pronoun headwords are mostly form-of entries)
    for e in by_pos["pron"]:
        cells = pron_cells(e)
        if cells:
            records.append({"pos": "pron", "lemma": e["word"], "letters": letters(e["word"]), "class": "-", "gender": gender_tag(e), "cells": cells})
    write_pronoun_classes(CLASSES / "pronoun.tsv")
    with CELLS.open("w", encoding="utf-8") as f:
        for r in records:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    print(f"wrote {len(records)} entries to {CELLS}")


if __name__ == "__main__":
    main()
