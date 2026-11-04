#!/usr/bin/env bash
set -uo pipefail

if [[ $# -lt 1 ]]; then
  printf 'Usage: %s <command> [args...]\n' "$0" >&2
  exit 2
fi

interval="${RESOURCE_SAMPLE_INTERVAL_SECONDS:-15}"

proc_value() {
  local value=""
  if [[ -r "$1" ]]; then
    value="$(awk -v key="$2" -v column="$3" -v divisor="$4" '
      $1 == key {
        value = $column
        sub(/^[a-z0-9]+=/, "", value)
        print (divisor > 1 ? int(value / divisor) : value)
        exit
      }
    ' "$1" 2>/dev/null)"
  fi
  printf '%s' "${value:-n/a}"
}

disk_available_mib() {
  local value
  value="$(df -Pm . 2>/dev/null | awk 'NR == 2 { print $4 }')"
  printf '%s' "${value:-n/a}"
}

top_process() {
  local value
  value="$(ps -eo rss=,comm= 2>/dev/null | sort -nr | awk 'NR == 1 { printf "%dMiB:%s", $1 / 1024, $2 }')"
  printf '%s' "${value:-n/a}"
}

sample() {
  printf '[resources] %s %s mem_available_mib=%s swap_free_mib=%s disk_available_mib=%s oom_kills=%s memory_full_avg10=%s top_rss=%s\n' \
    "$(date -u +%H:%M:%S)" \
    "$1" \
    "$(proc_value /proc/meminfo MemAvailable: 2 1024)" \
    "$(proc_value /proc/meminfo SwapFree: 2 1024)" \
    "$(disk_available_mib)" \
    "$(proc_value /proc/vmstat oom_kill 2 1)" \
    "$(proc_value /proc/pressure/memory full 2 1)" \
    "$(top_process)"
}

"$@" &
child=$!

stop_sampler() {
  kill "$sampler" 2>/dev/null
  wait "$sampler" 2>/dev/null
}

on_term() {
  stop_sampler
  sample received-sigterm
  kill -TERM "$child" 2>/dev/null
  wait "$child" 2>/dev/null
  exit 143
}

sample start

(
  while sleep "$interval" >/dev/null 2>&1; do
    sample running
  done
) &
sampler=$!

trap on_term TERM

wait "$child"
status=$?
stop_sampler
sample "exit-$status"
exit "$status"
