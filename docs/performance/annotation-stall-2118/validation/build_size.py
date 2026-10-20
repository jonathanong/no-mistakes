"""Measure the owned target without treating removed compiler temporaries as fatal."""
import os
import subprocess


def parse_size(code, stdout, stderr, target):
    removed = stderr.splitlines()
    transient = code == 1 and removed and all(
        line.startswith('du: cannot access ')
        and line.endswith(': No such file or directory') for line in removed)
    if code and not transient:
        raise ValueError(f'du failed: {stderr}')
    fields = stdout.rstrip('\n').split('\t')
    if len(fields) != 2 or not fields[0].isdigit() or fields[1] != str(target):
        raise ValueError('du did not return an owned-target byte count')
    return int(fields[0])


def target_size(target):
    result = subprocess.run(['du', '-sb', str(target)], capture_output=True, text=True,
                            env=dict(os.environ, LC_ALL='C'), timeout=30)
    return parse_size(result.returncode, result.stdout, result.stderr, target)
