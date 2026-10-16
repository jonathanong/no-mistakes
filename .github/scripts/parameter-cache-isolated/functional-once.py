import hashlib,json,os,pathlib,shutil,subprocess,time
from process_owner import OwnedProcess
from node_test_counts import require_tests
D=pathlib.Path(__file__).resolve().parent;R=pathlib.Path('/tmp/no-mistakes-2076-prospective2087');O=D/'functional';O.mkdir();deadline=time.monotonic()+600;ledger=[]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(argv,label,env):
 p=None;owner=None
 try:
  with(O/(label+'.stdout')).open('wb')as out,(O/(label+'.stderr')).open('wb')as err:
   p=subprocess.Popen(argv,cwd=R,env=env,stdout=out,stderr=err,start_new_session=True);owner=OwnedProcess(p)
   while p.poll()is None:
    assert time.monotonic()<deadline;time.sleep(.2)
  assert p.wait()==0
 finally:
  if p:assert owner is not None;owner.finish();assert not owner.members()
  ledger.append({'argv':argv,'label':label,'exit':p.returncode if p else None});(O/'ledger.json').write_text(json.dumps(ledger,indent=2))
 return(O/(label+'.stdout')).read_bytes()
node=pathlib.Path(subprocess.check_output(['node','-p','process.execPath'],text=True).strip()).resolve()
source_files=[R/'packages/no-mistakes'/p for p in ['index.js','index.mjs','scripts/query-reached-per-item-api.test.js','scripts/namespace-types-api.test.js','scripts/type-import-order-api.test.js']]
pins={str(p):sha(p)for p in source_files}
(O/'runtime-provenance.json').write_text(json.dumps({'node':str(node),'node_sha256':sha(node),'version':subprocess.check_output([str(node),'--version'],text=True).strip(),'sources':pins},indent=2))
records={};raw={}
for arm in ['baseline','candidate']:
 A=pathlib.Path('/tmp/nm2076-prospective2087-native/candidate') if arm=='candidate' else pathlib.Path('/tmp/nm-shared-baseline-2087-c896-final-artifacts');manifest=json.loads((A/'closed-artifacts.json').read_text())
 for label,name in [('cli','cli'),('addon','addon.node')]:assert sha(A/name)==manifest['artifacts'][label]['sha256']
 env=os.environ.copy();env.update(NO_MISTAKES_TEST_NAPI_ADDON_PATH=str(A/'addon.node'),RAYON_NUM_THREADS='4',XDG_RUNTIME_DIR=str(O/'runtime'))
 pathlib.Path(env['XDG_RUNTIME_DIR']).mkdir(mode=0o700,exist_ok=True);pathlib.Path(env['XDG_RUNTIME_DIR']).chmod(0o700)
 records[arm]=run([str(node),'--test','packages/no-mistakes/scripts/query-reached-per-item-api.test.js','packages/no-mistakes/scripts/namespace-types-api.test.js','packages/no-mistakes/scripts/type-import-order-api.test.js'],arm+'-api',env).decode()
 require_tests(records[arm],8)
 snapshot=O/(arm+'-prepared-result.json')
 run([str(node),str(D/'public-reports.cjs'),str(snapshot)],arm+'-prepared',env)
 raw[arm]=snapshot.read_bytes()
 for workload in json.loads((D/'performance-source-plan.json').read_text())['workloads']:
  name=workload['id'];data=run([str(A/'cli'),*workload['argv']],arm+'-'+name,env)
  if arm=='baseline':raw[name]=data
  else:assert data==raw[name]
assert raw['baseline']==raw['candidate']
assert all(sha(pathlib.Path(p))==digest for p,digest in pins.items())
(O/'actual-functional-result.json').write_text(json.dumps({'status':'PASS: both compiled native arms, CJS/ESM/prepared full reports and CLI raw equality','native_tests_per_arm':8,'prepared_bytes':len(raw['baseline']),'artifact_manifests':{a:sha((pathlib.Path('/tmp/nm2076-prospective2087-native/candidate') if a=='candidate' else pathlib.Path('/tmp/nm-shared-baseline-2087-c896-final-artifacts'))/'closed-artifacts.json')for a in ['baseline','candidate']}},indent=2))
