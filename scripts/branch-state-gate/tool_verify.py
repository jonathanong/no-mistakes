"""Verify the native replacement for only the unsupported Selene installer."""
import hashlib
import json
import os
from pathlib import Path
import subprocess


def validate(metadata, digest, expected_source, version):
    if not metadata.get('complete') or metadata.get('source') != expected_source:
        raise ValueError('Native tool producer source is incomplete or unexpected')
    if metadata.get('binary_sha256') != digest or metadata.get('tool_version') != '0.29.0':
        raise ValueError('Native Selene binary or version differs')
    if not metadata.get('default_features') or 'aarch64' not in metadata.get('target_triple', ''):
        raise ValueError('Expected the default-feature native ARM tool')
    if metadata.get('rustup_toolchain') != '1.99.1' or not metadata.get('rustc', '').startswith('rustc 1.99.1 '):
        raise ValueError('Native Selene was built with the wrong Rust toolchain')
    if version != 'selene 0.29.0':
        raise ValueError('Installed tool does not report the pinned Selene version')


def main():
    root = Path(os.environ['NATIVE_TOOL_ROOT']).resolve()
    output = Path(os.environ['MEASUREMENT_ROOT']).resolve()
    metadata = json.loads((root / 'provenance.json').read_text())
    binary = root / 'selene'
    with binary.open('rb') as source:
        digest = hashlib.file_digest(source, 'sha256').hexdigest()
    # Authenticate before executing the downloaded native artifact.
    validate(metadata, digest, os.environ['SOURCE_SHA'], metadata.get('version_output'))
    binary.chmod(0o755)
    version = subprocess.check_output([str(binary), '--version'], text=True, timeout=10).strip()
    validate(metadata, digest, os.environ['SOURCE_SHA'], version)
    metadata['installer_replacement'] = 'Cargo --locked default build replaces only aqua:Kampfkarren/selene'
    output.mkdir(parents=True, exist_ok=True)
    (output / 'tool-provisioning.json').write_text(json.dumps(metadata, indent=2)+'\n')


if __name__ == '__main__':
    main()
