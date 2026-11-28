#!/bin/bash
set -u

ok=(seed)
a=(1)
b=(1)
c=(1)
d=(1)
e=(1)
f=(1)
while next; do
  use "${ok[@]}"
  ok=()
  ok+=(refilled)
  use "${a[@]}"
  a=("${b[@]}")
  b=("${c[@]}")
  c=("${d[@]}")
  d=("${e[@]}")
  e=("${f[@]}")
  f=()
done
use "${ok[@]}"
