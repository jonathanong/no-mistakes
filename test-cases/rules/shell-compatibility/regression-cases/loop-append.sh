#!/bin/bash
set -u
args=(--x)
for f in "$@"; do
  args+=("$f")
done
cmd "${args[@]}"
