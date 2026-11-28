#!/bin/bash
set -u
args=(--x)
for f in "$@"; do
  args=()
  args+=("$f")
done
use "${args[@]}"

items=(--x)
for f in "$@"; do
  use "${items[@]}"
  items=()
  items+=(--y)
done
