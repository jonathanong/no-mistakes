"""Owned subprocess and immutable Git inventory helpers; no remote polling."""
import hashlib,json,os,platform,re,resource,subprocess,time
from pathlib import Path
from process_owner import OwnedProcess
import subreaper
from target_size import target_bytes
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def save(p,v):Path(p).write_text(json.dumps(v,indent=2)+'\n')
def machine():
    assert platform.machine()=='aarch64', 'ARM required, no fallback'
    return {'uname':list(platform.uname()),'cpuinfo':Path('/proc/cpuinfo').read_text(),'affinity':sorted(os.sched_getaffinity(0)),'image':os.environ.get('ImageVersion'),'run':os.environ['GITHUB_RUN_ID'],'attempt':os.environ['GITHUB_RUN_ATTEMPT']}
class MeasuredProcess(subprocess.Popen):
    """Keep wait4 usage even when identity capture waits for a fast child."""
    def wait(self, timeout=None):
        deadline=time.monotonic()+timeout if timeout is not None else float('inf')
        while self.returncode is None:
            pid,status,usage=os.wait4(self.pid,os.WNOHANG)
            if pid:
                self.returncode=os.waitstatus_to_exitcode(status);self.measured_usage=usage
                break
            if time.monotonic()>=deadline:raise subprocess.TimeoutExpired(self.args,timeout)
            time.sleep(.001)
        return self.returncode

def run(argv,out,seconds,env=None,target=None):
    assert seconds>0,'command budget exhausted'
    out=Path(out);out.parent.mkdir(parents=True,exist_ok=True);subreaper.require_empty()
    started=time.monotonic();p=owner=None;row={'argv':argv,'started_unix':time.time()}
    try:
        with out.with_suffix('.stdout').open('wb') as stdout,out.with_suffix('.stderr').open('wb') as stderr:
            p=MeasuredProcess(argv,stdout=stdout,stderr=stderr,env=env,start_new_session=True)
            owner=OwnedProcess(p)
            while True:
                if p.returncode is not None:
                    usage=p.measured_usage;break
                pid,status,usage=os.wait4(p.pid,os.WNOHANG)
                if pid:
                    p.returncode=os.waitstatus_to_exitcode(status);break
                assert time.monotonic()-started<seconds,'command deadline'
                if target:assert target_bytes(Path(target))<9000000000,'target disk cap'
                assert stdout.tell()+stderr.tell()<268435456,'raw output cap'
                time.sleep(.025)
            if target:assert target_bytes(Path(target))<9000000000,'target disk cap'
            row.update(exit=p.returncode,wall=time.monotonic()-started,cpu=usage.ru_utime+usage.ru_stime,user=usage.ru_utime,sys=usage.ru_stime,rss_kib=usage.ru_maxrss)
    finally:
        if owner:row['cleanup']=subreaper.finish(p,owner)
        row['stdout_sha256']=sha(out.with_suffix('.stdout'));row['stdout_bytes']=out.with_suffix('.stdout').stat().st_size
        save(out.with_suffix('.json'),row)
    return row

def git(root,*args):return subprocess.check_output(['git','-c','core.fsmonitor=false','-c','core.hooksPath=/dev/null','-C',str(root),*args],timeout=30)
def inventory(root):
    root=Path(root);rows=[]
    for raw in git(root,'ls-files','--stage','-z').split(b'\0'):
        if not raw:continue
        meta,name=raw.split(b'\t',1);mode,blob,stage=meta.decode().split();assert stage=='0'
        path=os.fsdecode(name);p=root/path
        if mode=='160000':data=blob.encode()
        elif mode=='120000':data=os.fsencode(os.readlink(p))
        else:data=p.read_bytes()
        rows.append({'path':path,'mode':mode,'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)})
    return rows

def command(binary,fixture,config,timeout,diag,jobs):
    argv=[str(binary),'--timeout',str(timeout),'--lock-timeout','0']
    if jobs!='default':argv+=['--jobs',jobs]
    if diag=='on':argv+=['--timings','--verbose-timings']
    return argv+['check','--root',str(fixture),'--config',str(config),'--format','json']

def parity(rows):
    assert rows and all(r['exit']==0 for r in rows),'check exit must be zero'
    assert len({(r['stdout_sha256'],r['stdout_bytes'])for r in rows})==1,'full stdout mismatch'

def schedule():return [(block,order,index,arm)for block,order in enumerate(['ABBA','BAAB'])for index,arm in enumerate(order)]

def validate_producer(producer,arm,spec,m,binary_sha,binary_bytes,elf_machine):
    assert producer['source']==spec[arm],'source mismatch'
    assert producer['machine']['run']==m['run'] and producer['machine']['attempt']==m['attempt'],'artifact run mismatch'
    assert binary_sha==producer['cli_sha256'] and binary_bytes==producer['bytes'],'binary mismatch'
    assert elf_machine==183,'AArch64 required'
    assert producer['cargo_selected']['fresh'] is False,'fresh native build required'


def package_guard(package,env):
    approved=env.get('APPROVED_SOURCE_SHA','')
    assert re.fullmatch('[0-9a-f]{40}',approved),'approved source SHA required'
    assert approved==env.get('GITHUB_SHA')==env.get('GITHUB_WORKFLOW_SHA'),'workflow/source binding mismatch'
    manifest=package/'package-manifest.json'
    assert sha(manifest)==env.get('PACKAGE_MANIFEST_SHA256'),'package manifest mismatch'
    files=json.loads(manifest.read_text())
    actual={str(p.relative_to(package))for p in package.rglob('*')if p.is_file()}
    assert actual==set(files)|{'package-manifest.json'},'package file inventory mismatch'
    for name,digest in files.items():
        path=package/name
        assert not path.is_symlink() and path.resolve().is_relative_to(package.resolve())
        assert sha(path)==digest,('package file drift',name)

def command_deadline(started_wall,env,now=None):
    now=time.time() if now is None else now
    job=float(env['JOB_STARTED_UNIX'])
    assert 0<=now-job<3600 and job<=started_wall<=now,'job clock binding'
    # 150s job margin: 120s always upload plus30s cleanup; 60s within step for cleanup.
    absolute=min(job+3600-150,started_wall+2520-60)
    assert absolute>now,'budget exhausted before command'
    return time.monotonic()+absolute-now
