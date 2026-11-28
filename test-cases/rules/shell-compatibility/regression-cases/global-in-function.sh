#!/bin/bash
set -u
base=(one two)
g=("${base[@]}" extra)
use_g() { printf '%s\n' "${g[@]}"; }
use_g
