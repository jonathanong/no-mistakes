import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys

BASE = "4ddb9c08dbe8e06501a87ffce0ab8db4b10e76fc"
ALLOWED = ['crates/no-mistakes/src/codebase/dependencies/graph/tests/mod.rs', 'test-cases/codebase-analysis/value-flow-receiver-evaluation-order/fixture/src/repeated-parameter-phase.mts', 'crates/no-mistakes/src/codebase/dependencies/graph/tests/parent_snapshot_phase_diagnostic.rs', '.github/workflows/parent-snapshot-phase-diagnostic.yml', '.github/scripts/parent-snapshot-phase-provenance.py']
FIXTURE = "test-cases/codebase-analysis/value-flow-receiver-evaluation-order/fixture/src/repeated-parameter-phase.mts"
PROBE = "crates/no-mistakes/src/codebase/dependencies/graph/tests/parent_snapshot_phase_diagnostic.rs"

def git(*args):
    return subprocess.check_output(["git", *args], text=True).strip()

if sys.argv[1:] == ["--collect"]:
    result = json.loads(Path("diagnostic-result.json").read_text())
    marker = "2021_PARENT_PROBE "
    records = [json.loads(line.split(marker, 1)[1]) for line in
               Path("diagnostic-private/cargo.log").read_text().splitlines()
               if line.startswith(marker)]
    exit_status = int(os.environ["PROBE_EXIT_STATUS"])
    result["observations"] = records
    result["probe_exit_status"] = exit_status
    if exit_status:
        result["failure_summary"] = [
            line.replace(str(Path.cwd()), ".")[:300]
            for line in Path("diagnostic-private/cargo.log").read_text().splitlines()
            if re.match(r"^(error(?:\[E\d+\])?:|thread .* panicked)", line)
        ][:10]
    else:
        assert len(records) == 4, records
    Path("diagnostic-result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))
else:
    assert not sys.argv[1:]
    assert git("rev-parse", "HEAD^") == BASE
    changed = git("diff", "--name-only", BASE, "HEAD").splitlines()
    assert set(changed) == set(ALLOWED), changed
    assert hashlib.sha256(Path(FIXTURE).read_bytes()).hexdigest() == "37bbd3f899456e154b0cec2ddde447b6bd7545df440b863c9cae3013334f1894"
    result = {
        "runtime_parent": BASE,
        "runtime_parent_tree": git("rev-parse", BASE + "^{tree}"),
        "diagnostic_commit": git("rev-parse", "HEAD"),
        "production_and_lockfile_unchanged": True,
        "test_only_changed_files": changed,
        "fixture_sha256": hashlib.sha256(Path(FIXTURE).read_bytes()).hexdigest(),
        "probe_sha256": hashlib.sha256(Path(PROBE).read_bytes()).hexdigest(),
        "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
        "features": "all-features",
        "claim": "semantic observation only; no desired target assertion or timing measurement",
    }
    Path("diagnostic-result.json").write_text(json.dumps(result, indent=2) + "\n")
