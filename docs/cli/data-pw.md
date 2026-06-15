# `no-mistakes data-pw`

Find every selector-attribute usage of a value (e.g. `data-pw="search-bar"`)
across component source files and test files.

```sh
no-mistakes data-pw search-bar --root . --format json
```

Use this when you need a blast radius for a test id before renaming or removing
it: which components declare it and which tests select on it. Results are split
into a `source` section (files that are not test files) and a `test` section
(files matching the configured `testInclude` globs).

The scanned attribute names come from `tests.playwright.selectors.testIds`
(e.g. `["data-testid", "data-pw"]`); override them with repeatable
`--attribute`. Source roots come from `tests.playwright.selectorRoots`; override
with repeatable `--scan`. `tests.playwright.selectorExclude` globs are skipped.

Matching is regex-based on the literal `attribute="value"` form, so it catches
JSX attributes (`<div data-pw="x">`) and CSS attribute selectors
(`page.locator('[data-pw="x"]')`). Dynamic values (`data-pw={x}`) are skipped.
Implicit references such as `getByTestId('x')`, which do not spell out
`attribute="value"`, are intentionally **not** matched.

Key options: `--attribute` (repeatable), `--scan` (repeatable),
`--include` (comma-separated subset of `source,test`), `--config`, `--format`,
and `--json`.

Output shape:

```json
{
  "value": "search-bar",
  "attributes": ["data-pw", "data-testid"],
  "source": [{ "file": "app/search.tsx", "line": 3, "attribute": "data-pw" }],
  "test": [{ "file": "e2e/search.spec.ts", "line": 5, "attribute": "data-pw" }]
}
```

Node API: `dataPw(options)`.
