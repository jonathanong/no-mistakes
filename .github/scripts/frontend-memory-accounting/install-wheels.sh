#!/usr/bin/env bash
# Download pinned wheels only when the wheel directory is empty, then install offline.
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: install-wheels.sh REQUIREMENTS WHEEL_DIR VENV_DIR" >&2
  exit 2
fi

requirements="$1"
wheel_dir="$2"
venv_dir="$3"

python3 -m venv "$venv_dir"
pip="${venv_dir}/bin/pip"
mkdir -p "$wheel_dir"

shopt -s nullglob
wheels=("${wheel_dir}"/*.whl)
if [[ ${#wheels[@]} -eq 0 ]]; then
  "$pip" download \
    --disable-pip-version-check \
    --only-binary=:all: \
    --no-deps \
    --retries 2 \
    --timeout 20 \
    --dest "$wheel_dir" \
    --requirement "$requirements"
  wheels=("${wheel_dir}"/*.whl)
  if [[ ${#wheels[@]} -eq 0 ]]; then
    echo "pinned wheels were not downloaded" >&2
    exit 1
  fi
fi

"$pip" install \
  --disable-pip-version-check \
  --only-binary=:all: \
  --no-deps \
  --no-index \
  --find-links "$wheel_dir" \
  --retries 0 \
  --timeout 20 \
  --requirement "$requirements"
