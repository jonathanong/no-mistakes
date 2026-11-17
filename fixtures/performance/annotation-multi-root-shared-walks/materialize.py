#!/usr/bin/env python3
"""Recreate the bounded multi-root PostgreSQL annotation workload."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
ENTRIES = 256
HELPERS = 4
RECORDS = 32
DEPTH = 12
EXCLUDED = 8


def save(name, content):
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def helper(index):
    lines = ['import { query } from "@bench/db";', 'import sql from "sql-template-strings";']
    if index:
        lines.append(f'import {{ records as previous }} from "./helper-{index - 1}.mjs";')
    else:
        lines.append('const previous = { kind: "origin" };')
    lines.extend([
        f'const statement = sql`/* shared_{index} */ SELECT {index}`;',
        'const level0 = { left: statement, right: statement };',
    ])
    for depth in range(1, DEPTH + 1):
        lines.append(f'const level{depth} = {{ left: level{depth - 1}, right: level{depth - 1} }};')
    for slot in range(RECORDS):
        lines.append(f'const record{slot:02d} = {{ value: level{DEPTH}, previous }};')
    lines.extend([
        'export function provide(flag: boolean) {',
        f'  const selected = flag ? (0, statement) : (0, "/* branch_{index} */ SELECT {index + 1}");',
        '  return query(selected);',
        '}',
        'export const records = {',
        *[f'  record{slot:02d},' for slot in range(RECORDS)],
        '  statement,',
        '};',
    ])
    return '\n'.join(lines) + '\n'


def main():
    for pattern in ['src/entry-*.mts', 'src/shared/helper-*.mts', 'excluded/unrelated-*.mts']:
        for path in ROOT.glob(pattern):
            path.unlink()
    for index in range(HELPERS):
        save(f'src/shared/helper-{index}.mts', helper(index))
    for index in range(ENTRIES):
        unannotated = '  await query(sql`SELECT id FROM items`);\n' if index == 0 else ''
        save(f'src/entry-{index:02d}.mts', f'''import {{ query }} from "@bench/db";
import sql from "sql-template-strings";
import {{ provide }} from "./shared/helper-{HELPERS - 1}.mjs";
export async function read{index:02d}(flag: boolean) {{
{unannotated}  return provide(flag);
}}
''')
    for index in range(EXCLUDED):
        save(f'excluded/unrelated-{index:02d}.mts', f'''import {{ query }} from "@bench/db";
export async function unrelated{index:02d}() {{
  return query("SELECT {index}");
}}
''')
    for effort in ['low', 'max']:
        save(f'{effort}.yml', f'''rules:
  - rule: postgres-require-query-annotation
    scope: repository
    options:
      importSpecifier: "@bench/db"
      include: ["src/**/*.mts", "excluded/**/*.mts"]
      exclude: ["excluded/**/*.mts"]
      unanalyzableSql: ignore
      effort: {effort}
''')
    save('expected.json', json.dumps({
        'roots': ENTRIES + HELPERS, 'excludedRoots': EXCLUDED,
        'finding': {'file': 'src/entry-00.mts', 'line': 5},
    }, indent=2) + '\n')
    save('.no-mistakes.yml', (ROOT / 'max.yml').read_text(encoding='utf-8'))
    save('tsconfig.json', json.dumps({
        'compilerOptions': {'module': 'NodeNext', 'moduleResolution': 'NodeNext'},
        'include': ['src/**/*.mts', 'excluded/**/*.mts'],
    }, indent=2) + '\n')


if __name__ == '__main__':
    main()
