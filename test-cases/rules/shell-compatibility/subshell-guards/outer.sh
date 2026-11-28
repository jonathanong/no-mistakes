#!/bin/bash
set -u
read -ra files <<< "$1"
count=$((1 + 1))
[[ ${#files[@]} -eq 0 ]] && exit 1
(case "$2" in skip) : ;;& strict) (printf '%s\n' "${files[@]}"); esac)
printf '%s\n' "${files[@]}"
