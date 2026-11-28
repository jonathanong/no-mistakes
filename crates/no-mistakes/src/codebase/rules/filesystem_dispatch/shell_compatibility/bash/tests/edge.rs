use super::super::analyze;

#[test]
fn edge_conditions_cases_and_invalid_jumps() {
    let script = r#"#!/bin/bash
set -u
semi=()
if a; b; c; then
  semi=(one)
fi
use "${semi[@]}"

pipe_if=()
if a | b; then
  pipe_if=(one)
fi
use "${pipe_if[@]}"

star_first=()
case "$1" in
  *|a) star_first=(one) ;;
esac
use "${star_first[@]}"

case_flow_fn() {
  local -a cf=()
  if c; then
    case "$1" in
      *) return 1 ;;
    esac
  else
    cf=(one)
  fi
  use "${cf[@]}"
}

case_no_semi_leave() {
  case "$1" in
    *) exit 1
  esac
}

if_flow_fn() {
  local -a iff=()
  if c; then
    if d; then return 0; else return 1; fi
  else
    iff=(one)
  fi
  use "${iff[@]}"
}

implicit_for=()
for f; do
  implicit_for+=("$f")
done
use "${implicit_for[@]}"

break 0

clear_cond() { cc=(); }
cc=()
if [[ "$M" == 1 ]]; then
  cc=(one)
fi
clear_cond
if [[ "$M" == 1 ]]; then
  use "${cc[@]}"
fi

mutate_dyn() { DYN=2; }
dyn_arr=()
if [[ "$DYN" == 1 ]]; then
  dyn_arr=(one)
fi
read_dyn() {
  if [[ "$DYN" == 1 ]]; then
    use "${dyn_arr[@]}"
  fi
}
"#;
    let res = analyze(script);
    assert_eq!(
        res.findings,
        vec![
            ("semi", 7),
            ("pipe_if", 13),
            ("implicit_for", 53),
            ("cc", 64)
        ]
    );
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![74]);
}

#[test]
fn function_nounset_effects_and_redefinition() {
    let script = r#"#!/bin/bash
set -u
disable_all() {
  if c; then
    set +u
    return 0
  fi
  set +u
}
disable_maybe() {
  if c; then
    set +u
    return 0
  fi
}
disable_subshell() (
  set +u
)

disable_maybe
still_on=()
printf '%s\n' "${still_on[@]}"

disable_subshell
sub_on=()
printf '%s\n' "${sub_on[@]}"

disable_all
now_off=()
printf '%s\n' "${now_off[@]}"

enable_maybe() {
  if c; then
    set -u
    return 0
  fi
}
enable_maybe
back_on=()
printf '%s\n' "${back_on[@]}"

disable_all() {
  :
}
disable_all
after_redef=()
printf '%s\n' "${after_redef[@]}"
"#;
    let result = analyze(script);
    assert_eq!(
        result.findings,
        vec![
            ("still_on", 22),
            ("sub_on", 26),
            ("back_on", 40),
            ("after_redef", 47),
        ]
    );
    assert_eq!(result.unanalyzable.len(), 1);
    assert_eq!(result.unanalyzable[0].0, 42);
}

#[test]
fn arithmetic_writes_let_and_case_dead_arm() {
    let script = r#"#!/bin/bash
set -u
COUNT=1
arr=()
if (( COUNT == 1 )); then
  arr=(ok)
fi
let "COUNT++"
if (( COUNT == 1 )); then
  use "${arr[@]}"
fi

STEP=1
arr2=()
if (( STEP == 1 )); then
  arr2=(ok)
fi
echo "$(( ++STEP ))"
if (( STEP == 1 )); then
  use "${arr2[@]}"
fi

IDX=0
arr3=()
if (( IDX == 0 )); then
  arr3=(ok)
fi
(( arr_idx[0] += 1, IDX |= 1 ))
if (( IDX == 0 )); then
  use "${arr3[@]}"
fi

dead_case=()
case "$1" in
  *) dead_case=(one) ;;
  unreachable) dead_case=() ;;
esac
use "${dead_case[@]}"

let 'COUNT += 1'
(( IDX <<= 1 ))
local_scalar_fn() {
  local scalar=1
  :
}
"#;
    let result = analyze(script);
    assert!(result.findings.is_empty());
    let unan: Vec<usize> = result.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![10, 20, 30]);
}
