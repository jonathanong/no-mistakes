"""Retain one action's identity and raw files before starting another sample."""
import hashlib
import json
import os
from pathlib import Path
import sys


def validate_records(record, previous):
    assert record["exit"] == 0
    assert record["sample"] == len(previous) + 1, "sample ordering changed"
    assert record["runPartIndex"] == len(previous), "upload index ordering changed"
    for other in previous:
        assert other["pid"] != record["pid"], "benchmark process was reused"
        assert other["binarySha256"] == record["binarySha256"]
        assert other["profileDirectory"] != record["profileDirectory"], "raw folder reused"


def record_sample(root, sample):
    directory = root / f"sample-{sample}"
    record = json.loads((directory / "sample.json").read_text())
    previous = [json.loads((root / f"sample-{index}" / "sample.json").read_text())
                for index in range(1, sample)]
    validate_records(record, previous)
    profile = Path(record["profileDirectory"])
    assert profile.is_relative_to(root), "raw profile escaped the retained evidence root"
    assert profile.is_dir()
    files = [path for path in profile.rglob("*") if path.is_file()]
    assert any("ExecutionTimestamps" in path.name for path in files), "missing marker timeline"
    assert any("MemtrackArtifact" in path.name for path in files), "missing raw memory events"
    record["runPartSuffix"] = {"executor": "memory", "run-part-index": sample - 1}
    record["rawFiles"] = {
        str(path.relative_to(root)): hashlib.sha256(path.read_bytes()).hexdigest()
        for path in files
    }
    (directory / "sample.json").write_text(json.dumps(record, indent=2) + "\n")


if __name__ == "__main__":
    root = Path(os.environ["FRONTEND_SMOKE_ROOT"])
    if sys.argv[1] == "all":
        assert os.environ["SAMPLE_RUN_ID"], "CodSpeed action did not return its shared run ID"
        for sample in range(1, 5):
            record_sample(root, sample)
        (root / "codspeed-run-id.txt").write_text(os.environ["SAMPLE_RUN_ID"] + "\n")
    else:
        record_sample(root, int(sys.argv[1]))
