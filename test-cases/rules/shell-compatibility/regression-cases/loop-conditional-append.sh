#!/bin/bash
set -u
for target in alpha beta; do
  arguments=()
  if [[ "$target" == alpha ]]; then
    arguments+=(--flag)
  fi
  printf '%s\n' "${arguments[@]}"
done
