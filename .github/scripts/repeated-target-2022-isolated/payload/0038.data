"""Isolated, untimed default/instrumented layout jobs for one source arm."""
import json
import os
from pathlib import Path

from common import resources, require
from pair import HERE, SPEC, reviewed_stage
from build_arm import cargo_environment, checkout_arm, cleanup_owned_target, digest, extract_artifact, link_probe, run


def layout(arm):
    require(arm in ('baseline','candidate'),'unknown_arm')
    reviewed_stage('layout-build')
    root=Path(os.environ['RUNNER_TEMP'])/'repeated-target-2022'
    root.mkdir(parents=True,exist_ok=True)
    resources(root,build=True)
    source,rows,proof=checkout_arm(root,arm)
    target_source=source/'crates/no-mistakes/src/codebase/dependencies/extract_collector.rs'
    before=target_source.read_bytes()
    output=root/arm/'layout-public';output.mkdir(parents=True,exist_ok=True)
    try:
        target_source.write_bytes(before+b'\n'+(HERE/'layout-helper.rs').read_bytes())
        for instrumented in (False,True):
            mode='instrumented' if instrumented else 'default'
            target=root/arm/('layout-target-'+mode)
            env=cargo_environment(target)
            args=['cargo','rustc','--locked','--release','-p','no-mistakes','--lib','--message-format=json']
            if instrumented:args+=['--features','test-instrumentation']
            args+=['--','--cfg','receiver_layout_probe','--check-cfg','cfg(receiver_layout_probe)']
            data=run(args,cwd=source,env=env,log=root/arm/(mode+'-cargo.log'))
            expected=SPEC['features']+(['test-instrumentation'] if instrumented else [])
            artifact=extract_artifact(data,'rlib',source/'crates/no-mistakes/src/lib.rs',expected)
            rlib=next(Path(p) for p in artifact['filenames'] if p.endswith('.rlib'))
            binary=root/arm/('layout-'+mode)
            link_probe(HERE/'layout-probe.rs',binary,rlib,target,env)
            rows=[line.split('\t') for line in run([str(binary)]).decode().splitlines()]
            require(len(rows)==6 and all(len(row)==3 for row in rows),'layout_rows_changed')
            record={'revision':SPEC[arm]['revision'],'tree':SPEC[arm]['tree'],'source_inventory_sha256':proof,
                    'mode':mode,'helper_sha256':digest(HERE/'layout-helper.rs'),'binary_sha256':digest(binary),
                    'features':expected,'layouts':[{'type':name,'size':int(size),'align':int(align)} for name,size,align in rows]}
            (output/(mode+'.json')).write_text(json.dumps(record,indent=2,sort_keys=True)+'\n')
    finally:
        target_source.write_bytes(before)
        for mode in ('default','instrumented'):
            cleanup_owned_target(root/arm/('layout-target-'+mode),root/arm)
        require(target_source.read_bytes()==before,'layout_source_not_restored')
    return output
