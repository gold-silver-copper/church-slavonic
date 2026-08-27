#!/usr/bin/env bash
# Fetch the two pinned sources the extractor reads (see SOURCE_LOCK.tsv) into
# references/downloads/ and verify them against SHA256SUMS. Idempotent: files
# whose checksum already matches are not downloaded again.
set -euo pipefail

cd "$(dirname -- "$0")"

while IFS=$'\t' read -r source_id artifact_id transport url path sha256 size format signature content_types; do
    [ "$source_id" = "source_id" ] && continue
    if [ -f "$path" ] && printf '%s  %s\n' "$sha256" "$path" | shasum -a 256 -c --status; then
        continue
    fi
    mkdir -p "$(dirname -- "$path")"
    echo "fetching $path"
    curl -fsSL --retry 3 -o "$path" "$url"
done < SOURCE_LOCK.tsv

shasum -a 256 -c --quiet SHA256SUMS
echo "references/downloads is complete and verified"
