import concurrent.futures,hashlib,json,os,pathlib,subprocess,sys
base=pathlib.Path('/tmp/nm-2682-commands-evidence')
rows=json.loads((base/'fixture-comparisons-input.json').read_text())
phase=sys.argv[1]
binary='/tmp/nm-2682-commands-baseline' if phase=='before' else str(pathlib.Path('target/debug/no-mistakes').resolve())
def run(row):
 root,config=row
 key=hashlib.sha256((root+'|'+config).encode()).hexdigest()[:16]
 runtime='/tmp/nm-2682-commands-compare-'+key
 pathlib.Path(runtime).mkdir(exist_ok=True)
 env=dict(os.environ,XDG_RUNTIME_DIR=runtime)
 result=subprocess.run([binary,'check','--root',root,'--config',config,'--format','json'],capture_output=True,text=True,env=env)
 (base/(phase+'-'+key+'.json')).write_text(result.stdout)
 (base/(phase+'-'+key+'.stderr')).write_text(result.stderr)
 try: report=json.loads(result.stdout)
 except ValueError:report={}
 return dict(root=root,config=config,key=key,status=result.returncode,findings=len(report.get('rules',[])),advisories=len(report.get('advisories',[])),stdout=hashlib.sha256(result.stdout.encode()).hexdigest(),stderr=hashlib.sha256(result.stderr.encode()).hexdigest())
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:out=list(pool.map(run,rows))
(base/(phase+'-comparison.json')).write_text(json.dumps(out,indent=2)+'\n')
print(phase,len(out),'saved configurations',flush=True)
if phase=='after':
 before=json.loads((base/'before-comparison.json').read_text())
 changes=[]
 for old,new in zip(before,out):
  if any(old[k]!=new[k] for k in ['status','stdout','stderr']):changes.append(dict(root=new['root'],config=new['config'],beforeFindings=old['findings'],afterFindings=new['findings'],beforeAdvisories=old['advisories'],afterAdvisories=new['advisories'],beforeStatus=old['status'],afterStatus=new['status']))
 (base/'comparison-changes.json').write_text(json.dumps(changes,indent=2)+'\n')
 print(json.dumps(changes,indent=2))
