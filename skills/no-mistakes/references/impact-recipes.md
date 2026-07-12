# Impact recipes

Use these recipes when a PR changes UI surfaces, shared TS helpers, workflow
files, or static-analysis gates and the next action is not obvious from a single
command. Prefer `--format json` for agent parsing and `--format paths` when
turning results into local commands. Playwright subcommands are the exception:
they use `--json` for structured output.

Recipe index:

- React component selector impact
- Selector-root expansion preview
- TS helper or named export impact
- Workflow and static-analysis changed files
- Diff test impact planning
- API response-shape fanout audit
- Package entrypoint and direct-subpath report
- Shared-helper dependent test discovery

## React component selector impact

For a changed component file or export:

```bash
no-mistakes react usages web/components/Button.tsx#Button --format json
no-mistakes playwright related web/components/Button.tsx --json
no-mistakes tests plan playwright --changed-file web/components/Button.tsx --format paths
no-mistakes impacted-checks web/components/Button.tsx --format paths
```

Then extract exact selector literals only where needed:

```bash
rg -n 'data-pw=|data-testid=|dataPw|testId' web/components/Button.tsx
no-mistakes data-pw save-button --format json
```

Read the results as:

- `react usages` answers which call sites, stories, and tests import or render the
  component before changing props or public component shape.
- `playwright related` answers which browser tests already cover the route or
  selector-bearing component.
- `tests plan playwright` prints selected test paths. Use `impacted-checks` when
  you need runnable local validation commands after editing.
- `data-pw` answers where a literal test id is declared in source and selected in
  tests. It does not match implicit `getByTestId("value")` calls, so use
  `playwright related` / `playwright tests` for coverage context.

Before finishing selector or route work, run the project coverage gate:

```bash
no-mistakes playwright check --json
no-mistakes check --format json
```

`no-mistakes check` only runs rules configured in `.no-mistakes.yml`; for
Storybook coverage, look for a configured `require-storybook-stories` rule or
use `react usages` to confirm story imports directly.

## Selector-root expansion preview

Use this before adding a new directory to `tests.playwright.selectorRoots` or
before deciding whether selectors under a directory are intentionally uncovered.

```bash
rg -n 'selectorRoots|selectorInclude|selectorExclude|testIdAttribute|testIds|componentTestIds' .no-mistakes.yml
rg -n 'data-pw=|data-testid=|dataPw|testId' web/path/to/candidate-root another/path/to/root
```

Replace `.no-mistakes.yml` with the effective config path for the repository
(`.no-mistakes.yml`, `.no-mistakes.yaml`, `.no-mistakes.json`,
`.no-mistakes.jsonc`, or the explicit `--config` path). Replace the candidate
root paths with directories from the current repository, and derive the selector
grep terms from configured `selectors.testIds` and `selectors.componentTestIds`
keys/values. The default example above only covers default-ish `data-pw`,
`data-testid`, `dataPw`, and `testId` names.

To preview uncovered selectors for new roots, first add the candidate
directories to `tests.playwright.selectorRoots` and update `selectorInclude`
when it would otherwise filter them out. Make that change in the real config or
in a temporary config copy. Then run:

```bash
no-mistakes playwright check --json
no-mistakes playwright check --config /tmp/no-mistakes-preview.yml --json
```

Use the first command when editing the real config and the second command when
previewing with a temporary config copy.

Report the preview as:

- effective config path, selector roots, includes, excludes, and tracked
  selector attributes or component test-id props;
- candidate directories that contain literal selector-bearing JSX or selector
  props;
- selector-bearing directories not yet covered by `selectorRoots`;
- uncovered selector values from `playwright check` after the proposed expansion.

Do not infer project-specific roots from naming conventions. If a directory
should be scanned, add it explicitly to config and let `playwright check` show
the new route or selector obligations.

## TS helper or named export impact

For a shared helper, SQL helper, or exported symbol:

