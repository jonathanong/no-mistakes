#!/bin/bash
set -u

check_local_mutation() {
  local MODE="${1:-}"
  local -a items=()
  if [[ "$MODE" == a ]]; then
    items=(one)
  else
    MODE=a
  fi
  if [[ "$MODE" == a ]]; then
    printf '%s\n' "${items[@]}"
  fi
}
