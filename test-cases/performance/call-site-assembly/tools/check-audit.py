#!/usr/bin/env python3
"""Check complete-history quarantine and exact-RSS selection on saved traces."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("quarantine", type=Path, help="compiled quarantine.cpp executable")
args = parser.parse_args()
cases = Path(__file__).resolve().parents[1] / "heap-cases"
replay = Path(__file__).with_name("parse-heap-raw.py")
expected = json.loads((cases / "expected.json").read_text())
with tempfile.TemporaryDirectory(prefix="nm-heap-audit-") as directory:
    output = Path(directory)
    for name, wanted in expected.items():
        raw = output / f"{name}.raw"
        audit = output / f"{name}.json"
        subprocess.run([str(args.quarantine.resolve()), str(cases / f"{name}.raw"),
                        str(raw), str(audit)], check=True)
        actual = json.loads(audit.read_text())
        assert actual["duplicateEvents"] == wanted["duplicates"]
        assert actual["unknownFrees"] == wanted["unknown"]
        assert actual["quarantinedAddresses"] == wanted["addresses"]
        assert actual["payloadUpperBound"] == wanted["bound"]
        compressed = raw.with_suffix(".raw.zst")
        subprocess.run(["zstd", "-q", str(raw), "-o", str(compressed)], check=True)
        replayed = json.loads(subprocess.check_output([sys.executable, str(replay), str(compressed)]))
        assert replayed["peak_rss"] == wanted["rss"]
        assert replayed["live_bytes_at_rss_peak"] == wanted["live"]
        assert replayed["peak_clock"] == 1
        assert replayed["unknown_free_count"] == 0
        assert replayed["live_bytes_at_eof"] == 0
        assert replayed["live_pointer_count_at_eof"] == 0
        if name == "clean":
            assert raw.read_bytes() == (cases / "clean.raw").read_bytes()
        print(f"{name}: passed")
