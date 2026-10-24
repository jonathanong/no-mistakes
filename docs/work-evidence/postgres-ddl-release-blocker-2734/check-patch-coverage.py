"""Check measured Rust diff lines against LCOV, without excluding filenames."""
import argparse
import json
import re
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument("lcov", type=Path)
parser.add_argument("--base", default="origin/main")
parser.add_argument("--minimum", type=float, default=98.0)
args = parser.parse_args()
root = Path(subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip())
base = subprocess.check_output(["git", "merge-base", args.base, "HEAD"], text=True).strip()
diff = subprocess.check_output(
    ["git", "-c", "core.quotepath=false", "diff", "--no-ext-diff", "--no-renames", "--unified=0", base, "--", "*.rs"],
    text=True,
)
changed = {}
current = None
for line in diff.splitlines():
    if line.startswith("+++ b/"):
        current = line[6:]
        changed.setdefault(current, set())
    elif line.startswith("@@") and current is not None:
        match = re.search(r"\+(\d+)(?:,(\d+))? @@", line)
        start, count = int(match[1]), int(match[2] or 1)
        changed[current].update(range(start, start + count))

coverage = {}
current = None
for line in args.lcov.read_text().splitlines():
    if line.startswith("SF:"):
        path = Path(line[3:])
        if not path.is_absolute():
            path = root / path
        try:
            current = str(path.resolve().relative_to(root))
        except ValueError:
            current = None
    elif line.startswith("DA:") and current is not None:
        number, hits, *_ = line[3:].split(",")
        lines = coverage.setdefault(current, {})
        number = int(number)
        lines[number] = lines.get(number, 0) + int(hits)

measured = covered = 0
missing = {}
for path, numbers in changed.items():
    for number in sorted(numbers):
        hits = coverage.get(path, {}).get(number)
        if hits is None:
            continue
        measured += 1
        if hits > 0:
            covered += 1
        else:
            missing.setdefault(path, []).append(number)
result = {
    "base": base,
    "measuredChangedLines": measured,
    "coveredChangedLines": covered,
    "coveragePercent": round(100 * covered / measured, 4) if measured else None,
    "requiredPercent": args.minimum,
    "uncoveredChangedLines": missing,
    "changedFilesWithoutLineRecords": sorted(path for path in changed if path not in coverage),
}
print(json.dumps(result, indent=2))
raise SystemExit(0 if measured and 100 * covered / measured >= args.minimum else 1)
