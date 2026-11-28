#!/bin/bash
set -u

nested_items=()
if [[ ${#nested_items[@]} -eq 0 ]]; then
  ((echo empty); exit 1)
fi
printf '%s\n' "${nested_items[@]}"

timed_items=()
if [[ ${#timed_items[@]} -eq 0 ]]; then
  time ( :; exit 1 )
fi
printf '%s\n' "${timed_items[@]}"
