#!/usr/bin/env bash
set -u

prepare() {
  (return 0)
  items=(ready)
}
items=()
prepare
printf '%s\n' "${items[@]}"
