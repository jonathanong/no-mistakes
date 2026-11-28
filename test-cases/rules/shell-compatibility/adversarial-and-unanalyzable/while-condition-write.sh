#!/bin/bash
set -u

items=(seed)
while read -ra items; do
  items=(refilled)
done
printf '%s\n' "${items[@]}"
