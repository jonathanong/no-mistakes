"""One bounded fresh process, with raw wait4 resources and CPU timeline."""
import json
import os
from pathlib import Path
import signal
import subprocess
import threading
import time

from common import CAP_SECONDS, digest

DIAGNOSTICS = ('[progress] ', '[timing] ', '[work] ', '[memory] ')


def cpu_seconds(pid):
    try:
        fields = Path(f'/proc/{pid}/stat').read_text().rpartition(')')[2].split()
        return (int(fields[11]) + int(fields[12])) / os.sysconf('SC_CLK_TCK')
    except (FileNotFoundError, ProcessLookupError):
        return None


def sample(binary, root, config, jobs, diagnostics, ledger, output, arm, ordinal):
    tag = f'{ordinal}-{arm}'
    runtime = (output / ('runtime-' + tag)).resolve()
    runtime.mkdir(mode=0o700)
    command = [str(binary), '--timeout', str(CAP_SECONDS)]
    if jobs != 'default':
        command += ['--jobs', jobs]
    if diagnostics == 'on':
        command += ['--progress', '--verbose-timings']
    command += ['check', '--root', str(root), '--config', str(config), '--format', 'json']
    if ledger:
        command += ['--include-sql-verdicts']
    env = dict(os.environ, XDG_RUNTIME_DIR=str(runtime))
    env.pop('RAYON_NUM_THREADS', None)
    stdout_path, stderr_path = output / (tag + '.json'), output / (tag + '.stderr')
    timeline, stop, deadline_hit = [], threading.Event(), threading.Event()
    with stdout_path.open('wb') as stdout, stderr_path.open('wb') as stderr:
        started = time.monotonic()
        process = subprocess.Popen(command, stdout=stdout, stderr=stderr, env=env,
                                   start_new_session=True)
        try:
            pidfd = os.pidfd_open(process.pid)
        except OSError:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise

        def observe():
            while not stop.wait(0.1):
                cpu = cpu_seconds(process.pid)
                if cpu is not None:
                    timeline.append([time.monotonic() - started, cpu])

        def deadline():
            if not stop.wait(CAP_SECONDS):
                deadline_hit.set()
                # The leader remains unreaped until this thread joins, so its
                # process-group identity cannot be reused by another command.
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                    if not stop.wait(5):
                        os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass

        threads = [threading.Thread(target=target, daemon=True)
                   for target in (observe, deadline)]
        for thread in threads:
            thread.start()
        leader_exited_at = None
        try:
            os.waitid(os.P_PIDFD, pidfd, os.WEXITED | os.WNOWAIT)
            leader_exited_at = time.monotonic()
        finally:
            stop.set()
            descendants = []
            for proc in Path('/proc').iterdir():
                if not proc.name.isdigit() or int(proc.name) == process.pid:
                    continue
                try:
                    fields = (proc / 'stat').read_text().rpartition(')')[2].split()
                    if int(fields[2]) == process.pid:
                        descendants.append(int(proc.name))
                except (FileNotFoundError, ProcessLookupError, PermissionError):
                    pass
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            for thread in threads:
                thread.join()
            _, status, usage = os.wait4(process.pid, 0)
            elapsed = (leader_exited_at or time.monotonic()) - started
            process.returncode = os.waitstatus_to_exitcode(status)
            os.close(pidfd)
    timeline = [point for point in timeline if point[0] < elapsed]
    timeline.append([elapsed, usage.ru_utime + usage.ru_stime])
    stderr = stderr_path.read_text(errors='replace')
    semantic = ''.join(line for line in stderr.splitlines(keepends=True)
                       if not line.startswith(DIAGNOSTICS))
    counts = {}
    for line in stderr.splitlines():
        if line.startswith('[work] '):
            metric, count = line[7:].rsplit(': ', 1)
            counts[metric] = int(count)
    eligible = process.returncode in (0, 1) and not descendants and not deadline_hit.is_set()
    try:
        json.loads(stdout_path.read_bytes())
    except (ValueError, UnicodeDecodeError):
        eligible = False
    return {'arm': arm, 'ordinal': ordinal, 'stdout': stdout_path.name,
            'command': command, 'exit': process.returncode,
            'orphanedDescendants': len(descendants), 'deadlineHit': deadline_hit.is_set(),
            'eligible': eligible, 'collectionCapSeconds': CAP_SECONDS,
            'wallSeconds': elapsed if eligible else None,
            'userSeconds': usage.ru_utime if eligible else None,
            'systemSeconds': usage.ru_stime if eligible else None,
            'peakRssKiB': usage.ru_maxrss if eligible else None,
            'censoredWallSeconds': None if eligible else elapsed,
            'stdoutSha256': digest(stdout_path), 'stderrSha256': digest(stderr_path),
            'semanticStderr': semantic, 'workCounts': counts, 'cpuTimeline': timeline,
            'cpuUtilization': [
                {'fromSeconds': a[0], 'toSeconds': b[0],
                 'cpuPerWall': (b[1] - a[1]) / (b[0] - a[0])}
                for a, b in zip(timeline, timeline[1:]) if b[0] > a[0]] if eligible else [],
            'resourceNote': 'Censored exits/signals have no comparable CPU/RSS maxima.'}
