#!/bin/bash
set -u

count=0
limit=1
arith_cond=()
if [[ count -eq limit ]]; then
  arith_cond=(ready)
fi
limit=0
if [[ count -eq limit ]]; then
  printf '%s\n' "${arith_cond[@]}"
fi

fs_cond=()
if [[ -e marker ]]; then
  fs_cond=(ready)
fi
touch marker
if [[ -e marker ]]; then
  printf '%s\n' "${fs_cond[@]}"
fi

make_empty() {
  created=()
}
make_empty
printf '%s\n' "${created[@]}"

hdr=()
inner=(seed)
while test -n "${hdr[*]}" && hdr=(filled); do
  inner=()
done

post_unset=(seed)
unset "$target"
if [[ ${#post_unset[@]} -gt 0 ]]; then
  printf '%s\n' "${post_unset[@]}"
fi

replay_arr=(seed)
for _ in 1 2; do
  helper_in_loop() { :; }
  replay_arr=()
done

inc=0
inc_arr=()
if [[ ++inc -eq 2 ]]; then inc_arr=(ok); fi
if [[ ++inc -eq 2 ]]; then printf '%s\n' "${inc_arr[@]}"; fi

flags=()
flag_arr=()
if [[ -v flags[0] ]]; then flag_arr=(ok); fi
flags[0]=yes
if [[ -v flags[0] ]]; then printf '%s\n' "${flag_arr[@]}"; fi

brace_arr=()
value=$({ :; }>/dev/null)
printf '%s\n' "${brace_arr[@]}"

REPLY=0
if [[ $REPLY == 1 ]]; then reply_arr=(ok); else reply_arr=(); fi
read -r
if [[ $REPLY == 1 ]]; then printf '%s\n' "${reply_arr[@]}"; fi

sub_arr=()
if [[ "${slots[0]}" == 1 ]]; then sub_arr=(ok); fi
read -r "slots[$idx]"
if [[ "${slots[0]}" == 1 ]]; then printf '%s\n' "${sub_arr[@]}"; fi

fill_redir() { redir_arr=(ok); }
redir_arr=()
fill_redir >/dev/null
printf '%s\n' "${redir_arr[@]}"

if false; then
  skipped_fill() { skipped_arr=(ok); }
fi
skipped_arr=()
skipped_fill
printf '%s\n' "${skipped_arr[@]}"

loop_var_arr=(ready)
for flag in no yes; do
  if [[ "$flag" == yes ]]; then
    printf '%s\n' "${loop_var_arr[@]}"
  fi
  if [[ "$flag" == yes ]]; then
    loop_var_arr=(ready)
  else
    loop_var_arr=()
  fi
done

maybe_branch_load() {
  if cond; then
    maybe_branch_items=(a b)
  fi
}
maybe_branch_items=()
maybe_branch_load
printf '%s\n' "${maybe_branch_items[@]}"

status_arr=()
grep -q x f
if [[ $? -eq 0 ]]; then status_arr=(x); fi
grep -q y f
if [[ $? -eq 0 ]]; then printf '%s\n' "${status_arr[@]}"; fi

pos_arr=()
if [[ $# -gt 0 ]]; then pos_arr=(x); fi
shift
if [[ $# -gt 0 ]]; then printf '%s\n' "${pos_arr[@]}"; fi

arg_arr=()
if [[ "$1" == prod ]]; then arg_arr=(x); fi
set -- dev
if [[ "$1" == prod ]]; then printf '%s\n' "${arg_arr[@]}"; fi

rand_arr=()
if [[ $RANDOM -eq 0 ]]; then rand_arr=(x); fi
if [[ $RANDOM -eq 0 ]]; then printf '%s\n' "${rand_arr[@]}"; fi

set +u
enable_redir() { set -u; }
redir_u=()
enable_redir >/dev/null
printf '%s\n' "${redir_u[@]}"

