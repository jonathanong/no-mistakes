"""A missing/failed or mismatched dependency prerequisite never admits controls."""

def verify(proof, candidate, lock_sha256):
    assert proof.get('complete') is True, 'JS dependency setup did not complete'
    assert proof.get('candidate') == candidate, 'JS source head mismatch'
    assert proof.get('lock_sha256') == lock_sha256, 'JS lockfile changed'
    assert proof.get('no_lifecycle_scripts') is True, 'Dependency lifecycle scripts not disabled'
    assert proof.get('no_native_build') is True, 'Unexpected native rebuild'
    assert proof.get('node_version') == 'v26.11.1', 'Node version mismatch'
    assert proof.get('pnpm_version') == '11.24.0', 'pnpm version mismatch'
