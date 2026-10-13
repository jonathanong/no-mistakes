#!/usr/bin/env bash
set -uo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  printf 'Usage: %s <stage> [target-dir]\n' "$0" >&2
  exit 2
fi

stage="$1"
target_dir="${2:-${CARGO_TARGET_DIR:-target}}"

printf '::group::Rust disk usage: %s\n' "$stage"
printf 'Workspace filesystem:\n'
if ! df -h .; then
  printf 'Could not read workspace filesystem usage.\n'
fi

if [[ -d "$target_dir" ]]; then
  printf 'Cargo target filesystem (%s):\n' "$target_dir"
  if ! df -h "$target_dir"; then
    printf 'Could not read Cargo target filesystem usage.\n'
  fi

  printf 'Cargo target breakdown (maximum depth 3, at most 30 rows):\n'
  if ! du -h --max-depth=3 "$target_dir" | sort -hr | awk 'NR <= 30'; then
    printf 'Could not read Cargo target directory usage.\n'
  fi
else
  printf 'Cargo target directory does not exist: %s\n' "$target_dir"
fi
printf '::endgroup::\n'
