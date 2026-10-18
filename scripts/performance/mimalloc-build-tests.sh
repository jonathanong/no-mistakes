#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
source "$root/scripts/performance/mimalloc-build-args.sh"
capture() { printf '%s\n' "$@"; }

for variant in system mimalloc; do
  actual=$(mimalloc_build "$variant" capture --release --locked)
  expected=$(cat "$root/fixtures/mimalloc-build/$variant-args.txt")
  if [[ "$actual" != "$expected" ]]; then
    printf '%s argument mismatch\nexpected:\n%s\nactual:\n%s\n' \
      "$variant" "$expected" "$actual" >&2
    exit 1
  fi
done
