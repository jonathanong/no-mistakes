#!/usr/bin/env bash
set -euo pipefail

compiler="$1"
shift
args=("$@")
while [[ $# -gt 0 ]]; do
  if [[ "$1" == "--crate-name" && "${2-}" == "no_mistakes" ]]; then
    exec flock "${RUNNER_TEMP:?}/nm-coverage-rustc.lock" "$compiler" "${args[@]}"
  fi
  shift
done
exec "$compiler" "${args[@]}"
