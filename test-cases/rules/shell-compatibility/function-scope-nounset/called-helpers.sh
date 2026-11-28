#!/bin/bash
set -u
disable_nounset() {
  set +u
}
enable_nounset() {
  set -u
}
disable_nounset
disabled_items=()
printf '%s\n' "${disabled_items[@]}"

enable_nounset
enabled_items=()
printf '%s\n' "${enabled_items[@]}"
