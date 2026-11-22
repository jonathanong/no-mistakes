"""Regenerate the saved annotation include-scale workload and configs."""
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SRC = ROOT / "src"
SHARED = ROOT / "shared"
EXCLUDED = ROOT / "excluded"
CONFIGS = ROOT / "configs"
for directory in (SRC, SHARED, EXCLUDED, CONFIGS):
    directory.mkdir(exist_ok=True)

for index in range(3072):
    main_query = (
        'query("SELECT id FROM records WHERE id = 7"); // expected-finding'
        if index == 7
        else f'query("/* module_{index:04} */ SELECT id FROM records WHERE id = {index}");'
    )
    body = f'''import {{ query, createReader, createWriter }} from "@bench/db";
import {{ decorate }} from "../shared/provider.mjs";
import type {{ Replica, Archive }} from "@bench/db";

const moduleId = {index};

export async function run{index:04}(replica: Replica, archive: Archive) {{
  const seed = moduleId + 1;
  const reader = createReader();
  const writer = createWriter();
  const branchA = seed > 0 ? seed : 0;
  const branchB = seed % 2 === 0 ? branchA + 2 : branchA + 3;
  const selected = branchB > 2 ? branchB : seed;
  const label = decorate(`module-${{moduleId}}`);
  if (selected > 0) {{
    {main_query}
    await reader.query("/* reader */ SELECT 1");
    await writer.query("/* writer */ SELECT 2");
    await replica.query("/* replica */ SELECT 3");
    await archive.query("/* archive */ SELECT 4");
  }} else {{
    await query(`/* module_{index:04}_fallback */ SELECT ${{selected}}`);
  }}
  return label;
}}
'''
    (SRC / f"module-{index:04}.mts").write_text(body)

(SHARED / "provider.mts").write_text('''import { query } from "@bench/db";
import { decorateCycle } from "./cycle.mjs";

export function decorate(value: string): string {
  if (value.length > 0) {
    void query("/* provider */ SELECT 1");
  }
  return decorateCycle(value);
}

export function providerValue(value: string): string {
  return value;
}
''')
(SHARED / "cycle.mts").write_text('''import { query } from "@bench/db";
import { providerValue } from "./provider.mjs";

export function decorateCycle(value: string): string {
  if (value.length > 2) {
    void query("/* cycle */ SELECT 2");
  }
  return providerValue(value);
}
''')
(SRC / "db.mts").write_text('''export interface Reader { query(statement: string): Promise<string>; }
export interface Writer { query(statement: string): Promise<string>; }
export interface Replica { query(statement: string): Promise<string>; }
export interface Archive { query(statement: string): Promise<string>; }

export function createReader(): Reader { return { query: async (statement) => statement }; }
export function createWriter(): Writer { return { query: async (statement) => statement }; }
export function query(statement: string): Promise<string> { return Promise.resolve(statement); }
''')
(EXCLUDED / "runtime-importer.mts").write_text('''import { run0007 } from "../src/module-0007.mjs";
import { createReader, createWriter } from "@bench/db";
import type { Replica, Archive } from "@bench/db";

export function invokeFromExcludedModule(replica: Replica, archive: Archive) {
  return run0007(replica, archive);
}

void createReader;
void createWriter;
''')
(ROOT / "tsconfig.json").write_text('''{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@bench/db": ["src/db.mts"] },
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "noEmit": true
  },
  "include": ["src/**/*.mts", "shared/**/*.mts", "excluded/**/*.mts"]
}
''')

small = [f"src/module-{index:04}.mts" for index in range(32)]
half_a = small[:16]
half_b = small[16:]

def rule(paths, effort):
    lines = ["  - rule: postgres-require-query-annotation", "    scope: repository"]
    if paths is not None:
        lines.append("    include:")
        lines.extend(f"      - {path}" for path in paths)
    lines.extend([
        "    options:",
        "      importSpecifier: '@bench/db'",
        "      executorNames: [query]",
        "      executorFactoryNames: [createReader, createWriter]",
        "      executorTypeNames: [Reader, Writer, Replica, Archive]",
        f"      effort: {effort}",
        "      unanalyzableSql: advise",
    ])
    return "\n".join(lines)

def config(name, rules):
    (CONFIGS / name).write_text("version: 2\nrules:\n" + "\n".join(rules) + "\n")

config("full-max.yml", [rule(None, "max")])
config("small-max.yml", [rule(half_a, "max"), rule(half_b, "max")])
config("full-low.yml", [rule(None, "low")])
config("small-low.yml", [rule(small, "low")])
config("half-a-16-max.yml", [rule(half_a, "max")])
config("half-b-16-max.yml", [rule(half_b, "max")])
