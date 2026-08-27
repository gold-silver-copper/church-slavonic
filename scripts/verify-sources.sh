#!/usr/bin/env bash
# Verify every downloaded source artifact against references/SHA256SUMS.
set -euo pipefail
cd "$(dirname -- "$0")/../references"
shasum -a 256 -c --quiet SHA256SUMS && echo "all pinned artifacts verified"
