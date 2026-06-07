# `playwright` command reference

## When to use

Use `playwright check` before finishing any Next.js App Router or Playwright
work — it validates that configured routes and selectors are covered by tests.
It is run by `no-mistakes check` only when Playwright is configured in
`.no-mistakes.yml`; call it directly when you need the gate regardless of global
config.

Use `playwright related` to find Playwright tests that cover a changed page,
route, or selector-bearing component.

Use `playwright tests` to see what a specific test proves (routes hit, selectors
asserted, fetches made) before editing it or its coverage expectations.

## Shared options

All `playwright` subcommands accept:

- `--playwright-config <FILE>` — path to a Playwright config (repeatable for
  multiple configs).
- `--project <NAME>` — filter by top-level no-mistakes Playwright config name
  (not Playwright's inner `projects` array).
- `--root <PATH>` — project root.
- `--config <FILE>` — path to `.no-mistakes.yml`.
- `--json` — emit JSON output.
- `--assert-conditional-tests` — require coverage from active (non-conditional)
  tests only; conditional tests (`test.skip`/`test.fixme`) do not satisfy
  coverage when this flag is set.
- `--allow-skipped-tests` — let skipped tests count as coverage (default:
  skipped tests are ignored).
- `--assert-unique-test-ids` — fail on duplicate `data-testid` / `data-pw`
  selectors.
- `--assert-unique-html-ids` — fail on duplicate HTML `id` attributes.

## `playwright check`

Fail on uncovered routes or uncovered configured selectors. Duplicate selector
failures require `--assert-unique-test-ids` or `--assert-unique-html-ids` to be
set (they are not checked by default).

```sh
no-mistakes playwright check --json
no-mistakes playwright check --assert-unique-test-ids --json
```

Node API: `playwrightCheck(options)`.

## `playwright related`

Tests that cover a route or selector-bearing component.

```sh
no-mistakes playwright related web/app/users/page.tsx --json
no-mistakes playwright related src/components/Button.tsx --json
```

Node API: `playwrightRelated(options)`.

## `playwright tests`

Route, selector, and fetch assertions grouped by test.

```sh
no-mistakes playwright tests playwright/tests/users.spec.ts --json
```

Node API: `playwrightTests(options)`.

## `playwright edges`

Raw test-to-route and test-to-selector edges (useful for debugging missing
coverage or building external tooling). No positional file argument —
use `playwright tests <test-file>` to inspect a single test's assertions.

```sh
no-mistakes playwright edges --json
```

Node API: `playwrightEdges(options)`.

## Selector configuration

Playwright coverage is driven by `tests.playwright` in `.no-mistakes.yml`:

```yaml
tests:
  playwright:
    configs: playwright.config.mts
    frontendRoot: web/app    # required for route discovery in Next.js App Router
    selectors:
      testIds:
        - data-pw
        - data-testid
      htmlIds: false
    selectorRoots:
      - web/app
      - web/components
    selectorExclude:
      - '**/*.stories.tsx'
```

`frontendRoot` sets the root directory for App Router route discovery;
`selectorRoots` sets the directories scanned for test ID selectors.
Consult `docs/configuration/tests.md` for the full schema.
