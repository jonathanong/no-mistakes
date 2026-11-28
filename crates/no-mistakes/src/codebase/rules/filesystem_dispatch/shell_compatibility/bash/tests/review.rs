use super::super::analyze;

#[test]
fn unmatched_done_and_dynamic_unset_edge_cases() {
    let unmatched = r#"#!/bin/bash
set -u
items=()
if cond; then
  done
  items=(ok)
fi
printf '%s\n' "${items[@]}"
"#;
    let res = analyze(unmatched);
    assert_eq!(res.findings, vec![("items", 8)]);
    assert_eq!(res.unanalyzable.len(), 1);
    assert_eq!(res.unanalyzable[0].0, 5);

    let dyn_unset = r#"#!/bin/bash
set -u
later=(x)
read_later() {
  printf '%s\n' "${later[@]}"
}
unset -v "$target"
read_later

dyn_flag=(x)
unset "-$flag" other_var
printf '%s\n' "${dyn_flag[@]}"

cmd_sub=(x)
unset $(pick_name)
printf '%s\n' "${cmd_sub[@]}"

static_sq=(x)
unset '$literal'
printf '%s\n' "${static_sq[@]}"

static_idx=(x)
unset "static_idx[$i]"
printf '%s\n' "${static_idx[@]}"
"#;
    let res = analyze(dyn_unset);
    assert_eq!(res.findings, vec![("static_idx", 24)]);
    let lines: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(lines, vec![5, 12, 16]);
}

