# `playwright` command reference

## When to use

Use `playwright check` before finishing any Next.js App Router or Playwright
work — it validates that configured routes and selectors are covered by tests.
It is already run by `no-mistakes check`; call it directly for a faster, focused
result during iteration.

Use `playwright related` to find Playwright tests that cover a changed page,
route, or selector-bearing component.

Use `playwright tests` to see what a specific test proves (routes hit, selectors
asserted, fetches made) before editing it or its coverage expectations.

## Shared options

All `playwright` subcommands accept:

- `--playwright-config <FILE>` — path to a Playwright config (repeatable for
  multiple projects).
- `--project <NAME>` — filter to a specific Playwright project.
- `--root <PATH>` — project root.
- `--config <FILE>` — path to `.no-mistakes.yml`.
- `--json` / `--format json|human` — output format.
- `--assert-conditional-tests` — fail on `test.skip` / `test.fixme` without
  issue references.
- `--allow-skipped-tests` — don't fail on skipped tests.
- `--assert-unique-test-ids` — fail on duplicate `data-testid` / `data-pw`
  selectors.
- `--assert-unique-html-ids` — fail on duplicate HTML `id` attributes.

## `playwright check`

Fail on uncovered routes, uncovered configured selectors, or duplicates.

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
coverage or building external tooling).

```sh
no-mistakes playwright edges --json
no-mistakes playwright edges playwright/tests/users.spec.ts --json
```

Node API: `playwrightEdges(options)`.

## Selector configuration

Playwright coverage is driven by `tests.playwright` in `.no-mistakes.yml`:

```yaml
tests:
  playwright:
    configs: playwright.config.mts
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

Consult `docs/configuration/tests.md` for the full schema.
