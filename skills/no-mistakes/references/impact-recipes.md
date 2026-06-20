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
rg -n 'selectorRoots|selectorExclude|testIdAttribute|testIds' .no-mistakes.yml
rg -n 'data-pw=|data-testid=|dataPw|testId' web/path/to/candidate-root another/path/to/root
```

Replace the example root paths with the candidate directories from the current
repository. To preview uncovered selectors for new roots, first add the
candidate directories to `tests.playwright.selectorRoots` in `.no-mistakes.yml`
or in a temporary config copy, then run:

```bash
no-mistakes playwright check --json
```

Report the preview as:

- existing selector roots and excludes from `.no-mistakes.yml`;
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
