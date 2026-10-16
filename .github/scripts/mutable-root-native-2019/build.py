"""Bounded production phases; each current arm has its own true Cargo producer."""
from common import *
from guard import reviewed
from source import checkout,compiler,select
from state import directory,restore,seal,cleanup
from machine import snapshot,elf
import shutil
PHASES=['core','cli','addon','probe','sql']

def refresh_library(data,root):
    records=[json.loads(s)for s in data.splitlines()if s.startswith(b'{')]
    for row in records:
        if row.get('reason')!='compiler-artifact'or row['target']['name']!='no_mistakes'or'rlib'not in row['target']['kind']:continue
        artifact=Path(next(f for f in row['filenames']if f.endswith('.rlib')))
        if row['fresh']:
            require(sha(artifact)==sha(root/'binaries/lib.rlib'),'fresh library hash differs from producer')
        else:
            require(row['features']==SPEC['features'],'library features changed')
            shutil.copy2(artifact,root/'binaries/lib.rlib');write(root/'proof/library-producer.json',row)
        write(root/'proof/library-observation.json',row)

def cargo(phase,source,root,env):
    args=['cargo','build','--locked','--release','-p','no-mistakes']
    args+=['--lib']if phase=='core'else['--bin','no-mistakes']
    if phase=='addon':args=['cargo','rustc','--locked','--release','-p','no-mistakes','--lib','--crate-type','cdylib']
    args+=['--message-format=json'];data=run(args,cwd=source,env=env,log=root/'proof'/phase,timeout=1500)
    (root/'proof'/(phase+'-cargo.jsonl')).write_bytes(data)
    name='no-mistakes'if phase=='cli'else'no_mistakes';kind={'core':'rlib','cli':'bin','addon':'cdylib'}[phase]
    item,_=select(data,name,kind)
    require(Path(item['manifest_path'])==source/'crates/no-mistakes/Cargo.toml','foreign producer')
    write(root/'proof'/(phase+'-producer.json'),item)
    if phase=='core':refresh_library(data,root)
    elif phase=='cli':
        refresh_library(data,root);shutil.copy2(item['executable'],root/'binaries/cli')
    else:shutil.copy2(next(f for f in item['filenames']if f.endswith('.so')),root/'binaries/addon.node')
    if phase!='core':elf(root/'binaries'/('cli'if phase=='cli'else'addon.node'))

def link(phase,root,env):
    materialize_harness(root/'harness')
    source=root/'harness/main.rs'if phase=='probe'else HERE/'payload/sql/probe.rs'
    output=root/'binaries'/('probe'if phase=='probe'else'sql-probe')
    args=['rustc','+1.99.1','--edition=2021','-C','opt-level=3','-C','lto=true','-C',
          'codegen-units=1','-C','debuginfo=0','-L','dependency='+str(root/'target/release/deps'),
          '--extern','no_mistakes='+str(root/'binaries/lib.rlib')]
    if phase=='sql':
        import tomllib
        from dependency_producer import select_release_rlib
        lock=tomllib.loads((root/'source/Cargo.lock').read_text())
        packages=[row for row in lock['package']if row['name']=='serde_json']
        require(len(packages)==1,'serde lock identity mismatch')
        package=packages[0];package_id=package['source']+'#serde_json@'+package['version']
        cargo_json=root/'proof/core-cargo.jsonl'
        producer,path=select_release_rlib(cargo_json.read_text(),'serde_json',root,package_id)
        require(path.is_file()and not path.is_symlink(),'serde library not regular')
        args+=['--extern','serde_json='+str(path)];write(root/'proof/serde-producer.json',producer)
        write(root/'proof/serde-library-binding.json',{'package_id':package_id,'path':str(path),
              'sha256':sha(path),'features':producer['features'],'profile':producer['profile'],
              'core_cargo_json_sha256':sha(cargo_json),'lock_sha256':sha(root/'source/Cargo.lock')})
    args+=[str(source),'-o',str(output)]
    run(args,env=env,log=root/'proof'/(phase+'-link'),timeout=1500);elf(output)
    if phase=='sql':
        inputs=HERE/'payload/sql/inputs'
        actual=run([str(output),*[str(inputs/n)for n in ['index-identity-feature-parity.sql',
                   'index-identity-feature-parity.expected.json','legacy-default-full-facts.json']]],
                   env=env,log=root/'proof/sql-run',timeout=120)
        require(actual==(inputs/'legacy-default-full-facts.json').read_bytes(),'SQL full raw parity mismatch')

def build(arm,phase):
    reviewed();require(phase in PHASES,'unknown phase');root=directory(arm);source=checkout(arm)
    previous=PHASES[PHASES.index(phase)-1]if phase!='core'else None
    try:
        if previous:restore(arm,previous)
        (root/'proof').mkdir(exist_ok=True);(root/'binaries').mkdir(exist_ok=True)
        write(root/'proof'/(phase+'-machine.json'),snapshot(build=True))
        run(['rustup','update','stable'],log=root/'proof'/(phase+'-stable-update'),timeout=120)
        rustc=compiler()
        env=environment(root/'target')
        if phase in ['core','cli','addon']:cargo(phase,source,root,env)
        else:link(phase,root,env)
        require(not git('status','--porcelain',cwd=source),'source changed during build')
        write(root/'proof'/(phase+'-closed.json'),{'arm':arm,'phase':phase,'revision':SPEC[arm]['revision'],
              'tree':SPEC[arm]['tree'],'rustc':rustc,'source_inventory':sha(HERE/'payload'/('source-'+arm+'.json'))})
        if phase!='sql':seal(arm,phase)
        if phase=='sql':
            from package import package
            package(arm)
    finally:cleanup(arm)
