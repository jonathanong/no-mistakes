#!/bin/bash
set -u
files=(a b)
show() { printf '%s\n' "${files[@]}"; }
show
files=()
