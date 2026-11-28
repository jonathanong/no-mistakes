#!/bin/bash
set -u
IFS=',' read -ra items <<< "$1"
if [[ ${#items[@]} -eq 0 ]]; then
  echo "Error: items requires an argument" >&2
  exit 1
fi
for item in "${items[@]}"; do
  echo "$item"
done
