#!/bin/bash
set -u
x=()
while read -r l; do
  x+=("$l")
done
use "${x[@]}"

y=()
if c; then
  y=(a)
fi
use "${y[@]}"

arr=()
for f in a b; do
  [[ c ]] && continue
  arr+=("$f")
done
cmd "${arr[@]}"

nested=()
for f in a b; do
  for g in x y; do
    [[ c ]] && break 2
  done
  nested+=("$f")
done
cmd "${nested[@]}"
