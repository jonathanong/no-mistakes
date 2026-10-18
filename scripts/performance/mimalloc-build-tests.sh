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

# Match Cargo's native runner parallelism on both ARM builders.
nproc() {
  printf '4\n'
}
sysctl() {
  [[ "$*" == '-n hw.logicalcpu' ]] || return 2
  printf '3\n'
}
[[ "$(mimalloc_cargo_jobs aarch64-apple-darwin)" == 3 ]]
[[ "$(mimalloc_cargo_jobs aarch64-unknown-linux-gnu)" == 4 ]]
if mimalloc_cargo_jobs unknown-target; then
  printf 'unexpected build target accepted\n' >&2
  exit 1
fi
