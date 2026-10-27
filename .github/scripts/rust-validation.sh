#!/usr/bin/env bash
set -euo pipefail

# CI coverage and release validation must compile/run the same test targets.
# Doctests run separately because llvm-cov does not instrument them here.
case "${1:-}" in
  build) extra=(--lib --bins --tests --no-run) ;;
  test) extra=(--lib --bins --tests) ;;
  doc) extra=(--doc) ;;
  *) echo 'Usage: rust-validation.sh {build|test|doc}' >&2; exit 2 ;;
esac
if [[ $# != 1 ]]; then
  echo 'Expected exactly one validation mode' >&2
  exit 2
fi
if [[ "$1" != test ]]; then
  exec cargo test --locked --workspace --all-features "${extra[@]}"
fi

# Always collect a fresh execution inventory: inherited references could hide
# a snapshot whose last test has been deleted. Keep coverage's cargo scope intact.
references=$(mktemp)
trap 'rm -f -- "$references"' EXIT
export INSTA_SNAPSHOT_REFERENCES_FILE="$references"
cargo test --locked --workspace --all-features "${extra[@]}"
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
node "$script_dir/rust-snapshot-references.cjs" "$references"
