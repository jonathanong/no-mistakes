"""Report changed executable Rust lines; retain unmapped files for review."""
import json
from pathlib import Path
import re
import subprocess
import sys

root, evidence, base, head = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], sys.argv[4]
hits = {}
filename = None
for line in (evidence / 'coverage.lcov').read_text().splitlines():
    if line.startswith('SF:'):
        path = Path(line[3:])
        if not path.is_absolute():
            path = root / path
        try:
            filename = str(path.resolve().relative_to(root))
        except ValueError:
            filename = None
        if filename is not None:
            hits.setdefault(filename, {})
    elif line.startswith('DA:') and filename is not None:
        number, count, *_ = line[3:].split(',')
        hits[filename][int(number)] = hits[filename].get(int(number), 0) + int(count)
patch = subprocess.check_output(['git', 'diff', '--unified=0', '--no-color', base, head,
                                 '--', '*.rs'], cwd=root, text=True)
changed, filename = {}, None
for line in patch.splitlines():
    if line.startswith('+++ b/'):
        filename = line[6:]
    elif line.startswith('@@') and filename:
        match = re.search(r'\+(\d+)(?:,(\d+))?', line)
        start, count = int(match.group(1)), int(match.group(2) or 1)
        changed.setdefault(filename, set()).update(range(start, start + count))
rows, unrepresented = [], []
for filename, lines in sorted(changed.items()):
    if not lines:
        continue
    if filename not in hits:
        unrepresented.append(filename)
    data = hits.get(filename, {})
    rows.append({'file': filename,
                 'covered': sorted(line for line in lines if data.get(line, 0) > 0),
                 'missing': sorted(line for line in lines if line in data and data[line] == 0)})
covered = sum(len(row['covered']) for row in rows)
missing = sum(len(row['missing']) for row in rows)
percentage = 100 * covered / (covered + missing) if covered + missing else None
result = {'base': base, 'head': head, 'covered': covered, 'missing': missing,
          'percentage': percentage, 'floor': 98, 'files': rows,
          'unrepresentedChangedFiles': unrepresented,
          'note': 'Unrepresented files require review against normal test/fixture exclusions and executable-source coverage; no custom filename exclusion is used.',
          'numericGatePassed': percentage is not None and percentage >= 98}
(evidence / 'patch-coverage.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({key: result[key] for key in ['covered', 'missing', 'percentage', 'numericGatePassed']}))
sys.exit(0 if result['numericGatePassed'] else 1)
