#!/bin/bash
set -u

items=()
case "${1:-}" in
  *) items=(one) ;&
  extra) printf '%s\n' "${items[@]}" ;;
esac
printf '%s\n' "${items[@]}"
