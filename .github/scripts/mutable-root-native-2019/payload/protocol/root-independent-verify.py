"""Independent artifact/full-output verification; run after worker completion only."""
from pathlib import Path
import gzip,hashlib,json,io,subprocess,tarfile,tomllib

out=Path(__file__).resolve().parent
from protocol_guard import require_reviewed_pair
reviewed_plan=require_reviewed_pair(out, live=False)
artifacts=Path('/tmp/nm2019-current-main-pair-evidence')
original=Path('/tmp/nm2019-three-arm-evidence')
needle=str(original/'inputs').encode()
replacement=b'<inputs>'
def file_hash(path):
 with path.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def stream_identity(path,packed):
 raw_hash=hashlib.sha256();normal_hash=hashlib.sha256();raw_bytes=normal_bytes=0;pending=b''
 opener=gzip.open if packed else open
 with opener(path,'rb') as stream:
  while chunk:=stream.read(1024*1024):
   raw_hash.update(chunk);raw_bytes+=len(chunk);data=pending+chunk
   cut=max(0,len(data)-len(needle)+1)
   start=data.rfind(needle)
   if start>=0 and start<cut<start+len(needle):cut=start
   normalized=data[:cut].replace(needle,replacement)
   normal_hash.update(normalized);normal_bytes+=len(normalized);pending=data[cut:]
 normalized=pending.replace(needle,replacement);normal_hash.update(normalized);normal_bytes+=len(normalized)
 return dict(raw_sha256=raw_hash.hexdigest(),raw_bytes=raw_bytes,sha256=normal_hash.hexdigest(),bytes=normal_bytes)
