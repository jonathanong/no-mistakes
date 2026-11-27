"""Materialize a TypeScript workload for comparing SQL variant recovery."""
import argparse
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('root', type=Path)
parser.add_argument('--consumers', type=int, default=2300)
parser.add_argument('--other', type=int, default=4000)
parser.add_argument('--routes', type=int, default=8)
args = parser.parse_args()
root = args.root
if args.consumers < 1 or args.routes < 1 or args.other < 0:
    parser.error("consumers and routes must be positive and other must be nonnegative")
if root.exists() and any(root.iterdir()):
    parser.error("workload output directory must be empty")

def save(rel, source):
    path = root / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source.rstrip() + '\n')

save('tsconfig.json', '{"compilerOptions":{"module":"NodeNext","moduleResolution":"NodeNext"},"include":["src/**/*.mts"]}')
for effort in ('low', 'max'):
    save(f'{effort}.yml', f'''rules:
  - rule: postgres-require-query-annotation
    scope: repository
    severity: error
    options:
      effort: {effort}
      unanalyzableSql: advise
      importSpecifier: "@bench/db"
      executorTypeNames: [Run]
      include: ["src/**/*.mts"]''')
save('src/helpers.mts', '''export function invoke<T>(callback: (text: string) => T, text: string) { return callback(text); }
export function around<T>(callback: (text: string) => T, text: string) { return invoke(callback, text); }
export function call<T>(callback: (text: string) => T, text: string) { return around(callback, text); }''')
for consumer in range(args.consumers):
    lines = ['import { query } from "@bench/db";', 'import { call } from "./helpers.mjs";', 'type Run = typeof query;']
    lines.append('query("/* seeded */ SELECT id FROM widgets");')
    if consumer == 0:
        lines.append('query("SELECT id FROM widgets");')
    for route in range(args.routes):
        name = f'lookup_{route}'
        prefix = f'/* {name} */ ' if route else ''
        lines += [
            f'export function {name}(options: {{ query?: Run }}, flag: boolean, item: string) {{',
            '  const run = options.query ?? query;',
            f'  const statement = flag ? "{prefix}SELECT id FROM widgets" : "{prefix}SELECT id FROM gadgets";',
            '  const queryWithArgs = (text: string) => run(text, [item]);',
            '  const execute = (text: string) => call(queryWithArgs, text);',
            '  return call(execute, statement);',
            '}',
        ]
    save(f'src/root-{consumer:04}.mts', '\n'.join(lines))
for other in range(args.other):
    lines = []
    for route in range(args.routes):
        lines += [
            f'export function title_{route}(flag: boolean, item: string) {{',
            f'  const display = flag ? "first-{route}" : "second-{route}";',
            '  const data = { display, item };',
            '  return data;',
            '}',
        ]
    save(f'src/other/value-{other:04}.mts', '\n'.join(lines))
