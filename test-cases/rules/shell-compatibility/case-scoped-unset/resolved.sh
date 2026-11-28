#!/bin/bash
set -u

case "${1:-}" in
  fast) mode=(fast) ;;
  full) mode=(full) ;;
  *) exit 1 ;;
esac

show_mode() {
  printf '%s\n' "${mode[@]}"
}

alpha=(seed)

while IFS= read -r name; do
  normalized="${name,,}"
  case "$normalized" in
    alpha|beta|gamma_dir)
      unset "$name"
      ;;
  esac
done < names.txt

printf '%s\n' "${mode[@]}"
show_mode
printf '%s\n' "${alpha[@]}"
