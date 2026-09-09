# `forbidden-calls`

Prevents configured roots from invoking selected JavaScript or TypeScript
functions.

## Why and when

Use this rule for a call boundary that cannot be expressed as an import
boundary: tests that must not schedule real timers, for example, or a server
layer that must not call a client-only helper. It uses the prepared,
binding-aware call graph rather than text matching, so an imported alias and a
static namespace member resolve to their canonical target while a shadowed
global is not mistaken for the global API.

## What it catches

It reports selected call or constructor occurrences reachable from configured
roots. `exact` selectors match the source callee spelling even when the call
cannot be resolved canonically, so `exact: page.waitForTimeout` still reports
`page.waitForTimeout()`. Other unresolved, shadowed, computed, or dynamic calls
are not reported unless the application sets `unknownCalls: finding`.

## Options and roots

`forbidden-calls` is repeatable. Each application has its own `name`, file
selection, roots, targets, traversal, and message; applications run in YAML
order and may intentionally overlap.

`options.roots` accepts a `file` or `module` root, a repository `function`
root, a `glob` of prepared graph files, or a Vitest or Playwright catalog
root. Catalog roots select runner-config projects and explicit
`tests.vitest.projects` / `tests.playwright.projects` entries through those
projects' include paths (`testDir` and `testMatch` for Playwright).
`vitest: true` or `playwright: true` includes that merged set, and a named
list still errors for names present in neither source. Rule-level `include`
and `exclude` still filter findings after expansion; they cannot select the
root collection.

`traversal` is `direct`, `file`, or `transitive`; `maxDepth`
caps the selected traversal. `direct` always stops after one call hop. Call
cycles are finite and each distinct reachable call occurrence is reported once.

`traversal` defaults to `direct`, `unknownCalls` defaults to `ignore`, and
`invocations` defaults to `call`. An omitted `maxDepth` is unbounded for
`file` and `transitive` traversal. `targets` must be non-empty unless
`unknownCalls: finding` is selected.

Every selector is a one-key mapping. These are the accepted shapes:

```yaml
roots:
  - file: src/entry.mts
  - module: src/startup.mts
  - function: { file: src/jobs.mts, symbol: runJob }
  - glob: e2e/**/*.spec.ts
  - vitest: [unit, integration]
  - playwright: true
targets:
  - global: setTimeout
  - exact: page.waitForTimeout
  - terminal: waitForTimeout
  - moduleExport: { module: "node:timers/promises", export: setTimeout }
  - function: { file: src/clock.mts, symbol: sleep }
invocations: [call, construct]
```

## Targets

Targets may select a global name, an exact source spelling, a terminal member
name, an imported module export, or a canonical repository function. Static
named imports, aliases, re-exports, and static namespace members such as
`import * as timers from "node:timers/promises"; timers.setTimeout()` are
resolved. Use `function` when a repository target must remain stable through a
barrel or import alias. Use `exact` for a source spelling that the graph records
as unknown, such as an unresolved member call. Other computed or dynamic calls
are not guessed.

## Valid example

Compliant example: a test injects a clock or imports an approved timer wrapper
instead of invoking a forbidden timer API directly.

```yaml
rules:
  - name: vitest-no-real-timers
    rule: forbidden-calls
    scope: repository
    options:
      roots:
        - vitest: true
      traversal: file
      unknownCalls: ignore
      targets:
        - global: setTimeout
        - moduleExport:
            module: node:timers/promises
            export: setTimeout
  - name: playwright-no-set-timeout
    rule: forbidden-calls
    scope: repository
    options:
      roots:
        - playwright: true
      traversal: file
      unknownCalls: ignore
      targets:
        - global: setTimeout
        - exact: page.waitForTimeout
        - moduleExport:
            module: node:timers/promises
            export: setTimeout
```

A `glob` root selects the same prepared file universe without a runner
catalog, for example `glob: e2e/**/*.spec.ts` or a list of patterns. Use
`playwright: true` when the repository already has Playwright `testDir` /
`testMatch` selection paths.

## Unknown calls and suppression

`unknownCalls: ignore` omits unresolved dynamic calls that do not match an
`exact` selector. `unknownCalls: finding` reports remaining unknown calls as
explicit policy findings. Configuration errors, such as an invalid root,
selector, or a requested root that fails to parse, are never suppressible.
Unrelated files that fail to parse are ignored unless they are selected as a
root. Source findings honor ordinary `no-mistakes-disable-file`,
`no-mistakes-disable-line`, and `no-mistakes-disable-next-line` directives.

```ts
// no-mistakes-disable-next-line forbidden-calls: test intentionally verifies timer integration
setTimeout(complete, 1);
```

Counterexample: configuring a root without a target (and without
`unknownCalls: finding`) is invalid; suppressions cannot hide that error.

## Fix

Inject the behavior, use the repository's approved wrapper, or narrow the rule
application to the layer where the call is forbidden. Do not add a textual
allowlist for aliases or namespace imports: configure the canonical target.

## Related rules

[`forbidden-dependencies`](forbidden-dependencies.md) protects module
reachability; `forbidden-calls` protects the binding-aware invocation boundary
inside those modules.
