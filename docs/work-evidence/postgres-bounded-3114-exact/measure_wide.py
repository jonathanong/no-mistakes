import pathlib,subprocess,resource,time,statistics,json,os,hashlib
repo=pathlib.Path.cwd();root=repo/'test-cases/rules/postgres-bounded-statements/fixture';config=repo/'test-cases/performance/postgres-bounded-3114'
exe={'A':'/tmp/nm-3114-main-final','B':'/tmp/nm-3114-candidate-final'};launch={}
for b in exe:
 p=pathlib.Path('/tmp/nm-3114-wide-launch')/b/'no-mistakes';p.parent.mkdir(parents=True,exist_ok=True);p.unlink(missing_ok=True);os.link(exe[b],p);launch[b]=str(p)
mode=__import__('sys').argv[1];phases=['A','B','B','A'] if mode=='balanced' else ['B','A','A','B'];cases=['on','off'] if mode=='balanced' else ['off'];rows=[]
expected={'A':{'on':1056,'off':1},'B':{'on':1053,'off':1}};advisories={'on':10,'off':26}
for phase,b in enumerate(phases):
 for sample in range(25 if mode=='balanced' else 50):
  for c in cases:
   before=resource.getrusage(resource.RUSAGE_CHILDREN);start=time.perf_counter();p=subprocess.run([launch[b],'check','--root',str(root),'--config',str(config/(c+'.yml')),'--format','json'],capture_output=True,env={**os.environ,'XDG_RUNTIME_DIR':'/tmp/nm-3114-exact-runtime'});wall=time.perf_counter()-start;after=resource.getrusage(resource.RUSAGE_CHILDREN)
   d=json.loads(p.stdout);assert p.returncode==1 and not p.stderr and len(d['rules'])==expected[b][c] and len(d['advisories'])==advisories[c] and not d['warnings']
   rows.append({'binary':b,'phase':phase,'sample':sample,'case':c,'userSeconds':after.ru_utime-before.ru_utime,'systemSeconds':after.ru_stime-before.ru_stime,'wallSeconds':wall,'stdoutSha256':hashlib.sha256(p.stdout).hexdigest()})
pathlib.Path('/tmp/nm-3114-wide-'+mode+'.json').write_text(json.dumps(rows,indent=2)+'\n')
for c in cases:
 for k in ['userSeconds','wallSeconds']:
  print(mode,c,k,[(b,round(statistics.mean(x[k] for x in rows if x['binary']==b and x['case']==c)*1000,3)) for b in exe])
