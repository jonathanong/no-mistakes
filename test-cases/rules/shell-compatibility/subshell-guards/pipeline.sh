#!/bin/bash
set -u
read -ra files <<< "$1"
printf '%s\n' ready | { [[ ${#files[@]} -eq 0 ]] && exit 1; }
printf '%s\n' "${files[@]}"
