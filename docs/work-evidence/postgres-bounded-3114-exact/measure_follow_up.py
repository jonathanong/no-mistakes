import argparse,csv,hashlib,json,os,statistics,tempfile,time
from pathlib import Path
from measurement_support import validate_sample
parser=argparse.ArgumentParser();parser.add_argument('--before',required=True);parser.add_argument('--after',required=True);parser.add_argument('--output',type=Path,required=True);parser.add_argument('--before-revision');parser.add_argument('--after-revision');options=parser.parse_args()
options.output.mkdir(parents=True,exist_ok=True)
root=Path.cwd();fixture=root/'test-cases/rules/postgres-bounded-statements/issue-3114-exact/fixture'
bins={'before':str(Path(options.before).resolve()),'after':str(Path(options.after).resolve())}
env=dict(os.environ,XDG_RUNTIME_DIR='/tmp/nm-3129-perf-runtime')
def run(side,args,expected_findings):
 with tempfile.TemporaryFile() as out,tempfile.TemporaryFile() as err:
  start=time.perf_counter();pid=os.posix_spawn(bins[side],[bins[side],*args],env,file_actions=[(os.POSIX_SPAWN_DUP2,out.fileno(),1),(os.POSIX_SPAWN_DUP2,err.fileno(),2)])
  _,status,usage=os.wait4(pid,0);wall=time.perf_counter()-start;out.seek(0);data=out.read();err.seek(0);stderr=err.read();report=json.loads(data)
  validate_sample(report,os.waitstatus_to_exitcode(status),stderr,expected_findings)
  return dict(userSeconds=usage.ru_utime,systemSeconds=usage.ru_stime,wallSeconds=wall,peakRssKiB=usage.ru_maxrss,exitCode=os.waitstatus_to_exitcode(status),stdoutSha256=hashlib.sha256(data).hexdigest(),stderrSha256=hashlib.sha256(stderr).hexdigest(),findings=len(report['rules']),advisories=len(report['advisories']),warnings=len(report['warnings']))
def args(config):return ['check','--root',str(fixture),'--config',config+'.yml','--format','json','--jobs','1']
def sha(path):
 with open(path,'rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
result=dict(baseline={'revision':options.before_revision,'verification':'unverified caller label'},candidate={'revision':options.after_revision,'verification':'unverified caller label'},profile='caller-provided executables; CLI jobs1',counters='not measured by this harness',binaryHashes={k:sha(v) for k,v in bins.items()},controls={},workloads={})
assert len(set(result['binaryHashes'].values()))==2
for case,count in [('r1',1),('r2',2),('r3',4),('r4',2),('off',0),('off-sql',1)]:
 a=run('before',args(case),count);b=run('after',args(case),count);assert (a['stdoutSha256'],a['stderrSha256'],a['exitCode'])==(b['stdoutSha256'],b['stderrSha256'],b['exitCode']),case
 result['controls'][case]={'before':a,'after':b}
for label,cmd,repeats,expected in [('follow-up',args('r5'),5,{'before':11,'after':6}),('rule-off',args('off-sql'),5,{'before':1,'after':1}),('repository',['check','--root',str(root),'--format','json','--jobs','1'],2,{'before':0,'after':0})]:
 run('before',cmd,expected['before']);run('after',cmd,expected['after']);samples={'before':[],'after':[]}
 for _ in range(repeats):
  for side in ['before','after','after','before']:samples[side].append(run(side,cmd,expected[side]))
 if label=='follow-up':assert all(x['findings']==11 for x in samples['before']) and all(x['findings']==6 for x in samples['after'])
 else:assert {x['stdoutSha256'] for x in samples['before']}=={x['stdoutSha256'] for x in samples['after']}
 summary={side:{key:statistics.mean(x[key] for x in rows) for key in ['userSeconds','wallSeconds','peakRssKiB']} for side,rows in samples.items()}
 result['workloads'][label]={'samples':samples,'summary':summary};print(label,json.dumps(summary),flush=True)
(options.output/'follow-up.json').write_text(json.dumps(result,indent=2)+'\n')
with open(options.output/'follow-up.csv','w',newline='') as f:
 w=csv.DictWriter(f,lineterminator='\n',fieldnames=['workload','side','sample',*next(iter(next(iter(result['workloads'].values()))['samples'].values()))[0].keys()]);w.writeheader()
 for label,data in result['workloads'].items():
  for side,rows in data['samples'].items():
   for i,row in enumerate(rows):w.writerow(dict(workload=label,side=side,sample=i+1,**row))
