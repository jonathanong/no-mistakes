"""Fresh same-run release NAPI addon only; no CLI or shared target."""
import time
ENTRY_WALL=time.time()
import json,os,shutil,sys
from pathlib import Path
from common import package_guard,command_deadline,git,inventory,machine,run,save,sha
import subreaper
P=Path(__file__).resolve().parent;root=Path(sys.argv[1]).resolve();pair=sys.argv[2];arm=sys.argv[3];out=Path(sys.argv[4]).resolve();out.mkdir(parents=True,exist_ok=True)
spec=json.loads((P/'protocol.json').read_text());expected=spec['pairs'][pair][arm];target=root/'target'
assert spec['status']=='REVIEWED_ASYNC_API_DIAGNOSTIC', 'DRAFT cannot build'
assert os.environ.get('PROTOCOL_SHA256')==sha(P/'protocol.json'),'exact protocol confirmation required'
package_guard(P,os.environ);deadline=command_deadline(ENTRY_WALL,os.environ)
assert not target.exists();assert git(root,'rev-parse','HEAD').decode().strip()==expected
assert git(root,'rev-parse','HEAD^{tree}').decode().strip()==spec['pairs'][pair]['trees'][arm]
before=inventory(root);save(out/'source-inventory.json',before);subreaper.enable()
env={k:v for k,v in os.environ.items()if not k.startswith(('CARGO_','RUSTFLAGS','RUSTC','RUSTDOC','LLVM_'))};env.update(CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_TARGET_DIR=str(target),RUSTUP_TOOLCHAIN='1.99.1')
try:
    m=machine();compiler=run(['rustc','+1.99.1','-Vv'],out/'compiler',min(20,deadline-time.monotonic()),env);assert compiler['exit']==0
    assert 'rustc 1.99.1 (58cf8f9c3 2026-10-13)' in (out/'compiler.stdout').read_text()
    row=run(['cargo','+1.99.1','rustc','--manifest-path',str(root/'Cargo.toml'),'--release','--locked','-p','no-mistakes','--lib','--crate-type','cdylib','--message-format=json'],out/'cargo',deadline-time.monotonic(),env,target)
    assert row['exit']==0
    artifacts=[json.loads(line)for line in (out/'cargo.stdout').read_text().splitlines()if line.startswith('{')]
    selected=[r for r in artifacts if r.get('reason')=='compiler-artifact' and r['target']['name']=='no_mistakes' and 'cdylib' in r['target']['crate_types']]
    assert [x['success']for x in artifacts if x.get('reason')=='build-finished']==[True]
    assert len(selected)==1 and selected[0]['fresh'] is False
    assert selected[0]['features']==['default','mermaid-validation']
    assert selected[0]['profile']==dict(opt_level='3',debuginfo=0,debug_assertions=False,overflow_checks=False,test=False)
    save(out/'cargo-artifacts.json',artifacts)
    save(out/'completed-dependencies.json',[{'path':str(p.relative_to(target)),'bytes':p.stat().st_size,'sha256':sha(p)}for p in sorted((target/'release/deps').iterdir())if p.is_file()])
    names=[Path(x) for x in selected[0]['filenames'] if x.endswith('.so')];assert len(names)==1;binary=names[0];shutil.copy2(binary,out/'addon.node');os.chmod(out/'addon.node',0o755)
    elf=(out/'addon.node').read_bytes();assert elf[:4]==b'\x7fELF' and int.from_bytes(elf[18:20],'little')==183
    assert inventory(root)==before
    save(out/'source-after.json',inventory(root))
    save(out/'producer.json',{'pair':pair,'arm':arm,'source':expected,'tree':git(root,'rev-parse','HEAD^{tree}').decode().strip(),'machine':m,'addon_sha256':sha(out/'addon.node'),'bytes':(out/'addon.node').stat().st_size,'cargo_selected':selected[0],'compiler':(out/'compiler.stdout').read_text(),'profile':'repository release opt3/fullLTO/cg1/debug0;default+mermaid','source_inventory_sha256':sha(out/'source-inventory.json'),'source_after_sha256':sha(out/'source-after.json'),'cargo_lock_sha256':sha(root/'Cargo.lock')})
finally:
    if target.exists():shutil.rmtree(target)
    save(out/'cleanup.json',{'target_absent':not target.exists(),'source_exact':inventory(root)==before})
    package_guard(P,os.environ)
