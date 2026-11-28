#!/bin/bash
set -euo pipefail

IFS=',' read -ra items <<< "${1:-}"
if [[ ${#items[@]} -eq 0 ]]; then
  exit 1
fi
# no-mistakes-disable-next-line bash-empty-array-nounset -- length guard above ensures non-empty
printf '%s\n' "${items[@]}"
