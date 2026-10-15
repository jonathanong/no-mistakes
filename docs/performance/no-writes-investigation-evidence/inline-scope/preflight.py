import hashlib,json,pathlib,subprocess,os
out=pathlib.Path('<LOCAL>/nm1972-query-reuse-evidence');root=pathlib.Path('<LOCAL>/no-mistakes-1972')
cases={'no_writes':'test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts','writes':'test-cases/codebase-analysis/import-facts/fixture/many-member-writes.mts','shared_histories':'test-cases/codebase-analysis/import-facts/fixture/many-late-defaults.mts','jsx':'test-cases/ast-snippets/selectors/fixture/app-text-targets.tsx'}
for arm in ['baseline','candidate']:
 p=out/f'{arm}-manifest.json';d=json.loads(p.read_text());d['binary_sha256']=hashlib.sha256((out/f'{arm}-facts').read_bytes()).hexdigest();p.write_text(json.dumps(d,indent=2)+'\n')
assert json.loads((out/'baseline-manifest.json').read_text())['binary_sha256']!=json.loads((out/'candidate-manifest.json').read_text())['binary_sha256']
rows=[]
for case,relative in cases.items():
 source=root/relative
 for demand in ['values','imports']:
  pair=[]
  for arm in ['baseline','candidate']:
   dump=out/f'{case}-{demand}-{arm}.dump'
   with dump.open('wb') as stdout:
    result=subprocess.run([str(out/f'{arm}-facts'),str(source),demand,'1','dump'],stdout=stdout,stderr=subprocess.PIPE,env=dict(os.environ,RAYON_NUM_THREADS='4'))
   assert result.returncode==0 and not result.stderr,(arm,case,demand,result.stderr)
   record={'arm':arm,'case':case,'demand':demand,'fixture':relative,'fixture_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'fixture_bytes':source.stat().st_size,'fact_dump_bytes':dump.stat().st_size,'fact_dump_sha256':hashlib.file_digest(dump.open('rb'),'sha256').hexdigest()}
   rows.append(record);pair.append(record);dump.unlink()
  assert pair[0]['fact_dump_sha256']==pair[1]['fact_dump_sha256'],(case,demand,pair)
(out/'semantic-parity.json').write_text(json.dumps(rows,indent=2)+'\n')
print('All 8 full-fact dump pairs match')
