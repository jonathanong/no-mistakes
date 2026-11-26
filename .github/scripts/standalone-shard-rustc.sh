#!/usr/bin/env bash
# One standalone archive compiles a slice of the unit tests. Items used only by
# another slice are unused here. Clippy still denies both lints for the full suite.
set -euo pipefail

compiler="$1"
shift
args=("$@")
shard="${NM_TEST_SHARD-}"
if [[ "$shard" == "0" || "$shard" == "1" || "$shard" == "2" || "$shard" == "postgres-source" ]]; then
  crate=""
  previous=""
  for arg in "$@"; do
    if [[ "$previous" == "--crate-name" ]]; then
      crate="$arg"
      break
    fi
    previous="$arg"
  done
  if [[ "$crate" == "no_mistakes" || "$crate" == "no_mistakes_invocation" || "$crate" == "no_mistakes_postgres_sql" ]]; then
    args+=("-A" "dead_code" "-A" "unused_imports")
  fi
fi
if [[ -n "${NM_INNER_RUSTC_WRAPPER-}" ]]; then
  exec "$NM_INNER_RUSTC_WRAPPER" "$compiler" "${args[@]}"
fi
exec "$compiler" "${args[@]}"
