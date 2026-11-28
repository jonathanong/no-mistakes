#!/bin/bash
set -u
x=()
while :; do
  x=(a)
  break
done
use "${x[@]}"

y=()
while true; do
  if c; then
    y=(a)
    break
  fi
done
use "${y[@]}"
