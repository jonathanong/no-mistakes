#!/usr/bin/env python3
"""Verify the actual Linux addon has no ELF STATIC_TLS requirement."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


def validate_dynamic(text):
    if 'Dynamic section at offset' not in text or not re.search(r'\(NEEDED\)', text):
        raise ValueError('readelf did not identify a dynamic addon')
    if re.search(r'\bSTATIC_TLS\b', text):
        raise ValueError('addon requires STATIC_TLS; use local_dynamic_tls for dlopen')


def verify(addon, output):
    addon, output = Path(addon), Path(output)
    result = subprocess.run(['readelf', '-d', str(addon)], capture_output=True,
                            text=True, check=True, timeout=30)
    dump = result.stdout
    validate_dynamic(dump)
    output.joinpath('addon-dynamic.txt').write_text(dump)
    proof = {'addonSha256': hashlib.sha256(addon.read_bytes()).hexdigest(),
             'dynamicDumpSha256': hashlib.sha256(dump.encode()).hexdigest(),
             'command': ['readelf', '-d', addon.name], 'staticTls': False}
    output.joinpath('addon-tls.json').write_text(json.dumps(proof, indent=2) + '\n')


if __name__ == '__main__':
    verify(*sys.argv[1:])
