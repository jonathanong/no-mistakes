#!/bin/bash
set -u

die() { exit 1; }
die() ( : )
items=()
if [[ ${#items[@]} -eq 0 ]]; then
  die
fi
printf '%s\n' "${items[@]}"

fill() { filled=(one); }
fill() ( : )
filled=()
fill
printf '%s\n' "${filled[@]}"
