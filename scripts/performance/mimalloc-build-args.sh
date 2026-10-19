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

