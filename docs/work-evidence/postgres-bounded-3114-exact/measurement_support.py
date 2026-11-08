import errno
import os
from pathlib import Path
import shutil
import tempfile


def launch_binaries(binaries, prefix):
    directory = tempfile.TemporaryDirectory(prefix=prefix)
    launches = {}
    try:
        for label, binary in binaries.items():
            path = Path(directory.name) / label / "no-mistakes"
            path.parent.mkdir()
            try:
                os.link(binary, path)
            except OSError as error:
                if error.errno != errno.EXDEV:
                    raise
                shutil.copy2(binary, path)
            launches[label] = str(path)
    except BaseException:
        directory.cleanup()
        raise
    return directory, launches


def validate_sample(report, status, stderr, expected_findings):
    if (
        status != int(expected_findings > 0)
        or stderr
        or len(report["rules"]) != expected_findings
        or any(report[group] for group in (
            "advisories", "warnings", "codebase", "integration", "queues", "react"
        ))
    ):
        raise ValueError("measurement sample has unexpected status or diagnostics")
