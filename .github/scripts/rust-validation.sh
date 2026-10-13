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
exec cargo test --locked --workspace --all-features "${extra[@]}"