#[test]
fn review_batch2_edge_cases_and_branch_coverage() {
    assert!(analyze("set -u\n((").findings.is_empty());

    let script = r#"#!/bin/bash
set -u
esc=(x)
unset "\$lit"
printf '%s\n' "${esc[@]}"

bt=(x)
unset "`pick`"
printf '%s\n' "${bt[@]}"

neg_fs=()
if ! [[ -e marker ]]; then neg_fs=(ok); fi
if ! [[ -e marker ]]; then printf '%s\n' "${neg_fs[@]}"; fi

dash_a=()
if [[ -a marker ]]; then dash_a=(ok); fi
if [[ -a marker ]]; then printf '%s\n' "${dash_a[@]}"; fi

redef_inf=()
if [[ ${#redef_inf[@]} -eq 0 ]]; then
  while true; do
    true() { return 1; }
  done
fi
printf '%s\n' "${redef_inf[@]}"

shadow_loop=()
while true; do
  shadow_loop=(ok)
  break
done
printf '%s\n' "${shadow_loop[@]}"

local_after_global=(seed)
touch_before_local() {
  local_after_global=()
  local -a local_after_global
}
touch_before_local
printf '%s\n' "${local_after_global[@]}"

local_before_empty=(seed)
touch_after_local() {
  local -a local_before_empty
  local_before_empty=()
}
touch_after_local
printf '%s\n' "${local_before_empty[@]}"

set +u
late_u=()
for n in a b; do
  printf '%s\n' "${late_u[@]}"
  set -u
done
"#;
    let res = analyze(script);
    assert_eq!(
        res.findings,
        vec![
            ("redef_inf", 25),
            ("shadow_loop", 32),
            ("local_after_global", 40),
            ("late_u", 53),
        ]
    );
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![9, 13, 17]);
}

#[test]
fn extra_branch_coverage_cases() {
    let unclosed_unset = analyze("set -u\na=(x)\nunset \"unclosed\n");
    assert!(unclosed_unset.findings.is_empty());
    let unclosed_read = analyze("set -u\nread -r \"unclosed\n");
    assert!(unclosed_read.findings.is_empty());
    let unclosed_for = analyze("set -u\nfor ((i=0; i<3; i++\n");
    assert!(!unclosed_for.unanalyzable.is_empty());

    let script = r#"#!/bin/bash
set -u
dq_sq=(x)
unset "it's"
printf '%s\n' "${dq_sq[@]}"

bt_cond=()
if `probe`; then bt_cond=(ok); fi
if `probe`; then printf '%s\n' "${bt_cond[@]}"; fi

if !; then :; fi

false() { :; }
until_shadow=()
until false; do
  until_shadow=(ok)
  break
done
printf '%s\n' "${until_shadow[@]}"
"#;
    let res = analyze(script);
    assert_eq!(res.findings, vec![("until_shadow", 19)]);
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![9]);

    let special_params = r#"#!/bin/bash
set -u
braced_q=()
if [[ "${?}" -eq 0 ]]; then braced_q=(x); fi
if [[ "${?}" -eq 0 ]]; then printf '%s\n' "${braced_q[@]}"; fi

braced_hash=()
if [[ "${#}" -gt 0 ]]; then braced_hash=(x); fi
if [[ "${#}" -gt 0 ]]; then printf '%s\n' "${braced_hash[@]}"; fi

sq_dollar=()
if [[ '$?' == "$MODE" ]]; then sq_dollar=(x); fi
if [[ '$?' == "$MODE" ]]; then printf '%s\n' "${sq_dollar[@]}"; fi

esc_dollar=()
if [[ "\$?" == "$MODE" ]]; then esc_dollar=(x); fi
if [[ "\$?" == "$MODE" ]]; then printf '%s\n' "${esc_dollar[@]}"; fi

len_ok=(a)
corr_len=()
if [[ ${#len_ok[@]} -gt 0 ]]; then corr_len=(x); fi
if [[ ${#len_ok[@]} -gt 0 ]]; then printf '%s\n' "${corr_len[@]}"; fi

sec_arr=()
if (( SECONDS > 0 )); then sec_arr=(x); fi
if (( SECONDS > 0 )); then printf '%s\n' "${sec_arr[@]}"; fi
"#;
    let res = analyze(special_params);
    assert!(res.findings.is_empty());
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![5, 9, 26]);
}

#[test]
fn correlated_conditions_advise_while_provably_empty_paths_flag() {
    let script = r#"#!/bin/bash
set -u
m=; a=()
if [[ $1 == x ]]; then m=x; a=(one); fi
if [[ $m == x ]]; then
  if inner; then a=(two); fi
  printf '%s\n' "${a[@]}"
fi
printf '%s\n' "${a[@]}"
[[ $m == x ]] && printf '%s\n' "${a[@]}"
[[ $m == x ]] && { printf '%s\n' "${a[@]}"; }
[[ ${#a[@]} -eq 0 ]] && printf '%s\n' "${a[@]}"
[[ ${#a[@]} -eq 0 ]] && { printf '%s\n' "${a[@]}"; }
if [[ $m != x ]]; then :; else printf '%s\n' "${a[@]}"; fi
case "$m" in
  x) printf '%s\n' "${a[@]}" ;;
  *) printf '%s\n' "${a[@]}" ;;
esac
case "$m" in
  *) printf '%s\n' "${a[@]}" ;;
esac
if [[ $m == x ]]; then
  deferred() { printf '%s\n' "${a[@]}"; }
fi
"#;
    let res = analyze(script);
    assert_eq!(
        res.findings,
        vec![("a", 9), ("a", 12), ("a", 13), ("a", 20), ("a", 23)]
    );
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![7, 10, 11, 14, 16, 17]);
}

#[test]
fn case_scoped_dynamic_unset_resolution_and_fallbacks() {
    let resolved = r#"#!/bin/bash
set -u
mode=(fast)
ALPHA=(seed)
beta=(seed)
keep=(seed)
read_keep() { printf '%s\n' "${keep[@]}"; }
read_alpha() { printf '%s\n' "${ALPHA[@]}"; }
sanitize() {
  while IFS= read -r name; do
    normalized="${name,,}"
    normalized="${normalized^^}"
    case "$normalized" in
      ALPHA|foo-bar) : ;&
      "BETA"|'GAMMA_DIR') unset "$name" ;;
    esac
  done < env.txt
}
sanitize
printf '%s\n' "${mode[@]}"
read_keep
printf '%s\n' "${ALPHA[@]}"
read_alpha
printf '%s\n' "${beta[@]}"

exact_keep=(seed)
while IFS= read -r key; do
  alias="$key"
  case "$alias" in
    drop_me) : ;;&
    drop_two) unset "$key" ;;
  esac
done < env.txt
printf '%s\n' "${exact_keep[@]}"
"#;
    let res = analyze(resolved);
    assert_eq!(
        res.findings,
        vec![("ALPHA", 8), ("ALPHA", 22), ("beta", 24)]
    );
    assert!(res.unanalyzable.is_empty());

    let fallbacks = r#"#!/bin/bash
set -u
fall_wild=(seed)
case "$k" in
  *) : ;&
  alpha) unset "$k" ;;
esac
printf '%s\n' "${fall_wild[@]}"

mut_arm=(seed)
case "$k" in
  alpha)
    k="$other"
    unset "$k"
    ;;
esac
printf '%s\n' "${mut_arm[@]}"

fn_inside=(seed)
case "$k" in
  alpha)
    inner_unset() { unset "$k"; }
    ;;
esac
inner_unset
printf '%s\n' "${fn_inside[@]}"

ext_pat=(seed)
case "$k" in
  @(a|b)) unset "$k" ;;
esac
printf '%s\n' "${ext_pat[@]}"

multi_src=(seed)
pair="${a}_${b}"
args_src="$@"
case "$pair" in
  alpha) unset "$a" ;;
