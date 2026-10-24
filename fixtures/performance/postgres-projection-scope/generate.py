#!/usr/bin/env python3
"""Regenerate the deterministic PostgreSQL projection-scope source fixture."""
from pathlib import Path

ROOT = Path(__file__).resolve().parent
COMPONENTS = ROOT / "src" / "components"
COMPONENTS.mkdir(parents=True, exist_ok=True)
for index in range(32):
    calls = []
    for query_index in range(16):
        sql = (
            f"SELECT id FROM projection_{index:02d}_{query_index:02d} "
            f"WHERE tenant_id = {query_index + 1}"
        )
        if index == 0 and query_index == 15:
            sql += " OFFSET 7"
        calls.append(f'  query("{sql}");')
    rows = ",\n".join(
        f'  {{ id: {row}, label: "row-{index:02d}-{row:03d}", group: {row % 9} }}'
        for row in range(96)
    )
    source = (
        'import { query } from "../db.mjs";\n\n'
        f"function prepareQueries{index:02d}() {{\n"
        + "\n".join(calls)
        + "\n}\n\n"
        + f"const rows{index:02d} = [\n{rows}\n];\n\n"
        + f"export function Component{index:02d}() {{\n"
        + f"  prepareQueries{index:02d}();\n"
        + f'  return <section data-component="{index:02d}">\n'
        + f"    <h2>Projection component {index:02d}</h2>\n"
        + f"    <ul>{{rows{index:02d}.map((row) => (\n"
        + "      <li key={row.id} data-group={row.group}>{row.label}</li>\n"
        + "    ))}</ul>\n  </section>;\n}\n"
    )
    (COMPONENTS / f"component-{index:02d}.tsx").write_text(source)

(ROOT / "src" / "db.mts").write_text(
    "export function query(_sql: string): void {}\n"
)
