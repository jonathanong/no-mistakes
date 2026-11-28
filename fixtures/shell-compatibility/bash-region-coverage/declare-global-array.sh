#!/usr/bin/env bash
set -u

show_items() {
  declare -ga global_items
  printf '%s\n' "${global_items[@]}"
}
show_items
