#!/bin/bash
set -u

items=(alpha)
shadow_local() {
  local -a items
  printf '%s\n' "${items[@]}"
}
shadow_local

queued=(beta)
on_exit() {
  printf '%s\n' "${queued[@]}"
}
queued=()
