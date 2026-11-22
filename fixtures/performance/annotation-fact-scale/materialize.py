#!/usr/bin/env python3
"""Regenerate the saved shared callback and scalar-payload workload."""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
SEED = json.loads((ROOT / "seed.json").read_text(encoding="utf-8"))
DEFAULT_ENTRIES = SEED["roots"]
DEFAULT_HELPERS = SEED["callbackDepth"]
PAYLOAD_DEPTH = SEED["payloadDepth"]
EXCLUDED = SEED["excludedUnrelatedModules"]


def save(name, content):
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def helper(index):
    if index:
        imports = f'import {{ invoke{index - 1:02d} }} from "./helper-{index - 1:02d}.mjs";\n'
        body = f"return invoke{index - 1:02d}(callback, payload);"
    else:
        imports = ""
        body = "return callback();"
    return f'''{imports}type QueryCallback = () => unknown;
export function invoke{index:02d}(callback: QueryCallback, payload: unknown) {{
  {body}
}}
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--roots", type=int, default=DEFAULT_ENTRIES)
    parser.add_argument("--depth", type=int, default=DEFAULT_HELPERS)
    args = parser.parse_args()
    if args.roots < 1 or args.depth < 1:
        parser.error("--roots and --depth must be positive")

    for pattern in ["src/entry-*.mts", "src/shared/helper-*.mts", "src/shared/payload.mts", "excluded/*.mts"]:
        for path in ROOT.glob(pattern):
            path.unlink()
    payload_lines = ['const level00 = { value: 1 };']
    for index in range(1, PAYLOAD_DEPTH + 1):
        payload_lines.append(
            f"const level{index:02d} = {{ left: level{index - 1:02d}, right: level{index - 1:02d} }};"
        )
    payload_lines.append(f"export const payload = level{PAYLOAD_DEPTH:02d};")
    save("src/shared/payload.mts", "\n".join(payload_lines) + "\n")

    for index in range(args.depth):
        save(f"src/shared/helper-{index:02d}.mts", helper(index))
    for index in range(args.roots):
        safe_name = f"entry_{index:02d}"
        save(f"src/entry-{index:02d}.mts", f'''import {{ query }} from "@bench/db";
import {{ invoke{args.depth - 1:02d} }} from "./shared/helper-{args.depth - 1:02d}.mjs";
import {{ payload }} from "./shared/payload.mjs";

export function read{index:02d}() {{
  invoke{args.depth - 1:02d}(() => query("/* {safe_name} */ SELECT {index}"), payload);
  return query("SELECT id FROM items WHERE id = {index}"); // expected-finding
}}
''')
    for index in range(EXCLUDED):
        save(f"excluded/unrelated-{index:02d}.mts", f'''import {{ query }} from "@bench/db";
export function unrelated{index:02d}() {{ return query("SELECT {index}"); }}
''')
    for effort in ("low", "max"):
        save(f"{effort}.yml", f'''rules:
  - rule: postgres-require-query-annotation
    scope: repository
    options:
      effort: {effort}
      unanalyzableSql: ignore
      importSpecifier: "@bench/db"
      include: ["src/**/*.mts", "excluded/**/*.mts"]
      exclude: ["excluded/**/*.mts"]
''')
    save(".no-mistakes.yml", (ROOT / "max.yml").read_text(encoding="utf-8"))
    save("tsconfig.json", json.dumps({
        "compilerOptions": {"module": "NodeNext", "moduleResolution": "NodeNext"},
        "include": ["src/**/*.mts", "excluded/**/*.mts"],
    }, indent=2) + "\n")
    source_paths = sorted(ROOT.rglob("*.mts"))
    source_lines = sum(len(path.read_text(encoding="utf-8").splitlines()) for path in source_paths)
    source_bytes = sum(path.stat().st_size for path in source_paths)
    default_workload = args.roots == DEFAULT_ENTRIES and args.depth == DEFAULT_HELPERS
    preflight = (
        {"version": "v0.109.109", "low": "confirmed", "max": "confirmed"}
        if default_workload
        else {"verification": "not confirmed for generated parameters"}
    )
    save("expected.json", json.dumps({
        "entries": args.roots,
        "sharedCallbackModules": args.depth,
        "sharedPayloadDepth": PAYLOAD_DEPTH,
        "excludedUnrelatedModules": EXCLUDED,
        "expectedFindings": args.roots,
        "expectedClearSqlVerdicts": args.roots,
        "findingSourceMarker": "expected-finding",
        "expectedAbstentions": 0,
        "expectedAdvisories": 0,
        "expectedWarnings": 0,
        "releasePreflight": preflight,
        "savedSourceFiles": len(source_paths),
        "savedSourceLines": source_lines,
        "savedSourceBytes": source_bytes,
    }, indent=2) + "\n")
    verification = (
        f"The exact report contract was confirmed on release `v0.109.109` at both effort levels: {args.roots} expected findings, {args.roots} proven-clear SQL verdicts, and zero abstentions, advisories, or warnings."
        if default_workload
        else "Expected counts for custom parameters are estimates and have not been checked against a release binary."
    )
    save("README.md", f'''# Annotation fact scale workload

The saved workload has {args.roots} entry modules, {args.depth} shared callback modules, a shared payload module with a depth-{PAYLOAD_DEPTH} scalar record DAG, and {EXCLUDED} excluded unrelated modules. Each entry passes the same imported payload through the callback chain, where it is unused, and calls a shared callback containing an explicitly annotated query. It also has one direct unannotated query marked `expected-finding`.

{verification} `expected.json` records this preflight and the saved source inventory: {len(source_paths)} source files, {source_lines} physical lines, and {source_bytes} bytes.

The deterministic CI ceiling is the work-count assertions in `codebase::postgres::query_annotation::evaluate::effects::call_value_work_tests`: scalar-record refresh visits are zero when there are no builder tables, at most one with an unrelated builder invalidated, and at most one for the callback scalar-record case. These count assertions guard the repeated argument-refresh cost. Wall-time max/low ratios are diagnostic only; timings vary with host load and are not a CI gate.

Regenerate the saved sources and configs with `python3 materialize.py` from this directory. Optional `--roots N --depth N` parameters let profiling vary scale; regenerate the seed defaults after experiments before committing. Generated TypeScript is saved for review, and benchmark runs read the corpus in place.
''')


if __name__ == "__main__":
    main()
