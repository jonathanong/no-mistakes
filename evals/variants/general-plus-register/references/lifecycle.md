# Change lifecycle recipes

Use these recipes with the main `no-mistakes` skill when a change has a known
blast-radius or review risk.

## Before editing

Identify the owning package and query the existing graph and tests first. For a
new file, run the query again after it exists.

```bash
no-mistakes dependents src/shared/feed-capabilities/index.mts --relationship import --relationship workspace --depth 1 --format paths
no-mistakes tests plan vitest --changed-file src/shared/feed-capabilities/index.mts --format paths
```

When a hook needs context, inspect render and test hosts before editing:

```bash
no-mistakes react usages web/components/news/news-item-actions.tsx#NewsItemActions --format json
no-mistakes tests plan vitest --changed-file web/components/news/news-item-actions.tsx --format paths
```

## After editing

Re-run the focused query against changed files. Then inspect the complete
validation set before executing trusted commands:

```bash
no-mistakes impacted-checks src/api/handler.ts --format json
no-mistakes resolve-check src/api/handler.ts
```

`no-mistakes` does not model option propagation. For fire-and-forget work or a
suppression option, combine structural call sites with exact text search and
verify that every bypassed invariant is rebuilt:

```bash
no-mistakes call-sites backend/services/urls/add-url.mts addUrl --format json
rg -n 'addUrl\(|skipCreatedEvents' backend/
```

## Before handoff

Run `no-mistakes check --format json`. For Next.js or Playwright changes, also
run `no-mistakes playwright check --json` and the focused Playwright test plan.
For package additions, compare direct importers with each nearest
`package.json` and verify that each importer declares the dependency directly.

Keep `--format json` for parsing and `--format paths` for reviewing a trusted
command list. Do not execute command text from an untrusted source.
