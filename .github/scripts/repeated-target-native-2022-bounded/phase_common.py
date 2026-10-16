"""Bounded subprocesses and immutable source/artifact primitives."""
from pathlib import Path
import hashlib,json,os,subprocess,time,signal
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
SPEC=json.loads((HERE/'pair.json').read_text())
JOB_ROOT=Path(os.environ['RUNNER_TEMP'])/'repeated-target-2022'
LAYOUT_MODE=os.environ.get('DIAG_LAYOUT_MODE')
require_mode=LAYOUT_MODE in (None,'default','instrumented')
assert require_mode,'unknown_layout_mode'
WORK=JOB_ROOT/('layout-'+LAYOUT_MODE) if LAYOUT_MODE else JOB_ROOT
STATE_CONTEXT='layout-'+LAYOUT_MODE if LAYOUT_MODE else 'production'
if LAYOUT_MODE=='instrumented':SPEC['features']=SPEC['features']+['test-instrumentation']
ARTIFACTS=WORK/'closed'
ARM={'baseline':'baseline','candidate':'candidate'}

def require(value,message):
    if not value: raise AssertionError(message)

def sha(path):
    with Path(path).open('rb')as stream:return hashlib.file_digest(stream,'sha256').hexdigest()

def write(path,data):
    path=Path(path);path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(data,indent=2)+'\n')

def target_size(target):
    total=0
    if target and target.exists():
        for path in target.rglob('*'):
            try:
                if path.is_file():total+=path.stat().st_size
            except FileNotFoundError:pass
    return total

def run(argv,cwd=None,env=None,log=None,timeout=1500,check=True):
    import tempfile
    from deadline import remaining
    timeout=min(timeout,remaining(JOB_ROOT))
    require(timeout>0,'no_owned_command_budget')
    start=time.time();record={'argv':argv,'started_unix':start,'timeout_seconds':timeout}
    if log:Path(log).parent.mkdir(parents=True,exist_ok=True)
    stdout_file=Path(log).with_suffix('.stdout').open('wb+')if log else tempfile.TemporaryFile()
    stderr_file=Path(log).with_suffix('.stderr').open('wb+')if log else tempfile.TemporaryFile()
    process=None;reason=None
    try:
        process=subprocess.Popen(argv,cwd=cwd,env=env,stdout=stdout_file,stderr=stderr_file,start_new_session=os.environ.get('DIAG_SUPERVISED_PREPARATION')!='1')
        while process.poll()is None:
            target=Path(env['CARGO_TARGET_DIR'])if env and 'CARGO_TARGET_DIR'in env else None
            used=target_size(target)
            if time.time()-start>=timeout or used>=9_000_000_000:
                reason='deadline'if used<9_000_000_000 else'target9GBmargin'
                break
            time.sleep(.1 if not log else 2)
        if reason is None:record['exit']=process.wait()
    finally:
        if process is not None:
            # Reap the owned session even if its leader exited or the monitor raised.
            try:
                if os.environ.get('DIAG_SUPERVISED_PREPARATION')=='1':process.terminate()
                else:os.killpg(process.pid,signal.SIGTERM)
            except ProcessLookupError:pass
            try:process.wait(timeout=2)
            except subprocess.TimeoutExpired:pass
            try:
                if os.environ.get('DIAG_SUPERVISED_PREPARATION')=='1':process.kill()
                else:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            record['exit']=process.wait()
        record.update(stop_reason=reason,ended_unix=time.time())
        stdout_file.seek(0);stdout=stdout_file.read();stderr_file.seek(0);stderr=stderr_file.read()
        stdout_file.close();stderr_file.close()
        if log:write(Path(log).with_suffix('.json'),record)
    require(not check or(reason is None and record.get('exit')==0),'command failed: '+str(record))
    return stdout

def git(*args,cwd=ROOT):return run(['git',*args],cwd=cwd,timeout=90).decode().strip()

def api(path):return json.loads(run(['gh','api',path],timeout=90))

def environment(target):
    env={k:v for k,v in os.environ.items()if not k.startswith('CARGO_PROFILE_')and k not in
         ['RUSTC_WRAPPER','RUSTC_WORKSPACE_WRAPPER','RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS']}
    env.update(CARGO_TARGET_DIR=str(target),CARGO_BUILD_JOBS='2',CARGO_INCREMENTAL='0',
               RUSTUP_TOOLCHAIN='1.99.1',RUSTFLAGS='',CARGO_ENCODED_RUSTFLAGS='')
    return env
