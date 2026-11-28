use super::analyze;

mod coverage;
mod edge;
mod review;
mod review_round;
mod review_round_2;
mod review_round_3;

pub(super) fn scan(source: &str) -> Vec<(&str, usize)> {
    analyze(source).findings
}

#[test]
fn case_exhaustive_and_fallthrough_semantics() {
    let script = r#"#!/bin/bash
set -u
case "${1:-}" in
  (a|b) f=(one) ;;
  (c|*) f=(two) ;;
esac
printf '%s\n' "${f[@]}"

partial=()
case "${1:-}" in
  x) partial=(one) ;;
  y) partial=(two) ;;
esac
printf '%s\n' "${partial[@]}"

fall=()
case "${1:-}" in
  a) : ;&
  b) fall=(one) ;;
  *) exit 1 ;;
esac
printf '%s\n' "${fall[@]}"

fall_leave=()
case "${1:-}" in
  a) exit 1 ;&
  b) fall_leave=(one) ;;
  *) exit 1 ;;
esac
printf '%s\n' "${fall_leave[@]}"

test_next=()
case "${1:-}" in
  a) test_next=(one) ;;&
  b) test_next=() ;;
  *) exit 1 ;;
esac
printf '%s\n' "${test_next[@]}"

no_semi=()
case "${1:-}" in
  a) no_semi=(one) ;;
  *) no_semi=(two)
esac
printf '%s\n' "${no_semi[@]}"

trailing_fall=()
case "${1:-}" in
  a) trailing_fall=(one) ;&
esac
printf '%s\n' "${trailing_fall[@]}"

exit_all=()
if enabled; then
  case "${1:-}" in
    *) exit 2 ;;
  esac
fi
printf '%s\n' "${exit_all[@]}"
"#;
    assert_eq!(
        scan(script),
        vec![
            ("partial", 14),
            ("test_next", 38),
            ("trailing_fall", 51),
            ("exit_all", 59),
        ]
    );
}

#[test]
fn case_optional_and_all_exit_in_branch() {
    let script = r#"#!/bin/bash
set -u
opt=()
enabled && case "${1:-}" in
  *) opt=(one) ;;
esac
printf '%s\n' "${opt[@]}"

guarded=()
if [[ ${#guarded[@]} -eq 0 ]]; then
  case "${1:-}" in
    a) exit 1 ;;
    *) exit 2 ;;
  esac
fi
printf '%s\n' "${guarded[@]}"

ext=()
case "${1:-}" in
  (@(a|b)) ext=(one) ;;
  *) ext=(two) ;;
esac
printf '%s\n' "${ext[@]}"
"#;
    assert_eq!(scan(script), vec![("opt", 7)]);
}

#[test]
fn global_nonempty_facts_in_functions_and_invalidation() {
    let script = r#"#!/bin/bash
set -u
base=(one two)
g=("${base[@]}" extra)
use_g() { printf '%s\n' "${g[@]}"; }
use_g

later=(one)
use_later() { printf '%s\n' "${later[@]}"; }
later=()
use_later

fn_emptied=(one)
clear_fn() { fn_emptied=(); }
use_fn_emptied() { printf '%s\n' "${fn_emptied[@]}"; }
use_fn_emptied

shadow=(one)
shadow_local() {
  local -a shadow
  printf '%s\n' "${shadow[@]}"
}
use_shadow() { printf '%s\n' "${shadow[@]}"; }

appended=(one)
use_appended() { printf '%s\n' "${appended[@]}"; }
appended+=("${unknown[@]}")
use_appended

local_init=(one)
other_local() {
  local -a local_init=(two)
  printf '%s\n' "${local_init[@]}"
}
use_local_init() { printf '%s\n' "${local_init[@]}"; }
"#;
    assert_eq!(
        scan(script),
        vec![("later", 9), ("fn_emptied", 15), ("shadow", 21),]
    );
}

#[test]
fn same_condition_correlation_and_invalidation() {
    let script = r#"#!/bin/bash
set -u
MODE="${1:-}"
cmd=()
if [[ "$MODE" == exec ]] && [[ -n "${TARGET:-}" ]]; then
  cmd=(run "$TARGET")
fi
if [[ "$MODE" == exec ]] && [[ -n "${TARGET:-}" ]]; then
  printf '%s\n' "${cmd[@]}"
fi

step=()
if [[ "$MODE" == exec ]]; then
  step=(run)
fi
if [[ "$MODE" == exec ]] && [[ -n "${TARGET:-}" ]]; then
  printf '%s\n' "${step[@]}"
fi

changed=()
if [[ "$MODE" == exec ]]; then
  changed=(run)
fi
MODE=other
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${changed[@]}"
fi

mutated_in_else=()
if [[ "$MODE" == exec ]]; then
  mutated_in_else=(run)
else
  MODE=exec
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${mutated_in_else[@]}"
fi

read_in_between=()
if [[ -v FLAG ]]; then
  read_in_between=(ok)
fi
read -r FLAG <<< "x"
if [[ -v FLAG ]]; then
  printf '%s\n' "${read_in_between[@]}"
fi

arith=()
if (( count > 0 )); then
  arith=(ok)
fi
if (( count > 0 )); then
  printf '%s\n' "${arith[@]}"
fi
count=0
if (( count > 0 )); then
  printf '%s\n' "${arith[@]}"
fi
"#;
    let res = analyze(script);
    assert!(res.findings.is_empty());
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![26, 36, 45, 57]);
}

