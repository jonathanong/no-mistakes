"""CI output anchoring/finalization; only explicit public metadata, never env dumps."""
import argparse,json,os
from pathlib import Path
from common import package_guard,save,sha
from transport import seal
P=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('phase',choices=['begin','finish']);p.add_argument('output',type=Path);a=p.parse_args()
package_guard(P,os.environ)
if a.phase=='begin':
    assert not a.output.exists();a.output.mkdir(parents=True)
    save(a.output/'job-metadata.json',{k:os.environ.get(k)for k in['GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT','GITHUB_SHA','GITHUB_WORKFLOW_SHA','JOB_STARTED_UNIX']})
    save(a.output/'source-pins.json',{'package_manifest_sha256':sha(P/'package-manifest.json'),'protocol_sha256':sha(P/'protocol.json'),'source_pair':json.loads((P/'source-pair.json').read_text())})
else:
    a.output.mkdir(parents=True,exist_ok=True)
    outcome=os.environ['STAGE_OUTCOME'];assert outcome in['success','failure','cancelled','skipped']
    save(a.output/'transport-terminal.json',{'stage_outcome':outcome,'needs':json.loads(os.environ.get('NEEDS_JSON','{}')),'run_id':os.environ['GITHUB_RUN_ID'],'attempt':os.environ['GITHUB_RUN_ATTEMPT']})
    seal(a.output)
