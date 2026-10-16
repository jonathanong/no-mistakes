"""All four fresh layout receipts are mandatory; no CPU/RSS inference from sizes."""
from phase_common import *


def verify(root):
    records={}
    for path in (root/'layouts').glob('*/public/layout-*.json'):
        row=json.loads(path.read_text());arm=next((arm for arm in ARM if row['revision']==SPEC[arm]['revision']),None)
        require(arm is not None and row['mode'] in ('default','instrumented'),'foreign_layout_source_or_mode')
        key=arm+':'+row['mode'];require(key not in records,'duplicate_layout')
        require(row['tree']==SPEC[arm]['tree'] and row['source_inventory_sha256']==SPEC['source_git_inventory_sha256'][arm]
            and row['helper_sha256']==sha(HERE/'layout-helper.rs'),'layout_source_or_helper_changed')
        features=SPEC['features']+(['test-instrumentation']if row['mode']=='instrumented'else[])
        producer=row['actual_producer']
        require(row['features']==features and producer['features']==features and producer['fresh']is False,'layout_producer_changed')
        require(producer['profile']==dict(opt_level='3',debuginfo=0,debug_assertions=False,overflow_checks=False,test=False),'layout_profile_changed')
        require(len(row['layouts'])==6 and all(type(item['size'])is int and item['size']>0 and type(item['align'])is int and item['align']>0 for item in row['layouts']),'incomplete_layout')
        records[key]={'artifact_relative_path':str(path.relative_to(root)),'sha256':sha(path),'record':row}
    require(set(records)=={arm+':'+mode for arm in ARM for mode in ('default','instrumented')},'missing_layout_arm_or_mode')
    write(root/'public/layout-preflight.json',records)
