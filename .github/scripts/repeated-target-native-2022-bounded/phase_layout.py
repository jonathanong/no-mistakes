"""Separate arm×feature layout chains; the sole overlay is explicitly authenticated."""
from phase_common import *
from phase_source import checkout, compiler, select, assert_source
from phase_state import directory, restore, seal, cleanup
from pair import reviewed_stage
import shutil


def layout(arm, phase):
    reviewed_stage('layout-build')
    require(LAYOUT_MODE in ('default','instrumented') and phase in ('core','probe'), 'unknown_layout_phase')
    root=directory(arm);source=checkout(arm)
    target_source=source/'crates/no-mistakes/src/codebase/dependencies/extract_collector.rs'
    original=target_source.read_bytes()
    try:
        target_source.write_bytes(original+b'\n'+(HERE/'layout-helper.rs').read_bytes())
        if phase=='probe':restore(arm,'layout-core')
        (root/'proof').mkdir(exist_ok=True);(root/'binaries').mkdir(exist_ok=True)
        run(['rustup','update','stable'],log=root/'proof/stable-update',timeout=120)
        env=environment(root/'target');rustc=compiler()
        if phase=='core':
            args=['cargo','rustc','--locked','--release','-p','no-mistakes','--lib','--message-format=json']
            if LAYOUT_MODE=='instrumented':args+=['--features','test-instrumentation']
            args+=['--','--cfg','receiver_layout_probe','--check-cfg','cfg(receiver_layout_probe)']
            data=run(args,cwd=source,env=env,log=root/'proof/layout-core')
            row,_=select(data,'no_mistakes','rlib')
            path=next(Path(name) for name in row['filenames'] if name.endswith('.rlib'))
            shutil.copy2(path,root/'binaries/lib.rlib');write(root/'proof/layout-producer.json',row)
            assert_source(source);seal(arm,'layout-core')
        else:
            binary=root/'binaries/layout'
            args=['rustc','+1.99.1',str(HERE/'layout-probe.rs'),'--edition=2021','-C','opt-level=3',
                '-C','lto=true','-C','codegen-units=1','-C','debuginfo=0','-L',
                'dependency='+str(root/'target/release/deps'),'--extern',
                'no_mistakes='+str(root/'binaries/lib.rlib'),'-o',str(binary)]
            run(args,env=env,log=root/'proof/layout-link')
            rows=[line.split('\t') for line in run([str(binary)],log=root/'proof/layout-readback',timeout=30).decode().splitlines()]
            require(len(rows)==6 and all(len(row)==3 for row in rows),'layout_rows_changed')
            record={'revision':SPEC[arm]['revision'],'tree':SPEC[arm]['tree'],
                'source_inventory_sha256':SPEC['source_git_inventory_sha256'][arm],'mode':LAYOUT_MODE,
                'helper_sha256':sha(HERE/'layout-helper.rs'),'binary_sha256':sha(binary),'features':SPEC['features'],
                'actual_producer':json.loads((root/'proof/layout-producer.json').read_text()),
                'layouts':[{'type':name,'size':int(size),'align':int(align)} for name,size,align in rows]}
            write(JOB_ROOT/'public'/('layout-'+arm+'-'+LAYOUT_MODE+'.json'),record)
        assert_source(source)
    finally:
        target_source.write_bytes(original);cleanup(arm)
        require(target_source.read_bytes()==original,'layout_source_not_restored')
