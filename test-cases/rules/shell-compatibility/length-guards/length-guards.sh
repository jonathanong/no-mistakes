#!/bin/bash
set -u
unknown=()
read -ra bracket <<< "$1"
if [[ ${#bracket[@]} -eq 0 ]]; then
  echo "Expected items" >&2
  exit 1
fi
collect "${bracket[@]}" "${unknown[@]}"

read -ra limit <<< "$1"
if [[ ${#limit[@]} -lt 1 ]]; then exit; fi
collect "${limit[@]}" "${unknown[@]}"

function show() {
  local -a single
  read -ra single <<< "$1"
  if [ "${#single[@]}" -eq 0 ]; then return 0; fi
  (collect "${single[@]}" "${unknown[@]}")
}

for value in "$@"; do
  read -ra equal <<< "$value"
  if (( ${#equal[@]} == 0 )); then continue; fi
  collect "${equal[@]}" "${unknown[@]}"
done
collect "${equal[@]}"

while next; do
  read -ra negated <<< "$1"
  if (( ! ${#negated[@]} )); then break; fi
  collect "${negated[@]}" "${unknown[@]}"
done
collect "${negated[@]}"

read -ra branch <<< "$1"
[[ ${#branch[@]} -eq 0 ]] && exit 1
collect "${branch[@]}" "${unknown[@]}"

read -ra fallback <<< "$1"
[[ ${#fallback[@]} -ne 0 ]] ||
  exit 1
collect "${fallback[@]}" "${unknown[@]}"

read -ra arithmetic <<< "$1"
(( ${#arithmetic[@]} )) || exit 1
collect "${arithmetic[@]}" "${unknown[@]}"

read -ra inverted <<< "$1"
! [[ ${#inverted[@]} -eq 0 ]] || exit 1
collect "${inverted[@]}" "${unknown[@]}"

literal=(ready)
collect "${literal[@]}" "${unknown[@]}"
for value in "$@"; do
  collect "${literal[@]}" "${unknown[@]}"
done
copy=("${literal[@]}")
guarded_copy=("${bracket[@]}")
empty_copy=("${unknown[@]}")
collect "${copy[@]}" "${guarded_copy[@]}" "${empty_copy[@]}"
copy=()
collect "${copy[@]}"
unset 'guarded_copy[0]'
collect "${guarded_copy[@]}"
read -ra bracket <<< "$1"
collect "${bracket[@]}"

read -ra inline <<< "$1"
((!${#inline[@]})) && exit
collect "${inline[@]}" "${unknown[@]}"

read -ra positive <<< "$1"
[[ ${#positive[@]} -gt 0 ]] || exit
collect "${positive[@]}" "${unknown[@]}"

read -ra alternative <<< "$1"
if [ "${#alternative[@]}" -lt 1 ]; then exit; else :; fi
collect "${alternative[@]}" "${unknown[@]}"

read -ra logged <<< "$1"
(( ${#logged[@]} )) || { echo "Expected items" >&2; exit 1; }
collect "${logged[@]}" "${unknown[@]}"

read -ra block <<< "$1"
if [[ ${#block[@]} -eq 0 ]]; then { echo "Expected items" >&2; exit 1; }; fi
collect "${block[@]}" "${unknown[@]}"

stored=()
{ stored=(ready); }
collect "${stored[@]}" "${unknown[@]}"

for outer in "$@"; do
  collect "${stored[@]}" "${unknown[@]}"
  (for inner in "$@"; do stored=(); done)
  (unset stored)
done
collect "${stored[@]}" "${unknown[@]}"

clear_stored() { stored=(); }
collect "${stored[@]}" "${unknown[@]}"
clear_isolated() ( stored=() )
clear_isolated
collect "${stored[@]}" "${unknown[@]}"
clear_stored
collect "${stored[@]}"

read -ra joined <<< "$1"
if [[ ${#joined[*]} -eq 0 ]]; then exit; fi
collect "${joined[*]}" "${unknown[@]}"
