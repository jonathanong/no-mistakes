#!/bin/bash
set -u

abort_after_subshell() {
  ( local_only=(ok); return 0 )
  exit 3
}
guarded=()
if [[ ${#guarded[@]} -eq 0 ]]; then
  abort_after_subshell
fi
printf '%s\n' "${guarded[@]}"

leak_check() {
  leaked=()
  ( leaked=(ok); return 0 )
  exit 3
}
leaked=()
leak_check
printf '%s\n' "${leaked[@]}"

nounset_subshell() {
  ( set +u; return 0 )
  exit 3
}
nounset_subshell
still_empty=()
printf '%s\n' "${still_empty[@]}"
