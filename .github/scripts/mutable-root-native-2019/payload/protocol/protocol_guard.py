"""Hosted ARM pair guards; never accepts x86 reuse receipts or historical rows."""
import os,sys,json,hashlib
from pathlib import Path
sys.path.insert(0,os.environ['ARM2019_SOURCE'])
from common import SPEC,ARM,ARTIFACTS,require,sha
from guard import reviewed
from package import verify

def require_reviewed_pair(out,live=True):
    require(SPEC['status']=='REVIEWED AND FROZEN','DRAFT: no protocol execution')
    if live:reviewed()
    plan=json.loads((out/'measurement-plan.json').read_text())
    require(plan['run_id']==os.environ['GITHUB_RUN_ID'],'wrong run')
    require(plan['baseline_revision']==SPEC['baseline']['revision']and plan['candidate_revision']==SPEC['candidate']['revision'],'wrong source pair')
    require(set(plan['controls'])=={'workloads.json','batches-1972.json','calibration-batches.json'},'controls missing')
    for name,digest in SPEC['controls'].items():require(plan['controls'][name]==digest==sha(out/name),'fixed controls changed')
    for arm,label in ARM.items():
        verify(arm);require(sha(ARTIFACTS/label/'manifest.json')==plan['artifact_manifests'][label],'artifact changed')
    return plan
