# CLI Reference

All commands are local and deterministic. Use `--format json` for tooling,
`--format paths` for shell pipelines, and `--timings` where available when an
agent needs to explain cost. `--json` is a shorthand for JSON on commands that
support it.

The npm package also exposes async N-API wrappers for the structured command
surfaces: `dependencies`, `dependents`, `related`, `symbols`, `fetches`,
`check`, `testsPlan`, `testsWhy`, `testsComment`, `testsGraph`,
`testsGraphMermaid`, `playwrightCheck`, `playwrightEdges`,
`playwrightRelated`, `playwrightTests`, queue helpers, server-route helpers,
React helpers, and `version`.

## `no-mistakes`

Unified entry point for codebase graph and check commands.

```sh
no-mistakes dependencies <FILE>... [--root <PATH>] [--tsconfig <FILE>]
no-mistakes dependents <FILE[#SYMBOL]>... [--root <PATH>] [--tsconfig <FILE>]
no-mistakes related <FILE[#SYMBOL]>...
no-mistakes symbols <FILE>...
no-mistakes fetches [--root <ROOT>] [--config <CONFIG>] [--format <FORMAT>] [--json] [TARGETS]...
no-mistakes playwright check [OPTIONS]
no-mistakes playwright edges [OPTIONS]
no-mistakes playwright related [OPTIONS] <FILES>...
no-mistakes playwright tests [OPTIONS] [FILES]...
no-mistakes test plan <playwright|vitest> [OPTIONS]
no-mistakes react analyze [TARGETS]...
no-mistakes react check [TARGETS]... [--assert-no-fetch]
no-mistakes queues edges [FILES]... [--depth N]
no-mistakes queues related <FILES>... [--direction deps|dependents|both]
no-mistakes queues check
no-mistakes server routes [FILES]...
no-mistakes server edges [ROOTS]... [--depth N]
no-mistakes server related <ROOTS>... [--direction deps|dependents|both]
no-mistakes check
```

### Graph Commands

`dependencies`, `dependents`, and `related` share these options:

| Option | Description |
| --- | --- |
| `--root <PATH>` | Project root. Defaults to the current working directory. |
| `--tsconfig <FILE>` | tsconfig for path aliases. If omitted, searches upward from root. |
| `--depth <N>` | Maximum traversal depth. `--max-depth` is an alias on graph, queue, and server edge commands. Queue/server `edges` default to direct edges when roots are provided, and to the full edge list otherwise. |
| `--filter <GLOB>` | Include only matching files. Repeatable; trailing `/` collapses to folder level. |
| `--target-module <GLOB>` | Include only matching external module nodes. Repeatable. |
| `--test <FRAMEWORK>` | Filter to `vitest`, `playwright`, or `cargo` test globs. Repeatable. |
| `--relationship <KIND>` | Follow only `import`, `import-static`, `import-dynamic`, `import-type`, `import-require`, `workspace`, `package`, `test`, `route`, `queue`, `md`, `ci`, `http`, `process`, `asset`, `react`, or `all`. Repeatable. |
| `--format <FORMAT>` | `json`, `md`, `yml`, `paths`, or `human`. |
| `--json` | Shorthand for `--format json`. |
| `--timings` | Emit phase timings on stderr. |
| `-j, --jobs <N>` | Rayon worker threads. `0` means all cores. |

Examples:

```sh
no-mistakes dependencies src/main.mts --relationship import --format json
no-mistakes dependencies src/main.mts --target-module '@react/*' --format paths
no-mistakes dependents '@react/client' --format paths
no-mistakes dependents src/utils.mts --test vitest --format paths
no-mistakes dependents src/queues.mts#sendEmail --json
no-mistakes related web/app/users/page.tsx --relationship test --format paths
```

`FILE#SYMBOL` is supported only by `dependents`/`related`. It finds files that
import that named export, including through re-export chains. Namespace imports
match all symbols.

### Symbols

```sh
no-mistakes symbols src/api.mts --include both --format json
no-mistakes symbols src/types.mts --kind type --kind interface
```

Options: `--root`, `--tsconfig`, repeatable `--kind`, `--include
exports|imports|both`, `--format`, `--json`, `--timings`, and `--jobs`.

### React

```sh
no-mistakes react analyze 'app/components/**/*.tsx' --format json
no-mistakes react check 'app/components/**/*.tsx' --assert-no-fetch
```

Options: `--root`, `--config`, `--format`, and `--json`. `--jobs` is a global
wrapper option, for example `no-mistakes --jobs 4 react ...`.

### Queues

```sh
no-mistakes queues edges --format json
no-mistakes queues edges backend/jobs/email.ts --depth 1
no-mistakes queues related backend/jobs/email.ts --direction dependents --format paths
no-mistakes queues check
```

