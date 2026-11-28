#!/bin/bash
set -u

files=(alpha beta)
show() {
  printf '%s\n' "${files[@]}"
}
( files=() )
show
