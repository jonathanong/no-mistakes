"""Supervised untimed preparation; every descendant inherits the owned session."""
import os
import subprocess
import time
from pathlib import Path
from common import require, write
require(os.environ.get('DIAG_SUPERVISED_PREPARATION')=='1','preparation_requires_owned_supervisor')
root=Path(os.environ['RUNNER_TEMP'])/'repeated-target-2022'
original_run=subprocess.run
command_number=0


def retained_run(argv,*args,**kwargs):
    """File-backed capture survives outer deadline termination of the worker."""
    global command_number
    command_number+=1
    prefix=root/'public/semantic-command-raw'/f'{command_number:04}'
    prefix.parent.mkdir(parents=True,exist_ok=True)
    capture=kwargs.pop('capture_output',False)
    stdout_pipe=kwargs.get('stdout')==subprocess.PIPE
    stderr_pipe=kwargs.get('stderr')==subprocess.PIPE
    if capture:
        require('stdout'not in kwargs and 'stderr'not in kwargs,'conflicting_capture')
        stdout_pipe=stderr_pipe=True
    text=kwargs.get('text',False)or kwargs.get('universal_newlines',False)or kwargs.get('encoding')is not None
    record={'argv':list(argv),'started_unix':time.time(),'timeout':kwargs.get('timeout'),'state':'running'}
    write(prefix.with_suffix('.json'),record)
    with prefix.with_suffix('.stdout').open('wb')as out,prefix.with_suffix('.stderr').open('wb')as err:
        if stdout_pipe:kwargs['stdout']=out
        if stderr_pipe:kwargs['stderr']=err
        try:
            result=original_run(argv,*args,**kwargs)
            record.update(state='completed',exit=result.returncode)
        except BaseException as error:
            record.update(state='failed',error_type=type(error).__name__)
            raise
        finally:
            record['ended_unix']=time.time();write(prefix.with_suffix('.json'),record)
    if stdout_pipe:
        raw=prefix.with_suffix('.stdout').read_bytes()
        result.stdout=raw.decode(kwargs.get('encoding')or'utf-8',kwargs.get('errors')or'strict')if text else raw
    if stderr_pipe:
        raw=prefix.with_suffix('.stderr').read_bytes()
        result.stderr=raw.decode(kwargs.get('encoding')or'utf-8',kwargs.get('errors')or'strict')if text else raw
    return result


subprocess.run=retained_run
from semantics import preflight
from pair import reviewed_stage
from napi_parity import run
from semantic_export import export
preflight(root)
reviewed_stage('napi-parity')
run(root)
export(root)
