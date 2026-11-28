#!/usr/bin/env bash
set -u
items=()
fill_items() { items=(ready); }
fill_items > ./missing/output.txt
printf '%s\n' "${items[@]}"
fill_items
printf '%s\n' "${items[@]}"
