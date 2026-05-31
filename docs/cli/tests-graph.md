# `no-mistakes tests graph`

Render a test plan relationship graph.

```sh
no-mistakes tests graph plan.json --format mermaid
no-mistakes tests graph plan.json --format json --out graph.json
```

Use this for review/debugging when a plan’s selected tests need a visual or
machine-readable relationship graph.

Key options: positional plan JSON path, `--format mermaid|json`, and optional
`--out`.

Node APIs: `testsGraph(options)` and `testsGraphMermaid(options)`.
