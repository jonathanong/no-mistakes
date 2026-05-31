# `no-mistakes tests comment`

Render a test plan JSON file as a Markdown PR comment.

```sh
no-mistakes tests comment plan.json
no-mistakes tests comment plan.json --out test-plan.md
```

Use this in CI or review automation after `tests plan --format json`.

Key options: positional plan JSON path and optional `--out`.

Node API: `testsComment(options)`.
