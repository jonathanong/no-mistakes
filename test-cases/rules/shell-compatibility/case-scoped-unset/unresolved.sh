#!/bin/bash
set -u

kept_glob=(seed)
while IFS= read -r name; do
  normalized="${name,,}"
  case "$normalized" in
    alpha|*)
      unset "$name"
      ;;
  esac
done < names.txt
printf '%s\n' "${kept_glob[@]}"

kept_unrelated=(seed)
case "${1:-}" in
  clean)
    unset "$target"
    ;;
esac
printf '%s\n' "${kept_unrelated[@]}"

kept_suffix=(seed)
derived="${target}_suffix"
case "$derived" in
  kept_suffix_suffix)
    unset "$target"
    ;;
esac
printf '%s\n' "${kept_suffix[@]}"
