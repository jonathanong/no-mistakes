#!/usr/bin/env bash
# Wrapper behind `pnpm run evals`. See evals/README.md.
#
#  1. `pnpm run evals -- <flags>` forwards the `--` itself into argv. The eval
#     CLI reads it as end-of-options, silently discards every flag after it,
#     and launches an unfiltered full-suite run. Strip it.
#
#  2. Refuse to launch without an explicit scope (--tag or --case). An
#     unfiltered run is every case x 3 runs x 2 arms and sweeps in the
#     `heldout` flow, which only means anything while it stays unseen. `--all`
#     opts in deliberately.
#
#  3. Pin the judge. The agent is claude-sonnet-5-5 (evals/generate.py); the
#     judge defaults to claude-opus-5-5 and must never equal the agent model.
set -euo pipefail

AGENT_MODEL=claude-sonnet-5-5
JUDGE_MODEL=claude-opus-5-5

args=()
scoped=0
allow_full=0
judge=""
agent="$AGENT_MODEL"
prev=""

for a in "$@"; do
  case "$a" in
    --) continue ;;
    --all) allow_full=1; continue ;;
    --tag | --tag=* | --case | --case=*) scoped=1 ;;
    --help | -h) scoped=1 ;;
    --judge-model=*) judge="${a#*=}" ;;
    --model=*) agent="${a#*=}" ;;
  esac
  case "$prev" in
    --judge-model) judge="$a" ;;
    --model) agent="$a" ;;
  esac
  prev="$a"
  args+=("$a")
done

if [ "$scoped" -eq 0 ] && [ "$allow_full" -eq 0 ]; then
  cat >&2 <<'MSG'
run-evals: refusing to launch an unfiltered run.

  Unfiltered is every case x 3 runs x 2 arms and includes the `heldout` flow,
  which is only meaningful while it stays unseen.

  Scope it with --tag <flow>... or --case <glob>, or pass --all to run
  everything on purpose. See evals/README.md.
MSG
  exit 2
fi

if [ -z "$judge" ]; then
  judge="$JUDGE_MODEL"
  args+=(--judge-model "$judge")
fi

family() {
  case "$1" in
    *haiku*) echo haiku ;;
    *sonnet*) echo sonnet ;;
    *opus*) echo opus ;;
    *fable*) echo fable ;;
    *) echo "$1" ;;
  esac
}

if [ "$(family "$judge")" = "$(family "$agent")" ]; then
  echo "run-evals: the judge model ($judge) must not be the agent model ($agent)." >&2
  exit 2
fi

exec claude plugin eval . ${args[@]+"${args[@]}"}
