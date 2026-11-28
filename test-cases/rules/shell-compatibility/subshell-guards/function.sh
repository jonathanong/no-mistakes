#!/bin/bash
set -u
read -ra files <<< "$1"
check_files() (
  case "$2" in
    strict)
      [[ ${#files[@]} -eq 0 ]] && exit 1
      ;;
    (checked|review) : ;;
  esac
)
check_files "$@"
printf '%s\n' "${files[@]}"