Options: `--root`, `--tsconfig`, repeatable `--filter`, `--depth` for `edges`,
`--max-depth` as a `--depth` alias, `--format`, `--json`, and `--timings`.
When `edges` receives roots and no depth, it returns direct edges only. `--jobs`
is a global wrapper option, for example `no-mistakes --jobs 4 queues ...`.

### Server Routes

```sh
no-mistakes server routes --format json
no-mistakes server edges backend/api/users.ts --depth 1
no-mistakes server related backend/api/users.ts --direction deps --format paths
```

Options: `--root`, `--tsconfig`, repeatable `--filter`, `--depth` for `edges`,
`--max-depth` as a `--depth` alias, `--format`, `--json`, and `--timings`.
When `edges` receives roots and no depth, it returns direct edges only. `--jobs`
is a global wrapper option, for example `no-mistakes --jobs 4 server ...`.

### Global Check

```sh
no-mistakes check --format json
no-mistakes check --json
```

Runs configured React, queue, integration, and codebase rules such as
`test-no-unmocked-dynamic-imports`, `nextjs-no-api-routes`, and
`nextjs-no-caching`. Independent check domains run in parallel, and results are
printed in deterministic order. Options: `--root`, `--config`, `--tsconfig`,
`--format`, `--json`, and `--timings`. `--jobs` is a global wrapper option, for
example `no-mistakes --jobs 4 check ...`.

`check` only runs configured checks. Use direct subcommands such as
`no-mistakes queues check` when you want a full scan for that domain without
adding it to `.no-mistakes.yml`.

Named Vitest/Playwright projects are normally read from their test runner
configs. When a runner config builds projects dynamically, define deterministic
project globs in `.no-mistakes.yml` and target that name from rules:

```yml
tests:
  vitest:
    projects:
      web:
        include:
          - web/**/*.test.{ts,tsx,mts}
        exclude:
          - web/generated/**
rules:
  - rule: test-no-unmocked-dynamic-imports
    tests:
      vitest: [web]
```

### Test Plans

`no-mistakes test plan <playwright|vitest>` selects focused test files from
changed files and configured priority groups. `tests plan` remains available as
the compatibility spelling.

```sh
no-mistakes test plan vitest --changed-file src/user.ts --format paths
no-mistakes test plan playwright --environment pre-push --json
```

| Option | Description |
| --- | --- |
| `--root <PATH>` | Project root. Defaults to the current working directory. |
| `--config <CONFIG>` | `.no-mistakes.*` config file. |
| `--tsconfig <FILE>` | tsconfig for path aliases. |
| `--base <REF>` / `--head <REF>` | Git diff range. |
| `--changed-file <FILE>` | Explicit changed file. Repeatable. |
| `--changed-files <FILE>` | Newline-separated changed file list. |
| `--environment <NAME>` | Test plan environment. Defaults to `pre-push`; `prePush` and `pre_push` config keys are equivalent. |
| `--limit-percent <N>` | Override the environment limit percentage. |
| `--limit-files <N>` | Override the environment file cap. |
| `--format <FORMAT>` | `json`, `paths`, `markdown`, or `md`. |
| `--json` | Shorthand for `--format json`. |

Config is read from `testPlan` (`test_plan` is also accepted):

```yml
testPlan:
  playwright:
    dependencies:
      projects:
        cloudflare-worker: true
        web:
          - next.config.*
          - proxy.ts
    environments:
      prePush:
        limit:
          percent: 5
          files: 100
        groups:
          - type: direct
          - type: coverage
          - type: dependencies
          - type: sample
            limit:
              percent: 1
              files: 100
  vitest:
    environments:
      prePush:
        groups:
          - type: direct
          - type: dependencies
```

Groups are mutually exclusive in declaration order. `coverage` is Playwright
only; Vitest supports `direct`, `dependencies`, and deterministic `sample`.
`dependencies.projects.<name>: true` runs the full selected framework when a
file under that configured project root/include set changes. Explicit project
dependency globs are relative to that project's root unless they already include
the root prefix.

### Filesystem Rules via `no-mistakes check`

The `rust-max-lines-per-file`, `rust-no-inline-tests`,
`rust-no-inline-allows`, and `agents-md-max-size` checks are built into
`no-mistakes check` and run when configured in `.no-mistakes.yml`. See the
Global Check section above and the configuration reference for available
options.

### Next.js Feature Ban Rules via `no-mistakes check`

Use zero-option project rules to disable Next.js features:

```yml
projects:
  web:
    type: nextjs
    root: web

rules:
  - rule: nextjs-no-api-routes
    projects: [web]
  - rule: nextjs-no-caching
    projects: [web]
```

`nextjs-no-api-routes` rejects App Router `app/**/route.*` and
`src/app/**/route.*` handlers and Pages Router `pages/api/**` and
`src/pages/api/**` files. `nextjs-no-caching` rejects Next.js cache
directives, `next/cache` APIs, cached `fetch` options, static cache segment
config, and cache-related `next.config.*` settings.

