#!/bin/bash
set -u

plain_filled=(seed)
for ((i = 0; i < 3; i++)); do
  plain_filled=()
  plain_filled+=(item)
done
printf '%s\n' "${plain_filled[@]}"

plain_empty=()
for ((i = 0; i < n; i++)); do
  plain_empty+=(item)
done
printf '%s\n' "${plain_empty[@]}"

nested_empty=()
while read -r line; do
  for ((i = 0; i < 3; i++)); do
    :
  done
  nested_empty+=("$line")
done
printf '%s\n' "${nested_empty[@]}"

nested_kept=(seed)
while read -r line; do
  for ((i = 0; i < 3; i++)); do
    :
  done
  nested_kept+=("$line")
done
printf '%s\n' "${nested_kept[@]}"

idx=0
corr=()
if (( idx == 0 )); then
  corr=(ready)
fi
for ((idx = 0; idx < 2; idx++)) do
  :
done
if (( idx == 0 )); then
  printf '%s\n' "${corr[@]}"
fi
