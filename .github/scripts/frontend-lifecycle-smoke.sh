#!/usr/bin/env bash
# Eight fresh processes of one artifact; never use these controls to waive gates.
set -euo pipefail

sample=${1:?sample id required}
mode=${2:?lifecycle mode required}
case "$sample:$mode" in
  1:original|2:cold_first_analysis|3:conditioning_inclusive|4:extract_after_recorded_prefix|5:extract_after_recorded_prefix|6:conditioning_inclusive|7:cold_first_analysis|8:original) ;;
  *) echo "Unexpected sample/order: $sample:$mode" >&2; exit 1 ;;
esac
root=${FRONTEND_SMOKE_ROOT:?evidence root required}
directory="$root/sample-$sample"
mkdir -p "$directory"
if (( sample > 1 )); then
  python3 .github/scripts/frontend-lifecycle-profile.py "$((sample - 1))"
fi
mapfile -t binaries < <(find target/codspeed/analysis -type f -perm -111)
[[ ${#binaries[@]} -eq 1 ]] || { echo 'Expected one frozen benchmark' >&2; exit 1; }
binary=${binaries[0]}
digest=$(sha256sum "$binary" | cut -d ' ' -f 1)
[[ "$digest" == "$(cat "$root/executable.sha256")" ]] || exit 1
if [[ "$mode" == original ]]; then
  unset NO_MISTAKES_FRONTEND_LIFECYCLE
  filter='language_frontends::extract$'
else
  export NO_MISTAKES_FRONTEND_LIFECYCLE="$mode"
  filter="language_frontend_lifecycle::$mode\$"
fi
"$binary" "$filter" > "$directory/benchmark.log" 2>&1 &
benchmark_pid=$!
status=0
wait "$benchmark_pid" || status=$?
cat "$directory/benchmark.log"
python3 - "$directory" "$sample" "$mode" "$benchmark_pid" "$digest" "$status" <<'PY'
import json, pathlib, sys
directory, sample, mode, pid, digest, status = sys.argv[1:]
path = pathlib.Path(directory)
record = dict(sample=int(sample), mode=mode, pid=int(pid), binarySha256=digest,
              exit=int(status), configuredRayonWorkers=4)
import os
record['profileDirectory'] = os.environ['CODSPEED_PROFILE_FOLDER']
record['runPartIndex'] = int(sample) - 1
record['ciExecution'] = {key: os.environ[key] for key in ['GITHUB_RUN_ID', 'GITHUB_RUN_ATTEMPT', 'GITHUB_JOB']}
assert os.environ['CODSPEED_SKIP_UPLOAD'] == 'true'
if mode != 'original':
    ledgers = [json.loads(line.removeprefix('FRONTEND_LIFECYCLE '))
               for line in (path / 'benchmark.log').read_text().splitlines()
               if line.startswith('FRONTEND_LIFECYCLE ')]
    assert len(ledgers) == 1
    record['ledger'] = ledger = ledgers[0]
    assert ledger['pid'] == int(pid)
    assert ledger['measuredClosures'] == 1 and ledger['rayonWorkers'] == 4
    assert ledger['fixtureInitializations'] == 1
    assert ledger['priorCollectors'] == (7 if mode == 'extract_after_recorded_prefix' else 0)
    assert ledger['measuredCollectors'] == (8 if mode == 'conditioning_inclusive' else 1)
(path / 'sample.json').write_text(json.dumps(record, indent=2) + '\n')
PY
exit "$status"
