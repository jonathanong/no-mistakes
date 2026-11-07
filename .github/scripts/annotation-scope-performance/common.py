"""Provenance and portable measurements for the public annotation fixtures."""
import hashlib
import json
import re
import subprocess
from pathlib import Path


def sha(value):
    if not re.fullmatch(r"[0-9a-f]{40}", value):
        raise ValueError("expected a lowercase 40-character commit SHA")
    return value


def command(*args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True, timeout=60).strip()


def refs(base, head):
    base, head = sha(base), sha(head)
    return sha(command("git", "merge-base", base, head)), head


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def rss_kib(value, system):
    return value / 1024 if system == "Darwin" else value


def distinct(left, right):
    if left == right:
        raise ValueError("identical binaries cannot establish a before/after comparison")


def descendant_usage(cpu_seconds, peak_rss_kib):
    if cpu_seconds < .1 or peak_rss_kib < 48 * 1024:
        raise ValueError("wait4 did not account for the completed descendant's CPU/RSS")


def parity(expected, actual, subject="public fixture report or exit status"):
    if expected != actual:
        raise ValueError(f"{subject} differs")


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
