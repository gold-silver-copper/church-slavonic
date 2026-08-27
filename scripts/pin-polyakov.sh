#!/usr/bin/env bash
# Rewrite the Polyakov rows of references/SHA256SUMS and SOURCE_LOCK.tsv from
# the files currently in references/downloads/polyakov/ (after a deliberate
# refresh via scripts/download-polyakov.sh). Review the diff before committing.
set -euo pipefail
cd "$(dirname -- "$0")/../references"
id="polyakov-church-slavonic-grammatical-dictionary"
grep -v "downloads/polyakov/" SHA256SUMS > SHA256SUMS.tmp && mv SHA256SUMS.tmp SHA256SUMS
grep -v "^$id	" SOURCE_LOCK.tsv > SOURCE_LOCK.tmp && mv SOURCE_LOCK.tmp SOURCE_LOCK.tsv
while read -r sha rel; do
    f=${rel#./}
    path="downloads/polyakov/$f"
    size=$(wc -c < "$path" | tr -d ' ')
    printf '%s  %s\n' "$sha" "$path" >> SHA256SUMS
    printf '%s\t%s:%s\tdirect\thttp://dic.feb-web.ru/slavonic/dicgram/%s\t%s\t%s\t%s\thtml\thtml\t-\n' "$id" "$id" "$f" "$f" "$path" "$sha" "$size" >> SOURCE_LOCK.tsv
done < downloads/polyakov/SHA256SUMS.txt
echo "pinned $(grep -c "^$id	" SOURCE_LOCK.tsv) Polyakov artifacts"
