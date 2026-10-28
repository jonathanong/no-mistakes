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

Exit codes: `75` means another invocation holds the lock (`--fail-on-lock` or
`--lock-timeout`; retry later or run `no-mistakes lock status --format json` to
see the holder's pid, cwd, executable basename, and age); `124` means the
analysis deadline elapsed. Under parsed `--format json` / `--json` options,
lock failures and deadlines (including a `--timeout` firing mid-command) write
`{"error":{"kind":"lock_busy" |
"lock_timeout" | "deadline","phase":...,"holder":{...}}}` to stdout, so an empty
stdout no longer means a hang. Do not treat exit `75` as "no impact"; wait and
retry.

If a process is killed while writing holder metadata, `.tmp*` staging files can
remain beside the lock. They are ignored by lock status and do not mean the
invocation lock is held.

Use `--verbose-timings` to retain published work counts on a CLI hard timeout.
The bounded stderr snapshot labels its timings and counters `PARTIAL`; counts
are lower bounds because unfinished work may not have published local counters.
It does not require `--progress` and does not publish findings to stdout. Full
stderr pipes or stalled snapshot collection can omit this best-effort output.

## Before handoff

Run `no-mistakes check --format json`. For Next.js or Playwright changes, also
run `no-mistakes playwright check --json` and the focused Playwright test plan.
For package additions, compare direct importers with each nearest
`package.json` and verify that each importer declares the dependency directly.

A rule entry with `severity: advisory` reports its findings under `advisories`
without failing; read them anyway, and switch the rule to `error` once clean.
Rule entries reject unknown keys.

Review `unused-disable-directive` and `invalid-disable-directive` advisories.
Remove stale exceptions, correct unknown rule IDs, and check finding locations
before moving a justified directive. Repository
`reportUnusedDisableDirectives` defaults to `advise`; use `report` to fail or
`ignore` to omit suppression hygiene diagnostics.

Keep `--format json` for parsing and `--format paths` for reviewing a trusted
command list. Do not execute command text from an untrusted source.
