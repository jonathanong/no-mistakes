#!/usr/bin/env bash
# Wrapper behind `pnpm run evals`. See evals/README.md.
#
# Exists for two reasons, both learned the hard way:
#
#  1. `pnpm run evals -- <flags>` forwards the `--` itself into argv. The eval
#     CLI reads it as end-of-options, silently discards every flag after it,
#     and launches an unfiltered full-suite run. Strip it.
#
#  2. Refuse to launch without an explicit scope. An unfiltered run is 2 arms x
#     every case (59 x 3 x 2 = 354 runs, ~$60) and sweeps in the `heldout` flow,
#     which only means anything while it stays unseen. `--all` opts in
#     deliberately.
set -euo pipefail

args=()
scoped=0
allow_full=0

for a in "$@"; do
  case "$a" in
    --) continue ;;
    --all) allow_full=1; continue ;;
    --tag | --tag=*) scoped=1 ;;
    --help | -h) scoped=1 ;;
  esac
  args+=("$a")
done

if [ "$scoped" -eq 0 ] && [ "$allow_full" -eq 0 ]; then
  cat >&2 <<'MSG'
run-evals: refusing to launch an unfiltered run.

  Unfiltered is 2 arms x every case (59 x 3 x 2 = 354 runs, ~$60) and includes the
  `heldout` flow, which is only meaningful while it stays unseen.

  Scope it with --tag <flow>..., or pass --all to run everything on purpose.
  See evals/README.md.
MSG
  exit 2
fi

exec claude plugin eval . ${args[@]+"${args[@]}"}
