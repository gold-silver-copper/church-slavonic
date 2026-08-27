#!/usr/bin/env bash
# Download A. E. Polyakov's corpus-based grammatical dictionary of Church
# Slavonic (tagged edition) from its public host into
# references/downloads/polyakov/. Discovers the letter pages from the
# navigation frame, so a re-run picks up any pages added upstream, then
# writes downloads/polyakov/SHA256SUMS.txt. The pinned checksums live in
# references/SHA256SUMS and references/SOURCE_LOCK.tsv; regenerate those rows
# with scripts/pin-polyakov.sh after a deliberate refresh.
set -euo pipefail
cd "$(dirname -- "$0")/../references"
base="http://dic.feb-web.ru/slavonic/dicgram"
dest="downloads/polyakov"
ua="church-slavonic-extractor (gold-silver-copper; institutional access)"
mkdir -p "$dest"
curl -fsSL --retry 3 -A "$ua" -o "$dest/indexnav.htm" "$base/indexnav.htm"
pages=$(grep -oE 'href="[^"]+\.htm"' "$dest/indexnav.htm" | cut -d'"' -f2 | sort -u)
for p in index1.htm $pages; do
    mkdir -p "$dest/$(dirname -- "$p")"
    echo "fetching $p"
    curl -fsSL --retry 3 -A "$ua" -o "$dest/$p" "$base/$p"
    sleep 1
done
(cd "$dest" && find . -type f -name '*.htm' | sort | xargs shasum -a 256 > SHA256SUMS.txt)
echo "downloaded $(wc -l < "$dest/SHA256SUMS.txt" | tr -d ' ') pages into $dest"
