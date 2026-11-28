#!/bin/bash
set -u
read -ra files <<< "$1"
output=$(case "$2" in
  strict)
    [[ ${#files[@]} -eq 0 ]] && exit 1
    ;;
esac)
printf '%s\n' "$output" "${files[@]}"
