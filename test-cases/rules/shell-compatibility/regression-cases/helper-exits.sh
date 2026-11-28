#!/bin/bash
set -u
if c; then x=(a); else die; fi
printf '%s\n' "${x[@]}"

fail_fast() {
  echo "$*" >&2
  exit 1
}
y=()
if c; then y=(a); else fail_fast "missing"; fi
printf '%s\n' "${y[@]}"
