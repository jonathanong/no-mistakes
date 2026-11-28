#!/bin/bash
set -u

unknown=(value)
clear_selected() {
  unset "$selected"
}
while next; do
  collect "${unknown[@]}"
  clear_selected
done
collect "${unknown[@]}"

remaining=()
fill_after_subshell() {
  (
    return
  )
  remaining=(value)
}
fill_after_subshell
collect "${remaining[@]}"

guarded=()
while next; do
  (( ! ${#guarded[*]} )) && continue
  collect "${guarded[@]}"
done

flags=()
if [[ "$mode" == ready ]]; then flags=(value); fi
while next; do
  let 'mode=other'
  if [[ "$mode" == ready ]]; then collect "${flags[@]}"; fi
done

remaining=()
if predicate; then
  { marker; }
  remaining=(value)
else
  remaining=()
fi
! case "$mode" in
  *) collect "${remaining[@]}";;
esac

declared=()
reset_declaration() {
  local -a -- "not-an-identifier"
  declared=()
}
reset_declaration
collect "${declared[@]}"

outside=()
while next; do
  ( break )
  collect "${outside[@]}"
done

opaque=()
while next; do
  (( ! count )) && continue
  collect "${opaque[@]}"
done
