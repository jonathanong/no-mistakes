"""One-arm release build; two independent ARM jobs call this concurrently."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

from common import require, resources
from pair import HERE, ROOT, SPEC, reviewed_stage, source_inventory
from proposal_guard import git
from transfer import record


def digest(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(args, cwd=None, env=None, log=None):
    result=subprocess.run(args,cwd=cwd,env=env,capture_output=True)
    if log:
        Path(log).parent.mkdir(parents=True,exist_ok=True)
        Path(log).write_bytes(result.stdout+b'\n--- stderr ---\n'+result.stderr)
    require(result.returncode==0,'build_command_failed',tool=args[0],exit_code=result.returncode)
    return result.stdout


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


def cargo_environment(target):
    env=os.environ.copy()
    for key in list(env):
        if key.startswith('CARGO_PROFILE_') or key in ('RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER'):
            env.pop(key,None)
    env.update(CARGO_TARGET_DIR=str(target),CARGO_BUILD_JOBS=str(SPEC['build_jobs']),CARGO_INCREMENTAL='0',
               RUSTUP_TOOLCHAIN=SPEC['toolchain'],RUSTFLAGS='',CARGO_ENCODED_RUSTFLAGS='')
    return env


def checkout_arm(root, arm):
    source=root/arm/'source'
    require(not source.exists(),'owned_source_already_exists',arm=arm)
    source.parent.mkdir(parents=True,exist_ok=True)
    revision=SPEC[arm]['revision']
    run(['git','fetch','--no-tags','--depth=5','origin',revision],cwd=ROOT)
    run(['git','worktree','add','--detach',str(source),revision],cwd=ROOT)
    require(git(source,'rev-parse','HEAD')==revision,'checked_out_revision_changed')
    require(git(source,'rev-parse','HEAD^{tree}')==SPEC[arm]['tree'],'checked_out_tree_changed')
    require(digest(source/'Cargo.lock')==SPEC['lock_sha256'],'lock_changed')
    profile=tomllib.loads((source/'Cargo.toml').read_text())['profile']['release']
    require(profile['lto'] is True and profile['codegen-units']==1,'release_profile_changed')
    rows,proof=source_inventory(source)
    require(proof==SPEC['source_git_inventory_sha256'][arm],'source_inventory_changed')
    return source,rows,proof


def link_probe(source_file,output,rlib,target,env,extra=(),log=None):
    args=['rustc',str(source_file),'--edition=2021','-C','opt-level=3','-C','lto=true','-C','codegen-units=1',
          '-L','dependency='+str(target/'release/deps'),'--extern','no_mistakes='+str(rlib),*extra,'-o',str(output)]
    run(args,env=env,log=log or output.with_suffix('.link.log'))
    require(output.is_file(),'linked_probe_missing')


def cleanup_owned_target(target, owner):
    require(target.parent == owner and not target.is_symlink(), 'target_cleanup_not_owned')
    if target.exists():
        shutil.rmtree(target)


def build(arm):
    require(arm in ('baseline','candidate'),'unknown_arm')
    reviewed_stage('production-build')
    root=Path(os.environ['RUNNER_TEMP'])/'typed-executor-2060'
    root.mkdir(parents=True,exist_ok=True)
    machine=resources(root,build=True)
    require(machine['effective_available_memory_bytes']>=SPEC['min_ram_gib_per_build_job']*1024**3,'build_memory_below_reviewed_floor')
    rustc=run(['rustc','-Vv']).decode()
    require(rustc.startswith('rustc 1.99.1 (58cf8f9c3 ') and 'host: '+SPEC['target'] in rustc,'compiler_or_target_changed')
    source,rows,proof=checkout_arm(root,arm)
    work=root/arm;target=work/'target';private=work/'private';private.mkdir(exist_ok=True)
    require(not target.is_symlink(),'target_symlink')
    try:
        env=cargo_environment(target)
        cargo_args=['cargo','build','--locked','--release','-p','no-mistakes','--lib','--bin','no-mistakes','--message-format=json']
        production=run(cargo_args,cwd=source,env=env,log=private/'production-cargo.log')
        lib=extract_artifact(production,'rlib',source/'crates/no-mistakes/src/lib.rs')
        cli=extract_artifact(production,'bin',source/'crates/no-mistakes/src/main.rs')
        rlib=next(Path(p) for p in lib['filenames'] if p.endswith('.rlib'))
        cli_path=Path(cli['executable'])
        require(rlib.is_file() and cli_path.is_file(),'production_artifact_missing')
        native_args=['cargo','rustc','--locked','--release','-p','no-mistakes','--lib','--crate-type','cdylib','--message-format=json']
        native=run(native_args,cwd=source,env=env,log=private/'native-cargo.log')
        addon=extract_artifact(native,'cdylib',source/'crates/no-mistakes/src/lib.rs')
        so=next(Path(p) for p in addon['filenames'] if p.endswith('.so'))
        require(so.is_file(),'native_addon_missing')
        transfer=work/'transfer';transfer.mkdir()
        shutil.copyfile(cli_path,transfer/'cli')
        shutil.copyfile(so,transfer/'addon.node')
        link_probe(HERE/'timing-probe.rs',transfer/'timing-probe',rlib,target,env,log=private/'timing-link.log')
        link_probe(HERE/'graph-probe.rs',transfer/'graph-probe',rlib,target,env,['--cfg','alternatives_api'],log=private/'graph-link.log')
        (transfer/'cargo.jsonl').write_bytes(production+native)
        (transfer/'source-inventory.json').write_text(json.dumps({'revision':SPEC[arm]['revision'],'tree':SPEC[arm]['tree'],'rows':rows,'sha256':proof},sort_keys=True)+'\n')
        manifest=record(transfer,arm,SPEC[arm]['revision'],SPEC[arm]['tree'],proof,rustc,SPEC['features'],SPEC['release_profile'])
        manifest['machine']=machine
        (work/'build-record.json').write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
        return manifest
    finally:
        cleanup_owned_target(target, work)
