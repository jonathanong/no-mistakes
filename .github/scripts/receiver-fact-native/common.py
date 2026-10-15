"""Exact historical inputs and curated records for issues #2012 and #1972."""
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess

from budgets import cpu_quota, effective_memory, memory_limits

REVISIONS = {'baseline': '57c927df4c09f450f67248277d49070f9576ba3b',
             'candidate': '8d81aa148eb2e62773919a290dd21439a1616f42'}
TREES = {'baseline': '4deecf0efbb9c9c2478025434e4dbfb0ba2fc47f',
         'candidate': '0a768383587495ce34a587073758dbee0e747604'}
LOCK = 'bb9bf453fde21cda4e892358e38325def3b39046309538eacf5cfaac75e36d6c'
FIXTURE = 'test-cases/codebase-analysis/import-facts/fixture/shared-destructuring.mts'
FIXTURE_SHA = 'e4de44094ed0e6caa0b2b6aeb8eb2597749273a6d49133510bc798f1d4ac22ff'
PROBE = 'docs/performance/imported-receiver-facts-evidence/facts-probe.rs'
PROBE_SHA = '32cb4e841940366c284ceed4855c2936adee156485d2d4a8ea82dd718afaaa33'
FEATURES = ['default', 'mermaid-validation']


class ExperimentFailure(Exception):
    def __init__(self, code, **details):
        super().__init__(code)
        self.code = code
        self.details = details


def require(condition, code, **details):
    if not condition:
        raise ExperimentFailure(code, **details)


def failure_record(error, stage, **context):
    # Expected invariant names and numeric details are safe; raw exception text is not.
    code = error.code if isinstance(error, ExperimentFailure) else 'unexpected_exception'
    details = error.details if isinstance(error, ExperimentFailure) else {}
    return dict(status='incomplete', stage=stage, error_code=code,
                reason=code.replace('_', ' '), error_type=type(error).__name__,
                **context, details=details)


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def write(path, data):
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(json.dumps(data, indent=2) + '\n')


def resources(root, build=False):
    memory = {line.split(':')[0]: int(line.split()[1]) for line in
              Path('/proc/meminfo').read_text().splitlines() if line.split()[1].isdigit()}
    disk = shutil.disk_usage(root).free
    require(disk >= (8 if build else 1) * 1024**3, 'insufficient_free_disk',
            observed_bytes=disk, required_bytes=(8 if build else 1) * 1024**3)
    try:
        limits = memory_limits()
        available = effective_memory(memory['MemAvailable'] * 1024, limits)
    except (OSError, ValueError):
        raise ExperimentFailure('effective_cgroup_memory_budget_unreadable')
    require(available >= (8 if build else 1) * 1024**3, 'insufficient_effective_memory',
            observed_bytes=available, required_bytes=(8 if build else 1) * 1024**3)
    allowed = sorted(os.sched_getaffinity(0))
    require(len(allowed) >= 4, 'fewer_than_four_allowed_cpus', observed_cpus=len(allowed))
    try:
        limit = cpu_quota()
    except (OSError, ValueError):
        raise ExperimentFailure('effective_cpu_quota_unreadable')
    require(limit is None or limit >= 4, 'cpu_quota_below_four', observed_quota=limit)
    return {'free_disk_bytes': disk, 'proc_available_memory_kib': memory['MemAvailable'],
            'effective_available_memory_bytes': available, 'visible_cgroup_memory_limits': limits,
            'memory_budget_scope': 'minimum of proc MemAvailable and visible cgroup ancestor headroom; hidden host limits unknown',
            'allowed_cpus': allowed, 'selected_cpus': allowed[:4], 'cpu_quota': limit}


def identity(root):
    model = next((line.split(':', 1)[1].strip() for line in Path('/proc/cpuinfo').read_text().splitlines()
                  if line.startswith('model name')), platform.processor())
    return dict(resources(root), architecture=platform.machine(), kernel=platform.release(),
                cpu_model=model, image_os=os.environ.get('ImageOS'),
                image_version=os.environ.get('ImageVersion'), rayon_workers=4)


def command(args, cwd=None, env=None, log=None):
    result = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True, check=False)
    if log:
        Path(log).write_text(result.stdout + result.stderr)
    if result.returncode:
        raise ExperimentFailure('tool_command_nonzero', tool=args[0], exit_code=result.returncode)
    return result.stdout.strip()


def cleanup_target(root, profile):
    # Never follow a shared target or remove any historical evidence/worktree.
    target = root / ('compile-' + profile)
    require(profile in ('opt3', 'opt0') and target.parent == root, 'cleanup_target_not_owned')
    require(not target.is_symlink(), 'compilation_target_is_symlink')
    if target.exists():
        shutil.rmtree(target)
