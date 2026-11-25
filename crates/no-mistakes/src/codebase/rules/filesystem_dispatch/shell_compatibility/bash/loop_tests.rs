use super::{analyze, tests::scan};

#[test]
fn loop_fixpoint_reset_refill_and_early_continue_break() {
    let script = r#"#!/bin/bash
set -u
args=(--x)
for f in "$@"; do
  args=()
  args+=("$f")
done
use "${args[@]}"

items=(--x)
for f in "$@"; do
  use "${items[@]}"
  items=()
  items+=(--y)
done

cont_empty=(--x)
for f in "$@"; do
  use "${cont_empty[@]}"
  cont_empty=()
  if c; then continue; fi
  cont_empty+=(--y)
done

brk_empty=(--x)
for f in "$@"; do
  brk_empty=()
  if c; then break; fi
  brk_empty+=(--y)
done
use "${brk_empty[@]}"

cont_after=()
for f in a b; do
  cont_after+=("$f")
  [[ c ]] && continue
done
use "${cont_after[@]}"

cont_outer=()
for f in a b; do
  for g in x y; do
    [[ c ]] && continue 2
  done
  cont_outer+=("$f")
done
use "${cont_outer[@]}"
"#;
    assert_eq!(
        scan(script),
        vec![("cont_empty", 19), ("brk_empty", 31), ("cont_outer", 47)]
    );
}

#[test]
fn infinite_loops_and_multi_pass_chains() {
    let script = r#"#!/bin/bash
set -u
u=()
until false; do
  u=(a)
  break
done
use "${u[@]}"

multi=()
while :; check; do
  multi=(a)
  break
done
use "${multi[@]}"

cond_inf=()
if c; then
  while :; do
    :
  done
else
  cond_inf=(a)
fi
use "${cond_inf[@]}"

opt_inf=()
enabled && while :; do
  opt_inf=(a)
  break
done
use "${opt_inf[@]}"

a=(1)
b=(1)
c_arr=(1)
d_arr=(1)
e_arr=(1)
while next; do
  use "${a[@]}"
  a=("${b[@]}")
  b=("${c_arr[@]}")
  c_arr=("${d_arr[@]}")
  d_arr=("${e_arr[@]}")
  e_arr=()
done

cond_loop=()
if [[ "$MODE" == exec ]]; then
  cond_loop=(a)
fi
for f in "$@"; do
  if [[ "$MODE" == exec ]]; then
    use "${cond_loop[@]}"
  fi
  MODE=other
done

always_leave=()
if c; then
  for f in a b; do
    exit 1
  done
else
  always_leave=(a)
fi
use "${always_leave[@]}"

brk_bad=()
while :; do
  if c; then break; fi
  brk_bad=(a)
  break
done
use "${brk_bad[@]}"
"#;
    let res = analyze(script);
    assert_eq!(
        res.findings,
        vec![
            ("multi", 15),
            ("opt_inf", 32),
            ("a", 40),
            ("b", 41),
            ("c_arr", 42),
            ("d_arr", 43),
            ("e_arr", 44),
            ("brk_bad", 75),
        ]
    );
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![54]);
}

#[test]
fn nested_loop_budget_reports_uncertainty_and_keeps_weakened_expansions() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/shell-compatibility/loop-budget/deep.sh"
    ));
    let result = super::analyze(source);
    for expected in [72, 82] {
        assert!(result.unanalyzable.iter().any(|(line, detail)| {
            *line == expected && detail.contains("file-wide Bash loop replay budget exhausted")
        }));
    }
    assert!(result.findings.contains(&("level_4_third", 55)));
    assert!(result.findings.contains(&("final", 76)));
    assert!(result.findings.contains(&("later", 83)));
}
