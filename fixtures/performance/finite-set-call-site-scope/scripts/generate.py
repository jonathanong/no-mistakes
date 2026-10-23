#!/usr/bin/env python3
"""Regenerate the literal TS/TSX workload saved beside the benchmark configs."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

for i in range(32):
    lines = [
        f"import {{ sharedValue{i % 4} as sharedValue, helper{i % 4} }} from '../shared/shared-{i % 4}';",
        "import type { ViewModel } from '../shared/types';",
        "",
        f"const label{i} = `component-{i}`;",
        f"function transform{i}(value: ViewModel) {{",
    ]
    for j in range(48):
        lines.append(f"  const item{i}_{j} = helper{i % 4}.mapValue(value, {j}, label{i});")
        lines.append(f"  helper{i % 4}.record(item{i}_{j}, sharedValue, {{ index: {j}, name: label{i} }});")
    lines += [
        "  return value;",
        "}",
        "",
        f"export function Component{i}(props: ViewModel) {{",
        f"  const value = transform{i}(props);",
        f"  return <section data-component={{label{i}}}>{{value.name}}</section>;",
        "}",
        "",
    ]
    path = ROOT / 'src/components' / f'component-{i:02}.tsx'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text('\n'.join(lines))

for k in range(4):
    path = ROOT / 'src/shared' / f'shared-{k}.ts'
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        f"export const sharedValue{k} = {k};\n"
        f"export const helper{k} = {{\n"
        "  mapValue<T>(value: T, index: number, label: string) { return value; },\n"
        "  record<T>(value: T, shared: number, detail: { index: number; name: string }) { return [value, shared, detail]; },\n"
        "};\n"
    )

(ROOT / 'src/shared/types.ts').write_text('export interface ViewModel { name: string; id: number; }\n')

for label in ('a', 'b'):
    lines = [
        "const scheduler = { register(name: string, payload: unknown) { return [name, payload]; } };",
        "",
    ]
    for j, name in enumerate(('daily-refresh', 'weekly-index', 'monthly-cleanup', 'hourly-sync')):
        lines.append(f"scheduler.register('{name}', {{ lane: {j}, enabled: true }});")
    lines += ["", f"export function runScheduledJobs{label.upper()}() {{"]
    for j in range(16):
        lines.append(f"  return scheduler.register('daily-refresh', {{ attempt: {j}, tags: ['batch', 'safe'] }});")
    lines += ["}", ""]
    (ROOT / 'src/schedules' / f'worker-{label}.ts').write_text('\n'.join(lines))

