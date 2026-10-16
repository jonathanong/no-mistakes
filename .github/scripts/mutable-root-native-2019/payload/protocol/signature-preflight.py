"""Untimed execution smoke for every timed argument shape; no CPU samples."""
import hashlib,json,os
from pathlib import Path
import subprocess
results=Path(__file__).resolve().parent
from protocol_guard import require_reviewed_pair
reviewed_plan=require_reviewed_pair(results, live=True)
out=Path('/tmp/nm2019-three-arm-evidence')
arms_root=Path('/tmp/nm2019-current-main-pair-evidence')
path=results/'timing-signatures.json'
assert not path.exists()
rows=json.loads((out/'workloads.json').read_text())
expected={}
for arm in ('ARM_BASE', 'ARM_CAND'):
 manifest=json.loads((arms_root/arm/'manifest.json').read_text())
 assert hashlib.sha256((arms_root/arm/'probe').read_bytes()).hexdigest()==manifest['binary_sha256']
for row in rows:
 expected[row['id']]={}
 args=[arg.replace('@INPUT@',str(out/'inputs')).replace('@COUNT@','1') for arg in row['args']]
 for arm in ('ARM_BASE', 'ARM_CAND'):
  result=subprocess.run([str(arms_root/arm/'probe'),*args],capture_output=True,env=dict(os.environ,RAYON_NUM_THREADS='4'))
  assert result.returncode==0 and not result.stderr and result.stdout.strip().isdigit(),(row['id'],arm)
  expected[row['id']][arm]=result.stdout.decode().strip()
 original=json.loads((out/'timing-signatures.json').read_text())[row['id']]
 assert expected[row['id']]['ARM_BASE']==original['A'],row['id']
 assert expected[row['id']]['ARM_CAND']==original['B'],row['id']
 if row['id'] in ['graph','graph-observer']:
  assert set(expected[row['id']].values())=={'937'},row['id']
 if row['id']=='graph-no-calls':assert set(expected[row['id']].values())=={'0'}
 print(row['id'],expected[row['id']],flush=True)
for row in rows:
 if row['id'].startswith('effects-') and row['id'].endswith('-off'):
  assert expected[row['id']]==expected[row['id'][:-4]+'-observer']
path.write_text(json.dumps(expected,indent=2)+'\n')
