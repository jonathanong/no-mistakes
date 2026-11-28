#!/bin/bash
set -u

m=
a=()
if [[ $1 == x ]]; then
  m=x
  a=(one)
fi
if [[ $m == x ]]; then
  printf '%s\n' "${a[@]}"
fi

unguarded=()
if [[ $1 == x ]]; then
  unguarded=(one)
fi
printf '%s\n' "${unguarded[@]}"

never_filled=()
if [[ $1 == x ]]; then
  printf '%s\n' "${never_filled[@]}"
fi

if [[ $1 == x ]]; then
  inner_reset=()
  if [[ $2 == y ]]; then
    inner_reset=(one)
  fi
  printf '%s\n' "${inner_reset[@]}"
fi

checked_empty=()
if [[ $1 == x ]]; then
  checked_empty=(one)
fi
if [[ ${#checked_empty[@]} -eq 0 ]]; then
  printf '%s\n' "${checked_empty[@]}"
fi
