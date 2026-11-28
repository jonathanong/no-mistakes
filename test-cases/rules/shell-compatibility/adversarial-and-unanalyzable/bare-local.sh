#!/bin/bash
set -u

items=()
fill_local() {
  local items
  items=(alpha beta)
}
fill_local
printf '%s\n' "${items[@]}"

present=(seed)
shadow_bare_local() {
  local present
  printf '%s\n' "${present[@]}"
}
shadow_bare_local
