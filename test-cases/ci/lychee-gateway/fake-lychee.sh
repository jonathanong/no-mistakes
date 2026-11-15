#!/bin/bash
set -euo pipefail

printf '%s\n' "$*" >> "${LYCHEE_FAKE_ARGV:?}"

output=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --output)
      output="${2:?}"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [[ -z "$output" ]]; then
  echo "fake lychee expected --output" >&2
  exit 1
fi

mkdir -p "$(dirname "$output")"
count_file="${LYCHEE_FAKE_COUNT:?}"
count=0
if [[ -f "$count_file" ]]; then
  count="$(cat "$count_file")"
fi
count="$((count + 1))"
printf '%s\n' "$count" > "$count_file"

if [[ "$count" -ge "${LYCHEE_FAKE_SUCCEED_ON:?}" ]]; then
  cat "${LYCHEE_FAKE_SUCCESS:?}" > "$output"
  exit 0
fi

cat "${LYCHEE_FAKE_FAILURE:?}" > "$output"
exit 2