### Storybook Component Coverage via `no-mistakes check`

`require-storybook-stories` requires selected exported React components to have
coverage from reachable Storybook stories. Coverage counts direct story imports and
React child components rendered by covered components. Dynamic import and mock
targets are not required by `include_all_react_*` unless explicitly included.

```yml
projects:
  web:
    type: nextjs
    root: web

rules:
  - rule: require-storybook-stories
    projects: [web]
    options:
      stories: ["storybook/**/*.stories.{ts,tsx,js,jsx}"]
      include: ["components/special/**/*.tsx"]
      exclude: ["components/generated/**"]
      ignore_index_and_private_files: true
      include_all_react_named_exports: true
      include_all_react_default_exports: true
      required_props: ["data-pw"]
      allow_colocated_tests: true
      allow_components:
        "components/actions/delete-button.tsx#DeleteButton": "Requires live mutation callbacks."
      allow_files:
        "components/generated/**": "Generated wrappers."
```

`stories` can be omitted. In that case the rule reads `tests.storybook.configs`
and uses the configured Storybook `stories` entries; if no Storybook config is
available it falls back to `**/*.stories.{ts,tsx,js,jsx}`. Test files are
excluded from component selection by default using configured Vitest,
Playwright, and conventional test globs.

`allow_colocated_tests: true` treats a same-directory test file as coverage for
components in the matching source file. The accepted names are
`<stem>.test.tsx`, `<stem>.mock.test.tsx`, `<stem>.test.ts`, and
`<stem>.mock.test.ts`.

Glob matching is slash-aware: `components/ui/*.tsx` matches direct children like
`components/ui/button.tsx`, while recursive selection requires `**`. Set
`ignore_index_and_private_files: true` to skip selected files named `index.tsx`
or `_*.tsx`, which is useful for barrel files and private implementation
components.

Story coverage uses AST-based runtime imports. Type-only imports such as
`import type { Button } from "../components/Button"` do not count as coverage.

Use `// no-mistakes-disable-next-line require-storybook-stories: reason` for a
single export or `// no-mistakes-disable-file require-storybook-stories: reason`
for a whole file.

`required_props` narrows automatic include-all selection to files that mention
at least one configured prop. Explicit `include` globs still select matching
component files even when those props are absent.

### Playwright

Static Playwright coverage for Next.js App Router routes, selectors, and fetch
assertions.

```sh
no-mistakes playwright check [OPTIONS]
no-mistakes playwright edges [OPTIONS]
no-mistakes playwright related [OPTIONS] <FILES>...
no-mistakes playwright tests [OPTIONS] [FILES]...
```

| Option | Description |
| --- | --- |
| `--root <ROOT>` | Repository or package root. |
| `--config <CONFIG>` | Analyzer config file. |
| `--playwright-config <FILE>` | Playwright config file. Repeatable. |
| `--project <NAME>` | Top-level Playwright config `name` filter. |
| `--json` | Emit JSON. |
| `--assert-conditional-tests` | Require active test coverage; conditional tests do not count. |
| `--allow-skipped-tests` | Allow skipped tests/suites to count. |
| `--assert-unique-test-ids` | Fail on duplicate exact test IDs across selector roots. |
| `--assert-unique-html-ids` | Fail on duplicate exact HTML `id` values. |
| `--assert-unique-selectors` | Deprecated compatibility alias. |

Examples:

```sh
no-mistakes playwright check --json
no-mistakes playwright related 'web/app/users/[id]/page.tsx'
no-mistakes playwright edges --json
no-mistakes playwright tests tests/e2e/users.spec.ts --json
```

In Node.js, use the dedicated async wrapper for Playwright's analyzer-specific
related report:

```js
const { playwrightRelated } = require("no-mistakes");

(async () => {
  const report = await playwrightRelated({
    root: process.cwd(),
    files: ["web/app/users/[id]/page.tsx"],
  });
})();
```

`related({ tests: ["playwright"] })` remains the generic dependency-graph test
filter; it is not a substitute for `no-mistakes playwright related`.

Supported analyzer config files: `.no-mistakes.yaml`, `.no-mistakes.yml`,
`.no-mistakes.json`, `.no-mistakes.jsonc`, and legacy
`.playwright-ast-coverage.*` files. When both are present, Playwright settings
from `.no-mistakes.*` are used first; if `.no-mistakes.*` has no Playwright
settings, the legacy file is used.

### Fetches

Maps Next.js App Router route files to static `fetch()` calls.

```sh
no-mistakes fetches [--root <ROOT>] [--config <CONFIG>] [--format <FORMAT>] [--json] [TARGETS]...
```

Targets may be routes such as `/users`, route files, or files imported by route
or layout files. Formats are `json`, `yml`, `paths`, `md`, and `human`; `md` and
`human` render the Markdown report.

```sh
no-mistakes fetches --root web --format json
no-mistakes fetches --root web /users app/shared/api.ts
```
