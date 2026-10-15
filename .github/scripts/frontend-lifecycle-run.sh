#!/usr/bin/env bash
# One unique lifecycle URI per normal memory run; preserve the original action.
set -euo pipefail
mode=${1:?lifecycle mode required}
case "$mode" in
  cold_first_analysis|conditioning_inclusive|extract_after_recorded_prefix) ;;
  *) echo "Unknown lifecycle mode: $mode" >&2; exit 1 ;;
esac
mapfile -t binaries < <(find target/codspeed/analysis -type f -perm -111)
[[ ${#binaries[@]} -eq 1 ]]
export NO_MISTAKES_FRONTEND_LIFECYCLE="$mode"
exec "${binaries[0]}" "language_frontend_lifecycle::$mode\$"