```bash
no-mistakes symbols backend/db/moderation.mts --include both --format json
no-mistakes symbols backend/db/moderation.mts --mode signature-impact --symbol updatePolicy --format json
no-mistakes dependents backend/db/moderation.mts#updatePolicy --format json
no-mistakes tests plan vitest --changed-file backend/db/moderation.mts --format paths
no-mistakes impacted-checks backend/db/moderation.mts --format paths
```

Use the outputs as:

- `symbols --include both` shows the file's public API and imports.
- `signature-impact` groups the symbol definition, export paths, production
  callers, test callers, and focused tests before changing a function signature.
- `dependents FILE#SYMBOL` finds importers of the named export; namespace imports
  are file-level matches, so follow up with `rg` inside returned files when exact
  call lines matter.
- `tests plan vitest` prints selected test paths. Use `impacted-checks` when you
  need runnable local validation commands from the configured test plan and
  changed-file checks.

For SQL/schema-adjacent helpers, no-mistakes follows TS/JS imports and configured
checks. It does not query databases or infer dynamic schema usage; add explicit
tests or configured checks when schema validation is required.

## Workflow and static-analysis changed files

Start from the command that returns runnable local validation:

```bash
no-mistakes impacted-checks --base origin/main --format paths
no-mistakes impacted-checks .github/workflows/ci.yml crates/no-mistakes/src/codebase/ci_graph/mod.rs --format paths
```

For workflow trigger impact:

```bash
no-mistakes ci impact .github/workflows/ci.yml --format json
no-mistakes ci impact crates/no-mistakes/src/codebase/ci_graph/mod.rs --format json
```

For Rust binaries invoked by supported Cargo commands in GitHub Actions:

```bash
no-mistakes dependents src/bin/pg_schema.rs --relationship ci --format json
```

Use the results as:

- `impacted-checks` combines configured generic checks with framework-specific
  test-plan commands and should be the first answer to "what should I run?".
- `ci impact` maps changed files to workflows/jobs whose path filters match and
  reports resolved permissions; it is branch-agnostic and intentionally does not
  recursively evaluate called reusable workflows.
- `--relationship ci` is narrow: it maps GitHub Actions workflow files to Rust
  binary sources invoked by supported Cargo command shapes. It is not a general
  shell, npm script, or workflow dependency graph.

## Diff test impact planning

For "which tests should run for everything changed in this diff?", pass the
refspec once instead of hand-building repeated `--changed-file` arguments:

```bash
no-mistakes tests plan vitest --from-git-diff origin/main...HEAD --format commands
no-mistakes tests plan playwright --from-git-diff origin/main...HEAD --environment pre-push --format commands
no-mistakes impacted-checks --base origin/main --format paths
```

Read the results as:

- `--from-git-diff <base...head>` is sugar for `--base <base> --head <head>`; it
  runs the same `git diff --relative --name-status <base>...<head>` lookup
  `tests plan` already performs for `--base`/`--head`. Use three-dot refspecs
  only (`origin/main...HEAD`, or a bare `origin/main` to diff against `HEAD`);
  two-dot (`origin/main..HEAD`) is rejected because it is a different
  comparison base in git itself.
- Test files that changed directly are included in the plan without needing a
  dependency-graph traversal.
- `--format commands` prints the exact runner invocation for every selected
  test (`vitest <file>`, `playwright test <file>`, `dotnet test <project>
  --no-restore`, `swift test --filter <target>`), ready to run as-is.
- `--environment <name>` selects a configured `testPlan` environment (e.g. a
  narrower `pre-push` budget vs. a broader `pull-request` budget); omit it to
  use the default groups.
- `impacted-checks --from-git-diff` is not currently supported — pass
  `--base`/`--head` to `impacted-checks` instead when you need the combined
  test-plan-plus-generic-checks report for a diff.

## API response-shape fanout audit

For a changed backend serializer, view, or shared response type, enumerate
every surface that moves with it — generated fixtures, web response types, and
native (Swift/`.NET`) models — before editing the shape:

```bash
no-mistakes exports-of backend/api/orders/serializer.mts --format json
no-mistakes dependents backend/api/orders/serializer.mts#OrderResponse --format json
no-mistakes importers shared/types/order.mts --tests --format json
no-mistakes tests plan vitest --changed-file backend/api/orders/serializer.mts --format paths
no-mistakes impacted-checks backend/api/orders/serializer.mts shared/types/order.mts --format paths
```

