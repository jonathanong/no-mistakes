#!/bin/bash
set -u
read -ra items <<< "$1"
if [[ ${#items[@]} -eq 0 ]]; then echo "Expected items"; fi
collect "${items[@]}"

other=()
if [[ ${#items[@]} -lt 1 ]]; then exit; fi
collect "${items[@]}" "${other[@]}"
items=()
collect "${items[@]}"

check_items() {
  collect }
  [[ ${#items[@]} -eq 0 ]] && return
  collect "${items[@]}"
}
collect "${items[@]}"

read -ra values <<< "$1"
if [[ ${#values[@]} -eq 0 ]]; then enabled && exit; fi
collect "${values[@]}"
[[ ${#values[@]} -eq 0 ]] || exit
collect "${values[@]}"

read -ra optional <<< "$1"
if [[ ${#optional[@]} -eq 0 ]] && enabled; then exit; fi
collect "${optional[@]}"
[[ ${#optional[@]} -eq 0 ]] && exit | collect
collect "${optional[@]}"
([[ ${#optional[@]} -eq 0 ]] && exit)
collect "${optional[@]}"

read -ra nested <<< "$1"
if enabled; then
  [[ ${#nested[@]} -eq 0 ]] && exit
  collect "${nested[@]}"
else
  collect "${nested[@]}"
fi
collect "${nested[@]}"

read -ra reset <<< "$1"
[[ ${#reset[@]} -eq 0 ]] && exit
if enabled; then unset reset; fi
collect "${reset[@]}"

read -ra in_loop <<< "$1"
while next; do
  if [[ ${#in_loop[@]} -eq 0 ]]; then
    continue
  fi
  collect "${in_loop[@]}"
  unset in_loop
done
collect "${in_loop[@]}"

read -ra redirected <<< "$1"
if [[ ${#redirected[@]} -eq 0 ]] < missing.txt; then exit; fi
[[ ${#redirected[@]} -eq 0 ]] && exit < missing.txt
collect "${redirected[@]}"

read -ra grouped <<< "$1"
if [[ ${#grouped[@]} -eq 0 ]]; then
  { enabled; } && exit
fi
collect "${grouped[@]}"

read -ra scoped <<< "$1"
check_scoped() (
  [ "${#scoped[@]}" -eq 0 ] && return
  collect "${scoped[@]}"
)
collect "${scoped[@]}"

literal=(ready)
for value in "$@"; do
  while next; do
    collect "${literal[@]}"
    unset literal
  done
done
collect "${literal[@]}"

read -ra mode_items <<< "$1"
case "$1" in
  show)
    if [[ ${#mode_items[@]} -eq 0 ]]; then exit; fi
    collect "${mode_items[@]}"
    ;;
  *)
    collect "${mode_items[@]}"
    ;;
esac
collect "${mode_items[@]}"

read -ra selected <<< "$1"
if [[ ${#selected[@]} -eq 0 ]]; then exit; fi
selected=("${items[@]}")
collect "${selected[@]}"

read -ra compound <<< "$1"
if [[
  ${#compound[@]} -eq 0 && $2 == yes
]]; then exit; fi
collect "${compound[@]}"
if [[ ${#compound[@]} -eq 0 ]]; (enabled); then exit; fi
collect "${compound[@]}"
if [[ ${#compound[@]} -eq 0 ]]; then collect example; elif enabled; then exit; fi
collect "${compound[@]}"

read -ra piped <<< "$1"
(( ${#piped[@]} )) || { echo "Expected items" >&2; exit 1; } | collect
collect "${piped[@]}"

skipped=()
enabled && { skipped=(ready); }
collect "${skipped[@]}"

read -ra skipped_test <<< "$1"
enabled && if [[ ${#skipped_test[@]} -eq 0 ]]; then exit; fi
collect "${skipped_test[@]}"

read -ra exact_count <<< "$1"
[[ ${#exact_count[@]} -eq 1 ]] && exit
collect "${exact_count[@]}"
