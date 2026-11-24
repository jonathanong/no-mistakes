#!/usr/bin/env bash
set -u
selected=$1
root=$2
candidates=("$selected" "$root"/Tool*.app)
for c in "${candidates[@]}"; do echo "$c"; done
globs=("$root"/Tool*.app)
for g in "${globs[@]}"; do echo "$g"; done
unquoted=($selected)
for u in "${unquoted[@]}"; do echo "$u"; done
args=("$@")
for a in "${args[@]}"; do echo "$a"; done
IFS=',' read -ra requested <<< "$1"
for r in "${requested[@]}"; do echo "$r"; done
requested=(stable)
for r in "${requested[@]}"; do echo "$r"; done
read -r -a pieces <<< "$1"
for p in "${pieces[@]}"; do echo "$p"; done
mapfile -t lines < "$1"
for l in "${lines[@]}"; do echo "$l"; done
readarray -t -O 1 more < "$1"
for m in "${more[@]}"; do echo "$m"; done
IFS=',' read -r first _ <<< "$1"
echo "$first"
