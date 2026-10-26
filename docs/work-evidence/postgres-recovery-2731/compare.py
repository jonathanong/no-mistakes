"""Reproduce the matched PostgreSQL fixture comparison from the repository root."""
import argparse
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser()
parser.add_argument('baseline')
parser.add_argument('candidate')
parser.add_argument('--out', type=Path, required=True)
parser.add_argument('--configs', type=Path, default=Path(__file__).with_name('configs.tsv'))
parser.add_argument('--jobs', type=int, default=4)
args = parser.parse_args()
args.out.mkdir(parents=True, exist_ok=True)
rows = [line.split('\t') for line in args.configs.read_text().splitlines()]

def compare(row):
    root, config = row
    key = hashlib.sha256((root + '|' + config).encode()).hexdigest()[:16]
    runtime = args.out / ('runtime-' + key)
    runtime.mkdir(exist_ok=True)
    results = []
    for phase, binary in [('before', args.baseline), ('after', args.candidate)]:
        result = subprocess.run(
            [str(Path(binary).resolve()), 'check', '--root', str(Path(root).resolve()),
             '--config', config, '--format', 'json'], capture_output=True,
            env=dict(os.environ, XDG_RUNTIME_DIR=str(runtime.resolve())),
        )
        (args.out / (phase + '-' + key + '.stdout')).write_bytes(result.stdout)
        (args.out / (phase + '-' + key + '.stderr')).write_bytes(result.stderr)
        results.append((result.returncode, result.stdout, result.stderr))
    return dict(root=root, config=config, identical=results[0] == results[1],
                beforeStatus=results[0][0], afterStatus=results[1][0],
                stdoutIdentical=results[0][1] == results[1][1],
                stderrIdentical=results[0][2] == results[1][2])

with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
    results = list(pool.map(compare, rows))
(args.out / 'summary.json').write_text(json.dumps(results, indent=2) + '\n')
print(json.dumps(dict(total=len(results), unchanged=sum(row['identical'] for row in results),
                      changed=[row for row in results if not row['identical']]), indent=2))
