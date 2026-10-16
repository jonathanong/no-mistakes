from pathlib import Path
import hashlib,json
out=Path(__file__).resolve().parent
from protocol_guard import require_reviewed_pair
reviewed_plan=require_reviewed_pair(out, live=False)
reference=Path('/tmp/nm2019-three-arm-evidence/semantic-diff-A-B.json')
expected=json.loads(reference.read_text())
a=json.loads((out/'semantic-private/graph-off-ARM_BASE.stdout').read_bytes())['semantic']
b=json.loads((out/'semantic-private/graph-off-ARM_CAND.stdout').read_bytes())['semantic']
assert len(a['sites'])==len(b['sites'])
changed=[]
for before,after in zip(a['sites'],b['sites']):
 if before!=after:
  changed.append(dict(file=before['file'],offset=before['offset'],callee=before['callee'],before=before,after=after))
assert changed==expected['changed_sites'], 'Exact before/after physical sites differ from the original reviewed48'
assert len(changed)==expected['A_B_changed_site_count']==48
for k in a:
 if k!='sites':assert a[k]==b[k],k
report=dict(status='PASS independently recomputed exact48 before/after site pairs and all other graph semantic fields equal',changed_site_count=48,candidate_revision=reviewed_plan['candidate_revision'],reference_sha256=hashlib.sha256(reference.read_bytes()).hexdigest())
p=out/'root-intended-delta-independent.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n');print(report['status'])