esac
case "$args_src" in
  alpha) unset "$a" ;;
esac
printf '%s\n' "${multi_src[@]}"

suffix_src=(seed)
suffixed="${target}_suffix"
case "$suffixed" in
  suffix_src_suffix) unset "$target" ;;
esac
printf '%s\n' "${suffix_src[@]}"

trim_src=(seed)
trimmed="${line%%=*}"
case "$trimmed" in
  other_name) unset "$line" ;;
esac
printf '%s\n' "${trim_src[@]}"
"#;
    let res = analyze(fallbacks);
    assert!(res.findings.is_empty());
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![8, 17, 26, 32, 43, 50, 57]);
}

#[test]
fn local_and_subshell_guard_mutation_breaks_condition_correlation() {
    let script = r#"#!/bin/bash
set -u
check_local() {
  local MODE="${1:-}"
  local -a arr=()
  if [[ "$MODE" == a ]]; then
    arr=(one)
  else
    MODE=a
  fi
  if [[ "$MODE" == a ]]; then
    printf '%s\n' "${arr[@]}"
  fi
}

check_subshell() (
  MODE="${1:-}"
  arr=()
  if [[ "$MODE" == a ]]; then
    arr=(one)
  else
    MODE=a
  fi
  if [[ "$MODE" == a ]]; then
    printf '%s\n' "${arr[@]}"
  fi
)

GLOBAL_MODE="${1:-}"
global_arr=()
if [[ "$GLOBAL_MODE" == ok ]]; then
  global_arr=(ready)
fi
shadow_global() {
  local GLOBAL_MODE=other
  GLOBAL_MODE=changed
}
shadow_global
if [[ "$GLOBAL_MODE" == ok ]]; then
  printf '%s\n' "${global_arr[@]}"
fi
"#;
    let res = analyze(script);
    assert!(res.findings.is_empty());
    let unan: Vec<usize> = res.unanalyzable.iter().map(|(l, _)| *l).collect();
    assert_eq!(unan, vec![12, 25]);
}

#[test]
fn case_unset_and_pattern_edge_coverage() {
    let script = r#"#!/bin/bash
set -u
keep=(seed)
v1="${k^}"; v2="${v1,}"; v3="${v2~}"; v4="${v3~~}"
if true; then
  case "$v4" in
    (alpha) : ;;&
    beta) : ;;&
    gamma) unset "$k" ;;
  esac
fi

c1="$c2"; c2="$c1"
case "$c1" in
  alpha) unset "$c1" ;;
esac

leave_fn() {
  case "$k" in
    alpha) return 0 ;;
    beta) return 0 ;&
    gamma) unset "$k" ;;
  esac
}

case "$k" in
  *|alpha) : ;;
  beta) unset "$k" ;;
esac

case "$k" in
  *) : ;;&
  alpha) unset "$k" ;;
esac

case "$k" in
  "$dyn") unset "$k" ;;
  alpha)
    mut_inner() { k=other; }
    unset "$k"
    ;;
esac

case
esac
esac
broken_fn() {
  case "$k" in
    alpha) :
}
case "$k" in
  unset "$k"
esac
printf '%s\n' "${keep[@]}"
"#;
    let res = analyze(script);
    assert!(res.findings.is_empty());
    assert!(!res.unanalyzable.is_empty());

    let edges = r#"#!/bin/bash
set -u
let m=1
let 'n=2'
(( arr[0] = 1 ))
if [[ $1 == a ]]; then :; fi
if (( ! x )); then exit 1; fi
if (( ${#a[@]} == )); then exit 1; fi
star=()
if [[ ${#star[*]} -eq 0 ]]; then exit 1; fi
printf '%s\n' "${star[@]}"
for i in a; do ( break ); done
for i in a; do exit 1; done
f_ret() { ( return ); }
self_x="${self_x,,}"
unset "bad]"
y="$src"
for i in a b; do y=other; done
u_arr=(seed)
if cond; then u_arr=(ok); else unset "$dyn"; fi
case "$k" in
  a b) unset "$k" ;;
esac
then :
else :
do :
fi
}
;;
"#;
    assert!(analyze(edges).findings.is_empty());
    assert!(analyze("set -u\necho \"$((unclosed\"\n")
        .findings
        .is_empty());
    assert!(analyze("set -u\n(( arr[0 = 1 ))\n").findings.is_empty());
    assert!(analyze("set -u\nif (( ${#a[@]} == 0\n").findings.is_empty());
    assert!(
        analyze("set -u\ncase \"$k\" in\n  \"unclosed) unset \"$k\" ;;\nesac\n")
            .findings
            .is_empty()
    );
}
