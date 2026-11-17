#!/usr/bin/env bash
set -euo pipefail

# CI coverage and release validation must compile/run the same test targets.
# Doctests run separately because llvm-cov does not instrument them here.
case "${1:-}" in
  build) extra=(--lib --bins --tests --no-run) ;;
  test) extra=(--lib --bins --tests) ;;
  test-unit) extra=(--lib --bins) ;;
  test-integration) extra=(--test "*") ;;
  doc) extra=(--doc) ;;
  *) echo 'Usage: rust-validation.sh {build|test|test-unit|test-integration|doc}' >&2; exit 2 ;;
esac
if [[ $# != 1 ]]; then
  echo 'Expected exactly one validation mode' >&2
  exit 2
fi
if [[ "$1" == build || "$1" == doc ]]; then
  exec cargo test --locked --workspace --all-features "${extra[@]}"
fi

case "$1" in
  test)
    # Inherited references must never hide snapshots whose tests were deleted.
    references=$(mktemp)
    ;;
  test-unit|test-integration)
    references=${RUST_VALIDATION_REFERENCES_FILE:?Set RUST_VALIDATION_REFERENCES_FILE for split tests}
    if [[ "$1" == test-integration && ! -f "$references" ]]; then
      echo 'Run test-unit successfully before test-integration with the same references file' >&2
      exit 2
    fi
    ;;
esac
trap 'rm -f -- "$references"' EXIT
if [[ "$1" == test-unit ]]; then
  : > "$references"
fi
export INSTA_SNAPSHOT_REFERENCES_FILE="$references"
cargo test --locked --workspace --all-features "${extra[@]}"
if [[ "$1" == test-unit ]]; then
  trap - EXIT
  exit 0
fi
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
node "$script_dir/rust-snapshot-references.cjs" "$references"
