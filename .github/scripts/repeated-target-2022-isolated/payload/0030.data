"""Use visible cgroup ancestors, not host RAM alone, as the resource budget."""
from pathlib import Path, PurePosixPath


def groups(controller):
    memberships = Path('/proc/self/cgroup').read_text().splitlines()
    mounts = Path('/proc/self/mountinfo').read_text().splitlines()
    for line in mounts:
        fields = line.split()
        separator = fields.index('-')
        kind = fields[separator + 1]
        version = 2 if kind == 'cgroup2' else 1
        if kind not in ('cgroup', 'cgroup2'):
            continue
        if version == 1 and controller not in fields[separator + 3].split(','):
            continue
        membership = next((m.split(':', 2)[2] for m in memberships
                           if (version == 2 and m.startswith('0::')) or
                           (version == 1 and controller in m.split(':', 2)[1].split(','))), None)
        if membership is None:
            continue
        mount = Path(fields[4])
        try:
            relative = PurePosixPath(membership).relative_to(PurePosixPath(fields[3]))
        except ValueError:
            relative = PurePosixPath('.')  # A cgroup namespace may expose its own root as '/'.
        directory = mount / str(relative)
        distance = 0
        while directory == mount or mount in directory.parents:
            yield version, distance, directory
            if directory == mount:
                break
            directory = directory.parent
            distance += 1


def memory_limits():
    limits = []
    for version, distance, directory in groups('memory'):
        maximum = directory / ('memory.max' if version == 2 else 'memory.limit_in_bytes')
        current = directory / ('memory.current' if version == 2 else 'memory.usage_in_bytes')
        if not maximum.exists():
            continue  # This hierarchy may not expose the memory controller.
        raw = maximum.read_text().strip()
        if raw == 'max':
            continue
        amount = int(raw)
        if amount >= 1 << 60:  # v1 represents unlimited with a very large sentinel.
            continue
        used = int(current.read_text().strip())
        limits.append({'version': version, 'ancestor_distance': distance,
                       'limit_bytes': amount, 'current_bytes': used,
                       'headroom_bytes': max(0, amount - used)})
    return limits


def effective_memory(proc_available_bytes, limits):
    return min([proc_available_bytes] + [limit['headroom_bytes'] for limit in limits])


def cpu_quota():
    limits = []
    for version, _distance, directory in groups('cpu'):
        if version == 2:
            path = directory / 'cpu.max'
            if not path.exists():
                continue
            amount, period = path.read_text().split()
            if amount == 'max':
                continue
        else:
            path = directory / 'cpu.cfs_quota_us'
            if not path.exists():
                continue
            amount = path.read_text().strip()
            if int(amount) < 0:
                continue
            period = (directory / 'cpu.cfs_period_us').read_text().strip()
        limits.append(int(amount) / int(period))
    return min(limits) if limits else None
