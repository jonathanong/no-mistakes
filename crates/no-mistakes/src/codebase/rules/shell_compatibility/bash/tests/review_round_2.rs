use super::analyze;

fn reports(source: &str, name: &str, line: usize) {
    let result = analyze(source);
    assert!(
        result
            .findings
            .iter()
            .any(|(found, found_line)| *found == name && *found_line == line),
        "findings={:?} unanalyzable={:?}",
        result.findings,
        result.unanalyzable
    );
}

fn quiet(source: &str) {
    let result = analyze(source);
    assert!(
        result.findings.is_empty() && result.unanalyzable.is_empty(),
        "findings={:?} unanalyzable={:?}",
        result.findings,
        result.unanalyzable
    );
}

fn advises_at(source: &str, line: usize) {
    let result = analyze(source);
    assert!(
        result.findings.is_empty(),
        "findings={:?} unanalyzable={:?}",
        result.findings,
        result.unanalyzable
    );
    assert!(
        result.unanalyzable.iter().any(|(found, _)| *found == line),
        "{:?}",
        result.unanalyzable
    );
}

#[test]
fn mapfile_offset_and_empty_append_preserve_maybe_empty() {
    advises_at(
        r#"#!/bin/bash
set -u
mapfile -t items < input
mapfile -t -O 1 items < /dev/null
if [[ $flag == yes ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        5,
    );
    advises_at(
        r#"#!/bin/bash
set -u
items=()
mapfile -t -O 1 items < input
if [[ $flag == yes ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        5,
    );
    quiet(
        r#"#!/bin/bash
set -u
items=(ready)
mapfile -t -O 1 items < /dev/null
printf '%s\n' "${items[@]}"
"#,
    );
    advises_at(
        r#"#!/bin/bash
set -u
mapfile -t items < input
items+=()
if [[ $flag == yes ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        5,
    );
    let copied = analyze(
        r#"#!/bin/bash
set -u
mapfile -t items < input
copy=("${items[@]}")
if [[ $flag == yes ]]; then printf '%s\n' "${copy[@]}"; fi
"#,
    );
    assert_eq!(copied.findings, vec![("items", 4)]);
    assert!(
        copied.unanalyzable.iter().any(|(line, _)| *line == 5),
        "{:?}",
        copied.unanalyzable
    );
}

#[test]
fn contradicting_length_guard_arms_are_infeasible() {
    quiet(
        r#"#!/bin/bash
set -u
items=(ready)
if [[ ${#items[@]} -eq 0 ]]; then
  printf '%s\n' "${items[@]}"
fi
printf '%s\n' "${items[@]}"
"#,
    );
    quiet(
        r#"#!/bin/bash
set -u
items=(ready)
if [[ ${#items[@]} -gt 0 ]]; then
  printf '%s\n' "${items[@]}"
else
  printf '%s\n' "${items[@]}"
fi
printf '%s\n' "${items[@]}"
"#,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
if [[ ${#items[@]} -gt 0 ]]; then
  items=(ready)
fi
printf '%s\n' "${items[@]}"
"#,
        "items",
        7,
    );
    reports(
        r#"#!/bin/bash
set -u
mapfile -t items < input
if [[ ${#items[@]} -eq 0 ]]; then
  printf '%s\n' "${items[@]}"
fi
"#,
        "items",
        5,
    );
}

#[test]
fn indexed_case_pattern_and_quoted_arith_and_shopt() {
    reports(
        r#"#!/bin/bash
set -u
items=(x)
target='items[0]'
case "$target" in
  "items[0]") unset "$target" ;;
esac
printf '%s\n' "${items[@]}"
"#,
        "items",
        8,
    );
    advises_at(
        r#"#!/bin/bash
set -u
items=(x)
case "$target" in
  "items[a[0]]") unset "$target" ;;
esac
printf '%s\n' "${items[@]}"
"#,
        7,
    );
    quiet(
        r#"#!/bin/bash
set -u
mode=(ok)
case "$target" in
  alpha)
    printf '%s\n' '$((target=0))' "\$((target=0))"
    unset "$target"
    ;;
esac
printf '%s\n' "${mode[@]}"
"#,
    );
    advises_at(
        r#"#!/bin/bash
set -u
mode=(ok)
case "$target" in
  alpha)
    printf '%s\n' "$((target=0))"
    unset "$target"
    ;;
esac
printf '%s\n' "${mode[@]}"
"#,
        10,
    );
    advises_at(
        r#"#!/bin/bash
set -u
flag=yes
items=()
[[ $flag == YES ]] && items=(ready)
shopt -s nocasematch
[[ $flag == YES ]] && echo "${items[@]}"
"#,
        7,
    );
    quiet(
        r#"#!/bin/bash
set -u
flag=yes
items=()
[[ $flag == YES ]] && items=(ready)
shopt -q nocasematch
[[ $flag == YES ]] && echo "${items[@]}"
"#,
    );
}

#[test]
fn case_fallthrough_skips_pattern_and_redirected_brace_group() {
    quiet(
        r#"#!/bin/bash
set -u
items=()
case x in
  *) : ;&
  "${items[@]}") : ;;
esac
"#,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
case x in
  *) : ;;&
  "${items[@]}") : ;;
esac
"#,
        "items",
        6,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
case x in
  a) : ;&
  "${items[@]}") : ;;
esac
"#,
        "items",
        6,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
{ items=(ready); } >/dev/null/out
printf '%s\n' "${items[@]}"
"#,
        "items",
        5,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
[[ ${#items[@]} -eq 0 ]] && { exit 1; } >/dev/null/out
printf '%s\n' "${items[@]}"
"#,
        "items",
        5,
    );
}

#[test]
fn quoted_and_isolated_function_unset_and_loop_fixes() {
    reports(
        r#"#!/bin/bash
set -u
stop_here() { exit 1; }
unset -f 'stop_here'
items=()
[[ ${#items[@]} -eq 0 ]] && stop_here
printf '%s\n' "${items[@]}"
"#,
        "items",
        7,
    );
    reports(
        r#"#!/bin/bash
set -u
stop_here() { exit 1; }
unset -- "stop_here"
items=()
[[ ${#items[@]} -eq 0 ]] && stop_here
printf '%s\n' "${items[@]}"
"#,
        "items",
        7,
    );
    reports(
        r#"#!/bin/bash
set +u
enable() { set -u; }
cleanup() { unset -f enable; }
( unset -f enable )
enable
items=()
printf '%s\n' "${items[@]}"
"#,
        "items",
        8,
    );
    quiet(
        r#"#!/bin/bash
set +u
enable() { set -u; }
cleanup() { unset -f enable; }
cleanup
enable
items=()
printf '%s\n' "${items[@]}"
"#,
    );
    reports(
        r#"#!/bin/bash
set +u
enable() { set -u; }
cleanup() { unset -f enable; }
if maybe; then cleanup; fi
enable
items=()
printf '%s\n' "${items[@]}"
"#,
        "items",
        8,
    );
    reports(
        r#"#!/bin/bash
set -u
a=()
b=(ready)
for x in "${a[@]}"; do
  b=()
done
"#,
        "a",
        5,
    );
    reports(
        r#"#!/bin/bash
set -u
a=()
for in in; do
  a=(ready)
done
printf '%s\n' "${a[@]}"
"#,
        "a",
        7,
    );
}

#[test]
fn ci_unset_scalars_compound_fn_bodies_and_trailing_redirections() {
    advises_at(
        r#"#!/bin/bash
set -u
FLAG=1
if [[ "${FLAG-}" == "" ]]; then items=(ready); else items=(); fi
key=FLAG
folded=${key,,}
case "$folded" in
  flag) unset "$key" ;;
esac
if [[ "${FLAG-}" == "" ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        10,
    );
    advises_at(
        r#"#!/bin/bash
set -u
FLAG=1
drop_flag() {
  local key=FLAG
  local folded=${key,,}
  case "$folded" in
    flag) unset "$key" ;;
  esac
}
if [[ "${FLAG-}" == "" ]]; then items=(ready); else items=(); fi
drop_flag
if [[ "${FLAG-}" == "" ]]; then printf '%s\n' "${items[@]}"; fi
"#,
        13,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
f() if [[ ${#items[@]} -eq 0 ]]; then if true; then exit 1; fi; fi
g() case x in *) exit 1 ;; esac
h() for x in a; do items=(ready); done
k() [[ ${#items[@]} -eq 0 ]]
m() (( idx++ ))
printf '%s\n' "${items[@]}"
"#,
        "items",
        9,
    );
    advises_at(
        r#"#!/bin/bash
set -u
f() echo hi
"#,
        3,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
{ :; } > "${items[@]}"
"#,
        "items",
        4,
    );
    reports(
        r#"#!/bin/bash
set -u
items=()
{ items=(ready); } > "${items[@]}"
"#,
        "items",
        4,
    );
    reports(
        r#"#!/bin/bash
set -u
items=(ready)
f() { :; } > "${items[@]}"
items=()
"#,
        "items",
        4,
    );
    reports(
        r#"#!/bin/bash
set -u
fill() { items=(x); } <<< input > /missing/out
items=()
fill
printf '%s\n' "${items[@]}"
"#,
        "items",
        6,
    );
    reports(
        r#"#!/bin/bash
set -u
fill() { items=(x); } <<EOF > /missing/out
body
EOF
items=()
fill
printf '%s\n' "${items[@]}"
"#,
        "items",
        8,
    );
}
