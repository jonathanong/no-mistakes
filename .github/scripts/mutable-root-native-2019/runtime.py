"""Fresh paired ARM artifact setup and separately retained untimed controls."""
from common import *
from guard import reviewed
from package import verify
from machine import snapshot
import shutil,re

def prepare():
    eligibility=reviewed();machine=snapshot()
    original=Path('/tmp/nm2019-three-arm-evidence');prior=Path('/tmp/nm2019-arm2019-expected-prior')
    require(not original.exists()and not prior.exists(),'fixed input location already occupied')
    shutil.copytree(HERE/'payload/original',original);shutil.rmtree(original/'harness');materialize_harness(original/'harness');shutil.copy2(HERE/'payload/protocol/workloads.json',original/'workloads.json');shutil.copytree(HERE/'payload/expected-prior',prior)
    out=WORK/'evidence';shutil.copytree(HERE/'payload/protocol',out)
    for name,digest in SPEC['controls'].items():require(sha(out/name)==digest,'numerical controls changed')
    manifests={}
    for arm,label in ARM.items():
        m=verify(arm);dest=ARTIFACTS/label;require(not dest.exists(),'artifact location occupied')
        dest.mkdir(parents=True);closed=WORK/'closed'/arm
        for name in ['probe','lib.rlib','cli','addon.node','sql-probe']:shutil.copy2(closed/'binaries'/name,dest/name)
        shutil.copy2(closed/'source-manifest.json',dest/'source-manifest.json');write(dest/'manifest.json',m);manifests[label]=m
    versions={a:__import__('tomllib').loads(run(['git','show',SPEC[a]['revision']+':crates/no-mistakes/Cargo.toml']).decode())['package']['version']for a in ARM}
    write(out/'measurement-plan.json',{'status':'REVIEWED: untimed controls; measurement requires actual closure','baseline_revision':SPEC['baseline']['revision'],
          'candidate_revision':SPEC['candidate']['revision'],'baseline_version':versions['baseline'],'candidate_version':versions['candidate'],
          'scope':'Supplementary isolated ARM; no x86 observations pooled or gate cleared','source_commit':git('rev-parse','HEAD'),
          'run_id':os.environ['GITHUB_RUN_ID'],'artifact_manifests':{a:sha(ARTIFACTS/a/'manifest.json')for a in manifests},
          'controls':SPEC['controls'],'eligibility':eligibility,'machine':machine})
    return out

def controls(out):
    private=out/'extra-private';private.mkdir();extra={}
    for item in json.loads((HERE/'payload/extra/controls.json').read_text()):
        path=HERE/'payload/extra/inputs'/item['input'];require(path.read_bytes()==run(['git','show',SPEC['candidate']['revision']+':'+item['source_git_path']]),'extra input differs from candidate Git');golden=HERE/'payload/extra/goldens'/item['golden']
        require(sha(golden)==item['golden_sha256'],'extra golden changed')
        raw=run([str(ARTIFACTS/'ARM_CAND/probe'),str(path),*item['mode']],
                env=dict(os.environ,RAYON_NUM_THREADS='4'),log=private/item['label'],timeout=120)
        require(raw==golden.read_bytes(),'extra full raw mismatch: '+item['label']);extra[item['label']]=sha(golden)
    require(len(extra)==13,'extra control count')
    write(out/'extra-controls.json',extra)
    native={}
    for arm,label in ARM.items():
        scripts=['postgres-source-api.test.js','postgres-boolean-stringification-api.test.js','postgres-conditional-executors-api.test.js']
        for script in scripts:
            path='packages/no-mistakes/scripts/'+script
            expected=run(['git','show',SPEC['baseline']['revision']+':'+path])
            require(run(['git','show',SPEC['candidate']['revision']+':'+path])==expected,'SQL native controls differ between arms')
            require((ROOT/path).read_bytes()==expected,'SQL native control differs from source')
        argv=['node','--test','--test-reporter=tap',*[str(ROOT/'packages/no-mistakes/scripts'/s)for s in scripts]]
        raw=run(argv,env=dict(os.environ,NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(ARTIFACTS/label/'addon.node')),
                log=private/('native-'+arm),timeout=300)
        text=raw.decode();counts={}
        for key in ['tests','pass','fail','skipped']:
            values=re.findall(r'^# '+key+r' (\d+)$',text,re.M);require(len(values)==1,'missing TAP total');counts[key]=int(values[0])
        require(counts['tests']>0 and counts['pass']==counts['tests']and counts['fail']==counts['skipped']==0,'native tests not all real passes')
        require('compiled CJS and ESM retain legacy nested index identity strings across feature modes'in text,'missing CJS/ESM identity control')
        native[arm]={'counts':counts,'stdout_sha256':hashlib.sha256(raw).hexdigest()}
    write(out/'native-sql-parity.json',native)

def semantics():
    out=prepare();controls(out)
    for name in ['preflight.py','compress-semantics.py','signature-preflight.py','root-independent-verify.py','root-intended-delta.py']:
        run(['python3',str(out/name)],env=dict(os.environ,ARM2019_SOURCE=str(HERE)),log=out/('stage-'+name),timeout=1500)
    write(out/'semantic-closure.json',{'status':'PASS fresh ARM retained controls','run_id':os.environ['GITHUB_RUN_ID'],
          'source':git('rev-parse','HEAD'),'proofs':{p.name:sha(p)for p in out.glob('*.json')},'observations':False})
