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
`page.waitForTimeout()`. `terminal` selectors match the last member of a
resolved target and, for unresolved member calls, the last segment of that
source spelling. Identifier receivers (`page.waitForTimeout()`), renamed
receivers (`browser.waitForTimeout()`), `this.waitForTimeout()`, static
computed members (`page["waitForTimeout"]()`), and chained receivers recorded
as `<unknown>.waitForTimeout()` all match `terminal: waitForTimeout`. Dynamic
computed access such as `page[method]()` and other unresolved, shadowed, or
dynamic calls report `unknown call` by default. Set `unknownCalls: ignore` to
omit unmatched unknown calls; definite selector matches still report.

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
list still errors for names present in neither source.

The executable examples below use a `glob` root so they do not need a runner
catalog. Catalog roots belong in a repository that already configures
`tests.vitest` or `tests.playwright`.

```yaml
rules:
  - rule: forbidden-calls
    scope: repository
    options:
      roots:
        - glob: src/**/*.ts
      traversal: file
      targets:
        - global: setTimeout
```

A `glob` root selects the same prepared file universe without a runner
catalog, for example `glob: e2e/**/*.spec.ts` or a list of patterns. Use
`playwright: true` when the repository already has Playwright `testDir` /
`testMatch` selection paths. The catalog form is configuration, not a snippet
fixture:

```yaml
roots:
  - vitest: true
  - playwright: true
```

`options.excludeRoots` removes matching source files from the initial root
collection before call-graph traversal. Patterns are relative to the repository root and accept
one glob or a list, and match the whole repository-relative path (`*` also
crosses `/`), so excluding a directory needs `dir/**` and
`src/mocks/mock-only` does not exclude `src/mocks/mock-only.mock.test.mts`. It
applies to all root selector kinds. If it removes every configured root, the
rule fails with a configuration error; this is intentional, so an exclusion
that empties the root set is never a silent no-op. Excluded root files
can still produce findings when another retained root reaches them. Rule-level
`include` and `exclude` continue to filter findings after traversal; they do
not remove traversal roots.

`traversal` is `direct`, `file`, or `transitive`; `maxDepth`
caps the selected traversal. `direct` always stops after one call hop. Call
cycles are finite and each distinct reachable call occurrence is reported once.

Transitive source selection shares one breadth-first traversal across expanded
roots, without constructing per-root call traces. At `maxDepth: N`, only call
sites whose owners are fewer than N call hops from a root are selected.

`traversal` defaults to `direct`, `unknownCalls` defaults to `finding`, and
`invocations` defaults to `call`. An omitted `maxDepth` is unbounded for
`file` and `transitive` traversal. `targets` must be non-empty unless
`unknownCalls: finding` is selected (including the default).

Every selector is a one-key mapping. These are the accepted shapes:

```yaml
roots:
  - file: src/entry.mts
  - module: src/startup.mts
  - function: { file: src/jobs.mts, symbol: runJob }
  - glob: e2e/**/*.spec.ts
  - vitest: [unit, integration]
  - playwright: true
excludeRoots: ["**/*.mock.test.*"]
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
resolved. Import specifiers resolve the way the import graph resolves them,
including workspace package names such as `@scope/pkg`, their `exports`
subpaths, and package `imports` (`#name`), so a call through a workspace package
reaches the package source. A specifier that resolves nowhere, such as an
external package outside the repository, stays a `moduleExport` target. Use `function` when
a repository target must remain stable through a barrel, a workspace package
name, or an import alias. Use `exact` for a full source spelling that the graph
records as unknown. Use `terminal` for the last member name, including
unresolved receivers such as typed callback parameters, renamed bindings,
`this`, static computed members, and chained receivers whose source spelling
ends in that member. Dynamic computed access (`page[method]()`) is not guessed.

A callable declared in a TypeScript `namespace` body has the scope of its own
name, as it does everywhere the graph names a scope, so `symbol: Twin` selects
`new A.Twin()` and `new B.Twin()` alike when two namespaces of one file both
declare `Twin`. The `Call` edge keeps the exact declaration, and
`unconstructed-error-class` reads that edge, but a `function` selector carries a
file and a symbol and no namespace path.

A class's `extends` clause is not a call. `class Child extends Base {}` never
matches a target naming `Base`, whether `function` or `construct`; only `new Base()`
does. Other rules that read the class hierarchy, such as
[`unconstructed-error-class`](unconstructed-error-class.md), share this call
graph without changing what `forbidden-calls` reports.

## Valid example

A selected file that does not invoke a forbidden timer API passes. Inject a
clock or import an approved wrapper instead of calling `setTimeout` directly.

```ts
export function wait(clock: { sleep(ms: number): void }) {
  clock.sleep(1);
}
```

