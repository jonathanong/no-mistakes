#!/usr/bin/env bash

mimalloc_build() {
  local variant="$1"
  shift
  if [[ "$variant" == system ]]; then
    "$@" --no-default-features --features mermaid-validation
  else
    "$@"
  fi
}

mimalloc_cargo_jobs() {
  case "$1" in
    aarch64-unknown-linux-gnu) nproc ;;
    aarch64-apple-darwin) sysctl -n hw.logicalcpu ;;
    *) return 2 ;;
  esac
}
