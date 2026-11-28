#!/bin/bash
set -u
items=()
load() { items=(a b); }
load
printf '%s\n' "${items[@]}"

flags=(--base)
add_flag() { flags+=(--extra); }
add_flag
printf '%s\n' "${flags[@]}"