The consumer acceptance matrix in
`test-cases/rules/forbidden-calls/consumer-matrix/` is the packaged-CLI
contract for repeatable timer, mock-migration, glob, and discovery behavior.
Release validation runs that fixture through `no-mistakes check --format json`.
Each material source case is asserted by application identity, target, import
spelling, and line; YAML application indexes must match declaration order.
`test-cases/rules/forbidden-calls/workspace-package/` covers `function` targets
reached through a workspace package name and one of its `exports` subpaths.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` is accepted: `check` emits an analysis warning (requested effort low; effective effort max because this rule has no low implementation) and runs the rule at `max`.

### Unanalyzable inputs

Following recall over precision, the intent is that inputs this rule cannot prove are never treated as clean ([how analysis works](../../README.md#how-analysis-works)), but unresolved dynamic calls are governed by this rule's own `unknownCalls` option rather than `options.unanalyzable`:

- `unknownCalls: finding` is the default and reports unresolved dynamic calls that match no `exact` or `terminal` selector as explicit policy findings.
- `unknownCalls: ignore` explicitly omits those unmatched calls.
- `unknownArguments` defaults to `finding`, so a call whose arguments cannot be evaluated is reported.

See the [migration note](../migrations/forbidden-calls-unknown-calls.md) for
the finding change in existing configurations.

Definite matches of a configured selector remain findings under either value. See below for the full description.

## Unknown calls and suppression

`unknownCalls: ignore` omits unresolved dynamic calls that do not match an
`exact` or `terminal` selector. The default, `finding`, reports remaining
unknown calls as explicit policy findings. Configuration errors, such as an
invalid root, selector, or a requested root that fails to parse, are never
suppressible.
Unrelated files that fail to parse are ignored unless they are selected as a
root. Source findings honor ordinary `no-mistakes-disable-file`,
`no-mistakes-disable-line`, and `no-mistakes-disable-next-line` directives.

```ts
// no-mistakes-disable-next-line forbidden-calls: test intentionally verifies timer integration
setTimeout(complete, 1);
```

Counterexample: configuring a root without a target while explicitly setting
`unknownCalls: ignore` is invalid; suppressions cannot hide that error.

## Counterexample

```ts
// src/timers.ts
setTimeout(() => {}, 0);
```

## Fix

Inject the behavior, use the repository's approved wrapper, or narrow the rule
application to the layer where the call is forbidden. For an `unknown call`,
rewrite dynamic invocation to a statically resolved call when possible; set
`unknownCalls: ignore` only if the application intentionally permits unmatched
dynamic calls. Do not add a textual allowlist for aliases or namespace imports:
configure the canonical target.

## Related rules

[`forbidden-dependencies`](forbidden-dependencies.md) protects module
reachability; `forbidden-calls` protects the binding-aware invocation boundary
inside those modules.

## Bounded argument predicates

A target can add `whenArguments.all` to require literal argument conditions after
its ordinary binding-aware selector matches. Positions are zero-based. Each
predicate selects the argument itself or one literal `property`, then uses exactly
one of `equals`, `arrayIncludes`, or `arrayExcludes` with a string, finite number,
boolean, or null. Conditions are conjunctive.

```yaml
options:
  unknownArguments: finding
  targets:
    - moduleExport: { module: 'node:child_process', export: execFile }
      whenArguments:
        all:
          - position: 1
            arrayIncludes: check
          - position: 1
            arrayExcludes: --root
          - position: 2
            property: cwd
            equals: /workspace
```

This reports matching calls that launch checks against the live workspace without
an explicit fixture root. Add `--root` to the literal argument array, use a fixture
working directory, or remove the forbidden invocation. Existing suppression
comments work unchanged. Imported aliases and static namespace members retain the
same canonical target matching as target-only applications.

Literal expressions and direct immutable scalar aliases are supported. Arrays
and options objects require closed ownership: mutation, export/re-export, escape,
captured carriers, uses inside loops, unsupported destructuring, dynamic expressions,
and spreads produce unknown facts.
`const` and `as const` do not freeze mutable contents. Missing arguments/properties
and known incompatible shapes do not match. Unknown values never count as proof
of array exclusion. `unknownArguments: finding` (the default) reports otherwise
matching uncertain calls; `ignore` explicitly opts out. This is independent of
`unknownCalls`, which still governs unresolved callees.

Limits are 16 predicates, 32 positions, 128-byte property names, 4096-byte strings,
and bounded depth and projection/reference work. Exhaustion produces unknown.
Argument facts are requested only when a target declares these predicates and
share the existing request's source inventory and parse. Target-only checks do
not collect argument summaries.

Argument evidence is collected only for files accepted by applications with
`whenArguments`, using the same project, include, and exclude filters as findings.
Other call graph consumers retain their requested facts.
