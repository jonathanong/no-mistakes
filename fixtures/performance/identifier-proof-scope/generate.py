#!/usr/bin/env python3
"""Materialize the saved dense template into an explicit benchmark directory."""
import argparse
import hashlib
import json
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--out', required=True, type=Path)
parser.add_argument('--modules', default=2048, type=int)
parser.add_argument('--rows', default=256, type=int)
parser.add_argument('--jsx', default=128, type=int)
args = parser.parse_args()
source = Path(__file__).resolve().parent
root = args.out.resolve()
(root / 'src').mkdir(parents=True, exist_ok=True)
rows = '\n'.join(f"  {{ code: 'row_{i}', label: 'Label {i}', group: 'group_{i % 9}' }}," for i in range(args.rows))
jsx = '\n'.join(f'    <span data-key="{i}" data-code={{row.code}} data-group={{row.group}}>{{row.label}}</span>' for i in range(args.jsx))
template = (source / 'template.tsx').read_text().replace('__ROWS__', rows).replace('__JSX__', jsx)
for i in range(args.modules):
    (root / 'src' / f'component-{i:04}.tsx').write_text(template.replace('__INDEX__', f'{i:04}'))
for name in ['db.ts', 'catalog.ts']:
    (root / 'src' / name).write_bytes((source / name).read_bytes())
(root / 'src' / 'sentinel.ts').write_text("import { query } from '@bench/db'\nexport function sentinel() { return query('SELECT id FROM dataset') }\n")
(root / 'tsconfig.json').write_bytes((source / 'tsconfig.json').read_bytes())
for cohort, annotation_effort, identifier in [('mixed', 'low', True), ('max', 'max', True), ('annotation-only', 'max', False)]:
    config = f'''version: 2
rules:
  - rule: postgres-require-query-annotation
    scope: repository
    options:
      importSpecifier: '@bench/db'
      executorNames: [query]
      include: ['src/**/*']
      effort: {annotation_effort}
'''
    if identifier:
        config += '''  - rule: postgres-no-unbounded-sql-identifier
    scope: repository
    options:
      importSpecifier: '@bench/db'
      executorNames: [query]
      include: ['src/**/*']
      effort: max
'''
    (root / f'{cohort}.yml').write_text(config)
files = sorted((root / 'src').glob('*'))
contents = [p.read_bytes() for p in files]
metadata = dict(modules=args.modules, rows_per_module=args.rows, jsx_rows_per_module=args.jsx,
                source_files=len(files), source_bytes=sum(map(len, contents)),
                source_lines=sum(data.count(b'\n') for data in contents),
                sql_calls=args.modules + 1, fixture_sha256=hashlib.sha256(b''.join(contents)).hexdigest())
(root / 'fixture.json').write_text(json.dumps(metadata, indent=2) + '\n')
print(json.dumps(metadata))
