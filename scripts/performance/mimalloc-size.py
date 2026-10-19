#!/usr/bin/env python3
"""Verify complete paired release identities before reporting four-target sizes."""
import hashlib
import json
from pathlib import Path
import re
import runpy
import sys

TLS = runpy.run_path(str(Path(__file__).with_name('mimalloc-tls.py')))
TARGETS = {'aarch64-apple-darwin': ('thin', '16'),
           'x86_64-unknown-linux-gnu': ('fat', '1'),
           'aarch64-unknown-linux-gnu': ('fat', '1'),
           'x86_64-pc-windows-msvc': ('thin', '16')}
IDENTITY = ('source', 'tree', 'lockSha256', 'rustc', 'cargo', 'node', 'profile', 'target', 'manifestRewrite')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_variant(folder, target, variant):
    proof = json.loads((folder / 'provenance.json').read_text())
    if proof.get('target') != target or proof.get('allocator') != variant:
        raise ValueError('release target or allocator provenance mismatch')
    if proof.get('manifestRewrite') != 'crate-type = ["rlib", "cdylib"]':
        raise ValueError('release manifest rewrite mismatch')
    features = 'default' if variant == 'mimalloc' else '--no-default-features --features mermaid-validation'
    if proof.get('features') != features:
        raise ValueError('release allocator feature mismatch')
    for key, length in [('source', 40), ('tree', 40), ('lockSha256', 64)]:
        if not re.fullmatch('[0-9a-f]{' + str(length) + '}', str(proof.get(key, ''))):
            raise ValueError('missing release source or lock identity')
    profile = proof['profile']
    lto, units = TARGETS[target]
    expected = {'CARGO_PROFILE_RELEASE_LTO': lto, 'CARGO_PROFILE_RELEASE_CODEGEN_UNITS': units,
                'CARGO_PROFILE_RELEASE_DEBUG': '0', 'CARGO_INCREMENTAL': '0',
                'RUSTC_WRAPPER': '', 'NO_MISTAKES_BUILD_NAPI': '1'}
    if set(profile) != set(expected) | {'RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS'}:
        raise ValueError('incomplete release flag inventory')
    if any(profile.get(k) != v for k, v in expected.items()):
        raise ValueError('release flags or profile mismatch')
    if any(profile.get(k) not in (None, '') for k in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')):
        raise ValueError('unexpected Rust compilation flags')
    binary = 'no-mistakes.exe' if target.endswith('windows-msvc') else 'no-mistakes'
    for name in (binary, 'no-mistakes.node'):
        record = proof['artifacts'][name]
        if record['sha256'] != sha(folder / name) or record['bytes'] != (folder / name).stat().st_size:
            raise ValueError('release artifact size or hash mismatch')
        if record['bytes'] <= 0:
            raise ValueError('empty release artifact')
    if target.endswith('linux-gnu'):
        evidence = json.loads((folder / 'addon-tls.json').read_text())
        dump = folder / 'addon-dynamic.txt'
        TLS['validate_dynamic'](dump.read_text())
        if (evidence.get('staticTls') is not False
                or evidence.get('addonSha256') != sha(folder / 'no-mistakes.node')
                or evidence.get('dynamicDumpSha256') != sha(dump)):
            raise ValueError('release TLS proof is not linked to the addon and readelf dump')
    return proof, binary


def summarize(root):
    root = Path(root)
    expected = {f'mimalloc-release-{target}-{variant}' for target in TARGETS
                for variant in ('system', 'mimalloc')}
    actual = {p.name for p in root.glob('mimalloc-release-*')}
    if actual != expected:
        raise ValueError('expected exactly four targets with both allocator variants')
    result, identities = [], set()
    for target in sorted(TARGETS):
        base, binary = read_variant(root / f'mimalloc-release-{target}-system', target, 'system')
        candidate, _ = read_variant(root / f'mimalloc-release-{target}-mimalloc', target, 'mimalloc')
        if any(not base.get(k) or base[k] != candidate.get(k) for k in IDENTITY):
            raise ValueError('paired release source, lock, compiler or flags differ')
        identities.add((base['source'], base['tree'], base['lockSha256']))
        row = {'target': target, 'valid': True, 'source': base['source']}
        for label, name in [('cli', binary), ('addon', 'no-mistakes.node')]:
            b, c = base['artifacts'][name], candidate['artifacts'][name]
            if b['sha256'] == c['sha256']:
                raise ValueError('allocator variants contain the same artifact')
            delta = c['bytes'] - b['bytes']
            row[label] = {'baselineBytes': b['bytes'], 'candidateBytes': c['bytes'],
                          'deltaBytes': delta, 'percentDelta': 100 * delta / b['bytes']}
        result.append(row)
    if len(identities) != 1:
        raise ValueError('release targets do not share one source revision')
    return result


if __name__ == '__main__':
    root = Path(sys.argv[1])
    result = summarize(root)
    (root / 'size-summary.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
