import argparse,hashlib,json,os,pathlib,resource,signal,subprocess,time
p=argparse.ArgumentParser();p.add_argument('binary');p.add_argument('root');p.add_argument('output');p.add_argument('--source',required=True);p.add_argument('--limit',type=float,default=10);p.add_argument('--diagnostics',action='store_true');a=p.parse_args()
root=pathlib.Path(a.root);out=pathlib.Path(a.output);out.mkdir(parents=True)
env=dict(os.environ,XDG_RUNTIME_DIR=str(out/'runtime'),LLVM_PROFILE_FILE=str(out/'probe-%p.profraw'));env.pop('RAYON_NUM_THREADS',None)
pathlib.Path(env['XDG_RUNTIME_DIR']).mkdir(exist_ok=True)
argv=[a.binary,'--timeout','0','--lock-timeout','0','check','--root',str(root),'--config',str(root/'.no-mistakes.yml'),'--format','json','--jobs','1']
if a.diagnostics:argv+=['--progress','--verbose-timings']
identity={str(x.relative_to(root)):hashlib.sha256(x.read_bytes()).hexdigest() for x in root.iterdir() if x.is_file() and x.name in ('.no-mistakes.yml','root.mts','runtime.mts','excluded.test.mts','tsconfig.json')}
(out/'argv.json').write_text(json.dumps(argv,indent=2)+'\n');start=time.monotonic();censored=False
with (out/'stdout.json').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr,(out/'resource.jsonl').open('w') as resources:
 process=subprocess.Popen(argv,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
 while process.poll() is None:
  elapsed=time.monotonic()-start
  try:
   stat=pathlib.Path(f'/proc/{process.pid}/stat').read_text().split();status=pathlib.Path(f'/proc/{process.pid}/status').read_text().splitlines();rss=next(int(x.split()[1]) for x in status if x.startswith('VmRSS:'))
   resources.write(json.dumps({'elapsed':elapsed,'cpuSeconds':(int(stat[13])+int(stat[14]))/os.sysconf('SC_CLK_TCK'),'rssKiB':rss})+'\n');resources.flush()
  except (OSError,StopIteration):pass
  if elapsed>=a.limit:
   censored=True;os.killpg(process.pid,signal.SIGTERM)
   try:process.wait(timeout=10)
   except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait()
   break
  time.sleep(0.05 if a.limit<30 else 1)
 usage=resource.getrusage(resource.RUSAGE_CHILDREN)
 result={'exit':process.returncode,'censored':censored,'wallSeconds':time.monotonic()-start,'cpuSeconds':usage.ru_utime+usage.ru_stime,'peakRssKiB':usage.ru_maxrss,'performanceEligible':False,'purpose':'Exact-source native diagnostic/report verification; coverage-instrumented debug binary; concurrent build load. Not a matched performance comparison.','source':a.source,'binarySha256':hashlib.sha256(pathlib.Path(a.binary).read_bytes()).hexdigest(),'inputs':identity}
 (out/'status.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)
