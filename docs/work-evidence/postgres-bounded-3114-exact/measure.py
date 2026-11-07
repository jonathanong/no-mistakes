import json,pathlib,subprocess,sys,resource,time,statistics,os,hashlib
repo=pathlib.Path.cwd()
mode=sys.argv[1]
label=os.environ.get('NM_MEASUREMENT_LABEL', mode)
binaries={'A':os.environ.get('NM_BASELINE_BINARY','/tmp/nm-3114-exact-main'),'B':os.environ.get('NM_CANDIDATE_BINARY','/tmp/nm-3114-exact-candidate')}
binary_hashes={k:hashlib.sha256(pathlib.Path(v).read_bytes()).hexdigest() for k,v in binaries.items()}
assert binary_hashes['A'] != binary_hashes['B'], 'baseline/candidate artifacts must differ'
expected={'A':{'r1':3,'r2':3,'r3':6,'r4':9,'off':0,'off-sql':1},'B':{'r1':2,'r2':2,'r3':4,'r4':4,'off':0,'off-sql':1}}
launches={}
for key,binary in binaries.items():
 path=pathlib.Path('/tmp/nm-3114-exact-launch')/key/'no-mistakes'
 path.parent.mkdir(parents=True,exist_ok=True)
 path.unlink(missing_ok=True)
 os.link(binary,path)
 launches[key]=str(path)
phases=['A'] if mode=='baseline' else (['B','A','A','B'] if mode=='reverse' else ['A','B','B','A'])
root=repo/'test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture'
cases=['off','off-sql'] if mode=='reverse' else ['r1','r2','r3','r4','off','off-sql']
samples=[]
for phase in ['A','B']:
 for shape in cases:
  cmd=[launches[phase],'check','--root',str(root),'--config',str(root/(shape+'.yml')),'--format','json']
  p=subprocess.run(cmd,capture_output=True,env={**os.environ,'XDG_RUNTIME_DIR':'/tmp/nm-3114-exact-runtime'})
  report=json.loads(p.stdout)
  assert p.returncode==int(expected[phase][shape]>0) and not p.stderr
  assert len(report['rules'])==expected[phase][shape]
  assert not report['warnings'] and not report['advisories']
for phase_index,phase in enumerate(phases):
 for iteration in range(50 if mode=="reverse" else 25):
  for shape in cases[iteration%len(cases):]+cases[:iteration%len(cases)]:
   cmd=[launches[phase],'check','--root',str(root),'--config',str(root/(shape+'.yml')),'--format','json']
   before=resource.getrusage(resource.RUSAGE_CHILDREN); start=time.perf_counter()
   p=subprocess.run(cmd,capture_output=True,env={**os.environ,'XDG_RUNTIME_DIR':'/tmp/nm-3114-exact-runtime'})
   wall=time.perf_counter()-start; after=resource.getrusage(resource.RUSAGE_CHILDREN)
   report=json.loads(p.stdout)
   assert p.returncode==int(expected[phase][shape]>0) and not p.stderr
   assert len(report['rules'])==expected[phase][shape], (phase,shape,report)
   assert not report['warnings'] and not report['advisories']
   samples.append({'binary':phase,'phase':phase_index,'sample':iteration,'case':shape,'userSeconds':after.ru_utime-before.ru_utime,'systemSeconds':after.ru_stime-before.ru_stime,'wallSeconds':wall,'status':p.returncode,'stdoutSha256':hashlib.sha256(p.stdout).hexdigest(),'findings':len(report['rules']),'advisories':len(report['advisories'])})
out={'argv0Basename':'no-mistakes','warmupPerBinaryPerCase':1,'binarySha256':binary_hashes,'mode':mode,'order':phases,'effort':'default','jobs':'default','samplesPerPhasePerCase':50 if mode=='reverse' else 25,'samples':samples}
pathlib.Path('/tmp/nm-3114-exact-cpu-'+label+'.json').write_text(json.dumps(out,indent=2)+'\n')
for binary in sorted(set(x['binary'] for x in samples)):
 for shape in cases:
  values=[x['userSeconds'] for x in samples if x['binary']==binary and x['case']==shape]
  print(binary,shape,'n',len(values),'user median',round(statistics.median(values),6),'mean',round(statistics.mean(values),6))
if mode=='reverse':
 rss=[]
 for phase_index,phase in enumerate(phases):
  for iteration in range(5):
   for shape in ['r4','off-sql']:
    cmd=[launches[phase],'check','--root',str(root),'--config',str(root/(shape+'.yml')),'--format','json']
    p=subprocess.run(['/usr/bin/time','-q','-f','%M','-o','/tmp/nm-3114-exact-rss.tmp',*cmd],capture_output=True,env={**os.environ,'XDG_RUNTIME_DIR':'/tmp/nm-3114-exact-runtime'})
    json.loads(p.stdout)
    rss.append({'binary':phase,'phase':phase_index,'sample':iteration,'case':shape,'peakRssKiB':int(pathlib.Path('/tmp/nm-3114-exact-rss.tmp').read_text().strip()),'status':p.returncode})
 pathlib.Path('/tmp/nm-3114-exact-rss-'+label+'.json').write_text(json.dumps(rss,indent=2)+'\n')
 for phase in ['A','B']:
  for shape in ['r4','off-sql']:
   rows=[x['peakRssKiB'] for x in rss if x['binary']==phase and x['case']==shape]
   print('RSS',phase,shape,'n',len(rows),'medianKiB',statistics.median(rows),'maxKiB',max(rows))
