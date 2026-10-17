"""Fresh same-run release CLI only; no addon or shared target."""
import time
ENTRY_WALL=time.time()
import json,os,shutil,sys
from pathlib import Path
from common import package_guard,command_deadline,git,inventory,machine,run,save,sha
import subreaper
P=Path(__file__).resolve().parent;root=Path(sys.argv[1]).resolve();arm=sys.argv[2];out=Path(sys.argv[3]).resolve();out.mkdir(parents=True,exist_ok=True)
spec=json.loads((P/'protocol.json').read_text());expected=spec[arm];target=root/'target'
assert spec['status']=='REVIEWED_ACTIVE_DEADLINE_DIAGNOSTIC', 'DRAFT cannot build'
assert os.environ.get('PROTOCOL_SHA256')==sha(P/'protocol.json'),'exact protocol confirmation required'
package_guard(P,os.environ);deadline=command_deadline(ENTRY_WALL,os.environ)
assert not target.exists();assert git(root,'rev-parse','HEAD').decode().strip()==expected
before=inventory(root);save(out/'source-inventory.json',before);subreaper.enable()
env={k:v for k,v in os.environ.items()if not k.startswith(('CARGO_','RUSTFLAGS','RUSTC','RUSTDOC','LLVM_'))};env.update(CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_TARGET_DIR=str(target),RUSTUP_TOOLCHAIN='1.99.1')
try:
    m=machine();compiler=run(['rustc','+1.99.1','-Vv'],out/'compiler',min(20,deadline-time.monotonic()),env);assert compiler['exit']==0
    assert 'rustc 1.99.1 (58cf8f9c3 2026-10-13)' in (out/'compiler.stdout').read_text()
    row=run(['cargo','+1.99.1','build','--manifest-path',str(root/'Cargo.toml'),'--release','--locked','--bin','no-mistakes','--message-format=json'],out/'cargo',deadline-time.monotonic(),env,target)
    assert row['exit']==0
    artifacts=[json.loads(line)for line in (out/'cargo.stdout').read_text().splitlines()if line.startswith('{')]
    selected=[r for r in artifacts if r.get('reason')=='compiler-artifact' and r['target']['name']=='no-mistakes' and r.get('executable')]
    assert len(selected)==1 and selected[0]['fresh'] is False
    save(out/'cargo-artifacts.json',artifacts)
    save(out/'completed-dependencies.json',[{'path':str(p.relative_to(target)),'bytes':p.stat().st_size,'sha256':sha(p)}for p in sorted((target/'release/deps').iterdir())if p.is_file()])
    binary=Path(selected[0]['executable']);shutil.copy2(binary,out/'cli');os.chmod(out/'cli',0o755)
    elf=(out/'cli').read_bytes();assert elf[:4]==b'\x7fELF' and int.from_bytes(elf[18:20],'little')==183
    assert inventory(root)==before
    save(out/'producer.json',{'arm':arm,'source':expected,'tree':git(root,'rev-parse','HEAD^{tree}').decode().strip(),'machine':m,'cli_sha256':sha(out/'cli'),'bytes':(out/'cli').stat().st_size,'cargo_selected':selected[0],'compiler':(out/'compiler.stdout').read_text(),'profile':'repository release opt3/fullLTO/cg1/debug0;default+mermaid','source_inventory_sha256':sha(out/'source-inventory.json'),'cargo_lock_sha256':sha(root/'Cargo.lock')})
finally:
    if target.exists():shutil.rmtree(target)
    save(out/'cleanup.json',{'target_absent':not target.exists(),'source_exact':inventory(root)==before})
    package_guard(P,os.environ)
