"""Linux adoption cleanup at command boundaries; never scans during samples."""
import ctypes
import os
from pathlib import Path
import signal
import subprocess
import time


def enable():
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(36, 1, 0, 0, 0) != 0:  # PR_SET_CHILD_SUBREAPER
        raise OSError(ctypes.get_errno(), 'PR_SET_CHILD_SUBREAPER')


def record(pid):
    path = Path('/proc') / str(pid)
    fields = (path / 'stat').read_text().rsplit(')', 1)[1].split()
    return {'pid': pid, 'state': fields[0], 'ppid': int(fields[1]),
            'pgid': int(fields[2]), 'sid': int(fields[3]), 'starttime': int(fields[19])}


def children():
    result = []
    for path in Path('/proc').iterdir():
        if not path.name.isdigit():
            continue
        try:
            child = record(int(path.name))
        except FileNotFoundError:
            continue
        if child['ppid'] == os.getpid():
            result.append(child)
    return result


def require_empty():
    assert not children(), 'Existing child ownership is unresolved; no command may start'


def finish(process, owner):
    """Reap the command and every positively adopted descendant, including new SIDs."""
    events = []
    initial = owner.initial
    if process.poll() is None:
        assert initial is not None and owner.check()
        os.killpg(owner.sid, signal.SIGTERM)
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            assert owner.check()
            os.killpg(owner.sid, signal.SIGKILL)
            process.wait(timeout=3)
    else:
        process.wait()
    deadline = time.monotonic() + 15
    sent = {}
    while True:
        adopted = children()
        if not adopted:
            break
        assert initial is not None, 'An uncaptured command cannot authorize adopted-child cleanup'
        assert time.monotonic() < deadline, 'Adopted child cleanup exceeded its bound'
        for child in adopted:
            assert child['starttime'] >= initial['starttime']
            assert child['ppid'] == os.getpid()
            try:
                current = record(child['pid'])
            except FileNotFoundError:
                continue
            assert all(current[k] == child[k] for k in ['pid', 'ppid', 'pgid', 'sid', 'starttime']), 'Child identity changed before cleanup'
            child = current
            if child['state'] == 'Z':
                pid, status = os.waitpid(child['pid'], os.WNOHANG)
                assert pid == child['pid']
                events.append({'adopted': child, 'reaped_status': status})
                continue
            key = (child['pid'], child['starttime'])
            if key not in sent:
                sig = signal.SIGTERM
                sent[key] = time.monotonic()
            elif time.monotonic() - sent[key] >= .2:
                sig = signal.SIGKILL
            else:
                continue
            # Signal only this verified direct child, never a guessed process group.
            os.kill(child['pid'], sig)
            events.append({'adopted': child, 'signal': int(sig)})
        time.sleep(.02)
    events.extend(owner.finish())
    assert not owner.members()
    require_empty()
    return events