Read the results as:

- `exports-of` lists the serializer's public shape and, per export, which files
  already import it — the starting map of consumers.
- `dependents FILE#SYMBOL` follows a specific exported type/symbol (e.g.
  `OrderResponse`) to its importers, including generated fixture files and
  cross-package consumers that a plain import scan would miss.
- `importers --tests` adds the transitive impacted-test set for a shared type
  file, but only over the TS/JS dependency graph — it surfaces backend and web
  tests, not native Swift/.NET tests.
- `tests plan` / `impacted-checks` turn the fanout into runnable commands once
  you know which files moved.

Caveats: no-mistakes follows static TS/JS imports for `dependents`,
`exports-of`, and `importers`. It does not parse Swift `Codable`/.NET model
files directly, generate API fixtures, or infer serialization at runtime —
trace the shared TS type/schema file each generated target is derived from.
For the native test targets themselves, use the framework-specific planners
(`tests plan swift` / `tests plan dotnet`, which read the configured
`tests.swift.packages` / `tests.dotnet.projects` graphs — see
[`tests plan`](https://github.com/jonathanong/no-mistakes/blob/main/docs/cli/tests-plan.md)),
and treat native/generated-fixture drift as something to verify by running the
project's contract or codegen check, not something no-mistakes proves by
itself.

## Package entrypoint and direct-subpath report

For "what does this workspace package publicly export, and who imports its
direct subpaths?" before changing package exports or adding a new subpath:

```bash
no-mistakes exports-of packages/shared-utils/src/index.mts --format json
no-mistakes importers packages/shared-utils/src/logger.mts --format json
no-mistakes exports-of packages/shared-utils/src/logger.mts --no-importers --format json
```

Read the results as:

- `exports-of <package-root-entry>` lists the package-root's named exports and,
  per export, its current importers — the package's public surface.
- `importers <direct-subpath-entry>` lists files that import that subpath
  directly (e.g. `packages/shared-utils/logger`), separate from the
  package-root surface, with a `dependentsCount` summary.
- `exports-of <direct-subpath-entry> --no-importers` is the fast path when you
  only need the subpath's own export list, not its consumers.

Caveat: side-effect-only imports (`import "shared-utils/register"`) and dynamic
`import()` calls are not counted by `importers` or `exports-of`. For those, or
for non-import edges (CommonJS `require`, cross-package resolution via a
`workspace:*` dependency name), use
[`dependents`](https://github.com/jonathanong/no-mistakes/blob/main/docs/cli/dependents.md)
instead — it follows the full typed edge set, not just static ES imports.

## Shared-helper dependent test discovery

Before changing a shared helper's runtime behavior, find every test that
covers it and decide whether a targeted plan or a full-suite fallback is the
right answer:

```bash
no-mistakes dependents shared/lib/format-date.mts --test vitest --format paths
no-mistakes tests plan vitest --changed-file shared/lib/format-date.mts --format paths
no-mistakes importers shared/lib/format-date.mts --tests --format json
```

Read the results as:

- `dependents FILE --test vitest` returns the test files reachable from the
  helper through the dependency graph, independent of any `testPlan` config.
- `tests plan vitest --changed-file <helper>` applies the repo's configured
  environment/limit/group rules, matching what would actually run in CI.
- `importers --tests` is the fast, import-scan-only cross-check (no full graph
  build) — use it to sanity-check the two commands above on a mid-size repo
  before running the slower graph-based ones.

When a broad fallback is expected, not a bug: if the helper is imported by a
barrel file, a global config file (`package.json`, `tsconfig.json`, framework
config), or any path matched by the repo's configured
`testPlan.*.globalConfigFallback`/full-suite triggers, `tests plan` sets
`fallback_triggered` with a `fallback_reason` and selects the framework's full
discovered test set instead of a narrow dependency path. Report that as the
correct, conservative answer — do not treat `fallback_triggered: true` as a
tracing failure to work around.
