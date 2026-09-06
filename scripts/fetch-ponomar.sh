#!/usr/bin/env bash
# Fetch and pin the Ponomar library (https://www.ponomar.net/maktabah/):
# every HTML page of every book directory the index lists, into
# data/corpus/ponomar/<book>/, with a manifest of URL and sha256 per page
# (data/corpus/ponomar/MANIFEST.tsv). A page already present and matching
# its manifest hash is not fetched again; one request at a time, a pause
# between requests. The maintainer licensed the texts to this project
# (data/corpus/ponomar/LICENSE.md).
set -euo pipefail
cd "$(dirname "$0")/.."
BASE="https://www.ponomar.net/maktabah"
DST="data/corpus/ponomar"
MANIFEST="$DST/MANIFEST.tsv"
PAUSE="${PAUSE:-0.5}"
mkdir -p "$DST"
[ -s "$MANIFEST" ] || printf 'book\tpage\turl\tsha256\n' > "$MANIFEST"
BOOKS=$(curl -fsSL "$BASE/index.html" | grep -oE "href=\"$BASE/[^\"/]+/\"" | sed -E 's#.*maktabah/([^"/]+)/"#\1#' | sort -u)
for book in $BOOKS; do
    mkdir -p "$DST/$book"
    pages=$(curl -fsSL "$BASE/$book/" | grep -oE 'href="[^"]+\.html?"' | sed -E 's/href="([^"]+)"/\1/' | grep -vE '^(https?:|/|\.\.)' | sort -u)
    sleep "$PAUSE"
    for page in $pages; do
        out="$DST/$book/$page"
        if [ -s "$out" ] && grep -qF "	$book	$page	" "$MANIFEST" 2>/dev/null; then
            continue
        fi
        if [ -s "$out" ] && grep -qP "^$book\t$page\t" "$MANIFEST"; then continue; fi
        curl -fsSL --max-time 120 "$BASE/$book/$page" -o "$out.tmp" && mv "$out.tmp" "$out"
        sha=$(shasum -a 256 "$out" | cut -d' ' -f1)
        printf '%s\t%s\t%s\t%s\n' "$book" "$page" "$BASE/$book/$page" "$sha" >> "$MANIFEST"
        sleep "$PAUSE"
    done
    echo "$book: $(ls "$DST/$book" | wc -l | tr -d ' ') pages"
done
echo "fetched: $(($(wc -l < "$MANIFEST") - 1)) pages in $(ls -d "$DST"/*/ | wc -l | tr -d ' ') books"