#[test]
fn condition_elif_else_short_circuit_and_nested_sentinels() {
    let script = r#"#!/bin/bash
set -u
a=()
if [[ "$MODE" == dry ]]; then
  :
else
  a=(run)
fi
if [[ "$MODE" == dry ]]; then
  :
else
  printf '%s\n' "${a[@]}"
fi

b=()
if [[ "$MODE" == dry ]]; then
  :
elif [[ "$MODE" == exec ]]; then
  b=(run)
fi
if [[ "$MODE" == dry ]]; then
  :
elif [[ "$MODE" == exec ]]; then
  printf '%s\n' "${b[@]}"
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${b[@]}"
fi

c=()
if [[ "$MODE" == dry ]]; then
  exit 1
elif [[ "$MODE" == exec ]]; then
  c=(run)
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${c[@]}"
fi

d=()
[[ "$MODE" == 'exec' ]] && d=(run)
[[ "$MODE" == 'exec' ]] && printf '%s\n' "${d[@]}"

e=()
if [[ "$MODE" == exec ]]; then
  if [[ "$MODE" == exec ]]; then
    e=(run)
  fi
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${e[@]}"
fi

sub=()
if $(check_mode); then
  sub=(run)
elif [[ "$MODE" == exec ]]; then
  sub=(run)
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${sub[@]}"
fi

or_cond=()
if [[ "$A" == 1 ]] || [[ "$B" == 2 ]]; then
  or_cond=(run)
fi
if [[ "$A" == 1 ]] || [[ "$B" == 2 ]]; then
  printf '%s\n' "${or_cond[@]}"
fi

redir_cond=()
if [[ "$MODE" == exec ]] >/dev/null; then
  redir_cond=(run)
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${redir_cond[@]}"
fi

escaped=()
if [[ "$MODE" == \exec ]]; then
  escaped=(run)
fi
if [[ "$MODE" == \exec ]]; then
  printf '%s\n' "${escaped[@]}"
fi
"#;
    let res = analyze(script);
    assert!(res.findings.is_empty());
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![27, 61, 77]);
}

#[test]
fn conditional_facts_in_functions_and_merges() {
    let script = r#"#!/bin/bash
set -u
keep=()
if [[ "$MODE" == exec ]]; then
  keep=(run)
fi
use_keep() {
  if [[ "$MODE" == exec ]]; then
    printf '%s\n' "${keep[@]}"
  fi
}

drop_var=()
if [[ "$TAG" == prod ]]; then
  drop_var=(run)
fi
use_drop_var() {
  if [[ "$TAG" == prod ]]; then
    printf '%s\n' "${drop_var[@]}"
  fi
}
TAG=dev

drop_arr=()
if [[ "$MODE" == exec ]]; then
  drop_arr=(run)
fi
use_drop_arr() {
  if [[ "$MODE" == exec ]]; then
    printf '%s\n' "${drop_arr[@]}"
  fi
}
drop_arr=()

merged=()
if [[ "$A" == 1 ]]; then
  merged=(one)
else
  if [[ "$MODE" == exec ]]; then
    merged=(two)
  fi
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${merged[@]}"
fi

merged_rev=()
if [[ "$A" == 1 ]]; then
  if [[ "$MODE" == exec ]]; then
    merged_rev=(two)
  fi
else
  merged_rev=(one)
fi
if [[ "$MODE" == exec ]]; then
  printf '%s\n' "${merged_rev[@]}"
fi

loop_var=()
if [[ "$item" == a ]]; then
  loop_var=(one)
fi
for item in a b; do
  if [[ "$item" == a ]]; then
    printf '%s\n' "${loop_var[@]}"
  fi
done
"#;
    let res = analyze(script);
    assert!(res.findings.is_empty());
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![19, 30, 65]);
}

#[test]
fn loops_functions_and_exiting_helpers_preserve_nonempty_facts() {
    let script = r#"#!/bin/bash
set -u
args=(--x)
for f in "$@"; do
  args+=("$f")
done
cmd "${args[@]}"

lit=()
for f in alpha beta; do
  lit+=("$f")
done
cmd "${lit[@]}"

broke=()
for f in alpha beta; do
  if c; then break; fi
  broke+=("$f")
done
cmd "${broke[@]}"

items=()
load() { items=(a b); }
load
printf '%s\n' "${items[@]}"

maybe_load() {
  if c; then return 1; fi
  maybe_items=(a b)
}
maybe_items=()
maybe_load
printf '%s\n' "${maybe_items[@]}"

both_returns() {
  if c; then
    both_items=(a)
    return 0
  fi
  both_items=(b)
  return 0
}
both_items=()
both_returns
printf '%s\n' "${both_items[@]}"

local_load() { local -a loc_items=(a); }
loc_items=()
local_load
printf '%s\n' "${loc_items[@]}"

die_branch() {
  if c; then exit 1; else exit 2; fi
}
y=()
if c; then y=(a); else die_branch; fi
printf '%s\n' "${y[@]}"

die_case() {
  case "$1" in
    a) exit 1 ;;
    *) exit 2 ;;
  esac
}
z=()
if c; then z=(a); else die_case "$1"; fi
printf '%s\n' "${z[@]}"

kept=(one)
for f in "$@"; do
  declare -a kept
  mapfile -O 1 kept < "$f"
done
printf '%s\n' "${kept[@]}"

undecl_case_fn() {
  case "$1" in
    a) u=(one) ;;
    b) u=(two) ;;
  esac
  printf '%s\n' "${u[@]}"
}
"#;
    assert_eq!(
        scan(script),
        vec![("broke", 20), ("maybe_items", 33), ("loc_items", 50)]
    );
}
