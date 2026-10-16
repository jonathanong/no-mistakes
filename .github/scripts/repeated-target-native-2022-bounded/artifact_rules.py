"""Original artifact selection and probe API controls, without a build entrypoint."""
import json
from pathlib import Path
from common import require
from pair import SPEC

def extract_artifact(data, kind, source, expected_features=None):
    rows=[json.loads(line) for line in data.splitlines() if line.startswith(b'{')]
    expected_name='no-mistakes' if kind=='bin' else 'no_mistakes'
    artifacts=[r for r in rows if r.get('reason')=='compiler-artifact' and r['target']['name']==expected_name]
    selected=[r for r in artifacts if kind in r['target']['kind'] or (kind=='bin' and r.get('executable'))]
    require(len(selected)==1,'artifact_count_or_kind_mismatch',kind=kind,count=len(selected))
    item=selected[0]
    require(item['fresh'] is False,'owned_artifact_reused',kind=kind)
    require(Path(item['target']['src_path']).resolve()==source,'artifact_source_changed',kind=kind)
    require(sorted(item['features'])==sorted(expected_features or SPEC['features']),'artifact_features_changed',kind=kind)
    require(item['profile']=={'opt_level':'3','debuginfo':0,'debug_assertions':False,'overflow_checks':False,'test':False},'artifact_profile_changed',kind=kind)
    return item

def graph_probe_cfg(arm):
    require(arm in ('baseline', 'candidate'), 'unknown_arm')
    return 'alternatives_api' if arm == 'candidate' else None
