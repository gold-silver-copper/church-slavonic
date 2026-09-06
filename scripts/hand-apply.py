#!/usr/bin/env python3
# hand-apply.py — the hand overlay workflow of 3.1 Part 4 (V3.0-PROMPT.md).
# It never chooses: it lists the set and :amb leaves of a `cargo xtask
# hand-draft` output with a number each, and applies a decisions file
# (one line per leaf: a cell name, `<id> <cell>` for an :amb word,
# `abbr "<prefix>" <id> <cell> [alt n]` for a titlo token, `w "<surface>"
# [:lemma … :case …]` for a verbatim leaf, a trailing `noalt`/`alt n`
# to correct the draft's alternative index) to produce the overlay text.
"""hand_apply.py list <draft>            -> numbered decisions to make
   hand_apply.py apply <draft> <decisions> <out>  -> the overlay text
A decision line: `<verse>:<n> <value>` where value is a cell name for a
set leaf (`acc.sg`, `pres.3.sg`, `long.pos.m.sg.nom`), or `<id> <cell>`
for an :amb word (a leaf is built from the id's pos), or `w` to keep an
:amb word verbatim, or `w :lemma X :case Y` for a verbatim leaf with notes."""
import re,sys
LEAF=re.compile(r'\((n|adj|adv|v|lp|part|pn|f|w|p) ("[^"]*"|[^\s()]+)((?:\s+:[a-z-]+ [^\s()]+)*)\)')
def leaves(line):
    return list(LEAF.finditer(line))
def needs(m):
    kind,ident,fs=m.group(1),m.group(2),m.group(3)
    if kind=='w' and ':amb' in fs: return True
    return '|' in fs
def kind_of(ident, cell):
    pos=ident.split('.')[1] if '.' in ident else 'x'
    if pos=='n': return 'n'
    if pos=='a': return 'adv' if cell.startswith('adv') else 'adj'
    if pos=='v':
        if cell.startswith('part.'): return 'part'
        if cell.startswith('lpart'): return 'lp'
        return 'v'
    if pos=='pron': return 'pn'
    return 'f'
def run_list(draft):
    verse=None
    for line in open(draft,encoding='utf-8'):
        if line.startswith('; ') and re.match(r'; \d+:\d+ ',line):
            verse=line.split()[1]; print(line.rstrip()[:400]); continue
        if line.startswith(';   '): print(line.rstrip()[:300]); continue
        if line.startswith('(verse'):
            n=0
            for m in leaves(line):
                if needs(m):
                    n+=1; print(f'  {verse}:{n} {m.group(0)}')
def run_apply(draft,decisions,out):
    dec={}
    for l in open(decisions,encoding='utf-8'):
        l=l.strip()
        if not l or l.startswith('#'): continue
        key,_,val=l.partition(' '); dec[key]=val.strip()
    res=[]; verse=None; missing=[]
    for line in open(draft,encoding='utf-8'):
        if line.startswith('; ') and re.match(r'; \d+:\d+ ',line):
            verse=line.split()[1]
        if line.startswith(';'): continue
        if line.startswith('(verse'):
            n=0; pieces=[]; last=0
            for m in leaves(line):
                if not needs(m): continue
                n+=1; key=f'{verse}:{n}'
                if key not in dec: missing.append(key); continue
                val=dec[key]; kind,ident,fs=m.group(1),m.group(2),m.group(3)
                alt=re.search(r':alt (\d+)',fs)
                if val.startswith('abbr '):
                    # abbr "<prefix>" <id> <cell> [alt n]: a titlo-written token
                    parts=val.split(); prefix=parts[1]; ident2=parts[2]; cell=parts[3]
                    alt2=(' :alt '+parts[5]) if len(parts)>5 and parts[4]=='alt' else ''
                    rep=f'(abbr {prefix} ({kind_of(ident2,cell)} {ident2} :cell {cell}{alt2}))'
                elif val.startswith('w "'):
                    # a verbatim leaf with its own surface (a wrong lexeme)
                    rep='('+val+')'
                elif kind=='w':
                    if val=='w' or val.startswith('w '):
                        notes=val[1:].strip()
                        rep=f'(w {ident}'+(f' {notes}' if notes else '')+')'
                    else:
                        ident2,cell=val.split()
                        k2=kind_of(ident2,cell)
                        rep=f'(f {ident2})' if k2=='f' else f'({k2} {ident2} :cell {cell})'
                else:
                    # `<cell> noalt` drops the draft's :alt (it named the set's
                    # form, not this cell's); `<cell> alt N` sets it
                    parts=val.split()
                    if len(parts)>=2 and parts[1]=='noalt': alt=None; val=parts[0]
                    elif len(parts)>=3 and parts[1]=='alt': val=parts[0]; alt=re.match(r'(\d+)',parts[2])
                    rep=f'({kind} {ident} :cell {val}'+(f' :alt {alt.group(1)}' if alt else '')+')'
                # a capitalised :amb token: the leaf goes under (cap …) with a
                # lowercase surface, as the lifter writes every other capital
                if kind=='w' and ident.startswith('"') and len(ident)>1 and ident[1].isupper() and not val.startswith('w'):
                    rep='(cap '+rep+')'
                elif kind=='w' and ident.startswith('"') and len(ident)>1 and ident[1].isupper() and val.startswith('w "'):
                    rep='(cap ('+val[0]+' "'+val[3].lower()+val[4:]+')'
                pieces.append(line[last:m.start()]); pieces.append(rep); last=m.end()
            pieces.append(line[last:]); line=''.join(pieces)
        res.append(line)
    if missing: print('MISSING', ' '.join(missing), file=sys.stderr)
    open(out,'w',encoding='utf-8').write(''.join(res))
if sys.argv[1]=='list': run_list(sys.argv[2])
else: run_apply(sys.argv[2],sys.argv[3],sys.argv[4])
