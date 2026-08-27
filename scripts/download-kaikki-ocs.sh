#!/usr/bin/env bash
# Download the Kaikki/Wiktextract extract of English Wiktionary's Old Church
# Slavonic entries into references/downloads/english-wiktionary-ocs/.
set -euo pipefail
cd "$(dirname -- "$0")/../references"
dest="downloads/english-wiktionary-ocs"
mkdir -p "$dest"
curl -fsSL --retry 3 -o "$dest/kaikki.org-dictionary-OldChurchSlavonic.jsonl" \
  "https://kaikki.org/dictionary/Old%20Church%20Slavonic/kaikki.org-dictionary-OldChurchSlavonic.jsonl"
shasum -a 256 "$dest/kaikki.org-dictionary-OldChurchSlavonic.jsonl"
