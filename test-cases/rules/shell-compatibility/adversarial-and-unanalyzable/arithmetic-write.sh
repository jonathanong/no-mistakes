#!/bin/bash
set -u

MODE=1
items=()
if (( MODE == 1 )); then
  items=(ready)
fi
(( MODE++ ))
if (( MODE == 1 )); then
  printf '%s\n' "${items[@]}"
fi
