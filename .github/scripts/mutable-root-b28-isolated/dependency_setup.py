"""Owned, bounded JS prerequisites; no lifecycle scripts, Cargo or native probe."""
import json
import os
from pathlib import Path
import sys
import subreaper
from deadline import install
from supervise import command
from transport import sha

D = Path(__file__).resolve().parent
cohort = sys.argv[1]
assert cohort in ['A', 'B']
out = Path(os.environ['RUNNER_TEMP']) / ('B28-dependencies-' + cohort)
out.mkdir(exist_ok=False)
subreaper.enable()
end = install(out, 'B28 complete workload cohort (' + cohort + ')')
source = Path('candidate-source').resolve()
spec = json.loads((D / 'hosted-spec.json').read_text())
assert spec['status'] == 'REVIEWED AND FROZEN'
command(['git', '-C', str(source), 'rev-parse', 'HEAD'], 'source-head', out, end, 10)
assert (out / 'source-head.stdout').read_text().strip() == spec['candidate_head']
command(['node', '--version'], 'actual-node', out, end, 10)
assert (out / 'actual-node.stdout').read_text().strip() == 'v26.11.1'
command(['pnpm', '--version'], 'actual-pnpm', out, end, 10)
assert (out / 'actual-pnpm.stdout').read_text().strip() == '11.24.0'
lock = source / 'pnpm-lock.yaml'
before = sha(lock)
env = dict(os.environ)
env.pop('GH_TOKEN', None)
command(['pnpm', '--dir', str(source), 'install', '--frozen-lockfile', '--ignore-scripts'],
        'install-no-lifecycle-scripts', out, end, 180, env)
assert sha(lock) == before
(out / 'closure.json').write_text(json.dumps({'complete': True, 'lock_sha256': before,
    'candidate': spec['candidate_head'], 'no_lifecycle_scripts': True, 'no_native_build': True,
    'node_version': 'v26.11.1', 'pnpm_version': '11.24.0'}, indent=2))
