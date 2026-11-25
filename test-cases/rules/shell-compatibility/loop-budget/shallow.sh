#!/bin/bash
set -u
saved=(sample)
while more; do
  while available; do
    saved=()
    saved+=(sample)
  done
  collect "${saved[@]}"
done
collect "${saved[@]}"
empty=(sample)
while more; do
  collect "${empty[@]}"
  empty=()
done
collect "${empty[@]}"
