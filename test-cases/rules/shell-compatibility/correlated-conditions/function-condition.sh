#!/bin/bash
set -u

contains() {
  [[ "${1:-}" == build ]]
}

tasks=()
if contains build || contains test; then
  tasks=(compile)
fi
if contains build; then
  printf '%s\n' "${tasks[@]}"
fi

printf '%s\n' "${tasks[@]}"