# A synthetic boundary check validates normalization independently of saved reports.
sample=b'x'*(1024*1024-len(needle)//2)+needle+b'y'*100
probe=out/'normalization-selfcheck.tmp';assert not probe.exists();probe.write_bytes(sample)
try:
 identity=stream_identity(probe,False)
 assert identity['sha256']==hashlib.sha256(sample.replace(needle,replacement)).hexdigest()
finally:probe.unlink()
manifest={arm:json.loads((artifacts/arm/'manifest.json').read_text()) for arm in ('ARM_BASE','ARM_CAND')}
expected={'ARM_BASE':'e5ae00bd6353a28045a5344f4accac411a2ccd74','ARM_CAND':reviewed_plan['candidate_revision']}
for arm,m in manifest.items():
 assert m['revision']==expected[arm]
 for name,key in [('probe','binary_sha256'),('lib.rlib','library_sha256'),('source-manifest.json','source_manifest_sha256')]:
  assert file_hash(artifacts/arm/name)==m[key],(arm,name)
 assert m['compiler_artifact']['fresh'] is False
 assert m['compiler_artifact']['profile']==dict(opt_level='3',debuginfo=0,debug_assertions=False,overflow_checks=False,test=False)
 assert m['harness']==[dict(path=p.name,sha256=file_hash(p)) for p in sorted((original/'harness').glob('*.rs'))]
for key in ['rustc','features','release_profile','harness']:assert manifest['ARM_BASE'][key]==manifest['ARM_CAND'][key]
for key in ['binary_sha256','library_sha256']:assert manifest['ARM_BASE'][key]!=manifest['ARM_CAND'][key]

# Independently compare every frozen source byte against its actual Git revision.
repo=__import__('os').environ['GITHUB_WORKSPACE']
locks={}
for arm,m in manifest.items():
 source_rows=json.loads((artifacts/arm/'source-manifest.json').read_text())
 archive=subprocess.check_output(['git','archive',m['revision'],'Cargo.toml','Cargo.lock','crates/no-mistakes'],cwd=repo)
 with tarfile.open(fileobj=io.BytesIO(archive)) as tree:
  selected={x.name for x in tree if x.isfile() and (x.name in ['Cargo.toml','Cargo.lock','crates/no-mistakes/Cargo.toml'] or x.name.startswith('crates/no-mistakes/') and x.name.endswith('.rs'))}
  assert len(source_rows)==len({r['path'] for r in source_rows}) and {r['path'] for r in source_rows}==selected,'Incomplete source inventory'
  for row in source_rows:
   raw=tree.extractfile(row['path']).read()
   assert hashlib.sha256(raw).hexdigest()==row['sha256'],(arm,row['path'])
  raw=tree.extractfile('Cargo.lock').read()
 assert hashlib.sha256(raw).hexdigest()==m['lock_sha256']
 assert (out/'lockfiles'/f'{arm}.Cargo.lock').read_bytes()==raw
 lock=tomllib.loads(raw.decode())
 own=[p for p in lock['package'] if p['name']=='no-mistakes']
 assert len(own)==1
 assert own[0]['version']=={'ARM_BASE':reviewed_plan['baseline_version'],'ARM_CAND':reviewed_plan['candidate_version']}[arm]
 own[0]['version']='<own-package-version>'
 locks[arm]=lock
assert manifest['ARM_BASE']['lock_sha256']==manifest['ARM_CAND']['lock_sha256'], 'Fresh pair raw lock mismatch'
assert locks['ARM_BASE']==locks['ARM_CAND'],'Dependencies differ beyond own package version'

for row in json.loads((original/'input-manifest.json').read_text())['files']:
 assert file_hash(original/'inputs'/row['path'])==row['sha256']
semantic=json.loads((out/'semantic-manifest.json').read_text())
compressed=json.loads((out/'semantic-compression.json').read_text());assert len(compressed)==8
packed_by_raw={r['path']:r for r in compressed};records={}
for label,arms in semantic['outputs'].items():
 assert set(arms)=={'ARM_BASE','ARM_CAND'}
 records[label]={}
 for arm,saved in arms.items():
  relative=f'semantic-private/{label}-{arm}.stdout';packed=relative in packed_by_raw
  path=out/(packed_by_raw[relative]['gzip_path'] if packed else relative)
  computed=stream_identity(path,packed)
  assert (computed['sha256'],computed['bytes'])==(saved['sha256'],saved['bytes']),(label,arm)
  assert (out/f'semantic-private/{label}-{arm}.stderr').stat().st_size==0
  if packed:
   row=packed_by_raw[relative]
   assert (computed['raw_sha256'],computed['raw_bytes'])==(row['raw_sha256'],row['raw_bytes'])
   assert file_hash(path)==row['gzip_sha256'] and path.stat().st_size==row['gzip_bytes']
   assert row['lossless_roundtrip_verified'] is True
  records[label][arm]=computed
assert len(records)==19 and sum(len(v) for v in records.values())==38
prior={'ARM_BASE':(Path('/tmp/nm2019-arm2019-expected-prior'),'A22'),'ARM_CAND':(Path('/tmp/nm2019-arm2019-expected-prior'),'B23')}
prior_manifests={arm:json.loads((directory/'semantic-manifest.json').read_text()) for arm,(directory,_) in prior.items()}
facts=[label for label in records if label.startswith('facts-')];assert len(facts)==13
import os,sys
sys.path.insert(0, os.environ['ARM2019_SOURCE'])
from fact_parity import FullFactParity
full_fact_parity=FullFactParity(Path(os.environ['ARM2019_SOURCE']),prior['ARM_BASE'][0]/'semantic-manifest.json')
saved_comparisons=json.loads((out/'full-fact-comparisons.json').read_text())
assert set(saved_comparisons)==set(facts)
for label in facts:
 assert set(saved_comparisons[label])==set(prior)
 for arm in prior:
  relative=f'semantic-private/{label}-{arm}.stdout';packed=relative in packed_by_raw
  path=out/(packed_by_raw[relative]['gzip_path'] if packed else relative)
  with (gzip.open(path,'rb') if packed else path.open('rb')) as stream:
   data=stream.read().replace(needle,replacement)
  recomputed=full_fact_parity.compare(label,arm,data,semantic['outputs'][label][arm])
  assert recomputed==saved_comparisons[label][arm],(label,arm,'saved fact comparison differs')
for label in ['effects-0','effects-1','effects-2']:
 for arm,(_,old_arm) in prior.items():
  assert semantic['outputs'][label][arm]==prior_manifests[arm]['outputs'][label][old_arm],(label,arm,'prior full output')
graphs={}
for mode in ['off','work','no-calls']:
 graphs[mode]={}
 for arm,(directory,old_arm) in prior.items():
  raw=(out/f'semantic-private/graph-{mode}-{arm}.stdout').read_bytes()
  before=(directory/f'semantic-private/graph-{mode}-{old_arm}.stdout').read_bytes()
  graphs[mode][arm]=json.loads(raw.replace(needle,replacement))['semantic']
  assert graphs[mode][arm]==json.loads(before.replace(needle,replacement))['semantic'],(mode,arm)
for arm in prior:
 assert graphs['off'][arm]==graphs['work'][arm]
 assert not graphs['no-calls'][arm]['sites']
 assert graphs['no-calls'][arm]['nodes'] and graphs['no-calls'][arm]['edges']
signatures=json.loads((out/'timing-signatures.json').read_text());original=json.loads((original/'timing-signatures.json').read_text());assert len(signatures)==22
for label,values in signatures.items():assert values==dict(ARM_BASE=original[label]['A'],ARM_CAND=original[label]['B'])
report={'status':'PASS independently recomputed artifacts/full outputs; no timing/freeze/START','artifact_pair':{a:{k:m[k] for k in ['revision','binary_sha256','library_sha256','source_manifest_sha256','rustc','features','release_profile','lock_sha256']} for a,m in manifest.items()},'outputs':records,'full_output_records':38,'fact_prior_pairs':26,'fact_comparisons':saved_comparisons,'fact_policy':'Raw exact first; otherwise immutable prior-bound full tokens/counts with anonymous map/set entry-order equivalence','effects_prior_pairs':6,'graph_prior_pairs':6,'compressed_roundtrips':8,'timing_signatures':22,'root_normalization':'Only exact original input root replaced by <inputs>; compression identities use raw bytes','baseline_provenance':'Original immutable ARM_BASE actuale5 and fresh ARM_CAND actual reviewed candidate; original ARM_BASE/new ARM_CAND compilerfreshfalse receipts, no historical observation reuse','targets_present':{p:Path(p).exists() for p in ['/tmp/no-mistakes-2019/target','/tmp/no-mistakes-2019-base-current/target']},'limits':'No performance observations yet; prior adverse B20 retained separately and never pooled'}
p=out/'root-independent-verification.json';assert not p.exists();p.write_text(json.dumps(report,indent=2)+'\n')
print('PASS ARM_BASE/ARM_CAND metadata,38 full outputs,8 gzip roundtrips,full prior-arm identity,22 signatures. No timing.')
