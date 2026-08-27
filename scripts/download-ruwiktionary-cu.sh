#!/usr/bin/env bash
# Download the Kaikki/Wiktextract extract of Russian Wiktionary's
# Церковнославянский section into references/downloads/ruwiktionary-cu/.
# The pinned checksum is in references/SHA256SUMS; a newer Kaikki build will
# not match it — that is a deliberate refresh, so re-pin and review.
set -euo pipefail
cd "$(dirname -- "$0")/../references"
page="https://kaikki.org/ruwiktionary/%D0%A6%D0%B5%D1%80%D0%BA%D0%BE%D0%B2%D0%BD%D0%BE%D1%81%D0%BB%D0%B0%D0%B2%D1%8F%D0%BD%D1%81%D0%BA%D0%B8%D0%B9/"
dest="downloads/ruwiktionary-cu"
mkdir -p "$dest"
link=$(curl -fsSL "$page" | grep -oE 'href="[^"]+\.jsonl"' | head -1 | cut -d'"' -f2)
curl -fsSL --retry 3 -o "$dest/kaikki.org-dictionary-Церковнославянский.jsonl" "$page$link"
shasum -a 256 "$dest/kaikki.org-dictionary-Церковнославянский.jsonl"
