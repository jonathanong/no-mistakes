#!/bin/bash
set -u
case "${6:-}" in
  x) f=(one) ;;
  y) f=(two three) ;;
  *) exit 2 ;;
esac
printf '%s\n' "${f[@]}"
