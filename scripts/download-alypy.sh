#!/usr/bin/env bash
# Download the 198 pages of Archbishop Alypy's Grammar of the Church Slavonic
# Language (web edition) into references/downloads/alypy-grammar/, using the
# page list pinned in references/SOURCE_LOCK.tsv.
set -euo pipefail
cd "$(dirname -- "$0")/../references"
grep '^alypy-gamanovich-grammar-web-2023	' SOURCE_LOCK.tsv | while IFS=$'\t' read -r _ _ _ url path _ _ _ _ _; do
    mkdir -p "$(dirname -- "$path")"
    curl -fsSL --retry 3 -o "$path" "$url"
done
echo "downloaded $(grep -c '^alypy-gamanovich-grammar-web-2023	' SOURCE_LOCK.tsv) pages"
