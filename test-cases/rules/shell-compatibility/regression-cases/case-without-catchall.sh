#!/bin/bash
set -u
partial=()
case "${1:-}" in
  x) partial=(one) ;;
  y) partial=(two) ;;
esac
printf '%s\n' "${partial[@]}"
