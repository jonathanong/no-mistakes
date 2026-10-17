"""Identity-aware owned process cleanup; no bare reused-PID/group signalling."""
from pathlib import Path
import os,signal,time,subprocess

def identity(pid):
 proc=Path('/proc')/str(pid);f=(proc/'stat').read_text().rsplit(')',1)[1].split()
 return {'pid':pid,'starttime':int(f[19]),'sid':int(f[3]),'pgid':int(f[2]),'exe':os.readlink(proc/'exe'),'argv':[(v.decode(errors='surrogateescape')) for v in (proc/'cmdline').read_bytes().split(b'\0')[:-1]]}

def current_same(expected):
 try:return identity(expected['pid'])==expected
 except FileNotFoundError:return False

class OwnedProcess:
    def __init__(self, process):
        self.process=process
        self.sid=process.pid
        self.initial=None
        self.records=[]
        try:
            self.initial=identity(process.pid)
            assert self.initial['sid']==self.sid and self.initial['pgid']==self.sid
        except FileNotFoundError:
            # A command may naturally finish before its first /proc read.
            assert process.poll() is not None
    def members(self):
        result=[]
        for path in Path('/proc').iterdir():
            if not path.name.isdigit():continue
            try:
                fields=(path/'stat').read_text().rsplit(')',1)[1].split()
                if int(fields[3])!=self.sid:continue
                if fields[0]=='Z':
                    result.append({'pid':int(path.name),'starttime':int(fields[19]),
                        'sid':int(fields[3]),'pgid':int(fields[2]),'state':'Z'})
                else:
                    value=identity(int(path.name));value['state']=fields[0];result.append(value)
            except FileNotFoundError:
                # Skip only actual disappearance; an extant unresolved owner fails.
                try:remaining=(path/'stat').read_text()
                except FileNotFoundError:continue
                remaining=remaining.rsplit(')',1)[1].split()
                if int(remaining[3])==self.sid:
                    if remaining[0]=='Z':
                        result.append({'pid':int(path.name),'starttime':int(remaining[19]),
                            'sid':int(remaining[3]),'pgid':int(remaining[2]),'state':'Z'})
                    else:raise AssertionError(('Owned /proc identity unavailable',path.name))
        return result
    def check(self):
        if self.process.poll() is not None:return False
        assert self.initial is not None
        try:current=identity(self.process.pid)
        except FileNotFoundError:
            assert self.process.poll() is not None
            return False
        assert all(current[k]==self.initial[k]for k in ['pid','starttime','sid','pgid'])
        return True
    def finish(self):
        self.check()
        # Natural leader exit never triggers killpg on its now-reusable number.
        if self.check():
            os.killpg(self.sid,signal.SIGTERM)
        try:self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            if self.check():os.killpg(self.sid,signal.SIGKILL)
            self.process.wait(timeout=5)
        deadline=time.monotonic()+10
        while True:
            members=self.members()
            if not members:break
            assert self.initial is not None,'Uncaptured leader with unresolved descendants'
            for member in members:
                assert member['starttime']>=self.initial['starttime']
                assert member['sid']==self.initial['sid']
                if member['state']=='Z':continue
                expected={k:v for k,v in member.items()if k!='state'}
                if current_same(expected):
                    os.kill(member['pid'],signal.SIGKILL)
                    self.records.append(member)
                else:
                    try:current=identity(member['pid'])
                    except FileNotFoundError:continue
                    raise AssertionError(('Owned process changed identity before signal',member,current))
            assert time.monotonic()<deadline,'Owned descendants not fully reaped'
            time.sleep(.1)
        self.records.append({'leader_exit':self.process.returncode,'owned_sid_matches_after_cleanup':[],'proc_read_errors':[]})
        return self.records
