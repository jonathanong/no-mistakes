#!/bin/bash
set -u
shopt -s extglob
read -ra files <<< "$1"
(! case "$2" in
  (@(strict|checked))
    case "$3" in
      fi) [[ ${#files[@]} -eq 0 ]] && exit 1
      ;;
    esac
    ;;
  *) case "$3" in esac ;;
esac)
printf '%s\n' "${files[@]}"
