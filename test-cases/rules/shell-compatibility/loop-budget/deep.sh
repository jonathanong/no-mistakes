#!/bin/bash
set -u

level_1_first=(sample)
level_1_second=(sample)
level_1_third=(sample)
while next_1; do
  collect "${level_1_first[@]}"
  level_2_first=(sample)
  level_2_second=(sample)
  level_2_third=(sample)
  while next_2; do
    collect "${level_2_first[@]}"
    level_3_first=(sample)
    level_3_second=(sample)
    level_3_third=(sample)
    while next_3; do
      collect "${level_3_first[@]}"
      level_4_first=(sample)
      level_4_second=(sample)
      level_4_third=(sample)
      while next_4; do
        collect "${level_4_first[@]}"
        level_5_first=(sample)
        level_5_second=(sample)
        level_5_third=(sample)
        while next_5; do
          collect "${level_5_first[@]}"
          level_6_first=(sample)
          level_6_second=(sample)
          level_6_third=(sample)
          while next_6; do
            collect "${level_6_first[@]}"
            level_7_first=(sample)
            level_7_second=(sample)
            level_7_third=(sample)
            while next_7; do
              collect "${level_7_first[@]}"
              level_7_first=("${level_7_second[@]}")
              level_7_second=("${level_7_third[@]}")
              level_7_third=()
            done
            collect "${level_7_first[@]}"
            level_6_first=("${level_6_second[@]}")
            level_6_second=("${level_6_third[@]}")
            level_6_third=()
          done
          collect "${level_6_first[@]}"
          level_5_first=("${level_5_second[@]}")
          level_5_second=("${level_5_third[@]}")
          level_5_third=()
        done
        collect "${level_5_first[@]}"
        level_4_first=("${level_4_second[@]}")
        level_4_second=("${level_4_third[@]}")
        level_4_third=()
      done
      collect "${level_4_first[@]}"
      level_3_first=("${level_3_second[@]}")
      level_3_second=("${level_3_third[@]}")
      level_3_third=()
    done
    collect "${level_3_first[@]}"
    level_2_first=("${level_2_second[@]}")
    level_2_second=("${level_2_third[@]}")
    level_2_third=()
  done
  collect "${level_2_first[@]}"
  level_1_first=("${level_1_second[@]}")
  level_1_second=("${level_1_third[@]}")
  level_1_third=()
done
collect "${level_1_first[@]}"

final=()
collect "${final[@]}"

later=(sample)
while next_later; do
  collect "${later[@]}"
  later=()
done
collect "${later[@]}"
