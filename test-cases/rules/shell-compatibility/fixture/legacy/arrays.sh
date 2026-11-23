#!/usr/bin/env bash
items=()
collect "${items[@]}"
set -euo pipefail
collect "${items[@]}"
collect "${items[*]}"
collect <<< "${items[@]}"
collect ${items[@]+"${items[@]}"}
collect ${items[*]+"${items[*]}"}
collect "${items[@]+"${items[@]}"}"
collect '${items[@]}' "\${items[@]}" ${items[@]}
# collect "${items[@]}"
collect "${other[@]}"
collect "${label:-café}" ${items[@]+"${items[@]}"}
items=('' stable)
collect "${items[@]}"
items=([0]=stable)
collect "${items[@]}"
items=($'it\'s stable')
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
declare -x -a -- declared
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
cat <<''
collect "${items[@]}"

# no-mistakes-disable-next-line bash-empty-array-nounset -- Conditional population is guaranteed by the caller.
collect "${items[@]}"
collect "${items[@]}" # no-mistakes-disable-line bash-empty-array-nounset -- Conditional population is guaranteed.
declare -a literal=stable
collect "${literal[@]}"

slots=()
slots[0]=stable
collect "${slots[@]}"
slots=()
slots[1]+=stable
collect "${slots[@]}"
slots=()
slots=stable
collect "${slots[@]}"
value=
slots=()
slots=$value
collect "${slots[@]}"
declare -a unquoted=$value quoted="$value"
collect "${unquoted[@]}" "${quoted[@]}"

: {{FORM_FEED}}
slots=()
slots=()
collect "${slots[@]}"
collect slots[0]=stable slots[1]+=stable
collect "${slots[@]}"
slots=stable collect example
collect "${slots[@]}"

items=(\
  stable <(collect example))
collect "${items[@]}"
collect "${label:-escaped\}\
brace}"
items=()
set -- +u
collect "${items[@]}"

items=("[*]" *.sample)
collect "${items[@]}"

single=()
single[0]=stable
collect "${single[@]}"
unset 'single[0]'
collect "${single[@]}"
single[0]=stable
unset single[0]
collect "${single[@]}"
single[0]=stable
unset -f single
collect "${single[@]}"
unset "single[@]"
collect "${single[@]}"
single[0]=stable
unset -v -- 'single[*]'
collect "${single[@]}"
