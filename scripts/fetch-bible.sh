#!/bin/sh
# Fetch the pinned 1757 Elizabethan Bible JSON — the phase-4 source, reused
# verbatim (see crates/church-slavonic-syntax/src/bible.rs). The 12 MB file
# is gitignored: the text is public domain, the JSON arrangement's repo
# declares no license, so nothing is vendored. Offline: exits 0 and the
# treebank pipeline skips soft.
set -e
cd "$(dirname "$0")/.."
DST=data/bible-src
URL="https://raw.githubusercontent.com/asdf-a11/ChurchSlavonicBibleInUtf8/main/CSlElizabeth-CS.json"
SHA="de40ffb4457c2d61f1330eff631496091ad69046efa08781326cdf733e28dc1e"
mkdir -p "$DST"
if [ ! -s "$DST/CSlElizabeth-CS.json" ]; then
    if ! curl -fsSL --max-time 300 "$URL" -o "$DST/CSlElizabeth-CS.json.tmp"; then
        rm -f "$DST/CSlElizabeth-CS.json.tmp"
        echo "fetch-bible: OFFLINE — the treebank pipeline will skip" >&2
        exit 0
    fi
    mv "$DST/CSlElizabeth-CS.json.tmp" "$DST/CSlElizabeth-CS.json"
fi
echo "$SHA  $DST/CSlElizabeth-CS.json" | shasum -a 256 -c - || {
    echo "fetch-bible: SHA256 MISMATCH — refusing the text"; exit 1; }
