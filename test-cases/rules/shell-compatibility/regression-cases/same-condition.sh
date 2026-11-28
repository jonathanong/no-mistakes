#!/bin/bash
set -u
MODE="${1:-}"
cmd=()
if [[ "$MODE" == exec ]]; then
  cmd=(run --fast)
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${cmd[@]}"
fi

steps=()
if contains build; then
  steps+=(compile)
fi
if contains build; then
  printf '%s\n' "${steps[@]}"
fi
