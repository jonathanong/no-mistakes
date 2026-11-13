#!/usr/bin/env bash
set -euo pipefail

if (( $# == 0 )); then
  echo 'expected a nextest test binary' >&2
  exit 1
fi

native_test_binary=$1
shift
native_real_args_test='fetches::tests::run_args_tests::test_run_without_test_argv_uses_real_cli_args_from_absence'
native_real_args_selected=0
native_listing=0
for native_test_arg in "$@"; do
  if [[ "$native_test_arg" == "$native_real_args_test" ]]; then
    native_real_args_selected=1
  fi
  if [[ "$native_test_arg" == --list ]]; then
    native_listing=1
  fi
done

if (( native_real_args_selected && !native_listing )); then
  native_test_args=()
  for native_test_arg in "$@"; do
    case "$native_test_arg" in
      --exact|--nocapture) ;;
      *) native_test_args+=("$native_test_arg") ;;
    esac
  done
  exec "$native_test_binary" "${native_test_args[@]}"
fi
exec "$native_test_binary" "$@"
