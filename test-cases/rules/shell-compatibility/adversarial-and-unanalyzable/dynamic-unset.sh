#!/bin/bash
set -u

tracked=(alpha)
target=tracked
unset "$target"
printf '%s\n' "${tracked[@]}"

refilled=(beta)
unset "$target"
refilled=(gamma)
printf '%s\n' "${refilled[@]}"

guarded=(delta)
unset "$target"
if [[ ${#guarded[@]} -eq 0 ]]; then
  exit 1
fi
printf '%s\n' "${guarded[@]}"

helper_target=(epsilon)
clear_dynamic() {
  unset "$1"
}
clear_dynamic helper_target
printf '%s\n' "${helper_target[@]}"

subshell_safe=(zeta)
( unset "$target" )
printf '%s\n' "${subshell_safe[@]}"
