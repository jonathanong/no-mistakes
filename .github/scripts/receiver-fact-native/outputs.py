"""Unexpected child bytes stay private; published records contain hashes only."""
import hashlib


def curate(stdout, stderr, demand):
    expected = '128' if demand == 'values' else '0'
    matches = stdout == (expected + '\n').encode()
    private = {}
    if not matches:
        private['stdout'] = stdout
    if stderr:
        private['stderr'] = stderr
    public = {'signature': expected if matches else 'mismatch',
              'stdout_bytes': len(stdout), 'stdout_sha256': hashlib.sha256(stdout).hexdigest(),
              'stderr': 'child stderr withheld' if stderr else '',
              'stderr_bytes': len(stderr), 'stderr_sha256': hashlib.sha256(stderr).hexdigest()}
    return public, private
