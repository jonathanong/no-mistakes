"""Count semantic progress; advancing elapsed-time heartbeats do not reset stalls."""
import json
import time


def fingerprint(line):
    if line.startswith('[work] '):
        return line
    if not line.startswith('[progress] '):
        return None
    try:
        row = json.loads(line[11:])
        return json.dumps({
            'started': row['startedSpans'], 'completed': row['completedSpans'],
            'active': sorted((span['id'], span['phase']) for span in row['active']),
            'workCounts': row.get('workCounts'), 'counters': row.get('counters'),
        }, sort_keys=True)
    except (ValueError, KeyError, TypeError):
        return None


class ProgressClock:
    def __init__(self):
        self.changed = time.monotonic()
        self.previous = None
        self.work = {}

    def observe(self, path):
        with path.open('rb') as stream:
            stream.seek(max(0, path.stat().st_size - 32768))
            lines = stream.read().decode(errors='replace').splitlines()
        latest, work = self.previous, dict(self.work)
        for line in lines:
            if line.startswith('[work] '):
                try:
                    metric, value = line[7:].rsplit(': ', 1)
                    work[metric] = int(value)
                except (ValueError, TypeError):
                    pass
                continue
            current = fingerprint(line)
            if current is not None:
                latest = current
        if latest != self.previous or work != self.work:
            self.changed = time.monotonic()
            self.previous, self.work = latest, work

    def stalled_seconds(self):
        return time.monotonic() - self.changed
