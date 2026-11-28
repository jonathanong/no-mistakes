#!/bin/bash
set -u
read -ra files <<< "$1"
(
  case "$2" in
    strict)
      [[ ${#files[@]} -eq 0 ]] && exit 1
      printf '%s\n' "${files[@]}"
      ;;
  esac
)
printf '%s\n' "${files[@]}"
