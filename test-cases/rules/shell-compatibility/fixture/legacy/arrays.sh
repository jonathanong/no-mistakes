#!/usr/bin/env bash
items=()
collect "${items[@]}"
set -euo pipefail
collect "${items[@]}"
collect "${items[*]}"
collect ${items[@]+"${items[@]}"}
collect ${items[*]+"${items[*]}"}
collect "${items[@]+"${items[@]}"}"
collect '${items[@]}' "\${items[@]}" ${items[@]}
# collect "${items[@]}"
collect "${other[@]}"
collect "${label:-café}" ${items[@]+"${items[@]}"}
items=('' stable)
collect "${items[@]}"
items=(
  # reset
); collect "${items[@]}"
items+=(stable)
collect "${items[@]}"
unset items
collect "${items[@]}"
set +u
collect "${items[@]}"
declare -a declared
declared+=()
set -o nounset
collect \
  "${declared[*]}"
declare -a appended+=("$@")
collect "${appended[@]}"
show_items() {
  local -a local_items
  collect "${local_items[@]}"
}
cat <<'EXAMPLE'
items=()
collect "${items[@]}"
EXAMPLE
# no-mistakes-disable-next-line bash-empty-array-nounset -- Conditional population is guaranteed by the caller.
collect "${items[@]}"
collect "${items[@]}" # no-mistakes-disable-line bash-empty-array-nounset -- Conditional population is guaranteed.
